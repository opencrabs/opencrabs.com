#!/usr/bin/env bash
# upload-prerelease.sh — mirror pre-release builds to opencrabs.com/prerelease/
#
# Pipeline: gh release download nightly -> verify SHA256SUMS -> rsync the drop
# dir to neuraprodnew:/srv/opencrabs/prerelease/<vX.Y.Z-pre-<sha>>/ -> generate
# index.html -> remove ALL older drops -> curl-verify public URLs.
#
# The page uses the site's shared chrome (nav + footer, /chrome.css,
# /chrome.js) extracted live from the crabsland source, shows the build
# date+timestamp and an Architecture column, and serves LATEST BUILD ONLY:
# every upload removes older drop dirs, no duplication.
#
# LOCAL_SRC=<dir>   skip GitHub entirely: archives are staged from the given
#                   directory (CI-convention names), SHA256SUMS is generated
#                   locally and a partial platform set is allowed. Used to
#                   ship hand-built platforms (mac) before CI finishes.
# REINDEX=<drop>    regenerate index.html for an already-uploaded drop without
#                   touching its archives (format refreshes).
# DRY_RUN=1         everything except upload/chmod/prune/verify. Still
#                   downloads real assets and checks them.
#
# Requires: gh (authenticated), rsync, curl, python3, shasum or sha256sum.
# SSH alias: neuraprodnew.
set -euo pipefail

REPO="opencrabs/opencrabs"
HOST="neuraprodnew"
REMOTE_DIR="/srv/opencrabs/prerelease"
BASE_URL="https://opencrabs.com/prerelease"
DRY_RUN="${DRY_RUN:-0}"
LOCAL_SRC="${LOCAL_SRC:-}"
DROP_OVERRIDE="${DROP_OVERRIDE:-}"
REINDEX="${REINDEX:-}"
CHROME_HTML="${CHROME_HTML:-$HOME/srv/rs/crabsland/src/root-files/download/index.html}"

log() { printf '==> %s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

file_size() { stat -f %z "$1" 2>/dev/null || stat -c %s "$1"; }

# Real build time of the binary INSIDE an archive. The zip file's own mtime is
# the packaging time and lies about when the build happened (caught live
# 2026-11-04: page showed packaging time for both rows).
inner_built() {
  [ -n "$1" ] && [ -f "$1" ] || return 0
  case "$1" in
    *.zip)
      local line d t yyyy mm dd
      line="$(unzip -l "$1" 2>/dev/null | awk '$NF == "opencrabs" {print $2, $3; exit}')"
      [ -n "$line" ] || return 0
      d="${line% *}"
      t="${line#* }"
      # unzip prints MM-DD-YYYY; normalise to YYYY-MM-DD HH:MM
      yyyy="${d##*-}"
      mm="${d%%-*}"
      dd="${d#*-}"
      dd="${dd%%-*}"
      printf '%s-%s-%s %s\n' "$yyyy" "$mm" "$dd" "$t"
      ;;
  esac
}

# Human-readable arch label, same wording as the download page.
arch_label() {
  case "$1" in
    *macos-arm64.*)   echo "macOS · Apple Silicon" ;;
    *macos-amd64.*)   echo "macOS · Intel" ;;
    *linux-amd64.*)   echo "Linux · x86_64" ;;
    *linux-arm64.*)   echo "Linux · ARM64" ;;
    *windows-amd64.*) echo "Windows · x86_64" ;;
    *)                echo "see filename" ;;
  esac
}

# Pull the shared chrome nav/footer verbatim from the landing source, so the
# page stays in sync with the site every time this script runs.
extract_chrome() {
  awk -v m="$1" '
    index($0, "<!-- chrome:" m " -->") {on=1; next}
    index($0, "<!-- /chrome:" m " -->") {on=0}
    on' "$CHROME_HTML" 2>/dev/null
}

STAGE="$(mktemp -d /tmp/prerelease-upload.XXXXXX)"
trap 'rm -rf "$STAGE"' EXIT

if [ -n "$REINDEX" ]; then
  DROP="$REINDEX"
  log "REINDEX mode: regenerating index for existing drop $DROP"
  case "$DROP" in
    *-pre-[0-9a-f]*)
      SHA="${DROP##*-pre-}"
      COMMIT_LINE="commit <a href=\"https://github.com/$REPO/commit/$SHA\"><code>$SHA</code></a>"
      FOOT_NOTE="checksums are served alongside the archives"
      ;;
    *)
      SHA=""
      COMMIT_LINE="hand-built drop, per-file commits are in the archive names"
      FOOT_NOTE="checksums are generated at upload time"
      ;;
  esac
  ssh "$HOST" "cd '$REMOTE_DIR/$DROP' && stat -c '%n %s' *" > "$STAGE/server-sizes.txt"
  ssh "$HOST" "cd '$REMOTE_DIR/$DROP' && TZ=UTC stat -c '%n %y' *" | awk '{print $1, $2, substr($3,1,5)}' > "$STAGE/times.txt"
  # Inner binary build times from inside each zip (packaging mtime lies).
  # Quote-free awk regex so the nested ssh quoting stays sane.
  ssh "$HOST" "command -v unzip >/dev/null 2>&1 && cd '$REMOTE_DIR/$DROP' && for f in *.zip; do t=\$(unzip -l \"\$f\" 2>/dev/null | awk '\$NF ~ /opencrabs\$/ {print \$2, \$3; exit}'); [ -n \"\$t\" ] && printf '%s %s\n' \"\$f\" \"\$t\"; done" > "$STAGE/inner-times.txt" 2>/dev/null || : > "$STAGE/inner-times.txt"
  if [ -s "$STAGE/inner-times.txt" ]; then
    awk 'NF >= 3 { split($2, a, "-"); print $1, a[3] "-" a[1] "-" a[2], $3 }' "$STAGE/inner-times.txt" > "$STAGE/inner-times2.txt" && mv "$STAGE/inner-times2.txt" "$STAGE/inner-times.txt"
  fi
  scp -q "$HOST:$REMOTE_DIR/$DROP/SHA256SUMS" "$STAGE/drop-SHA256SUMS"
  MANIFEST="$STAGE/manifest.txt"
  : > "$MANIFEST"
  while read -r f sz; do
    [ "$f" = "SHA256SUMS" ] && continue
    h="$(awk -v f="$f" '$2==f {print $1}' "$STAGE/drop-SHA256SUMS")"
    printf '%s %s %s\n' "$f" "$sz" "$h" >> "$MANIFEST"
  done < "$STAGE/server-sizes.txt"
  sort -o "$MANIFEST" "$MANIFEST"
  NEWEST_SRV="$(ssh "$HOST" "cd '$REMOTE_DIR/$DROP' && ls -t | grep -Ev '^SHA256SUMS$' | head -1")"
  BUILT="$(awk -v f="$NEWEST_SRV" '$1==f {print $2" "$3}' "$STAGE/inner-times.txt" 2>/dev/null)"
  [ -n "$BUILT" ] || BUILT="$(ssh "$HOST" "TZ=UTC stat -c '%y' '$REMOTE_DIR/$DROP/$NEWEST_SRV'" | cut -c1-16)"
elif [ -n "$LOCAL_SRC" ]; then
  log "LOCAL_SRC mode: staging local archives from $LOCAL_SRC"
  [ -d "$LOCAL_SRC" ] || die "LOCAL_SRC dir not found: $LOCAL_SRC"
  mkdir -p "$STAGE/assets"
  find "$LOCAL_SRC" -maxdepth 1 -type f \( -name '*.zip' -o -name '*.tar.gz' \) -exec cp -p {} "$STAGE/assets/" \;
  ls "$STAGE/assets" | grep -q . || die "no archives in LOCAL_SRC"
  RELEASE_URL=""
  GH_LINK=""
  FOOT_NOTE="checksums are generated from the staged artifacts at upload time"
  log "Downloading assets from the nightly release: skipped (LOCAL_SRC)"
else
  log "Fetching nightly release metadata"
  gh release view nightly -R "$REPO" --json tagName,createdAt,url > "$STAGE/release.json"
  RELEASE_URL="$(python3 -c "import json; print(json.load(open('$STAGE/release.json'))['url'])")"
  GH_LINK=" &middot; <a href=\"$RELEASE_URL\">GitHub release</a>"
  FOOT_NOTE="checksums are produced by the <a href=\"https://github.com/$REPO/blob/main/.github/workflows/prerelease.yml\">pre-release CI workflow</a>"
  log "Downloading assets"
  gh release download nightly -R "$REPO" --dir "$STAGE/assets" --clobber
fi

if [ -z "$REINDEX" ]; then
  cd "$STAGE/assets"
  log "Assets:"
  ls -lh

  # Drop name comes from the shared asset prefix: opencrabs-vX.Y.Z-pre-<sha>-<suffix>
  FIRST="$(ls | grep -E '^opencrabs-v[0-9]+\.[0-9]+\.[0-9]+-pre-[0-9a-f]+-' | head -1 || true)"
  [ -n "$FIRST" ] || die "no opencrabs-v*-pre-* archives found"
  if [ -n "$DROP_OVERRIDE" ]; then
    DROP="$DROP_OVERRIDE"
    SHA=""
    COMMIT_LINE="hand-built drop, per-file commits are in the archive names"
    log "Drop: $DROP (override)"
  else
    DROP="$(printf '%s' "$FIRST" | sed -E 's/^opencrabs-(v[0-9]+\.[0-9]+\.[0-9]+-pre-[0-9a-f]+)-.*/\1/')"
    [ -n "$DROP" ] || die "could not derive drop name from asset '$FIRST'"
    SHA="$(printf '%s' "$DROP" | sed -E 's/.*-pre-//')"
    COMMIT_LINE="commit <a href=\"https://github.com/$REPO/commit/$SHA\"><code>$SHA</code></a>$GH_LINK"
    log "Drop: $DROP (commit $SHA)"
  fi

  # Integrity gate: all 5 platforms must be present before anything ships
  log "Checking platform coverage"
  MISSING=0
  for s in linux-amd64 linux-arm64 macos-amd64 macos-arm64 windows-amd64; do
    ls | grep -q -- "-$s\." || { echo "missing: $s archive" >&2; MISSING=1; }
  done
  if [ "$MISSING" -ne 0 ]; then
    if [ -n "$LOCAL_SRC" ]; then
      log "LOCAL_SRC partial drop: continuing without the missing platforms"
    else
      die "partial nightly release, refusing to upload"
    fi
  fi

  if [ -n "$LOCAL_SRC" ]; then
    log "Generating SHA256SUMS from staged archives"
    ( shopt -s nullglob && shasum -a 256 -- *.zip *.tar.gz > SHA256SUMS )
  else
    log "Verifying SHA256SUMS"
    if command -v sha256sum >/dev/null 2>&1; then
      sha256sum -c SHA256SUMS
    else
      shasum -a 256 -c SHA256SUMS
    fi
  fi

  mkdir -p "$STAGE/drop"
  shopt -s nullglob
  for a in *.tar.gz *.zip SHA256SUMS; do mv -- "$a" "$STAGE/drop/"; done
  shopt -u nullglob

  # Build date+timestamp from the newest archive, so users see how fresh the
  # build is at a glance.
  NEWEST="$(ls -t "$STAGE/drop" | grep -Ev '^SHA256SUMS$' | head -1)"
  BUILT="$(inner_built "$STAGE/drop/$NEWEST")"
  [ -n "$BUILT" ] || BUILT="$(TZ=UTC stat -f '%Sm' -t '%Y-%m-%d %H:%M' "$STAGE/drop/$NEWEST" 2>/dev/null || echo "")"
  [ -n "$BUILT" ] || BUILT="$(date -u +%F' %H:%M')"

  MANIFEST="$STAGE/manifest.txt"
  : > "$MANIFEST"
  for f in $(ls "$STAGE/drop" | grep -Ev '^SHA256SUMS$' | sort); do
    sz="$(file_size "$STAGE/drop/$f")"
    h="$(awk -v f="$f" '$2==f {print $1}' "$STAGE/drop/SHA256SUMS")"
    printf '%s %s %s\n' "$f" "$sz" "$h" >> "$MANIFEST"
  done
fi

# Publish time is "now"; build time comes from the newest archive's mtime and
# is preserved end to end (cp -p in LOCAL_SRC, rsync -a on the wire).
PUBLISHED="$(date -u +'%F %H:%M')"

NAV_HTML="$(extract_chrome nav)"
FOOT_HTML="$(extract_chrome footer)"
[ -n "$NAV_HTML" ]  || NAV_HTML='<nav><div class="container"><a href="/" class="nav-logo"><img class="crab-icon" src="/public/opencrabs-logo.png" alt="" /> OpenCrabs</a><ul class="nav-links"><li><a href="/prerelease/">Pre-releases</a></li><li><a href="https://github.com/opencrabs/opencrabs">GitHub</a></li></ul></div></nav>'
[ -n "$FOOT_HTML" ] || FOOT_HTML='<footer><div class="container"><p class="footer-tagline">Built with 🦀 by <a href="https://github.com/adolfousier">Adolfo Usier</a> &amp; the OpenCrabs community.</p></div></footer>'

# --- index.html -----------------------------------------------------------
ROWS_FILE="$STAGE/rows.html"
: > "$ROWS_FILE"
while read -r f sz h; do
  MB="$(awk -v b="$sz" 'BEGIN{printf "%.1f MB", b/1048576}')"
  HS="$(printf '%s' "${h:-?}" | cut -c1-16)"
  A="$(arch_label "$f")"
  # Each row shows the binary's own build time read from INSIDE the archive;
  # the archive's mtime is packaging time, never build time.
  B="$(inner_built "$STAGE/drop/$f")"
  [ -n "$B" ] || B="$(awk -v f="$f" '$1==f {print $2" "$3}' "$STAGE/inner-times.txt" 2>/dev/null)"
  [ -n "$B" ] || B="$(TZ=UTC stat -f '%Sm' -t '%Y-%m-%d %H:%M' "$STAGE/drop/$f" 2>/dev/null || true)"
  [ -n "$B" ] || B="$(awk -v f="$f" '$1==f {print $2" "$3}' "$STAGE/times.txt" 2>/dev/null)"
  [ -n "$B" ] || B="?"
  printf '%s\n' "<tr><td><a href=\"$DROP/$f\"><code>$f</code></a></td><td>$A</td><td>$MB</td><td>$B</td><td><code>$HS</code></td></tr>" >> "$ROWS_FILE"
done < "$MANIFEST"

cat > "$STAGE/index.html" <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>OpenCrabs Pre-release Builds</title>
<link rel="icon" href="/public/favicon.ico?v=3" sizes="48x48" />
<link rel="stylesheet" href="/chrome.css" />
<script src="/chrome.js" defer></script>
<style>
  /* Base layer copied from the site's real pages (download page): the fixed
     nav needs 112px of body padding, and .container must be a centred
     1100px column. Nothing here redefines the site chrome. */
  :root{color-scheme:dark}
  *{box-sizing:border-box;margin:0;padding:0}
  body{
    background:#0b0e14;color:#e6e8ee;
    font:16px/1.6 -apple-system,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
    min-height:100vh;display:flex;flex-direction:column;align-items:center;
    padding:112px 20px 48px;
  }
  .container{max-width:1100px;width:100%;margin:0 auto;padding:0 24px;position:relative;z-index:1}
  h1{font-size:28px;margin:8px 0 4px;letter-spacing:-.02em}
  h1 span{color:#ff7a1a}
  h2{font-size:16px;margin:0 0 4px;font-family:'JetBrains Mono','SF Mono',monospace}
  a{color:#ff9a3d;text-decoration:none}
  a:hover{text-decoration:underline}
  .warn{max-width:980px;width:100%;background:#121722;border:1px solid #f0883e;border-radius:10px;padding:14px 18px;color:#ffb26b;margin-bottom:28px}
  code{background:#12121c;border:1px solid #1e1e2e;border-radius:4px;padding:2px 5px;font-size:12px;color:#ffb26b;word-break:break-all}
  .meta{color:#9aa3b2;font-size:13px;margin-bottom:6px}
  .meta b{color:#e6e8ee}
  .policy{color:#6d7686;font-size:13px;margin-bottom:24px}
  .card{width:100%;max-width:980px;background:#121722;border:1px solid #1f2735;border-radius:10px;padding:8px 18px 14px;margin-bottom:24px}
  .tblwrap{overflow-x:auto}
  table{border-collapse:collapse;width:100%;font-size:13px}
  th,td{border-bottom:1px solid #1f2735;padding:10px 8px;text-align:left;vertical-align:top}
  th{color:#9aa3b2;text-transform:uppercase;letter-spacing:.06em;font-size:11px;font-weight:600;border-bottom:1px solid #2a3448}
  tr:last-child td{border-bottom:0}
  .verify{color:#6d7686;font-size:13px;max-width:980px}
</style>
</head>
<body>
$NAV_HTML
<main class="container">
<h1>&#129408; OpenCrabs <span>Pre-release</span> Builds</h1>
<p class="warn">Pre-release build cut straight from <code>main</code>. Untested. For testing only &mdash; prefer tagged releases.</p>
<h2>$DROP</h2>
<p class="meta">Newest archive built <b>$BUILT UTC</b> &middot; published <b>$PUBLISHED UTC</b> &middot; $COMMIT_LINE</p>
<p class="policy">This page always serves the <b>latest build only</b>. Older pre-release drops are removed automatically when a new one is uploaded. Each row shows the build time of that exact binary.</p>
<div class="card"><div class="tblwrap">
<table>
<tr><th>Download</th><th>Architecture</th><th>Size</th><th>Built (UTC)</th><th>SHA256 (first 16)</th></tr>
$(cat "$ROWS_FILE")
</table>
</div></div>
<p class="verify">Verify an install: download the archive for your platform plus <a href="$DROP/SHA256SUMS"><code>SHA256SUMS</code></a> into one directory, then run <code>shasum -a 256 -c SHA256SUMS</code> (or <code>sha256sum -c SHA256SUMS</code>).</p>
</main>
$FOOT_HTML
</body>
</html>
HTML

log "index.html generated ($(wc -c < "$STAGE/index.html") bytes)"

if [ "$DRY_RUN" = "1" ]; then
  log "DRY_RUN: skipping upload, chmod, prune and URL verification"
  log "DRY_RUN complete. $DROP validated locally."
  exit 0
fi

if [ -n "$REINDEX" ]; then
  scp -q "$STAGE/index.html" "$HOST:$REMOTE_DIR/index.html"
  curl -sf "$BASE_URL/" | grep -q "chrome.css" || die "live page missing chrome.css after REINDEX"
  log "Done. Live at $BASE_URL/ (drop: $DROP, archives untouched)"
  exit 0
fi

log "Rsyncing $DROP to $HOST"
rsync -az --delete "$STAGE/drop/" "$HOST:$REMOTE_DIR/$DROP/"
ssh "$HOST" "chmod -R a+rX '$REMOTE_DIR/$DROP'"
scp -q "$STAGE/index.html" "$HOST:$REMOTE_DIR/index.html"

log "Removing older drops (latest-only policy)"
ssh "$HOST" "cd '$REMOTE_DIR' && ls -1dt -- v*-pre-*/ 2>/dev/null | grep -v '^$DROP/' | while IFS= read -r d; do echo removing \$d; rm -rf -- \"\$d\"; done"

log "Verifying public URLs"
curl -sf "$BASE_URL/" | grep -q "chrome.css" || die "$BASE_URL/ did not serve the chrome page"
curl -sfI "$BASE_URL/$DROP/$FIRST" > /dev/null || die "archive URL did not return 200"

log "Done. Live at $BASE_URL/ (drop: $DROP, GitHub: $RELEASE_URL)"

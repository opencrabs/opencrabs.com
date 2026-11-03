#!/usr/bin/env bash
# upload-prerelease.sh — mirror pre-release builds to opencrabs.com/prerelease/
#
# Pipeline: gh release download nightly -> verify SHA256SUMS -> rsync the drop
# dir to neuraprodnew:/srv/opencrabs/prerelease/<vX.Y.Z-pre-<sha>>/ -> regenerate
# index.html -> prune older drops (keeps the 2 newest) -> curl-verify public URLs.
#
# LOCAL_SRC=<dir> skips GitHub entirely: archives are staged from the given
# directory (CI-convention names, see prerelease.yml), SHA256SUMS is generated
# locally and a partial platform set is allowed. Used to ship hand-built
# platforms (mac) before CI finishes the rest.
#
# DRY_RUN=1 does everything except upload, chmod, prune and verify: it still
# downloads the real assets and checks them, so it is a safe full test.
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

log() { printf '==> %s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

file_size() { stat -f %z "$1" 2>/dev/null || stat -c %s "$1"; }

STAGE="$(mktemp -d /tmp/prerelease-upload.XXXXXX)"
trap 'rm -rf "$STAGE"' EXIT

if [ -n "$LOCAL_SRC" ]; then
  log "LOCAL_SRC mode: staging local archives from $LOCAL_SRC"
  [ -d "$LOCAL_SRC" ] || die "LOCAL_SRC dir not found: $LOCAL_SRC"
  mkdir -p "$STAGE/assets"
  find "$LOCAL_SRC" -maxdepth 1 -type f \( -name '*.zip' -o -name '*.tar.gz' \) -exec cp {} "$STAGE/assets/" \;
  ls "$STAGE/assets" | grep -q . || die "no archives in LOCAL_SRC"
  CREATED="$(date -u +%F)"
  RELEASE_URL=""
  GH_LINK=""
  FOOT_NOTE="checksums are generated from the staged artifacts at upload time"
else
  log "Fetching nightly release metadata"
  gh release view nightly -R "$REPO" --json tagName,createdAt,url > "$STAGE/release.json"
  CREATED="$(python3 -c "import json; print(json.load(open('$STAGE/release.json'))['createdAt'][:10])")"
  RELEASE_URL="$(python3 -c "import json; print(json.load(open('$STAGE/release.json'))['url'])")"
  GH_LINK=" &middot; <a href=\"$RELEASE_URL\">GitHub release</a>"
  FOOT_NOTE="checksums are produced by the <a href=\"https://github.com/$REPO/blob/main/.github/workflows/prerelease.yml\">pre-release CI workflow</a>"
  log "Downloading assets"
  gh release download nightly -R "$REPO" --dir "$STAGE/assets" --clobber
fi
cd "$STAGE/assets"

log "Assets:"
ls -lh

# Drop name comes from the shared asset prefix: opencrabs-vX.Y.Z-pre-<sha>-<suffix>
FIRST="$(ls | grep -E '^opencrabs-v[0-9]+\.[0-9]+\.[0-9]+-pre-[0-9a-f]+-' | head -1 || true)"
[ -n "$FIRST" ] || die "no opencrabs-v*-pre-* archives found in the nightly release"
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
  log "Drop: $DROP (commit $SHA, built $CREATED)"
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
  ( cd "$STAGE/assets" && shopt -s nullglob && shasum -a 256 -- *.zip *.tar.gz > SHA256SUMS )
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

# --- index.html -----------------------------------------------------------
ROWS_FILE="$STAGE/rows.html"
: > "$ROWS_FILE"
for f in $(ls "$STAGE/drop" | grep -Ev '^SHA256SUMS$' | sort); do
  B="$(file_size "$STAGE/drop/$f")"
  MB="$(awk -v b="$B" 'BEGIN{printf "%.1f MB", b/1048576}')"
  H="$(awk -v f="$f" '$2==f {print $1}' "$STAGE/drop/SHA256SUMS" | head -1)"
  HS="$(printf '%s' "${H:-?}" | cut -c1-16)"
  printf '%s\n' "<tr><td><a href=\"$DROP/$f\"><code>$f</code></a></td><td>$MB</td><td><code>$HS</code></td></tr>" >> "$ROWS_FILE"
done

cat > "$STAGE/index.html" <<HTML
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>OpenCrabs Pre-release Builds</title>
<style>
  body{background:#0d1117;color:#e6edf3;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;margin:0 auto;padding:2rem;max-width:880px;line-height:1.5}
  a{color:#58a6ff;text-decoration:none} a:hover{text-decoration:underline}
  .warn{background:#21262d;border:1px solid #f0883e;border-radius:6px;padding:.75rem 1rem;color:#f0883e}
  code{background:#161b22;border-radius:4px;padding:.1rem .35rem;font-size:.9em}
  table{border-collapse:collapse;width:100%;margin:1rem 0}
  th,td{border:1px solid #30363d;padding:.5rem .75rem;text-align:left;font-size:.95em}
  th{background:#161b22}
  footer{margin-top:2rem;color:#8b949e;font-size:.9em}
</style>
</head>
<body>
<h1>&#129408; OpenCrabs Pre-release Builds</h1>
<p class="warn">Pre-release build cut straight from <code>main</code>. Untested. For testing only &mdash; prefer tagged releases.</p>
<h2>$DROP</h2>
<p>Built $CREATED &middot; $COMMIT_LINE</p>
<table>
<tr><th>Download</th><th>Size</th><th>SHA256 (first 16)</th></tr>
$(cat "$ROWS_FILE")
</table>
<p>Verify an install: download the archive for your platform plus <a href="$DROP/SHA256SUMS"><code>SHA256SUMS</code></a> into one directory, then run <code>shasum -a 256 -c SHA256SUMS</code> (or <code>sha256sum -c SHA256SUMS</code>).</p>
<footer>Served from the official opencrabs.com origin &middot; $FOOT_NOTE.</footer>
</body>
</html>
HTML

log "index.html generated ($(wc -c < "$STAGE/index.html") bytes)"

if [ "$DRY_RUN" = "1" ]; then
  log "DRY_RUN: skipping upload, chmod, prune and URL verification"
  log "DRY_RUN complete. $DROP validated locally."
  exit 0
fi

log "Rsyncing $DROP to $HOST"
rsync -az "$STAGE/drop/" "$HOST:$REMOTE_DIR/$DROP/"
ssh "$HOST" "chmod -R a+rX '$REMOTE_DIR/$DROP'"
scp -q "$STAGE/index.html" "$HOST:$REMOTE_DIR/index.html"

log "Pruning drops older than the newest 2"
ssh "$HOST" "cd '$REMOTE_DIR' && ls -1dt -- v*-pre-*/ 2>/dev/null | tail -n +3 | while IFS= read -r d; do echo pruning \$d; rm -rf -- \"\$d\"; done"

log "Verifying public URLs"
curl -sfI "$BASE_URL/" > /dev/null || die "$BASE_URL/ did not return 200"
curl -sfI "$BASE_URL/$DROP/$FIRST" > /dev/null || die "archive URL did not return 200"

log "Done. Live at $BASE_URL/ (drop: $DROP, GitHub: $RELEASE_URL)"

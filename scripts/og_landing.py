#!/usr/bin/env python3
"""Per-page Open Graph previews for the OpenCrabs landing page.

The landing used to ship one shared image for every page, and that image had
been stale since March. This renders a purpose-built 1200x630 card per page
instead of screenshotting the page, because a landing page is a designed hero
with nav chrome and async state, and a screenshot of it reads as a mistake.

    python3 scripts/og_landing.py            generate the cards and rewrite the tags
    python3 scripts/og_landing.py --check    verify tags, files and dimensions
    python3 scripts/og_landing.py --help

Generation needs headless Chrome and sips, so it only runs on this Mac.
--check needs neither, so it is safe anywhere.

Pages whose og:image is not the shared default are left alone. That keeps the
hand-made collab card untouched if a page is ever added to the table by mistake.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import subprocess
import sys
import urllib.request
from pathlib import Path

SITE = "https://opencrabs.com"
REPO = Path(__file__).resolve().parent.parent
PUBLIC = REPO / "src" / "public"
OG_DIR = PUBLIC / "og"
LOGO = PUBLIC / "opencrabs-logo.png"
CHROME = os.environ.get(
    "CHROME_BIN", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
)
SIPS = "/usr/bin/sips"
RELEASES = "https://api.github.com/repos/opencrabs/opencrabs/releases/latest"
SHARED_DEFAULT = f"{SITE}/public/og.png"
WIDTH, HEIGHT = 1200, 630
JPEG_QUALITY = "82"
TMP = Path("/tmp/og-landing-cards")
PROFILE = Path("/tmp/og-landing-chrome-profile")

# Everything each card is made of is recorded here, so --changed can skip a
# card that is already current and --check can prove it without a browser.
MANIFEST = OG_DIR / "manifest.json"
# Bump when card_html changes shape, so every existing card counts as stale.
TEMPLATE_VERSION = "2"

# Approved copy. Nothing here is generated, every string is a decision.
PAGES = [
    {
        "html": "index.html",
        "slug": "home",
        "url": f"{SITE}/",
        "title": "OpenCrabs",
        "subtitle": "The all-in-one AI agent living in your terminal",
        "badges": ["macOS", "Linux", "Windows", "Telegram", "Discord", "Slack", "WhatsApp"],
        "alt": "OpenCrabs: The all-in-one AI agent living in your terminal",
    },
    {
        "html": "src/root-files/download/index.html",
        "slug": "download",
        "url": f"{SITE}/download/",
        "title": "Download OpenCrabs",
        "subtitle": "Free and open source",
        "badges": ["macOS 11+", "Linux", "Windows"],
        "alt": "Download OpenCrabs: Free and open source",
    },
    {
        "html": "stats/index.html",
        "slug": "stats",
        "url": f"{SITE}/stats/",
        "title": "OpenCrabs Inference Stats",
        "subtitle": "Live public usage of the OpenCrabs inference provider",
        "badges": ["Requests", "Tokens", "Cache hit rate", "Latency"],
        "alt": "OpenCrabs Inference Stats: live public usage of the OpenCrabs inference provider",
    },
]

META_IMAGE = re.compile(r'<meta\s+property="og:image"\s+content="([^"]*)"')


def latest_tag() -> str:
    req = urllib.request.Request(
        RELEASES,
        headers={
            "User-Agent": "opencrabs-landing-og",
            "Accept": "application/vnd.github+json",
        },
    )
    with urllib.request.urlopen(req, timeout=20) as resp:
        return json.load(resp)["tag_name"]


def jpeg_size(path: Path) -> tuple[int, int] | None:
    """Read width and height from a JPEG's SOF marker, no external tools."""
    data = path.read_bytes()
    if data[:2] != b"\xff\xd8":
        return None
    i = 2
    while i < len(data) - 9:
        if data[i] != 0xFF:
            i += 1
            continue
        marker = data[i + 1]
        if marker in (0xC0, 0xC1, 0xC2, 0xC3):
            height = int.from_bytes(data[i + 5 : i + 7], "big")
            width = int.from_bytes(data[i + 7 : i + 9], "big")
            return width, height
        if marker in (0xD8, 0xD9) or 0xD0 <= marker <= 0xD7:
            i += 2
            continue
        length = int.from_bytes(data[i + 2 : i + 4], "big")
        i += 2 + length
    return None


def fingerprint(page: dict, tag: str) -> str:
    """A hash of everything the card is made of: copy, release, logo, template."""
    payload = {
        "template": TEMPLATE_VERSION,
        "tag": tag,
        "logo": hashlib.sha256(LOGO.read_bytes()).hexdigest(),
        "page": {
            key: page[key]
            for key in ("html", "slug", "url", "title", "subtitle", "badges", "alt")
        },
    }
    blob = json.dumps(payload, sort_keys=True, ensure_ascii=False).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()

def load_manifest() -> dict:
    try:
        return json.loads(MANIFEST.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}

def save_manifest(entries: dict, tag: str) -> None:
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    payload = {"tag": tag, "template": TEMPLATE_VERSION, "pages": entries}
    MANIFEST.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

def card_html(page: dict, tag: str) -> str:
    logo = base64.b64encode(LOGO.read_bytes()).decode("ascii")
    badges = "".join(f'<span class="b">{name}</span>' for name in page["badges"])
    where = page["url"].replace("https://", "").rstrip("/")
    # 84px fits 20 characters across the card; longer titles step down so the
    # heading never runs into the frame.
    title_class = ' class="long"' if len(page["title"]) > 20 else ""
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><style>
  html, body {{ margin: 0; padding: 0; width: {WIDTH}px; height: {HEIGHT}px; overflow: hidden; }}
  body {{
    background: #08080d; color: #e8e8ef; position: relative;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  }}
  .glow {{
    position: absolute; left: -8%; top: -46%; width: 116%; height: 150%;
    background: radial-gradient(ellipse at 26% 34%, rgba(232,64,64,.30), rgba(232,64,64,.07) 40%, transparent 66%);
  }}
  .frame {{
    position: absolute; inset: 24px; border-radius: 28px;
    border: 1px solid rgba(232,64,64,.42);
    background: linear-gradient(158deg, rgba(18,18,28,.94), rgba(8,8,13,.70));
  }}
  .inner {{
    position: absolute; inset: 24px; box-sizing: border-box;
    padding: 58px 64px 52px; display: flex; flex-direction: column; justify-content: space-between;
  }}
  .top {{ display: flex; align-items: flex-start; justify-content: space-between; }}
  .logo {{ width: 168px; height: auto; filter: drop-shadow(0 12px 28px rgba(232,64,64,.34)); }}
  .pill {{
    font-size: 23px; font-weight: 700; color: #ff6b6b; letter-spacing: .01em;
    border: 1px solid rgba(232,64,64,.52); background: rgba(232,64,64,.13);
    padding: 9px 22px; border-radius: 999px;
  }}
  h1 {{ font-size: 84px; line-height: 1.02; margin: 0; font-weight: 800; letter-spacing: -.025em; }}
  h1.long {{ font-size: 66px; }}
  .sub {{ margin: 20px 0 0; font-size: 32px; line-height: 1.28; color: #9a9ab0; font-weight: 500; }}
  .badges {{ display: flex; flex-wrap: wrap; gap: 12px; }}
  .b {{
    font-size: 22px; font-weight: 600; color: #c9c9d6; white-space: nowrap;
    border: 1px solid rgba(255,255,255,.15); background: rgba(255,255,255,.045);
    padding: 8px 20px; border-radius: 999px;
  }}
  .foot {{ display: flex; align-items: center; justify-content: space-between; font-size: 21px; color: #55556a; }}
  .foot .url {{ color: #9a9ab0; font-weight: 600; letter-spacing: .02em; }}
</style></head>
<body>
  <div class="glow"></div><div class="frame"></div>
  <div class="inner">
    <div class="top">
      <img class="logo" alt="" src="data:image/png;base64,{logo}">
      <span class="pill">{tag}</span>
    </div>
    <div class="mid">
      <h1{title_class}>{page["title"]}</h1>
      <p class="sub">{page["subtitle"]}</p>
    </div>
    <div class="badges">{badges}</div>
    <div class="foot"><span class="url">{where}</span></div>
  </div>
</body></html>
"""


def render(page: dict, html_path: Path, png_path: Path) -> None:
    subprocess.run(
        [
            CHROME,
            "--headless=new",
            "--disable-gpu",
            "--no-first-run",
            "--hide-scrollbars",
            "--force-device-scale-factor=1",
            "--virtual-time-budget=3000",
            f"--user-data-dir={PROFILE}",
            f"--window-size={WIDTH},{HEIGHT}",
            f"--screenshot={png_path}",
            html_path.as_uri(),
        ],
        check=True,
        capture_output=True,
        timeout=90,
    )


def to_jpeg(png_path: Path, jpg_path: Path) -> None:
    jpg_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            SIPS,
            "-s", "format", "jpeg",
            "-s", "formatOptions", JPEG_QUALITY,
            str(png_path),
            "--out", str(jpg_path),
        ],
        check=True,
        capture_output=True,
    )


def replace_meta(doc: str, attr: str, name: str, value: str) -> tuple[str, int]:
    pattern = re.compile(rf'(<meta\s+{attr}="{re.escape(name)}"\s+content=")[^"]*(")')
    return pattern.subn(lambda m: m.group(1) + value + m.group(2), doc, count=1)


def ensure_meta(doc: str, anchor: str, attr: str, name: str, value: str) -> str:
    """Replace the tag if present, otherwise add it on its own line after the anchor tag."""
    new, hits = replace_meta(doc, attr, name, value)
    if hits:
        return new
    pattern = re.compile(
        rf'^([ \t]*)<meta\s+property="{re.escape(anchor)}"[^\n]*\n', re.M
    )
    match = pattern.search(doc)
    if not match:
        return doc
    line = f'{match.group(1)}<meta {attr}="{name}" content="{value}" />\n'
    return doc[: match.end()] + line + doc[match.end() :]


def rewrite_tags(doc: str, page: dict, image_url: str) -> str:
    doc, _ = replace_meta(doc, "property", "og:image", image_url)
    doc, _ = replace_meta(doc, "property", "og:image:width", str(WIDTH))
    doc, _ = replace_meta(doc, "property", "og:image:height", str(HEIGHT))
    doc = ensure_meta(doc, "og:image:height", "property", "og:image:type", "image/jpeg")
    doc = ensure_meta(doc, "og:image:type", "property", "og:image:alt", page["alt"])
    doc, _ = replace_meta(doc, "name", "twitter:image", image_url)
    return doc


def generate(tag: str, only_changed: bool = False) -> int:
    if not Path(CHROME).exists():
        print(f"headless Chrome not found: {CHROME}", file=sys.stderr)
        return 1
    if not LOGO.is_file():
        print(f"logo not found: {LOGO}", file=sys.stderr)
        return 1
    TMP.mkdir(parents=True, exist_ok=True)
    PROFILE.mkdir(parents=True, exist_ok=True)

    previous = load_manifest().get("pages", {}) if only_changed else {}
    entries: dict = {}
    written = 0
    current = 0
    kept = 0
    for page in PAGES:
        path = REPO / page["html"]
        doc = path.read_text(encoding="utf-8")
        found = META_IMAGE.search(doc)
        if not found:
            print(f"{page['html']}: no og:image tag, skipped", file=sys.stderr)
            continue
        value = found.group(1)
        if value != SHARED_DEFAULT and not re.search(r"/og/[a-z-]+-v[0-9]", value):
            kept += 1  # hand-made card, leave this page alone
            continue

        name = f'{page["slug"]}-{tag}.jpg'
        jpg = OG_DIR / name
        image_url = f"{SITE}/public/og/{name}"
        mark = fingerprint(page, tag)

        if only_changed:
            before = previous.get(page["slug"], {})
            if (
                before.get("fingerprint") == mark
                and before.get("file") == name
                and jpg.is_file()
                and value == image_url
            ):
                entries[page["slug"]] = before
                current += 1
                continue

        html_path = TMP / f'{page["slug"]}.html'
        png_path = TMP / f'{page["slug"]}.png'
        html_path.write_text(card_html(page, tag), encoding="utf-8")
        render(page, html_path, png_path)
        to_jpeg(png_path, jpg)

        path.write_text(rewrite_tags(doc, page, image_url), encoding="utf-8")
        entries[page["slug"]] = {"file": name, "fingerprint": mark}
        written += 1

        for old in OG_DIR.glob(f'{page["slug"]}-v*.jpg'):
            if old != jpg:
                old.unlink()
                print(f"  removed stale {old.name}")
        print(f"  {page['html']} -> og/{name}")

    save_manifest(entries, tag)
    print(
        f"cards written: {written}, already current: {current}, "
        f"hand-made cards kept: {kept}"
    )
    return 0


def check(tag_override: str | None = None) -> int:
    failures: list[str] = []
    tag = tag_override
    if not tag:
        try:
            tag = latest_tag()
        except Exception as exc:  # noqa: BLE001 - offline check still has value
            tag = None
            print(f"note: could not reach the releases API ({exc}), version not checked")

    manifest = load_manifest()
    expected_tag = tag or manifest.get("tag") or ""

    for page in PAGES:
        path = REPO / page["html"]
        if not path.is_file():
            failures.append(f"{page['html']}: file missing")
            continue
        doc = path.read_text(encoding="utf-8")
        found = META_IMAGE.search(doc)
        if not found:
            failures.append(f"{page['html']}: no og:image tag")
            continue
        value = found.group(1)
        if value == SHARED_DEFAULT:
            failures.append(f"{page['html']}: still points at the shared og.png")
            continue
        prefix = f"{SITE}/public/"
        if not value.startswith(prefix):
            failures.append(f"{page['html']}: og:image is not under /public/: {value}")
            continue
        image = PUBLIC / value[len(prefix) :]
        if not image.is_file():
            failures.append(f"{page['html']}: og:image file missing: {image}")
            continue
        size = jpeg_size(image)
        if size != (WIDTH, HEIGHT):
            failures.append(f"{image.name}: is {size}, expected {(WIDTH, HEIGHT)}")
        if tag and f"-{tag}." not in image.name:
            failures.append(f"{image.name}: filename version does not match latest release {tag}")
        for attr, name, want in (
            ("property", "og:image:width", str(WIDTH)),
            ("property", "og:image:height", str(HEIGHT)),
        ):
            got = re.search(rf'<meta\s+{attr}="{re.escape(name)}"\s+content="([^"]*)"', doc)
            if not got or got.group(1) != want:
                failures.append(f"{page['html']}: {name} is {got.group(1) if got else 'missing'}, expected {want}")
        if f'<meta name="twitter:image" content="{value}"' not in doc:
            failures.append(f"{page['html']}: twitter:image does not match og:image")
        if '<meta property="og:image:type" content="image/jpeg"' not in doc:
            failures.append(f"{page['html']}: og:image:type is missing")
        alt = re.search(r'<meta\s+property="og:image:alt"\s+content="([^"]*)"', doc)
        if not alt:
            failures.append(f"{page['html']}: og:image:alt is missing")
        elif alt.group(1) != page["alt"]:
            failures.append(f"{page['html']}: og:image:alt does not match the card copy")

        entry = manifest.get("pages", {}).get(page["slug"])
        if not entry:
            failures.append(f"{page['html']}: no manifest entry, run the generator")
        else:
            if entry.get("file") != image.name:
                failures.append(
                    f"{image.name}: manifest records {entry.get('file')}, run the generator"
                )
            if entry.get("fingerprint") != fingerprint(page, expected_tag):
                failures.append(
                    f"{page['html']}: card is stale, the copy, logo, template or release changed"
                )

    for line in failures:
        print(f"! {line}", file=sys.stderr)
    print(f"checked {len(PAGES)} pages, {len(failures)} problem(s)" + (f", latest {tag}" if tag else ""))
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate and verify per-page Open Graph cards for the landing page."
    )
    parser.add_argument("--check", action="store_true", help="verify tags and files, render nothing")
    parser.add_argument("--changed", action="store_true", help="regenerate only the cards whose inputs changed")
    parser.add_argument("--tag", help="override the release tag instead of asking GitHub")
    args = parser.parse_args()

    if args.check:
        return check(args.tag)

    tag = args.tag
    if not tag:
        try:
            tag = latest_tag()
        except Exception as exc:  # noqa: BLE001 - report and stop, never guess
            print(f"could not read the latest release: {exc}", file=sys.stderr)
            print("pass --tag <vX.Y.Z> to generate without the API", file=sys.stderr)
            return 1
    return generate(tag, only_changed=args.changed)


if __name__ == "__main__":
    sys.exit(main())

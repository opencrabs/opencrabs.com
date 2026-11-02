#!/usr/bin/env python3
"""Build the OpenCrabs favicon set from the canonical crab artwork.

Both sites used to ship a bad icon: the landing pointed at a 400x400, 226 KB
PNG while declaring it as 32x32, so the browser downscaled a pastel render into
a 16px slot, and the docs squashed a 397x235 landscape asset into a 32x32
square. This composes one mark at 1024px and derives every size from it.

    python3 scripts/build_favicons.py            write the icon set
    python3 scripts/build_favicons.py --check    verify sizes, links and leftovers
    python3 scripts/build_favicons.py --help

The source is vendored at assets/brand/opencrabs-crab.png, a copy of
opencrabs/src/assets/opencrabs-logo-no-bg.png, so this repository builds on its
own without reaching into a sibling checkout.

Only Pillow is required. No network, no browser.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "brand" / "opencrabs-crab.png"
PUBLIC = ROOT / "src" / "public"
DOCS_THEME = ROOT / "docs" / "theme"

# The site's card colour (--bg-card in style.css). A solid tile keeps the crab
# readable at 16px on both light and dark tab strips; an orange crab floating on
# transparency turns into a smudge on a light strip.
TILE_COLOUR = (18, 18, 28, 255)
MASTER_SIZE = 1024
CORNER_RADIUS = 0.22  # fraction of the tile edge
CRAB_WIDTH = 0.80  # fraction of the tile edge the crab occupies

PNG_SIZES = {
    "favicon-16x16.png": 16,
    "favicon-32x32.png": 32,
    "favicon-48x48.png": 48,
    "apple-touch-icon.png": 180,
    "android-chrome-192x192.png": 192,
    "android-chrome-512x512.png": 512,
}

MANIFEST = {
    "name": "OpenCrabs",
    "short_name": "OpenCrabs",
    "description": "Open-source AI agent that runs in your terminal.",
    "start_url": "/",
    "display": "standalone",
    "background_color": "#08080d",
    "theme_color": "#12121c",
    "icons": [
        {
            "src": "/public/android-chrome-192x192.png",
            "sizes": "192x192",
            "type": "image/png",
        },
        {
            "src": "/public/android-chrome-512x512.png",
            "sizes": "512x512",
            "type": "image/png",
        },
    ],
}

HEAD_SNIPPET = (
    '<link rel="icon" href="/public/favicon.ico?v=3" sizes="48x48" />',
    '<link rel="icon" href="/public/favicon-32x32.png?v=3" type="image/png" sizes="32x32" />',
    '<link rel="icon" href="/public/favicon-16x16.png?v=3" type="image/png" sizes="16x16" />',
    '<link rel="apple-touch-icon" href="/public/apple-touch-icon.png?v=3" />',
    '<link rel="manifest" href="/public/site.webmanifest?v=3" />',
)

HTML_HEADS = (
    ROOT / "index.html",
    ROOT / "src" / "root-files" / "download" / "index.html",
    ROOT / "src" / "root-files" / "collab" / "xiaomi-mimo-orbit" / "index.html",
    ROOT / "src" / "root-files" / "404.html",
    ROOT / "stats" / "index.html",
)


def build_master() -> Image.Image:
    """Compose the 1024px master: crab trimmed to content, centred on a tile."""
    crab = Image.open(SOURCE).convert("RGBA")
    crab = crab.crop(crab.getbbox())

    target_width = round(MASTER_SIZE * CRAB_WIDTH)
    scale = target_width / crab.width
    crab = crab.resize(
        (target_width, max(1, round(crab.height * scale))), Image.LANCZOS
    )

    tile = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(tile)
    draw.rounded_rectangle(
        [0, 0, MASTER_SIZE - 1, MASTER_SIZE - 1],
        radius=round(MASTER_SIZE * CORNER_RADIUS),
        fill=TILE_COLOUR,
    )
    tile.alpha_composite(
        crab,
        ((MASTER_SIZE - crab.width) // 2, (MASTER_SIZE - crab.height) // 2),
    )
    return tile


def render(master: Image.Image, size: int) -> Image.Image:
    """Downsample the master. Small sizes get a light sharpen, or they go soft."""
    icon = master.resize((size, size), Image.LANCZOS)
    if size <= 48:
        icon = icon.filter(ImageFilter.UnsharpMask(radius=0.6, percent=70, threshold=2))
    return icon


def write_assets() -> None:
    if not SOURCE.is_file():
        sys.exit(f"missing source artwork: {SOURCE}")

    master = build_master()
    PUBLIC.mkdir(parents=True, exist_ok=True)

    frames = [render(master, size) for size in (16, 32, 48)]
    frames[-1].save(
        PUBLIC / "favicon.ico",
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48)],
        append_images=frames[:2],
    )
    # mdBook copies docs/src into every book, so this lands at the site root and
    # answers the /favicon.ico request that used to 404 on docs.opencrabs.com.
    (ROOT / "docs" / "src" / "favicon.ico").write_bytes(
        (PUBLIC / "favicon.ico").read_bytes()
    )

    for name, size in PNG_SIZES.items():
        render(master, size).save(PUBLIC / name, format="PNG", optimize=True)

    render(master, 256).save(DOCS_THEME / "favicon.png", format="PNG", optimize=True)

    (PUBLIC / "site.webmanifest").write_text(
        json.dumps(MANIFEST, indent=2) + "\n", encoding="utf-8"
    )

    print("wrote:")
    for path in sorted(PUBLIC.glob("*.png")) + sorted(PUBLIC.glob("*.ico")):
        print(f"  {path.relative_to(ROOT)}  {path.stat().st_size} bytes")
    print(f"  {DOCS_THEME.relative_to(ROOT)}/favicon.png")


def check() -> int:
    problems: list[str] = []

    ico = PUBLIC / "favicon.ico"
    if not ico.is_file():
        problems.append("src/public/favicon.ico is missing")
    else:
        sizes = sorted(Image.open(ico).ico.sizes())
        if sizes != [(16, 16), (32, 32), (48, 48)]:
            problems.append(f"favicon.ico has {sizes}, expected 16/32/48")

    for name, size in PNG_SIZES.items():
        path = PUBLIC / name
        if not path.is_file():
            problems.append(f"{name} is missing")
            continue
        actual = Image.open(path).size
        if actual != (size, size):
            problems.append(f"{name} is {actual[0]}x{actual[1]}, expected {size}x{size}")
        # Tab icons are fetched on every page load, so they stay tiny. The 512
        # manifest icon is only fetched on install, so it gets a looser budget.
        budget = 200_000 if size >= 512 else 40_000
        if path.stat().st_size > budget:
            problems.append(f"{name} is {path.stat().st_size} bytes, over {budget}")

    docs_icon = DOCS_THEME / "favicon.png"
    if not docs_icon.is_file():
        problems.append("docs/theme/favicon.png is missing")
    elif Image.open(docs_icon).size != (256, 256):
        problems.append(f"docs/theme/favicon.png is {Image.open(docs_icon).size}")

    if (DOCS_THEME / "favicon.svg").exists():
        problems.append("docs/theme/favicon.svg still exists, mdBook emits it first")

    if not (PUBLIC / "site.webmanifest").is_file():
        problems.append("src/public/site.webmanifest is missing")

    docs_root_ico = ROOT / "docs" / "src" / "favicon.ico"
    if not docs_root_ico.is_file():
        problems.append("docs/src/favicon.ico is missing, /favicon.ico will 404")
    elif sorted(Image.open(docs_root_ico).ico.sizes()) != [(16, 16), (32, 32), (48, 48)]:
        problems.append("docs/src/favicon.ico does not carry 16/32/48 frames")

    for page in HTML_HEADS:
        text = page.read_text(encoding="utf-8")
        for snippet in HEAD_SNIPPET:
            if snippet not in text:
                problems.append(f"{page.relative_to(ROOT)} is missing {snippet}")

    stale = [
        page.relative_to(ROOT)
        for page in sorted((ROOT / "docs" / "book").rglob("*.html"))
        if "favicon.svg" in page.read_text(encoding="utf-8", errors="ignore")
    ]
    if stale:
        problems.append(f"{len(stale)} built docs pages still link the emoji SVG favicon")

    if problems:
        print("FAIL")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print("OK: icon set, sizes and head links all match")
    return 0


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify the generated set and the head links instead of writing",
    )
    args = parser.parse_args()
    if args.check:
        sys.exit(check())
    write_assets()


if __name__ == "__main__":
    main()
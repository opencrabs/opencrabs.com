#!/usr/bin/env bash
# Build the OpenCrabs docs in English + all translated locales.
# Usage: ./build-all.sh   (from the docs/ directory)
#
# Locales live in po/<locale>.po (gettext). A locale dir without a .po file
# is skipped. Missing translations fall back to English per-paragraph, so a
# partially translated book never renders broken.
set -euo pipefail
cd "$(dirname "$0")"

LOCALES=(pt-PT es fr ru id)

# Typography gate. The catalogs key on the exact English msgid, so one em-dash
# added to a source paragraph silently costs all five locales that translation:
# the msgid no longer matches and the page falls back to English with no error
# anywhere. Check before building, and fail loudly instead.
echo "==> Checking for em-dashes"
python3 scripts/dedash.py --check

# mdbook wipes its build directory on every build, and book/og holds the
# per-page preview images that are committed to the repo. Keep them across the
# rebuild: they belong to a page, not to a build, and a metadata-only run has no
# way to recreate them without a headless browser.
OG_BACKUP="$(mktemp -d)"
if [ -d book/og ]; then
    cp -R book/og "$OG_BACKUP/og"
fi

echo "==> Building English book"
mdbook build -d book

for loc in "${LOCALES[@]}"; do
    if [ -f "po/${loc}.po" ]; then
        echo "==> Building ${loc} book"
        MDBOOK_BOOK__language="${loc}" mdbook build -d "book/${loc}"
    else
        echo "==> Skipping ${loc} (no po/${loc}.po)"
    fi
done

# Put the previews back before anything reads them.
if [ -d "$OG_BACKUP/og" ]; then
    mv "$OG_BACKUP/og" book/og
fi

echo "==> All books built:"
ls -d book book/* 2>/dev/null | grep -v '^book/$' || true

# Every book is rendered from one template, so every locale page claims the
# English URL as its canonical and none of them advertise the others. Rewrite
# the head of each page and rebuild the sitemap before anything reads them.
echo "==> Rewriting per-locale canonical, hreflang and sitemap"
python3 scripts/seo.py

echo "==> Generating per-page Open Graph previews"
if [ "${SKIP_OG:-0}" = "1" ]; then
    # No rendering here, but the per-page og:title, og:description and og:url
    # still have to be written, so reuse the previews already on disk.
    echo "==> Rendering skipped (SKIP_OG=1), rewriting metadata only"
    python3 scripts/og_previews.py --meta-only
else
    python3 scripts/og_previews.py
fi

echo "==> Verifying SEO metadata"
python3 scripts/seo.py --check

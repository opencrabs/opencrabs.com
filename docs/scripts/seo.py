#!/usr/bin/env python3
"""Per-locale canonical, hreflang alternates and sitemap for the docs book.

mdbook renders all six books (English at the root, then pt-PT, es, fr, ru, id)
from one HTML template, so every locale page inherits the English canonical and
no page carries an hreflang alternate. A search engine reads six pages that all
claim to be the English URL as six copies of one page, keeps one, and drops the
rest, which is why the translated books never show up in results even though
they are live and fully translated.

This pass runs after the books are built. It rewrites the head of every page so
each locale points at itself, advertises every sibling locale, and then writes a
single sitemap at the book root that lists each page next to its alternates. The
per-locale sitemap copies mdbook drops into each book are removed, because they
list English URLs from inside a locale directory.

The English page is the x-default: that is what a visitor with an unlisted
language should land on.

Usage:
    python3 scripts/seo.py            # rewrite book/ in place
    python3 scripts/seo.py --check    # verify only, no writes
"""

from __future__ import annotations

import argparse
import re
import sys
from datetime import date
from pathlib import Path

DOCS = Path(__file__).resolve().parent.parent
BOOK = DOCS / "book"
SRC = DOCS / "src"
SITEMAP_SRC = SRC / "sitemap.xml"
SITE = "https://docs.opencrabs.com"

# Directory name under book/ and the hreflang value. English lives at the root,
# so its directory is the empty string.
LOCALES: list[tuple[str, str]] = [
    ("", "en"),
    ("pt-PT", "pt-PT"),
    ("es", "es"),
    ("fr", "fr"),
    ("ru", "ru"),
    ("id", "id"),
]
LOCALE_DIRS = {d for d, _ in LOCALES if d}

# print.html is the whole book on one URL and 404.html is an error page. Both
# would compete with the pages they duplicate, so they ask not to be indexed and
# carry no canonical or alternates of their own. Neither is listed in the source
# sitemap, so nothing else has to filter them.
NOINDEX = {"print.html", "404.html"}
ROBOTS = '<meta name="robots" content="noindex, follow" />'
ROBOTS_RE = re.compile(r'[ \t]*<meta name="robots" content="[^"]*"\s*/?>\n?')

MARK_START = "<!-- seo:alternates -->"
MARK_END = "<!-- /seo:alternates -->"

BLOCK_RE = re.compile(re.escape(MARK_START) + r".*?" + re.escape(MARK_END), re.S)
CANONICAL_RE = re.compile(r'[ \t]*<link rel="canonical" href="[^"]*"\s*/?>\n?')
OG_URL_RE = re.compile(r'(<meta property="og:url" content=")[^"]*(")')
IN_LANGUAGE_RE = re.compile(r'("inLanguage"\s*:\s*")[^"]*(")')
HEAD_END = "</head>"

URL_RE = re.compile(
    r"<url>\s*<loc>([^<]+)</loc>\s*(?:<priority>([^<]*)</priority>)?\s*"
    r"(?:<changefreq>([^<]*)</changefreq>)?\s*</url>"
)


def page_key(rel: Path) -> Path:
    """The locale-independent identity of a page, e.g. brain/tools.html."""
    if rel.parts and rel.parts[0] in LOCALE_DIRS:
        return Path(*rel.parts[1:])
    return rel


def page_url(key: Path, locale: str) -> str:
    posix = key.as_posix()
    if posix == "index.html":
        tail = ""
    elif posix.endswith("/index.html"):
        tail = posix[: -len("index.html")]
    else:
        tail = posix
    prefix = f"{locale}/" if locale else ""
    return f"{SITE}/{prefix}{tail}"


def alternates(key: Path) -> list[tuple[str, str]]:
    """Every locale that serves this page, English first, x-default last."""
    pairs = [(lang, page_url(key, directory)) for directory, lang in LOCALES]
    pairs.append(("x-default", page_url(key, "")))
    return pairs


def head_block(key: Path, locale: str) -> str:
    url = page_url(key, locale)
    lines = [
        MARK_START,
        f'<meta property="og:url" content="{url}" />',
        f'<link rel="canonical" href="{url}" />',
    ]
    for lang, url in alternates(key):
        lines.append(f'<link rel="alternate" hreflang="{lang}" href="{url}" />')
    lines.append(MARK_END)
    return "\n".join(lines)


def locale_of(rel: Path) -> str:
    if rel.parts and rel.parts[0] in LOCALE_DIRS:
        return rel.parts[0]
    return ""


def rewrite(page: Path) -> str | None:
    """Return the new document, or None when nothing had to change."""
    rel = page.relative_to(BOOK)
    key = page_key(rel)
    locale = locale_of(rel)
    doc = page.read_text(encoding="utf-8")
    original = doc

    # Drop whatever a previous run left, plus the hardcoded canonical that the
    # per-page head template carries: this pass is the only author of both.
    doc = BLOCK_RE.sub("", doc)
    doc = CANONICAL_RE.sub("", doc)
    doc = OG_URL_RE.sub(lambda m: m.group(1) + page_url(key, locale) + m.group(2), doc)

    lang = dict((d, l) for d, l in LOCALES)[locale]
    if lang != "en":
        doc = IN_LANGUAGE_RE.sub(lambda m: m.group(1) + lang + m.group(2), doc)

    if HEAD_END not in doc:
        return None
    if rel.name in NOINDEX:
        doc = ROBOTS_RE.sub("", doc)
        doc = doc.replace(HEAD_END, f"{ROBOTS}\n{HEAD_END}", 1)
    else:
        doc = doc.replace(HEAD_END, f"{head_block(key, locale)}\n{HEAD_END}", 1)

    return doc if doc != original else None


def source_mtime(key: Path) -> str:
    """The date the source page last changed, which is what lastmod should say."""
    name = key.as_posix()
    candidates = []
    if name == "index.html":
        candidates.append(SRC / "introduction.md")
    elif name.endswith("/index.html"):
        candidates.append(SRC / name[: -len("index.html")] / "README.md")
        candidates.append(SRC / (name[: -len("index.html")] + ".md"))
    else:
        candidates.append(SRC / name.replace(".html", ".md"))
    for path in candidates:
        if path.is_file():
            return date.fromtimestamp(path.stat().st_mtime).isoformat()
    return date.today().isoformat()


def sitemap_entries() -> list[tuple[Path, str, str]]:
    """(page key, priority, changefreq) for every page the source sitemap lists."""
    if not SITEMAP_SRC.is_file():
        print(f"missing {SITEMAP_SRC}", file=sys.stderr)
        return []
    entries = []
    for loc, priority, changefreq in URL_RE.findall(SITEMAP_SRC.read_text(encoding="utf-8")):
        path = loc[len(SITE) :].lstrip("/")
        if not path or path.endswith("/"):
            path += "index.html"
        key = Path(path)
        entries.append((key, priority or "0.5", changefreq or ""))
    return entries


def write_sitemap() -> int:
    entries = sitemap_entries()
    if not entries:
        return 1
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"',
        '        xmlns:xhtml="http://www.w3.org/1999/xhtml">',
    ]
    for key, priority, changefreq in entries:
        out.append("  <url>")
        out.append(f"    <loc>{page_url(key, '')}</loc>")
        out.append(f"    <lastmod>{source_mtime(key)}</lastmod>")
        out.append(f"    <priority>{priority}</priority>")
        if changefreq:
            out.append(f"    <changefreq>{changefreq}</changefreq>")
        for lang, url in alternates(key):
            out.append(f'    <xhtml:link rel="alternate" hreflang="{lang}" href="{url}" />')
        out.append("  </url>")
    out.append("</urlset>")
    (BOOK / "sitemap.xml").write_text("\n".join(out) + "\n", encoding="utf-8")

    # mdbook copies src/sitemap.xml into every book, so each locale ends up with
    # a sitemap that advertises English URLs from inside its own directory.
    removed = 0
    for directory in LOCALE_DIRS:
        stale = BOOK / directory / "sitemap.xml"
        if stale.is_file():
            stale.unlink()
            removed += 1
    print(f"sitemap: {len(entries)} pages x {len(LOCALES)} locales, {removed} stale copies removed")
    return 0


def pages() -> list[Path]:
    return sorted(p for p in BOOK.rglob("*.html") if (BOOK / "og") not in p.parents)


def check() -> int:
    problems = []
    for page in pages():
        rel = page.relative_to(BOOK)
        key = page_key(rel)
        locale = locale_of(rel)
        doc = page.read_text(encoding="utf-8")
        expected = page_url(key, locale)
        if rel.name in NOINDEX:
            if '<link rel="canonical"' in doc:
                problems.append(f"{rel}: noindex page carries a canonical")
            if not re.search(r'<meta name="robots" content="[^"]*noindex', doc):
                problems.append(f"{rel}: noindex page has no robots meta")
            continue
        canonicals = re.findall(r'<link rel="canonical" href="([^"]*)"', doc)
        if len(canonicals) != 1:
            problems.append(f"{rel}: {len(canonicals)} canonical tags, want 1")
        elif canonicals[0] != expected:
            problems.append(f"{rel}: canonical is {canonicals[0]}, want {expected}")
        og_urls = re.findall(r'<meta property="og:url" content="([^"]*)"', doc)
        if len(og_urls) != 1:
            problems.append(f"{rel}: {len(og_urls)} og:url tags, want 1")
        elif og_urls[0] != expected:
            problems.append(f"{rel}: og:url is {og_urls[0]}, want {expected}")
        found = set(re.findall(r'<link rel="alternate" hreflang="([^"]*)"', doc))
        want = {lang for lang, _ in alternates(key)}
        if found != want:
            problems.append(f"{rel}: hreflang set {sorted(found)} != {sorted(want)}")

    sitemap = BOOK / "sitemap.xml"
    if not sitemap.is_file():
        problems.append("book/sitemap.xml missing")
    else:
        text = sitemap.read_text(encoding="utf-8")
        for directory in LOCALE_DIRS:
            if f"{SITE}/{directory}/" not in text:
                problems.append(f"sitemap: no {directory} URLs")
        for directory in LOCALE_DIRS:
            if (BOOK / directory / "sitemap.xml").is_file():
                problems.append(f"book/{directory}/sitemap.xml still present")

    for problem in problems[:40]:
        print(problem, file=sys.stderr)
    if problems:
        print(f"{len(problems)} SEO problems", file=sys.stderr)
        return 1
    print(f"seo: {len(pages())} pages clean, sitemap covers {len(LOCALES)} locales")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Rewrite canonical and hreflang per locale, then rebuild the sitemap."
    )
    parser.add_argument("--check", action="store_true", help="verify without writing")
    args = parser.parse_args()
    if not BOOK.is_dir():
        print(f"book directory not found: {BOOK}", file=sys.stderr)
        return 1
    if args.check:
        return check()

    changed = 0
    for page in pages():
        new = rewrite(page)
        if new is not None:
            page.write_text(new, encoding="utf-8")
            changed += 1
    print(f"seo: rewrote {changed} of {len(pages())} pages")
    return write_sitemap()


if __name__ == "__main__":
    sys.exit(main())

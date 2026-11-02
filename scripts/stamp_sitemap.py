#!/usr/bin/env python3
"""Stamp <lastmod> into both sitemaps from git commit dates.

A hand-maintained lastmod goes stale the moment somebody forgets to bump it, and
a file mtime is useless here: on a fresh clone every file carries the checkout
time, so a CI build would stamp every URL with the deploy date and the value
would say nothing at all. The git commit date is identical in every clone and
moves exactly when a source moves, so that is what gets written.

The mapping is explicit because a URL does not know what produced it. A docs URL
comes from one markdown file. A landing URL comes from the Rust template, the
copy catalogs and the stylesheet, so its date is the newest of those. An
unmapped URL is a bug in this script, never a page without a source, so it fails
loudly instead of quietly writing today.

Usage:
    python3 scripts/stamp_sitemap.py          # rewrite both sitemaps
    python3 scripts/stamp_sitemap.py --check  # fail if either is out of date
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS_SITE = "https://docs.opencrabs.com"
LANDING_SITE = "https://opencrabs.com"

TARGETS = (ROOT / "docs" / "src" / "sitemap.xml", ROOT / "src" / "root-files" / "sitemap.xml")

# The landing pages are rendered from Rust, so the newest of the template, the
# copy catalogs and the stylesheet is the date their content last moved. The
# stats dashboard is its own crate with its own Trunk config: pointing it at the
# landing template made it move every time the landing did, which is a date that
# describes somebody else's page.
LANDING_SOURCES = {
    "/": ["src/main.rs", "index.html", "style.css", "locales"],
    "/download/": ["src/root-files/download/index.html"],
    "/stats/": ["stats"],
}

URL_RE = re.compile(r"<url><loc>(?P<loc>[^<]+)</loc>(?P<rest>.*?)</url>")
LASTMOD_RE = re.compile(r"<lastmod>[^<]*</lastmod>")

_dates: dict[str, str | None] = {}


def git_date(path: str) -> str | None:
    """Commit date of the last commit that touched one path, or None."""
    if path in _dates:
        return _dates[path]
    stamp = None
    if (ROOT / path).exists():
        done = subprocess.run(
            ["git", "log", "-1", "--format=%cs", "--", path],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        stamp = done.stdout.strip() or None
    _dates[path] = stamp
    return stamp


# Pages mdbook renders itself: they have no markdown of their own, so the file
# they are built from is the one named here.
GENERATED_DOCS = {"toc.html": "docs/src/SUMMARY.md"}


def docs_sources(path: str) -> list[str]:
    """The markdown that produces one docs URL."""
    if not path or path.endswith("/"):
        path += "index.html"
    if path in GENERATED_DOCS:
        return [GENERATED_DOCS[path]]
    if path == "index.html":
        return ["docs/src/introduction.md"]
    if path.endswith("/index.html"):
        base = path[: -len("index.html")]
        return [f"docs/src/{base}README.md", f"docs/src/{base.rstrip('/')}.md"]
    return [f"docs/src/{path.replace('.html', '.md')}"]


def sources_for(loc: str) -> list[str]:
    """Every source file that decides the content of one sitemap URL."""
    if loc.startswith(DOCS_SITE):
        candidates = docs_sources(loc[len(DOCS_SITE) :].lstrip("/"))
        for candidate in candidates:
            if (ROOT / candidate).exists():
                return [candidate]
        raise SystemExit(f"stamp_sitemap: no markdown for {loc} (tried {', '.join(candidates)})")
    if loc.startswith(LANDING_SITE):
        path = loc[len(LANDING_SITE) :].lstrip("/")
        key = f"/{path}" if path else "/"
        if key not in LANDING_SOURCES:
            raise SystemExit(f"stamp_sitemap: no source mapped for {loc}, add it to LANDING_SOURCES")
        return LANDING_SOURCES[key]
    raise SystemExit(f"stamp_sitemap: {loc} is on neither known host")


def lastmod_for(loc: str) -> str:
    """The newest commit date across every source of one URL."""
    sources = sources_for(loc)
    stamps = [stamp for stamp in (git_date(path) for path in sources) if stamp]
    if not stamps:
        raise SystemExit(f"stamp_sitemap: no git date for {loc} ({', '.join(sources)})")
    return max(stamps)


def render(target: Path) -> str:
    """The file as it should be: same lines, correct lastmod on each URL."""
    out: list[str] = []
    for line in target.read_text(encoding="utf-8").splitlines():
        match = URL_RE.search(line)
        if match is None:
            out.append(line)
            continue
        rest = LASTMOD_RE.sub("", match.group("rest"))
        stamp = lastmod_for(match.group("loc"))
        # Keep whatever indentation the file already uses: one URL per line at a
        # stable indent is what makes a sitemap diff readable.
        indent = line[: line.index("<url>")]
        out.append(f"{indent}<url><loc>{match.group('loc')}</loc><lastmod>{stamp}</lastmod>{rest}</url>")
    return "\n".join(out) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Stamp sitemap lastmod from git commit dates.")
    parser.add_argument("--check", action="store_true", help="verify without writing")
    args = parser.parse_args()

    stale = 0
    for target in TARGETS:
        if not target.is_file():
            print(f"stamp_sitemap: missing {target.relative_to(ROOT)}", file=sys.stderr)
            return 1
        wanted = render(target)
        current = target.read_text(encoding="utf-8")
        name = target.relative_to(ROOT)
        if wanted == current:
            print(f"stamp_sitemap: {name} up to date")
            continue
        if args.check:
            stale += 1
            print(
                f"stamp_sitemap: {name} is out of date, run: python3 scripts/stamp_sitemap.py",
                file=sys.stderr,
            )
            continue
        target.write_text(wanted, encoding="utf-8")
        print(f"stamp_sitemap: rewrote {name}")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
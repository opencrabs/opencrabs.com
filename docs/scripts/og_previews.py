#!/usr/bin/env python3
"""Per-page Open Graph previews for the OpenCrabs docs.

Run from the docs/ directory after `mdbook build` (docs/build-all.sh calls it):

    python3 scripts/og_previews.py

Phase 1 renders every content page to a 1200x630 JPEG with headless Chrome.
Phase 2 rewrites the og:*/twitter:* tags and the canonical link of each page so
a shared link shows that page's own preview, title and description.

Pages that already ship a hand-made card keep it. print.html and 404.html are
skipped. Set SKIP_OG=1 to skip the whole step.
"""

from __future__ import annotations

import argparse
import hashlib
import html.parser
import os
import re
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

SITE = "https://docs.opencrabs.com"
DEFAULT_IMAGE = f"{SITE}/img/og-docs.png"
CHROME = os.environ.get(
    "CHROME_BIN", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
)
SIPS = "/usr/bin/sips"
WIDTH, HEIGHT = 1200, 630
JPEG_QUALITY = "78"
SKIP_NAMES = {"print.html", "404.html"}

DOCS = Path(__file__).resolve().parent.parent
BOOK = DOCS / "book"
OG_DIR = BOOK / "og"

CANONICAL_RE = re.compile(r'(<link rel="canonical" href=")[^"]*(")')


class FirstParagraph(html.parser.HTMLParser):
    """Collect the text of the first paragraph inside <main>."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self._in_main = False
        self._in_p = False
        self._done = False
        self.parts: list[str] = []

    def handle_starttag(self, tag, attrs):
        if tag == "main":
            self._in_main = True
        elif tag == "p" and self._in_main and not self._done:
            self._in_p = True

    def handle_endtag(self, tag):
        if tag == "p" and self._in_p:
            self._in_p = False
            if self.parts:
                self._done = True
        elif tag == "main":
            self._in_main = False

    def handle_data(self, data):
        if self._in_p and not self._done:
            self.parts.append(data)

    @property
    def text(self) -> str:
        return " ".join(" ".join(self.parts).split())


def shorten(text: str, limit: int = 200) -> str:
    if len(text) <= limit:
        return text
    cut = text[:limit].rsplit(" ", 1)[0].rstrip(",.;: ")
    return f"{cut}..."


def free_port() -> int:
    """Bind an ephemeral port and hand it back.

    A fixed port is how one whole run rendered the same 404 card for all 300
    pages: a stale preview server from another checkout held the port, the
    readiness probe saw a listener and trusted it, and every page came back
    identical (#51).
    """
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def serves_book(port: int) -> bool:
    """True when the server on `port` is actually serving our book."""
    try:
        with urllib.request.urlopen(
            f"http://127.0.0.1:{port}/index.html", timeout=5
        ) as resp:
            return resp.status == 200
    except Exception:  # noqa: BLE001 - any failure means not our book
        return False


def page_url(rel: Path) -> str:
    posix = rel.as_posix()
    if posix == "index.html":
        return f"{SITE}/"
    if posix.endswith("/index.html"):
        return f"{SITE}/{posix[: -len('index.html')]}"
    return f"{SITE}/{posix}"


def preview_rel(rel: Path) -> str:
    return rel.with_suffix(".jpg").as_posix()


def replace_meta(doc: str, attr: str, name: str, value: str) -> tuple[str, int]:
    pattern = re.compile(
        rf'(<meta\s+{attr}="{re.escape(name)}"\s+content=")[^"]*(")'
    )
    return pattern.subn(lambda m: m.group(1) + value + m.group(2), doc, count=1)


def wait_for_server(port: int, timeout: float = 10.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        with socket.socket() as sock:
            sock.settimeout(0.25)
            if sock.connect_ex(("127.0.0.1", port)) == 0:
                return True
        time.sleep(0.1)
    return False


def render(page: Path, jpg: Path, port: int) -> None:
    """Render one page to `jpg`.

    Every render gets its own temp directory, screenshot path and Chrome
    profile. A shared screenshot path is how a whole run once wrote the same
    stale image for all 300 pages: Chrome exited 0 without writing, `check=True`
    passed, and the conversion read whatever the previous page had left behind.
    The freshness check below turns that silent failure into a loud one (#51).
    """
    with tempfile.TemporaryDirectory(prefix="og-render-") as tmp:
        png = Path(tmp) / "shot.png"
        profile = Path(tmp) / "profile"
        profile.mkdir()
        started = time.time()
        subprocess.run(
            [
                CHROME,
                "--headless=new",
                "--disable-gpu",
                "--no-first-run",
                "--hide-scrollbars",
                f"--user-data-dir={profile}",
                f"--window-size={WIDTH},{HEIGHT}",
                f"--screenshot={png}",
                f"http://127.0.0.1:{port}/{page.relative_to(BOOK).as_posix()}",
            ],
            check=True,
            capture_output=True,
            timeout=60,
        )
        if not png.is_file():
            raise RuntimeError("chrome exited without writing a screenshot")
        if png.stat().st_mtime < started - 1:
            raise RuntimeError("screenshot is older than this render")
        to_jpeg(png, jpg)

def to_jpeg(png: Path, jpg: Path) -> None:
    jpg.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            SIPS,
            "-s",
            "format",
            "jpeg",
            "-s",
            "formatOptions",
            JPEG_QUALITY,
            str(png),
            "--out",
            str(jpg),
        ],
        check=True,
        capture_output=True,
    )


def is_collapsing(counts: dict[str, int], rendered: int) -> bool:
    """True when one image covers more than a fifth of the rendered pages.

    A distinct-count assertion is not a valid invariant: a healthy run at HEAD
    held 293 distinct images for 300 pages, because the 404 page in each locale
    is legitimately identical. The failure worth failing on is one image taking
    over the whole run, which is what happened on 2026-10-20 (#51).
    """
    if rendered < 20:
        return False
    return max(counts.values(), default=0) * 5 > rendered


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

def book_pages() -> list[Path]:
    return sorted(
        page
        for page in BOOK.rglob("*.html")
        if OG_DIR not in page.parents and page.name not in SKIP_NAMES
    )

def check() -> int:
    """Verify every page's preview without rendering.

    Needs no Chrome and no sips, so it runs on the Linux deploy runner. It is
    the gate that stops a push shipping a book whose previews point at the
    site-wide default or at a file that no longer exists.
    """
    if not BOOK.is_dir():
        print(f"book directory not found: {BOOK}", file=sys.stderr)
        return 1

    pages = book_pages()
    if not pages:
        print("no pages found", file=sys.stderr)
        return 1

    failures: list[str] = []
    for page in pages:
        rel = page.relative_to(BOOK)
        doc = page.read_text(encoding="utf-8")
        found = re.search(r'<meta property="og:image" content="([^"]*)"', doc)
        if not found:
            failures.append(f"{rel}: no og:image tag")
            continue
        value = found.group(1)
        if value == DEFAULT_IMAGE:
            failures.append(f"{rel}: still points at the site-wide preview")
            continue
        prefix = f"{SITE}/og/"
        if not value.startswith(prefix):
            continue  # hand-made card, not ours to check
        image = OG_DIR / value[len(prefix) :]
        if not image.is_file():
            failures.append(f"{rel}: preview file missing: {image.relative_to(BOOK)}")
            continue
        size = jpeg_size(image)
        if size != (WIDTH, HEIGHT):
            failures.append(f"{rel}: preview is {size}, expected {(WIDTH, HEIGHT)}")
        if f'<meta name="twitter:image" content="{value}"' not in doc:
            failures.append(f"{rel}: twitter:image does not match og:image")
        href = re.search(r'<link rel="canonical" href="([^"]*)"', doc)
        if not href:
            failures.append(f"{rel}: canonical link missing")
        elif href.group(1) != page_url(rel):
            failures.append(f"{rel}: canonical is {href.group(1)}, expected {page_url(rel)}")
        og_url = re.search(r'<meta property="og:url" content="([^"]*)"', doc)
        if not og_url or og_url.group(1) != page_url(rel):
            failures.append(f"{rel}: og:url does not match the page url")

    files = list(OG_DIR.rglob("*.jpg"))
    counts: dict[str, int] = {}
    for path in files:
        digest = hashlib.md5(path.read_bytes()).hexdigest()
        counts[digest] = counts.get(digest, 0) + 1
    if is_collapsing(counts, len(files)):
        worst = max(counts.values(), default=0)
        failures.append(
            f"{worst} of {len(files)} previews share one image, the renders collapsed (#51)"
        )

    for line in failures[:20]:
        print(f"! {line}", file=sys.stderr)
    print(f"checked {len(pages)} pages, {len(failures)} problem(s)")
    return 1 if failures else 0

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate or verify per-page Open Graph previews for the docs."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify the previews without rendering, needs no Chrome",
    )
    parser.add_argument(
        "--meta-only",
        action="store_true",
        help="rewrite the og:* and twitter:* tags of pages that already have a "
        "preview on disk, without rendering and without Chrome",
    )
    args = parser.parse_args()
    if args.check:
        return check()
    if not BOOK.is_dir():
        print(f"book directory not found: {BOOK}", file=sys.stderr)
        return 1
    if not args.meta_only and not Path(CHROME).exists():
        print(f"headless Chrome not found: {CHROME}", file=sys.stderr)
        return 1

    pages = book_pages()
    if not pages:
        print("no pages found", file=sys.stderr)
        return 1

    # --meta-only reuses the previews already on disk, so there is nothing to
    # serve and nothing to screenshot. That keeps the head rewrite runnable on a
    # machine with no Chrome, which is where the deploy runner builds the book.
    port = 0
    server = None
    if not args.meta_only:
        port = free_port()
        server = subprocess.Popen(
            [
                sys.executable,
                "-m",
                "http.server",
                str(port),
                "--bind",
                "127.0.0.1",
                "--directory",
                str(BOOK),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

    rendered = 0
    kept = 0
    failed = 0
    failures: list[str] = []

    try:
        if server is not None and (not wait_for_server(port) or not serves_book(port)):
            print("local preview server did not start", file=sys.stderr)
            return 1

        for index, page in enumerate(pages, 1):
            rel = page.relative_to(BOOK)
            doc = page.read_text(encoding="utf-8")
            current = re.search(r'<meta property="og:image" content="([^"]*)"', doc)
            if not current:
                failed += 1
                failures.append(f"{rel}: no og:image tag")
                continue
            if current.group(1) != DEFAULT_IMAGE:
                kept += 1  # hand-made card, leave this page alone
                continue

            url = page_url(rel)
            preview = OG_DIR / rel.with_suffix(".jpg")
            if args.meta_only:
                if not preview.is_file():
                    failed += 1
                    failures.append(f"{rel}: no preview on disk, run without --meta-only")
                    continue
            else:
                try:
                    render(page, preview, port)
                except Exception as exc:  # noqa: BLE001 - report, keep going
                    failed += 1
                    failures.append(f"{rel}: render failed: {exc}")
                    continue

            title = re.search(r"<title>([^<]*)</title>", doc)
            title_text = title.group(1).strip() if title else "OpenCrabs Documentation"
            finder = FirstParagraph()
            finder.feed(doc)
            description = shorten(finder.text) or "OpenCrabs documentation."
            image_url = f"{SITE}/og/{preview_rel(rel)}"

            new = doc
            for attr, name, value in (
                ("property", "og:title", title_text),
                ("property", "og:description", description),
                ("property", "og:url", url),
                ("property", "og:image", image_url),
                ("property", "og:image:width", str(WIDTH)),
                ("property", "og:image:height", str(HEIGHT)),
                ("name", "twitter:title", title_text),
                ("name", "twitter:description", description),
                ("name", "twitter:image", image_url),
            ):
                new, hits = replace_meta(new, attr, name, value)
                if hits == 0:
                    failures.append(f"{rel}: {name} tag missing")
            new, hits = CANONICAL_RE.subn(
                lambda m: m.group(1) + url + m.group(2), new, count=1
            )
            if hits == 0:
                failures.append(f"{rel}: canonical link missing")
            page.write_text(new, encoding="utf-8")
            rendered += 1

            if index % 25 == 0:
                print(f"  {index}/{len(pages)} pages done", flush=True)
    finally:
        if server is not None:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()

    written = list(OG_DIR.rglob("*.jpg"))
    counts: dict[str, int] = {}
    for path in written:
        digest = hashlib.md5(path.read_bytes()).hexdigest()
        counts[digest] = counts.get(digest, 0) + 1
    distinct = len(counts)
    worst_digest, worst_count = max(counts.items(), key=lambda kv: kv[1], default=("", 0))
    print(
        f"previews written: {rendered}, hand-made cards kept: {kept}, failed: {failed}"
    )
    print(f"distinct previews on disk: {distinct} of {len(written)}")
    # A distinct-count assertion is not a valid invariant: a healthy run at HEAD
    # held 293 distinct images for 300 pages, because the 404 pages in each
    # locale are legitimately identical. The failure this guards against is one
    # image taking over the run, so fail when a single hash covers more than a
    # fifth of the pages (#51).
    if is_collapsing(counts, rendered):
        print(
            f"suspicious: {worst_count} of {rendered} rendered previews share one "
            f"image ({worst_digest}), the renders are collapsing into one card (#51)",
            file=sys.stderr,
        )
        return 1
    for line in failures[:20]:
        print(f"  ! {line}", file=sys.stderr)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())

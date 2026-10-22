#!/usr/bin/env python3
"""Remove em-dashes from the docs source and from the five translation catalogs.

Why a script and not a find and replace: most of the 1274 dashes in docs/src are
doing a specific job, and the punctuation that replaces them depends on that job.
A label followed by its description wants a colon. A sentence that continues
wants a comma. A blind swap to "-" turns the whole book into a wall of hyphens.

Why the .po files are touched too: mdbook-gettext looks a paragraph up by its
exact msgid. If the source loses the dash and the msgid keeps it, every touched
paragraph silently falls back to English in all five locales, which is a much
worse outcome than the dash. So the msgid gets the same transformation as the
source, and the msgstr gets the locale's own version of it.

Russian is the one exception in the msgstr path. A dash between subject and
predicate is grammar there, not decoration, so it becomes a spaced hyphen
instead of a comma, which would read as a mistake to a native speaker.

Usage:
  dedash.py            rewrite docs/src, docs/po and messages.pot in place
  dedash.py --check    report em-dashes in the guarded paths, exit 1 if any
  dedash.py --dry-run  print what would change, write nothing
"""
import pathlib
import re
import sys

DASH = "\u2014"

# The guard covers the files that are the source of truth for user-facing text.
# docs/book is generated, CHANGELOG.md and docs-audit-*.md are frozen records of
# past releases and audits, so rewriting those would misstate what shipped.
GUARDED = [
    "docs/src",
    "docs/theme/custom.js",
    "docs/theme/cards.css",
    "docs/theme/head.hbs",
    "docs/theme/favicon.svg",
    "docs/book.toml",
    # The catalogs are guarded so a dash cannot creep back into a msgid after
    # this sweep. They are listed one by one, not as `docs/po`, because that
    # directory also holds build_tr_v053.py, a frozen one-off migration script
    # whose dashes are code, not reader-facing prose.
    "docs/po/messages.pot",
    "docs/po/pt-PT.po",
    "docs/po/es.po",
    "docs/po/fr.po",
    "docs/po/ru.po",
    "docs/po/id.po",
    "docs/po/GLOSSARY.md",
    "README.md",
    "index.html",
    "stats/index.html",
    "src/root-files/404.html",
    "src/main.rs",
    "locales",
]

# Catalogs are rewritten block by block by `rewrite_po`, never as raw text:
# a plain string replace over a .po file would eat the escapes and the
# `msgid`/`msgstr` structure. The guard still counts them, so they belong in
# GUARDED and out of the raw rewrite loop.
PO_SUFFIXES = {".po", ".pot"}

PO_DIR = pathlib.Path("docs/po")
LOCALES = ["pt-PT", "es", "fr", "ru", "id"]

# A label is the short thing a dash introduces the definition of: a table cell
# key, a heading subject, a link text, a bold bullet lead-in. It is never a
# clause, so it is capped by length and rejected outright when it carries
# sentence punctuation.
LABEL_MAX = 45
LABEL_BAD = set(",.;:|")

# Markers that mean "this is a label position" but that xgettext strips before
# it writes the msgid. They are ignored when judging the label, never removed
# from the text that gets written back.
MARKER_RE = re.compile(r"^\s*(?:(?:[-*+>]|\d+\.|#{1,6})\s+)*")
PAREN_RE = re.compile(r"\([^()]*\)")
CODE_RE = re.compile(r"`[^`]*`")
VERSION_RE = re.compile(r"\bv?\d+(?:\.\d+)+\b")


def _label_text(raw):
    """The label with the noise removed: markers, asides, code spans, versions."""
    s = MARKER_RE.sub("", raw)
    s = PAREN_RE.sub("", s)
    s = CODE_RE.sub("", s)
    s = VERSION_RE.sub("", s)
    return s.strip()


def _is_label(raw):
    # A cell or line that opens with a code span is a path or a command being
    # introduced, so the dash after it is a definition even though the span
    # itself is full of dots.
    marked = MARKER_RE.sub("", raw)
    if marked.startswith("`") and "`" in marked[1:]:
        return True
    s = _label_text(raw)
    return 0 < len(s) <= LABEL_MAX and not (set(s) & LABEL_BAD)


def dedash_segment(seg, locale="en"):
    """Rewrite the dashes inside one run of text.

    This runs on a source line and on the catalog msgid built from that line,
    so it may only look at the text itself. Anything marker-dependent would
    give the source one answer and the msgid another, and mdbook-gettext does
    not warn when that happens: it renders English in that locale.

    Two dashes closing an aside become parentheses, because that is what the
    second dash means. A single dash after a short label becomes a colon. Any
    other dash is joining clauses, so it becomes a comma (a spaced hyphen in
    Russian, where the dash is a real punctuation mark between subject and
    predicate rather than decoration).
    """
    if DASH not in seg:
        return seg
    generic = " - " if locale == "ru" else ", "

    # A cell whose whole content is the dash is a placeholder for "no value".
    if seg.strip() == DASH:
        return seg.replace(DASH, "-")

    if seg.count(DASH) == 2:
        i = seg.index(DASH)
        j = seg.index(DASH, i + 1)
        middle = seg[i + 1: j].strip()
        # Only when the aside sits inside one sentence. A dash that ends a
        # sentence is a clause break, and wrapping across it would swallow the
        # full stop into the parentheses.
        if middle and ". " not in middle and len(middle) <= 160:
            return seg[:i].rstrip() + " (" + middle + ") " + seg[j + 1:].lstrip()

    i = seg.index(DASH)
    left, rest = seg[:i], seg[i + 1:]
    if _is_label(left):
        return left.rstrip() + ": " + rest.lstrip().replace(" " + DASH + " ", generic).replace(DASH, "-")
    return seg.replace(" " + DASH + " ", generic).replace(DASH, "-")


def dedash_line(ln, locale="en"):
    """Rewrite one line. A table row is rewritten cell by cell, because the
    catalog stores each cell as its own msgid."""
    if DASH not in ln:
        return ln
    if ln.lstrip().startswith("|"):
        return "|".join(dedash_segment(c, locale) for c in ln.split("|"))
    return dedash_segment(ln, locale)


def _is_table(ln):
    return ln.lstrip().startswith("|")


def _is_fence(ln):
    return ln.strip().startswith("```")


BLOCK_START_RE = re.compile(r"^\s*(?:[-*+]\s|\d+\.\s|#{1,6}\s|>\s)")


def _starts_block(ln):
    """A list item, heading or blockquote opens its own block, so a dash in it
    never pairs with a dash in the block above or below."""
    return bool(BLOCK_START_RE.match(ln))


def _sentence_break_between(text, first, second):
    """True when a full stop sits between two dashes, which means they close
    two different sentences and must not be wrapped into one parenthetical."""
    between = text[first:second]
    return any(mark in between for mark in (". ", "? ", "! "))


def _pair_run(run, locale="en"):
    """Pair the dashes of one paragraph, left to right, and return the lines.

    A paragraph routinely holds more than one aside, so pairing must not stop
    at the first two dashes. Walking the dashes in reading order and pairing
    each with the next one works for any count; a full stop between two dashes
    means they close different sentences (typically a label line followed by
    prose), so the first is left as a single dash and the second opens the
    next pair.
    """
    joined = "\n".join(run)
    starts = []
    off = 0
    for ln in run:
        starts.append(off)
        off += len(ln) + 1
    positions = [k for k, c in enumerate(joined) if c == DASH]
    pairs = []
    pending = None
    for p in positions:
        if pending is None:
            pending = p
        elif _sentence_break_between(joined, pending, p):
            pending = p
        else:
            pairs.append((pending, p))
            pending = None
    if not pairs:
        return run

    def locate(offset):
        for idx in range(len(run) - 1, -1, -1):
            if starts[idx] <= offset:
                return idx, offset - starts[idx]
        return 0, offset

    for first, second in pairs:
        line_a, col_a = locate(first)
        line_b, col_b = locate(second)
        if line_a == line_b:
            # Same line: dedash_segment already wraps a matching pair.
            continue
        run[line_a] = run[line_a][:col_a].rstrip() + " (" + run[line_a][col_a + 1:].lstrip(" ")
        run[line_b] = run[line_b][:col_b].rstrip() + ") " + run[line_b][col_b + 1:].lstrip(" ")
    return run

def dedash_text(text, locale="en"):
    """Rewrite a whole file body.

    Dashes are paired across a wrapped paragraph before anything else happens.
    A paragraph is written across several lines and both dashes of one aside
    can land on different lines, so handling each line alone turned a single
    aside into a comma followed by a colon: "in every locale, untranslated
    paragraphs fall back to English: so switching is ...". Pairing them first
    keeps the aside in parentheses where it belongs.
    """
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        if not lines[i].strip() or _is_table(lines[i]) or _is_fence(lines[i]):
            lines[i] = dedash_line(lines[i], locale)
            i += 1
            continue
        j = i + 1
        while (
            j < len(lines)
            and lines[j].strip()
            and not _is_table(lines[j])
            and not _is_fence(lines[j])
            and not _starts_block(lines[j])
        ):
            j += 1
        run = lines[i:j]
        if any(DASH in ln for ln in run):
            run = _pair_run(run, locale)
        lines[i:j] = [dedash_line(ln, locale) for ln in run]
        i = j
    return "\n".join(lines)


# --- .po plumbing -----------------------------------------------------------
# Kept in step with po_fill.py: msgids and msgstrs are stored as one or more
# quoted segments, and the only way to transform them safely is to join the
# segments, transform the whole string, then write it back out.

def unescape(s):
    return s.replace('\\"', '"').replace("\\n", "\n").replace("\\\\", "\\")


def escape(s):
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def read_string(block, keyword):
    m = re.search(
        r"^" + keyword + r' ((?:"(?:[^"\\]|\\.)*"\n?)+)', block, re.M
    )
    if not m:
        return None
    return "".join(unescape(p) for p in re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1)))


def render_string(keyword, value):
    val = escape(value)
    if len(val) <= 70 and "\\n" not in val:
        return '%s "%s"' % (keyword, val)
    chunks = val.split("\\n")
    lines = ['%s ""' % keyword]
    for i, c in enumerate(chunks):
        lines.append('"%s"' % (c + ("\\n" if i < len(chunks) - 1 else "")))
    return "\n".join(lines)


def rewrite_block(block, locale=None, msgid_only=False):
    """Transform the msgid, and the msgstr unless msgid_only is set."""
    mid = read_string(block, "msgid")
    if mid is None or mid == "":
        return block
    new_mid = dedash_text(mid, "en")
    if new_mid != mid:
        block = re.sub(
            r'^msgid (?:"(?:[^"\\]|\\.)*"\n?)+',
            lambda m: render_string("msgid", new_mid) + "\n",
            block,
            count=1,
            flags=re.M,
        )
    if msgid_only:
        return block
    mstr = read_string(block, "msgstr")
    if not mstr:
        return block
    new_str = dedash_text(mstr, locale)
    if new_str != mstr:
        block = re.sub(
            r'^msgstr (?:"(?:[^"\\]|\\.)*"\n?)+',
            lambda m: render_string("msgstr", new_str) + "\n",
            block,
            count=1,
            flags=re.M,
        )
    return block


def rewrite_po(path, locale, msgid_only=False, dry_run=False):
    raw = path.read_text(encoding="utf-8")
    if DASH not in raw:
        return 0
    blocks = raw.split("\n\n")
    changed = 0
    out = []
    for b in blocks:
        nb = rewrite_block(b, locale, msgid_only)
        if nb != b:
            changed += 1
        out.append(nb)
    if changed and not dry_run:
        path.write_text("\n\n".join(out), encoding="utf-8")
    return changed


# --- driver -----------------------------------------------------------------

def guarded_files(root):
    for entry in GUARDED:
        p = root / entry
        if p.is_dir():
            yield from sorted(p.rglob("*"))
        elif p.exists():
            yield p


# Suffixes that are not text. `.svg` is deliberately absent: an SVG is markup
# and its comments are prose, which is exactly where the favicon dash hid.
BINARY = {".png", ".jpg", ".jpeg", ".gif", ".ico", ".woff", ".woff2", ".webp"}


def live_text(p, text):
    """The part of a file a reader can actually see.

    A .po file keeps obsolete entries as `#~` lines. xgettext writes one when a
    msgid leaves the source, msgfmt and mdbook-gettext both ignore them, and
    `rewrite_po` never touches them because their lines do not start with
    `msgid`. Counting those dashes would fail the guard forever on text nobody
    can reach, so they are dropped before the count.
    """
    if p.suffix not in PO_SUFFIXES:
        return text
    return "\n".join(line for line in text.splitlines() if not line.startswith("#~"))


def check(root):
    hits = []
    for p in guarded_files(root):
        if not p.is_file() or p.suffix in BINARY:
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        n = live_text(p, text).count(DASH)
        if n:
            hits.append((p.relative_to(root), n))
    if hits:
        print("em-dashes found (U+2014):")
        for p, n in sorted(hits, key=lambda x: -x[1]):
            print("  %5d  %s" % (n, p))
        print("\ntotal: %d in %d files" % (sum(n for _, n in hits), len(hits)))
        return 1
    print("dedash: clean, no em-dashes in guarded paths")
    return 0




def readable(p):
    if p.suffix in BINARY or not p.is_file():
        return None
    try:
        return p.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return None


def rewrite_file(p, root, dry_run):
    """Rewrite one guarded file. Markdown and other display text get the prose
    rules. Rust source and SVG get a plain hyphen: a dash there sits in a
    comment or a format string as a separator, and prose punctuation inside
    markup is noise."""
    text = readable(p)
    if text is None or DASH not in text:
        return 0
    if p.suffix in {".rs", ".svg"}:
        new = text.replace(" " + DASH + " ", " - ").replace(DASH, "-")
    else:
        new = dedash_text(text, "en")
    n = text.count(DASH)
    if not dry_run:
        p.write_text(new, encoding="utf-8")
    print("  %-6s %s (%d)" % (p.suffix.lstrip("."), p.relative_to(root), n))
    return n


def repo_root():
    """Find the repository root from this script's own location.

    The guarded paths are written relative to the repo root, and the build
    script runs from docs/, so resolving from the cwd would silently check
    nothing at all. Walking up from the file that is running cannot drift.
    """
    here = pathlib.Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "docs/src").is_dir():
            return parent
    return pathlib.Path.cwd()

def main():
    root = repo_root()
    args = sys.argv[1:]
    if "--check" in args:
        return check(root)

    dry_run = "--dry-run" in args
    total = 0
    for p in guarded_files(root):
        # Catalogs are guarded so `--check` sees them, but a raw text rewrite
        # over a .po file would destroy its escapes and block structure. They
        # go through rewrite_po below, one msgid/msgstr at a time.
        if p.suffix in PO_SUFFIXES:
            continue
        total += rewrite_file(p, root, dry_run)

    for loc in LOCALES:
        po = root / "docs/po" / ("%s.po" % loc)
        if po.exists():
            n = rewrite_po(po, loc, dry_run=dry_run)
            print("  po     docs/po/%s (%d blocks)" % (po.name, n))

    pot = root / "docs/po/messages.pot"
    if pot.exists():
        n = rewrite_po(pot, "en", msgid_only=True, dry_run=dry_run)
        print("  pot    docs/po/messages.pot (%d blocks)" % n)

    print("dashes rewritten: %d" % total)
    return 0


if __name__ == "__main__":
    sys.exit(main())

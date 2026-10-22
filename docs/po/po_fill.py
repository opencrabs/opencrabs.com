#!/usr/bin/env python3
"""Fill msgstr entries in a .po file from a JSON {msgid: msgstr} map.

Unlike po_fill.py this REPLACES the whole msgstr (all continuation lines) and
clears the #, fuzzy flag. po_fill.py only matched `msgstr ""` on its own line,
so a multiline or fuzzy-merged entry got the new text prepended to the old one
and the translations accumulated across releases.
"""
import json, re, subprocess, sys


def unescape(s):
    return s.replace('\\"', '"').replace("\\n", "\n").replace("\\\\", "\\")


def escape(s):
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def block_msgid(block):
    ids = re.findall(r'^msgid ((?:"(?:[^"\\]|\\.)*"\n?)+)', block, re.M)
    if not ids:
        return None
    parts = re.findall(r'"((?:[^"\\]|\\.)*)"', ids[0])
    return "".join(unescape(p) for p in parts)


def render(val):
    val = escape(val)
    if len(val) <= 70 and "\\n" not in val:
        return 'msgstr "%s"' % val
    chunks = val.split("\\n")
    lines = ['msgstr ""']
    for i, c in enumerate(chunks):
        lines.append('"%s"' % (c + ("\\n" if i < len(chunks) - 1 else "")))
    return "\n".join(lines)


def fill_block(block, tr):
    mid = block_msgid(block)
    if mid is None or mid not in tr or not tr[mid]:
        return block, False
    # drop the whole existing msgstr (first line + continuations).
    # The replacement MUST be a lambda: a plain string replacement is itself
    # backslash-processed by re.sub, which turns the rendered "\n" escapes into
    # real newlines and halves the "\\" before a literal backslash (msgid
    # "\\#automation #smb" came out as "\#automation", 11 fatal msgfmt errors).
    block = re.sub(r'^msgstr (?:"(?:[^"\\]|\\.)*"\n?)+', lambda m: render(tr[mid]) + "\n", block, count=1, flags=re.M)
    block = re.sub(r'^#, fuzzy\n', "", block, flags=re.M)
    block = re.sub(r'^(#,.*?), fuzzy(.*)$', r'\1\2', block, flags=re.M)
    return block.rstrip("\n"), True


def main():
    po, jf = sys.argv[1], sys.argv[2]
    tr = json.load(open(jf, encoding="utf-8"))
    blocks = re.split(r"\n\n", open(po, encoding="utf-8").read())
    out, filled = [], 0
    for b in blocks:
        nb, did = fill_block(b, tr)
        filled += did
        out.append(nb)
    open(po, "w", encoding="utf-8").write("\n\n".join(out))
    print("filled %d entries" % filled)
    r = subprocess.run(["msgfmt", "--statistics", "-o", "/dev/null", po], capture_output=True, text=True)
    print(r.stderr.strip())


if __name__ == "__main__":
    main()

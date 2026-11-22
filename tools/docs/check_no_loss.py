#!/usr/bin/env python3
"""No-content-lost check for the learnings and rules splits.

  tools/docs/check_no_loss.py REF

Compares the lines of REF's one-file `docs/learnings.md` and `.claude/skills/legatum-rules/references/game-rules.md`
(as multisets of non-blank lines) with the monthly archives and the per-system rules files in the working tree. The old files'
titles and preambles are left out (the digest and index.md replace them); every other line must be present. Lines
only in the split files are the headers the split adds. Exit 1 on any lost line.
"""
import collections
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RULES = ".claude/skills/legatum-rules/references"


def git_lines(ref, path, header):
    """Non-blank lines of REF:PATH after its header (the old file's title and preamble, which the digest and
    index.md replace): HEADER is a line count, or None for everything before the first `## ` line."""
    out = subprocess.run(["git", "-C", ROOT, "show", f"{ref}:{path}"], check=True, capture_output=True, text=True)
    lines = out.stdout.split("\n")
    skip = header if header is not None else next(i for i, ln in enumerate(lines) if ln.startswith("## "))
    return [ln for ln in lines[skip:] if ln.strip()]


def file_lines(paths):
    out = []
    for p in paths:
        out += [ln for ln in open(p).read().split("\n") if ln.strip()]
    return out


def compare(name, src, dst):
    s, d = collections.Counter(src), collections.Counter(dst)
    lost = s - d
    extra = d - s
    print(f"{name}: {sum(s.values())} non-blank source lines, {sum(d.values())} in the split files, "
          f"{sum(lost.values())} lost, {sum(extra.values())} added (headers)")
    for ln in list(lost)[:10]:
        print("  LOST: " + ln[:140])
    for ln in list(extra)[:40]:
        print("  added: " + ln[:140])
    return not lost


def main(argv):
    if not argv:
        sys.exit(__doc__)
    ref = argv[0]
    arch = os.path.join(ROOT, "docs/learnings")
    months = sorted(os.path.join(arch, f) for f in os.listdir(arch) if f.endswith(".md"))
    ok = compare("learnings", git_lines(ref, "docs/learnings.md", None), file_lines(months))
    rules_dir = os.path.join(ROOT, RULES)
    parts = sorted(os.path.join(rules_dir, f) for f in os.listdir(rules_dir)
                   if f.endswith(".md") and f != "game-rules.md")
    ok &= compare("rules", git_lines(ref, f"{RULES}/game-rules.md", 6), file_lines(parts))
    # A merge that auto-merges master's one-file log into the digest silently re-inflates it (it happened once).
    for path, limit in (("docs/learnings.md", 30_000), (f"{RULES}/game-rules.md", 5_000)):
        size = os.path.getsize(os.path.join(ROOT, path))
        if size > limit:
            print(f"{path} is {size} bytes (limit {limit}): restore the digest or stub, then re-run the split")
            ok = False
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

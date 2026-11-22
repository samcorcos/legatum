#!/usr/bin/env python3
"""No-content-lost check for the learnings and rules splits.

  tools/docs/check_no_loss.py REF

Compares the lines of REF's one-file `docs/learnings.md` and `.claude/skills/legatum-rules/references/game-rules.md`
(as multisets of non-blank lines) with the monthly archives and the per-system rules files in the working tree. Lines
only in the split files are allowed only as the headers this split adds; every line of REF's files must be present.
Exit 1 on any lost line.
"""
import collections
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RULES = ".claude/skills/legatum-rules/references"


def git_lines(ref, path):
    out = subprocess.run(["git", "-C", ROOT, "show", f"{ref}:{path}"], check=True, capture_output=True, text=True)
    return [ln for ln in out.stdout.split("\n") if ln.strip()]


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
    ok = compare("learnings", git_lines(ref, "docs/learnings.md"), file_lines(months))
    rules_dir = os.path.join(ROOT, RULES)
    parts = sorted(os.path.join(rules_dir, f) for f in os.listdir(rules_dir)
                   if f.endswith(".md") and f != "game-rules.md")
    ok &= compare("rules", git_lines(ref, f"{RULES}/game-rules.md"), file_lines(parts))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

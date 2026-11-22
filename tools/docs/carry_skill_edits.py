#!/usr/bin/env python3
"""Carry edits that another branch made to a skill's old, long SKILL.md into the trimmed skill and its references.

  tools/docs/carry_skill_edits.py SKILL BASE_REF NEW_REF [--write]

The 2026-11-22 trim moved the history in `.claude/skills/<SKILL>/SKILL.md` verbatim into `references/*.md`. For each
hunk that NEW_REF changed in that skill's SKILL.md since BASE_REF (the commit the trim was made from), this finds the
hunk's old lines verbatim in the working tree's SKILL.md or references and replaces them with the new lines. Hunks
it cannot place (pure additions, or old text that the trim rewrote) are printed for you to place by hand: a current
rule goes in SKILL.md, history in the matching references file. Without --write it only reports.
"""
import difflib
import glob
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def show(ref, path):
    return subprocess.run(["git", "-C", ROOT, "show", f"{ref}:{path}"], check=True, capture_output=True,
                          text=True).stdout.split("\n")


def main(argv):
    if len(argv) < 3:
        sys.exit(__doc__)
    skill, base_ref, new_ref = argv[:3]
    write = "--write" in argv
    path = f".claude/skills/{skill}/SKILL.md"
    base, new = show(base_ref, path), show(new_ref, path)
    files = [os.path.join(ROOT, path)] + sorted(glob.glob(os.path.join(ROOT, f".claude/skills/{skill}/references/*.md")))
    texts = {f: open(f).read() for f in files}
    left = 0
    for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(None, base, new, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        old_lines, new_lines = base[i1:i2], new[j1:j2]
        placed = False
        if old_lines and any(ln.strip() for ln in old_lines):
            old = "\n".join(old_lines)
            hits = [f for f in files if texts[f].count(old) == 1]
            if len(hits) == 1:
                texts[hits[0]] = texts[hits[0]].replace(old, "\n".join(new_lines))
                print(f"{tag:7} base {i1 + 1}-{i2}: replaced in {os.path.relpath(hits[0], ROOT)}")
                placed = True
        if not placed and tag == "insert":
            # A pure addition goes after the nearest non-blank line before it, if that line is unique.
            pi = next((i for i in range(i1 - 1, -1, -1) if base[i].strip()), None)
            prev = base[pi] if pi is not None else None
            hits = [f for f in files if prev and texts[f].split("\n").count(prev) == 1]
            if len(hits) == 1:
                lines = texts[hits[0]].split("\n")
                at = min(lines.index(prev) + 1 + (i1 - pi - 1), len(lines))  # keep the blank lines in between
                texts[hits[0]] = "\n".join(lines[:at] + new_lines + lines[at:])
                print(f"insert  new {j1 + 1}-{j2}: after its anchor line in {os.path.relpath(hits[0], ROOT)}")
                placed = True
        if not placed:
            left += 1
            print(f"{tag:7} base {i1 + 1}-{i2} -> new {j1 + 1}-{j2}: PLACE BY HAND")
            for ln in old_lines:
                print("   - " + ln[:200])
            for ln in new_lines:
                print("   + " + ln[:200])
    if write:
        for f, t in texts.items():
            if open(f).read() != t:
                open(f, "w").write(t)
    print(f"{left} hunk(s) to place by hand")


if __name__ == "__main__":
    main(sys.argv[1:])

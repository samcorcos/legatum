#!/usr/bin/env python3
"""Carry edits made to the old one-file learnings log into the monthly archives.

  tools/docs/split_learnings.py NEW_REF [--base REF] [--write]

For a branch cut before the split: `tools/docs/split_learnings.py <branch> --base $(git merge-base origin/master
<branch>) --write` carries only that branch's own entries into the archives.

`docs/learnings.md` held every entry until 2026-11-22; it is now a short digest, and the entries live verbatim in
`docs/learnings/<YYYY-MM>.md`, newest first. A branch cut before the split may still add or edit entries in the old
file. After such a branch lands, run this with NEW_REF set to the commit that holds its full `docs/learnings.md` (for
example `origin/master` before the split merged, or the other branch's tip). It diffs that file's entries (`## `
blocks) against the version the archives were made from (`base` in tools/docs/learnings-split.json):
  - an entry that is new goes into the archive for the month in its heading, next to the entry it followed (or
    preceded) in NEW_REF, or at the top of that month when neither neighbour is in the same archive;
  - an entry whose text changed (same heading) replaces the old text in the archive;
  - an entry deleted in NEW_REF is removed from the archive.
Without --write it only reports. With --write it rewrites the archives and sets `base` to NEW_REF's commit. Then
check whether a new entry belongs in the digest (docs/learnings.md): only durable lessons go there.
"""
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
STATE = os.path.join(ROOT, "tools/docs/learnings-split.json")
ARCHIVE = os.path.join(ROOT, "docs/learnings")
DATE = re.compile(r"(20\d\d-\d\d)-\d\d")


def git(*args):
    return subprocess.run(["git", "-C", ROOT, *args], check=True, capture_output=True, text=True).stdout


def blocks(text):
    """(head lines, [entry blocks as lists of lines]) split at lines starting with '## '."""
    lines = text.split("\n")
    starts = [i for i, ln in enumerate(lines) if ln.startswith("## ")]
    if not starts:
        return lines, []
    ends = starts[1:] + [len(lines)]
    return lines[:starts[0]], [lines[s:e] for s, e in zip(starts, ends)]


def key(block):
    return "\n".join(block).rstrip("\n")


def month_of(block):
    m = DATE.search(block[0])
    if not m:
        sys.exit(f"no date in heading: {block[0]}")
    return m.group(1)


def main(argv):
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    new_ref, write = argv[0], "--write" in argv
    state = json.load(open(STATE))
    # --base REF: diff against REF instead (for an older branch, its merge base with master, so only that branch's own
    # additions and edits move and master's later entries are not taken for deletions); the recorded base is kept.
    base_ref = argv[argv.index("--base") + 1] if "--base" in argv else None
    _, base = blocks(git("show", f"{base_ref or state['base']}:docs/learnings.md"))
    _, new = blocks(git("show", f"{new_ref}:docs/learnings.md"))
    archives = {}
    for name in sorted(os.listdir(ARCHIVE)):
        if re.fullmatch(r"20\d\d-\d\d\.md", name):
            archives[name[:7]] = blocks(open(os.path.join(ARCHIVE, name)).read())

    def find(k):
        for month, (_, bl) in archives.items():
            for idx, b in enumerate(bl):
                if key(b) == k:
                    return month, idx
        return None

    base_keys = {key(b) for b in base}
    new_keys = {key(b) for b in new}
    by_head = {}
    for b in base:
        if key(b) not in new_keys:
            by_head.setdefault(b[0], []).append(b)
    report = {"added": 0, "changed": 0, "deleted": 0}
    by_hand = set()
    for pos, b in enumerate(new):
        k = key(b)
        if k in base_keys or find(k):  # unchanged, or already carried over
            continue
        old = by_head.get(b[0], [])
        if old:  # same heading, new text: replace in place
            found = find(key(old.pop(0)))
            if found:
                month, idx = found
                bl = archives[month][1]
                bl[idx] = b if b[-1] == "" or bl[idx][-1] != "" else b + [""]
                report["changed"] += 1
                print(f"changed {month}: {b[0][:100]}")
                continue
        month = month_of(b)
        if any(a[0] == b[0] for _, (_, bl) in archives.items() for a in bl):
            report.setdefault("by hand", 0)
            report["by hand"] += 1
            by_hand.add(b[0])
            print(f"BY HAND {month}: same heading already archived with other text: {b[0][:100]}")
            continue
        if month not in archives:
            archives[month] = ([f"# Learnings archive: {month}", "",
                                f"Entries dated {month}, newest first. Append new entries for this month at the top "
                                "of this file (below this note). Promote an entry to the digest in "
                                "`docs/learnings.md` only if it is durable.", ""], [])
        bl = archives[month][1]
        entry = b if b[-1] == "" else b + [""]
        prev = find(key(new[pos - 1])) if pos > 0 else None
        nxt = find(key(new[pos + 1])) if pos + 1 < len(new) else None
        if prev and prev[0] == month:
            at = prev[1] + 1
            if bl[prev[1]][-1] != "":
                bl[prev[1]] = bl[prev[1]] + [""]
        elif nxt and nxt[0] == month:
            at = nxt[1]
        else:
            at = 0
        if at == len(bl) and b[-1] != "":
            entry = b
        bl.insert(at, entry)
        report["added"] += 1
        print(f"added   {month} at {at}: {b[0][:100]}")
    for head, rest in by_head.items():
        for b in rest:
            found = find(key(b))
            if found:
                del archives[found[0]][1][found[1]]
                report["deleted"] += 1
                print(f"deleted {found[0]}: {head[:100]}")

    # No-loss check: every entry of NEW_REF is in an archive, verbatim.
    missing = [b[0] for b in new if not find(key(b)) and b[0] not in by_hand]
    assert not missing, f"entries not in any archive: {missing[:5]}"
    total = sum(len(bl) for _, bl in archives.values())
    print(f"{report}; {len(new)} entries in {new_ref}, all found verbatim in the archives ({total} archived entries)")
    if write:
        for month, (head, bl) in archives.items():
            text = "\n".join(head + [ln for b in bl for ln in b])
            open(os.path.join(ARCHIVE, f"{month}.md"), "w").write(text if text.endswith("\n") else text + "\n")
        if base_ref:
            print("written (learnings-split.json left as it was)")
            return
        state["base"] = git("rev-parse", new_ref).strip()
        with open(STATE, "w") as fh:
            json.dump(state, fh, indent=1)
            fh.write("\n")
        print("written; learnings-split.json base is now", state["base"][:12])


if __name__ == "__main__":
    main(sys.argv[1:])

#!/usr/bin/env python3
"""Carry edits made to the old one-file rules reference into the per-system files.

  tools/docs/split_rules.py NEW_REF [--write]

The rules reference was one file, `.claude/skills/legatum-rules/references/game-rules.md`, until 2026-11-22; it is now
one file per system in that folder (see `index.md`). A branch cut before the split may still edit the old file. After
such a branch lands, run this with NEW_REF set to the commit that holds the edited monolithic file (for example
`origin/master` before the split merged, or the other branch's tip): it diffs that file against the version the split
was made from (`base` in tools/docs/rules-map.json), maps every kept, changed or added line to the per-system file
that holds its neighbours (an added block that starts with a heading goes with the line after it, anything else with
the line before it), and regenerates the per-system files. Without --write it only reports. With --write it rewrites
the files and rules-map.json (whose `base` becomes NEW_REF's commit), so the next run starts from there.

Changes to the old file's first six lines (its header) are printed, not applied: fold them into `index.md` by hand.
Every line of the new file, header aside, lands in exactly one per-system file (checked before writing).
"""
import difflib
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
REF_DIR = ".claude/skills/legatum-rules/references"
OLD = REF_DIR + "/game-rules.md"
MAP = os.path.join(ROOT, "tools/docs/rules-map.json")
HEADER = 6  # the old file's title and preamble; their content lives in index.md


def git(*args):
    return subprocess.run(["git", "-C", ROOT, *args], check=True, capture_output=True, text=True).stdout


def lines_at(ref):
    text = git("show", f"{ref}:{OLD}")
    out = text.split("\n")
    return out[:-1] if out and out[-1] == "" else out


def render(title, question, body):
    out = [f"# {title}", "",
           f"Part of the Legatum rules reference (index: `index.md`). Answers: {question}",
           "Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings "
           "are kept inside.", ""]
    for block in body:
        out.extend(block)
        if out[-1].strip() != "":
            out.append("")
    return "\n".join(out).rstrip("\n") + "\n"


def ranges_of(nums):
    """Sorted 1-based line numbers -> [[start, end], ...]."""
    out = []
    for n in nums:
        if out and out[-1][1] == n - 1:
            out[-1][1] = n
        else:
            out.append([n, n])
    return out


def main(argv):
    if not argv or argv[0].startswith("-"):
        sys.exit(__doc__)
    new_ref, write = argv[0], "--write" in argv
    spec = json.load(open(MAP))
    base = lines_at(spec["base"])
    new = lines_at(new_ref)
    owner = [None] * len(base)  # 0-based base line -> file name
    for f in spec["files"]:
        for a, b in f["ranges"]:
            for i in range(a - 1, b):
                assert owner[i] is None, f"line {i + 1} in two files"
                owner[i] = f["file"]
    assert all(o is not None for o in owner[HEADER:]), "base lines without a file"

    new_owner = [None] * len(new)
    header_changed = False
    sm = difflib.SequenceMatcher(None, base, new, autojunk=False)
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "delete":
            if i1 < HEADER:
                header_changed = True
            continue
        for k, j in enumerate(range(j1, j2)):
            if tag in ("equal", "replace"):
                i = min(i1 + k, i2 - 1)
            else:  # insert: a heading goes with the following text, anything else with the text before
                i = i1 if new[j1].startswith("#") and i1 < len(base) else i1 - 1
            if i < HEADER:  # stays with the header (index.md)
                header_changed |= tag != "equal"
            else:
                new_owner[j] = owner[i]
        if tag != "equal":
            where = ", ".join(sorted({str(new_owner[j]) for j in range(j1, j2)}))
            print(f"{tag:7} old {i1 + 1}-{i2} -> new {j1 + 1}-{j2}: {where}")

    head_new = [new[j] for j in range(len(new)) if new_owner[j] is None]
    if header_changed:
        print("header lines changed; fold into index.md by hand:")
        for ln in difflib.unified_diff(base[:HEADER], head_new, lineterm=""):
            print("  " + ln)

    files = []
    for f in spec["files"]:
        nums = [j + 1 for j in range(len(new)) if new_owner[j] == f["file"]]
        rs = ranges_of(nums)
        text = render(f["title"], f["question"], [new[a - 1:b] for a, b in rs])
        files.append(dict(f, ranges=rs, text=text))
    # No-loss check: every body line of the new file is in exactly one generated file.
    assigned = sum(len(range(a, b + 1)) for f in files for a, b in f["ranges"])
    assert assigned == len([o for o in new_owner if o is not None]), "line count mismatch"
    changed = [f["file"] for f in files
               if not os.path.exists(os.path.join(ROOT, REF_DIR, f["file"]))
               or open(os.path.join(ROOT, REF_DIR, f["file"])).read() != f["text"]]
    print(f"base {spec['base'][:12]} ({len(base)} lines) -> {new_ref} ({len(new)} lines); "
          f"{assigned} body lines placed; files that change: {', '.join(changed) or 'none'}")
    if write:
        for f in files:
            open(os.path.join(ROOT, REF_DIR, f["file"]), "w").write(f["text"])
        spec["base"] = git("rev-parse", new_ref).strip()
        spec["files"] = [{k: f[k] for k in ("file", "title", "question", "ranges")} for f in files]
        with open(MAP, "w") as fh:
            json.dump(spec, fh, indent=1)
            fh.write("\n")
        print("written; rules-map.json base is now", spec["base"][:12])


if __name__ == "__main__":
    main(sys.argv[1:])

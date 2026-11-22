#!/usr/bin/env python3
"""Copy new Training-question answer files into docs/playtests/training-answers/ for the lead to read.

The desktop app appends one JSON line per answered or skipped question to
`<app data>/training/answers/<game-id>.jsonl`. This copies any file that is new or has grown
(never touching saves) and prints what it copied. Usage:

  tools/training/collect.py [--from DIR] [--dry-run]

--from defaults to the app data dir for com.legatum.game under ~/Library/Application Support.
"""
import argparse
import pathlib
import shutil
import sys

DEFAULT = pathlib.Path.home() / "Library/Application Support/com.legatum.game/training/answers"
DEST = pathlib.Path(__file__).resolve().parents[2] / "docs/playtests/training-answers"


def lines(path: pathlib.Path) -> int:
    return sum(1 for _ in path.open("rb"))


def collect(src: pathlib.Path, dest: pathlib.Path, dry: bool = False) -> list[str]:
    copied = []
    if not src.is_dir():
        return copied
    for f in sorted(src.glob("*.jsonl")):
        target = dest / f.name
        if target.exists() and lines(target) >= lines(f):
            continue
        copied.append(f"{f.name}: {lines(f)} lines")
        if not dry:
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(f, target)
    return copied


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--from", dest="src", type=pathlib.Path, default=DEFAULT)
    ap.add_argument("--to", dest="dest", type=pathlib.Path, default=DEST)
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    if not a.src.is_dir():
        print(f"no answers directory at {a.src}", file=sys.stderr)
        return 0
    done = collect(a.src, a.dest, a.dry_run)
    print("\n".join(done) if done else "nothing new")
    return 0


if __name__ == "__main__":
    sys.exit(main())

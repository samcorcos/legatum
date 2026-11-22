#!/usr/bin/env python3
"""Rename legacy faction identifiers and names in tracked text files.

This script intentionally uses only the Python standard library. It can be
rerun after merging onto another branch: already-renamed paths and content are
left alone, and existing destination files are treated as already migrated.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]

# These are kept explicit so a branch that has already received equivalent
# changes from another branch remains safe to migrate.
RENAMES = (
    ("tollkeepers", "switzerland"),
    ("tollkeeper", "swiss"),
    ("choir", "japan"),
    ("remnants", "byzantines"),
    ("remnant", "byzantium"),
    ("crucible", "germany"),
    ("veyr", "celts"),
    ("glassroad", "mongols"),
    ("oathbound", "sparta"),
    ("signalwrights", "athens"),
    ("signalwright", "athenian"),
)

# Ordinary-English uses are reviewed here rather than blindly changing them.
# Each entry is (tracked path, exact lowercase word); faction-name occurrences
# in that file are still renamed. A repository grep found no ordinary-English
# use of "remnant" or "remnants", so this stays empty until one is justified.
REMNANT_PROSE_EXCEPTIONS: tuple[tuple[str, str], ...] = ()

SKIP_PREFIXES = (".progress/", ".claude/worktrees/")
SKIP_FILES = ("tools/dev/rename-legacy-factions.py",)


def git(*args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=ROOT)


def replacement_for(match: re.Match[str]) -> str:
    old = match.group(0)
    for source, target in RENAMES:
        if old.lower() != source:
            continue
        if old.isupper():
            return target.upper()
        if old.islower():
            return target
        if old[0].isupper() and old[1:].islower():
            return target.title()
        # Preserve the initial capitalization for mixed-case identifiers.
        return target.title() if old[0].isupper() else target
    raise AssertionError(f"unmapped faction token: {old}")


def token_pattern() -> re.Pattern[str]:
    alternatives = "|".join(
        re.escape(source) for source, _ in sorted(RENAMES, key=lambda item: -len(item[0]))
    )
    # Recognize ordinary word boundaries, snake/kebab names, and CamelCase
    # components such as ChoirGoal. The trailing lowercase guard prevents
    # matching a faction-name prefix inside an unrelated English word.
    return re.compile(
        rf"(?<![A-Za-z0-9])(?i:(?:{alternatives}))(?![a-z])"
        rf"|(?<=[a-z0-9])(?i:(?:{alternatives}))(?![a-z])",
    )


def is_binary(data: bytes) -> bool:
    return b"\0" in data


def replace_path(path: str, pattern: re.Pattern[str]) -> str:
    return pattern.sub(replacement_for, path)


def preserved_ranges(path: str, text: str) -> list[tuple[int, int]]:
    """Keep compatibility literals stable when rerunning on an already-migrated branch."""
    ranges = [
        (match.start(), match.end())
        for match in re.finditer(r'#\[serde\([^\]]*\balias\s*=\s*"[^"]*"[^\]]*\)\]', text)
    ]
    if path == "crates/ai/src/params.rs":
        match = re.search(
            r"(?ms)^\s*fn legacy_faction_and_gene_names_still_load\(\)\s*\{.*?^\s*\}",
            text,
        )
        if match:
            ranges.append((match.start(), match.end()))
    return ranges


def main() -> int:
    pattern = token_pattern()
    exceptions = set(REMNANT_PROSE_EXCEPTIONS)
    tracked = [p.decode("utf-8") for p in git("ls-files", "-z").split(b"\0") if p]
    changes: dict[str, tuple[bytes, int]] = {}

    for name in tracked:
        if name.startswith(SKIP_PREFIXES) or name in SKIP_FILES:
            continue
        path = ROOT / name
        try:
            original = path.read_bytes()
        except FileNotFoundError:
            continue
        if is_binary(original):
            continue
        try:
            text = original.decode("utf-8")
        except UnicodeDecodeError:
            continue

        # Leave only hand-reviewed common-English uses of remnant(s) intact.
        protected = preserved_ranges(name, text)
        count = 0
        pieces = []
        last = 0
        for match in pattern.finditer(text):
            token = match.group(0)
            if any(start <= match.start() < end for start, end in protected):
                continue
            if (name, token.lower()) in exceptions:
                continue
            pieces.append(text[last : match.start()])
            pieces.append(replacement_for(match))
            last = match.end()
            count += 1
        if count:
            pieces.append(text[last:])
            changes[name] = ("".join(pieces).encode("utf-8"), count)

    # Rename tracked paths before content writes. Git mv records moves cleanly;
    # if another branch already created the destination, regard it as migrated.
    path_moves: list[tuple[str, str]] = []
    for name in tracked:
        if name.startswith(SKIP_PREFIXES) or name in SKIP_FILES:
            continue
        dest = replace_path(name, pattern)
        if dest != name:
            path_moves.append((name, dest))

    for source, dest in sorted(path_moves, key=lambda pair: (pair[0].count("/"), pair[0])):
        source_path = ROOT / source
        dest_path = ROOT / dest
        if not source_path.exists():
            continue
        if dest_path.exists():
            # A file/directory with the new name may already be present on the
            # target branch. Identical migrated files collapse to the existing
            # destination; divergent files are left for an explicit merge.
            if source_path.is_file() and dest_path.is_file():
                migrated = changes.get(source, (source_path.read_bytes(), 0))[0]
                if migrated == dest_path.read_bytes():
                    subprocess.check_call(["git", "rm", "--", source], cwd=ROOT)
                    continue
            raise SystemExit(
                f"rename conflict: both {source!r} and {dest!r} exist with different contents"
            )
        dest_path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.check_call(["git", "mv", "--", source, dest], cwd=ROOT)

    # A path move does not change bytes; write modified content at the current
    # working-tree path. Git's index is updated by git mv and ordinary writes.
    moved = dict(path_moves)
    summaries: list[tuple[str, int]] = []
    for name, (content, count) in sorted(changes.items()):
        current = moved.get(name, name)
        path = ROOT / current
        if not path.exists():
            continue
        if path.read_bytes() == content:
            continue
        path.write_bytes(content)
        summaries.append((current, count))

    print(f"Renamed {sum(count for _, count in summaries)} occurrences in {len(summaries)} files.")
    for name, count in summaries:
        print(f"{name}: {count}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

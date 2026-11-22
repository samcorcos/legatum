"""Writers for the run log contract in docs/archive/ai-evolution-log.md.

Whole files are replaced atomically (temp file + rename); JSONL lines are appended with one
O_APPEND write each, so readers tailing a live run see whole lines (and skip a torn last one).
"""

from __future__ import annotations

import gzip
import json
import os
from pathlib import Path


def write_json(path: Path, obj, indent: int | None = 2) -> None:
    write_text(path, json.dumps(obj, indent=indent) + "\n")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.tmp")
    tmp.write_text(text)
    os.replace(tmp, path)


def write_gz_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.tmp")
    with gzip.open(tmp, "wt") as f:
        json.dump(obj, f)
    os.replace(tmp, path)


def read_gz_json(path: Path):
    with gzip.open(path, "rt") as f:
        return json.load(f)


def append_jsonl(path: Path, obj) -> None:
    line = (json.dumps(obj, separators=(",", ":")) + "\n").encode()
    fd = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        os.write(fd, line)
    finally:
        os.close(fd)


def read_jsonl(path: Path) -> list[dict]:
    if not path.exists():
        return []
    out = []
    for line in path.read_text().splitlines():
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            pass  # a torn last line from an interrupted append
    return out


def truncate_jsonl(path: Path, keep) -> None:
    """Resume: drop lines a crash left for generations the saved state will redo."""
    rows = [r for r in read_jsonl(path) if keep(r)]
    write_text(path, "".join(json.dumps(r, separators=(",", ":")) + "\n" for r in rows))

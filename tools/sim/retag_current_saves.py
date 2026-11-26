#!/usr/bin/env python3
"""Stamp only current-rule test copies after a deliberate RULES_VERSION bump.

Recorded user saves and legacy fixtures are intentionally left at their original version.
Usage: python3 tools/sim/retag_current_saves.py 10
"""
import gzip
import json
from pathlib import Path
import sys

version = int(sys.argv[1])
root = Path(__file__).resolve().parents[2]
paths = [
    *root.glob("crates/wasm/tests/fixtures/**/*.current.legatum"),
    *root.glob("app/e2e/saves/*.current.legatum.gz"),
    *root.glob("app/src/test/fixtures/*.current.json"),
]

for path in sorted(paths):
    compressed = path.suffix == ".gz"
    raw = path.read_bytes()
    document = json.loads(raw if not (compressed) else gzip.decompress(raw))
    if not ("game" in document):
        document["save_version"] = version
    else:
        game = json.loads(document["game"])
        game["save_version"] = version
        document["game"] = json.dumps(game, separators=(",", ":"), ensure_ascii=False)
    data = (json.dumps(document, separators=(",", ":"), ensure_ascii=False) + "\n").encode()
    path.write_bytes(data if not (compressed) else gzip.compress(data, mtime=0))
    print(path.relative_to(root))

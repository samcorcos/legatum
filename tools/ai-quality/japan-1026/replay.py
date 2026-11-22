#!/usr/bin/env python3
"""Replay every packaged position with its pre-plan memory; retain phase traces and orders.

replay.py INPUT_DIR BINARY OUTPUT.json.gz [DECISIONS.json]
"""
import gzip
import json
import subprocess
import sys
from pathlib import Path

directory, binary, output = map(Path, sys.argv[1:4])
arm = sys.argv[4] if len(sys.argv) > 4 else str(directory / 'default.json')
results = []
for view in sorted(directory.glob('*.view.json.gz')):
    prefix = str(view).removesuffix('.view.json.gz')
    proc = subprocess.run([str(binary.resolve()), str(view), '3811224394289556', arm,
                           prefix + '.memory.json.gz'], check=True, capture_output=True, text=True)
    r = json.loads(proc.stdout)
    r['trace'] = [json.loads(line[6:]) for line in proc.stderr.splitlines() if line.startswith('AUDIT ')]
    recorded = json.loads(gzip.decompress(Path(prefix + '.recorded.json.gz').read_bytes()))
    r['matches_recorded'] = r['orders'] == recorded
    results.append(r)
output.parent.mkdir(parents=True, exist_ok=True)
output.write_bytes(gzip.compress(json.dumps(results, separators=(',', ':')).encode(), mtime=0))
print(f'{len(results)} positions; {sum(r["matches_recorded"] for r in results)} recorded-order matches -> {output}')

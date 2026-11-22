#!/usr/bin/env python3
"""Compare current planner arms on extracted fogged views and original brain memory.

prepare STATES PLANS INPUTS: write compact inputs for the remote box.
run INPUTS BINARY OUTPUT: replay each arm once; no simulation, no hidden state.
"""
import gzip
import json
import subprocess
import sys
from pathlib import Path

POSITIONS = [(18, 1), (19, 1), (37, 3), (38, 3), (129, 2), (142, 2), (164, 2), (193, 3)]
BORDER = dict(peace_exposure_percent=100, threat_horizon_turns=2,
              assessment_interval=1, army_threat_weight=100, field_strike=100)
CITY = dict(scout_threat=100, walled_defense_percent=100, counter_composition=100)
ARMS = dict(default={}, border=BORDER, city=CITY, both=BORDER | CITY,
            fixes=dict(founding_survival_weight=200, raid_arrival_weight=100))
if sys.argv[1] == 'prepare':
    states, plans, out = map(Path, sys.argv[2:])
    out.mkdir(parents=True, exist_ok=True)
    records = {(r['turn'], r['player']): r for r in map(json.loads, plans.read_text().splitlines())}
    for t, p in POSITIONS:
        prefix = f'T{t}-p{p}'
        (out / f'{prefix}.view.json.gz').write_bytes(gzip.compress((states / f'{prefix}.view.json').read_bytes(), mtime=0))
        r = records[t, p]
        for name, data in [('memory', r['memory_before']), ('recorded', r['orders'])]:
            (out / f'{prefix}.{name}.json.gz').write_bytes(gzip.compress(json.dumps(data).encode(), mtime=0))
    for name, decisions in ARMS.items():
        (out / f'{name}.json').write_text(json.dumps(decisions))
elif sys.argv[1] == 'run':
    directory, binary, out = map(Path, sys.argv[2:])
    results = []
    for t, p in POSITIONS:
        prefix = directory / f'T{t}-p{p}'
        recorded = json.loads(gzip.decompress(Path(f'{prefix}.recorded.json.gz').read_bytes()))
        for arm in ARMS:
            run = subprocess.run([str(binary.resolve()), f'{prefix}.view.json.gz', '7732777812994077',
                                  str(directory / f'{arm}.json'), f'{prefix}.memory.json.gz'],
                                 check=True, text=True, capture_output=True)
            result = json.loads(run.stdout)
            result.update(arm=arm, matches_recorded=result['orders'] == recorded)
            results.append(result)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(gzip.compress(json.dumps(results, separators=(',', ':')).encode(), mtime=0))
    print(f'{len(results)} planning replays -> {out}')
else:
    raise SystemExit(__doc__)

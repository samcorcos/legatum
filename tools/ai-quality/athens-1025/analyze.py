#!/usr/bin/env python3
"""Reproducible all-turn ledger from the immutable save copy and hash-checked extraction.

analyze.py SAVE STATES OUT.json.gz
Coordinates are axial. Unit fields and public city defense come from the original view;
raw city/garrison fields are labeled separately and are never supplied to a planner.
"""
import collections
import gzip
import json
import sys
from pathlib import Path

save, states, out = map(Path, sys.argv[1:])
game = json.loads(json.loads(save.read_text())['game'])
plans = {(r['turn'], r['player']): r for r in map(json.loads, (states/'plans.jsonl').read_text().splitlines())}
events = {r['turn']: r['events'] for r in map(json.loads, (states/'events.jsonl').read_text().splitlines())}
summaries = [json.loads(s) for s in (states/'summary.jsonl').read_text().splitlines()]
assert all(r['hash_ok'] for r in summaries)
rows = []
for tape in game['archive']['tape']['turns']:
    t = tape['turn']
    v = json.loads((states/f'T{t}-p5.view.json').read_text())
    raw = json.loads((states/f'T{t}.json').read_text())
    units = [u for u in v['units'] if u['owner'] == 5]
    row = dict(turn=t, swiss_units=units, swiss_aircraft=v.get('aircraft', []), swiss_economy=v['economy'], swiss_supply=v.get('supply'),
               visible_rivals=[u for u in v['units'] if u['owner'] != 5],
               known_cities=v['cities'], standings=v['standings'], titles=v['titles'],
               raw_cities=raw['cities'], raw_units=raw['units'],
               holds=raw.get('vp_holds',{}), orders=tape['orders'], events=events[t],
               plans={p: plans[t,p] for p in range(1,8) if (t,p) in plans})
    rows.append(row)
result = dict(metadata=game['archive']['metadata'],
              verification=dict(hashes=len(summaries), plan_matches=sum(r['matches_recorded'] for r in plans.values()), plans=len(plans)), turns=rows)
out.parent.mkdir(parents=True, exist_ok=True)
out.write_bytes(gzip.compress(json.dumps(result,separators=(',',':')).encode(),mtime=0))
print(result['verification'], out)

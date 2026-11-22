#!/usr/bin/env python3
"""Compact, reproducible ledger from hash-checked original-engine states and tape.
Usage: analyze.py SAVE STATES OUT
Turns are PRE-turn states; events and orders resolve during that turn.
"""
import collections
import gzip
import hashlib
import json
import sys
from pathlib import Path
save, directory, out = map(Path, sys.argv[1:])
out.mkdir(parents=True, exist_ok=True)
game = json.loads(json.loads(save.read_text())['game'])
tape = game['archive']['tape']['turns']
events = {r['turn']: r['events'] for r in map(json.loads, (directory/'events.jsonl').read_text().splitlines())}
plans = list(map(json.loads, (directory/'plans.jsonl').read_text().splitlines()))
ledger = []
for turn in range(1, game['turn'] + 1):
    state = json.loads((directory/f'T{turn}.json').read_text())
    players = []
    for p in state['players']:
        i = p['id']
        path = directory/f'T{turn}-p{i}.view.json'
        v = json.loads(path.read_text()) if path.exists() else {}
        units = [u for u in state['units'].values() if u['owner'] == i]
        cities = [c for c in state['cities'].values() if c['owner'] == i]
        city_ids = {c['id'] for c in cities}
        players.append(dict(id=i, faction=p['faction'], alive=p['alive'], gold=p['gold'],
                            techs=p['techs'], units=units, cities=cities,
                            aircraft=[a for a in state.get('aircraft',[]) if a['city'] in city_ids],
                            economy=v.get('economy'), supply=v.get('supply'),
                            public=next((x for x in v.get('players',[]) if x['id']==i),None),
                            upgrade_options=[{k:u.get(k) for k in ['id','kind','hp','upgrade_cost','upgrade_need','upgrade_supply_blocker','unsupplied']} for u in v.get('units',[]) if u['owner']==i],
                            observed_threats=v.get('observed_threats'),
                            visible_rivals=[u for u in v.get('units',[]) if u['owner'] != i]))
    ledger.append(dict(turn=turn, players=players, events=events.get(turn,[]),
                       orders=tape[turn-1]['orders'] if turn<=len(tape) else {}))
summary = list(map(json.loads, (directory/'summary.jsonl').read_text().splitlines()))
meta = dict(engine=game['archive']['metadata']['engine_commit'], seed=game['setup']['seed'],
            save_sha256=hashlib.sha256(save.read_bytes()).hexdigest(), turns=len(tape),
            matching_hashes=sum(r['hash_ok'] for r in summary),
            saved_snapshot_equal=all(state[k] == game.get(k) for k in state),
            planner_matches={i:[sum(r['matches_recorded'] for r in plans if r['player']==i),sum(r['player']==i for r in plans)] for i in range(1,5)})
assert meta['matching_hashes'] == len(tape), 'replay hash mismatch'
assert meta['saved_snapshot_equal'], 'saved snapshot differs from replay'
for name,data in [('ledger',dict(metadata=meta,turns=ledger)),('original-plans',plans)]:
    (out/f'{name}.json.gz').write_bytes(gzip.compress(json.dumps(data,separators=(',',':')).encode(),mtime=0))
print(json.dumps(meta,indent=2))
for i in range(5):
    first = next((r['turn']-1 for r in ledger if 'Flight' in r['players'][i]['techs']),None)
    builds=[(r['turn'],e) for r in ledger for e in r['events'] if e['type']=='ProductionCompleted' and e['owner']==i and e['item'].get('unit') in ['Fighter','Bomber']]
    strikes=[r['turn'] for r in ledger for o in r['orders'].get(str(i),[]) if o['type']=='AirStrike']
    print(i,'Flight completed',first,'air completions',builds,'strikes',len(strikes))
for r in ledger:
    for e in r['events']:
        if e['type'] in ['CityCaptured','PlayerEliminated','AircraftRebased','AirIntercepted']:
            print(r['turn'],e)

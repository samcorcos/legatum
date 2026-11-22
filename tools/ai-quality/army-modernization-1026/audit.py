#!/usr/bin/env python3
"""Summarize original-engine extract-state.sh output, without simulating another game.
Usage: audit.py STATES SOURCE_SAVE OUTPUT_DIR RULES_JSON
"""
import collections
import gzip
import hashlib
import json
import pathlib
import sys

states, source, out, rules_path = map(pathlib.Path, sys.argv[1:])
out.mkdir(parents=True, exist_ok=True)
rules = json.loads(rules_path.read_text())
stats = rules['unitStats']
final = json.loads((states / 'T143.json').read_text())
plans = [json.loads(line) for line in (states / 'plans.jsonl').read_text().splitlines()]
events = {r['turn']: r['events'] for r in map(json.loads, (states / 'events.jsonl').read_text().splitlines())}
recorded = json.loads(json.loads(source.read_text())['game'])['archive']['tape']['turns']
orders = {r['turn']: r['orders'] for r in recorded}

def army(units):
    return dict(collections.Counter(rules['eraNames'][stats[u['kind']]['eraTier']] for u in units if stats[u['kind']].get('class') in ['Melee', 'Ranged', 'Mounted']))

def eligibility(v, u):
    to = stats[u['kind']].get('upgradesTo')
    if not to:
        return None
    blocks = []
    tech = stats[to].get('requires')
    if tech and tech not in v['research']['known']:
        blocks.append('technology: ' + tech)
    col = u['pos']['q'] + u['pos']['r'] // 2
    tile = v['tiles'][u['pos']['r'] * v['width'] + col]
    if tile['owner'] != v['me']:
        blocks.append('outside own territory')
    cost = (u.get('upgrade_breakdown') or {}).get('total')
    if cost is not None and v['economy']['gold'] < cost:
        blocks.append('Gold')
    if u.get('upgrade_supply_blocker'):
        blocks.append(u['upgrade_supply_blocker'])
    return dict(unit=u['id'], kind=u['kind'], to=to, gold=cost, legal_quote=u['upgrade_cost'], blockers=blocks, hp=u['hp'], level=u['level'], pos=u['pos'])

ledger = []
for turn in range(103,143):
    state = json.loads((states / f'T{turn}.json').read_text())
    for player in state['players']:
        i = player['id']
        v = json.loads((states / f'T{turn}-p{i}.view.json').read_text())
        own = [u for u in v['units'] if u['owner'] == i]
        ledger.append(dict(turn=turn, player=i, faction=player['faction'], gold=player['gold'], gold_per_turn=v['economy']['gold_per_turn'], techs=player['techs'], supply=v['supply'], army_by_era=army(own), units=own,
            upgrades=[r for u in own if (r := eligibility(v,u))],
            production=[dict(city=c['id'], name=c['name'], producing=c['detail']['producing'], turns=c['detail']['turns_to_complete']) for c in v['cities'] if c['owner']==i],
            orders=orders[turn].get(str(i),[]),
            events=[e for e in events[turn] if e.get('owner',e.get('player')) == i and e['type'] in ['ProductionCompleted','UnitUpgraded','ItemBought','UnitDisbanded','TechResearched','CityCaptured','CityRazed']]))
summary=[]
for p in final['players']:
    i=p['id']; v=json.loads((states/f'T142-p{i}.view.json').read_text()); rows=[r for r in ledger if r['player']==i]
    options={}
    for c in v['cities']:
        if c['owner']!=i: continue
        for o in c['detail']['options']+c['detail']['unavailable']:
            if o['item']['kind']=='Unit':
                kind=o['item']['unit']
                prior=options.get(kind)
                rank=lambda q: (not bool(q.get('reason')), not bool(q.get('resource_blocker')))
                if prior is None or rank(o)>rank(prior): options[kind]=o
    best={}
    for cls in ['Melee','Ranged','Mounted','Air']:
        kinds=[k for k,s in stats.items() if s.get('class')==cls]
        known=[k for k in kinds if not stats[k].get('requires') or stats[k]['requires'] in p['techs']]
        buildable=[k for k in known if k in options and options[k].get('resource_blocker') is None and not options[k].get('reason')]
        best[cls]=dict(researched=sorted(known,key=lambda k:stats[k]['eraTier'],reverse=True), buildable_t142=sorted(buildable,key=lambda k:stats[k]['eraTier'],reverse=True), options_t142={k:options.get(k) for k in kinds})
    summary.append(dict(player=i,faction=p['faction'],alive=p['alive'],gold=p['gold'],known_techs=p['techs'],army_by_era=army([u for u in final['units'].values() if u['owner']==i]), army_by_kind=dict(collections.Counter(u['kind'] for u in final['units'].values() if u['owner']==i)), supply_t142=v['supply'],classes=best,gold_min=min(r['gold'] for r in rows),gold_max=max(r['gold'] for r in rows),completed=dict(collections.Counter(e['item']['unit'] for r in rows for e in r['events'] if e['type']=='ProductionCompleted' and e['item']['kind']=='Unit')),upgrades_bought=[dict(turn=r['turn'],gold_before=r['gold'],**e) for r in rows for e in r['events'] if e['type']=='UnitUpgraded']))
for name,value in [('ledger',ledger),('summary',summary)]:
    (out/f'{name}.json.gz').write_bytes(gzip.compress(json.dumps(value,separators=(',',':')).encode(),mtime=0))
(out/'provenance.json').write_text(json.dumps(dict(source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),original_engine=json.loads(json.loads(source.read_text())['game'])['archive']['metadata']['engine_commit'],turns=142,original_brain_matches=sum(p['matches_recorded'] for p in plans),original_brain_total=len(plans),rules_sha256=hashlib.sha256(rules_path.read_bytes()).hexdigest()),indent=2)+'\n')
for r in summary:
 print(r['faction'],r['army_by_era'],'Gold',r['gold'],'range',r['gold_min'],r['gold_max'],'built',r['completed'],'upgrades',len(r['upgrades_bought']))

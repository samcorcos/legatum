#!/usr/bin/env python3
"""Summarize hash-checked extract-state output, never modify the source saves.

Usage: analyze.py FINAL MID STATES OUT
Production = completed construction cost excluding purchases (which also emit completion).
Allocation = pre-resolution option yields into the ordered/current queue; a planning proxy,
not realized currency, since combat/capture/focus can change the economic phase.
"""
import collections
import hashlib
import json
import sys
from pathlib import Path

final, mid, directory, output = map(Path, sys.argv[1:])
load = lambda p: json.loads(p.read_text())
game = json.loads(load(final)['game'])
other = json.loads(load(mid)['game'])
tape = game['archive']['tape']['turns']
assert tape[:len(other['archive']['tape']['turns'])] == other['archive']['tape']['turns']
for snapshot in [other, game]:
 rebuilt = load(directory / f'T{snapshot["turn"]}.json')
 assert all(snapshot.get(key) == value for key, value in rebuilt.items()), 'snapshot differs from tape'
summary = [json.loads(s) for s in (directory/'summary.jsonl').read_text().splitlines()]
assert len(summary) == len(tape) and all(t['hash_ok'] for t in summary)
events = {r['turn']: r['events'] for r in map(json.loads, (directory/'events.jsonl').read_text().splitlines())}
noncombat = {'Scout', 'Settler', 'Heir'}
names = {'Soldier':'Spearman', 'Ranger':'Archer', 'Trooper':'Pikeman', 'AssaultTrooper':'Musketman', 'Marksman':'Crossbowman', 'Drone':'CombatDrone'}
combat = lambda item: item and item.get('kind') == 'Unit' and item['unit'] not in noncombat
walls = lambda item: item and item.get('building') in ['Walls','Castle','StarFort','CustomsBastion']

def army(v,p):
 us=[u for u in v['units'] if u['owner']==p and u['military'] and u['kind'] not in noncombat]
 return dict(count=len(us), composition=dict(collections.Counter(names.get(u['kind'],u['kind']) for u in us)), hp=sum(u['hp'] for u in us), levels=dict(collections.Counter(u['level'] for u in us)), strength_hp=sum(max(u['strength'],u['ranged'])*(100+u['level']*10)*u['hp']//10000 for u in us), units=[{k:u[k] for k in ['id','kind','pos','hp','strength','ranged','level','xp','fortify']} for u in us])

rows=[]; battles=[]; deaths=[]; captures=[]; diplomacy=[]
for entry in tape:
 t=entry['turn'];state=load(directory/f'T{t}.json');es=events[t]
 views={p:load(directory/f'T{t}-p{p}.view.json') for p in range(4)}
 units={int(k):u for k,u in state['units'].items()}
 for e in es:
  if e['type']=='Battle': battles.append(dict(turn=t,**e))
  if e['type']=='UnitDestroyed':
   involved=[b for b in es if b['type']=='Battle' and ((b['defender'].get('unit')==e['unit'] and b['defender_hp_after']<=0) or (b['attacker']==e['unit'] and b['attacker_hp_after']<=0))]
   attribution='direct'
   if not involved:
    involved=[b for b in es if b['type']=='Battle' and b['defender'].get('kind')=='City' and b['defender'].get('owner')==e['owner'] and b['at']==e['at']]
    attribution='city-overflow' if involved else 'unattributed'
   killers=sorted(set(b['attacker_owner'] if b['defender'].get('unit')==e['unit'] or attribution=='city-overflow' else b['defender']['owner'] for b in involved))
   shots=sorted(set(b['attacker'] for b in involved if (b['defender'].get('unit')==e['unit'] or attribution=='city-overflow') and isinstance(b['attacker'],int)))
   u=units.get(e['unit'],{});v=views.get(e['owner'],{}); uv=next((x for x in v.get('units',[]) if x['id']==e['unit']),{})
   deaths.append(dict(turn=t,**e,killers=killers,killing_units=shots,attribution=attribution,hp_before=u.get('hp'),level=uv.get('level'),order=next((o for o in entry['orders'].get(str(e['owner']),[]) if o.get('unit')==e['unit']),None),enemy_shot=any(b['ranged'] and b['defender'].get('unit')==e['unit'] for b in involved)))
  if e['type']=='CityCaptured': captures.append(dict(turn=t,**e))
  if any(s in e['type'] for s in ['War','Peace','Treaty']):diplomacy.append(dict(turn=t,**e))
 for p,v in views.items():
  cs=[c for c in v['cities'] if c['owner']==p];us=[u for u in v['units'] if u['owner']==p];orders=entry['orders'].get(str(p),[])
  alloc=collections.Counter();spent=collections.Counter();built=collections.Counter(); gold=collections.Counter()
  for c in cs:
   d=c['detail'];item=next((o['item'] for o in orders if o['type']=='SetProduction' and o['city']==c['id']),d['producing']);opt=next((o for o in d['options'] if o['item']==item),None);y=opt['yields']['total'] if opt else d['yields']['total'];alloc['all']+=y['production'];alloc['military' if combat(item) else 'walls' if walls(item) else 'civilian']+=y['production']
  bought={(e['city'],json.dumps(e['item'],sort_keys=True)) for e in es if e['type']=='ItemBought' and e['owner']==p}
  for e in es:
   if e.get('owner')!=p:continue
   typ=e['type'];item=e.get('item');cat='military' if combat(item) else 'walls' if walls(item) else 'civilian'
   if typ=='ProductionCompleted' and (e['city'],json.dumps(item,sort_keys=True)) not in bought:
    c=next((c for c in cs if c['id']==e['city']),None)
    if c:
     opt=next((o for o in c['detail']['options'] if o['item']==item),None)
     if opt:spent[cat]+=opt['cost'];built[names.get(item.get('unit'),item.get('unit')) or item.get('building') or item.get('improvement') or str(item)]+=1
   if typ=='ItemBought':gold[cat]+=e['gold']
   if typ=='UnitUpgraded':gold['upgrades']+=e['gold']
   if typ=='SiegeBought':gold['siege']+=e['gold']
  yr=collections.Counter();network=collections.Counter()
  for c in cs:
   yr.update(c['detail']['yields']['total'])
   for src in c['detail']['yields']['sources']:
    if src['label'].startswith('Japan'):network.update(src['yields'])
  legal=[u for u in us if u['upgrade_cost'] is not None]
  rows.append(dict(turn=t,player=p,alive=state['players'][p]['alive'],cities=len(cs),population=sum(c['population'] for c in cs),tiles=sum(x.get('owner')==p for x in state['map']['tiles']),yields=dict(yr),network=dict(network),gold=v['economy']['gold'],gold_income=v['economy']['gold_per_turn']['total'],techs=len(v['research']['known']),army=army(v,p),military_allocation=dict(alloc),completed_production=dict(spent),gold_spent=dict(gold),completed_items=dict(built),legal_upgrades=len(legal),skipped_legal_upgrades=[dict(unit=u['id'],cost=u['upgrade_cost'],order=next((o for o in orders if o.get('unit')==u['id']),None)) for u in legal if not any(o['type']=='UpgradeUnit' and o['unit']==u['id'] for o in orders)],supply=v['supply']))

# Pair war windows are historical, including the terminal turn's events; expenses are each
# participant's empire total during that interval, so overlapping wars must not be added.
wars=[];active={}
chronicle=game['history']['chronicle']
for e in chronicle:
 if e['kind']=='War':active[tuple(sorted([e['a'],e['b']]))]=(e['turn'],e['by'])
 if e['kind'] in ['Peace','Eliminated','Won']:
  pairs=[tuple(sorted([e['a'],e['b']]))] if e['kind']=='Peace' else [pair for pair in active if e['kind']=='Won' or e['player'] in pair]
  for pair in pairs:
   if pair not in active:continue
   start,by=active.pop(pair);wars.append(dict(pair=pair,start=start,end=e['turn'],declarer=by,ending=e['kind']))
for pair,(start,by) in active.items():wars.append(dict(pair=pair,start=start,end=tape[-1]['turn'],declarer=by,ending='save-end'))
for war in wars:
 a,b=war['pair'];start,end=war['start'],war['end'];war['sides']={}
 for p,enemy in [(a,b),(b,a)]:
  rr=[r for r in rows if r['player']==p and start<=r['turn']<=end];fields={}
  for f in ['completed_production','military_allocation','gold_spent','completed_items']:
   c=collections.Counter()
   for r in rr:c.update(r[f])
   fields[f]=dict(c)
  war['sides'][p]=dict(army_start=rr[0]['army'],army_end=rr[-1]['army'],gold_start=rr[0]['gold'],gold_peak=max(r['gold'] for r in rr),losses_to_opponent=sum(d['owner']==p and d['unit_kind'] not in noncombat and enemy in d['killers'] and start<=d['turn']<=end for d in deaths),scout_losses_to_opponent=sum(d['owner']==p and d['unit_kind']=='Scout' and enemy in d['killers'] and start<=d['turn']<=end for d in deaths),**fields)
 war['captures']=[c for c in captures if start<=c['turn']<=end and set([c.get('from'),c.get('to')])==set(war['pair'])]

out=dict(provenance=dict(engine=game['archive']['metadata']['engine_commit'],hashes_verified=len(summary),prefix_turns=len(other['archive']['tape']['turns']),snapshots_verified=[other['turn'],game['turn']],sources={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [final,mid]}),wars=wars,rows=rows,battles=battles,deaths=deaths,captures=captures,diplomacy=diplomacy)
output.parent.mkdir(parents=True,exist_ok=True)
output.write_text(json.dumps(out,separators=(',',':'))+'\n')
print('Wrote',output,'wars',len(wars),'battles',len(battles),'deaths',len(deaths))

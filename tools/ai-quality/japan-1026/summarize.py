#!/usr/bin/env python3
"""Readable German turn ledger plus per-unit phase attribution. Run after analyze/trace."""
import csv,gzip,io,json,sys
from pathlib import Path
out=Path(sys.argv[1]); ledger=json.load(gzip.open(out/'ledger.json.gz'))['turns']; traces=json.load(gzip.open(out/'original-trace.json.gz'))
tr={(r['turn'],r['player']):r for r in traces}
names={c['id']:c['name'] for r in ledger for p in r['players'] for c in p['cities']}
def item(x):return x.get('unit',x.get('building',x.get('improvement',str(x))))
def plan(r):
 k=r['memory']['campaigns']['plan']['active']['kind']
 return k['plan']+(' '+str(k.get('target',k.get('against',k.get('with')))) if len(k)>1 else '')
lines=['# Germany, every turn of the Japanese war','',
       'Turns are pre-turn positions; results happen during that turn. Orders and phase labels come from the matching original-engine planner. `Soldier` means Spearman; `Ranger` means Archer. A skip is an explicit planner order. Omitted units may retain standing movement. Each raw order and its last assigning phase is in `germany-unit-turns.csv.gz`; the full candidate scores are in `original-trace.json.gz`.','',
       '| Turn | Plan | German production / purchases / upgrades | Ground orders (move / fire / skip) | Results |','| --- | --- | --- | --- | --- |']
buf=io.StringIO();w=csv.writer(buf);w.writerow(['turn','unit','kind','hp','position','order','assigning_phase'])
for r in ledger[171:221]:
 t=r['turn'];p=r['players'][2];o=r['orders'].get('2',[]);trace=tr[t,2];phase={};prev=[]
 records=trace['trace'];start=max(i for i,x in enumerate(records) if x['phase']=='choose_plan')
 for x in records[start:]:
  if 'orders' not in x: continue
  for cmd in x['orders']:
   if cmd not in prev and 'unit' in cmd:phase[json.dumps(cmd,sort_keys=True)]=x['phase']
  prev=x['orders']
 for u in p['units']:
  cmd=next((x for x in o if x.get('unit')==u['id']),None)
  w.writerow([t,u['id'],u['kind'],u['hp'],json.dumps(u['pos']),json.dumps(cmd),phase.get(json.dumps(cmd,sort_keys=True),'no new order')])
 builds=[]
 for x in o:
  if x['type'] in ['SetProduction','BuyItem']:builds.append(('Buy ' if x['type']=='BuyItem' else '')+names[x['city']]+' '+item(x['item']))
  elif x['type']=='UpgradeUnit':builds.append('upgrade '+str(x['unit']))
 counts=[sum(x['type'] in types for x in o) for types in [('MoveUnit','MoveUnitPath'),('RangedAttack',),('SkipUnit',)]]
 results=[]
 for e in r['events']:
  if e['type']=='CityCaptured' and 2 in [e['from'],e['to']]:results.append(names[e['city']]+' → '+str(e['to']))
  if e['type']=='ProductionCompleted' and e['owner']==2 and e['item'].get('unit') in ['Bomber','Fighter']:results.append(item(e['item'])+' completes')
  if e['type']=='UnitDestroyed' and e['owner']==2:results.append(str(e['unit'])+' '+e['unit_kind']+' lost')
 lines.append(f'| {t} | {plan(trace)} | '+('; '.join(builds) or 'continues queues')+' | '+' / '.join(map(str,counts))+' | '+('; '.join(results) or '—')+' |')
(out/'germany-turns.md').write_text('\n'.join(lines)+'\n')
(out/'germany-unit-turns.csv.gz').write_bytes(gzip.compress(buf.getvalue().encode(),mtime=0))

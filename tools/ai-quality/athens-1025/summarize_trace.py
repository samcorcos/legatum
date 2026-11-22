#!/usr/bin/env python3
"""Turn offline order snapshots into a per-unit audit and a readable turn table.

summarize_trace.py LEDGER.json.gz TRACE.json.gz OUT_DIR
"""
import csv
import gzip
import io
import json
import sys
from pathlib import Path
ledger,trace,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=True)
rows=json.load(gzip.open(ledger))['turns']
records={(r['turn'],r['player']):r for r in json.load(gzip.open(trace))}
explanations={
'garrison_cities(early_mongol_guard)':'City garrison reserved before siege because of the Mongol war',
'garrison_cities(routine)':'Routine city garrison after home defense and siege planning',
'hold_captures':'Hold damaged/recently captured home city',
'recover_city_fire':'Leave city strike after forecast HP/exchange check',
'avoid_free_damage':'Avoid visible enemy/city damage before the late siege pass',
'recover_wounded':'Recover wounded unit before attacking',
'campaign':'Safe staging at campaign target; hold or upgrade when already on safe ring',
'defend_cities':'Home defense reservation',
'rest':'No earlier pass assigned an order; rest/return home/upgrade',
'dedupe_first_steps':'Movement preflight canceled or changed prior order due to traffic',
'siege_war':'Local siege ratio passed; take a survivable firing/screen tile',
'fire_from_holds':'Turn stationary hold into legal ranged fire',
'choose_stance':'Assigned before military passes (scouting/civilian/empire planning)',
'coordinate_city_assaults':'Coordinated city capture forecast passed',
'attack_targets':'Local target/arrival strength comparison passed',
'ranged_fire':'Select available ranged shot',
'counterattack':'Favorable immediate combat exchange',
'plan_siege_buys':'Buy Siege before army movement',
}
fields=['turn','unit','kind','q','r','hp','level','xp','strength','ranged','fortify','action','destination','decision_pass','explanation','prior_pass','siege_ratio_city14','siege_need','eligible_spots','minimum_max_city_strike','chosen_siege_spot']
f=io.StringIO();writer=csv.DictWriter(f,fieldnames=fields);writer.writeheader()
md=['# Switzerland: every war turn','', 'Pre-turn positions; orders resolve on the named turn. The [unit ledger](swiss-unit-turns.csv.gz) lists every unit, HP, promotion level, strength, order, assigning pass and siege rejection numbers. Full snapshots are in `original-trace.json.gz`; views, memory, events and raw-state checks are in `ledger.json.gz`.','', '| Turn | Plan / campaign | Unit orders by assigning pass | Siege gate at Athens city 14 |','| --- | --- | --- | --- |']
for row in rows:
 t=row['turn']; rec=records.get((t,5))
 if rec is None: continue
 last={};source={};prior={};spots={};ratios=[];past_early_guard=False
 for snap in rec['trace']:
  phase=snap['phase']
  if phase=='reserve_front_guards':past_early_guard=True
  if phase=='garrison_cities':phase += '(routine)' if past_early_guard else '(early_mongol_guard)'
  if phase=='siege_spot':spots[snap['unit']]=snap;continue
  if phase=='siege_ratio':
   if snap['city']==14:ratios.append(snap)
   continue
  cur={o['unit']:o for o in snap['orders'] if 'unit'in o}
  for unit,order in cur.items():
   if order!=last.get(unit):prior[unit]=source.get(unit,'');source[unit]=phase
  last=cur
 final={o['unit']:o for o in rec['orders'] if 'unit'in o}
 groups={}
 for u in row['swiss_units']:
  unit=u['id'];o=final.get(unit,{});p=source.get(unit,'none')
  spot=spots.get(unit,{});eligible=[c for c in spot.get('candidates',[]) if c['free'] and c['reachable'] and c['los']]
  ratio=ratios[-1] if ratios else {}
  writer.writerow(dict(turn=t,unit=unit,kind=u['kind'],q=u['pos']['q'],r=u['pos']['r'],hp=u['hp'],level=u['level'],xp=u['xp'],strength=u['strength'],ranged=u['ranged'],fortify=u['fortify'],action=o.get('type','none'),destination=json.dumps(o.get('to',o.get('target','')),separators=(',',':')),decision_pass=p,explanation=explanations.get(p,p),prior_pass=prior.get(unit,''),siege_ratio_city14=ratio.get('margin',''),siege_need=ratio.get('need',''),eligible_spots=len(eligible) if spot else '',minimum_max_city_strike=min((c['max_city_damage'] for c in eligible),default=''),chosen_siege_spot=json.dumps(spot.get('spot',''),separators=(',',':'))))
  groups.setdefault(p,[]).append(f'{unit}:{o.get("type","none").replace("Unit","").replace("RangedAttack","fire")}')
 for plane in row.get('swiss_aircraft', []):
  order=next((o for o in rec['orders'] if o.get('plane')==plane['id']), {})
  writer.writerow(dict(turn=t,unit=plane['id'],kind=plane['kind'],hp=plane['hp'],action=order.get('type','none'),destination=json.dumps(order.get('target','')),decision_pass='air::air_orders',explanation='Highest-value visible target in hangar range; no order when none is visible/legal'))
  groups.setdefault('air::air_orders',[]).append(f"{plane['id']}:{order.get('type','none')}")
 if not(67<=t<=96 or 177<=t<=223):continue
 plan=rec['memory']['campaigns']['plan']['active'];active=rec['memory']['campaigns']['active']
 kind=plan['kind'] if plan else {}
 gate='; '.join(f'{s["mine"]}/{s["their"]} = {s["margin"]}% vs {s["need"]}%' for s in ratios)
 md.append(f'| {t} | {kind.get("plan")} {kind.get("against", "")} / {active["target"] if active else "none"} | '+ '; '.join(f'`{p}` '+','.join(ids) for p,ids in groups.items()) +f' | {gate or "No eligible ratio check"} |')
(out/'swiss-unit-turns.csv.gz').write_bytes(gzip.compress(f.getvalue().encode(),mtime=0))
(out/'swiss-turns.md').write_text('\n'.join(md)+'\n')
print(len(md)-6,'war-turn rows, unit ledger saved')

#!/usr/bin/env python3
"""Adapt the noise-audit paired gate to predeclared parameter arms in one frozen binary.

The gate math and record validation are imported unchanged. This importer checks native
Spot logs, binary and parameter hashes, completed slices, arm identities and salt coverage.
Usage: gate_report.py PROTOCOL MANIFEST ARMS_JSON BATCH_ROOT [BATCH_ROOT ...]
Outputs per-arm validated result records beside PROTOCOL. Append disjoint slices only.
"""
import gzip
import importlib.util
import json
from pathlib import Path
import shlex
import sys

root=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('quality_gate',root/'tools/sim/quality_gate.py')
gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
protocol_path, manifest_path, params_path=map(Path,sys.argv[1:4])
p=gate.read(protocol_path);manifest=gate.read(manifest_path);params=gate.read(params_path)
assert manifest['commit']==p['control_commit']==p['candidate_commit']
assert manifest['maps_sha256']==p['maps_sha256'] and manifest['profile']=='sim'
assert all(arm in p['arms'] for arm in params['arms'])
assert len({a['label'] for a in params['arms']})==len(params['arms'])
allowed_params={}
for path in protocol_path.parent.glob('*arms*.json'):
 value=gate.read(path)
 if isinstance(value,dict) and value.get('arms') and all(arm in p['arms'] for arm in value['arms']):
  allowed_params['/tmp/ai-params-'+gate.digest(path.read_bytes())[:16]]=value['arms']
chunks=[]
for batch in map(Path,sys.argv[4:]):
 for directory in sorted(batch.glob('slice-*')):
  done=dict(s.split('=',1) for s in (directory/'DONE').read_text().splitlines())
  assert done['rc']=='0',directory
  log=(directory/'batch.log').read_text()
  assert '/bin/'+manifest['binary_sha256'][:16]+'/hard_vs_human.gz' in log
  commands=[shlex.split(s[2:]) for s in log.splitlines() if s.startswith('+ env ') and '/tmp/hard_vs_human ' in s]
  assert len(commands)==1
  cmd=commands[0];b=cmd.index('/tmp/hard_vs_human');env=cmd[1:b]
  assert 'LEGATUM_HVH_RESEED_DICE=1' in env
  assert all(x=='LEGATUM_HVH_RESEED_DICE=1' or x.startswith('RAYON_NUM_THREADS=') for x in env)
  opts={};i=b+1
  while i<len(cmd):
   key=cmd[i];assert key not in opts
   if key=='--arms':opts[key]=True;i+=1
   else:opts[key]=cmd[i+1];i+=2
  assert set(opts)<=set(['--arms','--ai-params','--games','--seed','--label','--out-dir','--out','--status-file'])
  assert opts['--arms'] and opts['--ai-params'] in allowed_params
  assert all(arm in allowed_params[opts['--ai-params']] for arm in params['arms'])
  chunks.append((int(opts['--seed']),int(opts['--games']),directory))
chunks.sort();next_salt=p['salt_start']
for start,n,_ in chunks:
 assert start==next_salt,('gap/overlap',start,next_salt)
 next_salt+=n
n=next_salt-p['salt_start'];assert n in p['looks']
for arm in params['arms']:
 label=arm['label'];games=[[] for _ in p['maps']];evidence=[]
 for start,count,directory in chunks:
  report=gate.read(directory/'out/hard-vs-human-arms.json.gz')[label]
  assert report['salt_start']==start and report['salts_per_map']==count
  assert report['brain']==arm['brain'] and report['seat_mode']=='all-seats'
  assert report['candidate_params']==arm['params']['name'] and not report.get('rules',[])
  path=directory/f'out/{label}-games.json.gz';raw=gate.read(path)
  assert raw['maps']==p['maps'] and all(len(row)==count for row in raw['games'])
  assert len(raw['games'])==len(games)
  for dst,rows in zip(games,raw['games']):dst.extend(rows)
  evidence.append({'directory':str(directory),'salt_start':start,'salts':count,'games_sha256':gate.digest(path.read_bytes()),'log_sha256':gate.digest((directory/'batch.log').read_bytes())})
 result=dict(manifest,protocol_id=gate.digest(gate.canonical(p)),maps=p['maps'],salts=n,salt_start=p['salt_start'],arm=label,wins=[[int(g['win']) for g in row] for row in games],game_hashes=[[gate.digest(gate.canonical(g)) for g in row] for row in games],chunks=evidence)
 gate.validate_records(result,p)
 gate.write(protocol_path.parent/f'{label}-result.json.gz',result)
 gate.write(protocol_path.parent/f'{label}-games.json.gz',{'maps':p['maps'],'games':games})
 if label!='ctl':
  control=gate.read(protocol_path.parent/'ctl-result.json.gz')
  repeat=gate.read(protocol_path.parent/'repeat/ctl-result.json.gz')
  gate.validate_records(repeat,p)
  assert repeat['salts']==p['looks'][0]
  assert repeat['commit']==control['commit'] and repeat['binary_sha256']==control['binary_sha256']
  assert repeat['game_hashes']==[r[:repeat['salts']] for r in control['game_hashes']], 'repeatability failed'
  comparison=gate.compare(control,result,p)
  comparison['enable']=comparison['decision']=='noninferior' and comparison['delta']>0
  gate.write(protocol_path.parent/f'{label}-look-{n}.json',comparison)
  print(label,json.dumps(comparison))
 else:print(label,n,sum(map(sum,result['wins'])))

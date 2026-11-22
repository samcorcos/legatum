#!/usr/bin/env python3
"""Frozen-control, paired fixed-map gate. Standard library only; filler first, validated spot imports when budget allows.

prepare writes an immutable protocol and priority-35 filler spec. Submit/fetch with
 tools/cloud/filler.sh; analyze validates provenance and applies the planned looks. ingest validates completed run-batch slices.
The worker fails closed outside Linux, validates the embedded maps, strips ambient
LEGATUM_* switches, hashes the binary and retains full per-game records in gzip.
"""
import argparse
import fcntl
import gzip
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import statistics
import shlex
import subprocess
import sys
import time

LOOKS = [128, 256, 512]  # salts PER MAP, 1,024/2,048/4,096 games
FLOOR = -0.015
ARMS = {"arms": [{"label": "ctl", "brain": "params", "mode": "all",
                  "params": {"schema": 1, "name": "builtin", "decisions": {}}}]}


def gene_arms(genes):
    """Compare default-on genes in one binary. The rivals arm explicitly turns them
    off in the measured seat; mode=opponents with the same binary would silently
    leave its default-on behavior in that seat too.
    """
    if not genes or len(set(genes)) != len(genes) or any(not g.isidentifier() for g in genes):
        raise ValueError('provide distinct gene names')
    # Read the same source that defines the simulator's factions and defaults. A shared
    # zero is not an off switch when with_builtin_faction replaces shared defaults.
    root = Path(__file__).resolve().parents[2]
    factions_source = (root / 'crates/core/src/types.rs').read_text()
    factions = re.findall(r'Self::(\w+)', factions_source.split('pub const ALL: [Faction;', 1)[1].split('];', 1)[0])
    source = (root / 'crates/ai/src/params.rs').read_text()
    defaults = {name: int(value) for name, value in re.findall(
        r'^    (\w+): (-?\d+), -?\d+, -?\d+,', source, re.M)}
    builtins = {(faction, name): int(value) for faction, name, value in re.findall(
        r'\(Faction::(\w+), "(\w+)", (-?\d+)\)',
        source.split('pub const BUILTIN_FACTION_DECISIONS', 1)[1].split('];', 1)[0])}
    unknown = set(genes) - defaults.keys()
    if unknown:
        raise ValueError(f'unknown genes: {sorted(unknown)}')
    off = {g: 0 for g in genes}
    on = {f: {g: builtins.get((f, g), defaults[g]) for g in genes} for f in factions}
    def params(name, values):
        return {'schema': 1, 'name': name, 'decisions': values,
                'faction_overrides': {f: dict(values) for f in factions}}
    on_params = {'schema': 1, 'name': 'genes-on',
                 'faction_overrides': on}
    return {'arms': [
        {'label': 'ctl', 'brain': 'params', 'mode': 'all', 'params': params('genes-off', off)},
        {'label': 'rivals', 'brain': 'params', 'mode': 'seat', 'params': params('seat-genes-off', off)},
        {'label': 'all', 'brain': 'params', 'mode': 'all', 'params': on_params},
    ]}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def read(path):
    path = Path(path)
    with (gzip.open(path, 'rt') if path.suffix == '.gz' else path.open()) as f:
        return json.load(f)


def write(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with (gzip.open(path, 'wt') if path.suffix == '.gz' else path.open('w')) as f:
        json.dump(value, f, sort_keys=True, indent=None if path.suffix == '.gz' else 2)
        f.write('\n')


def variance(xs):
    return statistics.variance(xs) if len(xs) > 1 else 0.0


def validate_records(data, protocol):
    if data['protocol_id'] != digest(canonical(protocol)):
        raise ValueError('protocol mismatch: never reuse this control')
    if data['maps'] != protocol['maps']:
        raise ValueError('map order or identity mismatch')
    if data.get('salt_start', 1) != protocol.get('salt_start', 1):
        raise ValueError('salt range mismatch')
    n = data['salts']
    if n not in protocol['looks']:
        raise ValueError('unplanned look')
    for rows in (data['wins'], data['game_hashes']):
        if len(rows) != len(protocol['maps']) or any(len(r) != n for r in rows):
            raise ValueError('incomplete map/salt records')
    if any(type(w) is not int or w not in (0, 1) for r in data['wins'] for w in r):
        raise ValueError('invalid win outcome')
    check_provenance(data['commit'], data['binary_sha256'])


def _hex_id(value, lengths):
    return (isinstance(value, str) and len(value) in lengths and set(value) <= set('0123456789abcdef')
            and set(value) != {'0'})


def check_provenance(commit, binary):
    """Reject a missing, malformed or all-zero commit (40 hex) or binary id (64 hex, or the 16-hex
    prefix a cloud batch records in its upload key). Zero placeholders must never pass as provenance."""
    if not _hex_id(commit, (40,)):
        raise ValueError(f'missing or placeholder source commit: {commit!r}')
    if not _hex_id(binary, (16, 64)):
        raise ValueError(f'missing or placeholder binary identifier: {binary!r}')


# Roles of the candidate arm. `compare` computes in "strength" units: positive means a stronger AI.
#   seat       candidate plays the human seat: strength = +(seat win delta)
#   opponents  candidate plays every other seat: lower seat wins is stronger, strength = -(seat win delta)
#   all-seats  both at once; the seat delta is (seat strength gain) - (opponent strength gain), so no
#              single sign recovers strength and the decision is 'role-mixed'.
ROLE_SIGN = {'seat': 1, 'opponents': -1}


def compare(control, candidate, protocol, role='seat'):
    """Paired comparison. `delta` and the intervals stay the raw candidate-minus-control seat win
    delta; `strength_effect`/`strength_interval` and the decision use the role-normalised sign."""
    if role not in ROLE_SIGN and role != 'all-seats':
        raise ValueError(f'unknown candidate role {role!r}')
    sign = ROLE_SIGN.get(role, 1)
    for data in (control, candidate):
        validate_records(data, protocol)
    if 'control_commit' in protocol and control['commit'] != protocol['control_commit']:
        raise ValueError('wrong frozen control source')
    n = candidate['salts']
    if control['salts'] < n:
        raise ValueError('control is shorter than candidate')
    m = len(protocol['maps'])
    differences = [[sign * (c - b) for c, b in zip(cr[:n], br[:n])]
                   for br, cr in zip(control['wins'], candidate['wins'])]
    delta = statistics.mean(statistics.mean(r) for r in differences)
    # Equal-weight fixed map strata. Also cluster by salt to avoid understating
    # observed covariance across maps. Use the larger SE for decisions.
    strat_var = sum(variance(r) for r in differences) / (m * m * n)
    salt_means = [statistics.mean(r[s] for r in differences) for s in range(n)]
    cluster_var = variance(salt_means) / n
    se = math.sqrt(max(strat_var, cluster_var))
    alpha = protocol['alpha']
    z = statistics.NormalDist().inv_cdf(1 - alpha / (2 * len(protocol['looks']) * protocol['family_size']))
    lo, hi = max(-1, delta - z * se), min(1, delta + z * se)
    # No degenerate certainty from an all-identical small cohort.
    sparse = sum(abs(d) for r in differences for d in r) < 20
    unresolved_variance = sparse or se == 0
    descriptive = [delta-1.959964*se, delta+1.959964*se]
    if unresolved_variance:
        lo, hi = -1, 1
        descriptive = [-1, 1]
        if abs(delta) == 1:
            # Constant losses/wins still provide evidence. Salt averages lie in
            # [-1, 1]; Hoeffding gives a conservative bound without a variance
            # estimate, assuming independent salts, even with dependent maps.
            def bounded_interval(error):
                radius = math.sqrt(2 * math.log(2 / error) / n)
                return [max(-1, delta-radius), min(1, delta+radius)]
            lo, hi = bounded_interval(alpha / (len(protocol['looks']) * protocol['family_size']))
            descriptive = bounded_interval(alpha)
    # Only a rule the designer already decided on other grounds may ship on an unresolved interval;
    # AI-development gates report 'inconclusive' instead.
    final = ('ship-monitor' if protocol.get('policy') == 'designer-rule-ship-unless-clear-harm'
             else 'inconclusive')
    decision = ('clear-harm' if hi < protocol['floor'] else
                'noninferior' if lo > protocol['floor'] else
                final if n == max(protocol['looks']) else 'continue')
    strength = {'effect': delta, 'ci95': descriptive, 'interval': [lo, hi]}
    if role == 'all-seats':
        decision, strength = 'role-mixed', {'effect': None, 'ci95': None, 'interval': None}
    # Raw seat-win view: undo the sign (flipping an interval swaps its ends).
    raw = (lambda x: x) if sign > 0 else (lambda x: -x)
    raw_pair = (lambda p: list(p)) if sign > 0 else (lambda p: [-p[1], -p[0]])
    delta = raw(delta)
    lo, hi = raw_pair([lo, hi])
    descriptive = raw_pair(descriptive)
    return {'role': role, 'strength_sign': sign if role != 'all-seats' else None,
            'strength_effect': strength['effect'], 'strength_ci95': strength['ci95'],
            'strength_interval': strength['interval'],
            'games': m*n, 'salts': n, 'control_wins': sum(sum(r[:n]) for r in control['wins']),
            'candidate_wins': sum(map(sum, candidate['wins'])), 'delta': delta,
            'se': se, 'stratified_se': math.sqrt(strat_var), 'salt_cluster_se': math.sqrt(cluster_var),
            'ci95_descriptive': descriptive,
            'simultaneous_interval': [lo, hi], 'critical_z': z, 'sparse_discordance': sparse,
            'variance_unresolved': unresolved_variance, 'decision': decision, 'per_map': [
                {'map': name, 'delta': raw(statistics.mean(r)), 'paired_variance': variance(r),
                 'control_wins': sum(control['wins'][i][:n]),
                 'candidate_wins': sum(candidate['wins'][i])}
                for i, (name, r) in enumerate(zip(protocol['maps'], differences))],
            'mde80': None if unresolved_variance else {str(g): (1.959964 + 0.841621)*se*math.sqrt(m*n/g) for g in (1024,2048,4096)},
            'sequential_mde80': None if unresolved_variance else {str(g): (z + 0.841621)*se*math.sqrt(m*n/g) for g in (1024,2048,4096)}}


def worker(args):
    if platform.system() != 'Linux':
        raise RuntimeError('Simulation worker is box-only; no laptop fallback')
    protocol = read('protocol.json')
    src = Path(args.binary).resolve().parents[3]
    maps_path = src / 'crates/sim/data/hard-vs-human-maps.json'
    if digest(maps_path.read_bytes()) != protocol['maps_sha256']:
        raise ValueError('candidate changed the fixed map fixture')
    if subprocess.check_output(['git','rev-parse','HEAD'], cwd=src, text=True).strip() != args.commit:
        raise ValueError('checkout commit does not match declared source')
    binary_hash = digest(Path(args.binary).read_bytes())
    env = {k:v for k,v in os.environ.items() if not k.startswith('LEGATUM_')}
    env['LEGATUM_HVH_RESEED_DICE'] = '1'
    write('arms.json', protocol['arms'])
    control = read('control.json') if Path('control.json').exists() else None
    if args.role == 'candidate' and control is None:
        raise ValueError('candidate needs frozen control')
    if control:
        validate_records(control, protocol)
        if control['commit'] != protocol['control_commit']:
            raise ValueError('wrong control source')
    combined = {'maps': protocol['maps'], 'games': [[] for _ in protocol['maps']]}
    started = time.time()
    result = None
    for n in protocol['looks']:
        count = n - len(combined['games'][0])
        start = protocol['salt_start'] + len(combined['games'][0])
        out = Path(f'chunk-{n}'); out.mkdir()
        cmd = [args.binary, '--arms', '--ai-params', 'arms.json', '--games', str(count),
               '--seed', str(start), '--out-dir', str(out)]
        print(json.dumps({'event':'start','commit':args.commit,'command':cmd}), flush=True)
        subprocess.run(cmd, env=env, check=True, timeout=protocol['chunk_timeout_s'], stdout=subprocess.DEVNULL)
        chunk = read(out / 'ctl-games.json')
        if chunk['maps'] != protocol['maps'] or any(len(r) != count for r in chunk['games']):
            raise ValueError('incomplete chunk')
        for dst, rows in zip(combined['games'], chunk['games']):
            dst.extend(rows)
        result = {'protocol_id': digest(canonical(protocol)), 'commit': args.commit,
                  'binary_sha256': binary_hash, 'maps': protocol['maps'], 'salts': n,
                  'salt_start': protocol['salt_start'],
                  'wins': [[int(g['win']) for g in row] for row in combined['games']],
                  'game_hashes': [[digest(canonical(g)) for g in row] for row in combined['games']],
                  'elapsed_s': time.time()-started,
                  'rustc': subprocess.check_output(['rustc','--version'], text=True).strip(),
                  'machine': platform.machine(), 'rayon_threads': env.get('RAYON_NUM_THREADS')}
        write('result.json', result)
        write('games.json.gz', combined)
        if args.role == 'repeat':
            break
        if control:
            assessment = compare(control, result, protocol)
            write(f'look-{n}.json', assessment)
            print(json.dumps(assessment), flush=True)
            if assessment['decision'] != 'continue':
                break
    print(json.dumps({'event':'complete','salts':result['salts'],'elapsed_s':time.time()-started}), flush=True)


def prepare(args):
    # Several worktrees may share a daily control. Serialize protocol and family
    # registration so concurrent submissions cannot silently exceed the alpha budget.
    dest = Path(args.directory)
    dest.mkdir(parents=True, exist_ok=True)
    with (dest / '.prepare.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        prepare_locked(args)


def prepare_locked(args):
    root = Path(__file__).resolve().parents[2]
    commit = subprocess.check_output(['git','rev-parse',args.commit+'^{commit}'], cwd=root, text=True).strip()
    dest = Path(args.directory); dest.mkdir(parents=True, exist_ok=True)
    proto_path = dest / 'protocol.json'
    if args.role == 'control':
        maps_bytes = subprocess.check_output(['git','show',commit+':crates/sim/data/hard-vs-human-maps.json'],cwd=root)
        maps = json.loads(maps_bytes)
        protocol = {'version':1,'control_commit':commit,'maps_sha256':digest(maps_bytes),
                    'maps':[m['id'] for m in maps], 'map_setups':[m['setup'] for m in maps],
                    'arms':ARMS,'dice_reseed':True,'salt_start':args.salt_start,'looks':LOOKS,
                    'floor':FLOOR,'alpha':0.05,'family_size':args.family_size,
                    'chunk_timeout_s':3600,'policy':'designer-rule-ship-unless-clear-harm'}
        if proto_path.exists() and read(proto_path) != protocol:
            raise ValueError('protocol is frozen; choose a new directory')
        write(proto_path, protocol)
    else:
        protocol = read(proto_path)
    params_path = dest / 'arms.json'
    if params_path.exists():
        if read(params_path) != protocol['arms']:
            raise ValueError('existing parameter file differs from frozen arms')
    else:
        write(params_path, protocol['arms'])
    inputs = {'gate.py':Path(__file__).read_text(),'protocol.json':json.dumps(protocol)}
    if args.role == 'candidate':
        control = read(dest / 'control/result.json')
        validate_records(control, protocol)
        if control['commit'] != protocol['control_commit'] or control['salts'] != max(LOOKS):
            raise ValueError('need complete frozen control')
        repeat = read(dest / 'repeat/result.json')
        validate_records(repeat, protocol)
        n = repeat['salts']
        if (repeat['commit'] != control['commit'] or repeat['binary_sha256'] != control['binary_sha256']
                or repeat['game_hashes'] != [r[:n] for r in control['game_hashes']]):
            raise ValueError('repeatability check failed; do not test candidates')
        inputs['control.json'] = json.dumps(control)
    if args.role == 'repeat' and commit != protocol['control_commit']:
        raise ValueError('repeat must use frozen commit')
    spec = {'id':args.id,'priority':35,'commit':commit,
            'build':['tools/sim/build-slot.sh','--','cargo','build','--locked','--profile','sim','-p','legatum-sim','--example','hard_vs_human'],
            'argv':['python3','gate.py','worker','--binary','/data/filler/src/target/sim/examples/hard_vs_human',
                    '--commit',commit,'--role',args.role], 'inputs':inputs,
            'env':{'RAYON_NUM_THREADS':'64' if args.role == 'candidate' else '48',
                   'CARGO_BUILD_JOBS':'8'},
            'outputs':['result.json','games.json.gz']+[f'look-{n}.json' for n in LOOKS]}
    spec_path = dest / (args.id+'.spec.json')
    if spec_path.exists() and read(spec_path) != spec:
        raise ValueError('job id already describes different inputs; use a new campaign/id')
    if args.role == 'candidate':
        registry_path = dest / 'candidates.json'
        registry = read(registry_path) if registry_path.exists() else {}
        if args.id in registry and registry[args.id] != commit:
            raise ValueError('candidate id changed source')
        registry[args.id] = commit
        if len(registry) > protocol['family_size']:
            raise ValueError('predeclared candidate family exhausted; start a new campaign')
        write(registry_path, registry)
    write(spec_path, spec)
    print(spec_path)



def ingest(protocol, manifest, chunks, params_path):
    """Import completed run-batch slices without relaxing the frozen protocol.

    The manifest comes from the exact-commit box build (commit, binary hash,
    map hash, profile, rustc). Slice logs must prove that binary, the exact
    parameter-file bytes, reseeding, --arms, and the declared salt interval.
    """
    if manifest['maps_sha256'] != protocol['maps_sha256'] or manifest['profile'] != 'sim':
        raise ValueError('build manifest does not match frozen map/profile')
    params = read(params_path)
    if params != protocol['arms']:
        raise ValueError('parameter file differs from frozen arms')
    params_remote = '/tmp/ai-params-' + digest(Path(params_path).read_bytes())[:16]
    expected_arm = protocol['arms']['arms'][0]
    if len(protocol['arms']['arms']) != 1:
        raise ValueError('ingest requires exactly one frozen arm')
    combined = {'maps': protocol['maps'], 'games': [[] for _ in protocol['maps']]}
    next_salt = protocol['salt_start']
    evidence = []
    thread_counts = set()
    for chunk in sorted(chunks, key=lambda c: c['salt_start']):
        if chunk['salt_start'] != next_salt:
            raise ValueError('salt gap, overlap, or duplicate slice')
        done = dict(line.split('=', 1) for line in Path(chunk['done_file']).read_text().splitlines())
        if done.get('rc') != '0':
            raise ValueError('incomplete or failed cloud slice')
        log = Path(chunk['log_file']).read_text()
        binary_key = '/bin/' + manifest['binary_sha256'][:16] + '/hard_vs_human.gz'
        if binary_key not in log:
            raise ValueError('slice used another binary')
        commands = [shlex.split(line[2:]) for line in log.splitlines()
                    if line.startswith('+ env ') and '/tmp/hard_vs_human ' in line]
        if len(commands) != 1:
            raise ValueError('missing or ambiguous simulation command')
        command = commands[0]
        binary_index = command.index('/tmp/hard_vs_human')
        environment = command[1:binary_index]
        if any(v != 'LEGATUM_HVH_RESEED_DICE=1' and not
               (v.startswith('RAYON_NUM_THREADS=') and v.split('=', 1)[1].isdigit())
               for v in environment):
            raise ValueError('unexpected simulation environment')
        thread_counts.update(v.split('=', 1)[1] for v in environment if v.startswith('RAYON_NUM_THREADS='))
        options = {}
        index = binary_index + 1
        allowed = {'--ai-params', '--games', '--seed', '--label', '--out-dir', '--out', '--status-file'}
        while index < len(command):
            flag = command[index]
            if flag in options or (flag != '--arms' and flag not in allowed):
                raise ValueError('unexpected or duplicate simulation option')
            options[flag] = True if flag == '--arms' else command[index + 1]
            index += 1 if flag == '--arms' else 2
        if ('LEGATUM_HVH_RESEED_DICE=1' not in environment or '--arms' not in options
                or options.get('--ai-params') != params_remote
                or int(options.get('--seed', -1)) != chunk['salt_start']
                or int(options.get('--games', -1)) != chunk['salts']):
            raise ValueError('slice command differs from frozen protocol')
        report = read(chunk['report_file'])[expected_arm['label']]
        if (report['salt_start'] != chunk['salt_start'] or report['salts_per_map'] != chunk['salts']
                or report['brain'] != expected_arm['brain'] or report['seat_mode'] != 'all-seats'
                or report.get('rules', []) != []
                or report['candidate_params'] != expected_arm['params']['name']):
            raise ValueError('slice report differs from frozen protocol')
        raw = read(chunk['games_file'])
        if (raw['maps'] != protocol['maps'] or len(raw['games']) != len(protocol['maps'])
                or any(len(row) != chunk['salts'] for row in raw['games'])):
            raise ValueError('incomplete slice or wrong maps')
        if any(type(g['win']) not in (bool, int) or g['win'] not in (0, 1)
               for row in raw['games'] for g in row):
            raise ValueError('invalid win outcome')
        for target, rows in zip(combined['games'], raw['games']):
            target.extend(rows)
        evidence.append(dict(chunk, games_sha256=digest(Path(chunk['games_file']).read_bytes()),
                             log_sha256=digest(Path(chunk['log_file']).read_bytes())))
        next_salt += chunk['salts']
    if len(thread_counts) != 1 or '0' in thread_counts:
        raise ValueError('missing or inconsistent thread count')
    result = dict(manifest, protocol_id=digest(canonical(protocol)), maps=protocol['maps'],
                  salt_start=protocol['salt_start'], salts=next_salt-protocol['salt_start'],
                  wins=[[int(g['win']) for g in row] for row in combined['games']],
                  game_hashes=[[digest(canonical(g)) for g in row] for row in combined['games']],
                  backend='spot', chunks=evidence, rayon_threads=thread_counts.pop())
    validate_records(result, protocol)
    return result, combined


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    g = sub.add_parser('gene-arms', help='write explicit off/control, rivals-only and all-seats arms for default-on genes')
    g.add_argument('--genes', nargs='+', required=True)
    g.add_argument('--out', required=True)
    p = sub.add_parser('prepare')
    p.add_argument('--directory',required=True); p.add_argument('--commit',required=True)
    p.add_argument('--id',required=True); p.add_argument('--role',choices=['control','repeat','candidate'],required=True)
    p.add_argument('--family-size',type=int,default=5)
    p.add_argument('--salt-start',type=int,default=1,help='predeclare a fresh block when freezing a new control')
    w = sub.add_parser('worker')
    w.add_argument('--binary',required=True); w.add_argument('--commit',required=True)
    w.add_argument('--role',choices=['control','repeat','candidate'],required=True)
    a = sub.add_parser('analyze')
    a.add_argument('--protocol',required=True); a.add_argument('--control',required=True); a.add_argument('--candidate',required=True)
    i = sub.add_parser('ingest', help='validate and combine completed spot slices')
    i.add_argument('--protocol', required=True); i.add_argument('--manifest', required=True)
    i.add_argument('--chunks', required=True); i.add_argument('--out-dir', required=True)
    i.add_argument('--params', help='defaults to arms.json beside the protocol')
    args = parser.parse_args()
    if args.command == 'gene-arms':
        write(args.out, gene_arms(args.genes))
    elif args.command == 'prepare':
        if args.family_size < 1: parser.error('family size must be positive')
        if not 1 <= args.salt_start <= 2**64 - 1 - max(LOOKS):
            parser.error('salt start must fit the simulator u64 range including all looks')
        prepare(args)
    elif args.command == 'worker': worker(args)
    elif args.command == 'ingest':
        result, games = ingest(read(args.protocol), read(args.manifest), read(args.chunks),
                               args.params or Path(args.protocol).parent / 'arms.json')
        write(Path(args.out_dir) / 'result.json', result)
        write(Path(args.out_dir) / 'games.json.gz', games)
        print(json.dumps({'games':len(result['maps'])*result['salts'],
                          'wins':sum(map(sum,result['wins'])), 'backend':'spot'}))
    else: print(json.dumps(compare(read(args.control), read(args.candidate), read(args.protocol)), indent=2))


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""W0.4 gate analysis: one bench binary, gene groups as arms, each arm paired with the
frozen control arm (`ctl`) of the same binary, maps and salts.

  python3 tools/ai-quality/w04_gate.py OUT_DIR BATCH_DIR [BATCH_DIR ...]

Each BATCH_DIR is a `tools/cloud/run-batch.sh --bin hard_vs_human` result directory
(slice-N/out/<label>-games.json[.gz]). Slices are joined in salt order per arm. A later
batch (for example the combined arm) must use the same binary and salt range; its `ctl`
arm, if present, must match the first batch's game for game, or the analysis fails.

The paired estimator is `tools/sim/quality_gate.py::compare` (equal-weight map strata, the
larger of the stratified and salt-cluster standard errors). One planned look; the family
correction is Bonferroni over every arm compared (`--family`).

Sign convention. The recorded outcome is always the human seat's win. `delta` and the raw
intervals keep that meaning (candidate minus control seat wins). The decision label uses the
role-normalised AI strength effect, positive = stronger AI:
  seat-only      strength = +(seat delta)   the candidate plays the seat
  opponent-only  strength = -(seat delta)   the candidate plays the rivals; fewer seat wins is stronger
  all-seats      no single sign recovers strength (seat delta = seat gain - rival gain), so the
                 decision is 'role-mixed'; read the matching seat-only and opponent-only arms.
Provenance (commit and binary id) is read from each batch's BATCH_INFO and slice logs, must agree
across slices and batches, and is never a placeholder. A repeated ctl must match game for game.
"""
import argparse
import gzip
import importlib.util
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('quality_gate', ROOT / 'tools/sim/quality_gate.py')
qg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(qg)

ROLE_OF_MODE = {'seat-only': 'seat', 'opponent-only': 'opponents', 'all-seats': 'all-seats'}


def load(path):
    path = Path(path)
    if not path.exists():
        path = path.with_name(path.name + '.gz')
    with (gzip.open(path, 'rt') if path.suffix == '.gz' else path.open()) as f:
        return json.load(f)


def slices(batch):
    out = []
    for s in sorted(Path(batch).glob('slice-*')):
        report = load(s / 'out/hard-vs-human-arms.json')
        out.append((s, report))
    return out


def batch_provenance(batch):
    """(commit, binary id) of a cloud batch, from its own BATCH_INFO and slice logs.

    BATCH_INFO records the source commit (an abbreviated hash); each slice's batch.log records
    the uploaded binary key `bin/<16 hex of the binary's sha256>/`. All slices must agree."""
    batch = Path(batch)
    info = batch / 'BATCH_INFO'
    if not info.exists():
        raise SystemExit(f'{batch}: no BATCH_INFO, provenance unknown')
    m = re.search(r'^commit (\S+)$', info.read_text(), re.M)
    short = m.group(1) if m else ''
    if not re.fullmatch(r'[0-9a-f]{7,40}', short) or set(short) == {'0'}:
        raise SystemExit(f'{batch}: BATCH_INFO commit is missing or a placeholder: {short!r}')
    try:
        commit = subprocess.check_output(['git', 'rev-parse', '--verify', '-q', short + '^{commit}'],
                                         cwd=ROOT, text=True).strip()
    except subprocess.CalledProcessError:
        raise SystemExit(f'{batch}: commit {short} is not in this repository')
    binaries = set()
    slice_dirs = sorted(batch.glob('slice-*'))
    if not slice_dirs:
        raise SystemExit(f'{batch}: no slices')
    for s in slice_dirs:
        log = s / 'batch.log'
        if not log.exists():
            raise SystemExit(f'{s}: no batch.log, binary unknown')
        found = set(re.findall(r'/bin/([0-9a-f]{16})/', log.read_text()))
        if len(found) != 1:
            raise SystemExit(f'{s}: expected one binary id in batch.log, found {sorted(found)}')
        binaries |= found
    if len(binaries) != 1:
        raise SystemExit(f'{batch}: slices ran different binaries {sorted(binaries)}')
    binary = binaries.pop()
    try:
        qg.check_provenance(commit, binary)
    except ValueError as error:
        raise SystemExit(f'{batch}: {error}')
    return commit, binary


def provenance(batches, commit=None, binary=None):
    """One provenance for every batch; optional overrides must agree with the manifests."""
    found = {str(b): batch_provenance(b) for b in batches}
    if len(set(found.values())) != 1:
        raise SystemExit(f'batches differ in commit or binary: {found}')
    c, b = next(iter(found.values()))
    if commit and not c.startswith(commit):
        raise SystemExit(f'--commit {commit} disagrees with the batch manifest {c}')
    if binary and not b.startswith(binary[:16]):
        raise SystemExit(f'--binary {binary} disagrees with the batch manifest {b}')
    return c, b


def arm_games(batches):
    """label -> (salt_start, maps, games per map)"""
    arms = {}
    for batch in batches:
        for s, report in slices(batch):
            for label, r in report.items():
                games = load(s / f'out/{label}-games.json')
                arms.setdefault(label, []).append((r['salt_start'], r['salts_per_map'], games, r))
    result = {}
    for label, parts in arms.items():
        parts.sort(key=lambda p: p[0])
        start = parts[0][0]
        expect = start
        maps = parts[0][2]['maps']
        rows = [[] for _ in maps]
        dup = []
        for part in parts:
            salt_start, n, games, _ = part
            if salt_start != expect:
                # The same arm in two batches (ctl repeated): keep the first copy, check the second.
                assert salt_start < expect, f'{label}: salt gap at {expect}'
                dup.append(part)
                continue
            assert games['maps'] == maps
            for dst, row in zip(rows, games['games']):
                assert len(row) == n
                dst.extend(row)
            expect += n
        result[label] = {'salt_start': start, 'maps': maps, 'games': rows,
                         'seat_mode': parts[0][3]['seat_mode'], 'parts': parts, 'dups': dup}
    return result


def records(arm, protocol, commit, binary):
    return {'protocol_id': qg.digest(qg.canonical(protocol)), 'commit': commit,
            'binary_sha256': binary, 'maps': arm['maps'], 'salts': len(arm['games'][0]),
            'salt_start': arm['salt_start'],
            'wins': [[int(g['win']) for g in row] for row in arm['games']],
            'game_hashes': [[qg.digest(qg.canonical(g)) for g in row] for row in arm['games']]}


def analyze(out_dir, batches, family=15, commit=None, binary=None):
    commit, binary = provenance(batches, commit, binary)
    arms = arm_games(batches)
    ctl = arms['ctl']
    n = len(ctl['games'][0])
    protocol = {'maps': ctl['maps'], 'looks': [n], 'salt_start': ctl['salt_start'], 'alpha': 0.05,
                'family_size': family, 'floor': qg.FLOOR}
    ctl_rec = records(ctl, protocol, commit, binary)
    out = Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    # A ctl arm repeated in a later batch must reproduce the first game for game.
    repeats = []
    for salt_start, n_rep, games, _ in ctl['dups']:
        offset = salt_start - ctl['salt_start']
        same = sum(qg.digest(qg.canonical(g)) == h
                   for row, hrow in zip(games['games'], ctl_rec['game_hashes'])
                   for g, h in zip(row, hrow[offset:offset + n_rep]))
        total = n_rep * len(ctl['maps'])
        repeats.append({'games': total, 'matching_game_hashes': same})
        if same != total:
            raise SystemExit(f'ctl repeat failed: only {same}/{total} games reproduced')
    summary = {'salts_per_map': n, 'games_per_arm': n * len(ctl['maps']), 'salt_start': ctl['salt_start'],
               'family_size': family, 'commit': commit, 'binary_id': binary,
               'control_wins': sum(map(sum, ctl_rec['wins'])), 'control_repeats': repeats, 'arms': {}}
    for label, arm in sorted(arms.items()):
        if label == 'ctl':
            continue
        if len(arm['games'][0]) != n or arm['salt_start'] != ctl['salt_start']:
            raise SystemExit(f'{label}: salt range differs from ctl')
        if arm['seat_mode'] not in ROLE_OF_MODE:
            raise SystemExit(f"{label}: unknown seat mode {arm['seat_mode']!r}")
        res = qg.compare(ctl_rec, records(arm, protocol, commit, binary), protocol,
                         role=ROLE_OF_MODE[arm['seat_mode']])
        res['seat_mode'] = arm['seat_mode']
        summary['arms'][label] = res
        with open(out / f'gate-{label}.json', 'w') as f:
            json.dump(res, f, indent=2)
    with open(out / 'gate-summary.json', 'w') as f:
        json.dump(summary, f, indent=2)
    return summary


def pp(x):
    return '        n/a' if x is None else f'{100*x:+10.2f}'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('out_dir')
    ap.add_argument('batches', nargs='+')
    ap.add_argument('--family', type=int, default=15)
    ap.add_argument('--commit', help='optional; must agree with the batch manifest')
    ap.add_argument('--binary', help='optional; must agree with the batch manifest')
    a = ap.parse_args()
    summary = analyze(a.out_dir, a.batches, a.family, a.commit, a.binary)
    print(f"control {summary['control_wins']}/{summary['games_per_arm']}  repeats {summary['control_repeats']}"
          f"  commit {summary['commit'][:9]}  binary {summary['binary_id']}")
    print('seat d = raw human-seat win delta vs ctl; strength = role-normalised, positive = stronger AI '
          '(opponent-only flips the sign; all-seats has none)')
    print(f"{'arm':16} {'mode':14} {'seat d pp':>10} {'strength pp':>11} {'strength 95% desc':>19} "
          f"{'adjusted':>19} {'decision':>13}")
    for label, r in summary['arms'].items():
        if r['strength_effect'] is None:
            desc = adj = ''
        else:
            lo, hi = r['strength_ci95']
            alo, ahi = r['strength_interval']
            desc = f'[{100*lo:+6.2f},{100*hi:+6.2f}]'
            adj = f'[{100*alo:+6.2f},{100*ahi:+6.2f}]'
        print(f"{label:16} {r['seat_mode']:14} {pp(r['delta'])} {pp(r['strength_effect']):>11} {desc:>19} "
              f"{adj:>19} {r['decision']:>13}")


if __name__ == '__main__':
    main()

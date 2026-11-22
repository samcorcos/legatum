#!/usr/bin/env python3
"""Audit the two-condition hand-off fix run and retain per-game and transition evidence.

python3 tools/league/run_hard_handoff.py --tables old handoff --out /tmp/handoff-fix
python3 tools/league/analyze_hard_handoff_fix.py /tmp/handoff-fix --out evidence.json.gz
python3 tools/league/analyze_hard_handoff_fix.py evidence.json.gz --tables
"""
import argparse
from collections import Counter
import gzip
import hashlib
import json
import math
from pathlib import Path
from statistics import median

from analyze_hard_confirmation import OPENINGS, SUBJECTS, compact, compare, format_rate
from analyze_hard_handoff import leader_pressure

TABLES = ('old', 'handoff')
REASONS = {
    'Rush': {'RushCaptured', 'RushAbandoned'},
    'GreedyExpand': {'CityTarget'},
    'Snowball': {'SnowballReady'},
    'Economic': {'CoreBuild'},
}


def distribution(runs, opening):
    seats = [p for subject in SUBJECTS for g in runs[f'{subject.lower()}-handoff']['games']
             for p in g['players'] if p['id'] != g['subject_seat'] and p['opening'] == opening]
    records = [p['opening_handoff'] for p in seats if p['opening_handoff'] is not None]
    turns = sorted(h['turn'] for h in records)
    return {
        'seats': len(seats), 'handoffs': len(records),
        'untransitioned': len(seats) - len(records),
        'reasons': dict(sorted(Counter(h['reason'] for h in records).items())),
        'turn_histogram': dict(sorted(Counter(turns).items())),
        'min': min(turns) if turns else None, 'median': median(turns) if turns else None,
        'max': max(turns) if turns else None,
        'p10': turns[math.ceil(len(turns) * 0.1) - 1] if turns else None,
        'p90': turns[math.ceil(len(turns) * 0.9) - 1] if turns else None,
    }


def audit(directory):
    source = json.loads((directory / 'source.json').read_text())
    assert source['tables'] == list(TABLES)
    receipts = json.loads((directory / 'runs.json').read_text())
    assert len(receipts) == len(SUBJECTS) * len(TABLES)
    runs = {}
    for receipt in receipts:
        label, table = receipt['label'], receipt['table']
        assert label == f'{receipt["opponent"].lower()}-{table}' and label not in runs
        command = receipt['command']
        assert '--hard-v2' not in command
        assert ('--hard-v2-handoff' in command) == (table == 'handoff')
        assert ('--legacy-hard' in command) == (table == 'old')
        assert receipt['binary_sha256'] == source['binary_sha256']
        assert receipt['binary_hash_basis'] == 'Frozen executable verified before and after execution.'
        paths = list((directory / 'raw' / label).glob('*.json'))
        assert len(paths) == 1
        runs[label] = compact(receipt, paths[0])
        for game in runs[label]['games']:
            for player in game['players']:
                h = player['opening_handoff']
                if h is not None:
                    assert h['reason'] in REASONS[player['opening']] | {'TurnCap'}
    assert set(runs) == {f'{s.lower()}-{t}' for s in SUBJECTS for t in TABLES}
    comparisons = {s: compare(runs[f'{s.lower()}-old'], runs[f'{s.lower()}-handoff']) for s in SUBJECTS}
    reference = runs['greedyexpand-handoff']['games']
    for subject in ['snowball', 'rush']:
        for a, b in zip(reference, runs[f'{subject}-handoff']['games'], strict=True):
            assert [p['faction'] for p in a['players']] == [p['faction'] for p in b['players']]
            assert [p['opening'] for p in a['players'] if p['id'] != a['subject_seat']] == [
                p['opening'] for p in b['players'] if p['id'] != b['subject_seat']]
    members = (directory / 'evolved-members.json').read_text()
    assert hashlib.sha256(members.encode()).hexdigest() == source['members_sha256']
    return {'format': 3, 'source': source, 'members_json': members, 'runs': runs,
            'comparisons': comparisons,
            'handoffs': {o: distribution(runs, o) for o in OPENINGS}}


def tables(data):
    runs = data['runs']
    print('| Challenger | Old Hard | Fixed hand-off | Change (pp) | Old-only / hand-off-only; paired p |')
    print('|---|---|---|---:|---|')
    for subject in SUBJECTS:
        a, b = [runs[f'{subject.lower()}-{t}']['summary']['subject'] for t in TABLES]
        c = data['comparisons'][subject]
        print(f'| {subject} | {format_rate(a)} | {format_rate(b)} | '
              f'{c["delta_percentage_points"]:+.2f} | '
              f'{c["old_only"]} / {c["v2_only"]}; p = {c["exact_two_sided_p"]:.4g} |')
    print('\n| Challenger | Table | Mean turns | Stalls | Wars/game | City captures/game | Settler captures/game | Leader at war / eligible turns |')
    print('|---|---|---:|---:|---:|---:|---:|---|')
    for subject in SUBJECTS:
        for table in TABLES:
            run = runs[f'{subject.lower()}-{table}']
            s = run['summary']
            war, eligible = leader_pressure(run['games'])
            print(f'| {subject} | {table} | {s["mean_turns"]:.2f} | '
                  f'{s["outcomes"].get("stalled", 0)} | {s["mean_war_starts"]:.2f} | '
                  f'{s["mean_city_captures"]:.3f} | {s["mean_settler_captures"]:.3f} | '
                  f'{war:,}/{eligible:,} = {war / eligible:.1%} |')
    print('\n| Opening | Opponent seats | Hand-offs | Turn min / p10 / median / p90 / max | Reasons |')
    print('|---|---:|---:|---|---|')
    for opening in OPENINGS:
        d = data['handoffs'][opening]
        span = ' / '.join(f'{d[k]:g}' for k in ['min', 'p10', 'median', 'p90', 'max']) if d['handoffs'] else '—'
        reasons = ', '.join(f'{k}: {v}' for k, v in d['reasons'].items())
        print(f'| {opening} | {d["seats"]} | {d["handoffs"]} | {span} | '
              f'{reasons or ("Default from start" if opening == "Default" else "Not drawn (weight 0)")} |')
    print('\n| Opening | Turns 2–20 | 21–40 | 41–60 | 61–79 | 80 |')
    print('|---|---:|---:|---:|---:|---:|')
    for opening in OPENINGS:
        d = data['handoffs'][opening]
        if not d['handoffs']:
            continue
        counts = [sum(n for t, n in d['turn_histogram'].items() if lo <= int(t) <= hi)
                  for lo, hi in [(2, 20), (21, 40), (41, 60), (61, 79), (80, 80)]]
        assert sum(counts) == d['handoffs']
        print(f'| {opening} | ' + ' | '.join(str(n) for n in counts) + ' |')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--tables', action='store_true')
    args = parser.parse_args()
    if args.tables:
        with gzip.open(args.input, 'rt') as handle:
            data = json.load(handle)
    else:
        if args.out is None:
            parser.error('--out is required for auditing raw results')
        data = audit(args.input)
        args.out.write_bytes(gzip.compress((json.dumps(data, sort_keys=True) + '\n').encode(), mtime=0))
    tables(data)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""Summarise army_measure/army_churn outputs and the hash-verified historical game_dump.
Usage: army-measure-report.py REPLAY_JSONL CALIBRATION_JSON BEFORE_JSON AFTER_JSON
Outputs companion evidence beside docs/sim-results/ai-quality/strongest-army-1025.md.
"""
import csv
import gzip
import json
import math
from pathlib import Path
import statistics
import sys

out = Path('docs/sim-results/ai-quality')
rules = json.loads(Path('app/src/rules.generated.json').read_text())
replay, calibration, before, after = map(Path, sys.argv[1:])


def value(base, hp, level, missing=False):
    effective = base * (100 + rules['promotion']['strengthPercent'] * level) // 100
    if missing:
        effective -= base * rules['supply']['unsuppliedStrengthPercent'] // 100
    full = effective * effective // rules['armyStrengthReference']
    return effective, full, full * hp // 100


def gz(name, data):
    with (out / name).open('wb') as target:
        with gzip.GzipFile(fileobj=target, mode='wb', mtime=0) as stream:
            stream.write(json.dumps(data, separators=(',', ':')).encode())


units, turns = [], []
for line in replay.read_text().splitlines():
    d = json.loads(line)
    if not 185 <= d['turn'] <= 199:
        continue
    totals = []
    for p, faction in [(0, 'Sparta'), (3, 'Athens')]:
        rows = []
        player = next(pv for pv in d['state']['players'] if pv['id'] == p)
        assert not any(r.get('idle') for r in player.get('supply', [])), 'include historical shortages'
        for u in sorted(d['state']['units'].values(), key=lambda u: u['id']):
            if u['owner'] != p:
                continue
            stat = rules['unitStats'][u['kind']]
            thresholds = rules['promotion']['xp'] + ([rules['promotion']['ironGuardXp']] if p == 0 else [])
            level = sum(u.get('xp', 0) >= n for n in thresholds)
            effective, full, new = value(max(stat['strength'], stat['ranged']), u['hp'], level)
            old = max(stat['strength'], stat['ranged']) * u['hp'] // 100
            if not stat['military']:
                old = new = full = effective = 0
            row = dict(turn=d['turn'], empire=faction, unit=u['id'], kind=u['kind'], name=rules['unitNames'][u['kind']],
                       melee=stat['strength'], ranged=stat['ranged'], hp=u['hp'], xp=u.get('xp', 0), level=level,
                       old=old, effective=effective, full_health=full, new=new, military=stat['military'])
            units.append(row)
            rows.append(row)
        old_total = sum(r['old'] for r in rows)
        recorded = dict(next(m['values'] for m in d['metrics'] if m['title'] == 'StrongestArmy'))[p]
        assert old_total == recorded, (d['turn'], p, old_total, recorded)
        totals.append(dict(empire=faction, count=sum(r['military'] for r in rows), old=old_total, new=sum(r['new'] for r in rows)))
    turns.append(dict(turn=d['turn'], scores=totals, holder=d['state']['titles']['StrongestArmy']))
with (out / 'strongest-army-1025-units.csv').open('w') as f:
    writer = csv.DictWriter(f, fieldnames=units[0].keys(), lineterminator='\n')
    writer.writeheader()
    writer.writerows(units)
cal = json.loads(calibration.read_text())
gz('strongest-army-1025-calibration.json.gz', cal)
cal_summary = []
for category in ['all', 'same_era', 'era_gap', 'mixed', 'wounded', 'saved']:
    for split in ['fit', 'held_out']:
        eligible = [r for r in cal['rows'] if (category == 'all' or r['category'] == category) and r['split'] == split]
        decisive = [r for r in eligible if max(r['wins'][:2]) >= cal['repeats'] * 2 * .75]
        correct = []
        for i, _ in enumerate(cal['candidates']):
            correct.append(sum((r['scores'][0][i] > r['scores'][1][i]) if r['wins'][0] > r['wins'][1]
                               else (r['scores'][1][i] > r['scores'][0][i]) for r in decisive))
        cal_summary.append(dict(category=category, split=split, cells=len(eligible), decisive=len(decisive), correct=correct))


def churn(games):
    rows = []
    for g in games:
        holders = [None, None]
        changes, late = [0, 0], [0, 0]
        for t in g['turns']:
            values = [{p: 0 for p in t['alive']} for _ in range(2)]
            for p, strength, ranged, hp, level, missing in t['units']:
                if p not in values[0]:
                    continue
                base = max(strength, ranged)
                values[0][p] += base * hp // 100
                values[1][p] += value(base, hp, level, missing)[2]
            for k in range(2):
                top = max(values[k].values(), default=0)
                leaders = [p for p, v in values[k].items() if v == top]
                holder = (leaders[0] if len(leaders) == 1 else holders[k] if holders[k] in leaders else None) if top > 0 else None
                if holder is not None and holders[k] is not None and holder != holders[k]:
                    changes[k] += 1
                    late[k] += t['turn'] >= 150
                holders[k] = holder
        rows.append(dict(seed=g['seed'], turns=len(g['turns']), late_turns=sum(t['turn'] >= 150 for t in g['turns']),
                         actual=g['changes'], actual_late=g['changes_late'], shadow=changes, shadow_late=late))
    return rows


raw_before, raw_after = json.loads(before.read_text()), json.loads(after.read_text())
base, changed = churn(raw_before), churn(raw_after)
assert all(r['actual'] == r['shadow'][0] for r in base)
assert all(r['actual'] == r['shadow'][1] for r in changed)
assert all(r['actual_late'] == r['shadow_late'][0] for r in base)
assert all(r['actual_late'] == r['shadow_late'][1] for r in changed)
assert [r['seed'] for r in base] == [r['seed'] for r in changed], 'paired seeds must match'
gz('strongest-army-1025-churn.json.gz', dict(before=raw_before, after=raw_after))
delta = [b['actual'] - a['actual'] for a, b in zip(base, changed)]
late_delta = [b['actual_late'] - a['actual_late'] for a, b in zip(base, changed)]
summary = dict(turns=turns, calibration=cal_summary, candidates=cal['candidates'],
               battles=len(cal['rows']) * cal['repeats'] * 2, unfinished=sum(r['unfinished'] for r in cal['rows']),
               invalid_orders=sum(r['invalid_orders'] for r in cal['rows']), before=base, after=changed,
               paired_mean_delta=statistics.mean(delta), paired_delta_95=1.96 * statistics.stdev(delta) / math.sqrt(len(delta)),
               paired_late_mean_delta=statistics.mean(late_delta), paired_late_delta_95=1.96 * statistics.stdev(late_delta) / math.sqrt(len(late_delta)))
(out / 'strongest-army-1025-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps({k:summary[k] for k in ['battles','unfinished','invalid_orders','paired_mean_delta','paired_delta_95','paired_late_mean_delta','paired_late_delta_95']}))
for name, cohort in [('before', base), ('after', changed)]:
    print(name, {k:sum(r[k] for r in cohort) for k in ['turns','late_turns','actual','actual_late']})

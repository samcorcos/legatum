#!/usr/bin/env python3
"""Summarize the fixed public-seed comparison; requires Python 3 and NumPy.

Usage: analyze-full-games.py RAW_JSONL OUTPUT_DIRECTORY
Resampling unit is a paired seed, never an assault. No policy tuning occurs here.
"""
import hashlib
import json
from pathlib import Path
import sys
import numpy as np

source, destination = Path(sys.argv[1]), Path(sys.argv[2])
rows = [json.loads(line) for line in source.read_text().splitlines() if line]
keys = [(row['arm'], row['seed']) for row in rows]
assert len(keys) == len(set(keys)), 'duplicate arm/seed'
expected = {(arm, seed) for arm in ('baseline', 'candidate') for seed in range(7101, 7401)}
assert set(keys) == expected, 'comparison incomplete or unexpected seeds/arms'
by_key = dict(zip(keys, rows))
cohorts = ('visible_within_two', 'visible_longer', 'unseen_at_start', 'left_censored')
outcomes = ('lost_to_attacker', 'other_ownership_end', 'peace', 'right_censored')
fields = ('arm', 'seed', 'seat', 'faction', 'finished', 'won', 'winners', 'end_turn',
          'alive', 'lost', 'captured', 'capital_lost', 'cities_end', 'vp', 'vp_share',
          'mismatches', 'late_city_assaults')
compact = [{k: row[k] for k in fields} for row in sorted(rows, key=lambda r: (r['seed'], r['arm']))]
for seed in range(7101, 7401):
    a, b = (by_key[(arm, seed)] for arm in ('baseline', 'candidate'))
    assert (a['seat'], a['faction']) == (b['seat'], b['faction'])
    assert a['late_city_assaults']['version'] == b['late_city_assaults']['version'] == 1
    assert a['mismatches'] == b['mismatches'] == 0

def episodes(row, cohort=None):
    return [e for e in row['late_city_assaults']['episodes']
            if cohort is None or e['cohort'] == cohort]

def measurements(row):
    ep = episodes(row)
    result = {'all_city_losses': row['lost'], 'late_assaults': len(ep),
              'late_unattributed_losses': len(row['late_city_assaults']['losses_without_assault']),
              'right_censored': sum(e['outcome'] == 'right_censored' for e in ep),
              'left_censored': sum(e['left_censored'] for e in ep)}
    for cohort in cohorts:
        cohort_ep = episodes(row, cohort)
        result[cohort + '_opportunities'] = len(cohort_ep)
        result[cohort + '_losses'] = sum(e['outcome'] == 'lost_to_attacker' for e in cohort_ep)
    return result

rng = np.random.default_rng(366)
resamples = rng.integers(0, 300, size=(20000, 300))
metrics = {}
for key in measurements(rows[0]):
    arms = [np.array([measurements(by_key[(arm, s)])[key] for s in range(7101, 7401)])
            for arm in ('baseline', 'candidate')]
    difference = arms[1] - arms[0]
    ci = np.quantile(difference[resamples].mean(axis=1), [0.025, 0.975])
    metrics[key] = {'baseline_total': int(arms[0].sum()), 'candidate_total': int(arms[1].sum()),
                    'baseline_mean': float(arms[0].mean()), 'candidate_mean': float(arms[1].mean()),
                    'candidate_minus_baseline_mean': float(difference.mean()),
                    'paired_bootstrap_95_percentile_interval': ci.tolist()}
completed_pairs = [s for s in range(7101, 7401)
                   if all(by_key[(arm, s)]['finished'] for arm in ('baseline', 'candidate'))]
wins = [np.array([int(by_key[(arm, s)]['won']) for s in completed_pairs])
        for arm in ('baseline', 'candidate')]
win_summary = {'completed_pairs': len(completed_pairs), 'excluded_unfinished_pairs': 300 - len(completed_pairs),
               'estimand': 'win difference conditional on both paired games finishing'}
if completed_pairs:
    win_indices = rng.integers(0, len(completed_pairs), size=(20000, len(completed_pairs)))
    diff = wins[1] - wins[0]
    win_summary.update({'baseline_wins': int(wins[0].sum()), 'candidate_wins': int(wins[1].sum()),
                        'candidate_minus_baseline_percentage_points': float(diff.mean() * 100),
                        'paired_bootstrap_95_percentile_interval_pp':
                        (np.quantile(diff[win_indices].mean(axis=1), [0.025, 0.975]) * 100).tolist(),
                        'baseline_only_wins': int(((wins[0] == 1) & (wins[1] == 0)).sum()),
                        'candidate_only_wins': int(((wins[0] == 0) & (wins[1] == 1)).sum())})
cohort_fractions = {}
for cohort in cohorts:
    counts = [np.array([[len(episodes(by_key[(arm, seed)], cohort)),
                         sum(e['outcome'] == 'lost_to_attacker'
                             for e in episodes(by_key[(arm, seed)], cohort))]
                        for seed in range(7101, 7401)]) for arm in ('baseline', 'candidate')]
    totals = [a.sum(axis=0) for a in counts]
    sampled = [a[resamples].sum(axis=1) for a in counts]
    valid = (sampled[0][:, 0] > 0) & (sampled[1][:, 0] > 0)
    differences = (sampled[1][valid, 1] / sampled[1][valid, 0]
                   - sampled[0][valid, 1] / sampled[0][valid, 0])
    cohort_fractions[cohort] = {
        'definition': 'observed attributed losses / all cohort opportunities; not eventual loss probability',
        'baseline': float(totals[0][1] / totals[0][0]) if totals[0][0] else None,
        'candidate': float(totals[1][1] / totals[1][0]) if totals[1][0] else None,
        'paired_bootstrap_difference_95_percentile_interval':
            np.quantile(differences, [0.025, 0.975]).tolist() if len(differences) else None,
        'valid_resamples': int(valid.sum()), 'zero_denominator_resamples': int((~valid).sum())}
summary = {'source_revision': '1f7db8eed96d513a7b1acc70b8ad72c27458a398',
           'binary_sha256': 'e9de9391bb09260a25227752336c358e9818b464f6395f064551640cd0d0ed91',
           'raw_jsonl_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
           'numpy_version': np.__version__, 'cohort_observed_loss_fractions': cohort_fractions,
           'pairs': 300, 'seeds': [7101, 7400], 'bootstrap_resamples': 20000,
           'bootstrap_seed': 366, 'safety_cap': 700, 'win_share': win_summary,
           'per_game_metrics': metrics, 'arms': {}}
for arm in ('baseline', 'candidate'):
    games = [r for r in rows if r['arm'] == arm]
    summary['arms'][arm] = {'finished': sum(r['finished'] for r in games),
        'unfinished_seeds': [r['seed'] for r in games if not r['finished']],
        'mean_end_turn': sum(r['end_turn'] for r in games) / len(games),
        'cohorts': {cohort: {outcome: sum(e['outcome'] == outcome for r in games
                                        for e in episodes(r, cohort)) for outcome in outcomes}
                    for cohort in cohorts}}
destination.mkdir(parents=True, exist_ok=True)
(destination / 'full-games-7101-7400.jsonl').write_text(''.join(json.dumps(r, separators=(',', ':')) + '\n' for r in compact))
(destination / 'full-games-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary, indent=2))

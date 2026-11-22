#!/usr/bin/env python3
"""Audit paired counter experiments; stdlib only. Labels are LABEL=REPORT[.gz].

Each treatment must contain the same (index, seed, faction, assigned policy) rows
as the first run. Bootstrap units are whole mirrored maps, not seats or games.
Print JSON plus an optional Markdown appendix. Never changes source reports.
"""
import argparse
from collections import Counter, defaultdict
import gzip
import hashlib
import json
import math
from pathlib import Path
import random
from statistics import mean, median


POLICIES = ['rush', 'wide', 'science', 'turtle', 'economic']


def load_bundle(spec):
    reports = []
    sources = []
    for filename in spec.split(','):
        path = Path(filename)
        raw = path.read_bytes()
        report = json.loads(gzip.decompress(raw) if path.suffix == '.gz' else raw)
        if not report['meta']['strategy_matchups'] or report['meta']['games'] % 30:
            raise ValueError('requires complete tournament blocks')
        # Audit every raw shard against Rust, retaining its censored denominators.
        for (a, b), groups in score_groups(report).items():
            values = [v for group in groups.values() for v in group]
            cell = report['summary']['diversity']['assigned']['matchups'][a][b]
            if cell['games'] != len(values) or abs(mean(values) - cell['score']) > 1e-12:
                raise ValueError(f'{filename}: matrix does not reconcile')
        sources.append({'file': filename, 'sha256': hashlib.sha256(raw).hexdigest(), 'meta': report['meta']})
        reports.append(report)
    merged = {'meta': reports[0]['meta'], 'sources': sources,
              'games': [g for report in reports for g in report['games']]}
    seeds = defaultdict(list)
    for i, g in enumerate(merged['games']):
        g['index'] = i  # Local indices restart in each shard; seeds do not.
        seeds[g['seed']].append(g)
    for seed, games in seeds.items():
        if len(games) != 2:
            raise ValueError(f'{seed}: expected exactly two orientations')
        x, y = games
        counts = Counter(p['strategy'] for p in x['players'])
        if len(counts) != 2 or set(counts.values()) != {4} or Counter(p['strategy'] for p in y['players']) != counts:
            raise ValueError(f'{seed}: expected the same two policies with four seats each')
        if any(a['strategy'] == b['strategy'] or a['faction'] != b['faction']
               for a, b in zip(x['players'], y['players'])):
            raise ValueError(f'{seed}: not mirrored with fixed factions')
    return merged


def finished(game):
    return game['outcome'] not in ('stalled', 'panic') and game['panic'] is None


def entropy(counts):
    total = sum(counts)
    return -sum(n / total * math.log(n / total) for n in counts if n) / math.log(len(counts)) if total else None


def identity(report):
    return [(g['index'], g['seed'], [(p['id'], p['team'], p['faction'], p['strategy'])
             for p in g['players']]) for g in report['games']]


def score_groups(report):
    blocks = defaultdict(dict)
    for g in report['games']:
        if not finished(g):
            continue
        eliminated = {e['player']: e['turn'] for e in g['eliminations']}
        def rank(p):
            return (p['team'] == g['winner_team'], p['alive'],
                    p['vp'] if p['alive'] else eliminated.get(p['id'], 0))
        policies = sorted({p['strategy'] for p in g['players']})
        a, b = policies
        pairs = [(x, y) for x in g['players'] if x['strategy'] == a
                 for y in g['players'] if y['strategy'] == b and x['team'] != y['team']]
        value = mean(1 if rank(x) > rank(y) else .5 if rank(x) == rank(y) else 0 for x, y in pairs)
        blocks[(a, b)].setdefault(g['seed'], []).append(value)
    return blocks


def scores(report):
    return {pair: {seed: mean(v) for seed, v in maps.items()} for pair, maps in score_groups(report).items()}


def interval(values, rng, replicates):
    draws = sorted(mean(rng.choices(values, k=len(values))) for _ in range(replicates))
    return [draws[int(.025 * (replicates - 1))], draws[int(.975 * (replicates - 1))]]


def summarize(report, replicates):
    rng = random.Random(972001)
    matrix = {p: {} for p in POLICIES}
    pairs = []
    for (a, b), groups in sorted(scores(report).items()):
        values = [groups[key] for key in sorted(groups, key=int)]
        if len(values) < 16:
            raise ValueError(f'{a}/{b}: fewer than 32 complete mirrored games; add blocks to ALL runs')
        lo, hi = interval(values, rng, replicates)
        score = mean(values)
        matrix[a][b], matrix[b][a] = score, 1 - score
        pairs.append({'row': a, 'column': b, 'games': 2 * len(values), 'seed_pairs': len(values),
                      'score': score, 'bootstrap_95_low': lo, 'bootstrap_95_high': hi})
    if len(pairs) != 15:
        raise ValueError('missing policy pairs')
    audit = {'method': 'Percentile bootstrap, whole mirrored maps, descriptive 95% intervals, no multiplicity correction',
             'replicates': replicates, 'rng_seed': 972001, 'pairs': pairs}
    assigned_wins = defaultdict(int)
    observed_wins = defaultdict(int)
    factions = defaultdict(lambda: {'wins': 0, 'seats': 0})
    rush = defaultdict(list)
    outcomes = defaultdict(int)
    policy_seats = defaultdict(list)
    diplomacy = Counter()
    for g in report['games']:
        outcomes[g['outcome']] += 1
        diplomacy.update({k: v for k, v in (g['telemetry'].get('diplomacy') or {}).items()
                          if isinstance(v, int)})
        for p in g['players']:
            policy_seats[p['strategy']].append({
                'win': p['team'] == g['winner_team'],
                'realized': p['archetype'] == p['strategy'],
                'cities': p['cities'],
                'peak_cities': g['telemetry'].get('peak_cities', {}).get(str(p['id']), p['cities']),
                'alive': p['alive'],
            })
            f = factions[p['faction']]
            f['seats'] += 1
            f['wins'] += p['team'] == g['winner_team']
            if p['archetype'] not in POLICIES + ['generalist']:
                raise ValueError('requires classifier v2 observations with inactive Diplomatic')
            if p['team'] == g['winner_team']:
                assigned_wins[p['strategy']] += 1
                observed_wins[p['archetype']] += 1
            if p['strategy'] == 'rush':
                evidence = g['telemetry']['strategy']['empires'][str(p['id'])]
                rush['early_attacks'].append(evidence['early_attacks'])
                rush['realized'].append(p['archetype'] == 'rush')
                rush['early_military_cost'].append(evidence['early_military_cost'])
                rush['early_founded'].append(evidence['early_founded'])
                investments = g['telemetry'].get('investments', {}).get(str(p['id']), {})
                if 'city_captures' in investments:
                    rush['captures'].append(investments['city_captures'])
    if not sum(assigned_wins.values()):
        raise ValueError('no winning seats; counter evidence is insufficient')
    for f in factions.values():
        f['win_rate'] = f['wins'] / f['seats']
    rates = [f['win_rate'] for f in factions.values()]
    clear = defaultdict(list)
    for pair in audit['pairs']:
        if pair['bootstrap_95_low'] > .5:
            clear[pair['column']].append(pair['row'])
        if pair['bootstrap_95_high'] < .5:
            clear[pair['row']].append(pair['column'])
    return {
        'games': len(report['games']), 'sources': report['sources'],
        'scheduled_games': len(report['scheduled_games']),
        'censored_games': [g['index'] for g in report['scheduled_games'] if not finished(g)],
        'excluded_maps': sorted({g['seed'] for g in report['scheduled_games']} - {g['seed'] for g in report['games']}, key=int),
        'setup': report['meta']['first_setup'], 'outcomes': dict(outcomes),
        'assigned_entropy': entropy([assigned_wins[p] for p in POLICIES]),
        'assigned_wins': dict(assigned_wins),
        'observed_entropy': entropy([observed_wins[p] for p in POLICIES + ['generalist']]),
        'policy_outcomes': {
            p: {'seats': len(seats), 'wins': sum(s['win'] for s in seats),
                'realized_wins': sum(s['win'] and s['realized'] for s in seats),
                'win_rate': mean(s['win'] for s in seats),
                'survival': mean(s['alive'] for s in seats),
                'realized': mean(s['realized'] for s in seats),
                'mean_final_cities': mean(s['cities'] for s in seats),
                'mean_peak_cities': mean(s['peak_cities'] for s in seats)}
            for p, seats in sorted(policy_seats.items())},
        'diplomacy': dict(diplomacy),
        'point_dominant': [p for p in POLICIES if all(v > .5 for v in matrix[p].values())],
        'audit': audit, 'matrix': matrix,
        'clear_counters': {p: clear[p] for p in POLICIES},
        'factions': dict(sorted(factions.items())),
        'faction_max_min': max(rates) / min(rates) if min(rates) else None,
        'turns': {'mean': mean(g['turns'] for g in report['games']),
                  'median': median(g['turns'] for g in report['games']),
                  'min': min(g['turns'] for g in report['games']),
                  'max': max(g['turns'] for g in report['games'])},
        'rush': {k: {'n': len(v), 'mean': mean(v), 'median': median(v)} for k, v in rush.items()},
        'captures': sum(g['city_captures'] for g in report['games']),
    }


def markdown(results):
    lines = ['## Generated paired results', '',
             'Cells are row half-tie finish scores (%), not duel win rates. Cells have at least 32 games / 16 complete mirrored maps. '
             'Clear counters require a descriptive map-bootstrap 95% interval excluding 50%; no multiplicity correction. '
             'The JSON contains all intervals and paired deltas.', '']
    for label, r in results.items():
        lines += [f'### {label}', '',
                  f"Scheduled {r['scheduled_games']}; eligible {r['games']}; censored {len(r['censored_games'])}; common excluded maps {len(r['excluded_maps'])}. Assigned winner entropy **{r['assigned_entropy']:.4f}**; observed **{r['observed_entropy']:.4f}**. "
                  f"Length mean **{r['turns']['mean']:.1f}**, median **{r['turns']['median']}**, "
                  f"range {r['turns']['min']}–{r['turns']['max']} turns. Captures: {r['captures']}.", '',
                  '| Row / opponent | ' + ' | '.join(POLICIES) + ' |',
                  '| --- | ' + ' | '.join(['---:'] * 6) + ' |']
        for a in POLICIES:
            lines.append('| ' + a + ' | ' + ' | '.join('—' if a == b else f"{100*r['matrix'][a][b]:.1f}" for b in POLICIES) + ' |')
        lines += ['', f"Point-estimate dominant policies: {', '.join(r['point_dominant']) or 'none'}. "
                  f"Every policy has a clear counter: {'yes' if all(r['clear_counters'].values()) else 'no'}.", '',
                  '| Assigned policy | Wins / seats | Seat win rate | Mean final / peak cities | Observed matching policy |',
                  '| --- | ---: | ---: | ---: | ---: |']
        for policy in POLICIES:
            v = r['policy_outcomes'][policy]
            lines.append(f"| {policy} | {v['wins']} / {v['seats']} | {100*v['win_rate']:.2f}% | "
                         f"{v['mean_final_cities']:.2f} / {v['mean_peak_cities']:.2f} | {100*v['realized']:.1f}% |")
        lines += ['', '| Faction | Wins / seats | Win rate |', '| --- | ---: | ---: |']
        for name, f in r['factions'].items():
            lines.append(f"| {name} | {f['wins']} / {f['seats']} | {100*f['win_rate']:.2f}% |")
        ratio = 'infinite (zero-win faction)' if r['faction_max_min'] is None else f"{r['faction_max_min']:.3f}×"
        lines += ['', f'Faction max/min: **{ratio}** (target ≤1.5×).', '',
                  '| Policy | Clear counters in this sample |', '| --- | --- |']
        for p, counters in r['clear_counters'].items():
            lines.append(f"| {p} | {', '.join(counters) or 'None demonstrated'} |")
        if 'paired_deltas' in r:
            lines += ['', 'Paired score change versus the experiment control (percentage points):', '',
                      '| Row / opponent | Change | Paired 95% interval |', '| --- | ---: | ---: |']
            for d in r['paired_deltas']:
                lo, hi = d['bootstrap_95']
                lines.append(f"| {d['row']} / {d['column']} | {100*d['delta']:+.1f} | [{100*lo:+.1f}, {100*hi:+.1f}] |")
        lines.append('')
    return '\n'.join(lines)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('runs', nargs='+', help='LABEL=path.json[.gz]; first run is control')
    p.add_argument('--replicates', type=int, default=10000)
    p.add_argument('--markdown', type=Path)
    args = p.parse_args()
    if args.replicates < 100:
        p.error('use at least 100 bootstrap replicates')
    reports = {}
    for arg in args.runs:
        label, spec = arg.split('=', 1)
        if label in reports:
            raise ValueError(f'duplicate label {label}')
        reports[label] = load_bundle(spec)
    control = identity(next(iter(reports.values())))
    good_seeds = None
    reference_setup = None
    for label, report in reports.items():
        setup = dict(report['meta']['first_setup'])
        setup.pop('rule_experiments', None)
        if reference_setup is None:
            reference_setup = setup
        if identity(report) != control or setup != reference_setup:
            raise ValueError(f'{label}: maps, orientations, factions, policies or setup differ')
        bad = {g['seed'] for g in report['games'] if not finished(g)}
        good = {g['seed'] for g in report['games']} - bad
        good_seeds = good if good_seeds is None else good_seeds & good
    results = {}
    reference_scores = None
    for label, report in reports.items():
        report['scheduled_games'] = report['games']
        report['games'] = [g for g in report['games'] if g['seed'] in good_seeds]
        r = summarize(report, args.replicates)
        if reference_scores is None:
            reference_scores = scores(report)
        else:
            rng = random.Random(972001)
            deltas = []
            for (a, b), groups in sorted(scores(report).items()):
                values = [groups[key] - reference_scores[a, b][key] for key in sorted(groups, key=int)]
                deltas.append({'row': a, 'column': b, 'delta': mean(values),
                               'bootstrap_95': interval(values, rng, args.replicates)})
            r['paired_deltas'] = deltas
        # Worst-case score bounds if every excluded map goes wholly either way.
        all_pairs = defaultdict(set)
        for g in report['scheduled_games']:
            pair = tuple(sorted({p['strategy'] for p in g['players']}))
            all_pairs[pair].add(g['seed'])
        r['censor_bounds'] = []
        for pair, groups in scores(report).items():
            n = len(all_pairs[pair])
            missing = n - len(groups)
            r['censor_bounds'].append({'row': pair[0], 'column': pair[1], 'excluded_maps': missing,
                'score_bounds': [sum(groups.values()) / n, (sum(groups.values()) + missing) / n]})
        results[label] = r
    print(json.dumps(results, indent=2, allow_nan=False))
    if args.markdown:
        args.markdown.write_text(markdown(results))


if __name__ == '__main__':
    main()

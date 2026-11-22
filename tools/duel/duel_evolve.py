#!/usr/bin/env python3
"""CMA-ES coevolution against current openings and a persistent hall of fame."""
from __future__ import annotations

import argparse
import copy
import json
import math
from pathlib import Path
import random
import signal
import subprocess
import statistics
import sys

if __package__ in (None, ''):
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from tools.duel import duel as d
from tools.evolve.cmaes import CMAES

ALPHA = 0.01
IDENTITY_MIN_WIN_DELTA = -0.02
# Held-out gate rates are noisy (60-120 seat-games); a strict no-regression rule vetoed
# large, significant win gains for openings near their gate. Allow this much slack.
IDENTITY_TOLERANCE = 0.05


def seed_for(domain, generation, seed):
    if domain not in (1, 2, 3) or not 0 <= generation < 2**30 or not 0 <= seed < 2**31:
        raise ValueError('seed must be in [0,2^31), generation in [0,2^30)')
    return (domain << 62) + (generation << 32) + seed


def sign_flip_p(differences, *, alternative='two-sided'):
    """Exact paired map randomization test, including quarter-integer differences."""
    if alternative not in ('two-sided', 'greater'):
        raise ValueError('unknown sign-flip alternative')
    steps = [round(4 * abs(x)) for x in differences if x]
    if any(abs(4 * abs(x) - round(4 * abs(x))) > 1e-9 for x in differences):
        raise ValueError('expected mirrored binary game scores')
    observed = round(4 * sum(differences))
    distribution = {0: 1}
    for step in steps:
        updated = {}
        for total, count in distribution.items():
            for sign in (-1, 1):
                key = total + sign * step
                updated[key] = updated.get(key, 0) + count
        distribution = updated
    return sum(count for total, count in distribution.items()
               if (total >= observed if alternative == 'greater' else abs(total) >= abs(observed))) / 2**len(steps)


def validate_holdout(candidate, incumbent):
    def keyed(games):
        result = {(g['seed'], g.get('mirrored', False)): g for g in games}
        if len(result) != len(games):
            raise ValueError('duplicate held-out assignments')
        return result
    after, before = keyed(candidate), keyed(incumbent)
    if after.keys() != before.keys():
        raise ValueError('held-out maps and orientations must match')
    for key in after:
        a, b = after[key], before[key]
        if (a['duel_members'] != b['duel_members'] or a['genome_hashes'][1] != b['genome_hashes'][1]
                or [(p['id'], p['faction']) for p in a['players']] != [(p['id'], p['faction']) for p in b['players']]):
            raise ValueError('held-out opponent/faction assignments must match')


def compare_holdout(candidate, incumbent, min_delta=0.0025):
    validate_holdout(candidate, incumbent)
    a_scores, _ = d.map_scores(candidate)
    b_scores, _ = d.map_scores(incumbent)
    valid = sorted(a_scores.keys() & b_scores.keys())
    differences = [a_scores[seed] - b_scores[seed] for seed in valid]
    count = len(differences)
    delta = statistics.mean(differences) if count else None
    se = statistics.stdev(differences) / math.sqrt(count) if count > 1 else None
    z = statistics.NormalDist().inv_cdf(1 - ALPHA / 2)
    ci = [delta - z * se, delta + z * se] if se is not None else [None, None]
    p_value = sign_flip_p(differences) if count else 1.0
    candidate_stalls = sum(g['outcome'] == 'stalled' for g in candidate)
    incumbent_stalls = sum(g['outcome'] == 'stalled' for g in incumbent)
    # No promotion can trade apparent wins on the retained maps for extra stalls.
    passed = (count >= 8 and delta > min_delta and p_value < ALPHA
              and candidate_stalls <= incumbent_stalls)
    return {'passed': passed, 'delta': delta, 'ci99_normal_approximation': ci, 'p_value': p_value,
            'alpha': ALPHA, 'independent_maps': count,
            'stalled_maps_dropped': len({g['seed'] for g in candidate}) - count,
            'candidate_stalls': candidate_stalls, 'incumbent_stalls': incumbent_stalls,
            'min_delta': min_delta, 'differences': differences,
            'method': 'two-sided exact paired map sign-flip test; fixed per-test alpha, not family-wise across generations'}


def compare_identity(candidate, incumbent, opening, rules):
    """Compare proven gate passes on held-out games, keeping mirrors paired by map."""
    validate_holdout(candidate, incumbent)
    after = d.identity.report(candidate, [opening], rules)['openings'][opening]
    before = d.identity.report(incumbent, [opening], rules)['openings'][opening]

    def pass_counts(games):
        # Validate complete mirror pairs, but retain stalls in the gate denominator.
        d.map_scores(games)
        counts = {}
        for game in games:
            seat = next(p for p in game['players'] if p['opening'] == opening)
            passed = d.identity.seat_gate(game, seat, rules)['passed'] is True
            counts[game['seed']] = counts.get(game['seed'], 0) + int(passed)
        return counts

    a_counts, b_counts = pass_counts(candidate), pass_counts(incumbent)
    differences = [a_counts[seed] - b_counts[seed] for seed in sorted(a_counts)]
    p_value = sign_flip_p(differences, alternative='greater')
    a_rate, b_rate = after['pass_rate'], before['pass_rate']
    # An incumbent at or near its gate keeps a fixed floor just below it, so repeated
    # promotions cannot ratchet identity away; a failing incumbent gets no slack.
    floor = rules['required_share'] - IDENTITY_TOLERANCE
    threshold = None if b_rate is None else floor if b_rate >= floor - 1e-9 else b_rate
    known = a_rate is not None and b_rate is not None
    improved = (known and b_rate < rules['required_share'] and a_rate > b_rate
                and len(differences) >= 8 and p_value < ALPHA)
    return {'candidate_identity': after, 'incumbent_identity': before,
            'identity_min_pass_rate': threshold,
            'identity_no_regression': known and a_rate >= threshold - 1e-9,
            'identity_improvement': {'passed': improved, 'p_value': p_value, 'alpha': ALPHA,
                'delta': a_rate - b_rate if known else None, 'independent_maps': len(differences),
                'pass_count_differences': differences, 'min_win_delta': IDENTITY_MIN_WIN_DELTA,
                'method': 'one-sided exact paired map sign-flip test of gate pass counts; '
                          'mirrors kept together, stalls and unknowns retained; fixed per-test alpha'}}


def promotion_reason(comparison, identity_gates):
    if identity_gates and not comparison['identity_no_regression']:
        return None
    if comparison['passed']:
        return 'wins'
    if (identity_gates and comparison['identity_improvement']['passed']
            and comparison['independent_maps'] >= 8 and comparison['delta'] is not None
            and comparison['delta'] >= IDENTITY_MIN_WIN_DELTA
            and comparison['candidate_stalls'] <= comparison['incumbent_stalls']):
        return 'identity'
    return None


def pfsp_weights(target, active, hall, matrix):
    """Weight each distinct opponent version using the last committed matrix."""
    if matrix is None:
        raise ValueError('PFSP requires a committed matrix')
    row = matrix['cells'][target['opening']]
    opponents, weights = [], []
    for current in active:
        if current['opening'] == target['opening']:
            continue
        for opponent in [current, *hall[current['opening']]]:
            if opponent in opponents:
                continue
            opponents.append(opponent)
            # Matrices are opening-keyed: historical versions inherit the current
            # member's score. Unobserved matchups use a neutral p=.5.
            score = row[opponent['opening']]['rate']
            p = .5 if score is None else score
            weights.append({'opponent': copy.deepcopy(opponent), 'score': score, 'weight': (1 - p)**2 + .05})
    return weights


def response_jobs(target, active, hall, seeds, first, opponent_weighting='uniform', matrix=None):
    """Legacy uniform coverage, or PFSP over current and historical versions."""
    others = [m for m in active if m['opening'] != target['opening']]
    if not others:
        raise ValueError('coevolution requires another opening')
    rng = random.Random(first)
    if opponent_weighting == 'pfsp':
        weights = pfsp_weights(target, active, hall, matrix)
        opponents = rng.choices([w['opponent'] for w in weights], [w['weight'] for w in weights], k=seeds)
        return [(target, copy.deepcopy(opponent), first + offset, 1) for offset, opponent in enumerate(opponents)]
    if opponent_weighting != 'uniform':
        raise ValueError('unknown opponent weighting')
    jobs = []
    for offset in range(seeds):
        if offset % len(others) == 0:
            rng.shuffle(others)
        opponent = others[offset % len(others)]
        if rng.random() < .5:
            opponent = rng.choice(hall[opponent['opening']])
        jobs.append((target, copy.deepcopy(opponent), first + offset, 1))
    return jobs


def resolve_targets(members, requested):
    tunable = [m['opening'] for m in members if m['opening'] != 'Default']
    if not tunable or len(members) < 2:
        raise ValueError('need at least two openings including an evolvable opening')
    targets = [name.strip() for name in requested.split(',')] if requested is not None else tunable
    if not targets or len(targets) != len(set(targets)) or any(name not in tunable for name in targets):
        raise ValueError('--targets must be distinct tunable openings in the member list')
    return targets


def select_target(targets, generation, order, matrix, history=(), max_consecutive=2, fairness_window=None):
    window = 2 * len(targets) if fairness_window is None else fairness_window
    if max_consecutive < 1 or window < len(targets):
        raise ValueError('max-consecutive must be positive and fairness-window must cover every target')
    if order == 'round-robin':
        return targets[generation % len(targets)]
    if order != 'worst-first':
        raise ValueError('unknown target order')
    # A single selected target cannot yield to another opening.
    if len(targets) == 1:
        return targets[0]
    validity = d.validity_report(matrix)['openings']

    def priority(name):
        result = validity[name]
        return (-sum(not passed for passed in result['criteria'].values()),
                result['mean_score'] if result['mean_score'] is not None else -1, name)

    eligible = list(targets)
    if len(history) >= max_consecutive and len(set(history[-max_consecutive:])) == 1:
        eligible = [name for name in eligible if name != history[-1]]
    last = {name: -1 for name in targets}
    for index, name in enumerate(history):
        if name in last:
            last[name] = index
    deadlines = {name: index + window for name, index in last.items()}

    def leaves_time_for_others(name):
        # Unseen targets share the first window's deadline. Reserve enough slots
        # for ALL pending targets rather than waiting until one is already overdue.
        pending = sorted(deadline for other, deadline in deadlines.items() if other != name)
        return all(generation + offset <= deadline for offset, deadline in enumerate(pending, 1))

    feasible = [name for name in eligible if leaves_time_for_others(name)]
    if feasible:
        return min(feasible, key=priority)
    # Old checkpoints may already violate fairness. Catch up oldest targets first.
    return min(eligible, key=lambda name: (deadlines[name], priority(name)))


def with_target(jobs, target):
    return [(target, opponent, seed, count) for _, opponent, seed, count in jobs]


def train_score(games):
    values, dropped = d.map_scores(games)
    # A cap contributes zero to training fitness, not a manufactured VP win.
    return sum(values.values()) / (len(values) + dropped)


def candidate_fitness(score, boundary_penalty, gate, enabled, rules):
    """Shape progress toward the gate; promotion is decided on held-out games."""
    shortfall = max(0, rules['required_share'] - (gate['pass_rate'] or 0))
    penalty = rules['fitness_penalty'] * shortfall if enabled else 0
    return {'fitness': score - boundary_penalty - penalty, 'identity_penalty': penalty}


def encode(genome, genes, indices):
    return [(genome[i] - genes[i]['min']) / (genes[i]['max'] - genes[i]['min']) for i in indices]


def decode(vector, genome, genes, indices):
    result = genome[:]
    for x, i in zip(vector, indices):
        g = genes[i]
        result[i] = round(g['min'] + min(1, max(0, x)) * (g['max'] - g['min']))
    return result


def write_journal(path, rows):
    temp = path.with_suffix('.tmp')
    with temp.open('w') as stream:
        for row in rows:
            stream.write(json.dumps(row, sort_keys=True, allow_nan=False) + '\n')
    temp.replace(path)


def sync_outputs(root, state, initial):
    # Checkpoints are the commit boundary. Rebuild journals after an interrupted append.
    checkpoints = [d.read_json(root / 'checkpoints' / f'{g:06d}.json') for g in range(state['next_gen'])]
    write_journal(root / 'generations.jsonl', [c['record'] for c in checkpoints])
    write_journal(root / 'matrix-history.jsonl', [{'generation': -1, 'summary': initial}] +
                  [{'generation': c['record']['generation'], 'summary': c['matrix']} for c in checkpoints])
    d.write_json(root / 'active-members.json', state['members'])
    d.write_json(root / 'hall-of-fame.json', state['hall'])
    return [c['record']['opening'] for c in checkpoints]


def evolve(sim, args, members):
    root = args.run
    initial_path = root / 'initial' / 'summary.json'
    initial = d.read_json(initial_path) if initial_path.exists() else d.round_robin(
        sim, members, args.seeds, seed_for(3, 0, args.seed), root / 'initial')
    state_path = root / 'state.json'
    if state_path.exists():
        state = d.read_json(state_path)
    else:
        hall = {}
        for member in members:
            builtin = {'opening': member['opening'], 'genome': None}
            hall[member['opening']] = [builtin] if member == builtin else [builtin, member]
        state = {'members': members, 'hall': hall, 'optimizers': {}, 'next_gen': 0, 'matrix': initial}
        d.write_json(state_path, state)
    history = sync_outputs(root, state, initial)
    for generation in range(state['next_gen'], args.generations):
        next_state = copy.deepcopy(state)
        target_name = select_target(args.targets, generation, args.target_order, state['matrix'],
                                    history, args.max_consecutive, args.fairness_window)
        target = next(m for m in state['members'] if m['opening'] == target_name)
        spec = sim.schema['openings'][target_name]
        genes = spec['genes']
        genome = target['genome'] or spec['genome']
        indices = [i for i, g in enumerate(genes) if g['max'] > g['min']]
        optimizer = (CMAES.from_dict(state['optimizers'][target_name]) if target_name in state['optimizers']
                     else CMAES(encode(genome, genes, indices), args.sigma, args.population, args.seed + generation))
        weights = pfsp_weights(target, state['members'], state['hall'], state['matrix']) if args.opponent_weighting == 'pfsp' else None
        jobs = response_jobs(target, state['members'], state['hall'], args.train_seeds,
                             seed_for(1, generation, args.seed), args.opponent_weighting, state['matrix'])
        def gate_rate(games):
            return d.identity.report(games, [target_name],
                                     sim.identity_rules)['openings'][target_name]

        base_games = sim.evaluate(jobs)
        base = train_score(base_games)
        base_identity = gate_rate(base_games)
        vectors = optimizer.ask()
        candidates, fitnesses = [], []
        for index, vector in enumerate(vectors):
            candidate = {'opening': target_name, 'genome': decode(vector, genome, genes, indices)}
            games = sim.evaluate(with_target(jobs, candidate))
            score = train_score(games)
            penalty = .05 * sum((x - min(1, max(0, x)))**2 for x in vector)
            gate = gate_rate(games)
            fitness = candidate_fitness(score, penalty, gate, args.identity_gates, sim.identity_rules)
            candidates.append({'index': index, 'member': candidate, 'score': score,
                               'stalls': sum(g['outcome'] == 'stalled' for g in games),
                               'boundary_penalty': penalty, 'identity': gate, **fitness})
            fitnesses.append(-fitness['fitness'])
        optimizer.tell(vectors, fitnesses)
        best = max(candidates, key=lambda c: c['fitness'])
        # Exactly one training-selected candidate sees these fresh holdout seeds.
        held_jobs = response_jobs(target, state['members'], state['hall'], args.holdout_seeds,
                                  seed_for(2, generation, args.seed), args.opponent_weighting, state['matrix'])
        held_before = sim.evaluate(held_jobs)
        held_after = sim.evaluate(with_target(held_jobs, best['member']))
        comparison = compare_holdout(held_after, held_before, args.min_delta)
        comparison.update(compare_identity(held_after, held_before, target_name,
                                           sim.identity_rules))
        comparison['promotion_reason'] = promotion_reason(comparison, args.identity_gates)
        accepted = comparison['promotion_reason'] is not None
        if accepted:
            next_state['members'] = [best['member'] if m['opening'] == target_name else m for m in state['members']]
            if best['member'] not in next_state['hall'][target_name]:
                next_state['hall'][target_name].append(best['member'])
            d.write_json(root / 'genomes' / f'{generation:06d}-{target_name}.json', best['member']['genome'])
            next_state['matrix'] = d.round_robin(sim, next_state['members'], args.seeds,
                                                seed_for(3, 0, args.seed), root / 'matrices' / f'{generation:06d}')
        next_state['optimizers'][target_name] = optimizer.to_dict()
        next_state['next_gen'] = generation + 1
        record = {'generation': generation, 'opening': target_name, 'accepted': accepted,
                  'incumbent_train_score': base, 'candidates': candidates, 'best': best, 'held_out': comparison,
                  'identity_gates': args.identity_gates, 'incumbent_train_identity': base_identity,
                  'train_seed': seed_for(1, generation, args.seed), 'holdout_seed': seed_for(2, generation, args.seed),
                  'opponents': [job[1] for job in jobs], 'held_out_opponents': [job[1] for job in held_jobs],
                  'opponent_weighting': args.opponent_weighting, 'opponent_weights': weights,
                  'target_order': args.target_order,
                  'sigma': optimizer.sigma, 'hall_sizes': {o: len(h) for o, h in next_state['hall'].items()}}
        d.write_json(root / 'checkpoints' / f'{generation:06d}.json', {'state': next_state, 'record': record, 'matrix': next_state['matrix']})
        d.write_json(state_path, next_state)
        state = next_state
        history = sync_outputs(root, state, initial)
        print(f"generation {generation} {target_name}: held-out delta={comparison['delta']} p={comparison['p_value']:.5g} accepted={accepted}", flush=True)
    return state


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    d.add_args(parser)
    parser.set_defaults(seeds=20)
    parser.add_argument('--generations', type=d.positive, default=24)
    parser.add_argument('--population', type=d.positive, default=6)
    parser.add_argument('--sigma', type=float, default=.2)
    parser.add_argument('--train-seeds', type=d.positive, default=24)
    parser.add_argument('--holdout-seeds', type=d.positive, default=100)
    parser.add_argument('--min-delta', type=float, default=.0025)
    parser.add_argument('--opponent-weighting', choices=['uniform', 'pfsp'], default='uniform')
    parser.add_argument('--targets', help='comma-separated tunable openings; default: all, in member order')
    parser.add_argument('--target-order', choices=['round-robin', 'worst-first'], default='round-robin')
    parser.add_argument('--max-consecutive', type=d.positive, default=2,
                        help='maximum consecutive generations per opening when multiple targets are selected (default: 2)')
    parser.add_argument('--fairness-window', type=d.positive,
                        help='every target must appear in each sliding window (default: twice the target count)')
    parser.add_argument('--identity-gates', action='store_true',
                        help='shape training toward identity and prevent held-out identity regression')
    args = parser.parse_args(argv)
    if (args.population < 2 or args.holdout_seeds < 8 or not math.isfinite(args.sigma) or args.sigma <= 0
            or not math.isfinite(args.min_delta) or args.min_delta < 0
            or max(args.seeds, args.train_seeds, args.holdout_seeds) >= 2**31):
        parser.error('population >=2, holdout-seeds >=8, sigma >0, min-delta >=0 and seed counts <2^31 required')
    seed_for(1, args.generations, args.seed)
    args.run = args.run.resolve()
    with d.locked(args.run):
        sim = d.Simulator(args, args.run)
        members = d.load_members(args, sim.schema)
        args.targets = resolve_targets(members, args.targets)
        if args.fairness_window is None:
            args.fairness_window = 2 * len(args.targets)
        if args.fairness_window < len(args.targets):
            parser.error('fairness-window must be at least the number of targets')
        d.ensure_config(args.run / 'config.json', {'binary_sha256': sim.binary_hash, 'settings': sim.settings,
            'identity_gates': args.identity_gates, 'identity_rules': sim.identity_rules,
            'members': members, 'seed': args.seed, 'matrix_seeds': args.seeds,
            'train_seeds': args.train_seeds, 'holdout_seeds': args.holdout_seeds,
            'population': args.population, 'sigma': args.sigma, 'min_delta': args.min_delta, 'alpha': ALPHA,
            'opponent_weighting': args.opponent_weighting, 'targets': args.targets, 'target_order': args.target_order,
            'max_consecutive': args.max_consecutive, 'fairness_window': args.fairness_window},
            # Missing options receive legacy defaults; explicitly saved values still must match.
            defaults={'opponent_weighting': 'uniform', 'targets': resolve_targets(members, None),
                      'target_order': 'round-robin', 'max_consecutive': 2,
                      'fairness_window': 2 * len(args.targets)})
        evolve(sim, args, members)
    return 0


if __name__ == '__main__':
    signal.signal(signal.SIGTERM, d.terminate_signal)
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
    except (ValueError, RuntimeError, OSError, subprocess.TimeoutExpired) as error:
        raise SystemExit(f'duel_evolve: {error}')

import argparse
import copy
import itertools
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from tools.duel import duel as d
from tools.duel import duel_evolve as e
from tools.evolve.cmaes import CMAES
from tools.duel.equilibrium import nash
from tools.duel.mathlib import direction_p, holm, interval, sign_p

ROOT = Path(__file__).resolve().parents[2]
FAKE = Path(__file__).with_name('fake_sim.py')


def pair(a, b, seed, score, stalled=False):
    """Two mirrored games; focal A scores 0, .5 or 1 on the map."""
    result = []
    for orientation in (False, True):
        focal = int(orientation)
        won = score == 1 or score == .5 and not orientation
        winner = focal if won else 1 - focal
        openings = [b, a] if orientation else [a, b]
        outcome = 'stalled' if stalled and orientation else 'vp'
        result.append({'seed': str(seed), 'mirrored': orientation, 'turns': 20, 'duel_members': [a, b],
                       'genome_hashes': ['focal', 'opponent'], 'outcome': outcome,
                       'victory_type': 'vp' if outcome == 'vp' else None,
                       'winner_players': [winner] if outcome == 'vp' else [],
                       'players': [{'id': s, 'faction': ['Japan', 'Celts'][s], 'vp': 10 if s == winner else 4,
                                    'opening': o, 'won': s == winner and outcome == 'vp',
                                    'cities_at_turns': {}, 'phase_switches': []} for s, o in enumerate(openings)]})
    return result


class MatrixTests(unittest.TestCase):
    def test_interval_counts_maps_and_handles_empty(self):
        self.assertEqual(interval([]), [None, None])
        low, high = interval([.5] * 100)
        self.assertAlmostEqual(high - .5, math.sqrt(math.log(40) / 200))
        self.assertAlmostEqual(.5 - low, high - .5)
        self.assertLess(interval([0] * 100)[1], .2)

    def test_exact_sign_test_and_holm(self):
        self.assertEqual(sign_p(0, 0), 1)
        self.assertAlmostEqual(sign_p(8, 0), 2 / 256)
        self.assertEqual(sign_p(20, 20), 1)
        self.assertEqual(holm([.01, .04, .03]), [.03, .06, .06])

    def test_fractional_draws_test_the_mean_instead_of_the_median(self):
        # Strongly negative median, exactly neutral mean: no directed edge.
        self.assertEqual(direction_p([.25] * 200 + [1] * 100), 1)
        self.assertAlmostEqual(direction_p([1] * 8 + [.5] * 20), 2 / 256)

    def tournament(self, cyclic=True):
        names = ['Default', 'Rush', 'GreedyExpand']
        games = []
        for i, a in enumerate(names):
            for j, b in enumerate(names[i:], i):
                score = .5 if i == j else 0 if cyclic and (i, j) == (0, 2) else 1
                for seed in range(100):
                    games += pair(a, b, seed, score)
        return d.summarize(games, names)

    def test_cycle_is_three_significant_edges_not_a_rank_order(self):
        cyclic = self.tournament()
        self.assertEqual(cyclic['cycles'], [['Default', 'Rush', 'GreedyExpand', 'Default']])
        self.assertEqual(self.tournament(False)['cycles'], [])
        self.assertTrue(cyclic['equilibrium']['converged'])
        for weight in cyclic['equilibrium']['weights']:
            self.assertAlmostEqual(weight, 1 / 3)
        cell = cyclic['cells']['Default']['Rush']
        self.assertEqual((cell['games'], cell['maps'], cell['rate']), (200, 100, 1))
        self.assertEqual(cyclic['cells']['Rush']['Default']['rate'], 0)

    def test_validity_uses_significant_edges_and_reports_each_failure(self):
        cyclic = self.tournament()
        validity = cyclic['validity']
        self.assertEqual(validity['verdict'], 'VALID')
        self.assertTrue(validity['passed'])
        self.assertEqual(validity['reasons'], [])
        self.assertEqual(validity['significant_cycles'], cyclic['cycles'])
        rush = validity['openings']['Rush']
        self.assertEqual(rush['mean_score'], .5)
        self.assertEqual(rush['significantly_beats'], ['GreedyExpand'])
        self.assertEqual(rush['significantly_beaten_by'], ['Default'])
        self.assertTrue(all(rush['criteria'].values()))
        self.assertIn('VALID: all openings meet all three criteria.', d.render(cyclic))
        self.assertIn('| Rush | 0.500 | GreedyExpand | Default | PASS | PASS | PASS |', d.render(cyclic))

        ranked = self.tournament(False)
        validity = ranked['validity']
        self.assertEqual(validity['verdict'], 'NOT VALID')
        self.assertFalse(validity['openings']['Default']['criteria']['does_not_beat_all'])
        self.assertTrue(validity['openings']['Rush']['passed'])
        weakest = validity['openings']['GreedyExpand']
        self.assertEqual(weakest['mean_score'], 0)
        self.assertFalse(weakest['criteria']['beats_at_least_one'])
        self.assertFalse(weakest['criteria']['mean_score_at_least_0_35'])
        self.assertEqual(len(validity['reasons']), 3)
        for reason in validity['reasons']:
            self.assertIn(reason, d.render(ranked))

    def test_validity_mean_is_unweighted_inclusive_and_requires_all_opponents(self):
        games = []
        for seed in range(20):
            games += pair('Rush', 'Snowball', seed, 1 if seed < 8 else 0)
        for seed in range(10):
            games += pair('Rush', 'Economic', seed, .5 if seed < 6 else 0)
        summary = d.summarize(games, ['Rush', 'Snowball', 'Economic'])
        rush = summary['validity']['openings']['Rush']
        self.assertEqual(rush['mean_score'], .35)  # (.4 + .3) / 2, regardless of map counts
        self.assertTrue(rush['criteria']['mean_score_at_least_0_35'])
        self.assertIsNone(summary['validity']['openings']['Snowball']['mean_score'])
        games += pair('Rush', 'Economic', 10, 0)
        self.assertFalse(d.summarize(games, summary['openings'])['validity']['openings']['Rush']
                         ['criteria']['mean_score_at_least_0_35'])

        small = d.summarize(sum((pair('Rush', 'Snowball', seed, 1) for seed in range(7)), []), ['Rush', 'Snowball'])
        self.assertFalse(small['validity']['openings']['Rush']['criteria']['beats_at_least_one'])
        self.assertEqual(small['validity']['verdict'], 'NOT VALID')
        stalled = d.summarize(pair('Rush', 'Snowball', 0, 1, stalled=True), ['Rush', 'Snowball'])
        self.assertIsNone(stalled['validity']['openings']['Rush']['mean_score'])
        self.assertIn('incomplete matchups', d.render(stalled))
        solo = d.summarize(pair('Rush', 'Rush', 0, .5), ['Rush'])
        self.assertFalse(solo['validity']['passed'])
        self.assertIsNone(solo['validity']['openings']['Rush']['mean_score'])

    def test_validity_does_not_add_a_triangle_requirement(self):
        names = ['Rush', 'Snowball', 'Economic', 'GreedyExpand']
        edges = {('Rush', 'Snowball'), ('Snowball', 'Economic'), ('Economic', 'GreedyExpand'), ('GreedyExpand', 'Rush')}
        games = []
        for i, a in enumerate(names):
            for b in names[i:]:
                score = 1 if (a, b) in edges else 0 if (b, a) in edges else .5
                for seed in range(20):
                    games += pair(a, b, seed, score)
        summary = d.summarize(games, names)
        self.assertEqual(summary['cycles'], [])
        self.assertEqual(summary['validity']['verdict'], 'VALID')

    def test_stalls_drop_the_whole_map_and_missing_maps_are_not_losses(self):
        games = pair('Rush', 'Snowball', 1, 1, stalled=True)
        scores, dropped = d.map_scores(games)
        self.assertEqual((scores, dropped), ({}, 1))
        summary = d.summarize(games, ['Rush', 'Snowball'])
        self.assertIsNone(summary['cells']['Rush']['Snowball']['rate'])
        self.assertFalse(summary['equilibrium']['converged'])
        with self.assertRaises(ValueError):
            d.map_scores(games[:1])

    def test_nash_certifies_nonuniform_cycle_and_dominance(self):
        for matrix in [[[.5, 1, 0], [0, .5, .7], [1, .3, .5]], [[.5, 1], [0, .5]]]:
            result = nash(matrix)
            self.assertTrue(result['converged'])
            self.assertLessEqual(result['regret'], 1e-6)
            self.assertAlmostEqual(sum(result['weights']), 1)
        with self.assertRaises(ValueError):
            nash([[.5, None], [None, .5]])

    def test_validation_rejects_wrong_seeds_openings_and_winners(self):
        a, b = {'opening': 'Rush'}, {'opening': 'Snowball'}
        games = pair('Rush', 'Snowball', 1, 1)
        d.validate_games(games, a, b, 1, 1)
        for change in ('seed', 'opening', 'winner', 'faction'):
            broken = copy.deepcopy(games)
            if change == 'seed':
                broken[0]['seed'] = '2'
            elif change == 'opening':
                broken[0]['players'][0]['opening'] = 'Default'
            elif change == 'winner':
                broken[0]['winner_players'] = []
            else:
                broken[1]['players'][0]['faction'] = 'Germany'
            with self.assertRaises(ValueError):
                d.validate_games(broken, a, b, 1, 1)


class EvolutionTests(unittest.TestCase):
    def test_heldout_alpha_and_paired_stall_drop(self):
        before, after = [], []
        for seed in range(12):
            before += pair('Rush', 'Default', seed, 0)
            after += pair('Rush', 'Default', seed, 1)
        result = e.compare_holdout(after, before)
        self.assertTrue(result['passed'])
        self.assertEqual(result['alpha'], .01)
        self.assertEqual(result['independent_maps'], 12)
        after[-1]['outcome'] = 'stalled'
        result = e.compare_holdout(after, before)
        self.assertEqual(result['stalled_maps_dropped'], 1)
        self.assertEqual(result['independent_maps'], 11)
        self.assertFalse(result['passed'])  # candidate cannot buy strength with more stalls
        before[-2]['outcome'] = 'stalled'
        self.assertTrue(e.compare_holdout(after, before)['passed'])
        self.assertFalse(e.compare_holdout(before, before)['passed'])
        self.assertFalse(e.compare_holdout(before, after)['passed'])

    def test_no_completed_holdout_maps_and_wrong_pairing(self):
        games = pair('Rush', 'Default', 1, .5, stalled=True)
        result = e.compare_holdout(games, games)
        self.assertIsNone(result['delta'])
        self.assertEqual(result['p_value'], 1)
        self.assertFalse(result['passed'])
        other = copy.deepcopy(games)
        other[0]['genome_hashes'][1] = 'different opponent'
        with self.assertRaises(ValueError):
            e.compare_holdout(other, games)

    def test_sign_flip_matches_exhaustive_enumeration(self):
        differences = [.25, .5, 1, -.5, 0]
        possible = [sum(x * sign for x, sign in zip(differences, signs))
                    for signs in itertools.product((-1, 1), repeat=len(differences))]
        expected = sum(abs(x) >= abs(sum(differences)) for x in possible) / len(possible)
        self.assertEqual(e.sign_flip_p(differences), expected)
        self.assertEqual(e.sign_flip_p([0] * 12), 1)

    def test_one_sided_gate_count_test_matches_exhaustive_enumeration(self):
        for differences in ([2, 1, -1, 0, 2], [-2, -1, 1, 0], [0] * 8):
            possible = [sum(x * sign for x, sign in zip(differences, signs))
                        for signs in itertools.product((-1, 1), repeat=len(differences))]
            expected = sum(x >= sum(differences) for x in possible) / len(possible)
            self.assertEqual(e.sign_flip_p(differences, alternative='greater'), expected)

    def test_hall_opponents_are_actually_played_and_seeds_disjoint(self):
        target = {'opening': 'Rush', 'genome': None}
        current = {'opening': 'Snowball', 'genome': [90]}
        old = {'opening': 'Snowball', 'genome': [10]}
        jobs = e.response_jobs(target, [target, current], {'Snowball': [old]}, 100, 42)
        self.assertIn(old, [j[1] for j in jobs])
        self.assertIn(current, [j[1] for j in jobs])
        self.assertTrue(all(j[1]['opening'] != 'Rush' for j in jobs))
        seeds = [e.seed_for(domain, generation, 42) for domain in (1, 2, 3) for generation in range(6)]
        self.assertEqual(len(set(seeds)), len(seeds))

    def sampling_fixture(self):
        active = [{'opening': 'Rush', 'genome': [50]}, {'opening': 'Snowball', 'genome': [90]},
                  {'opening': 'Economic', 'genome': [60]}]
        hall = {'Rush': [active[0]], 'Snowball': [{'opening': 'Snowball', 'genome': [10]},
                                           {'opening': 'Snowball', 'genome': None}],
                'Economic': [{'opening': 'Economic', 'genome': [20]}]}
        matrix = {'cells': {'Rush': {'Snowball': {'rate': 0}, 'Economic': {'rate': 1}}}}
        return active, hall, matrix

    def test_uniform_sampling_matches_pre_pfsp_seeded_assignments(self):
        active, hall, matrix = self.sampling_fixture()
        # Captured from the original uniform sampler before adding PFSP.
        expected = [('Economic', [20]), ('Snowball', [10]), ('Snowball', [90]), ('Economic', [60]),
                    ('Economic', [60]), ('Snowball', [10]), ('Snowball', [10]), ('Economic', [60]),
                    ('Snowball', None), ('Economic', [60]), ('Economic', [60]), ('Snowball', None)]
        jobs = e.response_jobs(active[0], active, hall, 12, 42)
        self.assertEqual([(job[1]['opening'], job[1]['genome']) for job in jobs], expected)
        self.assertEqual(jobs, e.response_jobs(active[0], active, hall, 12, 42, 'uniform', matrix))
        self.assertEqual([job[2:] for job in jobs], [(seed, 1) for seed in range(42, 54)])

    def test_pfsp_weights_versions_and_deterministically_prioritizes_losses(self):
        active, hall, matrix = self.sampling_fixture()
        hall['Snowball'].append(active[1])  # A current member also in the hall is one version.
        weights = e.pfsp_weights(active[0], active, hall, matrix)
        self.assertEqual(len(weights), 5)
        self.assertEqual([w['weight'] for w in weights], [1.05, 1.05, 1.05, .05, .05])
        self.assertEqual([w['score'] for w in weights], [0, 0, 0, 1, 1])
        jobs = e.response_jobs(active[0], active, hall, 1000, 42, 'pfsp', matrix)
        self.assertEqual(jobs, e.response_jobs(active[0], active, hall, 1000, 42, 'pfsp', matrix))
        self.assertNotEqual([job[1] for job in jobs], [job[1] for job in
                            e.response_jobs(active[0], active, hall, 1000, 43, 'pfsp', matrix)])
        self.assertGreater(sum(job[1]['opening'] == 'Snowball' for job in jobs), 900)
        self.assertEqual({d.digest(job[1]) for job in jobs}, {d.digest(w['opponent']) for w in weights})
        self.assertEqual([job[2:] for job in jobs], [(seed, 1) for seed in range(42, 1042)])
        self.assertTrue(all(job[1]['opening'] != 'Rush' for job in jobs))

        matrix['cells']['Rush']['Snowball']['rate'] = None
        unknown = e.pfsp_weights(active[0], active, hall, matrix)
        self.assertIsNone(unknown[0]['score'])
        self.assertAlmostEqual(unknown[0]['weight'], .30)
        with self.assertRaisesRegex(ValueError, 'committed matrix'):
            e.response_jobs(active[0], active, hall, 1, 42, 'pfsp')

    def test_target_selection_and_worst_first_ties(self):
        members = [{'opening': name, 'genome': None} for name in ['Default', 'Rush', 'Snowball', 'Economic']]
        self.assertEqual(e.resolve_targets(members, None), ['Rush', 'Snowball', 'Economic'])
        self.assertEqual(e.resolve_targets(members, ' Snowball, Rush '), ['Snowball', 'Rush'])
        for requested in ['', 'Default', 'Rush,Rush', 'Rush,', 'GreedyExpand', 'unknown']:
            with self.subTest(requested=requested), self.assertRaisesRegex(ValueError, '--targets'):
                e.resolve_targets(members, requested)
        names = ['Rush', 'Snowball', 'Economic']
        games = []
        for i, a in enumerate(names):
            for b in names[i:]:
                for seed in range(4):
                    games += pair(a, b, seed, 1 if (a, b) == ('Snowball', 'Economic') else .5)
        matrix = d.summarize(games, names)
        self.assertEqual(e.select_target(names, 0, 'worst-first', matrix), 'Economic')  # two failures
        self.assertEqual(e.select_target(['Snowball', 'Rush'], 0, 'worst-first', matrix), 'Rush')  # lower mean
        ties = d.summarize([g for g in games if g['duel_members'] != ['Snowball', 'Economic']] +
                           sum((pair('Snowball', 'Economic', seed, .5) for seed in range(4)), []), names)
        self.assertEqual(e.select_target(names, 1, 'worst-first', ties), 'Economic')  # alphabetical
        self.assertEqual(e.select_target(['Snowball'], 0, 'worst-first', matrix), 'Snowball')
        self.assertEqual([e.select_target(names, i, 'round-robin', matrix) for i in range(4)],
                         ['Rush', 'Snowball', 'Economic', 'Rush'])

    def test_cma_checkpoint_preserves_sampling_and_bounds(self):
        optimizer = CMAES([.5, .5], .2, 4, seed=4)
        xs = optimizer.ask()
        optimizer.tell(xs, [sum(x*x for x in row) for row in xs])
        resumed = CMAES.from_dict(json.loads(json.dumps(optimizer.to_dict())))
        self.assertEqual(optimizer.ask(), resumed.ask())
        genes = [{'min': 0, 'max': 10}, {'min': 0, 'max': 20}]
        self.assertEqual(e.decode([-4, 4], [5, 10], genes, [0, 1]), [0, 20])

    def test_worst_first_obeys_consecutive_and_every_sliding_window_limits(self):
        for count in range(2, 6):
            targets = d.OPENINGS[1:count + 1]
            for window, consecutive in itertools.product((count, count + 1, 2 * count), (1, 2, 3)):
                with self.subTest(count=count, window=window, consecutive=consecutive):
                    history = []
                    for generation in range(6 * window):
                        # Changing rankings must not reset fairness or defeat deadlines.
                        validity = {name: {'criteria': {'win': False},
                                          'mean_score': ((generation // 3 + i) % count) / count}
                                    for i, name in enumerate(targets)}
                        with patch.object(d, 'validity_report', return_value={'openings': validity}):
                            selected = e.select_target(targets, generation, 'worst-first', None,
                                                       history, consecutive, window)
                        history.append(selected)
                        if len(history) > consecutive:
                            self.assertGreater(len(set(history[-consecutive - 1:])), 1)
                        if len(history) >= window:
                            self.assertEqual(set(history[-window:]), set(targets))

    def test_fairness_reserves_unseen_slots_and_recovers_starved_legacy_history(self):
        targets = ['Rush', 'Snowball', 'Economic']
        validity = {name: {'criteria': {'win': False}, 'mean_score': i / 3}
                    for i, name in enumerate(targets)}
        with patch.object(d, 'validity_report', return_value={'openings': validity}):
            history = []
            for generation in range(6):
                history.append(e.select_target(targets, generation, 'worst-first', None,
                                               history, 10, 6))
            self.assertEqual(history, ['Rush'] * 4 + ['Snowball', 'Economic'])
            history = ['Rush'] * 10
            for generation in range(10, 12):
                history.append(e.select_target(targets, generation, 'worst-first', None, history))
            self.assertEqual(history[-2:], ['Snowball', 'Economic'])
        self.assertEqual(e.select_target(['Snowball'], 10, 'worst-first', None, ['Snowball'] * 10), 'Snowball')


class HarnessTests(unittest.TestCase):
    def test_cpu_budget_and_nice_do_not_raise_priority(self):
        with patch.object(os, 'cpu_count', return_value=8):
            self.assertEqual(d.cpu_limit(100), 6)
            self.assertEqual(d.cpu_limit(2), 2)
        with patch.object(os, 'getpriority', return_value=20):
            self.assertEqual(d.nice_command(['sim']), ['sim'])
        with patch.object(os, 'getpriority', return_value=0):
            self.assertEqual(d.nice_command(['sim']), ['nice', '-n', '10', 'sim'])

    def test_genome_schema_order_null_and_bad_values(self):
        schema = {'openings': {'Rush': {'genome': [2, 1], 'genes': [
            {'name': 'z', 'min': 0, 'max': 4}, {'name': 'a', 'min': 0, 'max': 2}]}}}
        self.assertEqual(d.canonical_genome('Rush', {'a': 2, 'z': 3}, schema), [3, 2])
        self.assertIsNone(d.canonical_genome('Rush', None, schema))
        for value in [[1.5, 1], [1], [9, 1], {'extra': 2}]:
            with self.assertRaises(ValueError):
                d.canonical_genome('Rush', value, schema)
        with self.assertRaises(ValueError):
            d.canonical_genome('Default', [], schema)

    def run_command(self, args):
        return subprocess.run([sys.executable, *args], cwd=ROOT, capture_output=True, text=True, timeout=60)

    def test_cli_round_robin_resume_and_genome_override(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            genome = root / 'rush.json'
            genome.write_text('[90]')
            args = ['tools/duel/duel.py', '--sim', str(FAKE), '--run', str(root / 'run'), '--seeds', '2',
                    '--workers', '2', '--opening-genome', f'Rush={genome}']
            first = self.run_command(args)
            self.assertEqual(first.returncode, 0, first.stderr)
            summary = d.read_json(root / 'run' / 'summary.json')
            self.assertEqual(summary['games'], 60)
            self.assertEqual(len(summary['matrix']), 5)
            self.assertEqual(summary['validity']['verdict'], 'NOT VALID')
            self.assertEqual(set(summary['validity']['openings']), set(d.OPENINGS))
            self.assertIn('## Validity', (root / 'run' / 'matrix.md').read_text())
            again = self.run_command(args)
            self.assertEqual(again.returncode, 0, again.stderr)
            self.assertEqual(summary['matrix'], d.read_json(root / 'run' / 'summary.json')['matrix'])
            genome.write_text('[80]')
            self.assertNotEqual(self.run_command(args).returncode, 0)

    def test_cli_evolve_journals_and_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = ['tools/duel/duel_evolve.py', '--sim', str(FAKE), '--run', tmp, '--seeds', '1',
                    '--workers', '2', '--openings', 'Default', 'Rush', '--generations', '1',
                    '--population', '2', '--train-seeds', '2', '--holdout-seeds', '8']
            result = self.run_command(args)
            self.assertEqual(result.returncode, 0, result.stderr)
            root = Path(tmp)
            rows = (root / 'generations.jsonl').read_text().splitlines()
            self.assertEqual(len(rows), 1)
            self.assertEqual(json.loads(rows[0])['held_out']['alpha'], .01)
            self.assertEqual(len((root / 'matrix-history.jsonl').read_text().splitlines()), 2)
            self.assertIn('Rush', d.read_json(root / 'hall-of-fame.json'))
            self.assertEqual(self.run_command(args).returncode, 0)
            self.assertEqual(rows, (root / 'generations.jsonl').read_text().splitlines())

    def evolution_command(self, root):
        return ['tools/duel/duel_evolve.py', '--sim', str(FAKE), '--run', str(root), '--seeds', '1',
                '--workers', '2', '--openings', 'Default', 'Rush', 'Snowball', '--population', '2',
                '--train-seeds', '2', '--holdout-seeds', '8', '--min-delta', '1']

    def test_cli_defaults_match_explicit_options_and_pin_configuration(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            default_args = self.evolution_command(root / 'default') + ['--generations', '2']
            explicit_args = self.evolution_command(root / 'explicit') + ['--generations', '2',
                             '--opponent-weighting', 'uniform', '--target-order', 'round-robin', '--targets', 'Rush,Snowball',
                             '--max-consecutive', '2', '--fairness-window', '4']
            for args in [default_args, explicit_args]:
                result = self.run_command(args)
                self.assertEqual(result.returncode, 0, result.stderr)
            journal = (root / 'default' / 'generations.jsonl').read_text()
            self.assertEqual(journal, (root / 'explicit' / 'generations.jsonl').read_text())
            self.assertEqual([json.loads(line)['opening'] for line in journal.splitlines()], ['Rush', 'Snowball'])
            config = d.read_json(root / 'default' / 'config.json')
            self.assertEqual(config['opponent_weighting'], 'uniform')
            self.assertEqual(config['target_order'], 'round-robin')
            self.assertEqual(config['targets'], ['Rush', 'Snowball'])
            self.assertEqual(config['max_consecutive'], 2)
            self.assertEqual(config['fairness_window'], 4)
            state = (root / 'default' / 'state.json').read_text()
            for changed in [['--opponent-weighting', 'pfsp'], ['--target-order', 'worst-first'],
                            ['--targets', 'Snowball'], ['--targets', 'Snowball,Rush'],
                            ['--max-consecutive', '3'], ['--fairness-window', '5']]:
                with self.subTest(changed=changed):
                    result = self.run_command(default_args + changed)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('configuration changed', result.stderr)
                    self.assertEqual(journal, (root / 'default' / 'generations.jsonl').read_text())
                    self.assertEqual(state, (root / 'default' / 'state.json').read_text())

    def test_cli_legacy_config_resumes_with_original_defaults(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            options = ['--openings', 'Snowball', 'Default', 'Rush']
            resumed = self.evolution_command(root / 'resumed') + options
            first_args = resumed + ['--generations', '1']
            result = self.run_command(first_args)
            self.assertEqual(result.returncode, 0, result.stderr)
            config_path = root / 'resumed' / 'config.json'
            config = d.read_json(config_path)
            legacy = {key: value for key, value in config.items()
                      if key not in ('opponent_weighting', 'targets', 'target_order',
                                     'max_consecutive', 'fairness_window')}
            d.write_json(config_path, legacy)
            journal = (root / 'resumed' / 'generations.jsonl').read_text()
            state = (root / 'resumed' / 'state.json').read_text()
            for changed in [['--opponent-weighting', 'pfsp'], ['--target-order', 'worst-first'],
                            ['--targets', 'Snowball'], ['--targets', 'Rush,Snowball'], ['--sigma', '.3'],
                            ['--max-consecutive', '3'], ['--fairness-window', '5']]:
                with self.subTest(changed=changed):
                    result = self.run_command(first_args + changed)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('configuration changed', result.stderr)
                    self.assertEqual(d.read_json(config_path), legacy)
                    self.assertEqual(journal, (root / 'resumed' / 'generations.jsonl').read_text())
                    self.assertEqual(state, (root / 'resumed' / 'state.json').read_text())
            result = self.run_command(first_args)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(d.read_json(config_path), config)
            self.assertEqual(journal, (root / 'resumed' / 'generations.jsonl').read_text())
            self.assertEqual(state, (root / 'resumed' / 'state.json').read_text())
            fresh = self.evolution_command(root / 'fresh') + options
            for args in [resumed, fresh]:
                result = self.run_command(args + ['--generations', '2'])
                self.assertEqual(result.returncode, 0, result.stderr)
            journal = (root / 'resumed' / 'generations.jsonl').read_text()
            self.assertEqual([json.loads(line)['opening'] for line in journal.splitlines()], ['Snowball', 'Rush'])
            self.assertEqual(journal, (root / 'fresh' / 'generations.jsonl').read_text())
            states = [d.read_json(root / name / 'state.json') for name in ['resumed', 'fresh']]
            for saved in states:
                saved['matrix'].pop('wall_seconds')  # Runtime varies even when evolution is identical.
            self.assertEqual(states[0], states[1])

    def test_cli_worst_first_fairness_and_legacy_history_survive_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            options = ['--openings', 'Default', 'Rush', 'Snowball', 'Economic', '--target-order', 'worst-first',
                       '--max-consecutive', '1', '--fairness-window', '3']
            resumed = self.evolution_command(root / 'resumed') + options
            fresh = self.evolution_command(root / 'fresh') + options
            for args in [resumed + ['--generations', '2'], resumed + ['--generations', '6'],
                         fresh + ['--generations', '6']]:
                result = self.run_command(args)
                self.assertEqual(result.returncode, 0, result.stderr)
            journal = (root / 'resumed' / 'generations.jsonl').read_text()
            self.assertEqual(journal, (root / 'fresh' / 'generations.jsonl').read_text())
            names = [json.loads(line)['opening'] for line in journal.splitlines()]
            self.assertEqual(names, ['Economic', 'Rush', 'Snowball'] * 2)
            for start in range(len(names) - 2):
                self.assertEqual(set(names[start:start + 3]), {'Economic', 'Rush', 'Snowball'})
            self.assertTrue(all(a != b for a, b in zip(names, names[1:])))

            # An older worst-first run has no fairness config or separate history.
            legacy = self.evolution_command(root / 'legacy') + ['--target-order', 'worst-first']
            result = self.run_command(legacy + ['--generations', '2'])
            self.assertEqual(result.returncode, 0, result.stderr)
            config_path = root / 'legacy' / 'config.json'
            config = d.read_json(config_path)
            for key in ('max_consecutive', 'fairness_window'):
                config.pop(key)
            d.write_json(config_path, config)
            # Journals are disposable; only committed checkpoint history counts.
            (root / 'legacy' / 'generations.jsonl').write_text('interrupted append\n')
            result = self.run_command(legacy + ['--generations', '3'])
            self.assertEqual(result.returncode, 0, result.stderr)
            rows = (root / 'legacy' / 'generations.jsonl').read_text().splitlines()
            self.assertEqual([json.loads(line)['opening'] for line in rows], ['Rush', 'Rush', 'Snowball'])

    def test_cli_rejects_impossible_fairness_settings(self):
        with tempfile.TemporaryDirectory() as tmp:
            for options in [['--max-consecutive', '0'], ['--fairness-window', '0'],
                            ['--fairness-window', '1']]:
                with self.subTest(options=options):
                    root = Path(tmp) / options[0][2:]
                    result = self.run_command(self.evolution_command(root) + options)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse((root / 'state.json').exists())

    def test_cli_pfsp_restricted_targets_are_deterministic_across_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            options = ['--openings', 'Default', 'Rush', 'Snowball', 'Economic', '--opponent-weighting', 'pfsp',
                       '--target-order', 'worst-first', '--targets', 'Snowball,Rush']
            resumed = self.evolution_command(root / 'resumed') + options
            fresh = self.evolution_command(root / 'fresh') + options
            for args in [resumed + ['--generations', '1'], resumed + ['--generations', '2'],
                         fresh + ['--generations', '2']]:
                result = self.run_command(args)
                self.assertEqual(result.returncode, 0, result.stderr)
            journal = (root / 'resumed' / 'generations.jsonl').read_text()
            self.assertEqual(journal, (root / 'fresh' / 'generations.jsonl').read_text())
            for line in journal.splitlines():
                record = json.loads(line)
                self.assertEqual(record['opening'], 'Rush')  # Worst-first ties use names, not the requested order.
                self.assertEqual(record['opponent_weighting'], 'pfsp')
                self.assertEqual(record['target_order'], 'worst-first')
                self.assertEqual({w['opponent']['opening'] for w in record['opponent_weights']},
                                 {'Default', 'Snowball', 'Economic'})  # Excluded targets remain opponents.
                self.assertEqual(len(record['opponent_weights']), 3)
                self.assertTrue(all(w['score'] == .5 and w['weight'] == .30 for w in record['opponent_weights']))
            self.assertEqual(set(d.read_json(root / 'resumed' / 'state.json')['optimizers']), {'Rush'})

    def test_pfsp_uses_committed_matrix_and_identical_heldout_jobs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            members = [{'opening': 'Rush', 'genome': [0]}, {'opening': 'Snowball', 'genome': [100]},
                       {'opening': 'Economic', 'genome': [50]}]
            d.write_json(root / 'members.json', members)
            args = self.evolution_command(root / 'run')[1:] + ['--members', str(root / 'members.json'),
                    '--seeds', '16', '--sigma', '.5', '--generations', '2', '--targets', 'Rush',
                    '--opponent-weighting', 'pfsp', '--target-order', 'worst-first']
            evaluations = []
            evaluate, compare = d.Simulator.evaluate, e.compare_holdout

            def tracked(sim, jobs):
                evaluations.append(copy.deepcopy(jobs))
                return evaluate(sim, jobs)

            def promoted(after, before, min_delta):
                result = compare(after, before, min_delta)
                result['passed'] = True  # Exercise a committed matrix refresh without a long search.
                return result

            with patch.object(d.Simulator, 'evaluate', tracked), patch.object(e, 'compare_holdout', promoted):
                self.assertEqual(e.main(args), 0)
            rows = [json.loads(line) for line in (root / 'run' / 'generations.jsonl').read_text().splitlines()]
            committed = d.read_json(root / 'run' / 'checkpoints' / '000000.json')['state']
            target = next(member for member in committed['members'] if member['opening'] == 'Rush')
            self.assertEqual(rows[1]['opponent_weights'], e.pfsp_weights(target, committed['members'],
                            committed['hall'], committed['matrix']))
            self.assertNotEqual(rows[0]['opponent_weights'], rows[1]['opponent_weights'])
            holdouts = [jobs for jobs in evaluations if jobs[0][2] >> 62 == 2]
            self.assertEqual(len(holdouts), 4)
            for before, after in zip(holdouts[::2], holdouts[1::2]):
                self.assertEqual([job[1:] for job in before], [job[1:] for job in after])
            self.assertNotEqual(holdouts[0][0][2], holdouts[2][0][2])

    def test_timeout_terminates_only_owned_process(self):
        with tempfile.TemporaryDirectory() as tmp:
            parser = argparse.ArgumentParser()
            d.add_args(parser)
            args = parser.parse_args(['--sim', str(FAKE), '--run', tmp])
            sim = d.Simulator(args, tmp)
            sim.timeout = .05
            with self.assertRaises(subprocess.TimeoutExpired):
                sim.process(['--delay', '2'])
            self.assertEqual(sim._children, set())


if __name__ == '__main__':
    unittest.main()

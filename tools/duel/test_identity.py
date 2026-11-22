import argparse
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.duel import duel as d, duel_evolve as e, identity


def fixture(opening):
    scale = 1
    seat = {'id': 1, 'opening': opening, 'wars_declared': 1,
            'first_opponent_war_turn': int(80 * scale),
            'first_opponent_city_pressure_turn': int(80 * scale),
            'guarded_expansion_turns': [int(120 * scale)]}
    empire = {'early_founded': 3, 'early_peak_cities': 3,
              'early_military_cost': 140,
              'development_turn': int(240 * scale), 'cities': 4,
              'production': 40, 'science': 24, 'markets': 2, 'gold': 18}
    game = {'players': [{'id': 0, 'opening': 'Default'}, seat], 'turns': int(240 * scale),
            'outcome': 'vp', 'telemetry': {'strategy': {'early_end_turn': int(120 * scale),
            'development_end_turn': int(240 * scale), 'empires': {'1': empire}}}}
    return game, seat, empire


class GateTests(unittest.TestCase):
    def test_all_gates_at_their_standard_boundaries(self):
        for opening in d.OPENINGS[1:]:
            with self.subTest(opening=opening):
                game, seat, _ = fixture(opening)
                result = identity.seat_gate(game, seat)
                self.assertTrue(result['passed'], result)

    def test_rush_needs_own_war_and_city_pressure_before_deadline(self):
        for field in ('first_opponent_war_turn', 'first_opponent_city_pressure_turn'):
            for value in (None, 0, 81):
                with self.subTest(field=field, value=value):
                    game, seat, _ = fixture('Rush')
                    seat[field] = value
                    self.assertFalse(identity.seat_gate(game, seat)['passed'])
        game, seat, _ = fixture('Rush')
        seat.pop('first_opponent_city_pressure_turn')
        seat.update(cities_captured=4, wars_with_battles=5, phase_switches=[{'turn': 1, 'phase': 'Conquer'}])
        self.assertIsNone(identity.seat_gate(game, seat)['passed'])
        seat['wars_declared'] = 0
        self.assertFalse(identity.seat_gate(game, seat)['passed'])

    def test_greedy_needs_founding_not_captures(self):
        game, seat, empire = fixture('GreedyExpand')
        empire['early_founded'] = 2
        seat.update(cities_at_turns={'100': 10}, cities_captured=10)
        self.assertFalse(identity.seat_gate(game, seat)['passed'])
        empire['early_founded'] = 3
        empire['early_military_cost'] = 141
        self.assertFalse(identity.seat_gate(game, seat)['passed'])

    def test_snowball_and_economic_need_their_actual_yields(self):
        cases = [('Snowball', 'production', 39), ('Snowball', 'cities', 3),
                 ('Economic', 'gold', 17), ('Economic', 'markets', 1)]
        for opening, field, value in cases:
            with self.subTest(opening=opening, field=field, value=value):
                game, seat, empire = fixture(opening)
                empire[field] = value
                seat.update(gold_hoarded_peak=99999, gold=99999)
                self.assertFalse(identity.seat_gate(game, seat)['passed'])

    def test_missing_and_wrong_horizon_telemetry_never_passes(self):
        for opening in d.OPENINGS[1:]:
            game, seat, _ = fixture(opening)
            game.pop('telemetry')
            for field in ('first_opponent_war_turn', 'first_opponent_city_pressure_turn', 'guarded_expansion_turns'):
                seat.pop(field)
            row = identity.report([game], [opening])['openings'][opening]
            self.assertEqual((row['status'], row['pass_rate'], row['unknown_games']), ('FAIL', 0, 1))
            self.assertEqual(row['pass_rate_upper_bound'], 1)
            self.assertEqual(row['missing_telemetry_games'], 1)
        game, seat, empire = fixture('Snowball')
        game['telemetry']['strategy']['early_end_turn'] = 240
        self.assertIsNone(identity.seat_gate(game, seat)['passed'])
        game, seat, empire = fixture('Economic')
        empire['development_turn'] = 0
        self.assertIsNone(identity.seat_gate(game, seat)['passed'])

    def test_early_ending_games_use_actual_observations_and_stalls_stay_in_denominator(self):
        game, seat, empire = fixture('Economic')
        game['turns'] = empire['development_turn'] = 100
        self.assertTrue(identity.seat_gate(game, seat)['passed'])
        failed = copy.deepcopy(game)
        failed['outcome'] = 'stalled'
        failed['telemetry']['strategy']['empires']['1']['gold'] = 0
        row = identity.report([game, failed], ['Economic'])['openings']['Economic']
        self.assertEqual((row['games'], row['pass_rate'], row['status']), (2, .5, 'FAIL'))

    def test_incompatible_observation_windows_cannot_prove_failure_either(self):
        game, seat, empire = fixture('GreedyExpand')
        game['telemetry']['strategy']['early_end_turn'] = 60
        empire['early_founded'] = 1  # Failure by turn 60 does not establish failure by turn 120.
        row = identity.report([game], ['GreedyExpand'])['openings']['GreedyExpand']
        self.assertEqual((row['pass_rate'], row['pass_rate_upper_bound'], row['unknown_games']), (0, 1, 1))
        game, seat, empire = fixture('Economic')
        empire['development_turn'] = 241
        empire['gold'] = 0
        self.assertIsNone(identity.seat_gate(game, seat)['passed'])

    def test_rates_use_seat_ids_mirrors_and_both_self_play_seats(self):
        game, seat, _ = fixture('Rush')
        game['players'][0] = {**seat, 'id': 0, 'first_opponent_city_pressure_turn': None}
        mirror = copy.deepcopy(game)
        mirror['players'].reverse()
        mirror['mirrored'] = True
        row = identity.report([game, mirror], ['Rush'])['openings']['Rush']
        self.assertEqual((row['games'], row['passed_games'], row['failed_games']), (4, 2, 2))
        game, _, _ = fixture('Rush')
        fail = copy.deepcopy(game)
        fail['players'][1]['first_opponent_war_turn'] = None
        report = identity.report([game] * 3 + [fail] * 2, ['Default', 'Rush', 'Snowball'])
        self.assertEqual(report['openings']['Rush']['pass_rate'], .6)
        self.assertEqual(report['openings']['Rush']['status'], 'PASS')
        self.assertEqual(report['openings']['Default']['status'], 'N/A')
        self.assertEqual(report['openings']['Snowball']['status'], 'FAIL')
        self.assertIsNone(report['openings']['Snowball']['pass_rate'])

    def test_shaping_penalizes_shortfall_and_ranks_progress_below_the_gate(self):
        rules = identity.load_rules()
        for rate, shortfall in [(None, .6), (0, .6), (.4, .2), (.6, 0), (1, 0)]:
            gate = {'pass_rate': rate}
            result = e.candidate_fitness(1, .1, gate, True, rules)
            self.assertAlmostEqual(result['fitness'], .9 - rules['fitness_penalty'] * shortfall)
            self.assertAlmostEqual(result['identity_penalty'], rules['fitness_penalty'] * shortfall)
            off = e.candidate_fitness(1, .1, gate, False, rules)
            self.assertEqual(off['fitness'], .9)
        closer = e.candidate_fitness(0, 0, {'pass_rate': .4}, True, rules)
        winner = e.candidate_fitness(1, 0, {'pass_rate': .2}, True, rules)
        self.assertGreater(closer['fitness'], winner['fitness'])


class IdentityEvolutionTests(unittest.TestCase):
    def command(self, root):
        return ['--sim', str(Path(__file__).with_name('fake_sim.py')), '--run', str(root),
                '--seeds', '1', '--workers', '2', '--openings', 'Default', 'Rush',
                '--generations', '1', '--population', '2', '--train-seeds', '2', '--holdout-seeds', '8']

    def test_identity_veto_survives_winning_holdout_and_disabled_flag_preserves_promotion(self):
        with tempfile.TemporaryDirectory() as tmp:
            for enabled in (False, True):
                root = Path(tmp) / str(enabled)
                args = self.command(root) + ['--holdout-seeds', '20'] + (['--identity-gates'] if enabled else [])
                with patch.object(e.CMAES, 'ask', return_value=[[.9], [.9]]):
                    self.assertEqual(e.main(args), 0)
                record = json.loads((root / 'generations.jsonl').read_text())
                self.assertTrue(record['held_out']['passed'])
                self.assertEqual(record['accepted'], not enabled)
                self.assertEqual(record['best']['identity']['pass_rate'], 0)
                self.assertEqual(record['incumbent_train_identity']['pass_rate'], 1)
                self.assertEqual(record['held_out']['candidate_identity']['pass_rate'], 0)
                self.assertEqual(record['best']['identity_penalty'], 600 if enabled else 0)
                self.assertFalse(record['held_out']['identity_no_regression'])
                active = d.read_json(root / 'active-members.json')[1]
                self.assertEqual(active['genome'], None if enabled else [90])
                journal = (root / 'generations.jsonl').read_text()
                self.assertEqual(e.main(args), 0)
                self.assertEqual((root / 'generations.jsonl').read_text(), journal)

    def test_selects_compliant_candidate_and_pins_flag_and_thresholds_on_resume(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            args = self.command(root) + ['--identity-gates']
            with patch.object(e.CMAES, 'ask', return_value=[[.9], [.5]]):
                self.assertEqual(e.main(args), 0)
            record = json.loads((root / 'generations.jsonl').read_text())
            self.assertEqual(record['best']['member']['genome'], [50])
            self.assertTrue(record['held_out']['identity_no_regression'])
            self.assertIn('## Identity', (root / 'initial' / 'matrix.md').read_text())
            with self.assertRaisesRegex(ValueError, 'configuration changed'):
                e.main(self.command(root))
            rules = identity.load_rules()
            rules['required_share'] = .7
            with patch.object(identity, 'load_rules', return_value=rules), \
                    self.assertRaisesRegex(ValueError, 'configuration changed'):
                e.main(args)



if __name__ == '__main__':
    unittest.main()

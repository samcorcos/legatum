"""Synthetic controls for the W0.4 gate: role-normalised sign, repeat-control and provenance enforcement.
No simulation or AWS access:  python3 tools/ai-quality/test_w04_gate.py"""
import copy
import importlib.util
import json
import random
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('w04_gate', Path(__file__).with_name('w04_gate.py'))
w04 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w04)

MAPS = list('abcdefgh')
SALTS = 128
COMMIT = 'a' * 40
BIN = '0123456789abcdef'


def control_wins(seed=1, p=0.4):
    rng = random.Random(seed)
    return [[int(rng.random() < p) for _ in range(SALTS)] for _ in MAPS]


def raised(wins, seed=2):
    """Every loss becomes a win with probability .5: a large positive paired seat-win delta."""
    rng = random.Random(seed)
    return [[1 if w or rng.random() < .5 else 0 for w in row] for row in wins]


def lowered(wins, seed=3):
    """Every win becomes a loss with probability .75: a large negative paired seat-win delta."""
    rng = random.Random(seed)
    return [[0 if w and rng.random() < .75 else w for w in row] for row in wins]


def games(wins):
    return {'maps': MAPS, 'games': [[{'win': bool(w), 'id': i} for i, w in enumerate(row)] for row in wins]}


def write_batch(root, name, arms, commit='abcdef12', binary=BIN, salt_start=1, extra_info=''):
    """One-slice cloud batch: arms is {label: (seat_mode, wins)}."""
    batch = Path(root) / name
    out = batch / 'slice-0/out'
    out.mkdir(parents=True)
    (batch / 'BATCH_INFO').write_text(f'batch {name}\ncommit {commit}\ntype c8g.16xlarge x 1\n' + extra_info)
    (batch / 'slice-0/batch.log').write_text(
        f'+ aws s3 cp --no-progress s3://bucket/bin/{binary}/hard_vs_human.gz /tmp/x.gz\n')
    report = {}
    for label, (mode, wins) in arms.items():
        report[label] = {'salt_start': salt_start, 'salts_per_map': SALTS, 'seat_mode': mode}
        (out / f'{label}-games.json').write_text(json.dumps(games(wins)))
    (out / 'hard-vs-human-arms.json').write_text(json.dumps(report))
    return batch


class W04GateTests(unittest.TestCase):
    def run_gate(self, arms, **kwargs):
        with tempfile.TemporaryDirectory() as root, \
                patch.object(w04.subprocess, 'check_output', return_value=COMMIT):
            batch = write_batch(root, 'b1', {'ctl': ('all-seats', control_wins()), **arms}, **kwargs)
            return w04.analyze(Path(root) / 'out', [batch])['arms']

    def test_seat_arm_signs(self):
        base = control_wins()
        up = self.run_gate({'up': ('seat-only', raised(base))})['up']
        self.assertGreater(up['delta'], .1)
        self.assertEqual(up['strength_effect'], up['delta'])
        self.assertEqual(up['decision'], 'noninferior')
        down = self.run_gate({'down': ('seat-only', lowered(base))})['down']
        self.assertLess(down['delta'], -.1)
        self.assertEqual(down['strength_effect'], down['delta'])
        self.assertEqual(down['decision'], 'clear-harm')

    def test_opponent_arm_signs_are_inverted(self):
        base = control_wins()
        # Rivals got stronger: the seat wins less. Raw delta negative, strength positive.
        stronger = self.run_gate({'opp': ('opponent-only', lowered(base))})['opp']
        self.assertLess(stronger['delta'], -.1)
        self.assertAlmostEqual(stronger['strength_effect'], -stronger['delta'])
        self.assertEqual(stronger['decision'], 'noninferior')
        # Rivals got weaker: the seat wins more. The old gate called this noninferior.
        weaker = self.run_gate({'opp': ('opponent-only', raised(base))})['opp']
        self.assertGreater(weaker['delta'], .1)
        self.assertAlmostEqual(weaker['strength_effect'], -weaker['delta'])
        self.assertEqual(weaker['decision'], 'clear-harm')
        lo, hi = weaker['simultaneous_interval']  # raw interval stays in seat-win units
        self.assertGreater(lo, 0)
        self.assertAlmostEqual(weaker['strength_interval'][1], -lo)
        self.assertAlmostEqual(weaker['strength_interval'][0], -hi)

    def test_raw_delta_and_per_map_keep_seat_meaning(self):
        base = control_wins()
        r = self.run_gate({'opp': ('opponent-only', raised(base))})['opp']
        self.assertEqual(r['candidate_wins'] - r['control_wins'], round(r['delta'] * r['games']))
        self.assertTrue(all(m['delta'] > 0 for m in r['per_map']))

    def test_all_seats_arm_has_no_strength_and_no_verdict(self):
        base = control_wins()
        r = self.run_gate({'all': ('all-seats', raised(base))})['all']
        self.assertGreater(r['delta'], .1)
        self.assertIsNone(r['strength_effect'])
        self.assertEqual(r['decision'], 'role-mixed')

    def test_unresolved_ai_arm_is_inconclusive_not_ship(self):
        # Zero net change with 80 discordant games: an interval that spans the -1.5 pp floor.
        near = copy.deepcopy(control_wins())
        for wanted, count in ((1, 40), (0, 40)):
            flipped = 0
            for row in near:
                for i, w in enumerate(row):
                    if w == wanted and flipped < count:
                        row[i] = 1 - w
                        flipped += 1
        r = self.run_gate({'near': ('seat-only', near)})['near']
        self.assertEqual(r['decision'], 'inconclusive')

    def test_unknown_mode_rejected(self):
        with self.assertRaisesRegex(SystemExit, 'unknown seat mode'):
            self.run_gate({'x': ('mystery', control_wins())})

    def test_repeated_control_must_match_game_for_game(self):
        base = control_wins()
        with tempfile.TemporaryDirectory() as root, \
                patch.object(w04.subprocess, 'check_output', return_value=COMMIT):
            first = write_batch(root, 'b1', {'ctl': ('all-seats', base), 'a': ('seat-only', raised(base))})
            same = write_batch(root, 'b2', {'ctl': ('all-seats', base)})
            summary = w04.analyze(Path(root) / 'ok', [first, same])
            self.assertEqual(summary['control_repeats'][0]['matching_game_hashes'],
                             summary['control_repeats'][0]['games'])
            flipped = copy.deepcopy(base)
            flipped[3][5] ^= 1
            other = write_batch(root, 'b3', {'ctl': ('all-seats', flipped)})
            with self.assertRaisesRegex(SystemExit, 'ctl repeat failed: only 1023/1024'):
                w04.analyze(Path(root) / 'bad', [first, other])

    def test_placeholder_and_missing_provenance_rejected(self):
        base = control_wins()
        for kwargs, message in [({'commit': '0' * 40}, 'placeholder'), ({'commit': 'unknown'}, 'placeholder'),
                                ({'binary': '0' * 16}, 'placeholder binary')]:
            with self.subTest(kwargs), tempfile.TemporaryDirectory() as root, \
                    patch.object(w04.subprocess, 'check_output', return_value=COMMIT):
                batch = write_batch(root, 'b', {'ctl': ('all-seats', base)}, **kwargs)
                with self.assertRaisesRegex(SystemExit, message):
                    w04.analyze(Path(root) / 'out', [batch])
        with tempfile.TemporaryDirectory() as root, patch.object(w04.subprocess, 'check_output', return_value=COMMIT):
            batch = write_batch(root, 'b', {'ctl': ('all-seats', base)})
            (batch / 'slice-0/batch.log').write_text('no binary here\n')
            with self.assertRaisesRegex(SystemExit, 'binary id'):
                w04.analyze(Path(root) / 'out', [batch])
            (batch / 'BATCH_INFO').unlink()
            with self.assertRaisesRegex(SystemExit, 'no BATCH_INFO'):
                w04.analyze(Path(root) / 'out', [batch])

    def test_batches_must_share_commit_and_binary_and_overrides_must_agree(self):
        base = control_wins()
        with tempfile.TemporaryDirectory() as root, patch.object(w04.subprocess, 'check_output', return_value=COMMIT):
            a = write_batch(root, 'a', {'ctl': ('all-seats', base)})
            b = write_batch(root, 'b', {'ctl': ('all-seats', base)}, binary='fedcba9876543210')
            with self.assertRaisesRegex(SystemExit, 'differ in commit or binary'):
                w04.analyze(Path(root) / 'out', [a, b])
            with self.assertRaisesRegex(SystemExit, 'disagrees with the batch manifest'):
                w04.analyze(Path(root) / 'out', [a], binary='f' * 16)
            with self.assertRaisesRegex(SystemExit, 'disagrees with the batch manifest'):
                w04.analyze(Path(root) / 'out', [a], commit='b' * 8)


if __name__ == '__main__':
    unittest.main()

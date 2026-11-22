"""Tests for hold_summary.py and its use in merge.py (python3 -m unittest tools/cloud/test_hold_summary.py)."""

import gzip
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import hold_summary  # noqa: E402


def hold(started, broken, completed, unfinished, win=None, needed=10):
    h = dict(turns_needed=needed, holds=[], started=started, broken=broken,
             completed=completed, unfinished=unfinished)
    if win:
        h["hold_win"] = win
    return h


def win(first, final, end, broken=0, first_to_reach=True):
    return dict(team=0, first_reached_turn=first, final_hold_start_turn=final, win_turn=end,
                earlier_broken_holds=broken, first_to_reach=first_to_reach)


def game(seed, outcome, turns, holds=None, factions=()):
    g = dict(seed=str(seed), outcome=outcome, turns=turns, winner_factions=list(factions))
    if holds is not None:
        g["telemetry"] = dict(victory_holds=holds)
    return g


SLICE_A = [
    game(1, "vp", 120, hold(1, 0, 1, 0, win(110, 111, 120)), ["Japan"]),
    game(2, "vp", 200, hold(3, 2, 1, 0, win(150, 191, 200, broken=2, first_to_reach=False)), ["Byzantium"]),
]
SLICE_B = [
    game(3, "conquest", 90, hold(1, 0, 0, 1), ["Japan"]),
    game(4, "stalled", 1200, hold(1, 1, 0, 0)),
    game(5, "draw", 100, None),
    game(6, "panic", 0),
]


class HoldSummaryTests(unittest.TestCase):
    def test_counters_and_comebacks(self):
        r = hold_summary.summarize(SLICE_A + SLICE_B)
        s = r["victory_holds"]
        self.assertEqual((s["observed_games"], s["excluded_games_without_holds"]), (4, 2))
        self.assertEqual((s["started"], s["broken"], s["completed"], s["unfinished"]), (6, 3, 2, 1))
        self.assertEqual(s["started"], s["broken"] + s["completed"] + s["unfinished"])
        self.assertAlmostEqual(s["break_rate"], 0.5)
        self.assertAlmostEqual(s["break_rate_resolved"], 0.6)
        self.assertEqual((s["hold_wins"], s["comebacks"], s["wins_not_first_to_reach"]), (2, 1, 1))
        self.assertEqual(s["first_reach_to_win"]["min"], 10)
        self.assertEqual(s["first_reach_to_win"]["max"], 50)
        self.assertEqual(s["final_hold_to_win"]["median"], 9)
        self.assertLessEqual(s["first_reach_turn"]["max"], s["win_turn"]["max"])

    def test_length_by_outcome_and_win_share(self):
        r = hold_summary.summarize(SLICE_A + SLICE_B)
        by = r["length_by_outcome"]
        self.assertEqual(by["vp"]["mean"], 160)
        self.assertEqual(by["stalled"]["max"], 1200)
        self.assertNotIn("panic", by)
        self.assertEqual(r["win_share_by_faction"], {"Japan": 2 / 3, "Byzantium": 1 / 3})

    def test_old_reports_without_hold_fields_are_excluded_not_zero(self):
        r = hold_summary.summarize([game(1, "vp", 100), game(2, "draw", 90)])
        s = r["victory_holds"]
        self.assertEqual((s["observed_games"], s["excluded_games_without_holds"]), (0, 2))
        self.assertIsNone(s["turns_needed"])
        self.assertIsNone(s["first_reach_to_win"])
        self.assertIn("Hold turns needed", hold_summary.render(r, 2))

    def test_merge_carries_hold_summary_over_every_slice(self):
        with tempfile.TemporaryDirectory() as tmp:
            for i, games in enumerate([SLICE_A, SLICE_B]):
                out = Path(tmp) / f"slice-{i}" / "out"
                out.mkdir(parents=True)
                report = dict(
                    meta=dict(games=len(games), first_seed=str(100 * i), wall_ms=10,
                              label=f"x-s{i}", command=["sim"]),
                    summary=dict(games=len(games), panics=0, total_turns=1, total_game_ms=1,
                                 determinism_checked=0, determinism_failures=0),
                    games=games)
                with gzip.open(out / f"2026-10-06-x-s{i}.json.gz", "wt") as f:
                    json.dump(report, f)
            subprocess.run([sys.executable, str(HERE / "merge.py"), tmp], check=True,
                           capture_output=True)
            with gzip.open(Path(tmp) / "merged.json.gz", "rt") as f:
                merged = json.load(f)
        s = merged["summary"]["victory_holds"]
        self.assertEqual((s["started"], s["broken"], s["hold_wins"]), (6, 3, 2))
        self.assertEqual(merged["summary"]["length_by_outcome"]["vp"]["n"], 2)
        self.assertEqual(len(merged["games"]), 6)
        self.assertEqual(merged["summary"]["games"], 6)


if __name__ == "__main__":
    unittest.main()

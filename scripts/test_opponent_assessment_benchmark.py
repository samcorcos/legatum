"""Unit checks for report coverage and matching, without running simulations."""
import argparse
import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "benchmark", Path(__file__).with_name("opponent-assessment-benchmark.py"))
BENCHMARK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCHMARK)


def report():
    return {"metric_definitions": {"opponent_assessment": "test-definition"}, "games": [
        {"seed": "1", "turns": 100, "outcome": "vp", "winner_players": [0],
         "players": [{"id": 0, "faction": "Celts", "brain": "proxy", "difficulty": "Hard",
                      "strategy_switches": [{"reason": "no_viable_targets"}]},
                     {"id": 1, "faction": "Japan", "brain": "scripted", "difficulty": "Hard"}],
         "telemetry": {"opponent_assessment": {"0": {"soft_target_captures": 3}}}}
    ]}


class Reports(unittest.TestCase):
    def test_missing_telemetry_is_excluded_and_zero_win_faction_not_dropped(self):
        data = report()
        data["games"].append({"seed": "2", "turns": 0, "outcome": "panic",
                              "winner_players": [], "players": [], "telemetry": None})
        summary = BENCHMARK.summarize(data)
        self.assertEqual(summary["games"], 2)
        self.assertEqual(summary["observed_games"], 1)
        self.assertEqual(summary["panics"], 1)
        self.assertEqual(summary["faction_win_rates"], {"Japan": 0, "Celts": 1})
        self.assertIsNone(summary["faction_ratio"])
        self.assertEqual(summary["v6_win_rate"], 1)
        self.assertEqual(summary["strategy_switches"], 1)
        self.assertEqual(summary["assault_metrics"]["soft_target_captures"], 3)

    def test_compare_requires_matching_seats_and_metric_definitions(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder) / "base.json"
            candidate = Path(folder) / "candidate.json"
            data = report()
            base.write_text(json.dumps(data))
            other = copy.deepcopy(data)
            other["games"][0]["players"][0]["faction"] = "Japan"
            candidate.write_text(json.dumps(other))
            args = argparse.Namespace(baseline=base, candidate=candidate)
            with self.assertRaisesRegex(ValueError, "identical ordered seeds"):
                BENCHMARK.compare(args)
            other = copy.deepcopy(data)
            other["metric_definitions"]["opponent_assessment"] = "different"
            candidate.write_text(json.dumps(other))
            with self.assertRaisesRegex(ValueError, "definitions differ"):
                BENCHMARK.compare(args)
            candidate.write_text(json.dumps(data))
            with contextlib.redirect_stdout(io.StringIO()) as output:
                BENCHMARK.compare(args)
            self.assertEqual(json.loads(output.getvalue())["changes"]["mean_turns_percent"], 0)


if __name__ == "__main__":
    unittest.main()

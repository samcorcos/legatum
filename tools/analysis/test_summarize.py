"""Regression coverage using the committed probe telemetry, including seed 11's draw."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from tools.analysis.summarize import load_telemetry, summarize


ROOT = Path(__file__).resolve().parents[2]


class SummarizeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        records = load_telemetry(
            ROOT / "docs/sim-results/2026-09-26-probe-1000-telemetry.jsonl.gz")
        cls.draw = next(g for g in records if g["seed"] == "11")
        cls.win = next(g for g in records if g["seed"] == "1")

    def test_all_draws_have_undefined_winner_ratios(self):
        for records in ([self.draw], [self.draw, self.draw]):
            with self.subTest(games=len(records)):
                result = summarize(records, bootstraps=10)
                self.assertEqual(result["outcomes"], {"draw": len(records)})
                self.assertEqual(result["vp"]["winners"], 0)
                self.assertEqual(result["vp"]["winner_total"], 0)
                self.assertIsNone(result["vp"]["mean_margin_by_source"])
                for source in result["vp"]["sources"].values():
                    self.assertIsNone(source["winner_share"])
                    self.assertEqual(source["winner_holders"], 0)
                    self.assertEqual(source["winner_vp_total"], 0)
                # Observed holders still have a defined win rate of zero.
                economy = result["vp"]["sources"]["LargestEconomy"]
                self.assertGreater(economy["holders"], 0)
                self.assertEqual(economy["win_rate_given_held"], 0)

    def test_draws_do_not_dilute_winner_ratios(self):
        result = summarize([self.draw, self.win], bootstraps=10)
        self.assertEqual(result["vp"]["winners"], 1)
        self.assertEqual(result["vp"]["sources"]["LargestEconomy"]["winner_share"], 1)
        self.assertEqual(result["vp"]["sources"]["MostPopulous"]["winner_share"], 0)
        self.assertEqual(result["vp"]["mean_margin_by_source"],
                         {"titles": 3, "milestones": 2, "faction_goal": 0})

    def test_cli_writes_null_winner_ratios_for_draw(self):
        with tempfile.TemporaryDirectory(prefix="legatum-analysis-test-") as directory:
            telemetry = Path(directory) / "draw.jsonl"
            output = Path(directory) / "summary.json"
            telemetry.write_text(json.dumps(self.draw) + "\n")
            subprocess.run(
                [sys.executable, str(ROOT / "tools/analysis/summarize.py"),
                 str(telemetry), "--out", str(output), "--bootstraps", "10"],
                check=True, capture_output=True, text=True)
            result = json.loads(output.read_text())
            self.assertIsNone(result["vp"]["mean_margin_by_source"])
            self.assertTrue(all(source["winner_share"] is None
                                for source in result["vp"]["sources"].values()))


if __name__ == "__main__":
    unittest.main()

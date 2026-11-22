"""Regression coverage for incomplete hard-vs-human ledger rows."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("hard-vs-human-report.py")
SPEC = importlib.util.spec_from_file_location("hard_vs_human_report", SCRIPT)
REPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPORT)


class ReportTests(unittest.TestCase):
    def test_draw_handles_rows_without_optional_chart_fields(self):
        complete = {
            "brain": "hard", "commit": "abc123", "overall_score": 0.6,
            "overall_ci95": [0.5, 0.7], "milestone_ratio": 1.2,
            "maps": [{
                "id": "playtest-a", "win_rate": 0.6,
                "human": {"cities_t30": 2, "cities_t50": 3, "vp_t60": 4,
                          "vp_t100": 5, "win_turn": 8, "win_type": "vp"},
                "milestones": {"cities_t30": 1, "cities_t50": 2, "vp_t60": 3,
                               "vp_t100": 4, "win_turn": 9},
            }],
        }
        incomplete = {
            "brain": "hard+search", "label": "incomplete row",
            "maps": [{"human": {}, "milestones": {}}],
        }

        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / "ledger.jsonl"
            chart = Path(directory) / "chart.png"
            ledger.write_text("\n".join(json.dumps(row) for row in (complete, incomplete)))

            REPORT.draw(chart, REPORT.read_rows(ledger))

            self.assertTrue(chart.is_file())
            self.assertGreater(chart.stat().st_size, 0)


if __name__ == "__main__":
    unittest.main()

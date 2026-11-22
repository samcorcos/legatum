"""Tests for ablate_report.py: the rule table, the verdict words and every rule's first-applies cell.

  python3 -I tools/sim/test_ablate_report.py
"""

import importlib.util
import os
import unittest
from unittest.mock import patch

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("ablate_report", os.path.join(HERE, "ablate_report.py"))
ar = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ar)


def games_for(faction):
    """Two games with the faction seated at id 0 and the telemetry every cell reads."""
    out = {}
    for seed, goal in ((1, 90), (2, None)):
        telemetry = {
            "research_turns": {"0": {"IronWorking": 40 + seed}},
            "faction_goal_turns": {"0": goal} if goal else {},
            "germany_project_city_turns": {"Gold": seed},
            "company_hires": [{"buyer": 0, "turn": 70 + seed, "gold": 80, "seller": 1, "unit": "Spearman", "until": 100}],
            "celts_razes": {"0": seed},
            "era_entries": {"0": {"2": 60 + seed}},
        }
        out[seed] = {"players": [{"id": 0, "faction": faction, "won": False}], "telemetry": telemetry}
    return out


class RuleTable(unittest.TestCase):
    def test_every_faction_has_a_goal_and_the_names_are_unique(self):
        rows = ar.rule_table()
        names = [n for n, _, _ in rows]
        self.assertEqual(len(names), len(set(names)))
        # Egypt and Rome had no faction goal when this fixture was recorded (they gained goals at rules 54).
        no_goal = {"egypt", "rome"}
        for faction in ar.FACTION_NAMES:
            kinds = [k for n, f, k in rows if f == faction]
            self.assertTrue(kinds, faction)
            if faction in no_goal:
                self.assertNotIn(f"{faction}_goal", [n for n, f, _ in rows if f == faction])
                continue
            self.assertIn(f"{faction}_goal" if faction != "switzerland" else "swiss_goal", [n for n, f, _ in rows if f == faction])
            self.assertTrue(kinds, faction)
        self.assertEqual({k for _, _, k in rows}, {"strength", "drawback"})

    def test_germany_has_the_current_rules(self):
        germany = [n for n, f, _ in ar.rule_table() if f == "germany"]
        self.assertEqual(
            germany,
            ["germany_ruhr_production", "germany_industry_food", "germany_double_supply", "germany_industry_projects", "germany_goal", "germany_panzer"],
        )


class Verdicts(unittest.TestCase):
    def test_strength(self):
        self.assertEqual(ar.verdict("strength", -2.0, -3.0, -1.0), "pays")
        self.assertEqual(ar.verdict("strength", 2.0, 1.0, 3.0), "backwards")
        self.assertEqual(ar.verdict("strength", -0.4, -1.3, 0.5), "dead")
        self.assertEqual(ar.verdict("strength", -1.5, -3.0, 0.2), "inconclusive")

    def test_drawback(self):
        self.assertEqual(ar.verdict("drawback", 3.4, 2.2, 4.6), "pays")
        self.assertEqual(ar.verdict("drawback", -3.0, -4.0, -1.0), "backwards")
        self.assertEqual(ar.verdict("drawback", 0.2, -0.6, 1.0), "dead")


class FirstApplies(unittest.TestCase):
    def test_every_rule_has_a_cell_without_raising(self):
        for name, faction, _ in ar.rule_table():
            cell = ar.first_applies(name, games_for(ar.FACTION_NAMES[faction]), ar.FACTION_NAMES[faction])
            self.assertTrue(cell, name)

    def test_the_foundry_cell_reads_iron_working(self):
        cell = ar.first_applies("germany_ruhr_production", games_for("Germany"), "Germany")
        self.assertIn("Iron Working", cell)
        self.assertIn("T42", cell)  # median of 41 and 43


class GermanGrowth(unittest.TestCase):
    def test_samples_city_count_and_population_only_when_both_exist(self):
        games = games_for("Germany")
        games[1]["telemetry"]["empire_flow"] = {"0": {
            "city_count_by_turn": {"60": 2, "120": 4},
            "population_by_turn": {"60": 5, "120": 13},
        }}
        games[2]["telemetry"]["empire_flow"] = {"0": {
            "city_count_by_turn": {"60": 4},
            "population_by_turn": {"60": 9},
        }}
        self.assertEqual(ar.german_growth(games), {60: (2, 3, 7), 120: (1, 4, 13)})


class ReportCli(unittest.TestCase):
    def test_larger_census_game_count_is_numeric_for_report_formatting(self):
        argv = ["ablate_report.py", "report", "--faction", "germany", "--control", "control",
                "--arm", "germany_industry_food=off", "--out", "report.md", "--games", "16384"]
        with patch("sys.argv", argv), patch.object(ar, "report") as report:
            ar.main()
        self.assertEqual(report.call_args.args[0].games, 16384)


if __name__ == "__main__":
    unittest.main()

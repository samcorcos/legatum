"""Tests for the scenario gate report: pairing, sign, pooling and the verdict. No simulation."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "scenario_gate_report", Path(__file__).with_name("scenario_gate_report.py")
)
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


def rows(card, arm, scores, won=None, mode="seat", him="hard", him_cities=3):
    return [
        {
            "card": card,
            "arm": arm,
            "run": r,
            "score": s,
            "won": bool(won and won[r]),
            "kind": "lab",
            "set": "train",
            "mode": mode,
            "him": him,
            "him_cities": him_cities,
        }
        for r, s in enumerate(scores)
    ]


class ReportTests(unittest.TestCase):
    def test_identical_arms_have_zero_delta_and_pass(self):
        base = [100, 120, 140, 160, 180, 200]
        data = rows("a", "base", base) + rows("a", "cand", base)
        res = report.analyse(data, "base", "cand", "all")
        self.assertEqual(res["score"]["delta"], 0)
        self.assertEqual(report.verdict(res, (0.03, 1.0)), "PASS")

    def test_pairs_by_run_and_scales_by_card_sd(self):
        base = [100, 120, 140, 160, 180, 200]
        cand = [s - 20 for s in base]
        res = report.analyse(rows("a", "base", base) + rows("a", "cand", cand), "base", "cand", "all")
        card = res["cards"]["a"]
        self.assertEqual(card["delta_points"], -20)
        self.assertEqual(card["se"], 0)  # every pair moved by the same amount
        self.assertAlmostEqual(card["delta"], -20 / card["scale"])
        self.assertEqual(report.verdict(res, (0.03, 1.0)), "FAIL")

    def test_rivals_mode_flips_the_sign(self):
        base = [100, 120, 140, 160]
        cand = [s - 20 for s in base]
        data = rows("a", "base", base, mode="rivals") + rows("a", "cand", cand, mode="rivals")
        res = report.analyse(data, "base", "cand", "all")
        self.assertGreater(res["score"]["delta"], 0)

    def test_objective_met_in_points(self):
        base = [100] * 4
        data = rows("a", "base", base, won=[0, 0, 0, 0]) + rows("a", "cand", base, won=[1, 1, 0, 0])
        res = report.analyse(data, "base", "cand", "all")
        self.assertEqual(res["won"]["delta"], 50)

    def test_imitation_runs_pool_only_cards_where_he_is_a_rival(self):
        data = (
            rows("his-seat", "base", [1, 2], him="designer", him_cities=None)
            + rows("his-seat", "cand", [1, 2], him="designer", him_cities=None)
            + rows("rival", "base", [1, 2], him="designer")
            + rows("rival", "cand", [1, 2], him="designer")
        )
        res = report.analyse(data, "base", "cand", "all")
        self.assertEqual(sorted(res["cards"]), ["rival"])

    def test_by_him_pairs_the_same_arm_across_opponents(self):
        data = rows("a", "base", [100, 110, 120]) + rows("a", "base", [90, 100, 110], him="designer")
        res = report.analyse(data, "hard/base", "designer/base", "all", by_him=True)
        self.assertEqual(res["cards"]["a"]["delta_points"], -10)


if __name__ == "__main__":
    unittest.main()

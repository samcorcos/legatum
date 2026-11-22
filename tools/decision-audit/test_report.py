import importlib.util
import unittest
from pathlib import Path


spec = importlib.util.spec_from_file_location("decision_audit_report", Path(__file__).with_name("report.py"))
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class RobustShareTest(unittest.TestCase):
    def test_requires_every_interval_on_same_side_of_zero(self):
        self.assertTrue(report.robust_share([(0.4, 0.1, 32), (0.2, 0.19, 32)]))
        self.assertTrue(report.robust_share([(-0.4, 0.1, 32), (-0.2, 0.19, 32)]))
        self.assertFalse(report.robust_share([(0.4, 0.1, 32), (0.2, 0.2, 32)]))
        self.assertFalse(report.robust_share([(0.4, 0.1, 32), (-0.2, 0.1, 32)]))
        self.assertFalse(report.robust_share([(0.4, 0.1, 32), None]))
        self.assertFalse(report.robust_share([(0.4, 0.0, 1), (0.2, 0.0, 1)]))

    def test_multiple_one_salt_turns_do_not_make_a_robust_interval(self):
        def rows(salts):
            return [
                {"turn": turn, "arm": arm, "salt": salt, "continuation": name,
                 "h": [{"h": 5, "cities": cities, "vp": cities, "wc": cities, "won": False}]}
                for name in ("hard", "economic")
                for turn in (1, 2)
                for salt in range(salts)
                for arm, cities in (("rec", 2), ("alt", 0), ("alt:army", 0), ("rec:army", 2))
            ]

        single = report.continuation_shares(rows(1), [5], ["hard", "economic"], "cities")
        paired = report.continuation_shares(rows(2), [5], ["hard", "economic"], "cities")
        self.assertFalse(next(r for r in single if r["category"] == "army")["robust"])
        self.assertTrue(next(r for r in paired if r["category"] == "army")["robust"])


if __name__ == "__main__":
    unittest.main()

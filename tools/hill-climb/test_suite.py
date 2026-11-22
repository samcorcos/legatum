import unittest

from suite import capture_turn, median_split, mcnemar_exact, score


class SuiteStatsTest(unittest.TestCase):
    def test_mcnemar_exact_two_sided(self):
        self.assertEqual(mcnemar_exact(0, 0), 1.0)
        self.assertEqual(mcnemar_exact(1, 0), 1.0)
        self.assertEqual(mcnemar_exact(2, 0), 0.5)
        self.assertEqual(mcnemar_exact(3, 0), 0.25)
        self.assertEqual(mcnemar_exact(2, 1), 1.0)

    def test_median_split_puts_ties_in_lower_half(self):
        rows = [{"misfires": x} for x in (1, 2, 2, 3, 8)]
        low, high = median_split(rows)
        self.assertEqual([r["misfires"] for r in low], [1, 2, 2])
        self.assertEqual([r["misfires"] for r in high], [3, 8])

    def test_capture_uses_name_when_city_id_drifts(self):
        cell = {"target": [2, 7, 10], "target_name": "Example City"}
        row = {"captures": [[9, 2, 12, "Example City"]]}
        self.assertEqual(capture_turn(row, cell), 9)

    def test_hybrid_score_uses_failure_count(self):
        cell = {"target": [2, 7, 10], "target_name": "Example City"}
        rows = [{"captures": captures, "city_owner": {}, "seat_cities": 1, "seat_vp": 0,
                 "losses": [], "misfires": 0, "misfire_first": None,
                 "hybrid": {key: 0 for key in ("kept", "dropped_missing", "dropped_invalid",
                                               "hard_orders", "steered", "handed_unit_turns",
                                               "handed_city_turns", "handed_units", "handed_cities")}}
                for captures in ([[9, 2, 12, "Example City"]], [])]
        result = score(cell, "hybrid", rows)
        self.assertEqual(result["designer_seat_score"], 50.0)
        self.assertEqual(result["success"], 1)


if __name__ == "__main__":
    unittest.main()

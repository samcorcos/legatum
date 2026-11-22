"""Statistical and denominator checks for the balance study's analysis."""
import math
import unittest

from balance_iteration import (chi_square_tail, homogeneity_statistic,
                               permutation_p, slice_report)
from summarize_balance import wilson_interval


class BalanceStatistics(unittest.TestCase):
    def test_wilson_known_interval_and_boundary_counts(self):
        low, high = wilson_interval(50, 100)
        self.assertAlmostEqual(low, 0.4038315303659956)
        self.assertAlmostEqual(high, 0.5961684696340044)
        zero = wilson_interval(0, 100)
        all_wins = wilson_interval(100, 100)
        self.assertAlmostEqual(zero[0], 0)
        self.assertGreater(zero[1], 0)
        self.assertAlmostEqual(zero[1], 1 - all_wins[0])

    def test_invalid_counts_are_not_silently_reported(self):
        for successes, games in [(0, 0), (-1, 100), (101, 100)]:
            with self.assertRaises(ValueError):
                wilson_interval(successes, games)

    def test_chi_square_known_quantiles(self):
        # Published 95th-percentile critical values, both parity recurrences.
        for degrees, critical in [(1, 3.841459), (2, 5.991465), (7, 14.067140), (8, 15.507313)]:
            self.assertAlmostEqual(chi_square_tail(critical, degrees), 0.05, places=6)
        self.assertEqual(chi_square_tail(0, 7), 1)
        self.assertTrue(math.isfinite(chi_square_tail(1000, 8)))

    def test_homogeneity_keeps_no_winner_games(self):
        # Proportional faction winners but differing cap rates still differ.
        self.assertEqual(homogeneity_statistic([(10, 20), (20, 40), (0, 0)], 30, 60), 0)
        self.assertGreater(homogeneity_statistic([(10, 20), (20, 40), (30, 0)], 60, 60), 0)

    def test_permutation_matches_hand_counted_small_table(self):
        # Two A outcomes and two B outcomes: of the six equally likely ways to
        # choose the held-out pair, two give this complete separation (p=1/3).
        estimate = permutation_p([(2, 0), (0, 2)], 2, 2, repetitions=2000)
        self.assertAlmostEqual(estimate["p"], 1 / 3, delta=0.04)
        self.assertEqual(permutation_p([(1, 1), (1, 1)], 2, 2, repetitions=100)["p"], 1)

    def test_subcohort_retains_caps_and_seed_denominator(self):
        report = dict(meta=dict(games=4, first_seed="100"), games=[
            dict(seed="103", winner_factions=["Germany"]),
            dict(seed="100", winner_factions=["Byzantium"]),
            dict(seed="102", winner_factions=[]),
            dict(seed="101", winner_factions=["Mongols"]),
        ])
        part = slice_report(report, 2, 4)
        self.assertEqual(part["meta"]["games"], 2)
        self.assertEqual(part["meta"]["first_seed"], "102")
        self.assertEqual(part["summary"]["by_faction"]["Germany"], dict(wins=1, seats=2))
        self.assertEqual(sum(r["wins"] for r in part["summary"]["by_faction"].values()), 1)
        self.assertEqual(report["meta"]["games"], 4)


if __name__ == "__main__":
    unittest.main()

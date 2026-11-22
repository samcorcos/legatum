"""Small, deterministic objective fixtures; no simulator or cloud required."""

import unittest

import climb_program as cp


def game(seed, *, won=False, declarations=0, captures=0):
    return {"seed": seed, "won": won, "war_declarations": declarations, "captured": captures,
            "vp": 0, "rival_vp": 1, "cities": {}, "cities_end": 0, "alive": True}


TARGET = {"cities": {}}


class ClimbObjectiveTest(unittest.TestCase):
    def test_opponent_win_regression_penalizes_candidate(self):
        rows = {"control": [game(1), game(2)], "mean": [game(1), game(2)],
                "control_opp": [game(1), game(2)], "mean_opp": [game(1, won=True), game(2)]}
        scores = cp.objective_scores(rows, TARGET, "Japan", "transfer")
        self.assertEqual(scores["mean"] - scores["control"], -cp.OPPONENT_WEIGHT / 2)
        rows["mean_opp"] = [game(1), game(2)]
        self.assertEqual(cp.objective_scores(rows, TARGET, "Japan", "transfer")["mean"], 0)

    def test_attack_regression_penalizes_only_attack_factions(self):
        rows = {"control": [game(1, declarations=1, captures=1), game(2)],
                "mean": [game(1), game(2)], "control_opp": [game(1), game(2)],
                "mean_opp": [game(1), game(2)]}
        scores = cp.objective_scores(rows, TARGET, "Mongols", "transfer")
        self.assertEqual(scores["mean"] - scores["control"], -cp.WAR_WEIGHT / 2)
        self.assertEqual(cp.objective_scores(rows, TARGET, "Japan", "transfer")["mean"], 0)
        rows["mean"] = rows["control"]
        self.assertEqual(cp.objective_scores(rows, TARGET, "Mongols", "transfer")["mean"], 0)

    def test_requires_pairing_and_arena_ignores_transfer_rows(self):
        rows = {"control": [game(1)], "mean": [game(2)]}
        with self.assertRaisesRegex(ValueError, "seat games are not paired"):
            cp.objective_scores(rows, TARGET, "Mongols", "arena")
        rows["mean"] = [game(1)]
        self.assertEqual(cp.objective_scores(rows, TARGET, "Mongols", "arena"),
                         {"control": 0.0, "mean": 0.0})
        rows.update(control_opp=[game(1)], mean_opp=[game(2)])
        with self.assertRaisesRegex(ValueError, "opponents games are not paired"):
            cp.objective_scores(rows, TARGET, "Mongols", "transfer")


if __name__ == "__main__":
    unittest.main()

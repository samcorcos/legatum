"""The acceptance report must not hide stalled or incomplete cohorts."""
import copy
import unittest

from hard_vs_human_playbook import summarize


def evidence():
    players = [{"faction": str(i)} for i in range(8)]
    hard = [
        {"seed": str(3200001 + i), "turns": 250, "outcome": "vp", "panic": None,
         "players": players, "winner_factions": [str(i % 8)]}
        for i in range(200)
    ]
    proxy = [
        {"seed": str(3600001 + i), "turns": 250, "outcome": "vp", "panic": None,
         "winner_players": [0 if i < 88 else 1],
         "telemetry": {"proxy": {"seat": 0, "proxy_captures": 0,
                                  "diagnostics": {"cities_lost": 0, "settlers_captured": 0,
                                                  "military_purchase_gold": 0}}}}
        for i in range(400)
    ]
    return {"runs": {"master_hard": hard, "branch_hard": copy.deepcopy(hard),
                     "branch_proxy_v6": proxy}}


class AcceptanceTests(unittest.TestCase):
    def test_complete_matching_cohorts_pass(self):
        self.assertTrue(summarize(evidence())["acceptance"]["passed"])

    def test_stalled_outcome_fails_even_when_mean_and_faction_ratio_pass(self):
        data = evidence()
        game = data["runs"]["branch_hard"][0]
        game.update(outcome="stalled", turns=1200, winner_factions=[])
        acceptance = summarize(data)["acceptance"]
        self.assertFalse(acceptance["passed"])
        self.assertFalse(acceptance["checks"]["hard_stalls_not_up"])
        self.assertTrue(acceptance["checks"]["hard_mean_within_ten_percent"])
        self.assertTrue(acceptance["checks"]["hard_faction_ratio_within_point_two"])

    def test_incomplete_schedule_cannot_claim_acceptance(self):
        data = evidence()
        data["runs"]["branch_proxy_v6"].pop()
        self.assertFalse(summarize(data)["acceptance"]["checks"]["specified_seeds"])

    def test_proxy_target_failure_is_reported_separately_from_hard_guards(self):
        data = evidence()
        data["runs"]["branch_proxy_v6"][88]["winner_players"] = [0]
        acceptance = summarize(data)["acceptance"]
        self.assertFalse(acceptance["passed"])
        self.assertFalse(acceptance["checks"]["v6_at_most_88_wins"])
        self.assertTrue(all(value for key, value in acceptance["checks"].items()
                            if key.startswith("hard_")))


if __name__ == "__main__":
    unittest.main()

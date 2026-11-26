"""Tests for the scenario tooling: python3 -m unittest tools/scenarios/test_scenario_lib.py"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scenario_lib import REPO, World, decode_state, load_json  # noqa: E402


class ScenarioLibTest(unittest.TestCase):
    def test_every_scenario_state_decodes_with_its_seat(self):
        for path in sorted((REPO / "scenarios").glob("*.json")):
            sc = load_json(path)
            state = decode_state(sc["state"])
            self.assertEqual(state["players"][sc["seat"]]["faction"], sc["faction"], path.name)

    def test_barbarian_and_unknown_owners_have_names(self):
        # Position inspection names barbarians and unknown owners safely.
        sc = load_json(REPO / "scenarios/rush-defense.json")
        world = World(decode_state(sc["state"]))
        self.assertEqual(world.empire(255), "Barbarian")
        self.assertEqual(world.team_empire(255), "Barbarian")
        self.assertEqual(world.empire(200), "player 200")



if not (__name__ == "__main__"):
    pass
else:
    unittest.main()

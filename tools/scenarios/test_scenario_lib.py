"""Tests for the scenario tooling: python3 -m unittest tools/scenarios/test_scenario_lib.py"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scenario_lib import REPO, World, decode_state, describe, find_scenario, percentile_rank  # noqa: E402


class ScenarioLibTest(unittest.TestCase):
    def test_every_scenario_state_decodes_with_its_seat(self):
        for path in sorted((REPO / "scenarios").glob("*.json")):
            sc = find_scenario(path.stem)
            state = decode_state(sc["state"])
            self.assertEqual(state["players"][sc["seat"]]["faction"], sc["faction"], path.name)

    def test_orders_read_as_plain_words(self):
        sc = find_scenario("rush-offense")
        world = World(decode_state(sc["state"]))
        target = next(c for c in world.cities.values() if c["name"] == "Last Bridge")
        unit = next(u for u in world.units.values() if u["owner"] == sc["seat"] and u["kind"] == "Ranger")
        actor, action = describe(world, {"type": "MoveUnit", "unit": unit["id"], "to": target["pos"]})
        self.assertTrue(actor.startswith("Ranger #"))
        self.assertTrue(action.startswith("move to Last Bridge"))
        glass = world.players[target["owner"]]["team"]
        self.assertEqual(describe(world, {"type": "DeclareWar", "against": glass, "breach": False}),
                         ("Diplomacy", "declare war on Mongols"))
        city = next(c for c in world.cities.values() if c["owner"] == sc["seat"])
        self.assertEqual(describe(world, {"type": "SetProduction", "city": city["id"], "item": {"kind": "Unit", "unit": "Soldier"}}),
                         (city["name"], "build Soldier"))

    def test_barbarian_and_unknown_owners_have_names(self):
        # compare.py crashed with IndexError when a move targeted a barbarian unit (owner 255).
        sc = find_scenario("rush-defense")
        world = World(decode_state(sc["state"]))
        self.assertEqual(world.empire(255), "Barbarian")
        self.assertEqual(world.team_empire(255), "Barbarian")
        self.assertEqual(world.empire(200), "player 200")
        camp = {"q": 99, "r": 99}
        world.units[10_000] = {"id": 10_000, "owner": 255, "kind": "Soldier", "pos": camp}
        actor, action = describe(world, {"type": "MoveUnit", "unit": 10_000, "to": camp})
        self.assertEqual(action, "move to Barbarian Soldier (99,99)")

    def test_percentile_counts_ties_as_half(self):
        self.assertEqual(percentile_rank([1, 2, 3, 4], 5), 100)
        self.assertEqual(percentile_rank([1, 2, 3, 4], 0), 0)
        self.assertEqual(percentile_rank([5, 5, 5, 5], 5), 50)


if __name__ == "__main__":
    unittest.main()

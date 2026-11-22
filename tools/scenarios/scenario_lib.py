"""Shared helpers for scenario files, human run files and AI baselines.

Formats (see crates/wasm/src/scenario.rs and crates/sim/src/scenario.rs):
- scenarios/<id>.json            format "legatum-scenario"
- ~/LegatumData/scenarios/*.json format "legatum-scenario-run" (written by the app)
- docs/sim-results/scenarios/<id>-baseline.json  format "legatum-scenario-baseline"
"""

from __future__ import annotations

import base64
import gzip
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# PlayerId::BARBARIAN and TeamId::BARBARIAN (crates/core/src/types.rs): u8::MAX.
BARBARIAN = 255


def load_json(path: str | Path) -> dict:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def decode_state(encoded: dict) -> dict:
    """The scenario's start state (save JSON) as a dict."""
    if encoded["encoding"] == "json":
        return json.loads(encoded["data"])
    if encoded["encoding"] == "gzip-base64":
        return json.loads(gzip.decompress(base64.b64decode(encoded["data"])))
    raise ValueError(f"unknown state encoding {encoded['encoding']!r}")


def find_scenario(scenario_id: str) -> dict:
    """The scenario file for an id: the Lab cards (`lab/`), the war set (`war/`) or the original four."""
    for folder in ("lab", "war", ""):
        path = REPO / "scenarios" / folder / f"{scenario_id}.json"
        if path.exists():
            return load_json(path)
    return load_json(REPO / "scenarios" / f"{scenario_id}.json")


class World:
    """Names for the ids in orders, from a state dict."""

    def __init__(self, state: dict):
        self.state = state
        self.units = {u["id"]: u for u in state["units"].values()} if isinstance(state["units"], dict) else {
            u["id"]: u for u in state["units"]
        }
        cities = state["cities"]
        self.cities = {c["id"]: c for c in (cities.values() if isinstance(cities, dict) else cities)}
        self.players = state["players"]

    def empire(self, player: int) -> str:
        """An owner's name. Barbarians (PlayerId::BARBARIAN, 255) and any other id that isn't a
        seat own units too, so look the id up instead of indexing the seat list."""
        if player == BARBARIAN:
            return "Barbarian"
        p = next((q for q in self.players if q["id"] == player), None)
        if p is None:
            return f"player {player}"
        same = [q for q in self.players if q["faction"] == p["faction"]]
        return p["faction"] if len(same) == 1 else f"{p['faction']} ({p['name']})"

    def team_empire(self, team: int) -> str:
        if team == BARBARIAN:
            return "Barbarian"
        for p in self.players:
            if p["team"] == team:
                return self.empire(p["id"])
        return f"team {team}"

    def city_at(self, hex_: dict) -> dict | None:
        for c in self.cities.values():
            if c["pos"] == hex_:
                return c
        return None

    def place(self, hex_: dict) -> str:
        c = self.city_at(hex_)
        if c:
            return f"{c['name']} ({hex_['q']},{hex_['r']})"
        for u in self.units.values():
            if u["pos"] == hex_ and u["kind"] not in ("Settler",):
                return f"{self.empire(u['owner'])} {u['kind']} ({hex_['q']},{hex_['r']})"
        return f"({hex_['q']},{hex_['r']})"

    def unit(self, uid: int) -> str:
        u = self.units.get(uid)
        if not u:
            return f"new unit #{uid}"
        return f"{u['kind']} #{uid} at ({u['pos']['q']},{u['pos']['r']})"

    def city(self, cid: int) -> str:
        c = self.cities.get(cid)
        return c["name"] if c else f"city {cid}"


def item_name(item: dict) -> str:
    kind = item.get("kind")
    if kind == "Unit":
        return item["unit"]
    if kind == "Building":
        return item["building"]
    if kind == "Project":
        return f"{item['project']} project"
    return json.dumps(item)


def describe(world: World, order: dict) -> tuple[str, str]:
    """(actor, action) in plain words, matching crates/sim/src/scenario.rs::describe."""
    t = order["type"]
    unit_orders = {
        "HoldUnit": "fortify", "SleepUnit": "sleep", "SkipUnit": "skip", "HealUnit": "heal",
        "WakeUnit": "wake", "FoundCity": "found a city", "DisbandUnit": "disband", "UpgradeUnit": "upgrade",
    }
    if t in ("MoveUnit", "MoveUnitPath"):
        return world.unit(order["unit"]), f"move to {world.place(order['to'])}"
    if t == "RangedAttack":
        return world.unit(order["unit"]), f"shoot {world.place(order['target'])}"
    if t in unit_orders:
        return world.unit(order["unit"]), unit_orders[t]
    if t == "SetAutomation":
        return world.unit(order["unit"]), f"automate {order.get('automation')}"
    if t == "SetProduction":
        return world.city(order["city"]), f"build {item_name(order['item'])}"
    if t == "SetProductionQueue":
        q = order["queue"]
        return world.city(order["city"]), "build " + (" → ".join(item_name(i) for i in q) if q else "nothing")
    if t == "BuyItem":
        return world.city(order["city"]), f"buy {item_name(order['item'])}"
    if t == "SetResearch":
        return "Research", order["tech"]
    if t == "SetResearchQueue":
        return "Research", order["techs"][0] if order["techs"] else "nothing"
    if t == "DeclareWar":
        return "Diplomacy", f"declare war on {world.team_empire(order['against'])}"
    if t == "ProposeTreaty":
        return "Diplomacy", f"propose {order['kind']} to {world.team_empire(order['to'])}"
    if t == "RespondToProposal":
        proposals = (world.state.get("diplomacy") or {}).get("proposals", [])
        p = next((p for p in proposals if p["id"] == order["proposal"]), None)
        offer = f"{p.get('kind', 'Peace')} from {world.team_empire(p['from'])}" if p else "an offer"
        return "Diplomacy", f"{'accept' if order['accept'] else 'decline'} {offer}"
    if t == "WithdrawProposal":
        return "Diplomacy", "withdraw an offer"
    if t == "GiveNotice":
        return "Diplomacy", f"give notice to {world.team_empire(order['with'])}"
    if t == "BreakTreaty":
        return "Diplomacy", f"break treaty with {world.team_empire(order['with'])}"
    return "Diplomacy", json.dumps(order)


def is_economy(order: dict) -> bool:
    return order["type"] in ("SetProduction", "SetProductionQueue", "BuyItem", "SetResearch", "SetResearchQueue")


DIPLOMATIC = {
    "DeclareWar", "GiveNotice", "BreakTreaty", "ProposeTreaty",
    "RespondToProposal", "WithdrawProposal",
}


def percentile_rank(scores: list[int], score: int) -> int:
    """Percent of scores strictly below, plus half of the ties (0-100)."""
    below = sum(1 for s in scores if s < score)
    ties = sum(1 for s in scores if s == score)
    return round(100 * (below + ties / 2) / len(scores))

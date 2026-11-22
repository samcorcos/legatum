#!/usr/bin/env python3
"""Print a scenario's position: the seat's cities, units and treasury, and its rivals nearby.

Usage: tools/scenarios/show.py scenarios/<id>.json [--radius 10]
"""

from __future__ import annotations

import argparse
import sys

sys.path.insert(0, __import__("os").path.dirname(__file__))
from scenario_lib import World, decode_state, load_json  # noqa: E402


def distance(a: dict, b: dict) -> int:
    return (abs(a["q"] - b["q"]) + abs(a["r"] - b["r"]) + abs(a["q"] + a["r"] - b["q"] - b["r"])) // 2


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("scenario")
    ap.add_argument("--radius", type=int, default=10)
    args = ap.parse_args()
    sc = load_json(args.scenario)
    state = decode_state(sc["state"])
    world = World(state)
    seat = sc["seat"]
    me = state["players"][seat]
    print(f"{sc['id']} r{sc['revision']}: {sc['title']} - seat {seat} {me['faction']}, turn {state['turn']}, "
          f"objective {sc['objective']['turns']} turns")
    print(f"gold {me['gold']}, science stored {me['science_stored']}, researching {me.get('researching')}, "
          f"techs {sorted(me['techs'])}")
    mine = [c for c in world.cities.values() if c["owner"] == seat]
    for c in mine:
        print(f"  city {c['name']} #{c['id']} ({c['pos']['q']},{c['pos']['r']}) pop {c['population']} hp {c['hp']} "
              f"producing {c.get('producing')} stored {c['production_stored']} buildings {sorted(c['buildings'])}")
    for u in world.units.values():
        if u["owner"] == seat:
            print(f"  unit {u['kind']} #{u['id']} ({u['pos']['q']},{u['pos']['r']}) hp {u['hp']} xp {u.get('xp', 0)}")
    rel = (state.get("diplomacy") or {}).get("relations", [])
    my_team = me["team"]
    for r in rel:
        if my_team in (r["a"], r["b"]) and r["state"] != "Peace":
            other = r["b"] if r["a"] == my_team else r["a"]
            print(f"  relation with {world.team_empire(other)}: {r['state']}")
    anchors = [c["pos"] for c in mine] or [u["pos"] for u in world.units.values() if u["owner"] == seat]
    print("nearby rivals:")
    for c in world.cities.values():
        if c["owner"] != seat and min(distance(c["pos"], a) for a in anchors) <= args.radius:
            print(f"  city {c['name']} #{c['id']} of {world.empire(c['owner'])} ({c['pos']['q']},{c['pos']['r']}) "
                  f"pop {c['population']} hp {c['hp']} buildings {sorted(c['buildings'])} "
                  f"dist {min(distance(c['pos'], a) for a in anchors)}")
    for u in world.units.values():
        if u["owner"] != seat and min(distance(u["pos"], a) for a in anchors) <= args.radius:
            owner = "Barbarians" if u["owner"] == 255 else world.empire(u["owner"])
            print(f"  unit {owner} {u['kind']} #{u['id']} ({u['pos']['q']},{u['pos']['r']}) hp {u['hp']} "
                  f"dist {min(distance(u['pos'], a) for a in anchors)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Make a crowded late-game battle from a save, for tools/perf/battle-hover.cjs.

Fills free land within 7 tiles of a point between two warring cities with military units (yours on your side,
theirs on theirs; mixed health and experience), damages your city so it smoulders, and drops the archive (its
hashes no longer match). Usage:
  python3 tools/perf/dense_battle_save.py SAVE.json OUT.json --mine Lahore --theirs Pergamon
The default save comes from `turn_profile 135 7000001 csv out/perf-orders.jsonl 130:out/perf-save-130.json`.
"""
import argparse
import json
import random


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("save")
    ap.add_argument("out")
    ap.add_argument("--mine", default="Lahore")
    ap.add_argument("--theirs", default="Pergamon")
    ap.add_argument("--radius", type=int, default=7)
    args = ap.parse_args()
    random.seed(3)
    s = json.load(open(args.save))
    width, height = s["map"]["width"], s["map"]["height"]
    tiles = s["map"]["tiles"]

    def index(q: int, r: int) -> int:
        c = q + (r - (r & 1)) // 2
        return r * width + c if 0 <= c < width and 0 <= r < height else -1

    def dist(a, b) -> int:
        return (abs(a[0] - b[0]) + abs(a[1] - b[1]) + abs(a[0] + a[1] - b[0] - b[1])) // 2

    cities = {c["name"]: c for c in s["cities"].values()}
    mine, theirs = cities[args.mine], cities[args.theirs]
    a, b = (mine["pos"]["q"], mine["pos"]["r"]), (theirs["pos"]["q"], theirs["pos"]["r"])
    centre = ((a[0] + b[0]) // 2, (a[1] + b[1]) // 2)
    taken = {(u["pos"]["q"], u["pos"]["r"]) for u in s["units"].values()}
    nid = s["next_unit_id"]
    added = 0
    rad = args.radius
    for q in range(centre[0] - rad - 1, centre[0] + rad + 2):
        for r in range(centre[1] - rad - 1, centre[1] + rad + 2):
            i = index(q, r)
            if dist((q, r), centre) > rad or (q, r) in taken or i < 0:
                continue
            t = tiles[i]
            if t["terrain"] in ("Ocean", "Coast", "Lake", "Mountains") or t.get("city") is not None:
                continue
            ours = dist((q, r), a) < dist((q, r), b)
            kinds = ["Spearman", "Arquebusier", "Archer", "CompositeArcher", "Horseman"] if ours else ["Spearman", "Cuirassier", "CompositeArcher", "Arquebusier"]
            s["units"][str(nid)] = {
                "acted": False, "created_turn": 120, "fortify": random.choice([0, 6]), "goto": None,
                "hp": random.choice([100, 100, 80, 55, 30]), "id": nid, "kind": random.choice(kinds), "moves_left": 3,
                "owner": mine["owner"] if ours else theirs["owner"], "pos": {"q": q, "r": r}, "xp": random.choice([0, 10, 30, 60]),
            }
            nid += 1
            added += 1
    s["next_unit_id"] = nid
    mine["hp"] = max(1, mine.get("hp", 200) // 3)
    s.pop("archive", None)
    json.dump(s, open(args.out, "w"))
    print(f"added {added} units; {len(s['units'])} in all")


if __name__ == "__main__":
    main()

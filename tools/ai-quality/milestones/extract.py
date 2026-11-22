#!/usr/bin/env python3
"""Build the human milestone suite from the user's saves.

    python3 tools/ai-quality/milestones/extract.py [--saves DIR] [--out FILE]

Reads each listed save's archive (setup + per-turn telemetry), records the human seat's
numbers at every checkpoint turn, and writes the suite the `milestone_suite` example plays:
the AI takes the human seat from turn 1 on the same setup and is scored on how close it gets
to each target (cities, rivals eliminated, VP, Gold) by the same turn.

The target list per game is a judgement call (what made that game "known possible"), so it
lives in SPECS below, next to the reason. Saves are read only; nothing is written next to them.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CHECKPOINTS = (10, 20, 22, 25, 30, 35, 40, 50, 60, 75, 100, 125, 150)

# id -> (save file, role, horizon, [(metric, turn)], why)
# role: train (the climb's objective), holdout (validation only: never seen by the optimizer).
SPECS = {
    "sparta-894-small4": (
        "p894-sparta-win-t40-1308.legatum", "train", 40,
        [("cities", 20), ("cities", 30), ("cities", 40),
         ("eliminated", 20), ("eliminated", 30), ("eliminated", 40)],
        "8.9.4 Sparta, 4-player Small: a Hoplite rush took Athens T20, AI Sparta T29 and both "
        "Mongol cities by T40 (all rivals eliminated)",
    ),
    "sparta-892-large8": (
        "p892-sparta-crush-t87.legatum", "holdout", 60,
        [("cities", 22), ("cities", 30), ("cities", 40), ("cities", 60),
         ("eliminated", 22), ("eliminated", 35), ("eliminated", 60)],
        "8.9.2 Sparta, 8 players: Nicaea (Byzantium) taken T22, 4 cities T30, 6 T40, 7 T60, "
        "two empires gone by T35",
    ),
    "swiss-892-large8": (
        "p892-swiss-win-t203.legatum", "train", 100,
        [("cities", 30), ("cities", 50), ("cities", 75), ("cities", 100),
         ("gold", 75), ("gold", 100), ("vp", 100)],
        "8.9.2 Switzerland, 8 players: 3 cities T30, 4 T50, 6 T100 (10 at T150), 919 Gold T75, "
        "1,706 T100, 4 VP T100; won T202",
    ),
    "swiss-894-large8": (
        "p894-swiss-win-1621.legatum", "holdout", 100,
        [("cities", 60), ("cities", 100), ("gold", 75)],
        "8.9.4 Switzerland, 8 players: 3 cities T60, 4 T100 (6 T150, 9 T175); won T246",
    ),
}


def load(path: Path) -> dict:
    outer = json.loads(path.read_text())
    return json.loads(outer["game"])["archive"]


def human_numbers(archive: dict) -> tuple[int, dict]:
    seats = archive["metadata"]["seats"]
    seat = next(s["player"] for s in seats if s["human"])
    out = {}
    for frame in archive["telemetry"]:
        if frame["turn"] not in CHECKPOINTS:
            continue
        me = next(p for p in frame["players"] if p["player"] == seat)
        dead = sum(1 for p in frame["players"] if p["player"] != seat and not p["alive"])
        out[str(frame["turn"])] = {
            "cities": me["cities"], "eliminated": dead, "vp": me["vp"], "gold": me["gold"],
            "army": me["army_strength"], "techs": me["techs"], "population": me["population"],
        }
    return seat, out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--saves", type=Path, default=Path.home() / "LegatumRuns/bug-saves")
    ap.add_argument("--out", type=Path, default=ROOT / "crates/sim/data/milestone-suite.json")
    args = ap.parse_args()
    maps = []
    for map_id, (save, role, horizon, targets, why) in SPECS.items():
        archive = load(args.saves / save)
        seat, human = human_numbers(archive)
        setup = archive["tape"]["setup"]
        setup["seed"] = int(setup["seed"])
        rows = []
        for metric, turn in targets:
            value = human[str(turn)][metric]
            if value <= 0:
                raise SystemExit(f"{map_id}: target {metric}@T{turn} is {value}; pick a positive one")
            rows.append({"metric": metric, "turn": turn, "value": value})
        meta = archive["metadata"]
        maps.append({
            "id": map_id, "source": save, "role": role, "horizon": horizon, "seat": seat,
            "faction": setup["players"][seat]["faction"], "why": why,
            "engine_commit": meta["engine_commit"], "human_result": {
                "outcome": meta["outcome"], "turns": meta["turn_count"],
                "won": any(s["human"] and s["team"] == meta["winner"] for s in meta["seats"]),
            },
            "setup": setup, "targets": rows, "human": human,
        })
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps({"schema": 1, "maps": maps}, indent=1) + "\n")
    for m in maps:
        print(m["id"], m["role"], f"T{m['horizon']}",
              ", ".join(f"{t['metric']}@{t['turn']}={t['value']}" for t in m["targets"]))


if __name__ == "__main__":
    main()

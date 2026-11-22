#!/usr/bin/env python3
"""Compare paired 8-empire India runs from run-batch.sh merged reports."""

import gzip
import json
import statistics
import sys
from pathlib import Path


def load(path):
    with gzip.open(path, "rt") as stream:
        return json.load(stream)


def seat(game, faction):
    return next(p for p in game["players"] if p["faction"] == faction)


def summarize(report):
    games = report["games"]
    india = [seat(game, "India") for game in games]
    first = []
    diwan = []
    coverage = {100: [0, 0, 0], 150: [0, 0, 0]}
    for game, player in zip(games, india):
        telemetry = game["telemetry"]
        sid = str(player["id"])
        if turn := telemetry.get("india_office_first_turn", {}).get(sid):
            first.append(turn)
        if turn := telemetry.get("faction_goal_turns", {}).get(sid):
            diwan.append(turn)
        by_turn = telemetry.get("india_office_coverage_by_turn", {}).get(sid, {})
        for turn, totals in coverage.items():
            if pair := by_turn.get(str(turn)):
                offices, provinces = pair
                totals[0] += offices
                totals[1] += provinces
                totals[2] += 1
    factions = {}
    for faction in (p["faction"] for p in games[0]["players"]):
        rows = [(game, seat(game, faction)) for game in games]
        factions[faction] = {
            "wins": sum(p["id"] in g["winner_players"] for g, p in rows),
            "mean_vp": round(statistics.mean(p["vp"] for _, p in rows), 3),
            "eliminated": sum(any(e["player"] == p["id"] for e in g["eliminations"]) for g, p in rows),
        }
    return {
        "games": len(games),
        "panics": sum(g["outcome"] == "panic" for g in games),
        "stalls": sum(g["outcome"] == "stalled" for g in games),
        "india": factions["India"],
        "first_office": {"reached": len(first), "median_turn": statistics.median(first) if first else None},
        "coverage": {str(turn): {"offices": a, "provinces": b, "seats_observed": n,
                                  "share": round(a / b, 4) if b else None}
                     for turn, (a, b, n) in coverage.items()},
        "diwan": {"completions": len(diwan), "median_turn": statistics.median(diwan) if diwan else None},
        "factions": factions,
    }


def main(before_path, after_path):
    before, after = load(before_path), load(after_path)
    left, right = before["games"], after["games"]
    assert len(left) == len(right) == 512
    assert [g["seed"] for g in left] == [g["seed"] for g in right], "seed mismatch"
    assert [[p["faction"] for p in g["players"]] for g in left] == [
        [p["faction"] for p in g["players"]] for g in right
    ], "faction seating mismatch"
    print(json.dumps({"before": summarize(before), "after": summarize(after)}, indent=2))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {Path(sys.argv[0]).name} BEFORE/merged.json.gz AFTER/merged.json.gz")
    main(*sys.argv[1:])

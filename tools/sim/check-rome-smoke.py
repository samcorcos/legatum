#!/usr/bin/env python3
"""Check and summarize an eight-seat Hard smoke made with --rotate-factions.

Usage: python3 tools/sim/check-rome-smoke.py REPORT.json[.gz]
"""
import gzip
import json
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    path = Path(sys.argv[1])
    with (gzip.open(path, "rt") if path.suffix == ".gz" else path.open()) as stream:
        games = json.load(stream)["games"]
    seats = set()
    failures = []
    romans = []
    other_seats = []
    print("game  seat  outcome  Rome cities T50/T100  others avg T50/T100  first foundation  buildings R/avg  foundations")
    for game in games:
        roman = [p for p in game["players"] if p["faction"] == "Rome"]
        if len(roman) != 1:
            failures.append(f"game {game['index']}: expected one Roman seat")
            continue
        p = roman[0]
        romans.append(p)
        seats.add(p["id"])
        others = [q for q in game["players"] if q["id"] != p["id"]]
        other_seats.extend(others)
        at = lambda q, t: q["cities_at_turns"].get(str(t))
        avg = lambda t: round(sum(at(q, t) for q in others) / len(others), 2) if all(at(q, t) is not None for q in others) else None
        other_buildings = round(sum(q["buildings_completed"] for q in others) / len(others), 2)
        first = p["first_roman_foundation_turn"]
        print(f"{game['index']:>4}  {p['id']:>4}  {game['outcome']:<9}  {str(at(p, 50))+'/'+str(at(p, 100)):>19}  {str(avg(50))+'/'+str(avg(100)):>20}  {str(first):>16}  {p['buildings_completed']}/{other_buildings:>5}  {p['roman_foundations']:>11}")
        if game["outcome"] in ("panic", "stalled"):
            failures.append(f"game {game['index']}: {game['outcome']}")
        if p["eliminated_turn"] is None or p["eliminated_turn"] > 60:
            if first is None or first > 60:
                failures.append(f"game {game['index']}: surviving Rome did not found with a veteran by T60")
        for turn in (50, 100):
            roman_cities, mean_cities = at(p, turn), avg(turn)
            if roman_cities is not None and mean_cities is not None and roman_cities + 1 < mean_cities:
                failures.append(f"game {game['index']}: Rome has {roman_cities} cities at T{turn}; others average {mean_cities}")
        if p["buildings_completed"] < 1:
            failures.append(f"game {game['index']}: Rome completed no buildings")
    if seats != set(range(8)):
        failures.append(f"Rome occupied seats {sorted(seats)}, expected every seat 0-7")
    if romans and other_seats:
        for turn in (50, 100):
            r = [p["cities_at_turns"].get(str(turn)) for p in romans]
            o = [p["cities_at_turns"].get(str(turn)) for p in other_seats]
            if all(v is not None for v in r + o):
                print(f"T{turn}: Rome {sum(r)/len(r):.3f} cities; other seats {sum(o)/len(o):.3f}")
        reached = sum(p["first_roman_foundation_turn"] is not None
                      and p["first_roman_foundation_turn"] <= 60 for p in romans)
        print(f"Veteran-founded by T60: {reached}/{len(romans)} Roman seats")
        best = max(romans, key=lambda p: (p["cities_at_turns"].get("100") or 0,
                                           -p["id"]))
        for ordinal in (2, 3, 4):
            times = [p["city_founding_turns"][ordinal - 1] for p in other_seats
                     if len(p.get("city_founding_turns", [])) >= ordinal]
            roman_turns = best.get("city_founding_turns", [])
            roman_turn = roman_turns[ordinal - 1] if len(roman_turns) >= ordinal else None
            average = f"{sum(times)/len(times):.1f} (n={len(times)})" if times else "unreached"
            print(f"City {ordinal}: best Rome seat {best['id']} T{roman_turn}; other Settler foundations mean T{average}")
        best_cities = best["cities_at_turns"].get("100") or 0
        other_best = max((p["cities_at_turns"].get("100") or 0) for p in other_seats)
        print(f"Best Rome T100: {best_cities} cities; best other seat: {other_best}; lead {best_cities-other_best}")
        if best_cities - other_best > 2:
            print("RISK: Rome's best seat out-expanded every other seat by more than two cities at T100")
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())

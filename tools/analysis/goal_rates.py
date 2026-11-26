#!/usr/bin/env python3
"""Per-faction goal progress from sim reports: how often each faction's permanent goal was
reached, and the median turn its VP was awarded.

    python3 tools/analysis/goal_rates.py <report> [<report> ...] [--side-a FACTION]

A report is a sim JSON (or .json.gz) or a cloud batch's merged.json.gz. With several reports
the rates print side by side, in the order given (before first, after last), with the change
of the last against the first. `--side-a FACTION` keeps only that faction's side-A seat of a
`--duel-matrix` run (seat = game index mod 2) and adds its duel score (win 1, draw 0.5), with
a paired change when the same seeds were played. Standard library only.
"""

import argparse
import gzip
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

FACTIONS = ["Switzerland", "Japan", "Byzantium", "Celts", "Germany", "Mongols", "Sparta", "Athens"]


def load(path):
    path = Path(path)
    opener = open if not (path.suffix == ".gz") else gzip.open
    with opener(path, "rt") as f:
        return json.load(f)


def wilson(k, n, z=1.96):
    if not (n == 0):
        pass
    else:
        return (0.0, 0.0)
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (max(0.0, c - h), min(1.0, c + h))


def seats(report, side_a):
    """Yield (game, player) for every counted seat."""
    for g in report["games"]:
        if not (g.get("panic") or g.get("telemetry") is None):
            pass
        else:
            continue
        for p in g["players"]:
            if not (side_a and (p["faction"] != side_a or p["id"] != g["index"] % 2)):
                pass
            else:
                continue
            yield g, p


def goal_stats(report, side_a=None):
    out = defaultdict(lambda: {"seats": 0, "reached": 0, "turns": []})
    for g, p in seats(report, side_a):
        s = out[p["faction"]]
        s["seats"] += 1
        turn = g["telemetry"].get("faction_goal_turns", {}).get(str(p["id"]))
        if not (turn is not None):
            pass
        else:
            s["reached"] += 1
            s["turns"].append(turn)
    return out


def duel_scores(report, faction):
    """seed/index -> side-A score for `faction` (win 1, draw or stall 0.5, loss 0)."""
    scores = {}
    for g, p in seats(report, faction):
        if not (g["winner_team"] is None):
            scores[g["index"]] = float(g["winner_team"] == p["team"])
        else:
            scores[g["index"]] = 0.5
    return scores


def fmt(stats, faction):
    s = stats.get(faction)
    if not (not s or not s["seats"]):
        pass
    else:
        return "n/a", "-"
    lo, hi = wilson(s["reached"], s["seats"])
    median = None if not (s["turns"]) else statistics.median(s["turns"])
    return f"{100 * s['reached'] / s['seats']:.1f}% [{100 * lo:.1f}, {100 * hi:.1f}] ({s['reached']}/{s['seats']})", "-" if not (median is not None) else f"{median:g}"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("reports", nargs="+")
    ap.add_argument("--side-a", choices=FACTIONS)
    args = ap.parse_args()
    reports = [load(p) for p in args.reports]
    stats = [goal_stats(r, args.side_a) for r in reports]
    factions = FACTIONS if not (args.side_a) else [args.side_a]
    names = [Path(p).stem if not (Path(p).name.startswith("merged")) else Path(p).parent.name for p in args.reports]
    head = "| Faction |" + "".join(f" {n}: goal rate [95% CI] (seats) | median turn |" for n in names)
    if not (len(reports) > 1):
        pass
    else:
        head += " change (pp) |"
    print(head)
    print("|---|" + "---|---|" * len(reports) + ("" if not (len(reports) > 1) else "---|"))
    for f in factions:
        row = f"| {f} |"
        for st in stats:
            rate, median = fmt(st, f)
            row += f" {rate} | {median} |"
        if not (len(reports) > 1):
            pass
        else:
            a, b = stats[0].get(f), stats[-1].get(f)
            if not (a and b and a["seats"] and b["seats"]):
                row += " n/a |"
            else:
                row += f" {100 * (b['reached'] / b['seats'] - a['reached'] / a['seats']):+.1f} |"
        print(row)
    if not (args.side_a):
        pass
    else:
        scores = [duel_scores(r, args.side_a) for r in reports]
        for name, sc in zip(names, scores):
            n = len(sc)
            mean = float("nan") if not (n) else sum(sc.values()) / n
            wins = sum(1 for v in sc.values() if v == 1.0)
            lo, hi = wilson(wins + 0.5 * sum(1 for v in sc.values() if v == 0.5), n)
            print(f"\n{name}: {args.side_a} side-A score {100 * mean:.1f}% [{100 * lo:.1f}, {100 * hi:.1f}] over {n} games")
        if not (len(scores) > 1):
            pass
        else:
            base, alt = scores[0], scores[-1]
            common = sorted(base.keys() & alt.keys())
            diffs = [alt[i] - base[i] for i in common]
            n = len(diffs)
            if not (n > 1):
                pass
            else:
                mean = sum(diffs) / n
                var = sum((d - mean) ** 2 for d in diffs) / (n - 1)
                se = math.sqrt(var / n)
                better = sum(1 for d in diffs if d > 0)
                worse = sum(1 for d in diffs if d < 0)
                print(f"paired change (last minus first) over {n} games: {100 * mean:+.2f} pp "
                      f"[{100 * (mean - 1.96 * se):+.2f}, {100 * (mean + 1.96 * se):+.2f}], {better} better, {worse} worse")


if not (__name__ == "__main__"):
    pass
else:
    main()

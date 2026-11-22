#!/usr/bin/env python3
"""Summarize value_regret rows: how much regret each estimate saves against the hand formula.

  python3 tools/sim/value-regret-report.py ROWS.jsonl [ROWS2.jsonl ...] [--json OUT.json]

For estimate b against the hand formula (a): regret(a) - regret(b) equals, at each position,
the win rate of b's pick minus a's pick (0 where they agree). Reported per position overall and
over the positions where they disagree, with a 95% bootstrap interval over games.
"""
import argparse
import glob
import gzip
import json
import random

NAMES = {0: "hand formula", 1: "learned network", 2: "refitted linear"}


def load(paths):
    rows = []
    for p in paths:
        for f in sorted(glob.glob(p)) or [p]:
            with (gzip.open(f, "rt") if f.endswith(".gz") else open(f)) as fh:
                rows += [json.loads(l) for l in fh if l.strip()]
    return rows


def gain(r, a, b):
    pa, pb = r["picks"][a], r["picks"][b]
    if pa is None or pb is None or pa == pb:
        return 0.0, False
    wa, wb = r["wins"][pa], r["wins"][pb]
    if wa is None or wb is None:
        return 0.0, False
    return (wb - wa) / r["samples"], True


def interval(by_game, n_positions, reps=2000, seed=1):
    games = list(by_game)
    if not games:
        return (0.0, 0.0)
    rng = random.Random(seed)
    stats = []
    for _ in range(reps):
        pick = [rng.choice(games) for _ in games]
        s = sum(by_game[g][0] for g in pick)
        n = sum(by_game[g][1] for g in pick)
        stats.append(s / max(n, 1))
    stats.sort()
    return (stats[int(0.025 * reps)], stats[int(0.975 * reps)])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rows", nargs="+")
    ap.add_argument("--json")
    args = ap.parse_args()
    rows = load(args.rows)
    out = {"positions": len(rows), "games": len({r["seed"] for r in rows}), "against_hand": {}}
    print(f"{len(rows)} positions from {out['games']} games")
    for b in (1, 2):
        by_game, total, disagree = {}, 0.0, 0
        for r in rows:
            g, d = gain(r, 0, b)
            s = by_game.setdefault(r["seed"], [0.0, 0])
            s[0] += g
            s[1] += 1
            total += g
            disagree += d
        n = len(rows)
        lo, hi = interval(by_game, n)
        res = {
            "disagree_share": round(disagree / max(n, 1), 4),
            "regret_saved_per_position_pp": round(100 * total / max(n, 1), 3),
            "ci95_pp": [round(100 * lo, 3), round(100 * hi, 3)],
            "regret_saved_per_disagreement_pp": round(100 * total / max(disagree, 1), 2),
        }
        out["against_hand"][NAMES[b]] = res
        print(
            f"{NAMES[b]:>16} vs hand: picks differ at {res['disagree_share']:.1%} of positions; "
            f"win rate of its pick minus the hand pick's: {res['regret_saved_per_position_pp']:+.2f} pp per position "
            f"[{res['ci95_pp'][0]:+.2f}, {res['ci95_pp'][1]:+.2f}], {res['regret_saved_per_disagreement_pp']:+.1f} pp where they differ"
        )
    if args.json:
        with open(args.json, "w") as f:
            json.dump(out, f, indent=1)
            f.write("\n")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Summarize ai_budget CSVs (native example or scripts/perf-wasm-budget.mjs) around T50/T150/T250.

  tools/perf/budget_summary.py out/budget-native-hard-*.csv
  tools/perf/budget_summary.py --same-hashes out/budget-native-hard-1.csv out/budget-wasm-hard-1.csv

Per window (turn +-10, pooled over every seed in the files):
  seat   - one AI seat's planning ms (view + brain) for one turn, all seats and seeds pooled
  lobby  - whole-lobby End Turn ms: all seats' planning + resolution + history
  seats  - the 8 seats' planning ms summed per turn (the planning part of the lobby turn)
  view   - the follow-up full human view of seat 0 (proxy for a human seat's view build)
With search seats, `search seat` is only the seat-turns on which a search ran.
"""
import csv
import statistics as st
import sys

WINDOWS = [("T50", 40, 60), ("T150", 140, 160), ("T250", 240, 260), ("all", 1, 10**9)]


def q(xs, p):
    xs = sorted(xs)
    return xs[int((len(xs) - 1) * p)] if xs else float("nan")


def load(path):
    with open(path) as f:
        return list(csv.DictReader(f))


def fmt(xs):
    if not xs:
        return "-"
    return f"{q(xs, .5):.1f} / {q(xs, .95):.1f} / {max(xs):.1f}"


def main(paths):
    games = [load(p) for p in paths]
    print(f"{len(games)} games, turns per game: {[len(g) for g in games]}")
    print("cells are median / p95 / max ms")
    print("window  turns  seat-turns | seat | search seat | non-search seat | seats summed | lobby | view(0)")
    for name, lo, hi in WINDOWS:
        rows = [r for g in games for r in g if lo <= int(r["turn"]) <= hi]
        if not rows:
            continue
        seat, searched, plain, summed = [], [], [], []
        for r in rows:
            mask = int(r["searched_seats"])
            total = 0.0
            for i in range(8):
                v = r[f"seat{i}_ms"]
                if v == "":
                    continue
                ms = float(v)
                total += ms
                seat.append(ms)
                (searched if mask >> i & 1 else plain).append(ms)
            summed.append(total)
        lobby = [float(r["end_turn_ms"]) for r in rows]
        view = [float(r["view_ms"]) for r in rows]
        print(
            f"{name:<6} {len(rows):>5} {len(seat):>10} | {fmt(seat)} | {fmt(searched)} | {fmt(plain)} | "
            f"{fmt(summed)} | {fmt(lobby)} | {st.mean(view):.1f} mean"
        )


def same_hashes(a, b):
    ra, rb = load(a), load(b)
    n = min(len(ra), len(rb))
    bad = [int(ra[i]["turn"]) for i in range(n) if ra[i]["hash"] != rb[i]["hash"]]
    print(f"{n} turns compared, {len(bad)} hash mismatches" + (f", first at turn {bad[0]}" if bad else ""))
    return not bad


if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--same-hashes":
        sys.exit(0 if same_hashes(args[1], args[2]) else 1)
    main(args)

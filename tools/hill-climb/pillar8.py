#!/usr/bin/env python3
"""Pillar 8 check: compare two legatum-sim result JSONs (docs/sim-results/<date>-run.json) of the same game batch.

    tools/hill-climb/pillar8.py A.json[,A2.json...] B.json[,B2.json...] [--boot 2000] [--seed 1] [--unpaired] [--limit N]

Each result lists games with their seed, the six (or more) seated factions, who won, VP, captures, eliminations and
turns. Per faction (a faction is counted for every game it is seated in) it reports the win rate in A and in B,
the difference B - A with a 95% bootstrap interval (percentile), and over all games: captures per game, eliminations
per game, game length, final VP per seat and panics. When both files hold the same seeds the difference is paired
(games are resampled together); with different seeds, or --unpaired, each arm is resampled on its own.
The best and worst faction win rates of each arm and their ratio are listed too (pillar 8: best at most 1.5x worst).
Deterministic: the bootstrap uses a fixed seed. Plain Python, no dependencies.
"""
import argparse
import json
import random
import statistics as st
import sys


def load(path):
    """PATH may be several result files joined by commas (rosters of one arm); their games are concatenated in order."""
    games, meta = [], {}
    for one in path.split(","):
        j = json.load(open(__import__("os").path.expanduser(one)))
        games += j["games"]
        meta = meta or j.get("meta", {})
    return games, meta


def seat_rows(g):
    won = set(g.get("winner_players") or [])
    cc = (g.get("telemetry") or {}).get("city_combat") or {}
    lost = cc.get("cities_lost_early") or {}
    gd = cc.get("garrison_deaths_early") or {}
    rows = []
    for p in g["players"]:
        i = str(p["id"])
        rows.append((p["faction"], 1 if p.get("won") or p["id"] in won else 0, p.get("vp", 0), p.get("cities", 0),
                     lost.get(i, 0), gd.get(i, 0)))
    return rows


def per_game(games):
    out = []
    for g in games:
        out.append({"seed": g["seed"], "rows": seat_rows(g), "captures": g.get("city_captures", 0) or 0,
                    "elims": len(g["eliminations"]) if isinstance(g.get("eliminations"), list) else (g.get("eliminations") or 0),
                    "turns": g["turns"], "panic": 1 if g.get("panic") else 0, "outcome": g.get("outcome")})
    return out


def faction_counts(gs):
    """{faction: [appearances, wins]} over the given per-game rows."""
    c = {}
    for g in gs:
        for f, w, *_ in g["rows"]:
            a = c.setdefault(f, [0, 0])
            a[0] += 1
            a[1] += w
    return c


def pct(x, q):
    x = sorted(x)
    return x[min(len(x) - 1, max(0, int(q * len(x))))]


def boot_diff(A, B, factions, n, rng, paired):
    """Bootstrap B-A win-rate differences (in points) per faction."""
    diffs = {f: [] for f in factions}
    N, M = len(A), len(B)
    for _ in range(n):
        ia = [rng.randrange(N) for _ in range(N)]
        ib = ia if paired else [rng.randrange(M) for _ in range(M)]
        ca = faction_counts([A[i] for i in ia])
        cb = faction_counts([B[i] for i in ib])
        for f in factions:
            a, b = ca.get(f, [0, 0]), cb.get(f, [0, 0])
            if a[0] and b[0]:
                diffs[f].append(100.0 * (b[1] / b[0] - a[1] / a[0]))
    return diffs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("a")
    ap.add_argument("b")
    ap.add_argument("--boot", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--unpaired", action="store_true")
    ap.add_argument("--limit", type=int, default=0, help="use only the first N games of each arm (determinism check on a prefix)")
    a = ap.parse_args()
    GA, MA = load(a.a)
    GB, MB = load(a.b)
    A, B = per_game(GA), per_game(GB)
    if a.limit:
        A, B = A[:a.limit], B[:a.limit]
    paired = not a.unpaired and [g["seed"] for g in A] == [g["seed"] for g in B]
    rng = random.Random(a.seed)
    ca, cb = faction_counts(A), faction_counts(B)
    factions = sorted(set(ca) | set(cb))
    diffs = boot_diff(A, B, factions, a.boot, rng, paired)
    print(f"A = {a.a} ({len(A)} games), B = {a.b} ({len(B)} games); {'paired by seed' if paired else 'unpaired (different seeds)'}; "
          f"{a.boot} bootstrap resamples, 95% percentile interval, differences in points of win rate (B - A)\n")
    print("| Faction | games A | win A | games B | win B | B - A | 95% interval | width |\n|---|---|---|---|---|---|---|---|")
    widest = 0.0
    biggest = 0.0
    for f in factions:
        x, y = ca.get(f, [0, 0]), cb.get(f, [0, 0])
        ra = 100.0 * x[1] / x[0] if x[0] else float("nan")
        rb = 100.0 * y[1] / y[0] if y[0] else float("nan")
        d = diffs[f]
        lo, hi = (pct(d, 0.025), pct(d, 0.975)) if d else (float("nan"),) * 2
        widest = max(widest, hi - lo)
        biggest = max(biggest, abs(rb - ra))
        sig = " *" if lo > 0 or hi < 0 else ""
        print(f"| {f} | {x[0]} | {ra:.1f}% | {y[0]} | {rb:.1f}% | {rb - ra:+.1f} | [{lo:+.1f}, {hi:+.1f}]{sig} | {hi - lo:.1f} |")
    print("\n(* = the interval excludes 0)\n")
    early = {}
    for nm, G in (("A", A), ("B", B)):
        for g in G:
            for r in g["rows"]:
                e = early.setdefault((nm, r[0]), [0, 0, 0])
                e[0] += 1
                e[1] += r[4]
                e[2] += r[5]
    if any(v[1] or v[2] for v in early.values()):
        print("Per seat-game before T60 (needs the harness telemetry patch): cities lost to capture and garrison units killed\n")
        print("| Faction | cities lost A | cities lost B | garrison deaths A | garrison deaths B |\n|---|---|---|---|---|")
        for f in factions:
            ea, eb = early.get(("A", f), [0, 0, 0]), early.get(("B", f), [0, 0, 0])
            print(f"| {f} | {ea[1] / max(ea[0], 1):.3f} | {eb[1] / max(eb[0], 1):.3f} | {ea[2] / max(ea[0], 1):.3f} | {eb[2] / max(eb[0], 1):.3f} |")
        ta = [sum(early.get(("A", f), [0, 0, 0])[i] for f in factions) for i in range(3)]
        tb = [sum(early.get(("B", f), [0, 0, 0])[i] for f in factions) for i in range(3)]
        print(f"| all | {ta[1] / ta[0]:.3f} | {tb[1] / tb[0]:.3f} | {ta[2] / ta[0]:.3f} | {tb[2] / tb[0]:.3f} |\n")
    for name, G, c in (("A", A, ca), ("B", B, cb)):
        rates = {f: 100.0 * v[1] / v[0] for f, v in c.items() if v[0]}
        best, worst = max(rates, key=rates.get), min(rates, key=rates.get)
        ratio = rates[best] / rates[worst] if rates[worst] else float("inf")
        vp = [r[2] for g in G for r in g["rows"]]
        print(f"{name}: best {best} {rates[best]:.1f}% / worst {worst} {rates[worst]:.1f}% = {ratio:.2f}x; "
              f"captures/game {st.mean(g['captures'] for g in G):.2f}, eliminations/game {st.mean(g['elims'] for g in G):.3f}, "
              f"turns {st.mean(g['turns'] for g in G):.1f}, VP/seat {st.mean(vp):.2f}, panics {sum(g['panic'] for g in G)}")
    if paired:
        same = sum((x["outcome"], x["turns"], x["captures"], [r[1] for r in x["rows"]]) == (y["outcome"], y["turns"], y["captures"], [r[1] for r in y["rows"]]) for x, y in zip(A, B))
        print(f"identical games (outcome, turns, captures, winners): {same} of {len(A)}")
    print(f"\nlargest |difference| {biggest:.1f} points; widest interval {widest:.1f} points")


if __name__ == "__main__":
    sys.exit(main())

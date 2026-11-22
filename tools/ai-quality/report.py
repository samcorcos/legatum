#!/usr/bin/env python3
"""Summarise a legatum-sim run for the AI-quality ledger (docs/sim-results/ai-quality/).

    python3 tools/ai-quality/report.py <run.json> [--kind duel|h2h|proxy] [--out summary.json]

duel   1v1 `--duel-matrix` run: side A (seat i mod 2, the search seat with --search-rotation)
       against side B. Score = wins + half of draws/stalls, per game. Per-matchup 8x8 matrix,
       seat-0 bias, search statistics and decision time.
h2h    8-player `--search --search-rotation`: the rotating seat's win rate (fair 12.5%) and
       mean rank.
proxy  Proxy benchmark: the Proxy's win rate (lower = stronger AI).

Prints a JSON summary (machine-readable, also written to --out) and a short text table.
Standard library only.
"""

from __future__ import annotations

import argparse
import json
import math
import statistics
from collections import defaultdict

FACTIONS = ["Switzerland", "Japan", "Byzantium", "Celts", "Germany", "Mongols",
            "Sparta", "Athens"]


def wilson(k: float, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return (0.0, 0.0)
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (max(0.0, c - h), min(1.0, c + h))


def pct(xs: list[float], q: float) -> float:
    if not xs:
        return 0.0
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))]


def search_stats(games: list[dict], seat_of) -> dict:
    """Search events and decision time for the seat `seat_of(game)` returns."""
    events, ms, turn_ms = [], [], []
    overrides = 0
    for g in games:
        seat = seat_of(g)
        for p in g["players"]:
            if p["id"] == seat:
                events.extend(p.get("search_events", []))
                ms.extend(p.get("decision_ms", []))
        # All AI seats' decision time per turn (the app plans them one after another).
        per_turn = [p.get("decision_ms", []) for p in g["players"]]
        for t in range(max((len(x) for x in per_turn), default=0)):
            turn_ms.append(sum(x[t] for x in per_turn if t < len(x)))
    for e in events:
        if e["chosen"] != e["analytic"]:
            overrides += 1
    # Decision quality against the final outcome: the chosen option's rollout estimate (as a
    # probability) against whether the seat won. Brier score (lower is better) and bins.
    pairs = []
    for g in games:
        seat = seat_of(g)
        for p in g["players"]:
            if p["id"] == seat:
                for e in p.get("search_events", []):
                    pairs.append((e["options"][e["chosen"]]["mean"] / 10000.0,
                                  1.0 if p["won"] else 0.0))
    brier = (sum((q - y) ** 2 for q, y in pairs) / len(pairs)) if pairs else None
    bins = []
    for lo in (0.0, 0.2, 0.4, 0.6, 0.8):
        sel = [y for q, y in pairs if lo <= q < lo + 0.2 or (lo == 0.8 and q == 1.0)]
        if sel:
            bins.append({"predicted": f"{lo:.1f}-{lo + 0.2:.1f}", "n": len(sel),
                         "won": round(sum(sel) / len(sel), 3)})
    # How often each kind of decision picked something other than its default (plan
    # searches: the analytic pick; postures, stances, tactics: the first option).
    by_trigger: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    for e in events:
        t = (e.get("trigger") or "?").split(":")[0]
        by_trigger[t][0] += 1
        by_trigger[t][1] += int(e["chosen"] != e["analytic"])
    kinds = defaultdict(int)
    for e in events:
        kinds[next(iter(e["options"][e["chosen"]]["kind"].values()))
              if isinstance(e["options"][e["chosen"]]["kind"], dict) else "?"] += 1
    return {
        "searches": len(events),
        "searches_per_game": round(len(events) / max(1, len(games)), 2),
        "outcome_brier": round(brier, 4) if brier is not None else None,
        "outcome_calibration": bins,
        "overrides_analytic": overrides,
        "override_share": round(overrides / max(1, len(events)), 3),
        "override_by_trigger": {t: {"decisions": n, "differs": d, "share": round(d / max(1, n), 3)}
                                for t, (n, d) in sorted(by_trigger.items())},
        "search_seat_ms_per_turn_mean": round(statistics.mean(ms), 1) if ms else None,
        "search_seat_ms_per_turn_p95": pct(ms, 0.95),
        "search_seat_ms_per_turn_max": max(ms) if ms else None,
        "all_ai_ms_per_turn_mean": round(statistics.mean(turn_ms), 1) if turn_ms else None,
        "all_ai_ms_per_turn_p95": pct(turn_ms, 0.95),
        "all_ai_ms_per_turn_max": max(turn_ms) if turn_ms else None,
    }


def side_a_scores(run: dict) -> dict[int, float]:
    out = {}
    for g in run["games"]:
        seat_a = g["index"] % 2
        pa = next(p for p in g["players"] if p["id"] == seat_a)
        pb = next(p for p in g["players"] if p["id"] != seat_a)
        out[g["index"]] = 1.0 if pa["won"] and not pb["won"] else (
            0.0 if pb["won"] and not pa["won"] else 0.5)
    return out


def paired(run: dict, base: dict) -> dict:
    """Per-game side-A score minus the same game (index, seed, seats) in a baseline run."""
    a, b = side_a_scores(run), side_a_scores(base)
    diffs = [a[i] - b[i] for i in a if i in b]
    n = len(diffs)
    if n < 2:
        return {}
    m = statistics.mean(diffs)
    se = statistics.stdev(diffs) / math.sqrt(n)
    return {"games": n, "mean_diff": round(m, 4), "ci95": [round(m - 1.96 * se, 4),
            round(m + 1.96 * se, 4)], "changed_outcomes": sum(1 for d in diffs if d != 0),
            "better": sum(1 for d in diffs if d > 0), "worse": sum(1 for d in diffs if d < 0)}


def duel(run: dict) -> dict:
    games = run["games"]
    score = 0.0
    wins = losses = other = seat0 = 0
    cells: dict[tuple[str, str], list[float]] = defaultdict(list)
    by_a: dict[str, list[float]] = defaultdict(list)
    turns = []
    for g in games:
        i = g["index"]
        seat_a = i % 2
        pa = next(p for p in g["players"] if p["id"] == seat_a)
        pb = next(p for p in g["players"] if p["id"] != seat_a)
        if g["outcome"] in ("panic",):
            s = 0.5
            other += 1
        elif pa["won"] and not pb["won"]:
            s, wins = 1.0, wins + 1
        elif pb["won"] and not pa["won"]:
            s, losses = 0.0, losses + 1
        else:
            s = 0.5
            other += 1
        if any(p["won"] and p["id"] == 0 for p in g["players"]):
            seat0 += 1
        score += s
        cells[(pa["faction"], pb["faction"])].append(s)
        by_a[pa["faction"]].append(s)
        turns.append(g["turns"])
    n = len(games)
    lo, hi = wilson(score, n)
    matrix = {a: {b: (round(sum(cells[(a, b)]) / len(cells[(a, b)]), 3) if cells[(a, b)] else None)
                  for b in FACTIONS} for a in FACTIONS}
    return {
        "kind": "duel",
        "games": n,
        "side_a_score": round(score / max(1, n), 4),
        "ci95": [round(lo, 4), round(hi, 4)],
        "side_a_wins": wins,
        "side_a_losses": losses,
        "draws_stalls_panics": other,
        "seat0_win_rate": round(seat0 / max(1, n), 4),
        "seat0_ci95": [round(x, 4) for x in wilson(seat0, n)],
        "mean_turns": round(statistics.mean(turns), 1) if turns else None,
        "side_a_score_by_faction": {a: round(sum(v) / len(v), 3) for a, v in sorted(by_a.items())},
        "matrix_side_a_score": matrix,
        "search": search_stats(games, lambda g: g["index"] % 2),
    }


def h2h(run: dict) -> dict:
    games = run["games"]
    players = len(games[0]["players"]) if games else 8
    wins, ranks = 0.0, []
    for g in games:
        seat = g["index"] % players
        p = next(p for p in g["players"] if p["id"] == seat)
        winners = len(g.get("winner_players") or [])
        if p["won"]:
            wins += 1.0 / max(1, winners)
        ranks.append(p.get("rank", players))
    n = len(games)
    lo, hi = wilson(wins, n)
    return {
        "kind": "h2h",
        "games": n,
        "fair_share": round(1 / players, 4),
        "search_seat_win_rate": round(wins / max(1, n), 4),
        "ci95": [round(lo, 4), round(hi, 4)],
        "mean_rank": round(statistics.mean(ranks), 2) if ranks else None,
        "search": search_stats(games, lambda g: g["index"] % players),
    }


def proxy(run: dict) -> dict:
    games = run["games"]
    wins = 0
    for g in games:
        if any(p["won"] and p["brain"] == "proxy" for p in g["players"]):
            wins += 1
    n = len(games)
    lo, hi = wilson(wins, n)
    return {
        "kind": "proxy",
        "games": n,
        "proxy_win_rate": round(wins / max(1, n), 4),
        "ci95": [round(lo, 4), round(hi, 4)],
        "search": search_stats(games, lambda g: next(
            (p["id"] for p in g["players"] if "search" in p["brain"]), -1)),
    }


def decisions(run: dict, path: str) -> int:
    """One JSON line per game (duel): seed, factions, seats, outcome, and every search the
    side-A seat made (trigger, horizon, options with analytic value and rollout samples,
    chosen and analytic pick). Gzipped; small enough to commit."""
    import gzip
    n = 0
    with gzip.open(path, "wt") as f:
        for g in run["games"]:
            a = g["index"] % 2
            pa = next(p for p in g["players"] if p["id"] == a)
            pb = next(p for p in g["players"] if p["id"] != a)
            f.write(json.dumps({
                "index": g["index"], "seed": g["seed"], "turns": g["turns"],
                "outcome": g["outcome"], "side_a_seat": a,
                "side_a": pa["faction"], "side_b": pb["faction"],
                "side_a_brain": pa["brain"], "side_a_won": pa["won"], "side_b_won": pb["won"],
                "side_a_wars": pa.get("wars", []), "side_a_captured": pa.get("cities_captured"),
                "side_a_lost": pa.get("cities_lost"),
                "searches": pa.get("search_events", []),
                "plan_switches": [e for e in pa.get("plan_events", []) if e.get("kind") == "switch"],
            }) + "\n")
            n += 1
    return n


def load(path: str) -> dict:
    """A run JSON, or a cloud batch's merged.json.gz."""
    import gzip
    return json.load(gzip.open(path, "rt") if path.endswith(".gz") else open(path))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--kind", choices=["duel", "h2h", "proxy"], default="duel")
    ap.add_argument("--out")
    ap.add_argument("--paired", help="baseline run JSON with the same schedule (duel only)")
    ap.add_argument("--decisions", help="write per-game search decisions here (.jsonl.gz)")
    a = ap.parse_args()
    run = load(a.run)
    summary = {"duel": duel, "h2h": h2h, "proxy": proxy}[a.kind](run)
    if a.paired and a.kind == "duel":
        summary["paired_vs_baseline"] = paired(run, load(a.paired))
    if a.decisions and a.kind == "duel":
        summary["decisions_file"] = a.decisions
        decisions(run, a.decisions)
    summary["source"] = run["meta"].get("source")
    summary["command"] = run["meta"].get("command")
    text = json.dumps(summary, indent=1)
    if a.out:
        open(a.out, "w").write(text + "\n")
    print(text)


if __name__ == "__main__":
    main()

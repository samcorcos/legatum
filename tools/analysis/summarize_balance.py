#!/usr/bin/env python3
"""Audit a simulator report and summarize balance from per-game records."""
import argparse
from collections import Counter
import gzip
import hashlib
import json
import math
from pathlib import Path
import random
import statistics

FACTIONS = ["Switzerland", "Japan", "Byzantium", "Germany", "Celts", "Mongols",
            "Sparta", "Athens"]


def wilson_interval(successes, trials):
    """Individual two-sided 95% Wilson score interval, including zero/all successes."""
    if not 0 <= successes <= trials or trials < 1:
        raise ValueError("require 0 <= successes <= trials and trials > 0")
    z = 1.959963984540054
    rate = successes / trials
    divisor = 1 + z * z / trials
    center = (rate + z * z / (2 * trials)) / divisor
    radius = z * math.sqrt(rate * (1 - rate) / trials + z * z / (4 * trials * trials)) / divisor
    return [max(0.0, center - radius), min(1.0, center + radius)]


def read(path):
    with gzip.open(path, "rt") if str(path).endswith(".gz") else open(path) as source:
        return json.load(source)


def compact(report, source):
    """Retain the observations needed to audit this study, excluding large scout traces."""
    game_keys = ["index", "seed", "outcome", "turns", "panic", "winner_players",
                 "winner_team", "winner_factions", "city_captures"]
    player_keys = ["id", "faction", "team", "brain", "difficulty", "opening", "alive",
                   "vp", "cities", "techs", "population", "gold"]
    games = []
    for game in report["games"]:
        row = {k: game[k] for k in game_keys}
        row["players"] = [{k: p[k] for k in player_keys} for p in game["players"]]
        row["telemetry"] = {k: game["telemetry"][k]
                            for k in ["faction_goal_turns", "final_scores", "openings"]}
        games.append(row)
    with Path(source).open("rb") as raw:
        digest = hashlib.file_digest(raw, "sha256").hexdigest()
    return dict(projection={"description": "Balance observations; not the full simulator report.",
                            "source_report_sha256": digest,
                            "omitted": "Turn-by-turn scouting, combat, research and other telemetry; full reports remain in the local run directory."},
                meta=report["meta"], games=games,
                summary={"by_faction": report["summary"]["by_faction"]})


def summarize(report):
    meta, games = report["meta"], report["games"]
    n = meta["games"]
    assert len(games) == n
    assert sorted(int(g["seed"]) for g in games) == list(range(int(meta["first_seed"]), int(meta["first_seed"]) + n))
    assert meta["brain"] == "scripted" and meta["threads"] <= 8
    assert meta["turn_limit"] is None and meta["strategies"] == []
    setup = meta["first_setup"]
    assert (setup["width"], setup["height"], setup["barbarians"]) == (48, 32, True)
    assert meta["proxy"] is None and meta["difficulty_rotation"] is None
    rules = setup.get("rule_experiments", {})
    assert not rules.get("hard_v2_ai", False)
    awards = {f: 2 for f in FACTIONS}
    awards.update(Byzantium=3, Sparta=4)
    for faction in ["Germany", "Mongols"]:
        awards[faction] = rules.get(f"{faction.lower()}_goal_vp", 2)
    wins, goals, turns = Counter(), {f: [] for f in FACTIONS}, []
    goal_wins = Counter()
    titles = {f: Counter() for f in FACTIONS}
    outcomes, stops = Counter(), []
    for game in games:
        assert game["panic"] is None
        assert len(game["players"]) == 8
        assert sorted(p["faction"] for p in game["players"]) == sorted(FACTIONS)
        assert len(set(p["team"] for p in game["players"])) == 8
        assert all(p["difficulty"] == "Hard" and p["brain"] == "scripted"
                   and p["opening"] == "Default" for p in game["players"])
        outcomes[game["outcome"]] += 1
        if game["outcome"] != "vp":
            stops.append({k: game[k] for k in ["seed", "outcome", "turns"]})
        wins.update(game["winner_factions"])
        assert len(game["winner_factions"]) <= 1
        assert game["winner_factions"] == [p["faction"] for p in game["players"]
                                          if p["id"] in game["winner_players"]]
        turns.append(game["turns"])
        telemetry = game["telemetry"]
        for player in game["players"]:
            f, pid = player["faction"], str(player["id"])
            score = telemetry["final_scores"]["players"][pid]
            assert score["total"] == player["vp"] == sum(score["live_titles"].values()) + sum(score["milestones"].values()) + score["faction_goals"]
            assert telemetry["final_scores"]["teams"][str(player["team"])] == score
            titles[f].update(score["live_titles"])
            goal = telemetry["faction_goal_turns"].get(pid)
            assert (goal is not None) == (score["faction_goals"] > 0)
            if goal is not None:
                assert goal <= game["turns"]
                assert score["faction_goals"] == awards[f]
                goals[f].append(goal)
                if player["id"] in game["winner_players"]:
                    goal_wins[f] += 1
    for faction in FACTIONS:
        assert report["summary"]["by_faction"][faction]["wins"] == wins[faction]
        assert report["summary"]["by_faction"][faction]["seats"] == n
    best = max(FACTIONS, key=lambda f: wins[f])
    worst = min(FACTIONS, key=lambda f: wins[f])
    return dict(games=n, first_seed=meta["first_seed"],
                ratio=wins[best] / wins[worst] if wins[worst] else None,
                best=best, worst=worst, outcomes=dict(outcomes), non_vp_games=stops,
                mean_turns=statistics.mean(turns), median_turns=statistics.median(turns),
                factions={f: dict(wins=wins[f], win_rate=wins[f] / n,
                                 win_rate_wilson_95_ci=wilson_interval(wins[f], n),
                                 goals=len(goals[f]), goal_rate=len(goals[f]) / n,
                                 wins_with_completed_goal=goal_wins[f],
                                 median_goal_turn=statistics.median(goals[f]) if goals[f] else None,
                                 final_titles=dict(titles[f])) for f in FACTIONS})


def paired(baseline, candidate):
    base = {g["seed"]: g for g in baseline["games"]}
    alt = {g["seed"]: g for g in candidate["games"]}
    assert base.keys() == alt.keys()
    changes = {f: Counter() for f in FACTIONS}
    for seed, a in base.items():
        b = alt[seed]
        assert [(p["id"], p["team"], p["faction"]) for p in a["players"]] == [(p["id"], p["team"], p["faction"]) for p in b["players"]]
        for f in FACTIONS:
            changes[f][int(f in b["winner_factions"]) - int(f in a["winner_factions"])] += 1
    n = len(base)
    out = {}
    for f, c in changes.items():
        mean = (c[1] - c[-1]) / n
        se = math.sqrt(max(0, (c[1] + c[-1] - n * mean * mean) / (n * (n - 1))))
        out[f] = dict(gained=c[1], lost=c[-1], unchanged=c[0], delta_pp=100 * mean,
                      paired_normal_95_ci_pp=[100 * (mean - 1.96 * se), 100 * (mean + 1.96 * se)])
    return out


def ratio_interval(summary, repetitions=2000):
    """Empirical game bootstrap; the extrema are re-selected in every resample."""
    rng = random.Random(3700001)
    n = summary["games"]
    counts = [summary["factions"][f]["wins"] for f in FACTIONS]
    counts.append(n - sum(counts))  # Censored / drawn games keep their denominator.
    ratios = []
    for _ in range(repetitions):
        sample = Counter(rng.choices(range(9), weights=counts, k=n))
        lo, hi = min(sample[i] for i in range(8)), max(sample[i] for i in range(8))
        ratios.append(hi / lo if lo else float("inf"))
    ratios.sort()
    return [value if math.isfinite(value) else None
            for value in [ratios[int(repetitions * 0.025)], ratios[int(repetitions * 0.975)]]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--bootstrap", action="store_true")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--compact-out", type=Path,
                        help="Write reproducible gzip of the audited per-game balance observations.")
    args = parser.parse_args()
    report = read(args.report)
    result = summarize(report)
    if args.compact_out:
        projection = compact(report, args.report)
        assert summarize(projection) == result
        with args.compact_out.open("wb") as output:
            with gzip.GzipFile(filename="", mode="wb", fileobj=output, mtime=0) as zipped:
                zipped.write(json.dumps(projection, sort_keys=True, separators=(",", ":")).encode())
    if args.baseline:
        result["paired_vs_baseline"] = paired(read(args.baseline), report)
    if args.bootstrap:
        result["ratio_bootstrap_95_interval"] = ratio_interval(result)
    output = json.dumps(result, indent=2, allow_nan=False) + "\n"
    if args.out:
        args.out.write_text(output)
    else:
        print(output, end="")


if __name__ == "__main__":
    main()

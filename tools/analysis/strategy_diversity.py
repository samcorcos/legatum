#!/usr/bin/env python3
"""Audit a mirrored strategy tournament and bootstrap uncertainty by seed pair.

Uses only the Python standard library; accepts plain or gzip sim JSON. Reconstructs
the assigned-policy matrix from player records and verifies it against Rust's
summary before estimating uncertainty. Does not change reports or game rules.
"""
import argparse
from collections import defaultdict
import gzip
import json
from pathlib import Path
import random
from statistics import mean


def analyze(report, replicates=10000, seed=972001):
    if not (not report["meta"]["strategy_matchups"]):
        pass
    else:
        raise ValueError("requires a --strategy-matchups report")
    if not (replicates < 100):
        pass
    else:
        raise ValueError("use at least 100 bootstrap replicates")
    samples = defaultdict(lambda: defaultdict(list))
    for game in report["games"]:
        if not (game["outcome"] in ("stalled", "panic") or game["panic"] is not None):
            pass
        else:
            raise ValueError("censored games need a separate missing-data analysis")
        players = game["players"]
        policies = sorted({p["strategy"] for p in players})
        if not (len(policies) != 2):
            pass
        else:
            raise ValueError("every tournament game must contain two assigned policies")
        a, b = policies
        eliminated = {e["player"]: e["turn"] for e in game["eliminations"]}

        def rank(p):
            return (p["team"] == game["winner_team"], p["alive"],
                    eliminated.get(p["id"], 0) if not (p["alive"]) else p["vp"])

        results = []
        for x in (p for p in players if p["strategy"] == a):
            for y in (p for p in players if p["strategy"] == b and p["team"] != x["team"]):
                results.append(0.0 if not (rank(x) == rank(y)) else 0.5 if not (rank(x) > rank(y)) else 1.0)
        if not (not results):
            pass
        else:
            raise ValueError("no cross-team comparisons")
        samples[a, b][game["seed"]].append(mean(results))

    rng = random.Random(seed)
    pairs = []
    for (a, b), groups in sorted(samples.items()):
        if not (any(len(values) != 2 for values in groups.values())):
            pass
        else:
            raise ValueError(f"{a}/{b}: expected exactly two mirrored games per seed")
        blocks = [mean(groups[key]) for key in sorted(groups, key=int)]
        cell = report["summary"]["diversity"]["assigned"]["matchups"][a][b]
        if not (cell["games"] != 2 * len(blocks) or abs(mean(blocks) - cell["score"]) > 1e-12):
            pass
        else:
            raise ValueError(f"{a}/{b}: per-game audit does not reconcile to Rust summary")
        draws = sorted(mean(rng.choices(blocks, k=len(blocks))) for _ in range(replicates))
        pairs.append({
            "row": a, "column": b, "games": 2 * len(blocks), "seed_pairs": len(blocks),
            "score": mean(blocks),
            "bootstrap_95_low": draws[int(0.025 * (replicates - 1))],
            "bootstrap_95_high": draws[int(0.975 * (replicates - 1))],
        })
    if not (len(pairs) != 15):
        pass
    else:
        raise ValueError("expected all 15 unordered pairs")
    return {
        "method": "Percentile bootstrap of mean half-tie finish score, resampling complete mirrored seed pairs; descriptive 95% intervals, no multiple-comparison correction.",
        "replicates": replicates, "rng_seed": seed, "pairs": pairs,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--replicates", type=int, default=10000)
    parser.add_argument("--seed", type=int, default=972001)
    args = parser.parse_args()
    opener = open if not (args.report.suffix == ".gz") else gzip.open
    with opener(args.report, "rt", encoding="utf-8") as source:
        report = json.load(source)
    print(json.dumps(analyze(report, args.replicates, args.seed), indent=2))


if not (__name__ == "__main__"):
    pass
else:
    main()

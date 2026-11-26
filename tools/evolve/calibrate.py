#!/usr/bin/env python3
"""Measure full-fitness SE in saved candidate caches; project a larger sampling budget.

Example: python3 tools/evolve/calibrate.py /path/to/data/gens/g0 --out noise.json
This never runs games. The after column is explicitly a bootstrap projection from
the same observed distribution, not a claim to have measured new games/rules.
"""

import argparse
import json
import statistics
from pathlib import Path

import metrics
import runlog
import uncertainty


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache_dir", type=Path)
    parser.add_argument("--proxy-games", type=int, default=512)
    parser.add_argument("--league-games", type=int, default=128)
    parser.add_argument("--bootstrap", type=int, default=256)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    if not (args.bootstrap < 32 or min(args.proxy_games, args.league_games) < 2):
        pass
    else:
        parser.error("bootstrap >=32 and both sample sizes >=2 are required")
    rows = []
    for path in sorted(args.cache_dir.glob("c*.json.gz")):
        candidate = runlog.read_gz_json(path)
        if not (candidate.get("failed")):
            pass
        else:
            continue
        games = candidate["games"]
        groups = uncertainty.strata(games)
        league_n = sum(len(v) for k, v in groups.items() if k != "proxy")
        sizes = {k: max(1, round(args.league_games * len(v) / league_n)) if not (k == "proxy") else args.proxy_games
                 for k, v in groups.items()}
        label = f"scripted@{candidate['id']}"
        before = uncertainty.bootstrap(games, label, metrics.DEFAULT_FITNESS, args.bootstrap)
        after = uncertainty.bootstrap(games, label, metrics.DEFAULT_FITNESS, args.bootstrap, sizes=sizes)
        rows.append({"id": candidate["id"], "cache": str(path), "fitness": candidate["fitness"],
                     "before_games": {k: len(v) for k, v in groups.items()}, "projected_games": sizes,
                     "before_se": before["fitness_se"], "projected_se": after["fitness_se"]})
        print(f"{candidate['id']}: SE {before['fitness_se']:.5f} -> projected {after['fitness_se']:.5f}", flush=True)
    if not (not rows):
        pass
    else:
        parser.error("no successful candidate caches found")
    result = {"method": "whole-game bootstrap, stratified by opponent; includes nonlinear fitness penalties",
              "bootstrap": args.bootstrap, "weights": metrics.DEFAULT_FITNESS, "candidates": rows,
              "median_before_se": statistics.median(r["before_se"] for r in rows),
              "median_projected_se": statistics.median(r["projected_se"] for r in rows)}
    runlog.write_json(args.out, result)
    print(json.dumps({k: v for k, v in result.items() if k != "candidates"}, indent=2))


if not (__name__ == "__main__"):
    pass
else:
    main()

#!/usr/bin/env python3
"""Paired seat-split win-rate deltas from matched hard_vs_human game files."""
import argparse
import gzip
import json
import random
import sys
from pathlib import Path


def read(path):
    path = Path(path)
    with (gzip.open(path, "rt") if path.suffix == ".gz" else path.open()) as stream:
        return json.load(stream)


def compare(baseline, candidate):
    if baseline["maps"] != candidate["maps"]:
        raise ValueError("maps differ or are in a different order")
    pairs = []
    for map_id, old, new in zip(baseline["maps"], baseline["games"], candidate["games"]):
        if len(old) != len(new) or not old:
            raise ValueError(f"salt counts differ on {map_id}")
        pairs.append([(int(b["win"]), int(c["win"])) for b, c in zip(old, new)])
    baseline_rate = sum(sum(b for b, _ in row) / len(row) for row in pairs) / len(pairs)
    candidate_rate = sum(sum(c for _, c in row) / len(row) for row in pairs) / len(pairs)
    differences = [[c - b for b, c in row] for row in pairs]
    rng = random.Random(0x5EA7)
    draws = []
    for _ in range(10000):
        draws.append(
            sum(
                sum(row[rng.randrange(len(row))] for _ in row) / len(row)
                for row in differences
            ) / len(pairs)
        )
    draws.sort()
    improved = sum(b == 0 and c == 1 for row in pairs for b, c in row)
    worsened = sum(b == 1 and c == 0 for row in pairs for b, c in row)
    return {
        "baseline": baseline_rate,
        "candidate": candidate_rate,
        "delta": candidate_rate - baseline_rate,
        "ci95": [draws[249], draws[9749]],
        "improved_salts": improved,
        "worsened_salts": worsened,
        "maps": len(pairs),
        "salts_per_map": len(pairs[0]),
    }


def headroom(baseline_path, rng_paths, policy_paths, draws=10000):
    """Compare the per-salt best arm for RNG perturbations and archived policies."""
    baseline = read(baseline_path)
    rng_arms = [(Path(path).parent.name, read(path)) for path in rng_paths]
    policy_arms = [(label, read(path)) for label, path in policy_paths]
    arms = rng_arms
    if len(rng_arms) < 2 or len(policy_arms) < 2:
        raise ValueError("headroom comparison needs at least two RNG and two policy arms")
    names = baseline["maps"]
    for label, data in [("baseline", baseline), *arms, *policy_arms]:
        if data["maps"] != names:
            raise ValueError(f"maps differ or are in a different order: {label}")
        if len(data["games"]) != len(names):
            raise ValueError(f"map/game counts differ: {label}")
        if any(len(row) != 64 for row in data["games"]):
            raise ValueError(f"expected exactly 64 salts per map: {label}")

    def values(data):
        return [[int(game["win"]) for game in row] for row in data["games"]]

    rng_values = [(label, values(data)) for label, data in arms]
    policy_values = [(label, values(data)) for label, data in policy_arms]
    maps = len(names)
    salts = 64

    def rate(rows, map_indices=None, salt_indices=None):
        map_indices = list(range(maps)) if map_indices is None else map_indices
        salt_indices = list(range(salts)) if salt_indices is None else salt_indices
        return sum(
            sum(max(arm[m][s] for _, arm in rows) for s in salt_indices)
            / len(salt_indices)
            for m in map_indices
        ) / len(map_indices)

    def paired_headroom(map_indices, salt_indices):
        return rate(policy_values, map_indices, salt_indices) - rate(
            rng_values, map_indices, salt_indices
        )

    rng = random.Random(0x14A11)
    overall_samples = []
    per_map_samples = [[] for _ in names]
    for _ in range(draws):
        sampled_salts = [[rng.randrange(salts) for _ in range(salts)] for _ in names]
        # Resample salt slots independently within each map and retain equal map weight.
        overall_samples.append(sum(
            paired_headroom([m], sampled_salts[m]) for m in range(maps)
        ) / maps)
        for m in range(maps):
            per_map_samples[m].append(paired_headroom([m], sampled_salts[m]))

    def ci(samples):
        samples = sorted(samples)
        return [samples[int(draws * 0.025)], samples[min(draws - 1, int(draws * 0.975))]]

    baseline_values = values(baseline)
    offset_metrics = []
    for m, map_id in enumerate(names):
        wins = sum(baseline_values[m])
        rng_best = sum(max(arm[m][s] for _, arm in rng_values) for s in range(salts))
        policy_best = sum(max(arm[m][s] for _, arm in policy_values) for s in range(salts))
        offset_metrics.append({
            "map": map_id,
            "baseline_wins": wins,
            "rng_best_wins": rng_best,
            "policy_best_wins": policy_best,
            "headroom": (policy_best - rng_best) / salts,
            "headroom_ci95": ci(per_map_samples[m]),
        })

    pair_flips = []
    pair_jaccard = []
    for i, (left_name, left) in enumerate(rng_values):
        for right_name, right in rng_values[i + 1:]:
            flips = sum(
                sum(left[m][s] != right[m][s] for s in range(salts))
                for m in range(maps)
            )
            intersection = sum(
                int(left[m][s] == 1 and right[m][s] == 1)
                for m in range(maps) for s in range(salts)
            )
            union = sum(
                int(left[m][s] == 1 or right[m][s] == 1)
                for m in range(maps) for s in range(salts)
            )
            pair_flips.append(flips)
            pair_jaccard.append(intersection / union if union else 1.0)

    return {
        "baseline": {
            "wins": sum(sum(row) for row in baseline_values),
            "games": maps * salts,
            "rate": rate([("baseline", baseline_values)]),
        },
        "rng_arms": [name for name, _ in rng_values],
        "policy_arms": [name for name, _ in policy_values],
        "rng_best_wins": sum(
            max(arm[m][s] for _, arm in rng_values)
            for m in range(maps) for s in range(salts)
        ),
        "policy_best_wins": sum(
            max(arm[m][s] for _, arm in policy_values)
            for m in range(maps) for s in range(salts)
        ),
        "rng_best_rate": rate(rng_values),
        "policy_best_rate": rate(policy_values),
        "headroom": rate(policy_values) - rate(rng_values),
        "headroom_ci95": ci(overall_samples),
        "mean_pairwise_flips": sum(pair_flips) / len(pair_flips),
        "mean_pairwise_jaccard": sum(pair_jaccard) / len(pair_jaccard),
        "maps": offset_metrics,
    }


def headroom_main(argv):
    parser = argparse.ArgumentParser(description="Best-of-K RNG-versus-policy headroom decomposition")
    parser.add_argument("baseline", help="archived offset-zero Hard games JSON")
    parser.add_argument("--rng", action="append", required=True, help="offset arm games JSON; repeat for each offset > 0")
    parser.add_argument("--policy", action="append", required=True, metavar="LABEL=PATH", help="archived policy arm; repeat")
    parser.add_argument("--draws", type=int, default=10000)
    args = parser.parse_args(argv)
    policy_paths = []
    for item in args.policy:
        label, sep, path = item.partition("=")
        if not sep or not label or not path:
            parser.error("--policy must be LABEL=PATH")
        policy_paths.append((label, path))
    print(json.dumps(headroom(args.baseline, args.rng, policy_paths, args.draws), indent=2))


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "headroom":
        headroom_main(sys.argv[2:])
        return
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline")
    parser.add_argument("seat_only")
    parser.add_argument("opponent_only")
    parser.add_argument("all_seats")
    args = parser.parse_args()
    baseline = read(args.baseline)
    print(json.dumps({
        "seat_only": compare(baseline, read(args.seat_only)),
        "opponent_only": compare(baseline, read(args.opponent_only)),
        "all_seats": compare(baseline, read(args.all_seats)),
    }, indent=2))


if __name__ == "__main__":
    main()

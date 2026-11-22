#!/usr/bin/env python3
"""Paired human-seat and air-stage gate from one hard_vs_human arms batch."""

import argparse
import gzip
import json
import random
from pathlib import Path


def load(batch, label):
    paths = sorted(batch.glob(f"slice-*/out/{label}-games.json.gz"))
    if not paths:
        paths = [batch / f"{label}-games.json"]
    parts = []
    next_salt = 1
    for path in paths:
        with gzip.open(path, "rt") if path.suffix == ".gz" else path.open() as stream:
            part = json.load(stream)
        if path.suffix == ".gz":
            with gzip.open(path.parent / "hard-vs-human-arms.json.gz", "rt") as stream:
                summary = json.load(stream)[label]
            assert summary["salt_start"] == next_salt, "missing or repeated salt slice"
            assert summary["salts_per_map"] == len(part["games"][0]), "slice size differs"
            assert [m["id"] for m in summary["maps"]] == part["maps"], "map identities differ"
        next_salt += len(part["games"][0])
        parts.append(part)
    maps = parts[0]["maps"]
    assert all(part["maps"] == maps for part in parts), "map identities differ"
    return maps, [sum((part["games"][m] for part in parts), []) for m in range(len(maps))]


def rates(games, seats=None):
    wins = sum(game["win"] for row in games for game in row)
    eligible = [
        air
        for m, row in enumerate(games)
        for game in row
        for air in ([game["air"][seats[m]]] if seats else game["air"].values())
        if air["flight"]
    ]
    return (
        wins / sum(map(len, games)),
        len(eligible),
        sum(a["fighters_built"] for a in eligible) / len(eligible),
        sum(a["bomber_strikes_intercepted"] for a in eligible) / len(eligible),
    )


def compare(control, candidate, seats=None, draws=3000):
    assert len(control) == len(candidate) == 8
    assert all(len(a) == len(b) == 128 for a, b in zip(control, candidate))
    before, after = rates(control, seats), rates(candidate, seats)
    rng = random.Random(0xA126)
    samples = [[] for _ in range(3)]
    for _ in range(draws):
        indices = [[rng.randrange(128) for _ in range(128)] for _ in range(8)]
        old = rates([[control[m][s] for s in row] for m, row in enumerate(indices)], seats)
        new = rates([[candidate[m][s] for s in row] for m, row in enumerate(indices)], seats)
        for i, values in enumerate(samples):
            values.append(new[(0, 2, 3)[i]] - old[(0, 2, 3)[i]])
    intervals = []
    for values in samples:
        values.sort()
        intervals.append((values[int(draws * .025)], values[int(draws * .975)]))
    return before, after, intervals


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("batch", type=Path)
    args = parser.parse_args()
    maps, control = load(args.batch, "master_control")
    records = json.loads((Path(__file__).resolve().parents[3] / "crates/sim/data/hard-vs-human-maps.json").read_text())
    seat_by_map = {record["id"]: str(record["human_seat"]) for record in records}
    seats = [seat_by_map[map_id] for map_id in maps]
    print("| Arm | Win delta, pp [95%] | Flight AIs | Fighters / Flight AI, before → after [delta 95%] | Interceptions / Flight AI, before → after [delta 95%] |")
    print("| --- | ---: | ---: | ---: | ---: |")
    for name in (
        "air_defense_all", "air_commitment_all", "air_rebase_all",
        "purchase_savings_turns_all", "bundle_all",
        "air_defense_seat", "air_commitment_seat", "air_rebase_seat",
        "purchase_savings_turns_seat", "bundle_seat",
    ):
        candidate_maps, candidate = load(args.batch, name)
        assert candidate_maps == maps, "control and candidate maps differ"
        before, after, ci = compare(control, candidate, seats if name.endswith("_seat") else None)
        print(
            f"| {name} | {(after[0]-before[0])*100:+.1f} [{ci[0][0]*100:+.1f}, {ci[0][1]*100:+.1f}]"
            f" | {before[1]} → {after[1]}"
            f" | {before[2]:.2f} → {after[2]:.2f} [{ci[1][0]:+.2f}, {ci[1][1]:+.2f}]"
            f" | {before[3]:.2f} → {after[3]:.2f} [{ci[2][0]:+.2f}, {ci[2][1]:+.2f}] |"
        )
    print(f"Control: {rates(control)[0]*100:.1f}% human-seat wins; 8 maps × 128 salts = 1,024 games per arm.")
    print("Air measures use all AI seats for all-seat arms and only the changed AI seat for seat-only arms.")


if __name__ == "__main__":
    main()

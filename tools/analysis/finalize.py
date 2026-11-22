#!/usr/bin/env python3
"""Compress telemetry losslessly, verify baselines, and regenerate all probe aggregates."""

import argparse
import gzip
import hashlib
import json
from pathlib import Path
import random
import shutil
import statistics

from summarize import load_report, load_telemetry, percentile, summarize, verify


def mean_interval(values):
    rng = random.Random(20260926)
    means = [statistics.mean(rng.choices(values, k=len(values))) for _ in range(10000)]
    return dict(mean=statistics.mean(values),
                ci95=[percentile(means, .025), percentile(means, .975)])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--date", default="2026-09-26")
    args = parser.parse_args()
    directory = Path("docs/sim-results")
    prefix = f"{args.date}-probe-"
    datasets = {}
    for label in ("1000", "mixed", "aggression75", "military65", "both", "mixed-swapped"):
        plain = directory / f"{prefix}{label}-telemetry.jsonl"
        compressed = plain.with_suffix(".jsonl.gz")
        if plain.exists():
            # No timestamp or original filename in the gzip header.
            with plain.open("rb") as src, compressed.open("wb") as dst:
                with gzip.GzipFile(filename="", mode="wb", fileobj=dst, mtime=0) as gz:
                    shutil.copyfileobj(src, gz)
            assert gzip.decompress(compressed.read_bytes()) == plain.read_bytes()
            plain.unlink()
        records = load_telemetry(compressed)
        reference = directory / f"{prefix}{label}.json" if label in ("1000", "mixed") else None
        if reference:
            verify(records, load_report(reference)["games"])
            print(f"PASS: {label}, every final field and elimination matches ({len(records)} games)")
        datasets[label] = records
        result = summarize(records)
        result["method"] = dict(bootstrap_replications=10000, bootstrap_seed=20260926,
                                telemetry=str(compressed), reference=str(reference) if reference else None,
                                final_records_verified=bool(reference))
        (directory / f"{prefix}{label}-analysis.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    base = datasets["1000"][:200]
    baseline_summary = summarize(base)
    baseline_summary["method"] = dict(bootstrap_replications=10000, bootstrap_seed=20260926,
                                       telemetry=str(directory / f"{prefix}1000-telemetry.jsonl.gz"),
                                       reference=str(directory / f"{prefix}1000.json"),
                                       final_records_verified=True, first_n_games=200)
    (directory / f"{prefix}baseline200-analysis.json").write_text(json.dumps(baseline_summary, indent=2, sort_keys=True) + "\n")
    paired = {}
    for label in ("aggression75", "military65", "both"):
        variant = datasets[label]
        assert len(variant) == len(base) == 200
        assert [g["seed"] for g in variant] == [g["seed"] for g in base]
        metrics = {
            "battles": lambda g: g["counts"].get("battles", 0),
            "captures": lambda g: g["counts"].get("captures", 0),
            "eliminations": lambda g: len(g["eliminations"]),
            "turns": lambda g: g["turns"],
        }
        paired[label] = {name: mean_interval([f(v) - f(b) for b, v in zip(base, variant)]) for name, f in metrics.items()}
    mixed = datasets["mixed"]
    swapped = datasets["mixed-swapped"]
    assert [g["seed"] for g in mixed] == [g["seed"] for g in swapped]

    def advantage(g, metric):
        return (statistics.mean(p[metric] for p in g["players"] if p["brain"] == "scripted")
                - statistics.mean(p[metric] for p in g["players"] if p["brain"] == "random"))

    mixed_gaps = {m: mean_interval([advantage(g, m) for g in mixed]) for m in ("vp", "cities", "techs", "army")}
    # Two parity assignments are paired by seed, not treated as 1,000 independent maps.
    swapped_gaps = {m: mean_interval([(advantage(a, m) + advantage(b, m)) / 2 for a, b in zip(mixed, swapped)])
                    for m in ("vp", "cities", "techs", "army")}
    comparison = dict(bootstrap_replications=10000, bootstrap_seed=20260926,
                      paired_knob_deltas=paired, mixed_scripted_minus_random=mixed_gaps,
                      both_parities_scripted_minus_random=swapped_gaps)
    (directory / f"{prefix}comparisons.json").write_text(json.dumps(comparison, indent=2, sort_keys=True) + "\n")
    files = sorted(p for p in directory.glob(f"{prefix}*") if not p.name.endswith("manifest.json"))
    manifest = {str(p): dict(bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in files}
    (directory / f"{prefix}manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"Wrote all aggregates, paired intervals and manifest ({len(files)} files)")


if __name__ == "__main__":
    main()

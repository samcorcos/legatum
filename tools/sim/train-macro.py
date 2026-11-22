#!/usr/bin/env python3
"""Fit pairwise macro values, quantize to fixed-point Rust weights, report held-out rank.

Requires NumPy. Rows come from the macro_labels example. The train and holdout files must
have disjoint seeds; no hidden state enters inference. Values are long-horizon rollout means.
"""
import argparse
import json
import pathlib
import sys

import numpy as np

FEATURES = 30
HEADS = 10


def load(paths):
    rows = []
    for path in paths:
        with open(path, encoding="utf-8") as file:
            rows.extend(json.loads(line) for line in file if line.strip())
    return rows


def design(option, scales):
    result = np.zeros(FEATURES * HEADS + 8 * HEADS)
    start = option["head"] * FEATURES
    result[start:start + FEATURES] = np.array(option["features"]) / scales
    faction = max(0, min(7, option["features"][3]))
    result[FEATURES * HEADS + option["head"] * 8 + faction] = 1
    return result


def score(rows, weights, scales):
    exact = first_two = 0
    regret = two_regret = 0
    total = 0
    by_kind = {}
    for row in rows:
        options = row["options"]
        if len(options) < 2:
            continue
        oracle = max(range(len(options)), key=lambda i: options[i]["value"])
        predicted = max(range(len(options)), key=lambda i: (design(options[i], scales) @ weights, -i))
        cheap = max(range(len(options)), key=lambda i: (options[i]["first_two"], -i))
        exact += predicted == oracle
        first_two += cheap == oracle
        regret += options[oracle]["value"] - options[predicted]["value"]
        two_regret += options[oracle]["value"] - options[cheap]["value"]
        total += 1
        kind = "opening" if row["turn"] == 1 else (
            "macro_plan" if all(o["head"] <= 2 for o in options) else "mixed_plan"
        )
        entry = by_kind.setdefault(kind, [0, 0, 0, 0, 0])
        entry[0] += 1
        entry[1] += predicted == oracle
        entry[2] += cheap == oracle
        entry[3] += options[oracle]["value"] - options[predicted]["value"]
        entry[4] += options[oracle]["value"] - options[cheap]["value"]
    return {"groups": total, "model_agreement": exact / total if total else None,
            "two_sample_agreement": first_two / total if total else None,
            "model_mean_regret": regret / total if total else None,
            "two_sample_mean_regret": two_regret / total if total else None,
            "by_kind": {k: {"groups": v[0], "model_agreement": v[1] / v[0],
                           "two_sample_agreement": v[2] / v[0],
                           "model_mean_regret": v[3] / v[0],
                           "two_sample_mean_regret": v[4] / v[0]}
                        for k, v in by_kind.items()}}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--train", type=pathlib.Path, nargs="+", required=True)
    parser.add_argument("--held-out", type=pathlib.Path, nargs="+", required=True)
    parser.add_argument("--source", type=pathlib.Path, required=True)
    parser.add_argument("--report", type=pathlib.Path, required=True)
    args = parser.parse_args()
    train = load(args.train)
    held = load(args.held_out)
    train_seeds = {r["seed"] for r in train}
    held_seeds = {r["seed"] for r in held}
    if train_seeds & held_seeds:
        sys.exit("training and held-out seeds overlap")
    if not train or not held:
        sys.exit("training and held-out rows are required")
    features = np.array([o["features"] for r in train for o in r["options"]], dtype=float)
    scales = np.maximum(np.max(np.abs(features), axis=0), 1)
    pairs = []
    targets = []
    for row in train:
        options = row["options"]
        for i, left in enumerate(options):
            for right in options[i + 1:]:
                pairs.append(design(left, scales) - design(right, scales))
                targets.append((left["value"] - right["value"]) / 1000)
    x = np.array(pairs)
    y = np.array(targets)
    # Ridge on within-decision value differences. Shared game strength cancels; targets
    # are the full 40/80-turn win-chance values, not a short defense objective.
    ridge = 1.0
    weights = np.linalg.solve(x.T @ x + ridge * np.eye(x.shape[1]), x.T @ y)
    raw = weights[:FEATURES * HEADS].reshape(HEADS, FEATURES) / scales * 1000 * 1024
    raw_bias = weights[FEATURES * HEADS:].reshape(HEADS, 8) * 1000 * 1024
    if max(np.max(np.abs(raw)), np.max(np.abs(raw_bias))) >= 2**31:
        sys.exit("quantized weight exceeds i32")
    integer = np.rint(raw).astype(np.int32)
    integer_bias = np.rint(raw_bias).astype(np.int32)
    text = args.source.read_text()
    rows = ",\n    ".join("[" + ", ".join(str(int(w)) for w in row) + "]" for row in integer)
    replacement = "pub const WEIGHTS: [[i32; FEATURES]; HEADS] = [\n    " + rows + ",\n];"
    marker = "pub const WEIGHTS: [[i32; FEATURES]; HEADS] = "
    if text.count(marker) != 1:
        sys.exit("could not replace Rust weights")
    start = text.index(marker)
    end = text.index("\n\npub fn value", start)
    bias_rows = ",\n    ".join("[" + ", ".join(str(int(w)) for w in row) + "]" for row in integer_bias)
    bias_replacement = "pub const FACTION_BIAS: [[i32; 8]; HEADS] = [\n    " + bias_rows + ",\n];"
    text = text[:start] + replacement + "\n" + bias_replacement + text[end:]
    args.source.write_text(text)
    quantized = np.concatenate((integer.reshape(-1) / 1024 / 1000 * np.tile(scales, HEADS),
                                integer_bias.reshape(-1) / 1024 / 1000))
    report = {"training_rows": len(train), "held_out_rows": len(held), "training_seeds": sorted(train_seeds),
              "held_out_seeds": sorted(held_seeds), "pairs": len(pairs), "ridge": ridge,
              "training": score(train, quantized, scales), "held_out": score(held, quantized, scales)}
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report["held_out"], indent=2))


if __name__ == "__main__":
    main()

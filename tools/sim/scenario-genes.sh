#!/usr/bin/env bash
# Score a Scenario Lab scenario with and without AI genes, every seat on the same parameters.
#
#   tools/cloud/remote.sh <worktree> --with-uncommitted -- tools/sim/scenario-genes.sh [--runs N] SCENARIO[,SCENARIO...] PARAMS.json [PARAMS.json ...]
#
# SCENARIO is a file in scenarios/lab/ (or scenarios/war/) without the extension. Prints, per scenario and
# parameter file, the score median and mean and how the runs ended. Include
# docs/sim-results/hard-vs-human/params/baseline.json as the control. Runs on the cloud build box.
set -euo pipefail
runs=100
if [ "${1:-}" = --runs ]; then runs="${2:?}"; shift 2; fi
scenarios="${1:?scenario id list}"; shift
cargo build --release -p legatum-sim --bin legatum-sim
dir="$(mktemp -d)"
trap 'rm -rf "$dir"' EXIT
for sc in ${scenarios//,/ }; do
  file="scenarios/lab/$sc.json"; [ -f "$file" ] || file="scenarios/war/$sc.json"
  for params in "$@"; do
    name="$(basename "$params" .json)"
    out="$dir/$sc-$name"; mkdir -p "$out"
    target/release/legatum-sim scenario "$file" --runs "$runs" --threads 16 --out "$out" --ai-params "$params" > "$out/log" 2>&1 || { tail -5 "$out/log"; exit 1; }
    echo "== $sc / $name"
    grep -E '^\| (Median|Mean|Objective met)|^Outcomes|^First war|× ' "$out"/*-baseline.md | head -12
  done
done

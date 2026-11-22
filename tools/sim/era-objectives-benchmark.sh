#!/usr/bin/env bash
# Human-seat benchmark arms for the era-objective rules: Hard in the user's recorded seat on every fixed map,
# reseeded dice, every seat on the same AI. Arms: default, and seek0 (the `objective_seek` gene at 0 in every seat).
#
#   tools/sim/era-objectives-benchmark.sh OUTDIR SALTS [ARM...]     ARM: default | seek0   (default: both)
#
# Run it on the box, once on master as the control and once on the branch (remote.sh takes a ref or a worktree):
#   tools/cloud/remote.sh master -- tools/sim/era-objectives-benchmark.sh out/master 128 default   (script from the branch: --with-uncommitted
#   or a ref that has it); compare with tools/sim/hard-vs-human-compare.py or the per-map win rates in each report.json.
set -euo pipefail
cd "$(dirname "$0")/../.."
out="$1"; salts="$2"; shift 2
[ $# -gt 0 ] || set -- default seek0
cargo build --release -p legatum-sim --example hard_vs_human 2>&1 | tail -1
for arm in "$@"; do
  mkdir -p "$out/$arm"
  case "$arm" in
    default) args=(--brain hard);;
    seek0) args=(--brain params --ai-params docs/sim-results/era-objectives/params/seek-0.json --all-seats);;
    *) echo "unknown arm $arm" >&2; exit 2;;
  esac
  echo "=== $arm: $salts salts per map"
  start=$(date +%s)
  LEGATUM_HVH_RESEED_DICE=1 target/release/examples/hard_vs_human "${args[@]}" --salts "$salts" --seed 1 \
    --out-dir "$out/$arm" > "$out/$arm/report.json"
  echo "seconds: $(( $(date +%s) - start ))"
  gzip -f "$out"/"$arm"/*-games.json 2>/dev/null || true
done

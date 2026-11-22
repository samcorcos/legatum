#!/usr/bin/env bash
# The live AI speed budget (plans/ai-war.md W0.3): 8-empire Standard games, every seat AI, per-seat
# planning and whole-lobby turn times, native (one thread) and WASM (Node/V8), Hard and Hard+Search.
#   tools/perf/ai-budget.sh                       # build both, 6 seeds x hard,search x native,wasm
#   SEEDS="7000001 7000002" TURNS=100 MODES=hard tools/perf/ai-budget.sh
# CSVs land in ~/LegatumRuns/ai-war-w0/budget/ (OUT=...); summarize with tools/perf/budget_summary.py. Runs JOBS (default 2)
# games at a time under nice, so the Mac isn't pinned.
set -euo pipefail
cd "$(dirname "$0")/../.."
SEEDS="${SEEDS:-7000001 7000002 7000003 7000004 7000005 7000006}"
TURNS="${TURNS:-260}"
MODES="${MODES:-hard search}"
JOBS="${JOBS:-2}"
OUT="${OUT:-$HOME/LegatumRuns/ai-war-w0/budget}"
mkdir -p "$OUT"

if [ -z "${SKIP_BUILD:-}" ]; then
  nice cargo build --release -p legatum-wasm --features seat-timing --example ai_budget 2>&1 | tail -1
  nice wasm-pack build crates/wasm --target nodejs --profile wasm-release \
    --out-dir ../../target/wasm-budget -- --features seat-timing 2>&1 | tail -2
fi
ls -l target/wasm-budget/*.wasm

jobs=()
for mode in $MODES; do
  for seed in $SEEDS; do
    jobs+=("RAYON_NUM_THREADS=1 target/release/examples/ai_budget $seed $TURNS $mode > $OUT/native-$mode-$seed.csv")
    jobs+=("node scripts/perf-wasm-budget.mjs target/wasm-budget $seed $TURNS $mode > $OUT/wasm-$mode-$seed.csv")
  done
done
printf '%s\0' "${jobs[@]}" | xargs -0 -n1 -P "$JOBS" nice bash -c

for mode in $MODES; do
  for engine in native wasm; do
    echo "== $engine $mode"
    tools/perf/budget_summary.py "$OUT"/$engine-$mode-*.csv
  done
  for seed in $SEEDS; do
    tools/perf/budget_summary.py --same-hashes "$OUT/native-$mode-$seed.csv" "$OUT/wasm-$mode-$seed.csv"
  done
done

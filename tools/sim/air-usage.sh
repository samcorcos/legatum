#!/usr/bin/env bash
# How often the AIs build and use aircraft: records full all-AI games with order tapes, then replays them
# counting Bomber and Fighter builds, strike orders, landed strikes and interceptions
# (crates/sim/examples/air_usage.rs). Two arms on one binary: the Fighter-strike gene off (Fighters never strike: the old AI) and on (the default).
#
#   tools/sim/air-usage.sh [GAMES] [FIRST_SEED]        (run it on the box: tools/cloud/remote.sh <worktree> -- tools/sim/air-usage.sh 96)
set -euo pipefail
games="${1:-96}"; seed="${2:-9200001}"
cd "$(dirname "$0")/../.."
cargo build --release -p legatum-sim --bin legatum-sim --example air_usage 2>&1 | tail -2
out="${LEGATUM_AIR_USAGE_DIR:-target/air-usage}"; rm -rf "$out"; mkdir -p "$out"
for arm in off on; do
  extra=()
  [ "$arm" = on ] || extra=(--ai-params docs/sim-results/hard-vs-human/params/fighter-strike-off.json)
  echo "=== Fighter strike gene $arm: $games games, seeds from $seed"
  target/release/legatum-sim --games "$games" --seed "$seed" --players 8 --difficulty hard \
    --tape-out "$out/tapes-$arm" --out-dir "$out/report-$arm" --label "air-usage-$arm" ${extra[@]+"${extra[@]}"} 2>&1 | tail -4
  target/release/examples/air_usage "$out/tapes-$arm"
done

#!/usr/bin/env bash
# Peace-then-war flip-flops and city churn in all-AI games, from the engine's own events.
#
#   tools/cloud/remote.sh <worktree> --with-uncommitted -- tools/sim/flip-metrics.sh [--games N] [--seed S] [--params FILE] [--threads T] [--trace N]
#
# Plays N eight-player Hard games (random factions, Standard speed, barbarians on) with --tape-out, then
# replays the tapes with the `war_metrics` example: wars and captures per game, declarations within 5/10/30
# turns of a Peace with the same target (by the declarer's faction), and cities captured again within 20 turns.
# --params applies an AiParams file to every seat. Runs on the cloud build box (never the Mac).
set -euo pipefail
games=192; seed=7300001; params=""; threads=24; trace=0
while [ $# -gt 0 ]; do
  case "$1" in
    --games) games="${2:?}"; shift 2;;
    --seed) seed="${2:?}"; shift 2;;
    --params) params="${2:?}"; shift 2;;
    --threads) threads="${2:?}"; shift 2;;
    --trace) trace="${2:?}"; shift 2;;
    -h|--help) sed -n 2,9p "$0"; exit 0;;
    *) echo "unknown option: $1" >&2; exit 2;;
  esac
done
cargo build --release -p legatum-sim --bin legatum-sim --example war_metrics --example flip_trace
dir="$(mktemp -d)"
trap 'rm -rf "$dir"' EXIT
args=(--players 8 --difficulty hard --barbarians true --games "$games" --seed "$seed" --threads "$threads"
  --tape-out "$dir/tapes" --out-dir "$dir/out" --out "$dir/replays" --label flip-metrics)
[ -z "$params" ] || args+=(--ai-params "$params")
target/release/legatum-sim "${args[@]}" > "$dir/sim.log" 2>&1 || { tail -20 "$dir/sim.log"; exit 1; }
tail -3 "$dir/sim.log"
target/release/examples/war_metrics "$dir/tapes" "$threads"
if [ "$trace" -gt 0 ]; then
  echo "--- flip_trace (first $trace tapes)"
  for t in $(ls "$dir"/tapes/*.json | head -"$trace"); do
    echo "=== $(basename "$t")"
    target/release/examples/flip_trace "$t" 10 ${params:+"$params"}
  done
fi

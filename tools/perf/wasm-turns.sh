#!/usr/bin/env bash
# WASM per-turn latency (Node/V8) replaying the native turn_profile games; run on the box.
#   tools/cloud/remote.sh . --with-uncommitted --artifact out -- bash tools/perf/wasm-turns.sh
set -e
mkdir -p out
cargo build --release -p legatum-wasm --example turn_profile 2>&1 | tail -1
for s in 7000001 7000002 7000003 7000004; do
  RAYON_NUM_THREADS=1 target/release/examples/turn_profile 400 $s csv out/orders-$s.jsonl > out/native-$s.csv &
done
wait
wasm-pack build crates/wasm --target nodejs --release --out-dir ../../target/wasm-node 2>&1 | tail -2
ls -l target/wasm-node/*.wasm
for s in 7000001 7000002 7000003 7000004; do
  node scripts/perf-wasm-turns.mjs target/wasm-node $s out/orders-$s.jsonl > out/wasm-$s.csv &
done
wait
tools/perf/summarize_turns.py out/native-*.csv
tools/perf/summarize_turns.py out/wasm-*.csv

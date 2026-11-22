#!/usr/bin/env bash
# A/B the shipped WASM build on the same replayed games: default release (wasm-pack) against
# the dedicated wasm-release profile (fat LTO, one codegen unit, wasm-opt -O3). Run on the box:
#   tools/cloud/remote.sh . --with-uncommitted --artifact out -- bash tools/perf/wasm-lto.sh
set -euo pipefail
mkdir -p out
tools/sim/build-slot.sh -- cargo build --release -p legatum-wasm --example turn_profile 2>&1 | tail -1
for s in 7000001 7000002 7000003 7000004; do
  RAYON_NUM_THREADS=1 target/release/examples/turn_profile 400 $s csv out/orders-$s.jsonl > out/native-$s.csv &
done
wait
wasm-pack build crates/wasm --target nodejs --release --out-dir ../../target/wasm-node 2>&1 | tail -1
rm -rf target/wasm-final-clean
start=$SECONDS
CARGO_TARGET_DIR=target/wasm-final-clean \
  wasm-pack build crates/wasm --target nodejs --profile wasm-release \
    --out-dir ../../target/wasm-node-lto 2>&1 | tail -1
echo "clean final WASM build: $((SECONDS - start))s"
ls -l target/wasm-node/*.wasm target/wasm-node-lto/*.wasm
for variant in wasm-node wasm-node-lto; do
  for s in 7000001 7000002 7000003 7000004; do
    node scripts/perf-wasm-turns.mjs target/$variant $s out/orders-$s.jsonl > out/$variant-$s.csv &
  done
  wait
  echo "== $variant"; tools/perf/summarize_turns.py out/$variant-*.csv
done

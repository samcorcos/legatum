#!/usr/bin/env bash
# Compare Cargo opt levels and Binaryen passes on the same four replayed games.
# Run on the box: tools/cloud/remote.sh . --with-uncommitted --artifact out -- bash tools/perf/wasm-profile-matrix.sh
set -euo pipefail
mkdir -p out
tools/sim/build-slot.sh -- cargo build --release -p legatum-wasm --example turn_profile > out/wasm-matrix-native-build.log 2>&1
for seed in 7000001 7000002 7000003 7000004; do
  RAYON_NUM_THREADS=1 target/release/examples/turn_profile 400 "$seed" csv "out/orders-$seed.jsonl" > "out/native-$seed.csv" &
done
wait

target_dir="$PWD/target/wasm-profile-matrix"
rm -rf "$target_dir"
build() {
  local level="$1" pass="$2" dir="target/wasm-node-opt${1}-${2}" start=$SECONDS wasm tmp wasm_opt flag
  CARGO_TARGET_DIR="$target_dir" CARGO_PROFILE_WASM_RELEASE_OPT_LEVEL="$level" \
    wasm-pack build crates/wasm --target nodejs --profile wasm-release --no-opt \
      --out-dir "../../$dir" > "out/wasm-opt${level}-${pass}-build.log" 2>&1
  wasm_opt="$(command -v wasm-opt || true)"
  if [ -z "$wasm_opt" ]; then
    wasm_opt="$(find "$HOME/.cache/.wasm-pack" -type f -name wasm-opt -print -quit)"
  fi
  [ -x "$wasm_opt" ] || { echo 'wasm-opt executable not found' >&2; exit 1; }
  wasm="$dir/legatum_wasm_bg.wasm"
  tmp="$wasm.optimized"
  flag=-O
  [ "$pass" != o3 ] || flag=-O3
  "$wasm_opt" "$flag" "$wasm" -o "$tmp"
  mv "$tmp" "$wasm"
  printf 'opt%s %s: build %ss, wasm %s bytes\n' "$level" "$pass" "$((SECONDS - start))" \
    "$(wc -c < "$dir/legatum_wasm_bg.wasm")"
  for seed in 7000001 7000002 7000003 7000004; do
    node scripts/perf-wasm-turns.mjs "$dir" "$seed" "out/orders-$seed.jsonl" \
      > "out/wasm-opt${level}-${pass}-$seed.csv" &
  done
  wait
  tools/perf/summarize_turns.py "out/wasm-opt${level}-${pass}-"*.csv
}
build 3 o
build 3 o3
build 2 o
build 2 o3

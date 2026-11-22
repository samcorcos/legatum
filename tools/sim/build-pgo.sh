#!/usr/bin/env bash
# Profile-guided build of the simulator (sim profile: fat LTO + PGO). Rebuild after big engine or
# AI changes; a stale profile still works but helps less.
#
#   tools/sim/build-pgo.sh            # -> target/pgo/sim/legatum-sim, target/pgo/sim/examples/sim_bench
#   NATIVE=1 tools/sim/build-pgo.sh   # also -C target-cpu=native (measured: no gain on Apple silicon)
#
# 1. Instrumented build (target/pgo-gen), 2. train on the bench's duel, small-map duel, eight-player
# and rollout workloads (one thread, a few minutes), 3. merge the profiles, 4. optimized build.
# Only the simulator is built this way; WASM and the app keep their normal profiles.
# Results are byte-identical (the bench digests don't move); see docs/sim-results/perf.
set -euo pipefail
cd "$(dirname "$0")/../.."
renice 10 -p $$ >/dev/null 2>&1 || true

data="$PWD/target/pgo-data"
profile="$PWD/target/pgo.profdata"
extra=""
[ "${NATIVE:-0}" = 1 ] && extra="-C target-cpu=native"

# llvm-profdata: rustup's llvm-tools if installed (matches rustc's LLVM), else Xcode's.
profdata=$(ls ~/.rustup/toolchains/*/lib/rustlib/*/bin/llvm-profdata 2>/dev/null | head -1 || true)
[ -n "$profdata" ] || profdata="xcrun llvm-profdata"

rm -rf "$data" && mkdir -p "$data"
echo "== instrumented build"
RUSTFLAGS="-Cprofile-generate=$data $extra" CARGO_TARGET_DIR=target/pgo-gen \
  cargo build --profile sim -p legatum-sim --example sim_bench
bench=target/pgo-gen/sim/examples/sim_bench
echo "== training"
"$bench" duel 32 1
"$bench" small 16 1
"$bench" eight 4 1
"$bench" rollout
echo "== merge ($profdata)"
$profdata merge -o "$profile" "$data"/*.profraw
echo "== optimized build"
RUSTFLAGS="-Cprofile-use=$profile -Cllvm-args=-pgo-warn-missing-function=false $extra" \
  CARGO_TARGET_DIR=target/pgo cargo build --profile sim -p legatum-sim --bin legatum-sim --example sim_bench
echo "built target/pgo/sim/legatum-sim and target/pgo/sim/examples/sim_bench"

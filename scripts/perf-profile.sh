#!/usr/bin/env bash
# Reproducible single-threaded B13 corpus and profiler. Run from this worktree only.
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1
root="$CARGO_TARGET_DIR/performance-pass"
corpus="$root/corpus"
mkdir -p "$root"
case "${1:-}" in
  record)
    # Refuse to overwrite the original input/expected-output evidence.
    test ! -e "$corpus"
    git rev-parse HEAD > "$root/source-sha.txt"
    rustc -Vv > "$root/rustc.txt"
    cargo build --release -p legatum-ai --example perf_profile
    cp "$CARGO_TARGET_DIR/release/examples/perf_profile" "$root/baseline-perf_profile"
    "$root/baseline-perf_profile" record "$corpus"
    PERF_CORPUS_DIR="$corpus" PERF_JSON_MODE=record cargo test --release -p legatum-core \
      --test visibility_equivalence recorded_corpus_states_and_serialized_views_match -- --ignored --exact --nocapture
    ;;
  measure)
    label="${2:?provide a unique measurement label}"
    [[ "$label" =~ ^[a-zA-Z0-9_-]+$ ]]
    test ! -e "$root/$label.csv"
    if [[ $# -ge 3 ]]; then
      binary="$3"
    else
      cargo build --release -p legatum-ai --example perf_profile
      binary="$CARGO_TARGET_DIR/release/examples/perf_profile"
    fi
    "$binary" measure "$corpus" "${PERF_SAMPLES:-31}" "${PERF_WARMUP:-5}" "${PERF_BATCH:-5}" \
      > "$root/$label.csv" 2> "$root/$label.log"
    ;;
  verify-json)
    PERF_CORPUS_DIR="$corpus" PERF_JSON_MODE=verify cargo test --release -p legatum-core \
      --test visibility_equivalence recorded_corpus_states_and_serialized_views_match -- --ignored --exact --nocapture
    ;;
  *)
    echo 'usage: scripts/perf-profile.sh record | measure LABEL [BINARY] | verify-json' >&2
    exit 2
    ;;
esac

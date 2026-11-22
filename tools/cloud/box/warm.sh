#!/usr/bin/env bash
# Build the standard gate artifacts for master in the golden slot, so new job slots seed their target/ from it.
# Run via `tools/cloud/box.sh warm` (a golden-slot job). Each step is attempted even if an earlier one failed.
set -uo pipefail

# The golden slot is shared by every box job, so an exact master tree only needs one warm.
# Keep the receipt under target/ (gitignored and preserved by git clean) so it is removed
# automatically if the cached Cargo artifacts are removed.
warm_key() {
  local wasm_opt_version="not-used"
  if [ "${LEGATUM_WASM_NO_OPT:-0}" != 1 ]; then
    wasm_opt_version="$(wasm-opt --version 2>/dev/null || echo unavailable)"
  fi
  {
    git rev-parse 'HEAD^{tree}'
    rustc -Vv
    wasm-pack --version
    printf '%s\n' "$wasm_opt_version"
    env | LC_ALL=C sort | grep -E '^(CARGO|RUST|LEGATUM_WASM|CC=|CXX=|AR=|CFLAGS=|CXXFLAGS=|LDFLAGS=|PKG_CONFIG)' || true
  } | sha256sum | awk '{print $1}'
}

key="$(warm_key)"
receipt="${LEGATUM_BOX_SLOT_DIR:-$PWD}/target/.legatum-warm-key"
if [ -f "$receipt" ] && [ "$(cat "$receipt")" = "$key" ]; then
  echo "warm skipped: master tree and build environment already have a successful golden-slot warm"
  exit 0
fi

fail=0
step() { echo "== $*"; "$@" || { echo "FAILED: $*"; fail=$((fail + 1)); }; }
step cargo fetch --locked
step cargo clippy --workspace --all-targets --release -- -D warnings
step cargo test --workspace --profile release-checked --no-run
step cargo build --release -p legatum-sim
step scripts/build-wasm.sh
[ ! -f app/package.json ] || { cd app && step npm run build; }
echo "warm done, $fail failed step(s)"
if [ "$fail" -eq 0 ]; then
  mkdir -p "$(dirname "$receipt")"
  printf '%s\n' "$key" > "$receipt"
fi
exit "$fail"

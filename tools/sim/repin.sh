#!/usr/bin/env bash
# Regenerate the three mixed AI tapes, rehash the fixed-order tapes, and repin runner digests.
#
# LEGATUM_REPIN_PROFILE selects the cargo profile (default release). tools/astra/verify-branch.sh sets
# release-checked: it reuses the artifacts of the gate's `cargo test --workspace --profile release-checked
# --no-run` build (it runs that build first, a no-op when fresh) instead of compiling a second feature set of the
# same crates (a `-p legatum-sim` build and the workspace test build invalidate each other's legatum-wasm
# artifacts). The pins are integer maths, so the profile does not change them. Callers wrap this whole script in
# tools/sim/build-slot.sh when they want a build slot; it takes none itself (build-slot is not re-entrant).
set -euo pipefail
# Codex lanes run this on the cloud build box (what it changes comes back); see tools/cloud/offload.sh.
_self="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
if [ "${LEGATUM_OFFLOAD:-0}" = 1 ] && [ -z "${LEGATUM_ON_BOX:-}" ] && [ -x "$(dirname "$0")/../cloud/offload.sh" ]; then
  exec "$(dirname "$0")/../cloud/offload.sh" -- "$_self" "$@"
fi

if [ "${LEGATUM_TIMED_REPIN:-0}" != 1 ] && [ -f "$(dirname "$0")/../ops/timed.py" ]; then
  exec python3 "$(dirname "$0")/../ops/timed.py" build repin --label "threads=${LEGATUM_REPIN_THREADS:-6}" -- \
    env LEGATUM_TIMED_REPIN=1 "$0" "$@"
fi

cd "$(dirname "$0")/../.."
fixtures=crates/sim/tests/fixtures
# Threads for the sim runs; LEGATUM_REPIN_THREADS=4 keeps a machine with a live game responsive.
threads="${LEGATUM_REPIN_THREADS:-6}"
profile="${LEGATUM_REPIN_PROFILE:-release}"

tapes=(--games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed
  --threads "$threads" --label determinism-fixtures --tape-out "$fixtures" --out-dir target/fixture-reports)

if [ "$profile" = release ]; then
  # Keep the sim under the shared machine-wide thread cap.
  tools/sim/slot.sh --threads "$threads" -- cargo run --release -p legatum-sim -- "${tapes[@]}"

  cargo run --release -p legatum-sim --example rehash_fixtures -- "$fixtures"/*.json

  # This ignored test uses exactly the same runner and hash functions as the pin assertions.
  tools/sim/slot.sh --threads "$threads" -- cargo test --release -p legatum-sim --bin legatum-sim \
    runner::tests::repin_runner_tapes -- --ignored --exact
  tools/sim/slot.sh --threads "$threads" -- cargo test --release -p legatum-sim --bin legatum-sim pinned
  cargo test --release -p legatum-sim --test determinism
else
  sim="target/$profile/legatum-sim"
  rehash="target/$profile/examples/rehash_fixtures"
  # A no-op when the gate's test build is fresh; builds the binaries (and makes sure they are current) otherwise.
  cargo test --workspace --profile "$profile" --no-run

  tools/sim/slot.sh --threads "$threads" -- "$sim" "${tapes[@]}"

  "$rehash" "$fixtures"/*.json

  # Every cargo call here selects the whole workspace with no --bin/--test filter: the unit graph (and so the
  # feature set and legatum-wasm's artifacts) then stays the one the gate's test build made. A filter such as
  # `--test determinism` builds a smaller graph and forces a rebuild of five crates. Test-name filters do the
  # narrowing instead; binaries with no matching test start and exit at once.
  tools/sim/slot.sh --threads "$threads" -- cargo test --workspace --profile "$profile" -- \
    --ignored --exact runner::tests::repin_runner_tapes
  tools/sim/slot.sh --threads "$threads" -- cargo test --workspace --profile "$profile" -- pinned
  if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --workspace --cargo-profile "$profile" -E 'binary(=determinism)'
  else
    cargo test --workspace --profile "$profile" --test determinism
  fi
fi

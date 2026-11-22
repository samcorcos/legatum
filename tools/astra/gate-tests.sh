#!/usr/bin/env bash
# The gate's Rust test run. Same tests as `cargo test --workspace --profile <profile>`,
# including examples that opt into tests in their Cargo manifests,
# executed by cargo-nextest: every test is its own process, scheduled across all binaries at once, instead of one
# test binary after another. Nextest does not run doctests, so they run separately.
#
#   tools/astra/gate-tests.sh [profile]        profile defaults to release-checked (overflow checks and
#                                              debug assertions on in an optimised build); `release` also works
#
# LEGATUM_GATE_TEST_THREADS sets the number of concurrent tests (default: nextest picks the core count).
# LEGATUM_GATE_SCOPE=fast skips the long AI, sim and property tests listed in tools/astra/extended-rust-tests.txt
# (the fast profile of verify-branch.sh); anything else, or unset, runs every test (extended and release profiles).
# Without cargo-nextest (`cargo install cargo-nextest --locked`) it falls back to plain `cargo test`.
set -uo pipefail
profile="${1:-release-checked}"
threads="${LEGATUM_GATE_TEST_THREADS:-}"
scope="${LEGATUM_GATE_SCOPE:-extended}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if cargo nextest --version >/dev/null 2>&1; then
  args=(run --workspace --no-fail-fast --cargo-profile "$profile")
  [ -z "$threads" ] || args+=(--test-threads "$threads")
  if [ "$scope" = fast ]; then
    skip="$(sed -e 's/[[:space:]]*#.*$//' -e '/^[[:space:]]*$/d' "$here/extended-rust-tests.txt" | paste -sd'|' -)"
    [ -z "$skip" ] || args+=(-E "not ($skip)")
    echo "gate-tests: fast scope, skipping the extended tests in tools/astra/extended-rust-tests.txt" >&2
  fi
  cargo nextest "${args[@]}" || exit $?
  cargo test --workspace --doc --profile "$profile"
else
  echo "gate-tests: cargo-nextest not installed; running cargo test (slower, and always every test)" >&2
  cargo test --workspace --no-fail-fast --profile "$profile"
fi

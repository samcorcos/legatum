#!/usr/bin/env bash
# The box's cargo shim turns simple `cargo test` runs into nextest (+ doctests) and leaves everything else alone.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/cargo/bin"
cat > "$tmp/cargo/bin/cargo" <<'S'
#!/usr/bin/env bash
if [ "${1:-}" = nextest ] && [ "${2:-}" = --version ]; then exit 0; fi
echo "cargo $*" >> "$STUB_LOG"
[ -z "${STUB_FAIL:-}" ] || { case "$*" in *"$STUB_FAIL"*) exit 7;; esac; }
S
chmod +x "$tmp/cargo/bin/cargo"
export CARGO_HOME="$tmp/cargo" STUB_LOG="$tmp/log"
run() { : > "$STUB_LOG"; "$here/shim/cargo" "$@" 2>/dev/null; cat "$STUB_LOG" | tr '\n' '|'; }
check() { [ "$1" = "$2" ] || { echo "FAIL: expected [$2] got [$1]" >&2; exit 1; }; }

check "$(run test --workspace)" 'cargo nextest run --no-fail-fast --no-tests=pass --workspace|cargo test --doc --workspace|'
check "$(run test -p legatum-core --release foo)" 'cargo nextest run --no-fail-fast --no-tests=pass -p legatum-core --release foo|cargo test --doc -p legatum-core --release foo|'
check "$(run test --no-fail-fast --profile release-checked)" 'cargo nextest run --no-fail-fast --no-tests=pass --cargo-profile release-checked|cargo test --doc --profile release-checked|'
check "$(run test --profile=release-checked)" 'cargo nextest run --no-fail-fast --no-tests=pass --cargo-profile=release-checked|cargo test --doc --profile=release-checked|'
check "$(run test -p legatum-sim --test hvh)" 'cargo nextest run --no-fail-fast --no-tests=pass -p legatum-sim --test hvh|'
check "$(run test --workspace -- --nocapture)" 'cargo test --workspace -- --nocapture|'
check "$(run test --no-run)" 'cargo test --no-run|'
check "$(run test --doc)" 'cargo test --doc|'
check "$(run clippy --workspace)" 'cargo clippy --workspace|'
check "$(LEGATUM_BOX_NO_NEXTEST=1 run test --workspace)" 'cargo test --workspace|'
rc=0; STUB_FAIL="nextest run" "$here/shim/cargo" test --workspace >/dev/null 2>&1 || rc=$?
[ "$rc" = 7 ] || { echo "FAIL: a failing nextest run must fail the command (rc $rc)" >&2; exit 1; }
echo "cargo shim test passed"

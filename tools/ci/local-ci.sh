#!/usr/bin/env bash
# Run the same checks as .github/workflows/ci.yml on this machine (a Claude cloud container, the user's Mac or the
# build box), one after another, and print a Markdown summary to paste into the PR as the CI record.
#
#   tools/ci/local-ci.sh [--base origin/master] [--summary FILE]
#
# Used while GitHub Actions is unavailable (2026-11-21: the private repo's Actions limit). Every step runs even if an
# earlier one fails, so one run shows every failure. In a Claude cloud container set LEGATUM_WASM_NO_OPT=1: its proxy
# blocks the binaryen download wasm-pack needs for wasm-opt. Exit status is non-zero if any step failed.
set -uo pipefail
cd "$(dirname "$0")/../.."

base="origin/master"
summary=""
while [ $# -gt 0 ]; do
  case "$1" in
    --base) base="$2"; shift 2 ;;
    --summary) summary="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[ -n "$summary" ] || summary="$(mktemp -t local-ci-XXXX.md)"

rows=()
failed=0
step() {
  local name="$1"; shift
  local start=$SECONDS
  echo "::: $name"
  if "$@"; then
    rows+=("| $name | pass | $(( (SECONDS - start) / 60 ))m$(( (SECONDS - start) % 60 ))s |")
  else
    rows+=("| $name | **FAIL** | $(( (SECONDS - start) / 60 ))m$(( (SECONDS - start) % 60 ))s |")
    failed=1
  fi
}

rust_group() {
  # shellcheck disable=SC2046
  if cargo nextest --version >/dev/null 2>&1; then
    cargo nextest run --cargo-profile release $(python3 tools/ci/rust_test_groups.py args "$1") --no-fail-fast
  else
    cargo test --release --no-fail-fast $(python3 tools/ci/rust_test_groups.py args "$1")
  fi
}

step "Gates (check.py --checks-only)" python3 tools/gates/check.py HEAD --base "$base" --checks-only --skip art
step "Test groups cover every target" python3 tools/ci/rust_test_groups.py check
step "Rust format" cargo fmt --all --check
step "Rust lint (clippy --all-targets)" cargo clippy --workspace --all-targets -- -D warnings
step "Doctests" cargo test --workspace --doc
for g in $(python3 tools/ci/rust_test_groups.py names | tr -d '[]",'); do
  step "Rust tests: $g" rust_group "$g"
done
if ! command -v wasm-pack >/dev/null 2>&1; then
  # Same version as CI; a fresh cloud container has none (about a minute to build).
  step "Install wasm-pack 0.15.0" cargo install wasm-pack --version 0.15.0 --locked
fi
step "WASM build (playtest profile)" env LEGATUM_PLAYTEST_PROFILE=1 scripts/build-wasm.sh
step "App dependencies" bash -c 'cd app && npm ci'
if [ -z "${PLAYWRIGHT_BROWSERS_PATH:-}" ] && [ ! -d /opt/pw-browsers ]; then
  step "Chromium for art render tests" bash -c 'cd app && npx playwright install chromium'
fi
step "App tests" bash -c 'cd app && npm test'
step "App build" bash -c 'cd app && npm run build'
step "App build references" node tools/release/check-dist-refs.mjs app/dist

{
  echo "### Local CI: $(git rev-parse --short HEAD) on $(uname -sm), $(getconf _NPROCESSORS_ONLN 2>/dev/null || echo '?') cores"
  echo
  echo "The same checks as \`.github/workflows/ci.yml\`, run by \`tools/ci/local-ci.sh\` while GitHub Actions is unavailable."
  [ "${LEGATUM_WASM_NO_OPT:-0}" != 1 ] || echo "WASM built without wasm-opt (LEGATUM_WASM_NO_OPT=1: this host cannot download binaryen); wasm-opt only shrinks the file."
  echo
  echo "| Step | Result | Time |"
  echo "| --- | --- | --- |"
  printf '%s\n' "${rows[@]}"
  echo
  if [ "$failed" = 0 ]; then echo "**Result: all checks passed.**"; else echo "**Result: FAILED.**"; fi
} > "$summary"
echo
cat "$summary"
exit "$failed"

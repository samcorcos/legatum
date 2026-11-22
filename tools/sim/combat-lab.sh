#!/usr/bin/env bash
# The combat lab (crates/sim/examples/combat_lab.rs; docs in docs/sim-results/combat-lab/README.md).
#
#   tools/sim/combat-lab.sh sweep  [--repeats 32] [-- combat_lab args]   full grid on the cloud box, then the report
#   tools/sim/combat-lab.sh check                                        fixed subset against the committed baseline
#   tools/sim/combat-lab.sh baseline                                     rewrite the fixed-subset baseline
#
# `check` is the regression gate for tactical changes: 21 cells, 12 dice seeds each, five
# policies against the live defender. It exits 1 if the AI's capture rate in any fixed cell fell
# by more than 34 points, a forced policy moved by more than 34, or the mean AI capture rate fell
# by more than 5 points. After an intentional change
# that moves the numbers, rerun `baseline` and commit the new file with an explanation.
# Heavy work runs on the cloud build box (tools/cloud/remote.sh), never on this Mac.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
out_dir="${COMBAT_LAB_OUT:-docs/sim-results/combat-lab}"
repo_dir="docs/sim-results/combat-lab"
baseline="$repo_dir/fixed-baseline.json"
build='set -eo pipefail; mkdir -p docs/sim-results/combat-lab; cargo build --release -p legatum-sim --example combat_lab 2>&1 | tail -3; '
mode="${1:-}"; [ $# -eq 0 ] || shift
case "$mode" in
  sweep)
    repeats=32
    while [ $# -gt 0 ] && [ "$1" != "--" ]; do
      case "$1" in
        --repeats) repeats="${2:?}"; shift 2;;
        *) echo "unknown option: $1" >&2; exit 2;;
      esac
    done
    [ "${1:-}" != "--" ] || shift
    mkdir -p "$out_dir"
    artifacts="$(mktemp -d)"
    tools/cloud/remote.sh . --with-uncommitted --label combat-lab-sweep --artifact out --artifact-dir "$artifacts" -- \
      bash -c "$build./target/release/examples/combat_lab --repeats $repeats --out out $*"
    artifacts="$artifacts/out"
    cp "$artifacts/summary.json" "$out_dir/summary.json"
    gzip -c "$artifacts/cells.csv" > "$out_dir/cells.csv.gz"
    python3 tools/sim/combat-lab-report.py "$out_dir/cells.csv.gz" --summary "$out_dir/summary.json" --out "$out_dir"
    case "$out_dir" in
      docs/sim-results/*)
        python3 tools/sim/freshness.py stamp "$out_dir/summary.json" "$out_dir/cells.csv.gz" \
          "$out_dir/results.md" "$out_dir/forecast-vs-storm.png" "$out_dir/policies.png";;
    esac
    ;;
  check)
    tools/cloud/remote.sh . --with-uncommitted --label combat-lab-check -- \
      bash -c "$build./target/release/examples/combat_lab --subset fixed --check $baseline"
    ;;
  baseline)
    mkdir -p "$repo_dir"
    artifacts="$(mktemp -d)"
    tools/cloud/remote.sh . --with-uncommitted --label combat-lab-baseline --artifact "$baseline" --artifact-dir "$artifacts" -- \
      bash -c "$build./target/release/examples/combat_lab --subset fixed --write-baseline $baseline"
    cp "$artifacts/$baseline" "$baseline"
    python3 tools/sim/freshness.py stamp "$baseline"
    ;;
  *)
    sed -n 2,13p "$0"; exit 2;;
esac

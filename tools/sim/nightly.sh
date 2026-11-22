#!/usr/bin/env bash
# Run the complete master battery. This is an orchestrator; simulation and builds
# run only on the cloud build box or on run-batch.sh's budgeted spot instances.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

dry=0
case "${1:-}" in
  "") ;;
  --dry-run) dry=1; shift;;
  *) echo "usage: tools/sim/nightly.sh [--dry-run]" >&2; exit 2;;
esac
[ "$#" -eq 0 ] || { echo "unexpected arguments" >&2; exit 2; }
master="$(git rev-parse master)"
head="$(git rev-parse HEAD)"
[ "$(git merge-base master HEAD)" = "$master" ] || { echo "merge current master into this checkout first" >&2; exit 1; }
git diff --quiet master -- crates scenarios Cargo.toml Cargo.lock || { echo "engine or scenarios differ from master" >&2; exit 1; }
git diff --quiet HEAD -- crates scenarios || { echo "engine or scenarios are uncommitted" >&2; exit 1; }
rules="$(python3 tools/sim/freshness.py --help >/dev/null; git show master:crates/core/src/lib.rs | sed -n 's/^pub const RULES_VERSION: u32 = \([0-9]*\);/\1/p')"
[ -n "$rules" ] || { echo "cannot find master's RULES_VERSION" >&2; exit 1; }
history=docs/sim-results/nightly/history.jsonl
if [ -f "$history" ] && python3 - "$history" "$master" <<'PY'
import json,sys
sys.exit(0 if any(json.loads(line).get('commit')==sys.argv[2] for line in open(sys.argv[1]) if line.strip()) else 1)
PY
then
  echo "nightly: master $master already measured; skipping"
  exit 0
fi
if [ "$dry" = 1 ]; then
  echo "nightly plan: master $master, rules v$rules"
  echo "daily: 4096 Hard games seed 9311003; quality-gate default arm 512 salts x 8 maps; war suite 64 runs; every Scenario Lab card 50 runs; combat fixed 21 cells x 12 seeds; designer-seat score 6 tapes x 5 start turns x 48 runs (box)"
  echo "weekly Sunday UTC: ablate.sh for all nine factions, 4096 games per arm, parallel 3; existing spot budget applies"
  exit 0
fi

work="$(mktemp -d)"
cleanup() {
  rc=$?
  tools/cloud/status.sh >&2 || true
  rm -rf "$work"
  exit "$rc"
}
trap cleanup EXIT
start="$(date +%s)"
box_seconds=0
spot_seconds=0
mkdir -p docs/sim-results/nightly

# One exact-source build serves the census, human-seat benchmark and war suite.
# A box failure must fail the battery; build-linux.sh is forbidden to use local Docker here.
step_start="$(date +%s)"
LEGATUM_CLOUD_REQUIRE_BOX=1 LEGATUM_CLOUD_EXAMPLES="hard_vs_human war_suite" tools/cloud/build-linux.sh
box_seconds=$((box_seconds + $(date +%s) - step_start))
build_commit="$(sed -n 's/^commit //p' target/cloud/linux-arm64/BUILD_INFO | head -1)"
case "$head" in "$build_commit"*) ;; *) echo "cloud binary is not from the checked source: $build_commit" >&2; exit 1;; esac

run_batch() {
  local name="$1"; shift
  local step_start="$(date +%s)"
  tools/cloud/run-batch.sh --count 1 --max-hours 2 "$@" | tee "$work/$name.log"
  spot_seconds=$((spot_seconds + $(date +%s) - step_start))
  sed -n 's/^results: \([^ ]*\).*/\1/p' "$work/$name.log" | tail -1 > "$work/$name.dir"
  [ -s "$work/$name.dir" ] || { echo "$name: no batch result directory" >&2; exit 1; }
  tools/cloud/status.sh
}
run_batch census -- --players 8 --difficulty hard --games 4096 --seed 9311003 --label nightly-census
census_dir="$(cat "$work/census.dir")"
[ -s "$census_dir/merged.json.gz" ] || { echo "missing merged census" >&2; exit 1; }

python3 - "$work/arms.json" <<'PY'
import json,sys
from tools.sim.quality_gate import ARMS
with open(sys.argv[1],'w') as f: json.dump(ARMS,f)
PY
run_batch human --bin hard_vs_human --ai-params "$work/arms.json" --env LEGATUM_HVH_RESEED_DICE=1 -- \
  --arms --games 512 --seed 1 --label nightly-human
human_dir="$(cat "$work/human.dir")"
human_report="$human_dir/slice-0/out/hard-vs-human-arms.json.gz"
[ -s "$human_report" ] || { echo "missing human-seat report" >&2; exit 1; }

run_batch war --bin war_suite -- --brain hard --games 64 --seed 0 --label nightly-war
war_dir="$(cat "$work/war.dir")"
war_report="$war_dir/slice-0/out/war-suite.json.gz"
[ -s "$war_report" ] || { echo "missing war-suite report" >&2; exit 1; }

# Designer-seat score (docs/status/designer-seat.md): Hard in his seat on his six tapes, one box job.
step_start="$(date +%s)"
python3 tools/decision-audit/designer_seat.py --where box --label nightly
box_seconds=$((box_seconds + $(date +%s) - step_start))

step_start="$(date +%s)"
tools/cloud/remote.sh . --with-uncommitted --label nightly-labs \
  --artifact docs/sim-results/nightly/scenario-lab-raw.json \
  --artifact docs/sim-results/nightly/combat-lab-raw.json --artifact-dir "$work" -- \
  bash -c 'tools/sim/build-slot.sh -- cargo build --locked --release -p legatum-sim --bin legatum-sim --example combat_lab && python3 tools/sim/nightly_scenarios.py --binary target/release/legatum-sim --out docs/sim-results/nightly/scenario-lab-raw.json && target/release/examples/combat_lab --subset fixed --write-baseline docs/sim-results/nightly/combat-lab-raw.json'
box_seconds=$((box_seconds + $(date +%s) - step_start))
scenario_report="$work/docs/sim-results/nightly/scenario-lab-raw.json"
[ -s "$scenario_report" ] || { echo "missing Scenario Lab report" >&2; exit 1; }
combat_report="$work/docs/sim-results/nightly/combat-lab-raw.json"
[ -s "$combat_report" ] || { echo "missing combat lab report" >&2; exit 1; }

if [ "$(date -u +%u)" = 7 ]; then
  for faction in germany switzerland india byzantium celts mongols sparta athens aztecs; do
    step_start="$(date +%s)"
    tools/sim/ablate.sh "$faction" --games 4096 --seed 9311003 --no-build --control-dir "$census_dir" --parallel 3
    tools/cloud/status.sh
    spot_seconds=$((spot_seconds + 3 * ($(date +%s) - step_start)))
  done
fi

defaults_hash="$(shasum -a 256 crates/ai/src/params.rs | awk '{print $1}')"
minutes="$(python3 - "$start" <<'PY'
import sys,time
print(round((time.time()-int(sys.argv[1]))/60,2))
PY
)"
box_minutes="$(awk -v s="$box_seconds" 'BEGIN {printf "%.2f", s/60}')"
spot_minutes="$(awk -v s="$spot_seconds" 'BEGIN {printf "%.2f", s/60}')"
python3 tools/sim/nightly_report.py --commit "$master" --build-commit "$head" \
  --rules-version "$rules" --ai-defaults-hash "$defaults_hash" \
  --census "$census_dir/merged.json.gz" --human "$human_report" --war "$war_report" \
  --scenarios "$scenario_report" --combat "$combat_report" --history "$history" \
  --chart docs/sim-results/nightly/trend.png --elapsed-minutes "$minutes" \
  --box-minutes "$box_minutes" --spot-minutes "$spot_minutes"
python3 tools/sim/freshness.py stamp --commit "$master" "$history" docs/sim-results/nightly/trend.png \
  docs/sim-results/nightly/census.json docs/sim-results/nightly/human-seat.json \
  docs/sim-results/nightly/war-suite.json docs/sim-results/nightly/scenario-lab.json \
  docs/sim-results/nightly/combat-lab.json docs/sim-results/designer-seat/history.jsonl
echo "nightly: complete; source master $master"

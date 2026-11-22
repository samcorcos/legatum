#!/usr/bin/env bash
# Score the AI on the war scenarios cut from the user's playtests (scenarios/war/, docs in
# docs/sim-results/hard-vs-human/war-scenarios.md). The hill-climb command for AI changes.
#
#   tools/sim/war-scenarios.sh [--brain hard|hard+search|training] [--runs 64]
#                              [--ai-params FILE [--params-seats seat|opponents|all]]
#                              [--ids a,b] [--label TEXT] [--local] [--no-ledger] [--check]
#
# --check is the attack regression check: it runs only The Golden Horde comes, thin treasury (the
# first AI offensive that beat the user, playtest 8.9.2, against a Swiss defender with 500 Gold)
# and exits 1 if the AI attacker took Aarau in fewer than ATTACK_FLOOR percent of the runs
# (standard error about 5 points at 64 runs; use --runs 128 for a tighter reading). It tests the
# attacker's execution against a fixed defence: every other seat plays ATTACK_DEFENDERS, the
# built-in AI with door defense off (2026-10-30: with the door on, the 500-Gold Swiss buys the
# counter at the door and every attack case reads 0%, so a floor there could catch nothing; the
# door's own effect is measured by the suite's defence cases). Its baseline with that defence is
# in docs/sim-results/ai-war/swiss-gold.md. The 2,235-Gold original no longer works as the check:
# since territory sight and the attack reveal the Hard Swiss defender sees the army coming and buys
# it off (docs/sim-results/hard-vs-human/war-scenarios/README.md).
# 2026-11-04: the historical 60% floor is stale on current rules (0/64). The saved Mongol army
# becomes unsupplied after one turn; see docs/sim-results/ai-war/golden-regression-1104/README.md.
# Leave the failure visible until a current-rules attack fixture supplies a calibrated floor.
#
# Default: builds the war_suite example on the cloud build box, runs it on one spot Graviton
# server (run-batch.sh --bin war_suite), then writes the report, appends a ledger row and redraws
# the chart. --local runs on this Mac instead (only when the user allows local CPU).
# Runs are seeded by index, so two reports pair run by run:
#   python3 tools/sim/war-scenarios-report.py --compare BASE.json CANDIDATE.json
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
brain=hard; runs=64; params=""; seats=seat; ids=""; label=""; mode=cloud; ledger=1; check=0
ATTACK_ID=war-golden-horde-thin; ATTACK_FLOOR=60; ATTACK_DEFENDERS=tools/sim/attack-check-defenders.json
opponents=""
while [ $# -gt 0 ]; do
  case "$1" in
    --brain) brain="${2:?}"; shift 2;;
    --runs) runs="${2:?}"; shift 2;;
    --ai-params) params="${2:?}"; shift 2;;
    --params-seats) seats="${2:?}"; shift 2;;
    --ids) ids="${2:?}"; shift 2;;
    --label) label="${2:?}"; shift 2;;
    --local) mode=local; shift;;
    --no-ledger) ledger=0; shift;;
    --check) check=1; ledger=0; ids="$ATTACK_ID"; opponents="$ATTACK_DEFENDERS"; shift;;
    -h|--help) sed -n 2,20p "$0"; exit 0;;
    *) echo "unknown option: $1" >&2; exit 2;;
  esac
done
case "$brain" in hard|hard+search|training) ;; *) echo "--brain must be hard, hard+search or training" >&2; exit 2;; esac
case "$runs" in ''|*[!0-9]*|0) echo "--runs must be a positive integer" >&2; exit 2;; esac

# Every scenario file must be built into the binary.
for f in scenarios/war/*.json; do
  grep -q "scenarios/war/$(basename "$f")" crates/sim/examples/war_suite.rs \
    || { echo "$f is not listed in war_suite.rs EMBEDDED" >&2; exit 1; }
done

for f in scenarios/war/scripts/*.json; do
  grep -q "scenarios/war/scripts/$(basename "$f")" crates/sim/examples/war_script/mod.rs \
    || { echo "$f is not listed in war_script/mod.rs SCRIPTS" >&2; exit 1; }
done

if [ "$ledger" = 1 ]; then
  out_dir="docs/sim-results/hard-vs-human/war-scenarios"
else
  out_dir="${TMPDIR:-/tmp}/legatum-war-scenarios"
fi
mkdir -p "$out_dir/runs"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
tag="${brain/+/-}${params:+-$(basename "$params" .json)-$seats}"
report="$out_dir/runs/$stamp-$tag.json"
sim_args=(--brain "$brain")
[ -z "$params" ] || sim_args+=(--params-seats "$seats")
[ -z "$ids" ] || sim_args+=(--ids "$ids")

if [ "$mode" = local ]; then
  cargo build --release -p legatum-sim --example war_suite
  local_params=()
  [ -z "$params" ] || local_params=(--ai-params "$params")
  [ -z "$opponents" ] || local_params+=(--opponent-params "$opponents")
  "${CARGO_TARGET_DIR:-target}/release/examples/war_suite" "${sim_args[@]}" ${local_params[@]+"${local_params[@]}"} \
    --runs "$runs" --threads 6 --report "$report"
else
  # WAR_SKIP_BUILD=1 reuses target/cloud/linux-arm64/war_suite from the previous run.
  [ "${WAR_SKIP_BUILD:-0}" = 1 ] || LEGATUM_CLOUD_EXAMPLES=war_suite tools/cloud/build-linux.sh
  batch_log="$(mktemp)"; trap 'rm -f "$batch_log"' EXIT
  batch_params=()
  [ -z "$params" ] || batch_params=(--ai-params "$params")
  [ -z "$opponents" ] || batch_params+=(--opponent-params "$opponents")
  tools/cloud/run-batch.sh --count 1 --max-hours "${WAR_MAX_HOURS:-2}" --bin war_suite ${batch_params[@]+"${batch_params[@]}"} -- \
    "${sim_args[@]}" --games "$runs" --seed 0 --label "war-$tag" | tee "$batch_log"
  result_root="$(sed -n 's/^results: \([^ ]*\).*/\1/p' "$batch_log" | tail -1)"
  [ -n "$result_root" ] || { echo "cloud batch result directory not found" >&2; exit 1; }
  gzip -dc "$result_root/slice-0/out/war-suite.json.gz" > "$report"
fi

report_args=("$report" --label "${label:-$tag}")
if [ "$ledger" = 1 ]; then
  report_args+=(--ledger "$out_dir/ledger.jsonl" --table "$out_dir/latest-$tag.md" --chart "$out_dir/chart-$tag.png")
fi
# The chart needs Pillow, which macOS's /usr/bin/python3 lacks.
py=python3; python3 -c 'import PIL' 2>/dev/null || { [ ! -x /opt/homebrew/bin/python3 ] || py=/opt/homebrew/bin/python3; }
"$py" tools/sim/war-scenarios-report.py "${report_args[@]}"
if [ "$ledger" = 1 ]; then
  python3 tools/sim/freshness.py stamp "$report" "$out_dir/latest-$tag.md" "$out_dir/chart-$tag.png"
fi
echo "report: $report"
if [ "$check" = 1 ]; then
  "$py" - "$report" "$ATTACK_ID" "$ATTACK_FLOOR" <<'PY'
import json, sys
report, scenario, floor = sys.argv[1], sys.argv[2], int(sys.argv[3])
s = next(x for x in json.load(open(report))["scenarios"] if x["id"] == scenario)
pct, runs = s["success_percent"], s["runs"]
print(f"attack regression check: {scenario} {pct}% of {runs} runs (floor {floor}%)")
sys.exit(0 if pct >= floor else 1)
PY
fi

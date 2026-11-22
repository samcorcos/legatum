#!/usr/bin/env bash
# Scenario gate: score a candidate AI configuration against a baseline on every Lab card and war
# scenario, paired by dice seed, with a pooled delta, a 95% interval and a noninferiority verdict.
# Docs: docs/sim-results/scenario-gate/README.md. Local only (this Mac's cores).
#
#   tools/sim/scenario-gate.sh CANDIDATE.json [--base builtin|FILE] [--runs 480] [--set held_out]
#                              [--mode seat|rivals] [--him hard|designer] [--margin 0.05] [--won-margin 1.5]
#                              [--threads 16] [--out DIR] [--ids a,b]
#   tools/sim/scenario-gate.sh --aa [--runs 480] ...   # A/A: Hard against itself on other dice
#
# --set: train (tune here), held_out (accept here) or all. The split is fixed in
#   docs/sim-results/scenario-gate/split.json; never tune on held_out.
# --mode seat: the candidate plays the scenario seat (is it stronger in the position);
#   rivals: it plays the seat's AI rivals (does the seat do worse against it).
# --him designer: the designer's own empire (player 0), where it is a rival of the seat, is played
#   by the imitation seat (legatum_ai::designer) instead of Hard: the opponent column.
# Exit status 0 on PASS, 1 on FAIL. Writes DIR/runs.jsonl, report.md and report.json.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
cand=""; base=builtin; runs=480; set=held_out; mode=seat; him=hard; margin=0.05; won_margin=1.5
threads=16; out=""; ids=""; aa=0
while [ $# -gt 0 ]; do
  case "$1" in
    --base) base="${2:?}"; shift 2;;
    --runs) runs="${2:?}"; shift 2;;
    --set) set="${2:?}"; shift 2;;
    --mode) mode="${2:?}"; shift 2;;
    --him) him="${2:?}"; shift 2;;
    --margin) margin="${2:?}"; shift 2;;
    --won-margin) won_margin="${2:?}"; shift 2;;
    --threads) threads="${2:?}"; shift 2;;
    --out) out="${2:?}"; shift 2;;
    --ids) ids="${2:?}"; shift 2;;
    --aa) aa=1; shift;;
    -h|--help) sed -n 2,20p "$0"; exit 0;;
    -*) echo "unknown option: $1" >&2; exit 2;;
    *) cand="$1"; shift;;
  esac
done
if [ "$aa" = 1 ]; then
  cand_arm="cand=$base@100000"; label=aa
else
  [ -n "$cand" ] || { echo "give a CANDIDATE.json (or --aa)" >&2; exit 2; }
  cand_arm="cand=$cand"; label="$(basename "$cand" .json)"
fi
out="${out:-${TMPDIR:-/tmp}/legatum-scenario-gate/$(date -u +%Y%m%dT%H%M%SZ)-$label-$mode-$him-$set}"
mkdir -p "$out"
tools/sim/build-slot.sh -- nice -n 10 cargo build --release -q -p legatum-sim --example scenario_gate
bin="${CARGO_TARGET_DIR:-$root/target}/release/examples/scenario_gate"
args=(--arm "base=$base" --arm "$cand_arm" --runs "$runs" --threads "$threads"
  --params-seats "$mode" --him "$him" --split docs/sim-results/scenario-gate/split.json --set "$set"
  --out "$out/runs.jsonl")
[ -z "$ids" ] || args+=(--ids "$ids")
nice -n 10 "$bin" "${args[@]}"
python3 tools/sim/scenario_gate_report.py "$out/runs.jsonl" --base base --cand cand \
  --margin "$margin" --won-margin "$won_margin" --md "$out/report.md" --json "$out/report.json"

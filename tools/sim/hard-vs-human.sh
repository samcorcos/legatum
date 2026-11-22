#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
mode=auto; brain=hard; salts=64; params=""; label=""; seat_mode=""; rival=""; type="c8g.16xlarge"
while [ $# -gt 0 ]; do
  case "$1" in
    --local) mode=local; shift;;
    --cloud) mode=cloud; shift;;
    --brain) brain="${2:?--brain needs a value}"; shift 2;;
    --salts) salts="${2:?--salts needs a value}"; shift 2;;
    --type) type="${2:?--type needs a value}"; shift 2;;
    --ai-params) params="${2:?--ai-params needs a file}"; shift 2;;
    --seat-brain|--opponent-brain|--all-seats)
      [ -z "$seat_mode" ] || { echo "seat modes are mutually exclusive" >&2; exit 2; }
      seat_mode="$1"; shift;;
    --label) label="${2:?--label needs a value}"; shift 2;;
    --rival-search) rival="--rival-search"; shift;;
    -h|--help) echo "usage: tools/sim/hard-vs-human.sh [--local|--cloud] (default: --cloud when AWS credentials work, else --local; the cloud path builds on the build box and runs on one spot server, so the Mac stays idle) [--brain hard|hard+search|normal|easy] [--salts N] [--type c8g.4xlarge] [--ai-params FILE] [--seat-brain|--opponent-brain|--all-seats] [--rival-search (rivals play Hard+Search where the map says so; default is plain Hard like the app)] [--label TEXT]"; exit 0;;
    *) echo "unknown option: $1" >&2; exit 2;;
  esac
done
if [ "$mode" = auto ]; then
  mode=local
  if ( . tools/cloud/lib.sh && awsx sts get-caller-identity >/dev/null 2>&1 ); then mode=cloud; fi
  echo "hard-vs-human: --$mode (default: cloud whenever AWS credentials work; pass --local to run on this machine)" >&2
fi
case "$salts" in ''|*[!0-9]*|0) echo "--salts must be a positive integer" >&2; exit 2;; esac
[ -z "$seat_mode" ] || [ -n "$params" ] || { echo "seat mode requires --ai-params" >&2; exit 2; }
report="$(mktemp)"
batch_log="$(mktemp)"
target_dir="${CARGO_TARGET_DIR:-target}"
timing_label="brain=$brain;salts=$salts"
[ -z "$params" ] || timing_label="$timing_label;params=1"
[ -z "$label" ] || timing_label="$timing_label;run=$label"
cleanup() { rm -f "$report" "$batch_log"; }
trap cleanup EXIT
if [ "$mode" = local ]; then
  python3 tools/ops/timed.py build hard-vs-human-build --label "$timing_label" -- \
    env CARGO_TARGET_DIR="$target_dir" CARGO_BUILD_JOBS=4 tools/sim/build-slot.sh -- cargo build --release -p legatum-sim --example hard_vs_human
  if [ -n "$params" ]; then
    param_brain=params
    [ "$brain" != hard+search ] || param_brain=params+search
    sim_args=(--brain "$param_brain" --salts "$salts" --ai-params "$params")
    if [ -n "$seat_mode" ]; then sim_args+=("$seat_mode"); fi
    if [ -n "$rival" ]; then sim_args+=("$rival"); fi
    python3 tools/ops/timed.py sim hard-vs-human --label "$timing_label" -- \
      env RAYON_NUM_THREADS=4 "$target_dir/release/examples/hard_vs_human" "${sim_args[@]}" > "$report"
  else
    sim_args=(--brain "$brain" --salts "$salts")
    if [ -n "$rival" ]; then sim_args+=("$rival"); fi
    python3 tools/ops/timed.py sim hard-vs-human --label "$timing_label" -- \
      env RAYON_NUM_THREADS=4 "$target_dir/release/examples/hard_vs_human" "${sim_args[@]}" > "$report"
  fi
else
  # build-linux.sh builds on the cloud build box (no local CPU); only its local Docker fallback needs a build slot.
  if [ "${LEGATUM_CLOUD_LOCAL_BUILD:-0}" = 1 ]; then
    LEGATUM_CLOUD_EXAMPLES=hard_vs_human CARGO_BUILD_JOBS=4 tools/sim/build-slot.sh -- tools/cloud/build-linux.sh
  else
    LEGATUM_CLOUD_EXAMPLES=hard_vs_human CARGO_BUILD_JOBS=4 tools/cloud/build-linux.sh
  fi
  batch_args=(--type "$type" --count 1 --max-hours "${HVH_MAX_HOURS:-1}" --bin hard_vs_human)
  if [ -n "$params" ]; then batch_args+=(--ai-params "$params"); fi
  if [ "${LEGATUM_HVH_RESEED_DICE:-0}" = 1 ]; then batch_args+=(--env LEGATUM_HVH_RESEED_DICE=1); fi
  sim_args=(--brain "$brain")
  if [ -n "$params" ]; then
    param_brain=params
    [ "$brain" != hard+search ] || param_brain=params+search
    sim_args=(--brain "$param_brain")
  fi
  if [ -n "$seat_mode" ]; then sim_args+=("$seat_mode"); fi
  if [ -n "$rival" ]; then sim_args+=("$rival"); fi
  tools/cloud/run-batch.sh "${batch_args[@]}" -- "${sim_args[@]}" --games "$salts" --seed 1 --label hard-vs-human | tee "$batch_log"
  result_root="$(sed -n 's/^results: \([^ ]*\).*/\1/p' "$batch_log" | tail -1)"
  [ -n "$result_root" ] || { echo "cloud batch result directory not found" >&2; exit 1; }
  if [ -f "$result_root/slice-0/out/hard-vs-human.json" ]; then
    cp "$result_root/slice-0/out/hard-vs-human.json" "$report"
  else
    gzip -dc "$result_root/slice-0/out/hard-vs-human.json.gz" > "$report"
  fi
fi
python3 tools/sim/hard-vs-human-report.py "$report" \
  --ledger docs/sim-results/hard-vs-human/ledger.jsonl \
  --chart docs/sim-results/hard-vs-human/chart.png --label "$label"
python3 tools/sim/freshness.py stamp docs/sim-results/hard-vs-human/chart.png

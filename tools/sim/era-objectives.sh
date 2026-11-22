#!/usr/bin/env bash
# Era-objective measures: one all-AI Hard cohort per arm on one binary, then the comparison report.
#
#   tools/sim/era-objectives.sh OUTDIR GAMES SEED PLAYERS [ARM...]
#     ARM  default | seek0 | name=params.json   (default: default and seek0; the first is the control)
#
# seek0 is the `objective_seek` gene at 0 (docs/sim-results/era-objectives/params/seek-0.json): no AI
# pursues an era objective, so what it meets is met by accident. Run it on the box:
#   tools/cloud/remote.sh <worktree> --with-uncommitted --artifact OUTDIR -- tools/sim/era-objectives.sh OUTDIR 256 9100 8
set -euo pipefail
cd "$(dirname "$0")/../.."
out="$1"; games="$2"; seed="$3"; players="$4"; shift 4
[ $# -gt 0 ] || set -- default seek0
cargo build --release -p legatum-sim --bin legatum-sim 2>&1 | tail -2
mkdir -p "$out"
reports=()
for arm in "$@"; do
  name="${arm%%=*}"; params=""
  case "$arm" in
    default) ;;
    seek0) params="docs/sim-results/era-objectives/params/seek-0.json";;
    *=*) params="${arm#*=}";;
    *) echo "unknown arm $arm" >&2; exit 2;;
  esac
  extra=(); [ -z "$params" ] || extra=(--ai-params "$params")
  echo "=== arm $name: $games games, $players players, seed $seed"
  start=$(date +%s)
  target/release/legatum-sim --games "$games" --seed "$seed" --players "$players" --difficulty hard \
    --safety-cap 600 --out-dir "$out/$name" --label "era-objectives-$name" ${extra[@]+"${extra[@]}"} 2>&1 | tail -3
  echo "sim seconds: $(( $(date +%s) - start ))"
  reports+=("$name=$(ls "$out/$name"/*.json | head -1)")
done
python3 tools/sim/era-objectives-report.py "${reports[@]}" | tee "$out/report.md"
# The raw reports are large: keep only the comparison, or with KEEP_RAW=1 a gzipped copy of each.
for arm in "$@"; do
  name="${arm%%=*}"
  if [ "${KEEP_RAW:-0}" = 1 ]; then gzip -c "$out/$name"/*.json > "$out/raw-$name.json.gz"; fi
  rm -f "$out/$name"/*.json
done

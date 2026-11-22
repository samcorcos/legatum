#!/usr/bin/env bash
# Overreach v3 sweep: every cell at 4p and 8p on cloud servers, N cells at a time.
#   tools/sim/overreach-v3-sweep.sh <parallel> <games> <cell>=<flags> [<cell>=<flags> ...]
# e.g. tools/sim/overreach-v3-sweep.sh 2 256 off=decadence-slope=0 slope25=decadence-slope=25 default=
# A cell's flags are comma-separated rules experiments (see crates/core/src/experiments.rs). Results land in
# ~/LegatumRuns/cloud/vp2-<cell>-<4p|8p>-<date>-<time>; summarise with tools/sim/overreach_v3_summary.py.
# Same seeds in every cell (4p 2026101001+, 8p 2026102001+) so the maps are paired. Run under bgjob.sh (cloud-* name).
set -uo pipefail
cd "$(dirname "$0")/../.."
par="$1"; games="$2"; shift 2
run_cell() {
  local spec="$1" games="$2" cell flags
  cell="${spec%%=*}"; flags="${spec#*=}"
  [ "$flags" != "$spec" ] || flags=""
  for lobby in 4p 8p; do
    seed=2026101001; [ "$lobby" = 8p ] && seed=2026102001
    tools/sim/vp-v2-batch.sh "$cell-$lobby" "${lobby%p}" "$games" "$seed" "$flags" || echo "FAILED $cell-$lobby"
  done
}
export -f run_cell
printf '%s\n' "$@" | xargs -P "$par" -I{} bash -c 'run_cell "$1" "$2"' _ {} "$games"
echo "sweep done"

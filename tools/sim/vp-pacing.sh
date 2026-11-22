#!/usr/bin/env bash
# VP pacing arms: one cohort per arm on one binary (all-AI, Standard), the sim report plus the air usage
# of the same games (tapes replayed by crates/sim/examples/air_usage.rs).
#
#   tools/sim/vp-pacing.sh OUTDIR GAMES SEED PLAYERS BRAIN ARM... 
#     BRAIN  hard | search   (search = Hard + Search k=3,h=20,d=4 in every seat)
#     ARM    name or name=flag,flag  (rules experiments, comma separated), e.g. base  t12=vp-target=12
#
# Run it on the box: tools/cloud/remote.sh <worktree> --with-uncommitted --artifact OUTDIR -- tools/sim/vp-pacing.sh ...
set -euo pipefail
cd "$(dirname "$0")/../.."
out="$1"; games="$2"; seed="$3"; players="$4"; brain="$5"; shift 5
cargo build --release -p legatum-sim --bin legatum-sim --example air_usage 2>&1 | tail -2
mkdir -p "$out"
search=()
[ "$brain" = search ] && search=(--search k=3,h=20,d=4)
for arm in "$@"; do
  name="${arm%%=*}"; flags=""
  case "$arm" in *=*) flags="${arm#*=}";; esac
  extra=()
  [ -z "$flags" ] || extra=(--rules-experiment "$flags")
  echo "=== arm $name ($flags): $games games, $players players, seed $seed, $brain"
  rm -rf "target/vp-pacing-tapes-$name"
  start=$(date +%s)
  target/release/legatum-sim --games "$games" --seed "$seed" --players "$players" --difficulty hard \
    ${search[@]+"${search[@]}"} --safety-cap 600 --tape-out "target/vp-pacing-tapes-$name" \
    --out-dir "$out/$name" --label "vp-pacing-$name" ${extra[@]+"${extra[@]}"} 2>&1 | tail -3
  echo "sim seconds: $(( $(date +%s) - start ))"
  target/release/examples/air_usage "target/vp-pacing-tapes-$name" | tee "$out/$name/air-usage.txt"
  rm -rf "target/vp-pacing-tapes-$name"
done

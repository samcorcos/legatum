#!/usr/bin/env bash
# Human-seat benchmark for a VP rule change, all seats, control and candidate on ONE binary
# (hard_vs_human --rules-experiment applies the candidate rule to every map's setup).
#
#   tools/sim/vp-pacing-hvh.sh OUTDIR SALTS BRAIN CANDIDATE_FLAGS [CONTROL_FLAGS]
#     BRAIN  hard | hard+search
# Run it on the box: tools/cloud/remote.sh <worktree> --artifact OUTDIR -- tools/sim/vp-pacing-hvh.sh ...
set -euo pipefail
cd "$(dirname "$0")/../.."
out="$1"; salts="$2"; brain="$3"; cand="$4"; ctl="${5:-}"
cargo build --release -p legatum-sim --example hard_vs_human 2>&1 | tail -2
mkdir -p "$out"
tag="${brain/+/-}"
for arm in control candidate; do
  flags="$ctl"; [ "$arm" = candidate ] && flags="$cand"
  extra=(); [ -z "$flags" ] || extra=(--rules-experiment "$flags")
  echo "=== $arm ($flags) $brain, $salts salts"
  target/release/examples/hard_vs_human --brain "$brain" --all-seats --salts "$salts" ${extra[@]+"${extra[@]}"} > "$out/hvh-$tag-$arm.json"
done

#!/usr/bin/env bash
# One VP v2 cohort on a cloud server: Hard + Search in every seat, Standard speed.
#   tools/sim/vp-v2-batch.sh <name> <players> <games> <seed> [rules-experiment flags, comma separated]
# e.g. tools/sim/vp-v2-batch.sh slope25-4p 4 256 2026101001 decadence-slope=25
#      tools/sim/vp-v2-batch.sh v2-4p 4 256 2026101001
#      tools/sim/vp-v2-batch.sh objective125-4p 4 128 2026101501 objective-scale=125
# Same seeds across cohorts of one lobby give paired maps. Run under bgjob.sh (cloud-* name).
set -euo pipefail
cd "$(dirname "$0")/../.."
name="$1"; players="$2"; games="$3"; seed="$4"; flags="${5:-}"
args=(--games "$games" --seed "$seed" --players "$players" --difficulty hard --search k=3,h=20,d=4
  --safety-cap 600 --label "vp2-$name")
[ -z "$flags" ] || args+=(--rules-experiment "$flags")
exec tools/cloud/run-batch.sh --max-hours 1 --count 1 ${VP2_BATCH_OPTS:-} -- "${args[@]}"

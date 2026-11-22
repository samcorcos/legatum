#!/usr/bin/env bash
# Exercise only the batch wrapper's dry-run parser and generated user-data.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
bin_dir="$root/target/cloud/linux-arm64"
bin="$bin_dir/seat-run-input-test-$$"
scratch="$(mktemp -d)"
trap 'rm -f "$bin"; rm -rf "$scratch"' EXIT
mkdir -p "$bin_dir"
printf '#!/bin/sh\nexit 0\n' > "$bin"
chmod +x "$bin"
printf '{}\n' > "$scratch/tape.json"
printf '{}\n' > "$scratch/states.jsonl"
printf '{}\n' > "$scratch/params.json"

"$root/tools/cloud/run-batch.sh" --dry-run --bin "${bin##*/}" \
  --input-file "tape=$scratch/tape.json" \
  --opponent-params "$scratch/params.json" -- \
  --tape /tmp/input-tape --games 2 --label purchase-seat \
  > "$scratch/purchase.out"
user_data="$root/target/cloud/dry-run/user-data-0.sh"
rg -Fq 'aws s3 cp --no-progress "s3://$BUCKET/inputs/' "$user_data"
rg -Fq '/tmp/input-tape' "$user_data"
rg -Fq -- '--opponent-params /tmp/opp-params-' "$user_data"
rg -Fq -- '--games 2 --seed 1 --label purchase-seat-s0' "$user_data"

"$root/tools/cloud/run-batch.sh" --dry-run --bin "${bin##*/}" \
  --input-file "tape=$scratch/tape.json" --input-file "states=$scratch/states.jsonl" \
  --input-file "rivals=$scratch/params.json" -- \
  --tape /tmp/input-tape --states /tmp/input-states \
  --continue rivals:/tmp/input-rivals --games 2 --label neighbour-seat \
  > "$scratch/neighbour.out"
rg -Fq -- '--continue rivals:/tmp/input-rivals' "$user_data"
rg -Fq '/tmp/input-rivals' "$user_data"
rg -Fq '/tmp/input-states' "$user_data"

if "$root/tools/cloud/run-batch.sh" --dry-run --bin "${bin##*/}" \
  --input-file "tape=$scratch/tape.json" --input-file "tape=$scratch/states.jsonl" \
  -- --games 2 > "$scratch/duplicate.out" 2>&1; then
  echo 'duplicate input name unexpectedly accepted' >&2
  exit 1
fi
rg -Fq 'duplicate --input-file name tape' "$scratch/duplicate.out"
echo 'run-batch input-file dry-run parsing passed'

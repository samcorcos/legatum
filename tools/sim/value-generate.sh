#!/usr/bin/env bash
# Generate a value-network training dataset on AWS spot in one command (AI war plan step 3), surviving
# spot reclaims: games a reclaimed server lost are re-run on their own seeds (on c7g after a c8g reclaim).
#
#   tools/sim/value-generate.sh LABEL [--games 50000] [--seed 8810001] [--servers 4] [--max-hours 0.75]
#       [--budget 4] [--players 8,8,8,4] [--every 10] [--rounds 3] [--no-build] [--dry-run]
#
# 1. Builds value_data for Linux on the build box (skip with --no-build when target/cloud/linux-arm64 is current).
# 2. Runs tools/cloud/run-batch.sh --bin value_data over --servers spot servers, then re-runs every seed range a
#    server lost, up to --rounds batches in all.
# 3. Never launches a batch whose worst case (servers x max-hours x spot price, run-batch's own figure) is above
#    what is left of --budget dollars; a finished or lost batch is charged its wall time at the spot price.
# 4. Appends every batch directory with data to ~/LegatumRuns/value-data/LABEL.dirs (a rerun of missing
#    seeds with the same LABEL adds to it); train with
#      tools/sim/train-value.py --data-list ~/LegatumRuns/value-data/LABEL.dirs ...
# Rerun the same LABEL after a rule change with a new LABEL (and a new --seed block); old lists are untouched.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
label="${1:?usage: value-generate.sh LABEL [options]}"; shift
games=50000; seed=8810001; servers=4; max_hours=0.75; budget=4; players=8,8,8,4; every=10; rounds=3; build=1; dry=0
while [ $# -gt 0 ]; do
  case "$1" in
    --games) games="$2"; shift 2;; --seed) seed="$2"; shift 2;; --servers) servers="$2"; shift 2;;
    --max-hours) max_hours="$2"; shift 2;; --budget) budget="$2"; shift 2;; --players) players="$2"; shift 2;;
    --every) every="$2"; shift 2;; --rounds) rounds="$2"; shift 2;; --no-build) build=0; shift;;
    --dry-run) dry=1; shift;;
    *) echo "value-generate: unknown option $1" >&2; exit 2;;
  esac
done
align=$(( $(echo "$players" | tr ',' '\n' | wc -l) ))
out_dir="$HOME/LegatumRuns/value-data"; mkdir -p "$out_dir"
list="$out_dir/$label.dirs"; touch "$list"
log="$out_dir/$label.log"

if [ "$build" = 1 ] && [ "$dry" = 0 ]; then
  LEGATUM_CLOUD_EXAMPLES=value_data tools/cloud/build-linux.sh 2>&1 | tail -3
fi

# Pending seed ranges, "games:seed" each.
pending=("$games:$seed")
spent=0
type=c8g.16xlarge
for ((round = 1; round <= rounds && ${#pending[@]} > 0; round++)); do
  next=()
  for job in "${pending[@]}"; do
    g="${job%%:*}"; s="${job##*:}"
    n="$servers"; [ "$round" -gt 1 ] && n=1
    while [ "$n" -gt 1 ] && [ "$g" -lt $(( n * align )) ]; do n=$(( n - 1 )); done
    left="$(awk -v b="$budget" -v s="$spent" 'BEGIN{printf "%.2f", b-s}')"
    echo "== round $round: $g games from seed $s on $n x $type (budget left \$$left)" | tee -a "$log"
    args=(--bin value_data --type "$type" --count "$n" --max-hours "$max_hours" --align "$align")
    [ "$dry" = 0 ] || args+=(--dry-run)
    started=$(date +%s)
    set +e
    LEGATUM_CLOUD_CONFIRM_USD="$left" tools/cloud/run-batch.sh "${args[@]}" -- --games "$g" --seed "$s" \
      --players "$players" --every "$every" --label "$label" 2>&1 | tee -a "$log" > "$out_dir/$label.last"
    rc=${PIPESTATUS[0]}
    set -e
    wall=$(( $(date +%s) - started ))
    cat "$out_dir/$label.last"
    [ "$dry" = 0 ] || continue
    price="$(sed -n 's/.*about \$\([0-9.]*\)\/h each.*/\1/p' "$out_dir/$label.last" | head -1)"
    if grep -q "is above \$" "$out_dir/$label.last"; then
      echo "value-generate: stopped, the next batch's worst case is above the \$$left left of the budget" | tee -a "$log"
      next+=("$job"); continue
    fi
    spent="$(awk -v s="$spent" -v n="$n" -v w="$wall" -v p="${price:-0.9}" 'BEGIN{printf "%.2f", s + n*w/3600*p}')"
    dir="$(sed -n 's/^results: \([^ ]*\) .*/\1/p' "$out_dir/$label.last" | tail -1)"
    [ -n "$dir" ] && ls "$dir"/slice-*/out/value-schema.json* >/dev/null 2>&1 && echo "$dir" >> "$list"
    # Seed ranges a reclaimed server lost.
    while read -r lg ls; do next+=("$lg:$ls"); done < <(sed -n 's/.*ended without finishing.*: games \([0-9]*\) from seed \([0-9]*\) are missing.*/\1 \2/p' "$out_dir/$label.last")
    if grep -q "exited [1-9]" "$out_dir/$label.last"; then
      echo "value-generate: value_data itself failed (not a reclaim); see the slice batch.log; stopping" | tee -a "$log"; exit 1
    fi
    [ "$rc" = 0 ] || grep -q "ended without finishing" "$out_dir/$label.last" || { echo "value-generate: run-batch failed (rc $rc)" | tee -a "$log"; exit 1; }
  done
  pending=(${next[@]+"${next[@]}"})
  [ ${#pending[@]} -eq 0 ] || type=c7g.16xlarge
done
echo "value-generate: about \$$spent spent (wall time x spot price; the AWS bill is authoritative)" | tee -a "$log"
echo "value-generate: data in $(wc -l < "$list" | tr -d ' ') batch dir(s), listed in $list" | tee -a "$log"
if [ ${#pending[@]} -gt 0 ]; then
  echo "value-generate: still missing: ${pending[*]} (games:seed); rerun with --games G --seed S --no-build" | tee -a "$log"; exit 1
fi

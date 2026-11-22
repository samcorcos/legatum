#!/usr/bin/env bash
# List the running legatum-sims servers: age, expiry and batch. Read-only.
#   tools/cloud/status.sh
# Exit status 0 with "no servers running" is the state you want after every batch.
set -euo pipefail
. "$(dirname "$0")/lib.sh"
require_profile

# The persistent build box (infra/cloud-box) is tagged project=legatum-box, so the sims checks below do not see it.
box="$(awsx ec2 describe-instances --filters "Name=tag:project,Values=legatum-box" "Name=tag:role,Values=build-box" "Name=instance-state-name,Values=pending,running,stopping" \
  --query 'Reservations[].Instances[].[InstanceId,InstanceType,State.Name,LaunchTime]' --output text 2>/dev/null || true)"
[ -z "$box" ] || echo "build box RUNNING (stops itself 20 idle minutes after the last job; tools/cloud/box.sh stop): $box"

now="$(date -u +%s)"
rows="$(running_servers)"
if [ -z "$rows" ]; then
  echo "no legatum-sims servers running ($REGION)"
  exit 0
fi
printf '%-20s %-14s %-9s %-9s %-22s %-14s %s\n' ID TYPE STATE AGE EXPIRES BATCH OWNER
echo "$rows" | while IFS=$'\t' read -r id type launch expiry batch owner state; do
  launch_epoch="$(date -u -j -f '%Y-%m-%dT%H:%M:%S' "${launch%%.*}" +%s 2>/dev/null || date -u -d "$launch" +%s)"
  age=$(( now - launch_epoch ))
  left="?"
  if exp_epoch="$(date -u -j -f '%Y-%m-%dT%H:%M:%SZ' "$expiry" +%s 2>/dev/null)"; then
    left=$(( (exp_epoch - now) / 60 ))m
  fi
  printf '%-20s %-14s %-9s %-9s %-22s %-14s %s\n' "$id" "$type" "$state" "$((age / 60))m" "$expiry (in $left)" "${batch:-?}" "${owner:-?}"
done
echo "$(echo "$rows" | wc -l | tr -d ' ') server(s) running. Stop everything: tools/cloud/kill-all.sh"

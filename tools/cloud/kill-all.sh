#!/usr/bin/env bash
# Terminate every legatum-sims server (or one batch's). Safe to run any time: results already in S3 stay.
#   tools/cloud/kill-all.sh            # all servers tagged project=legatum-sims
#   tools/cloud/kill-all.sh <batch-id> # only that batch
set -euo pipefail
. "$(dirname "$0")/lib.sh"
require_profile

filters=("Name=tag:project,Values=$TAG_PROJECT" "Name=instance-state-name,Values=pending,running,stopping,stopped")
[ -z "${1:-}" ] || filters+=("Name=tag:batch,Values=$1")
ids="$(awsx ec2 describe-instances --filters "${filters[@]}" --query 'Reservations[].Instances[].InstanceId' --output text)"
if [ -z "$ids" ]; then echo "no legatum-sims servers running ($REGION)"; exit 0; fi
echo "terminating: $ids"
# shellcheck disable=SC2086  # ids is a whitespace-separated list
awsx ec2 terminate-instances --instance-ids $ids --query 'TerminatingInstances[].[InstanceId,CurrentState.Name]' --output text

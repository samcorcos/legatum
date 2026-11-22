#!/usr/bin/env bash
# The Legatum cloud build box: a stopped-when-idle Graviton machine for heavy compile, test, WebKit and
# sim-build jobs (infra/cloud-box). You pay for EBS while it is stopped, and for compute only while it is up.
#
#   tools/cloud/box.sh init       create the dedicated ssh key (once) and print the line for terraform.tfvars
#   tools/cloud/box.sh start      start it and wait until ssh over Session Manager answers
#   tools/cloud/box.sh stop       stop it now (it also stops itself after 20 idle minutes)
#   tools/cloud/box.sh status     state, today's running minutes against the cap, jobs, disk, estimated cost
#   tools/cloud/box.sh shell [cmd...]   interactive shell (or run one command) on the box
#   tools/cloud/box.sh prepare    ship the box scripts and run the idempotent toolchain bootstrap (first use, or after
#                                 toolchain changes); takes about 10 minutes the first time
#   tools/cloud/box.sh warm [ref] build master (default) in the golden slot so new job slots start from a warm cache
#   tools/cloud/box.sh jobs       running and recent jobs
#   tools/cloud/box.sh gc         remove old job directories and refs; free disk by dropping the coldest slot caches
#   tools/cloud/box.sh sweep      trim the idle slots now (incremental dirs, over-budget debug targets); never touches a busy slot
#   tools/cloud/box.sh util [h]   mean CPU utilisation per hour (default 24 h), and how much of it was idle-filler work
# Idle-filler batches (low-priority background CPU work): tools/cloud/filler.sh.
#
# Normally nothing here is run by hand: remote.sh and verify-remote.sh start the box on demand.
set -euo pipefail
. "$(dirname "$0")/box-lib.sh"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

cmd="${1:-status}"; [ $# -eq 0 ] || shift
case "$cmd" in
  init)
    mkdir -p "$BOX_HOME"; chmod 700 "$BOX_HOME"
    [ -f "$BOX_KEY" ] || ssh-keygen -q -t ed25519 -N "" -C "legatum-box" -f "$BOX_KEY"
    echo "ssh_public_key = \"$(cat "$BOX_KEY.pub")\""
    ;;
  start)
    box_up
    echo "box up: $(box_id) ($(box_ssh 'nproc' 2>/dev/null) vCPUs)"
    ;;
  stop)
    require_profile
    id="$(box_id)"
    box_ssh_base "$id"; "${BOX_SSH[@]}" -O exit 2>/dev/null || true
    awsx ec2 stop-instances --instance-ids "$id" --query 'StoppingInstances[].[InstanceId,CurrentState.Name]' --output text
    ;;
  status)
    require_profile
    id="$(box_id)"
    row="$(awsx ec2 describe-instances --instance-ids "$id" --query 'Reservations[0].Instances[0].[State.Name,InstanceType,InstanceLifecycle,LaunchTime,Tags[?Key==`cap-used-min`]|[0].Value,Tags[?Key==`cap-day`]|[0].Value]' --output text)"
    state="$(echo "$row" | cut -f1)"; type="$(echo "$row" | cut -f2)"; life="$(echo "$row" | cut -f3)"; launch="$(echo "$row" | cut -f4)"
    used="$(echo "$row" | cut -f5)"; day="$(echo "$row" | cut -f6)"
    [ "$day" = "$(date -u +%F)" ] || used=0
    echo "box $id: $state ($type, ${life:-on-demand}); launched $launch"
    echo "runtime today (UTC, per the AWS-side counter, updated every 5 min): ${used:-0} minutes"
    if [ "$state" = running ]; then
      box_ssh 'echo "load $(cut -d" " -f1-3 /proc/loadavg), $(nproc) vCPUs, memory free $(free -g | awk "/Mem:/{print \$7}") GB"; df -h /data | tail -1; echo "idle-watch: $(cat /run/legatum-idle/busy 2>/dev/null || echo "idle since $(date -d @$(cat /run/legatum-idle/since 2>/dev/null || date +%s) +%T)")"; /data/bin/box-job list 2>/dev/null || true; n=0; N=$(. /data/bin/config; echo ${SLOTS:-12}); for k in $(seq 1 $N); do ( flock -n 8 ) 8>/data/slots/s$k.lock || n=$((n+1)); done; echo "slots busy: $n of $N; queued for a slot: $(( $(cat /data/slots/next-ticket 2>/dev/null || echo 1) - $(cat /data/slots/serving 2>/dev/null || echo 1) ))"; echo "CPU utilisation, last 6 h (mean per hour):"; /data/bin/box-job util 6 2>/dev/null | sed "s/^/  /"; /data/bin/box-filler status 2>/dev/null | sed "s/^/filler /"' || echo "(ssh not answering yet)"
    fi
    ;;
  shell)
    box_up
    box_ssh_base "$(box_id)"
    if [ $# -eq 0 ]; then exec "${BOX_SSH[@]}" -t; else exec "${BOX_SSH[@]}" "$@"; fi
    ;;
  sync)
    box_up; box_sync
    ;;
  prepare)
    box_up; box_sync
    # Dependency manifests so bootstrap can install Playwright's system libraries and browsers.
    ( cd "$root" && COPYFILE_DISABLE=1 tar czf - app/package.json app/package-lock.json ) \
      | box_ssh 'rm -rf /data/bootstrap-src && mkdir -p /data/bootstrap-src && tar --warning=no-unknown-keyword -xzf - -C /data/bootstrap-src'
    box_ssh "RUST_VERSION='${RUST_VERSION:-$(rustc --version | awk '{print $2}')}' bash /data/bin/bootstrap.sh"
    ;;
  warm)
    ref="${1:-master}"
    exec "$here/remote.sh" "$ref" --golden --npm -- bash /data/bin/warm.sh
    ;;
  jobs)
    box_up; box_sync; box_ssh '/data/bin/box-job list --all'
    ;;
  gc)
    box_up; box_sync; box_ssh '/data/bin/box-job gc --force'
    ;;
  sweep)
    box_up; box_sync; box_ssh '/data/bin/box-job sweep'
    ;;
  util)
    box_up; box_sync; box_ssh "/data/bin/box-job util ${1:-24}"
    ;;
  *) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 2;;
esac

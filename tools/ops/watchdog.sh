#!/usr/bin/env bash
# Token-free watchdog for background work. It needs no Claude: plain bash plus osascript. It only reports.
#   nohup tools/ops/watchdog.sh > .progress/jobs/watchdog.log 2>&1 &    # daemon, one pass per 5 minutes
#   tools/ops/watchdog.sh --once      # a single pass
#   tools/ops/watchdog.sh --status    # print job status, no alerts (used by snapshot.sh)
# Checks (and one self-heal: a serialised sccache server is restarted, logged to alerts.log):
#   - every job registered by tools/ops/bgjob.sh: FAILED (non-zero exit, no ok regex in the log on exit 0,
#     or the process vanished without an exit record) and STALLED (running, log unchanged for stall_min).
#   - cloud servers (tools/cloud): when LEGATUM_AWS_PROFILE is set (the sims launcher profile; unset
#     means skip), list running servers tagged project=legatum-sims with a
#     read-only describe-instances. Alerts: a server past its expiry tag (EXPIRED), a server while no
#     cloud-* job from bgjob.sh is running (UNREGISTERED), AWS unreachable (UNREACHABLE).
# Each new alert (de-duplicated by job and kind in .progress/jobs/alerted.txt; cleared when the condition
# clears) appends to .progress/alerts.log and posts a macOS notification.
# Env: LEGATUM_AWS_PROFILE, LEGATUM_CLOUD_REGION (default us-east-1), LEGATUM_PROGRESS (default <repo>/.progress), WATCHDOG_INTERVAL (seconds, default 300).
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
prog="${LEGATUM_PROGRESS:-$root/.progress}"
jobs="$prog/jobs"
interval="${WATCHDOG_INTERVAL:-300}"
mkdir -p "$jobs"
git --version >/dev/null 2>&1 || export DEVELOPER_DIR=/Library/Developer/CommandLineTools

jget() { sed -n "s/^  \"$2\": \(.*\)\$/\1/p" "$1" | head -1 | sed -e 's/,$//' -e 's/^"//' -e 's/"$//' -e 's/\\"/"/g' -e 's/\\\\/\\/g'; }
mtime() { stat -f %m "$1" 2>/dev/null || echo 0; }

alert() {   # alert <name:KIND> <message>
  grep -qxF "$1" "$jobs/alerted.txt" 2>/dev/null && return 0
  echo "$1" >> "$jobs/alerted.txt"
  printf '%s | %s\n' "$(date +%FT%T)" "$2" >> "$prog/alerts.log"
  osascript -e "display notification \"$(printf '%s' "$2" | tr -d '"\\')\" with title \"Legatum\"" >/dev/null 2>&1
  return 0
}
clear_alert() {   # clear_alert <name:KIND>
  grep -qxF "$1" "$jobs/alerted.txt" 2>/dev/null || return 0
  grep -vxF "$1" "$jobs/alerted.txt" > "$jobs/alerted.txt.tmp"; mv "$jobs/alerted.txt.tmp" "$jobs/alerted.txt"
}

# classify <job json>: sets name, status (RUNNING|STALLED|OK|FAILED) and detail
classify() {
  local jf="$1" pid log ok stall ec age now
  name="$(jget "$jf" name)"; pid="$(jget "$jf" pid)"; log="$(jget "$jf" log)"; ok="$(jget "$jf" ok)"
  stall="$(jget "$jf" stall_min)"; ec="$(jget "$jf" exit_code)"; now="$(date +%s)"
  if [ "$ec" = null ]; then
    if [ "$pid" != 0 ] && ps -p "$pid" -o command= 2>/dev/null | grep -q bgjob; then
      age=$(( (now - $(mtime "$log")) / 60 ))
      if [ "$age" -ge "${stall:-30}" ]; then status=STALLED; detail="log unchanged ${age}m (limit ${stall}m)"
      else status=RUNNING; detail="pid $pid, log changed ${age}m ago"; fi
    else status=FAILED; detail="process $pid vanished without an exit record"; fi
  elif [ "$ec" != 0 ]; then status=FAILED; detail="exit code $ec"
  elif [ -n "$ok" ] && ! grep -Eq -- "$ok" "$log" 2>/dev/null; then status=FAILED; detail="exit 0 but ok pattern '$ok' not in log"
  else status=OK; detail="exit 0"; fi
}

surfaces() {   # WindowServer-backed IOSurfaces; 46k of them crashed WindowServer on 2026-10-05
  local c; c="$(ioclasscount IOSurface 2>/dev/null | awk '{print $NF}')"
  [ -n "$c" ] || return 0
  echo "$(date +%FT%T) $c" >> "$jobs/iosurface.log"
  if [ "$c" -ge "${WATCHDOG_SURFACE_MAX:-10000}" ]; then
    alert "system:IOSURFACE" "IOSurface count $c (WindowServer aborted at 46k): stop Playwright WebKit runs and close browsers"
  else
    clear_alert "system:IOSURFACE"
  fi
}

cloud() {   # read-only: a no-op until LEGATUM_AWS_PROFILE is set
  [ -n "${LEGATUM_AWS_PROFILE:-}" ] || return 0
  command -v aws >/dev/null 2>&1 || return 0
  local rows now id expiry batch exp_epoch registered=0 jf expired=" " key
  rows="$(aws --profile "$LEGATUM_AWS_PROFILE" --region "${LEGATUM_CLOUD_REGION:-us-east-1}" ec2 describe-instances \
    --filters "Name=tag:project,Values=legatum-sims" "Name=instance-state-name,Values=pending,running,stopping" \
    --query 'Reservations[].Instances[].[InstanceId,Tags[?Key==`expiry`]|[0].Value,Tags[?Key==`batch`]|[0].Value]' \
    --output text 2>/dev/null)" || { alert "cloud:UNREACHABLE" "Cloud check cannot reach AWS (profile $LEGATUM_AWS_PROFILE): servers may be running unseen"; return 0; }
  clear_alert "cloud:UNREACHABLE"
  now="$(date -u +%s)"
  for jf in "$jobs"/cloud-*.json; do
    [ -e "$jf" ] || continue
    classify "$jf"
    case "$status" in RUNNING|STALLED) registered=1;; esac
  done
  if [ -n "$rows" ]; then
    while IFS=$'\t' read -r id expiry batch; do
      [ -n "$id" ] || continue
      exp_epoch="$(date -u -j -f '%Y-%m-%dT%H:%M:%SZ' "$expiry" +%s 2>/dev/null || echo 0)"
      if [ "$exp_epoch" -eq 0 ] || [ "$now" -gt "$exp_epoch" ]; then
        expired="$expired$id "
        alert "cloud:$id:EXPIRED" "Cloud server $id (batch ${batch:-?}) is past its expiry ${expiry:-missing}: tools/cloud/kill-all.sh"
      fi
    done <<< "$rows"
    if [ "$registered" = 0 ]; then
      alert "cloud:UNREGISTERED" "Cloud servers running with no cloud-* job in .progress/jobs (run batches under tools/ops/bgjob.sh cloud-<name>): tools/cloud/status.sh"
    else
      clear_alert "cloud:UNREGISTERED"
    fi
  else
    clear_alert "cloud:UNREGISTERED"
  fi
  # an expired server that has since disappeared stops alerting
  while read -r key; do
    case "$key" in cloud:*:EXPIRED) id="${key#cloud:}"; id="${id%:EXPIRED}"; case "$expired" in *" $id "*) ;; *) clear_alert "$key";; esac;; esac
  done < <(cat "$jobs/alerted.txt" 2>/dev/null)
}

sccache_reqs() { sccache --show-stats 2>/dev/null | awk '/^Compile requests  /{print $3}'; }
sccache_heal() {   # the shared server degrades to one compile at a time while clients queue (2026-10-10, twice)
  local srv waiting kids a b
  command -v sccache >/dev/null || return 0
  srv="$(ps -axo pid=,command= | awk '$2 ~ /sccache$/ && NF==2 {print $1; exit}')"
  [ -n "$srv" ] || return 0
  waiting="$(pgrep -f '^/opt/homebrew/bin/sccache /' | wc -l | tr -d ' ')"
  kids="$(pgrep -P "$srv" | wc -l | tr -d ' ')"
  [ "$waiting" -ge 10 ] && [ "$kids" -le 1 ] || { clear_alert "sccache:SERIALISED"; return 0; }
  a="$(sccache_reqs)"; sleep 20; b="$(sccache_reqs)"
  kids="$(pgrep -P "$srv" | wc -l | tr -d ' ')"
  [ "$kids" -le 1 ] && [ $(( ${b:-0} - ${a:-0} )) -le 2 ] || return 0
  # Waiting clients survive a restart: SCCACHE_IGNORE_SERVER_IO_ERROR=1 makes them compile locally.
  env -u MAKEFLAGS -u CARGO_MAKEFLAGS -u MFLAGS sccache --stop-server >/dev/null 2>&1
  env -u MAKEFLAGS -u CARGO_MAKEFLAGS -u MFLAGS SCCACHE_CACHE_SIZE=30G SCCACHE_IGNORE_SERVER_IO_ERROR=1 \
    sccache --start-server >/dev/null 2>&1
  printf '%s | %s\n' "$(date +%FT%T)" "sccache serialised ($waiting waiting, $a→$b requests in 20s); restarted" >> "$prog/alerts.log"
}

pass() {
  local jf
  sccache_heal
  surfaces
  cloud
  pgrep -f "tools/ops/no-host-webkit.sh" >/dev/null || (nohup "$(dirname "$0")/no-host-webkit.sh" >/dev/null 2>&1 &)   # host WebKit guard must always run
  for jf in "$jobs"/*.json; do
    [ -e "$jf" ] || continue
    classify "$jf"
    case "$status" in
      FAILED) alert "$name:FAILED" "Job $name FAILED: $detail (log $(jget "$jf" log))"; clear_alert "$name:STALLED";;
      STALLED) alert "$name:STALLED" "Job $name STALLED: $detail";;
      RUNNING) clear_alert "$name:STALLED";;
      OK) clear_alert "$name:STALLED";;
    esac
  done
}

case "${1:-}" in
  --status)
    for jf in "$jobs"/*.json; do
      [ -e "$jf" ] || continue
      classify "$jf"; printf '  %-28s %-8s %s\n' "$name" "$status" "$detail"
    done
    if [ -n "${LEGATUM_AWS_PROFILE:-}" ]; then
      printf '  %-28s %s\n' "cloud servers" "$(aws --profile "$LEGATUM_AWS_PROFILE" --region "${LEGATUM_CLOUD_REGION:-us-east-1}" ec2 describe-instances \
        --filters "Name=tag:project,Values=legatum-sims" "Name=instance-state-name,Values=pending,running" \
        --query 'length(Reservations[].Instances[])' --output text 2>/dev/null || echo unreachable)"
    fi
    exit 0;;
  --once) pass; exit 0;;
  "") ;;
  *) echo "usage: $0 [--once|--status]" >&2; exit 2;;
esac

pidfile="$jobs/watchdog.pid"
if [ -f "$pidfile" ] && ps -p "$(cat "$pidfile")" -o command= 2>/dev/null | grep -q watchdog.sh; then
  echo "watchdog already running (pid $(cat "$pidfile"))" >&2; exit 1
fi
echo $$ > "$pidfile"
echo "watchdog started $(date +%FT%T) pid $$, every ${interval}s"
while :; do pass; sleep "$interval"; done

#!/usr/bin/env bash
# Launch a long background job and register it for tools/ops/watchdog.sh.
#   tools/ops/bgjob.sh <name> --log <file> [--ok <regex>] [--progress-min <minutes>] -- <command...>
# Runs the command under nohup, appends its output to the log, and writes .progress/jobs/<name>.json
# (name, pid, log, ok regex, started_at, stall_min, exit_code, ended_at). When the command exits, the log
# gets a final "__EXIT <code>" line and the json gets the exit code.
# --ok: extended regex that must appear in the log for a zero exit to count as success.
# --progress-min: the job is STALLED when its log doesn't change for this many minutes (default 30).
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
self="$root/tools/ops/bgjob.sh"
prog="${LEGATUM_PROGRESS:-$root/.progress}"
jobs="$prog/jobs"

esc() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }

if [ "${1:-}" = "--record-exit" ]; then   # internal: bgjob.sh --record-exit <json> <code>
  jf="$2"; code="$3"; [ -f "$jf" ] || exit 0
  sed -e "s/^  \"exit_code\": .*/  \"exit_code\": $code,/" -e "s/^  \"ended_at\": .*/  \"ended_at\": \"$(date +%FT%T)\"/" "$jf" > "$jf.tmp" && mv "$jf.tmp" "$jf"
  exit 0
fi

usage() { sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
[ $# -ge 1 ] || usage
name="$1"; shift
case "$name" in ''|*[!A-Za-z0-9._-]*|-*) echo "bgjob: bad job name '$name' (use letters, digits, . _ -)" >&2; exit 2;; esac
log=""; ok=""; stall=30
while [ $# -gt 0 ]; do
  case "$1" in
    --log) log="${2:-}"; shift 2;;
    --ok) ok="${2:-}"; shift 2;;
    --progress-min) stall="${2:-}"; shift 2;;
    --) shift; break;;
    *) usage;;
  esac
done
[ -n "$log" ] && [ $# -gt 0 ] || usage
case "$stall" in ''|*[!0-9]*) echo "bgjob: --progress-min must be a whole number" >&2; exit 2;; esac
case "$log" in /*) ;; *) log="$PWD/$log";; esac

mkdir -p "$jobs" "$(dirname "$log")"
jf="$jobs/$name.json"
if [ -f "$jf" ]; then
  oldpid="$(sed -n 's/^  "pid": \([0-9]*\).*/\1/p' "$jf" | head -1)"
  if [ -n "$oldpid" ] && [ "$oldpid" != 0 ] && ps -p "$oldpid" -o command= 2>/dev/null | grep -q bgjob \
     && grep -q '"exit_code": null' "$jf"; then
    echo "bgjob: job '$name' is already running (pid $oldpid)" >&2; exit 1
  fi
fi
# A fresh launch under an old name starts with a clean alert state.
if [ -f "$jobs/alerted.txt" ]; then
  grep -v "^$name:" "$jobs/alerted.txt" > "$jobs/alerted.txt.tmp"; mv "$jobs/alerted.txt.tmp" "$jobs/alerted.txt"
fi

cat > "$jf" <<JSON
{
  "name": "$(esc "$name")",
  "pid": 0,
  "log": "$(esc "$log")",
  "ok": "$(esc "$ok")",
  "started_at": "$(date +%FT%T)",
  "started_epoch": $(date +%s),
  "stall_min": $stall,
  "exit_code": null,
  "ended_at": null
}
JSON

nohup bash -c '
  jf=$1; log=$2; self=$3; shift 3
  n=0; while grep -q "\"pid\": 0," "$jf" && [ $n -lt 100 ]; do sleep 0.05; n=$((n+1)); done
  "$@" >>"$log" 2>&1 </dev/null
  rc=$?
  echo "__EXIT $rc" >>"$log"
  "$self" --record-exit "$jf" "$rc"
' bgjob "$jf" "$log" "$self" "$@" >/dev/null 2>&1 </dev/null &
pid=$!
disown "$pid" 2>/dev/null || true
sed "s/^  \"pid\": 0,/  \"pid\": $pid,/" "$jf" > "$jf.tmp" && mv "$jf.tmp" "$jf"
echo "bgjob: started '$name' pid $pid, log $log, job file $jf"

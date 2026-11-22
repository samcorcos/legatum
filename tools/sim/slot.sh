#!/bin/bash
# Run a heavy command only when its threads fit under a machine-wide cap, shared by every agent.
#
#   tools/sim/slot.sh -- <command and args>          # threads read from --threads N (else the cap)
#   tools/sim/slot.sh --threads 4 -- <command...>
#
# Many agents start 3-6 thread sims at once on 16 cores (load passed 200), which slows every
# build, test and UI check. Sims are CPU bound, so queueing them loses no throughput. Each run
# takes N tokens from a pool of LEGATUM_SIM_THREAD_CAP (default 12, leaving 4 cores for builds,
# tests and the UI) and waits until they're free. Tokens of dead holders are reclaimed.
# Waiters are served first-come, first-served (a ticket queue), so small requests can't be starved
# by larger ones that keep grabbing freed tokens; a lock left empty by a killed process is broken.
set -uo pipefail
cap="${LEGATUM_SIM_THREAD_CAP:-12}"
dir="${LEGATUM_SLOT_DIR:-${LEGATUM_RUNS:-$HOME/LegatumRuns}/slots}"
want=""
if [[ "${1:-}" == "--threads" ]]; then want="$2"; shift 2; fi
[[ "${1:-}" == "--" ]] && shift
[[ $# -gt 0 ]] || { echo "usage: $0 [--threads N] -- <command...>" >&2; exit 2; }
if [[ -z "$want" ]]; then
  prev=""
  for a in "$@"; do
    [[ "$prev" == "--threads" ]] && want="$a"
    case "$a" in --threads=*) want="${a#--threads=}" ;; esac
    prev="$a"
  done
fi
[[ "$want" =~ ^[0-9]+$ && "$want" -gt 0 ]] || want="$cap" # legatum-sim defaults to all cores
(( want > cap )) && want="$cap"

mkdir -p "$dir/held" "$dir/queue"
mutex="$dir/mutex"
token="$dir/held/$$_$want"

# The workspace sandbox can deny kill -0 for a live process outside this process
# group. An EPERM result still means the pid exists; only ESRCH means it died.
alive() {
  local error
  error="$(kill -0 "$1" 2>&1)" && return 0
  [[ "$error" == *"permitted"* ]]
}

lock() { # mkdir mutex holding the owner's pid; a dead or missing owner's mutex is broken.
  until mkdir "$mutex" 2>/dev/null; do
    owner="$(ls "$mutex" 2>/dev/null | head -1)"
    if [[ -n "$owner" ]] && ! alive "$owner"; then
      rmdir "$mutex/$owner" "$mutex" 2>/dev/null
    elif [[ -z "$owner" && -n "$(find "$mutex" -maxdepth 0 -mmin +1 2>/dev/null)" ]]; then
      rmdir "$mutex" 2>/dev/null # created by a process killed before it recorded its pid
    fi
    sleep 1
  done
  mkdir "$mutex/$$"
}
unlock() { rmdir "$mutex/$$" "$mutex"; }

used() { # sum of live holders' tokens; reclaims tokens of dead holders
  local total=0 t pid n
  for t in "$dir"/held/*_*; do
    [[ -d "$t" ]] || continue
    t="${t##*/}"; pid="${t%_*}"; n="${t#*_}"
    if alive "$pid"; then total=$((total + n)); else rmdir "$dir/held/$t" 2>/dev/null; fi
  done
  echo "$total"
}

head_ticket() { # oldest ticket whose owner is alive; dead owners' tickets are dropped
  local t pid
  for t in $(ls "$dir/queue" 2>/dev/null | sort); do
    pid="${t#*_}"; pid="${pid%_*}"
    if alive "$pid"; then echo "$t"; return; fi
    rmdir "$dir/queue/$t" 2>/dev/null
  done
}

ticket=""
child=""
release() { rmdir "$token" 2>/dev/null; [[ -n "$ticket" ]] && rmdir "$dir/queue/$ticket" 2>/dev/null; }
trap 'release' EXIT
lock
seq=$(( $(cat "$dir/seq" 2>/dev/null || echo 0) + 1 ))
echo "$seq" > "$dir/seq"
ticket="$(printf '%012d' "$seq")_$$_$want"
mkdir "$dir/queue/$ticket"
unlock

waited=0
while :; do
  lock
  if [[ "$(head_ticket)" == "$ticket" ]] && (( $(used) + want <= cap )); then
    mkdir "$token"; rmdir "$dir/queue/$ticket"; ticket=""; unlock; break
  fi
  unlock
  (( waited == 0 )) && echo "slot: queued for $want of $cap sim threads" >&2
  waited=1
  sleep 5
done
trap '[[ -n "$child" ]] && kill "$child" 2>/dev/null; exit 143' TERM INT HUP
"$@" &
child=$!
wait "$child"

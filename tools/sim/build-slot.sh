#!/bin/bash
# Serialize heavy Cargo invocations through a machine-wide FIFO pool.
# Usage: tools/sim/build-slot.sh -- cargo test --workspace --release
set -uo pipefail

alive() { # sandboxed kill -0 can report EPERM for a live process outside the sandbox.
  local output
  output="$(kill -0 "$1" 2>&1)" && return 0
  [[ "$output" == *"not permitted"* ]]
}

cap="${LEGATUM_BUILD_CAP:-3}"
[[ "$cap" =~ ^[0-9]+$ && "$cap" -gt 0 ]] || { echo "LEGATUM_BUILD_CAP must be a positive integer" >&2; exit 2; }
[[ "${1:-}" == "--" ]] && shift
[[ $# -gt 0 ]] || { echo "usage: $0 -- <command...>" >&2; exit 2; }
# Codex lanes (LEGATUM_OFFLOAD=1) forward cargo test/nextest/clippy/check to the cloud build box (tools/cloud/offload.sh),
# which costs this Mac no CPU: no slot to take. While the box is marked unavailable they run here, so keep the cap.
if [[ "${LEGATUM_OFFLOAD:-0}" == 1 && "$1" == cargo ]]; then
  case "${2:-}" in
    test|nextest|clippy|check)
      unavailable_until="$(cat "${LEGATUM_RUNS:-$HOME/LegatumRuns}/box/offload-unavailable-until" 2>/dev/null || echo 0)"
      (( $(date +%s) < ${unavailable_until:-0} )) || exec "$@" ;;
  esac
fi
dir="${LEGATUM_BUILD_SLOT_DIR:-${LEGATUM_RUNS:-$HOME/LegatumRuns}/build-slots}"
mkdir -p "$dir/held" "$dir/queue"
mutex="$dir/mutex"

lock() { # mkdir mutex holding the owner's pid; a dead or missing owner's mutex is broken.
  until mkdir "$mutex" 2>/dev/null; do
    owner="$(ls "$mutex" 2>/dev/null | head -1)"
    if [[ -n "$owner" ]] && ! alive "$owner"; then
      rmdir "$mutex/$owner" "$mutex" 2>/dev/null
    elif [[ -z "$owner" && -n "$(find "$mutex" -maxdepth 0 -mmin +1 2>/dev/null)" ]]; then
      rmdir "$mutex" 2>/dev/null
    fi
    sleep 1
  done
  mkdir "$mutex/$$"
}
unlock() { rmdir "$mutex/$$" "$mutex"; }

used() { # count live holders; reclaim tokens from dead owners.
  local total=0 t pid
  for t in "$dir"/held/*; do
    [[ -d "$t" ]] || continue
    t="${t##*/}"; pid="${t#*_}"
    if alive "$pid"; then total=$((total + 1)); else rmdir "$dir/held/$t" 2>/dev/null; fi
  done
  echo "$total"
}

head_ticket() { # oldest live ticket; remove tickets left by dead waiters.
  local t pid
  for t in $(ls "$dir/queue" 2>/dev/null | sort); do
    pid="${t#*_}"
    if alive "$pid"; then echo "$t"; return; fi
    rmdir "$dir/queue/$t" 2>/dev/null
  done
}

ahead_count() {
  local t count=0
  for t in $(ls "$dir/queue" 2>/dev/null | sort); do
    [[ "$t" == "$ticket" ]] && break
    ((count += 1))
  done
  echo "$count"
}

ticket=""
token="$dir/held/$$"
child=""
release() { rmdir "$token" 2>/dev/null; [[ -n "$ticket" ]] && rmdir "$dir/queue/$ticket" 2>/dev/null; }
trap 'release' EXIT
lock
seq=$(( $(cat "$dir/seq" 2>/dev/null || echo 0) + 1 ))
echo "$seq" > "$dir/seq"
ticket="$(printf '%012d' "$seq")_$$"
mkdir "$dir/queue/$ticket"
unlock

last_ahead=""
while :; do
  lock
  if [[ "$(head_ticket)" == "$ticket" ]] && (( $(used) < cap )); then
    mkdir "$token"; rmdir "$dir/queue/$ticket"; ticket=""; unlock; break
  fi
  ahead="$(ahead_count)"
  unlock
  if [[ "$ahead" != "$last_ahead" ]]; then
    echo "waiting for build slot: $ahead ahead" >&2
    last_ahead="$ahead"
  fi
  sleep 2
done
trap '[[ -n "$child" ]] && kill "$child" 2>/dev/null; exit 143' TERM INT HUP
export LEGATUM_BUILD_SLOT_HELD=1 # tools/cloud/offload.sh must not queue a second time when it falls back to local
"$@" &
child=$!
wait "$child"

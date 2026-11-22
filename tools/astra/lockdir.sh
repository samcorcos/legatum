# shellcheck shell=bash
# mkdir-based lock directories whose holder is checked for life. Sourced by verify-branch.sh (macOS bash 3.2 and the
# box's bash 5). A lock is a directory DIR with a file DIR/pid holding "<pid> <process start time>".
#
# Why: waiting on `mkdir DIR` alone never asks whether the holder is alive. A verify that was killed (box-job kill,
# a dropped client, a box reboot) left its lock behind and every later verify of that branch or step waited the full
# four hours (2026-10-18: stale vrbranch locks and 23 dead gate-cache locks stranded the whole box).
#
#   lockdir_try DIR       0 = DIR is now ours; 1 = held by a live process (the caller sleeps and retries).
#                         A lock whose holder is dead is taken over (and says so on stderr).
#   lockdir_release DIR   drop our lock (no-op when DIR is not ours)
#
# Holder liveness: pid alive AND its start time equals the recorded one (a recycled pid is not the holder). A lock
# with no pid file is a holder between mkdir and its first write: alive for 60 s, stale after that.
# Takeover renames the stale directory aside first so exactly one waiter wins, and puts it back when what it moved
# turns out to be a live lock that a faster waiter had just created.

lockdir_start_of() { ps -o lstart= -p "$1" 2>/dev/null | tr -s ' ' | sed 's/^ //'; }

lockdir_stamp() { printf '%s %s\n' "$1" "$(lockdir_start_of "$1")" > "$2/pid.tmp.$1" && mv -f "$2/pid.tmp.$1" "$2/pid"; }

lockdir_holder_alive() { # 0 when the lock's holder is (or may still be) running
  local dir="$1" line pid start now
  if [ ! -f "$dir/pid" ]; then
    [ -z "$(find "$dir" -maxdepth 0 -mmin +1 2>/dev/null)" ]   # younger than a minute: still being created
    return
  fi
  line="$(cat "$dir/pid" 2>/dev/null)"
  pid="${line%% *}"
  case "$pid" in ''|*[!0-9]*) return 1;; esac
  now="$(lockdir_start_of "$pid")"
  [ -n "$now" ] || return 1
  case "$line" in *" "*) start="${line#* }"; [ -z "$start" ] || [ "$start" = "$now" ] || return 1;; esac
  return 0
}

lockdir_try() {
  local dir="$1" aside
  if mkdir "$dir" 2>/dev/null; then lockdir_stamp "$$" "$dir"; return 0; fi
  [ -d "$dir" ] || return 1
  lockdir_holder_alive "$dir" && return 1
  aside="$dir.stale.$$"
  rm -rf "$aside"
  mv "$dir" "$aside" 2>/dev/null || return 1   # another waiter already took it over
  if lockdir_holder_alive "$aside"; then       # we moved a live lock (a faster waiter had just replaced the stale one)
    [ -e "$dir" ] || mv "$aside" "$dir" 2>/dev/null
    return 1
  fi
  echo "lockdir: $dir held by a dead process ($(cat "$aside/pid" 2>/dev/null || echo 'no pid recorded')): taking it over" >&2
  rm -rf "$aside"
  if mkdir "$dir" 2>/dev/null; then lockdir_stamp "$$" "$dir"; return 0; fi
  return 1
}

lockdir_release() {
  local dir="$1" line
  line="$(cat "$dir/pid" 2>/dev/null)" || return 0
  [ "${line%% *}" = "$$" ] || return 0
  rm -rf "$dir"
}

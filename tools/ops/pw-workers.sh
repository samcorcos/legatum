#!/usr/bin/env bash
# Playwright worker sizing shared by release checks.
# Formula matches tools/astra/verify-branch.sh: min(cores / 3, (cores - load) / 2),
# clamped to 2..24. Optional arguments make the rule easy to test.
pw_workers() {
  local cores load free workers
  cores="${1:-}"
  load="${2:-}"
  if [ -z "$cores" ]; then
    cores="$(getconf _NPROCESSORS_ONLN 2>/dev/null)"
    [ -n "$cores" ] || cores="$(sysctl -n hw.ncpu 2>/dev/null)"
    [ -n "$cores" ] || cores=4
  fi
  if [ -z "$load" ]; then
    if [ "$(uname -s)" = Darwin ]; then
      load="$(sysctl -n vm.loadavg 2>/dev/null | awk '{printf "%d", $2}')"
    else
      load="$(awk '{printf "%d", $1}' /proc/loadavg 2>/dev/null)"
    fi
    [ -n "$load" ] || load=0
  fi
  free=$(( (cores - load) / 2 ))
  workers=$(( cores / 3 < free ? cores / 3 : free ))
  [ "$workers" -ge 2 ] || workers=2
  [ "$workers" -le 24 ] || workers=24
  echo "$workers"
}

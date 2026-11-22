#!/bin/bash
# Print how many threads a new heavy job may use on this Mac right now: the cores not already busy, less a reserve
# kept open for the user (default 2), never under 2.
#
#   CARGO_BUILD_JOBS=$(tools/ops/free-threads.sh) cargo build ...
#   tools/ops/free-threads.sh 2      # the share for each of 2 jobs started together
#
# Busy cores are the one-minute load average, so a running game, sim or other agent's build shrinks the answer and an
# idle machine gives nearly all of it. LEGATUM_RESERVE_CORES changes the reserve.
set -euo pipefail
jobs="${1:-1}"
reserve="${LEGATUM_RESERVE_CORES:-2}"
ncpu="$(sysctl -n hw.ncpu 2>/dev/null || nproc)"
load="$(uptime | sed -E 's/.*load averages?: *([0-9]+)[.,].*/\1/')"
free=$(( (ncpu - reserve - load) / jobs ))
(( free >= 2 )) || free=2
echo "$free"

#!/usr/bin/env bash
# Run one heavy command on the cloud build box instead of this Mac, then apply the files it changed. Used by the
# Codex lanes (tools/astra/runner.sh) so builds, tests and repins do not drain the laptop's battery.
#
#   tools/cloud/offload.sh [--prep none|wasm|dist|web] [--artifact PATH]... -- <command...>
#
# The command runs on the box in a fresh slot checkout of this worktree (HEAD plus uncommitted and untracked
# files), in the same sub-directory, with output and exit code streamed back. Afterwards every non-ignored file the
# command added, changed or deleted is written back into this worktree (repin's fixtures and pins.json, a rewritten
# Cargo.lock). Git-ignored outputs (wasm-pkg, dist, test-results, target/) come back only when named with --artifact.
#   --prep wasm|dist|web  the box rebuilds app/src/wasm-pkg (and app/dist, and for web the e2e web dist) first when stale
#   --artifact PATH          also copy this git-ignored path back (relative to the worktree root); repeatable
#
# It runs the command LOCALLY at low priority (nice -n 15, and in the build-slot FIFO for cargo), and never fails the caller
# for infrastructure reasons, when
#   - LEGATUM_OFFLOAD is not 1, or LEGATUM_ON_BOX is set (already on the box);
#   - .progress/astra-queue/no-offload exists in the main checkout (the kill switch);
#   - the box was unavailable in the last few minutes (a marker, ~/LegatumRuns/box/offload-unavailable-until);
#   - remote.sh could not run the job at all (exit 3: capacity, daily cap, credentials, spot interruption; or the
#     job never started). A command that ran on the box and failed keeps its exit code: no re-run locally.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"

prep=none; extra=(); cmd=()
while [ $# -gt 0 ]; do
  case "$1" in
    --prep) prep="${2:?--prep needs a value}"; shift 2;;
    --artifact) extra+=("${2:?--artifact needs a path}"); shift 2;;
    --) shift; cmd=("$@"); break;;
    *) echo "offload: unknown option $1" >&2; exit 2;;
  esac
done
[ "${#cmd[@]}" -gt 0 ] || { echo "usage: offload.sh [--prep none|wasm|dist|web] [--artifact PATH]... -- <command...>" >&2; exit 2; }

lcmd=("${cmd[@]}") # what runs here; cmd is rewritten for the box below
runs="${LEGATUM_RUNS:-$HOME/LegatumRuns}"
marker="$runs/box/offload-unavailable-until"
top="$(git rev-parse --show-toplevel 2>/dev/null || true)"
main=""; [ -z "$top" ] || main="$(cd "$(git rev-parse --git-common-dir)/.." 2>/dev/null && pwd)"

# The real (non-shim) tool: the lane bin directory is dropped from PATH.
real_path() {
  local out="" d IFS=:
  for d in $PATH; do [ "$d" = "${LEGATUM_LANE_BIN:-}" ] || out="${out:+$out:}$d"; done
  printf '%s' "$out"
}

niced_exec() { # exec at low priority; the Codex sandbox denies setpriority, so lanes start under nice -n 15 (LEGATUM_LANE_NICED=1)
  if [ "${LEGATUM_LANE_NICED:-0}" = 1 ] || [ -n "$(nice -n 15 true 2>&1)" ]; then exec "$@"; else exec nice -n 15 "$@"; fi
}

local_run() { # run the command here at low priority; nothing below re-offloads
  local why="$1"; shift
  [ -z "$why" ] || echo "offload: running locally, niced ($why): $*" >&2
  export PATH; PATH="$(real_path)"
  export LEGATUM_OFFLOAD=0
  local slot=()
  if [ "$1" = cargo ] && [ -z "${LEGATUM_BUILD_SLOT_HELD:-}" ] && [ -x "$top/tools/sim/build-slot.sh" ]; then
    case "${2:-}" in test|nextest|clippy|check|build|run|bench|doc) slot=("$top/tools/sim/build-slot.sh" --);; esac
  fi
  niced_exec ${slot[@]+"${slot[@]}"} "$@"
}

[ "${LEGATUM_OFFLOAD:-0}" = 1 ] && [ -z "${LEGATUM_ON_BOX:-}" ] || local_run "" "${lcmd[@]}"
[ -n "$top" ] || local_run "not in a git worktree" "${lcmd[@]}"
[ ! -e "$main/.progress/astra-queue/no-offload" ] || local_run "kill switch .progress/astra-queue/no-offload" "${lcmd[@]}"
if [ -f "$marker" ] && [ "$(date +%s)" -lt "$(cat "$marker" 2>/dev/null || echo 0)" ]; then
  local_run "the box was unavailable a moment ago" "${lcmd[@]}"
fi

rel="$(git rev-parse --show-prefix)"; rel="${rel%/}"; : "${rel:=.}"
# A script given by absolute path (scripts that offload themselves): run it from the checkout root by relative path.
if [[ "${cmd[0]}" == "$top/"* ]]; then cmd[0]="${cmd[0]#"$top"/}"; rel=.; fi
errf="$(mktemp)"; adir="$(mktemp -d)"
trap 'rm -rf "$errf" "$adir"' EXIT

args=(--with-uncommitted --artifact-dir "$adir" --label "lane:$(basename "$top"):${cmd[0]} ${cmd[1]:-}" --artifact .offload-out)
for a in ${extra[@]+"${extra[@]}"}; do args+=(--artifact "$a"); done
[ "$prep" = none ] || args+=(--npm)
# Environment that changes what the command does and is the same on any machine.
for v in RUST_BACKTRACE RUST_LOG PROPTEST_CASES LEGATUM_REPIN_PROFILE LEGATUM_REPIN_THREADS LEGATUM_WASM_NO_OPT; do
  val="${!v:-}"
  case "$val" in ''|*[!A-Za-z0-9_.,=:+-]*) ;; *) args+=(--env "$v=$val");; esac
done

"$here/remote.sh" "$top" "${args[@]}" -- bash /data/bin/lane-run.sh "$rel" "$prep" -- "${cmd[@]}" 2>"$errf"
rc=$?
cat "$errf" >&2

# Apply what the command changed, whether it passed or failed.
if [ -d "$adir/.offload-out" ]; then
  if [ -s "$adir/.offload-out/changes.tar" ]; then tar xf "$adir/.offload-out/changes.tar" -C "$top" 2>/dev/null; fi
  if [ -s "$adir/.offload-out/deleted.nul" ]; then ( cd "$top" && xargs -0 rm -f < "$adir/.offload-out/deleted.nul" ); fi
  rm -rf "$adir/.offload-out"
fi
for a in ${extra[@]+"${extra[@]}"}; do
  if [ -e "$adir/$a" ]; then mkdir -p "$top/$(dirname "$a")" && rm -rf "$top/$a" && cp -a "$adir/$a" "$top/$a"; fi
done

# A job that really ran prints "box: job ... exited N"; without that line the box never ran the command.
if grep -aq '^box: job .* exited [0-9]' "$errf"; then exit "$rc"; fi
[ "$rc" -ne 0 ] || exit 0
mkdir -p "$(dirname "$marker")"
secs=600; grep -aqi 'daily cap' "$errf" && secs=3600
echo $(( $(date +%s) + secs )) > "$marker"
local_run "the box could not run it (exit $rc)" "${lcmd[@]}"

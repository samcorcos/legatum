#!/usr/bin/env bash
# Full release check for a playtest worktree.
# Usage: tools/release/check.sh <worktree> <log> "<label>"   e.g. "Playtest 7a" (run detached: nohup ... & disown)
# Set RELEASE_CHECK_DRY_RUN=1 to exercise every step and its timing row without running builds or tests.
# Steps: build sim + server (release and debug; Playwright's staging spec spawns the debug server),
# re-record every scenario baseline, fmt, clippy, tests, WASM, tsc, vitest, app build, Playwright.
# Each step logs "== name HH:MM" then "exit N"; the summary prints at the end.
# Release is the critical path: its Cargo steps do not queue in tools/sim/build-slot.sh (Codex tasks do; the
# inner scripts must never take a slot because build-slot is not re-entrant).
# Independent steps overlap: fmt and clippy run beside the tests, and the WebKit container runs beside vitest and
# Playwright Chromium. Each overlapped step logs to its own file that is appended to $L when it is joined, so the
# "== name" / "exit N" pairs stay together. Everything still runs, and any failed step fails the check.
# Linux: the surface counter is skipped.
set -uo pipefail
W="${1:?worktree}"; L="${2:?log file}"; export LEGATUM_BUILD_LABEL="${3:?build label, e.g. Playtest 7a}"
cd "$W" || exit 1
# Later steps enter app/, so keep paths to the worktree and its WebKit runner absolute.
W="$(pwd -P)"
source "$W/tools/ops/pw-workers.sh"
renice 15 -p $$ >/dev/null 2>&1 || true
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
: > "$L"
surf() { [ "$(uname -s)" = Darwin ] && ioclasscount IOSurface 2>/dev/null | awk '{print $NF}'; true; }
load_avg() { if [ "$(uname -s)" = Darwin ]; then sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}'; else awk '{print $1}' /proc/loadavg 2>/dev/null; fi; }
# IOSurface counts before/after each step attribute WindowServer surface growth (it aborted at 46k on 2026-10-05).
timed_record() {
  local kind="$1" name="$2" label="$3" ref="$4" rc="$5" wall="$6" cpu="$7" load="$8"
  python3 "$W/tools/ops/timed.py" --record "$kind" "$name" "$label" "$ref" "$rc" "$wall" "$cpu" "$load" >/dev/null 2>&1 || true
}
step() {
  local name="$1" n rc t0 load1 times0 times1 cpu tree out="${STEP_LOG:-$L}"
  shift
  n="$(surf)"
  if [ "${n:-0}" -ge "${SURFACE_ABORT:-40000}" ]; then
    echo "== $name ABORTED: $n IOSurfaces (WindowServer aborts near 46k; log out or restart first)" >> "$out"
    echo "exit 99" >> "$out"
    echo "$name" >> "$L.failed"
    tree="$(git rev-parse --short=12 'HEAD^{tree}' 2>/dev/null || echo unknown)"
    timed_record release "$name" "$LEGATUM_BUILD_LABEL" "$tree" 99 0 0 0
    exit 99
  fi
  echo "== $name $(date +%H:%M) surfaces $n" >> "$out"
  times0="$(mktemp 2>/dev/null || true)"; times1="$(mktemp 2>/dev/null || true)"
  [ -z "$times0" ] || times > "$times0"
  t0=$SECONDS
  load1="$(load_avg)"; [ -n "$load1" ] || load1=0
  if [ "${RELEASE_CHECK_DRY_RUN:-0}" = 1 ]; then
    echo "dry run: $*" >> "$out"
    rc=0
  else
    "$@" >> "$out" 2>&1
    rc=$?
  fi
  [ -z "$times1" ] || times > "$times1"
  if [ -n "$times0" ] && [ -n "$times1" ]; then
    cpu="$(awk 'FNR==2 { s=0; for (i=1; i<=2; i++) { split($i, a, "m"); sub("s", "", a[2]); s += a[1]*60 + a[2] } print s }' "$times0" "$times1" | awk 'NR==1{a=$1} NR==2{printf "%.1f", $1-a}')"
    rm -f "$times0" "$times1" 2>/dev/null || true
  else cpu=0; fi
  [ -n "$cpu" ] || cpu=0
  tree="$(git rev-parse --short=12 'HEAD^{tree}' 2>/dev/null || echo unknown)"
  timed_record release "$name" "$LEGATUM_BUILD_LABEL" "$tree" "$rc" "$((SECONDS - t0))" "$cpu" "$load1"
  case "$name" in build-*)
    timed_record build "$name" "$LEGATUM_BUILD_LABEL" "$tree" "$rc" "$((SECONDS - t0))" "$cpu" "$load1";;
  esac
  echo "exit $rc (surfaces $(surf))" >> "$out"
  [ "$rc" -eq 0 ] || echo "$name" >> "$L.failed"
  return 0
}
# bg_step <name> <command...>: run a step in the background with its own log; join_steps appends the logs to $L.
BG_PIDS=""; BG_NAMES=""
bg_step() {
  local name="$1"
  ( STEP_LOG="$L.$name"; : > "$STEP_LOG"; step "$@" ) &
  BG_PIDS="$BG_PIDS $!"; BG_NAMES="$BG_NAMES $name"
}
join_steps() {
  local pid name
  for pid in $BG_PIDS; do wait "$pid"; done
  for name in $BG_NAMES; do cat "$L.$name" >> "$L"; rm -f "$L.$name"; done
  BG_PIDS=""; BG_NAMES=""
}
rm -f "$L.failed"
[ -e app/node_modules ] || ln -s "$(git rev-parse --path-format=absolute --git-common-dir)/../app/node_modules" app/node_modules
step build-sim cargo build -q --release -p legatum-sim
for f in scenarios/*.json; do
  s="$(basename "$f" .json)"
  step "baseline-$s" ./target/release/legatum-sim scenario "$f" --runs 50 --difficulty hard --threads 6 --record --out docs/sim-results/scenarios
done
# fmt and clippy use the debug profile directory, the tests the release one, so they overlap without cargo lock waits.
bg_step fmt cargo fmt --all --check
bg_step clippy cargo clippy --workspace --all-targets -- -D warnings
step test tools/astra/gate-tests.sh release
join_steps
step wasm scripts/build-wasm.sh
cd app || exit 1
step tsc npx tsc --noEmit
step build npm run build
# WebKit never runs on the macOS host (it crashed WindowServer on 2026-10-07 even at one worker); it runs in a Linux container.
# It needs only app/dist, so it starts as soon as the build is done and runs beside vitest and Playwright Chromium.
if [ "$(uname -s)" != Darwin ] && [ -x "$W/tools/cloud/box/webkit.sh" ] && [ -z "${SKIP_WEBKIT:-}" ]; then
  # On the cloud box (Linux) WebKit runs natively: no container, no 4-CPU cap, and it reuses the build above.
  # Its output directory is its own, because Chromium wipes and rewrites app/test-results while this runs.
  bg_step playwright-webkit bash -c 'cd "$1" && exec env LEGATUM_WEBKIT_PREBUILT=1 LEGATUM_WEBKIT_OUTPUT=test-results-webkit bash tools/cloud/box/webkit.sh' _ "$W"
elif [ -x "$W/tools/release/webkit-docker.sh" ] && [ -z "${SKIP_WEBKIT:-}" ]; then
  # Its results go to app/test-results-webkit: Chromium wipes and rewrites app/test-results while this runs.
  bg_step playwright-webkit env LEGATUM_WEBKIT_RESULTS="$W/app/test-results-webkit" "$W/tools/release/webkit-docker.sh" "$W"
else echo "== playwright-webkit SKIPPED (no Linux container runner, or SKIP_WEBKIT)" >> "$L"; echo "exit 0 (skipped)" >> "$L"; fi
step vitest npx vitest run --maxWorkers=6 --testTimeout=60000
# Workers: LEGATUM_PW_WORKERS (on the box, the job's core-budget share: tools/cloud/box/budget.sh), else the load-aware sizing.
# The 2026-10-18 sweep (docs/ops/test-gate-timings.md) found the Chromium wall time flat from 12 to 48 workers on a busy box,
# with failures growing with the count: more workers than cores to feed them buy nothing.
step playwright npx playwright test --workers="${LEGATUM_PW_WORKERS:-$(pw_workers)}"
join_steps
grep -E "^== |^exit " "$L" | paste - - | tee -a "$L.summary"
# Every step runs so one check reports everything, but any failed step fails the release check.
FAILED=""; [ ! -f "$L.failed" ] || FAILED="$(tr '\n' ' ' < "$L.failed")"
if [ -n "$FAILED" ]; then echo "RELEASE CHECK FAILED:$FAILED" | tee -a "$L" "$L.summary"; exit 1; fi
echo "RELEASE CHECK PASSED" | tee -a "$L" "$L.summary"

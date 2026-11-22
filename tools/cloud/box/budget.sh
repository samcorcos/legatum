# shellcheck shell=bash
# Sourced ON the box by box-job, verify-job.sh, lane-run.sh and build-sim.sh: this job's share of the cores.
#
# The box has ncpu cores and up to SLOTS jobs at once. A job's budget is 1.25 x ncpu / (slots held + jobs queued, between 4 and SLOTS):
# about 6 cores each at full occupancy, 20 for a lone job, which keeps the load near 64 to 80.
#
# box-job calls box_core_budget_apply when a job starts. Commands also call it so jobs started by a box-job without a
# core budget receive one. Every value is a default for unset variables.
: "${DATA:=/data}"

box_core_budget_apply() {
  local slotdir="${SLOTDIR:-$DATA/slots}" slots ncpu n=0 k q B force=0
  slots="${SLOTS:-$(sed -n 's/^SLOTS=//p' "$DATA/bin/config" 2>/dev/null)}"; slots="${slots:-12}"
  ncpu="$(nproc)"
  [ -n "${LEGATUM_CORE_BUDGET:-}" ] || force=1
  if [ "$force" = 1 ]; then
    # Slots held right now, from the slot locks (this job's is among them): a held lock refuses a non-blocking flock.
    for k in $(seq 1 "$slots" | sed 's/^/s/') golden; do
      [ -e "$slotdir/$k.lock" ] || continue
      ( flock -n 8 ) 8>"$slotdir/$k.lock" || n=$((n + 1))
    done
    q=$(( $(cat "$slotdir/next-ticket" 2>/dev/null || echo 1) - $(cat "$slotdir/serving" 2>/dev/null || echo 1) ))
    [ "$q" -ge 0 ] || q=0
    [ "$n" -ge 1 ] || n=1
    n=$((n + q)); [ "$n" -ge 4 ] || n=4; [ "$n" -le "$slots" ] || n="$slots"
    B=$((ncpu * 5 / 4 / n)); [ "$B" -ge 4 ] || B=4
    export LEGATUM_CORE_BUDGET="$B"
  fi
  B="$LEGATUM_CORE_BUDGET"
  _bsetv() { if [ "$force" = 1 ] || [ -z "${!1:-}" ]; then export "$1=$2"; fi; }
  _bsetv BOX_CARGO_JOBS "$B"
  _bsetv RAYON_NUM_THREADS "$B"   # any rayon program (sims, tests) otherwise spawns one thread per core
  _bsetv NEXTEST_TEST_THREADS "$B"
  _bsetv RUST_TEST_THREADS "$(( B > 32 ? 32 : B ))"
  # wasm-opt (binaryen) otherwise starts one thread per core: 64 of them, in every job that builds the WASM package.
  _bsetv BINARYEN_CORES "$(( B / 2 < 2 ? 2 : (B / 2 > 16 ? 16 : B / 2) ))"
  # vitest otherwise starts one worker per core; VITEST_MAX_WORKERS is its environment switch for maxWorkers.
  _bsetv LEGATUM_VITEST_WORKERS "$(( B / 2 < 2 ? 2 : (B / 2 > 12 ? 12 : B / 2) ))"   # CPU-bound: 570 cpu-s, one worker took 450 s of a fast verify
  _bsetv VITEST_MAX_WORKERS "$LEGATUM_VITEST_WORKERS"
  # Playwright otherwise starts half the cores' worth of workers (32), each with its own browser and server.
  _bsetv LEGATUM_PW_WORKERS "$(( B / 3 < 1 ? 1 : (B / 3 > 24 ? 24 : B / 3) ))"
  [ "$force" = 0 ] || echo "[box $(date +%T)] core budget $B of $ncpu (slots held $((n - q)), queued $q): cargo jobs $BOX_CARGO_JOBS, test threads $NEXTEST_TEST_THREADS"
  CARGO_BUILD_JOBS="$BOX_CARGO_JOBS"; export CARGO_BUILD_JOBS
}

# tools/astra/verify-branch.sh reads these overrides on any master age (newer ones also read LEGATUM_CORE_BUDGET itself).
box_verify_sizes() {
  local B="${LEGATUM_CORE_BUDGET:?run box_core_budget_apply first}"
  _vclamp() { local v="$1"; [ "$v" -ge "$2" ] || v="$2"; [ "$v" -le "$3" ] || v="$3"; echo "$v"; }
  export LEGATUM_VERIFY_JOBS="${LEGATUM_VERIFY_JOBS:-$(_vclamp $((B / 3)) 2 32)}"
  export LEGATUM_VERIFY_TEST_THREADS="${LEGATUM_VERIFY_TEST_THREADS:-$(_vclamp "$B" 2 32)}"
  export LEGATUM_VERIFY_SIM_THREADS="${LEGATUM_VERIFY_SIM_THREADS:-$(_vclamp $((B / 4)) 2 8)}"
  export LEGATUM_VERIFY_PW_WORKERS="${LEGATUM_VERIFY_PW_WORKERS:-$(_vclamp $((B / 3)) 1 24)}"
  export LEGATUM_VERIFY_VITEST_WORKERS="${LEGATUM_VERIFY_VITEST_WORKERS:-$(_vclamp $((B / 2)) 2 12)}"
}

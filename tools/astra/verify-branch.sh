#!/usr/bin/env bash
# Verify a branch in an isolated, locked worktree. Intended to run outside Codex's sandbox.
set -uo pipefail

usage() {
  cat <<'EOF'
Usage: tools/astra/verify-branch.sh <branch> [--profile fast|extended|release] [--duel] [--app] [--dry-run] [--no-cache] [--full] [--serial]
       [--pw-shard K/N] [--skip-playwright]

Creates/reuses .claude/worktrees/verify-<branch-sanitised>, merges the branch onto
current master, runs the landing checks, and writes logs below .progress/verify/<branch>/.
--dry-run prints the planned steps without changing the repository.
--no-cache forces every step to run. Release checks never read this cache.
--full runs every requested step: nothing is skipped because of what the branch touches.
--serial runs the independent lanes one after another instead of at the same time.
--profile picks how much to run (docs: the legatum-workflow skill, "Three test gates"):
  fast      tsc, vitest, the app build, a small Playwright smoke set (game, turn-flow, research-click, production-picker,
            exit-to-title), and the Rust tests except the long AI, sim and property tests in tools/astra/extended-rust-tests.txt.
            The default for a change with no Rust (app, docs, tools).
  extended  the full Rust workspace and every Playwright spec except the screen tour. The default when Rust changed.
  release   extended plus the full two-size screen tour (e2e/tour); WebKit runs through verify-remote.sh --webkit.
  Playwright browser transport errors (net::ERR_NETWORK_CHANGED, a closed browser) get ONE rerun of only the failed tests;
  the summary lists the first failure, the rerun and the error classes. Assertion failures are never rerun.
--pw-shard K/N runs only Playwright shard K of N (and the merge, WASM, app build and server it needs): the pieces of one
verify that tools/cloud/verify-remote.sh --shards spreads over several box slots. --skip-playwright is the other half:
everything except the app build, server and Playwright.

Independent steps run concurrently in lanes (release-checked build, tests, repin and smoke; clippy and the server
build; the master baseline; the WASM, tsc, vitest and Playwright chain), sized from the core count. Steps the branch
cannot affect are skipped, and the summary says which and why: docs and skills only, nothing but syntax checks;
app only, no cargo steps; crates only, no Playwright unless crates/core/src, crates/wasm, crates/server, app or the
scenarios changed. Unclassified paths, a branch that is master, and --full run everything. Changing app/ turns the
app lane on even without --app. The master side comes from tools/sim/cached-baseline.sh, keyed by the sim code.
Overrides: LEGATUM_VERIFY_JOBS (cargo jobs), _TEST_THREADS, _SIM_THREADS, _PW_WORKERS, _VITEST_WORKERS, _SERIAL=1.
EOF
}

die() { echo "verify-branch: $*" >&2; exit 2; }
# shellcheck source=tools/astra/lockdir.sh
. "$(dirname "${BASH_SOURCE[0]}")/lockdir.sh"
[ "$#" -ge 1 ] || { usage >&2; exit 2; }
BRANCH=""
DO_DUEL=0
DO_APP=0
DRY_RUN=0
NO_CACHE=0
FULL=0
SERIAL="${LEGATUM_VERIFY_SERIAL:-0}"
PROFILE_ARG=""; PW_SHARD=""; SKIP_PW=0
while [ "$#" -gt 0 ]; do
  arg="$1"; shift
  case "$arg" in
    --profile) [ "$#" -ge 1 ] || die "--profile needs fast, extended or release"; PROFILE_ARG="$1"; shift ;;
    --profile=*) PROFILE_ARG="${arg#--profile=}" ;;
    --pw-shard) [ "$#" -ge 1 ] || die "--pw-shard needs K/N"; PW_SHARD="$1"; shift ;;
    --pw-shard=*) PW_SHARD="${arg#--pw-shard=}" ;;
    --skip-playwright) SKIP_PW=1 ;;
    --duel) DO_DUEL=1 ;;
    --app) DO_APP=1 ;;
    --dry-run) DRY_RUN=1 ;;
    --no-cache) NO_CACHE=1 ;;
    --full) FULL=1 ;;
    --serial) SERIAL=1 ;;
    -h|--help) usage; exit 0 ;;
    -*) die "unknown option: $arg" ;;
    *) [ -z "$BRANCH" ] || die "unexpected argument: $arg"; BRANCH="$arg" ;;
  esac
done
[ -n "$BRANCH" ] || { usage >&2; exit 2; }
case "${PROFILE_ARG:-${LEGATUM_VERIFY_PROFILE:-}}" in ''|fast|extended|release) ;; *) die "unknown profile: ${PROFILE_ARG:-$LEGATUM_VERIFY_PROFILE} (fast, extended or release)" ;; esac
case "$PW_SHARD" in ''|[1-9]*/[1-9]*) ;; *) die "--pw-shard needs K/N, for example 2/3" ;; esac

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "run inside a git checkout"
cd "$ROOT" || die "cannot enter $ROOT"
git check-ref-format --branch "$BRANCH" >/dev/null 2>&1 || die "invalid branch name: $BRANCH"
git show-ref --verify --quiet "refs/heads/$BRANCH" || git show-ref --verify --quiet "refs/remotes/$BRANCH" || die "branch not found: $BRANCH"
case "$BRANCH" in refs/*) BRANCH_REF="$BRANCH" ;; */*) BRANCH_REF="refs/heads/$BRANCH" ;; *) BRANCH_REF="refs/heads/$BRANCH" ;; esac
if ! git show-ref --verify --quiet "$BRANCH_REF"; then BRANCH_REF="refs/remotes/$BRANCH"; fi

SAFE_BRANCH="$(printf '%s' "$BRANCH" | tr -cs '[:alnum:]._-' '-')"
SAFE_BRANCH="${SAFE_BRANCH#-}"
SAFE_BRANCH="${SAFE_BRANCH%-}"
[ -n "$SAFE_BRANCH" ] || die "branch name sanitizes to empty"
WT="$ROOT/.claude/worktrees/verify-$SAFE_BRANCH"
LOG_DIR="$ROOT/.progress/verify/$BRANCH"
MASTER_SHA="$(git rev-parse refs/heads/master 2>/dev/null)" || die "refs/heads/master is unavailable"

# ---- machine sizing: scale with the cores so the 64-core box uses them and a loaded 16-core Mac stays modest
ncpu="$(getconf _NPROCESSORS_ONLN 2>/dev/null)"; [ -n "$ncpu" ] || ncpu="$(sysctl -n hw.ncpu 2>/dev/null)"; [ -n "$ncpu" ] || ncpu=4
load_average() {
  if [ "$(uname -s)" = Darwin ]; then sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}'
  else
    # The box's idle filler (tools/cloud/box/box-filler) runs at the lowest CPU weight and yields to any real work:
    # its cores are not load for sizing purposes. util-sample.sh leaves the filler's core count in this file.
    awk -v f="$(cat /run/legatum-filler-cores 2>/dev/null || echo 0)" '{v = $1 - f; print (v < 0 ? 0 : v)}' /proc/loadavg 2>/dev/null
  fi
}
clamp() { v="$1"; [ "$v" -ge "$2" ] || v="$2"; [ "$v" -le "$3" ] || v="$3"; echo "$v"; }
# On the cloud box, box-job gives every job a share of the cores (LEGATUM_CORE_BUDGET: about 6 at full occupancy, more when
# fewer jobs run) and everything below is sized from that share instead of the whole machine: a verify's lanes run at
# once, so cargo jobs are a third of it, tests use all of it (the other lanes are compiling), Playwright workers a third.
# Twelve verifies each sizing themselves for 64 cores ran the box at load 365 (2026-10-18).
if [ -n "${LEGATUM_CORE_BUDGET:-}" ]; then
  CB="$LEGATUM_CORE_BUDGET"
  JOBS="${LEGATUM_VERIFY_JOBS:-$(clamp $((CB / 3)) 2 32)}"
  TEST_THREADS="${LEGATUM_VERIFY_TEST_THREADS:-$(clamp "$CB" 2 32)}"
  SIM_THREADS="${LEGATUM_VERIFY_SIM_THREADS:-$(clamp $((CB / 4)) 2 8)}"
  PW_WORKERS="${LEGATUM_VERIFY_PW_WORKERS:-$(clamp $((CB / 3)) 1 24)}"
  VITEST_WORKERS="${LEGATUM_VERIFY_VITEST_WORKERS:-$(clamp $((CB / 2)) 2 12)}"
else
JOBS="${LEGATUM_VERIFY_JOBS:-$(clamp $((ncpu / 4)) 4 32)}"
TEST_THREADS="${LEGATUM_VERIFY_TEST_THREADS:-$(clamp $((ncpu / 2)) 4 32)}"
SIM_THREADS="${LEGATUM_VERIFY_SIM_THREADS:-$(clamp $((ncpu / 8)) 4 8)}"
# The screen tour is 89% of Playwright time and every one of its tests is independent, so workers scale with the cores
# (each worker has its own Chromium; the static e2e dist and the relay are shared). On a machine that is already busy the
# count also shrinks with the load: extra workers there only make every test slower (three tour tests hit the
# 45 s limit at load 100 with five workers) and add no throughput.
load0="$(load_average | awk '{printf "%d", $1}')"; [ -n "$load0" ] || load0=0
PW_FREE=$(( (ncpu - load0) / 2 ))
PW_WORKERS="${LEGATUM_VERIFY_PW_WORKERS:-$(clamp $((ncpu / 3 < PW_FREE ? ncpu / 3 : PW_FREE)) 2 24)}"
VITEST_WORKERS="${LEGATUM_VERIFY_VITEST_WORKERS:-$(clamp $((ncpu / 6)) 2 12)}"
fi
# Per-test timeout for tests that set none (the config's 45 s): doubled so a loaded machine does not turn slowness
# into failures. Assertion waits (expect timeouts) are unchanged, and tests that set their own timeout keep it.
export LEGATUM_PW_TIMEOUT="${LEGATUM_VERIFY_PW_TIMEOUT:-90000}"

# ---- what the branch touches (from git refs only, so --dry-run can print it)
RUN_RUST=1; RUN_APP="$DO_APP"; RUN_PW=1; RUN_TOOLS=0; PLAN_NOTE="all steps"; AUTO_APP_NOTE=""
CHANGED_FILES=""
classify() {
  RUN_PW=1
  if [ "$FULL" -eq 1 ]; then PLAN_NOTE="--full: nothing skipped"; RUN_TOOLS=0; return; fi
  if [ "$BRANCH" = master ]; then PLAN_NOTE="master itself: nothing skipped"; return; fi
  base="$(git merge-base "$MASTER_SHA" "$BRANCH_REF" 2>/dev/null)" || { PLAN_NOTE="no merge base: nothing skipped"; return; }
  CHANGED_FILES="$(git diff --name-only "$base" "$BRANCH_REF")"
  [ -n "$CHANGED_FILES" ] || { PLAN_NOTE="empty diff: nothing skipped"; return; }
  c_rust=0; c_app=0; c_pw=0; c_tools=0; c_docs=0; c_other=""
  while IFS= read -r f; do
    case "$f" in
      '') ;;
      docs/sim-results/*.json) c_rust=1 ;;
      docs/*|.claude/*|.progress/*|.reviews/*|plans/*|infra/*|*.md|.gitignore) c_docs=1 ;;
      crates/wasm/*|crates/server/*|crates/core/src/*) c_rust=1; c_pw=1 ;;
      crates/*|tools/sim/*|rust-toolchain*|.cargo/*|.config/*) c_rust=1 ;;
      Cargo.toml|Cargo.lock) c_rust=1; c_pw=1 ;;
      scenarios/*) c_rust=1; c_pw=1 ;;
      app/*|scripts/build-wasm.sh|scripts/*.mjs) c_app=1; c_pw=1 ;;
      tools/*|scripts/*) c_tools=1 ;;
      *) [ -n "$c_other" ] || c_other="$f" ;;
    esac
  done <<EOF
$CHANGED_FILES
EOF
  if [ -n "$c_other" ]; then PLAN_NOTE="unclassified path $c_other: nothing skipped"; return; fi
  RUN_RUST="$c_rust"; RUN_TOOLS="$c_tools"; RUN_PW="$c_pw"
  RUN_APP=0
  if [ "$c_app" -eq 1 ] || { [ "$DO_APP" -eq 1 ] && [ "$c_rust" -eq 1 ]; }; then RUN_APP=1; fi
  if [ "$c_app" -eq 1 ] && [ "$DO_APP" -eq 0 ]; then AUTO_APP_NOTE="app files changed: app lane enabled without --app"; fi
  PLAN_NOTE="selected by changed paths (rust=$c_rust app=$c_app playwright=$c_pw tools=$c_tools docs=$c_docs)"
}
classify

# ---- docs drift: rule code or RULES_VERSION changed but the rules reference didn't (a warning, never a failure;
# the legatum-docs-reviewer agent and the pre-merge docs gate do the real check)
DOCS_WARN=""
docs_drift_check() {
  [ "$BRANCH" != master ] || return 0
  local base files rule_files
  base="$(git merge-base "$MASTER_SHA" "$BRANCH_REF" 2>/dev/null)" || return 0
  files="$(git diff --name-only "$base" "$BRANCH_REF")"
  printf '%s\n' "$files" | grep -q '^.claude/skills/legatum-rules/references/' && return 0
  rule_files="$(printf '%s\n' "$files" | grep -E '^crates/core/src/((data|economy|combat|diplomacy|research|wonders|projects|objectives|goals|scoring|eras|settlement|improvements|resources|army|company|roads|landmarks|orders)(\.rs|/)|rules/)|^crates/core/src/.*faction' \
    | grep -vE '_tests\.rs$|/tests/|\.test\.tsx?$|^app/e2e/|^docs/|\.md$|\.(png|svg|txt)$' | head -3 | tr '\n' ' ' | sed 's/ $//')"
  if git diff "$base" "$BRANCH_REF" -- crates/core/src/lib.rs | grep -qE '^\+pub const RULES_VERSION'; then
    DOCS_WARN="RULES_VERSION bumped${rule_files:+; rule code changed ($rule_files)}"
  elif [ -n "$rule_files" ]; then
    DOCS_WARN="rule code changed ($rule_files)"
  fi
  [ -z "$DOCS_WARN" ] || DOCS_WARN="WARNING docs: $DOCS_WARN but nothing under .claude/skills/legatum-rules/references/ changed; run the legatum-docs-reviewer agent before merging"
}
docs_drift_check

# ---- profile: how much of the app and Rust suites to run (D13, the three test gates)
PW_SMOKE_SPECS="e2e/game.spec.ts e2e/turn-flow.spec.ts e2e/research-click.spec.ts e2e/production-picker.spec.ts e2e/exit-to-title.spec.ts"
PROFILE="${PROFILE_ARG:-${LEGATUM_VERIFY_PROFILE:-}}"; PROFILE_NOTE="chosen with --profile"
if [ -z "$PROFILE" ]; then
  if [ "$RUN_RUST" -eq 0 ]; then PROFILE=fast; PROFILE_NOTE="default: the change touches no Rust"
  else PROFILE=extended; PROFILE_NOTE="default: the change touches Rust"; fi
fi
GATE_SCOPE=extended; [ "$PROFILE" != fast ] || GATE_SCOPE=fast
# Shard modes split one verify over several box slots (tools/cloud/verify-remote.sh --shards).
if [ -n "$PW_SHARD" ]; then
  # Only the Playwright shard and what it needs: the merge, the WASM package, the app build and the release server.
  RUN_RUST=0; RUN_TOOLS=0; RUN_APP=1; RUN_PW=1; DO_DUEL=0
  PLAN_NOTE="Playwright shard $PW_SHARD only"
elif [ "$SKIP_PW" -eq 1 ]; then
  RUN_PW=0; PLAN_NOTE="$PLAN_NOTE; Playwright runs in shard jobs"
fi

pw_specs_note() {
  case "$PROFILE" in
    fast) echo "smoke set (game, turn-flow, research-click, production-picker, exit-to-title)" ;;
    release) echo "every spec including the two-size screen tour" ;;
    *) if [ "${LEGATUM_PW_ALL:-$([ "$ncpu" -ge 32 ] && echo 1 || echo 0)}" = 1 ] || [ -n "${PROFILE_ARG:-${LEGATUM_VERIFY_PROFILE:-}}" ]; then echo "every spec except the screen tour"
       else echo "smoke set plus layout, contrast and lenses (small machine)"; fi ;;
  esac
  [ -z "$PW_SHARD" ] || echo "(shard $PW_SHARD)"
}
# The spec arguments of the profile; the extended list comes from the merged tree, so call it after the merge.
pw_spec_args() {
  case "$PROFILE" in
    fast) echo "$PW_SMOKE_SPECS" ;;
    release) echo "e2e" ;;
    *) if [ "${LEGATUM_PW_ALL:-$([ "$ncpu" -ge 32 ] && echo 1 || echo 0)}" = 1 ] || [ -n "${PROFILE_ARG:-${LEGATUM_VERIFY_PROFILE:-}}" ]; then
         (cd "$WT/app" && ls e2e/*.spec.ts | tr '\n' ' ')
       else echo "$PW_SMOKE_SPECS e2e/layout.spec.ts e2e/contrast.spec.ts e2e/lenses.spec.ts"; fi ;;
  esac
}

print_plan() {
  echo "Verify branch: $BRANCH ($BRANCH_REF)"
  echo "Master snapshot: $MASTER_SHA"
  echo "Worktree: $WT (locked; reset to master before merge)"
  echo "Logs: $LOG_DIR"
  echo "Selection: $PLAN_NOTE${AUTO_APP_NOTE:+; $AUTO_APP_NOTE}"
  [ -z "$DOCS_WARN" ] || echo "$DOCS_WARN"
  echo "Profile: $PROFILE ($PROFILE_NOTE): Rust tests $([ "$GATE_SCOPE" = fast ] && echo 'without the extended tests' || echo 'all'), Playwright $(pw_specs_note)"
  if [ "$RUN_RUST" -eq 1 ]; then
    echo "Steps: merge; fmt; clippy --release; release-checked build; workspace tests (nextest, doctests) --profile release-checked; repin + pin diff; 50-game branch smoke (seed 9100001) and master smoke from the baseline cache"
  else
    echo "Steps: merge; cargo steps SKIPPED (no crates, Cargo, sim-tool or fixture changes)"
  fi
  [ "$DO_DUEL" -eq 0 ] || { [ "$RUN_RUST" -eq 1 ] && echo "Duel: 64-game 2-player small-map proxy7 matrix, seed 9100001, branch and master" || echo "Duel: SKIPPED (no engine changes)"; }
  [ "$RUN_TOOLS" -eq 0 ] || echo "Tools: syntax checks of changed shell and python scripts, plus the verify cache self-test when it changed"
  if [ "$RUN_APP" -eq 1 ]; then
    if [ "$RUN_PW" -eq 1 ]; then echo "App: wasm; vitest; build (includes tsc); server; Chromium $(pw_specs_note | tr '\n' ' ')"
    else echo "App: wasm; vitest (Playwright, app build and server SKIPPED: only ai/sim/analysis/archive crates changed)"; fi
  elif [ "$DO_APP" -eq 1 ]; then echo "App: SKIPPED (no crates or app changes)"; fi
  [ "$NO_CACHE" -eq 0 ] || echo "Cache: disabled (--no-cache)"
  echo "Concurrency: ${ncpu} cpus${LEGATUM_CORE_BUDGET:+ (core budget $LEGATUM_CORE_BUDGET)}; lanes $([ "$SERIAL" -eq 1 ] && echo serial || echo concurrent); cargo jobs $JOBS, test threads $TEST_THREADS, sim threads $SIM_THREADS, playwright workers $PW_WORKERS, vitest workers $VITEST_WORKERS"
}
if [ "$DRY_RUN" -eq 1 ]; then print_plan; exit 0; fi

mkdir -p "$LOG_DIR/steps" "$LOG_DIR/results" "$ROOT/.claude/worktrees" "$ROOT/.progress/verify/.locks" || die "cannot create verifier directories"
GATE_CACHE="$HOME/LegatumRuns/gate-cache"
mkdir -p "$GATE_CACHE/locks" || die "cannot create gate cache: $GATE_CACHE"
# Remove records strictly older than 14 days.
python3 - "$GATE_CACHE" <<'PY'
import pathlib, sys, time
cutoff = time.time() - 14 * 24 * 60 * 60
for path in pathlib.Path(sys.argv[1]).glob('*.pass'):
    try:
        if path.stat().st_mtime < cutoff:
            path.unlink()
    except OSError:
        pass
PY
LOCK_DIR="$ROOT/.progress/verify/.locks/$SAFE_BRANCH"
lock_wait_start=$SECONDS
until lockdir_try "$LOCK_DIR"; do
  if [ "$((SECONDS - lock_wait_start))" -ge 14400 ]; then die "timed out waiting for another verifier at $LOCK_DIR"; fi
  sleep 2
done
# The verifier worktree is locked only while this script runs (stale-worktrees.sh reclaims idle ones; a locked
# one would pile up at 2.6-4 GB each). EXIT also fires after the signal traps below and on every failure path.
cleanup_lock() {
  lockdir_release "$LOCK_DIR"
  [ ! -d "$WT" ] || git worktree unlock "$WT" >/dev/null 2>&1 || true
}
trap cleanup_lock EXIT
trap 'exit 130' HUP INT TERM

export CARGO_BUILD_JOBS="$JOBS" RAYON_NUM_THREADS="$SIM_THREADS"
export LEGATUM_GATE_TEST_THREADS="$TEST_THREADS" LEGATUM_PW_WORKERS="$PW_WORKERS" LEGATUM_VITEST_WORKERS="$VITEST_WORKERS"
export PLAYWRIGHT_JSON_OUTPUT_NAME="$LOG_DIR/playwright.json"
export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
renice 15 -p $$ >/dev/null 2>&1 || true
VERIFY_START=$SECONDS

STATUS_FILE="$LOG_DIR/status.tsv"
FAILED_FILE="$LOG_DIR/failed.txt"
SKIPPED_FILE="$LOG_DIR/skipped.txt"
: > "$STATUS_FILE"; : > "$FAILED_FILE"; : > "$SKIPPED_FILE"
# Step states of an earlier run of this branch would satisfy wait_step before the step has started.
rm -f "$LOG_DIR/steps/"*.status
rm -f "$LOG_DIR/results/"* "$LOG_DIR/playwright.json" "$LOG_DIR/playwright-slowest.txt" "$LOG_DIR/playwright-first.json" "$LOG_DIR/playwright-rerun.json" "$LOG_DIR/playwright-first-run.txt"
printf '%s\n' "$CHANGED_FILES" > "$LOG_DIR/changed-files.txt"

# Background lanes cannot set shell variables for the parent, so everything they report goes through files.
set_result() { printf '%s\n' "$2" > "$LOG_DIR/results/$1"; }
get_result() { if [ -f "$LOG_DIR/results/$1" ]; then cat "$LOG_DIR/results/$1"; else echo "${2:-not run}"; fi; }
mark_failed() { printf '%s\n' "$1" >> "$FAILED_FILE"; }
record() { printf '%s\t%s\t%s\n' "$1" "$2" "$3" >> "$STATUS_FILE"; }
RUSTC_VERSION="$(rustc -V 2>/dev/null || echo unavailable)"
NODE_VERSION="$(node --version 2>/dev/null || echo unavailable)"
key_hash() { if command -v shasum >/dev/null 2>&1; then shasum -a 256; else sha256sum; fi | awk '{print $1}'; }
# The tree every non-master step is keyed by: the merge result, before repin rewrites fixtures.
TREE_MERGED=""
# Master steps are keyed by the sim code (crates, Cargo.toml, Cargo.lock), as tools/sim/cached-baseline.sh is,
# so a master move that only touches docs, the app or tools keeps them cached.
MASTER_CODE_KEY="$( { git rev-parse "$MASTER_SHA:crates" "$MASTER_SHA:Cargo.toml" "$MASTER_SHA:Cargo.lock" 2>/dev/null || echo none; } | key_hash)"
step_tree() {
  case "$1" in
    sim-build-master|smoke-master|duel-build-master|duel-master) printf '%s' "$MASTER_CODE_KEY" ;;
    *) printf '%s' "$TREE_MERGED" ;;
  esac
}
step_command_line() {
  # Stubbed runs (the cache self-test) key every step as `true`, except Playwright, whose spec arguments are the profile.
  if [ "${VERIFY_STEP_STUB:-0}" = 1 ]; then printf 'true'; [ "$1" != playwright_run ] || printf ' %s' "$*"; return; fi
  printf '%q ' "$@"
}
cache_record_matches() {
  [ -f "$1" ] || return 1
  grep -Fqx "key=$2" "$1" && grep -Fqx "tree=$3" "$1" && grep -Fqx "command=$4" "$1" \
    && grep -Fqx "rustc=$RUSTC_VERSION" "$1" && grep -Fqx "node=$NODE_VERSION" "$1" \
    && grep -Fqx "app_lock=$5" "$1"
}
write_cache_record() {
  cache_file_value="$1"; cache_key_value="$2"; cache_name="$3"; cache_tree="$4"; cache_cmd="$5"; app_lock_value="$6"; cache_log="$7"
  cache_tmp="$GATE_CACHE/.$cache_key_value.$$.$RANDOM.tmp"
  artifact_patch=""
  case "$cache_name" in
    repin|repin-merge) artifact_patch="$(git -C "$WT" diff --binary -- crates/sim/tests/fixtures crates/sim/tests/pins.json | base64 | tr -d '\n')" ;;
  esac
  {
    printf 'key=%s\nstep=%s\ntree=%s\ncommand=%s\nrustc=%s\nnode=%s\napp_lock=%s\ntime=%s\nbranch=%s\nlog=%s\n' \
      "$cache_key_value" "$cache_name" "$cache_tree" "$cache_cmd" "$RUSTC_VERSION" "$NODE_VERSION" "$app_lock_value" \
      "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$BRANCH" "$cache_log"
    printf 'artifact_patch=%s\n' "$artifact_patch"
  } > "$cache_tmp" && mv -f "$cache_tmp" "$cache_file_value"
}
restore_cached_artifact() {
  cache_name="$1"; cache_file_value="$2"
  case "$cache_name" in
    repin|repin-merge) ;;
    *) return 0 ;;
  esac
  grep -q '^artifact_patch=' "$cache_file_value" || return 0
  [ -n "$(sed -n 's/^artifact_patch=//p' "$cache_file_value")" ] || return 0
  sed -n 's/^artifact_patch=//p' "$cache_file_value" | python3 -c 'import base64,sys; sys.stdout.buffer.write(base64.b64decode(sys.stdin.read()))' | git -C "$WT" apply -
}
# Timing ledger (tools/ops/timed.py): CPU comes from the `times` builtin's children line, which
# must run in this shell (a pipe or $(...) forks and reads a fresh subshell's counters). A lane runs in its
# own subshell, so its steps read their own counters; the temp files are per step.
step() {
  name="$1"; shift
  log="$LOG_DIR/steps/$name.log"
  : > "$LOG_DIR/steps/$name.status"
  tree="$(step_tree "$name")"
  command_line="$(step_command_line "$@")"
  app_lock="-"
  case "$name" in app-*) app_lock="$(git -C "$WT" rev-parse "HEAD:app/package-lock.json" 2>/dev/null || echo missing)" ;; esac
  key="$(printf '%s\0%s\0%s\0%s\0%s\0%s\0' "$name" "$tree" "$command_line" "$RUSTC_VERSION" "$NODE_VERSION" "$app_lock" | key_hash)"
  cache_file="$GATE_CACHE/$key.pass"
  cache_lock="$GATE_CACHE/locks/$key.lock"
  # Steps whose product is files in THIS worktree (the WASM package and app/dist) never skip on a cache hit: a PASS record
  # made in another slot or worktree leaves this one without them (CL6). The WASM build has its own exact-input cache that
  # restores the package and hash-checks it against its manifest (scripts/build-wasm.sh).
  use_cache="$((1 - NO_CACHE))"
  case "$name" in app-wasm|app-build) use_cache=0 ;; esac
  if [ "$use_cache" -eq 1 ]; then
    if cache_record_matches "$cache_file" "$key" "$tree" "$command_line" "$app_lock"; then
      restore_cached_artifact "$name" "$cache_file" || { mark_failed "$name-cache-restore"; record "$name" FAIL "$log"; printf 'FAIL\n' > "$LOG_DIR/steps/$name.status"; return 0; }
      printf '== %s (cached)\n' "$name" | tee "$log"
      printf 'CACHED\n' > "$LOG_DIR/steps/$name.status"
      load1="$(load_average)"; [ -n "$load1" ] || load1=0
      python3 "$ROOT/tools/ops/timed.py" --record verify "$name" "$BRANCH" "$(printf '%s' "$tree" | cut -c1-12)" 0 0 0 "$load1" || true
      record "$name" CACHED "$log"
      return 0
    fi
    wait_start=$SECONDS
    while ! lockdir_try "$cache_lock"; do
      if cache_record_matches "$cache_file" "$key" "$tree" "$command_line" "$app_lock"; then
        restore_cached_artifact "$name" "$cache_file" || { mark_failed "$name-cache-restore"; record "$name" FAIL "$log"; printf 'FAIL\n' > "$LOG_DIR/steps/$name.status"; return 0; }
        printf '== %s (cached after wait)\n' "$name" | tee "$log"
        printf 'CACHED\n' > "$LOG_DIR/steps/$name.status"
        load1="$(load_average)"; [ -n "$load1" ] || load1=0
        python3 "$ROOT/tools/ops/timed.py" --record verify "$name" "$BRANCH" "$(printf '%s' "$tree" | cut -c1-12)" 0 0 0 "$load1" || true
        record "$name" CACHED "$log"
        return 0
      fi
      if [ "$((SECONDS - wait_start))" -ge 14400 ]; then
        echo "Timed out waiting for concurrent verifier step $name ($key)" | tee "$log"
        record "$name" FAIL "$log"; mark_failed "$name-lock-timeout"
        printf 'FAIL\n' > "$LOG_DIR/steps/$name.status"
        return 0
      fi
      sleep 2
    done
  fi
  printf '== %s\n' "$name" | tee "$log"
  times > "$LOG_DIR/steps/$name.times0"; t0=$SECONDS; load1="$(load_average)"; [ -n "$load1" ] || load1=0
  if [ "${VERIFY_STEP_STUB:-0}" = 1 ]; then
    if [ "${VERIFY_STEP_FAIL_STEP:-}" = "$name" ]; then (cd "$WT" && false) >>"$log" 2>&1
    else
      (cd "$WT" && true) >>"$log" 2>&1
    fi
  else
    (cd "$WT" && "$@") >>"$log" 2>&1
  fi
  rc=$?
  times > "$LOG_DIR/steps/$name.times1"
  cpu="$(awk 'FNR==2 { s=0; for (i=1; i<=2; i++) { split($i, a, "m"); sub("s", "", a[2]); s += a[1]*60 + a[2] } print s }' "$LOG_DIR/steps/$name.times0" "$LOG_DIR/steps/$name.times1" | awk 'NR==1{a=$1} NR==2{printf "%.1f", $1-a}')"
  rm -f "$LOG_DIR/steps/$name.times0" "$LOG_DIR/steps/$name.times1"
  short_tree="$(printf '%s' "$tree" | cut -c1-12)"
  python3 "$ROOT/tools/ops/timed.py" --record verify "$name" "$BRANCH" "$short_tree" "$rc" "$((SECONDS - t0))" "$cpu" "$load1" || true
  if [ "$rc" -eq 0 ]; then state=PASS; else state=FAIL; mark_failed "$name"; fi
  record "$name" "$state" "$log"
  printf '%s\n' "$state" >> "$LOG_DIR/steps/$name.status"
  printf -- '-- %s %s (%ss wall, %ss cpu)\n' "$name" "$state" "$((SECONDS - t0))" "$cpu"
  if [ "$use_cache" -eq 1 ]; then
    if [ "$rc" -eq 0 ]; then
      write_cache_record "$cache_file" "$key" "$name" "$tree" "$command_line" "$app_lock" "$log" \
        || echo "WARN: could not write gate-cache record for $name" | tee -a "$log"
    fi
    lockdir_release "$cache_lock"
  fi
  return 0
}
skip_step() {
  printf '== %s SKIPPED: %s\n' "$1" "$2" | tee "$LOG_DIR/steps/$1.log"
  printf 'SKIPPED\n' > "$LOG_DIR/steps/$1.status"
  record "$1" SKIPPED "$LOG_DIR/steps/$1.log"
  printf '%s: %s\n' "$1" "$2" >> "$SKIPPED_FILE"
}
# step_state <name>: the finished state of a step another lane runs ("" while it has not finished).
step_state() {
  s="$(tail -n 1 "$LOG_DIR/steps/$1.status" 2>/dev/null)"
  case "$s" in PASS|CACHED|FAIL|SKIPPED) printf '%s' "$s" ;; esac
}
# wait_step <name>: block until another lane finishes a step; succeeds when it passed.
wait_step() {
  ws_start=$SECONDS
  while :; do
    s="$(step_state "$1")"
    case "$s" in PASS|CACHED) return 0 ;; FAIL|SKIPPED) return 1 ;; esac
    [ "$((SECONDS - ws_start))" -lt 14400 ] || return 1
    sleep 2
  done
}

# ---- worktree: a registered verifier worktree is the only path this script may reset or clean
WT_REGISTERED=0
if git worktree list --porcelain | awk -v p="$WT" '$1 == "worktree" && $2 == p { found=1 } END { exit !found }'; then WT_REGISTERED=1; fi
if [ "$WT_REGISTERED" -eq 1 ]; then
  [ -d "$WT" ] || die "registered verifier worktree path is missing: $WT"
  # Already locked from a previous run is fine; lock only fails then or on a real error.
  git -C "$WT" worktree lock --reason "astra verify-branch $BRANCH" "$WT" >>"$LOG_DIR/worktree.log" 2>&1 || git worktree list --porcelain | awk -v p="$WT" '$1 == "worktree" { cur = ($2 == p) } cur && $1 == "locked" { f=1 } END { exit !f }' || die "could not lock verifier worktree"
  git -C "$WT" reset --hard "$MASTER_SHA" >>"$LOG_DIR/worktree.log" 2>&1 || die "could not reset verifier worktree to master"
  git -C "$WT" clean -fd >>"$LOG_DIR/worktree.log" 2>&1 || die "could not clean verifier worktree"
else
  [ ! -e "$WT" ] || die "unregistered path already exists; refusing to touch it: $WT"
  git worktree add --detach --lock --reason "astra verify-branch $BRANCH" "$WT" "$MASTER_SHA" >>"$LOG_DIR/worktree.log" 2>&1 || die "could not create locked verifier worktree"
fi

MERGE_LOG="$LOG_DIR/steps/merge.log"
if git -C "$WT" merge --no-commit --no-ff "$BRANCH_REF" >"$MERGE_LOG" 2>&1; then
  git -C "$WT" -c user.name='Branch Verifier' -c user.email='verify@localhost' commit --allow-empty -m "Verify merge $BRANCH" >>"$MERGE_LOG" 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then record merge PASS "$MERGE_LOG"; else mark_failed merge-commit; record merge FAIL "$MERGE_LOG"; fi
elif git -C "$WT" diff --name-only --diff-filter=U > "$LOG_DIR/conflict-files.txt" \
  && ! grep -qvE '^crates/sim/tests/(pins\.json|fixtures/)' "$LOG_DIR/conflict-files.txt"; then
  # Only regenerated pin or fixture files conflict: take master's copy; the repin step below
  # regenerates them on the merged code (never resolve pins by hand).
  while read -r f; do git -C "$WT" checkout --ours -- "$f" && git -C "$WT" add -- "$f"; done < "$LOG_DIR/conflict-files.txt"
  PIN_CONFLICT=1
  echo "Pin/fixture-only conflict resolved to master's copy; repin regenerates: $(tr '\n' ' ' < "$LOG_DIR/conflict-files.txt")" >>"$MERGE_LOG"
  git -C "$WT" -c user.name='Branch Verifier' -c user.email='verify@localhost' commit --allow-empty -m "Verify merge $BRANCH (pins regenerated)" >>"$MERGE_LOG" 2>&1
  rc=$?
  if [ "$rc" -eq 0 ]; then record merge PASS "$MERGE_LOG"; else mark_failed merge-commit; record merge FAIL "$MERGE_LOG"; fi
else
  git -C "$WT" merge --abort >>"$MERGE_LOG" 2>&1 || true
  record merge CONFLICT "$MERGE_LOG"
  echo "CONFLICT: $(tr '\n' ' ' < "$LOG_DIR/conflict-files.txt")"
  print_plan
  echo "VERIFY FAILED: merge"
  exit 1
fi
TREE_MERGED="$(git -C "$WT" rev-parse 'HEAD^{tree}')"
# Heavy cargo steps queue in the machine-wide build pool. Only the outermost command takes a slot: build-slot is
# not re-entrant, so the scripts it runs (repin.sh, build-wasm.sh) must not wrap their own cargo calls in it.
SLOT="tools/sim/build-slot.sh --"
[ -x "$WT/tools/sim/build-slot.sh" ] || SLOT="$ROOT/tools/sim/build-slot.sh --"
GATE_TESTS="tools/astra/gate-tests.sh"
[ -x "$WT/$GATE_TESTS" ] || GATE_TESTS="$ROOT/$GATE_TESTS"
CHECKED_BIN="target/release-checked/legatum-sim"

# ---- the steps' commands (functions run inside the worktree)
build_checked() { $SLOT cargo test --workspace --profile release-checked --no-run; }
smoke() {
  label="$1"; report="$1"; bin="$2"; threads="$3"
  mkdir -p "$LOG_DIR/$report"
  tools/sim/slot.sh --threads "$threads" -- "$bin" \
    --games 50 --seed 9100001 --difficulty hard --map-size small --threads "$threads" \
    --label "verify-$label" --out-dir "$LOG_DIR/$report"
}
# The smoke and duel run a private copy of the binary: later cargo calls in the lane re-link target/.../legatum-sim.
snap_bin() { cp -f "$CHECKED_BIN" "$LOG_DIR/bin-$1" && echo "$LOG_DIR/bin-$1"; }
smoke_branch() { build_checked && smoke branch-smoke "$(snap_bin smoke)" "$SIM_THREADS"; }
duel() {
  label="$1"; bin="$2"; out="$LOG_DIR/duel-$label"; mkdir -p "$out"
  tools/sim/slot.sh --threads "$SIM_THREADS" -- "$bin" --games 64 --seed 9100001 \
    --players 2 --map-size small --difficulty hard --duel-matrix --duel-opponent proxy7 --threads "$SIM_THREADS" \
    --label "verify-duel-$label" --out-dir "$out"
}
duel_branch() { build_checked && duel branch "$(snap_bin duel)"; }
# Master's side comes from the shared baseline cache (keyed by the sim code), at a fixed thread count so every
# machine shares one entry.
BASELINE="$ROOT/tools/sim/cached-baseline.sh"
baseline_build() { "$BASELINE" --commit "$MASTER_SHA" --build-only; }
baseline_copy() { rm -rf "$LOG_DIR/$1"; mkdir -p "$LOG_DIR/$1"; cp -R "$2/." "$LOG_DIR/$1/"; }
smoke_master() {
  dir="$("$BASELINE" --commit "$MASTER_SHA" -- --games 50 --seed 9100001 --difficulty hard --map-size small --threads 4)" || return 1
  baseline_copy master-smoke "$dir"
}
duel_master() {
  dir="$("$BASELINE" --commit "$MASTER_SHA" -- --games 64 --seed 9100001 --players 2 --map-size small --difficulty hard --duel-matrix --duel-opponent proxy7 --threads 4)" || return 1
  baseline_copy duel-master "$dir"
}
tools_check() {
  rc=0
  while IFS= read -r f; do
    [ -f "$f" ] || continue
    case "$f" in
      *.sh) bash -n "$f" || { echo "syntax error: $f"; rc=1; } ;;
      *.py) python3 -c 'import ast,sys; ast.parse(open(sys.argv[1]).read(), sys.argv[1])' "$f" || { echo "syntax error: $f"; rc=1; } ;;
    esac
  done < "$LOG_DIR/changed-files.txt"
  # A change to the verifier or its cache test must pass the cache test; timing tools have their own test.
  if grep -qE '^tools/astra/(verify-branch|test-verify-cache|lockdir)\.sh$' "$LOG_DIR/changed-files.txt" && [ -x tools/astra/test-verify-cache.sh ]; then
    tools/astra/test-verify-cache.sh || rc=1
  fi
  # The cloud box tooling has its own local tests.
  if grep -qE '^tools/cloud/' "$LOG_DIR/changed-files.txt"; then
    for t in tools/cloud/test-box-*.sh tools/cloud/box/tests/*.sh; do [ -f "$t" ] && { bash "$t" || rc=1; }; done
  fi
  if grep -qE '^tools/ops/(timed|timings-report|test_timings_report)\.py$' "$LOG_DIR/changed-files.txt" && [ -f tools/ops/test_timings_report.py ]; then
    python3 tools/ops/test_timings_report.py || rc=1
  fi
  if grep -qE '^tools/ops/(pw-summary|test_pw_summary)\.py$' "$LOG_DIR/changed-files.txt" && [ -f tools/ops/test_pw_summary.py ]; then
    python3 tools/ops/test_pw_summary.py || rc=1
  fi
  if grep -qE '^tools/ops/(pw-workers\.sh|tests/test-pw-workers\.sh)$' "$LOG_DIR/changed-files.txt" && [ -f tools/ops/tests/test-pw-workers.sh ]; then
    bash tools/ops/tests/test-pw-workers.sh >/dev/null || rc=1
  fi
  return "$rc"
}

# ---- reports (parsed from the sim's JSON output)
smoke_report() { # dir -> "N games; panics=..; invalid_orders=..; stalls=.." ; fails on a wrong game count or a panic
  python3 - "$1" <<'PY'
import glob, json, os, sys
if os.environ.get('VERIFY_STEP_STUB') == '1': print('stubbed'); raise SystemExit(0)
files=glob.glob(os.path.join(sys.argv[1], '*.json'))
if not files: print('report missing'); raise SystemExit(1)
r=json.load(open(max(files, key=os.path.getmtime)))
s=r.get('summary', {})
games=r.get('games', [])
invalid=0; found=False
def scan(x):
    global invalid, found
    if isinstance(x, dict):
        for k,v in x.items():
            if 'invalid' in k.lower() and 'order' in k.lower() and isinstance(v,(int,float)): invalid += int(v); found=True
            scan(v)
    elif isinstance(x,list):
        for v in x: scan(v)
scan(r)
panics=s.get('panics', sum(g.get('outcome')=='panic' for g in games))
stalls=s.get('stalled', sum(g.get('outcome')=='stalled' for g in games))
print(f"{len(games)} games; panics={panics}; invalid_orders={invalid if found else 'unreported'}; stalls={stalls}")
if len(games)!=50 or panics: raise SystemExit(1)  # stalls also occur on master: compare them, do not fail
PY
}
duel_report() { # dir
  python3 - "$1" <<'PY'
import glob,json,os,sys
if os.environ.get('VERIFY_STEP_STUB') == '1': print('stubbed'); raise SystemExit(0)
fs=glob.glob(os.path.join(sys.argv[1],'*.json'))
if not fs: print('report missing'); raise SystemExit(1)
r=json.load(open(max(fs,key=os.path.getmtime))); games=r.get('games',[])
wins=sum(1 for i,g in enumerate(games) if (i % 2) in g.get('winner_players', []))
print(f"{len(games)} games; Hard side-A wins={wins}")
if len(games)!=64: raise SystemExit(1)
PY
}
# report_for <result key> <step> <report function> <dir> <failure label>
report_for() {
  if [ "$(tail -n 1 "$LOG_DIR/steps/$2.status")" = CACHED ]; then set_result "$1" cached; return 0; fi
  [ "$(tail -n 1 "$LOG_DIR/steps/$2.status")" = SKIPPED ] && return 0
  out="$("$3" "$4")" || { mark_failed "$5"; out="report failed"; }
  set_result "$1" "$out"
}
playwright_slowest() {
  [ -f "$PLAYWRIGHT_JSON_OUTPUT_NAME" ] || return 0
  python3 - "$PLAYWRIGHT_JSON_OUTPUT_NAME" "$PW_WORKERS" <<'PY' | tee -a "$LOG_DIR/steps/app-playwright.log" > "$LOG_DIR/playwright-slowest.txt"
import json, sys, collections
data = json.load(open(sys.argv[1]))
files = collections.Counter(); tests = []
def walk(suite, file):
    file = suite.get('file') or file
    for spec in suite.get('specs', []):
        for t in spec.get('tests', []):
            d = sum(r.get('duration', 0) for r in t.get('results', []))
            files[spec.get('file') or file] += d
            tests.append((d, f"{spec.get('file') or file}: {spec.get('title','')[:90]}"))
    for s in suite.get('suites', []): walk(s, file)
for s in data.get('suites', []): walk(s, '')
print(f"Playwright slowest specs (summed test time, {sys.argv[2]} workers):")
for f, d in files.most_common(6): print(f"  {f}: {d/1000:.0f}s")
print("Playwright slowest tests:")
for d, n in sorted(tests, reverse=True)[:8]: print(f"  {d/1000:.0f}s {n}")
PY
  # Counts, workers and failures by error class (transport, timeout, assertion, other) of the final report.
  summary_tool="$WT/tools/ops/pw-summary.py"; [ -f "$summary_tool" ] || summary_tool="$ROOT/tools/ops/pw-summary.py"
  python3 "$summary_tool" "$PLAYWRIGHT_JSON_OUTPUT_NAME" --workers "$PW_WORKERS" | grep -E '^pw-summary:|^  \[' | tee -a "$LOG_DIR/steps/app-playwright.log" >> "$LOG_DIR/playwright-slowest.txt" || true
}

# Playwright with one bounded rerun. Runs inside the worktree; args are the spec arguments.
# A failed run whose failures are ALL browser transport errors (tools/ops/pw-summary.py classifies them) reruns only the
# failed tests, once, and says so; an assertion, timeout or unclassified failure is never rerun.
playwright_run() {
  pw_json="$LOG_DIR/playwright.json"
  shard_arg=(); [ -z "$PW_SHARD" ] || shard_arg=(--shard="$PW_SHARD")
  summary_tool="$WT/tools/ops/pw-summary.py"; [ -f "$summary_tool" ] || summary_tool="$ROOT/tools/ops/pw-summary.py"
  pw_cmd() { (cd app && LEGATUM_E2E_BROWSER=chromium npx playwright test --project=game --workers="${LEGATUM_PW_WORKERS:-1}" --timeout="${LEGATUM_PW_TIMEOUT:-45000}" --reporter=list,json ${shard_arg[@]+"${shard_arg[@]}"} "$@"); }
  pw_cmd "$@"; rc=$?
  [ "$rc" -ne 0 ] || return 0
  if [ -f "$summary_tool" ] && python3 "$summary_tool" "$pw_json" --transport-only; then
    echo "playwright: the first run failed with browser transport errors only; rerunning ONLY the failed tests, once (--last-failed)"
    python3 "$summary_tool" "$pw_json" --workers "${LEGATUM_PW_WORKERS:-1}" | sed 's/^/first run: /' | tee "$LOG_DIR/playwright-first-run.txt"
    cp "$pw_json" "$LOG_DIR/playwright-first.json"
    PLAYWRIGHT_JSON_OUTPUT_NAME="$LOG_DIR/playwright-rerun.json" pw_cmd --last-failed "$@"; rc=$?
    if [ "$rc" -eq 0 ]; then echo "playwright: the rerun passed; the first run's failures were transport errors, not assertions" | tee -a "$LOG_DIR/playwright-first-run.txt"
    else echo "playwright: the rerun failed too" | tee -a "$LOG_DIR/playwright-first-run.txt"; cp "$LOG_DIR/playwright-rerun.json" "$pw_json" 2>/dev/null || true; fi
  fi
  return "$rc"
}

# ---- lanes
LANE_PIDS=""
lane() {
  if [ "$SERIAL" -eq 1 ]; then "$@"; return 0; fi
  "$@" &
  LANE_PIDS="$LANE_PIDS $!"
}
join_lanes() { for p in $LANE_PIDS; do wait "$p"; done; LANE_PIDS=""; }

# Branch side, release-checked profile: build once, then the tests and the smoke games share the artifacts. Repin
# rewrites the fixtures the tests read, so it starts only when the tests are done; the smoke runs beside both.
lane_checked() {
  if [ "${PIN_CONFLICT:-0}" = 1 ]; then
    step repin-merge $SLOT env LEGATUM_REPIN_PROFILE=release-checked tools/sim/repin.sh
  fi
  step sim-build-branch build_checked
  smoke_pid=""
  if [ "$SERIAL" -eq 1 ]; then
    step smoke-branch smoke_branch
    report_for smoke-branch smoke-branch smoke_report "$LOG_DIR/branch-smoke" smoke-branch-report
  else
    ( step smoke-branch smoke_branch; report_for smoke-branch smoke-branch smoke_report "$LOG_DIR/branch-smoke" smoke-branch-report ) &
    smoke_pid=$!
  fi
  duel_pid=""
  if [ "$DO_DUEL" -eq 1 ]; then
    if [ "$SERIAL" -eq 1 ]; then
      step duel-branch duel_branch
      report_for duel-branch duel-branch duel_report "$LOG_DIR/duel-branch" duel-branch-report
    else
      ( step duel-branch duel_branch; report_for duel-branch duel-branch duel_report "$LOG_DIR/duel-branch" duel-branch-report ) &
      duel_pid=$!
    fi
  fi

  step test $SLOT env LEGATUM_GATE_SCOPE="$GATE_SCOPE" "$GATE_TESTS" release-checked

  cp "$WT/crates/sim/tests/pins.json" "$LOG_DIR/pins.before.json" 2>/dev/null || true
  step repin $SLOT env LEGATUM_REPIN_PROFILE=release-checked tools/sim/repin.sh
  PINS_STAT="none"; MOVED_PINS="none"
  if [ -f "$LOG_DIR/pins.before.json" ] && [ -f "$WT/crates/sim/tests/pins.json" ]; then
    cp "$WT/crates/sim/tests/pins.json" "$LOG_DIR/pins.after.json"
    PINS_STAT="$(git -C "$WT" diff --no-ext-diff --stat -- crates/sim/tests/pins.json | tr '\n' ' ')"
    [ -n "$PINS_STAT" ] || PINS_STAT="no pins diff"
    MOVED_PINS="$(python3 - "$LOG_DIR/pins.before.json" "$LOG_DIR/pins.after.json" <<'PY'
import json, sys
before, after = (json.load(open(p)) for p in sys.argv[1:])
def flatten(value, path=''):
    if isinstance(value, dict):
        out = {}
        for key, child in value.items(): out.update(flatten(child, f'{path}.{key}' if path else key))
        return out
    if isinstance(value, list):
        out = {}
        for i, child in enumerate(value): out.update(flatten(child, f'{path}[{i}]'))
        return out
    return {path: value}
a, b = flatten(before), flatten(after)
changed = [key for key in sorted(set(a) | set(b)) if a.get(key) != b.get(key)]
print(', '.join(changed) if changed else 'none')
PY
)"
  else
    PINS_STAT="pins file unavailable"; MOVED_PINS="unknown"
  fi
  set_result pins-stat "$PINS_STAT"; set_result pins-moved "$MOVED_PINS"
  # Preserve the exact repinned verification tree (left in the worktree for inspection) without changing the branch.
  git -C "$WT" add crates/sim/tests/fixtures crates/sim/tests/pins.json >>"$LOG_DIR/repin-capture.log" 2>&1
  git -C "$WT" -c user.name='Branch Verifier' -c user.email='verify@localhost' commit --allow-empty -m "Capture repinned verification tree" >>"$LOG_DIR/repin-capture.log" 2>&1 || mark_failed repin-capture
  [ -z "$smoke_pid" ] || wait "$smoke_pid"
  [ -z "$duel_pid" ] || wait "$duel_pid"
}

# Release profile: lints.
lane_release() {
  step fmt cargo fmt --all --check
  step clippy $SLOT cargo clippy --workspace --all-targets --release -- -D warnings
}

# Master side: the baseline binary and results come from the shared cache.
lane_master() {
  step sim-build-master baseline_build
  step smoke-master smoke_master
  report_for smoke-master smoke-master smoke_report "$LOG_DIR/master-smoke" smoke-master-report
  if [ "$DO_DUEL" -eq 1 ]; then
    step duel-build-master baseline_build
    step duel-master duel_master
    report_for duel-master duel-master duel_report "$LOG_DIR/duel-master" duel-master-report
  fi
}

# App: the WASM package feeds tsc, vitest and the app build; Playwright runs against the static e2e web dist built by app-build (relay at 127.0.0.1:8787).
lane_app() {
  # app/src/wasm-pkg and app/dist are git-ignored: build them. node_modules comes from the main checkout.
  if [ ! -e "$WT/app/node_modules" ] && [ -d "$ROOT/app/node_modules" ]; then ln -s "$ROOT/app/node_modules" "$WT/app/node_modules"; fi
  step app-wasm $SLOT scripts/build-wasm.sh
  if ! wait_step app-wasm; then
    for n in app-vitest app-build app-playwright; do skip_step "$n" "app-wasm failed"; done
    return 0
  fi
  vitest_pid=""
  if [ -z "$PW_SHARD" ]; then   # a Playwright shard job leaves the app build and vitest to the main job
    ( step app-vitest bash -c 'cd app && npx vitest run --testTimeout=30000 --maxWorkers=${LEGATUM_VITEST_WORKERS:-2}' ) &
    vitest_pid=$!
  fi
  if [ "$RUN_PW" -eq 1 ]; then
    step app-build env LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm
    if [ "$(step_state app-build)" != FAIL ]; then
      # What runs comes from the profile (fast smoke set, extended: every spec but the tour, release: everything).
      # The specs are command arguments, so they are part of the step's cache key and profiles never share a PASS record.
      read -r -a pw_args <<< "$(cd "$WT" && pw_spec_args)"
      step app-playwright playwright_run "${pw_args[@]}"
      playwright_slowest
    else
      skip_step app-playwright "the e2e web build failed"
    fi
  fi
  [ -z "$vitest_pid" ] || wait "$vitest_pid"
}

# What cannot be affected is skipped and reported, never silently dropped.
if [ "$RUN_RUST" -eq 0 ]; then
  RUST_WHY="no crates, Cargo, sim-tool or fixture changes"; [ -z "$PW_SHARD" ] || RUST_WHY="a Playwright shard job runs only the app chain"
  for n in fmt clippy sim-build-branch test repin smoke-branch sim-build-master smoke-master; do skip_step "$n" "$RUST_WHY"; done
  if [ "$DO_DUEL" -eq 1 ]; then for n in duel-branch duel-build-master duel-master; do skip_step "$n" "$RUST_WHY"; done; fi
fi
if [ "$RUN_APP" -eq 0 ] && [ "$DO_APP" -eq 1 ]; then
  for n in app-wasm app-vitest app-build app-playwright; do skip_step "$n" "no crates or app changes"; done
elif [ "$RUN_APP" -eq 1 ] && [ "$RUN_PW" -eq 0 ]; then
  PW_WHY="only ai/sim/analysis/archive crates changed; wasm, core, app and scenarios are untouched"; [ "$SKIP_PW" -eq 0 ] || PW_WHY="the Playwright shard jobs run the app build and Playwright"
  for n in app-build app-playwright; do skip_step "$n" "$PW_WHY"; done
fi
[ "$RUN_TOOLS" -eq 0 ] || step tools-check tools_check

if [ "$RUN_RUST" -eq 1 ]; then
  lane lane_checked
  lane lane_release
  lane lane_master
fi
[ "$RUN_APP" -eq 0 ] || lane lane_app
join_lanes

PINS_STAT="$(get_result pins-stat none)"; MOVED_PINS="$(get_result pins-moved none)"
SMOKE_BRANCH="$(get_result smoke-branch)"; SMOKE_MASTER="$(get_result smoke-master)"
DUEL_BRANCH="$(get_result duel-branch)"; DUEL_MASTER="$(get_result duel-master)"
FAILED="$(tr '\n' ' ' < "$FAILED_FILE")"

echo ""
echo "VERIFY SUMMARY ($BRANCH)"
echo "Selection: $PLAN_NOTE${AUTO_APP_NOTE:+; $AUTO_APP_NOTE}"
[ -z "$DOCS_WARN" ] || echo "$DOCS_WARN"
echo "Profile: $PROFILE ($PROFILE_NOTE); Rust tests $([ "$GATE_SCOPE" = fast ] && echo 'without the extended tests' || echo all); Playwright $(pw_specs_note | tr '\n' ' ')"
echo "Merge / fmt / clippy / tests / repin: $(awk -F '\t' '$1=="merge"{m=$2}$1=="fmt"{f=$2}$1=="clippy"{c=$2}$1=="test"{t=$2}$1=="repin"{r=$2} END{printf "merge=%s fmt=%s clippy=%s tests=%s repin=%s",m,f,c,t,r}' "$STATUS_FILE")"
echo "Sim builds: branch=$(awk -F '\t' '$1=="sim-build-branch"{v=$2} END{print v}' "$STATUS_FILE") master=$(awk -F '\t' '$1=="sim-build-master"{v=$2} END{print v}' "$STATUS_FILE")"
echo "Smoke: branch=$(awk -F '\t' '$1=="smoke-branch"{v=$2} END{print v}' "$STATUS_FILE") {$SMOKE_BRANCH} | master=$(awk -F '\t' '$1=="smoke-master"{v=$2} END{print v}' "$STATUS_FILE") {$SMOKE_MASTER}"
echo "Pins changed: $PINS_STAT; moved pins: $MOVED_PINS"
[ "$DO_DUEL" -eq 0 ] || echo "Duel: branch=$(awk -F '\t' '$1=="duel-branch"{v=$2} END{print v}' "$STATUS_FILE") {$DUEL_BRANCH} | master=$(awk -F '\t' '$1=="duel-master"{v=$2} END{print v}' "$STATUS_FILE") {$DUEL_MASTER}"
[ "$RUN_TOOLS" -eq 0 ] || echo "Tools: $(awk -F '\t' '$1=="tools-check"{v=$2} END{print v}' "$STATUS_FILE")"
if [ "$DO_APP" -eq 1 ] || [ "$RUN_APP" -eq 1 ]; then echo "App: $(awk -F '\t' '$1 ~ /^app-/{printf "%s=%s ",$1,$2}' "$STATUS_FILE")"; fi
[ ! -s "$LOG_DIR/playwright-slowest.txt" ] || sed 's/^/  /' "$LOG_DIR/playwright-slowest.txt"
[ ! -s "$LOG_DIR/playwright-first-run.txt" ] || sed 's/^/  /' "$LOG_DIR/playwright-first-run.txt"
if [ -s "$SKIPPED_FILE" ]; then echo "Skipped: $(awk -F': ' '{r[$2]=r[$2] (r[$2]?", ":"") $1} END{for (k in r) printf "%s (%s); ", r[k], k}' "$SKIPPED_FILE")"; fi
echo "Wall: $((SECONDS - VERIFY_START))s; $([ "$SERIAL" -eq 1 ] && echo serial || echo concurrent) lanes on $ncpu cpus"
echo "Logs: $LOG_DIR (status.tsv and steps/*.log)"
if [ -n "${FAILED// /}" ]; then
  echo "VERIFY FAILED:$(printf '%s' "$FAILED" | sed 's/ *$//')"
  exit 1
fi
echo "VERIFY PASSED"

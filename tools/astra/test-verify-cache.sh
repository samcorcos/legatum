#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SOURCE_ROOT="$(git -C "$SCRIPT_DIR/../.." rev-parse --show-toplevel)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/verify-cache-test.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
REPO="$TMP/repo"
HOME="$TMP/home"
mkdir -p "$REPO/tools/astra" "$REPO/tools/ops" "$REPO/crates/sim/tests" "$HOME"
cp "$SOURCE_ROOT/tools/astra/verify-branch.sh" "$REPO/tools/astra/verify-branch.sh"
cp "$SOURCE_ROOT/tools/astra/lockdir.sh" "$REPO/tools/astra/lockdir.sh"
cp "$SOURCE_ROOT/tools/ops/timed.py" "$REPO/tools/ops/timed.py"
chmod +x "$REPO/tools/astra/verify-branch.sh"
git -C "$REPO" init -q -b master
git -C "$REPO" config user.name 'Cache Test'
git -C "$REPO" config user.email 'cache-test@localhost'
printf 'pub fn marker() -> u8 { 1 }\n' > "$REPO/lib.rs"
printf '{}\n' > "$REPO/crates/sim/tests/pins.json"
git -C "$REPO" add .
git -C "$REPO" commit -qm 'master baseline'
git -C "$REPO" switch -qc feature

run_verify() {
  (cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB="${VERIFY_STEP_STUB:-1}" bash tools/astra/verify-branch.sh feature --duel "$@")
}
status_file() { printf '%s/.progress/verify/feature/status.tsv' "$REPO"; }
assert_all_steps() {
  expected="$1"
  awk -F '\t' -v want="$expected" '$1 != "merge" && $1 != "" { if ($2 != want) { print "expected " $1 "=" want ", got " $2 > "/dev/stderr"; bad=1 } count++ } END { if (!count || bad) exit 1 }' "$(status_file)"
}
assert_master_steps() {
  expected="$1"
  awk -F '\t' -v want="$expected" '$1 ~ /^(sim-build-master|smoke-master|duel-build-master|duel-master)$/ { if ($2 != want) { print "expected " $1 "=" want ", got " $2 > "/dev/stderr"; bad=1 } count++ } END { if (count != 4 || bad) exit 1 }' "$(status_file)"
}

run_verify
assert_all_steps PASS
echo 'PASS: first stub run executes every gate step'
run_verify
assert_all_steps CACHED
echo 'PASS: second stub run caches every gate step'

printf 'pub fn marker() -> u8 { 2 }\n' > "$REPO/lib.rs"
git -C "$REPO" add lib.rs
git -C "$REPO" commit -qm 'change merged tree'
run_verify
assert_master_steps CACHED
awk -F '\t' '$1 ~ /^(fmt|clippy|test|repin|sim-build-branch|smoke-branch|duel-branch)$/ { count++; if ($2 != "PASS") { print "expected merged step " $1 "=PASS, got " $2 > "/dev/stderr"; bad=1 } } END { if (count != 7 || bad) exit 1 }' "$(status_file)"
echo 'PASS: merged-tree change reruns branch gates and keeps all four master gates cached'

run_verify --no-cache
assert_all_steps PASS
echo 'PASS: --no-cache reruns every gate step'

# Use a fresh merged tree so the injected failure has no earlier PASS for its key.
printf 'pub fn marker() -> u8 { 3 }\n' > "$REPO/lib.rs"
git -C "$REPO" add lib.rs
git -C "$REPO" commit -qm 'fresh tree for failure check'
if (cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB=1 VERIFY_STEP_FAIL_STEP=fmt bash tools/astra/verify-branch.sh feature --duel); then
  echo 'FAIL: injected failure unexpectedly passed' >&2
  exit 1
fi
awk -F '\t' '$1 == "fmt" && $2 != "FAIL" { exit 1 }' "$(status_file)"
run_verify
awk -F '\t' '$1 == "fmt" && $2 != "PASS" { print "failed fmt step was incorrectly cached: " $2 > "/dev/stderr"; exit 1 }' "$(status_file)"
echo 'PASS: failed steps are not cached'

# Affected-only selection: what the branch cannot affect is skipped, with the reason in the summary.
branch_with() { # <branch> <path> : a branch off master that adds one file
  git -C "$REPO" switch -qc "$1" master
  mkdir -p "$REPO/$(dirname "$2")"
  printf '%s\n' "$1" > "$REPO/$2"
  git -C "$REPO" add "$2"
  git -C "$REPO" commit -qm "$1"
  git -C "$REPO" switch -q feature
}
verify_branch() { b="$1"; shift; (cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB=1 bash tools/astra/verify-branch.sh "$b" "$@"); }
states() { # <branch> <step regex> : "step=STATE ..." for matching steps
  awk -F '\t' -v re="$2" '$1 ~ re { printf "%s=%s ", $1, $2 }' "$REPO/.progress/verify/$1/status.tsv"
}
expect_states() { # <branch> <step regex> <state> <count>
  got="$(awk -F '\t' -v re="$2" -v want="$3" '$1 ~ re { n++; if ($2 != want) bad=1 } END { print n+0, bad+0 }' "$REPO/.progress/verify/$1/status.tsv")"
  if [ "$got" != "$4 0" ]; then echo "FAIL: $1 steps /$2/ expected $4 x $3, got: $(states "$1" "$2") (skipped: $(tr '\n' ';' < "$REPO/.progress/verify/$1/skipped.txt"))" >&2; exit 1; fi
}
RUST_STEPS='^(fmt|clippy|test|repin|sim-build-branch|smoke-branch|sim-build-master|smoke-master)$'
RUST_BRANCH_STEPS='^(fmt|clippy|test|repin|sim-build-branch|smoke-branch)$'
APP_STEPS='^(app-wasm|app-vitest|app-build|app-playwright)$'

branch_with docsonly docs/note.md
verify_branch docsonly --app >/dev/null
expect_states docsonly "$RUST_STEPS" SKIPPED 8
expect_states docsonly "$APP_STEPS" SKIPPED 4
echo 'PASS: a docs-only branch skips every build step and still passes'

branch_with apponly app/src/thing.ts
verify_branch apponly >/dev/null
expect_states apponly "$RUST_STEPS" SKIPPED 8
expect_states apponly "$APP_STEPS" PASS 4
app_timing="$(verify_branch apponly)"
case "$app_timing" in *'-- app-build PASS ('*) ;; *) echo "FAIL: app build timing is missing: $app_timing" >&2; exit 1 ;; esac
case "$app_timing" in *app-tsc*) echo 'FAIL: standalone TypeScript step is still present' >&2; exit 1 ;; esac
printf '%s\n' "$app_timing" | grep -E '^(-- app-(wasm|vitest|build|playwright)|App:)'
echo 'PASS: an app-only branch skips the cargo steps and turns the app lane on without --app'

branch_with ailogic crates/ai/src/thing.rs
verify_branch ailogic --app >/dev/null
expect_states ailogic "$RUST_BRANCH_STEPS" PASS 6
expect_states ailogic '^(app-wasm|app-vitest)$' PASS 2
expect_states ailogic '^(app-build|app-playwright)$' SKIPPED 2
echo 'PASS: an ai-only crates branch keeps wasm and vitest but skips the app build and Playwright'

branch_with coreview crates/core/src/thing.rs
verify_branch coreview --app >/dev/null
expect_states coreview "$APP_STEPS" PASS 4
echo 'PASS: a core change runs the full app lane including Playwright'

verify_branch docsonly --app --full >/dev/null
expect_states docsonly "$RUST_BRANCH_STEPS" PASS 6
expect_states docsonly '^(app-wasm|app-vitest|app-build|app-playwright)$' PASS 4
echo 'PASS: --full skips nothing'

plan="$(verify_branch docsonly --dry-run)"
case "$plan" in *'cargo steps SKIPPED'*) ;; *) echo 'FAIL: dry run does not show the skipped cargo steps' >&2; exit 1 ;; esac
echo 'PASS: --dry-run prints the selection'

verify_branch coreview --app --serial >/dev/null
expect_states coreview '^(app-vitest|app-playwright)$' CACHED 2
# WASM and app/dist are worktree files: they are rebuilt, never restored from the gate cache.
expect_states coreview '^(app-wasm|app-build)$' PASS 2
echo 'PASS: --serial runs the same steps; WASM and the web build run'

# Profiles (D13): the default follows what changed, --profile overrides it, and the plan says what runs.
plan="$(verify_branch apponly --dry-run)"
case "$plan" in *'Profile: fast (default: the change touches no Rust)'*'smoke set'*) ;; *) echo "FAIL: an app-only branch should default to the fast profile: $plan" >&2; exit 1 ;; esac
plan="$(verify_branch coreview --dry-run)"
case "$plan" in *'Profile: extended (default: the change touches Rust)'*) ;; *) echo "FAIL: a Rust branch should default to the extended profile: $plan" >&2; exit 1 ;; esac
plan="$(verify_branch apponly --profile release --dry-run)"
case "$plan" in *'Profile: release (chosen with --profile)'*'screen tour'*) ;; *) echo "FAIL: --profile release should run the screen tour: $plan" >&2; exit 1 ;; esac
if verify_branch apponly --profile bogus --dry-run >/dev/null 2>&1; then echo 'FAIL: an unknown profile was accepted' >&2; exit 1; fi
echo 'PASS: profiles default by what changed, --profile overrides, unknown profiles are refused'

# The profile is part of the cache key: a fast PASS must not stand in for an extended or release one.
verify_branch apponly --profile fast >/dev/null
verify_branch apponly --profile release >/dev/null
expect_states apponly '^app-playwright$' PASS 1
verify_branch apponly --profile release >/dev/null
expect_states apponly '^app-playwright$' CACHED 1
echo 'PASS: Playwright PASS records are per profile'

# Shards: --pw-shard runs only the app chain and Playwright; --skip-playwright runs everything else.
verify_branch coreview --app --pw-shard 1/2 >/dev/null
shard_states="$(states coreview '.')"
case "$shard_states" in *app-playwright=*) ;; *) echo "FAIL: a Playwright shard did not run Playwright: $shard_states" >&2; exit 1;; esac
case "$shard_states" in *'test=SKIPPED'*'app-playwright='*|*'app-playwright='*'test=SKIPPED'*) ;; *) echo "FAIL: a Playwright shard ran the Rust tests: $shard_states" >&2; exit 1;; esac
verify_branch coreview --app --skip-playwright >/dev/null
expect_states coreview '^(app-build|app-playwright)$' SKIPPED 2
expect_states coreview '^(app-wasm)$' PASS 1
expect_states coreview '^app-vitest$' CACHED 1
echo 'PASS: --pw-shard and --skip-playwright split one verify into complementary parts'

# The verifier worktree is locked only while a run is in progress (idle locked ones piled up at 2.6-4 GB each).
if git -C "$REPO" worktree list --porcelain | awk '/^worktree .*verify-/ { v = 1; next } /^worktree / { v = 0 } v && /^locked/ { bad = 1 } END { exit !bad }'; then
  echo 'FAIL: a verifier worktree is still locked after the run' >&2; exit 1
fi
echo 'PASS: verifier worktrees are unlocked after the run, passing or failing'

# Dead lock holders (2026-10-18: killed verifies left lock directories that stranded every later verify for four hours).
dead_pid() { bash -c 'echo $$'; }   # a pid that existed a moment ago and is gone
branch_with lockbranch docs/lock-note.md
lockdir="$REPO/.progress/verify/.locks/lockbranch"
mkdir -p "$lockdir"; dead_pid > "$lockdir/pid"
start=$SECONDS
out="$(cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB=1 timeout 60 bash tools/astra/verify-branch.sh lockbranch 2>&1)" || { echo "FAIL: verify did not take over a branch lock held by a dead pid: $out" >&2; exit 1; }
[ $((SECONDS - start)) -lt 30 ] || { echo 'FAIL: dead branch lock took too long to take over' >&2; exit 1; }
case "$out" in *'held by a dead process'*) ;; *) echo 'FAIL: takeover of the dead branch lock was not reported' >&2; exit 1;; esac
[ ! -d "$lockdir" ] || { echo 'FAIL: the branch lock was not released after the run' >&2; exit 1; }
echo 'PASS: a branch lock held by a dead pid is taken over'

# A live holder is waited for, not stolen: this shell holds the lock, so a verify with a short wait must not get it.
sleep 600 & holder=$!
. "$REPO/tools/astra/lockdir.sh"
rm -rf "$lockdir"; mkdir "$lockdir"; lockdir_stamp "$holder" "$lockdir"
if (cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB=1 timeout 8 bash tools/astra/verify-branch.sh lockbranch >/dev/null 2>&1); then
  kill "$holder" 2>/dev/null; echo 'FAIL: verify ran while a live process held the branch lock' >&2; exit 1
fi
[ -d "$lockdir" ] || { kill "$holder" 2>/dev/null; echo 'FAIL: a live holder lost its lock' >&2; exit 1; }
kill "$holder" 2>/dev/null; wait "$holder" 2>/dev/null || true
echo 'PASS: a branch lock held by a live process is waited for'

# A gate-cache step lock left by a dead verify: drop one cached PASS so the step must run, and lock its key.
rm -rf "$lockdir"
printf 'pub fn marker() -> u8 { 4 }\n' > "$REPO/lib.rs"
git -C "$REPO" add lib.rs
git -C "$REPO" commit -qm 'fresh tree for the step lock check'
run_verify >/dev/null
pass_file="$(ls -t "$HOME"/LegatumRuns/gate-cache/*.pass | sed -n 1p)"  # not head -1: under pipefail ls gets SIGPIPE (141) when it is still writing
key="$(basename "$pass_file" .pass)"
rm -f "$pass_file"
mkdir -p "$HOME/LegatumRuns/gate-cache/locks/$key.lock"; dead_pid > "$HOME/LegatumRuns/gate-cache/locks/$key.lock/pid"
(cd "$REPO" && HOME="$HOME" VERIFY_STEP_STUB=1 timeout 60 bash tools/astra/verify-branch.sh feature --duel >/dev/null 2>&1) \
  || { echo 'FAIL: verify did not take over a gate-cache step lock held by a dead pid' >&2; exit 1; }
[ ! -d "$HOME/LegatumRuns/gate-cache/locks/$key.lock" ] || { echo 'FAIL: step lock left behind' >&2; exit 1; }
echo 'PASS: a gate-cache step lock held by a dead pid is taken over'

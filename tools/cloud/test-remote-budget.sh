#!/usr/bin/env bash
# End-to-end client budget test against the box-lib test seam; no AWS or SSH calls are made.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/box-hook.sh" <<'HOOK'
box_up() { :; }
box_sync() { :; }
box_bootstrapped() { return 0; }
box_bucket() { echo stub-bucket; }
box_state() { echo running; }
awsx() { return 0; }
box_ssh_timed() { shift; box_ssh "$@"; }
box_ssh() {
  local last="${!#}"
  case "$last" in
    */data/bin/box-job\ known*) echo "$STUB_SHA" ;;
    */data/bin/box-job\ start\ *) echo started ;;
    *"for ((i=s; i<e; i++))"*) echo 2 ;;
    */data/bin/box-job\ follow\ *)
      while [ ! -e "$STUB_STATE_FILE" ]; do sleep 0.1; done
      echo __BOX_RC=143 ;;
    */data/bin/box-job\ kill\ *) touch "$STUB_STATE_FILE"; echo killed ;;
    *"/data/jobs/running/"*) [ -n "${STUB_ALIVE_FILE:-}" ] && [ -e "$STUB_ALIVE_FILE" ] ;;
    *"cat '/data/bin/VERSION"*) echo stub ;;
    *"mkdir -p /data/jobs/"*) cat >/dev/null ;;
    *) : ;;
  esac
}
HOOK
set +e
STUB_SHA="$(git -C "$root" rev-parse HEAD)" STUB_STATE_FILE="$tmp/remote-done" LEGATUM_BOX_TEST_HOOK="$tmp/box-hook.sh" LEGATUM_BOX_HOME="$tmp/box-home" \
  bash "$here/remote.sh" "$root" --budget 2 -- bash -c 'sleep 30' >"$tmp/out" 2>&1
rc=$?
set -e
cat "$tmp/out"
[ "$rc" -eq 124 ] || { echo "expected timeout exit 124, got $rc" >&2; exit 1; }
grep -q 'budget 2s covers queue plus run' "$tmp/out"
grep -q 'queue position approximately 2' "$tmp/out"
grep -Eq 'timed out after queueing/running.*job .*last known state: queued.*remote.sh --attach' "$tmp/out"
reported_line="$(grep -n 'box: job .* exited 143' "$tmp/out" | cut -d: -f1)"
timeout_line="$(grep -n 'timed out after queueing/running' "$tmp/out" | cut -d: -f1)"
[ -n "$reported_line" ] && [ "$reported_line" -lt "$timeout_line" ] || { echo "client timed out before the remote exit report" >&2; exit 1; }
echo "remote budget stub test passed: timed out with job id, state, queue position and attach command"

# Stall guard: a job whose stream goes silent (box restarted, ssh hung) ends with 124 instead of waiting forever.
set +e
STUB_SHA="$(git -C "$root" rev-parse HEAD)" STUB_STATE_FILE="$tmp/never" LEGATUM_BOX_TEST_HOOK="$tmp/box-hook.sh" LEGATUM_BOX_HOME="$tmp/box-home2" \
  bash "$here/remote.sh" "$root" --budget 600 --stall 3 -- true >"$tmp/out2" 2>&1
rc=$?
set -e
cat "$tmp/out2"
[ "$rc" -eq 124 ] || { echo "expected stall exit 124, got $rc" >&2; exit 1; }
grep -Eq 'printed nothing for 3s.*remote.sh --attach' "$tmp/out2"
echo "remote stall stub test passed"

# A silent job that is still alive is not abandoned: the guard keeps waiting until the budget.
touch "$tmp/alive"
set +e
STUB_SHA="$(git -C "$root" rev-parse HEAD)" STUB_STATE_FILE="$tmp/never" STUB_ALIVE_FILE="$tmp/alive" LEGATUM_BOX_TEST_HOOK="$tmp/box-hook.sh" LEGATUM_BOX_HOME="$tmp/box-home3" \
  bash "$here/remote.sh" "$root" --budget 8 --stall 2 -- true >"$tmp/out3" 2>&1 &
pid3=$!
sleep 5; touch "$tmp/never"; wait $pid3; rc=$?
set -e
cat "$tmp/out3"
grep -q 'still alive on the box; still waiting' "$tmp/out3" || { echo "live silent job was not kept" >&2; exit 1; }
! grep -q 'printed nothing for' "$tmp/out3" || { echo "live silent job was abandoned" >&2; exit 1; }
echo "remote live-silent stub test passed (rc $rc)"

# Hard deadline: a command that hangs (like a Session Manager proxy before the SSH handshake) is killed with 124.
. "$here/box-lib.sh"
start=$(date +%s); set +e; box_deadline 2 sh -c 'sleep 60; sleep 60'; rc=$?; set -e
[ "$rc" -eq 124 ] && [ $(( $(date +%s) - start )) -lt 10 ] || { echo "box_deadline did not kill the hung command (rc $rc)" >&2; exit 1; }
set +e; box_deadline 5 sh -c 'exit 7'; rc=$?; set -e
[ "$rc" -eq 7 ] || { echo "box_deadline lost the exit code (rc $rc)" >&2; exit 1; }
echo "box_deadline test passed"

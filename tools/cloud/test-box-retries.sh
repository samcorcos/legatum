#!/usr/bin/env bash
# Local-only regression checks for startup retries and unavailable exit mapping.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat > "$tmp/ssh" <<'SH'
#!/usr/bin/env bash
n=0
[ ! -f "$SSH_COUNT" ] || n="$(cat "$SSH_COUNT")"
n=$((n + 1))
echo "$n" > "$SSH_COUNT"
[ "$n" -gt "${SSH_FAILS:-0}" ]
SH
chmod +x "$tmp/ssh"
PATH="$tmp:$PATH"
export PATH SSH_COUNT="$tmp/ssh-count"
. "$here/box-lib.sh"
box_id() { echo i-test; }
box_ssh_base() { BOX_SSH=(ssh); }
sleep() { :; }

export LEGATUM_BOX_SSH_WAIT_TRIES=4 LEGATUM_BOX_RETRY_DELAY=0 SSH_FAILS=2
box_wait_ssh 4
[ "$(cat "$SSH_COUNT")" = 3 ] || { echo "expected two failed SSH attempts then success" >&2; exit 1; }

echo 0 > "$SSH_COUNT"
export LEGATUM_BOX_SSH_WAIT_TRIES=3 SSH_FAILS=99
if box_wait_ssh 3; then echo "wait unexpectedly succeeded" >&2; exit 1; fi
[ "$(cat "$SSH_COUNT")" = 3 ] || { echo "SSH retries were not bounded to three attempts" >&2; exit 1; }

mount_attempts=0
box_ssh() {
  mount_attempts=$((mount_attempts + 1))
  cat >/dev/null
  [ "$mount_attempts" -gt 2 ]
}
box_ensure_data
[ "$mount_attempts" = 3 ] || { echo "idempotent mount did not retry twice" >&2; exit 1; }

[ "$(box_classify_job_exit 143 stopped)" = "$BOX_UNAVAILABLE" ]
[ "$(box_classify_job_exit 97 stopping)" = "$BOX_UNAVAILABLE" ]
[ "$(box_classify_job_exit 143 running)" = 143 ]
[ "$(box_classify_job_exit 1 stopped)" = 1 ]
[ "$(box_classify_job_exit 0 stopped)" = 0 ]
echo "box retry stubs passed: fail twice then succeed; bounded exhaustion; unavailable mapping; test failure unchanged"

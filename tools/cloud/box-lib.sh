# shellcheck shell=bash disable=SC2034
# Shared by tools/cloud/box.sh, remote.sh, verify-remote.sh and build-linux.sh (sourced, not run).
# Bash 3.2 compatible (macOS default). Builds on lib.sh (profile, region, awsx, die).
#
#   LEGATUM_BOX_HOME    default ~/.legatum-box (ssh key, instance-id cache, ssh control sockets)
#   LEGATUM_BOX_RESULTS default ~/LegatumRuns/box (per-job output and artifacts)
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

BOX_HOME="${LEGATUM_BOX_HOME:-$HOME/.legatum-box}"
BOX_KEY="$BOX_HOME/id_ed25519"
BOX_RESULTS="${LEGATUM_BOX_RESULTS:-$HOME/LegatumRuns/box}"
BOX_TAG_PROJECT="legatum-box"
BOX_USER="ubuntu"
# Exit status that means "the box could not be used at all" (callers such as build-linux.sh fall back to local).
BOX_UNAVAILABLE=3
export PATH="/opt/homebrew/bin:$PATH"   # session-manager-plugin lives here

box_unavailable() { echo "box: $*" >&2; exit "$BOX_UNAVAILABLE"; }

# box_id: the instance id (cached). Found by tag, so Terraform replacing the instance needs no edits.
box_id() {
  local cache="$BOX_HOME/instance-id" id
  if [ -s "$cache" ]; then cat "$cache"; return; fi
  id="$(awsx ec2 describe-instances \
    --filters "Name=tag:project,Values=$BOX_TAG_PROJECT" "Name=tag:role,Values=build-box" \
              "Name=instance-state-name,Values=pending,running,stopping,stopped" \
    --query 'Reservations[].Instances[].InstanceId' --output text 2>/dev/null | awk '{print $1}')" || true
  [ -n "$id" ] && [ "$id" != None ] || box_unavailable "no build box found (has infra/cloud-box been applied, and are credentials for ${LEGATUM_AWS_PROFILE:-the default AWS profile} valid?)"
  mkdir -p "$BOX_HOME"; echo "$id" > "$cache"
  echo "$id"
}

box_forget_id() { rm -f "$BOX_HOME/instance-id"; }

# box_state: pending|running|stopping|stopped|... (empty when AWS cannot be reached)
box_state() {
  awsx ec2 describe-instances --instance-ids "$(box_id)" --query 'Reservations[0].Instances[0].State.Name' --output text 2>/dev/null || true
}

box_ssh_base() {
  local id="$1"
  local socket_dir="/tmp/lb${UID:-$(id -u)}"
  mkdir -p "$socket_dir" && chmod 700 "$socket_dir" \
    || box_unavailable "cannot prepare SSH control socket directory $socket_dir"
  BOX_SSH=(ssh -i "$BOX_KEY" -o IdentitiesOnly=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null
    -o LogLevel=ERROR -o BatchMode=yes -o ConnectTimeout=10 -o ServerAliveInterval=30 -o ServerAliveCountMax=8
    -o ControlMaster=auto -o "ControlPath=$socket_dir/cm-%C" -o ControlPersist=90
    -o "ProxyCommand=aws ssm start-session --target %h --document-name AWS-StartSSHSession --parameters portNumber=%p ${LEGATUM_AWS_PROFILE:+--profile $LEGATUM_AWS_PROFILE }--region $REGION"
    "$BOX_USER@$id")
}

# box_ssh <remote command...>: run on the box (arguments are joined by ssh, as usual; quote for the remote shell).
box_ssh() {
  box_ssh_base "$(box_id)"
  "${BOX_SSH[@]}" "$@"
}

# box_deadline SECONDS cmd...: run an executable with a hard wall-clock deadline. On expiry its whole process group
# (ssh plus the aws ssm proxy) is killed and the exit code is 124. ConnectTimeout does not cover a Session Manager
# proxy that hangs before the SSH handshake: on 2026-11-08 one hung for six hours when the box restarted mid-upload.
box_deadline() {
  perl -e 'my $s = shift; my $pid = fork; die "fork: $!" unless defined $pid;
    if (!$pid) { setpgrp(0, 0); exec @ARGV or exit 127 }
    $SIG{ALRM} = sub { kill "TERM", -$pid; sleep 2; kill "KILL", -$pid; waitpid($pid, 0); exit 124 };
    alarm $s; waitpid($pid, 0); exit($? & 127 ? 128 + ($? & 127) : $? >> 8)' "$@"
}

# box_ssh_timed SECONDS <remote command...>: box_ssh with a hard deadline (exit 124 on expiry). Use it for every
# short call; only `box-job follow` streams for hours and has its own stall guard in remote.sh.
box_ssh_timed() {
  local secs="$1"; shift
  box_ssh_base "$(box_id)"
  box_deadline "$secs" "${BOX_SSH[@]}" "$@"
}

# box_wait_ssh [attempts]
box_wait_ssh() {
  local tries="${1:-${LEGATUM_BOX_SSH_WAIT_TRIES:-60}}" delay="${LEGATUM_BOX_RETRY_DELAY:-5}" attempt=1
  while [ "$attempt" -le "$tries" ]; do
    box_ssh true 2>/dev/null && return 0
    [ "$attempt" -lt "$tries" ] || break
    echo "box: waiting for SSH ($attempt/$tries)" >&2
    sleep "$delay"
    attempt=$((attempt + 1))
  done
  return 1
}

# Only use this mapping for a job exit that indicates the runner was terminated with its instance.
# A test command's own nonzero exit stays a failure, even if a retry would probably pass.
box_classify_job_exit() {
  local rc="$1" state="$2"
  case "$rc:$state" in
    143:running|97:running) echo "$rc";;
    143:*|97:*) echo "$BOX_UNAVAILABLE";;
    *) echo "$rc";;
  esac
}

# box_up: make sure the box is running and reachable over ssh. Exits BOX_UNAVAILABLE when it cannot be.
box_up() {
  command -v session-manager-plugin >/dev/null || box_unavailable "session-manager-plugin is not installed (brew install --cask session-manager-plugin)"
  [ -f "$BOX_KEY" ] || box_unavailable "no ssh key at $BOX_KEY: run tools/cloud/box.sh init"
  local state id waited=0
  id="$(box_id)"
  state="$(box_state)"
  [ -n "$state" ] && [ "$state" != None ] || { box_forget_id; box_unavailable "cannot read the box state (credentials? ${LEGATUM_AWS_PROFILE:-the default AWS profile})"; }
  while [ "$state" = stopping ] && [ "$waited" -lt 180 ]; do echo "box: waiting for it to finish stopping" >&2; sleep 10; waited=$((waited + 10)); state="$(box_state)"; done
  case "$state" in
    running|pending) ;;
    stopped)
      # Over the daily cap the AWS-side Lambda stops the box within 5 minutes, so starting it only ends in an ssh
      # failure (exit 255). Report it as unavailable so callers fall back to local.
      local cap_row
      cap_row="$(awsx ec2 describe-instances --instance-ids "$id" --query 'Reservations[0].Instances[0].[Tags[?Key==`cap-day`]|[0].Value,Tags[?Key==`cap-used-min`]|[0].Value]' --output text 2>/dev/null || true)"
      if [ "$(echo "$cap_row" | cut -f1)" = "$(date -u +%Y-%m-%d)" ] \
        && awk -v u="$(echo "$cap_row" | cut -f2)" -v c="${LEGATUM_BOX_CAP_MIN:-1440}" 'BEGIN{exit !(u+0 >= c)}'; then
        box_unavailable "daily cap reached ($(echo "$cap_row" | cut -f2) of ${LEGATUM_BOX_CAP_MIN:-1440} minutes today, UTC); it resets at 00:00 UTC"
      fi
      echo "box: starting $id (cold start about 1 to 2 minutes)" >&2
      local err tries=0
      # A just-stopped persistent spot instance's request needs up to a couple of minutes to become startable again
      # (IncorrectSpotRequestState): retry that error only.
      while ! err="$(awsx ec2 start-instances --instance-ids "$id" 2>&1 >/dev/null)"; do
        case "$err" in
          *IncorrectSpotRequestState*) tries=$((tries + 1)); [ "$tries" -le 24 ] || box_unavailable "start failed: the spot request stayed unstartable"; sleep 10;;
          *) box_unavailable "start failed: $(echo "$err" | tr '\n' ' ' | cut -c1-300)";;
        esac
      done
      ;;
    *) box_unavailable "box is in state '$state'" ;;
  esac
  awsx ec2 wait instance-running --instance-ids "$id" 2>/dev/null || box_unavailable "box did not reach running"
  box_wait_ssh "${LEGATUM_BOX_SSH_WAIT_TRIES:-60}" \
    || box_unavailable "box is running but ssh over Session Manager does not answer (try tools/cloud/box.sh shell, or box.sh prepare)"
}

# box_ensure_data: make sure the data volume is mounted at /data (see box/mount-data.sh).
box_ensure_data() {
  local here attempt
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  # mount-data.sh is explicitly idempotent and the attachment can lag instance startup.
  for attempt in 1 2 3; do
    if box_ssh 'sudo bash -s' < "$here/box/mount-data.sh" >&2; then return 0; fi
    [ "$attempt" -lt 3 ] || box_unavailable "the data volume is not mounted on the box after $attempt attempts"
    echo "box: retrying idempotent data-volume mount ($attempt/3)" >&2
    sleep "${LEGATUM_BOX_RETRY_DELAY:-5}"
  done
}

# box_sync: copy tools/cloud/box/* to $BOX_BIN (/data/bin) on the box, FORWARD ONLY.
# The box records the source commit ($BOX_BIN/VERSION: the last commit that touched tools/cloud/box). A caller syncs only when
#   - its tools/cloud/box has no uncommitted edits, and that commit is on master, and
#   - the box's commit is an ancestor of it (so the box only moves forward along master),
# or when the box has no VERSION yet, or LEGATUM_BOX_SYNC_FORCE=1 (testing unmerged box scripts; the next master sync
# replaces them, unless the box's commit is unrelated to master: then force again).
# Box scripts are made immutable after sync so a client with an outdated sync implementation cannot replace newer
# files. This sync lifts immutability, copies, and restores it; the digest check above repairs drift.
BOX_BIN="${LEGATUM_BOX_BIN:-/data/bin}"

# Digest of tools/cloud/box as the box should have it, and of the same files as they are on the box.
box_local_digest() {
  local here; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  ( cd "$here/box" && find . -type f | LC_ALL=C sort | while read -r f; do shasum -a 256 "$f"; done | shasum -a 256 | cut -c1-16 )
}
box_remote_digest() {
  local here files; here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  files="$(cd "$here/box" && find . -type f | LC_ALL=C sort | tr '\n' ' ')"
  box_ssh "cd '$BOX_BIN' && for f in $files; do sha256sum \"\$f\" 2>/dev/null || echo missing \"\$f\"; done | sha256sum | cut -c1-16" 2>/dev/null | tr -d '[:space:]'
}

box_sync() {
  box_ensure_data
  local here root sha remote_sha reason=""
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  root="$(cd "$here/../.." && pwd)"
  sha="$(git -C "$root" log -1 --format=%H -- tools/cloud/box 2>/dev/null || true)"
  remote_sha="$(box_ssh "cat '$BOX_BIN/VERSION' 2>/dev/null" 2>/dev/null | tr -d '[:space:]' || true)"
  [ "$remote_sha" != unknown ] || remote_sha=""
  if [ "${LEGATUM_BOX_SYNC_FORCE:-0}" != 1 ] && [ -n "$remote_sha" ] && [ -n "$sha" ]; then
    if [ -n "$(git -C "$root" status --porcelain -- tools/cloud/box 2>/dev/null)" ]; then
      reason="tools/cloud/box has uncommitted edits"
    elif [ "$sha" = "$remote_sha" ] && [ "$(box_local_digest)" = "$(box_remote_digest)" ]; then
      return 0   # same commit and the files on the box really are those (an old client can overwrite them in place)
    elif ! git -C "$root" merge-base --is-ancestor "$sha" master 2>/dev/null; then
      reason="this checkout's box scripts (${sha:0:9}) are not on master"
    elif ! git -C "$root" cat-file -e "$remote_sha^{commit}" 2>/dev/null \
      || ! git -C "$root" merge-base --is-ancestor "$remote_sha" "$sha" 2>/dev/null; then
      reason="the box already has newer or unrelated scripts (${remote_sha:0:9})"
    fi
    if [ -n "$reason" ]; then
      echo "box: keeping the box scripts at ${remote_sha:0:9}, not syncing: $reason (LEGATUM_BOX_SYNC_FORCE=1 overrides)" >&2
      return 0
    fi
  fi
  # Atomic per file (extract aside, then rename over): box-job is a bash script, and bash reads a running script
  # incrementally. Extracting in place rewrote box-job under the jobs waiting for a slot (2026-10-11: two jobs died
  # with exit 95 after a sync from a checkout with a different box-job). A rename leaves them their old inode.
  { cd "$here/box" && COPYFILE_DISABLE=1 tar czf - .; } | box_ssh "B='$BOX_BIN'; sudo chattr -R -i \"\$B\" 2>/dev/null; mkdir -p \"\$B\" && t=\"\$(mktemp -d \"\$B/.sync.XXXXXX\")\" && tar --warning=no-unknown-keyword -xzf - -C \"\$t\" && chmod +x \"\$t\"/*.sh \"\$t\"/box-job \"\$t\"/shim/* 2>/dev/null; ( cd \"\$t\" && find . -type f ) | while read -r f; do mkdir -p \"\$B/\$(dirname \"\$f\")\" && mv -f \"\$t/\$f\" \"\$B/\$f\"; done; rm -rf \"\$t\"; printf '%s\\n' '${sha:-unknown}' > \"\$B/VERSION.tmp\" && mv -f \"\$B/VERSION.tmp\" \"\$B/VERSION\"; find \"\$B\" -type f ! -name config -exec sudo chattr +i {} + 2>/dev/null; [ ! -x \"\$B/install-util.sh\" ] || sudo bash \"\$B/install-util.sh\" >/dev/null 2>&1; true"
}

# box_bootstrapped: succeeds when bootstrap.sh finished on the box.
box_bootstrapped() { box_ssh 'test -f /data/.bootstrapped' 2>/dev/null; }

# box_bucket: the sims bucket (the box prefix lives in it).
box_bucket() { bucket_name; }

# Test seam: a file that redefines box_up, box_ssh, awsx, ... so the whole pipeline can run against a local container.
[ -z "${LEGATUM_BOX_TEST_HOOK:-}" ] || . "$LEGATUM_BOX_TEST_HOOK"

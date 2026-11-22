#!/usr/bin/env bash
# Run a command on the cloud build box against an exact commit, streaming its output back.
#
#   tools/cloud/remote.sh <worktree-path|ref> [options] -- <command...>
#
# <worktree-path>  a checkout: its HEAD is shipped (add --with-uncommitted for its tracked diff and untracked files)
# <ref>            a branch, tag or sha of THIS repository
# The command runs on the box with cwd = a per-job slot checkout of that commit. Up to 12 jobs run at once, each in
# its own slot with its own warm cargo target/ (seeded from master's by reflink). The command's exit code is
# remote.sh's exit code. Exit 3 means the box could not be used at all (callers may fall back to local).
#
#   --with-uncommitted     also ship `git diff HEAD` and untracked, non-ignored files (worktree targets only)
#   --artifact PATH        copy PATH (relative to the checkout) back; repeatable
#   --artifact-dir DIR     extract artifacts here (default ~/LegatumRuns/box/<job>/artifacts)
#   --extra-ref NAME[=REF] also ship REF (default NAME) as branch NAME on the box (verify needs master and the branch)
#   --env K=V              environment variable for the command (letters, digits, _ in K; no spaces or quotes in V)
#   --budget SECONDS       client budget for queueing plus running (default 21600); expiry leaves job attachable
#   --label TEXT           name shown in `box.sh jobs` and the timing ledger
#   --npm                  npm ci (if the lockfile changed) and Playwright browsers before the command
#   --prefer PATH          prefer a free slot that already has PATH (a warm worktree)
#   --golden               run in the golden slot (what `box.sh warm` uses); one at a time
#   --keep-on-interrupt    Ctrl-C only detaches; the job keeps running (default: Ctrl-C kills it)
#   --stall SECONDS        give up (exit 124, job left attachable) when the job prints nothing for this long
#                          (default 3600, env LEGATUM_BOX_STALL_SECS); every short ssh call also has a hard deadline
#                          (LEGATUM_BOX_SSH_DEADLINE, default 300s), so no phase can hang silently
#   --attach JOB           re-attach to a running job (after a dropped terminal) and fetch its artifacts
#   --kill JOB             stop a job
#
# Do not wrap this script in an outer `timeout`: it hides the job id and final state. Use --budget.
# Output also lands in ~/LegatumRuns/box/<job>/output.log. The box starts on demand and stops itself 20 minutes
# after the last job (or earlier with `tools/cloud/box.sh stop`).
set -euo pipefail
. "$(dirname "$0")/box-lib.sh"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

usage() { sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; }

target=""; with_unc=0; artifacts=(); artifact_dir=""; budget=21600; extra_env=""; label=""; extra_refs=()
npm=0; prefer=""; golden=0; keep=0; attach=""; killjob=""; cmd=()
stall="${LEGATUM_BOX_STALL_SECS:-3600}"; ssh_deadline="${LEGATUM_BOX_SSH_DEADLINE:-300}"
while [ $# -gt 0 ]; do
  case "$1" in
    --with-uncommitted) with_unc=1; shift;;
    --artifact) artifacts+=("${2:?--artifact needs a path}"); shift 2;;
    --artifact-dir) artifact_dir="${2:?}"; shift 2;;
    --extra-ref) extra_refs+=("${2:?}"); shift 2;;
    --env)
      case "${2:-}" in [A-Za-z_]*=*) ;; *) die "--env needs K=V";; esac
      case "$2" in *[[:space:]\'\"\$\`\\]*) die "--env value must not contain spaces, quotes or backslashes";; esac
      extra_env="$extra_env $2"; shift 2;;
    --budget) budget="${2:?}"; shift 2;;
    --label) label="${2:?}"; shift 2;;
    --npm) npm=1; shift;;
    --prefer) prefer="${2:?}"; shift 2;;
    --golden) golden=1; shift;;
    --keep-on-interrupt) keep=1; shift;;
    --stall) stall="${2:?}"; shift 2;;
    --attach) attach="${2:?}"; shift 2;;
    --kill) killjob="${2:?}"; shift 2;;
    -h|--help) usage; exit 0;;
    --) shift; cmd=("$@"); break;;
    -*) die "unknown option $1";;
    *) [ -z "$target" ] || die "unexpected argument $1"; target="$1"; shift;;
  esac
done

if [ -n "$killjob" ]; then box_up; box_sync; box_ssh_timed "$ssh_deadline" "/data/bin/box-job kill $killjob"; exit 0; fi

finish_job() { # finish_job <id>: artifacts, timings, exit code. Uses meta.env saved by the submit step.
  local id="$1" jobres="$BOX_RESULTS/$1"
  # shellcheck disable=SC1090,SC1091
  . "$jobres/meta.env"
  local raw="$jobres/output.raw" rc
  rc="$(sed -n 's/^__BOX_RC=//p' "$raw" | tail -1)"
  if [ -z "$rc" ]; then echo "box: job $id has no exit code yet (still running? box.sh jobs)" >&2; return 75; fi
  if [ "$rc" = 143 ] || [ "$rc" = 97 ]; then
    local state classified
    state="$(box_state)"
    classified="$(box_classify_job_exit "$rc" "$state")"
    if [ "$classified" = "$BOX_UNAVAILABLE" ]; then
      echo "box: job $id was interrupted while the box was '$state' (spot interruption or daily cap); recording unavailable" >&2
      rc="$classified"
    fi
  fi
  if grep -aq '^artifacts uploaded' "$raw"; then
    local dest="${ARTIFACT_DIR:-$jobres/artifacts}" tgz="$jobres/artifacts.tgz"
    mkdir -p "$dest"
    if awsx s3 cp --no-progress "s3://$(box_bucket)/box/jobs/$id/artifacts.tgz" "$tgz" >/dev/null; then
      tar xzf "$tgz" -C "$dest" && echo "box: artifacts in $dest" >&2
      if ls "$dest"/.box-timings/*.tsv >/dev/null 2>&1; then append_timings "$dest/.box-timings"; fi
      rm -rf "$dest/.box-timings"
    else
      echo "box: could not fetch the artifacts" >&2
    fi
  fi
  awsx s3 rm --recursive --only-show-errors "s3://$(box_bucket)/box/jobs/$id/" >/dev/null 2>&1 || true
  local wall=$(( $(date +%s) - START_EPOCH ))
  python3 "$root/tools/ops/timed.py" --record box remote "${LABEL:-job}" "${SHA:0:12}" "$rc" "$wall" 0 0 2>/dev/null || true
  echo "box: job $id exited $rc after ${wall}s" >&2
  return "$rc"
}

# Rows the job's own timed.py runs wrote on the box go into the local ledger as kind "box-<kind>".
append_timings() {
  local dir="$1" ledger="${LEGATUM_TIMINGS_DIR:-$HOME/LegatumRuns/timings}" f out
  mkdir -p "$ledger"
  for f in "$dir"/*.tsv; do
    out="$ledger/$(basename "$f")"
    [ -f "$out" ] || head -1 "$f" > "$out"
    awk -F'\t' -v OFS='\t' 'NR>1{ $2="box-"$2; print }' "$f" >> "$out"
  done
}

follow() { # follow <id>: stream until the job writes its exit code; reconnect on dropped connections
  local id="$1" jobres="$BOX_RESULTS/$1" raw off fails=0 state
  raw="$jobres/output.raw"; touch "$raw"
  while ! grep -aq '^__BOX_RC=' "$raw"; do
    off="$(wc -c < "$raw" | tr -d ' ')"
    box_ssh "/data/bin/box-job follow $id $off" 2>>"$jobres/ssh.err" | tee -a "$raw" | awk '/^__BOX_RC=/{next}{print; fflush()}' || true
    if ! grep -aq '^__BOX_RC=' "$raw"; then
      fails=$((fails + 1))
      state="$(box_state)"
      if [ "$state" != running ]; then
        echo "box: the box is $state while job $id was running (spot interruption or the daily cap): unavailable" >&2
        return "$BOX_UNAVAILABLE"
      fi
      [ "$fails" -le 30 ] || { echo "box: lost the connection too many times; job $id may still be running (remote.sh --attach $id)" >&2; return 75; }
      echo "box: connection dropped, reconnecting ($fails)" >&2
      sleep 5
    fi
  done
  cp "$raw" "$jobres/output.log"
}

if [ -n "$attach" ]; then
  [ -f "$BOX_RESULTS/$attach/meta.env" ] || die "no local record of job $attach"
  box_up
  follow "$attach" || exit $?
  finish_job "$attach"; exit $?
fi

[ -n "$target" ] || { usage >&2; exit 2; }
[ "${#cmd[@]}" -gt 0 ] || die "no command (put it after --)"
case "$budget" in ''|*[!0-9]*|0) die "--budget needs positive seconds";; esac
require_profile
budget_start="$(date +%s)"

# ---- what to ship ------------------------------------------------------------------------------------
if [ -d "$target" ]; then repo="$(cd "$target" && git rev-parse --show-toplevel)"; ref="HEAD"; else repo="$root"; ref="$target"; fi
sha="$(git -C "$repo" rev-parse --verify "$ref^{commit}")" || die "cannot resolve $ref in $repo"
[ "$with_unc" = 0 ] || [ -d "$target" ] || die "--with-uncommitted needs a worktree path"
refs_spec=""; ship_shas="$sha"
for er in ${extra_refs[@]+"${extra_refs[@]}"}; do
  name="${er%%=*}"; lref="${er#*=}"
  esha="$(git -C "$repo" rev-parse --verify "$lref^{commit}")" || die "cannot resolve extra ref $lref"
  refs_spec="$refs_spec $name=$esha"; ship_shas="$ship_shas $esha"
done
[ -n "$label" ] || label="$(basename "$repo"):${ref}"
rust_version="$(cd "$repo" && rustc --version | awk '{print $2}')"

id="$(date -u +%Y%m%dT%H%M%SZ)-$(openssl rand -hex 3)"
jobres="$BOX_RESULTS/$id"; mkdir -p "$jobres"
tmp="$(mktemp -d)"; mkdir -p "$tmp/job"
START_EPOCH="$(date +%s)"
cleanup_tmp() { git -C "$repo" for-each-ref "refs/boxsync/$id-*" --format='%(refname)' | while read -r r; do git -C "$repo" update-ref -d "$r"; done; rm -rf "$tmp"; }
trap cleanup_tmp EXIT

echo "box: job $id ($label) commit ${sha:0:12}$([ "$with_unc" = 1 ] && echo " + uncommitted")" >&2
box_up
box_sync
if ! box_bootstrapped; then
  echo "box: first use, running bootstrap (about 10 minutes, once)" >&2
  "$here/box.sh" prepare >&2 || box_unavailable "bootstrap failed"
  box_sync
fi

# ---- bundle only what the box lacks ------------------------------------------------------------------------
neg=()
while read -r s; do
  [ -n "$s" ] || continue
  git -C "$repo" cat-file -e "$s^{commit}" 2>/dev/null && neg+=("^$s")
done < <(box_ssh_timed "$ssh_deadline" '/data/bin/box-job known')
bargs=(); i=0
for s in $ship_shas; do
  git -C "$repo" update-ref "refs/boxsync/$id-$i" "$s"; bargs+=("refs/boxsync/$id-$i"); i=$((i + 1))
done
has_bundle=0
if git -C "$repo" bundle create "$tmp/in.bundle" "${bargs[@]}" ${neg[@]+"${neg[@]}"} >/dev/null 2>"$tmp/bundle.err"; then
  has_bundle=1
  size="$(du -h "$tmp/in.bundle" | cut -f1)"
  echo "box: uploading $size of git objects" >&2
  awsx s3 cp --no-progress "$tmp/in.bundle" "s3://$(box_bucket)/box/jobs/$id/in.bundle" >/dev/null
elif ! grep -q "empty bundle" "$tmp/bundle.err"; then
  cat "$tmp/bundle.err" >&2; die "git bundle failed"
fi

has_patch=0; has_untracked=0
if [ "$with_unc" = 1 ]; then
  if ! git -C "$repo" diff --quiet HEAD; then git -C "$repo" diff HEAD --binary > "$tmp/job/uncommitted.patch"; has_patch=1; fi
  if [ -n "$(git -C "$repo" ls-files -o --exclude-standard | head -1)" ]; then
    git -C "$repo" ls-files -z -o --exclude-standard | (cd "$repo" && COPYFILE_DISABLE=1 tar czf "$tmp/job/untracked.tgz" --null -T -); has_untracked=1
  fi
  echo "box: shipping $([ $has_patch = 1 ] && echo "a patch ")$([ $has_untracked = 1 ] && echo "untracked files")" >&2
fi

# ---- the job directory ----------------------------------------------------------------------------------------
{
  printf 'SHA=%s\n' "$sha"
  printf 'REFS=%q\n' "${refs_spec# }"
  printf 'LABEL=%q\n' "$label"
  printf 'TIMEOUT=%s\n' "$budget"
  printf 'ARTIFACTS=%q\n' "${artifacts[*]-}"
  printf 'PREFER_PATH=%q\n' "$prefer"
  printf 'WANT_NPM=%s\nGOLDEN=%s\nHAS_BUNDLE=%s\nHAS_PATCH=%s\nHAS_UNTRACKED=%s\n' "$npm" "$golden" "$has_bundle" "$has_patch" "$has_untracked"
  printf 'RUST_VERSION=%s\n' "$rust_version"
  printf 'EXTRA_ENV=%q\n' "${extra_env# }"
} > "$tmp/job/spec.env"
{
  echo '#!/usr/bin/env bash'
  echo 'cd "$LEGATUM_BOX_SLOT_DIR"'
  printf '%q ' "${cmd[@]}"; echo
} > "$tmp/job/cmd.sh"
{
  printf 'LABEL=%q\nSHA=%s\nSTART_EPOCH=%s\nARTIFACT_DIR=%q\n' "$label" "$sha" "$START_EPOCH" "$artifact_dir"
} > "$jobres/meta.env"
( cd "$tmp/job" && COPYFILE_DISABLE=1 tar czf - . ) > "$tmp/job.tgz"
box_ssh_timed "$ssh_deadline" "mkdir -p /data/jobs/$id && tar --warning=no-unknown-keyword -xzf - -C /data/jobs/$id" < "$tmp/job.tgz" \
  || { echo "box: uploading job $id failed or hung past ${ssh_deadline}s (box state: $(box_state 2>/dev/null || echo unknown)); nothing was started" >&2; exit "$BOX_UNAVAILABLE"; }
box_ssh_timed "$ssh_deadline" "/data/bin/box-job start $id" >/dev/null \
  || { echo "box: starting job $id failed or hung past ${ssh_deadline}s (box state: $(box_state 2>/dev/null || echo unknown)); re-run, or check with tools/cloud/remote.sh --attach $id" >&2; exit "$BOX_UNAVAILABLE"; }

queue_count="$(box_ssh_timed 60 'n=0; s=$(cat /data/slots/serving 2>/dev/null || echo 1); e=$(cat /data/slots/next-ticket 2>/dev/null || echo 1); for ((i=s; i<e; i++)); do [ -r /data/slots/ticket.$i ] && n=$((n+1)); done; echo "$n"' 2>/dev/null || echo unknown)"
case "$queue_count" in ''|*[!0-9]*) queue_position=unknown;; *) queue_position="$queue_count"; [ "$queue_position" -gt 0 ] || queue_position="0 (at slot acquisition)";; esac
echo "box: budget ${budget}s covers queue plus run; queue position approximately $queue_position (job $id)" >&2
budget_expired() {
  local elapsed state
  elapsed=$(( $(date +%s) - budget_start ))
  [ "$elapsed" -ge "$budget" ] || return 1
  timeout_state="$(grep -aq '^__BOX_RC=' "$jobres/output.raw" 2>/dev/null && echo finished || { grep -aq 'slot s' "$jobres/output.raw" 2>/dev/null && echo running || echo queued; })"
  box_ssh_timed 60 "/data/bin/box-job kill $id" >/dev/null 2>&1 || true
  return 0
}
timeout_message() { echo "box: timed out after queueing/running (budget ${budget}s); job $id last known state: $timeout_state. Re-attach: tools/cloud/remote.sh --attach $id" >&2; }

interrupted() {
  trap - INT TERM HUP
  local state
  state="$(grep -aq '^__BOX_RC=' "$jobres/output.raw" && echo finished || { grep -aq 'slot s' "$jobres/output.raw" && echo running || echo queued; })"
  if [ "$keep" = 1 ]; then
    echo "box: cancelled; job $id last known state: $state and keeps running. Re-attach: tools/cloud/remote.sh --attach $id; stop it with --kill $id" >&2
  else
    box_ssh_timed 60 "/data/bin/box-job kill $id" >/dev/null 2>&1 || true
    echo "box: cancelled; stopped job $id (last known state: $state). Re-attach: tools/cloud/remote.sh --attach $id" >&2
  fi
  exit 130
}
trap interrupted INT TERM HUP

if budget_expired; then follow "$id" || true; finish_job "$id" || true; timeout_message; exit 124; fi
follow "$id" &
follow_pid=$!
last_size=-1; last_change="$(date +%s)"
while kill -0 "$follow_pid" 2>/dev/null; do
  size="$(wc -c < "$jobres/output.raw" 2>/dev/null | tr -d ' ' || echo 0)"
  if [ "$size" != "$last_size" ]; then last_size="$size"; last_change="$(date +%s)"
  elif [ $(( $(date +%s) - last_change )) -ge "$stall" ]; then
    # Silent for too long. A quiet but live job (queued, or a step that prints only at the end) keeps going; a job
    # whose process is gone, or a box that does not answer, ends the wait. The budget still caps the total.
    if box_ssh_timed 60 "f=/data/jobs/running/$id.pid; [ -f \$f ] && kill -0 \$(cat \$f)" >/dev/null 2>&1; then
      echo "box: job $id silent for ${stall}s but still alive on the box; still waiting" >&2
      last_change="$(date +%s)"; sleep 1; continue
    fi
    kill "$follow_pid" 2>/dev/null || true
    echo "box: job $id printed nothing for ${stall}s (box state: $(box_state 2>/dev/null || echo unknown)); giving up. Re-attach: tools/cloud/remote.sh --attach $id; stop it: --kill $id" >&2
    exit 124
  fi
  if budget_expired; then
    # The already-running follow must receive the remote exit marker before the client returns.
    wait "$follow_pid" || true
    finish_job "$id" || true
    timeout_message
    exit 124
  fi
  sleep 1
done
wait "$follow_pid" || { rc=$?; echo "box: job $id last known state: $(grep -aq '^__BOX_RC=' "$jobres/output.raw" && echo finished || echo running). Re-attach: tools/cloud/remote.sh --attach $id" >&2; exit "$rc"; }
trap - INT TERM HUP
finish_job "$id"

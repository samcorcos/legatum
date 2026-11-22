#!/usr/bin/env bash
# tools/astra/verify-branch.sh on the cloud build box. Same checks, same summary, off the Mac.
#
#   tools/cloud/verify-remote.sh <branch> [--profile fast|extended|release] [--shards N|auto|1] [--duel] [--app] [--no-cache]
#                                [--dry-run] [--webkit] [--budget SECONDS]
#
# Ships master and <branch> as a git bundle (only the objects the box lacks), runs verify-branch.sh in a warm slot
# there, streams its output, and prints the same VERIFY SUMMARY / PASSED / FAILED. Exit code = verify-branch's.
# Logs come back to .progress/verify-remote/<branch>/ (status.tsv and steps/*.log). Step timings go to the local
# ledger (~/LegatumRuns/timings, kinds verify-remote and box-verify; see tools/ops/timings-report.py).
#   --webkit   also run the Playwright WebKit specs (screen tour, exit-to-title) natively on the box, in parallel,
#              against the branch tip (results under ~/LegatumRuns/box/<job>/artifacts/app/test-results).
#   --profile  fast | extended | release (see verify-branch.sh --help; the default follows what the branch changed).
#   --tools-from-branch  run the BRANCH's own tools/astra/verify-branch.sh (the slot is checked out at the branch tip,
#              not at master). For testing changes to the verifier itself before they land; the merge is still onto master.
#   --shards   spread ONE verify over several box slots so it uses many cores at once instead of queuing behind itself:
#              a main job (Rust, WASM, tsc, vitest) plus N Playwright shard jobs (each: WASM from the cache, app build,
#              release server, Playwright --shard K/N). Results merge into one summary; the exit code is the first failure.
#              auto (the default) shards only when the box has the slots free right now (no queue, at least 3 free), up to
#              4 shards, and only when the branch needs Playwright; 1 never shards. Sharding does not add CPU: on a busy
#              box the shards queue like any job, so auto stays at 1 there.
# verify-branch.sh has no macOS-only step.
# The box needs no GitHub or user credentials: code travels as a bundle, over S3 and Session Manager.
# Do not wrap this script in an outer `timeout`: it hides job ids and final status. Use --budget instead.
set -euo pipefail
. "$(dirname "$0")/box-lib.sh"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

branch=""; vargs=(); webkit=0; app=0; budget=28800; shards=auto; tools_from_branch=0
while [ $# -gt 0 ]; do
  case "$1" in
    --webkit) webkit=1; shift;;
    --budget) budget="${2:?}"; shift 2;;
    --app) app=1; vargs+=("$1"); shift;;
    --profile) vargs+=(--profile "${2:?--profile needs fast, extended or release}"); shift 2;;
    --tools-from-branch) tools_from_branch=1; shift;;
    --shards) shards="${2:?--shards needs N, auto or 1}"; shift 2;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    --duel|--no-cache|--dry-run) vargs+=("$1"); shift;;
    -*) die "unknown option $1";;
    *) [ -z "$branch" ] || die "unexpected argument $1"; branch="$1"; shift;;
  esac
done
[ -n "$branch" ] || { echo "usage: verify-remote.sh <branch> [--profile fast|extended|release] [--shards N|auto|1] [--duel] [--app] [--no-cache] [--dry-run] [--webkit] [--budget SECONDS]" >&2; exit 2; }
case "$budget" in ''|*[!0-9]*|0) die "--budget needs positive seconds";; esac
case "$shards" in auto|[1-9]|[1-9][0-9]) ;; *) die "--shards needs a number, auto or 1";; esac
git -C "$root" rev-parse --verify --quiet "refs/heads/$branch^{commit}" >/dev/null \
  || git -C "$root" rev-parse --verify --quiet "refs/remotes/$branch^{commit}" >/dev/null \
  || die "branch not found: $branch"
git -C "$root" rev-parse --verify --quiet refs/heads/master >/dev/null || die "master not found"
branch_ref="$branch"; git -C "$root" rev-parse --verify --quiet "refs/heads/$branch" >/dev/null && branch_ref="refs/heads/$branch"
sha="$(git -C "$root" rev-parse "$branch_ref^{commit}")"

safe="$(printf '%s' "$branch" | tr -cs '[:alnum:]._-' '-')"
out="$root/.progress/verify-remote/$safe"
rm -rf "$out"; mkdir -p "$out"
adir="$(mktemp -d)"
npm_flag=(); [ "$app" = 0 ] || npm_flag=(--npm)
start=$SECONDS
echo "verify-remote: budget ${budget}s covers queue plus run" >&2
budget_left() { local left=$((budget - (SECONDS - start))); [ "$left" -gt 0 ] || left=1; echo "$left"; }

wk_pid=""; wk_log="$out/webkit.log"
if [ "$webkit" = 1 ]; then
  "$here/remote.sh" "$branch_ref" --npm --label "webkit:$branch" --artifact app/test-results --budget "$(budget_left)" \
    -- bash /data/bin/webkit.sh > "$wk_log" 2>&1 &
  wk_pid=$!
  echo "verify-remote: WebKit run started in parallel (log: $wk_log)" >&2
fi

# ---- how many Playwright shards: only when the branch needs Playwright and the box has the slots free right now
nshards=1
if [ "$shards" != 1 ] && [[ " ${vargs[*]-} " != *" --dry-run "* ]]; then
  plan="$(cd "$root" && tools/astra/verify-branch.sh "$branch" --dry-run ${vargs[@]+"${vargs[@]}"} 2>/dev/null || true)"   # not `| grep -q`: pipefail turns its early exit into a failure
  if printf '%s\n' "$plan" | grep -q '^App: wasm; tsc; vitest; build; server;'; then
    if [ "$shards" = auto ]; then
      box_up; box_sync
      free="$(box_ssh 'n=0; N=$(. /data/bin/config; echo ${SLOTS:-12}); for k in $(seq 1 $N); do ( flock -n 8 ) 8>/data/slots/s$k.lock || n=$((n+1)); done; q=$(( $(cat /data/slots/next-ticket 2>/dev/null || echo 1) - $(cat /data/slots/serving 2>/dev/null || echo 1) )); [ "$q" -le 0 ] || { echo 0; exit 0; }; echo $((N - n))' 2>/dev/null || echo 0)"
      case "$free" in ''|*[!0-9]*) free=0;; esac
      # the main job needs one slot too; below three free slots there is nothing to gain over a single job
      if [ "$free" -ge 3 ]; then nshards=$(( free - 1 )); [ "$nshards" -le 4 ] || nshards=4; fi
      echo "verify-remote: $free free slot(s), no queue needed: using $nshards Playwright shard(s)" >&2
    else
      nshards="$shards"
    fi
  fi
fi
if [ "$nshards" -gt 1 ]; then npm_flag=(--npm); fi

slot_ref=master; if [ "$tools_from_branch" = 1 ]; then slot_ref="$branch_ref"; fi
common=(--extra-ref master=master --extra-ref "$branch=$branch_ref" --npm --prefer .claude/worktrees/verify-vrbranch/target)
shard_pids=(); shard_rcs=()
if [ "$nshards" -gt 1 ]; then
  echo "verify-remote: main job plus $nshards Playwright shards in parallel (logs: $out/shard-K.log)" >&2
  for ((k = 1; k <= nshards; k++)); do
    sdir="$(mktemp -d)"
    (
      "$here/remote.sh" "$slot_ref" "${common[@]}" --budget "$(budget_left)" --label "verify:$branch shard $k/$nshards" --artifact .progress/verify-remote/out --artifact-dir "$sdir"         -- bash /data/bin/verify-job.sh "$branch" ${vargs[@]+"${vargs[@]}"} --pw-shard "$k/$nshards" > "$out/shard-$k.log" 2>&1
      src=$?
      mkdir -p "$out/shard-$k"
      [ ! -d "$sdir/.progress/verify-remote/out" ] || cp -a "$sdir/.progress/verify-remote/out/." "$out/shard-$k/"
      rm -rf "$sdir"
      echo "$src" > "$out/shard-$k.rc"
    ) &
    shard_pids+=("$!")
  done
  vargs+=(--skip-playwright)
fi

rc=0
"$here/remote.sh" "$slot_ref" --extra-ref master=master --extra-ref "$branch=$branch_ref" \
  --label "verify:$branch" ${npm_flag[@]+"${npm_flag[@]}"} --artifact .progress/verify-remote/out \
  --prefer .claude/worktrees/verify-vrbranch/target --artifact-dir "$adir" --budget "$(budget_left)" \
  -- bash /data/bin/verify-job.sh "$branch" ${vargs[@]+"${vargs[@]}"} || rc=$?

if [ -d "$adir/.progress/verify-remote/out" ]; then
  cp -a "$adir/.progress/verify-remote/out/." "$out/"
  echo "verify-remote: logs in $out" >&2
fi
rm -rf "$adir"

# ---- merge the shard results into the one summary callers read
if [ "$nshards" -gt 1 ]; then
  for pid in "${shard_pids[@]}"; do wait "$pid" || true; done
  echo "" >&2
  echo "VERIFY SUMMARY ($branch), merged: main job + $nshards Playwright shards" >&2
  echo "main: exit $rc" >&2
  shard_fail=""
  for ((k = 1; k <= nshards; k++)); do
    src="$(cat "$out/shard-$k.rc" 2>/dev/null || echo 99)"
    echo "shard $k/$nshards: exit $src; $(grep -a -h -E '^pw-summary: expected' "$out/shard-$k/playwright-slowest.txt" 2>/dev/null | head -1)" >&2
    grep -a -h -E '^pw-summary: failure classes|^  \[' "$out/shard-$k/playwright-slowest.txt" 2>/dev/null | sed 's/^/  /' >&2 || true
    grep -a -h -E 'transport errors only|the rerun' "$out/shard-$k/playwright-first-run.txt" 2>/dev/null | sed 's/^/  /' >&2 || true
    if [ "$src" -ne 0 ]; then shard_fail="$shard_fail $k"; [ "$rc" -ne 0 ] || rc=$src; fi
  done
  if [ -n "$shard_fail" ]; then echo "VERIFY FAILED: Playwright shard(s)$shard_fail (see $out/shard-K.log)" >&2
  elif [ "$rc" -eq 0 ]; then echo "VERIFY PASSED (merged)" >&2; fi
fi

if [ -n "$wk_pid" ]; then
  wrc=0; wait "$wk_pid" || wrc=$?
  echo "verify-remote: WebKit exit $wrc; tail of $wk_log:" >&2
  tail -n 12 "$wk_log" >&2
  [ "$rc" -ne 0 ] || rc=$wrc
fi

wall=$((SECONDS - start))
python3 "$root/tools/ops/timed.py" --record verify-remote total "$branch" "${sha:0:12}" "$rc" "$wall" 0 0 2>/dev/null || true
echo "verify-remote: $branch finished in ${wall}s with exit $rc" >&2
[ "$rc" -ne "$BOX_UNAVAILABLE" ] || echo "verify-remote: the box is unavailable; run tools/astra/verify-branch.sh $branch locally" >&2
exit "$rc"

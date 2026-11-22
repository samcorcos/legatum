#!/usr/bin/env bash
# Runs ON the box inside a slot (cwd = slot root, checked out at master with the branch's ref present).
#   verify-job.sh <real-branch-name> <verify-branch.sh args...>
# verify-branch.sh builds in a per-branch worktree .claude/worktrees/verify-<branch>, which would start cold for
# every new branch name. Here the branch is aliased to a fixed local name (vrbranch), so a slot keeps ONE warm
# verifier worktree whose incremental target/ is reused by every verify that lands on it. Output names are mapped
# back to the real branch. The verifier worktree is created here first, so its target/ can be seeded with a reflink
# copy of the slot's (golden-seeded) target/ on first use.
set -uo pipefail
# Apply this job's share of the cores. The command-specific defaults cover master ages through verify-branch overrides.
. /data/bin/budget.sh && box_core_budget_apply && box_verify_sizes
real="$1"; shift
alias_name=vrbranch
sha="$(git rev-parse "refs/heads/$real")" || { echo "verify-job: branch $real is not in the slot" >&2; exit 2; }
git branch -f "$alias_name" "$sha"
wt=".claude/worktrees/verify-$alias_name"
if ! git worktree list --porcelain | grep -q "worktree .*/$wt\$"; then
  rm -rf "$wt"
  git worktree prune
  git worktree add -q --detach "$wt" master
  if [ -d target ] && [ ! -d "$wt/target" ]; then echo "seeding $wt/target from the slot target (reflink)"; cp -a --reflink=auto target "$wt/target" || rm -rf "$wt/target"; fi
fi
rm -rf ".progress/verify/$alias_name"
# verify-branch.sh waits up to 4 hours on lock directories and never checks whether the holder is alive, so a verify
# that was killed (box-job kill, a dropped client) strands every later verify. The slot is exclusively ours: any
# per-branch lock under it is stale. The gate-cache step locks are shared by all slots: a lock whose recorded pid is
# dead (or that never recorded one) is stale, swept now and every 20 s while this verify runs.
rm -rf .progress/verify/.locks/*
sweep_gate_locks() {
  local d p
  for d in "$HOME"/LegatumRuns/gate-cache/locks/*.lock; do
    [ -d "$d" ] || continue
    p="$(cut -d' ' -f1 "$d/pid" 2>/dev/null)"   # "<pid> <start time>" since lockdir.sh; older verifiers wrote the pid alone
    if [ -n "$p" ]; then kill -0 "$p" 2>/dev/null || rm -rf "$d"
    elif [ -n "$(find "$d" -maxdepth 0 -mmin +1 2>/dev/null)" ]; then rm -rf "$d"
    fi
  done
}
sweep_gate_locks
( while sleep 20; do sweep_gate_locks; done ) &
sweeper=$!
trap 'kill "$sweeper" 2>/dev/null' EXIT
real_esc="$(printf '%s' "$real" | sed 's/[&|\\]/\\&/g')"
tools/astra/verify-branch.sh "$alias_name" "$@" 2>&1 | sed -u "s|$alias_name|$real_esc|g"
rc=${PIPESTATUS[0]}
mkdir -p ".progress/verify-remote"
rm -rf ".progress/verify-remote/out" && cp -a ".progress/verify/$alias_name" ".progress/verify-remote/out" 2>/dev/null || true
exit "$rc"

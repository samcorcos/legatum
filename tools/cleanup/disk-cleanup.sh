#!/usr/bin/env bash
# Reclaim disk from finished agent/Astra worktrees and rebuildable build output.
# Dry run by default: prints what it would remove and how big it is.
#   tools/cleanup/disk-cleanup.sh            # report only
#   tools/cleanup/disk-cleanup.sh --apply    # remove (user-approved 2026-09-28)
# Only removes things that can be recreated exactly:
#   - worktrees whose branch is fully merged into master, with no modified or untracked
#     files: `git worktree remove --force` (the branch ref stays, so `git worktree add` restores it;
#     --force only lets git delete ignored build output; modified or untracked files are checked first);
#   - Rust build output (target/) of other idle worktrees: rm -rf (rebuilt on the next build).
# The user approved permanent deletion of stale worktrees and build output on 2026-09-28.
# Never touches: master's checkout, locked worktrees (running Claude agents),
# worktrees changed in the last IDLE_HOURS, or any source file.
set -uo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
apply=0; [[ "${1:-}" == "--apply" ]] && apply=1
idle_hours="${IDLE_HOURS:-3}"
wt_dir=".claude/worktrees"
total_kb=0

act() { # $1 = path, $2 = kind (worktree | build-output)
  local kb; kb=$(du -sk "$1" 2>/dev/null | cut -f1); kb=${kb:-0}
  total_kb=$((total_kb + kb))
  printf '%8s MB  %-12s %s\n' $((kb / 1024)) "$2" "$1"
  (( apply )) || return 0
  if [[ "$2" == worktree ]]; then
    git worktree remove --force "$1" || echo "SKIPPED (git refused): $1"
  else
    rm -rf "$1" || echo "SKIPPED (rm failed): $1"
  fi
}

for d in "$wt_dir"/*/; do
  d="${d%/}"; n="$(basename "$d")"
  [[ -f ".git/worktrees/$n/locked" ]] && continue
  if [[ -n "$(find "$d" -maxdepth 3 -mmin "-$((idle_hours * 60))" -not -path '*/target/*' -not -path '*/node_modules/*' -print -quit 2>/dev/null)" ]]; then
    continue
  fi
  br="$(git -C "$d" rev-parse --abbrev-ref HEAD 2>/dev/null || echo '')"
  dirty="$(git -C "$d" status --porcelain 2>/dev/null | wc -l | tr -d ' ')"
  if [[ -n "$br" && "$br" != HEAD && "$dirty" == 0 ]] && git merge-base --is-ancestor "$br" master 2>/dev/null; then
    act "$d" "worktree"
  elif [[ -d "$d/target" && -f "$d/Cargo.toml" ]]; then
    act "$d/target" "build-output"
  fi
done

(( apply )) && git worktree prune
printf 'Total: %s GB %s\n' $((total_kb / 1024 / 1024)) "$( (( apply )) && echo 'removed' || echo 'reclaimable (dry run; pass --apply)')"

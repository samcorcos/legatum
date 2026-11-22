#!/usr/bin/env bash
# Unlock git worktrees whose lock names a process that no longer exists ("... pid N"), e.g. after a
# reboot or an agent crash, so disk-cleanup.sh can reclaim them once merged. Locks without a PID
# (release builds, shared baseline builds) are left alone. User-approved 2026-10-04.
# Usage: unlock-stale-locks.sh [--apply]   (dry run by default)
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
apply=0; [[ "${1:-}" == --apply ]] && apply=1
git worktree list --porcelain | awk '/^worktree /{w=substr($0,10)} /^locked/{print w "\t" substr($0,8)}' |
while IFS=$'\t' read -r wt reason; do
  pid=$(printf '%s' "$reason" | sed -nE 's/.*pid[ =:]*([0-9]+).*/\1/p')
  [[ -z "$pid" ]] && continue
  kill -0 "$pid" 2>/dev/null && continue
  if (( apply )); then
    git worktree unlock "$wt" && echo "$(date '+%Y-%m-%dT%H:%M:%S') unlocked $wt (dead pid $pid)"
  else
    echo "would unlock $wt (dead pid $pid)"
  fi
done

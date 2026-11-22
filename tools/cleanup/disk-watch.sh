#!/usr/bin/env bash
# Disk and load supervisor. Every INTERVAL seconds, appends one line to .progress/disk.log:
# free space, worktree count and size, main target size, load average.
# Below LOW_GB free it logs "DISK LOW" plus the dry-run cleanup total.
# With AUTO_APPLY=1 it also runs the cleanup (only enable if you want unattended cleanup).
set -uo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
interval="${INTERVAL:-1800}"; low_gb="${LOW_GB:-150}"
log=".progress/disk.log"; mkdir -p .progress
while true; do
  free_gb=$(df -g /System/Volumes/Data 2>/dev/null | awk 'NR==2{print $4}')
  wt_n=$(ls .claude/worktrees 2>/dev/null | wc -l | tr -d ' ')
  wt_gb=$(du -sg .claude/worktrees 2>/dev/null | cut -f1)
  tgt_gb=$(du -sg target 2>/dev/null | cut -f1)
  load=$(sysctl -n vm.loadavg | awk '{print $2}')
  echo "$(date '+%Y-%m-%dT%H:%M:%S') free=${free_gb}G worktrees=${wt_n} (${wt_gb}G) target=${tgt_gb}G load=${load}" >> "$log"
  # Every interval: remove merged, clean, idle, unlocked worktrees so stale agent workspaces never
  # pile up (user request 2026-10-03). The low-disk branch below also clears build output.
  # Locks naming a dead PID (reboot, crashed agent) block cleanup; clear them first (user OK 2026-10-04).
  tools/cleanup/unlock-stale-locks.sh --apply >> "$log" 2>&1
  tools/cleanup/disk-cleanup.sh --apply >> "$log" 2>&1
  if (( ${free_gb:-0} < low_gb )); then
    summary=$(tools/cleanup/disk-cleanup.sh | tail -n 1)
    echo "$(date '+%Y-%m-%dT%H:%M:%S') DISK LOW: ${free_gb}G free (< ${low_gb}G). ${summary}" >> "$log"
    [[ "${AUTO_APPLY:-0}" == 1 ]] && tools/cleanup/disk-cleanup.sh --apply >> "$log" 2>&1
  fi
  sleep "$interval"
done

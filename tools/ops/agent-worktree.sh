#!/usr/bin/env bash
# Reuse warm Claude worktrees from .claude/worktrees/pool-1 through pool-6.
set -u

repo_root="$(cd "$(git rev-parse --git-common-dir)/.." 2>/dev/null && pwd -P)" || {
  echo "agent-worktree: cannot locate repository root" >&2
  exit 1
}
pool_root="$repo_root/.claude/worktrees"
max_pools=6
wait_seconds=1200
poll_seconds=30

usage() {
  echo "usage: $0 acquire <branch> [--base master] | release <path> | status" >&2
  exit 2
}

pool_number() {
  case "$1" in
    pool-[1-6]) printf '%s\n' "${1#pool-}" ;;
    *) return 1 ;;
  esac
}

pid_alive() {
  [ -n "$1" ] && case "$1" in *[!0-9]*) return 1 ;; esac
  kill -0 "$1" 2>/dev/null
}

read_lock() {
  lock_pid=""; lock_branch=""; lock_time=""
  if [ -f "$1" ]; then
    IFS=' ' read -r lock_pid lock_branch lock_time < "$1" || true
  fi
}

acquire_guard() {
  # mkdir makes the claim atomic across simultaneous acquire processes.
  guard="$pool_root/.agent-worktree-acquire"
  start="$(date +%s)"
  while ! mkdir "$guard" 2>/dev/null; do
    owner=""
    [ -f "$guard/pid" ] && read -r owner < "$guard/pid"
    if [ -n "$owner" ] && ! pid_alive "$owner"; then
      rm -rf "$guard"
      continue
    fi
    now="$(date +%s)"
    if [ $((now - start)) -ge 120 ]; then
      echo "agent-worktree: timed out waiting for acquisition lock" >&2
      return 1
    fi
    sleep 1
  done
  printf '%s\n' "$$" > "$guard/pid"
  trap 'rm -rf "$guard"' EXIT HUP INT TERM
}

release_guard() {
  rm -rf "$guard"
  trap - EXIT HUP INT TERM
}

age_text() {
  local then="$1" now delta
  case "$then" in ''|*[!0-9]*) echo "unknown"; return;; esac
  now="$(date +%s)"; delta=$((now - then)); [ "$delta" -lt 0 ] && delta=0
  if [ "$delta" -lt 3600 ]; then echo "$((delta / 60))m"
  elif [ "$delta" -lt 86400 ]; then echo "$((delta / 3600))h"
  else echo "$((delta / 86400))d"; fi
}

status() {
  printf '%-8s %-7s %-28s %s\n' "POOL" "STATE" "BRANCH" "AGE"
  n=1
  while [ "$n" -le "$max_pools" ]; do
    wt="$pool_root/pool-$n"; lock="$pool_root/pool-$n.busy"
    [ -d "$wt" ] || { n=$((n + 1)); continue; }
    branch="$(git -C "$wt" branch --show-current 2>/dev/null || echo detached)"
    read_lock "$lock"
    state=idle; age="-"
    if [ -f "$lock" ] && pid_alive "$lock_pid"; then
      state="busy:$lock_pid"; age="$(age_text "$lock_time")"
      [ -n "$lock_branch" ] && branch="$lock_branch"
    elif [ -f "$lock" ]; then
      state="stale-lock"; age="$(age_text "$lock_time")"
    fi
    printf '%-8s %-7s %-28s %s\n' "pool-$n" "$state" "${branch:--}" "$age"
    n=$((n + 1))
  done
}

acquire() {
  branch="$1"; shift
  case "$branch" in ''|-*|*[!A-Za-z0-9._/-]*) echo "agent-worktree: invalid branch '$branch'" >&2; exit 2;; esac
  base=master
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --base) [ "$#" -ge 2 ] || usage; base="$2"; shift 2;;
      *) usage;;
    esac
  done
  case "$base" in ''|-*) echo "agent-worktree: invalid base '$base'" >&2; exit 2;; esac
  mkdir -p "$pool_root"
  started="$(date +%s)"
  while :; do
    acquire_guard || exit 1
    chosen=""
    n=1
    while [ "$n" -le "$max_pools" ]; do
      wt="$pool_root/pool-$n"; lock="$pool_root/pool-$n.busy"
      if [ -f "$lock" ]; then
        read_lock "$lock"
        # Locks end only by `release` or after 12 hours. The stored PID belongs to a one-shot
        # shell that exits at once, so PID liveness must never free a slot (2026-10-09: it wiped
        # two agents' uncommitted work).
        if [ $(( $(date +%s) - ${lock_time:-0} )) -gt 43200 ]; then rm -f "$lock"; fi
      fi
      # Never take a slot whose worktree has uncommitted changes: reset/clean would destroy them.
      if [ ! -e "$lock" ] && [ -d "$wt" ] && [ -n "$(git -C "$wt" status --porcelain 2>/dev/null)" ]; then
        echo "agent-worktree: pool-$n has uncommitted changes; skipping it" >&2
        n=$((n + 1)); continue
      fi
      if [ ! -e "$lock" ]; then
        if [ ! -d "$wt" ]; then
          if ! git -C "$repo_root" worktree add "$wt" "$base" >&2; then
            release_guard; exit 1
          fi
        fi
        chosen="$wt"; break
      fi
      n=$((n + 1))
    done
    if [ -n "$chosen" ]; then
      lock="$chosen.busy"
      # Store the caller (agent process), not this short-lived helper process.
      printf '%s %s %s\n' "$PPID" "$branch" "$(date +%s)" > "$lock"
      git -C "$chosen" reset --hard >&2 &&
        git -C "$chosen" clean -fd -e target >&2 &&
        if git -C "$repo_root" show-ref --verify --quiet "refs/heads/$branch"; then
          branch_path="$(git -C "$repo_root" worktree list --porcelain | awk -v b="$branch" '$1 == "branch" && $2 == "refs/heads/" b { if (path) print path; path = prev } $1 == "worktree" { prev = $2 }')"
          if [ -n "$branch_path" ] && [ "$(cd "$branch_path" && pwd -P)" != "$(cd "$chosen" && pwd -P)" ]; then
            echo "agent-worktree: branch '$branch' is checked out at $branch_path" >&2
            result=1
          else
            git -C "$chosen" checkout "$branch" >&2
          fi
        else
          git -C "$chosen" checkout -b "$branch" "$base" >&2
        fi
      result=$?
      if [ "$result" -ne 0 ]; then rm -f "$lock"; release_guard; exit "$result"; fi
      abs="$(cd "$chosen" && pwd -P)"
      release_guard
      printf '%s\n' "$abs"
      return 0
    fi
    release_guard
    now="$(date +%s)"
    if [ $((now - started)) -ge "$wait_seconds" ]; then
      echo "agent-worktree: all $max_pools pools stayed busy for 20 minutes" >&2
      return 1
    fi
    sleep "$poll_seconds"
  done
}

release() {
  [ "$#" -eq 1 ] || usage
  path="$1"
  case "$path" in /*) ;; *) path="$PWD/$path";; esac
  parent="$(cd "$(dirname "$path")" 2>/dev/null && pwd -P)" || { echo "agent-worktree: path not found: $path" >&2; return 1; }
  name="$(basename "$path")"
  number="$(pool_number "$name")" || { echo "agent-worktree: release only accepts pool-1 through pool-6" >&2; return 2; }
  [ "$parent" = "$(cd "$pool_root" && pwd -P)" ] || { echo "agent-worktree: path is outside $pool_root" >&2; return 2; }
  lock="$pool_root/pool-$number.busy"
  [ -f "$lock" ] || { echo "agent-worktree: pool-$number is already idle" >&2; return 1; }
  read_lock "$lock"
  if [ "$lock_pid" != "$PPID" ] && pid_alive "$lock_pid"; then
    echo "agent-worktree: pool-$number is owned by pid $lock_pid ($lock_branch)" >&2
    return 1
  fi
  rm -f "$lock"
  echo "released $path"
}

[ "$#" -ge 1 ] || usage
command="$1"; shift
case "$command" in
  acquire) [ "$#" -ge 1 ] || usage; acquire "$@";;
  release) release "$@";;
  status) [ "$#" -eq 0 ] || usage; status;;
  *) usage;;
esac

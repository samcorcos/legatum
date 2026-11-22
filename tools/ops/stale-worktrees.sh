#!/usr/bin/env bash
# Remove old, unused non-lane worktrees and optionally clean idle lane caches.
set -euo pipefail

apply=0
lanes=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --apply) apply=1 ;;
    --lanes) lanes=1 ;;
    *) echo "usage: $0 [--apply] [--lanes]" >&2; exit 2 ;;
  esac
  shift
done

repo_root="$(cd "$(git rev-parse --git-common-dir)/.." 2>/dev/null && pwd -P)" || {
  echo "stale-worktrees: cannot locate repository root" >&2; exit 1;
}
pool_root="$repo_root/.claude/worktrees"
[ -d "$pool_root" ] || { echo "stale-worktrees: no worktree directory"; exit 0; }
now=$(date +%s)
cutoff=$((now - 43200))
lane_cutoff=$((now - 259200))
patch_dir="$HOME/LegatumData/worktree-patches-$(date +%F)"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/stale-worktrees.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
trap 'exit 130' HUP INT TERM
ps -Ao pid=,command= > "$tmp/ps" 2>/dev/null || :
lsof -Fn -d cwd 2>/dev/null | awk '/^p/ { pid=substr($0,2) } /^n/ { print pid "\t" substr($0,2) }' > "$tmp/cwd" || :

is_used() {
  local path=$1 pid command cwd
  while read -r pid command; do
    case "$command" in *"$path"*) return 0;; esac
  done < "$tmp/ps"
  while IFS=$'\t' read -r pid cwd; do
    case "$cwd" in "$path"|"$path"/*) return 0;; esac
  done < "$tmp/cwd"
  return 1
}

is_running_task() {
  local branch=$1 f task_branch
  [ -n "$branch" ] || return 1
  for f in "$repo_root"/.progress/astra-queue/running/*.md; do
    [ -f "$f" ] || continue
    task_branch=$(grep -Eo 'astra/[A-Za-z0-9._/-]+' "$f" | head -1 || :)
    [ "$task_branch" = "$branch" ] && return 0
  done
  return 1
}

is_lane_in_running_task() {
  local lane_name=$1 f
  for f in "$repo_root"/.progress/astra-queue/running/*.md; do
    [ -f "$f" ] || continue
    grep -Fq "$lane_name" "$f" && return 0
  done
  return 1
}

lock_is_live() {
  local reason=$1 pid
  case "$reason" in "claude agent"*) ;;
    *) return 0;;
  esac
  pid=$(printf '%s\n' "$reason" | sed -nE 's/.*\(pid ([0-9]+).*/\1/p')
  [ -n "$pid" ] || return 1
  kill -0 "$pid" 2>/dev/null
}

format_size() { awk -v b="$1" 'BEGIN { if (b < 1024) printf "%d B", b; else if (b < 1048576) printf "%.1f KB", b/1024; else if (b < 1073741824) printf "%.1f MB", b/1048576; else printf "%.1f GB", b/1073741824 }'; }
freed=0
printf '%-30s %-10s %-12s %s\n' PATH SIZE ACTION

# Registered worktrees, retaining lock metadata exactly as reported by git.
git -C "$repo_root" worktree list --porcelain > "$tmp/list"
printf 'worktree \n' >> "$tmp/list"
wt=""; branch=""; locked=0; reason=""
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    'worktree '*)
      if [ -n "$wt" ]; then
        name=${wt##*/}
        case "$wt" in "$pool_root"/lane-*) :;; "$pool_root"/*)
          if [ -d "$wt" ]; then
            bytes=$(du -sk "$wt" 2>/dev/null | awk '{print $1*1024}'); bytes=${bytes:-0}
            mt=$(find "$wt" -type f -not -path '*/.git/*' -exec stat -f %m {} + 2>/dev/null | sort -nr | head -1); mt=${mt:-0}
            action=keep
            if [ "$mt" -lt "$cutoff" ] && ! is_used "$wt" && ! is_running_task "$branch" && ! { [ "$locked" -eq 1 ] && lock_is_live "$reason"; }; then
              action=remove
              if [ "$apply" -eq 1 ]; then
                if [ -n "$(git -C "$wt" status --porcelain --untracked-files=all 2>/dev/null)" ]; then
                  mkdir -p "$patch_dir"
                  patch="$patch_dir/$name.patch"
                  { printf 'HEAD %s BRANCH %s\n' "$(git -C "$wt" rev-parse HEAD)" "${branch:-detached}"; git -C "$wt" diff HEAD -- . ':(exclude)**/*.png' ':(exclude)*.png'; printf '\nUntracked files:\n'; git -C "$wt" ls-files --others --exclude-standard; } > "$patch"
                fi
                if git -C "$repo_root" worktree remove -f -f "$wt"; then freed=$((freed + bytes)); action=removed; else action=remove-failed; fi
              fi
            fi
            printf '%-30s %-10s %s\n' "$name" "$(format_size "$bytes")" "$action"
          fi;; esac
      fi
      wt=${line#worktree }; branch=""; locked=0; reason="";;
    'branch refs/heads/'*) branch=${line#branch refs/heads/};;
    locked) locked=1; reason=locked;;
    'locked '*) locked=1; reason=${line#locked };;
  esac
done < "$tmp/list"
# Orphan directories that git does not know about are disposable after the same activity check.
while IFS= read -r -d '' dir; do
  [ -d "$dir" ] || continue
  is_registered=0
  while IFS= read -r line; do [ "$line" = "worktree $dir" ] && is_registered=1; done < "$tmp/list"
  [ "$is_registered" -eq 0 ] || continue
  if ! is_used "$dir"; then
    bytes=$(du -sk "$dir" 2>/dev/null | awk '{print $1*1024}'); bytes=${bytes:-0}
    action=remove
    if [ "$apply" -eq 1 ]; then rm -rf "$dir"; freed=$((freed + bytes)); action=removed; fi
    printf '%-30s %-10s %s\n' "${dir##*/}" "$(format_size "$bytes")" "$action"
  fi
done < <(find "$pool_root" -mindepth 1 -maxdepth 1 -type d -print0)

if [ "$lanes" -eq 1 ]; then
  while IFS= read -r -d '' lane; do
    is_used "$lane" && continue
    { is_running_task "$(git -C "$lane" branch --show-current 2>/dev/null)" || is_lane_in_running_task "${lane##*/}"; } && continue
    newest=$(find "$lane" -type f -not -path '*/target/*' -not -path '*/.git/*' -exec stat -f %m {} + 2>/dev/null | sort -nr | head -1); newest=${newest:-0}
    [ "$newest" -lt "$lane_cutoff" ] || continue
    bytes=$(du -sk "$lane/target" 2>/dev/null | awk '{print $1*1024}'); bytes=${bytes:-0}
    action=clean
    if [ "$apply" -eq 1 ]; then
      (cd "$lane" && cargo clean)
      freed=$((freed + bytes)); action=cleaned
    fi
    printf '%-30s %-10s %s\n' "${lane##*/}/target" "$(format_size "$bytes")" "$action"
  done < <(find "$pool_root" -mindepth 1 -maxdepth 1 -type d -name 'lane-*' -print0)
fi
if [ "$apply" -eq 1 ]; then echo "Done; estimated space freed: $(format_size "$freed")"; else echo "Dry run; use --apply to perform listed actions."; fi

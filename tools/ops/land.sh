#!/bin/bash
set -u

usage() {
  echo "Usage: $0 <branch> [--dry-run]" >&2
  exit 2
}

[ "$#" -ge 1 ] && [ "$#" -le 2 ] || usage
branch=$1
dry_run=0
if [ "$#" -eq 2 ]; then
  [ "$2" = "--dry-run" ] || usage
  dry_run=1
fi

# The first worktree entry is the canonical checkout, regardless of cwd.
main=$(git worktree list --porcelain 2>/dev/null | awk 'NR == 1 && /^worktree / { sub(/^worktree /, ""); print; exit }')
if [ -z "$main" ]; then
  echo "land: cannot find the main checkout" >&2
  exit 1
fi

current=$(git -C "$main" symbolic-ref --quiet --short HEAD 2>/dev/null || true)
if [ "$current" != master ]; then
  echo "land: main checkout must have master checked out" >&2
  exit 1
fi

if ! git -C "$main" show-ref --verify --quiet "refs/heads/$branch"; then
  echo "land: branch does not exist: $branch" >&2
  exit 1
fi

master_tip=$(git -C "$main" rev-parse refs/heads/master) || exit 1
branch_tip=$(git -C "$main" rev-parse "refs/heads/$branch") || exit 1
if ! git -C "$main" merge-base --is-ancestor "$master_tip" "$branch_tip"; then
  echo "land: merge master into $branch first" >&2
  exit 1
fi

if [ "$dry_run" -eq 1 ]; then
  echo "Would run in $main: git merge --ff-only $branch"
  exit 0
fi

paths_file=$(mktemp "${TMPDIR:-/tmp}/land-paths.XXXXXX") || exit 1
trap 'rm -f "$paths_file"' EXIT HUP INT TERM
if ! git -C "$main" diff --name-only -z "$master_tip" "$branch_tip" >"$paths_file"; then
  echo "land: could not determine paths changed by $branch" >&2
  exit 1
fi

# Read NUL-delimited paths into a bash 3.2 array, preserving spaces.
paths=()
while IFS= read -r -d '' path; do
  paths+=("$path")
done <"$paths_file"

if ! git -C "$main" merge --ff-only "$branch"; then
  exit 1
fi

landed=$(git -C "$main" rev-parse HEAD) || exit 1
if [ "$landed" != "$branch_tip" ]; then
  echo "land: verification failed: HEAD is not the tip of $branch" >&2
  exit 1
fi
if [ "${#paths[@]}" -gt 0 ]; then
  if ! git -C "$main" diff --quiet HEAD -- "${paths[@]}"; then
    echo "land: verification failed: working tree differs on paths changed by the merge" >&2
    exit 1
  fi
  if ! git -C "$main" diff --cached --quiet -- "${paths[@]}"; then
    echo "land: verification failed: index differs on paths changed by the merge" >&2
    exit 1
  fi
fi
echo "land: fast-forwarded master to $branch_tip; changed paths are clean"

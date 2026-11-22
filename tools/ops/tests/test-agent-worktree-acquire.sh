#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")/.." && pwd)/agent-worktree.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/test-agent-worktree.XXXXXX")
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/.claude/worktrees"
git init -q -b master "$repo"
git -C "$repo" config user.name Test
git -C "$repo" config user.email test@example.invalid
echo base > "$repo/file"
git -C "$repo" add file
git -C "$repo" commit -qm base
git -C "$repo" checkout -qb astra/existing
echo commit > "$repo/commit-file"
git -C "$repo" add commit-file
git -C "$repo" commit -qm 'existing branch commit'
expected=$(git -C "$repo" rev-parse HEAD)
git -C "$repo" checkout -q master
git -C "$repo" checkout -qb astra/driver

out=$(cd "$repo" && bash "$script" acquire astra/existing)
actual=$(git -C "$out" rev-parse HEAD)
[ "$actual" = "$expected" ] || { echo "FAIL: existing branch moved from $expected to $actual" >&2; exit 1; }
[ "$(git -C "$out" branch --show-current)" = astra/existing ] || { echo 'FAIL: wrong branch checked out' >&2; exit 1; }
echo 'PASS: test-agent-worktree-acquire.sh (existing branch commit preserved)'

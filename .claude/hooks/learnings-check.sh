#!/usr/bin/env bash
# Stop hook: a one-line reminder, never a block. When code changed since the last check and no learnings file
# (the docs/learnings.md digest or a docs/learnings/<YYYY-MM>.md archive) changed, print one reminder line,
# once per change set. It used to block finishing, which grew docs/learnings.md to 1.5 MB (2026-11-22).
# It must never fail a session: any missing tool (git, jq, shasum) or error means a silent exit 0.
trap 'exit 0' ERR
set -uo pipefail

input=$(cat 2>/dev/null || true)
case "$input" in *'"stop_hook_active":true'*|*'"stop_hook_active": true'*) exit 0 ;; esac

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}" 2>/dev/null || exit 0
command -v git >/dev/null 2>&1 || exit 0
git rev-parse --git-dir >/dev/null 2>&1 || exit 0

# A content digest from whichever tool exists; with none, stay quiet.
if command -v shasum >/dev/null 2>&1; then
  digest() { shasum "$@" 2>/dev/null; }
elif command -v sha256sum >/dev/null 2>&1; then
  digest() { sha256sum "$@" 2>/dev/null; }
else
  exit 0
fi

state_file=".claude/hooks/.last-learnings-check"
non_code='^(docs/|plans/|CLAUDE\.md|AGENTS\.md|\.claude/|\.gitignore)'

last_head="" last_fp="" last_learn=""
if [ -f "$state_file" ]; then
  read -r last_head last_fp last_learn < "$state_file" || true
fi
head=$(git rev-parse --verify -q HEAD 2>/dev/null || echo none)
learn=$( { digest docs/learnings.md; find docs/learnings -name '*.md' -type f 2>/dev/null | sort | while IFS= read -r f; do digest "$f"; done; } \
  | digest | cut -d' ' -f1)
[ -n "$learn" ] || learn=none

wt_code=$(git status --porcelain --untracked-files=all 2>/dev/null | cut -c4- | sed 's/^"//' | grep -v -E "$non_code" || true)
committed_code=""
if [ -n "$last_head" ] && [ "$last_head" != none ] && [ "$head" != none ] && [ "$last_head" != "$head" ]; then
  committed_code=$(git diff --name-only "$last_head" "$head" 2>/dev/null | grep -v -E "$non_code" || true)
fi

fp=$( {
  printf '%s\n' "$committed_code"
  printf '%s\n' "$wt_code" | while IFS= read -r f; do
    if [ -f "$f" ]; then digest "$f"; else printf 'deleted %s\n' "$f"; fi
  done
} | digest | cut -d' ' -f1)
printf '%s %s %s\n' "$head" "$fp" "$learn" > "$state_file" 2>/dev/null || true

[ -z "$wt_code$committed_code" ] && exit 0
[ "$learn" != "$last_learn" ] && exit 0
[ "$fp" = "$last_fp" ] && exit 0

printf '%s\n' '{"systemMessage": "Reminder: if this work taught something non-obvious, add a dated entry to the current month'"'"'s docs/learnings/<YYYY-MM>.md (promote it to the docs/learnings.md digest only if durable)."}'
exit 0

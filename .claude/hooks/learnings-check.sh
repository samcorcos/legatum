#!/usr/bin/env bash
# Stop hook: when code changed since the last check and docs/learnings.md wasn't updated,
# block once so Claude considers recording what it learned. Fires once per change set.
set -euo pipefail

input=$(cat)
if [ "$(printf '%s' "$input" | jq -r '.stop_hook_active // false')" = "true" ]; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}"
git rev-parse --git-dir >/dev/null 2>&1 || exit 0

state_file=".claude/hooks/.last-learnings-check"
non_code='^(docs/|plans/|CLAUDE\.md|\.claude/|\.gitignore)'

last_head=""
last_fp=""
last_learn=""
if [ -f "$state_file" ]; then
  read -r last_head last_fp last_learn < "$state_file" || true
fi
head=$(git rev-parse --verify -q HEAD || echo none)
learn=$(shasum docs/learnings.md 2>/dev/null | cut -d' ' -f1 || echo none)

# Uncommitted code changes, including untracked files.
wt_code=$(git status --porcelain --untracked-files=all | cut -c4- | sed 's/^"//' | grep -v -E "$non_code" || true)

# Code committed since the last check.
committed_code=""
if [ -n "$last_head" ] && [ "$last_head" != none ] && [ "$head" != none ] && [ "$last_head" != "$head" ]; then
  committed_code=$(git diff --name-only "$last_head" "$head" 2>/dev/null | grep -v -E "$non_code" || true)
fi

# Fingerprint by file contents so edits to untracked files count too.
fp=$( {
  printf '%s\n' "$committed_code"
  printf '%s\n' "$wt_code" | while IFS= read -r f; do
    if [ -f "$f" ]; then shasum "$f"; else printf 'deleted %s\n' "$f"; fi
  done
} | shasum | cut -d' ' -f1)
printf '%s %s %s\n' "$head" "$fp" "$learn" > "$state_file"

[ -z "$wt_code$committed_code" ] && exit 0
[ "$learn" != "$last_learn" ] && exit 0
[ "$fp" = "$last_fp" ] && exit 0

jq -n '{
  decision: "block",
  reason: "Code changed since the last learnings check. If this work produced anything non-obvious (a surprising bug cause, a balance or simulation finding, a dead end, a gotcha), add a dated entry to docs/learnings.md. Put design decisions in the design doc decisions log. If nothing is worth recording, say so in one line and finish."
}'

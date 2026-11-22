#!/usr/bin/env bash
# Tests for the Stop and PreToolUse hooks: they never block on learnings, remind once per change set, and exit 0
# silently when jq or shasum is missing. Run: bash .claude/hooks/test-hooks.sh
set -u
here="$(cd "$(dirname "$0")" && pwd)"
fail=0
check() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got [$2] want [$3]"; fail=1; fi; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/docs/learnings" "$repo/.claude/hooks" "$repo/src"
git -C "$repo" init -q
git -C "$repo" -c user.email=t@example.invalid -c user.name=t commit -q --allow-empty -m init
echo "# digest" > "$repo/docs/learnings.md"

run_learn() { printf '{}' | CLAUDE_PROJECT_DIR="$repo" PATH="$1" bash "$here/learnings-check.sh"; echo "exit=$?"; }

out=$(run_learn "$PATH")
check "first run records state and stays quiet" "$out" "exit=0"
echo "fn main() {}" > "$repo/src/a.rs"
out=$(run_learn "$PATH")
case "$out" in *systemMessage*Reminder*exit=0) r=reminder ;; *) r="$out" ;; esac
check "code change gives one reminder, exit 0" "$r" "reminder"
case "$out" in *'"decision"'*) b=blocked ;; *) b=no ;; esac
check "never blocks" "$b" "no"
out=$(run_learn "$PATH")
check "same change set stays quiet" "$out" "exit=0"
echo "fn main() { 1; }" > "$repo/src/a.rs"
echo "## 2026-11-22: x" > "$repo/docs/learnings/2026-11.md"
out=$(run_learn "$PATH")
check "a learnings archive edit counts" "$out" "exit=0"

# A PATH with git, bash and coreutils but no shasum, sha256sum or jq.
bin="$tmp/bin"
mkdir -p "$bin"
for t in bash git cat cut sed grep find sort mktemp env dirname printf; do
  p=$(command -v "$t" 2>/dev/null) && ln -s "$p" "$bin/$t"
done
echo "fn main() { 2; }" > "$repo/src/a.rs"
out=$(run_learn "$bin")
check "no hash tool: silent exit 0" "$out" "exit=0"

out=$(printf '{"tool_input":{"command":"pkill -f vitest"}}' | PATH="$bin" bash "$here/no-pkill.sh"; echo "exit=$?")
check "no-pkill without jq: allows, exit 0" "$out" "exit=0"
if command -v jq >/dev/null 2>&1; then
  out=$(printf '{"tool_input":{"command":"pkill -f vitest"}}' | bash "$here/no-pkill.sh")
  case "$out" in *deny*) d=deny ;; *) d="$out" ;; esac
  check "no-pkill with jq: denies pkill -f" "$d" "deny"
fi

out=$(printf '{"transcript_path":"/nonexistent"}' | CLAUDE_PROJECT_DIR="$repo" PATH="$bin" bash "$here/docs-check.sh"; echo "exit=$?")
check "docs-check without jq: silent exit 0" "$out" "exit=0"

exit $fail

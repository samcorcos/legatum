#!/usr/bin/env bash
# PreToolUse(Bash): block `pkill -f` and `killall`. They match other agents' processes (Playwright,
# vitest, cargo) by command line; three sub-agents slipped on 2026-10-09..11. Kill by PID instead.
# Without jq it allows everything rather than fail a contributor's session.
command -v jq >/dev/null 2>&1 || exit 0
cmd=$(jq -r '.tool_input.command // empty' 2>/dev/null) || exit 0
if printf '%s' "$cmd" | grep -Eq '(^|[;&|[:space:]])(pkill[[:space:]]+(-[a-zA-Z]*f|--full)|killall)([[:space:]]|$)'; then
  jq -n '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:"pkill -f and killall can kill other agents'"'"' processes. Find the PID (pgrep -af PATTERN, check it is yours) and kill that PID."}}'
fi
exit 0

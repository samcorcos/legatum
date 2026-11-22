#!/usr/bin/env bash
# Watches Astra task results for ChatGPT usage-limit failures. A task that failed because the
# account hit its limit is parked in hold/ (so no work is lost and workers don't spin on an
# exhausted account), and a "LIMIT:" line with Codex's own message goes to .progress/astra.log so
# the lead can act (use the account's reset, then move the task back to pending/). Runs beside the queue workers without restarting them:
#   nohup tools/astra/limit-watch.sh >> .progress/astra-limit.log 2>&1 &
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
queue="$root/.progress/astra-queue"
# Only Codex's own error lines count, never transcript text (a game's "turn limit reached" or a
# line number containing 429 must not match).
error_line='^[[:space:]]*(\[[^]]*\][[:space:]]*)?(error|err)\b'
pattern='usage limit|rate limit|quota|out of credits|insufficient credits|429 too many requests'
log() { echo "$(date +%FT%T) | $*" >> "$root/.progress/astra.log"; }

tail -n 0 -F "$root/.progress/astra.log" | while read -r line; do
  [[ "$line" =~ finished\ task:\ ([^ ]+)\ \(exit\ ([0-9]+)\) ]] || continue
  name="${BASH_REMATCH[1]}" status="${BASH_REMATCH[2]}"
  # 0 is success; 143/137 are deliberate stops (SIGTERM/SIGKILL), never a limit.
  case "$status" in 0|143|137) continue ;; esac
  out="$root/.reviews/astra-$name.md.log"
  [ -f "$out" ] || continue
  hit="$(tail -n 40 "$out" | grep -iE "$error_line" | grep -iE "$pattern" | tail -n 1)"
  [ -n "$hit" ] || continue
  task="$(grep -l "^name: *$name\$" "$queue/done"/*.md 2>/dev/null | head -1)"
  if [ -n "$task" ]; then
    printf '\nNOTE: an earlier attempt stopped at a usage limit. Continue from your branch'"'"'s existing commits and worktree state instead of starting over.\n' >> "$task"
    mkdir -p "$queue/hold"
    mv "$task" "$queue/hold/"
  fi
  log "LIMIT: $name stopped at a usage limit (parked in hold: ${task:+yes}); codex said: $hit"
done

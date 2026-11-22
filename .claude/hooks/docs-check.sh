#!/usr/bin/env bash
# Stop hook: when the user's latest message looks like a decision, preference, rejection or
# correction and no skill, memory, CLAUDE.md or human-reasoning doc changed while answering it,
# block once so Claude records it (CLAUDE.md "Recording what we learn"). Fires at most once per
# user message; the design doc decisions log can't be fingerprinted here, so a one-line
# "recorded in the design doc" or "nothing to record" answer is enough to finish.
# It must never fail a session: without jq, python3 or a hash tool (shasum or sha256sum) it exits 0 silently.
trap 'exit 0' ERR
set -uo pipefail
command -v jq >/dev/null 2>&1 || exit 0
command -v python3 >/dev/null 2>&1 || exit 0
if command -v shasum >/dev/null 2>&1; then
  digest() { shasum "$@" 2>/dev/null; }
elif command -v sha256sum >/dev/null 2>&1; then
  digest() { sha256sum "$@" 2>/dev/null; }
else
  exit 0
fi

input=$(cat)
if [ "$(printf '%s' "$input" | jq -r '.stop_hook_active // false')" = "true" ]; then
  exit 0
fi
transcript=$(printf '%s' "$input" | jq -r '.transcript_path // empty')
[ -n "$transcript" ] && [ -f "$transcript" ] || exit 0

cd "${CLAUDE_PROJECT_DIR:-$(pwd)}"
# Claude Code keeps per-project memory under ~/.claude/projects/<main checkout path with / and . as ->/memory.
main_checkout="$(cd "$(git rev-parse --git-common-dir 2>/dev/null)/.." 2>/dev/null && pwd || pwd)"
memory_dir="$HOME/.claude/projects/$(printf '%s' "$main_checkout" | sed 's#[/.]#-#g')/memory"
state_file=".claude/hooks/.last-docs-check"

# Latest genuine user prompt: its id and whether it reads like a decision or preference.
read -r msg_id cue < <(python3 - "$transcript" <<'PY'
import json, re, sys
import os
last = None
with open(sys.argv[1], "rb") as fb:
    size = os.path.getsize(sys.argv[1])
    fb.seek(max(0, size - 8_000_000))  # transcripts reach hundreds of MB; the last prompt is near the end
    if fb.tell():
        fb.readline()
    lines = fb.read().decode("utf-8", "replace").splitlines()
if True:
    for line in lines:
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("type") != "user" or e.get("isSidechain"):
            continue
        c = e.get("message", {}).get("content")
        if isinstance(c, list):
            if any(isinstance(x, dict) and x.get("type") == "tool_result" for x in c):
                continue
            c = " ".join(x.get("text", "") for x in c if isinstance(x, dict) and x.get("type") == "text")
        if not isinstance(c, str):
            continue
        text = re.sub(r"<system-reminder>.*?</system-reminder>", "", c, flags=re.S).strip()
        if not text or text.startswith("<") or "task-notification" in text[:200] or text.startswith("Stop hook feedback"):
            continue
        last = (e.get("uuid", ""), text)
if not last:
    print("none 0"); sys.exit()
cues = r"\b(let'?s (go|do|use|try|make)|go with|i (want|'d like|would like|prefer|like|love|hate|don'?t (want|like|think))|we (should|need|shouldn'?t)|should (be|have|not)|please (do|make|fix|use|keep|don'?t)|don'?t|never|always|instead|rather than|decid|reject|not (a fan|good)|too (much|easy|hard|pale|subtle|techy)|keep (it|in mind)|from now on|make sure|to be clear|(my|our) [a-z-]+( [a-z-]+)? (likes?|loves?|hates?|wants?|said|thinks?)|(he|she|they) (likes?|loves?|hates?|wants?|said|thinks?))"
print(last[0] or "x", 1 if re.search(cues, last[1], re.I) else 0)
PY
)

fp=$( {
  find .claude/skills CLAUDE.md docs/playtests/human-reasoning.md docs/learnings.md docs/learnings "$memory_dir" \
    -type f \( -name '*.md' -o -name '*.json' \) 2>/dev/null | sort | while IFS= read -r f; do digest "$f"; done
} | digest | cut -d' ' -f1)

last_msg=""; last_fp=""
[ -f "$state_file" ] && read -r last_msg last_fp < "$state_file" || true
printf '%s %s\n' "$msg_id" "$fp" > "$state_file"

# Only the first stop after a new user message is checked.
[ "$msg_id" = "$last_msg" ] && exit 0
[ "$cue" = "1" ] || exit 0
[ -n "$last_fp" ] && [ "$fp" != "$last_fp" ] && exit 0

jq -n '{
  decision: "block",
  reason: "The user'"'"'s last message reads like a decision, preference, rejection or correction, and no skill, memory, CLAUDE.md or human-reasoning doc changed while answering it. Record it where it belongs (design doc decisions log; legatum-design / legatum-factions / legatum-ui / legatum-art / legatum-workflow / legatum-rules skill; memory; docs/playtests/human-reasoning.md for playtest reasoning). If it is already recorded or nothing needs recording, say so in one line and finish."
}'

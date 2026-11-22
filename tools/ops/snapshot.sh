#!/usr/bin/env bash
# One-call status snapshot for the ops supervisor: budgets, Codex tasks and lanes, orphans,
# slot pool, builds, machine, failures. Read-only: it never kills, moves or unlocks anything.
#   tools/ops/snapshot.sh            # fast (~5 s)
#   tools/ops/snapshot.sh --sizes    # also worktree target sizes (slow: du over ~70 worktrees)
#   SINCE=2026-10-04T09:50 tools/ops/snapshot.sh   # events since a time (default: last 2 hours)
set -uo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
q="$root/.progress/astra-queue"
runs="${LEGATUM_RUNS:-$HOME/LegatumRuns}"
since="${SINCE:-$(date -v-2H +%FT%T)}"
hr() { printf '\n== %s\n' "$1"; }

hr "time"
echo "local $(date +%FT%T)  utc $(date -u +%FT%TZ)  events since $since"

hr "toolchain"
if git --version >/dev/null 2>&1; then echo "git ok: $(git --version)"; else
  echo "ALERT: git unusable: $(git --version 2>&1 | head -1 | cut -c1-120)"
  echo "  (fix: the user runs 'sudo xcodebuild -license accept'; this script falls back to the CLT)"
  export DEVELOPER_DIR=/Library/Developer/CommandLineTools
fi
echo "xcode-select: $(xcode-select -p)  Xcode $(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' /Applications/Xcode.app/Contents/Info.plist 2>/dev/null)"

hr "usage (.progress/supervisor/usage.json, written by the lead)"
cat .progress/supervisor/usage.json 2>/dev/null || echo "missing"
echo

hr "machine"
uptime
echo "cores $(sysctl -n hw.logicalcpu)  swap: $(sysctl -n vm.swapusage)"
df -h "$HOME" | tail -1
mem_free="$(memory_pressure 2>/dev/null | awk -F': ' '/free percentage/{print $2}')"
echo "memory free: ${mem_free:-unknown}"

hr "codex write tasks (orphan = parent pid 1, i.e. its runner loop is gone)"
ps -axo pid=,ppid=,etime=,command= | awk '/codex exec -s/ && !/awk/ {
  wt=""; for (i=1;i<=NF;i++) if ($i=="-C") wt=$(i+1); sub(".*/", "", wt);
  printf "%-6s %-8s %-11s %s%s\n", $1, ($2==1?"ORPHAN":"ppid " $2), $3, wt, ($0 ~ /read-only/ ? " (read-only)" : "") }'
echo "runner loops:"; ps -axo pid=,etime=,command= | grep -E 'runner.sh (queue|reviewer|reaper)|start-lanes-later' | grep -v grep | sed 's/^/  /'

hr "queue"
for d in pending running hold held; do printf '%-8s %s\n' "$d" "$(ls "$q/$d" 2>/dev/null | grep -c '\.md$')"; done
echo "running tasks: last commit on astra/<name> and task-log age, in minutes (stalled = commit >60 and log >10):"
now=$(date +%s)
for f in "$q"/running/*.md; do
  [ -e "$f" ] || continue
  n="$(sed -n 's/^name: *//p' "$f" | head -1)"
  ct=$(git log -1 --format=%ct "astra/$n" 2>/dev/null || echo "$now")
  lg=".reviews/astra-$n.md.log"; lt=$(stat -f %m "$lg" 2>/dev/null || echo 0)
  live=live; pgrep -f "astra-$n.md" >/dev/null || live="NO CODEX PROCESS (reaper should file it within ~1 min)"
  printf '  %-34s commit %4s  log %4s  %s\n' "$n" $(( (now-ct)/60 )) $(( (now-lt)/60 )) "$live"
done
held_n=$(ls "$HOME/LegatumRuns/slots/held" 2>/dev/null | wc -l | tr -d ' ')
sims_via_slot=$(ps -axo ppid=,comm= | awk '{n=split($2,a,"/"); if (a[n] ~ /^legatum-sim/) print $1}' | while read -r pp; do ps -o command= -p "$pp" 2>/dev/null; done | grep -c 'slot.sh')
[ "$sims_via_slot" -gt 0 ] && [ "$held_n" = 0 ] && echo "  ALERT: $sims_via_slot sim(s) running under slot.sh but slots/held is empty: the cap is bypassed (sandbox kill -0 bug; fixed on master 003fe8b6, but tasks use their own worktree copy of slot.sh until they merge master)"
echo "pending (runner order, oldest first):"; ls -1tr "$q"/pending/*.md 2>/dev/null | sed 's#.*/#  #'
# Requirement audit (2026-10-04): per-branch balance cohorts are cut; full balance runs once per integration.
# Prompts written before a policy change keep asking for it, so flag them (edit pending/hold; running ones go to the lead).
for f in "$q"/running/*.md "$q"/pending/*.md "$q"/hold/*.md; do
  [ -e "$f" ] || continue
  grep -qiE '400 games|--games 400|v6, 400|merge master again and re-run' "$f" && echo "  PROMPT predates per-change policy: ${f#"$q"/} (400-game cohorts, v6 or a full re-check after the master merge)"
done
echo "master reviews since $since: $(awk -v s="$since" -F' [|] ' '$1>=s && /master review done/' .progress/astra.log | wc -l | tr -d ' ') run, $(awk -v s="$since" -F' [|] ' '$1>=s && /skipped prose-only/' .progress/astra.log | wc -l | tr -d ' ') prose-only skipped"

hr "lanes (.claude/worktrees/lane-k)"
found=0
for wt in .claude/worktrees/lane-*; do
  [ -d "$wt" ] || continue; found=1
  echo "  $wt on $(git -C "$wt" rev-parse --abbrev-ref HEAD) dirty=$(git -C "$wt" status --porcelain | wc -l | tr -d ' ')"
done
[ "$found" = 1 ] || echo "  none created yet (tasks whose branch already has a worktree reuse it)"

hr "astra.log since $since (starts, finishes, reaps, failures)"
awk -v s="$since" -F' [|] ' '$1>=s && /started task|finished task|reaped|ops:|lane/' .progress/astra.log | tail -25
echo "non-zero exits:"; awk -v s="$since" -F' [|] ' '$1>=s && /finished task/ && !/exit 0/ && !/reaped/' .progress/astra.log

hr "codex rate-limit / capacity errors in task logs touched since $since"
find .reviews -name 'astra-*.md.log' -newermt "${since/T/ }" 2>/dev/null | while read -r f; do
  c="$(grep -ciE '^(error|warn|\[[^]]*\] *error).*(rate.?limit|too many requests|usage limit|capacity|429)' "$f")"
  [ "$c" -gt 0 ] && echo "  $f: $c"
done

hr "builds"
echo "cargo $(pgrep -x cargo | wc -l | tr -d ' ')  rustc $(pgrep -x rustc | wc -l | tr -d ' ')  test binaries $(ps -axo command= | grep -c '/target/debug/deps/')"
echo "worktrees $(git worktree list | wc -l | tr -d ' ')  locked $(git worktree list --porcelain | grep -c '^locked')"
blk="$(find .reviews -name 'astra-*.md.log' -newermt "${since/T/ }" -exec grep -l 'Blocking waiting for file lock' {} + 2>/dev/null | wc -l | tr -d ' ')"
echo "task logs with 'Blocking waiting for file lock': $blk"
sccache --show-stats 2>/dev/null | grep -E '^Cache hits rate +[0-9]|Cache hits rate \(Rust\)|^Cache size|^Max cache size'
# Stall check (2026-10-10): a 12 h old server ran one compile at a time while 25 clients queued.
srv="$(ps -axo pid=,command= | awk '$2 ~ /sccache$/ && NF==2 {print $1; exit}')"
if [ -n "$srv" ]; then
  waiting="$(pgrep -f '^/opt/homebrew/bin/sccache /' | wc -l | tr -d ' ')"
  running="$(pgrep -P "$srv" | wc -l | tr -d ' ')"
  echo "sccache server $srv up $(ps -o etime= -p "$srv" | tr -d ' '): compiling $running, clients waiting $waiting"
  [ "$waiting" -ge 10 ] && [ "$running" -le 1 ] && echo "  ALERT: sccache serialised; restart it outside cargo (legatum-ops, Standing checks)"
fi
if [ "${1:-}" = "--sizes" ]; then
  echo "largest targets:"; du -sh .claude/worktrees/*/target 2>/dev/null | sort -rh | head -8 | sed 's/^/  /'
fi

hr "slot pool ($runs/slots)"
echo "held: $(ls "$runs/slots/held" 2>/dev/null | tr '\n' ' ')"
echo "queue: $(ls "$runs/slots/queue" 2>/dev/null | tr '\n' ' ')"
[ -d "$runs/slots/mutex" ] && echo "mutex: $(ls "$runs/slots/mutex")"
echo "slot.sh processes (a waiter older than 30 min with no ticket is stuck):"
ps -axo pid=,ppid=,etime=,command= | grep 'slot.sh' | grep -v grep | cut -c1-160 | sed 's/^/  /'

hr "release and check logs in $runs modified since $since"
find "$runs" -maxdepth 1 -name '*.log' -newermt "${since/T/ }" 2>/dev/null | while read -r f; do
  printf '  %s: %s\n' "${f##*/}" "$(tail -1 "$f" | cut -c1-120)"
done

hr "background jobs and alerts (tools/ops/bgjob.sh jobs; watchdog.sh is the token-free alerter)"
if pgrep -f 'tools/ops/watchdog.sh' >/dev/null; then echo "watchdog daemon: running (pid $(pgrep -f 'tools/ops/watchdog.sh' | head -1))"
else echo "ALERT: watchdog daemon NOT running: nohup tools/ops/watchdog.sh > .progress/jobs/watchdog.log 2>&1 &"; fi
tools/ops/watchdog.sh --status
echo "last 10 lines of .progress/alerts.log:"
tail -10 .progress/alerts.log 2>/dev/null | sed 's/^/  /' || true
[ -s .progress/alerts.log ] || echo "  (none)"

hr "heartbeat"
python3 tools/ops/heartbeat.py 2>&1 | tail -22

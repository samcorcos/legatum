#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
tmp="$(mktemp -d)"
runner_pid=""
cleanup() {
  [ -z "$runner_pid" ] || { kill "$runner_pid" 2>/dev/null || true; wait "$runner_pid" 2>/dev/null || true; }
  rm -rf "$tmp"
}
trap cleanup EXIT

mkdir -p "$tmp/tools/astra" "$tmp/.progress/astra-queue/pending" "$tmp/bin"
cp "$root/tools/astra/runner.sh" "$tmp/tools/astra/runner.sh"
cat > "$tmp/.progress/astra-queue/pending/failing-lane.md" <<'TASK'
mode: write
name: failing-lane

Should never reach a worker.
TASK
cat > "$tmp/bin/git" <<'STUB'
#!/usr/bin/env bash
case "$*" in
  'rev-parse -q --verify master') exit 0 ;;
  'worktree list --porcelain') exit 0 ;;
  worktree\ add\ *) echo 'stub: worktree add failed' >&2; exit 17 ;;
  *) echo "unexpected git: $*" >&2; exit 99 ;;
esac
STUB
chmod +x "$tmp/bin/git"
PATH="$tmp/bin:$PATH" bash "$tmp/tools/astra/runner.sh" queue 1 >"$tmp/runner.stdout" 2>"$tmp/runner.stderr" &
runner_pid=$!
for ((i=0; i<100; i++)); do
  [ -e "$tmp/.progress/astra-queue/hold/failing-lane.md" ] &&
    grep -q 'lane 1 setup failed for failing-lane: git worktree add failed .* → hold/' "$tmp/.progress/astra.log" && break
  sleep 0.1
done
test -f "$tmp/.progress/astra-queue/hold/failing-lane.md"
test ! -e "$tmp/.progress/astra-queue/running/failing-lane.md"
test ! -e "$tmp/.progress/astra-queue/done/failing-lane.md"
grep 'lane 1 setup failed for failing-lane: git worktree add failed .* → hold/' "$tmp/.progress/astra.log"
echo 'PASS: failed git worktree add parks task with diagnostic'

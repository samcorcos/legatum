#!/usr/bin/env bash
set -euo pipefail

script="$(cd "$(dirname "$0")/.." && pwd)/stale-worktrees.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/test-stale-worktrees.XXXXXX")
trap 'if [ -n "${sleeper:-}" ]; then kill "$sleeper" 2>/dev/null || :; fi; rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo/.claude/worktrees" "$repo/.progress/astra-queue/running"
git init -q -b master "$repo"
git -C "$repo" config user.name Test
git -C "$repo" config user.email test@example.invalid
echo base > "$repo/file"
git -C "$repo" add file
git -C "$repo" commit -qm base
add_wt() { git -C "$repo" worktree add -qb "$2" "$repo/.claude/worktrees/$1" master >/dev/null; }
age_tree() { touch -t 202001010000 "$1"; find "$1" -type f -exec touch -t 202001010000 {} +; }
fail() { echo "FAIL: $*" >&2; exit 1; }

add_wt clean astra/clean
add_wt dirty astra/dirty
echo dirty >> "$repo/.claude/worktrees/dirty/file"
echo untracked > "$repo/.claude/worktrees/dirty/output.txt"
add_wt busy astra/busy
add_wt locked astra/locked
git -C "$repo" worktree lock "$repo/.claude/worktrees/locked" --reason 'claude agent finished (pid 99999999)'
add_wt task astra/task
printf 'branch: astra/task\n' > "$repo/.progress/astra-queue/running/job.md"
add_wt lane-1 astra/lane
mkdir -p "$repo/.claude/worktrees/lane-1/target/debug"
echo cache > "$repo/.claude/worktrees/lane-1/target/debug/cache"
for n in clean dirty busy locked task lane-1; do age_tree "$repo/.claude/worktrees/$n"; done

# Keep a process whose cwd is inside the busy worktree.
(cd "$repo/.claude/worktrees/busy" && exec sleep 60) &
sleeper=$!
sleep 1
patch_home="$tmp/home"
mkdir -p "$patch_home"
mkdir -p "$tmp/bin"
cat > "$tmp/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
rm -rf target
CARGO
chmod +x "$tmp/bin/cargo"
out=$(cd "$repo" && HOME="$patch_home" PATH="$tmp/bin:$PATH" bash "$script" --apply)
[ ! -e "$repo/.claude/worktrees/clean" ] || fail 'clean idle tree remained'
[ ! -e "$repo/.claude/worktrees/dirty" ] || fail 'dirty idle tree remained'
[ -e "$repo/.claude/worktrees/busy" ] || fail 'live cwd tree was removed'
[ ! -e "$repo/.claude/worktrees/locked" ] || fail 'dead agent lock tree remained'
[ -e "$repo/.claude/worktrees/task" ] || fail 'running task tree was removed'
[ -d "$patch_home/LegatumData/worktree-patches-$(date +%F)" ] || fail 'patch directory missing'
patch=$(find "$patch_home/LegatumData" -name dirty.patch -print -quit)
[ -n "$patch" ] || fail 'dirty patch missing'
grep -q '^HEAD .* BRANCH astra/dirty$' "$patch" || fail 'patch header missing'
grep -q '^output.txt$' "$patch" || fail 'untracked file list missing'

out=$(cd "$repo" && HOME="$patch_home" PATH="$tmp/bin:$PATH" bash "$script" --apply --lanes)
[ ! -e "$repo/.claude/worktrees/lane-1/target/debug/cache" ] || fail '--lanes did not clean target cache'
echo "PASS: test-stale-worktrees.sh (idle, dirty patch, live cwd, dead lock, task protection, lane clean)"

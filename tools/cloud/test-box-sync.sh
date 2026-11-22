#!/usr/bin/env bash
# box_sync only moves the box scripts forward along master (box-lib.sh). Runs the real box_sync against a local
# directory standing in for /data/bin.
set -euo pipefail
src="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"; bin="$tmp/bin"
mkdir -p "$repo/tools/cloud/box" "$bin"
cp "$src/box-lib.sh" "$src/lib.sh" "$repo/tools/cloud/"
g() { git -C "$repo" -c user.name=t -c user.email=t@example.invalid "$@"; }
g init -q -b master
cat > "$tmp/hook.sh" <<'H'
box_ensure_data() { :; }
box_ssh() { bash -c "$*"; }
H
# macOS bsdtar rejects the GNU-only --warning flag the box command passes.
mkdir -p "$tmp/stubs"
printf '#!/bin/bash\nargs=(); for a in "$@"; do case "$a" in --warning=*) ;; *) args+=("$a");; esac; done\nexec /usr/bin/tar "${args[@]}"\n' > "$tmp/stubs/tar"
chmod +x "$tmp/stubs/tar"
printf '#!/bin/bash\nexec shasum -a 256 "$@"\n' > "$tmp/stubs/sha256sum"
printf '#!/bin/bash\nexec "$@"\n' > "$tmp/stubs/sudo"
printf '#!/bin/bash\nexit 0\n' > "$tmp/stubs/chattr"
chmod +x "$tmp/stubs/sha256sum" "$tmp/stubs/sudo" "$tmp/stubs/chattr"; export PATH="$tmp/stubs:$PATH"
export LEGATUM_BOX_TEST_HOOK="$tmp/hook.sh" LEGATUM_BOX_BIN="$bin"
version() { cat "$bin/VERSION" 2>/dev/null || echo none; }
sync() { ( cd "$repo" && . tools/cloud/box-lib.sh && box_sync ) 2>&1; }
commit_box() { echo "$1" > "$repo/tools/cloud/box/box-job"; g add -A; g commit -qm "$1"; g rev-parse HEAD; }
fail() { echo "FAIL: $*" >&2; exit 1; }

c1="$(commit_box one)"
sync >/dev/null; [ "$(version)" = "$c1" ] || fail "first sync should install $c1, box has $(version)"
[ "$(cat "$bin/box-job")" = one ] || fail "first sync did not copy the scripts"
echo "PASS: the first sync installs the scripts and records their commit"

c2="$(commit_box two)"
sync >/dev/null; [ "$(version)" = "$c2" ] && [ "$(cat "$bin/box-job")" = two ] || fail "a newer master commit should sync forward"
echo "PASS: a newer master checkout syncs forward"

# An agent on an old checkout (c1) must not downgrade the box.
g worktree add -q "$tmp/old" "$c1" --detach
mkdir -p "$tmp/old/tools/cloud"; cp "$src/box-lib.sh" "$src/lib.sh" "$tmp/old/tools/cloud/"
out="$(cd "$tmp/old" && . tools/cloud/box-lib.sh && box_sync 2>&1)"
[ "$(version)" = "$c2" ] && [ "$(cat "$bin/box-job")" = two ] || fail "an old checkout downgraded the box"
case "$out" in *"not syncing"*) ;; *) fail "the skipped sync was not reported: $out";; esac
echo "PASS: an old checkout does not downgrade the box"

# A branch with its own unmerged box edits does not deploy them.
g checkout -q -b feature; c3="$(commit_box feature-edit)"; g checkout -q master
g checkout -q feature; sync | grep -q "not on master" || fail "unmerged branch scripts were not refused"
[ "$(version)" = "$c2" ] || fail "an unmerged branch changed the box"
echo "PASS: unmerged branch scripts are not deployed"

# Uncommitted edits are not deployed either, but FORCE overrides.
g checkout -q master; echo dirty > "$repo/tools/cloud/box/box-job"
sync | grep -q "uncommitted" || fail "dirty box scripts were not refused"
[ "$(cat "$bin/box-job")" = two ] || fail "dirty edits reached the box"
( cd "$repo" && . tools/cloud/box-lib.sh && LEGATUM_BOX_SYNC_FORCE=1 box_sync ) >/dev/null 2>&1
[ "$(cat "$bin/box-job")" = dirty ] || fail "LEGATUM_BOX_SYNC_FORCE=1 did not force the sync"
echo "PASS: uncommitted edits are refused unless forced"

# A box with no VERSION (installed before versioning) takes the first sync.
rm "$bin/VERSION"; git -C "$repo" checkout -q -- tools/cloud/box/box-job
sync >/dev/null; [ "$(version)" = "$c2" ] || fail "an unversioned box should take the first sync"
echo "PASS: an unversioned box is synced"

# Same commit but the files on the box were overwritten in place by an old client: the digest check repairs it.
echo tampered > "$bin/box-job"
sync >/dev/null; [ "$(cat "$bin/box-job")" = two ] || fail "a tampered box-job was not repaired"
echo "PASS: drift under an unchanged VERSION is repaired"

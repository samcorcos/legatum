#!/bin/bash
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../../.." && pwd)
land_script="$repo_root/tools/ops/land.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/test-land.XXXXXX")
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

fail() { echo "FAIL: $*" >&2; exit 1; }
assert_contains() { case "$1" in *"$2"*) ;; *) fail "expected output to contain: $2";; esac; }
new_repo() {
  name=$1
  root="$tmp/$name"
  mkdir -p "$root/main"
  git init -q -b master "$root/main"
  git -C "$root/main" config user.name "Land Test"
  git -C "$root/main" config user.email "land-test@example.invalid"
  echo base > "$root/main/file.txt"
  git -C "$root/main" add file.txt
  git -C "$root/main" commit -qm base
  git -C "$root/main" worktree add -qb feature "$root/second" master
}
add_branch_commit() {
  echo "$2" > "$1/feature.txt"
  git -C "$1" add feature.txt
  git -C "$1" commit -qm feature
}

# A clean fast-forward succeeds even when called from outside the main checkout.
new_repo clean
add_branch_commit "$tmp/clean/second" clean
out=$(cd "$tmp/clean/second" && bash "$land_script" feature)
assert_contains "$out" "fast-forwarded master"
[ "$(git -C "$tmp/clean/main" rev-parse HEAD)" = "$(git -C "$tmp/clean/main" rev-parse feature)" ] || fail "clean fast-forward did not land branch tip"

# A branch that diverged after master moved is refused.
new_repo nonff
echo branch > "$tmp/nonff/second/branch.txt"
git -C "$tmp/nonff/second" add branch.txt
git -C "$tmp/nonff/second" commit -qm branch
echo master > "$tmp/nonff/main/master.txt"
git -C "$tmp/nonff/main" add master.txt
git -C "$tmp/nonff/main" commit -qm master-change
before=$(git -C "$tmp/nonff/main" rev-parse master)
if out=$(cd "$tmp/nonff/second" && bash "$land_script" feature 2>&1); then fail "non-fast-forward branch was accepted"; fi
assert_contains "$out" "merge master into feature first"
[ "$(git -C "$tmp/nonff/main" rev-parse master)" = "$before" ] || fail "non-fast-forward changed master"

# An unrelated dirty path is preserved while the fast-forward succeeds.
new_repo unrelated
add_branch_commit "$tmp/unrelated/second" unrelated
echo local > "$tmp/unrelated/main/local.txt"
out=$(cd "$tmp/unrelated/main" && bash "$land_script" feature)
assert_contains "$out" "fast-forwarded master"
[ -f "$tmp/unrelated/main/local.txt" ] || fail "unrelated dirty file was lost"

# A dirty path that the merge changes is refused by git, leaving master unchanged.
new_repo touched
echo branch > "$tmp/touched/second/file.txt"
git -C "$tmp/touched/second" add file.txt
git -C "$tmp/touched/second" commit -qm change-file
echo local > "$tmp/touched/main/file.txt"
before=$(git -C "$tmp/touched/main" rev-parse master)
if out=$(cd "$tmp/touched/main" && bash "$land_script" feature 2>&1); then fail "merge touching dirty file was accepted"; fi
assert_contains "$out" "local changes"
[ "$(git -C "$tmp/touched/main" rev-parse master)" = "$before" ] || fail "refused dirty merge changed master"

# The command refuses if the first worktree is not on master.
new_repo detached
git -C "$tmp/detached/main" checkout -q --detach
if out=$(cd "$tmp/detached/main" && bash "$land_script" feature 2>&1); then fail "main checkout without master was accepted"; fi
assert_contains "$out" "main checkout must have master checked out"

echo "PASS: test-land.sh (clean fast-forward, non-fast-forward, unrelated dirty file, touched dirty file, master checkout)"

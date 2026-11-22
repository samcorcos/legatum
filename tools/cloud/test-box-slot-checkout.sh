#!/usr/bin/env bash
# Exercise the idempotent slot fetch retry against a local bare repository.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
export REAL_GIT="$(command -v git)" FETCH_COUNT="$tmp/fetch-count"
mkdir -p "$tmp/bin" "$tmp/work"
cat > "$tmp/bin/git" <<'SH'
#!/usr/bin/env bash
case "$*" in
  *" fetch "*)
    n=0; [ ! -f "$FETCH_COUNT" ] || n="$(cat "$FETCH_COUNT")"; n=$((n + 1)); echo "$n" > "$FETCH_COUNT"
    if [ "$n" -eq 1 ]; then echo "stub: transient fetch failure" >&2; exit 1; fi
    ;;
esac
exec "$REAL_GIT" "$@"
SH
chmod +x "$tmp/bin/git"

"$REAL_GIT" init -q "$tmp/work"
"$REAL_GIT" -C "$tmp/work" -c user.name=stub -c user.email=stub@example.invalid commit --allow-empty -qm base
base="$("$REAL_GIT" -C "$tmp/work" rev-parse HEAD)"
"$REAL_GIT" init -q --bare "$tmp/repo.git"
"$REAL_GIT" --git-dir="$tmp/repo.git" symbolic-ref HEAD refs/heads/master
"$REAL_GIT" --git-dir="$tmp/repo.git" fetch -q "$tmp/work" "$base:refs/heads/master"
"$REAL_GIT" clone -q --no-checkout "$tmp/repo.git" "$tmp/slot"
"$REAL_GIT" -C "$tmp/work" -c user.name=stub -c user.email=stub@example.invalid commit --allow-empty -qm target
sha="$("$REAL_GIT" -C "$tmp/work" rev-parse HEAD)"
"$REAL_GIT" --git-dir="$tmp/repo.git" fetch -q "$tmp/work" "$sha:refs/jobs/job-1/ref/master"
PATH="$tmp/bin:$PATH"
export PATH
. "$here/box/slot-checkout.sh"
slot_fetch_checkout "$tmp/repo.git" job-1 "$sha" "$tmp/slot"
[ "$(cat "$FETCH_COUNT")" = 2 ]
[ "$(git -C "$tmp/slot" rev-parse HEAD)" = "$sha" ]
echo "slot checkout stub passed: transient fetch failure retried, requested commit checked out"

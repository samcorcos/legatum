#!/usr/bin/env bash
# Runs ON the box inside a slot (cwd = slot root). The remote half of tools/cloud/offload.sh.
#
#   lane-run.sh <rel-dir> <prep> -- <command...>
#
#   <rel-dir>  directory inside the checkout to run in ("." for the root, "app" for app/)
#   <prep>     none | wasm | dist | web
#                wasm    rebuild app/src/wasm-pkg when crates/ or Cargo files changed since the slot's last build
#                dist    wasm + `npm run build` in app/
#                web     wasm + the static e2e web dist (relay at 127.0.0.1:8787) that Playwright's fixture serves
#
# After the command it writes every file the command added, changed or deleted (git-tracked or untracked,
# not ignored) to .offload-out/changes.tar and .offload-out/deleted.nul, so the Mac can apply them: repin
# rewrites fixtures and pins.json, `cargo test` may rewrite Cargo.lock, a formatter rewrites sources.
# Exit code = the command's.
set -uo pipefail
. /data/bin/budget.sh && box_core_budget_apply   # this job's share of the cores (see budget.sh)
rel="$1"; prep="$2"; shift 2
[ "${1:-}" = "--" ] && shift
rm -rf .offload-out
tmpidx="$(mktemp -u)"
snapshot() { # prints a tree id of the current working tree (non-ignored files)
  GIT_INDEX_FILE="$tmpidx" git read-tree HEAD && GIT_INDEX_FILE="$tmpidx" git add -A . >/dev/null 2>&1 && GIT_INDEX_FILE="$tmpidx" git write-tree
  rm -f "$tmpidx"
}
before="$(snapshot)"

src_key() { find crates Cargo.toml Cargo.lock -type f -not -path '*/target/*' 2>/dev/null | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -c1-16; }
case "$prep" in
  none) ;;
  wasm|dist|web)
    key="$(src_key)"
    if [ "$(cat app/src/wasm-pkg/.src-key 2>/dev/null)" != "$key" ]; then
      echo "lane-run: building the WASM package"
      # Unoptimised: the tests do not need wasm-opt, which takes 2m40s.
      LEGATUM_WASM_NO_OPT=1 scripts/build-wasm.sh || { echo "lane-run: WASM build failed" >&2; exit 1; }
      echo "$key" > app/src/wasm-pkg/.src-key
    fi
    if [ "$prep" != wasm ]; then
      akey="$prep-$(find app -type f -not -path 'app/node_modules/*' -not -path 'app/dist/*' -not -path 'app/test-results/*' \
        -not -path 'app/src/wasm-pkg/*' -not -path 'app/playwright-report/*' | LC_ALL=C sort | xargs sha256sum | sha256sum | cut -c1-16)-$key"
      if [ "$(cat app/dist/.src-key 2>/dev/null)" != "$akey" ]; then
        echo "lane-run: building app/dist ($prep)"
        if [ "$prep" = web ]; then
          LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh --skip-wasm || { echo "lane-run: web build failed" >&2; exit 1; }
        else
          ( cd app && npm run build ) || { echo "lane-run: app build failed" >&2; exit 1; }
        fi
        echo "$akey" > app/dist/.src-key
      fi
    fi
    ;;
  *) echo "lane-run: unknown prep $prep" >&2; exit 2;;
esac

( cd "$rel" && "$@" )
rc=$?

after="$(snapshot)"
mkdir -p .offload-out
git diff-tree -z -r --no-renames --name-only --diff-filter=ACMRT "$before" "$after" > .offload-out/changed.nul
git diff-tree -z -r --no-renames --name-only --diff-filter=D "$before" "$after" > .offload-out/deleted.nul
if [ -s .offload-out/changed.nul ]; then
  tar cf .offload-out/changes.tar --null -T .offload-out/changed.nul
fi
echo "lane-run: $(tr -cd '\0' < .offload-out/changed.nul | wc -c | tr -d ' ') changed and $(tr -cd '\0' < .offload-out/deleted.nul | wc -c | tr -d ' ') deleted file(s) sent back"
exit "$rc"

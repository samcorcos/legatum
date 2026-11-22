#!/usr/bin/env bash
# Playwright WebKit on the cloud build box (native Linux, no VM, never on the macOS host): the screen tour and the
# exit-to-title spec that tools/release/webkit-docker.sh runs in a container.
#
#   tools/cloud/webkit-remote.sh <worktree-path|ref> [--with-uncommitted] [--artifact-dir DIR] [-- playwright args]
#
# Builds the WASM package, the app and the release server on the box, then runs the specs with
# LEGATUM_E2E_BROWSER=webkit. Results: <artifact dir>/app/test-results (default ~/LegatumRuns/box/<job>/artifacts).
# LEGATUM_WEBKIT_SPECS overrides the spec list; LEGATUM_E2E_BROWSER=chromium tells a WebKit-only failure from a
# test failure.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
target=""; ropts=(); pw=()
while [ $# -gt 0 ]; do
  case "$1" in
    --) shift; pw=("$@"); break;;
    --with-uncommitted) ropts+=("$1"); shift;;
    --artifact-dir) ropts+=("$1" "${2:?}"); shift 2;;
    -h|--help) sed -n '2,/^set -euo/p' "$0" | sed '$d;s/^# \{0,1\}//'; exit 0;;
    *) [ -z "$target" ] || { echo "webkit-remote: unexpected argument $1" >&2; exit 2; }; target="$1"; shift;;
  esac
done
[ -n "$target" ] || { echo "usage: webkit-remote.sh <worktree-path|ref> [options] [-- playwright args]" >&2; exit 2; }
env_opts=()
for v in LEGATUM_WEBKIT_SPECS LEGATUM_E2E_BROWSER LEGATUM_WEBKIT_TIMEOUT; do
  [ -z "${!v:-}" ] || env_opts+=(--env "$v=$(printf '%s' "${!v}" | tr ' ' ',')")   # spaces are not allowed in --env values
done
exec "$here/remote.sh" "$target" ${ropts[@]+"${ropts[@]}"} ${env_opts[@]+"${env_opts[@]}"} --npm --label "webkit:$target" --artifact app/test-results \
  -- bash /data/bin/webkit.sh ${pw[@]+"${pw[@]}"}

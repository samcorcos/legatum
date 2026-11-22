#!/usr/bin/env bash
# The map in real WebKit on this Mac (the engine of Safari), which Playwright's Linux WebKit and
# Chromium do not reproduce: playtest 8.9.4 showed rectangles of water on land that appeared in about half of fresh
# loads until the chunk readback in TerrainCache.renderChunk (docs/learnings.md, 2026-10-18).
#
#   tools/ui/webkit-mac/check.sh [--runs N] [--dist DIR] [--save FILE] [--out DIR]
#
# Loads the saved game in an invisible Retina WKWebView 1440x900, zooms Karakorum through four levels, and saves the map
# canvas after each settles; then compares the runs (compare.py) and exits 1 if any differs by a patch. No screenshots are
# taken (canvas.toDataURL only), so it does not touch WindowServer. Light on the CPU; build the dist on the box:
#   tools/cloud/remote.sh <worktree> --npm --artifact app/dist --artifact-dir DIR -- bash -c 'scripts/build-wasm.sh && (cd app && npm run build)'
# and pass --dist DIR/app/dist (default app/dist).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
runs=8; dist="$root/app/dist"; save="$root/crates/wasm/tests/fixtures/saves/p894-rects-t36.current.legatum"; out=""
while [ $# -gt 0 ]; do
  case "$1" in
    --runs) runs="$2"; shift 2;;
    --dist) dist="$2"; shift 2;;
    --save) save="$2"; shift 2;;
    --out) out="$2"; shift 2;;
    *) echo "unknown option $1" >&2; exit 2;;
  esac
done
[ -f "$dist/index.html" ] || { echo "no built app at $dist (see the header for how to build it on the box)" >&2; exit 2; }
work="$(mktemp -d)"; out="${out:-$work/out}"; mkdir -p "$out"
swiftc -O "$here/wk.swift" -o "$work/wk"
port="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')"
(cd "$dist" && exec python3 -m http.server "$port" --bind 127.0.0.1 >/dev/null 2>&1) &
server=$!
trap 'kill $server 2>/dev/null || true' EXIT
sleep 1
for i in $(seq 1 "$runs"); do
  "$work/wk" "http://127.0.0.1:$port/?client=wasm" "$here/body.js" "$out/run$i" "$save" 1440 900 | grep -E '^(z|error)' || true
done
python3 "$here/compare.py" "$out"/run*
echo "screenshots in $out"

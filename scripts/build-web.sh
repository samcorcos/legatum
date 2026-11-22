#!/usr/bin/env bash
# Build the hosted web version ("Play in your browser") into app/dist: static files for a plain web host
# such as Cloudflare Pages, with no legatum-server behind them.
#   1. the WASM engine (crates/wasm -> app/src/wasm-pkg)
#   2. the third-party licence notices (app/public/THIRD_PARTY_LICENSES.txt)
#   3. the web UI with VITE_LEGATUM_HOSTED=1 (app/src/platform/hosted.ts: no server probes, network play explained)
#   4. the host files from app/hosted/ (_headers, 404.html)
# Usage: scripts/build-web.sh [--skip-wasm]     --skip-wasm reuses an engine already in app/src/wasm-pkg
# LEGATUM_SITE_URL overrides the public address used in the link-preview tags (default: app/package.json homepage).
# Needs: Rust (cargo + wasm-pack), Node/npm. Check the result with: cd app && npm run e2e:hosted
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

if [ "${1:-}" != "--skip-wasm" ]; then scripts/build-wasm.sh; fi
[ -f app/src/wasm-pkg/legatum_bg.wasm ] || { echo "app/src/wasm-pkg is missing: run scripts/build-wasm.sh" >&2; exit 1; }

cd app
[ -d node_modules ] || npm ci
node ../tools/release/third-party-licenses.mjs

# The title screen shows this label; it carries the app version (package.json).
version="$(node -p "require('./package.json').version")"
commit="$(git rev-parse --short=7 HEAD 2>/dev/null || true)"
export LEGATUM_BUILD_LABEL="${LEGATUM_BUILD_LABEL:-Version $version${commit:+ · web $commit}}"
VITE_LEGATUM_HOSTED=1 npm run build

cp hosted/_headers hosted/404.html dist/
# A hosted build must not ship source maps or a file over Cloudflare Pages' 25 MiB limit.
if find dist -name '*.map' | grep -q .; then echo "dist contains source maps" >&2; exit 1; fi
big="$(find dist -type f -size +25M)"
[ -z "$big" ] || { echo "over the 25 MiB per-file limit: $big" >&2; exit 1; }

echo
echo "Built: $PWD/dist ($(du -sh dist | cut -f1), $(find dist -type f | wc -l | tr -d ' ') files) as \"$LEGATUM_BUILD_LABEL\""

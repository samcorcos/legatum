#!/usr/bin/env bash
# Build the web version ("Play in your browser") into app/dist: static files for a plain web host such as
# Cloudflare Pages. Online games run in the host player's tab and reach other players through the relay
# (infra/relay, plans/online-play.md), so no game server sits behind these files.
#   1. the WASM engine (crates/wasm -> app/src/wasm-pkg)
#   2. the third-party licence notices (app/public/THIRD_PARTY_LICENSES.txt)
#   3. the web UI, with the relay's address baked in (app/src/platform/online.ts)
#   4. the host files from app/hosted/ (_headers, whose connect-src names the relay, and 404.html)
# LEGATUM_GAMES_URL (optional) is the game-upload Worker (infra/games): every game played on the build is sent
# there (app/src/platform/gameUpload.ts), and _headers lets the page connect to it. Unset, nothing is uploaded.
# Usage: scripts/build-web.sh [--skip-wasm]     --skip-wasm reuses an engine already in app/src/wasm-pkg
# LEGATUM_SITE_URL overrides the public address used in the link-preview tags (default: app/package.json homepage).
# LEGATUM_CARRY_ASSETS_FROM (the production deploy sets it to the live site) keeps the live site's earlier files in
# the build, so a game left open in a tab keeps its art after the deploy (tools/release/carry-assets.mjs).
# Every build is also published at v/<id>/ and listed in versions.json, and the earlier builds the site lists stay
# (tools/release/keep-builds.mjs), so a game always finishes on the version it started on. LEGATUM_BUILD_ID
# overrides the id (default: the commit).
# LEGATUM_RELAY_URL overrides the relay (default: the live one below); set it to "none" for a build without
# online play (the menu then offers hot seat only).
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
# The build's id: its address on the site is v/<id>/ (tools/release/keep-builds.mjs), and saves record it so an
# older game opens in the build that made it. The commit, plus a hash of the uncommitted changes when there are any.
if [ -z "${LEGATUM_BUILD_ID:-}" ]; then
  LEGATUM_BUILD_ID="${commit:-local}"
  if [ -n "$commit" ] && [ -n "$(git status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    LEGATUM_BUILD_ID="$commit-$(git diff HEAD 2>/dev/null | git hash-object --stdin | cut -c1-6)"
  fi
fi
[[ "$LEGATUM_BUILD_ID" =~ ^[a-z0-9][a-z0-9-]{0,39}$ ]] || { echo "LEGATUM_BUILD_ID must be lower-case letters, digits and hyphens: $LEGATUM_BUILD_ID" >&2; exit 1; }
# The live relay, and the origins the page's Content-Security-Policy allows for it (hosted/_headers).
# hosted/_headers also lists the earlier workers.dev addresses: builds kept at v/<id>/ from before the domain
# still call them. Drop them there when keep-builds no longer keeps such a build.
default_relay="https://relay.playlegatum.com"
relay="${LEGATUM_RELAY_URL:-$default_relay}"
relay="${relay%/}"
[ "$relay" = none ] && relay=""
case "$relay" in ""|http://*|https://*) ;; *) echo "LEGATUM_RELAY_URL must start with http:// or https://" >&2; exit 1 ;; esac
games_url="${LEGATUM_GAMES_URL:-}"
games_url="${games_url%/}"
if [ -n "$games_url" ] && ! [[ "$games_url" =~ ^https://[A-Za-z0-9.-]+$ ]]; then
  echo "LEGATUM_GAMES_URL must be an https origin such as https://games.playlegatum.com" >&2; exit 1
fi
# Bundled PNG art is served as WebP on the website (app/vite-plugin-webp.ts). LEGATUM_WEBP=0 keeps PNG.
LEGATUM_WEBP="${LEGATUM_WEBP:-1}" VITE_LEGATUM_BUILD_ID="$LEGATUM_BUILD_ID" VITE_LEGATUM_RELAY_URL="$relay" VITE_LEGATUM_GAMES_URL="$games_url" npm run build

cp hosted/404.html dist/
if [ -z "$relay" ]; then
  sed -e "s| $default_relay||; s| ${default_relay/https/wss}||" hosted/_headers > dist/_headers
else
  sed -e "s|$default_relay|$relay|; s|${default_relay/https/wss}|$(printf %s "$relay" | sed 's/^http/ws/')|" hosted/_headers > dist/_headers
fi
# The page may also connect to the upload Worker, and to nothing else off its own site.
if [ -n "$games_url" ]; then sed -i.bak "s#connect-src 'self'#connect-src 'self' $games_url#" dist/_headers && rm dist/_headers.bak; fi
# Keep earlier builds' files, so a tab opened before this deploy still finds its art, fonts and code chunks.
# LEGATUM_KEEP_LEGACY adds builds from before versions.json, as "<deployment-url>=<id>:<rules>" separated by spaces.
legacy_crawl=(); legacy_keep=()
for spec in ${LEGATUM_KEEP_LEGACY:-}; do legacy_crawl+=(--crawl "${spec%%=*}"); legacy_keep+=(--legacy "$spec"); done
if [ -n "${LEGATUM_CARRY_ASSETS_FROM:-}" ]; then node ../tools/release/carry-assets.mjs dist --from "$LEGATUM_CARRY_ASSETS_FROM" ${legacy_crawl[@]+"${legacy_crawl[@]}"}; fi
# Publish this build at v/<id>/ too and keep the earlier builds' pages, so an older save opens in its own version.
rules="$(node -p "/pub const RULES_VERSION: u32 = (\\d+);/.exec(require('fs').readFileSync('../crates/core/src/lib.rs','utf8'))[1]")"
protocol="$(sed -n 's/^export const ONLINE_PROTOCOL = \([0-9]*\);.*/\1/p' src/platform/online.ts)"
node ../tools/release/keep-builds.mjs dist --id "$LEGATUM_BUILD_ID" --rules "$rules" --protocol "$protocol" \
  --label "$LEGATUM_BUILD_LABEL" --commit "$commit" ${LEGATUM_CARRY_ASSETS_FROM:+--from "$LEGATUM_CARRY_ASSETS_FROM"} \
  ${legacy_keep[@]+"${legacy_keep[@]}"}
# Every file the build points to must be in it (a missing one is a broken image in the game).
node ../tools/release/check-dist-refs.mjs dist
# A hosted build must not ship source maps or a file over Cloudflare Pages' 25 MiB limit.
if find dist -name '*.map' | grep -q .; then echo "dist contains source maps" >&2; exit 1; fi
big="$(find dist -type f -size +25M)"
[ -z "$big" ] || { echo "over the 25 MiB per-file limit: $big" >&2; exit 1; }

echo
echo "Built: $PWD/dist ($(du -sh dist | cut -f1), $(find dist -type f | wc -l | tr -d ' ') files) as \"$LEGATUM_BUILD_LABEL\"${relay:+, online play through $relay}"


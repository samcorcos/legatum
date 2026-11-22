#!/usr/bin/env bash
# Build the Legatum macOS desktop app from a clean checkout:
#   1. the WASM engine (crates/wasm -> app/src/wasm-pkg)
#   2. the third-party licence notices (app/public/THIRD_PARTY_LICENSES.txt: shown in Credits, copied into the bundle)
#   3. the web UI (app/dist, run by Tauri's beforeBuildCommand)
#   4. the Tauri shell (app/src-tauri) bundled as Legatum.app
# This build is unsigned. The signed, notarized release is built by .github/workflows/release-mac.yml
# (docs/ops/releasing.md).
# Usage: scripts/build-mac-app.sh [--dmg]
# LEGATUM_PLAYTEST_PROFILE=1 builds the faster thin-LTO playtest profile (WASM and Tauri shell); the default is the shipped release profile.
# LEGATUM_RELAY_URL=https://legatum-relay.<account>.workers.dev bakes in the online lobby's relay
# (app/src-tauri/src/online.rs reads it at compile time and adds its origins to the CSP); unset, the
# app still hosts on 127.0.0.1 but has no lobby. LEGATUM_BUILD_LABEL names the build as in tools/release/check.sh.
# Needs: Rust (cargo + wasm-pack), Node/npm, Xcode command line tools.
set -euo pipefail
cd "$(dirname "$0")/.."
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

bundles=app
if [ "${1:-}" = "--dmg" ]; then bundles=app,dmg; fi

for tool in cargo wasm-pack npm; do
  command -v "$tool" >/dev/null || { echo "missing $tool" >&2; exit 1; }
done

if [ "${LEGATUM_PLAYTEST_PROFILE:-0}" = 1 ]; then
  # Cargo reads these as overrides of the Tauri crate's [profile.release]; the bundle path stays target/release.
  export CARGO_PROFILE_RELEASE_LTO=thin CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
fi

if [ -n "${LEGATUM_RELAY_URL:-}" ]; then
  case "$LEGATUM_RELAY_URL" in
    https://*|http://localhost*|http://127.0.0.1*) export LEGATUM_RELAY_URL ;;
    *) echo "LEGATUM_RELAY_URL must be an https:// URL (or http://localhost for testing)" >&2; exit 1 ;;
  esac
  echo "Relay: $LEGATUM_RELAY_URL"
else
  echo "Relay: none (LEGATUM_RELAY_URL unset; online hosting stays local)"
fi

scripts/build-wasm.sh

cd app
[ -d node_modules ] || npm ci
node ../tools/release/third-party-licenses.mjs --scope desktop
# tauri.dist.conf.json adds what only a distributed build has: the licence notices as a bundle resource.
npx tauri build --bundles "$bundles" --config src-tauri/tauri.dist.conf.json

app_path="$PWD/src-tauri/target/release/bundle/macos/Legatum.app"
[ -d "$app_path" ] || { echo "build finished but $app_path is missing" >&2; exit 1; }
echo
echo "Built: $app_path ($(du -sh "$app_path" | cut -f1))"
echo "Run:   open \"$app_path\""

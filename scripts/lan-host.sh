#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

scripts/build-wasm.sh
if [ ! -d app/node_modules ]; then
  (cd app && npm ci)
fi
(cd app && npm run build)
# The server prints the localhost URL and every LAN IPv4 URL to share.
exec cargo run --release -p legatum-server -- --static app/dist "$@"

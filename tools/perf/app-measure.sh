#!/usr/bin/env bash
# Release-build Chromium measurements on the cloud box:
# tools/cloud/remote.sh . --with-uncommitted --npm --artifact out -- bash tools/perf/app-measure.sh baseline
set -euo pipefail
tag="${1:-baseline}"
mkdir -p out
tools/sim/build-slot.sh -- cargo build --release -p legatum-wasm --example turn_profile
RAYON_NUM_THREADS=1 target/release/examples/turn_profile 300 7000001 csv \
  out/perf-orders.jsonl 200:out/perf-save-200.json out/perf-save-start.json > out/perf-native-fixture.csv
scripts/build-wasm.sh
build_args=()
if [ "${LEGATUM_PERF_PROFILE:-0}" = 1 ]; then build_args+=(-- --sourcemap); fi
(cd app && npm run build "${build_args[@]}" > ../out/perf-build.log 2>&1) || { tail -40 out/perf-build.log; exit 1; }
tail -18 out/perf-build.log
if [ "${LEGATUM_PERF_PROFILE:-0}" = 1 ]; then cp app/dist/assets/index-*.js.map out/; fi
port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')"
(cd app && exec node_modules/.bin/vite preview --host 127.0.0.1 --port "$port" --strictPort > ../out/perf-preview.log 2>&1) &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT
for i in $(seq 1 60); do
  kill -0 "$server" 2>/dev/null || { cat out/perf-preview.log; exit 1; }
  if curl --silent --fail "http://127.0.0.1:$port/" >/dev/null; then break; fi
  sleep 1
done
curl --silent --fail "http://127.0.0.1:$port/" >/dev/null
test_args=()
if [ -n "${LEGATUM_PERF_GREP:-}" ]; then test_args+=(--grep "$LEGATUM_PERF_GREP"); fi
(cd app && LEGATUM_PERF=1 LEGATUM_PERF_TAG="$tag" LEGATUM_PERF_URL="http://127.0.0.1:$port" \
  npx playwright test e2e/perf-app.spec.ts --project=game --reporter=line "${test_args[@]}") 2>&1 | tee "out/perf-app-$tag.log"

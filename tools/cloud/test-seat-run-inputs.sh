#!/usr/bin/env bash
# Smoke both seat_run input forms against a short pinned tape; no AWS calls.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
out="target/seat-run-input-tests"
mkdir -p "$out/neighbour"
tools/sim/build-slot.sh -- cargo build --profile release-checked -p legatum-sim \
  --example seat_run --example audit_states
tape="crates/sim/tests/fixtures/game-00000-seed-18446744073709551613.json"
printf '{"schema":1,"name":"smoke"}\n' > "$out/params.json"
target/release-checked/examples/audit_states "$tape" "$out/states.jsonl"
target/release-checked/examples/seat_run --tape "$tape" --turns 2 \
  --games 1 --end-turn 2 --opponent-params "$out/params.json" \
  --out "$out/purchase.jsonl"
target/release-checked/examples/seat_run --tape "$tape" --states "$out/states.jsonl" \
  --turns 2 --games 1 --end-turn 2 --continue "rivals:$out/params.json" \
  --out-dir "$out/neighbour" --status-file "$out/status.json" --label neighbour
python3 - "$out" <<'PY'
import json
import pathlib
import sys
out = pathlib.Path(sys.argv[1])
for path in (out / 'purchase.jsonl', out / 'neighbour/seat-run.jsonl'):
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert len(rows) == 1, (path, len(rows))
    assert rows[0]['branch_turn'] == 2, (path, rows[0])
print('seat_run tape-only and tape-plus-states smoke passed')
PY

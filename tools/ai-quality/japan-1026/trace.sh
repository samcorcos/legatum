#!/usr/bin/env bash
# Original-engine fogged planning, with pre-plan memory, production scores and military phases.
# Usage: trace.sh STATES OUT
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
states="${1:?states}"; out="${2:?out}"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git clone --quiet --shared --no-checkout "$root" "$tmp/engine"
git -C "$tmp/engine" checkout --quiet --detach d1233d48678a2e89630fa456f710db100084a7de
cp "$root/crates/ai/examples/war_position.rs" "$tmp/engine/crates/ai/examples/war_position.rs"
python3 "$root/tools/ai-quality/japan-1026/instrument.py" "$tmp/engine"
cp "$root/tools/ai-quality/japan-1026/replay.py" "$tmp/engine/replay.py"
python3 - "$states" "$tmp/engine/inputs" <<'PY'
import gzip,json,sys
from pathlib import Path
states,out=map(Path,sys.argv[1:]); out.mkdir()
for line in (states/'plans.jsonl').read_text().splitlines():
 r=json.loads(line);t,p=r['turn'],r['player']
 if not (p==2 and t>=155 or t in [160,172,180,185,197,204,214,224]): continue
 prefix=f'T{t}-p{p}'
 for suffix,data in [('view',(states/f'{prefix}.view.json').read_bytes()),('memory',json.dumps(r['memory_before']).encode()),('recorded',json.dumps(r['recorded_orders']).encode())]:
  (out/f'{prefix}.{suffix}.json.gz').write_bytes(gzip.compress(data,mtime=0))
(out/'default.json').write_text('{}')
PY
"$root/tools/cloud/remote.sh" "$tmp/engine" --with-uncommitted --label japan-war-trace --artifact trace.json.gz --artifact-dir "$out" -- \
  bash -euo pipefail -c 'tools/sim/build-slot.sh -- cargo build --profile release-checked -q -p legatum-ai --example war_position && python3 replay.py inputs target/release-checked/examples/war_position trace.json.gz'

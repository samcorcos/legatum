#!/usr/bin/env bash
# Original-engine, fogged one-turn planning diagnoses. Offline instrumentation only.
# trace.sh STATES OUT [ENGINE] [strategic]
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
states="${1:?states directory}"; out="${2:?output directory}"
engine="${3:-8e9433e5cf53ea4ffe4e8729d5904c6320cb519a}"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
# A disposable local clone leaves the worker branch and its worktree untouched.
git clone --quiet --shared --no-checkout "$root" "$tmp/engine"
git -C "$tmp/engine" checkout --quiet --detach "$engine"
cp "$root/crates/ai/examples/war_position.rs" "$tmp/engine/crates/ai/examples/war_position.rs"
python3 "$root/tools/ai-quality/athens-1025/instrument.py" "$tmp/engine"
cp "$root/tools/ai-quality/athens-1025/replay.py" "$tmp/engine/athens_replay.py"
python3 - "$states" "$tmp/engine/athens-input" "${4:-war}" <<'PY'
import gzip,json,sys
from pathlib import Path
states,out=map(Path,sys.argv[1:3]); out.mkdir()
plans=[json.loads(s) for s in (states/'plans.jsonl').read_text().splitlines()]
for r in plans:
 t,p=r['turn'],r['player']
 if sys.argv[3] == "strategic":
  if t not in [194,201,215,221]: continue
 elif not (p==5 and t>=67 or p==2 and t>=185): continue
 prefix=f'T{t}-p{p}'
 for suffix,data in [('view',(states/f'{prefix}.view.json').read_bytes()),('memory',json.dumps(r['memory_before']).encode()),('recorded',json.dumps(r['orders']).encode())]:
  (out/f'{prefix}.{suffix}.json.gz').write_bytes(gzip.compress(data,mtime=0))
(out/'default.json').write_text('{}')
PY
"$root/tools/cloud/remote.sh" "$tmp/engine" --with-uncommitted --label athens-war-trace --artifact trace.json.gz --artifact-dir "$out" -- \
  bash -euo pipefail -c 'tools/sim/build-slot.sh -- cargo build --profile release-checked -q -p legatum-ai --example war_position && python3 athens_replay.py athens-input target/release-checked/examples/war_position trace.json.gz'

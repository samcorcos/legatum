#!/usr/bin/env bash
# Alliance census (run on the box: tools/cloud/remote.sh <worktree> --with-uncommitted -- tools/sim/alliance-census.sh [games]): two arms of eight-player Hard games (current rules and current rules without the bloc-size cost), tapes replayed by the alliance_metrics example.
set -euo pipefail
games="${1:-256}"; seed=7400001
cargo build --release -p legatum-sim --bin legatum-sim --example alliance_metrics
dir="$(mktemp -d)"; trap 'rm -rf "$dir"' EXIT
echo '{"schema":1,"name":"bloc0","decisions":{"bloc_size_penalty":0}}' > "$dir/bloc0.json"
run() { # name extra-args...
  local name="$1"; shift
  target/release/legatum-sim --players 8 --difficulty hard --barbarians true --games "$games" --seed "$seed" --threads 20 \
    --tape-out "$dir/$name/tapes" --out-dir "$dir/$name/out" --out "$dir/$name/replays" --label "census-$name" "$@" > "$dir/$name.log" 2>&1 || { tail -20 "$dir/$name.log"; exit 1; }
}
run new &
run bloc0 --ai-params "$dir/bloc0.json" &
wait
for name in new bloc0; do
  echo "=== $name"; tail -3 "$dir/$name.log"
  target/release/examples/alliance_metrics "$dir/$name/tapes" 24
done

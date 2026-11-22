#!/usr/bin/env bash
# perf profile of the app-path turn loop (turn_profile, one thread, 8 Hard empires); run on the box.
#   tools/cloud/remote.sh . --with-uncommitted --artifact out -- bash tools/perf/native-perf.sh [seed]
set -e
seed=${1:-7000001}
mkdir -p out
export RUSTFLAGS="-C force-frame-pointers=yes -C debuginfo=1"
export CARGO_TARGET_DIR=target/perf
cargo build --release -p legatum-wasm --example turn_profile 2>&1 | tail -1
id; sudo -n true && echo sudo-ok || true
P="perf"; [ "$(cat /proc/sys/kernel/perf_event_paranoid)" -gt 1 ] && P="sudo -n perf"
RAYON_NUM_THREADS=1 $P record -F 499 --call-graph fp -o out/perf.data -- target/perf/release/examples/turn_profile 400 $seed csv > out/perf-run.csv
$P report -i out/perf.data --no-children --percent-limit 0.4 --stdio -g none 2>/dev/null | grep -v '^#' | grep -v '^$' | head -80 > out/perf-flat.txt
$P report -i out/perf.data --children --percent-limit 2 --stdio -g none 2>/dev/null | grep -v '^#' | grep -v '^$' | head -120 > out/perf-children.txt
for sym in current_era "deposits" "GameState>::hash" has_building current_users "yields_base" "update_explored" format_escaped_str; do
  $P report -i out/perf.data --no-children --symbol-filter="$sym" --stdio -G --percent-limit 1 -g caller,0.5,callee,function,percent 2>/dev/null | grep -v '^#' | grep -v '^$' | cut -c1-200 | head -45 > "out/callers-$(echo $sym | tr -c 'a-zA-Z0-9\n' _).txt"
done
rm -f out/perf.data
cat out/perf-flat.txt

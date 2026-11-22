set -e
cargo build --release -p legatum-wasm --example turn_profile 2>&1 | tail -3
mkdir -p out
for s in 7000001 7000002 7000003 7000004; do
  RAYON_NUM_THREADS=1 target/release/examples/turn_profile 400 $s csv > out/native-$s.csv &
done
wait
for s in 7000001 7000002 7000003 7000004; do wc -l out/native-$s.csv; done

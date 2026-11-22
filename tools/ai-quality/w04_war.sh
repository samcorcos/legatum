#!/usr/bin/env bash
# W0.4: the war suite (64 runs, params on the measured seat) for the control and each gene-group arm.
# Run on the build box: tools/cloud/remote.sh claude/w04-bench --artifact war-out -- bash tools/ai-quality/w04_war.sh [group...]
set -euo pipefail
cargo build --locked --profile sim -p legatum-sim --example war_suite 2>&1 | tail -1
mkdir -p war-out
groups=("$@")
[ ${#groups[@]} -gt 0 ] || groups=(ctl planner staging localwar airdef gold modern)
for g in "${groups[@]}"; do
  if [ "$g" = ctl ]; then
    ./target/sim/examples/war_suite --brain hard --runs 64 --report "war-out/$g.json" > "war-out/$g.log" 2>&1
  else
    ./target/sim/examples/war_suite --brain hard --runs 64 --ai-params "docs/sim-results/ai-war/w04/$g.json" \
      --params-seats seat --report "war-out/$g.json" > "war-out/$g.log" 2>&1
  fi
  echo "$g done"
done

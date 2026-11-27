# Sparta offense, round 2

Four further opening ideas were implemented as independent Hard Sparta decision genes, initially all off by default. The control is the built-in rules-63 AI, including the already enabled Sparta conquest opening. Each arm changes one gene for the Sparta seat against five built-in Hard opponents. `offense_ladder` plays 300 paired full games per arm, seeds 1001–1300, with six Rayon threads. The gate is a positive paired 95% interval for final VP share and no decrease in observed wins; early captures and eliminations are diagnostic only.

| Arm | Wins / 300 | Final VP share | Paired VP change vs control (95% interval) | Capitals by T50 / game | Rivals out by T50 / game | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Control | 65 | 0.2156 | — | 0.737 | 0.543 | — |
| `sparta_center_search=1` | 53 | 0.2107 | −0.0049 [−0.0257, +0.0160] | 0.757 | 0.620 | Keep 0 |
| `sparta_scout_declare=1` | 65 | 0.2156 | 0.0000 [0.0000, 0.0000] | 0.737 | 0.543 | Keep 0 |
| `sparta_chain_opening=1` | 75 | 0.2281 | +0.0125 [+0.0006, +0.0244] | 0.817 | 0.617 | **Default 1** |
| `sparta_finish_60=1` | 61 | 0.2147 | −0.0009 [−0.0094, +0.0077] | 0.737 | 0.543 | Keep 0 |

The centre search sent Sparta to war by T30 more often (78% to 84%) and raised early eliminations, but lost 12 full games and did not improve final VP share. The one-melee-plus-Scout declaration arm produced the same outcomes as control on every seed; the runner did not count how often the shortcut became eligible. Extending the living-victim window to T60 gained no final VP and lost four wins. Chaining the opening after an actual elimination raised early capture counts and gained ten wins, so it alone passed the gate. The chain does not trigger merely because the first war stalls at Walls and ends in Peace.

The VP intervals use the seedwise difference in final share, its sample standard error, and a two-sided 1.97 standard-error margin. These are nominal intervals for four exploratory arms; the chain's positive margin is narrow. Win counts are observed outcomes, not a claim of a statistically established win-rate increase.

To reproduce independent arms after banking `sparta_chain_opening=1`, explicitly turn it off in every control and other single-gene arm:

```sh
cargo build --release -j 6 -p legatum-sim --example offense_ladder
RAYON_NUM_THREADS=6 target/release/examples/offense_ladder \
  --faction Sparta --games 300 --seed 1001 \
  --arm base:sparta_chain_opening=0 \
  --arm center:sparta_chain_opening=0,sparta_center_search=1 \
  --arm scout_declare:sparta_chain_opening=0,sparta_scout_declare=1 \
  --arm chain:sparta_chain_opening=1 \
  --arm finish60:sparta_chain_opening=0,sparta_finish_60=1 \
  --out /tmp/sparta-offense-r2-recheck.jsonl
```


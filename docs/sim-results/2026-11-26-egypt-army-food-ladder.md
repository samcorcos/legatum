# Egypt army Food: 300-game offense ladder (2026-11-26)

Command: `offense_ladder --faction Egypt --games 300 --seed 5001` with `RAYON_NUM_THREADS=6`. Each game has six Hard seats; Egypt is the observed seat. The default rules include 3 Food per Egyptian land military unit, shared across cities. No rule ablation was enabled.

| Measure | Result |
| --- | ---: |
| Egypt wins | 25.7% |
| First war by turn 30 | 20% |
| Captures by turn 50, mean | 0.14 |
| Captures by turn 75, mean | 0.45 |
| Army size at turns 30 / 50, mean | 2.7 / 6.0 |
| Cities at turns 50 / 100, mean | 3.9 / 6.2 |
| VP share, mean | 0.229 |
| Alive at finish | 79% |

The plan's earlier experiment reported 26.0% with the same seed and improved AI plus city upkeep; this default-rule run is within rounding distance. The previous base was 47.3%. This run measures the combined current rule set and does not isolate the army Food rule's causal effect; use `--rule-off egypt_army_food` for that ablation.

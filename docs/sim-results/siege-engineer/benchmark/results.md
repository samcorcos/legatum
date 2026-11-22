# Human-seat benchmark with the three Military Engineer genes (2026-10-18)

Hard brain, 8 playtest maps x 64 salts (512 games per arm), seed 1, one c8g.16xlarge spot server, built from the Engineer branch (`arms.json` here). The win rate is the seat's (the human's place) share of wins; `compare.json` is `tools/sim/hard-vs-human-compare.py` output (paired by map and salt, bootstrap 95% interval).

| Arm | Win rate | Delta vs base | 95% interval | Salts better / worse |
|---|---|---|---|---|
| base (genes off) | 22.1% | - | - | - |
| genes on for the seat only | 18.2% | -3.9 points | -7.2 to -0.6 | 28 / 48 |
| genes on for the opponents only | 25.0% | +2.9 points | -0.6 to +6.6 | 52 / 37 |

Engineers built per game (`strategic_built`): the seat 1.55 (base), 1.82 (genes on for the seat); the five AI empires together 4.58 (base), 4.78 (genes on for the opponents). The genes barely change how many Engineers are built, because the default AI already builds some once Mathematics is known.

Verdict: the seat arm is a significant loss and the opponents arm is not a gain for the AI (the seat wins more against them), so the three genes (`walls_as_cost`, `engineer_for_walls`, `sally_engineer`) stay default 0. They remain in `params.rs` for the hill-climb ledger.

Update 2026-10-20: `walls_as_cost` and `engineer_for_walls` now default to 1 after the coach's review of the 8.9.6 Athens win (the AI had built no Engineer in 233 turns and treated walls as a veto). `sally_engineer` stays 0. The same genes first read -7.8 pp seat-only after fixing only the phantom cover for a faction without Engineers, and were brought to -1.9 pp [-5.2, +1.4] by pricing the Engineers' build time and planning cover only for a force the cities can cover within ten turns. See `docs/sim-results/siege-engineer/walls-as-price/results.md`.

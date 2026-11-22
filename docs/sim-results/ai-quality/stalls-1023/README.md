# Rare stalls: goal deadlines in a game with no limit, and the Courier Hall (2026-10-23)

Branch `claude/ai-stalls`. Diagnosis and the fix are in `docs/learnings.md` (2026-10-23, "Rare stalls"). Genes `goal_open_end` and `hall_decisive` (both default 1; at 0 the binary reproduces master: 6,144 games, every turn count and winner identical).

## Tail of the game-length distribution (8-player Hard, 6,144 games, seeds 20001+, Medium map, no turn limit, safety cap 1,200)

| | Control (master `b4cd176d7`) | Genes on | Genes at 0 (same binary) |
| --- | ---: | ---: | ---: |
| Mean turns | 305.2 | 299.4 | 305.2 |
| p99 | 520 | 493 | 520 |
| Longest | 959 | 762 | 959 |
| Over 500 turns | 85 | 55 | 85 |
| Over 600 turns | 22 | 8 | 22 |
| Over 800 turns | 5 | 0 | 5 |
| Stalled at the cap | 0 | 0 | 0 |

Other cohorts on the same master: 400 games from seed 5200001 and 3,600 from 5200401 (the opening-ramp cohort): 0 stalls; 256 and 1,536 games from seed 9100 (the era-objectives cohort): one stall (seed 9983, a 9-VP Mongols), and 9322 ending at turn 1,160.

## Faction goal completion and wins, all-AI games (6,144)

| Faction | Goal done, control | Goal done, genes on | Wins, control | Wins, genes on |
| --- | ---: | ---: | ---: | ---: |
| Mongols | 28% | 37% | 728 | 1,042 |
| Japan | 10% | 14% | 115 | 134 |
| Germany | 46% | 45% | 607 | 562 |
| Byzantium | 86% | 86% | 1,671 | 1,575 |
| Athens | 78% | 77% | 1,215 | 1,119 |
| Switzerland | 85% | 85% | 963 | 899 |
| Celts | 96% | 96% | 447 | 418 |
| Sparta | 2% | 2% | 398 | 395 |

## Human-seat benchmark (8 maps x 128 reseeded salts, Hard, 1,024 games per arm, `arms.json`, one binary)

| Arm | Win rate | Delta against control (95% interval) | Salts better / worse |
| --- | ---: | --- | ---: |
| Control (both genes 0) | 39.26% | | |
| Seat only | 38.77% | -0.49 points (-1.07, +0.10) | 2 / 7 |
| Opponents only | 38.67% | -0.59 points (-1.37, +0.20) | 5 / 11 |
| All seats | 38.48% | -0.78 points (-1.66, +0.10) | 7 / 15 |

War suite (64 runs per scenario, `war_suite --brain hard`): mean 42.7% with and without the genes, every scenario identical; the attack check (`war-golden-horde-thin`) reads 0% on both, as on plain master (`docs/learnings.md` 2026-10-23, "the war-suite attack check reads 0% on master").

Raw runs: `~/LegatumRuns/cloud/stalls-gate1-1022-232348` (benchmark); the sim cohorts ran on the build box (`tools/sim/.tmp-*` scratch, not committed).

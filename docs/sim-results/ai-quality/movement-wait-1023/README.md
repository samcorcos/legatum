# Movement preflight: movers that claim one tile take turns (2026-10-23)

Branch `claude/ai-movement-wait`; cause, fix and discussion in `docs/learnings.md` (2026-10-23, "The AI movement preflight cancelled turns"). Gene `movement_wait` (default 1; 0 is the old cancellation, identical to master by construction).

## Human-seat benchmark (8 maps x 128 reseeded salts, Hard, 1,024 games per arm, `arms.json`, one binary)

| Arm | Win rate | Delta against control (95% interval) | Salts better / worse |
| --- | ---: | --- | ---: |
| Control (gene 0) | 37.01% | | |
| Seat only | 39.06% | +2.05 points (-1.46, +5.47) | 172 / 151 |
| Opponents only | 34.86% | -2.15 points (-5.57, +1.37) | 157 / 179 |
| All seats | 37.60% | +0.59 points (-2.93, +4.10) | 176 / 170 |

## War suite (the Golden Horde scenes, `war_suite --brain hard`, same salt per run)

| Scenario | Master tree 8d7bec73c, 256 runs: old / new | Tree `c5082605b`, 128 runs: old / new |
| --- | ---: | ---: |
| `war-golden-horde-thin` (the attack check) | 69% / 71% | 0% / 11% |
| `war-golden-horde` | 84% / 65% | 0% / 24% |
| `war-golden-horde-hold` (the Swiss hold) | 14% / 33% | 100% / 75% |
| Suite mean (14 scenarios) | | 42.5% / 43.6% |

The old-preflight arm read 0% on tree `c5082605b` (unexplained: 68% on `98e01c48c`; the cause given in the learnings entry "reads 0% on master" was not verified), so the 95% to 67% drop that move-through caused is not recovered by this change: the new preflight leaves the thin check where it was (69% to 71%).

## 6,144 eight-player games (seeds 30001+, Hard, no limit)

| | Gene 0 | Gene 1 |
| --- | ---: | ---: |
| Mean turns | 304.9 | 305.0 |
| Captures a game | 18.8 | 19.3 |
| Over 600 turns | 12 | 8 |
| Longest | 818 | 764 |

Raw runs: `~/LegatumRuns/cloud/mw-gate-1023-011455` (benchmark).

# Removing tech boosts: era pacing, before and after (2026-10-17)

The user asked for boosts to go ("things that happen randomly and not things that affect strategic decision making"). Era lengths were calibrated with them, so the tech costs came down by the average discount boosts gave, era by era. Cloud (AWS spot, Graviton), Standard speed, 4 players, every seat Hard + Search (`--difficulty hard --search k=3,h=20,d=4`), medium maps, barbarians on, seeds from 2026101701.

| Run | Games | Source |
|---|---:|---|
| Before (master `c8e5b859c` plus a telemetry-only patch that records `BoostEarned` turns) | 256 | `nb-base-4p` |
| Pass 1 (boosts gone, costs scaled by the measured discount) | 256 | `nb-p1` |
| Pass 2 (final) | 512 | `nb-p2` |

## How often boosts were earned

Share of researched technologies whose boost was earned before the technology was learned (every empire in 256 games, about 1,000 per technology), and the discount that adds up to: 40% of the technology's cost times that share, weighted by cost within the era.

| Era | Boost earned, share of techs | Effective cost cut |
|---|---:|---:|
| Ancient | 10.6% | 4.3% |
| Classical | 24.8% | 9.9% |
| Medieval | 32.1% | 12.8% |
| Renaissance | 32.1% | 12.9% |
| Industrial | 44.7% | 17.9% |
| Modern | 45.5% | 18.2% |
| Future | 23.1% | 9.2% |

By technology (share earned): Combustion 89%, Gunpowder 78%, Ballistics 75%, Rifling 64%, Mathematics 62%, Electricity 61%, Flight 59%, Code of Laws and Chivalry 52% each, down to Philosophy 1.5%, Steel 1.2%, Recursive AI 0.6%. So the same boost was nearly free for one technology and out of reach for the next: the discount followed which boost a typical empire met by accident (Combustion's "40 total population", Gunpowder's "win 3 battles"), not a decision.

## Era entry, median winner (turn)

| Era entered | Before (256) | Pass 1 | After, final (512) | Change |
|---|---:|---:|---:|---:|
| Classical | 27 | 28.5 | 27 | 0 |
| Medieval | 63 | 64 | 63 | 0 |
| Renaissance | 103 | 106 | 103 | 0 |
| Industrial | 140 | 140 | 140 | 0 |
| Modern | 173 | 174 | 174 | +1 |
| Future | 200 | 203 | 204 | +4 |

95% bootstrap intervals of the medians are about plus or minus 1 turn up to Industrial and plus or minus 3 for Future (the before cell is 197 to 205.5); the Future change is inside the noise. Median game length 243.5 before, 242 after (mean 251.9, 249.8; 90th percentile 310, 302). Every game ended in a VP win both times; every winner entered Modern, 95% entered Future.

The turns the task quoted (about T32, 64, 100, 132, 162, 189) are the numbers of the 2026-10-10 calibration; master has drifted since (median game 243 turns today against 220 then; other rule and AI changes landed in between), so "current pacing" here is what master measures today.

## Costs

Base cost in Science; Standard charges twice this, after tech diffusion (unchanged: 10% per other empire that knows the technology, cap 50%). Each era was scaled by one factor; pass 2 moved a little cost from Renaissance to Medieval and Ancient and Modern down a touch.

| Era | Before (five techs) | Sum | After | Sum |
|---|---|---:|---|---:|
| Ancient | 13, 13, 17, 17, 21 | 81 | 12, 13, 15, 16, 19 | 75 |
| Classical | 27, 27, 31, 34, 38 | 157 | 24, 24, 28, 31, 34 | 141 |
| Medieval | 72, 76, 79, 83, 85 | 395 | 60, 63, 66, 68, 70 | 327 |
| Renaissance | 120, 124, 128, 131, 135 | 638 | 108, 112, 115, 119, 124 | 578 |
| Industrial | 210, 214, 217, 224, 229 | 1094 | 172, 176, 178, 184, 188 | 898 |
| Modern | 245, 248, 253, 260, 264 | 1270 | 194, 197, 201, 207, 211 | 1010 |
| Future | 360, 369, 378, 387, 396 | 1890 | 327, 335, 343, 351, 360 | 1716 |

## Notes

- A first-order rule worked: scale each era's costs by (1 minus its measured effective cut). Pass 1 landed within 3 turns everywhere; pass 2 was a rebalance between neighbouring eras (what one era gained the next lost, so later entries moved with them).
- Not measured: 8-player lobbies (the pacing there was 30/64/101/135/168/200 in 2026-10-10 and the per-era cuts are the same scale).
- The research-order patience the AI had for nearly earned boosts is gone with them (`crates/ai/src/research.rs`).

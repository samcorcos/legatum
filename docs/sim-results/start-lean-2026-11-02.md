# Start positions: floor only, and why the faction lean was parked (2026-11-02)

Recalibration of `astra/start-positions` (Codex, 2026-11-01). Its faction lean gave Germany, Sparta and the Aztecs about 23% more radius-2 Production than master (20.2 against 16.4) and cost the human seat 7.5 points (360 wins). Designer's rule (`legatum-design`, "Start positions: a production floor plus a faction lean"): fitting, never better. Pillar 8: best faction win rate at most 1.5x the worst.

**Result: ship the Production floor with the lopsided-site scoring and India's wide-land swap; leave the faction lean out.** Floor-only has no start under the floor (master: 5.9% of starts), keeps the faction ratio at master's on the current engine (3.17 against 3.25) and returns the human seat to master's level (+0.8 pp). A gentler lean (8% quality, 2 Production) keeps Production within 1.4% of the mean but did not lift any weak faction and read slightly worse than master (3.32, wider spread). It is commit `a412c2534` on `claude/start-lean` if the designer wants it back.

Variants (one engine, same factions and seeds across variants):

- **master**: no floor, no lean, India's wide-land swap.
- **floor**: the Production floor (two Production units above the plains baseline within radius 2) and the lopsided-site penalty in the start quality; India's swap as on master; no other faction lean.
- **lean 8/2**: floor, plus a swap of the fair sites between non-Indian seats. A seat may only take a site within 8% of the quality and within 2 Production (radius 2) of the site it was picked for. Fit is the kind of terrain: tiles with 2+ Production (Sparta, Germany, Aztecs), hills and mountains (Switzerland), grassland and plains (Mongols), forest (Celts); Byzantium and Athens have none.
- **lean 10/4**: the same with a wider band, to show where the band stops being gentle.

## Win rates, AI only

`legatum-sim --players 8 --difficulty hard`, 8 players, Standard (48x32), no panics in any run. Interval is Wilson 95%; the ratio interval is a game-level bootstrap. Noise alone (all factions equal) gives a best/worst ratio of about 1.11 at 6,000 games (95th percentile 1.18).

### Current engine (master `a8463a971`, after the expansion genes), 6,000 games per variant

Seeds 9,400,000-9,402,999 and 9,500,000-9,502,999.

| Faction | master | floor | lean 8/2 |
| --- | ---: | ---: | ---: |
| Switzerland | 6.9% [6.3, 7.7] | 7.3% [6.6, 8.0] | 6.3% [5.7, 7.0] |
| India | 10.2% [9.4, 11.1] | 10.0% [9.2, 10.9] | 10.1% [9.3, 10.9] |
| Byzantium | 12.6% [11.7, 13.5] | 12.6% [11.8, 13.6] | 12.7% [11.8, 13.6] |
| Germany | 8.7% [7.9, 9.4] | 7.5% [6.8, 8.2] | 7.9% [7.2, 8.7] |
| Celts | 6.3% [5.6, 6.9] | 6.3% [5.6, 6.9] | 6.6% [5.9, 7.3] |
| Mongols | 15.7% [14.7, 16.7] | 16.3% [15.3, 17.3] | 14.9% [13.9, 15.8] |
| Sparta | 16.9% [15.9, 17.9] | 18.2% [17.2, 19.3] | 18.5% [17.5, 19.6] |
| Athens | 20.4% [19.3, 21.5] | 19.8% [18.8, 20.9] | 20.9% [19.8, 22.0] |
| Aztecs | 14.8% [13.9, 15.8] | 14.4% [13.5, 15.4] | 14.6% [13.7, 15.6] |
| **Best / worst** | **3.25** (Athens / Celts) | **3.17** (Athens / Celts) | **3.32** (Athens / Switzerland) |
| Bootstrap 95% of the ratio | 2.94-3.69 | 2.82-3.61 | 3.03-3.76 |
| SD of the nine rates | 4.55 pp | 4.75 pp | 4.91 pp |
| P(ratio <= master's) | | 0.66 | 0.37 |
| P(SD <= master's) | | 0.20 | 0.05 |

### First engine (Codex's base, `4df5189`/`adc86f5d1`), 6,000 games per variant

Seeds 9,200,000-9,201,999 and 9,300,000-9,303,999. The wider band is included here.

| Faction | master | floor | lean 8/2 | lean 10/4 |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 6.7% | 6.5% | 6.2% | 5.9% |
| India | 10.3% | 9.2% | 9.7% | 9.5% |
| Byzantium | 12.3% | 12.7% | 12.9% | 11.5% |
| Germany | 8.0% | 7.6% | 8.4% | 8.1% |
| Celts | 6.9% | 6.7% | 7.6% | 7.6% |
| Mongols | 15.3% | 15.1% | 13.6% | 13.6% |
| Sparta | 18.6% | 18.1% | 19.2% | 21.3% |
| Athens | 18.9% | 20.7% | 19.1% | 19.1% |
| Aztecs | 15.4% | 15.8% | 15.8% | 16.0% |
| **Best / worst** | **2.82** | **3.17** | **3.09** | **3.63** |
| SD of the nine rates | 4.53 pp | 4.94 pp | 4.59 pp | 5.09 pp |
| P(ratio <= master's) | | 0.09 | 0.16 | 0.00 |
| P(SD <= master's) | | 0.04 | 0.41 | 0.01 |

What this says:

- **Master misses Pillar 8 by a factor of two on both engines** (2.82, then 3.25): Athens and Sparta near 17 to 20%, Switzerland and the Celts near 6 to 7%. The imbalance is faction strength; the start layer neither causes nor cures it. Codex's 3.32 on 192 games was inside the noise of that gap.
- **Neither start variant is shown to move the ratio.** The two engines disagree on which is closer to master (floor: 3.17 against 2.82, then against 3.25; lean 8/2: 3.09 then 3.32), and every pairwise difference of the ratio has P between 0.09 and 0.66, so none is significant on its own. The pooled reading: floor-only is never worse than master by more than noise; the lean is slightly wider on the current engine (SD P 0.05) and gains nothing.
- **The lean does not lift the weak factions.** Hills and forest starts do not make Switzerland or the Celts win (Switzerland -0.5 and -0.6 pp, Celts +0.7 and +0.3 pp on the first and current engine); it mostly shuffles the strong ones (Sparta +1.7 pp, Mongols -0.8 pp on the current engine).
- **The wide band is worse, as expected.** Lean 10/4 gives Sparta +2.7 pp and the largest ratio (3.63; P 0.00).

## Start quality (4,000 Standard maps, 32,000 starts per variant)

Seeds 0-1,999 and 9,200,000-9,201,999, the runner's own faction draw (`start_metrics`). Quality spread is the best minus the worst seat on each map under that variant's own quality formula, so master's and the branch's spreads are on different scales; both stay within the 24-point limit. Yield spread is the same ruler for every variant: Food plus Production over radius 3 with radius 2 counting triple, best minus worst seat.

| Measure | master | floor | lean 8/2 | lean 10/4 |
| --- | ---: | ---: | ---: | ---: |
| Mean quality spread | 11.79 | 13.93 | 13.94 | 13.91 |
| Max quality spread | 24 | 24 | 24 | 24 |
| Mean yield spread | 19.8 | 31.6 | 31.6 | 31.6 |
| Starts under the Production floor (fewer than 2 tiles of 2+ Production) | 1,898 (5.93%) | 0 | 0 | 0 |
| Mean radius-2 Production, all seats | 16.43 | 17.47 | 17.47 | 17.47 |
| Largest faction deviation from that mean | 1.3% | 1.1% | 1.4% | 3.1% |
| Discarded map attempts | 434 | 477 | 468 | 469 |

The floor adds about one Production to every start and removes all dead starts. It also widens the plain food-plus-production spread between seats from 19.8 to 31.6, because the lopsided-site penalty lets two sites with the same yield total differ in quality. That is a real cost of the floor itself (the lean adds none), and the first place to tune if the spread turns out to matter.

Mean radius-2 Production by faction (difference from the all-faction mean in brackets):

| Faction | master | floor | lean 8/2 | lean 10/4 |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 16.57 (+0.8%) | 17.35 (-0.7%) | 17.53 (+0.3%) | 17.71 (+1.4%) |
| India | 16.22 (-1.3%) | 17.27 (-1.1%) | 17.27 (-1.2%) | 17.27 (-1.2%) |
| Byzantium | 16.39 (-0.3%) | 17.52 (+0.3%) | 17.41 (-0.4%) | 17.08 (-2.2%) |
| Germany | 16.63 (+1.2%) | 17.45 (-0.1%) | 17.53 (+0.3%) | 17.86 (+2.2%) |
| Celts | 16.37 (-0.4%) | 17.55 (+0.5%) | 17.39 (-0.5%) | 17.06 (-2.3%) |
| Mongols | 16.34 (-0.6%) | 17.39 (-0.4%) | 17.28 (-1.1%) | 17.11 (-2.0%) |
| Sparta | 16.42 (-0.1%) | 17.47 (-0.0%) | 17.57 (+0.6%) | 17.89 (+2.4%) |
| Athens | 16.60 (+1.0%) | 17.63 (+0.9%) | 17.55 (+0.5%) | 17.23 (-1.4%) |
| Aztecs | 16.37 (-0.4%) | 17.59 (+0.7%) | 17.72 (+1.4%) | 18.01 (+3.1%) |

The lean moves the kind of terrain, not the amount (tiles of each faction's style within radius 2):

| Faction (style) | master | floor | lean 8/2 | lean 10/4 |
| --- | ---: | ---: | ---: | ---: |
| Sparta / Germany / Aztecs (tiles with 2+ Production) | 5.8-5.9 | 6.3 | 7.0-7.1 | 7.4 |
| Switzerland (Hills, Mountains) | 2.83 | 3.13 | 4.76 | 5.34 |
| Celts (Forest) | 3.48 | 3.65 | 5.18 | 5.62 |
| Mongols (Grassland, Plains) | 11.79 | 11.02 | 13.28 | 14.16 |

### Band tuning (same maps, no games)

| Band (quality / Production) | Largest faction deviation | Sparta production tiles | Switzerland hill tiles | Celts forest tiles | Mongols open tiles |
| --- | ---: | ---: | ---: | ---: | ---: |
| floor (no lean) | 1.1% | 6.25 | 3.13 | 3.65 | 11.02 |
| 5% / 0 | 1.2% | 6.42 | 3.54 | 4.04 | 11.56 |
| 5% / 1 | 1.2% | 6.67 | 4.11 | 4.61 | 12.38 |
| 5% / 2 | 1.3% | 6.88 | 4.51 | 4.95 | 12.94 |
| 5% / 3 | 1.5% | 7.06 | 4.78 | 5.17 | 13.36 |
| 8% / 2 | 1.4% | 7.01 | 4.76 | 5.18 | 13.28 |
| 10% / 4 | 3.1% | 7.38 | 5.34 | 5.62 | 14.16 |
| no band | 8.4% | 7.93 | 5.66 | 5.87 | 15.00 |

8/2 keeps every faction within 1.4% of the mean (ceiling 5%) while raising style tiles by 12% (Sparta) to 52% (Switzerland) over the floor. 10/4 triples the deviation to 3.1% and moves Sparta's win rate by 2.7 pp, so it is not gentle.

## Human-seat benchmark

`hard_vs_human --arms` with the frozen built-in arm, `LEGATUM_HVH_RESEED_DICE=1`, seed 310001, 256 reseeded salts on each of the 8 fixed maps (2,048 paired games), plain Hard. Scored with `quality_gate.compare(role='all-seats')`, which labels every start-rule comparison **role-mixed**: starts change for the human seat and every opponent, so the seat-win change is not a measure of AI strength.

| Candidate | Control | Human-seat wins, control | Candidate | Paired change | Descriptive 95% | Simultaneous (3 looks) |
| --- | --- | ---: | ---: | ---: | --- | --- |
| **floor only** (`a2ca3e979`) | current master `a8463a971` | 449 (21.9%) | 465 (22.7%) | **+0.78 pp** | [-1.5, +3.1] | [-2.6, +4.2] |
| lean 8/2 (`80663cf8e`, merged with master) | current master | 449 | 428 (20.9%) | -1.03 pp | [-3.4, +1.3] | [-4.5, +2.5] |
| lean 8/2 (`a412c2534`) | `adc86f5d1`, before the expansion genes | 513 (25.1%) | 443 (21.6%) | -3.42 pp | [-5.8, -1.0] | [-7.0, +0.2] |
| Codex's first lean (`68ae527`) | `4df5189` | 513 | 360 (17.6%) | -7.47 pp | [-9.8, -5.2] | [-10.3, -4.7] |

Floor-only per fixed map against current master (wins of 256):

| Fixed map | master | floor only | Change (pp) |
| --- | ---: | ---: | ---: |
| 8.3 Switzerland win | 105 | 9 | -37.5 |
| 8.4 Sparta win | 36 | 103 | +26.2 |
| 8.4 Japan win | 45 | 62 | +6.6 |
| 8.4 Celts win | 19 | 3 | -6.3 |
| 8.5 Mongols win | 2 | 60 | +22.7 |
| 8.5 Switzerland win | 22 | 19 | -1.2 |
| 8.6 Germany loss | 110 | 54 | -21.9 |
| 8.6 Sparta win | 110 | 155 | +17.6 |

Each fixed scenario regenerates its starts under the new rules, so single maps swing by tens of points in both directions; only the pooled number is meaningful.

## Reproduction

`crates/sim/examples/start_metrics.rs` (`--seed S --games N`) prints the map measures; the win rates come from `legatum-sim --players 8 --difficulty hard --games N --seed S --threads T` on the build box. The lean variants during the sweep were selected by a temporary `LEGATUM_START_VARIANT` switch in `mapgen.rs`, removed before commit; the current-engine floor run is the merged tree with the one `lean_starts` call removed. Master numbers reproduce Codex's (937 starts under the floor in seeds 0-1,999, 513 human-seat wins on `adc86f5d1`).

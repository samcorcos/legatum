# Decadence proportional to the lead, and on every income (2026-10-17, `claude/decadence-proportional`)

The user's two rulings: "all things that feel like income, production, etc should be taxed by decadence" and "it should be proportional to how far ahead each civ is." The coach had found that the saturating curve `45 L / (L + 45)` made the runner-up pay nearly what the leader paid (18% against 20.5%), and that Swiss interest replaced about 80% of the Gold decadence took from one empire. Rules: `.claude/skills/legatum-rules/references/game-rules.md` section 6.

**New rule.** Rate = `L x c / 100` percent (nearest percent, integer maths), `L` the lead over the median living empire (Power, trailing 30 turns, unchanged), `c` = 40 (`decadence-slope=40`; 0 is off). No cap. It is taken from every city yield as before (Production, Gold, Science, and growth) and now also from Switzerland's treasury interest, as its own breakdown line. The old curve is behind `legacy-decadence` (`decadence-max` / `decadence-k` set it).

## Setup

Hard + Search in every seat, Standard, medium map, barbarians on, same seeds in every cell (4p 2026101001+, 8p 2026102001+), 256 games per cell and lobby (512 per cell), c8g.16xlarge spot. **Base** is master at `ebb7e564a` (the current rule: saturating curve, interest exempt) with only the read-only telemetry added, so the max and runner-up rates are measured on both. The 8p slope-50 cell ran on the rebased tip, which differs from the others by unrelated master commits (docs and the Byzantium forced-peace research notes); no rule in the sweep is affected. Commands: `tools/sim/overreach-v3-sweep.sh`, `tools/sim/vp_v2_report.py` (new rows: runner-up rate, highest rate seen).

## The sweep (4p + 8p pooled, 512 games per cell)

| Metric | Base (old curve) | c 25 | **c 40 (landed)** | c 50 | c 60 |
|---|---:|---:|---:|---:|---:|
| Median game length (turns) | 250.5 | 231 | 240 | 247 | 254.5 |
| No winner (%) | 0 | 0.2 | 0 | 0 | 0.4 |
| T50 leader wins (%) | 41.8 | 49.1 | 46.1 | 43.2 | 45.3 |
| T100 leader wins (%) | 49.4 | 55.2 | 51.4 | 47.1 | 47.8 |
| Lead changes per game | 9.69 | 8.26 | 9.00 | 9.63 | 10.22 |
| Last lead change (% of game, median) | 89.2 | 87.1 | 88.5 | 91.1 | 89.8 |
| Top Power over median, T100 (%) | 34.3 | 43.2 | 36.6 | 33.2 | 31.8 |
| Top Power over median, T150 (%) | 33.6 | 44.8 | 37.0 | 32.6 | 30.7 |
| Top empire's lead, T100 median / p90 (%) | 28 / 63 | 38 / 75 | 33 / 64 | 30 / 56 | 28 / 56 |
| Top empire's decadence, T100 median / p90 (%) | 17 / 26 | 10 / 19 | 13 / 26 | 15 / 28 | 17 / 34 |
| Top empire's decadence, mean over samples (%) | 16.7 | 10.3 | 14.1 | 16.9 | 18.6 |
| Runner-up's decadence, mean (%) | 9.5 | 5.0 | 6.8 | 8.3 | 9.2 |
| Highest rate seen in a game: median / p90 (%) | 24 / 32 | 16 / 28 | 22 / 39 | 26 / 45 | 31 / 49 |
| **Highest rate seen in any game (%)** | **40** | 64 | **87** | 101 | 294 |
| Games that ever reach 50% / 75% / 100% | 0 / 0 / 0 | 2 / 0 / 0 | 11 / 4 / 0 | 37 / 6 / 2 | 52 / 6 / 1 |
| Units deserted for unpaid overreach per game | 2.01 | 1.58 | 1.56 | 1.93 | 1.73 |
| Era objectives scored, eras 2 to 7 (% of entrants) | 64 / 64 / 55 / 53 / 43 / 34 | 61 / 60 / 53 / 49 / 39 / 33 | 64 / 63 / 55 / 52 / 41 / 36 | 65 / 63 / 57 / 54 / 43 / 35 | 66 / 65 / 59 / 54 / 44 / 35 |

Sampling error is about 2 points on a win share and 0.4 on lead changes.

## Findings

- **The chosen slope.** c = 40 is the user's own anchor (a 30% lead pays 12%, 60% pays 24%, a leader twice as far ahead as the runner-up pays twice the rate) and it keeps the tail short: 2% of games ever pass 50%, none pass 100%. Every race metric is inside the noise of the old curve (T100 leader wins 51.4% against 49.4%, lead changes 9.0 against 9.7, game 240 turns against 250); Power compression is slightly weaker (top over median 36.6% against 34.3%) because a typical 30% lead pays 13% where the old curve took 17%.
- **c 50 is the closest match to the old curve's mean** (top empire's mean rate 16.9% against 16.7%, Power spread 33.2% against 34.3%, lead changes 9.63 against 9.69), at the price of a longer tail (7% of games pass 50%, two touch 100%). It is one knob away (`decadence-slope=50`); c 60 is too steep (a game stalled at 600 turns with a 294% rate, which is zero yields and no way back), c 25 too weak (Power spread 43%).
- **The rate settles, except for runaway conquerors.** The median game's highest rate is 22%, the p90 39%. The large values are one pattern: an empire that snowballed to 18 to 30 cities (Sparta in the games I opened) while the median Power, the mean of the middle two empires, stayed small, so its lead over the median kept growing. Seed 2026101200 (4p, 364 turns, Sparta 25 cities against 7, 3 and 1; its lead was already 82% at turn 200) ended with a peak rate of 87%; seed 2026101011 (Sparta 30 cities, two rivals eliminated) peaked at 81%. Power is measured after decadence, so the loop does shrink the lead, but not fast enough when the pack stays small. The cut cannot take more than a yield (`decadence_cut` clamps), so a rate over 100% is a dead economy, not a negative one. No cap was added: a question for the lead (below).
- **The runner-up pays about half what the leader pays, not nine tenths.** Mean over samples 6.8% against 14.1% (old curve 9.5% against 16.7%, 57%). The ratio is 48% for c 40 because the runner-up's lead is usually much smaller than the leader's, which is the proportional rule working; a leader exactly twice as far ahead pays exactly twice (unit test `decadence_is_proportional_to_the_lead`).
- **Interest is small next to the cities.** Swiss interest is 9% of a Swiss seat's gross Gold at T100 to T200 (mean 2.1 to 4.6 Gold a turn against city Gold 21 to 49), so the cut is about 1% of its Gold at a 13% rate; a hoarder is the case it bites (1,000 Gold at 24% loses 2 Gold of 10). Win shares by faction are recorded, not tuned (Switzerland 16.9% base, 14.6% at c 40, within noise).

## Human-seat benchmark (AI-impact gate)

`tools/sim/hard-vs-human.sh`, all seats (a rule change applies to every seat, so there is no seat-only or opponents-only arm; the same reasoning as the Swiss-interest change), 8 maps, same salts, master control against the branch:

| Brain | Salts per map | Control | Proportional decadence | Difference |
|---|---:|---:|---:|---:|
| Hard | 256 | 30.1% (28.5 to 31.5) | **33.4%** (31.9 to 34.9) | +3.4 pp |
| Hard + Search | 64 | 27.7% (24.6 to 31.1) | **32.8%** (29.5 to 35.9) | +5.1 pp |

Hard did not get worse; it gained on the two Sparta maps (8.6: 28% to 55%; 8.4: 85% to 93%) and lost a little on Switzerland 8.5 (49% to 45%). Milestone ratio 0.33 to 0.37.

Rerun on the branch rebased onto master's phase 1 AI genes (Hard, 256 salts, control is that master): 31.0% (29.4 to 32.5) against 32.1% (30.3 to 33.8), +1.1 pp. Sparta 8.6 again gains (32% to 55%); Switzerland 8.5 falls (55% to 39%), the one map where the interest cut may bite (not investigated).

## War scenarios, attack check

`tools/sim/war-scenarios.sh --check` (The Golden Horde comes, Hard takes Aarau): control 44% (113 of 256), branch 39% (100 of 256) over the same 256 seeds, paired 95% interval -8.6 to -2.0 points (128 runs: 45% and 40%; unchanged after the rebase). Above the 30% floor, below the 43 to 50% baseline band. A rule change, not an AI change; flagged to the lead.

## Open question for the lead

A runaway conqueror can pay a rate over 50% (11 of 512 games at c 40, the highest 87%) and, at steeper slopes, over 100%. The user rejected hard caps and the loop is meant to be the limit, but it fails when the median empire collapses. Two non-cap options, not built: measure the lead against the mean of the empires that are not the leader, or against the median of the living empires with a floor of one third of the leader's Power. Neither is a cap on the rate; both change what "ahead of the pack" means once the pack is mostly dead.

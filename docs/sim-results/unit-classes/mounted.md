# The mounted fix (2026-10-31)

The designer: "i basically never buy things from that class until there are tanks." The fix, the Civ 6 way, keeps the triangle and changes the numbers:

- **Pre-Modern mounted units stand 130% of their era's melee line** (`data::MOUNTED_STRENGTH_PERCENT`): Horseman 26, Cataphract (and the Keshig) 33, Knight 42, Cuirassier 57, Cavalry 79 (were 18, 24, 32, 45, 64). Costs unchanged (110, 150, 204, 278, 378: 1.30x to 1.38x the melee line). Tank and Autonomous Armour unchanged.
- **The class bonus is 40% of the unit's base strength** (`combat::CLASS_BONUS_PERCENT`, was a flat +18): Spearman +8 ... Rifleman +24; Horseman +10 ... Cavalry +31.
- **Mounted units charging a city lose 33% of base strength** (`combat::MOUNTED_CITY_PENALTY_PERCENT`), so the stronger riders don't replace the melee line as city-takers.

Tools: `combat_scenarios --mounted --games 6` (`crates/sim/examples/combat_scenarios/mounted.rs`: duels, equal-Production armies with the Production exchange, and every melee/ranged/mounted mix against four archetypes), and a `combat_lab --repeats 16 --stride 5 --factions generic` subset (2,016 city cells x 16 seeds x 5 policies) per variant. Raw reports: [mounted-before.txt](mounted-before.txt), [mounted-after.txt](mounted-after.txt).

## The grid (Human policy, open ground, equal Production, Ancient to Industrial)

Line > horse: the spear line's win share and the exchange (mounted Production lost per melee Production lost). Horse > gunline: the riders against a third-melee, two-thirds-ranged army. Dominant mix: a melee/ranged/mounted mix that beats all four archetypes at 60%+ in some era (the red flag).

| Variant (mounted %, bonus %) | Line > horse exchange | Horse > ranged exchange | Horse > gunline win | Dominant mix |
|---|---|---|---|---|
| **before** (at/below line, +18 flat) | 3.9 to 5.5x (massacre) | 2.6 to 3.5x | 0 to 8% | none |
| 125, 30 | 1.25 to 1.48x | 2.6 to 7.1x | 97 to 100% | **4/1/2, Renaissance and Industrial** |
| 125, 40 | 1.62 to 2.27x | 3.2 to 9.1x | 95 to 100% | none |
| 125, 50 | 1.76 to 3.92x | 3.9 to 11.7x | 35 to 100% | none |
| 130, 30 | 1.15 to 1.28x (spears lose 7 to 23%) | 2.6 to 8.1x | 100% | none (worst case 10 to 47%) |
| **130, 40 (chosen)** | **1.44 to 1.76x** | **3.5 to 10.3x** | **96 to 100%** | **none** |
| 130, 50 | 1.67 to 2.98x | 4.5 to 11.7x | 77 to 100% | none |
| 135, 30 | 0.96 to 1.20x (spears lose up to 52%) | 3.2 to 9.1x | 100% | none (worst case 8%) |
| 135, 40 | 1.37 to 1.51x | 3.9 to 11.7x | 100% | **4/1/2, Industrial** |
| 135, 50 | 1.62 to 2.28x | 5.1 to 11.7x | 100% | none |

Ranged > line and gunline > line do not depend on these numbers (0 to 100% and 41 to 100% by era, as before).

**Why 130/40:** the spear line still wins every era 100% and trades 1.4 to 1.8x per Production (a good trade, not the 4 to 5.5x massacre), and that ratio is the flattest across eras of any variant, which is what a percentage is for. Riders now crush ranged (3.5 to 10x) and beat a screened gunline in the open, where they lost every time before. No mix beats every archetype in any era. A lower bonus (30%) or a stronger rider (135%) lets riders beat spears or a 4/1/2 mix dominate; a higher bonus (50%) brings the massacre back late (3x).

Duels (mean rolls, open ground, full health), after: a Horseman charging a Spearman deals 28 and takes 32 (was 13 and 67); a Spearman charging a Horseman deals 32 and takes 28. A Horseman charging an Archer deals 92 and takes 10 (unchanged); a Cavalry charging a Field Cannon kills it (222).

Best responses (top-3 mixes against each archetype): mounted Production share 35 to 39% (28 to 37% before). Against a gunline the best answer is now riders (1/0/5, 0/0/6); against riders it is the pure line (8/0/0); against the line it is ranged-heavy. Rock-paper-scissors at army level.

## City assaults (combat lab, storm policy / best of five, cities in sight)

Equal Production: four melee (Ancient 320) against three mounted (330), six melee against four mounted.

| Variant | Melee 4 vs mounted 3 | walled | Melee 6 vs mounted 4 | walled |
|---|---|---|---|---|
| before | 46 vs 22% | 26 vs 13% | 59 vs 35% | 45 vs 20% |
| 130/40, no city penalty | 45 vs 65% | 26 vs 44% | 59 vs 73% | 46 vs 61% |
| 130/40, -20% | 45 vs 56% | 26 vs 34% | 59 vs 66% | 46 vs 48% |
| 130/40, -25% | 45 vs 53% | 26 vs 28% | 59 vs 64% | 46 vs 45% |
| **130/40, -33% (chosen)** | **45 vs 49%** | **26 vs 22%** | **59 vs 60%** | **46 vs 38%** |

Without a city rule the stronger riders became the best city-takers (73% against the melee line's 59%), which breaks "city assaults still need the melee line". At -33% a rider charges a city a little below its era's melee line (Horseman 18 against the Spearman's 20): level with the melee line on open cities (its 4 moves) and below it against Walls. Melee and melee-plus-shooter armies are unchanged by every variant.

## Limits

- Equal-Production arena fights, one order per unit per turn, Human policy (the Hard policy plays these duels timidly). Six seeds per cell, four start distances, both arena sides.
- The combat lab sizes armies by count, so the table pairs counts of about equal Production; Mongol and Sparta attackers were not in the subset.
- Modern and Future units are outside the triangle and unchanged.

## Merged-branch measurement (2026-10-31, `claude/mounted-on` = the rebalance plus master 8c61139ca)

Files: [merged-measure/](merged-measure/) (`benchmark-paired.txt`, `paired.py`, `arms-ctl.json`, `census-before.json`, `census-after.json`). Master and the branch were built the same way and played the same salts; the rule applies in every seat, so this is the all-seats measure.

**Human-seat benchmark** (`hard_vs_human`, one `ctl` arm, 8 maps x 256 salts = 2,048 games per binary, seed 290001, `LEGATUM_HVH_RESEED_DICE=1`, rivals plain Hard, 4 spot servers each, about $0.05 a batch):

| | Seat wins | Rate |
|---|---|---|
| master 8c61139ca | 431 / 2,048 | 21.04% |
| branch | 423 / 2,048 | 20.65% |

Paired by (slice, map, salt): **-0.39 pp, 95% [-2.44, +1.66]** (map-stratified bootstrap; the normal interval is +-2.06). Both won 196, master only 235, branch only 227. Neutral: the rebalance neither helps nor hurts the seat's win rate.

| Map | master | branch | diff |
|---|---|---|---|
| 8.3 Switzerland | 27 | 28 | +1 |
| 8.4 Sparta | 148 | 133 | -15 |
| 8.4 Japan | 38 | 39 | +1 |
| 8.4 Celts | 17 | 21 | +4 |
| 8.5 Mongols | 8 | 2 | -6 |
| 8.5 Switzerland | 17 | 7 | -10 |
| 8.6 Germany | 43 | 55 | +12 |
| 8.6 Sparta | 133 | 138 | +5 |

Each map has 256 games, so a single map moves by about 10 either way from noise alone (the fixed maps share their first-turn layout); only the pooled figure is a measurement. Stalls and invalid orders: the 50-game branch smoke in verify reported panics 0, stalls 0 (invalid orders are not counted by that harness). In the benchmark, two branch games on the Japan map (salts 290003 and 290087) ran to the 1,200-turn safety cap with every empire alive; master had none (its longest ended at turn 921), and the branch had two other long games that ended normally (one elimination win at 634). Not investigated further.

The seat (Hard in the human's place) now builds 9.8 mounted units a game against 6.3 and puts 29.4% of its Production in mounted against 19.9%. Units lost a game (29.5 against 29.6), cities captured a game (2.37 against 2.35) and game length (255.6 against 255.4 turns) are unchanged.

**Unit census** (`unit_census`, 128 eight-player all-Hard games, seed 9700001, completed land units before the Modern era):

| Class | Units before / after | Share of units | Share of Production |
|---|---|---|---|
| Melee | 11,039 / 9,964 | 50.3 -> 43.9% | 40.3 -> 34.7% |
| Ranged | 7,512 / 7,250 | 34.3 -> 31.9% | 42.2 -> 38.5% |
| **Mounted** | **3,377 / 5,493** | **15.4 -> 24.2%** | **17.5 -> 26.9%** |

Use (attacks made before the Modern era, `Event::Battle`; units lost, `Event::UnitDestroyed`):

| Class | Attacks on units before / after | Attacks on cities before / after | Lost before / after |
|---|---|---|---|
| Melee | 10,435 / 9,347 | 3,200 / 3,203 | 7,840 / 7,377 |
| Ranged | 23,648 / 22,179 | 3,915 / 3,463 | 5,076 / 5,093 |
| Mounted | 2,984 / 8,056 | 774 / 1,942 | 2,228 / 3,580 |

A rider made 1.1 attacks before and 1.8 now, and 0.66 of them were lost either way, so riders are used more and no more recklessly. Melee city attacks did not move (3,200 to 3,203): the city penalty did its job, the extra rider city attacks (774 to 1,942) came out of the ranged line's (3,915 to 3,463). Mounted units made 9.8% of all pre-Modern city attacks before and 22.6% now, yet the share of a rider's own attacks that hit a city stayed the same (20.6% before, 19.4% now): they do the same job, more of it. No class dominates: ranged stays the largest Production share (38.5%), melee second (34.7%), mounted third (26.9%). Game length 233.6 against 229.0 turns on average. The Modern-and-later mix moved too (ranged 29.2 -> 37.1% of units; melee 61.1 -> 54.5%) from 600-odd units, so treat that as noise-level.

**Combat lab on the merged branch:** `combat_scenarios --mounted --games 6` is byte-identical to [mounted-after.txt](mounted-after.txt): duels, equal-Production armies and every mix against the four archetypes are as recorded above, so the 130/40/-33 results hold after the merge.

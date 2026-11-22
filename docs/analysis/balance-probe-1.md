# Balance probe 1

2026-09-26. Gameplay source: `f59bb2546a7cdc4df4120b81bbcc5baaf2edb058`. Analysis only; no gameplay, AI-default, or production-workspace changes.

The factions have similar observed win rates. The larger imbalance is between building a defensive empire and actually conquering one: scripted games average 88.5 battles but only 0.091 city captures. Titles provide most of the winner's advantage over the runner-up; Relay Uplink frequently supplies the last points. A turn-50 lead is valuable but far from decisive. Scripted AI overwhelmingly beats RandomBrain, including when their seat assignments are reversed.

The requested runs were executed exactly:

```sh
cargo run --release -p legatum-sim -- --games 1000 --seed 1 --label probe-1000
cargo run --release -p legatum-sim -- --games 500 --seed 5000 --brain mixed --label probe-mixed
```

The first run covers seeds 1–1000; the second covers 5000–5499. Both use eight independent empires, a 48×32 map, and a 150-turn limit. The original reports took 19.2s and 6.7s after compilation, respectively, on 16 workers. Sources: [scripted report](../sim-results/2026-09-26-probe-1000.md), [scripted records](../sim-results/2026-09-26-probe-1000.json), [mixed report](../sim-results/2026-09-26-probe-mixed.md), and [mixed records](../sim-results/2026-09-26-probe-mixed.json).

A separate [analysis harness](../../tools/analysis/README.md) replayed all 1,500 seed/configuration pairs with the existing brains and rules to observe events, scoring, and turn-50 standings. Every final player field, outcome, turn count, winner, and elimination event matches the original JSON. Another 1,100 games test AI knobs and reversed brain assignments. The [scripted aggregates](../sim-results/2026-09-26-probe-1000-analysis.json), [mixed aggregates](../sim-results/2026-09-26-probe-mixed-analysis.json), [paired comparisons](../sim-results/2026-09-26-probe-comparisons.json), compressed per-game telemetry, and [SHA-256 manifest](../sim-results/2026-09-26-probe-manifest.json) are included alongside this report.

Faction win rates count wins per initial seat; draws and eliminated seats remain in the denominator. Intervals are 95% percentile bootstrap intervals from 10,000 resamples of whole games, preserving dependence among competing seats and repeated factions. Binary game-level intervals below use Wilson scores. These characterize variation across these seed populations, not every possible map or human strategy.

| Faction | Scripted wins / seats | Win rate, 95% CI | Mixed wins / seats | Win rate, 95% CI |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 319 / 2,666 | 11.97% [10.91, 13.04] | 168 / 1,334 | 12.59% [11.05, 14.10] |
| Japan | 309 / 2,667 | 11.59% [10.53, 12.66] | 143 / 1,333 | 10.73% [9.27, 12.26] |
| Byzantium | 299 / 2,667 | 11.21% [10.15, 12.26] | 168 / 1,333 | 12.60% [11.09, 14.13] |

The nominal equal-seat benchmark is 12.5% when every game has a winner. With the scripted run's draws, the observed average is 11.59%. The largest observed faction difference, Switzerland minus Byzantium, is only 0.75 percentage points; its paired game-bootstrap CI is −1.02 to +2.55 points. All three pairwise difference intervals include zero. This does not justify a faction nerf. Mixed faction totals also combine two very different brain strengths, so they are a secondary balance check.

Outcomes and duration show a substantial late-game tail:

| Outcome | Scripted, 1,000 games | Mixed, 500 games |
| --- | ---: | ---: |
| VP target | 769 (76.9%) | 407 (81.4%) |
| Highest VP at turn limit | 158 (15.8%) | 72 (14.4%) |
| Draw | 73 (7.3%) | 21 (4.2%) |
| Conquest | 0 | 0 |
| Unfinished | 0 | 0 |
| Panic | 0 | 0 |

| Turn distribution | Mean | Min | P10 | P25 | Median | P75 | P90 | Max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Scripted | 130.17 | 65 | 103 | 122 | 133 | 148 | 150 | 150 |
| Mixed | 123.24 | 64 | 87 | 104 | 131 | 144 | 150 | 150 |

Scripted lengths fall into 51–75 / 76–100 / 101–125 / 126–150 turns in 3 / 88 / 215 / 694 games. Mixed counts are 12 / 98 / 114 / 276. Exactly 239 scripted games (23.9%) and 97 mixed games (19.4%) reach turn 150, including eight and four VP-target finishes on that turn. Percentiles use the simulator's rounded order-statistic convention, not interpolation.

VP sources tell different stories about accumulating points, separating from rivals, and triggering victory. Across the 927 scripted winners, terminal VP breaks down as follows:

| Source | Winner VP total | Mean per winner | Share of winner VP | Mean advantage over runner-up |
| --- | ---: | ---: | ---: | ---: |
| Titles | 3,267 | 3.52 | 35.6% | +2.615 |
| Milestones | 4,075 | 4.40 | 44.4% | +0.618 |
| Faction goal | 1,846 | 1.99 | 20.1% | +0.045 |

Titles account for 79.8% of the mean 3.278-point winning margin. Every scripted winner has FourCities, SixTechs, and TwentyPopulation; 923/927 have their faction goal. Much of this permanent score is shared with their strongest rival. Removing titles from every player's terminal score leaves the actual winner strictly ahead in only 316/927 games (34.1%); removing milestones leaves 859/927 ahead, and removing faction goals leaves 899/927 ahead. These are static score decompositions, not alternate-rule simulations or estimates of causal win-rate changes. Runner-up ties are selected by population, then seat ID.

The scoring ceiling explains the title dependence: milestones provide at most five permanent VP, faction goals two, and each title one. At a target of ten, even a fully banked empire needs three titles. In the 769 scripted VP-target victories, winners hold three / four / five titles in 420 / 104 / 245 games. All 241 VP winners without Relay Uplink need all five titles. See the [scoring rules](../../crates/core/src/scoring.rs) and [VP constants](../../crates/core/src/data.rs).

| Title | Held by scripted winners / 927 | Share of winners | Newly gained on a VP-winning turn / 769 |
| --- | ---: | ---: | ---: |
| MostAdvanced | 732 | 79.0% | 147 (19.1%) |
| MostPopulous | 708 | 76.4% | 46 (6.0%) |
| StrongestArmy | 651 | 70.2% | 132 (17.2%) |
| LargestEconomy | 630 | 68.0% | 55 (7.2%) |
| MostTerritory | 546 | 58.9% | 37 (4.8%) |

MostAdvanced compares recent science output, not researched-tech count. StrongestArmy measures health-weighted standing strength, not combat success. Its prominence therefore does not imply conquest is important.

At least one title is newly gained on 390/769 winning turns (50.7%); a milestone on 420 (54.6%); a faction goal on four (0.5%). Categories overlap when sources arrive together. Relay Uplink alone appears among the positive gains on 377 winning turns (49.0%), and is the only positive source change on 338 (44.0%). It is the most common immediate trigger. Among all scripted seats, relay completion has median turn 132, versus 67 for FourCities, 76 for SixTechs, and 103 for TwentyPopulation. Mixed games lean more heavily on titles: winners average 4.00 title VP, 3.88 milestone VP, and 2.00 goal VP; Relay is gained on 144/407 VP-winning turns (35.4%).

Faction-goal difficulty is unequal even though total wins are similar. Switzerland complete their goal in 2,521/2,666 seats (94.6%), Byzantium in 2,605/2,667 (97.7%), and Japan in only 684/2,667 (25.6%). Conditional completion medians are turns 87, 72, and 89, respectively. Japan's six-city requirement is a much stronger filter, but 305/309 Japan winners pass it. These completion times are censored by games ending; they are not unconditional expected completion times.

Eliminations are rare, not literally zero. The scripted report rounds four eliminations in 1,000 games to “0.00 per game.” They occur in four games: 0.4%, 95% CI 0.16–1.02%. Mixed has 38 eliminations in 36 games (7.2% of games); all 38 eliminated seats use RandomBrain. No game ends by conquest.

| Combat measure | Scripted | Mixed |
| --- | ---: | ---: |
| Battles per game, mean; P10 / median / P90 | 88.49; 38 / 82 / 149 | 132.27; 49 / 123 / 221 |
| Battles against city HP per game | 8.18 | 25.05 |
| Battles absorbed by city garrisons per game | 18.34 | 11.12 |
| City captures, total / per game | 91 / 0.091 | 139 / 0.278 |
| Games with a city capture | 81/1,000 (8.1%) | 116/500 (23.2%) |
| Combat units destroyed per game, excluding Scouts | 11.45 | 14.49 |
| Eliminations per game | 0.004 | 0.076 |

A battle here is an individual `Battle` event, including Scout encounters and ranged shots. It is not a war or a whole siege. All games contain battles. Scripted first combat has median turn seven, but the first direct city-HP attack has median turn 83 among the 758 games with one. Ranged fire supplies 76.5% of scripted battles and 86.0% of direct city attacks. Only 1,149/88,487 battles (1.30%) are melee against city HP.

Typical armies are small. Excluding Scouts and Settlers, scripted empires have mean 1.87 units at turn 50, median two, P10–P90 one to three. At game end, they have mean 5.23, median five, P10–P90 three to eight; the whole-game living-empire-turn mean is 3.00. Final combat units are 46.2% Troopers, 38.1% Rangers, and 15.7% Soldiers. Final cities average 4.13 per empire. Capturing one city rarely eliminates an empire: the [elimination rule](../../crates/core/src/rules/units.rs) requires losing both every city and every Settler.

The strongest evidence points to offensive deployment and coordination as the primary bottleneck:

- Scripted combat units spend 79.1% of unit-turns on their own city tiles and 97.3% within two hexes of an own city; only 0.58% are adjacent to an enemy city. Hold orders constitute 89.8% of submitted combat-unit actions. These are state/order observations, not planner-intent labels.
- The [military planner](../../crates/ai/src/military.rs) reserves defenders and one garrison per city before considering attacks. Marching needs at least two healthy free units and stops if any own city has a positive visible threat. Such a threat exists in 20.6% of scripted empire-turns; an enemy city is known in 84.9%, so missing enemy-city information alone cannot explain the inactivity.
- The [production target](../../crates/ai/src/production.rs), `1 + cities * (20 + military + aggression / 2) / 70`, gives all three default factions five combat units at four cities. Four garrisons leave one free unit, below the two-unit marching minimum. Raising either tested knob commonly crosses that discrete threshold.
- Across scripted games, 218 distinct cities reach zero HP during combat, but only 88 distinct cities are ever captured (91 captures include recaptures). Thus at least 130 cities reach zero and still never change owner before the game ends. Reaching zero HP is insufficient by itself: ranged fire cannot capture, and this points to missing melee follow-through as well as any damage bottleneck.

Cities do add attrition, but the observations do not show universal defensive invulnerability. Only 20.4% of direct city battles hit walled cities. The median observed melee hit deals 42 HP and receives 21; the median ranged hit deals 27. Target selection favors weak or already wounded cities, so these numbers do not establish that every city is easy to attack. Successful captures take median 13 turns from the first battle on the city tile, P10–P90 three to 40; this includes interruptions, garrison fights, and earlier fighting before recaptures, rather than measuring continuous siege duration.

The [combat math](../../crates/core/src/combat.rs) deals 30 base damage at equal strength: roughly four hits for a 100-HP unit or seven for a 200-HP city before wounds, rolls, retaliation, and healing. That is not intrinsically dozens of turns of active fighting. However, a full-health population-five walled city has strength `10 + 2*5 + 10 = 30`; a strength-15 Ranger deals 16 damage at the central roll. Without an adjacent enemy, the city's ten-point healing reduces that first turn's net damage to six. Weak ranged sieges can be slow, especially without melee units to stop healing. Overall: AI assembly/capture behavior is the leading explanation, with city HP/healing an amplifier; the precise causal contribution of each city constant remains untested.

Matched experiments on seeds 1–200 use exactly the same rules and change only exposed AI knobs for every faction:

| AI setting | Battles/game | Captures/game | Total eliminations | Final army/seat | Mean turns | VP-target share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Default, matched baseline | 87.35 | 0.100 | 0 | 5.24 | 128.54 | 78.0% |
| Aggression 75 | 178.43 | 0.335 | 7 | 5.81 | 130.79 | 80.0% |
| Military 65 | 203.76 | 0.430 | 6 | 6.05 | 134.24 | 75.0% |
| Both | 263.74 | 1.010 | 23 | 6.29 | 135.74 | 62.5% |

Paired 95% bootstrap intervals for extra captures/game are +0.235 [0.135, 0.340], +0.330 [0.220, 0.445], and +0.910 [0.740, 1.080]. The combined change raises mean duration by 7.21 turns [3.98, 10.40], and draws rise from 6.5% to 10.0%. Aggression affects both attack thresholds and desired army size, so it does not isolate willingness to attack from production. These experiments establish that AI knobs can substantially change conquest with current city rules; they do not establish a finished balance prescription.

Snowballing was measured after resolving turn 50, not at the start of it. Every requested game survives that long. Draws count as no leader win, and tied leaders are handled explicitly:

| Turn-50 definition | Scripted | Mixed |
| --- | ---: | ---: |
| Unique leader eventually wins | 330/779 = 42.4% [38.9, 45.9] | 219/402 = 54.5% [49.6, 59.3] |
| Winner belongs to tied leader set, among tied games | 123/221 = 55.7% [49.1, 62.1] | 71/98 = 72.4% [62.9, 80.3] |
| Winner belongs to any turn-50 leader set, all games | 453/1,000 = 45.3% | 290/500 = 58.0% |

If ties are broken by choosing a leader uniformly at random, expected prediction accuracy is 38.6% scripted and 50.7% mixed. This avoids arbitrarily crediting the lowest seat ID. Scripted unique leaders with one-point leads win 113/306 games (36.9%); two-point leads win 92/254 (36.2%); leads of three or more win 125/219 (57.1%). A lead is predictive, but 449/779 unique scripted leaders do not win. This measures persistence, not a causal effect of VP itself; it does not justify a general anti-snowball penalty from this probe alone.

The mixed run's brain gap is much larger than any observed faction gap:

| Brain | Wins / seats | Seat win rate, 95% CI | Mean final VP | Cities | Techs | Combat units |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Scripted | 478 / 2,000 | 23.90% [23.36, 24.27] | 5.497 | 4.723 | 8.396 | 6.070 |
| Random | 1 / 2,000 | 0.05% [0.009, 0.281] | 1.555 | 2.138 | 6.896 | 2.488 |

Brain intervals come from game-level win probabilities divided by four seats, avoiding an independent-seat assumption. Scripted wins 95.6% of all mixed games and 478/479 decided games (99.8%); the remaining 21 games draw. The per-seat difference is 23.85 percentage points. The within-game mean VP difference is +3.942 [3.850, 4.036]. Scripted takes 120 cities from RandomBrain and loses one to it; Random's only win is seed 5030, seat three, turn 144, by VP target.

The requested mixed mode always assigns scripted brains to even seats. To check that confound, a second 500-game population reverses parity on the same seeds: scripted wins 484, random one, and 15 draw. Averaging both assignments within seed gives a VP advantage of +3.889 [3.831, 3.949]. The gap survives the seat swap. The two assignments are dependent and are not counted as 1,000 independent maps in that interval. This is evidence against RandomBrain, not a measure of strength against human players or another competent strategy.

Validation: `cargo fmt --all` passed; `cargo clippy --workspace --all-targets -- -D warnings` passed with no warnings; `cargo test --workspace` passed all 182 tests (AI 27, core 130, simulator 14, WASM 11), with zero failures or ignored tests and zero doctests. The standalone harness also passes formatting and strict Clippy. All 1,500 baseline terminal records and elimination sequences match, and the Python aggregators compile and validate their accounting. The requested runs did not enable per-turn determinism hashes; final-record agreement is the verification claim here.

Limitations: the 200-seed knob studies are smaller than the main probe; terminal score subtraction and completion timing are not causal balance tests; battles against selected targets do not measure all siege opportunities; current telemetry cannot assign a precise share of lost attacks to each planner stage. Eight AI free-for-all empires are not the standard two-human cooperative team setup. City-stat changes below are hypotheses, not measured improvements. Similar faction win rates argue for keeping faction goals and VP targets unchanged until a subsequent controlled run identifies a problem.

Prioritized tuning experiments, with exact proposed values; none are applied in this branch:

1. Set `Knobs::for_faction(...).aggression` in [crates/ai/src/knobs.rs](../../crates/ai/src/knobs.rs) from Switzerland/Japan/Byzantium **35/40/45 → 75/75/75**, leaving the other knobs unchanged. Attack ratios become 100% instead of 116/114/112%, and desired armies also grow. This is the best first isolated experiment: matched captures rise 3.35× with a smaller duration shift than raising military alone. Verify captures, zero-HP capture conversion, faction CIs, and the duration tail on the full seed ranges before adopting it.
2. Test `Knobs::for_faction(...).military` **40/45/45 → 65/65/65** separately before combining it with step one. It supplies enough units to leave two available beyond four garrisons and raises matched captures 4.3×. Expect more field forces and combat, but diverted production and longer games. The combined setting has evidence for much more conquest and also for slower VP completion; do not treat both changes as an automatic package.
3. After the AI experiment, test `CITY_HEAL` in [crates/core/src/data.rs](../../crates/core/src/data.rs) **10 → 5**, with all other city constants fixed. It should make unsupported ranged damage persist: the example Ranger's initial net damage rises from six to eleven. It will not make ranged units capture cities or release garrisons. Accept only if damaged cities convert to meaningful captures without making random, unsupported bombardment dominant.
4. If coordinated attacks remain too slow after that, test `CITY_HP` in the same file **200 → 160**, independently of the healing change. This removes 20% of the initial HP pool while retaining base strength 10, strength/population 2, and Walls +10. Expect fewer damaging hits per capture. This ranks below AI changes because at least 130 cities already reach zero HP without capture; it also needs a defense check before use in human games.
5. Once military pacing is settled, test `building_stats(BuildingKind::RelayUplink).cost` in the same file **400 → 360** while retaining its two VP. Earlier Relay completion may reduce the 23.9% scripted turn-150 tail; it is the last positive source on nearly half of VP-winning turns. This is lower priority because it shortens the window for conquest and may intensify the existing Relay race. Compare duration, draws, title concentration, and capture opportunity before accepting it.

# Balance V10 iteration: paid goals and refined drawbacks

Recommend **bundle D for Sam's decision only**: it meets the requested **≤1.6× observed ratio in all four primary cohorts**. Its tuning ratios are **1.491× Quick / 1.164× Standard**; its combined 1,000-game held-out ratios are **1.571× / 1.273×**.

**Pillar 8 remains unmet**: D's Quick held-out ratio is 1.571× against the 1.5× target.

All **18,000 new games** are complete. All rules remain opt-in, and defaults are unchanged. The intervals below describe sampling uncertainty; they do not establish population-wide or human-play balance.

## Scope and method

This continues the [merged remeasurement](2026-10-01-balance-remeasure.md) on
`04cfebf` (the starting master for this worktree). No Rust rules, AI behavior,
or default setup is changed in this iteration. All candidates use the merged
`--rules-experiment` levers. Signature strengths, goal requirements, and every
other experiment remain unchanged.

The historical diagnosis reuses the previous study's measured baseline at
`6d6eff3`, rather than relabeling it as a fresh 4,000-game run. Its original report
checksums are verified, and enriched observations retain the previously omitted
build and elimination telemetry. The current iteration's campaigns use the frozen
`04cfebf` executable. Default equivalence and compiled-source provenance are
recorded separately below.
The reused baseline ratios are 2.594× Quick / 2.489× Standard on tuning and
2.300× / 3.107× on the original 500-game held-out cohorts. They are not
1,000-game validation measurements.

Every campaign uses scripted Hard, eight independent empires with one of each
faction in seed-shuffled seats, medium 48×32 maps, Default openings, normal
barbarians/diplomacy/resources/promotions/city fire, and no imposed turn limit.
The safety caps are 600 Quick and 1,200 Standard turns. Wins divide by every
scheduled game, including caps and draws. Quick and Standard are never pooled.

| Cohort | Inclusive seeds | Games per speed per candidate |
| --- | --- | ---: |
| Tuning | 3700001–3702000 | 2,000 |
| Held-out, combined | 3800001–3801000 | 1,000 |
| Previously inspected block | 3800001–3800500 | 500 |
| Added block | 3800501–3801000 | 500 |

The first 500 held-out seeds were already used in the prior study and in this
iteration's diagnosis. They are **reused validation seeds**, not an independent
holdout. Candidate parameters are frozen before opening their held-out results;
the extra 500 are reported separately. The requested combined 1,000-game gate
and the smaller, fresh block answer different questions. Selecting a bundle
from multiple held-out comparisons also requires care; no result here establishes
human-play balance or justifies automatically enabling an experiment.

Faction intervals are individual two-sided **95% Wilson score intervals**, not
simultaneous bounds across factions or bundles. Independent cohort differences
use Newcombe's Wilson construction. Paired rule effects use same-seed changes
and their unadjusted normal 95% intervals. Ratio intervals are descriptive
percentile intervals from 2,000 empirical game bootstrap draws; best and worst
factions are reselected each time, with no-winner games retained. The extrema
statistic is selection-biased and these intervals do not certify the pillar's
threshold. Recommendation and pillar gates below use observed cohort ratios.
These calculations treat seed outcomes as exchangeable games under the fixed
scripted policy; deterministic reruns are not additional independent samples.

The Claude design artifact referenced by `CLAUDE.md` could not be read through the available web
tool, and no connector for it was available. The supplied pillar 8 and checked-in
mechanisms govern these experimental proposals; no design adoption is claimed.

## Why the old Standard held-out ratio swung

The large changes in A are **Switzerland rising and Athens falling**.
Byzantium remains the minimum in both Standard cohorts. All previous 500-game
held-out observations are retained; nothing is dropped as an outlier.
A's Standard ratio is 1.955× on tuning (descriptive bootstrap 95% interval
1.763–2.421×) and 3.167× on held-out (2.297–5.000×). The intervals are wide;
the faction-level changes below identify what drives the difference.

| Faction | A tuning wins / 2,000, rate [95% CI] | A held-out wins / 500, rate [95% CI] | Held-out minus tuning, pp [95% CI] |
| --- | ---: | ---: | ---: |
| Switzerland | 265 (13.25%; 11.83–14.81) | 95 (19.00%; 15.80–22.67) | +5.75 [+2.19, +9.68] |
| Japan | 295 (14.75%; 13.26–16.37) | 68 (13.60%; 10.87–16.88) | -1.15 [-4.32, +2.45] |
| Byzantium | 156 (7.80%; 6.70–9.06) | 30 (6.00%; 4.23–8.44) | -1.80 [-3.97, +0.87] |
| Germany | 281 (14.05%; 12.60–15.64) | 83 (16.60%; 13.60–20.11) | +2.55 [-0.85, +6.35] |
| Celts | 270 (13.50%; 12.07–15.07) | 57 (11.40%; 8.90–14.48) | -2.10 [-5.05, +1.30] |
| Mongols | 261 (13.05%; 11.64–14.60) | 73 (14.60%; 11.77–17.96) | +1.55 [-1.67, +5.20] |
| Sparta | 165 (8.25%; 7.12–9.54) | 44 (8.80%; 6.62–11.61) | +0.55 [-1.98, +3.58] |
| Athens | 305 (15.25%; 13.74–16.89) | 50 (10.00%; 7.67–12.94) | -5.25 [-8.10, -1.94] |

The paired A-versus-default penalty for Byzantium is −7.95 percentage points on
tuning seeds and −8.00 on held-out seeds. Sparta loses −5.80 / −8.00 points.
Those are evidence of overcorrection even before considering uncertainty in the
highest-rate faction. A's added costs moved its weakest faction well below the
rest; doubling the validation sample cannot repair that mechanism.

Switzerland already run higher and Athens lower on the baseline held-out
seeds. The additional change in A's effect between cohorts is uncertain:
Switzerland' +3.10-point interaction and Athens' −3.20-point interaction
both have 95% intervals crossing zero. This does not establish a seed-dependent
rule defect. Nor do overlapping ratio intervals establish that A is balanced.
The exploratory global permutation test for A Standard gives p=0.0028
(10,000 shuffles; Monte Carlo SE 0.0005). The corresponding p values are 0.386
for baseline Standard and 0.338 for A Quick. After Holm adjustment across the
eight A Standard faction comparisons, Switzerland and Athens have
p=0.0084 and p=0.0184. These are post hoc diagnostics, not a new confirmation
study; adjustment does not cover every variant, speed or analysis choice. Sparse
no-winner cells are retained in the permutation test. The cohort-by-bundle
interaction intervals are Switzerland −1.00 to +7.20 points and Athens
−6.57 to +0.17 points.

Five hundred games are insufficient for a precise gate near 1.5×. At an observed
12.4% (62/500), the Wilson interval is 9.79–15.58%; at 12.5% (125/1,000), it is
10.59–14.69%. Even for a *prespecified* 15%-versus-10% pair, a normal log-ratio
interval around 1.5× is approximately 1.05–2.15× at n=500 and 1.16–1.93× at
n=1,000. Selecting the extrema adds uncertainty. This iteration uses 1,000 at
each speed to reduce noise, without presenting the larger sample as proof of
balance or treating the previous results as noise to discard.

## Why Germany and Mongols lose under Hard

These factions mostly **survive and lose the VP race**. Of 2,000 baseline tuning
games at each speed, Germany is eliminated twice on Quick (seed 3700492, turn
112; seed 3700723, turn 146) and never on Standard. Mongols is never eliminated
at either speed. Surviving losers number 1,855 / 1,865 for Germany and
1,837 / 1,836 for Mongols. Absence of elimination is not absence of military
pressure; the reports record only the observed games, ending at the winner's
victory or safety cap.

| Observation | Germany Quick | Germany Standard | Mongols Quick | Mongols Standard |
| --- | ---: | ---: | ---: | ---: |
| Eliminations / 2,000 | 2 | 0 | 0 | 0 |
| Mean final VP | 4.03 | 3.93 | 3.67 | 3.95 |
| Mean final cities | 3.01 | 3.22 | 3.94 | 4.25 |
| Goal completions / 2,000 | 914 | 827 | 531 | 790 |
| Median goal turn among completions | 115.0 | 229 | 104 | 195.0 |

Mean final VP per initial empire, by source:

| Source | Germany Quick | Germany Standard | Mongols Quick | Mongols Standard |
| --- | ---: | ---: | ---: | ---: |
| FactionGoal | 0.914 | 0.827 | 0.531 | 0.790 |
| SixTechs | 0.999 | 0.999 | 1.000 | 1.000 |
| FourCities | 0.546 | 0.603 | 0.669 | 0.736 |
| TwentyPopulation | 0.418 | 0.464 | 0.492 | 0.536 |
| RelayUplink | 0.793 | 0.716 | 0.511 | 0.434 |
| All live titles combined | 0.359 | 0.325 | 0.472 | 0.454 |

Germany builds its discounted infrastructure: roughly 2.7–2.9 Foundries
(Workshop-family completions), 2.6–2.8 Granaries and 2.4–2.6 Laboratories per
empire. It also pays for about 2.8 Settlers and 3.8–4.0 Rangers. Its first trained
Soldier arrives at median turn 77 / 150, and its second founded city at 37 / 72.5,
among empires reaching those milestones. Starting and captured units are excluded.
Its slow expansion and costly units are retained as identity costs. The reward
under test applies only after paying for the original active population-eight
city with all six original building families.

Mongols completes about 3.1 / 3.2 Markets and trains 3.9 / 3.8 Settlers,
4.5 / 5.0 Rangers and 3.9 / 4.2 Soldiers. Its second city arrives at median turn
30 / 56. Yet 838 / 655 games include at least three Market completions with no
network-goal completion. Build counts do not prove that those buildings coexist
in active cities, are properly spaced, or even remain owned. They support a
spatial-network bottleneck hypothesis, not a measured attribution among those
conditions. The candidate pays more for meeting the unchanged pairwise-eight-hex
network requirement; mobility, lack of terrain defense and default conditional
remote-commerce income remain intact.

Germany completes its goal in 45.70% / 41.35% of games, at median completion
turns 115 / 229. Mongols completes it in 26.55% / 39.50%, at turns 104 / 195.
Germany's wins with a completed goal are 142/143 Quick and 134/135 Standard;
Mongols's are 158/163 and 162/164. These correlations motivate testing the
payoff, but do not isolate goal value from the economy that made completion
possible. Every completion rate is censored at the game's end.

Build figures are completed-production events per initial empire, including
purchases and eliminated empires, not queue choices or final building stock.
VP figures are final-state averages: live titles are current holdings, milestones
and goals are banked awards. Player and team ledgers are independently reconciled.

## Candidate mechanism boundaries

C tests the two paid-goal reward increases together. Later bundles refine the extra
infrastructure and civilian-expansion costs that overcorrected A. Each selection
record retains its timestamp and the evidence available when chosen. The exact
flags are reproduced below. A candidate is eligible for a recommendation only
if **every one of its four combined cohort ratios is ≤1.6×**. Pillar 8 remains
**≤1.5×** in all four; the relaxed recommendation gate is not a replacement.

| Lever | Default | C | D | E |
| --- | ---: | ---: | ---: | ---: |
| Byzantium conventional-building price | 120% | 120% | 130% | 135% |
| Celts conventional-building price | 100% | 100% | 110% | 110% |
| Sparta extra Settler Production | +0 | +0 | +5 | +0 |
| Germany paid-city goal | 2 VP | 3 VP | 3 VP | 3 VP |
| Mongols network goal | 2 VP | 3 VP | 3 VP | 3 VP |
| Mongols conditional remote-commerce Gold | +2 | +2 | +2 | +2 |

Building prices are percentages of the unfactioned base cost before speed
scaling, with Great Works excluded. Byzantium's exploration rewards, Legionaries
and 3-VP goal remain intact. Celts retains hill yields and Cairn Singer strength.
Sparta retains infantry strength, wound immunity, healing and its 4-VP goal.
The two weak factions earn the larger award only by satisfying their existing
identity-specific goal requirements. These are the merged levers; this branch
adds analysis and evidence, with no rule implementation changes.

D reduces A's excessive building and Settler costs. E was fixed from the complete
C/D tuning results at 17:31:55 UTC, before opening any candidate held-out outcome.
D's Quick Byzantium rate remained 15.80%, while Sparta was 11.15% Quick and the
Standard minimum at 11.25%. E therefore deepens only Byzantium's conventional
building drawback by another five percentage points and removes D's additional
Sparta tax. Byzantium's 12.05% Standard rate in D made overcorrection at that
speed an explicit risk. No fourth bundle or held-out-driven parameter change
was made.

The reward-only bundle C raises Germany's paired tuning win rate by 4.75 points
on Quick (95% interval 3.79–5.71) and 5.05 on Standard (3.97–6.13). Mongols
gains 2.00 points (1.04–2.96) and 3.15 (2.05–4.25). Goal completion counts actually
fall slightly: Germany 914→895 Quick and 827→778 Standard; Mongols 531→515 and
790→783. The joint reward experiment improves conversion into wins without
relaxing either goal's requirements. It does not isolate each faction's causal
effect, and shorter games can censor later completions.

## Refined bundle results

Every rate is wins / scheduled games, including safety caps. Brackets are individual 95% Wilson intervals. The ratio reselects the highest and lowest rate within that cohort.

| Bundle | Quick tuning | Standard tuning | Quick held-out | Standard held-out | Worst cohort |
| --- | ---: | ---: | ---: | ---: | ---: |
| V10C | 1.778× | 1.308× | 1.979× | 1.612× | 1.979× |
| V10D | 1.491× | 1.164× | 1.571× | 1.273× | 1.571× |
| V10E | 1.377× | 1.340× | 1.404× | 1.617× | 1.617× |

### V10C: faction rates

| Faction | Quick tuning, n=2,000 | Standard tuning, n=2,000 | Quick held-out, n=1,000 | Standard held-out, n=1,000 |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 255 (12.75% [11.36, 14.28]) | 230 (11.50% [10.18, 12.97]) | 127 (12.70% [10.78, 14.91]) | 116 (11.60% [9.76, 13.73]) |
| Japan | 198 (9.90% [8.67, 11.29]) | 247 (12.35% [10.98, 13.86]) | 95 (9.50% [7.83, 11.48]) | 115 (11.50% [9.67, 13.63]) |
| Byzantium | 352 (17.60% [15.99, 19.33]) | 297 (14.85% [13.36, 16.48]) | 188 (18.80% [16.50, 21.34]) | 130 (13.00% [11.06, 15.23]) |
| Germany | 238 (11.90% [10.55, 13.39]) | 236 (11.80% [10.46, 13.29]) | 118 (11.80% [9.95, 13.95]) | 105 (10.50% [8.75, 12.55]) |
| Celts | 264 (13.20% [11.79, 14.75]) | 290 (14.50% [13.02, 16.11]) | 147 (14.70% [12.64, 17.03]) | 166 (16.60% [14.42, 19.03]) |
| Mongols | 203 (10.15% [8.90, 11.55]) | 227 (11.35% [10.03, 12.81]) | 103 (10.30% [8.57, 12.34]) | 122 (12.20% [10.31, 14.37]) |
| Sparta | 258 (12.90% [11.50, 14.44]) | 240 (12.00% [10.65, 13.50]) | 121 (12.10% [10.22, 14.27]) | 142 (14.20% [12.17, 16.50]) |
| Athens | 232 (11.60% [10.27, 13.08]) | 233 (11.65% [10.32, 13.13]) | 100 (10.00% [8.29, 12.02]) | 103 (10.30% [8.57, 12.34]) |

### V10D: faction rates

| Faction | Quick tuning, n=2,000 | Standard tuning, n=2,000 | Quick held-out, n=1,000 | Standard held-out, n=1,000 |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 261 (13.05% [11.64, 14.60]) | 258 (12.90% [11.50, 14.44]) | 132 (13.20% [11.24, 15.44]) | 130 (13.00% [11.06, 15.23]) |
| Japan | 212 (10.60% [9.33, 12.03]) | 262 (13.10% [11.69, 14.65]) | 111 (11.10% [9.30, 13.20]) | 127 (12.70% [10.78, 14.91]) |
| Byzantium | 316 (15.80% [14.27, 17.46]) | 241 (12.05% [10.70, 13.55]) | 154 (15.40% [13.30, 17.77]) | 110 (11.00% [9.21, 13.09]) |
| Germany | 265 (13.25% [11.83, 14.81]) | 256 (12.80% [11.41, 14.34]) | 136 (13.60% [11.61, 15.86]) | 117 (11.70% [9.85, 13.84]) |
| Celts | 248 (12.40% [11.03, 13.92]) | 262 (13.10% [11.69, 14.65]) | 121 (12.10% [10.22, 14.27]) | 128 (12.80% [10.87, 15.01]) |
| Mongols | 231 (11.55% [10.22, 13.03]) | 237 (11.85% [10.51, 13.34]) | 129 (12.90% [10.96, 15.12]) | 140 (14.00% [11.99, 16.29]) |
| Sparta | 223 (11.15% [9.84, 12.60]) | 225 (11.25% [9.94, 12.71]) | 119 (11.90% [10.04, 14.05]) | 127 (12.70% [10.78, 14.91]) |
| Athens | 243 (12.15% [10.79, 13.65]) | 259 (12.95% [11.55, 14.49]) | 98 (9.80% [8.11, 11.80]) | 120 (12.00% [10.13, 14.16]) |

### V10E: faction rates

| Faction | Quick tuning, n=2,000 | Standard tuning, n=2,000 | Quick held-out, n=1,000 | Standard held-out, n=1,000 |
| --- | ---: | ---: | ---: | ---: |
| Switzerland | 268 (13.40% [11.98, 14.96]) | 238 (11.90% [10.55, 13.39]) | 128 (12.80% [10.87, 15.01]) | 128 (12.80% [10.87, 15.01]) |
| Japan | 216 (10.80% [9.51, 12.24]) | 262 (13.10% [11.69, 14.65]) | 110 (11.00% [9.21, 13.09]) | 111 (11.10% [9.30, 13.20]) |
| Byzantium | 292 (14.60% [13.12, 16.22]) | 215 (10.75% [9.47, 12.18]) | 136 (13.60% [11.61, 15.86]) | 94 (9.40% [7.74, 11.37]) |
| Germany | 274 (13.70% [12.26, 15.28]) | 238 (11.90% [10.55, 13.39]) | 128 (12.80% [10.87, 15.01]) | 133 (13.30% [11.34, 15.55]) |
| Celts | 227 (11.35% [10.03, 12.81]) | 256 (12.80% [11.41, 14.34]) | 136 (13.60% [11.61, 15.86]) | 144 (14.40% [12.36, 16.71]) |
| Mongols | 212 (10.60% [9.33, 12.03]) | 235 (11.75% [10.41, 13.24]) | 112 (11.20% [9.39, 13.31]) | 131 (13.10% [11.15, 15.33]) |
| Sparta | 263 (13.15% [11.74, 14.70]) | 288 (14.40% [12.93, 16.01]) | 146 (14.60% [12.55, 16.92]) | 152 (15.20% [13.11, 17.56]) |
| Athens | 248 (12.40% [11.03, 13.92]) | 267 (13.35% [11.93, 14.91]) | 104 (10.40% [8.66, 12.45]) | 107 (10.70% [8.93, 12.77]) |

### Ratio uncertainty and outcomes

| Bundle | Speed | Cohort | Best / worst | Ratio | Bootstrap 95% interval | VP wins / games | Mean turns |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| V10C | Quick | tuning | Byzantium / Japan | 1.778× | 1.564–2.174× | 2000/2000 | 124.32 |
| V10C | Standard | tuning | Byzantium / Mongols | 1.308× | 1.257–1.631× | 2000/2000 | 240.11 |
| V10C | Quick | heldout | Byzantium / Japan | 1.979× | 1.696–2.635× | 999/1000 | 123.22 |
| V10C | Standard | heldout | Celts / Athens | 1.612× | 1.417–2.148× | 999/1000 | 241.27 |
| V10D | Quick | tuning | Byzantium / Japan | 1.491× | 1.311–1.801× | 1999/2000 | 126.16 |
| V10D | Standard | tuning | Japan, Celts / Sparta | 1.164× | 1.138–1.476× | 2000/2000 | 243.21 |
| V10D | Quick | heldout | Byzantium / Athens | 1.571× | 1.324–2.038× | 1000/1000 | 125.71 |
| V10D | Standard | heldout | Mongols / Byzantium | 1.273× | 1.183–1.689× | 999/1000 | 242.13 |
| V10E | Quick | tuning | Byzantium / Mongols | 1.377× | 1.274–1.695× | 2000/2000 | 124.65 |
| V10E | Standard | tuning | Sparta / Byzantium | 1.340× | 1.213–1.607× | 1999/2000 | 243.13 |
| V10E | Quick | heldout | Sparta / Athens | 1.404× | 1.279–1.887× | 1000/1000 | 126.55 |
| V10E | Standard | heldout | Sparta / Byzantium | 1.617× | 1.385–2.150× | 1000/1000 | 243.88 |

### Previously inspected seeds and the added 500

| Bundle | Speed | Original 500 ratio | Added 500 ratio | Combined 1,000 ratio |
| --- | --- | ---: | ---: | ---: |
| V10C | Quick | 2.022× | 2.366× | 1.979× |
| V10C | Standard | 1.580× | 1.812× | 1.612× |
| V10D | Quick | 1.471× | 1.681× | 1.571× |
| V10D | Standard | 1.622× | 1.196× | 1.273× |
| V10E | Quick | 1.473× | 1.646× | 1.404× |
| V10E | Standard | 2.000× | 1.617× | 1.617× |

## Decision limits and remaining work

D is the only bundle that qualifies and also has the lowest maximum observed
ratio across the four primary cohorts. It uses Byzantium building price 130%,
Celts building price 110%, +5 Sparta Settler Production, and 3-VP Germany and
Mongols goals. This is a proposal for the user; defaults remain unchanged.

C leaves Quick Byzantium too strong (18.8% held-out, against Japan's 9.5%). E
meets the observed 1.5× target in three cohorts but fails the recommendation gate
on Standard held-out: **1.617×**, Sparta 15.2% [13.11, 17.56] versus Byzantium
9.4% [7.74, 11.37]. That near miss is not rounded into a pass. E's two changes
relative to D were tested together, so this study does not isolate the effect of
the extra Byzantium building cost from removing the Sparta Settler surcharge.

Remaining pillar-8 gaps for that bundle:

- Quick heldout: 1.571×, Byzantium 15.40% versus Athens 9.80%.

For D, the added 500 seeds alone produce **1.681× Quick / 1.196× Standard**. Smaller samples have noisier selected-extrema ratios, but the combined result should not be read as independent confirmation that every unseen block meets the target.

The original and added 500-seed blocks are diagnostic splits, not additional primary gates. Their variation and the wide ratio intervals remain relevant to the decision. Selecting the lowest observed maximum does not establish a reliable advantage over every other bundle. No further parameter set was fitted to these validation results. Any subsequent confirmation should freeze the chosen rules and use fresh seeds.

The strongest reusable result is the paid-goal mechanism: Germany and Mongols mostly survive but lose on VP; paying more for their unchanged identity-specific requirements improves their conversion into wins. A's much larger generic costs overcorrected Byzantium and Sparta. The present bundles retain every signature strength and test smaller compensating costs.

## Non-VP outcomes

Every draw or safety cap remains in the denominator. No accepted game panicked. All player/team VP ledgers and faction-goal awards reconcile.

- `v10c-quick-heldout`: seed `3800045`, draw, turn 142.
- `v10c-standard-heldout`: seed `3800783`, stalled, turn 1200.
- `v10d-quick-tuning`: seed `3700145`, stalled, turn 600.
- `v10d-standard-heldout`: seed `3800071`, draw, turn 274.
- `v10e-standard-tuning`: seed `3701922`, stalled, turn 1200.

C's Quick draw at seed 3800045 has Mongols and Germany tied on 10 VP and 27 population; D's Standard draw at 3800071 has Mongols and Switzerland tied on 10 VP and 30 population. Both are consistent with the existing tie rule. The C Standard cap at 3800783 and D Quick cap at 3700145 each retain all eight empires alive, with maximum VP 9. The D cap seed ended in a Celts victory at turn 118 under both baseline and C. It is an unfinished game, not a panic; `quick-cap-comparison.json` retains that comparison.

## Checks and unchanged defaults

| Command or check | Exit | Result |
| --- | ---: | --- |
| `cargo fmt --all` | 0 | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Passed |
| `cargo test --workspace --release --no-fail-fast` | 0 | 1,213 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | 0 | 1,213 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | 0 | 14 passed, 0 failed, 0 ignored |
| Python balance-analysis checks | 0 | 6 passed |
| Final default-equivalence comparison | 0 | 16 paired full games; all 2,928 turn hashes and all game fields except wall time match |
| `git diff --check` and staged diff check | 0 | Passed |
| Conflict-marker scan | 0 | No markers in tracked or unignored text files |

The release test process exit was explicitly checked. The four ignored tests
are existing optional/expensive checks; no Rust source or ignored-test list was
changed. Rust checks used `CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2` and
`RAYON_NUM_THREADS=1`. C's tuning campaigns used three workers each while the
checks ran; subsequent campaigns used four each, with at most two campaign
processes at a time. The campaign audit verifies a maximum of eight requested
simulation workers. Build, test and parity commands are recorded separately.

No Rust source, Cargo manifest or lockfile is changed. The source audit compares
every Rust file under `crates/` to the frozen measurement build. Simulations run from a frozen
copy of the executable, because the workspace checks rebuild the ordinary
`target/release/legatum-sim` artifact. Executable hashes distinguish those builds
from the launch-time checkout metadata in `meta.source`.

The initial current-master default comparison passed 16 paired full games and
all 2,928 turn hashes, including internal `--determinism` repeats. A separate
historical comparison bridges the old `6d6eff3` baseline to the frozen `04cfebf`
build: the same 16 games and 2,928 turn hashes match. Its only report-schema
normalization removes the new, explicitly asserted `opening_handoff: null`
player field; wall-clock duration is excluded. The initial strict historical
record comparison stopped on that added field, not on a game-state mismatch.
The final comparison against the post-check release executable is recorded in
`default-parity-final.json`; it permits no report-field difference other than
wall-clock duration. Golden replay fixtures remain unchanged.

## Reproduction and evidence

Run one campaign-driver invocation at a time. The driver defaults to 2,000
tuning games and **500** held-out games for backward compatibility; this study
explicitly requests **1,000** held-out games. Its configurable `--threads` times
`--parallel` is capped at eight. All measurement invocations use `nice -n 10`;
receipts record the inherited priority rather than assuming the adjustment
succeeded in the managed sandbox.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
CARGO_BUILD_JOBS=2 cargo build --release -p legatum-sim
```

Use the exact frozen flags below. The driver converts `--flags` into the
simulator's `--rules-experiment` argument; omitted values retain their defaults.

```sh
python3 tools/analysis/balance_remeasure.py \
  --binary target/release/legatum-sim \
  --output target/balance-v10-iteration/reproduction --variant v10c \
  --heldout-games 1000 --threads 4 --parallel 2 \
  --flags 'germany-goal-vp=3,mongols-goal-vp=3'
python3 tools/analysis/balance_remeasure.py \
  --binary target/release/legatum-sim \
  --output target/balance-v10-iteration/reproduction --variant v10d \
  --heldout-games 1000 --threads 4 --parallel 2 \
  --flags 'byzantium-building-percent=130,celts-building-percent=110,sparta-settler-surcharge=5,germany-goal-vp=3,mongols-goal-vp=3'
python3 tools/analysis/balance_remeasure.py \
  --binary target/release/legatum-sim \
  --output target/balance-v10-iteration/reproduction --variant v10e \
  --heldout-games 1000 --threads 4 --parallel 2 \
  --flags 'byzantium-building-percent=135,celts-building-percent=110,germany-goal-vp=3,mongols-goal-vp=3'
```

Recompute a published cohort's faction rates, Wilson intervals, score audit and
selected-extrema bootstrap without simulating again:

```sh
python3 tools/analysis/summarize_balance.py \
  docs/sim-results/2026-10-01-balance-v10-iteration/v10d-quick-tuning.json.gz \
  --baseline docs/sim-results/2026-10-01-balance-v10-iteration/baseline-quick-tuning.json.gz \
  --bootstrap
```

Recompute the historical cohort and weak-faction telemetry diagnosis from the
published observations:

```sh
python3 tools/analysis/balance_iteration.py diagnose \
  --previous docs/sim-results/2026-10-01-balance-remeasure \
  --full-reports docs/sim-results/2026-10-01-balance-v10-iteration \
  --output target/balance-v10-iteration/reproduced-diagnosis
```

The [evidence directory](2026-10-01-balance-v10-iteration/) contains frozen
selections, the decision policy and unsealing receipt, campaign commands and
exit codes, per-game observations, faction/ratio summaries, goal and investment
telemetry, check logs, executable/source provenance and a checksum manifest.
The enriched baseline observations are reused measurements, not additional new
games. Candidate compact observations retain completed builds and eliminations
as well as goals, outcomes, final VP ledgers and seat assignments. Large combat,
research and scouting traces remain in the local full reports under
`target/balance-v10-iteration/runs/<label>/report.json.gz`, whose hashes are
preserved in the campaign receipts.

The independent study audit checks contiguous seed coverage, exactly one of
each faction in paired seats, scripted Hard/Default policies, experiment values,
all goal awards, player/team score reconciliation, held-out block reconstruction,
reported rates and campaign-worker overlap. The manifest protects artifact
integrity; it is separate from the game-state determinism checks.
The study-specific `audit-study.py` is preserved beside its output. Run it from
the repository root with the local campaign reports and frozen executables
present to repeat those checks.

## Fresh-seed Quick check (Claude, 2026-10-01 15:20)

Master ea66a2e, 8 players, Quick, scripted Hard, 2,000 games, seed 3900001 (no overlap with the tuning or held-out cohorts). Runs: ~/LegatumRuns/bundle-d-check/.

| Faction | Default rules | Bundle D |
| --- | --- | --- |
| Byzantium | 19.4% | 15.3% |
| Germany | 7.5% | 14.1% |
| Sparta | 14.8% | 12.5% |
| Switzerland | 13.9% | 13.6% |
| Celts | 13.1% | 10.8% |
| Athens | 10.8% | 12.4% |
| Mongols | 8.3% | 10.7% |
| Japan | 12.1% | 10.6% |
| **Best/worst ratio** | **2.59x** | **1.44x** |

Bundle D meets pillar 8's 1.5x on this fresh Quick cohort, so the 1.68x reading on the extra 500-seed cohort looks like noise.

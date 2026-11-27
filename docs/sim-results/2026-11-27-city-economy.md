# Economic city automation

The delegated Economy chooser is separate from Hard AI's production planner. It values Gold first, then Science and Production, with additional weight for avoiding starvation and relieving low production. Buildings, improvements and growth projects compete on estimated payoff before the game ends. Military units and defensive-only construction are not economic fallbacks.

## Experiment

`crates/ai/examples/city_economy_lab.rs` runs actual `resolve_turn` outcomes, not a replacement economy simulator. It creates eight controlled city archetypes on each of two map seeds: young and established fertile/hill cities, young plains, and late fertile/hill/plains cities. Population and territory are matched; six nearby deposits offer farms, pastures, quarries and mines. Early positions know the first economic technologies. Late positions know every technology and begin on turn 250. Every branch runs for 60 turns without purchases, research decisions, expansion, combat or opponent orders.

The final matrix covers Byzantium, Egypt, Germany and India: 64 positions × four policies, or 15,360 resolved turns. Two seeds relocate the controlled terrain and resources; they are reproducibility checks, not independent evidence of broad map robustness. These are purpose-built economic positions, including food-poor cities, not sampled complete games. Late positions start without buildings to exercise investment sequences; already-built cities are covered by the existing completed-city/project tests.

Policies:

- **automation:** the candidate's bounded payoff model.
- **gold-first:** weights direct building/improvement Gold, Science, Production and Food at 8/5/3/1, discounted by quoted construction time.
- **growth-first:** weights Food, Production, Gold and Science at 8/5/3/1.
- **production-first:** weights Production, Food, Gold and Science at 8/5/3/1.

The three simple policies build positive-yield buildings/improvements and then run Gold. They do not explicitly choose growth projects. Their labels describe their build rankings, not an exhaustive optimal policy for a faction.

The earlier chooser at master `8815b7516c3475a8bc3f66c7967965283ac303e3` was measured with the same harness for the 16 Byzantine positions. The expanded faction matrix compares the candidate against the three simple policies; it does not claim an old-chooser comparison for those factions.

CSV columns are cumulative **city** Gold, Science and Production yields over 60 turns, final population and number of improved resource tiles. Gold is not treasury income after empire administration or post-research conversion; Production is actual output, including Egyptian build progress. These columns are separate outcomes, not a win-rate or faction-balance score.

## What changed after measurement

The old chooser often opened peaceful cities with Walls or a Great Work and delayed ordinary income improvements. The first candidate replaced that with marginal yield forecasts, construction time, a 60-turn payoff window and the income forgone by not running Gold. It considers core-provided Revenue Office savings and narrowly identified idle-industry supply relief as well as direct yields.

The expanded matrix caught an overvaluation of Egyptian Grow. The first growth model treated extra Nile build capacity as a permanent benefit even when the city would simply run Gold. A young fertile Egyptian city grew to population 19 and produced 338 city Gold, versus over 830 under the simple investment policies. That model was rejected. Growth now values additional Nile build capacity at the actual Gold-project conversion rate when comparing the growth project's cost. Food/Grow uses the core's project food forecasts and next-claim preview; growth beyond the work radius is not treated as free productive territory. A regression test also checks that Grow remains available when it has a worthwhile claim.

The scoring weights are a practical policy, not proof of a globally optimal build order. Different cities and horizons favor different sequences. The model cannot anticipate combat, future research, purchases, future manual queues or complete faction victory plans. Improvement previews are marginal tile yields; broad empire tax interactions are not exhaustively forecast. No general Hard AI defaults, core rules or tape pins changed.

## Results

Cumulative output at seed 11 (seed 29 gave the same controlled-city totals):

| Byzantine city | Old Gold → new | Old Science → new | Old Production → new | Improved tiles, old → new |
|---|---:|---:|---:|---:|
| young-fertile | 60 → 284 | 282 → 312 | 300 → 344 | 0 → 2 |
| young-hills | 131 → 542 | 222 → 293 | 1102 → 1294 | 3 → 4 |
| young-plains | 66 → 434 | 239 → 348 | 800 → 920 | 4 → 4 |
| established-fertile | 60 → 284 | 368 → 398 | 300 → 344 | 0 → 2 |
| established-hills | 136 → 565 | 236 → 328 | 1134 → 1398 | 3 → 4 |
| late-fertile | 60 → 434 | 635 → 824 | 783 → 1240 | 0 → 6 |
| late-hills | 149 → 842 | 425 → 649 | 1840 → 2956 | 5 → 6 |
| late-plains | 60 → 548 | 847 → 991 | 1654 → 2523 | 1 → 6 |

The corrected Egyptian young fertile case produces **884 Gold / 458 Science**, against 836 / 282 for gold-first, 846 / 276 for growth-first, and 848 / 276 for production-first. It ends at population 8, rather than the rejected model's 19. Its sequence earns the cost of some growth without growing indefinitely. Late fertile Egypt produces 1,213 Gold versus the simple policies' 1,049–1,130.

There is no universal winner. On young plains in Egypt, automation yields 629 Gold and 305 Science; production-first yields 650 Gold and 261 Science. On late hills in Germany, automation yields 1,386 Gold / 520 Science / 2,945 Production, while growth-first yields 934 / 544 / 3,447. These are useful tradeoffs, not regressions hidden inside a single aggregate score. The candidate achieves the most Gold (ties included) in 16/16 Byzantine, 14/16 Egyptian, 14/16 German and 14/16 Indian positions; each count includes the two relocated copies of each archetype.

After the Round 1 revision excluded World Wonders from automation, the full 256-row matrix was rerun on implementation commit `0f29183e673b20b54f1d557c5f017e32a5b8735a`. The CSV is byte-for-byte identical to the recorded final matrix, including all outputs and first picks; the tables and comparisons above still describe the final policy. This does not establish that Wonder exclusion is neutral outside these controlled positions.

Raw data: [baseline](2026-11-27-city-economy-baseline.csv), [rejected Grow model](2026-11-27-city-economy-rejected-grow.csv), [final four-faction matrix](2026-11-27-city-economy.csv).

## Reproduce

```sh
CARGO_TARGET_DIR=target-city-economy cargo run -p legatum-ai --release --example city_economy_lab > city-economy.csv
cargo test -p legatum-ai --lib economic_automation
cargo test -p legatum-ai --lib production::automation_tests
```

To reproduce the old chooser comparison, use an isolated checkout of the baseline SHA, copy the example there and restrict its faction list to Byzantium. Use that checkout's own target directory. The baseline CSV's case names predate the faction prefix; its positions otherwise match the final Byzantine cases.

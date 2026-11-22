# Great Works and tall development: tuning comparison (2026-09-27)

All three Great Works now complete and appear in winning plans at both speeds and in held-out validation. Sustained compact winners rise from zero to **8/1,000 Quick, 1/500 Standard and 7/1,000 held-out Quick**. Median length is **154 / 288 / 153** respectively, and every faction's point estimate is within **6–20%** in each final cohort. All 2,500 final games finish by VP with **zero stalls, zero panics and 2,500/2,500 determinism passes**.

This establishes observed sometimes-winning paths for the scripted AI. Compact play is still uncommon, particularly at Standard, and is mostly a Germany plan. It does not establish equally strong routes, human playtest acceptance, permanent-team balance or population win rates safely inside the requested band. Athens and Celts remain near its edges. The held-out candidate was frozen before execution; no rule or AI tuning used its results.

## Causes and changes

Foundry was expensive in three ways at once. Its original prerequisite closure cost 515 base Science, and two complete industrial families plus the Work cost 1,440 base Production, versus Exchange's 435S/720P and Relay's 955S/500P. These are undiscounted totals from no infrastructure, excluding population growth and other economic investments; they are not elapsed-time forecasts. Only 257 of the 8,000 baseline Quick empires completed any Autoworks, and only 152 completed two. The old planner also added research and construction serially instead of scheduling their overlap. Price changes alone produced only six Foundries and one Foundry winner in 1,000 Quick games, with zero Standard Foundry winners. A single industrial host, paid-chain planning and coherent branch pacing are all part of the remedy.

The resulting undiscounted closures are Foundry **315S/740P**, Exchange **485S/620P**, Relay **675S/400P**. Foundry rewards developed production in one city; Exchange needs a two-city commercial network; Relay spends more research with less supporting construction. Their infrastructure continues to provide different economic benefits. They still grant the same two-point award and compete for one empire completion slot.

Population 8 was reachable; sustained tall planning was missing. The baseline already had 407 Quick and 132 Standard tall-only milestone claims, but no compact winners. The planner could abandon a city whose current food could not sustain population 8 even though Hydroponics could reach population 6 and unlock Arcology. It also targeted at least four cities and could queue a fourth settler while the third was still travelling. A test of restricting only already-owned city count missed that commitment. The final planner evaluates both cities' complete food paths and budgets queued/travelling settlers before consuming another city's population. Economic personalities can remain at three cities and then pursue the live titles needed to win. The population-8 threshold and one-point milestone are unchanged.

The faction interaction was a route-timing advantage. In baseline Quick, 191 of Celts's 221 wins used Exchange. Its population-5 cities already served its food/terrain goal and met the commercial population gate, while industrial and research routes waited behind expensive late prerequisites. The final candidate changes shared route pacing, not Celts's faction traits or goal; Celts now wins through Foundry 95, Exchange 43 and Relay 32, plus 16 without a Work. Its reached-goal median remains effectively unchanged at 110 → 109 turns. This supports the route interaction explanation, without identifying a separate causal effect for every cumulative edit.

Athens' existing goal specifically requires Relay Restoration and three laboratories. The late-victory gate postponed that plan even after committing to the lab network. Its full prerequisite closure is now prioritized earlier, and the industrial/energy/Relay prices fit the game's tempo. In Quick, the reached-goal median moves from 120 to 101 turns, with 731/1,000 goal completions in both cohorts; its win rate rises from 4.7% to 6.5%. Its science identity, weak treasury, unique yields and goal requirements remain intact. These timings condition on reaching the goal and do not count non-completers as fast finishes.

Pacing supplies the length recovery. The first price-only intervention moved medians from 169/318 to 161/308. Later route prices brought them to 152/294 before the final growth-path fixes; the final result is 154/288. Standard's median is 1.87x Quick, within the requested approximate doubling. No victory points, thresholds, safety caps or forced turn limits were reduced.

## Cohorts and evidence

Normal scripted brains, eight single-empire teams, one of each faction shuffled among seats, 48×32 maps, no turn limit, eight threads. Quick development uses seeds 700001–701000; Standard development uses 700001–700500; held-out Quick uses 910001–911000. Each game is played twice with per-turn determinism comparison and final save/load validation. Only first plays enter outcome statistics. Safety caps remain 600 Quick / 1,200 Standard; none were reached. Every initial faction seat, including eliminated empires, remains in the win-rate denominator.

The fresh merged baseline reproduces [the supplied slice evidence](2026-09-27-tech-bicycle-2-comparison.md). For context, the earlier pre-slice baseline had medians 156/299. All completed tuning iterations, including failures, are archived in the [implementation and checks record](2026-09-27-tech-bicycle-2-tuning-implementation.md), with exact commands, price changes, executable/source hashes, gzip-9 JSON, intentional fixture regeneration and native/WASM checks.

| Cohort | Report | Data |
| --- | --- | --- |
| Merged Quick | [Runner report](2026-09-27-tech-bicycle-2-tuning-baseline-quick.md) | [gzip-9 JSON](2026-09-27-tech-bicycle-2-tuning-baseline-quick.json.gz) |
| Merged Standard | [Runner report](2026-09-27-tech-bicycle-2-tuning-baseline-standard.md) | [gzip-9 JSON](2026-09-27-tech-bicycle-2-tuning-baseline-standard.json.gz) |
| Tuned Quick | [Runner report](2026-09-27-tech-bicycle-2-tuning-growth-paths-quick.md) | [gzip-9 JSON](2026-09-27-tech-bicycle-2-tuning-growth-paths-quick.json.gz) |
| Tuned Standard | [Runner report](2026-09-27-tech-bicycle-2-tuning-growth-paths-standard.md) | [gzip-9 JSON](2026-09-27-tech-bicycle-2-tuning-growth-paths-standard.json.gz) |
| Held-out Quick | [Runner report](2026-09-27-tech-bicycle-2-tuning-heldout-quick.md) | [gzip-9 JSON](2026-09-27-tech-bicycle-2-tuning-heldout-quick.json.gz) |

## Length and reproducibility

Quantiles are minimum / P10 / median / P90 / maximum; nearest rank uses `(p * (n - 1) + 50) // 100`. Only first plays enter counts. Safety caps and eliminated empires remain in denominators.

| Cohort | Games | Mean turns | Turn quantiles | VP outcomes | Stalls | Panics | Determinism |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Merged Quick | 1000 | 169.93 | 79 / 136 / 169 / 204 / 475 | 1000 | 0 | 0 | 1000/1000 |
| Merged Standard | 500 | 315.13 | 165 / 237 / 318 / 379 / 599 | 500 | 0 | 0 | 500/500 |
| Tuned Quick | 1000 | 155.28 | 77 / 121 / 154 / 186 / 408 | 1000 | 0 | 0 | 1000/1000 |
| Tuned Standard | 500 | 287.33 | 124 / 225 / 288 / 343 / 458 | 500 | 0 | 0 | 500/500 |
| Held-out Quick | 1000 | 156.43 | 77 / 124 / 153 / 194 / 391 | 1000 | 0 | 0 | 1000/1000 |

## Faction win rates

Wins / initial seats, including eliminated empires. The roughly 6–20% target is reported without rounding borderline results into the band.

| Faction | Merged Quick | Merged Standard | Tuned Quick | Tuned Standard | Held-out Quick |
| --- | --- | --- | --- | --- | --- |
| Japan | 81/1000 (8.1%) | 31/500 (6.2%) | 96/1000 (9.6%) | 39/500 (7.8%) | 104/1000 (10.4%) |
| Germany | 173/1000 (17.3%) | 86/500 (17.2%) | 154/1000 (15.4%) | 63/500 (12.6%) | 147/1000 (14.7%) |
| Mongols | 106/1000 (10.6%) | 36/500 (7.2%) | 106/1000 (10.6%) | 61/500 (12.2%) | 102/1000 (10.2%) |
| Sparta | 110/1000 (11.0%) | 80/500 (16.0%) | 108/1000 (10.8%) | 78/500 (15.6%) | 110/1000 (11.0%) |
| Byzantium | 73/1000 (7.3%) | 36/500 (7.2%) | 121/1000 (12.1%) | 52/500 (10.4%) | 114/1000 (11.4%) |
| Athens | 47/1000 (4.7%) | 27/500 (5.4%) | 65/1000 (6.5%) | 31/500 (6.2%) | 63/1000 (6.3%) |
| Switzerland | 189/1000 (18.9%) | 73/500 (14.6%) | 164/1000 (16.4%) | 77/500 (15.4%) | 164/1000 (16.4%) |
| Celts | 221/1000 (22.1%) | 131/500 (26.2%) | 186/1000 (18.6%) | 99/500 (19.8%) | 196/1000 (19.6%) |

## Great Work completions and winning paths

Completions count ProductionCompleted events, excluding captures and uncompleted starts. Winner counts refer to the eventual winning empire completing that route, without claiming that the Work alone caused victory. Each empire has at most one completed Work.

| Cohort | Path | Completions (all empires) | Winners | Completion-turn quantiles |
| --- | --- | --- | --- | --- |
| Merged Quick | PlanetaryFoundry | 0 | 0 | — |
| Merged Quick | WorldExchange | 1302 | 585 | 120 / 140 / 159 / 184 / 246 |
| Merged Quick | RelayUplink | 1209 | 258 | 128 / 147 / 166 / 198 / 259 |
| Merged Quick | No completed Work | — | 157 | — |
| Merged Standard | PlanetaryFoundry | 0 | 0 | — |
| Merged Standard | WorldExchange | 507 | 240 | 251 / 279 / 312 / 356 / 548 |
| Merged Standard | RelayUplink | 393 | 108 | 256 / 283 / 324 / 376 / 454 |
| Merged Standard | No completed Work | — | 152 | — |
| Tuned Quick | PlanetaryFoundry | 902 | 283 | 106 / 131 / 149 / 172 / 226 |
| Tuned Quick | WorldExchange | 222 | 101 | 125 / 139 / 157 / 177 / 222 |
| Tuned Quick | RelayUplink | 2207 | 490 | 104 / 120 / 142 / 169 / 294 |
| Tuned Quick | No completed Work | — | 126 | — |
| Tuned Standard | PlanetaryFoundry | 362 | 121 | 223 / 258 / 292 / 325 / 440 |
| Tuned Standard | WorldExchange | 83 | 46 | 259 / 277 / 316 / 343 / 372 |
| Tuned Standard | RelayUplink | 805 | 201 | 192 / 234 / 275 / 323 / 440 |
| Tuned Standard | No completed Work | — | 132 | — |
| Held-out Quick | PlanetaryFoundry | 875 | 272 | 111 / 131 / 148 / 172 / 255 |
| Held-out Quick | WorldExchange | 213 | 94 | 124 / 141 / 158 / 177 / 207 |
| Held-out Quick | RelayUplink | 2271 | 522 | 104 / 120 / 142 / 169 / 309 |
| Held-out Quick | No completed Work | — | 112 | — |

## Tall development

Tall-only claims own fewer than four cities and at least two population-8 cities at the first claim. Wide-only and both conditions are reported separately. Compact winners are tall-only claimants ending with at most three cities. Sustained compact winners additionally never owned more than three cities after any resolved turn; — means that cohort predates peak-city telemetry. These definitions do not treat a later wide empire as sustained tall play.

| Cohort | Tall-only claims | Wide-only claims | Both | Tall-claim winners | Compact winners | Sustained compact winners | Tall-claim turn quantiles | Tall-winning factions | Tall-winning Works |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Merged Quick | 407 | 5573 | 4 | 8 | 0 | — | 82 / 106 / 137 / 172 / 229 | Germany 3, Celts 5 | RelayUplink 2, WorldExchange 6 |
| Merged Standard | 132 | 2817 | 1 | 6 | 0 | — | 172 / 205 / 256 / 319 / 399 | Germany 1, Athens 1, Celts 4 | RelayUplink 1, WorldExchange 5 |
| Tuned Quick | 839 | 5069 | 13 | 38 | 8 | 8 | 82 / 99 / 123 / 156 / 213 | Germany 30, Celts 8 | PlanetaryFoundry 24, RelayUplink 4, WorldExchange 10 |
| Tuned Standard | 315 | 2581 | 1 | 13 | 1 | 1 | 156 / 191 / 232 / 282 / 347 | Germany 8, Switzerland 1, Celts 4 | NoWork 1, PlanetaryFoundry 9, RelayUplink 1, WorldExchange 2 |
| Held-out Quick | 825 | 5053 | 10 | 40 | 7 | 7 | 68 / 100 / 125 / 159 / 255 | Germany 26, Celts 14 | PlanetaryFoundry 26, RelayUplink 4, WorldExchange 10 |


## Compact winning examples

These examples never owned more than three cities after any resolved turn. Their tall point substitutes for the four-city point; it is not an additional award. All three Work choices occur among Quick compact winners. The table is illustrative, not an extra statistical sample.

| Cohort | Seed | Faction | Win turn | Cities | Population | Tall claim turn | Work | Winning live titles |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Tuned Quick | 700365 | Germany | 154 | 3 | 26 | 151 | RelayUplink | LargestEconomy, MostAdvanced, MostPopulous |
| Tuned Quick | 700538 | Germany | 152 | 3 | 30 | 124 | WorldExchange | LargestEconomy, MostPopulous, StrongestArmy |
| Tuned Quick | 700738 | Germany | 127 | 3 | 23 | 113 | PlanetaryFoundry | LargestEconomy, MostAdvanced, StrongestArmy |
| Tuned Standard | 700239 | Germany | 314 | 3 | 28 | 200 | PlanetaryFoundry | LargestEconomy, MostAdvanced, MostPopulous |
| Held-out Quick | 910262 | Germany | 158 | 3 | 28 | 119 | WorldExchange | LargestEconomy, MostAdvanced, MostPopulous |

The rarity of sustained compact wins is the main remaining design uncertainty. Many tall claimants later expand or conquer; those are reported as tall-claim winners, not compact successes. Nothing in this experiment proves that a player should ignore expansion on an unsuitable map. Human playtests should test whether choosing dense development creates interesting decisions and a credible title contest, rather than relying on the AI's specific economic personality.

## Recompute and verification

The generated metric sections can be reproduced from the archived data:

```sh
python3 scripts/summarize-tech-bicycle-2-tuning.py \
  'Merged Quick=docs/sim-results/2026-09-27-tech-bicycle-2-tuning-baseline-quick.json.gz' \
  'Merged Standard=docs/sim-results/2026-09-27-tech-bicycle-2-tuning-baseline-standard.json.gz' \
  'Tuned Quick=docs/sim-results/2026-09-27-tech-bicycle-2-tuning-growth-paths-quick.json.gz' \
  'Tuned Standard=docs/sim-results/2026-09-27-tech-bicycle-2-tuning-growth-paths-standard.json.gz' \
  'Held-out Quick=docs/sim-results/2026-09-27-tech-bicycle-2-tuning-heldout-quick.json.gz'
```

All requested checks pass: formatting, warnings-denied workspace Clippy, 464 Rust tests (four existing ignored), optimized WASM build, 468 app tests in 47 files, and the TypeScript/Vite build. Native release determinism has six passing tests. WASM replay checks all three regenerated high-u64 opening tapes and eight selected full games: **11 tapes / 2,015 turns**, including initial-state, every-turn and save/load hashes. Final archive round trips, source-freeze hashes and `git diff --check` pass. No rules/AI files changed after the held-out freeze; the only later app-source edit exposes the existing population-6 quote, and its app checks pass.

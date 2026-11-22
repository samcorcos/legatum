# Full game tech tree: final cohorts and investment windows — 2026-09-27

The seven Inheritance technologies and their paid receivers are implemented. After repricing the late research bottlenecks, Quick median length is 154 turns, Standard is 292, and held-out Quick is 153. All 2,500 final games finished by VP with zero stalls, panics or determinism failures. Each final technology has witnessed investment changes before the game ends, but not every discovery is useful.

The design's entry target is not fully met: median first Inheritance entry among empires reaching it is 146 Quick / 283 Standard, rather than 105–125 / 210–250. Celts's Standard win rate is 22.4%, above the rough 20% upper guideline (descriptive 95% Wilson interval 19.0–26.3%). These are remaining pacing/balance questions, not hidden acceptance passes. Faction traits and preferences were retained. No tuning followed the held-out run.

The [implementation record](2026-09-27-tech-full-implementation.md) contains the exact price changes, rules, AI method, checks, fixture regeneration and complete native/WASM replay evidence. The comparison baseline is the approved [Bicycle 2 tuning report](2026-09-27-tech-bicycle-2-tuning-comparison.md), using its archived raw JSON.

## Cohorts and provenance

All cohorts use eight normal scripted AI empires, one of each faction shuffled into seats, identical map settings, no gameplay turn limit, and the existing 600-Quick/1,200-Standard safety caps. Each reported game is its first play; determinism replay is a check, not another sample. There are 8,000 / 4,000 / 8,000 initial empires. Held-out Quick uses seeds 910001–911000; development uses 700001–701000 Quick and 700001–700500 Standard.

| Cohort | Games | Report | Raw JSON, gzip -9 |
| --- | --- | --- | --- |
| Quick | 1000 | [tech-full-final-quick](2026-09-27-tech-full-final-quick.md) | [JSON](2026-09-27-tech-full-final-quick.json.gz) |
| Standard | 500 | [tech-full-final-standard](2026-09-27-tech-full-final-standard.md) | [JSON](2026-09-27-tech-full-final-standard.json.gz) |
| Heldout Quick | 1000 | [tech-full-heldout-quick](2026-09-27-tech-full-heldout-quick.md) | [JSON](2026-09-27-tech-full-heldout-quick.json.gz) |

The final executable was frozen before starting these three cohorts. Its [source manifest](2026-09-27-tech-full-source.json.gz) records the base commit, executable hash and source hashes. Reports correctly identify a dirty worktree based on `5388b27944e450aa6f39ee0cfb5c865327473b05`; this branch commit supplies the changes.

## Change from Bicycle 2 tuning

| Cohort | Mean turns, before → after | Median, before → after | Stalls, before → after | Integration completed per initial empire | Inheritance completed per initial empire |
| --- | --- | --- | --- | --- | --- |
| Quick | 155.28 → 155.71 | 154 → 154 | 0 → 0 | 2.89 → 2.89 | 0 → 0.98 |
| Standard | 287.33 → 291.54 | 288 → 292 | 0 → 0 | 2.15 → 2.24 | 0 → 0.64 |
| Heldout Quick | 156.43 → 155.48 | 153 → 153 | 0 → 0 | 2.94 → 2.85 | 0 → 0.94 |

Standard median is 1.90× Quick. The Quick median remains inside 110–160. Neither longer tails nor differences in research completion are hidden by reporting only the median; the full quantiles follow below.

| Faction | Quick win %, before → after | Standard win %, before → after | Held-out Quick win %, before → after |
| --- | --- | --- | --- |
| Japan | 9.6 → 7.7 | 7.8 → 7.4 | 10.4 → 9.0 |
| Germany | 15.4 → 15.6 | 12.6 → 11.2 | 14.7 → 16.2 |
| Mongols | 10.6 → 10.2 | 12.2 → 9.6 | 10.2 → 10.6 |
| Sparta | 10.8 → 9.3 | 15.6 → 16.4 | 11.0 → 9.7 |
| Byzantium | 12.1 → 13.8 | 10.4 → 11.4 | 11.4 → 12.7 |
| Athens | 6.5 → 6.3 | 6.2 → 6.8 | 6.3 → 7.1 |
| Switzerland | 16.4 → 17.1 | 15.4 → 14.8 | 16.4 → 16.3 |
| Celts | 18.6 → 20.0 | 19.8 → 22.4 | 19.6 → 18.4 |

## Why the final research was repriced

An [initial 100-game Quick pilot](2026-09-27-tech-full-initial-quick.md) ([raw JSON](2026-09-27-tech-full-initial-quick.json.gz)) kept the proposal's 330–420 Science final prices and the merged 380/340 Science Research Networks/Mutual Defense gates. Five final discoveries had medians of at least turn 179, and the 13 Predictive Computation discoveries had only seven median turns left. That construction window was too short for a paid Core. Final prices are 160–200 Science and those two gates are 190/170; no production receiver was made free and no faction trait was weakened.

| Initial pilot technology | Discoveries | Median discovery turn | Median turns remaining |
| --- | --- | --- | --- |
| NutrientSynthesis | 133 | 161 | 18 |
| PrecisionIndustry | 36 | 190 | 16 |
| PredictiveComputation | 13 | 197 | 7 |
| StrategicLogistics | 45 | 179 | 23 |
| PoweredArmor | 41 | 181 | 18 |
| PlanetaryAccord | 18 | 205 | 20 |
| LegacyIntegration | 62 | 149 | 22 |

The initial probe withheld effects for one planning view while holding current stocks fixed. It undercounts cumulative growth and income effects, so its usefulness percentages are not compared with final percentages. The subsequent carried-forward [Quick](2026-09-27-tech-full-pace-quick.md) / [Standard](2026-09-27-tech-full-pace-standard.md) development runs are archived with gzip JSON. They preceded the public-upgrade-quote correction and explicit metric metadata. All 1,500 final development games reproduce every actual turn hash from those runs; observer evidence can differ. Repeated development seeds are not extra independent evidence.

The early eras are not research quotas. Lower final prices do not require equal technology counts across factions, nor does a complete-tree result by a rare long-game empire imply that every faction should finish all 32.

## Length and determinism

| Cohort | Games | Mean turns | Min / P10 / median / P90 / max | VP | Stalls | Panics | Determinism |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | 1000 | 155.71 | 77 / 122 / 154 / 191 / 381 | 1000 | 0 | 0 | 1000/1000 |
| Standard | 500 | 291.54 | 124 / 225 / 292 / 353 / 618 | 500 | 0 | 0 | 500/500 |
| Heldout Quick | 1000 | 155.48 | 77 / 125 / 153 / 187 / 384 | 1000 | 0 | 0 | 1000/1000 |

## Faction win rates

All initial seats, including eliminated empires.

| Faction | Quick | Standard | Heldout Quick |
| --- | --- | --- | --- |
| Japan | 77/1000 (7.7%) | 37/500 (7.4%) | 90/1000 (9.0%) |
| Germany | 156/1000 (15.6%) | 56/500 (11.2%) | 162/1000 (16.2%) |
| Mongols | 102/1000 (10.2%) | 48/500 (9.6%) | 106/1000 (10.6%) |
| Sparta | 93/1000 (9.3%) | 82/500 (16.4%) | 97/1000 (9.7%) |
| Byzantium | 138/1000 (13.8%) | 57/500 (11.4%) | 127/1000 (12.7%) |
| Athens | 63/1000 (6.3%) | 34/500 (6.8%) | 71/1000 (7.1%) |
| Switzerland | 171/1000 (17.1%) | 74/500 (14.8%) | 163/1000 (16.3%) |
| Celts | 200/1000 (20.0%) | 112/500 (22.4%) | 184/1000 (18.4%) |

## Research completion by era

Counts pool every initial empire, including eliminated empires. Entry quantiles condition on reaching that era. Full era means all its technologies; those are not quotas.

| Cohort | Era | Mean completed | Count quantiles | Entered | Full era | First entry turn quantiles |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | Recovery | 6.88 | 3 / 7 / 7 / 7 / 7 | 8000/8000 (100.0%) | 7345/8000 (91.8%) | 1 / 1 / 19 / 23 / 49 |
| Quick | Consolidation | 7.06 | 0 / 3 / 8 / 9 / 9 | 7877/8000 (98.5%) | 3929/8000 (49.1%) | 14 / 65 / 76 / 92 / 177 |
| Quick | Integration | 2.89 | 0 / 0 / 2 / 8 / 9 | 5937/8000 (74.2%) | 405/8000 (5.1%) | 74 / 94 / 121 / 146 / 245 |
| Quick | Inheritance | 0.98 | 0 / 0 / 0 / 4 / 7 | 2969/8000 (37.1%) | 152/8000 (1.9%) | 101 / 124 / 146 / 170 / 278 |
| Standard | Recovery | 6.80 | 1 / 6 / 7 / 7 / 7 | 4000/4000 (100.0%) | 3500/4000 (87.5%) | 1 / 10 / 36 / 50 / 81 |
| Standard | Consolidation | 6.33 | 0 / 2 / 7 / 9 / 9 | 3881/4000 (97.0%) | 1446/4000 (36.1%) | 80 / 131 / 150 / 180 / 332 |
| Standard | Integration | 2.24 | 0 / 0 / 2 / 6 / 9 | 2599/4000 (65.0%) | 78/4000 (1.9%) | 137 / 186 / 238 / 283 / 397 |
| Standard | Inheritance | 0.64 | 0 / 0 / 0 / 2 / 7 | 1117/4000 (27.9%) | 24/4000 (0.6%) | 180 / 238 / 283 / 337 / 448 |
| Heldout Quick | Recovery | 6.88 | 2 / 7 / 7 / 7 / 7 | 8000/8000 (100.0%) | 7343/8000 (91.8%) | 1 / 1 / 19 / 24 / 42 |
| Heldout Quick | Consolidation | 7.08 | 0 / 3 / 8 / 9 / 9 | 7898/8000 (98.7%) | 3915/8000 (48.9%) | 39 / 65 / 76 / 92 / 168 |
| Heldout Quick | Integration | 2.85 | 0 / 0 / 2 / 8 / 9 | 5960/8000 (74.5%) | 352/8000 (4.4%) | 72 / 94 / 121 / 146 / 211 |
| Heldout Quick | Inheritance | 0.94 | 0 / 0 / 0 / 4 / 7 | 2975/8000 (37.2%) | 127/8000 (1.6%) | 101 / 124 / 145 / 170 / 379 |

## Research totals and specialization

| Cohort | Faction | Total technology count quantiles | Entire tree |
| --- | --- | --- | --- |
| Quick | Japan | 4 / 9 / 15 / 21 / 32 | 6/1000 (0.6%) |
| Quick | Germany | 3 / 12 / 19 / 29 / 32 | 68/1000 (6.8%) |
| Quick | Mongols | 3 / 7 / 12 / 20 / 32 | 3/1000 (0.3%) |
| Quick | Sparta | 5 / 10 / 16 / 22 / 32 | 7/1000 (0.7%) |
| Quick | Byzantium | 4 / 12 / 19 / 28 / 32 | 5/1000 (0.5%) |
| Quick | Athens | 5 / 17 / 25 / 30 / 32 | 10/1000 (1.0%) |
| Quick | Switzerland | 3 / 10 / 17 / 25 / 32 | 22/1000 (2.2%) |
| Quick | Celts | 4 / 11 / 17 / 24 / 32 | 25/1000 (2.5%) |
| Standard | Japan | 3 / 9 / 13 / 20 / 30 | 0/500 (0.0%) |
| Standard | Germany | 2 / 10 / 17 / 25 / 32 | 14/500 (2.8%) |
| Standard | Mongols | 2 / 6 / 11 / 19 / 28 | 0/500 (0.0%) |
| Standard | Sparta | 1 / 9 / 15 / 21 / 30 | 0/500 (0.0%) |
| Standard | Byzantium | 3 / 10 / 18 / 26 / 31 | 0/500 (0.0%) |
| Standard | Athens | 4 / 12 / 22 / 28 / 32 | 1/500 (0.2%) |
| Standard | Switzerland | 3 / 9 / 15 / 22 / 32 | 3/500 (0.6%) |
| Standard | Celts | 4 / 10 / 15 / 22 / 32 | 6/500 (1.2%) |
| Heldout Quick | Japan | 4 / 10 / 15 / 21 / 32 | 3/1000 (0.3%) |
| Heldout Quick | Germany | 3 / 12 / 19 / 29 / 32 | 44/1000 (4.4%) |
| Heldout Quick | Mongols | 3 / 7 / 12 / 20 / 32 | 8/1000 (0.8%) |
| Heldout Quick | Sparta | 5 / 10 / 16 / 22 / 32 | 8/1000 (0.8%) |
| Heldout Quick | Byzantium | 5 / 12 / 20 / 28 / 32 | 1/1000 (0.1%) |
| Heldout Quick | Athens | 6 / 17 / 25 / 29 / 32 | 11/1000 (1.1%) |
| Heldout Quick | Switzerland | 5 / 10 / 17 / 23 / 32 | 19/1000 (1.9%) |
| Heldout Quick | Celts | 5 / 11 / 17 / 24 / 32 | 29/1000 (2.9%) |

## Discovery to changed investment decision

Each final-era discovery forks an observer-only replay with that technology withheld. The fork follows the actual submitted orders, carrying forward changed stocks, growth and completion timing. Before each next turn, the same deterministic brain sees the fork's own fogged view; its investment orders are compared with actual orders. Investment orders are research, production selection, purchase and unit upgrade. The withheld discovery remains marked learned only in the planning view, preventing a spurious decision to relearn it. Opponents never re-plan in the fork. This is decision sensitivity under fixed submitted orders, not proof that a technology caused victory.

Lag is planning turn minus resolved discovery turn; the earliest value is 1. Null observations remain in the discovery denominator. They are censored when actual play ends, the empire is eliminated, or its replay finishes/eliminates it. Remaining-window quantiles condition only on discovery. Lag and post-decision window quantiles condition on a witnessed difference. Initial one-turn pilot metrics use an earlier definition and must not be pooled with these.

| Cohort | Technology | Discoveries | Discovery turn quantiles | Turns left at discovery | Changed decision | Lag quantiles | Turns left after decision | Eventual winners with witness |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | NutrientSynthesis | 2655 | 109 / 131 / 148 / 172 / 293 | 0 / 3 / 17 / 56 / 248 | 1673/2655 (63.0%) | 1 / 3 / 9 / 20 / 64 | 0 / 2 / 16 / 54 / 241 | 444 |
| Quick | PrecisionIndustry | 584 | 118 / 139 / 165 / 199 / 302 | 0 / 2 / 18 / 60 / 230 | 483/584 (82.7%) | 1 / 1 / 4 / 8 / 20 | 0 / 2 / 18 / 61 / 220 | 128 |
| Quick | PredictiveComputation | 306 | 101 / 121 / 182 / 219 / 297 | 0 / 2 / 21 / 67 / 207 | 244/306 (79.7%) | 1 / 1 / 3 / 13 / 94 | 0 / 2 / 21 / 65 / 198 | 70 |
| Quick | StrategicLogistics | 1196 | 117 / 136 / 163 / 196 / 292 | 0 / 2 / 18 / 57 / 216 | 956/1196 (79.9%) | 1 / 1 / 3 / 11 / 52 | 0 / 2 / 18 / 57 / 213 | 190 |
| Quick | PoweredArmor | 975 | 125 / 139 / 165 / 200 / 306 | 0 / 2 / 17 / 55 / 221 | 618/975 (63.4%) | 1 / 1 / 3 / 22 / 66 | 0 / 2 / 16 / 55 / 216 | 122 |
| Quick | PlanetaryAccord | 437 | 136 / 151 / 181 / 215 / 332 | 0 / 2 / 14 / 56 / 204 | 312/437 (71.4%) | 1 / 2 / 5 / 12 / 27 | 0 / 2 / 15 / 61 / 198 | 85 |
| Quick | LegacyIntegration | 1704 | 102 / 120 / 147 / 182 / 339 | 0 / 3 / 21 / 61 / 236 | 1484/1704 (87.1%) | 1 / 1 / 5 / 7 / 32 | 0 / 3 / 21 / 61 / 230 | 259 |
| Standard | NutrientSynthesis | 971 | 192 / 250 / 287 / 337 / 493 | 0 / 5 / 29 / 93 / 284 | 637/971 (65.6%) | 1 / 5 / 13 / 32 / 99 | 0 / 3 / 27 / 87 / 268 | 186 |
| Standard | PrecisionIndustry | 171 | 240 / 270 / 320 / 375 / 586 | 0 / 3 / 25 / 82 / 269 | 139/171 (81.3%) | 1 / 1 / 5 / 10 / 23 | 0 / 4 / 26 / 84 / 266 | 35 |
| Standard | PredictiveComputation | 78 | 210 / 229 / 327 / 410 / 608 | 0 / 3 / 27 / 83 / 210 | 49/78 (62.8%) | 1 / 1 / 8 / 30 / 81 | 1 / 4 / 28 / 78 / 158 | 10 |
| Standard | StrategicLogistics | 366 | 232 / 265 / 315 / 375 / 507 | 0 / 6 / 32 / 87 / 301 | 291/366 (79.5%) | 1 / 1 / 4 / 19 / 53 | 0 / 4 / 33 / 87 / 298 | 64 |
| Standard | PoweredArmor | 309 | 243 / 271 / 312 / 389 / 522 | 0 / 6 / 28 / 86 / 317 | 202/309 (65.4%) | 1 / 1 / 5 / 37 / 169 | 0 / 4 / 24 / 75 / 315 | 42 |
| Standard | PlanetaryAccord | 95 | 262 / 294 / 344 / 418 / 465 | 0 / 3 / 22 / 102 / 228 | 73/95 (76.8%) | 2 / 4 / 8 / 18 / 22 | 0 / 2 / 22 / 105 / 210 | 24 |
| Standard | LegacyIntegration | 588 | 180 / 232 / 282 / 349 / 552 | 0 / 5 / 40 / 106 / 354 | 505/588 (85.9%) | 1 / 3 / 7 / 11 / 17 | 0 / 4 / 41 / 102 / 346 | 80 |
| Heldout Quick | NutrientSynthesis | 2654 | 110 / 131 / 147 / 172 / 379 | 0 / 2 / 15 / 51 / 236 | 1612/2654 (60.7%) | 1 / 3 / 9 / 19 / 56 | 0 / 2 / 14 / 50 / 229 | 424 |
| Heldout Quick | PrecisionIndustry | 534 | 123 / 139 / 164 / 198 / 315 | 0 / 2 / 17 / 56 / 229 | 443/534 (83.0%) | 1 / 1 / 4 / 8 / 24 | 0 / 3 / 17 / 55 / 226 | 119 |
| Heldout Quick | PredictiveComputation | 247 | 108 / 127 / 183 / 223 / 336 | 0 / 2 / 19 / 60 / 185 | 192/247 (77.7%) | 1 / 1 / 3 / 13 / 43 | 0 / 3 / 18 / 63 / 184 | 60 |
| Heldout Quick | StrategicLogistics | 1107 | 117 / 134 / 161 / 193 / 290 | 0 / 2 / 16 / 54 / 211 | 870/1107 (78.6%) | 1 / 1 / 3 / 10 / 33 | 0 / 2 / 16 / 58 / 210 | 179 |
| Heldout Quick | PoweredArmor | 873 | 122 / 140 / 163 / 201 / 261 | 0 / 2 / 16 / 55 / 203 | 539/873 (61.7%) | 1 / 1 / 3 / 18 / 80 | 0 / 2 / 16 / 60 / 199 | 105 |
| Heldout Quick | PlanetaryAccord | 398 | 132 / 153 / 179 / 213 / 336 | 0 / 2 / 14 / 62 / 197 | 285/398 (71.6%) | 1 / 2 / 5 / 12 / 25 | 0 / 2 / 18 / 62 / 187 | 76 |
| Heldout Quick | LegacyIntegration | 1683 | 101 / 121 / 146 / 180 / 262 | 0 / 3 / 20 / 57 / 228 | 1439/1683 (85.5%) | 1 / 2 / 5 / 7 / 27 | 0 / 2 / 19 / 55 / 224 | 261 |

## Paid late construction and Great Works

| Cohort | Inference Cores completed | Assault Troopers produced | Assault upgrades | Foundry completions / winners | Exchange completions / winners | Relay completions / winners |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | 387 | 1534 | 1147 | 755 / 225 | 118 / 49 | 2428 / 611 |
| Standard | 54 | 313 | 137 | 304 / 95 | 41 / 20 | 951 / 265 |
| Heldout Quick | 319 | 1306 | 1029 | 729 / 202 | 130 / 43 | 2397 / 652 |

## Interpretation and remaining uncertainty

The question “does discovery change a decision soon enough?” has a measurable answer here: the table reports the first different investment orders, their lag, the remaining game window and the share of all discoveries with no witnessed difference. Paid Core and Assault completion counts confirm that the new options are exercised beyond merely entering the research catalog. Eventual-winner witness counts show presence in winning plans; they do not establish that the breakthrough caused a win.

Median remaining windows condition on discovery, and medians after a decision condition on a witness; they describe different populations and must not be subtracted from one another. Quantiles are min / P10 / median / P90 / max using the runner's rounded-index convention. A zero remaining window means a discovery or investment occurred on the final resolved turn. Censored observations remain failures to witness a changed decision, not evidence the effect can never matter.

Inheritance entry remains later than the proposal's typical window, Standard Celts exceeds the rough upper win-rate guideline, and some discoveries never affect an investment before elimination or game end. The AI cannot know the actual finish turn from its fogged view. These runs establish executable rules, observed late-game use, length and determinism; they do not replace human full-game acceptance, alternate team layouts, rotated starts, or diplomacy integration. The dedicated Tuning and acceptance increment remains necessary.

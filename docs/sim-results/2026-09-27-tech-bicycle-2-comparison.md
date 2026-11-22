# Bicycle second slice: measured route comparison (2026-09-27)

The 2,500-game final candidate demonstrates Exchange and Relay wins, but does **not** establish all three Great Works or sustained compact play as viable winning paths: **0 Planetary Foundries** and **0 compact winners**. All final games end by VP with no stalls or determinism failures. Games run longer than the tuning baseline, and Athens / Celts remain outside the 6–20% band.

This compares the implemented national Great Works and the tall milestone alternative with the frozen `network` candidate and held-out Quick cohort in [the approved tuning comparison](2026-09-27-tech-bicycle-tuning-comparison.md). It reports observed scripted free-for-all outcomes, not human playtest acceptance.

The permanent ceiling stays seven VP per empire: three ordinary milestones, one two-point Great Work, and the two-point faction goal. `FourCities` and `RelayUplink` remain serialized identifiers for Developed Empire and Great Work, preserving existing saves; their displayed names and meanings are updated.

## Cohorts and reproducibility

All cohorts use Normal scripted brains, one of every faction shuffled among eight single-empire teams, 48×32 maps, no turn limit, eight threads, and unchanged safety caps of 600 Quick / 1,200 Standard. Development seeds are 700001–701000 Quick and 700001–700500 Standard. Held-out Quick uses 910001–911000. Every game is played twice with per-turn determinism comparison and final save/load validation; only first plays enter statistics. These cohorts do not validate permanent-team or human strategies.

The final executable was frozen before held-out execution. No rules or AI tuning used held-out results. See [the implementation and checks record](2026-09-27-tech-bicycle-2-implementation.md) for the binary/source hashes, exploratory runs, exact checks and intentional fixture regeneration. The runner records its starting Git revision, dirty flag, setup and exact argv in each JSON; the source hashes identify the uncommitted candidate that was later committed on this branch.

| Cohort | Report | Raw data |
| --- | --- | --- |
| Baseline Quick | [Markdown](2026-09-27-tech-bicycle-tuning-network-quick.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-tuning-network-quick.json.gz) |
| Slice 2 Quick | [Markdown](2026-09-27-tech-bicycle-2-quick.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-2-quick.json.gz) |
| Baseline Standard | [Markdown](2026-09-27-tech-bicycle-tuning-network-standard.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-tuning-network-standard.json.gz) |
| Slice 2 Standard | [Markdown](2026-09-27-tech-bicycle-2-standard.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-2-standard.json.gz) |
| Baseline held-out Quick | [Markdown](2026-09-27-tech-bicycle-tuning-heldout-quick.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-tuning-heldout-quick.json.gz) |
| Slice 2 held-out Quick | [Markdown](2026-09-27-tech-bicycle-2-heldout-quick.md) | [gzip -9 JSON](2026-09-27-tech-bicycle-2-heldout-quick.json.gz) |

Archives decompress byte-for-byte to the original JSON. Quantiles sort observations and select `(percentile * (n - 1) + 50) // 100`, matching the runner. Eliminated empires stay in all-seat denominators. Completion timing excludes non-completers; counts include them.

## Length, outcome share and stalls

| Cohort | Games | Mean turns | Min / P10 / median / P90 / max | VP outcomes | Stalls / panics | Determinism |
| --- | --- | --- | --- | --- | --- | --- |
| Baseline Quick | 1000 | 155.86 | 81 / 118 / 156 / 190 / 329 | 1000/1000 (100.0%) | 0 / 0 | 1000/1000 (100.0%) |
| Slice 2 Quick | 1000 | 169.93 | 79 / 136 / 169 / 204 / 475 | 1000/1000 (100.0%) | 0 / 0 | 1000/1000 (100.0%) |
| Baseline Standard | 500 | 296.71 | 164 / 218 / 299 / 358 / 530 | 500/500 (100.0%) | 0 / 0 | 500/500 (100.0%) |
| Slice 2 Standard | 500 | 315.13 | 165 / 237 / 318 / 379 / 599 | 500/500 (100.0%) | 0 / 0 | 500/500 (100.0%) |
| Baseline held-out Quick | 1000 | 158.21 | 85 / 123 / 157 / 192 / 386 | 1000/1000 (100.0%) | 0 / 0 | 1000/1000 (100.0%) |
| Slice 2 held-out Quick | 1000 | 168.63 | 87 / 137 / 167 / 205 / 315 | 1000/1000 (100.0%) | 0 / 0 | 1000/1000 (100.0%) |

## Per-faction wins

Wins divided by initial seats of that faction (1,000 per Quick cohort, 500 per Standard cohort). The 6–20% range is a review target; point estimates outside it are not silently treated as passes.

| Faction | Baseline Quick | Slice 2 Quick | Baseline Standard | Slice 2 Standard | Baseline held-out Quick | Slice 2 held-out Quick |
| --- | --- | --- | --- | --- | --- | --- |
| Japan | 85/1000 (8.5%) | 81/1000 (8.1%) | 31/500 (6.2%) | 31/500 (6.2%) | 92/1000 (9.2%) | 85/1000 (8.5%) |
| Germany | 180/1000 (18.0%) | 173/1000 (17.3%) | 75/500 (15.0%) | 86/500 (17.2%) | 187/1000 (18.7%) | 183/1000 (18.3%) |
| Mongols | 88/1000 (8.8%) | 106/1000 (10.6%) | 39/500 (7.8%) | 36/500 (7.2%) | 100/1000 (10.0%) | 89/1000 (8.9%) |
| Sparta | 89/1000 (8.9%) | 110/1000 (11.0%) | 74/500 (14.8%) | 80/500 (16.0%) | 87/1000 (8.7%) | 105/1000 (10.5%) |
| Byzantium | 102/1000 (10.2%) | 73/1000 (7.3%) | 57/500 (11.4%) | 36/500 (7.2%) | 88/1000 (8.8%) | 85/1000 (8.5%) |
| Athens | 65/1000 (6.5%) | 47/1000 (4.7%) | 37/500 (7.4%) | 27/500 (5.4%) | 54/1000 (5.4%) | 50/1000 (5.0%) |
| Switzerland | 184/1000 (18.4%) | 189/1000 (18.9%) | 77/500 (15.4%) | 73/500 (14.6%) | 193/1000 (19.3%) | 193/1000 (19.3%) |
| Celts | 207/1000 (20.7%) | 221/1000 (22.1%) | 110/500 (22.0%) | 131/500 (26.2%) | 199/1000 (19.9%) | 210/1000 (21.0%) |

Outside the target band: Slice 2 Quick: Athens 4.7%; Slice 2 Quick: Celts 22.1%; Slice 2 Standard: Athens 5.4%; Slice 2 Standard: Celts 26.2%; Slice 2 held-out Quick: Athens 5.0%; Slice 2 held-out Quick: Celts 21.0%.

## Sources of winning VP

Sum terminal winning-empire VP only. One empire per team makes these also winning-team totals; do not add these populations together. Captured Great Works never add credit. The baseline project column is Relay alone; slice 2 uses the shared Great Work milestone.

| Cohort | Winner VP | Live titles | Ordinary milestones | Great Work | Faction goals |
| --- | --- | --- | --- | --- | --- |
| Baseline Quick | 10157 | 3544/10157 (34.9%) | 2993/10157 (29.5%) | 1682/10157 (16.6%) | 1938/10157 (19.1%) |
| Slice 2 Quick | 10181 | 3581/10181 (35.2%) | 3000/10181 (29.5%) | 1686/10181 (16.6%) | 1914/10181 (18.8%) |
| Baseline Standard | 5092 | 1901/5092 (37.3%) | 1495/5092 (29.4%) | 744/5092 (14.6%) | 952/5092 (18.7%) |
| Slice 2 Standard | 5109 | 1955/5109 (38.3%) | 1500/5109 (29.4%) | 696/5109 (13.6%) | 958/5109 (18.8%) |
| Baseline held-out Quick | 10180 | 3510/10180 (34.5%) | 2996/10180 (29.4%) | 1726/10180 (17.0%) | 1948/10180 (19.1%) |
| Slice 2 held-out Quick | 10203 | 3565/10203 (34.9%) | 3000/10203 (29.4%) | 1706/10203 (16.7%) | 1932/10203 (18.9%) |

## Great Work completions and winning paths

Each completion is an observed ProductionCompleted event, not a captured building or an announced start. Each empire can finish at most one. “Winners with route” means the eventual winning empire completed it before the game ended; it does not show that the Work was necessary or caused victory. No additional points are awarded for choosing a particular route.

| Cohort | Route | Completing empires | Games with completion | Winners with route | Completion turn: min / P10 / median / P90 / max | Winning factions (counts) |
| --- | --- | --- | --- | --- | --- | --- |
| Slice 2 Quick | Planetary Foundry | 0/8000 (0.0%) | 0/1000 (0.0%) | 0/1000 (0.0%) | — | — |
| Slice 2 Quick | World Exchange | 1302/8000 (16.3%) | 769/1000 (76.9%) | 585/1000 (58.5%) | 120 / 140 / 159 / 184 / 246 | Japan 40, Germany 127, Mongols 45, Sparta 46, Switzerland 136, Celts 191 |
| Slice 2 Quick | Relay Uplink | 1209/8000 (15.1%) | 618/1000 (61.8%) | 258/1000 (25.8%) | 128 / 147 / 166 / 198 / 259 | Japan 25, Germany 37, Mongols 35, Sparta 39, Byzantium 46, Athens 39, Switzerland 28, Celts 9 |
| Slice 2 Standard | Planetary Foundry | 0/4000 (0.0%) | 0/500 (0.0%) | 0/500 (0.0%) | — | — |
| Slice 2 Standard | World Exchange | 507/4000 (12.7%) | 327/500 (65.4%) | 240/500 (48.0%) | 251 / 279 / 312 / 356 / 548 | Japan 15, Germany 61, Mongols 9, Sparta 20, Byzantium 1, Switzerland 37, Celts 97 |
| Slice 2 Standard | Relay Uplink | 393/4000 (9.8%) | 242/500 (48.4%) | 108/500 (21.6%) | 256 / 283 / 324 / 376 / 454 | Japan 7, Germany 20, Mongols 11, Sparta 24, Byzantium 15, Athens 15, Switzerland 11, Celts 5 |
| Slice 2 held-out Quick | Planetary Foundry | 0/8000 (0.0%) | 0/1000 (0.0%) | 0/1000 (0.0%) | — | — |
| Slice 2 held-out Quick | World Exchange | 1299/8000 (16.2%) | 778/1000 (77.8%) | 575/1000 (57.5%) | 118 / 140 / 159 / 184 / 236 | Japan 36, Germany 135, Mongols 37, Sparta 48, Byzantium 3, Switzerland 130, Celts 186 |
| Slice 2 held-out Quick | Relay Uplink | 1186/8000 (14.8%) | 624/1000 (62.4%) | 278/1000 (27.8%) | 129 / 146 / 167 / 198 / 251 | Japan 32, Germany 42, Mongols 24, Sparta 42, Byzantium 52, Athens 47, Switzerland 33, Celts 6 |

## Tall milestone versus wide development

The first scoring claim records both owned-city count and the number at population ≥8. A tall-only claim has fewer than four cities and at least two population-8 cities. Both conditions can hold on the same claim; that still awards only one point. Later conquest or growth never reclassifies the original claim. “Compact winners” below means a tall-only claimant still owns at most three cities at game end, not that it never expanded or conquered.

| Cohort | Tall-only claims | Wide-only claims | Both at claim | Winners with tall-only claim | Compact winners | Tall claim turn: min / P10 / median / P90 / max | Winning tall claimants by Work |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Slice 2 Quick | 407/8000 (5.1%) | 5573/8000 (69.7%) | 4 | 8/1000 (0.8%) | 0 | 82 / 106 / 137 / 172 / 229 | Relay Uplink 2, World Exchange 6 |
| Slice 2 Standard | 132/4000 (3.3%) | 2817/4000 (70.4%) | 1 | 6/500 (1.2%) | 0 | 172 / 205 / 256 / 319 / 399 | Relay Uplink 1, World Exchange 5 |
| Slice 2 held-out Quick | 375/8000 (4.7%) | 5555/8000 (69.4%) | 4 | 13/1000 (1.3%) | 0 | 80 / 104 / 142 / 181 / 238 | Relay Uplink 1, World Exchange 12 |

## Interpretation and limits

World Exchange and Relay Uplink are distinguishable paid finishing routes and both appear among winners. The commerce route requires two population-gated Clearinghouse investments; Relay requires recovered-power research and an Archive city. Their completion counts support observed alternative routes, not equal strength or causal win effects.

Planetary Foundry must be judged from its actual cohort counts above. Targeted fogged-view AI tests show it is selected when two existing Autoworks make it the fastest finish; that is not proof of a competitive full-game industrial plan. The retained tuned Machine Shop / Autoworks prices (160P / 300P, before speed and applicable faction modifiers) make a pair of Autoworks a large additional commitment. A zero or negligible campaign completion count means the third winning route has not been demonstrated.

Likewise, a tall milestone claim is not sufficient evidence that sustained two- or three-city play is competitive. Compare its winning-claimant and compact-winner counts with the wide route, rather than labeling every later wide empire a successful tall strategy. Cities continue to feed multiple titles and the ordinary twenty-population milestone.

The original tuning baseline used an 80S Relay Restoration shortcut and a 300P Uplink. This slice restores the approved 220S Recovered Power + Civic Networks prerequisite and all three 400P Works; all other tuned prices and faction rules remain. The diagnostic with the old shortcut produced only Relay completions. Commerce wins appear under the final rules, which also change research timing and the Athenian goal route. Longer games or faction rates outside 6–20% remain acceptance issues, not facts to conceal with a headline pass.

Diplomacy-dependent benefits remain untouched. Human investment windows, permanent teams, adversarial interruption timing and robust industrial/tall viability still need further acceptance work. No rule or AI changes were made after inspecting held-out results.

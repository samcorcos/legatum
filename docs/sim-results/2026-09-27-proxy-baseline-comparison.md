# Human-proxy baseline: 900 Standard games

300 games per difficulty; seeds 700001–700300; one Proxy versus seven independent AI teams; balanced seat/faction rotation; no turn limit; safety cap 1,200. All runs use `--determinism` and the hash-verified master incident detector.

Source: `31b8eaf53cd9f6bd52d26fad3a097d899d03afe1`, clean checkout for every run. See [runner and metric definitions](../archive/ai-proxy-benchmark.md). These are the baseline **before** the +2 capital Science change (`41694da`) and scout/garrison tactics changes (`d712784`), which landed on master during the work. The final branch merges those changes and strategy-diversity reporting; reproducing these hashes requires the recorded source revision.

| Metric | Seven Easy | Seven Normal | Seven Hard |
| --- | ---: | ---: | ---: |
| Proxy wins | 120/300 (40.0%) | 54/300 (18.0%) | 73/300 (24.3%) |
| Median winning turn | 296.5 | 354.5 | 323.0 |
| AI cities alive at 100, mean per lobby | 16.96 (n=300) | 16.32 (n=300) | 17.41 (n=300) |
| AI cities alive at 150, mean per lobby | 22.75 (n=300) | 22.79 (n=300) | 23.45 (n=300) |
| AI cities alive at 200, mean per lobby | 24.71 (n=295) | 26.80 (n=290) | 26.53 (n=288) |
| AI cities founded, mean per original AI seat | 4.08 (n=2100) | 4.36 (n=2100) | 4.33 (n=2100) |
| AI techs at 150, mean per original AI seat | 5.64 (n=2100) | 5.37 (n=2100) | 6.15 (n=2100) |
| Unspent gold, mean per eliminated AI | 34.70 (n=337) | 23.96 (n=273) | 26.16 (n=302) |
| Proxy captures, mean per game | 7.83 (n=300) | 5.99 (n=300) | 6.65 (n=300) |
| Proxy cities founded, mean per game | 5.93 (n=300) | 5.48 (n=300) | 5.62 (n=300) |
| Proxy cities at 150, mean per reached game | 4.89 (n=300) | 4.27 (n=300) | 4.60 (n=300) |
| Proxy with 7+ cities at 150 | 55/300 | 26/300 | 39/300 |
| Determinism checks passed | 300/300 | 300/300 | 300/300 |
| Panics / safety-cap stalls | 0 / 0 | 0 / 0 | 0 / 0 |

`n` is the actual observation count, not an imputed denominator. Snapshots exclude games ending before the named turn, so late-turn averages are conditional on games reaching that turn. Foundings include capitals but not captures; captures include recaptures; treasury observations exclude surviving AIs.

## Interpretation

This policy did not reproduce the expected strong-human performance: it did not crush any difficulty, and only 39/300 Hard games reached seven Proxy cities by turn 150. Its 24.3% Hard win rate must not be interpreted as meeting the plan's AI-competitiveness acceptance criterion; the Proxy itself still needs calibration against the human strategy. Normal also stopped this policy more often than Hard in these eight-team lobbies.

The fixed plan and complete metrics provide a reproducible first benchmark. They establish what this policy did at the recorded source revision, not that it is an adequate substitute for the human who conquered everyone by turn 197.

## AI incidents

Only the seven scripted opponents are counted. All 300 games per difficulty were analyzed with detector version 1 and the default 12-turn settler threshold. Rules overlap; summing categories does not count independent failures. Per-rule opportunity denominators and faction rates are included in each full report.

| Incident | Seven Easy | Seven Normal | Seven Hard |
| --- | ---: | ---: | ---: |
| garrison_under_ranged_fire | 636 | 441 | 415 |
| gold_hoarded_under_attack | 34 | 4 | 7 |
| idle_military_threatened_city | 46278 | 44269 | 39666 |
| movement_oscillation | 32803 | 122845 | 116938 |
| production_idle | 38275 | 3011 | 7558 |
| repeated_bounce | 1315 | 602 | 625 |
| research_idle | 17891 | 9326 | 12970 |
| scout_not_fleeing | 608 | 40 | 66 |
| settler_wandering | 97137 | 184990 | 168367 |
| unanswered_attack | 1379 | 686 | 698 |

## Reports and data

- Easy: [Markdown](2026-09-27-proxy-vs-easy-standard.md), [gzip -9 JSON](2026-09-27-proxy-vs-easy-standard.json.gz).
- Normal: [Markdown](2026-09-27-proxy-vs-normal-standard.md), [gzip -9 JSON](2026-09-27-proxy-vs-normal-standard.json.gz).
- Hard: [Markdown](2026-09-27-proxy-vs-hard-standard.md), [gzip -9 JSON](2026-09-27-proxy-vs-hard-standard.json.gz).

Each JSON includes all 300 seeds, seat/faction assignments, per-turn hashes, per-game metrics, detector version, aggregate denominators, and exact invocation. The paired difficulties share the same maps and assignments. Each seat and faction appears 37 or 38 times per run.

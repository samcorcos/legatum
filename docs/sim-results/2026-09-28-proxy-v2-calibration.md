# Proxy v2 calibration: 900 Standard games

**Status: stronger, but not fully calibrated.** Version 2 meets the Hard win-rate target and the winning-turn target. It misses the 85% Normal target, typically has only four or five cities at turn 100, and does not establish a technology lead over the strongest AI at turn 150. Do not yet use this as an accepted replacement for the human yardstick.

Measured source: `a0b4bf0d00c7a3ec98c173eb6bffcf9fdd8922bf`, clean at the start of all three runs; master base `ca7e3f7c77b7ab68dc0129889423b01a5e1e196e`; `PROXY_VERSION = 2`. Later master changes are outside this measurement. The Proxy policy and its observation tools are the only behavior-related changes in this branch relative to that base; the scripted AIs are unchanged.

Each difficulty uses 300 Standard games, seeds 700001–700300, a 48×32 map, eight separate teams, `--proxy 0 --rotate-proxy --determinism`, no turn limit, and a 1,200-turn safety cap. Every seat and faction appears 37 or 38 times; each 64-game block covers all seat/faction pairs. These are the fixed comparison seeds from the original benchmark, not an untouched holdout after loss profiling. Development variants used separate, smaller 820001-series batches.

## Outcomes and requested milestones

| Metric | Seven Easy | Seven Normal | Seven Hard |
| --- | ---: | ---: | ---: |
| Proxy wins | 267/300 (89.0%) | 222/300 (74.0%) | 249/300 (83.0%) |
| Median winning turn | 202.00 (n=267) | 216.50 (n=222) | 203.00 (n=249) |
| Proxy cities at T100, median | 5.00 (n=300) | 4.00 (n=300) | 4.00 (n=300) |
| Proxy cities at T100, mean | 4.58 (n=300) | 3.73 (n=300) | 4.45 (n=300) |
| Proxy techs at T100, median | 6.00 (n=300) | 5.00 (n=300) | 6.00 (n=300) |
| Proxy techs at T100, mean | 5.64 (n=300) | 5.30 (n=300) | 5.68 (n=300) |
| Techs minus strongest AI at T100, median | -1.00 (n=300) | -1.00 (n=300) | -1.00 (n=300) |
| Proxy cities at T150, median | 7.00 (n=284) | 6.00 (n=293) | 7.00 (n=273) |
| Proxy cities at T150, mean | 7.35 (n=284) | 6.01 (n=293) | 6.70 (n=273) |
| Proxy techs at T150, median | 10.00 (n=284) | 9.00 (n=293) | 10.00 (n=273) |
| Proxy techs at T150, mean | 10.12 (n=284) | 9.11 (n=293) | 9.85 (n=273) |
| Techs minus strongest AI at T150, median | -0.50 (n=284) | -1.00 (n=293) | -1.00 (n=273) |
| Games with at least six Proxy cities at T100 | 82/300 | 39/300 | 86/300 |
| Games with a positive tech lead at T150 | 95/284 | 59/293 | 53/273 |
| Determinism checks passed | 300/300 | 300/300 | 300/300 |
| Panics / safety-cap stalls | 0 / 0 | 0 / 0 | 0 / 0 |

Snapshot means and medians include only games reaching that turn, with the actual denominator shown. Early finishes are excluded, never zero-filled or carried forward. The tech lead compares against the **largest retained tech count among the seven original AI seats**, including eliminated seats. A lead over the average opponent is insufficient. All 900 games ended through the VP target; these are not 900 conquests. The policy can earn faction goals and build a Great Work, so a fast win alone does not reproduce the historical 19 captures.

## AI survival and economy

| Metric | Seven Easy | Seven Normal | Seven Hard |
| --- | ---: | ---: | ---: |
| AI cities alive at T100, mean per lobby | 13.55 (n=300) | 14.08 (n=300) | 14.18 (n=300) |
| AI empires alive at T100, mean per lobby | 6.11 (n=300) | 6.33 (n=300) | 6.26 (n=300) |
| AI cities alive at T150, mean per lobby | 16.76 (n=284) | 18.92 (n=293) | 18.71 (n=273) |
| AI empires alive at T150, mean per lobby | 5.65 (n=284) | 5.98 (n=293) | 5.92 (n=273) |
| AI cities alive at T200, mean per lobby | 19.24 (n=176) | 21.61 (n=240) | 21.78 (n=188) |
| AI empires alive at T200, mean per lobby | 5.81 (n=176) | 5.87 (n=240) | 5.95 (n=188) |
| AI cities founded per original AI, mean | 3.27 (n=2100) | 3.71 (n=2100) | 3.61 (n=2100) |
| AI techs at T150 per original AI, mean | 7.84 (n=1988) | 8.00 (n=2051) | 8.29 (n=1911) |
| Gold at elimination per eliminated AI, mean | 36.41 (n=560) | 27.26 (n=501) | 29.52 (n=511) |
| Captures by Proxy per game, mean | 6.97 (n=300) | 6.85 (n=300) | 7.00 (n=300) |
| Cities founded by Proxy per game, mean | 3.99 (n=300) | 3.94 (n=300) | 3.82 (n=300) |

Foundings include capitals, exclude captures, and count each original AI seat. Captures include recaptures. Gold excludes surviving AIs. The full reports include all AI incident counts and opportunity denominators; observer, AI-incident and Proxy-incident coverage are 300/300 for every difficulty. Detector v1 uses its default 12-turn settler threshold, and overlapping incident categories must not be added as independent failures.

## What the loss profiling changed

The original unversioned policy was also tested for 24 rotated Hard games on then-master `2aa0058`; it won 8/24 with a median winning turn of 380.5. Hash-verified replays of seeds 810007 and 810004 showed only three cities at turn 100 and no completed buildings. Their founding turns were 1/49/98 and 1/47/93. Neither had a Proxy production-idle or research-idle incident: the problem was allocation, especially increasingly expensive settlers before infrastructure. That small run and the historical 40.0% / 18.0% / 24.3% reports used different engine/AI revisions from this full measurement, so they are not a paired estimate of policy-only improvement.

Version 2 changes the opening army and expansion priorities, values near-term worked tiles when settling, accounts for queued troops, invests in science and production, and spends quoted gold with an upkeep reserve. Ranged units coordinate minimum-damage volleys before infantry decide whether they can finish a city. Infantry can follow a killed garrison, wounded units take safe finishing captures before healing, scouts capture exposed settlers or defeated cities, and local defenders intercept threats. The policy also chooses a fast eligible Great Work host. All decisions remain stateless, deterministic and based only on `PlayerView`; telemetry is an observer, never a policy input.

Two losses from this exact 900-game source are preserved with their hash-verified analysis in [the replay bundle](proxy-v2-replays/README.md). Normal seed 700004 demonstrates early captures followed by two Rangers retreating too late: adjacent melee contact pins them before movement. Hard seed 700001 demonstrates four cities at turn 100 with no completed infrastructure, a lost captured city, and only eight technologies at 150. The remaining gaps are durable ranged support, earlier infrastructure on weak starts, and expansion that survives counterattacks.

| Proxy diagnostic | Seven Easy | Seven Normal | Seven Hard |
| --- | ---: | ---: | ---: |
| Production-idle incidents | 7 | 6 | 7 |
| Research-idle incidents | 0 | 0 | 0 |
| Threatened-city idle incidents | 1143 | 1026 | 811 |
| Movement-oscillation incidents | 686 | 944 | 859 |
| Settler-wandering incidents | 729 | 771 | 1249 |
| Median first attack, any Proxy unit | 18.0 | 16.5 | 16.5 |
| Median first capture, games with a capture | 57.0 | 66.0 | 54 |
| Mean Proxy cities lost per game | 0.63 | 1.42 | 0.92 |

The first attack includes scout skirmishes and is not a clean measure of the main army’s departure. Per-game JSON records founding and research turns, completed production, unit losses, disbands, city losses, and T100/T150/T200 unit composition and science output. Low idle counts do not imply good investment allocation.

At T100 the median number of combat troops, excluding scouts and settlers, is three against
Easy, two against Normal, and three against Hard. Normal's category medians are one ranged
unit and one infantry unit. By T150 the median total is still three in each difficulty.
The nominal army target therefore overstates the force that actually survives and reaches
the front. Replacing casualties competes directly with settling and science construction.
Research itself never idles: the policy opens Soil Reclamation, Fabrication and Signal Theory,
then Fortification and Ballistics. Unlocking a Laboratory early does not help if its city keeps
replacing troops. The later research branch also pursues the faction/Great Work victory route;
the measured tech-count deficit shows that this has not produced the requested broad lead.

## Later development experiments

Four additional hypotheses were compared with the unchanged v2 binary on identical maps, rotations and rules. None closed the full set of targets, so none was promoted. All used Normal difficulty and 64 games per block. These are development results, not replacements for the 300-game baselines above.

| Candidate | First seed | Wins | Median winning turn | Median cities T100 | Median tech lead T150 |
| --- | ---: | ---: | ---: | ---: | ---: |
| followthrough-normal | 820001 | 54/64 | 215.0 | 4.0 | -1.0 |
| investment-normal | 820001 | 51/64 | 226.0 | 3.0 | 0.0 |
| preservation-normal | 820001 | 53/64 | 211.0 | 4.0 | -1.0 |
| two-infantry-normal | 820001 | 51/64 | 198.0 | 4.0 | -1.0 |
| two-infantry-second-normal | 830001 | 53/64 | 208.0 | 5.0 | -1.0 |
| v2-control-normal | 820001 | 54/64 | 219.0 | 4.0 | -1.0 |
| v2-control-second-normal | 830001 | 49/64 | 212.0 | 4.0 | -1.0 |

The investment candidate prioritized safe cities’ Workshops/Laboratories and assigned reinforcements to faster cities. The preservation candidate forecast a second volley and recognized contact-pinned retreats. The follow-through candidate guarded a damaged capture only against visible melee units with a known route and enough damage to retake it. The last candidate raised the early infantry minimum from one to two.

The extra-infantry candidate was checked on a second disjoint seed block because its first result traded fewer wins for faster wins. Across both blocks it won 104/128 versus the control’s 103/128, with no clear technology lead. That one-game difference does not establish a strength improvement. The other candidates either reduced wins or left the requested milestones unchanged.

The [development data bundle](2026-09-28-proxy-v2-development.json.gz) contains all seven reports and the exact policy patches relative to the measured v2 source. These were uncommitted candidates using the provisional next version number; distinguish them by label and patch, not by treating each as the same released version 3. The final policy remains version 2.

## Verification and artifacts

All requested checks passed on the frozen measured source. The [check receipt](proxy-v2-checks/checks.json) records commands, exit codes, timings and source cleanliness; compressed logs are retained beside it.

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Exit 0; source unchanged |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `cargo test --workspace --release` | 660 passed, 0 failed, 4 ignored |
| `cargo test --workspace` | 660 passed, 0 failed, 4 ignored |
| `cargo test -p legatum-sim --test determinism` | 8 passed, 0 failed |

Six focused unit tests and ten real-engine scenarios cover deterministic decisions, hidden-state isolation, ranged/melee phase ordering, captures, defender purchases and local defense. Every one of the 900 benchmark games additionally passed complete duplicate-play and save/load determinism checks and hash-verified incident analysis.

- Easy: [Markdown](2026-09-28-proxy-v2-current-vs-easy-standard.md), [gzip JSON](2026-09-28-proxy-v2-current-vs-easy-standard.json.gz).
- Normal: [Markdown](2026-09-28-proxy-v2-current-vs-normal-standard.md), [gzip JSON](2026-09-28-proxy-v2-current-vs-normal-standard.json.gz).
- Hard: [Markdown](2026-09-28-proxy-v2-current-vs-hard-standard.md), [gzip JSON](2026-09-28-proxy-v2-current-vs-hard-standard.json.gz).
- [Artifact validation and SHA-256 hashes](proxy-v2-checks/benchmark-validation.json).

The JSON gzip streams have a zero timestamp and no embedded filename. Validation checked every seed, rotation, team/difficulty assignment, policy version, coverage count, snapshot median and determinism result, then verified decompression against the original JSON.

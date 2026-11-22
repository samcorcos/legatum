# Capital Science pacing — 2026-09-27

D1 adds +2 Science per turn to an empire's current capital, with a separate **Capital** line in its city yield breakdown. The bonus starts on the founding turn, applies equally to humans and AI at every speed, and is mirrored in the mock. No research prices, faction traits or AI decisions were tuned in this change.

The controlled Quick opening meets the first-tech target: all 800 trials complete Fabrication on resolved turn 10 or 11. Every selected Recovery technology completes within 14 research turns. These are 100 generated map seeds × eight factions following the specified economic opening below, without ruins, diffusion or attacks; they are not 800 observations of human play.

The AI median first/third discovery turns are **7/22 Quick** and **13/45 Standard**. Median game lengths are **148 / 288** turns (means 155.77 / 296.56). All 1,500 determinism checks pass with zero panics; Quick has one 600-turn safety-cap stop, while every Standard game ends by VP.

The universal ≤15-turn early-tech target is **not fully met in AI play**. Quick Recovery has 13 over-limit intervals among 55,887 completed technologies: 11 Signal Theory intervals take 16–17 turns and two Civic Networks intervals take 16. All Recovery medians are at most eight turns and P90s at most 12. In Consolidation, Machine Tools is the main bottleneck: median 15, P90 26, maximum 33; 3,950/7,997 completions (49.4%) take more than 15 turns. Intervals include any idle time; their exact definition is below. This change intentionally does not reprice the tree to force a pass.

Faction balance remains outside pillar 8's 1.5× criterion: **3.88× Quick** (Celts 19.0% / Athens 4.9%) and **4.00× Standard** (Mongols 22.4% / Athens 5.6%). Rebalance and held-out validation remain for the later run with diplomacy.

## Capital ownership and explanation

`GameState::capital_of(owner)` selects an owned city whose capital flag is set. Capture clears that flag. An existing secondary city is not automatically promoted: the next city founded while the empire has no capital becomes its replacement. Capturing or recapturing an old capital does not stack the bonus. Resistance suppresses all yields, including capital Science. The guide and zero-income explanation now describe founding, capital loss, growth and resistance accurately.

Core regression coverage exercises every faction and speed, founding-turn income, the view and breakdown sum, secondary cities, resistance, capture, replacement founding and recapture. Real-WASM/mock comparisons check the displayed Capital source, city and empire Science, stored Science, capture and replacement founding across save/load boundaries.

## Cohorts and reproduction

The AI cohorts use eight Normal scripted empires, one of each faction shuffled into seats on the standard 48×32 map. Quick uses seeds 700001–701000; Standard uses 700001–700500. There is no gameplay turn limit. The existing simulation safety caps are 600 and 1,200 turns respectively. Each game is played twice with per-turn hash comparison and a final save/load check; only the first play contributes observations. Four worker threads ran each batch. Shared-host contention makes wall-clock throughput unsuitable for comparing engine performance.

The simulation source revision is `4cb53642408154d0bc2f260ae405e3f5fe68e2b9`. The [source manifest](2026-09-27-capital-science-source.json.gz) records source and executable SHA-256s. The runner path was subsequently replaced by the release-determinism test build, after the original binary hash was captured and while both cohorts were already running; the manifest preserves both hashes and timestamps. Later integration of the optional AI-analysis CLI did not change core, AI or WASM rules; that observer was not enabled in these runs. This is a development-seed pacing measurement, not the later diplomacy balance acceptance run or an independent held-out evaluation.

| Cohort | Games | Runner report | Raw observations, gzip -9 |
| --- | --- | --- | --- |
| Quick | 1,000 | [Report](2026-09-27-capital-science-quick.md) | [JSON](2026-09-27-capital-science-quick.json.gz) |
| Standard | 500 | [Report](2026-09-27-capital-science-standard.md) | [JSON](2026-09-27-capital-science-standard.json.gz) |
| Controlled Quick opening | 800 trials | Tables below | [JSON](2026-09-27-capital-science-opening-quick.json.gz) |

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 1000 --speed quick --seed 700001 --determinism --threads 4 --label capital-science-quick --status-file target/capital-science-quick-status.json
./target/release/legatum-sim --games 500 --speed standard --seed 700001 --determinism --threads 4 --label capital-science-standard --status-file target/capital-science-standard-status.json
cargo run --release -p legatum-core --example science_opening -- quick 100 700001 > docs/sim-results/2026-09-27-capital-science-opening-quick.json
python3 scripts/summarize-capital-science.py docs/sim-results/2026-09-27-capital-science-quick.json.gz docs/sim-results/2026-09-27-capital-science-standard.json.gz docs/sim-results/2026-09-27-capital-science-opening-quick.json.gz
```

All JSON companions are compressed with `gzip -9 -c` and checked by decompressing and comparing against the raw bytes. The game-length table includes turns observed before a safety-cap stop; such a length is a lower bound on time to victory, not a completed game. Capped games award no win and remain in the faction-seat denominator.

## AI first and third technologies

All initial empires with first-play telemetry, including eliminated empires. Resolved turn 1 is the first turn. Quantiles are min / P10 / median / P90 / max, using the runner's rounded-index convention. Missing milestones stay in the denominator; timing quantiles condition on reaching them. Ruin Science and diffusion remain enabled in the AI games. Determinism replays are not extra observations.

| Speed | Technology number | Reached | Never reached | Completion turn quantiles |
| --- | --- | --- | --- | --- |
| Quick | 1 | 8000/8000 (100.0%) | 0 | 1 / 1 / 7 / 9 / 11 |
| Quick | 3 | 8000/8000 (100.0%) | 0 | 3 / 14 / 22 / 35 / 45 |
| Standard | 1 | 4000/4000 (100.0%) | 0 | 1 / 5 / 13 / 18 / 23 |
| Standard | 3 | 4000/4000 (100.0%) | 0 | 10 / 38 / 45 / 75 / 89 |

## Controlled opening

Settle in place on turn 1; select Fabrication on turn 2, then Signal Theory, Soil Reclamation, Exchange, Civic Networks, Fortification, Salvage Protocols without research gaps. Build Scout, Workshop, Laboratory, Granary as legal with normal paid production; no purchases or expansion. Units stay still, so there are no ruins or combat; rivals retain starting Settlers and never research, so no diffusion. Resolved turn 1 is the first turn. Research durations include the selection and completion turns. Factions share each map seed; trials are not independent human games.

100 map seeds beginning at 700001, eight factions per map, 800 Quick trials. First-tech target 8–12 resolved turns: 800/800 (100.0%). First-tech quantiles: 10 / 10 / 10 / 10 / 11. This is a specified human-like economic opening, not measured human play or a competitive full-game proxy.

| Technology | Completed | Completion turn quantiles | Selected-to-completed turns | At most 15 turns |
| --- | --- | --- | --- | --- |
| Fabrication | 800/800 (100.0%) | 10 / 10 / 10 / 10 / 11 | 9 / 9 / 9 / 9 / 10 | 800/800 (100.0%) |
| SignalTheory | 800/800 (100.0%) | 22 / 23 / 24 / 24 / 25 | 12 / 13 / 14 / 14 / 14 | 800/800 (100.0%) |
| SoilReclamation | 800/800 (100.0%) | 27 / 28 / 30 / 30 / 31 | 5 / 5 / 6 / 6 / 7 | 800/800 (100.0%) |
| Exchange | 800/800 (100.0%) | 34 / 35 / 36 / 39 / 43 | 4 / 6 / 8 / 9 / 12 | 800/800 (100.0%) |
| CivicNetworks | 800/800 (100.0%) | 39 / 42 / 44 / 47 / 51 | 4 / 6 / 7 / 11 / 12 | 800/800 (100.0%) |
| Fortification | 800/800 (100.0%) | 42 / 46 / 48 / 52 / 56 | 3 / 3 / 4 / 5 / 6 | 800/800 (100.0%) |
| SalvageProtocols | 800/800 (100.0%) | 45 / 50 / 52 / 56 / 60 | 2 / 3 / 4 / 5 / 5 | 800/800 (100.0%) |

## AI early-era completion intervals

An interval is the resolved completion turn minus that empire's previous completion turn, or minus zero for its first technology. It includes idle turns and research switches; it is the wait between discoveries, not time exclusively assigned to a technology. Stored Science, ruins and diffusion can shorten it. Quantiles condition on completion; missing technologies are reported as not completed, not zero-duration. Both opening eras are shown so the ≤15 Quick-turn target cannot hide slow individual technologies behind an overall median.

| Speed | Era | Technology | Completed | Not completed | Interval quantiles | At most 15 turns |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | Recovery | SoilReclamation | 8000 | 0 | 1 / 1 / 6 / 8 / 10 | 8000/8000 (100.0%) |
| Quick | Recovery | Fabrication | 8000 | 0 | 1 / 2 / 5 / 9 / 11 | 8000/8000 (100.0%) |
| Quick | Recovery | Fortification | 7998 | 2 | 1 / 2 / 4 / 9 / 14 | 7998/7998 (100.0%) |
| Quick | Recovery | Exchange | 8000 | 0 | 1 / 5 / 8 / 11 / 14 | 8000/8000 (100.0%) |
| Quick | Recovery | SignalTheory | 7999 | 1 | 1 / 5 / 8 / 12 / 17 | 7988/7999 (99.9%) |
| Quick | Recovery | CivicNetworks | 7994 | 6 | 1 / 4 / 7 / 12 / 16 | 7992/7994 (100.0%) |
| Quick | Recovery | SalvageProtocols | 7896 | 104 | 1 / 1 / 3 / 7 / 14 | 7896/7896 (100.0%) |
| Quick | Consolidation | Ballistics | 7987 | 13 | 1 / 3 / 7 / 15 / 44 | 7285/7987 (91.2%) |
| Quick | Consolidation | DeepSurvey | 7175 | 825 | 1 / 4 / 6 / 13 / 29 | 6796/7175 (94.7%) |
| Quick | Consolidation | IntensiveCultivation | 7487 | 513 | 1 / 3 / 7 / 13 / 27 | 7184/7487 (96.0%) |
| Quick | Consolidation | MachineTools | 7997 | 3 | 3 / 10 / 15 / 26 / 33 | 4047/7997 (50.6%) |
| Quick | Consolidation | PublicAdministration | 7118 | 882 | 1 / 3 / 5 / 10 / 29 | 7084/7118 (99.5%) |
| Quick | Consolidation | DiplomaticProtocols | 6317 | 1683 | 2 / 3 / 5 / 7 / 21 | 6291/6317 (99.6%) |
| Quick | Consolidation | FieldLogistics | 7892 | 108 | 1 / 3 / 7 / 11 / 25 | 7836/7892 (99.3%) |
| Quick | Consolidation | Academies | 5917 | 2083 | 1 / 3 / 5 / 10 / 28 | 5740/5917 (97.0%) |
| Quick | Consolidation | CommercialLaw | 6861 | 1139 | 1 / 3 / 5 / 11 / 21 | 6803/6861 (99.2%) |
| Standard | Recovery | SoilReclamation | 4000 | 0 | 1 / 5 / 10 / 14 / 17 | 3817/4000 (95.4%) |
| Standard | Recovery | Fabrication | 4000 | 0 | 1 / 9 / 12 / 18 / 23 | 3232/4000 (80.8%) |
| Standard | Recovery | Fortification | 3999 | 1 | 1 / 4 / 9 / 19 / 25 | 3383/3999 (84.6%) |
| Standard | Recovery | Exchange | 4000 | 0 | 1 / 9 / 16 / 22 / 28 | 1941/4000 (48.5%) |
| Standard | Recovery | SignalTheory | 4000 | 0 | 4 / 11 / 15 / 25 / 31 | 2070/4000 (51.8%) |
| Standard | Recovery | CivicNetworks | 3997 | 3 | 2 / 9 / 13 / 23 / 59 | 2617/3997 (65.5%) |
| Standard | Recovery | SalvageProtocols | 3924 | 76 | 1 / 3 / 6 / 18 / 42 | 3481/3924 (88.7%) |
| Standard | Consolidation | Ballistics | 3985 | 15 | 1 / 7 / 14 / 30 / 53 | 2246/3985 (56.4%) |
| Standard | Consolidation | DeepSurvey | 3511 | 489 | 4 / 8 / 13 / 26 / 61 | 2120/3511 (60.4%) |
| Standard | Consolidation | IntensiveCultivation | 3690 | 310 | 2 / 5 / 14 / 26 / 50 | 2072/3690 (56.2%) |
| Standard | Consolidation | MachineTools | 3996 | 4 | 7 / 21 / 31 / 52 / 126 | 144/3996 (3.6%) |
| Standard | Consolidation | PublicAdministration | 3424 | 576 | 3 / 5 / 10 / 20 / 54 | 2688/3424 (78.5%) |
| Standard | Consolidation | DiplomaticProtocols | 2957 | 1043 | 3 / 6 / 9 / 15 / 51 | 2718/2957 (91.9%) |
| Standard | Consolidation | FieldLogistics | 3901 | 99 | 3 / 6 / 14 / 21 / 66 | 2459/3901 (63.0%) |
| Standard | Consolidation | Academies | 2764 | 1236 | 3 / 7 / 10 / 19 / 68 | 2200/2764 (79.6%) |
| Standard | Consolidation | CommercialLaw | 3245 | 755 | 2 / 5 / 10 / 21 / 38 | 2458/3245 (75.7%) |

## Game length and determinism

| Speed | Games | Mean turns | Turn quantiles | VP wins | Stalls | Panics | Determinism passes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | 1000 | 155.77 | 67 / 123 / 148 / 198 / 600 | 999 | 1 | 0 | 1000/1000 |
| Standard | 500 | 296.56 | 146 / 239 / 288 / 367 / 542 | 500 | 0 | 0 | 500/500 |

## Faction win rates

Wins per initial faction seat, including eliminated empires.

| Faction | Quick | Standard |
| --- | --- | --- |
| Japan | 94/1000 (9.4%) | 43/500 (8.6%) |
| Germany | 171/1000 (17.1%) | 69/500 (13.8%) |
| Mongols | 161/1000 (16.1%) | 112/500 (22.4%) |
| Sparta | 61/1000 (6.1%) | 37/500 (7.4%) |
| Byzantium | 96/1000 (9.6%) | 53/500 (10.6%) |
| Athens | 49/1000 (4.9%) | 28/500 (5.6%) |
| Switzerland | 177/1000 (17.7%) | 78/500 (15.6%) |
| Celts | 190/1000 (19.0%) | 80/500 (16.0%) |

Quick: maximum Celts 19.0%; minimum Athens 4.9%; max/min 3.88x.

Standard: maximum Mongols 22.4%; minimum Athens 5.6%; max/min 4.00x.

## Incomplete game and limits

Quick seed **700347** reaches the 600-turn safety cap with no winner. It replays deterministically. The highest final score is Celts's 8 VP; seven empires know all 32 technologies, while the eliminated Byzantium has 27. The cause of the unfinished victory race was not diagnosed here. Its 600 observed turns are included in the length statistics, and all eight initial seats remain in win-rate denominators. Quick faction win percentages therefore sum to 99.9%, not 100%.

The opening harness isolates a Fabrication-first economic plan. It does not establish timing for every first-tech choice, delayed settlement, attacks or capital loss. AI intervals are measured between discoveries, not instrumented selection-to-completion times. Late or never-completed technologies remain visible in the denominators. No faction tuning or claim of post-diplomacy balance acceptance is made.

## Validation and intentional fixture regeneration

| Check | Result |
| --- | --- |
| `cargo fmt --all` and final `cargo fmt --all --check` | Exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 after integrating the analysis crate |
| `cargo test --workspace` | Exit 0; 547 passed, 0 failed, 4 existing ignored tests |
| `scripts/build-wasm.sh` | Exit 0; optimized web WASM build |
| `npm test && npm run build` in `app/` | Exit 0; 703 passed, 1 existing mock-only archive-test skip; TypeScript and Vite build pass |
| `cargo test --release -p legatum-sim --test determinism` | Exit 0; 7 passed |
| Node-target WASM build and `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | Exit 0; all 4 tapes, 240 turns, initial/per-turn/save-load hashes match |
| Quick / Standard simulation commands | Both exit 0; 1,000/1,000 and 500/500 determinism checks pass |
| JSON archival audit | Exact seed ranges, counts and all determinism records checked; gzip -9 companions decompress byte-for-byte to the raw reports |

The writable WASM tool cache was set to `$PWD/target/wasm-pack-cache` with `WASM_PACK_CACHE`; the first attempt hit the default cache's filesystem restriction. No optimizer or check was disabled. Earlier app runs encountered an existing five-second timeout and a wall-clock performance assertion under heavy host load; the isolated performance test and the final unmodified full command pass. An overlapping Rust rebuild invalidated an initial server doctest artifact; the final workspace command was rerun without overlapping Rust builds and passes.

The three mixed-AI fixtures and the research-queue fixture were intentionally regenerated using:

```sh
cargo run --release -p legatum-sim -- --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/fixture-reports
cargo run --release -p legatum-wasm --example research_queue_tape > crates/sim/tests/fixtures/research-queue.json
```

All four initial hashes are unchanged. All 240 turn hashes change beginning on turn 1, when founded capitals earn Science. The three mixed-AI order sequences first differ on turns 8, 6 and 5; all 120 queue-tape order maps are unchanged. Frozen original-three compatibility fixtures remain untouched. The [fixture README](../../crates/sim/tests/fixtures/README.md) records this rule-change rationale.

`master` was merged into `astra/science-pacing`, including the optional AI-analysis tools, without changing the cohort's core/AI/WASM sources. The final integration after testing brought only documentation and watcher-tool updates. `docs/learnings.md` and `CLAUDE.md` were not manually edited.

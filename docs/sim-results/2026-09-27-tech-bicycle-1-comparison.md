# Tech-tree Bicycle: first-slice comparison (2026-09-27)

The requested 1,000 Quick and 500 Standard games all finish by VP, with zero stalls, panics or determinism failures. Median length rises from 139 to 176 Quick turns and from 267 to 333 Standard turns on matched seeds. **The inclusive 6–20% faction band fails:** Germany wins 39.0% / 38.4%; Japan and Athens are below 6% at both speeds, and Mongols is below 6% on Standard. These results are evidence of a balance regression, not acceptance of the full Bicycle design.

This slice implements the 25 technologies in Recovery, Consolidation and Integration (7 / 9 / 9; 2,665 base Science), paid building-family upgrades, standalone Embassy/Aid Station/Archive/Reactor, Marksmen and siege. Core remains integer-only; costs use `GameSpeed::scale_cost`, faction modifiers and existing diffusion. Population gates apply when starting or buying upgrades; completed tiers retain the base family and faction yields. No standing setting was introduced. Great Works, the tall milestone alternative and diplomacy-dependent benefits remain absent. The existing Relay Uplink and scoring are retained. Federated Markets is a paid research node with no benefit in this slice, so the scripted AI deliberately skips it.

## Campaigns and comparison population

- [Quick generated report](2026-09-27-tech-bicycle-1-quick.md), [gzip -9 JSON](2026-09-27-tech-bicycle-1-quick.json.gz): 1,000 games, seeds 700001–701000.
- [Standard generated report](2026-09-27-tech-bicycle-1-standard.md), [gzip -9 JSON](2026-09-27-tech-bicycle-1-standard.json.gz): 500 games, seeds 700001–700500.
- Baseline: [approved AI endgame comparison](2026-09-27-ai-endgame-comparison.md) and its archived Quick/Standard JSON. Standard comparisons below use its first 500 games unless explicitly marked “published 1,000.”

Both speeds use Normal scripted brains, eight shuffled faction seats (one each), 48×32 maps, no turn limit, unchanged safety caps of 600 / 1,200 and eight simulation threads. Every game is played twice with per-turn hash comparison and final save/load validation. Report metadata records base checkout `0960f80ce265d611b63727fd1fa460c012cabba9`, dirty at run start. These reports accompany that implementation in the same commit; subsequent changes were regression fixtures/tests, UI previews and documentation. This compares the combined rules and AI change, not an isolated causal experiment. The fixed seeds were used during development, not held out.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 1000 --seed 700001 --speed quick --determinism --threads 8 --label tech-bicycle-1-quick --out-dir target/bicycle-final
./target/release/legatum-sim --games 500 --seed 700001 --speed standard --determinism --threads 8 --label tech-bicycle-1-standard --out-dir target/bicycle-final
```

Generated Markdown is copied unchanged. JSON was compressed with `gzip -9 -c` and verified byte-for-byte after decompression. Quantiles use the simulation report convention: sort, then select index `(percentile * (n - 1) + 50) // 100`. Counts below include only first plays.

## Length, VP share and stalls

| Speed | Cohort | Games | Mean turns | Min / P10 / median / P90 / max | VP outcomes | Stalls | Panics |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | Baseline | 1000 | 146.6 | 71 / 113 / 139 / 187 / 433 | 100.0% | 0 | 0 |
| Quick | Bicycle | 1000 | 175.0 | 81 / 126 / 176 / 219 / 326 | 100.0% | 0 | 0 |
| Standard | Baseline | 500 | 275.6 | 159 / 215 / 267 / 347 / 560 | 100.0% | 0 | 0 |
| Standard | Bicycle | 500 | 330.4 | 162 / 236 / 333 / 413 / 548 | 100.0% | 0 | 0 |

The published 1,000-game Standard baseline has mean 276.7 and median 268 (158 / 216 / 268 / 353 / 748). The matched 500-game baseline median is 267. There are no censored stops in either matched comparison. New determinism results are 1,000/1,000 Quick and 500/500 Standard, with zero mismatches. Neither cohort has an outright conquest victory; taking cities instead contributes to VP wins.

| Faction | Baseline Quick (1,000) | Bicycle Quick (1,000) | Baseline Standard (500) | Bicycle Standard (500) | 6–20% at both speeds |
| --- | --- | --- | --- | --- | --- |
| Japan | 8.5% | 5.1% | 6.6% | 4.8% | FAIL |
| Germany | 17.7% | 39.0% | 16.4% | 38.4% | FAIL |
| Mongols | 9.0% | 7.0% | 8.4% | 4.6% | FAIL |
| Sparta | 12.0% | 8.8% | 18.2% | 10.2% | PASS |
| Byzantium | 13.6% | 6.1% | 11.0% | 7.4% | PASS |
| Athens | 6.7% | 3.1% | 5.2% | 3.8% | FAIL |
| Switzerland | 12.9% | 15.2% | 13.8% | 13.4% | PASS |
| Celts | 19.6% | 15.7% | 20.4% | 17.4% | PASS |

The published 1,000-game baseline meets the band for all eight factions. Its first 500 Standard games alone put Athens at 5.2% and Celts at 20.4%; this sampling distinction does not explain Germany rising to 38.4% in the matched cohort. The approved technology/building costs and current faction rules have not been altered to force a passing rate.

Terminal winning-team VP shares show how the point mix changes. Each percentage divides that source by the total terminal VP of winners in that row; rows are separate populations. “Other milestones” means Four Cities, Six Technologies and Twenty Population.

| Speed | Cohort | Winner VP | Live titles | Other milestones | Relay milestone | Faction goals |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | Baseline | 10158 | 3725 (36.7%) | 2995 (29.5%) | 1666 (16.4%) | 1772 (17.4%) |
| Quick | Bicycle | 10188 | 3920 (38.5%) | 3000 (29.4%) | 1296 (12.7%) | 1972 (19.4%) |
| Standard | Baseline | 5074 | 1980 (39.0%) | 1500 (29.6%) | 722 (14.2%) | 872 (17.2%) |
| Standard | Bicycle | 5086 | 2112 (41.5%) | 1500 (29.5%) | 486 (9.6%) | 988 (19.4%) |

## Research completion by era

These are technologies actually known at game end, including eliminated initial seats. “All seats” uses 8,000 Quick / 4,000 Standard empires; “Winners” uses 1,000 / 500. Total completions counts each learned technology once per empire; it is not science spent or the time of first entry into an era. Baseline JSON predates era telemetry and has a different nine-technology catalog, so no comparable baseline era counts are inferred.

| Speed | Population | Era (available) | Total completions | Mean | Min / P10 / median / P90 / max | Whole era completed |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | All seats | Recovery (7) | 54576 | 6.82 | 2 / 6 / 7 / 7 / 7 | 7017/8000 |
| Quick | All seats | Consolidation (9) | 61084 | 7.64 | 0 / 4 / 9 / 9 / 9 | 5008/8000 |
| Quick | All seats | Integration (9) | 31704 | 3.96 | 0 / 0 / 3 / 8 / 8 | 0/8000 |
| Quick | Winners | Recovery (7) | 6990 | 6.99 | 6 / 7 / 7 / 7 / 7 | 990/1000 |
| Quick | Winners | Consolidation (9) | 8734 | 8.73 | 2 / 9 / 9 / 9 / 9 | 915/1000 |
| Quick | Winners | Integration (9) | 6828 | 6.83 | 0 / 2 / 8 / 8 / 8 | 0/1000 |
| Standard | All seats | Recovery (7) | 26843 | 6.71 | 2 / 6 / 7 / 7 / 7 | 3249/4000 |
| Standard | All seats | Consolidation (9) | 28290 | 7.07 | 0 / 2 / 9 / 9 / 9 | 2136/4000 |
| Standard | All seats | Integration (9) | 12541 | 3.14 | 0 / 0 / 2 / 8 / 8 | 0/4000 |
| Standard | Winners | Recovery (7) | 3494 | 6.99 | 6 / 7 / 7 / 7 / 7 | 494/500 |
| Standard | Winners | Consolidation (9) | 4306 | 8.61 | 1 / 8 / 9 / 9 / 9 | 436/500 |
| Standard | Winners | Integration (9) | 3140 | 6.28 | 0 / 0 / 8 / 8 / 8 | 0/500 |

Quick: all-seat mean 18.42, median 19 technologies; winner mean 22.55, median 24. 779/1000 winners finish every currently useful technology.

Standard: all-seat mean 16.92, median 17 technologies; winner mean 21.88, median 24. 344/500 winners finish every currently useful technology.

No scripted empire completes all nine Integration technologies because Federated Markets has no implemented unlock. A median winning empire reaching 24/25 also means research specialization is weak by the finish: the bigger distinction is what each empire can afford to build. This is a first-slice observation, not evidence that the proposed 32-technology final tree meets its 18–22 completion target or era-entry timing.

## Paid content actually used

| Investment or event | Quick (1,000 games) | Standard (500 games) |
| --- | --- | --- |
| Hydroponics | 14904 | 5764 |
| MachineShop | 15318 | 6477 |
| Assembly | 2013 | 485 |
| Embassy | 6003 | 1976 |
| AidStation | 5906 | 1975 |
| College | 8682 | 3131 |
| Archive | 4197 | 1273 |
| Clearinghouse | 8308 | 2921 |
| Arcology | 3760 | 1003 |
| Autoworks | 5747 | 1871 |
| Citadel | 3655 | 869 |
| Reactor | 2738 | 933 |
| Marksman trained | 30435 | 13525 |
| SiegeEngine trained | 13215 | 6095 |
| Ranger → Marksman paid upgrades | 1643 | 571 |
| Marksman ranged battles | 173079 | 99696 |
| Siege Engine ranged battles | 55335 | 33749 |

Production counts come from `Event::ProductionCompleted`, including completed purchases, excluding starting and captured pieces. Upgrade counts come from `Event::UnitUpgraded`. Ranged counts come from ranged `Event::Battle` records keyed by the attacker’s base strength label; they include attacks on units/garrisons as well as city HP, so they do not claim every siege shot received the +12 city bonus. These counters are read-only simulation telemetry and never feed the AI, saves, RNG or hashes.

The AI uses only `PlayerView`: family-aware production choices, population/food gates, quoted costs, research detours, paid Ranger upgrades, Marksman quotas and escorted siege formations directed toward reachable visible/remembered enemy cities. Its existing campaign uncertainty and attack-safety checks remain. It does not gain a hidden-map target or the simulation counters.

## Conquest and development winning paths

Classify a winner as **capture-assisted** if that empire recorded at least one city capture, including recaptures; otherwise classify it as **no city captures**. This measures observed histories, not an assigned strategy or proof of peace. Both groups can fight, capture settlers, expand by settlement and develop infrastructure. All these wins still use the VP victory condition. The baseline lacks per-empire capture counters, so its winners cannot be classified identically.

| Speed | Winner history | Wins | Median turns | Mean final cities | Mean final population | Mean captures | Relay milestone held | Mean title VP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | No city captures | 260/1000 (26.0%) | 152 | 5.63 | 34.00 | 0.00 | 141/260 | 3.99 |
| Quick | Capture-assisted | 740/1000 (74.0%) | 184 | 8.24 | 49.76 | 2.46 | 507/740 | 3.90 |
| Standard | No city captures | 43/500 (8.6%) | 251 | 6.42 | 29.84 | 0.00 | 10/43 | 4.56 |
| Standard | Capture-assisted | 457/500 (91.4%) | 338 | 9.60 | 51.33 | 3.94 | 233/457 | 4.19 |

The paths differ in actual territorial gains, end-state scale and time: winners without city captures finish earlier and smaller; capture-assisted winners finish with more cities and population. Relay is optional in both groups. These observations do **not** establish distinct balanced specializations: there are no outright conquest wins, the median winner almost exhausts the useful research tree, and both groups remain wide. The deferred tall milestone and Great Work alternatives cannot be evaluated here.

| Speed | Seed | Winner | Turns | Captures | Final cities | Techs | Relay VP | Marksmen trained | Siege trained |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | 700007 | Germany | 160 | 0 | 4 | 24 | Yes | 4 | 2 |
| Quick | 700001 | Celts | 234 | 4 | 10 | 24 | No | 11 | 8 |
| Standard | 700023 | Germany | 251 | 0 | 7 | 19 | No | 0 | 0 |
| Standard | 700003 | Celts | 398 | 7 | 12 | 24 | No | 12 | 8 |

Quick’s no-capture winners include 163 Germany empires out of 260; Standard’s include 21 out of 43. Germany also wins 227 capture-assisted Quick games and 171 Standard games. Industry strength is therefore not confined to a peaceful route. Its existing Foundry output and building discount continue through the paid family ladder; industrial compounding is a plausible contributor to the imbalance, but these combined rules/AI campaigns do not isolate causality.

## Verification and limits

- `cargo fmt --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: 437 passed, 0 failed, 4 ignored across 24 test suites (including doc-test suites).
- `cargo test -p legatum-sim --test determinism`: 6 passed, 0 failed.
- `cargo test --release -p legatum-sim --test determinism`: 6 passed, 0 failed.
- `scripts/build-wasm.sh`: passed.
- `wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node`: passed, with `WASM_PACK_CACHE=/tmp/legatum-wasm-pack-cache` for the sandbox-writable cache.
- `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node`: 3 tapes passed, 40 turns each; initial, every-turn and save/load hashes match native.
- In `app/`, `npm test`: 427 passed, 0 failed, 45 files passed, including WASM tests.
- In `app/`, `npm run build`: passed (TypeScript and Vite).
- `git diff --check`: passed. Compressed campaign JSON round-trips byte-for-byte; summed per-empire capture counters equal each game's canonical capture count, era counts equal each seat's learned-tech count, and terminal winner VP sources reconcile.

Focused tests cover predecessor/population gates, completion after population loss, inherited faction yields and goals, Reactor queue exclusivity and food costs, Scout fog boundaries, healing before faction penalties, Citadel defense, siege against garrisons versus city HP, matching combat forecasts, Ranger upgrades and new unit campaign use. App tests cover the larger connector layout, prerequisite paths, forecasts, accessibility and mock rules.

All three high-u64 determinism tapes were intentionally regenerated using the fixture README command. Initial hashes are unchanged; the first changed recorded orders and hashes are at turn 1 in all three tapes. The expanded available research list changes random-brain choices; revised scripted choices, rules and fog-city Citadel serialization then affect subsequent hashes. Native debug/release and WASM replay verify the new tapes; historical rule versions are not emulated.

Visual inspection remains unverified: the sandbox denied the headless browser launch and the browser connector was unavailable. Build, accessibility and connector-geometry tests pass, but they do not substitute for seeing the larger tree in a live browser. Human play, multi-team tuning and full Bicycle acceptance remain outstanding. No Great Work, tall-milestone or diplomacy placeholder was introduced, and `docs/learnings.md` was not edited.

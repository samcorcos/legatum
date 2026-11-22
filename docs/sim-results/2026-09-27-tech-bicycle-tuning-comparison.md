# Bicycle investment and pacing tuning (2026-09-27)

The tuned slice curbs Germany's dominant industrial feedback loop and restores the requested finish window without changing victory targets or VP awards. On matched development seeds, median length falls from **178 to 156 Quick turns** and **333 to 299 Standard turns**. Median winners finish **20 / 19 technologies instead of 24 / 24**. The frozen candidate's held-out medians are **157 / 295**, with **zero stalls, zero panics and 100% determinism** in both campaigns.

The faction result is close to the requested *roughly* 6–20% band, not a literal all-cohort pass. Development Celts is 20.7% Quick / 22.0% Standard. Held-out Standard puts all eight factions inside the band; held-out Quick puts Athens at **5.4% (54/1,000)**, 0.6 percentage points below the floor. Keep these visible in review. Quick meets the requested 110–160 acceptance window, but still exceeds the original 110–150 design target by 6–7 turns. There were no rule or AI changes after looking at held-out outcomes.

## Comparison and provenance

The first command was `git merge astra/tech-tree-bicycle-1`, producing merge tree `308d6e5d1c07b72e13433eb6c7ddeede7d1cef8e` on `astra/tech-bicycle-1-tuning`. The [original Bicycle report](2026-09-27-tech-bicycle-1-comparison.md) measured a different integration tree. Master also contributed its settler reconnaissance fix, so this report uses a fresh baseline from the merged tree: Germany 38.1% / 35.4% and lengths 178 / 333, rather than mixing those runs with the published 39.0% / 38.4% and 176 / 333.

Each campaign uses Normal scripted brains, one of every faction shuffled into eight single-empire teams, 48×32 maps, no turn limit, unchanged safety caps of 600 / 1,200, eight threads and `--determinism`. Each game is played twice with per-turn hash comparison and final save/load validation. Only first plays enter the statistics. Development uses Quick seeds 700001–701000 and Standard 700001–700500. Held-out uses Quick 910001–911000 and Standard 910001–910500. Quick and Standard sharing a seed does not make their gameplay identical: speed scales economic costs and timers, not movement or combat.

There were 21 development parameter sets including the baseline, all at the full requested 1,000 Quick / 500 Standard scale. They reuse development seeds and are not independent confirmations. The [experiment ledger](2026-09-27-tech-bicycle-tuning-experiments.md) records every trial, its results, frozen executable hash and exact engine/AI diff from the merge tree. Small diagnostic and trace archives are exploratory evidence, not additional acceptance cohorts.

The selected trial is `network`, frozen at **2026-09-27T07:41:37.455491Z**, before held-out execution. Executable SHA-256: `f7cc6ce7bc16e898b2cbd44660715bc27fb4ac68a31b6978a34c635e28fd8abd`. Engine/AI diff SHA-256: `51121e81a26c68e457ef8315d2612f7e0aae3002ebfe83152c95c68b25fdfb47`. Both development and held-out campaigns used that same frozen executable. Final engine/AI source was checked against the frozen diff. Report metadata correctly records the merge commit and a dirty checkout; the ledger's patches identify the actual tested rules. Later presentation edits only corrected speed-scaled tooltips.

| Cohort | Quick report / raw JSON | Standard report / raw JSON |
| --- | --- | --- |
| Merged baseline | [Report](2026-09-27-tech-bicycle-tuning-baseline-quick.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-baseline-quick.json.gz) | [Report](2026-09-27-tech-bicycle-tuning-baseline-standard.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-baseline-standard.json.gz) |
| Selected development | [Report](2026-09-27-tech-bicycle-tuning-network-quick.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-network-quick.json.gz) | [Report](2026-09-27-tech-bicycle-tuning-network-standard.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-network-standard.json.gz) |
| Held-out validation | [Report](2026-09-27-tech-bicycle-tuning-heldout-quick.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-heldout-quick.json.gz) | [Report](2026-09-27-tech-bicycle-tuning-heldout-standard.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-heldout-standard.json.gz) |

Generated Markdown is copied unchanged; same-stem JSON archives use gzip level 9 and were verified byte-for-byte after decompression. Quantiles sort the observations and select index `(percentile * (n - 1) + 50) // 100`, matching the sim runner.

## What changed and why

The industrial problem was a concrete interaction: the Foundry's +6 Production remained underneath the new additive family ladder, while Germany's 20% building discount also reduced the price of every new tier. Production paid for more production, science buildings and troops before opponents could exploit the investment window. Ordinary workshops add +2; the old completed Germany family provided +14 versus an ordinary +10.

| Industrial investment | Before | Tuned | Simple marginal payback, Quick turns |
| --- | --- | --- | --- |
| Machine Shop, ordinary / Germany | 100P / 80P, +3P | 160P / 160P, +3P | 33.3 / 26.7 → 53.3 / 53.3 |
| Autoworks, ordinary / Germany | 160P / 128P, +5P | 300P / 300P, +5P | 32.0 / 25.6 → 60.0 / 60.0 |
| Germany Foundry | 48P, +6P | 48P, +4P | Still twice an ordinary Workshop's output |

Payback is added cost divided by added output after completion, before distance penalties or opportunity costs; it is not an exact whole-city return forecast. Standard doubles these payback times. Germany still gets cheap new buildings (including standalone buildings), the exceptional Foundry, its original complete-city goal and its +20P unit weakness. Family upgrades pay full price; their inherited Foundry still gives the completed industrial family +12 versus ordinary +10. This is a change to where its industry advantage compounds, not a universal faction multiplier or shared personality.

The isolated `industry` trial changed tier prices and the discount rule while leaving the +6 Foundry. Germany fell from 38.1% / 35.4% to 31.3% / 32.8%, but medians worsened to 186 / 346. Industry pricing alone did not fix either the whole advantage or pacing. Later, the combined Foundry reduction and Ballistics restoration (`specialization` → `windows`) reduced Germany from 32.3% / 28.2% to 18.6% / 16.4%. Those two simultaneous changes cannot be assigned separate causal shares.

Late research was too affordable once the science economy grew, while the existing finish project had been pushed behind a large industrial prerequisite detour. Making everything slower worsened that mismatch. The final research changes preserve Recovery prices, make advanced branches costly commitments, restore the earlier military window, and make the existing Relay route reachable without buying into industry.

| Technology or project | Before | Tuned |
| --- | --- | --- |
| Ballistics | 80S | 60S, restoring its pre-Bicycle price |
| Relay Restoration | 220S; Recovered Power + Civic Networks | 80S; Signal Theory + Civic Networks |
| Relay Uplink | 400P, +2VP, cannot buy | 300P, +2VP, cannot buy |
| Closed Habitats | 180S | 360S |
| Industrial Automation | 200S | 400S |
| Research Networks | 190S | 380S |
| Mutual Defense | 170S | 340S |
| Recovered Power | 180S | 360S |
| Combined Arms / Siege Engineering | 180S each | 240S each |

The complete base Science cost becomes **3,545**, split Recovery **225**, Consolidation **750**, Integration **2,570**; before it was 2,665 = 225 + 770 + 1,670. The full Relay prerequisite closure costs 245S instead of 775S before diffusion, including Relay itself. Signal Theory still requires Fabrication. The 300P project remains a real production commitment; no free milestone, extra VP source, altered faction goal or lower victory target was introduced. Before the project repricing, the `listening` trial already brought lengths to 161 / 311; changing only the project to 300P (`relay300`) moved them to 155 / 303. That comparison supports using project cost as a secondary adjustment alongside the research route and late costs.

Athens' old 90P/+4S Listening House was only a small science edge over an 80P/+3S Laboratory, and a College narrowed the whole family's advantage to 8 versus 7 Science. Extra output locked behind a population-5 College did not repair its weak opening: the `colleges` trial still won only 5.1% / 6.4%. The final Listening House is **70P/+6S**, retaining +6 under the ordinary +4 College, for 10 versus 7 Science. Its science-only ruins, receiving-network goal, and +1G Market weakness stay intact. The bonus now arrives where a science faction can use it. The other six factions' rules are unchanged.

The sole retained AI adjustment values Machine Shops and Autoworks using their public quoted marginal Production over the planner's existing 30-Quick-turn horizon, divided by quoted cost. It respects speed and Japan's rounded distance penalty; the immediate-victory title override remains. It uses `PlayerView`, with faction-data fallback for old/test views lacking unlock forecasts, and reads no hidden state or telemetry. Experimental shared food urgency, root-tech discounts, extra College output and Intensive Cultivation repricing were rejected. The selected candidate retains the original faction personality knobs and Granary priority.

## Faction results

Every faction has 1,000 seats per Quick cohort and 500 per Standard cohort. Entries are wins divided by that faction's initial seats, including eliminated empires. These are point estimates for this scripted free-for-all setup, not confidence that each underlying rate is inside a hard band.

| Faction | Before Quick | Tuned Quick | Held-out Quick | Before Standard | Tuned Standard | Held-out Standard |
| --- | --- | --- | --- | --- | --- | --- |
| Japan | 4.9% | 8.5% | 9.2% | 4.6% | 6.2% | 6.4% |
| Germany | 38.1% | 18.0% | 18.7% | 35.4% | 15.0% | 17.8% |
| Mongols | 7.0% | 8.8% | 10.0% | 4.2% | 7.8% | 8.8% |
| Sparta | 9.6% | 8.9% | 8.7% | 10.8% | 14.8% | 11.2% |
| Byzantium | 5.8% | 10.2% | 8.8% | 8.2% | 11.4% | 10.0% |
| Athens | 3.5% | 6.5% | 5.4% | 5.0% | 7.4% | 8.6% |
| Switzerland | 15.2% | 18.4% | 19.3% | 14.4% | 15.4% | 17.6% |
| Celts | 15.9% | 20.7% | 19.9% | 17.4% | 22.0% | 19.6% |


No faction rule was adjusted to force a held-out rate into the band. Athens' held-out Quick result and Celts's development upper edge remain review watchpoints. A strict requirement that every observed percentage be within 6–20% would fail; the stronger claim that Germany's ~39% industrial domination is gone is supported in both seed families.

## Length and research completion

| Speed | Cohort | Games | Mean turns | Min / P10 / median / P90 / max | VP outcomes | Stalls / panics | Determinism |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | Before | 1000 | 177.51 | 87 / 130 / 178 / 220 / 393 | 1000/1000 | 0 / 0 | 1000/1000 |
| Quick | Tuned development | 1000 | 155.86 | 81 / 118 / 156 / 190 / 329 | 1000/1000 | 0 / 0 | 1000/1000 |
| Quick | Held-out | 1000 | 158.21 | 85 / 123 / 157 / 192 / 386 | 1000/1000 | 0 / 0 | 1000/1000 |
| Standard | Before | 500 | 331.52 | 149 / 230 / 333 / 417 / 548 | 500/500 | 0 / 0 | 500/500 |
| Standard | Tuned development | 500 | 296.71 | 164 / 218 / 299 / 358 / 530 | 500/500 | 0 / 0 | 500/500 |
| Standard | Held-out | 500 | 291.93 | 165 / 213 / 295 / 358 / 480 | 500/500 | 0 / 0 | 500/500 |


Standard's median is 1.92× Quick in development and 1.88× on held-out seeds. Every game finishes by VP; there are no censored safety-cap stops. No claim is made that the upper tail fits the median target.

The slice has 25 technologies. Federated Markets still has no implemented benefit and the scripted AI skips it, so 24 means every currently useful technology. All-seat observations include eliminated empires; winners are separate populations. Counts measure technologies known, not Science spent. The [research appendix](2026-09-27-tech-bicycle-tuning-research.md) includes every faction, all quantiles, completion bands and era-entry denominators, regenerated from the archives by `scripts/summarize-tech-bicycle.py`.

| Speed | Cohort | Population | N | Mean techs | Min / P10 / median / P90 / max | All useful techs |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | Before | All seats | 8000 | 18.65 | 2 / 10 / 20 / 24 / 24 | 2908/8000 (36.4%) |
| Quick | Before | Winners | 1000 | 22.73 | 8 / 18 / 24 / 24 / 24 | 800/1000 (80.0%) |
| Quick | Tuned development | All seats | 8000 | 16.59 | 2 / 9 / 18 / 22 / 24 | 520/8000 (6.5%) |
| Quick | Tuned development | Winners | 1000 | 19.57 | 9 / 17 / 20 / 23 / 24 | 90/1000 (9.0%) |
| Quick | Held-out | All seats | 8000 | 16.78 | 3 / 10 / 18 / 22 / 24 | 553/8000 (6.9%) |
| Quick | Held-out | Winners | 1000 | 19.75 | 8 / 18 / 20 / 24 / 24 | 105/1000 (10.5%) |
| Standard | Before | All seats | 4000 | 16.93 | 2 / 8 / 17 / 24 / 24 | 1040/4000 (26.0%) |
| Standard | Before | Winners | 500 | 21.88 | 7 / 15 / 24 / 24 / 24 | 347/500 (69.4%) |
| Standard | Tuned development | All seats | 4000 | 15.47 | 2 / 8 / 17 / 21 / 24 | 151/4000 (3.8%) |
| Standard | Tuned development | Winners | 500 | 18.99 | 8 / 16 / 19 / 22 / 24 | 26/500 (5.2%) |
| Standard | Held-out | All seats | 4000 | 15.28 | 2 / 8 / 17 / 21 / 24 | 142/4000 (3.5%) |
| Standard | Held-out | Winners | 500 | 18.82 | 7 / 14 / 19 / 22 / 24 | 30/500 (6.0%) |


Winners are usually still broad through Recovery and Consolidation, but their median Integration completions fall from 8 to 4 Quick and 3 Standard in both new cohorts. Their 24-tech completion share falls from 80.0% / 69.4% to 9.0% / 5.2% in development and is 10.5% / 6.0% on held-out seeds. This demonstrates lasting late research choices; it does not establish that every branch is equally attractive or that full-game specialization is accepted. Athens retain a science distinction: held-out all-seat median known techs is 21 Quick / 19 Standard, versus 18 / 17 for the whole field.

The added read-only `research_turns` telemetry records each first `TechResearched` event. It never enters game state, AI decisions, RNG or hashes. The fresh baseline has no timing field, so a before/after era-entry shift cannot be inferred from its final completion counts. In held-out games, all-seat first Consolidation entry is median 76 Quick / 150 Standard among those reaching it; first Integration entry is 130 / 252 and includes Relay. These remain later than the full 32-technology proposal's illustrative 35–50 / 70–90 Quick schedule. That broader era schedule, the absent Inheritance era and human investment windows remain unvalidated.

## Where the victory points come from

Each row sums terminal winning-empires' VP. Since teams have one empire here, these are also winning-team totals. The columns reconcile exactly; populations across rows must not be added. Other milestones are Four Cities, Six Technologies and Twenty Population. Relay's value remains 2VP.

| Speed | Cohort | Winner VP | Live titles | Other milestones | Relay | Faction goals |
| --- | --- | --- | --- | --- | --- | --- |
| Quick | Before | 10179 | 3867 (38.0%) | 3000 (29.5%) | 1334 (13.1%) | 1978 (19.4%) |
| Quick | Tuned development | 10157 | 3544 (34.9%) | 2993 (29.5%) | 1682 (16.6%) | 1938 (19.1%) |
| Quick | Held-out | 10180 | 3510 (34.5%) | 2996 (29.4%) | 1726 (17.0%) | 1948 (19.1%) |
| Standard | Before | 5067 | 2101 (41.5%) | 1500 (29.6%) | 486 (9.6%) | 980 (19.3%) |
| Standard | Tuned development | 5092 | 1901 (37.3%) | 1495 (29.4%) | 744 (14.6%) | 952 (18.7%) |
| Standard | Held-out | 5107 | 1918 (37.6%) | 1497 (29.3%) | 738 (14.5%) | 954 (18.7%) |


In development 84.1% of Quick winners and 74.4% of Standard winners have Relay; held-out proportions are 86.3% / 73.8%. The route is common, but wins without it remain observed. This is not evidence that Relay is optional under every human strategy or that an alternative Great Work path exists. The slice still lacks the proposed tall milestone, other Great Works and diplomacy benefits.

## Truthful forecasts and regression coverage

Rust rules, former TypeScript client rules and displayed tech/building data use the same prices, prerequisite route and faction exceptions. New Rust regressions exercise all eight factions at both speeds for both industrial tiers: cost-breakdown sums, Gold purchase quotes, actual completion and resulting city yields. Relay tests check the non-industrial research route, maximum diffusion, carryover, quoted build time, the full paid project and its unchanged 2VP. Existing population/predecessor gates and faction inheritance remain covered. The mock independently resolves purchase quotes and completion yields for every faction at both speeds.

City tooltips now scale unique-piece Production and upgrade Gold prices with game speed while leaving yields and combat statistics alone. Guide prices and Foundry output derive from data; the guide also names Marksmen and Siege Engines in the already-implemented Production-priority rule. Research telemetry is reconciled independently against replayed events. The reporting script verifies each player's era counts equal its technology total and its VP source breakdown equals its VP.

## Intentional fixtures and checks

The fixture regeneration follows [the fixture README](../../crates/sim/tests/fixtures/README.md). All three initial hashes remain unchanged. Only seed `18446744073709551613` changes: its first changed order/hash is turn 5, where scripted Sparta seat 2 selects Signal Theory instead of Fortification because of the changed downstream Relay route. The other two tapes are byte-identical to their old versions. All contain 40 turns. This explanation accompanies the intentional rules/AI change; the fixtures were not regenerated to conceal unexplained drift.

| Check | Final result |
| --- | --- |
| `cargo fmt --all` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, no warnings |
| `cargo test --workspace` | PASS: 444 passed, 0 failed, 4 explicitly ignored |
| `cargo test --release -p legatum-sim --test determinism` | PASS: 6 passed |
| `scripts/build-wasm.sh` | PASS, web WASM built |
| `wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node` | PASS |
| `npm test` in `app/` | PASS: 436 tests in 45 files |
| `npm run build` in `app/` | PASS, TypeScript and production bundle |
| WASM replay: committed fixtures | PASS: 3 tapes × 40 turns |
| WASM replay: full Quick games, seeds 700001–700006 | PASS: 6 tapes, 112 / 168 / 191 / 148 / 194 / 163 turns |
| WASM replay: full Standard games, seeds 700001–700003 | PASS: 3 tapes, 289 / 296 / 283 turns |

The 12 WASM tapes cover 1,964 turn hashes plus initial and every-turn save/load checks. [Quick full-game report](2026-09-27-tech-bicycle-tuning-tapes-quick.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-tapes-quick.json.gz) and [Standard report](2026-09-27-tech-bicycle-tuning-tapes-standard.md) / [JSON.gz](2026-09-27-tech-bicycle-tuning-tapes-standard.json.gz) record their generation commands. These nine games overlap development seeds and are compatibility tests, not independent balance evidence.

The four ignored Rust tests are the existing `midgame_speed`, `paired_evaluation`, `many_games_summary` and explicit performance-corpus comparison. App tests emit Node's experimental localStorage warning without `--localstorage-file`; they pass. WASM packaging notes optional Cargo package metadata; there is no compile failure.

## Reproduction

Build the final branch's rules, then run the same campaign populations. The generated reports retain the exact executed argv (including the frozen `network-sim` path and status files); these commands use the equivalent final release binary. The experiment ledger supplies prior-trial diffs for the merged baseline tree.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --games 1000 --seed 700001 --speed quick --determinism --threads 8 --label tech-bicycle-tuning-network-quick --out-dir target/bicycle-repro
./target/release/legatum-sim --games 500 --seed 700001 --speed standard --determinism --threads 8 --label tech-bicycle-tuning-network-standard --out-dir target/bicycle-repro
./target/release/legatum-sim --games 1000 --seed 910001 --speed quick --determinism --threads 8 --label tech-bicycle-tuning-heldout-quick --out-dir target/bicycle-repro
./target/release/legatum-sim --games 500 --seed 910001 --speed standard --determinism --threads 8 --label tech-bicycle-tuning-heldout-standard --out-dir target/bicycle-repro
```

The intentionally executed regeneration used `network-sim` with the following README parameters; the final release build has the same rules:

```sh
./target/release/legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed --label determinism-fixtures --tape-out crates/sim/tests/fixtures --out-dir target/bicycle-tuning/fixture-reports
cargo test --release -p legatum-sim --test determinism
export WASM_PACK_CACHE=/tmp/legatum-bicycle-wasm-pack-cache
scripts/build-wasm.sh
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node
node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node
./target/release/legatum-sim --games 6 --seed 700001 --speed quick --determinism --label tech-bicycle-tuning-tapes-quick --tape-out target/bicycle-tuning/tapes-quick --out-dir target/bicycle-tuning/tape-reports
./target/release/legatum-sim --games 3 --seed 700001 --speed standard --determinism --label tech-bicycle-tuning-tapes-standard --tape-out target/bicycle-tuning/tapes-standard --out-dir target/bicycle-tuning/tape-reports
node scripts/wasm-tape-check.mjs target/bicycle-tuning/tapes-quick target/wasm-node
node scripts/wasm-tape-check.mjs target/bicycle-tuning/tapes-standard target/wasm-node
```

These results are a reviewable tuned Bicycle slice, not acceptance of the unimplemented full tree. Balance outside Normal eight-faction free-for-all, complete human games and alternative team layouts was not measured. `docs/learnings.md` was deliberately left unchanged as requested; the non-obvious findings and rejected experiments are recorded here and in the experiment ledger.

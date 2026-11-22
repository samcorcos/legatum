# Proxy v4: alliances and coalition calibration

Measured on master `02941d270c1b3484b99a54fe193cf56aae3b4e3c`, the branch point containing alliances, explicit calls and strict-leader coalitions. Game rules and scripted opponents are unchanged in this branch. V3 remains frozen as `ProxyV3Brain`; v2 is still selectable. `PROXY_VERSION = 4` is the default.

Master advanced during measurement with independent escort changes. Both versions remain pinned to the branch point above; these results do not claim to measure those later changes.

Each cell is a complete 300-game Standard run: seeds 700001–700300, default 48×32 map, eight separate teams, seven opponents at the named difficulty, `--rotate-proxy`, no turn limit, and the unchanged 1,200-turn safety cap. Each seat and faction appears 37 or 38 times; every complete 64-game block covers every seat/faction pair. Win rates include every requested game; medians use wins only.

| Start | Difficulty | V3 wins | V3 median win | V4 wins | V4 median win |
| --- | --- | ---: | ---: | ---: | ---: |
| Peace | Easy | 265/300 (88.3%) | 203 | 273/300 (91.0%) | 203 |
| Peace | Normal | 220/300 (73.3%) | 218 | 222/300 (74.0%) | 211 |
| Peace | Hard | 212/300 (70.7%) | 214 | 213/300 (71.0%) | 214 |
| Legacy War | Easy | 252/300 (84.0%) | 210 | 252/300 (84.0%) | 210 |
| Legacy War | Normal | 199/300 (66.3%) | 220 | 199/300 (66.3%) | 220 |
| Legacy War | Hard | 185/300 (61.7%) | 210 | 185/300 (61.7%) | 210 |

V4 meets the requested Hard target: 213/300 wins and median turn 214. V3 already narrowly met it on this master (212/300, median 214); the older 75% result was measured before the alliance/coalition merge. This comparison does not isolate that merge from every other intervening master change.

All 3,600 acceptance games were replayed and hash-verified by the analysis observer. All 1,800 v3 games and the 300-game v4 Hard Peace run were also played twice with `--determinism`; the other v4 runs used one play plus tape replay. No first plays are counted twice.
Outcomes: `{'vp': 3600}`. There were no panics or determinism failures.

Legacy War disables diplomacy entirely. V3 and v4 both delegate to frozen v2 in that mode; every turn hash matches between the two complete 300-game runs at each difficulty. Full-game selector probes also match original-master hashes for explicit v3 Peace and v2 War. The v3 source differs from the original only in its exported names.

V4 accepts trustworthy alliances that secure a known flank or help a shared war while preserving a conquest route. It proposes an alliance when there is a common war; otherwise it seeks a flank pact. It accounts for actual treaty protection, public army strength, direct enemy allies and local fronts before opening a war. Existing allied combatants contribute only half their public strength to the strategic budget; unsigned offers and unanswered calls contribute none. It never assumes an alliance chain enters war automatically.

Calls require a reachable useful front, affordable combined opposition and safe reserves. One accepted call replaces a new independent declaration, the eight-turn commitment is respected, and its target can guide staging without authorizing premature attacks. The proxy calls its own allies only when a useful war needs more strength. Near 50% victory progress, an exposed empire can replace an extra settler build to assemble three infantry and four ranged units instead of two and three. Upkeep recovery remains in force.

A new Great Work waits behind an existing investment or defender if its two points would create an unaffordable coalition threat. A project already underway, sufficient flank protection, or an immediately winning project proceeds. These are public-information estimates, not guarantees against unseen positioning or future treaty breaches.

Resolved events and public coalition exposure in the Hard Peace runs (300 games each):

| Metric | V3 | V4 |
| --- | ---: | ---: |
| Alliances activated | 0 | 319 |
| Calls answered | 0 | 18 |
| Calls fulfilled | 0 | 17 |
| Allies answering the proxy | 0 | 70 |
| Calls abandoned | 0 | 0 |
| Treaties breached | 0 | 0 |
| Wars declared | 1402 | 1218 |
| Wars received | 4821 | 3830 |
| Turns as coalition target | 17732 | 17882 |
| Targeted turns with a coalition member at war | 13124 | 10306 |

These count engine outcomes, not attempted orders. A game can end with an active call before eight turns pass. Coalition exposure is interpreted alongside wins: losing the lead also reduces exposure. The observer now distinguishes activated alliances from pacts; the original observer labeled both as pacts. V3 exposure was measured by a separate hash-verified replay of all 300 original tapes with the original engine sources.

On matched Hard maps and assignments, 22 v3 losses became v4 wins and 21 v3 wins became losses. The one-game net difference is not evidence of higher underlying strength. This is a fixed-seed calibration, not proof of a population win rate or a reconstruction of the human's conquest. All policies read only `PlayerView`; v4 adds no RNG, private state, observer feedback, difficulty bonus, or calls into the opponent's diplomacy planner.

Other yardsticks remain separate: Hard turn-100 proxy cities average 3.28 (median 3, n=300); the turn-150 technology gap against the strongest original AI averages -1.02 (n=288). Snapshots exclude games ending before that turn. Normal Peace remains below the older guide's 85% target, and its wider expansion and science targets are not established by meeting this task's Hard win-rate/turn target.

The 64-game development screens were used to diagnose decisions, not replace the acceptance samples. V3 won 45/64 (70.3%). The first alliance-aware candidate with broad public-army caution won 40/64; preserving local siege judgment reached 41/64; the larger reserve reached 44/64. Selective outgoing alliances and the call/score-timing changes retained 44/64. Limiting the reserve to current public strength shortfalls fell to 43/64 with median turn 223 and was rejected. The full acceptance runs above decide whether the selected policy meets the target.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
./target/release/legatum-sim --proxy 0 --proxy-version 4 \
  --proxy-start peace --rotate-proxy --difficulty hard \
  --games 300 --seed 700001 --speed standard --threads 8 --determinism \
  --archive --archive-dir /tmp/proxy-v4-reproduction/hard-peace/archive \
  --out-dir /tmp/proxy-v4-reproduction/hard-peace --label proxy-v4-hard-peace
```

Repeat for `--proxy-version 3`, all three difficulties and both starts. To reproduce the recorded repeat counts exactly, omit `--determinism` for v4 Easy/Normal Peace and all v4 War runs. The raw receipts contain every exact command.

Large JSON, tapes, binaries, source manifests, development trials and logs stay outside Git under `/tmp/legatum-proxy-v4`. Each run has `receipt.json`, `source-manifest.json`, `run.log`, `status.json` and an archive. `compact.json` and `outcomes.json` summarize and validate the results. `compatibility.json` records selector checks.

Selected v4 policy SHA-256: `02023d6e427fad1f90c141656f6ee05bc037e695caa492d65f7f92285164a612`.

| Report relative to the artifact directory | SHA-256 |
| --- | --- |
| `baseline-v3-easy-peace/2026-09-28-baseline-v3-easy-peace.json` | `edfa34fbbc1822b9aa11ba091e5b55895c2a1baf93a0e8ed2aa0db972c24c613` |
| `baseline-v3-normal-peace/2026-09-28-baseline-v3-normal-peace.json` | `161a6c00b339363e2efe2c0290eef4cca8109a73770e03e794828b15ff86beb1` |
| `baseline-v3-hard-peace/2026-09-28-baseline-v3-hard-peace.json` | `25e26091607d38f535cd85a6f7c7ea55b793bb31b84a9736239fa03f05bf1666` |
| `baseline-v3-easy-war/2026-09-28-baseline-v3-easy-war.json` | `6117efbeaeafc5e358fa43e89698dce06ad50504a12970ddd7ecea6c7daea51a` |
| `baseline-v3-normal-war/2026-09-28-baseline-v3-normal-war.json` | `ea788e5812e43e2273bbbfe75e26b3a2c39124a6259faa32f4ffe557acfa918b` |
| `baseline-v3-hard-war/2026-09-28-baseline-v3-hard-war.json` | `de018f94ddaefc687ec484d8ac73fc8f01ff116dbfd3ba9fc88e29ddcdf46102` |
| `candidate-5-full-easy-peace/2026-09-28-candidate-5-full-easy-peace.json` | `9d8d3452e7ffba87635b80e9fdeeae6fa8d51cac28166ff14bc20648782e6dd2` |
| `candidate-5-full-normal-peace/2026-09-28-candidate-5-full-normal-peace.json` | `202eb68e0f3a6df13d1f652532e506acf48c3c5cc4af635c02692cc4cbc86b99` |
| `candidate-5-full-hard/2026-09-28-candidate-5-full-hard.json` | `a528b6829521b715905ae156e64bd2843a51bf6b3128961e9c7358dcbbc423f3` |
| `candidate-5-full-easy-war/2026-09-28-candidate-5-full-easy-war.json` | `dbbbd05adc2ca33f7476fa237e232c2675d24dbb25f8ccd04545eedb3cdc5846` |
| `candidate-5-full-normal-war/2026-09-28-candidate-5-full-normal-war.json` | `58a9f1500e7aa4a14c8ef841f52c9dd2309a0d595b92aef4facd9ade212c0d2f` |
| `candidate-5-full-hard-war/2026-09-28-candidate-5-full-hard-war.json` | `1653caf8e98c23e97d5218c043841249890b711fbb019c5848cc10574df0d14d` |

Final checks use `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4` and Rust 1.98.1. They run serially against frozen source.

| Command | Exit code |
| --- | ---: |
| `cargo fmt --all` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace --release --no-fail-fast` | 0 |
| `cargo test --workspace` | 0 |
| `cargo test -p legatum-sim --test determinism` | 0 |
| `git diff --check` | 0 |
| `python3 /tmp/legatum-proxy-v4/conflicts.py` | 0 |

- release: 898 passed, 0 failed, 4 ignored.
- debug: 898 passed, 0 failed, 4 ignored.
- determinism: 12 passed, 0 failed, 0 ignored.

An earlier release check exited 101 with E0460 during server doctests because an overlapping candidate build replaced an unhashed WASM library. The serial frozen-source rerun above resolves that artifact collision; no test was disabled. Earlier check receipts are retained separately. `CLAUDE.md` and `docs/learnings.md` are unchanged.

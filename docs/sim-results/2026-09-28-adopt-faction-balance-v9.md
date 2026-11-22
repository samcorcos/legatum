# V9 faction balance becomes the default

Ported the approved rules and AI policies from `astra/faction-balance-rerun` (`b30e3a6`) onto current master `3937b1d`. All seven changes are unconditional; no faction Cargo/Vite flags or experimental save markers were imported. Athens are unchanged. Existing master diplomacy/Rush ordering is preserved. Historical saves load under the current rules rather than emulating old rules.

Fresh 2,000-game samples measure **1.350× Quick / 1.567× Standard**, with no cap stops or panics. **Standard exceeds the source report's 1.5× observed balance gate.** The approved bundle is adopted without retuning against these held-out results.

| Faction | Adopted change (Quick base values) |
| --- | --- |
| Japan | Existing distant-city penalty applies beyond 6 hexes from the owned capital. |
| Germany | All units, including Settlers, cost +30 Production; first Settler 105. |
| Byzantium | Ordinary buildings cost 120%; exploration goal awards 3 VP. |
| Mongols | Active Market-family cities at least 8 hexes from the owned capital gain +2 Gold; planning continues an unfinished spaced network after a close third city. |
| Sparta | Oathbearer 28 melee; Trooper/Assault Trooper +80 Production; healthy Soldier home guards qualify; veteran expedition in actual War awards 4 VP. Military recovery is 2 HP at home, 15 at an active owned Aid Station, zero in the field. Expedition planning preserves guards and legal diplomacy. |
| Switzerland | +1 Gold upkeep per complete pair of paid military units, after two free units; AI budgets the actual marginal upkeep and Logistics rounding. |
| Celts | Grassland 1 Food; Soldier/Trooper/Assault Trooper −2 melee. |

The shared faction descriptions update the picker, guide and tooltips. Speed-scaled costs, goal awards, Aid Station and Market-family descriptions, and terrain inspection agree with the rules. The former TypeScript client has native/WASM parity coverage for costs, yields, upkeep, healing, goals and settlement planning. Simulator VP accounting uses each faction's actual goal award.

## Fresh default-build measurement

Seeds **3150001–3152000** were reserved before launch: 2,000 first plays at each speed, Normal scripted AI, default 48×32 map, one of each faction shuffled among eight solo teams, Peace starts, no gameplay turn limit. Every game is played twice for per-turn determinism; repeat plays and selected diagnostics are not extra observations. Safety caps are 600 Quick / 1,200 Standard; cap stops count as non-wins. Every initial seat, including eliminated empires, stays in the denominator. Gameplay and AI source remained unchanged after the executable was frozen.

| Speed | Best / worst wins | Ratio | Cap stops | Mean turns | Median | p95 | p99 | Maximum |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Quick | Celts 293 / Sparta 217 | 1.350× | 0 / 2,000 | 140.39 | 137 | 176 | 216 | 375 |
| Standard | Celts 315 / Germany 201 | 1.567× | 0 / 2,000 | 274.53 | 270 | 341 | 397 | 808 |

| Faction | Quick wins / 2,000 | Standard wins / 2,000 |
| --- | --- | --- |
| Japan | 232 (11.60%) | 287 (14.35%) |
| Germany | 277 (13.85%) | 201 (10.05%) |
| Mongols | 219 (10.95%) | 276 (13.80%) |
| Sparta | 217 (10.85%) | 258 (12.90%) |
| Byzantium | 258 (12.90%) | 211 (10.55%) |
| Athens | 251 (12.55%) | 235 (11.75%) |
| Switzerland | 253 (12.65%) | 217 (10.85%) |
| Celts | 293 (14.65%) | 315 (15.75%) |

Both cohorts have 2,000 VP wins, zero panics, and 2,000/2,000 matching deterministic second plays; both processes exit 0. Three fresh Standard games exceed 600 turns; the longest is seed 3151316 (808 turns, Byzantium victory). These are descriptive sample estimates for Normal FFA, not a population bound or evidence for human/permanent-team balance. The source branch's historical 1.28–1.365× V9 and 5.2× Quick / 2.8–2.9× Standard baseline measurements used different seeds and an earlier master. This measurement alone cannot separate seed variation from intervening master changes.

Reproduction from this commit (no feature arguments):

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo build --release -p legatum-sim
artifact_dir=/tmp/legatum-sim-artifacts/adopt-balance-v9
mkdir -p "$artifact_dir"
cp target/release/legatum-sim "$artifact_dir/legatum-sim"
for speed in quick standard; do
  "$artifact_dir/legatum-sim" --games 2000 --speed "$speed" --seed 3150001 --threads 4 --determinism --label "adopt-v9-heldout-$speed" --out-dir "$artifact_dir" --status-file "$artifact_dir/$speed-status.json"
done
```

The measured runs used the same arguments concurrently, with stdout/stderr in `quick.log` and `standard.log`. Frozen executable SHA-256: `8566bff2b02423c2db8a266932279748620b9dd858eceab7ef4b10e432b25939`.

## Title-lock diagnosis and proposed follow-up

Historical Standard seed **2051748** reproduces its **1,028 turns** with the old frozen V9 executable, matching all original turn hashes. The default executable produces the same 1,028 order maps, final scores, and every native replay frame (territory, cities, units, titles, VP and notable events). This diagnostic is outside the fresh cohort.

Celts holds Economy, Science and Population continuously on turns **348–974: 627 turns**, without its faction goal. At turns 400, 600, 900 and 1,028, only one of its five cities owns the required two hills within radius two. Mongols also lacks a spaced three-city network. At turn 600 Celts has 5 permanent + 3 live VP = 8; Japan has 7 + Territory + Army = 9; Sparta has 9 permanent VP without a title; Byzantium has 8 permanent VP without a title. Army transfers between Japan and Celts do not end that allocation. Switzerland captures an Sparta city and takes Population on turn 1021, then takes Science and wins on turn 1028.

Economy (gross city Gold + Production), Science (recent generation), and Population are correlated development metrics. Titles persist while their holder leads, including incumbent ties, with no expiry. An empire missing its geographic goal can deny three titles yet remain short of victory. V9's larger Byzantium/Sparta goals allow other possible finishes: this is a demonstrated AI/pacing trap, not proof that victory is mathematically impossible.

Proposed separate AI experiment: carry a feasible paid faction-goal plan through settlement and conquest. Celts's shared settlement bonus currently counts foreign hills, while `celts_hill_site` correctly excludes them; settler VP scoring only credits the goal when it finds enough distinct remaining sites. Count owned/claimable hills, value useful partial progress toward three qualifying cities, and plan legal captures that change the blocking title metrics while preserving visible-information and defense constraints. Validate selected stalls and fresh cohorts, including the long tail and faction balance. **This proposal is not implemented, and title rules are unchanged.** Unlimited games still have no general termination guarantee; title expiry, additional scoring or a mandatory end condition requires the lead's decision.

## Verification and artifact pointer

All nine determinism fixtures were intentionally regenerated using the documented commands in [the fixture README](../../crates/sim/tests/fixtures/README.md). Initial hashes are unchanged. The three mixed-AI tapes change 40/35/40 turn hashes and 23/10/6 order maps respectively under the adopted rules; all six manual scenarios preserve their recorded hashes and orders. Generators write tape version 4. Native and WASM replay cover **9 tapes / 443 turns**, including save/load at every boundary.

| Check | Exit | Result |
| --- | --- | --- |
| `cargo fmt --all` | 0 | Clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | Pass |
| `cargo check --workspace` after the port | 0 | Pass |
| `cargo test --workspace --release --no-fail-fast` | 0 | 948 passed, 4 ignored, 0 failed |
| `CARGO_TARGET_DIR=target/verification CARGO_BUILD_JOBS=4 cargo test --workspace` | 0 | 948 passed, 4 ignored, 0 failed, including doctests |
| `cargo test -p legatum-sim --test determinism` | 0 | 12 passed |
| `scripts/build-wasm.sh` | 0 | Pass |
| Node `wasm-pack` build + `scripts/wasm-tape-check.mjs` | 0 | 9 tapes / 443 turns |
| `cd app && npx vitest run --maxWorkers=6` | 0 | 90 files; 1,165 passed, 1 skipped, 0 failed |
| `cd app && npm run build` | 0 | Pass |
| `git diff --check` + `git diff --cached --check` | 0 | No whitespace errors or conflict markers |

An earlier shared-target debug run passed its tests but exited **1** during server doctest linking (`E0460`: concurrent Cargo checks replaced the un-hashed WASM rlib). The isolated full rerun above passes, including doctests; the earlier invocation is not counted as a pass. WASM builds used `WASM_PACK_CACHE=/tmp/legatum-wasm-pack-cache`. The existing settler-detour regression retains its archived initial state and waypoint evidence, and clears the new earlier win only inside the test so its route checks still run.

Large reports/JSON, gzip copies, per-game hashes, executable, frozen source patch, diagnostic tapes/replays and logs are external at **`/tmp/legatum-sim-artifacts/adopt-balance-v9/`**. `summary.json` contains independent win/seat recounts and raw/gzip SHA-256 values; `provenance.json` and `freeze-verification.json` identify the measured source; `checks.json` holds commands/results; `long-game-verification.json` records the historical/default comparison. The old branch's approximately 96k-line artifact history was not imported. Its campaign remains at `/tmp/legatum-sim-artifacts/faction-balance-rerun/` and recoverable from that branch.

The external `artifact-manifest.json` lists 68 files (280,322,179 bytes). Its SHA-256 is `394787686d4b8f09114b37827bf294405d566f3ce4f6fb1e0712a497769847ca`. Raw cohort SHA-256 values: Quick `faaeca353d809db3a4266c6db09aac5d68d14c1e805ce34b44fc78db39e855bc`; Standard `71241e72e61188f92bccc0919ac5ba3164ef1edb74194988d52bcdc69d397c9b`.

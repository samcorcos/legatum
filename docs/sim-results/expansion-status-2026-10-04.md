# Expansion AI validation — 2026-10-04

## Decision and methods

Merged `claude/expansion` (`f0935036`). Normal and Hard AI now value later colonies by land, cost, and time left instead of stopping at a fixed city quota. I kept the original `ca4eb54a` valuation and reverted four later v4 additions: six-site tally cap, population-three unassisted export, scaled Hard Settler pipeline, and half-value settling from threatened defended cities. A paired 100-game Standard ablation measured 4.11 T200 cities for v4 and 4.12 for the rollback; best/worst faction win ratios were 10.67 and 4.67. The win ratios are noisy, but v4 supplied no expansion gain. This branch changes AI decisions, not game rules.

The 400-game validation compares master `47dc9397` with expansion `1317c0a9`, both after Byzantium salvage integration. Each speed uses eight scripted Hard players, paired games from seed `5200001`, the same faction rotation, and six or fewer simulator threads. Master runs came only through `tools/sim/cached-baseline.sh --commit 47dc9397 -- ...`; every cohort used `tools/sim/slot.sh`. Raw reports, tapes, checked traces, and scripts are under `~/LegatumRuns/expansion/`.

## City counts and faction balance

| Speed | Source | T50 | T100 | T150 | T200 | Eligible seats at T50/100/150/200 |
|---|---|---:|---:|---:|---:|---|
| Standard | master | 1.87 | 2.78 | 3.29 | 3.71 | 3200 / 3193 / 3054 / 2393 |
| Standard | expansion | 1.89 | 2.88 | 3.56 | 4.10 | 3200 / 3197 / 2984 / 2292 |
| Quick | master | 2.16 | 3.23 | 3.80 | 4.34 | 3200 / 2936 / 414 / 88 |
| Quick | expansion | 2.25 | 3.47 | 4.13 | 4.50 | 3200 / 2793 / 565 / 80 |

Standard gains **0.39 cities per player at T200** (+10.5%); Quick gains **0.33 at T150** (+8.7%). Quick T200 has only 88 master and 80 expansion seats, so that 0.16 difference is unstable. Counts are immediately before the marked turn in games still running; eliminated seats in an eligible game count as zero. All 400 tapes per speed and source replayed with matching-source binaries and initial/per-turn hash checks. Traces from a mismatched binary were excluded.

| Faction | Standard master | Standard expansion | Quick master | Quick expansion |
|---|---:|---:|---:|---:|
| Japan | 9.8% | 8.8% | 10.5% | 12.2% |
| Germany | 14.5% | 12.0% | 12.0% | 14.2% |
| Mongols | 26.0% | 28.5% | 17.8% | 16.2% |
| Sparta | 6.0% | 9.2% | 8.2% | 6.5% |
| Byzantium | 14.8% | 15.5% | 21.2% | 21.5% |
| Athens | 8.8% | 8.2% | 9.8% | 9.8% |
| Switzerland | 7.2% | 5.2% | 8.2% | 6.5% |
| Celts | 13.0% | 12.5% | 12.2% | 13.0% |
| **Best/worst ratio** | **4.33** | **5.43** | **2.58** | **3.31** |

Each faction has 400 seats per speed. All games finished by VP with zero panics. Mean finish moved 226.4→229.8 turns at Standard and 128.8→129.6 at Quick. The ratio increases are balance risks, especially Standard Mongols (26.0→28.5%) and Switzerland (7.2→5.2%). At 400 seats, each faction rate still has several percentage points of sampling uncertainty.

## Human-playbook proxy v6

A separate 400-game Standard cohort from seed `3610001` rotated the proxy seat against legacy Hard AI (`--proxy 0 --proxy-version 6 --proxy-start peace --rotate-proxy --legacy-hard all --difficulty hard --barbarians true --speed standard`). Master used the same cached-baseline wrapper and pinned source. All games finished by VP with zero panics.

| Faction | Master win rate | Expansion win rate |
|---|---:|---:|
| Japan | 9.8% | 8.7% |
| Germany | 12.8% | 11.7% |
| Mongols | 28.2% | 31.8% |
| Sparta | 11.0% | 10.2% |
| Byzantium | 12.8% | 11.3% |
| Athens | 8.5% | 7.7% |
| Switzerland | 6.2% | 8.0% |
| Celts | 10.7% | 10.5% |
| **Best/worst ratio** | **4.52** | **4.10** |

Mean finish moved 221.9→225.1 turns. The ratio improves in v6 despite Mongols rising 3.6 points. This opponent mix differs from the all-scripted Hard cohorts, so the ratios should be read separately.

## Diagnosis and RULES PROPOSALS — for Sam, not implemented

The original 500-game diagnosis before later master integrations found Standard T200 cities 3.79→4.19 and Quick 4.29→4.56 with value-based colonies. The 48×32 map had about 650 land tiles, or 81 per eight-player seat. Good free sites within 10 hexes fell from 10.6 at T25 to 2.7 at T200. Median city population was 2/3/4 at T50/100/200, and Settler cost divided by best-city Production stayed near 24 turns through T200. Removing the AI quota exposes land, cost, and growth limits. The following pilots used earlier source snapshots, so their effects are directional and should be remeasured after newer master rules settle.

1. **Map size per player.** Test a target near 120–127 land tiles per seat, scaling dimensions by player count. The paired 200-game 60×40 pilot had about 1,016 land tiles (127 per seat) and raised Standard T200 cities 4.17→5.46; mean finish changed 244.5→225.4 turns and win ratio 6.18→4.64. Combining that map with a 0.67× all-price Settler discount reached 6.15 cities. More land may reduce early border pressure and lengthen travel, and larger maps cost more to generate, pathfind, and render. Check rush timing too.
2. **Settler cost step.** Test lowering the trained-Settler increment from +20 to +10 while leaving the first Settler's base price alone. In a paired 100-game Standard pilot, T200 cities rose 4.12→4.27 and mean finish fell 233.3→229.4 turns; win ratio moved 4.67→6.20, a noisy balance warning. At base speed, +10 saves 10 Production on the second trained Settler and 20 on the third. A separate 200-game 0.67× discount on **all** Settler prices raised T200 cities 4.17→4.48 but also cheapened the first Settler. Cheaper later colonies may dominate and weaken the military counter; check first-war timing and faction wins.
3. **Growth curve.** Defer a broad change. Testing `GROWTH_PER_POP` 8→6 with base 15 and the pop-two Settler requirement preserved changed the pop-three Food threshold 31→27 (62→54 at full Standard scaling) and pop four 39→33 (78→66). A paired 100-game pilot barely changed T200 cities (4.12→4.11) or mean finish (233.3→229.5); win ratio worsened 4.67→13.00, driven partly by only two Swiss wins and therefore highly noisy. Faster growth boosts all yields and faction goals, not just Settlers. Seek a targeted opening-growth mechanism and measure its expansion gain first.

Production banking, a cheaper first Settler, and faster first growth would improve the opening and could reduce the case for a blanket Settler discount; they would not create more legal land. Removing Influence would replace border expansion with a timer and allow Gold tile purchases. That may change colony-site availability and compete for Gold otherwise spent on Settlers or defense. Re-run the map and cost pilots after these rule changes instead of adding their hypothetical gains to the measured effects.

## Integration and verification

The branch later merged master through `faecae92`, including terrain sight, Byzantium, Sparta, and other AI/rules work. No 400-game balance cohort was rerun on that later integrated tree. The three mixed 40-turn fixtures regenerated to the same decoded setups, orders, and hashes as master, so their bytes were retained. The 80-turn mixed runner pin stayed unchanged; the 160-turn scripted Hard pin was re-recorded as `7496051003047737313`. The project rules reference was merged with master; this branch introduces no game-rule constant changes. A debug regression at an extreme explicit turn limit revealed `i32` overflow in colony payback. The calculation now uses `i64` intermediates, with the same values at normal limits.

The five supplied analysis scripts were copied to `~/LegatumRuns/expansion/scripts/` and their dead `/private/tmp` path removed. `expansion_trace` now supports compact marked-turn output and rejects initial or per-turn hash mismatches. Logs, tapes, and pilots are under `~/LegatumRuns/expansion/`; mismatched-binary traces are preserved separately under `invalid-traces/` and excluded.

Final checks on the integrated tree:

- `cargo fmt --all`: pass; `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace --release --no-fail-fast`: exit 0. `cargo test --workspace`: exit 0 on a serial rerun. An earlier run passed unit/integration tests but hit rustdoc E0460 from mixed `legatum-wasm` build artifacts during concurrent Cargo work; rebuilding and rerunning the server doctest resolved it. `cargo test -p legatum-sim --test determinism`: 14 passed, exit 0.
- `scripts/build-wasm.sh`: the optimizer hit the sandbox's `Operation not permitted`; `LEGATUM_WASM_NO_OPT=1 scripts/build-wasm.sh` succeeded. The Node WASM tape check passed all 11 fixtures against native hashes, including save/load boundaries.
- After the web WASM rebuild, `cd app && npx tsc --noEmit` passed and Vitest passed 124 files, 1,608 tests, one skipped. Vitest used `--configLoader runner` with `--maxWorkers=4` because the default loader writes into a node_modules symlink outside the writable sandbox.
- `git diff --check`: pass; no conflict markers found.

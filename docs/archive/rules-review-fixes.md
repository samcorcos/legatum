> Archived 2026-11-22: kept for history; it no longer describes the project.

# Rules review fixes

All 11 findings from `.reviews/astra-review-rules.md` were covered by regression tests written and run before their corresponding fixes. The core suite grew from 130 to 148 tests. Existing tests that encoded civilian destruction, live fog information, or nontransferable earned production were corrected.

| Issue | Observed failure before the fix | Result after the fix |
| --- | --- | --- |
| 1 | A blocked reserve caused two battles instead of one. | Destination meetings require an empty round-snapshot tile; swaps and three-way empty-tile meetings still pass. |
| 2 | Switching a purchased settler to a Relay transferred 80 production; switching after a blocked spawn retained 81 instead of 11 earned production. | A purchase records its item, quoted cost, and purchased portion. Switching forfeits that portion while retaining earned production. |
| 3 | A city founded outside sight appeared immediately; captured cities changed live and razed cities disappeared in fog. | Per-player tile-indexed memory retains the last observed owner, ruin and public city details, including HP. Only visible tiles update it, and teammates share observations. |
| 4 | A civilian in a captured city was removed. | City capture preserves civilians for movement's capture pass, which transfers them with zero moves. |
| 5 | A Scout continued its three-step goto after taking a settler. | Capturing a settler ends the captor's movement and leaves its remaining goto for a later turn. |
| 6 | Only one of two purchased settlers completed; a blocked purchase also lost its original price on the next turn. | Both complete at their paid quotes, even across blocked turns, with only earned overflow remaining. |
| 7 | Completing a third military unit left one gold instead of zero. | Upkeep includes new units; city gold and science still use the yield snapshot from before growth and production. |
| 8 | Capturing a Relay awarded its completion milestone; losing it before scoring removed the producer's claim. | Production completion permanently records credit for the producing empire. Capture grants no completion credit. |
| 9 | The new mode regression, initially routed through the old default, returned ranged combat for a melee order. | Exported `forecast_attack_mode` supports `AttackMode::{Melee, Ranged}`. Adjacent Ranger melee predicts strength 8, damage 18 and retaliation 47; the documented default retains ranged-first behavior. |
| 10 | An idle city with 30 stored production showed 40 turns for a Soldier instead of 10. | Every option includes transferable progress, excludes purchases of other items, and uses the purchased item's fixed quote. |
| 11 | Fully funded research with no science income showed no ETA. | Funded research shows one turn, matching resolution; resistance still blocks production estimates. |

Additional coverage includes save/load of remembered city information, visibility refresh after capture or razing, the original one-buy-per-city-per-turn guard, invalid ranged forecasts, and a melee forecast checked against actual movement combat. Production ETAs remain estimates at current yields, assuming population and spawn space are available.

The new state fields have serde defaults. Older saves cannot reconstruct observations they never stored: mutable information on fogged tiles remains unknown until those tiles are seen again. State hashes and old replay expectations change with the new rules and state fields.

Verification on `astra/fix-rules-review`:

- `cargo fmt --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed, no warnings.
- `cargo test --workspace`: passed; core 148, AI 27, simulator 14, WASM 11 (200 total), zero failures or ignored tests. All doc-test targets passed (zero doc tests).
- `git diff --check`: passed.
- `cargo run --release -p legatum-sim -- --games 100 --seed 1 --determinism --label post-fix`: exit 0. Seeds 1–100 each played twice with identical per-turn hashes; save/load retained the final hash. Zero panics, 88 VP victories, 10 turn-limit victories, 2 draws, average 123.9 turns, 4.7 seconds simulation wall time.

The full simulation results are in [the Markdown report](../sim-results/2026-09-26-post-fix.md) and the per-game JSON (`sim-results/2026-09-26-post-fix.json`; raw run output, not in the public repository).

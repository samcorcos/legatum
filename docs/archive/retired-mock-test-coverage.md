> Archived 2026-11-22: kept for history; it no longer describes the project.

# TypeScript mock retirement: test coverage audit

The retirement removes 15 mock-only test files from `app/src/mock` (100 direct `it` declarations, some parameterized). Those cases exercised the deleted TypeScript implementation. The live engine is the Rust core compiled to WASM. This inventory records where the behavior is checked after retirement.

| Removed suite | Behavior it exercised | Current real-engine coverage |
| --- | --- | --- |
| `ai.test.ts` | City fire, wounded retreats, target selection through fog | `crates/ai/tests/ranged_garrison.rs`, `defense_retreat.rs`, `threat_response.rs`, and `app/src/test/cityStrike.wasm.test.ts` |
| `autoFocus.test.ts` | Automatic city focus while building and after growth | `crates/core/tests/auto_focus.rs` and `app/src/test/retiredMockCoverage.wasm.test.ts` |
| `bicycle.test.ts` | AI first Settler and population two gate | `crates/ai/tests/settler_threats.rs`, `crates/core/tests/factions.rs`, and `app/src/test/banking.wasm.test.ts` |
| `cityForecasts.test.ts` | Growth, production, research, resistance and concurrent Settler forecasts | `app/src/test/purchases.wasm.test.ts`, `cityProgress.wasm.test.ts`, and `eras.wasm.test.ts` |
| `diplomacyAi.test.ts` | Alliance affordability, shared threats, contact and betrayal | `crates/ai/tests/diplomacy.rs`, `offer_pacing.rs`, and `app/src/test/diplomacy.wasm.test.ts` |
| `eras.test.ts` | Research gate, diffusion, era entry, tech effects and older saves | `app/src/test/eras.wasm.test.ts`, `uniquePieces.wasm.test.ts`, and `crates/core` era and save migration tests |
| `hall.test.ts` | Hall uniqueness, held VP, capture and peaceful passage | `app/src/test/hall.wasm.test.ts` and `crates/core/tests/mongols_passage.rs` |
| `mockClient.test.ts` | Setup, map generation, views, founding, validation, movement, forecasts, determinism, save/load and end of game | `app/src/test/actions.wasm.test.ts`, `movement.wasm.test.ts`, `purchases.wasm.test.ts`, `retiredMockCoverage.wasm.test.ts`, and the Rust core integration tests |
| `offerPacing.test.ts` | Acquaintance timer and staggered AI alliance offers | `crates/ai/tests/offer_pacing.rs` and `app/src/test/diplomacy.wasm.test.ts` |
| `resources.test.ts` | Seeded placement and strategic supply | `app/src/test/retiredMockCoverage.wasm.test.ts` pins WASM placement; `resources.wasm.test.ts` checks supply, blockers and shortages |
| `territory.test.ts` | Worked tiles, ring claims and released tiles | `app/src/test/actions.wasm.test.ts`, `movement.wasm.test.ts`, and `crates/core/tests/factions.rs` |
| `validate.test.ts` | Purchases, order permutations, supply and occupied spawn sites | `app/src/test/purchases.wasm.test.ts`, `resources.wasm.test.ts`, and core order validation tests |
| `visibility.test.ts` | Borders, Celtic Marksmen and attackers seen by their victims | `app/src/test/visibility.wasm.test.ts`, `fogBorders.wasm.test.ts`, and `crates/core/tests/visibility_equivalence.rs` |
| `vpHold.test.ts` | Hold start, reset, completion, ties, elimination, and save/load | `app/src/test/vpHold.wasm.test.ts` |
| `wallsStrike.test.ts` | Walls gate, shooter list, and city strike forecast | `app/src/test/cityStrike.wasm.test.ts` |

The two gaps in the WASM-facing suite were a pinned seeded map placement and the city focus transition at the Settler population gate. `retiredMockCoverage.wasm.test.ts` adds those checks. The AI decision cases remain in `crates/ai/tests`, where they exercise the production planner; their old TypeScript planner no longer ships.

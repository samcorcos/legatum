# Retiring Mercenary Companies and thin levies (2026-11-26)

Both Byzantium rules were removed from the engine, the AI and the app, to make room for Buy a city.

- Saves and tapes: `Unit` has no `deny_unknown_fields`, so a saved `contract_until` and `origin` are simply ignored (a Company becomes an ordinary unit of its owner). A saved `HireCompany` order is dropped on tape and archive load (`wasm/src/tape.rs`, `wasm/src/history.rs`, with tests). Saved rule-off lists that name `byzantium_companies` or `byzantium_levy` are dropped by `FactionRuleSet::deserialize`.
- `FactionRule` bits 14 and 15 stay vacant, so `ByzantiumTagmatic` now carries an explicit `= 16`. Check that any new variant keeps the later bits (`ByzantiumGoal = 18`).
- `GameState::made_by` and the `made_by` parameters in `combat.rs` stay: they now always mean the owner's faction. Only Companies ever set `Unit::origin`.
- `rules.generated.json` loses its `company` block: refresh with `UPDATE_UI_RULES=1 cargo test -p legatum-ai --test ui_rules`.
- Not touched on purpose: old telemetry readers for retired events (`tools/ai-quality/gold_usage_1026.py`, `tools/games/analyse_games.py`) and `tools/decision-audit/legacy_adapter.py`, which patches old commits; its `HireCompany` entry no longer matches the current example source.

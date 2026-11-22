# Playtest save fixtures

Small saves taken from playtest games and regenerated (`*.current.legatum`) at the current core rules version, so they load under `legatum_core::RULES_VERSION`. A save from an older rules version is refused with an explicit error rather than migrated, so only current copies are kept.

- `p892-alliance-loop-t20.current.legatum` (the alliance-offer loop; `crates/wasm/tests/p892_alliance_loop.rs`)
- `p892-blue-rects-t117.current.legatum` (Switzerland, turn 117; the map-chunk-cache Playwright spec and `current_browser_fixtures.rs`)
- `p892-sea-rect-t141.current.legatum` (Switzerland, turn 141; the same)
- `p893-unseen-attacker-2119.current.legatum` (border-visibility regression; `unseen_attacker.rs`)
- `p893-zoom-scout-t34.current.legatum` (a Mongol Scout on a coast, turn 34; the wheel-zoom spec)
- `p894-rects-t36.current.legatum` (Mongols, turn 37; water drawn on land, a Mac WebKit canvas race; `current_browser_fixtures.rs` and `tools/ui/webkit-mac`)

The fixtures keep their serialized game and archive data, so each regression exercises the same game JSON the app passes to WASM.

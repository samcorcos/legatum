# Settler hazard investigation

Game: `native-1790550637054682000-56150-9`. Review incidents: I000030, I000033,
I000162, I000443, I000693 (`movement_oscillation`). Investigation base: `530507c`.

## Reproduction

The archive's rules fingerprint is
`962cb5a56b02cc66dfbee38a1d028a5be8462f6da1af6b195dbfde4515a7f86e`.
Current base fingerprint was
`6eafcdb89480ce9f740342bc716e14e636ebf757e35e4f4004145c700620221f`;
capital Science and other intervening changes prevent an exact current-engine replay.

An isolated source export of original engine `db4a21f05b3d327d30bd1e6d47f75282e236bb12`
matched the archive fingerprint, initial hash, and all 343 resolved turn hashes.
Recreating a Normal `ScriptedBrain` for each player each turn, using the session's
`Rng::derive(game_seed, &[purpose::AI, player_id]).next_u64()` seed, matched all
2,744 archived player-turn order lists, including T89–120 and T188–191.

The current policy was then run against those verified states. Recent tactical fixes
change the retreat **destination**, but not the resulting position or loop:

| Settler | Approach | Current retreat | Original retreat goto |
| --- | --- | --- | --- |
| 48 | `(27,10)` → `(25,12)`, revealing Soldier 30 at `(24,13)` | `(25,12)` → `(27,10)` | `(28,9)` |
| 105 | `(13,24)` → `(14,23)`, revealing Soldier 12 at `(14,21)` | `(14,23)` → `(13,24)` | `(11,25)` |

Continuing from T89 and T188 with the current engine, fresh brains, and save/load
after every turn repeated both loops. The selected settler received newly generated
orders; all other orders remained archived. This is a policy counterfactual,
not a claim that current rules reproduce the original whole game's hashes.

Pending/running tasks were checked before edits and again before integration.
`ai-fix-90a74a9ba80af2f5` owns emergency movement priority; `movement-contention-2`
owns engine movement resolution. No direct edits to either movement resolver were
made here. Proxy benchmark work and the separate emergency-retreat fix landed
during this investigation and were incorporated through master `eb6c2fe` before
final validation. The T243 retreat regression passes alongside both settler windows.

## Change and boundaries

- Keep at most 128 recent military sightings per player, expiring 12 turns after
  the last observation. Humans and AI use the same shared team vision. Each record
  identifies the observed unit, owner, kind, location and turn.
- Store observations before orders as well as during ordinary exploration updates,
  including when resuming an older save. Never seed memory from unseen live units.
- A visible new location replaces an old sighting. Seeing the old tile empty removes
  it. Hidden movement, upgrades and death cannot refresh or clear a sighting.
- Keep sightings separate from live units and combat forecasts. Only settler route
  safety consumes the memory; military targeting and danger-strength calculations
  continue to use current visible units.
- Reject routes through remembered danger, choose a reachable alternative site,
  or hold if no safe route exists. Use adjacent waypoints near danger because an
  ordinary distant goto does not carry the planner's safe route. No persistent site
  blacklist is needed: the rejected approach is excluded by its observed hazard,
  while safe approaches to the same destination remain available.

These are observation and decision changes. Combat, movement, yields, sight,
bonuses, player orders and UI flow are unchanged (pillars 5, 6 and 9). Observation
retention and threat avoidance are conservative policy limits, not new combat math.
Added saved observations change canonical state hashes and the rules fingerprint;
old archives still require their original engine. Older saves default to no memory.

## Regression evidence

`crates/ai/tests/fixtures/settler-hazards.json.gz` contains the exact original T89
and T188 states, their source hashes, and archived orders through T120/T191. It was
extracted from the hash-verified original replay. It is deliberately a test fixture;
only a fogged `PlayerView` reaches the brain. Its provenance fields name the game,
original engine and rules fingerprint. Gzip uses timestamp zero.

`cargo test -p legatum-ai --test settler_threats` exercises both archive windows,
an alternate founding site, a safe detour to the only viable site, and reconsideration
after witnessed removal. Brains are recreated; generated orders and resolved states
are compared across save/load. The alternate-site regression failed before the AI
change on step 2, when the settler returned to the hazardous approach.

With the fix, archive Settler 48 retreats once, follows `(26,10)`, `(26,9)`, `(26,8)`
and the safe northern route, then founds at `(22,7)` on T99. Settler 105 leaves the
original approach during T188–191; this short window does not establish its eventual
founding outcome. Synthetic alternate-site and detour cases both found successfully.

`crates/core/tests/threat_memory.rs` covers shared human/team observations, fogged
movement/death, new sightings, expiry, deterministic capacity, legacy saves and
serialization. The historical visibility-equivalence oracle still compares all of
its original tile/city fields; the new observation field is covered separately.

No full-game win-rate or faction-balance claim is made. Near hazards, safe waypoint
orders can spend less than a settler's full movement budget. Expiry permits cautious
reconsideration rather than permanent avoidance of a unit that may have left.

## Final validation

Validated the merged source at `8b02920` (master through `eb6c2fe`); the final
handoff commit changes only this report.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | Passed; no formatting changes |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0 |
| `cargo test --workspace` | 591 passed, 0 failed, 4 existing opt-in tests ignored; exit 0 |
| `cargo test --release -p legatum-sim --test determinism` | 7 passed, 0 failed |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | All 4 tapes / 240 turns passed, including save/load |

WASM was rebuilt from the merged source with
`cargo build --release --target wasm32-unknown-unknown -p legatum-wasm`.
`wasm-pack` initially could not write its helper cache under the workspace sandbox;
the already installed `wasm-bindgen` 0.2.129 CLI generated the Node bindings from
the same compiled module. The checks above used those bindings (without wasm-opt).

The first workspace run embedded the old fixture hashes and failed four
determinism assertions. After the serialization-only hash refresh, the full
workspace suite passed both before and after the final master integration.
No direct changes were made to `CLAUDE.md` or `docs/learnings.md`.

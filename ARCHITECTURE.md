# Architecture

Legatum is a deterministic Rust simulation with thin hosts around it. The rules live in one crate, `crates/core`. Everything else (AI, the WASM API with the online host, the batch runner, the UI) drives that crate and never reimplements it. This document explains how the pieces fit. For build and run commands see [README.md](README.md).

## Crate graph

Dependencies are taken from each crate's `Cargo.toml`. An arrow means "depends on".

```
  hosts            app (WASM in the browser)       crates/sim   crates/analysis
                            |                           |              |
                            v                           v              v
  shared loop       crates/wasm  (Session, saves, tapes, position loading)  <- crates/archive
                         |
                         v
  brains            crates/ai    (Brain trait: PlayerView -> Orders)
                         |
                         v
  rules             crates/core  (resolve_turn, views, data; no I/O)
```

Every host also links `crates/core` directly for types. `crates/archive` and `crates/analysis` sit beside the session layer and read its recording format.

Direct dependencies, from the manifests:

| Crate | Depends on |
| --- | --- |
| `core` | none (workspace crates) |
| `ai` | `core` |
| `wasm` | `core`, `ai` |
| `archive` | `wasm` |
| `analysis` | `core`, `wasm`, `archive` |
| `sim` | `core`, `ai`, `wasm`, `archive`, `analysis` |

Notes:

- `crates/core` has only `serde` and `serde_json` as dependencies. Outside tests it does no file I/O, spawns no threads and never reads the clock. `lib.rs` also sets `#![deny(clippy::float_arithmetic)]`.
- `crates/wasm` is both a `cdylib` (for `wasm-pack`) and an `rlib`. The native crates (`sim`, `archive`, `analysis`) link it as a plain Rust library to share the `Session` game loop and the save and tape code. On native targets it adds `rayon` so AI seats plan in parallel; on `wasm32` it stays single-threaded.
- `crates/ai` has an optional `parallel-search` feature (rayon) used only by the simulation runner's forward search (`--search`).
- The game ships only as a static web app (`app/dist`, built by `scripts/build-web.sh`, served by Cloudflare Pages). There is no native shell and no server program.

## The turn pipeline

State changes by exactly one function:

```
resolve_turn(&GameState, &BTreeMap<PlayerId, Vec<Order>>) -> (GameState, Vec<Event>)
```

defined in `crates/core/src/rules/mod.rs`. It clones the state, applies every player's orders together, and returns the next state and a turn report. Invalid orders are skipped and reported as `OrderFailed` events; they are never fatal. Related entry points:

| Step | Where |
| --- | --- |
| Create a game from a `GameSetup` | `new_game` in `crates/core/src/game.rs` (map from `mapgen.rs`, then each empire's founded capital, a Scout and a Spearman) |
| The order vocabulary | `Order` in `crates/core/src/orders.rs` |
| Validate a player's orders without applying them | `validate_orders` in `crates/core/src/rules/mod.rs` |
| Resolve a turn | `resolve_turn` in `crates/core/src/rules/mod.rs` |
| What a player may see | `player_view` and `public_view` in `crates/core/src/view.rs` |
| Combat previews | `forecast_attack_mode` in `crates/core/src/forecast.rs` |

Orders are simultaneous: nobody acts "first", and each unit, city or research slot receives at most one order per turn. `resolve_turn` runs these phases in order (see the body of the function; per-system detail is in `crates/core/src/rules/`):

1. Start-of-turn effects and surprise declarations of war.
2. Barbarian planning, unit orders, city orders, then one-time empire actions.
3. Ranged attacks.
4. Movement rounds with combat, repeated until nobody advances; then barbarian camp clearing.
5. City founding, elimination, healing and fortification.
6. City production and growth, gold and upkeep, research, borders, roads, visibility.
7. Scoring and victory checks, diplomacy updates at the turn boundary, barbarian end-of-turn, then `turn += 1`.

### Who produces orders

`crates/wasm/src/session.rs` defines `Session`: a `GameState` plus one `Brain` per AI seat. Every host uses the same three calls:

1. `Session::ai_orders()` builds each living AI seat's own `PlayerView` and asks that seat's brain for orders. Seats are independent, so native builds plan them in parallel and gather results by seat id.
2. The host adds human orders to the resulting `OrderMap`.
3. `Session::step(&orders)` calls `resolve_turn`, replaces the state and appends to the optional recording.

The WASM `Game` type in `crates/wasm/src/lib.rs` wraps a `Session` for JavaScript. Its `end_turn_json` validates that the submitted orders belong to human seats, adds AI orders, resolves the turn, and returns the events each human seat is allowed to see (`crates/core/src/event_visibility.rs` filters the report per viewer) along with the new state hash.

## Determinism

The same setup and orders must yield the same state on every platform, native and WASM. The rules that make this true:

- **Integer maths only.** `crates/core` denies float arithmetic at compile time. Percentages, rates and ratios are integers with explicit rounding.
- **Seeded RNG streams, one per purpose.** `crates/core/src/rng.rs` implements SplitMix64 and `Rng::derive(seed, &[turn, purpose, entity ids...])`. Each draw is a function of the game seed, the turn, a purpose constant (`MAPGEN`, `COMBAT`, `RUINS`, `AI`, `CITY_CAPTURE`, `BARBARIANS`, `OBJECTIVES` and others) and the entities involved, so the result never depends on the order in which systems call it. AI seats get their own seed from `brain_seed` in `session.rs`.
- **No iteration-order dependence.** State uses ordered collections (`BTreeMap`, `BTreeSet`) and sorted vectors. Ties break on ids or hex order, never on hash order.
- **A canonical hash.** `GameState::hash` (`crates/core/src/state.rs`) is FNV-1a 64 over the state's JSON serialization. 64-bit seeds and hashes cross the JavaScript boundary as decimal strings (`crates/wasm/src/json.rs`).
- **Hidden information stays hidden.** Clients and the brains of Easy, Normal and Hard seats receive a `PlayerView`, not the state. AI difficulty changes decisions, not resources or rules. Two research tools read the true state and are listed under [AI layers](#ai-layers).

How the rules are enforced by tests:

- **Order tapes** (`crates/wasm/src/tape.rs`) record a game's setup, every turn's orders and the state hash after each turn. `replay_checked` replays a tape twice, the second time saving and reloading at every turn boundary, and fails on the first mismatching hash.
- **Pinned fixtures.** `crates/sim/tests/determinism.rs` replays three committed 40-turn tapes (seeds near `u64::MAX`) and checks all 120 hashes. `crates/sim/tests/pins.json` holds digests for longer runner games and a scripted-Hard city-fire tape. After an intentional rules change, `tools/sim/repin.sh` regenerates the tapes and pins; the procedure is in `crates/sim/tests/fixtures/README.md`. A diff in those files is meant to be reviewed, not waved through.
- **Cross-target checks.** `scripts/wasm-tape-check.mjs` replays native tapes in the WASM build under Node and compares hashes. `scripts/wasm-search-check.mjs` plays whole games with the `search` flag set on Hard seats through the app's entry point on both targets and compares per-turn hashes.
- **Runner self-check.** `legatum-sim --determinism` plays every game twice and compares per-turn hashes.
- **Property tests.** `crates/core/tests/property_fuzz.rs` resolves randomized turns twice and checks invariants. `visibility_equivalence.rs` compares production visibility against a reference traversal, and `order_privacy.rs` checks that rejected orders do not reveal hidden entities.

## How the UI talks to the engine

The UI depends on one interface, `GameClient` in `app/src/client.ts`. Local games use WASM; online seats use the remote client:

| Client | File | Used for |
| --- | --- | --- |
| WASM | `app/src/wasmClient.ts` | Single player. The engine runs in the page, loaded from `app/src/wasm-pkg` (built by `scripts/build-wasm.sh`). Views, validation, forecasts, saves and turn resolution are JSON strings across the WASM boundary. |
| Remote | `app/src/remote.ts` | Online play. The client is bound to one human seat and talks to the host's tab through the relay's room URL (`setServerBase`); `app/src/relaySocket.ts` reassembles `chunk` messages. The host's own seat uses an in-tab socket instead. |

The game state lives in a Zustand store (`app/src/store.ts`); the map is drawn on a 2D canvas (`app/src/map/`, entry points `MapCanvas.tsx` and `render.ts`). Test fixtures in `app/src/test/` start from Rust-generated saves and resolve turns through WASM.

**Online host.** Hosting runs in the host's browser tab. `crates/wasm/src/host` is the authoritative game: one `Session`, its staging room, seat credentials and every connected player. It has no transport and does no I/O. `app/src/net/browserHost.ts` feeds it messages and sends what it queues to the relay tunnel or to the host's own UI, and keeps the save and game histories in the browser (IndexedDB). Clients send tagged JSON messages (`crates/wasm/src/host/protocol.rs`: `Join`, `Orders`, `Ready`, `Forecast` and others). When every connected human seat is Ready, the host asks the session for AI orders, steps the turn, saves and queues per-seat views. A seat only ever receives its own view. The host contains no game rules.

**Game uploads.** The hosted web build (`scripts/build-web.sh` with `LEGATUM_GAMES_URL`) uploads each game's recording every five turns and at the end (`app/src/platform/gameUpload.ts`, called from `deliver()` in `archive.ts`). The engine's `Recording::anonymized` (exposed as `Game::public_archive` in `crates/wasm/src/lib.rs`) renames every seat to its default and replays the orders to recompute the hashes; the `infra/games` Worker stores the file in a private R2 bucket and refuses any other seat name. Other builds upload nothing.

**Online relay.** Online play (`plans/online-play.md`) adds no game code. The host's tab dials out to a Cloudflare Worker (`infra/relay`: a `Lobby` Durable Object listing rooms and one `Room` Durable Object per game) over a WebSocket tunnel (`app/src/net/tunnel.ts`). The relay pipes frames between the host's tunnel and up to 8 guests and parses none of the game messages; the host stays authoritative. Closing the host's tab ends the game; reloading it resumes from the save. Guests need the same version. The lobby screen is `app/src/screens/OnlineLobbyScreen.tsx` (`app/src/platform/relayLobby.ts`).

## Save files and versioning

- **Save format.** A save is the serialized `GameState` as JSON, written by `state_to_json` and read by `parse_state` in `crates/wasm/src/json.rs`. `Session::save_json` adds two optional sections: `archive` (the game's recording, see below) and `ai_opponent_memory` (what each AI seat has learned from its own views, which contains no hidden state). Faction ids in saves are the `Faction` enum names (for example `Switzerland`); a renamed faction keeps its old name as a serde alias (`Japan` still parses as India). Units, buildings and improvements are named the same way: the identifier is the PascalCase of the display name (`Spearman`, `GreatLibrary`), and the names used before the 2026-11-22 rename (`Soldier`, `RelayUplink`, ...) still parse as serde aliases in `crates/core/src/types.rs`, for tapes, archived games and fixtures. The state hash is taken over this JSON, so renaming an identifier changes every hash and needs a `RULES_VERSION` bump.
- **Versions.** The writer stamps every save with `save_version = legatum_core::RULES_VERSION`, and `parse_state` refuses a save whose version differs (or is missing) with an explicit "incompatible rules version" error. Any change to the rules or to the serialized shape bumps the constant; there are no migrations. The hosted site opens an other-rules save in the earlier build that made it: every web build is also published at `v/<id>/` and listed in `versions.json` for 30 days (`tools/release/keep-builds.mjs`, `app/src/platform/versions.ts`; `docs/ops/releasing.md`), and the autosave database stays at version 1 so every kept build can open it. Tapes carry their own `version`.
- **Stable serialization by serde defaults.** New fields are added with `#[serde(default)]` and, where a default value should leave existing serializations untouched, `skip_serializing_if`. The comments in `crates/core/src/state.rs` call this out ("omitted when absent, so old hashes hold"), which keeps pinned tape hashes stable.
- **Fixtures.** Current-version playtest saves live in `crates/wasm/tests/fixtures/saves/` (see its README); `current_browser_fixtures.rs` loads them, resolves a turn and reloads the result. Other fixtures in `crates/wasm/tests/fixtures/` pin specific reported bugs, each replayed from a saved state.
- **Archive and replays.** `crates/wasm/src/history.rs` defines a portable recording (tape, metadata, periodic checkpoints). `crates/archive` stores recordings as compressed files outside the repository, and `crates/analysis` replays them to produce AI diagnostics. On load, an embedded recording is kept only if it replays under the current rules and matches the saved state; otherwise the game resumes from the saved state with a fresh recording.

## AI layers

All brains implement `Brain` (`crates/ai/src/lib.rs`): `orders(&mut self, &PlayerView) -> Vec<Order>`. The brains behind the game's three levels (Easy, Normal and Hard) are deterministic functions of their view and seed, and keep only view-derived memory. `make_brain` in `session.rs` picks the implementation for a seat. The last two rows of the table are research tools that read the true state; each needs its own setup flag, which no level sets.

| Layer | Where | Role |
| --- | --- | --- |
| Random | `random.rs` | A random-but-plausible baseline, used as the floor in simulations. |
| Scripted | `scripted.rs`, `production.rs`, `military.rs`, `research.rs`, `diplomacy.rs` and others | The main AI. Per-faction knobs (expansion, military, science, economy, aggression) steer production, research, expansion, settling, defence and diplomacy. `difficulty.rs` scales the knobs for Easy, Normal and Hard without touching resources or rules. |
| Plans | `plan.rs`, `campaign.rs`, `threat.rs`, `opponents.rs` | Hard reviews strategic plans (build economy, make peace, run a campaign against a target, contain a leader, snipe, hold) and tracks rivals and threats from public information. |
| Openings | `openings.rs`, `tuned_openings.rs`, `hard_v2.rs` | Named opening policies (Rush, GreedyExpand, Snowball, Economic) with tunable genomes; Hard v2 is an opt-in experiment. |
| Search | `search/` | `SearchBrain` wraps the scripted brain. At plan review points it builds sampled worlds from its own `PlayerView` (`determinize`), plays each candidate plan forward on the real engine with `resolve_turn`, scores outcomes with a win-chance estimate (`crates/ai/src/winchance.rs`) and adopts the best plan. Rollouts are seeded, so games replay exactly. |
| Search diagnostic | `search/` with `SearchConfig::omniscient` | Simulator only. An upper-bound measurement that gives the search the true state instead of sampled worlds. The app never sets it. |
| Battle study | `training.rs` | A Hard seat with the setup's `search` flag plays plain Hard, except that each battle is cut from the true state into a small arena, including units the seat cannot see, and searched. The results are saved as training data. It is a research tool: the app's level pickers offer Easy, Normal and Hard only, and list it only in a development build opened with `?training=1` (`app/src/lobby.ts`). The search is capped by wall-clock time, so such a game replays from its recorded orders but not from its seed alone. |
| Tunable parameters | `params.rs` | `AiParams` is a JSON file of decision knobs. It cannot express rules, costs or vision. |
| Benchmark opponents | `proxy*.rs` | Fixed scripted policies modelled on human play, used as reference opponents by the runner. |

## The simulation runner and scenarios

`crates/sim` builds `legatum-sim` (`src/main.rs`; `--help` lists every flag). It plays AI-only games in parallel with rayon, one game per task, using the same `Session` as the app. Typical outputs are markdown and JSON reports (win rates by faction, pacing, telemetry modules such as `hold_telemetry.rs` and `opponent_telemetry.rs`), optional order tapes (`--tape-out`) and replays. Other modes: `--determinism`, `--proxy` (a human-proxy benchmark seat), `--duel-matrix`, `--search`, `--ai-params`, `--replay` and `--tape-in`. Larger studies live in `crates/sim/examples/`, such as `hard_vs_human.rs` and `war_suite`, which the scripts in `tools/sim/` wrap.

Scenario files are AI measurement positions with a checked start state, objective and predictions:

- **Format.** `crates/wasm/src/scenario.rs` defines the file format (a gzip and base64 encoded save, checked against a recorded hash). `crates/core/src/scenario.rs` defines objectives and scoring, which read only game state, so the runner and replays score a position identically.
- **Files.** `scenarios/*.json` holds root-level test positions; `scenarios/lab/*.json` holds AI measurement cuts from saves. `scenarios/war/*.json` holds short wars cut from recorded human games and brought forward to the current rules by `legatum-sim war-build` (`crates/sim/src/war_build.rs`).
- **AI baselines.** `legatum-sim scenario <file> --runs N` has the scripted AI play the scenario seat across many seeds and writes a report. The human-seat benchmark and the war suite compare AI changes against recorded human results; the methodology is in [docs/sim-results/hard-vs-human/README.md](docs/sim-results/hard-vs-human/README.md).

## Test layers

| Layer | Location | Command |
| --- | --- | --- |
| Rust unit tests | Inline `#[cfg(test)]` modules and `*_tests.rs` files in each crate | `cargo test --workspace` |
| Rust integration tests | `crates/*/tests/`: determinism and pins (`sim`), playtest saves and scenarios (`wasm`), the online host protocol (`wasm`), property and privacy tests (`core`), AI behaviour (`ai`) | `cargo test --workspace` |
| TypeScript unit tests | `app/src/**/*.test.ts(x)` (Vitest, Node environment) | `cd app && npm test` |
| WASM behavior tests | `app/src/**/*.wasm.test.ts`: core-backed fixtures and browser-facing engine APIs | `cd app && npm test` (needs `scripts/build-wasm.sh` first) |
| Browser end-to-end | `app/e2e/*.spec.ts` (Playwright, Chromium) against the static production build and a local relay (`wrangler dev`); CI runs a smoke subset per pull request and the full suite nightly | `cd app && npm run e2e` |
| Cross-target determinism | `scripts/wasm-tape-check.mjs`, `scripts/wasm-search-check.mjs` | see the headers of those scripts |

`cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` are the expected lint gates for Rust; `npm run build` type-checks the TypeScript, including the Playwright specs.

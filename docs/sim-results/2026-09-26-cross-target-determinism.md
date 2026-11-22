# S8 cross-target determinism — 2026-09-26

All checked hashes matched. The requested 30-game run (seeds 1000–1029) produced
2,398 turns: 29 games reached 80 turns and seed 1025 finished at turn 78. Native
debug, native release, and optimized wasm32 in Node agree on the initial state
and every recorded turn, including continuation after save/load. No engine
divergence was found; game rules and the state-hash algorithm are unchanged.

The exact commands, exit codes, and complete stdout/stderr (`2026-09-26-cross-target-determinism.log`; raw run
output, not in the public repository) include the original checks, the expanded checks, fixture generation, all project
checks, and a supplementary JavaScript API check.

## Changes

- Shared `tape::replay_checked` verifies uninterrupted replay and a second replay
  that saves and loads before turn one and after every turn, then uses the loaded
  state to continue. Every save/load must preserve the complete state; every
  resumed turn must match the recorded hash. Final saves are included.
- Native `--tape-in` and WASM `Game.replay_tape` use this checker. WASM now rejects
  an incorrect initial hash, which the old JavaScript comparison omitted.
  Incorrect turn numbers and extra replay hashes are rejected too.
- Three mixed-brain, 40-turn tapes near `u64::MAX` pin 120 turns independently of
  future AI changes. Six integration tests check the fixed hashes and rejection
  of corrupted initial hashes, turn hashes, and turn numbers. The fixtures and
  [intentional regeneration instructions](../../crates/sim/tests/fixtures/README.md)
  live under `crates/sim/tests/`.
- Enabled three previously ignored integration tests for full core turns, native
  simulation replay, and API save/load; their dependencies are now implemented.

## Commands and results

All final commands exited 0. The complete output is in the linked transcript.

```sh
cargo run --release -p legatum-sim -- --games 30 --seed 1000 --turn-limit 80 --label tapes --tape-out target/tapes
cargo run --release -p legatum-sim -- --tape-in target/tapes
cargo run -p legatum-sim -- --tape-in target/tapes
wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node
node scripts/wasm-tape-check.mjs target/tapes target/wasm-node
```

| Check | Result |
| --- | --- |
| Generate the requested 30 games | `30 games, 0 panics, avg 79.9 turns` |
| Native release replay | `PASS: 30 tapes replayed natively, all hashes match (initial state, every turn, and save/load)` |
| Native debug replay | Same PASS output as release |
| Optimized WASM build | `Your wasm pkg is ready to publish at target/wasm-node.` |
| WASM replay | `PASS: 30 tapes, WASM hashes match native (initial state, every turn, and save/load)` |
| `cargo test -p legatum-sim --test determinism` | 6 passed, 0 failed, 0 ignored; 0.85s |
| `cargo test --release -p legatum-sim --test determinism` | 6 passed, 0 failed, 0 ignored; 0.11s |
| `node scripts/wasm-tape-check.mjs crates/sim/tests/fixtures target/wasm-node` | 3 tapes, all 120 turn hashes match, including save/load |
| `cargo fmt --all` | No output, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | No warnings, exit 0 |
| `cargo test --workspace` | 187 passed, 0 failed, 0 ignored: AI 27, core 129, sim 14, fixtures 6, WASM API 11 |

The supplementary Node check used the real `Game` constructor, `end_turn`, `save`,
and `load` exports, regenerating scripted AI orders and restoring after every
turn. All 30 games and 2,398 turns matched the native tapes. The actual WASM
export also rejected deliberately corrupted initial and per-turn hashes. The
exact Node command is preserved in the transcript.

## Environment and limits

Native host: `aarch64-apple-darwin`; Rust `1.98.1`, Cargo `1.98.1`, wasm-pack
`0.15.0`, Node `v26.10.0`. WASM used the normal release build with `wasm-opt`.
These results cover the recorded games and Node runtime, not every possible game
or browser runtime.

The first WASM build compiled the crate but exited 1 when `wasm-pack` tried to
lock its cache outside the sandbox. Reusing the installed tools in a writable
cache fixed the build without disabling optimization:

```sh
cp -R $HOME/Library/Caches/.wasm-pack target/s8-checks/wasm-pack-cache
export WASM_PACK_CACHE="$PWD/target/s8-checks/wasm-pack-cache"
```

The 30 full-size tapes and simulation detail reports remain generated artifacts
under `target/`; only the small regression tapes and this verification record
are included in the change.

Direct staging was blocked because the worktree's shared Git metadata is outside
the writable roots (`index.lock: Operation not permitted`). A commit bundle is
prepared inside `target/s8-checks/` for the lead to import; the original worktree
branch cannot be updated from this sandbox. No merge into master was attempted.

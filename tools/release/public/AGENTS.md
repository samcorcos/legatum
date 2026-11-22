# Legatum

The full project instructions for AI agents are in [CLAUDE.md](CLAUDE.md) and `.claude/skills/`; this is the short version.

A historical 4X on a hex map. Rust engine, React/TypeScript UI. Read [CONTRIBUTING.md](CONTRIBUTING.md) for
setup and the rules, [ARCHITECTURE.md](ARCHITECTURE.md) for the design, and [the rules reference](.claude/skills/legatum-rules/references/index.md) for how the
game actually works before proposing any mechanic.

## Layout

- `crates/core`: game state, orders, turn resolution. Deterministic, integer maths, no I/O.
- `crates/ai`: scripted AI brains. `crates/sim`: headless runner and tests. `crates/wasm`: browser bindings.
- `crates/archive`, `crates/analysis`: game recordings and analysis.
- `app/`: the React web UI. `scenarios/`: hand-built test positions.
- `docs/`: rules reference, design notes, methodology and engineering notes.

## Commands

```sh
scripts/build-wasm.sh && (cd app && npm ci && npm test && npm run build)
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
```

## Conventions

- One engine: rules live only in `crates/core`; never re-implement them elsewhere.
- Deterministic: integer maths, seeded RNG streams, no hash-map iteration order. Rule changes that change
  outcomes re-pin tapes with `tools/sim/repin.sh`.
- Saves and tapes from another rules version are refused and there are no migrations: bump `crates/core/src/lib.rs::RULES_VERSION` when a change alters saved state or results; update the matching file under `.claude/skills/legatum-rules/references/` with any rule change; delete dead code instead of commenting it out.

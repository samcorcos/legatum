# Tools and scripts

Helper programs around the engine. Everything here is optional for building and playing.

| Path | What it does |
| --- | --- |
| `tools/sim/` | Simulation, combat and pacing runs, and `repin.sh`, which regenerates the pinned determinism tapes and hashes after an intended rule change. |
| `tools/analysis/` | Balance, strategy-diversity and human-playbook analysis. |
| `tools/scenarios/` | Extract, cut, compare and show saved game states and scenarios. |
| `tools/art/` | Unit-art distinctness checks and derivative generation. |
| `tools/audio/` | Build the shipped audio from raw downloads: trim, loudness-normalise, encode, write credits. |
| `scripts/build-wasm.sh` | Build the engine as WASM for the web UI. |
| `scripts/archive-stats` | Summarise or export the game-archive index. |
| `scripts/wasm-tape-check.mjs`, `scripts/wasm-search-check.mjs` | Check that WASM and native builds produce identical per-turn state hashes. |
| `scripts/perf-profile.sh` | Record a reproducible single-threaded performance profile. |

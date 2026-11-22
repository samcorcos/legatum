# Simulation replays

Run `node tools/dashboard/server.mjs`, open `http://localhost:4747`, and scroll to
**Replays**. Pick a game, scrub the timeline, or play at 1×, 4×, or 16×. Click a
chronicle event to jump to its turn. With the section focused, use Left/Right,
Home/End, and Space. Hover the map for tile, city, population, unit mix, and
health-weighted army strength; expand a player's VP sources for scoring details.

## Generate an archive

From the repository root:

```sh
cargo run --release -p legatum-sim -- \
  --games 200 --seed 1 --label replay-showcase --export-replays 6
```

The exporter records orders during the run, chooses up to N distinct finished
games, and replays the selected tapes through `resolve_turn`. It checks the
initial state hash and every resolved turn against the original recording.
Panics, unfinished games, and failed determinism checks are excluded. Selection
reserves one conquest if available, then cycles through:

- Closest final VP margin between the top two surviving teams (including draws).
- Most city captures.
- Largest deficit overcome by the eventual winner after turn 75, provided that
  turn had a unique VP leader and that leader lost. Tied leaders and games that
  ended at or before turn 75 do not qualify.

Ties are ordered by simulation index. Selection is independent of worker order.
Each index entry describes its actual margin, captures, and comeback deficit.
For N smaller than the available categories, conquest takes priority, followed
by close games, captures, then comebacks. Other slots fill from those rankings.

To import an existing order tape or a flat directory of tape JSON files:

```sh
cargo run --release -p legatum-sim -- \
  --replay-export path/to/tape.json --out .progress/replays --label imported

cargo run --release -p legatum-sim -- \
  --replay-export path/to/tapes --out .progress/replays --label imported
```

`--out` defaults to `.progress/replays` in this checkout; `--out-dir` still
controls simulation reports. Native game replays are named
`<safe-label>-game-<index>-seed-<seed>.json`, with an index padded to five digits.
Importing native `game-<index>-seed-<seed>.json` tapes preserves that index,
including when importing one file or a subset. Unindexed legacy tapes retain
`<safe-label>-seed-<seed>.json` names.
Re-exporting a filename replaces it deterministically; other index entries stay
intact. Files are written via temporary files and renames, with the index written
last. A directory import rejects duplicate (game index, seed) identities before
writing output. Mirrored games can share a seed because their indices differ.
Use separate labels for separate runs that reuse the same indices and seeds.
Use one exporter at a time per output directory.

## Format (version 1)

- `header`: label, optional native `game_index`, decimal-string u64 seed, width,
  height, terrain index per tile,
  `terrain_kinds`, players (id, name, faction, team, color_index), winning **team**
  id or null, outcome, number of resolved turns, and VP targets keyed by team id.
- `frames`: exactly one snapshot **after** each resolved turn. Frame 0 is turn 1,
  not the initial setup. The last frame includes the final scoring and outcome
  events. Partial tapes export with outcome `unfinished`.
- `owners`: `[owner, count]` run-length pairs, row-major odd-r; -1 is unclaimed.
  Expanding the runs produces exactly width × height values.
- City and army `pos` is axial `[q, r]`. Row is r and column is
  `q + (r - (r & 1)) / 2`. The viewer renders pointy-top hexes with odd rows
  offset to the right.
- `cities`: id, owner, name, pos, pop, capital. `units`: one entry per player and
  tile, with unit-kind counts and health-weighted military strength (the same
  metric as the Strongest Army title).
- `vp`: player id, total, named sources, alive. `titles`: title to player id or
  null. `events`: resolved turn plus human-readable text. Battles are grouped by
  tile per turn; sieges, lethal battles, and 3+ exchanges qualify as notable.

This is an omniscient replay: it includes all empires, with no fog of war.
The chart shows individual players' VP and the applicable team target lines.
Color is reinforced with player numbers and distinct chart dash patterns.

## Showcase run and checks

The `replay-showcase` archive was generated from a fresh 200-game scripted run,
seeds 1–200: 156 VP wins, 37 turn-limit wins, 7 draws, no panics, no conquests.
Selected seeds are 30, 42, 54, 77, 109, and 126. Seed 42 has five city captures;
seed 109's winner recovered from an eight-point deficit at turn 75. Replays
contain 134–150 frames and are all below 1 MB. The complete run records and
summary are in `docs/sim-results/2026-09-26-replay-showcase.{json,md}`.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test tools/dashboard/replay.test.mjs
```

The Node check creates a tiny replay archive in a temporary directory and removes
it afterward; it does not need generated `.progress/replays` files. It exercises
the real HTTP handler without binding a port: it reads the page and assets,
parses the fixture index and replay, checks frame counts,
coordinates, ownership, score totals and size, and rejects unsafe paths and
symlinks. It does not verify browser rendering. The viewer uses scoped CSS and
an isolated script; it does not participate in the existing dashboard refresh.
Generated replay JSON files and their index remain ignored with `.progress/`.

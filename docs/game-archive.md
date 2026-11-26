# Game archive

Legatum records full owner histories so balance analysis does not depend on replaying an old
engine. The simulator's native archive defaults to `~/LegatumData/archive`. `LEGATUM_ARCHIVE`
overrides the default, and `--archive-dir <directory>` overrides the environment. Archive roots inside **any Git checkout** are rejected, including symlink aliases.
The archive is private local data, outside the repository. Games played in the browser stay in that
browser (below) until they are exported. Histories from the Mac app, before it was dropped on
2026-11-22, are in the same `~/LegatumData/archive` on that Mac.
The one upload is the hosted web build's: with `LEGATUM_GAMES_URL` set it sends each game's recording, every
seat renamed to its default (`Recording::anonymized`, replayed so the hashes still match), to the R2 bucket behind
`infra/games` (see `infra/games/README.md` and the README's Privacy section).

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo run --release -p legatum-sim -- --games 100 --archive
cargo run --release -p legatum-sim -- --games 100 --archive --archive-dir /path/to/archive
scripts/archive-stats
scripts/archive-stats --csv /tmp/legatum-analysis
cargo run --release -p legatum-sim -- --replay ~/LegatumData/archive/2026/09/<game_id>.tape.json.zst
```

Simulations archive every primary game when `--archive` is set, including safety-cap stops
(`stalled`) and caught turn failures (`panic`). Determinism verification plays are checks of
the same game, not additional archive entries. Initialization failures before a game exists
are reported as failures rather than fabricated histories. Omitting `--archive` keeps the
existing simulator behavior and report locations. The server and real local WASM game always
record; the mock UI engine is not a gameplay archive source.

## Files and schema 1

```text
archive/
  2026/09/<game_id>.tape.json.zst
  in-progress/2026/09/<game_id>.tape.json.zst
  index.jsonl
  archive.lock
```

The UTC creation date determines the year/month directories. Native writers use zstd level 9
with a frame checksum (a throughput/compression tradeoff); readers also accept gzip `.gz`
and uncompressed `.json` exports. There is one final file per game and one metadata-only JSON
line per finished/stopped game in `index.jsonl`. Checkpoints are not counted as finished games.
Finishing removes that game's in-progress checkpoint. Index appends and file replacement share
an OS file lock across simulator threads and processes. Files are
flushed and renamed atomically; repeated uploads/finish recovery do not duplicate index lines.
Finished files are immutable. Earlier checkpoints cannot replace later ones.

Each decompressed file is an object:

| Field | Contents |
| --- | --- |
| `metadata` | Summary described below; exactly this object is appended to the index |
| `tape` | Versioned order tape: `version`, `setup`, decimal-string `initial_hash`, and `turns` with turn number, orders per player, and decimal-string resulting hash. Current diplomacy games use version 3; legacy all-war games retain version 1. |
| `initial_state` | Normally null; opaque state JSON for recording resumed from a legacy save |
| `telemetry` | Initial snapshot and one snapshot after every resolved turn |

Metadata fields:

| Field | Meaning |
| --- | --- |
| `schema_version`, `game_id` | Envelope version and collision-resistant, path-safe identity |
| `created_at`, `finished_at` | RFC3339 UTC timestamps; finish is null while in progress |
| `source` | `human-local`, legacy `human-hotseat` (2026-11-23, read only), `lan`, or `sim`; online Host games use `lan` (`history::Source::Lan`, a historical name). The legacy value remains readable in old archives |
| `speed`, `map`, `seed` | Economic speed, generator plus width/height, decimal-string u64 seed |
| `seats` | Player id, entered name, faction, team, difficulty, human flag, final/current VP and held titles |
| `winner`, `outcome` | Winning **team id**, or null; `vp`, `elimination`, `turn-limit`, `draw`, `stalled`, `panic`, `abandoned`, or `in-progress` |
| `turn_count` | Number of recorded resolved turns; excludes the initial telemetry snapshot |
| `engine_commit` | Git commit compiled into the engine; see provenance limitation below |
| `rules_fingerprint` | SHA-256 of sorted core/WASM Rust source paths and contents, including data tables, resolution, state hashing and serialization |
| `first_turn`, `complete_history` | Where recording starts, and whether orders cover the game from setup |
| `parent_game_id` | Original identity when an explicitly loaded save forks another play |

Names stay as entered, including the names online guests join with, independently of hashed engine state (they are in the state hash, which is why an anonymized recording replays its orders to recompute every hash). The only exception is the recording the hosted web build uploads and queues: seat names are "Player N" / "AI N".
No seat credentials, authorization headers, tokens, pending socket messages or connection
addresses enter the recording. Archives intentionally contain full game information for the
owner's analysis, including information hidden by fog in the player UI.

Client telemetry rides in the recording but never in a hash: `turn_times` (active play seconds per turn and
session) and `frame_times` (`TurnFrameTime` in `crates/wasm/src/history.rs`, filled by `app/src/frameTelemetry.ts`).
`frame_times` has one row per turn the player viewed: `frames`, `frame_p50_ms10` and `frame_p95_ms10` (gaps in
tenths of ms), `frame_max_ms`, `slow_frames` (gaps over 50 ms), `inputs`, `input_p95_ms`, `input_max_ms`,
`long_tasks`, `long_task_ms`, `long_draw_ms`, `long_terrain_ms` and `long_engine_ms` (how much of the long tasks was
map drawing, terrain chunk painting and engine calls; the rest is interface work; added 2026-11-24, absent in
older records), `map_draws`, `map_draw_mean_ms10`, `map_draw_max_ms`, `units`, `cities`, `zoom100`
and `canvas_mp10` (map canvas in tenths of a megapixel). It identifies no browser, window or screen. It is
optional and default-empty (omitted when empty), added in 2026-11 without a `RULES_VERSION` bump, so saves from
before it still load; it is uploaded with the game and summarised by `tools/games/analyse_games.py`.

Each telemetry snapshot has `turn` and `players`. Player rows contain `player`, `vp`, `cities`,
`population`, `science` (produced in that resolved turn), `science_stored`, `gold` (treasury),
`army_strength` (health-weighted Strongest Army title metric), `techs` (completed count),
`alive`, `wars`, and `peace`. Turn 0 is the starting snapshot for complete histories.
While this player is alive, `wars` lists living foreign players whose teams are currently
at War, and `peace` lists living foreign players whose teams are at Peace or in a Pact.
Permanent teammates are excluded from both lists; their teams appear in seat metadata.
Legacy all-war games still list every living foreign player in `wars`. These snapshots
describe active relationships after resolution, not counts of declarations or agreements.

## Saves, checkpoints and browser delivery

Session save JSON retains its existing state fields and adds `archive`, carrying the entire
recording and stable identity. Native/WASM state hashes do not include the recording. The
browser's per-turn autosave keeps it; a standalone compressed checkpoint is written at game creation,
every five turns by default, and at finish (the browser also writes one after each of the first five turns). A restart restores the same history and retries
the archive write, including a finish interrupted before indexing. Lost online-seat credentials are
still handled by the host-controlled seat recovery; credentials are never restored
from a tape. Explicit file loads fork the archive identity rather than overwriting another play.
Replacing an unfinished game preserves its final recorded turns and indexes it as `abandoned`.

Legacy saves lack earlier orders. A save is loadable only when its `save_version` matches the
current `RULES_VERSION`; saves from other rules versions are rejected, with no migrations by
design. A loadable legacy save gets an `initial_state` anchor and `complete_history: false`, and
records subsequent turns. The replay command verifies from that anchor. Earlier turns are never
invented.

In the browser, the app captures each game's history at setup, after each of the first five turns, then every five turns, and at the finish,
and keeps it in IndexedDB (gzip-compressed where browser compression is available) until it is
delivered; a newer checkpoint of a game replaces the older one. The complete tape is also in the
ordinary per-turn autosave. **Load an earlier turn** (local games only) opens one of the game's last 10 autosaved turns under the same game id; the tape then holds only the line played on, the record is uploaded at once instead of at the next checkpoint, and the times the player went back ride along as the upload-only `rewinds` key (`app/src/store.ts` `rewindTo`, `Rewind`; `infra/games/README.md`). With an upload address, uploads retry after a reload, when the browser
comes online, and at the next checkpoint; without one, histories stay in the browser.
**Export game history** in the game menu (Advanced) downloads the current game and every retained
one, one gzip file per game (plain JSON if CompressionStream is unavailable); the browser may ask to
allow multiple downloads. The current game can be exported even if browser storage is denied or
full. Do not clear browser storage before exporting.

An online game's host is the engine in the host's browser tab (`crates/wasm/src/host`); it hands its
checkpoints and final history to that tab, which files them like a local game (`app/src/net/hosting.ts`,
`recordGameHistory`). A build that uploads games takes them with every seat under its default name (`take_public_recordings`, `crates/wasm/src/host/net.rs`), so online and co-op games are uploaded like single-player ones; other builds keep the names as entered. There is no archive server (the native `legatum-server` is gone, 2026-11-22).

## Analysis, replay and recovery

`scripts/archive-stats` reads only `index.jsonl` for counts by source, speed, outcome and
faction. Faction counts mean games containing that faction, so a game may contribute to
several factions. `--csv <directory>` also reads compressed telemetry and writes `games.csv`,
`players.csv`, and `turns.csv`. It streams games and never invokes the engine. Lists are JSON
cells. CSV text starting with spreadsheet formula characters is prefixed with an apostrophe;
the JSON archive keeps the exact entered name. Python 3.14 includes zstd; older Python needs
the `zstandard` package for CSV export or repair (index-only counts need no extra package).

`legatum-sim --replay <file>` first checks schema and the rules fingerprint, then replays orders
and verifies every state hash, stopping at the first divergent turn. Different commits with
identical engine sources can replay; mismatched fingerprints report both revisions and do not
execute the tape. The fingerprint is deliberately conservative (even source-only changes can
invalidate it). `engine_commit` on an uncommitted development build identifies its base commit;
the fingerprint identifies the actual compiled engine sources. Retain old engine revisions
when exact old-game replay matters. Telemetry and CSV remain useful after rules changes.

Checksums and bounded decompression reject truncated/corrupt zstd files. Corruption is reported,
never silently replaced by a new game. There is no destructive automatic cleanup of completed
games. If a crash committed a finished file but not its index line, retrying that game repairs
the line. `scripts/archive-stats --repair-index` can also scan finished files and append missing
metadata under the same lock; a torn final index line is discarded before recovery.

The original 2026-09-27 tape-only measurements were 245–614 KB raw, 12–32 KB with gzip -9 and
10–26 KB with zstd -19 per Standard game (roughly 25 GB per million). This envelope adds
telemetry and uses zstd level 9, so those are planning estimates rather than measured archive
sizes. Measure the finished files for a representative workload before projecting capacity.

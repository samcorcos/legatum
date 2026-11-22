# Game archive

Legatum records full owner histories so balance analysis does not depend on replaying an old
engine. Native storage defaults to `~/LegatumData/archive`. `LEGATUM_ARCHIVE` overrides the
default, and `--archive-dir <directory>` overrides the environment in the simulator, server,
and Mac app. Archive roots inside **any Git checkout** are rejected, including symlink aliases.
The archive is private local data, outside the repository; it is not a cloud upload.

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo run --release -p legatum-sim -- --games 100 --archive
cargo run --release -p legatum-sim -- --games 100 --archive --archive-dir /path/to/archive
cargo run --release -p legatum-server -- --archive-checkpoint-turns 5
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
  incoming/                       # temporary Mac app plugin-fs writes
```

The UTC creation date determines the year/month directories. Native writers use zstd level 9
with a frame checksum (a throughput/compression tradeoff); readers also accept gzip `.gz`
and uncompressed `.json` exports. There is one final file per game and one metadata-only JSON
line per finished/stopped game in `index.jsonl`. Checkpoints are not counted as finished games.
Finishing removes that game's in-progress checkpoint. Index appends and file replacement share
an OS file lock across simulator threads, server processes and the desktop app. Files are
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
| `source` | `human-local`, `human-hotseat`, `lan`, or `sim`; server Host games use `lan` |
| `speed`, `map`, `seed` | Economic speed, generator plus width/height, decimal-string u64 seed |
| `seats` | Player id, entered name, faction, team, difficulty, human flag, final/current VP and held titles |
| `winner`, `outcome` | Winning **team id**, or null; `vp`, `elimination`, `turn-limit`, `draw`, `stalled`, `panic`, `abandoned`, or `in-progress` |
| `turn_count` | Number of recorded resolved turns; excludes the initial telemetry snapshot |
| `engine_commit` | Git commit compiled into the engine; see provenance limitation below |
| `rules_fingerprint` | SHA-256 of sorted core/WASM Rust source paths and contents, including data tables, resolution, state hashing and serialization |
| `first_turn`, `complete_history` | Where recording starts, and whether orders cover the game from setup |
| `parent_game_id` | Original identity when an explicitly loaded save forks another play |

Names stay as entered, including LAN join names, independently of hashed engine state.
No seat credentials, authorization headers, tokens, pending socket messages or connection
addresses enter the recording. Archives intentionally contain full game information for the
owner's analysis, including information hidden by fog in the player UI. The server does not
provide LAN download access to full archives.

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
recording and stable identity. Native/WASM state hashes do not include the recording. Server
autosaves persist it every turn; a standalone compressed checkpoint is written at game creation,
every five turns by default, and at finish. A restart restores the same history and retries
the archive write, including a finish interrupted before indexing. Lost LAN credentials are
still handled by the existing host-controlled seat recovery; credentials are never restored
from a tape. Explicit file loads fork the archive identity rather than overwriting another play.
Replacing an unfinished game preserves its final recorded turns and indexes it as `abandoned`.

Legacy saves lack earlier orders. A save is loadable only when its `save_version` matches the
current `RULES_VERSION`; saves from other rules versions are rejected, with no migrations by
design. A loadable legacy save gets an `initial_state` anchor and `complete_history: false`, and
records subsequent turns. The replay command verifies from that anchor. Earlier turns are never
invented.

For local single-player and hot-seat WASM sessions, the app captures history at setup, every
five turns, and finish. The complete tape is also in the ordinary per-turn autosave.
Pending deliveries are retained in IndexedDB, gzip-compressed where browser compression is
available, before attempting delivery. Uploads retry after reload, when the browser comes
online, at the next checkpoint, or through **Retry history archive** in the game menu.
The current game can be exported even if browser storage is denied/full; failures surface in
the existing error UI. Do not clear browser storage before exporting undelivered histories.

When served by `legatum-server`, the app sends JSON to `POST /api/archive`. The route uses the
same explicit Host/Origin allowlist and JSON content-type requirement as other server APIs.
Loopback TCP peers may write; LAN peers must present an existing seat/staging token in
`Authorization: Bearer …`. The token is transport-only. Requests have the existing 16 MiB body
limit; native decompression is bounded at 64 MiB. LAN games themselves are recorded by the
authoritative server, never reconstructed from player views. A LAN visitor without a seat
credential retains local games for export until an authorized upload is possible.

On Mac, `tauri-plugin-fs` writes the compressed payload under the resolved archive root.
Native commands prepare checksum-enabled zstd and finalize the staged file/index under the
shared lock. The plugin's added runtime scope is limited to that archive directory; existing
user-selected save permissions and the single app-data autosave remain. macOS may not inherit
shell variables when launched from Finder; launch the executable with `--archive-dir` or an
environment variable to override the default.

A plain static host retains histories locally and offers **Export game history** in the game
menu. It exports the current game and retained games, one gzip file per game (plain JSON if
CompressionStream is unavailable). The browser may ask to allow multiple downloads. Exported
files can be analyzed/replayed without access to the original browser. Automatic writes to
arbitrary Mac directories are unavailable to ordinary static web pages.

Mobile storage in app data and syncing to an authorized server are follow-up work; no mobile
archive transport is implemented here.

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
metadata under the same lock; a torn final index line is discarded before recovery. Retained
`incoming` Mac files are staging artifacts; browser retry can complete interrupted deliveries.

The original 2026-09-27 tape-only measurements were 245–614 KB raw, 12–32 KB with gzip -9 and
10–26 KB with zstd -19 per Standard game (roughly 25 GB per million). This envelope adds
telemetry and uses zstd level 9, so those are planning estimates rather than measured archive
sizes. Measure the finished files for a representative workload before projecting capacity.

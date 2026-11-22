# Legatum LAN host

From the repository root, install the UI dependencies once and start the host:

```sh
cd app && npm ci && cd ..
./scripts/lan-host.sh
```

The script builds `app/dist` and runs:

```sh
cargo run --release -p legatum-server -- --static app/dist
```

Open `http://localhost:7878` on the host computer. The server prints each non-loopback
IPv4 URL to share with the other players. It binds `0.0.0.0:7878`. To override
the port or saves directory:

```sh
./scripts/lan-host.sh --port 7879 --saves /tmp/legatum-saves
```

The default saves directory is `saves/` in the platform data folder:
`~/Library/Application Support/Legatum/saves/` on macOS, `~/.local/share/legatum/saves/`
(or `$XDG_DATA_HOME/legatum/saves/`) on Linux, `%APPDATA%\Legatum\saves\` on Windows.
Finished games are also recorded in the local archive, `~/LegatumData/archive/` by default
(`--archive-dir` or `LEGATUM_ARCHIVE` to change it; not inside the Legatum source checkout).
If the archive cannot be opened, the server hosts without it and prints one line.
`autosave.json` is restored on startup. New/load operations and every resolved
turn write that file and `turn-<n>.json`, retaining the ten most recently written
turn snapshots. The snapshot write and pruning precede the atomic autosave
replacement (temporary file, flush, rename), which is the final fallible step.
Failed new/load operations preserve the live game and restart autosave. A corrupt startup
save produces an error instead of being overwritten. If autosaving fails after a
turn, the turn still broadcasts, followed by an explicit error.

## Online room (relay)

`--relay <url> --name <host name> --build <label>` also opens a room on the online relay
(`plans/online-play.md`): the server announces it with `POST /rooms`, prints `Online room: <id>`,
and holds one outbound tunnel WebSocket (`ws://` or `wss://`, rustls with bundled web roots, no
proxy). Without these flags nothing changes. Every relayed guest runs the same connection loop as a
LAN socket with `local = false` and one shared admission peer (`100::1`, so at most eight relayed
sockets). Guest-bound texts over 256 KiB travel as `chunk` messages; LAN sockets never see them.
The tunnel pushes `status` (with `is_local: false` and no LAN URLs) and the lobby `summary` on
connect and whenever they change, pings every 20 s, reconnects with 1-30 s backoff to the same
room and key (announcing a new room only if the relay answers 404/410), and sends `bye` on Ctrl-C.

The desktop app embeds the server with `legatum_server::embedded::start(EmbedConfig)`: its own
Tokio runtime thread, `127.0.0.1` on a free port unless `bind_lan`, the UI from the `assets`
callback (`/` is `index.html`, unknown paths 404), and the relay room when configured.
`Handle::stop()` sends `bye`, closes the listener and joins the thread.

## Network trust

The server listens on all interfaces and uses plain HTTP and WebSocket. Seat tokens travel in
the clear, so use it only on networks you trust, such as a home LAN, not public Wi-Fi. Scenario
run uploads remain available to LAN players without a seat (a scenario is a single-player game in
one browser), with a 2 MiB limit per request; the reply names the file relative to the saves
directory. Every response carries `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`
(and `frame-ancestors 'none'`) and `Referrer-Policy: no-referrer`. The server keeps
at most 500 scenario-run files and 100 MiB in `<saves>/scenarios/`, deleting the oldest files
by modification time before a new run is written.

Choose Multiplayer → Host Game to configure a staging room. Guests take open
Human seats, choose a name/faction, and mark Ready. The host starts after every
open seat is connected and Ready. Settings changes reset readiness. The room
supports 1-8 humans plus AI seats (2-8 total players), and is enabled by default.
The direct Host Game path remains available with `?staging=0`.

Integration details:

- `turn.turn` is the **new planning turn**, matching the following `view.turn`;
  the first resolution sends turn 2. `hash` and `game_id` are strings.
- `POST /api/game` accepts `GameSetup` with numeric or decimal-string seeds.
  Humans occupy the first 1-8 seats, followed by any AIs.
  `POST /api/load` accepts the raw save text (also a JSON-encoded string) and
  returns status. Read `GET /api/save` with `response.text()` and preserve it
  exactly; JSON numbers inside it may exceed JavaScript's integer precision.
- Admin routes require a loopback TCP peer; forwarded headers cannot grant access.
  Admin routes, `/api/status`, and `/ws` additionally check Host and any supplied Origin against
  localhost, 127.0.0.1, and the printed LAN URLs at the actual listener port.
  POSTs require `Content-Type: application/json`. Use the UI served by this server;
  unrelated browser origins and DNS names resolving to the server are rejected.
- Every human must join once after creating/loading/restarting. A disconnected
  seat then stops blocking its connected teammate; all connected seats must be
  ready, and at least one must be connected. Pending orders survive reconnects until resolution. Rejoining
  clears Ready, and the newest connection with the correct credential exclusively owns the seat. `welcome.orders`
  returns the retained list so the browser can show and cancel it before Ready. Credentials
  survive browser disconnects/reloads. Server restarts lock restored seats until a loopback
  host explicitly reassigns them; credentials are not included in saves.
- Creating/loading a game closes existing sockets with an error so clients
  rejoin. Views, order validation, and forecasts are always tied to the socket's
  seat. Unjoined sockets receive only public seat broadcasts and errors/pongs.
- `POST /api/staging`, `POST /api/staging/start`, and `DELETE /api/staging`
  manage the in-memory room using the same admin guards. Room snapshots use
  a separate coalesced, bounded queue. Starting persists the game before
  atomically transferring seat credentials and handing occupants their new human seat IDs;
  a failed save preserves the room. Guest snapshots omit the generation seed. The loopback
  host explicitly subscribes to private settings; `/api/status` is always public and cached.
- WebSocket admission caps each remote IP at eight connections, reserves four of the 32
  global slots for loopback, and limits remote unjoined/watchers to 16. Unjoined sockets
  expire after 30 seconds even when they answer heartbeats.
- Native work and disk writes run on a blocking worker while holding the single
  game mutex. The existing WASM crate's native `Session` supplies the game loop,
  AI seeding, and per-AI views; the server does not implement game rules.

Checks:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`crates/server/tests/protocol.rs` starts real ephemeral HTTP/WebSocket servers
and checks 1-8 human seats, staging handoff, private views, native AI determinism, readiness,
takeover, reconnects, save/load, restart, and snapshot retention. Unit tests cover
the loopback guard, stale sockets, bounded staging broadcasts, and visibility filtering.

The browser suite starts the real Rust binary with temporary saves and uses two
isolated browser contexts. It covers reload reclamation, a held settings request
(the Start debounce race), occupied-row removal, start, and the first turn:

```sh
scripts/build-wasm.sh
cargo build --release -p legatum-server
cd app
npm ci
npx playwright install chromium
npm test && npm run build
npx playwright test
```

The fixture runs `target/release/legatum-server` (or `LEGATUM_SERVER_BIN`).
`npm run test:e2e` builds the release server and runs this suite against the
existing UI build. `LEGATUM_E2E_BROWSER=firefox` or `webkit` selects another installed engine.

# Online play: lobby, relay and the in-tab host

Sam, 2026-11-21: two players should click **Multiplayer**, see hosted games in a lobby, click one and play. No
addresses, no links, no Terminal, and no dependence on the LAN or a firewall. 2026-11-22: the game is a web app
(the desktop app was dropped), so the host's game runs **in the host's browser tab**, in the WASM engine; the
cloud only lists games and passes messages. There is no game server program anymore.

This file is the contract between the parts. `plans/lan-protocol.md` stays the contract for the game protocol
itself (the messages between a player's UI and the host), which did not change.

## Shape

```
guest tab                 --wss-->  Cloudflare Worker (infra/relay)  <--wss (dials out)--  host tab
  UI + RemoteClient                  Lobby DO: list of rooms                                 UI + BrowserHost (WASM):
  (app/src/remote.ts)                Room DO: one per hosted game, dumb pipe                 authoritative game, AI, save
```

- **Host Game** (`store.hostOnline`, `app/src/net/hosting.ts`) loads the engine's `BrowserHost`
  (`crates/wasm/src/host`: the old server's host, staging, protocol and admission logic, with no I/O),
  announces the room on the relay and holds one tunnel WebSocket (`app/src/net/browserHost.ts`). Every guest
  connection is multiplexed over it and is a non-local connection to the engine.
- The host's own UI talks to the same engine through an in-tab socket and in-tab status/admin calls
  (`LoopbackSocket` and `BrowserHost.fetch`, installed with `setLocalEndpoint` in `remote.ts`). It is the one
  `local` connection, so `RemoteClient`, the staging room and the host-only admin rules are unchanged.
- The guest points `RemoteClient` at the relay's room URL.
- The relay never parses game messages, holds no game state and makes no rule decisions.
- **Limits**: the game lives in the host's tab. Closing that tab (or its browser) ends the game for everyone
  (the relay drops the room after 120 s, or at once on `bye` when the host leaves through the menu). A reload of
  the host tab resumes the game in the same room from the save kept in `sessionStorage`; a reload of a guest
  tab rejoins its room and seat. A new tab cannot take over a hosted game. To outlive the tab, the host
  saves to a file (game menu, host only) and later hosts from it (see "Save to file and host from a save"). A WASM panic (a host bug) stops the
  game: Rust's `catch_unwind` does not work in WASM.

## Relay HTTP and WebSocket API (Worker, `infra/relay/`)

Base URL `RELAY` (e.g. `https://legatum-relay.<account>.workers.dev`). All responses carry
`Access-Control-Allow-Origin: *` and `Cache-Control: no-store`; `OPTIONS` preflight is answered.

| Method | Path | Meaning |
| --- | --- | --- |
| GET | `/lobby` | `{ "rooms": [RoomInfo] }`, newest first, only rooms whose host tunnel is connected |
| POST | `/rooms` | Body `RoomAnnounce`. Creates a room. Response `{ "room": "<id>", "host_key": "<secret>" }` |
| GET (WS) | `/rooms/<id>/host?key=<host_key>` | The host tunnel. One at a time; a new one with the right key replaces the old |
| GET | `/rooms/<id>/api/status` | The host's last pushed `Status` JSON with `is_local` forced to `false`; 404 when the room is gone, 503 when the host is not connected |
| GET (WS) | `/rooms/<id>/ws` | A guest connection, piped to the host. Closed with code 1013 when the host tunnel is absent |

```
RoomAnnounce = { name: string (<= 80 chars, the host's display name),
                 build: string (<= 40, the app's build label, e.g. "8.9.74"),
                 rules_version: number, protocol: number }
RoomInfo     = RoomAnnounce & { room: string, players: number, humans: number, open_seats: number,
                 phase: "staging" | "playing", turn: number, updated_ms: number }
```

`protocol` is `ONLINE_PROTOCOL = 1`, bumped on any incompatible change to this file or to the game protocol.
`players/humans/open_seats/phase/turn` come from the host's `summary` control frames (below); the relay stores
them as opaque numbers and strings.

Limits (relay): at most 64 rooms, 8 guest sockets per room, 64 KiB per guest-to-host message, 1 MiB per
host-to-guest message, room id 10+ and host key 32+ random URL-safe characters. A room is deleted 120 s after
its host tunnel drops without returning, and immediately on a `bye` frame. Use the WebSocket Hibernation API.
Since the relay hardening (2026-11-22) it also caps rooms and creations per client address, guest sockets per address,
message rates (close 4029), cleans room names and has a `LOBBY_OPEN` kill switch; `infra/relay/README.md` ("Limits and
kill switch") holds the numbers. Security review (2026-11-22): an IPv6 client counts as its /64 for every per-address limit,
the `build` label takes only letters, digits, spaces and `. · ( ) _ + -`, a host `status` frame over 64 KiB is dropped, and
every relay answer carries `X-Content-Type-Options: nosniff`.

### Limits

What the relay enforces (numbers from `infra/relay/src/shared.ts`; `infra/relay/README.md` has the tunables) and what
the player sees in the app. Player text never says "relay", "socket" or an HTTP code.

| Limit | Relay answer | The host or guest sees |
| --- | --- | --- |
| 6 room creations per 10 minutes per address | 429 with `Retry-After` (exposed to browsers by `Access-Control-Expose-Headers`; the body says "too quickly" too) | Host: "You have opened too many games lately. Try again in N minutes." (N from `Retry-After`, rounded up, at least 1; "Try again in a few minutes." when the header cannot be read). Host Game says "Try again in N minutes" and is not the primary; Join Game is. |
| 2 live rooms per address | 429, no `Retry-After` | Host: "Your internet connection already has 2 games open. Close one before hosting another." (the 2 mirrors `MAX_ROOMS_PER_IP` as `MAX_ROOMS_PER_ADDRESS`). Host Game says "Close one of your games first" and is not the primary. |
| 64 rooms in all | 503 | Host: "The game list is full right now. Try again in a few minutes." Host Game says "Try again" and stays the primary. |
| `LOBBY_OPEN=0` kill switch | 503 ("paused") | Host: "Hosting new games is paused. Try again later." Host Game says "Paused for now" and is not the primary. |
| Relay unreachable | no answer | Host: "Could not open your game. Check your internet connection and try again." |
| 4 guest sockets per address per room | WebSocket close 1013, reason "too many connections from one address" | Guest: "Too many people on your internet connection have joined this game. Close any extra windows, then pick your seat again." The app stops retrying. Close 1013 for "host not connected" or "room is full" reconnects as usual. |
| Per-socket message and byte budgets | WebSocket close 4029 | Guest: "You are sending too much, too fast. Pausing for 30 seconds, then reconnecting." One retry; a second 4029 before any game state arrives stops it with "You are still sending too much, too fast. Wait a minute, then pick your seat again." The host's own tunnel just reconnects. |
| Room names cut to 40 visible characters, symbols and emoji dropped | name cleaned, "Game XXXX" when empty | Host: the name box holds 40 characters and says "Shown in the game list as “X”." when the list will show something else, or "The game list will show “Game” and four characters." when nothing showable is left (the Worker names an empty game "Game" and the first four characters of its id). The note replaces the label above the box, so it adds no height (`app/src/net/roomName.ts` mirrors the Worker's rule, minus its word blocklist) |
| 60 s host heartbeat | room leaves the list | The app's tunnel sends a ping every 20 s, so nothing shows; a host whose tab sleeps drops off the list |

When the relay forgets a room while the host's tunnel is down, the host announces a new room; if that is refused, the
refusal above replaces the plain "Reconnecting…" note and the host keeps trying.

### Tunnel frames (host <-> relay), binary

`[conn: u32 big-endian][kind: u8][payload: bytes]`

| kind | direction | payload | meaning |
| --- | --- | --- | --- |
| 0 open | relay -> host | empty | guest socket `conn` connected (`conn` >= 1, never reused within a tunnel) |
| 1 data | both | UTF-8 text | one text message from/to guest `conn`, forwarded verbatim |
| 2 close | both | empty | guest `conn` closed / host asks the relay to close it |
| 3 status | host -> relay | UTF-8 JSON `Status` | `conn` = 0. Replace the cached `/api/status` body |
| 4 summary | host -> relay | UTF-8 JSON `{players, humans, open_seats, phase, turn}` | `conn` = 0. Lobby row |
| 5 bye | host -> relay | empty | `conn` = 0. Host is closing the room for good |
| 6 ping | host -> relay | empty | `conn` = 0. Keepalive; ignored by the relay |

When a new tunnel replaces an old one, or the tunnel drops, the relay closes every guest socket (1012); guests
reconnect by themselves. A browser cannot send WebSocket pings, so the host sends a `ping` frame (kind 6,
`conn` = 0, empty) every 20 s instead; the relay ignores unknown kinds, and the traffic keeps the tunnel alive.

### Large messages (host -> guest), inside `data` payloads

A `view` can exceed what one relayed message may carry. The host splits any guest-bound text longer than
`CHUNK_BYTES = 256 * 1024` UTF-8 bytes (split on character boundaries) into text messages

```
{"type":"chunk","id":<u32, per connection, increasing>,"i":<0-based index>,"n":<count>,"data":"<piece>"}
```

sent in order with nothing from the same connection interleaved. The guest's relay socket wrapper
(`app/src/relaySocket.ts`) concatenates the pieces and delivers the original text to `RemoteClient` as one
message; anything else passes through untouched. Guest-to-host messages are never chunked (64 KiB cap exists
already). The host's own in-tab connection never sees chunks.

## Host (`crates/wasm/src/host`, `app/src/net`)

- `NetHost` (Rust, no I/O) drives the host: `connect(local, now)`, `receive`, `disconnect`, `tick(now)` and
  `drain(now)`, which returns every connection's queued texts and closes. It keeps the old server's heartbeats
  and admission: a guest must join within 30 s (pings do not extend that), a seated guest silent for 120 s is
  closed (background tabs throttle timers, so this is longer than the old 60 s), at most 8 guests. The local
  connection is never timed out. `BrowserHost` is its wasm-bindgen wrapper; `online_rules_version()` exports
  the rules version.
- `app/src/net/browserHost.ts` is the transport: `POST RELAY/rooms`, the tunnel at
  `RELAY/rooms/<id>/host?key=`, the frame codec and chunking (`tunnel.ts`), a 1 s timer that ticks the engine,
  `status` and `summary` pushed when they change, the 20 s `ping` frame, reconnect with backoff (1 s to 30 s)
  keeping the room and key, and `bye` on leave. A browser cannot see why a WebSocket upgrade failed, so when a
  reconnect never opens, it asks `GET RELAY/rooms/<id>/api/status`: 404 means the relay forgot the room, and it
  announces a new one.
- Saves: the engine hands over the live game's save whenever it changes; `hosting.ts` keeps it (and the room
  key) in `sessionStorage` so a reload resumes. Game histories go to the browser's archive (`recordGameHistory`)
  like a local game's. Resumed seats get fresh credentials: the host reclaims its seat, guests take theirs again.
- Save to file and host from a save: the host's game menu offers Save to file, which writes the engine's current
  save (`BrowserHost.save_game`) in the normal `legatum-save` envelope with `online: true`. The engine save is the
  same format as a solo save, so Load from file opens it as a solo game too. Title > Multiplayer > Host from a
  save picks a file and calls `hostOnlineFromSave`, which starts a new room with `new BrowserHost(save)`
  (`startHosting`'s `fromSave`; an unreadable chosen save is an error, never a fresh game). Seat names carry
  over. Seat credentials are not saved. `open_guest_seats` opens every seat but seat 0 (the host's, which
  `hostOnlineFromSave` takes at once), so guests join from the list and take their old seat with its saved name. A new room has a new id and key.
- Seat credentials are 256-bit random hex from `crypto.getRandomValues`.

## Build (`scripts/build-web.sh`)

- `VITE_LEGATUM_RELAY_URL` is baked in: the live relay by default, `LEGATUM_RELAY_URL` overrides it (`none` for a
  build without online play). The script writes the relay's https and wss origins into `dist/_headers`'s
  `connect-src`.
- The rules version comes from `crates/core/src/lib.rs` at build time (`vite.config.ts`), so the title screen
  knows it without loading the engine; a unit test checks it against the engine.

## UI (`app/src`)

- **Multiplayer > Host Game**: asks for the name shown in the game list, starts the in-tab host and lands in the
  existing staging room. In a running game the host's menu item is **End the game for everyone** (after a confirm);
  it sends `bye`, closes the room and returns to the title. A guest's item is **Leave the game (free this seat)**.
- **Multiplayer > Join Game**: a lobby list from `GET /lobby` (refresh every 3 s): host name,
  players, open seats, "Waiting in lobby" or "Turn N". One primary action per row: **Join**. A row whose
  `build` differs from this build's (both known) shows "This game is on version X and you have Y. Update the
  older one to play together."; one whose `rules_version` or `protocol` differs, or whose build is unknown, shows
  "This game is on a different version from yours. Update both to the newest to play together." The note comes first
  in the row with an alert mark; neither can be joined. Empty state: "No games are being hosted right now."
- Joining sets the server base to `RELAY/rooms/<id>`: status from `<base>/api/status`, socket at `<base>/ws`
  through `relaySocket.ts`. Everything after that is the existing staging room and game.
- A build without a relay offers hot seat only.

## Tests

- Rust: the host's unit tests and `crates/wasm/tests/net_host.rs` (the old server protocol tests, driven synchronously).
- Vitest: `app/src/net/tunnel.test.ts`, `browserHost.test.ts` (fake engine and tunnel: routing, chunking, pings,
  backoff, re-announce, `bye`, the in-tab socket and admin calls).
- Playwright: `app/e2e/game.spec.ts` "Online: ..." plays host and guest in two browser contexts through the real
  relay under `wrangler dev` (the `relay` fixture), on the static build from
  `LEGATUM_RELAY_URL=http://127.0.0.1:8787 scripts/build-web.sh`.

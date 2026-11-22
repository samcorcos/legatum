# Online play: lobby and relay

Sam, 2026-11-21: two players should click **Multiplayer**, see hosted games in a lobby, click one and play. No
addresses, no links, no Terminal, no browser, and no dependence on the LAN or the Mac's firewall. The game still
runs on the host's own Mac; the cloud only lists games and passes messages.

This file is the contract between the four parts. `plans/lan-protocol.md` stays the contract for the game
protocol itself, which does not change.

## Shape

```
guest app (tauri://)  --wss-->  Cloudflare Worker  <--wss (dials out)--  host app (embedded legatum-server)
  UI + RemoteClient              Lobby DO: list of rooms                   authoritative game, AI, saves, archive
                                 Room DO: one per hosted game, dumb pipe   host's own UI talks to it on 127.0.0.1
```

- The host's Legatum app starts `legatum-server` **inside the app process**, bound to `127.0.0.1` on a free
  port (nothing listens on the LAN). The host's window loads the UI from that server
  (`http://127.0.0.1:<port>/`), so the existing same-origin server mode and loopback admin rules apply unchanged.
- The embedded server dials **out** to the relay and holds one tunnel WebSocket. Every guest connection is
  multiplexed over it. To the server, a relayed guest is an ordinary **non-loopback** connection.
- The guest's app stays on its bundled UI and points `RemoteClient` at the relay's room URL.
- The relay never parses game messages, holds no game state and makes no rule decisions.

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

When a new tunnel replaces an old one, or the tunnel drops, the relay closes every guest socket (1012); guests
reconnect by themselves as they do on the LAN. The host sends WebSocket pings every 20 s; the relay answers.

### Large messages (host -> guest), inside `data` payloads

A `view` can exceed what one relayed message may carry. The host splits any guest-bound text longer than
`CHUNK_BYTES = 256 * 1024` UTF-8 bytes (split on character boundaries) into text messages

```
{"type":"chunk","id":<u32, per connection, increasing>,"i":<0-based index>,"n":<count>,"data":"<piece>"}
```

sent in order with nothing from the same connection interleaved. The guest's relay socket wrapper
(`app/src/relaySocket.ts`) concatenates the pieces and delivers the original text to `RemoteClient` as one
message; anything else passes through untouched. Guest-to-host messages are never chunked (64 KiB cap exists
already). LAN connections never see chunks.

## Server (`crates/server`)

- `connection()` becomes generic over the transport, so a relayed guest runs the same loop, heartbeats,
  outbox and admission as a LAN socket, with `local = false`. All relayed guests share one admission "peer".
- `relay` module: `Server::run_relay(RelayConfig { url, announce: RoomAnnounce })` announces the room, opens the
  tunnel, pushes `status` and `summary` whenever the cached status changes, reconnects with exponential backoff
  (1 s to 30 s) keeping the same room and host key, and sends `bye` on shutdown.
- `embedded` API for the app: `legatum_server::embedded::start(EmbedConfig) -> Result<Handle, String>` where
  `EmbedConfig { saves_dir, archive_dir, assets: Arc<dyn Fn(&str) -> Option<(Vec<u8>, String /*mime*/)> + Send + Sync>,
  relay: Option<RelayConfig>, bind_lan: bool }` and `Handle { port: u16, url: String, room: Option<String> }` with
  `Handle::stop()`. It runs on its own Tokio runtime thread. `bind_lan = false` binds `127.0.0.1` only.
- The CLI gains `--relay <url> --name <host name> --build <label>` (so the relay can be tested without the app)
  and keeps working exactly as today without them.

## App shell (`app/src-tauri`)

- Commands: `online_host_start({ name }) -> { url, room }` (starts the embedded server with the relay from the
  build-time `LEGATUM_RELAY_URL`, then the UI navigates the window to `url`), `online_host_stop()`,
  `online_info() -> { relay: string | null, build: string, rules_version: number, protocol: number }`.
- The host window on `http://127.0.0.1:<port>` must still be able to return to the bundled UI (Leave game).
- CSP `connect-src` gains the relay's https and wss origins.

## UI (`app/src`)

- **Multiplayer > Host Game** in the desktop app: calls `online_host_start`, lands in the existing staging room.
- **Multiplayer > Join Game** in the desktop app: a lobby list from `GET /lobby` (refresh every 3 s): host name,
  players, open seats, "Waiting in lobby" or "Turn N". One primary action per row: **Join**. A row whose
  `build` differs from `online_info()` (both known) shows "This game is on version X and you have Y. Update the
  older one to play together."; one whose `rules_version` or `protocol` differs, or whose build is unknown, shows
  "This game is on a different version from yours. Update both to the newest to play together." The note comes first
  in the row with an alert mark; neither can be joined. Empty state: "No games are being hosted right now."
- Joining sets the server base to `RELAY/rooms/<id>`: status from `<base>/api/status`, socket at `<base>/ws`
  through `relaySocket.ts`. Everything after that is the existing staging room and game.
- In a browser served by `legatum-server` (LAN mode) nothing changes.

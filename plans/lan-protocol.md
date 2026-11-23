# Online play: the game protocol

The messages between a player's UI and the authoritative game, for **two or more humans on one team plus AI, each entering their own orders**. (Single player runs locally in WASM.) The file keeps its old name; since 2026-11-22 there is no LAN server and no native program. `plans/online-play.md` is the contract for where the game runs and how guests reach it; this file is the protocol on top.

## Shape

- **The host's browser tab hosts.** `BrowserHost` (`app/src/net/browserHost.ts`) drives the engine's `NetHost` (`crates/wasm/src/host/`, the old `legatum-server`'s host, staging, protocol and admission logic with no I/O) with `legatum-core` + `legatum-ai` as WASM, and tunnels guest connections through the Cloudflare relay (`infra/relay`). Closing the host's tab ends the game.
- **Everyone plays in a browser** on the web build. Nothing to install, no address to type: guests pick the game from the relay's lobby.
- **Each browser is bound to one seat.** A client only ever receives its own seat's `PlayerView` (fog and privacy stay in the host, like the AI).
- The turn resolves when at least one living human seat is connected and every connected living human seat is Ready. New or explicitly loaded games also wait for each living human seat to join once; a resumed game skips that initial join barrier. Eliminated players never block turns. AI orders are computed in the host from each AI's own view.
- **"Loopback" and `is_local` below mean the host's own tab** (the in-tab connection, `connect(local = true)`); every guest through the relay is a non-local connection. There are no TCP peers, `Host`/`Origin` checks, per-IP limits or `--static`/`--saves` flags any more; the relay carries up to 8 guests per room, 64 KiB per guest message (`net.rs::MAX_GUESTS`, `MAX_GUEST_MESSAGE`). Texts over 256 KiB reach guests as `chunk` messages (`app/src/net/tunnel.ts`, reassembled by `app/src/relaySocket.ts`).
- **Saving:** the host's tab keeps the latest save in `sessionStorage` (`app/src/net/hosting.ts`), so reloading that tab resumes the same room. There is no autosave directory and no `turn-<n>.json`.
- Imports and resumes reject science histories longer than `SCIENCE_HISTORY_TURNS` (5), tile memories longer than the map, and explored bitsets whose word count does not match their declared map-sized length (`check_state`, `crates/wasm/src/host/mod.rs`). Shorter/omitted tile memories remain compatible with older saves.

## Host admin calls

There is no HTTP server. The host's own UI reaches the engine through `BrowserHost.fetch`, which answers these paths in the tab (`app/src/net/browserHost.ts`); a guest's `GET <room>/api/status` is answered by the relay from the host's last pushed status, with `is_local` forced to `false`.

| Method | Path | Body / response |
| --- | --- | --- |
| GET | `/api/status` | `{ "mode": "server", "has_game": bool, "turn": n, "seats": [SeatStatus], "is_local": bool, "staging": StagingState | null }` (`is_local` = the host's own tab). `lan_urls` is gone. |
| POST | `/api/game` | Body: `GameSetup` JSON (seed as decimal string or number). Creates a new game. Host's tab only. Response: the status payload. |
| POST | `/api/load` | Body: a save string. Host's tab only. |
| POST / DELETE | `/api/staging`, POST `/api/staging/start` | Open, configure or close the staging room; start the game (see Staging room below). Host's tab only. |

`/api/save` and `/api/archive` no longer exist; the app saves from the engine's own save call. The status never carries the seed or any seat credential.


## WebSocket `/ws`

JSON text frames. Every message has `"type"`.

Admission (`NetHost::connect`): at most 32 connections including unjoined ones, at most 8 of them guests. The per-IP and HTTP 503 rules of the old server no longer apply (all guests arrive through the relay). Excess connections are refused with "The game is full".

Unjoined sockets, including `staging_watch` subscribers, must occupy a staging seat or join a game within 30 seconds. Ping, Pong, application messages and repeated watches never extend that deadline. Losing/leaving a seat starts a new 30-second deadline. Joined sockets retain the usual 60-second heartbeat timeout. Each connection has a reliable queue of at most 16 deliveries and an 8 MiB serialized byte budget, including an in-flight delivery. Welcome/view and turn/view pairs each form one delivery and stay adjacent on the wire. Seat broadcasts and staging broadcasts each use one replaceable latest snapshot per connection, with identical updates suppressed; intermediate states can be skipped. Staging snapshots are bounded by eight slots, 80-character names and a 20-digit seed. Closing or starting a room clears pending staging snapshots before sending the reliable terminal message. Seat names are capped at 80 characters, including names from saves.

Reliable queue overflow evicts the connection, clears its seat's Ready flag, and signals `error` with `code: "slow_consumer"` followed by close on a best-effort basis. The socket task can cancel a stalled write on this separate signal; no network wait occurs under the game mutex. Pending orders survive eviction as with other disconnects. Heartbeat deadlines take precedence over queued deliveries. Both incoming messages and individual frames are limited to 64 KiB.

Client → server:

| type | fields | meaning |
| --- | --- | --- |
| `game_take` | `seat`, `code?: string` (join code, at most 32 characters) | Claim an unassigned seat (a direct-game seat or one opened by the host); replies privately with `seat_token`. Seat 0 requires loopback. If already reserved, only a loopback host action can reassign it; reassignment rotates its token, clears pending orders/Ready, and closes its previous socket with `seat_taken`. Staging-created seats are already reserved. On a game with a join code (`plans/online-play.md`), a non-loopback claim without the right code gets `code: "join_code_needed"` or `"wrong_join_code"` ("That join code is not right."); the fifth wrong code from one connection closes it with `code: "too_many_codes"`. A non-loopback connection that already holds or has claimed another seat is refused ("You already have a seat in this game."). |
| `game_remove` | `seat` | Loopback only. Take a remote player out of a seat: closes its connection with `code: "removed"`, replaces its credential with one nobody holds, clears its orders/Ready and leaves the seat reserved (closed), not open. The host's own seat, an unreserved seat and a seat held by the loopback connection are refused. A removed seat has no connection, so it does not hold up turn resolution; `game_release` opens it. |
| `game_release` | `seat` | Loopback only. Open a disconnected human seat for the next claim, invalidating its credential and clearing pending orders/Ready. Leaves it unassigned (`reserved: false`, `connected: false`), preserves its empire/name and the host's own seat, and broadcasts `seats`. Invalid indices and connected seats are rejected without changes; the connection check and release are atomic. No credential is issued to the host. |
| `join` | `seat` (0 through N−1), `name`, `token: string` | Join a human seat using its credential. Missing/wrong tokens fail without displacing the owner or revealing orders/view; a wrong token returns `code: "invalid_seat_credential"`. The same credential can rejoin after disconnect or displace a stale connection (`seat_taken`), preserving the existing takeover UX. Names are cleaned (control and format characters such as bidi overrides and zero-width marks removed, whitespace collapsed, at most 80 characters) before they are stored in the seat, the saved history and staging; a name that cleans to nothing keeps the seat's current name. A repeat `join` on the connection that already joined this seat in the same planning turn gets no reply, and the host saves only when the name changed. |
| `orders` | `game_id: string`, `turn: number`, `orders: Order[]` | Replace this seat's pending orders for the specified planning turn, including an empty array to cancel all. Server validates and replies `validation`. Setting orders clears Ready. At most 2,000 orders per seat per turn: a longer list is not stored and gets one `validation` error. A `validation` reply lists at most 100 errors. |
| `ready` | `game_id: string`, `turn: number`, `ready: bool` | Mark this seat ready / not ready for the specified planning turn. |
| `forecast` | `req_id`, `unit`, `target: Hex`, `mode: "Melee" \| "Ranged"` | Battle forecast from this seat's view for the intended order: `MoveUnit` uses Melee, `RangedAttack` uses Ranged. |
| `ping` | — | keepalive |
| `map_ping` | `hex: Hex` | Team map ping (2026-11-22). Relayed as `map_ping` to the other connected seats on the sender's team only; never stored in the game, a save or a tape (no `RULES_VERSION` change). Send it only when `welcome` said `map_pings: true`: a host from before pings answers an unknown type with an `error`. A tile off the map, or a connection with no seat, gets an `error`; a seat's 25th ping in one planning turn (limit 24) and any ping once the game is finished are dropped without a reply. Pings use their own queue of the latest 8 per connection, outside the reliable queue, so a slow reader loses old pings and is not evicted. |

Server → client:

| type | fields | when |
| --- | --- | --- |
| `seat_token` | `seat`, `game_id`, `token` | private acknowledgment of `game_take`; the client stores it before sending `join` |
| `welcome` | `seat`, `game_id`, `orders: Order[]`, `map_pings?: true` (omitted when false) | after every successful `join`; includes this seat's authoritative pending orders, even when empty; `map_pings` says the host relays `map_ping` |
| `view` | `view: PlayerView` | after join and after every turn |
| `seats` | `seats: SeatStatus[]` | latest seat state after joins, leaves, orders/ready changes, or resolution (coalesced broadcast) |
| `validation` | `errors: OrderError[]` | after `orders` |
| `turn` | `turn`, `events: Event[]` (filtered to what this seat's team could see), `hash` (decimal string), `finished`, `winner` | after each resolution, before the new `view` |
| `forecast` | `req_id`, `forecast: BattleForecast \| null` | reply |
| `error` | `message: string`, optional `code: string` | bad request, seat taken, no game, etc. |
| `pong` | — | reply to ping |
| `map_ping` | `from: PlayerId`, `hex: Hex`, `turn` | a teammate pinged a tile in planning turn `turn`; the client shows it only while its own view is on that turn, and drops it when the turn changes. An older client ignores the type. |

A guest's client stops retrying by itself after an `error` with `code` `join_code_needed`, `wrong_join_code`, `too_many_codes` or `removed` (the player acts first; `removed` also forgets the seat's credential).

On takeover, the displaced socket receives `{"type":"error","code":"seat_taken","message":"Your seat was claimed by another connection."}` before it is closed. Only `code: "seat_taken"` stops automatic reconnects; this applies while connecting or reconnecting too, even before `welcome` or the first complete `view`. The player must explicitly rejoin to reclaim the seat. Other errors have a different code or omit it, and a subsequent disconnect still reconnects automatically (including after the host replaces the game). Clients must not infer takeover from the human-readable message or the WebSocket close code.

`orders` and `ready` require the `game_id` from `welcome` and the planning `turn` from the last complete `view`. Both must match the live game under the same lock as mutation. Missing preconditions, stale or future turns, and different game IDs are rejected without changing pending orders or readiness. A `turn` report identifies the new planning turn; the client waits for its following view before sending edits or Ready. The client stores `TurnResult.turn` as the wire `turn - 1`, the turn just resolved, matching local WASM reports; the following `PlayerView.turn` remains the new planning turn. Delayed frames retain their original preconditions and cannot become commands for the next turn. This is a protocol change: update the client and server together.

The server retains pending orders across disconnects and authenticated takeovers until resolution, and resets Ready on every join. Explicit host release/reassignment clears those orders. `welcome` is immediately followed by the seat's current `view`, produced under the same game lock. The client restores the welcome orders into its cache and the visible pending-order list before enabling Ready. It does not replay previously sent orders. Unsent edits in the same browser may replace the snapshot only after the view confirms the same `game_id` and turn; changed games/turns discard those drafts. A fresh browser/reload always uses the server list, so retained orders such as `DisbandUnit` remain visible and can be cancelled. A cancellation sends `orders: []` even when it leaves no local orders. Server restarts restore the game state, with empty pending orders. Credentials live only in server memory and per-tab sessionStorage, never in localStorage, public snapshots or exported saves. Restarted seats are locked with fresh credentials until explicitly opened or reassigned by the host. Direct new/load actions open unassigned seats for first claim (seat 0 remains host-only); they invalidate old credentials. An invalid stored credential never silently falls back to taking a reserved seat. Status advertising `reserved: false` lets an explicit join bootstrap an unassigned seat instead of reusing an older credential; reload never automatically claims an opened seat.

Readiness and the first-join barrier apply only to living players. A fresh or explicitly loaded game waits for each living human seat to join once, then disconnected seats do not block. Attendance is not saved, so startup restoration treats every seat as already joined rather than requiring absent players to return after a restart. A restored living seat blocks only while connected and not Ready; joining resets Ready as usual. Releasing a seat preserves this policy. Eliminated seats never need to join or become Ready, though a connected eliminated player still receives turn reports and views. At least one living human must be connected to resolve a turn, so eliminated spectators cannot advance an unattended game. These rules preserve play with departed teammates without changing credential requirements.

Client-facing `MoveUnit` validation uses the seat's `PlayerView`: unknown destinations are allowed, and paths to known destinations treat unknown intermediate tiles as potentially passable. Known impassable destinations or known disconnections can be rejected. Resolution uses the actual map, but a failed move reports only a generic `OrderFailed` reason, without hidden terrain or connectivity details.

Orders targeting absent units/cities and foreign units/cities return byte-identical generic ownership errors (`No such unit of yours` / `No such city of yours`), including hidden entities. Both validation and resolution check ownership before reporting entity details; guessed IDs never reveal existence or a hidden city's name.

`SeatStatus = { seat, player: PlayerId, name, faction, connected: bool, reserved: bool, ready: bool, orders: number }`. `reserved` reveals only whether a credential exists, never its value. Guests without that credential cannot take a reserved seat from the Join screen.

Events filtering uses the union of pre- and post-turn visible tiles. Team events and battles involving the team remain available. Enemy movement requires at least one visible endpoint; hidden `UnitMoved.from`/`to` and `MoveBlocked.at`/`toward` endpoints are `null` in LAN reports. A partly hidden block gets a generic reason. Steps and encounters entirely behind fog are omitted even when an actor was visible earlier; a visible city does not disclose hidden tile claims. Locationless enemy disband/upgrade events are omitted because their visibility at the event cannot be established. Stationary city events require a visible city, and private research/failed orders remain team-only. Global events (titles, VP, game won, megaproject announcements) go to everyone. Core event logs remain complete; the app formats fog crossings explicitly and omits animation paths containing hidden segments.

## UI (app/)

- `RemoteClient` implements `GameClient` over the protocol. `view(p)` returns the cached view from the last `view` message; `endTurn` is replaced by `ready()`; the client listens for `turn` + `view`.
- On load, the UI calls `/api/status`. If it responds, the UI is in **server mode**:
  - Title screen: **Multiplayer → Host Game** (`is_local` only) opens the staging room described below. **Join Game** opens an available room, or lists the human seats of the running game when no room is open. Servers without staging support, or an explicit staging opt-out, use the direct `POST /api/game` flow; it accepts any valid lobby-built setup with 1-8 humans first.
  - After joining: the normal game screen for that one seat. No seat switcher. The seat panel lists every human seat (connected, orders count, ready). Ready = send `ready`; the button shows "Waiting for <names>…" until the turn resolves, omitting eliminated players from the waiting names.
  - Only the loopback host (`status.is_local`) sees **Open seat** on other disconnected reserved human seats. Confirmation asks **Let another device take over <name>'s empire?** and sends `game_release` on the existing socket. The UI waits for the server's seat snapshot before showing the seat as open. The action is disabled while reconnecting and is never queued for replay. A guest sees **Reserved** and **Ask the host to open this seat**; after release, the Join screen polls the open status and offers **Rejoin as <name>**.
  - The host screen shows no address to share: the game appears in the relay's lobby (`plans/online-play.md`).
  - Reconnect automatically on disconnect (exponential backoff) and re-join the same seat (remember its credential and seat in per-tab sessionStorage), unless the socket received `error` with `code: "seat_taken"`. Reload auto-rejoins a reserved seat with that tab's credential even if status still sees its previous socket connected; credential authentication permits replacing the stale socket. A new browser context has no credential. Ready stays disabled until welcome orders and the corresponding view are restored. Leaving a running game keeps its credential with `autoJoin: false`, so a later reload stays at the title and an explicit rejoin can still reclaim the reservation.
  - A join rejected with `invalid_seat_credential` offers **Reclaim my seat** only to the loopback host (`status.is_local`), on the Join screen or in the running game's connection banner. This deliberate action sends one `game_take`, then saves the returned `seat_token` before joining with it. It lets a remembered host tab recover after a restart rotates credentials. Ordinary Join and automatic reconnect keep using the saved credential; reconnect never sends `game_take`, even if a claim's acknowledgment was lost. An interrupted claim without any credential requires another explicit Join. A guest must ask the host to open the seat. The server still enforces loopback reassignment.
  - Starting or loading a local game, leaving, or going Back to title from a finished online game disconnects the online session; a stale socket's messages are ignored.
- If `/api/status` fails, Single Player stays available through local WASM; Multiplayer is disabled until a relay is configured.

---

## Staging room

Network play works like a Civ 6 multiplayer staging room. The host opens a room and configures it; players join open seats, pick their own name and faction, and mark Ready; the host starts the game once everyone is Ready. Public room settings and occupant status are visible to everyone in it. Generation seeds and seat credentials remain private.

### Game setup

- Both the room and direct `POST /api/game` accept **1-8 human players in the first seats** (humans first, then AIs), with 2-8 total players. The room always produces setups in that order.
- `PlayerSetup.difficulty` (`"Easy" | "Normal" | "Hard"`, omitted = Normal) passes through unchanged; the server's `Session` already hands it to the AI.

### Types

```
StagingSeat   = { kind: "host" | "open" | "ai" | "closed", name: string, faction: Faction | "Random",
                  team: number, difficulty: "Easy" | "Normal" | "Hard" }
StagingSettings = { width, height, turn_limit, seats: StagingSeat[] }
StagingConfig = { seed: string (decimal u64), ...StagingSettings } // host input only
                  // 2-8 seats; exactly one "host" seat, at index 0.
Occupant      = { name: string, faction: Faction | "Random", connected: bool, ready: bool, is_host: bool }
StagingState  = { room_id: string, config: StagingSettings, occupants: (Occupant | null)[], seed?: string }
                  // seed exists only in a loopback host WebSocket subscription, never /api/status.
                  // occupants[i] belongs to the stable slot config.seats[i]; null = nobody there (always null for ai/closed).
```

A seat's `name`/`faction` in the config are the host's defaults; an occupant's own `name`/`faction` override them for human seats. A slot ID is its position in `config.seats`: closing/removing a row leaves a `closed` slot in place, so other occupants never shift. Human-first compaction happens only when building the game.

Configs require 2-8 slots and exactly one host at slot 0; map dimensions are 8-128, and the turn limit is 1 through `u32::MAX - 1`. The seed is a decimal u64 string of at most 20 digits. Factions and difficulties must be valid enum values, and config names are limited to 80 characters. Occupant names are trimmed and capped at 80 characters (blank becomes `Player`). Invalid edits leave the room and game unchanged. Rooms may have fewer than two active seats while being edited, but starting must produce a valid game setup.

### HTTP (loopback only, like `/api/game`)

| Method | Path | Body / response |
| --- | --- | --- |
| POST | `/api/staging` | `StagingConfig`. Opens a room (replacing any room; a running game keeps running until the room starts) or updates the open room's settings. Every occupant's `ready` resets to false when settings change; posting identical settings is idempotent. Occupants of open seats that are removed or changed to AI/Closed are removed (they get `staging_seat_lost`). Response: `/api/status`. |
| POST | `/api/staging/start` | Starts the game. `409 {"code":"not_ready"}` unless every `open` seat has a connected, Ready occupant (the host seat needs no Ready). Response: `/api/status`. |
| DELETE | `/api/staging` | Closes the room; everyone in it gets `staging_closed`. |

`GET /api/status` includes `"staging": StagingState | null` even when no room exists, advertising support. All three admin routes use the same TCP loopback, Host/Origin, and POST JSON-content-type checks as `/api/game`, including the bodyless Start POST. A start without an open room returns `409` with `code: "no_room"`. A failed validation or save returns `400` and preserves the room, connected occupants, running game and committed autosave.

Rooms live in memory and do not survive server restarts. Creating/updating/closing a room leaves the running game alone. Starting commits the new game before sending the handoff; staging sockets stay connected for `staging_started`, while game-only sockets receive the usual replacement/rejoin error.

### WebSocket additions

Client → server:

| type | fields | meaning |
| --- | --- | --- |
| `staging_watch` | `host?: bool` | Subscribe without a seat. `host: true` includes the private seed only when the TCP peer is loopback; guests always get public settings. |
| `staging_take` | `seat`, `name`, `token?: string`, `code?: string` (join code, at most 32 characters) | Claim an empty open seat, or reclaim a reservation with its credential. The host slot requires loopback. Loopback can explicitly reassign any room seat without a credential, rotating it; authenticated reconnects keep the existing token. Success leaves any other room seat this socket held (one seat per connection); failed claims leave it unchanged. On a game with a join code, a claim that does not present the seat's own token needs the right `code` (`join_code_needed`, `wrong_join_code`, and the same five-strike close, `too_many_codes`). |
| `staging_remove` | `seat` | Loopback only. Take a connected remote player out of an open-slot seat: closes its connection with `code: "removed"`, rotates the reservation's credential to one nobody holds and keeps the name and faction as a disconnected reservation, so the seat is closed to everyone but the host until `staging_release`. Rejects non-loopback callers (`host_only`), the host's slot, empty and non-open slots (`not_removable`). |
| `staging_release` | `seat` | Loopback only. Clear a disconnected open-slot reservation, including its credential/Ready, without changing the slot kind, settings, other occupants or the host's seat. Broadcasts `staging`. Reject connected occupants, host/AI/closed slots and invalid indices. |
| `staging_update` | `name?`, `faction?` | Change your own name or faction. Clears your Ready. |
| `staging_ready` | `ready: bool` | |
| `staging_leave` | — | Free your seat (you keep watching); acknowledged privately with `staging_left`. |

Server → client:

| type | fields | when |
| --- | --- | --- |
| `staging` | `state: StagingState` | to every watcher and occupant, on every change (and right after `staging_watch`/`staging_take`) |
| `staging_taken` | `room_id`, `seat`, `token` | private acknowledgment of a successful seat claim; sent reliably before its room snapshot |
| `staging_left` | `room_id` | reliable private acknowledgment after `staging_leave` releases the reservation; remains deliverable even if room snapshots coalesce past the empty seat |
| `staging_started` | `game_id`, `seat`, `token` | private handoff to each connected occupant; `seat` is the compacted human seat. The client stores it and sends credentialed `join`. |
| `staging_seat_lost` | `message` | your seat was closed or turned into an AI by the host |
| `staging_closed` | — | the host closed the room |
| `error` | `code`: `seat_occupied` (connected reservation, wrong/missing credential), `seat_reserved` (disconnected reservation, wrong/missing credential), `seat_expired` (credential presented for an empty seat), `host_only` (release from a LAN peer), `seat_connected` (release raced with a reconnect), `not_releasable` (release targets a host/AI/closed/invalid slot), `not_open` (claim targets a seat that is not open), `no_room`, `join_code_needed`, `wrong_join_code`, `not_removable` (remove targets a host/empty/non-open slot or the host's own seat) | Release errors preserve the host's own credential. |

Every new reservation gets a 256-bit OS-random token independent of the generation seed. Disconnects retain it (`connected: false`, `ready: false`). A matching token preserves faction and resets Ready, even when displacing a still-connected stale socket; that socket receives `seat_taken` and stops automatic reconnects. A fresh browser cannot reclaim the reservation. The host opens a disconnected seat with `staging_release`; the next guest claims it without a token and receives a fresh credential. A stale LAN reconnect presenting the old token cannot reclaim an emptied slot automatically (`seat_expired`); the client forgets that expired reservation and offers a deliberate fresh claim. `staging_leave`, release, closure, and reassignment invalidate the old token.

Start copies all human credentials into the new game under the same mutex as readiness checks, persistence and game replacement, before any handoff or join can run. Even an absent host seat is reserved. A failed save leaves the old room and credentials intact. If a reload misses `staging_started`, a credentialed `staging_take` when no room exists recovers the matching game's compacted seat and repeats the private handoff; guessing a room slot grants nothing.

### Building the game

On start the server builds a `GameSetup` from the room: player ids go to human seats first (host, then open seats in seat order), then AI seats in seat order; closed seats are dropped. Names/factions come from occupants (human seats) or the config. `"Random"` resolves as `Faction::ALL[(seed % 8 + seat_index) % 8]` (seat index in the room), the same rule the local lobby uses. Team numbers are compacted in player order. `turn_limit`, `width`, `height`, `seed` and AI `difficulty` pass through.

### UI

- Staging is enabled by default when `/api/status` includes `staging`. `?staging=0` or `localStorage['legatum.staging'] = '0'` opts out; `?staging=1` opts back in. No fake-server detection is used.
- Host: Multiplayer → Host Game opens the same lobby as single player, with seat types Network player (open) / AI / Closed. Edits are posted to `/api/staging` with a 250 ms debounce and serialized writes. Open seats show who took them and their Ready state; a disconnected occupant has a confirmed **Open seat** action (`staging_release`) for the loopback host, without toggling AI/Closed. **Start game** is disabled immediately on any local edit until that exact edit is acknowledged and matches the live room, the host's own name/faction are synchronized, and every open seat is taken and Ready again. Removing a row closes its slot in place.
- Reloading a remembered seat 0 on a local connection restores the host lobby and its **Start game** control. Configuration writes wait for the matching room's host-only WebSocket snapshot, including its private seed. Hydration preserves the existing settings and configured seat defaults; guests' own names/factions remain display overrides, so the initial write is identical and does not clear their Ready flags. Other remembered staging seats restore the guest screen.
- Players: Multiplayer → Join Game (enabled when `status.staging` is set) shows the room read-only with a **Take seat** button on each empty open seat. Disconnected reservations say **Reserved — ask the host to open this seat**; the owning tab automatically restores its credential on reload/reconnect. The planet seed reads **Hidden**. After taking one: your name, your faction (same picker as the lobby), **Ready**, **Leave seat**.
- **Leave seat** is disabled while disconnected, with **You can leave once you are connected again.** (On a relayed online game: **You can leave once the host is back.**) The client also rejects disconnected leave calls. After sending a leave, the button waits for the server and UI ownership and the credential remain until the matching room's `staging_left` acknowledgment. Leave is never queued or replayed automatically. If the connection drops before acknowledgment, the client retains the credential and follows the normal credentialed staging reconnect flow; a refused/expired reservation clears ownership. Another leave requires a deliberate click.
- The client stores `{ seat, name, token, roomId, gameId: null }` at `sessionStorage['legatum.lan.seat']` only after `staging_taken`. `staging_started` atomically replaces it with `{ seat, name, token, gameId }` before opening the game socket. Name updates refresh the saved name. Acknowledged leave, seat loss, and room closure clear the reservation credential only when the client's defined room ID matches the stored room ID; watching another room never clears a saved running-game credential. Disconnects and `seat_taken` preserve the credential for deliberate rejoin. Deploy the client and server together for the `staging_left` acknowledgment and `invalid_seat_credential` code.
- On `staging_started`, every client joins its seat with the token and the normal game screen opens. A new guest browser needs an empty seat or the host's **Open seat** action.

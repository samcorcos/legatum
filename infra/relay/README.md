# Legatum relay

A Cloudflare Worker that is a lobby and a dumb pipe. It lists hosted games and copies bytes between a
host's tunnel and its guests. It holds no game state and makes no rule decisions. The contract is
`plans/online-play.md`.

- `Lobby` Durable Object (one instance): the list of rooms for `GET /lobby`.
- `Room` Durable Object (one per hosted game): the host tunnel and up to 8 guest sockets, using the
  WebSocket Hibernation API. It reads only the 5-byte tunnel header, plus the host's `status` and
  `summary` frames.

Both classes are SQLite-backed (`new_sqlite_classes`), so they run on the Workers Free plan.

## Run locally

```
npm install
npm run dev          # wrangler dev on http://127.0.0.1:8787
node scripts/fake-host.mjs            # announces a room, echoes guest texts (Node 22+)
node scripts/fake-host.mjs http://127.0.0.1:8787 --name "Sam" --build 8.9.74 --rules 1 --protocol 1
npm test             # vitest in the Workers runtime
npm run typecheck    # tsc --noEmit
```

`fake-host.mjs` prints the guest socket URL and the status URL. Ctrl-C sends `bye`.

## Deploy

```
export CLOUDFLARE_API_TOKEN=...      # a token with "Edit Cloudflare Workers"
npx wrangler deploy
```

The URL is `https://relay.playlegatum.com` (the `routes` entry in `wrangler.jsonc`; a fork changes it to a domain in its own Cloudflare account, or removes it and sets `"workers_dev": true` to get `https://legatum-relay.<your-subdomain>.workers.dev`). Nothing secret is
committed. If the token sees several accounts, also set `CLOUDFLARE_ACCOUNT_ID`.

The limits' vars deploy with the Worker from `wrangler.jsonc`. To stop new rooms in an emergency without a
deploy, set `LOBBY_OPEN` to `0` in the dashboard (Workers > legatum-relay > Settings > Variables); the next
`wrangler deploy` puts back the value in `wrangler.jsonc`.

CI (`.github/workflows/relay.yml`) runs the typecheck and tests on every change under `infra/relay`. On `master` it then runs `wrangler deploy` itself when the repository secrets `CLOUDFLARE_API_TOKEN` (Workers Scripts: Edit) and `CLOUDFLARE_ACCOUNT_ID` exist, and skips the deploy with a notice when they do not. The manual deploy above still works.

## Behaviour notes

- Room ids are 16 random characters, host keys 40, both from `crypto.getRandomValues`. The relay stores
  only a SHA-256 of the host key and compares digests in constant time.
- A guest closed with 1013 means "no host", "room full" or "too many connections from one address" (the
  reason says which). 1012 means "tunnel dropped or replaced, reconnect".
- The relay closes the host tunnel with 1009 if the host sends a message over 1 MiB, and with 1003 if it
  sends text (tunnel frames are binary). A guest is closed with 1009 over 64 KiB and 1003 for binary.
- Either side is closed with 4029 when it breaks its rate cap (below); the reason is `too many messages`
  or `too much data`. A closed tunnel behaves like any dropped tunnel: guests get 1012, the host reconnects.
- Invalid `summary` frames and non-object `status` frames are ignored; the last good value stays.
- An unclaimed room, or one whose host left, is deleted after 120 s. `bye` deletes it at once.
- Kind 6 is the host's keepalive `ping` frame (the browser host sends one every 20 s, because a browser
  cannot send WebSocket pings). The relay never forwards it and does not answer it.
- WebSocket protocol pings (the desktop host's, every 20 s) are answered by the runtime without waking the
  room, so the room cannot see them.

## Limits and kill switch

Every number below is enforced in the Worker and its Durable Objects, so nothing needs configuring in the
Cloudflare dashboard. The ones marked "var" can be changed in `wrangler.jsonc` `vars` (or in the dashboard
under the Worker's Variables) without a code change; a missing or unusable value falls back to the default.

| Limit | Default | Var | Where | When broken |
| --- | --- | --- | --- | --- |
| New rooms allowed at all | on | `LOBBY_OPEN` (`"0"` = off) | Worker | 503 "Hosting new games is paused. Try again later." Running rooms carry on. |
| Rooms on the relay | 256 | `MAX_ROOMS` | Lobby | 503 "This lobby is full. Try again in a few minutes." |
| Live rooms per client address | 2 | `MAX_ROOMS_PER_IP` | Lobby | 429 "You already have 2 games open. Close one before hosting another." |
| Live rooms per network (IPv4 /24, IPv6 /48) | 8 | `MAX_ROOMS_PER_NET` | Lobby | 429 "Too many games are open from your network. Try again later." |
| Room creations per client address | 6 per 10 minutes | `ROOM_CREATES_PER_10_MIN` | Lobby | 429 "You are opening games too quickly. Try again in a few minutes." with `Retry-After` |
| Guest sockets per client address in one room | 4 | `MAX_GUESTS_PER_IP` | Room | closed 1013 `too many connections from one address` |
| Guest sockets per room | 8 | | Room | closed 1013 `room is full` |
| Guest messages | burst 300, then 5 per second | | Room | closed 4029 `too many messages` |
| Guest bytes | burst 4 MiB, then 128 KiB per second | | Room | closed 4029 `too much data` |
| Host tunnel frames | burst 4,000, then 20 per second | | Room | closed 4029 `too many messages` |
| Host tunnel bytes | burst 64 MiB, then 4 MiB per second | | Room | closed 4029 `too much data` |
| Message size | 64 KiB guest to host, 1 MiB host to guest | | Room | closed 1009 |
| Announce body | 2 KiB | | Worker | 413 |
| Host `status` frame | 64 KiB | | Room | dropped; `/api/status` keeps the last good one |
| Host heartbeat | 60 s | | Lobby | a host that has sent a `ping` frame and then nothing for 60 s is left out of `GET /lobby` until it is heard again |

- The `error` strings of the 429 and 503 answers of `POST /rooms` are short plain sentences meant to be shown to the player as they are. The 400, 413 and 500 strings (`invalid room announcement`, `body too large`, `internal error`) are technical and are not for players.
- Client address means `CF-Connecting-IP`, with an IPv6 address cut to its first 64 bits (`addressKey`): one
  home or phone line owns a whole /64, so its addresses count as one. The Lobby hashes it at once (SHA-256 with a random salt that
  never leaves the Lobby) and keeps only the hash: on the room's row, gone when the room is deleted, and in a
  create-rate bucket, gone once the bucket has refilled (at most 10 minutes). Rooms hash guest addresses the
  same way with their own salt, in the guest socket's attachment, gone with the socket. Raw addresses are
  never stored or logged.
- A refused create does not spend a create token; only rooms actually made count toward the rate.
- Rate caps are token buckets per socket, sized from the protocol: a turn is about one `view` per guest
  chunked at 256 KiB (a big late-game view is a few MiB, so 8 guests are 100-200 frames and about 32 MiB at
  once), and guests send small texts (orders, chat, a ping every 20 s). The bursts cover many turns back to
  back; the low refill rates bound what a flooding socket costs, since incoming messages are what is billed.
- The heartbeat applies only to hosts that send `ping` frames (the browser host), because the room cannot
  see the desktop host's protocol pings. A pinging host costs one Lobby call about every 40 s.
- The list never shows text a host typed. Anyone can host, so a free-text name would let anyone put any message
  in front of every player. Each room is listed as two words and a number picked by its room id (`listedName` in
  `src/shared.ts`, for example "Amber Harbour 07"); `POST /rooms` returns it as `name`, and the app shows the host
  the same name (`app/src/net/roomName.ts` mirrors the rule). A `name` an older page still sends is checked for
  type and length, then dropped. `sanitizeName` and `BLOCKED_NAME_TERMS` remain for the build-label check.
- The build label must be one the game makes (`buildOk`): `Version <n.n.n>` with an optional `· web <hash>` or
  `(<hash>)`, `Playtest <n>`, `Dev · after <n> (<hash>)`, `Dev (<hash>)`, `Dev build`, a bare version number, or a
  preview name `playtest-N`, `preview-N`, `test-N` or `beta-N`. Anything else is refused with 400, since the list
  shows the label to players on another version.
- Per-network cap: a network is an IPv4 /24 or an IPv6 /48 (`networkKey`), hashed with the same salt and kept only
  on the room's row. One free IPv6 tunnel account is a /48, which is 65,536 /64s; without this cap one person
  could fill the list.
- `POST /rooms` may carry `locked: true` (a boolean, default false; any other type is refused with 400). It means the
  host set a join code. The relay stores and lists only that flag in `GET /lobby` rows (`locked`); the code itself is
  checked by the host's engine and never reaches the relay. Deploy the relay before an app build that sends `locked`.
- Every answer carries `X-Content-Type-Options: nosniff`, since `/api/status` hands back text a host wrote.

### Edge rule

The relay is served at `https://relay.playlegatum.com` (a custom domain, set in `wrangler.jsonc`), so the
zone's rate-limiting rule stops a flood at Cloudflare's edge before it reaches the Worker or counts toward
the day's requests. The rule is set once in the dashboard (Security > WAF > Rate limiting rules); the free
plan allows one rule with a 10-second window, shared with `games.playlegatum.com`:

- When incoming requests match: hostname is `relay.playlegatum.com` or `games.playlegatum.com`
  (expression `http.host in {"relay.playlegatum.com" "games.playlegatum.com"}`).
- Same characteristics: IP. Requests: 40. Period: 10 seconds. Action: Block for 10 seconds.

A player sends a handful of requests a minute (the list refreshes, one socket per game; messages inside a
socket are not requests), so the rule only meets scripts. Both Workers have their `workers.dev` address
turned off (`"workers_dev": false`), so the custom domains, and the rule, are the only way in.

## Security model

- The lobby is public: anyone can list rooms and open guest sockets. Room ids are unguessable, but they
  are not secrets once listed.
- Only the holder of the host key can open the tunnel. The key is returned once by `POST /rooms`.
- A host can lock its game with a join code and remove players; both are decided in the host's tab, so the relay
  learns nothing more than the `locked` flag.
- Seat tokens and every game message stay end to end between guest and host. The relay copies the bytes
  and cannot tell a seat token from any other text.
- `POST /rooms` needs no account, and CORS stays open (`*`) because an origin check stops no one who
  scripts requests. What bounds a stranger is the limits table above: one address can hold 2 rooms and
  make 6 every 10 minutes, and one network 8 rooms, so filling the 256-room lobby takes 128 addresses on at least
  32 networks, and `LOBBY_OPEN=0` stops new rooms
  at once if it happens. Someone with many addresses can still fill it; then tighten the edge
  rule above (lower the limit) or add a shared secret on `POST /rooms`.

## Cost on the free plan

Free plan allowances (see facts below): 100,000 Worker requests per day, 100,000 Durable Object requests
per day, 13,000 GB-s of Durable Object duration per day. Incoming WebSocket messages count 20:1 as
requests; outgoing messages and protocol pings are free. An idle hibernating Room is not billed for duration.

Rough day for one 2-human game of 4 hours:

- Lobby polling: the UI polls `GET /lobby` every 3 s while the Join screen is open. That is 1 Worker request
  plus 1 Lobby request per poll, about 20 per minute per open lobby screen. One person browsing for 10 minutes
  costs about 200 requests of each kind.
- Game traffic: say 1 guest message per 2 s while playing, 4 hours = 7,200 messages, which bills as 360
  Durable Object requests. Status pushes from the host are messages too (about one per turn change or
  guest action), the same order of magnitude.
- Guest `GET /api/status` polls, if any, are 1 Worker request plus 1 Room request each.

So a normal evening is a few thousand requests, far below 100,000. The first limit to bite would be many people
sitting on the Join screen (3 s polling: one screen open for an hour is about 1,200 Worker plus 1,200 Lobby
requests, so roughly 80 person-hours of open lobby screens per day). If that ever matters, poll less often
or cache `/lobby` in the Worker for a few seconds.

Duration: the Room hibernates between messages, so it is billed only while it handles a message. Active rooms
with chatty guests might keep it awake a few percent of the time: 128 MB x 0.05 x 86,400 s is about 550 GB-s,
well under 13,000. The Lobby only wakes on requests.

## Cloudflare facts relied on (checked 2026-11-21)

- Hibernation API (`ctx.acceptWebSocket(ws, tags)`, `webSocketMessage/Close/Error`, `getWebSockets(tag)`,
  `serializeAttachment` with a 16,384 byte limit that is lost when the socket closes; handlers get
  `string | ArrayBuffer`; protocol pings do not call the handler):
  https://developers.cloudflare.com/durable-objects/best-practices/websockets/
- Durable Object limits: WebSocket received message size 32 MiB (so the contract's 1 MiB host-to-guest limit
  fits with room to spare, and the host's 256 KiB chunks are not needed for size, only for the relay's own
  1 MiB cap). CPU per request or message 30 s default on SQLite-backed objects. SQLite-backed objects on
  Free, 5 GB per account:
  https://developers.cloudflare.com/durable-objects/platform/limits/
- Pricing and free allowances: 100,000 requests/day and 13,000 GB-s/day for Durable Objects, 20:1 billing of
  incoming WebSocket messages, no duration charge while eligible for hibernation, outgoing messages and
  incoming pings not charged:
  https://developers.cloudflare.com/durable-objects/platform/pricing/
- Workers Free plan: 100,000 requests/day, 10 ms CPU per HTTP request:
  https://developers.cloudflare.com/workers/platform/limits/
- Testing: `@cloudflare/vitest-pool-workers` is deprecated and renamed `@cloudflare/vitest-plugin`; the config
  uses `cloudflareTest({ wrangler: { configPath } })` and tests import `env` and `exports` from
  `cloudflare:workers`:
  https://developers.cloudflare.com/workers/testing/vitest-integration/get-started/write-your-first-test/

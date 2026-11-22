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

The URL is `https://legatum-relay.<your-subdomain>.workers.dev`. Nothing secret or account-specific is
committed. If the token sees several accounts, also set `CLOUDFLARE_ACCOUNT_ID`.

CI (`.github/workflows/relay.yml`) runs the typecheck and tests on every change under `infra/relay`. On `master` it then runs `wrangler deploy` itself when the repository secrets `CLOUDFLARE_API_TOKEN` (Workers Scripts: Edit) and `CLOUDFLARE_ACCOUNT_ID` exist, and skips the deploy with a notice when they do not. The manual deploy above still works.

## Behaviour notes

- Room ids are 16 random characters, host keys 40, both from `crypto.getRandomValues`. The relay stores
  only a SHA-256 of the host key and compares digests in constant time.
- A guest closed with 1013 means "no host" or "room full". 1012 means "tunnel dropped or replaced, reconnect".
- The relay closes the host tunnel with 1009 if the host sends a message over 1 MiB, and with 1003 if it
  sends text (tunnel frames are binary). A guest is closed with 1009 over 64 KiB and 1003 for binary.
- Invalid `summary` frames and non-object `status` frames are ignored; the last good value stays.
- Rooms: at most 64 (a room counts from `POST /rooms` until it is deleted). An unclaimed room, or one whose
  host left, is deleted after 120 s. `bye` deletes it at once.
- The host's WebSocket pings (every 20 s) are answered by the runtime without waking the room.

## Security model

- The lobby is public: anyone can list rooms and open guest sockets. Room ids are unguessable, but they
  are not secrets once listed.
- Only the holder of the host key can open the tunnel. The key is returned once by `POST /rooms`.
- Seat tokens and every game message stay end to end between guest and host. The relay copies the bytes
  and cannot tell a seat token from any other text.
- Hard limits (rooms, guests, message sizes, 2 KiB announce body) bound what a stranger can cost.
- Anyone can create up to 64 rooms and fill the lobby. That is an accepted risk for a small game; add
  rate limiting or a shared secret on `POST /rooms` if it ever matters.

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

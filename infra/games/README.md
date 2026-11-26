# Legatum game uploads

A Cloudflare Worker that receives the record of every game played on the hosted web build and stores it in a
private R2 bucket, so the AI's strength can be measured and bugs found (the README's Privacy section is the
promise to players).

- The page (`app/src/platform/gameUpload.ts`) sends `PUT /v1/games/<game id>` with the gzipped record after each
  of the first five turns, then every five turns, and when the game ends or is abandoned. A later checkpoint replaces the earlier one. An online game is sent by its host's tab, at the host engine's own checkpoints.
- Every seat is under its default name ("Player 1", "AI 2"): the engine's `public_archive` renames seats and
  replays the orders to recompute the hashes (`Recording::anonymized`). The Worker refuses any other seat name.
- Stored as `games/<YYYY-MM>/<game id>.json.gz` (the month of the start time in the game id), with custom metadata for listing without downloading:
  `outcome`, `turns`, `engine`, `winner`, `seats` (`Faction*:Difficulty`, `*` for a human seat). `winner` must be
  null or the winning team's number (an integer 0 to 255); anything else is answered 422 "bad winner".
- The record may carry the player's coach-question answers as a top-level `coach_answers` list (at most 2000
  objects with the answer line's own keys, text up to 2000 characters, evidence up to 32,000; the Worker
  refuses anything else, and the app clips answers to fit). It is not part of the engine's `Recording`, which
  denies unknown keys, so strip it before feeding a record to the engine.
- The record may also carry `rewinds`, one entry per "Load an earlier turn" in a local game: `{at, from_turn, to_turn}`
  (known keys only, `at` an ISO time, `to_turn` below `from_turn`, turns 0 to 10,000, at most 1000 entries; the
  Worker answers 422 "bad rewinds" otherwise; `checkRecord` in `src/index.ts`). The tape holds only the line the
  player kept, so `rewinds` is how analysis learns that other turns were played and discarded. Like
  `coach_answers` it is upload-only: strip it before feeding a record to the engine, and exports drop it.
- Only what the game itself writes is accepted (uploads are written by strangers and read on the analysis Mac):
  seat factions and difficulties from the lists in `src/names.ts` (a core test keeps that list equal to the
  game's factions), `engine_commit` a full 40-digit git commit, and `created_at` a real ISO time
  no more than a day after the start time in the game id (earlier is fine: a save loaded from a file gets a
  new id and keeps its `created_at`). That start time must fall after 2026-11-01 and not in the future, and it
  names the storage month, so a sender cannot pick the month. Anything else is answered 422.
- Each game has an upload key: 32 random hex digits the page makes the first time it uploads that game, keeps
  in the browser's `localStorage` (`legatum.uploadKeys.v1`, the 200 games uploaded most recently) and sends as `X-Upload-Key`.
  The Worker stores its SHA-256 as the object's `key` metadata and answers `409` to a checkpoint for that game
  under another key or none, so someone who learns a game id (it is inside every exported save) cannot
  replace the record. The key names a game, never a player. A game stored without a key (an older build, or a
  browser with no storage) stays open until a keyed checkpoint locks it. A browser that lost its keys gets
  `409` for its unfinished games and drops them from its queue; what was stored stays.
- Only pages listed in `ALLOWED_ORIGINS` (`wrangler.jsonc`) may upload: the site, its preview deploys and
  local dev. Bodies are capped at 2 MiB gzipped and 16 MiB unpacked.
- One client address (`CF-Connecting-IP`, an IPv6 address cut to its /64) may store at most 50 new games per
  UTC day: the Worker var `MAX_NEW_GAMES_PER_DAY` in `wrangler.jsonc` (a bad value means 50). A new game is a
  `PUT` whose R2 key does not exist yet; one more is answered `429` with `{"error":"too many new games today"}`
  and a `Retry-After` (seconds until 00:00 UTC), and nothing is stored. A later checkpoint of a stored game is
  never counted or refused. The page keeps such a record queued and retries later.
- One network (an IPv4 /24 or IPv6 /48, `networkKey` in `src/quota.ts`) may store at most 200 new games per UTC
  day, so one host with many /64s cannot spend the total below by itself. Its row is a salted hash too.
- All addresses together may store at most 50000 new games per UTC day (`MAX_NEW_GAMES_TOTAL_PER_DAY`, a bad
  value means 50000), so many addresses cannot grow the bucket without limit; past it every new game gets the
  same `429`. It is far above real traffic (about 1,000 new games on the busiest day so far, 2026-11-25) and only stops a flood; play is never limited, only what is saved for analysis. Bodies may unpack to at most 16 MiB (the largest real record is about 1.4 MB).
- What is kept about a sender: a salted SHA-256 of the address (cut to /64 for IPv6) and of its network, each with a count, for the
  current UTC day only, inside the `Quota` Durable Object (SQLite-backed; the salt is made once and never
  leaves it; older rows are deleted on every call). The raw address is never stored or logged.
  Observability (request logs) is off on purpose: logs would hold IP addresses.

## Run locally

```
npm install
npm test             # vitest in the Workers runtime, with a local R2
npm run typecheck
npm run dev          # wrangler dev on http://127.0.0.1:8787
```

## Deploy

```
export CLOUDFLARE_API_TOKEN=...      # Workers Scripts: Edit and Workers R2 Storage: Edit
npx wrangler r2 bucket create legatum-games    # once
npx wrangler deploy
```

The URL is `https://games.playlegatum.com` (the `routes` entry in `wrangler.jsonc`; a fork changes it to a domain in its own Cloudflare account, or removes it and sets `"workers_dev": true` to get `https://legatum-games.<your-subdomain>.workers.dev`). The zone's rate-limiting rule covers it (`infra/relay/README.md`, "Edge rule"). Set it as the repository variable
`LEGATUM_GAMES_URL` (or the environment variable of the same name for `scripts/build-web.sh`): the web build
then uploads games there and allows the page to connect to it. Without it the web build uploads nothing.

## Reading the games

By hand:

```
npx wrangler r2 object get legatum-games/games/2026-11/<game id>.json.gz --file game.json.gz --remote
```

Unzip it and take the `tape` key: that is the Tape JSON the simulator's examples replay (`game_dump`,
`audit_states`). The bucket is private and no page can read it.

The analysis job on the developer's Mac (`tools/games`, see its README) reads through a private API on the same
Worker: `GET /v1/read/games?cursor=` lists the stored games with their metadata and `GET /v1/read/games/<key>`
returns one record, still gzipped. It needs `Authorization: Bearer <READ_TOKEN>`; `READ_TOKEN` is a Worker secret
(`npx wrangler secret put READ_TOKEN`, at least 32 characters, or `tools/games/install-mac.sh --token`). Without
the secret, or with a wrong token, every `/v1/read/` path answers 404, so its existence is not advertised. It
ignores `ALLOWED_ORIGINS`, so no browser page is allowed to use it.

# Legatum game uploads

A Cloudflare Worker that receives the record of every game played on the hosted web build and stores it in a
private R2 bucket, so the AI's strength can be measured and bugs found (the README's Privacy section is the
promise to players).

- The page (`app/src/platform/gameUpload.ts`) sends `PUT /v1/games/<game id>` with the gzipped record every
  five turns and when the game ends or is abandoned. A later checkpoint replaces the earlier one.
- Every seat is under its default name ("Player 1", "AI 2"): the engine's `public_archive` renames seats and
  replays the orders to recompute the hashes (`Recording::anonymized`). The Worker refuses any other seat name.
- Stored as `games/<YYYY-MM>/<game id>.json.gz`, with custom metadata for listing without downloading:
  `outcome`, `turns`, `engine`, `winner`, `seats` (`Faction*:Difficulty`, `*` for a human seat).
- The record may carry the player's coach-question answers as a top-level `coach_answers` list (at most 2000
  objects with the answer line's own keys, text up to 2000 characters, evidence up to 32,000; the Worker
  refuses anything else, and the app clips answers to fit). It is not part of the engine's `Recording`, which
  denies unknown keys, so strip it before feeding a record to the engine.
- Seat factions and difficulties must be plain words, and `created_at` a real ISO time (it names the storage
  month): uploads are written by strangers and read on the analysis Mac.
- Only pages listed in `ALLOWED_ORIGINS` (`wrangler.jsonc`) may upload: the site, its preview deploys and
  local dev. Bodies are capped at 4 MiB gzipped and 64 MiB unpacked.
- It keeps nothing about the sender. Observability (request logs) is off on purpose: logs would hold IP
  addresses.

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

The URL is `https://legatum-games.<your-subdomain>.workers.dev`. Set it as the repository variable
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

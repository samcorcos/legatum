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

```
npx wrangler r2 object get legatum-games/games/2026-11/<game id>.json.gz --file game.json.gz --remote
```

Unzip it and take the `tape` key: that is the Tape JSON the simulator's examples replay (`game_dump`,
`audit_states`). The bucket is private; there is no public read path.

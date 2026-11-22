# Automatic analysis of uploaded games

Every game played on the website is uploaded to a private R2 bucket (`infra/games`). This job, on the
developer's Mac, notices new games, analyses them with the Mac's own model budget, and keeps a ledger of what
has been analysed (user, 2026-11-22).

## What it does

Every 15 minutes (launchd, `io.legatum.game-analysis`), `analyse_games.py poll`:

1. Lists the bucket through the games Worker's private read API (`GET /v1/read/games`, bearer token held in the
   Mac's keychain as `legatum-games-read` and on the Worker as the secret `READ_TOKEN`).
2. Updates `~/LegatumData/game-analysis/ledger.json`: every game seen, its latest upload (etag, turns, outcome)
   and which upload was analysed.
3. Analyses each game that is due:
   - finished or abandoned: once per upload;
   - still in progress but idle for 6 hours with new turns since its last report: a checkpoint report;
   - fewer than 10 turns: skipped; a game that fails 3 times on the same upload is left for a person.
4. For each game, in `reports/<game id>/`:
   - `facts.md`: standings, wars, expansion, losses, key events, the AI incident detector and the coach
     questions with the player's answers. The replay uses the engine the game was played on: `game_dump` and
     `legatum-analyze` are built at the game's `engine_commit` and cached in `engine/bin/<commit>`.
   - `review.md`: a short coach-style triage written by Luna (Codex), or Haiku (Claude Code) if Codex fails,
     from `review_prompt.md`.
   - `events.jsonl`, `coach-answers.json`, `record.json.gz` (the upload, for replays).
5. Rewrites `reports/INDEX.md` (newest first) and shows a macOS notification.

Engine builds are heavy, so analysis waits while the Mac is on battery (`--force` overrides).

## Install

```
tools/games/install-mac.sh --token   # first time: new read token in the keychain and on the Worker, then the job
tools/games/install-mac.sh           # refresh the job after changing the script
tools/games/install-mac.sh --remove  # stop it (ledger and reports stay)
```

`--token` needs `CLOUDFLARE_API_TOKEN` for wrangler. Deploy the Worker with the read API first
(`cd infra/games && npx wrangler deploy`).

## By hand

```
python3 tools/games/analyse_games.py poll --dry-run           # what is due
python3 tools/games/analyse_games.py poll --only <game id> --force
python3 tools/games/analyse_games.py facts record.json.gz --events events.jsonl
```

Settings: `LEGATUM_GAMES_URL` (the Worker), `LEGATUM_REPO` (where to fetch engine commits),
`LEGATUM_REVIEWER` (`codex,claude` order), `LEGATUM_GAMES_READ_TOKEN` (instead of the keychain).

Reports are a first pass. The full post-game review is the `legatum-game-coach` agent; the hill-climb work
replays the player's orders against changed AIs.

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
   - fewer than 10 turns: skipped; a game that fails 3 times on the same upload is left for a person. If the reviewer itself is down (a trivial canary call fails too, e.g. the Claude login expired), the failure does not count against the game: the run stops and you get one notification (`reviewer_down` in the ledger) until a review works again.
4. For each game, in `reports/<game id>/`:
   - `facts.md`: standings, wars, expansion, losses, key events, the AI incident detector and the coach
     questions with the player's answers. The replay uses the engine the game was played on: `game_dump` and
     `legatum-analyze` are built at the game's `engine_commit` and cached in `engine/bin/<commit>`.
   - `review.md`: a short coach-style triage written by Haiku (Claude Code) from `review_prompt.md` with
     `facts.md` pasted in, headed as untrusted model output.
   - `events.jsonl`, `coach-answers.json`, `record.json.gz` (the upload, for replays).
5. Rewrites `reports/INDEX.md` (newest first) and shows a macOS notification.

Engine builds are heavy, so analysis waits while the Mac is on battery (`--force` overrides).

## Safety

Anyone can upload a game, so everything in an upload is treated as hostile:

- Only engine commits already on `master` are built: GitHub serves pull-request commits from the same
  repository, and building whatever commit a game names would run a stranger's code.
- The macOS notification gets the text as an argument, never inside AppleScript source. The Worker accepts
  only plain-word faction and difficulty names.
- At most 3 analyses per run and 12 per day (`LEGATUM_ANALYSE_PER_RUN`, `LEGATUM_ANALYSE_PER_DAY`), and only
  games with a human seat, so junk uploads cannot spend the Mac's model budget. When more are due, finished games go first, then the
  newest uploads (never the longest: an upload can claim any turn count).
- The reviewer gets `facts.md` pasted into its prompt and no tools at all (`claude -p --tools ""`), so text a
  player typed cannot make it read files, run commands or reach the network. Player text is cut to one line
  of 500 characters in `facts.md` (with runs of `=` thinned, so nobody can fake the prompt's end-of-data
  marker), and `review.md` starts with an "untrusted" line for whoever reads it next.
  Codex cannot turn its shell off, so it is opt-in (`LEGATUM_REVIEWER=codex,claude`) and runs `-s read-only`.
- `install-mac.sh --token` hands the token to `wrangler` and `security` on stdin, never as an argument, so it
  never shows in the process list.
- Every command has a timeout, so a crafted tape or a stuck model cannot hold the job's lock.

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
`LEGATUM_REVIEWER` (default `claude`; `codex,claude` tries Codex first), `LEGATUM_GAMES_READ_TOKEN` (instead of the keychain).

Reports are a first pass. The full post-game review is the `legatum-game-coach` agent; the hill-climb work
replays the player's orders against changed AIs.

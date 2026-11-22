# AI quality: the hill-climb ledger

Every measured change to the AI is one row in `ledger.jsonl`, so we can see how the AI's
quality climbed and what moved it. `chart.html` renders it (x = AI version, y = metric with
95% CI bars, each point labelled with what changed). Search findings that look like broken
rules or unfun exploits go in `exploits.md` for the lead to decide.

Regenerate after adding rows:

```sh
python3 tools/ai-quality/report.py <run.json> --kind duel --paired <baseline.json> --out summary.json
python3 tools/ai-quality/ledger.py --version "<AI version>" --change "<what changed>" \
    --summary summary.json --run "<where the raw run lives>" --notes "<anything else>"
python3 tools/ai-quality/chart.py
```


**Optimization program (2026-10-18):** [`optimization-program.md`](optimization-program.md) is the plan for hill-climbing the AI toward the user's own milestones: the research, the human milestone suite (`milestones/`), the unattended CMA-ES climb, the gaps search can't reach, and a staged roadmap with cloud and token costs.

## Metrics

**Primary (since 2026-10-06, lead decision delegated by the user): 1v1 score against the
human-playbook proxy** (`duel_1v1_vs_proxy`). The proxy (Proxy v6 in its 1v1 mode,
`--duel-opponent proxy6`) is side B of the duel matrix below; side A is the AI being measured.
Plain Hard scored 24.9% when the switch was made (the proxy won 75.1%). The proxy reads only
its own view (no cheats). In a 2-player lobby its 8-player schedule is replaced by a duel
mode (never consolidates, any known rival city is a target, staging spills onto ring 4, one
infantryman or a rebuilt scout finds an unseen rival, army of 8 shooters and 4 infantry).
8-player proxy games are bit-for-bit unchanged, so `proxy_v6_8p` stays comparable.
- Overfitting guard: tune on seeds 8100001+; report every kept change on the standard seeds
  7100001+ (512 games, the ledger value) and on held-out seeds 9100001+ (in the row's notes).
- Every kept change must show no significant drop in the old primary metric below (now a
  regression check). The user's playtests are the final judge.
- The chart's proxy panel marks the switch with a dashed line; rows before it measured the
  proxy as an exploiting-opponent benchmark only.

```sh
legatum-sim --players 2 --map-size small --duel-matrix --difficulty hard --duel-opponent proxy6 \
    [--search k=3,h=20,d=4 --search-rotation] --games 512 --seed 7100001
```

**Second benchmark (since 2026-10-06): Proxy v7, the conqueror** (`duel_1v1_vs_proxy7`, and
`proxy_v7_8p` for 8 players): the user's playtest-8.2 playbook (Soldiers from turn 2, Settler
snipes, declare-and-strike assaults with 3+ Soldiers, no consolidation, peace only to move a
stalled army). `--duel-opponent proxy7`; `--proxy 0 --rotate-proxy --proxy-version 7
--proxy-start peace --barbarians true --difficulty hard` for 8 players. Same seed sets as above.

**Big batches run on the cloud** (`tools/cloud/run-batch.sh`, see `infra/cloud-sims/README.md`):
2,048 games per side for any keep-or-revert decision expected under 4 points. A duel matrix runs
on one server (`--count 1`); `report.py` reads the batch's `merged.json.gz`.

**Regression check (primary until 2026-10-06): 1v1 score against plain Hard** (`duel_1v1_vs_hard`).
For a change to plain Hard itself, side A is the new Hard and side B the previous Hard, seated
with `--opponent-params` set to the old decision-gene values (side B gets the opponent
parameters in every duel-matrix game); 50% = no change in strength.

Schedule and fields (shared by both 1v1 metrics):
- Two empires on a Small map (40×26, what the app recommends for 2–4 players), Standard
  speed, barbarians on, VP target 10.
- Schedule `--duel-matrix`: game *i* plays ordered faction pair (*i* / 2) mod 64 on seed
  first + *i* / 2. Side A (the AI being measured, `--search-rotation`) sits in seat *i* mod 2,
  so every pair is played from both seats on the same map. 512 games = 4 full cycles: every
  side-A faction meets every side-B faction (mirrors included) 8 times.
- Value: side A's score per game (win 1, loss 0, draw or stall ½). 50% = as strong as plain
  Hard. The interval is a 95% Wilson interval.
- `paired_vs_baseline`: the mean per-game score change against plain Hard on exactly the same
  games (same index, seed, seats). Much tighter than comparing two win rates; use it to judge
  a change. `better` / `worse` count games whose outcome flipped.
- Standard seeds 7100001+.

**Bias check** (`duel_1v1_seat_bias`): plain Hard against plain Hard, seat 0's win rate.

**Decision cost** (in every search row, `decision_ms`): wall milliseconds per turn of the
search seat's decision (mean, p95, max), and of all AI seats together, measured inside the
simulator while other games ran in parallel (so it overstates a quiet machine). Pillar 7:
an AI turn must not keep players waiting.

**Dropped for now (user, 2026-10-06)**: 8-player games and the Proxy v6 human-playbook
benchmark (`proxy_v6_8p`, lower = stronger AI). Each player count gets its own simulations
and tuning; 3 players with alliances comes after 1v1. The Proxy rows kept here are history:
three backfilled from `docs/learnings.md` and one measured on 2026-10-06 before the switch.

## Row fields

`date`, `commit`, `version` (AI version label), `change`, `metric`, `value`, `ci95`, `games`,
`decision_ms`, `searches_per_game`, `override_share` (share of searches that picked a plan
other than the analytic favourite), `paired_vs_baseline`, `backfilled`, `notes`, `run` (raw
run location, outside the repo), `details` (per-faction scores, the 8×8 matchup matrix,
search statistics).

Raw runs (per-game JSON with every search: options, rollout samples, chosen plan, trigger
and horizon, in `players[].search_events`) live under `~/LegatumRuns/search-ai/`.

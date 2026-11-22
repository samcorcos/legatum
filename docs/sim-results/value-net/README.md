# Learned value network (AI war plan step 3)

A small integer network that replaces the fitted linear score inside Hard's chance-to-win estimate (`crates/ai/src/winchance.rs`). Every plan the goal planner weighs (Economy, Peace, Campaign, Contain, Snipe, Hold, field strikes) and the diplomacy prices are differences of that estimate over projected facts. So a better estimate changes which plan Hard picks without new behaviours. Gene `learned_value`, default 0.

## What it is

- **Target:** each team's chance to win the game from this seat's `PlayerView`, trained against the eventual winner of whole games. The network gives each team a score. The estimate is that team's share of a softmax over all teams, plus the rule-based overlay: a public hold that completes wins outright, and the `vp_race` term is added to this seat's score. Training goes through the same overlay, so what is fitted is exactly what Hard reads.
- **Why this formula first:** it's the one currency every Hard macro decision is priced in. The planner already projects its inputs ("one more city, two fewer Soldiers, a shorter runway"), so a drop-in scorer over the same facts needs no planner change. The rules stay exact: title transfers, holds and race terms aren't relearned. The earlier `learned_macro` failed for a different reason. It regressed plan value from rollouts that were themselves scored by the hand formula, per plan head. This network learns positions from real outcomes.
- **Inputs (20 per team, permille, `value_net::INPUT_NAMES`):**
  - the nine linear features;
  - whether the team is this seat;
  - phase, and one over the team count;
  - hold state, hold progress and hold margin;
  - crowns held;
  - the gap to the best other team on VP progress, cities, economy and army.
- **Network:** 20 → 32 → 16 → 1, ReLU, 1,217 parameters. Weights are ×4096 integers and biases are milli-units, in `crates/ai/data/value_net.json`, compiled in. Inference uses i64 with truncating division. A unit test checks the trainer's golden integer scores.

## Pipeline

| Step | Command | Where |
|---|---|---|
| Games and rows | `crates/sim/examples/value_data.rs` (`--games --seed --players 8,8,8,4 --every 10 [--ai-params]`) | box (`tools/sim/value-net.sh smoke`) or spot (`tools/sim/value-generate.sh LABEL`, wrapping `run-batch.sh --bin value_data`) |
| Train and export | `tools/sim/train-value.py --data DIR... --out crates/ai/data/value_net.json --report FIT.json` (PyTorch, `tools/sim/requirements-value.txt`) | Mac, niced (box once it has `python3-venv`: `value-net.sh train SRC`) |
| Cost | About $1.25 expected (1.4 server-hours at the 10-31 spot price of $0.90 an hour). The cap is 2 servers × 2 h = $3.59 (`--dry-run` figure; run-batch prints the live one) |
| Full-game smoke | `value-net.sh league N`: gene on in half the seats against plain Hard | box |

A row is one alive seat at one sampled turn. It holds the per-team inputs from that seat's view, the hold overlay, the hand estimates and the winner's slot. The layout is in `value-schema.json`, written with every run. Splits are by whole game: seed mod 5 = 0 is held out, and a tenth of the rest is the early-stopping validation slice.

## Smoke (2026-10-31, branch base `0fa889678`; `smoke-2026-10-31/`)

**Games.** 384 Hard self-play games on the box: seeds 8800001 to 8800384, players 8, 8, 8 and 4 in rotation, sampled every 10 turns. Every game had a winner. There were 62,268 rows, 370 bytes each. The games took 41 s on 64 cores: 6.6 core-seconds per 8-player game and 3.7 per 4-player game.

**Fit.** 276 training games and 76 held-out games (12,211 rows). Training took 6 s on 4 threads.

| Held-out estimate | Brier (skill vs uniform) | Log loss | Picks the winner |
|---|---|---|---|
| Uniform | 0.1251 (0%) | 0.415 | n/a |
| Hand formula, as Hard's plan uses it | 0.1667 (−33%) | 0.511 | 42.4% |
| Fitted linear (Search leaves) | 0.1757 (−41%) | 0.526 | 41.0% |
| Linear refit, same inputs | 0.1072 (+14%) | 0.356 | 42.7% |
| Network, integer (what Rust runs) | 0.1061 (+15%) | 0.350 | 43.3% |

"Picks the winner" is the share of (game, turn) positions where the seat with the highest own estimate is the eventual winner.

- **The hand formula is worse than uniform on current master.** It is overconfident: its skill is −80% at T1–60, −32% at T61–120, −17% at T121–200 and +3% after T200. It was fitted on 10-04 rules with skill +25%; VP v2 and later changes have moved under it. The network is +8%, +9%, +17% and +29% by the same phases.
- **Ranking moves much less than calibration.** "Picks the winner" goes from 42.4% to 43.3%. On 76 held-out games, the network's edge over a linear refit is small.

**Cost per decision.** These are native Graviton figures (c8g), on one thread, from six 8-player games per arm.
- One estimate takes 0.8 to 1.9 µs with the hand formula and 6 to 9 µs with the network, about +6 µs at 7 or 8 teams.
- The whole-lobby planning time is unchanged within noise: mean 11.8 to 17.9 ms a turn with the gene on, against 13.8 to 24.7 with it off. The p95 at T150 is 7.8 to 56.8 ms on, against 7.6 to 52.1 off.
- Gene-on lobbies all ended before T230, so there is no T250 window for them.
- Measured plain Hard in WASM on the laptop is 31 ms p95 at T250, out of a 1,000 ms budget. The network adds microseconds per estimate, so the budget is not at risk.

**Full games.** 1,024 8-player Hard games, seeds 8900001 and up. Gene-on seats won 528 of 4,096 (12.9%) and plain Hard seats 496 of 4,096 (12.1%), +0.8 pp. That is inside noise (the seat assignment alone has been worth ±1.2 pp), but nothing broke: no panics, and every game ended in a hold win.

**Risk to watch.** Several genes are fixed charges in chance units, for example `peace_credibility` 300 and `campaign_commit` 300. A calibrated estimate is less confident than the hand one, so its chance differences are smaller and those charges weigh relatively more. The gate has to check behaviour (wars declared, captures), not only wins.

## Full dataset v1 (2026-10-31; for finishing the pipeline, regenerated after the mounted-units rule change)

| | |
|---|---|
| Games | 50,000 (37,500 8-player, 12,500 4-player), seeds 8810001 to 8860000 |
| Rows | About 8.1 M rows, 3.0 GB |
| Compute | About 82 core-hours, 1.3 server-hours of c8g.16xlarge |
| Wall time | About 20 minutes of games on 4 spot servers (12,500 each), plus a few minutes of boot and fetch |
| Cost | About $1 expected. The round-1 cap is 4 × 0.75 h ≈ $2.0 at $0.67 an hour; `--budget` bounds the retries |

One command, which survives spot reclaims:

```sh
tools/ops/bgjob.sh cloud-value-v1 --log .progress/cloud-value-v1.log -- \
  tools/sim/value-generate.sh value-v1 --games 50000 --seed 8810001 --servers 4 --max-hours 0.75 --budget 3.5
```

It builds `value_data` on the box, runs the batch and re-runs every seed range a reclaimed server lost, on c7g, up to 3 rounds. It never starts a batch whose worst case exceeds what is left of `--budget`. The batch directories it produced are listed in `~/LegatumRuns/value-data/value-v1.dirs`. **To regenerate after a rule change,** run it with a new label and a new seed block, for example `value-v2 --seed 8910001`.

The first launch (10-31 01:55 UTC, batch `value-v1-1030-215542`) lost both servers to AWS spot reclaims after 6 and 10 minutes (`StateReason Server.SpotInstanceTermination`, in us-east-1a). A reclaimed server uploads nothing: no rows, no `batch.log`, no `DONE`. That is why the wrapper re-runs lost seed ranges instead of failing.

### v1 results (2026-10-31; `v1-2026-10-31/`)

**Generation.** The batch was `value-v1-1030-223134`: 4 c8g servers, 12.5 minutes of games each, 0.85 server-hours, about $0.6. It produced 50,000 games and 8.22 M rows. 9 games hit the 1,000-turn cap and were dropped.

**Fit.** 35,995 games to fit, 3,997 to validate, and 9,999 held out (1.64 M rows). Training took 111 s on 4 Mac threads.

| Held-out estimate | Brier skill vs uniform | Log loss | Picks the winner |
|---|---|---|---|
| Hand formula Hard uses today | −34% | 0.510 | 40.2% |
| Fitted linear (Search leaves) | −39% | 0.517 | 40.2% |
| Linear refit | +15% | 0.344 | 40.9% |
| Network, integer | +18% | 0.330 | 42.5% |

- **By phase** (network against hand formula, Brier skill): T1–60 +6% against −83%; T61–120 +12% against −36%; T121–200 +23% against −18%; after T200 +35% against +5%.
- **By lobby size:** 8-player +17% against −36%; 4-player +22% against −28%.

**Full games.** 2,048 8-player Hard games. Gene-on seats won 1,057 of 8,192 (12.9%) and plain Hard seats 991 of 8,192 (12.1%), +0.8 pp. That is inside noise.

**Cost.** One estimate takes about 6.9 µs, against about 0.85 µs for the hand formula (native Graviton). Whole-lobby planning at T250 is 31.8 ms p95 with the gene on, against 27 to 29 ms with it off.

### War suite (2026-10-31; `war-suite-2026-10-31/`, gzipped reports)

`tools/sim/war-scenarios.sh --brain hard --runs 64 --no-ledger [--ai-params docs/sim-results/value-net/learned-value-N.json --params-seats seat]`, commit `e0040d7e`, 30 scenarios. Runs pair by index; compare with `war-scenarios-report.py --compare`.

| Arm | Mean goal success | Scenarios at human parity | Paired change |
|---|---|---|---|
| Off (hand formula) | 47% | 13/30 | n/a |
| `learned_value=1` (network) | 48% | 14/30 | +0.5 points. Mongols at Chur T137 +20.3 pp [+9.4, +31.2]; Mongols against the Bomber T150 +4.7 [−3.1, +12.5]; Japan against the Veteran T142 −9.4 [−20.3, +1.6]; every other scenario unchanged |
| `learned_value=2` (refit) | 47% | 13/30 | −0.2 points. Japan T142 −6.2 [−15.6, +3.1]; every other scenario unchanged |

Neither arm drops the war suite. Both Golden Horde attacker cases read 0% on the control too, so the attack check (`--check`, floor 30%) fails on today's base, not because of these genes.

### Decision regret (2026-10-31; `crates/sim/examples/value_regret.rs`, `v1-2026-10-31/regret*`)

**Method.**
- 768 fresh Hard self-play games: seeds 8880001 to 8880768, which are not training seeds, with 8, 8, 8 and 4 players in rotation.
- At T40, T80, T120 and T160, one alive seat's planner lists its plan candidates under each estimate. Each estimate's pick is its top-valued candidate; the planner's hysteresis is left out.
- Where the picks differ, each pick is forced for 20 turns from the true state, then the game is played to the end, 24 times with reseeded dice and the same dice for both picks.
- regret(hand) − regret(b) is the win rate of b's pick minus the hand pick's, and 0 where they agree.
- The run used 0.38 spot server-hours, about $0.25. Summarize with `tools/sim/value-regret-report.py`.

**Results.** 3,069 positions.

| Against the hand formula | Picks differ | Regret saved per position | Where they differ |
|---|---|---|---|
| Network (`learned_value=1`) | 17.8% | +0.04 pp [−0.05, +0.14] | +0.2 pp |
| Refit (`learned_value=2`) | 7.0% | −0.00 pp [−0.05, +0.05] | −0.0 pp |

- **The network sues for peace more.** 344 of its 547 disagreements are the hand formula choosing Contain where the network chooses Peace, and 89 are the reverse.
- **The pick rarely changes the outcome.** At disagreements the two picks' win rates differ by 3.1 pp on average. In 299 of the 547, both picks win 0 of 24, because the seat loses either way.

**Verdict.** Against Hard opponents, the better-calibrated estimate does not lower plan-choice regret. The go/no-go "lower regret than the hand formula on held-out positions" is not met: the result is equal, with a tight interval. The plain-Hard plan choice that the estimate controls is not where Hard loses games. This agrees with step 2 being neutral in full games.

run-batch fetches the rows to `~/LegatumRuns/cloud/BATCH/slice-*/out/`. Train on the Mac, niced, on 4 of its 16 cores (`--data-list ~/LegatumRuns/value-data/value-v1.dirs` in place of `--data`). The v1 fit (8.2 M rows, `--batch 8192`) took 111 s:

```sh
python3 -m venv ~/LegatumRuns/value-venv && ~/LegatumRuns/value-venv/bin/pip install -r tools/sim/requirements-value.txt
nice -n 15 ~/LegatumRuns/value-venv/bin/python tools/sim/train-value.py --data-list ~/LegatumRuns/value-data/value-v1.dirs --threads 4 --batch 8192 \
  --data-label "value-v1: 50,000 Hard self-play games from seed 8810001, players 8,8,8,4, every 10 turns" \
  --out crates/ai/data/value_net.json --report docs/sim-results/value-net/fit-v1.json
```

The box can't train yet. It has no `python3-venv` and no pip, and its role can read only the bucket's `box/` prefix, not `batches/`. Installing `python3.12-venv` on the box is a box configuration change for the lead to decide. After that, `value-net.sh train` runs there on a copy of the rows under `box/`.

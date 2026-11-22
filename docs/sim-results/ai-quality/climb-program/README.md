# The AI climb program

## Round 2 diplomacy and war candidates (2026-10-24 audits)

`tools/evolve/genesets/climb-r2.json` extends the current gene set for the next directed round. It does not change any faction default or restart the running round. The director should gate these on the human-seat benchmark and war suite as well as the arena.

- **Fable 12, Germany attacker:** `conquest_enabled`, `siege_exempt`, `campaign_contain_only`, `scout_front`. The first two are already in round 1; the latter two are added for round 2. Fixture: Germany with a funded army and reachable rival city must have a legal campaign and siege when genes are set to 1, 0, 0, 1 respectively. Cloud war scenario: at least half of Germany attackers declare by turn 20 and attempt a city capture. Compare against unchanged built-in Germany values.
- **Fable 15, dormant commitments:** `leader_commit`, `war_pressure`, `war_garrison`, `peace_retaliation`, `settler_safety`, `peace_scout_danger`. `war_pressure` is already in round 1. Keep all six built-in defaults; test combinations through the same gates and report declarations followed by combat, peace followed by rival re-declaration within 10 turns, and avoidable Settler captures.
- **This bundle's new A/B genes:** `second_front_siege_price`, `rival_redeclare_percent`, `peace_withdraw_price`, `ally_defense_percent`, `ally_vision_cost`, `ally_passage_cost`, `ally_exit_discount_percent`, `leader_peace_horizon`, `ally_commitment_model`, `ally_offensive_join_percent`. Zero on the behavior switches restores the old path; `ally_offensive_join_percent` sets the probability within the new participation model. Keep the older six commitment genes at their current defaults until the climb selects values.

The user, 2026-10-23: "you have a bunch of hard-coded rules and you're not iterating or exploring them. That's probably not good. Also, you seem unable to replicate even my level of success... iterate on the AIs on our 64 cpu box until they at least get to the performance that I got to in terms of points and stuff and win percentage, then use that winning model to compete against the other winning models." And: "be sure to have incremental progress you can bank and communicate to me to ensure that if we stop at some point before full completion we don't lose everything."

This page is the program's plan, its rules and its live log. The scheduled task `legatum-climb-director` (every 6 hours) reads it top to bottom, does the "Director checklist" below, and adds a line to the progress log.

## Status (director, 2026-10-24 13:10)

Per faction: the current master default against the target, on 4-player Small held-out games. Banked rows show the arena gate's 1,024 held-out games (seeds 950001+), master before → banked after. Cities are the mean at T100. **Round r3 (all eight factions, transfer objective, master `084f45c31`) is running**: it is Celts' third round and everyone else's second.

| Faction (r1 folder) | Win rate now (held-out) | His target | Cities T100 now (target) | Rounds | Last banked | Last gate |
|---|---|---|---|---|---|---|
| Athens (Signalwrights) | 38.4% (was 30.6%) | 50% | 4.3 (5.5) | 1 + r3 | r1, 2026-10-24, +7.8 pp | passed all three |
| Byzantium (Remnant) | 42.1% (was 37.5%) | 80% | 4.5 (6) | 1 + r3 | r1, 2026-10-24, +4.6 pp | passed all three |
| Switzerland (Tollkeepers) | 28.8% (was 25.3%) | 83% | 4.1 (13) | 1 + r3 | r1, 2026-10-24, +3.5 pp | passed all three |
| Germany (Crucible) | 25.3% (was 22.0%) | 0% (he went 0/2); cities miss | 3.9 (5) | 1 + r3 | r1, 2026-10-24, +3.3 pp | passed all three |
| Japan (Choir) | 14.1% (master) | 80% | 4.8 (8) | 1 + r3 | none | r1 rejected: opponents-only +1.9 pp (limit +1.0) |
| Mongols (Glassroad) | 27.3% (master) | 80% | 4.9 (8) | 1 + r3 | none | r1 rejected: war check 54% → 0% |
| Sparta (Oathbound) | 24.9% (master) | 100% | 3.9 (8) | 1 + r3 | none | r1 rejected: opponents-only +1.6 pp (limit +1.0) |
| Celts (Veyr) | 9.8% (master, r2 gate seeds) | 100% | 4.2 (7.5) | 2 + r3 | none | r2 rejected: arena −0.6 pp [−2.9, +1.8] |

No faction is at target. None has plateaued, but **Celts plateau if r3 fails its gate** (two rounds without a gated gain so far); Japan, the Mongols and Sparta have one. No climb has raised cities at T100 yet: the cities term may need more weight or new expansion genes.

## Progress log (newest first)

One line per event: a round started or ended, a candidate gated, banked or rejected, a faction plateaued (with its structural question), a league result. Plain English; numbers are held-out unless said.

- **2026-10-24 13:10** Director. **Banked round 1 for Athens, Byzantium, Switzerland and Germany** (master `a41204c18`; ledger rows already in `ledger.jsonl`). Held-out arena wins, master → banked: Athens 30.6% → 38.4% (target 50%), Byzantium 37.5% → 42.1% (80%), Switzerland 25.3% → 28.8% (83%), Germany 22.0% → 25.3% (cities target 5 at T100 not met).
  - Landed Codex `climb-bank-r1-land` (`77305485e`, merged with master): `supply::within_budget` now budgets Siege upgrades' Iron with unit upgrades (fixes Hard's invalid Siege order at turn 138), and hand-built AI tests use fixed `fixture::TEST_KNOBS` instead of a faction's banked defaults. Box verify passed: 1,888 tests, no pins moved, 50-game smoke clean. **Lead's call on the behaviour change:** banked Byzantium builds Granaries rather than the Heir in `goal_forge_heir`'s hand-built position; accepted, because the full-game gate (arena +4.6 pp, benchmark and war checks) is the evidence that counts. The test now pins the old genes to check Heir assignment.
  - Landed Codex `climb-objective-transfer` (master `084f45c31`, box verify passed twice): new rounds use the transfer objective (paired opponents arm, attack term for Mongols, Sparta, Germany and Celts; 1.5× games).
  - **Celts round 2 rejected** at the arena gate: −0.6 pp [−2.9, +1.8] (9.2% vs 9.8%), fitness −0.006. Two rounds without a gated gain.
  - **Round r3 started for all eight factions** (master `084f45c31`, gene set `climb-r1`, transfer objective, seeds 50000001+); banked factions climb from their new defaults. One daemon (pid 51205): `start` launches a daemon itself, so a following `ensure` races it and starts a second; I stopped the duplicate before it submitted anything.
  - Spend: AWS month to date within the monthly cap; program spot $17.49 of $25.
- **2026-10-24 07:20** Director. **Benchmark and war checks finished for the five arena passers (master binaries, `r1/gate-r1b.log`): four pass, Sparta fails.**
  - Pass: Athens seat-only +0.0 pp (no Athenian human seat in the benchmark), opponents-only −3.0 [−4.8, −1.4]; Byzantium +0.0 / −1.6 [−3.3, +0.1]; Switzerland +1.3 [−0.0, +2.7] / −1.4 [−3.3, +0.3]; Germany +0.6 [−0.4, +1.6] / −0.5 [−2.0, +0.9]. War check exit 0 for all (Golden Horde thin 53%).
  - **Sparta rejected:** opponents-only +1.6 pp [+0.2, +2.9], past the +1.0 limit (the same transfer failure as Japan). Seat-only +0.0, war exit 0.
  - **Banking blocked by tests.** Branch `climb/bank-r1-athens-byz-swiss-germany` (commits `0bfffddd5`, `4fff86caf`: the four factions' `BUILTIN_FACTION_DECISIONS` and Knobs lines built against master's `params.rs` with gene aliases mapped, repinned, four ledger rows) fails the box verify with 19 legatum-ai unit tests. Most assert Switzerland's old Hard defaults, because `crates/ai/src/fixture.rs` makes the test player Switzerland. Without the Swiss lines four still fail: `era_research` exposes a latent AI bug (Hard orders Siege with Iron held for a University in progress: invalid orders at turn 138), the difficulty test can't see Hard exceed Byzantium's banked military knob of 100, and banked Byzantium builds Granaries instead of the Heir in `goal_forge_heir` (a behaviour change for the lead to judge). Queued Codex task `climb-bank-r1-land` (Sol) to fix the bug and decouple the tests without touching the banked values; the next director run reviews and lands it, then starts round 2 for these four from the new defaults.
  - **Objective change queued:** Codex task `climb-objective-transfer` (Sol) adds an opponents-vs-proxy arm and a war term to the climb objective, for Japan, the Mongols and Sparta.
  - **Celts round 2 finished** at generation 12 (best held-out at gen 4, fitness gain +0.014; last check wins 8.8% vs 8.4%). Arena gate queued on the filler (`gate-r2-celts-1024064748`).
  - Spend: AWS month to date within the monthly cap; program spot $17.49 of $25. Daemon restarted by `ensure` (it exits when nothing runs).
- **2026-10-24 00:50** Director. **Six more arena gates resolved; five passed:** Sparta +9.3 pp [+5.7, +12.9] (24.9% → 34.2%), Athens +7.8 pp (30.6% → 38.4%), Byzantium +4.6 pp (37.5% → 42.1%), Switzerland +3.5 pp (25.3% → 28.8%), Germany +3.3 pp (22.0% → 25.3%), fitness gains all positive. **Celts rejected** (+2.6 pp [−0.3, +5.6], fitness lower bound −0.0001).
  - The five passers' benchmark (seat-only, opponents-only) and war checks run as bgjob `cloud-gate-r1b-hvh-war` on **master** binaries (worktree `.claude/worktrees/climb-gate-r1b`, commit `3ca727e95`): three spot servers at a time, then the war checks one by one. Per-faction output, with the `seat-only`, `opponents-only` and `war-check exit N` lines, is in `~/LegatumRuns/climb-program/r1/<Faction>/gate-hvh-r1b.log`; the summary is in `r1/gate-r1b.log`.
  - **Japan's and the Mongols' rejections stand.** A rerun on the round-1 binaries (`gate-hvh-r1-*-1023-213457`) read differently (Japan seat-only +2.6, opponents-only −2.1), but on that code the war check reads 0% whatever the params (the Mongols never declare in 61 to 64 of 64 runs), so round-1 binaries can't gate. Gates run on the code a candidate lands on.
  - **Round 2 started for Celts** (`r2/Celts`, master runner, commit `3ca727e95`, gene set `climb-r1`, new seeds 30000001+), on the box filler. Japan and the Mongols wait: rerunning the same objective would likely repeat the transfer failure and spend plateau rounds. The objective needs a war-scenario term and an opponents-vs-human-proxy term first (code work, not done in a director run).
  - Spend: AWS month to date within the monthly cap; program spot $16.83 of $25, plus about $6 for the five benchmark servers.
- **2026-10-24** Banking gates for round 1. **Neither Japan nor the Mongols banks.**
  - **Japan (Choir):** passed the arena check (27.4% vs 14.1% wins over 1,024 games) and the human-seat seat-only arm (+0.8 pp [-0.0, +1.6], 2,048 games) and the war check (53%). It failed the opponents-only arm: the human seat wins +1.9 pp [-0.1, +3.9] against tuned Japanese opponents, past the +1 pp limit, so tuned Japan is softer against a human-style player.
  - **Mongols (Glassroad):** passed the arena (34.3% vs 27.3%), seat-only (-0.4 pp) and opponents-only (+0.2 pp). It failed the war check: Golden Horde thin went 54% → **0%**, so the tuned genes stop the Mongol attack outright.
  - **Lesson:** the 4-player-Small arena objective doesn't transfer to the human-seat benchmark or the war scenarios. Round 2 should add a war-scenario term (or fold the check into the fitness) and an opponents-vs-human-proxy term before climbing further.
  - Results: `~/LegatumRuns/cloud/gate-hvh-r1-{choir,glassroad}-1023-211900`.
- **2026-10-23 21:30** Round 1 finished for all eight factions (Celts last, at 21:25). The Japan and Mongols arena gates **passed** (1,024 held-out games: Japan 14.1% → 27.4%, gain +13.4 pp [+10.3, +16.6]; Mongols 27.3% → 34.3%, +6.9 pp [+3.5, +10.5]; fitness gains positive). Their benchmark (`gate-hvh-r1-choir`, `gate-hvh-r1-glassroad`, spot) and war checks run as bgjob `cloud-gate-r1-hvh-war` on the round-1 code (binaries built from the `climb-r1` worktree, commit `23a6588cb`). Output, including the `hvh-delta` lines and `war-check exit N`, goes to `~/LegatumRuns/climb-program/r1/{Choir,Glassroad}/gate-hvh.log`. Arena gates were queued for the other six factions (filler, ids `gate-r1-<faction>-10232126..`); their results appear in `status`. Round 2 waits for these gates, so it can start from the banked defaults. Spend: AWS month to date within the monthly cap; program spot $14.21 of $25. The earlier daemon had exited on its own at 21:25 (nothing left running); the round-1 runner's `ensure` started a fresh one (pid 17744).

- **2026-10-23** Legacy faction names renamed on master (6c0b1c3ca). **Round 1 keeps running on the old code:** the daemon (pid 48803) was started before the rename, its state folders are `r1/Choir`, `r1/Veyr` and so on, and its binaries (commit `23a6588cb`) only know the old names. Don't restart it with today's `climb_program.py` (`ensure` included) until round 1 ends. If it must restart, run `climb_program.py` from a checkout of `23a6588cb`. Round 2 starts on current master with the real names. Gates for Japan (Choir) and Mongols (Glassroad) were queued at 08:51 on the old code. Banking writes real names into `BUILTIN_FACTION_DECISIONS`, and gene aliases accept the old ones.
- **2026-10-23** Fixed a box filler bug that stalled round 1's first two hours: a batch paused while the box was busy was never reaped after it finished (`tools/cloud/box/box-filler`, learnings 2026-10-23). Five gen-0 batches went to spot meanwhile.
- **2026-10-23** Round 1 started: all eight factions, gene set `climb-r1` (76 genes), commit `23a6588cb`, on the box filler with spot fallback. Nothing banked yet. First generation (Japan, 160 games per arm): master wins 17.5% of 4-player Small games as Japan against his 80% target.
- **2026-10-23** Stage 2 landed: the user's targets from 24 recorded games ([`targets.md`](targets.md)): 13 wins of 17 finished games (76%), 8 of 10 on 4-player Small maps.
- **2026-10-23** Stage 1 landed: 101 hard-coded gates are now genes, defaults unchanged (pins identical); inventory in [`gates.md`](gates.md).
- **2026-10-23** Stage 0: this program, the banking rule, `BUILTIN_FACTION_DECISIONS` (per-faction defaults in `crates/ai/src/params.rs`) and the `climb_arena` objective.

## Goal and stopping condition

**Goal.** On a 4-player Small map (40x26, barbarians on, every AI Hard), each faction's Hard AI in a seat reaches the user's milestones for that faction and wins at least his win rate against the field. Then a league: every faction's best plays the others' bests, and the hall of fame (each faction's champion) becomes the field the next climbs must beat.

**The targets** ([`targets.json`](targets.json), rebuilt by `python3 tools/ai-quality/climb-targets.py` after each playtest): his win rate for the faction when he finished at least two games with it, else his 4-player Small rate (80%); and his median cities at T50 / T100 / T150 for that faction, falling back to his 4-player Small medians where he has no game that long.

| Faction | His win rate (finished games) | Cities T50 / T100 / T150 used |
|---|---|---|
| Sparta (Sparta) | 3/3 | 5 / 8 / 10 |
| Switzerland (Switzerland) | 5/6 | 5 / 13 / 6 (mixed bases, see targets.md) |
| Celts (Celts) | 2/2 | 4 / 7.5 / 10 |
| Japan (Japan) | 1/1 (uses 80%) | 5 / 8 / 10 |
| Mongols (Mongols) | 1/1 (uses 80%) | 4 / 8 / 10 |
| Athens (Athens) | 1/2 | 3 / 5.5 / 8.5 |
| Germany (Germany) | 0/2 | 2.5 / 5 / 10 |
| Byzantium (Byzantium) | none finished (uses 80%) | 4 / 6 / 10 |

Today's built-in Hard wins about 22% of these games as Sparta (fair share 25%), against his 100%.

**Stopping condition, per faction.** Stop climbing a faction when either (a) on the held-out arena games its banked default wins at least its target rate and meets the T50 and T100 city medians, or (b) the plateau rule fires. The program ends when every faction has stopped and the final league is written up, or when the user redirects.

## The rules

**Plateau rule.** A *round* is one climb of one faction (one CMA-ES run, up to 24 generations, stopping early after two held-out checks in a row without a new best). A round has a *gated gain* when its best candidate passes the banking gate. **Three rounds in a row without a gated gain: stop climbing that faction** and write the structural question to the progress log: which behaviour or knob is missing (read the round's logs and a few of its games; the coach's patterns in `.claude/skills/legatum-coach/SKILL.md` and `docs/sim-results/ai-quality/optimization-program.md` section 4 list the known gaps). That question goes to Claude or the user as code work; the faction's climbing resumes only when a new gene for it lands.

**Banking rule.** Any candidate that beats the current master on the full-game gate lands on master as a default change, in the same day, with a ledger row and a progress-log line. Nothing lives only in a run directory. The gate, every part on games the climb never saw:
1. **Arena, held-out maps and salts:** `climb_program.py gate` plays the candidate and master on 1,024 fresh 4-player Small games (seeds 950001+: new maps, new seats, new rivals, new dice). Pass: win-rate gain above 0 with the 95% interval's lower bound above 0, and the fitness gain also positive.
2. **Human-seat benchmark, seat-only and opponents-only** (the eight recorded maps, 256 salts 900001+, own dice per salt). Pass: seat-only no worse than -1.0 pp with an interval that includes 0 or better; opponents-only no higher than +1.0 pp (the candidate must not make the AI rivals weaker).
3. **War suite attack check:** `tools/sim/war-scenarios.sh --check --ai-params best.json --params-seats all` exits 0.

A candidate is faction-scoped (it only sets `faction_overrides.<Faction>` and that faction's knobs), so it lands as that faction's lines in `BUILTIN_FACTION_DECISIONS` (`crates/ai/src/params.rs`) and its `Knobs::new` line in `crates/ai/src/knobs.rs`. A gene that every faction's banked value moved the same way may later become a shared default, through the same gate.

## How the climber works

- **Objective** (`crates/sim/examples/climb_arena.rs`): generated 4-player Small games, the climbed faction in seat `seed % 4`, three rivals drawn from the other seven factions, everyone Hard (plain, no search), played to the end. Every arm plays the same seeds, so candidate minus master is paired. A game scores 0.65 for a win, plus 0.2 times VP over the best rival's VP (capped at 1), plus 0.15 times the mean of min(cities ÷ his median, 1) at T50, T100 and T150. About 7.6 core-seconds a game.
- **Transfer objective for new rounds** (`start --objective transfer`, the default, stored in each `state.json`; `--objective arena` reproduces round 1): each arm also plays half as many paired opponent games. The same lobby and seed move the climbed faction one seat clockwise, leaving a master-played rival in the observed seat; only the climbed faction gets the arm's parameters. If that observed seat wins more often than against a master climbed-faction rival, its excess win rate costs twice that rate in fitness. For Mongols, Sparta, Germany and Celts, the seat games also penalize declines in attacks: a war declaration by the climbed team and a city capture (including a Celtic sack) each count half a point per game, capped at one of each; lost points against master cost 0.3 per point. These are training proxies, not substitutes for the benchmark and war banking gates. The extra opponent games take the planned per-generation simulations from 16 × 160 to 16 × (160 + 80), exactly 1.5×; attack tracking adds no games. Held-out checks use the same objective and game ratio when choosing a best, while the banking gate remains unchanged.
- **Optimizer** (`tools/evolve/climb_program.py`): CMA-ES (`tools/evolve/cmaes.py`) over the faction's genes from the gene set (`tools/evolve/genesets/climb-r1.json`: the five knobs, the 44 decision genes of the 10-21 climbs and 29 former gates from stage 1), starting at the faction's built-in values. Population 14, plus the CMA mean and master as arms; 160 fresh games per arm per generation; a held-out check (mean against master, 512 games, seeds 900001+) every 4 generations; `best.json` is the mean at the best held-out check.
- **Where it runs:** every batch goes to the cloud box's idle filler (`tools/cloud/filler.sh`, priority 40, lowest CPU weight so verifies win). A batch the filler hasn't started after 90 minutes moves to one AWS spot server (at most 3 at once: the 448-vCPU spot quota is shared) (`tools/cloud/run-batch.sh --bin climb_arena`) only while the program's spot budget lasts ($25, in `~/LegatumRuns/climb-program/program.json`) and the month's AWS spend is under a guard set below the monthly cap (`LEGATUM_MTD_GUARD_USD`, read with `LEGATUM_AWS_ADMIN_PROFILE`; both come from the environment or `.private/cloud.env`, and if either is unset or the check fails, no spot). A generation is about 5 minutes of the whole box; eight factions take turns.
- **Unattended:** the daemon (`climb_program.py daemon`, run under `caffeinate -i` by `ensure`) steps every running climb once a minute and saves `state.json` after every step, so a restart of the Mac, the daemon or the box loses at most the batch in flight: filler batches killed by a box restart are rerun by the filler itself, batches that end with rc=143 or another failure are resubmitted (up to 4 times). `ensure` is idempotent; the director calls it every run.
- **Rounds after round 1** (the director's choice, written here when made): a faction whose round passed the gate starts its next round from the new master defaults; one that didn't gets the next gene set (economy and research genes, then the remaining stage 1 genes, then whatever its plateau note asks for) or a new seed range. From round 2 the rivals can be the hall of fame (`start --field hall.json`, the league file of the latest bests), so a candidate must beat the other winning models, not just master.
- **League** (`climb_program.py league --round R`): every faction's best of the round in every seat at once (one `faction_overrides` file), against master in every seat, on 512 games per faction. It reports each faction's win rate among the bests; it runs after a round has a best for every faction.

## Commands

Everything below runs from the main checkout; nothing runs simulations on the Mac.

```sh
# Status: one line per climb (generation, mean against master, held-out gain, what is in flight)
python3 tools/evolve/climb_program.py status
tail -20 ~/LegatumRuns/climb-program/daemon.log
python3 tools/evolve/climb_program.py ensure            # restart the daemon if it died (idempotent)
tools/cloud/filler.sh status                             # the box queue

# Per-climb results: log.jsonl (every generation and held-out row), best.json (an AiParams file), state.json
ls ~/LegatumRuns/climb-program/r1/Sparta/

# The banking gate for one finished climb
python3 tools/evolve/climb_program.py gate --round r1 --faction Sparta           # 1. arena (filler); result lands in status
LEGATUM_CLOUD_EXAMPLES=hard_vs_human tools/cloud/build-linux.sh                     # 2. benchmark binary, once per commit
tools/cloud/run-batch.sh --count 1 --max-hours 3 --yes --bin hard_vs_human \
    --ai-params ~/LegatumRuns/climb-program/r1/Sparta/hvh-arms.json --env LEGATUM_HVH_RESEED_DICE=1 \
    -- --arms --games 256 --seed 900001 --label gate-hvh-r1-sparta              # prints "results: DIR"
python3 tools/evolve/climb_program.py hvh-delta DIR                                  # seat-only and opponents-only deltas
tools/sim/war-scenarios.sh --check --ai-params ~/LegatumRuns/climb-program/r1/Sparta/best.json --params-seats all

# Bank a candidate that passed all three
python3 tools/evolve/climb_program.py bank --round r1 --faction Sparta   # prints the params.rs and knobs.rs lines
# Then in a fresh worktree: paste the lines, repin, run the box verify (legatum-workflow skill), add a row to
# docs/sim-results/ai-quality/ledger.jsonl and a progress-log line here, merge master, and land with
# `git merge --ff-only` in the main checkout (explicit-path commits only).

# League once a round has bests; a new round
python3 tools/evolve/climb_program.py league --round r1
python3 tools/evolve/climb_program.py start --round r2 --factions Sparta,Japan --geneset climb-r2 --commit master
python3 tools/evolve/climb_program.py stop --round r1 --faction Celts
```

## Director checklist (every 6 hours)

1. `climb_program.py ensure`, then `status`. If a climb shows `failed`, read the last `error` row in its `log.jsonl` and `daemon.log`; fix or restart it (a new round id), and log the event.
2. For each climb that finished since the last run: run the gate's three parts. Pass: bank it (above), log "banked". Fail: log "rejected" with the failing part and number; that round counts toward the faction's plateau.
3. Plateau rule: a faction with three rounds in a row without a gated gain stops; write its structural question in the log.
4. Queue the next round for factions that are not stopped (new master after banking; a new gene set after a failure), and the league when a round has bests for every faction.
5. Check spend: the program's spot spend is in `status`; the month's AWS spend with `aws --profile <admin-profile> ce get-cost-and-usage` (the program stops using spot when the month nears its cap).
6. Tell the user what was banked, in plain numbers (held-out win rate before and after, against his rate), and what's running.

## Files

- [`targets.json`](targets.json) and [`targets.md`](targets.md): his milestones (stage 2).
- [`gates.md`](gates.md): the inventory of hard-coded gates turned into genes (stage 1).
- `tools/evolve/climb_program.py`, `tools/evolve/genesets/climb-r*.json`, `crates/sim/examples/climb_arena.rs`: the climber and its objective.
- `~/LegatumRuns/climb-program/<round>/<Faction>/`: live state, logs, batches and `best.json` (regenerable working files; anything that passes the gate lands on master).
- The earlier programs this builds on: [`../optimization-program.md`](../optimization-program.md) (methods, structural gaps) and [`../climb-1021/README.md`](../climb-1021/README.md) (milestone objectives failed the full-game gate; a full-game objective on one map overfit).

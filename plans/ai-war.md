> **2026-11-07: strategy reset (designer approved, accelerated).** The plan now follows `docs/design/ai-strategy/fable-2026-11-07.md`. The headline metric is the **designer-seat score**: Hard taking over his seat on his own held-out tapes (`docs/status/designer-seat.md`; control 2026-11-07: held out 57.4%, training 44.4%). AI-only censuses are kept only for the faction-balance ratio. The method is operations found by offline search, judged by the learned value model and executed by the scripted executors. There's a rules freeze after the Byzantium, Sparta-desertion and Germany-discount changes land. Increments 0 and 1 started 11-07 in parallel. Increment 1's first operation-search skateboard (`claude/operation-search`, `docs/sim-results/ai-quality/operation-search-1107/`) failed its kill check twice (2 of 22 and 5 of 22 windows confirmed, mark 30%; 0 of 21 on the Celts tape): named operations rarely become the plan, so the generator must be fixed (offline forced campaigns) before increment 2; the checkpoint target moves earlier than 11-27. Increment 1b (`claude/operation-executor`, gene `operations` off) built a committed operation executor: strikes now take in 97% of samples, but executed strikes lose and the kill check fails a third time (3 of 38 windows); the next generator varies strike timing and roster, not targets.

# Plan: a Hard AI that can beat the designer at war

Started 2026-10-27. The tracked checklist is `plans/ai-war.prd.json`, and the session log is `plans/ai-war-progress.txt`. The research behind this plan is `docs/design/research/ai-strategy-options-2026-10-27.md`.

All times are real wall-clock time with agents and cloud compute. Each date names the bottleneck that sets it.

## Goal and finish line

Hard is good enough that the designer usually loses to it, without cheating (pillar 6). The finish line is measured three ways, and all three must hold:

1. **Human-seat benchmark:** Hard playing the designer's seat wins 50% or more on the eight benchmark maps. Today it wins 23.0% against the app's real Hard rivals (see `docs/status/state-of-the-ai.md`; 16.2% on 10-28; the old 32–39% was against Search rivals that never bombed). The benchmark is eight maps, one 2-player and seven 4-player (`crates/sim/data/hard-vs-human-maps.json`), so an equal-seat reference is (0.5 + 7 x 0.25) / 8 = 28.1%, not 12.5%. It is a reference, not a prediction: factions and starts are asymmetric. Today's Hard sits well below it, which matches the designer's experience. Target kept at 50% (designer, 10-28).
2. **War suite:** in the attacker scenarios, the AI matches the designer's own result in at least half the cases. Today it scores 0–3% on most of them.
3. **The designer's games:** he loses at least two of four fresh Hard games, and the losses come from wars he saw coming but couldn't stop.

The gap is running a war: getting an army to arrive together, fight as one, keep its best units in the fight, use its era's units and aircraft, and finish a siege. More tuning won't close it. The CMA-ES tuning program has plateaued, and perfect information was worth nothing when measured. Better decisions at the campaign level will: picking the best of 8 plans per game wins 66% instead of 28%.

## How we make sure this lands

- **Every increment ends in a playtest build on a Pages preview**, with release notes saying what to look for. No increment is done until it's been played.
- **Every AI change sits behind a gene** (off by default). It lands only through the three gates:
  - the frozen-control human-seat gate;
  - the war suite and attack check;
  - the release check.
- **The designer's dashboard** (user, 2026-10-27) is `plans/ai-war-dashboard.html`, generated from `plans/ai-war-status.json` by `python3 tools/ai-plan/dashboard.py`. Update the status JSON and regenerate it in the same commit as every progress-log entry: step progress, finish-line numbers, spend, learnings and playtests.
- **Each step has a go/no-go number written before the work starts** (below). A step that misses it gets diagnosed, not quietly retried.
- **The lead updates `plans/ai-war.prd.json` and the progress log every session.** The ops check reads both and flags a stalled story.
- **Stop rule:** if two rounds of the same idea fail their gate, stop. Write down why in `docs/learnings.md` and change approach, rather than tuning around it.
- **Rule changes:** the freeze was lifted on 2026-11-11 (designer: "lift the freeze and add the things that are backlogged"). Rule changes land in batches, one `RULES_VERSION` per batch (27: relief chain, rival-attacker tie-breaker, aircraft XP, occupation lock off by default), and the designer-seat score and fixtures are re-baselined once per batch. Before the lift: combat, air, siege and VP rule changes waited until the checkpoint. Each one invalidates the fixtures and the gates. UI and art work carries on.

## Increments (skateboard first)

| # | Increment | What the designer can try | Go / no-go | Target date | Set by |
|---|---|---|---|---|---|
| 0 | **Baseline and fixtures** | Nothing new in play; 8.9.13 installed | A gate that reproduces exactly; 7 new fixtures with written AI baselines; the already-built fixes ranked | Fixtures Wed 10-28; numbers Wed 10-28 if AWS is unblocked, else Sun 11-01 | AWS cap |
| 1 | **Campaign planner (skateboard)** | Hard that commits, gathers or withdraws; keeps its best units at the front; bombs *and* captures | War suite: ≥ +15 pp capture-by-deadline on the new fixtures; Golden Horde ≥ 40/64; seat-only ≥ +3 pp; opponents-only ≥ −3 pp | Playtest build Sat 10-31 to Mon 11-02 | Build-test-fix rounds (about half a day each) |
| 2 | **Battle search** | Armies that focus fire, kill the exposed shooter, screen weak units, wait for stragglers | Better exchange ratio on held-out engagements; the 53:3 fixture; whole-lobby turn p95 under 1 s in WASM on the laptop | Built alongside 1; playtest build Wed 11-04 to Fri 11-06 | Rounds, plus the WASM speed budget |
| 3 | **Learned evaluator and strategy bundles** | Same behaviours, better choices under pressure; Hard visibly commits to a plan (air war, early conquest) and switches when it fails | Lower regret than the hand formula on held-out positions; +3 pp seat-only; no war-suite drop | Playtest build Mon 11-09 to Thu 11-12 | Training data from the box, Codex back 11-02 |
| ✔ | **Checkpoint** | Two to four fresh Hard games | The three finish-line measures | Around Fri 11-13 to Wed 11-18 | Designer's playtests (a few a week) |
| 4 | **RL controllers and an exploiter league** (only if the checkpoint calls for it) | Counters to the designer's own plans; several strong styles per faction | Exploiters beat Hard from his seat, then Hard beats the exploiters | About 1–2 weeks after the checkpoint | Training rounds |

These dates hold if the first designs mostly work. History says many won't: the CMA-ES tuning failed 12 of 12 candidates, and the staging-ring fix took two rounds. Each failed round costs about half a day, so a bad week moves the dates out by days, not months.

## Adjustment after W1 (2026-10-27)

W1's campaign planner (`claude/ai-operation-planner`, 6b33c8896, gene `operation_planner` off) works but moves the war suite only +3.5 pp. Swiss T220 goes from 23% to 65%, nothing else regresses, and the bleeding in hopeless attacks is gone. Its marks are only partly met: +14 pp against the +15 needed, and the AWS gate is still to run.

The replay case showed the bigger cause. Against the designer's own Athens, the Swiss army (Spearmen and Archers against a walled city at T180) could never win. **Army quality comes before operations.** So the next increment adds army quality alongside battle search: turn on and measure 8.9.13's off-by-default modernization genes (upgrade prioritization, return routing), let upgrades and production keep the army within an era of its targets, and time wars for when the army is ahead. **Air stage (user, 2026-10-27):** Bombers stay as strong as they are, so the AI must contest the air. W2 includes valuing Flight when a rival bombs within reach, building and stationing Fighters (`air_defense`), and defending Great Work cities by their VP. That game's coach review (`docs/playtests/2026-10-27-athens-bombers-review.md`) also adds "commit to the leader's softest city" for the strongest neighbour.

**Two coach reviews on 10-27 (Athens and Sparta; `docs/playtests/2026-10-27-*-review.md`) agree on W2's priorities:**
1. **Unit upgrades on** (`modernization_upgrade`). It's the one measured lever in both games:
   - Athens' win from T150 drops from 31 to 26 of 32;
   - Hard Sparta from T1 drops from 7 to 2 of 32.
2. **Defend where the enemy army is,** with the era's best unit, upgrades bought with Gold, and counterattacks on wounded attackers. This is idea #23.
3. **The strongest neighbour commits** to the VP leader's softest empty city (idea #35).
4. **AIs at war with the same leader accept each other's Alliance.**
5. **Contest the air** (#36).

A bigger army quota backfired against an adaptive Sparta, so the fix is better defence, not more units.

The first measurement is the AWS human-seat gate with these arms:
- the planner;
- the modernization genes;
- the five branch fixes;
- all of them combined.

## Increment 0 in detail (first step)

1. **Land 8.9.13** (Codex finished it 10-27: gate noninferior +1.0 pp on 4,096 games, release check passed) and install it. It becomes the frozen baseline for everything below. *About 1 hour, Claude.*
2. **Seven fixtures from the designer's saves** (in `~/LegatumRuns/bug-saves/`), each with a goal, a deadline and his own result:
   - Swiss T180, T210 and T220, from the Athens game, where the Swiss army sat tepid at his border (`athens-win-swiss-tepid-20261025`);
   - German T214 and T219, from the Japan bomber game (`japan-win-bombers-20261026`);
   - Japan T142, from the Sparta game (`sparta-crushing-japan-20261025`);
   - the Chur ring, T143 (`staging-ring-chur-20261026`, re-run at Hard, since that game ran at Normal).

   They go in `scenarios/war/` with the existing format, and each gets a written prediction before its first run. *About 2–3 hours, 2 agents in parallel.*
3. **Measure the speed budget:** turn time per AI seat per turn, native and in WASM, at T50, T150 and T250. This sets how much search the live AI can afford. *About 1 hour, 1 agent, Mac (short runs).*
4. **Rank the fixes already sitting on branches:**
   - `astra/fix-staging-ring-r2`;
   - `astra/ai-local-opportunism`;
   - `astra/ai-air-defense-r2`;
   - `astra/search-air-execution`;
   - `astra/gold-usage-diagnosis`.

   Each runs as an arm on the new fixtures and the frozen-control gate. The winners become the planner's starting point, and the losers get deleted. *About 1–2 hours of compute, needs AWS.*
5. **The baseline report:** today's Hard on all fixtures, its benchmark score and the speed numbers, written to `docs/sim-results/ai-war/baseline.md` and sent as a portrait chart.

**Cost of increment 0:**
- **AWS:** about $5–15. That's two or three gate batches at about $2–3 each on 4 spot servers, plus cents for the war suite.
- **Claude:** a modest share of this week's limit, an estimate; the weekly limit reset 10-27.
- **Codex:** none.
- **The designer's time:** none, beyond a quick look at the fixture list.

**Blocker:** AWS month-to-date hit the monthly cap on 10-27, and the budget policy blocks starting the box. Either the designer raises October's cap a little, or steps 4–5 run on Sunday 11-01 when the month rolls over. Steps 1–3 don't need AWS.

## Budget across the whole plan

- **AWS:** about $110–300 in total on spot, more of it in November. It stays under the monthly cap if the climb program's spot use pauses during the big batches.
- **Claude:**
  - the design, reviews and go/no-go calls;
  - increment 0 and most of increment 1, while Codex is out (until Mon 11-02 06:08 UTC).

  Check `get_usage` before every wave.
- **Codex:** most of the heavy implementation from 11-02. Check its quota before every Sol task.
- **The designer:** about 6–8 Hard playtests in total (two per increment from 1 on), plus 1–2 hours judging tricky decisions for the strategy bundles.

## Open decisions for the designer

1. **Order:** wars first, with the expansion gap after the checkpoint? Or expansion in parallel?
2. **Turn speed:** is p95 under 1 s for the whole lobby OK? Or should the smarter play become a separate "Hard+"? Measured 10-27 (`docs/sim-results/ai-war/baseline.md`): plain Hard uses 31 ms p95 for the whole 8-AI lobby in WASM at T250, which leaves about 950 ms. Today's Hard+Search takes 2.3–5.0 s p95 late in the game, so the new searches need a hard per-turn time cap. Because turns are simultaneous, the AI could also plan in a Web Worker while the human thinks, which would give it seconds rather than milliseconds.
3. ~~**Rule freeze** on combat, air, siege and VP until the checkpoint?~~ Answered 2026-11-11: lifted.
4. **His time:** about 6–8 playtests plus 1–2 hours of judging?
5. **Spend:** OK with the range above, and with the climb program pausing during big batches?
6. **Checkpoint:** if the sims pass but he still wins comfortably, stop and diagnose, or go on to increment 4?
7. **Difficulty labels:** keep today's Hard as Normal once Hard really fights?
8. **Agent limit:** run 6–8 Claude agents at once (instead of 4) while the weekly limit is fresh?

## Short-horizon objective (2026-11-10, `claude/ai-objective`, not merged)

On three Lab cards the designer finds easy, a sim-only arm that tells Hard the card's goal (`legatum-sim scenario --tell-objective`) splits "doesn't know the goal" from "can't execute it": Hold Biel 34 to 49 of 50, Ride to Dyrrachium 2 to 50, Two Cities 0 to 0. So on two of the three the gap is choosing the objective; on Two Cities it is execution (Bomber strikes, upgrades mid-strike, no Xochimilco candidate). Gene `capture_horizon` (off) prices a capture by when it lands and takes Dyrrachium to 48/50 untold. The general step is built behind gene `short_objective` (off): after round 3 (priced in Production, Hold only over a plan that leaves home) untold 50 / 50 / 0 against the told 49 / 50 / 0, war suite +2.5 pp with no significant loss, designer seat flat (held out 58.8 to 59.7). Next: censuses and a rivals-side check before any default change (`docs/sim-results/ai-quality/ai-objective-1110/README.md`).

## Survive-him program (2026-11-20, approved by the designer; in flight)

Goal: Hard survives him. In his last three wins (Athens 8.9.47, Sparta 8.9.57, Rome 8.9.67 to 8.9.71) he took a neighbour early, nobody stopped him, and the game was decided by T100. The measure, on his own recorded games with fixed Hard in the AI seats: the turn his first conquest lands and his city count at T100. No balance sims.

Increments, each ending in a build he can play against:
1. **Victim seat (Fable).** Positions cut about 10 turns before his first attack; Fable plays the victim with that player's view against his taped orders; every difference from Hard is confirmed by replay before it becomes a valuation change. Branch `claude/sh-victim-seat`.
2. **One agent per known failure (Opus), in parallel:** neighbours do not arm when the empire next door falls (`claude/sh-neighbour-alarm`); coalitions arrive one at a time (`claude/sh-coalition`); Hard stops expanding at war and does not secure strategic resources (`claude/sh-expand-at-war`); Hard buys units that lose the fight in front of it (`claude/sh-matchup-buys`); Hard cannot play Rome (`claude/sh-hard-rome`).
3. **Hill-climb round 2 on Codex** with conditional directives, scored on the same measure (`astra/climb-loop-2`).

**A seventh line, from his Swiss 8.9.72 win (coach review 2026-11-21, `docs/playtests/reviews/2026-11-20-8.9.72-switzerland-win.md`): a serial conqueror never turns on the leader.** A war on him was the Mongols' top-valued plan on 23 turns and gene `reachable_wars` removed it every turn (one-front clauses, reach measured as units within 8 tiles today, and his Reserve Gold counted as defenders he could buy). With the gene off for the Mongols alone from T135, his recorded orders win by T150 in 25 of 48 runs instead of 48. This is the late-game half of the coalition line (the army existed and was busy elsewhere). Candidate fix: idea #58 in `docs/sim-results/ai-quality/ideas.md`, behind a gene. Gates: it fires on the Mongols at T134 to T140 of that tape; scenario `mongols-turn-on-the-banker` (Hard 0/50, branch `coach/swiss-8972-scenario`); the dead-war count from the Germany 8.9.43 review does not rise. Not started; proposed to the designer as its own stream.

Conditions he set: a supervisor checks that the work yields results and adjusts course (scheduled task `legatum-survive-him-supervisor`, every 2 h, verdicts in `.progress/survive-him/supervisor.md`); Claude usage never exceeds 80% of the weekly budget. Shared brief: `.progress/survive-him/BRIEF.md`. Ledgers: `docs/sim-results/ai-quality/survive-him/<stream>/README.md` on each branch. Known risk: Fable's play may not reduce to rules Hard can follow without a model; then increment 1 yields observations only.

## Status

Kept in `plans/ai-war.prd.json` (one story per step, `passes` set only after its verification ran) and `plans/ai-war-progress.txt`.

# Training mode as a learning loop: extracting the most knowledge per game

Decision memo, Sat 2026-11-07, written against master `8ad0b9983` (RULES_VERSION 20). Companion to `fable.md` and its Addendum; this one covers the loop around the search, not the search. Every number names its file.

## 1. The verdict in one paragraph

The two specs in flight (`siege-oracle`, `training-mode`) build the right things in the right order, but as written they produce a **log that answers "did search beat Hard here" and nothing else**: the 10-06 log again. The knowledge is in what they don't log: how much each unit's choice mattered, which defender punished which move, the position in Hard's own features, and what then happened in the real game. Twelve changes to the briefs (section 10) cost about a day and turn the deviation log into a training set. Second correction: a full-search self-play batch is $25 to $40, not the $3 in `fable.md`, because the search costs 100x the game it runs inside. Third: his games are more useful than "held-out test" allows; a time split keeps the test honest while the older games teach.

## 2. The loop as a system

Each round is the same six steps. A round is strictly better than the last only if every step has a number and the numbers are compared on frozen sets.

| Step | What | Frozen control that makes the round comparable |
|---|---|---|
| Data | Positions: fixed lab cells, the 35 war scenarios and Scenario Lab cuts (the hardest first), then self-play on real maps, then his Training games | A **fixed engagement set** (`crates/sim/data/training-fixed.json`): 21 lab cells × 12 seeds plus 64 self-play engagements, versioned by `RULES_VERSION` |
| Labels | The omniscient teacher's best joint order set, its score components, the per-unit profile, regret of Hard's set | Same dice seeds across candidates and across rounds (common random numbers) |
| Distil | Fit the student (reads `PlayerView` only) behind a gene; fit on lab, self-play and his games, never on the fixed set or the held-back test games | Held-out lab cells by stride, held-out census seed 8241001, a third of the war scenarios by save |
| Gates | Lab capture, war suite, human-seat gate seat-only and rivals-only, census best/worst ratio, transfer gap | The frozen-control gate in `plans/ai-war.md`; the previous round's Hard as a league opponent (section 7) |
| Deploy | Gene default flips to 1; one ledger row (`tools/ai-quality/ledger.py`) with the round's numbers | The ledger is the only record; a round without a row didn't happen |
| New data | The next round's self-play runs the **new** Hard (expert iteration), so the teacher searches the positions the better student creates | Deviation rate on the fixed set must fall round over round; if it rises, the student regressed somewhere the gates don't see |

Three things stop it regressing. The **teacher is versioned too**: when its continuation policy improves (section 5), relabel the fixed set and report teacher-vs-teacher, so a student gain isn't a teacher change. **Every label carries `rules_version` and `ai_defaults_sha256`** (the nightly already stamps both), so a bump invalidates labels automatically; six bumps landed this week, and the combat, siege and Walls freeze must hold through a round or the round is lost. And the **deviation rate is the convergence metric**: the loop works when the teacher finds less to correct each round. Lab capture rising while the deviation rate on the fixed set doesn't fall means the student learned something other than what the teacher knows.

## 3. What to log

One JSON line per searched engagement, schema versioned, written identically by the app and `legatum-sim --training`.

| Field group | Contents | Why |
|---|---|---|
| Identity | `schema`, `rules_version`, `ai_defaults_sha256`, `engine_commit`, game id, turn, seat, faction, engagement id, `source` (lab, scenario, selfplay, designer-game) | Invalidation and splits |
| Position | The siege-oracle feature vector (units by class, era and HP each side, city HP, Walls tier, terrain, distance, relief within 6, Gold each side), computed by **one Rust feature function shared with increment 1** | The student can only learn from features it has; one function means the lab fit, the log and the live AI agree |
| Hard's set | Orders per unit, score components, rank among candidates | The regret baseline |
| Best set | Orders per unit, score components, the top 5 joint sets with components | Not one scalar: the components (captured × campaign value, own city kept, units lost and killed at cost, city HP delta, turns) let the weights be refit without rerunning the search |
| Per-unit profile | For each unit, the score of every legal order at the final pass, with the others fixed | **How much each choice matters.** A unit whose 20 orders score within 2% is noise; one whose best and second-best differ by 30 pp is the decision to distil. Also the sensitivity map a human or Codex reads |
| Defender split | Score of Hard's and best set under each defender policy separately (Hard, sortie, designer-defender) | Which defender punished which move. A move that wins against Hard and loses to the sortie is the overextension the memo warned about |
| Search trace | Restarts tried, passes to convergence, rollouts spent, wall time, `hidden_dependent` (best set references a unit or garrison outside the seat's `PlayerView`) | The transfer gap at the label level: a deviation only an omniscient searcher could find is not a lesson for Hard |
| Realised outcome | Closed N turns later: city captured or held and on which turn, units lost each side, Gold spent on defence. For his games, who the defender was | The delayed label. Calibrates the teacher against reality: its rollouts use scripted defenders; the real one is Hard or him |
| Agreement sample | One in twenty: the 20-turn full-state rollout's verdict | Tests the premise that the local verdict is the game's verdict |

**Volume and cost.** About 5 KB a line raw, 1 KB gzipped; a 4,096-game self-play batch has roughly 100k engagements, 100 MB gzipped, S3 pennies. Compute is the real number: one engagement at full width (8 units × 20 orders × 64 rollouts × 4 ms × 2.5 passes) is about 100 core-seconds, about 50 with successive halving (16 rollouts first, 64 only for the top quarter). So 100k engagements is about 1,400 core-hours, 22 hours of one c8g.16xlarge, **$25 to $40 a full batch** on spot. Increment 2 needs a tenth of that: every fixed cell and scenario, plus self-play engagements where the cheap first pass shows candidates more than 5 pp apart. Raw lines go to `s3://…/training/<rules_version>/<batch>/`; `docs/sim-results/training/` holds round summaries only.

## 4. Distillation, ranked by decision type

| Form | Explainable (pillar 5) | Speed in WASM | Capped by | Use for |
|---|---|---|---|---|
| **Fitted valuation table or logistic in fixed-point** (increment 1's form) | Yes: a breakdown per term | microseconds | Its features | Scalar gates: commit, continue or abort an assault; stance (storm, focus, wait for shooters, hold); target choice among cities; buy Walls or a unit under siege |
| **Small per-unit order-value model** (20 to 40 features of unit, target and position; linear or shallow trees; weights in one JSON file) | Yes if linear or small trees; a tree ensemble can print its path | microseconds per order | Feature design | Per-unit orders inside the battle: which target, charge or shoot, step or hold. Lives inside Hard's value scoring as a priced term, so the design-skill rule holds ("behaviour emerges from value scoring") |
| **Pattern mining into readable lessons** (cluster deviations by features, name the cluster, a human or Codex reviews) | Fully | n/a | Humans | The review layer, not the shipped AI: names the feature the models lack, catches label bugs, and writes the coach's tactical paragraph. The 10-11 lesson stands: don't write rules from clusters |
| **Per-faction specialisation** | As above | as above | Data per faction | A faction one-hot or offset term in the shared model, only where the shared fit's residual is faction-shaped (Mongol Keshig kiting, Spartan capture-heal). Gated on the best/worst ratio, since a tactical gain only Sparta can use widens it |
| Deep net | No | too slow single-threaded; the box can't run PyTorch | | Not now |

Ranking: table first (increment 1, in flight), per-unit order value second (increment 3), mining alongside both as the report, faction terms last and only on evidence. Later: a **1-ply search with the fitted value as the leaf** costs microseconds and does not repeat the 10-06 problem, because the leaf, not a candidate list, carries the knowledge.

## 5. Push back on the search spec itself

Four things in `training-mode-claude.md` will cap the teacher at "Hard plus one ply", which is the result we already have.

1. **Coordinate descent from Hard's orders finds the optimum nearest Hard.** Restart from each of the four lab policies and from Hard, keep the best, log which restart won. If Hard's restart wins 95% of the time, the generator has not widened.
2. **"Hard as the continuation policy" means the rollouts stage and don't shoot the walls**, so "shoot from range 2 for two turns, then ride in" never scores well because turn 2 is played by the AI that doesn't do it. Make the continuation a stance policy chosen per rollout (Hard, storm, focus, wait), and let the searcher pick the stance for the following turns as one more searched dimension. Next round, the continuation is the fitted student. That is the expert-iteration step the Addendum names; without it there is no iteration.
3. **No common random numbers in the spec.** An argmax over 160 candidates at 64 dice draws each picks noise unless every candidate sees the same draws. Pair the deltas; report Hard's regret with its interval.
4. **One integer score hides the objective.** Keep the integer for the argmax, but log the components and price them in Hard's currency (`plan.rs` campaign value, unit cost). The lab's storm wins cells and bleeds 2 to 3x the Production; a score that can't see that teaches it.

Two things the Addendum settled need one field each: omniscient labels are fine **if `hidden_dependent` is logged**, and the 5–20 turn horizon is right **if the horizon reached is logged** (a siege finished in 4 turns and one finished in 18 are different lessons for the commit gate).

## 6. Beyond tactics: the same method, in order

Order by label cost (horizon × decisions per game) against the measured gap in his games (`docs/playtests/diffs/patterns-2026-11.md`).

| Decision | His measured edge | Horizon | Labeller | When |
|---|---|---|---|---|
| Purchases under siege (Walls, a shooter), both sides | part of garrisons and escorts +4.7 VP | inside the battle | The battle searcher, if city orders are in the searched set | **Increment 2, free** |
| Settler escort, route and sniping | +35 pp on the India T13–T17 window; escorts +4.7 | 5 to 10 turns, local | Same arena around a Settler: escort, wait, route, snipe; true-state rollouts | Increment 6, week of 11-23 |
| Army target: which city, when to march | +6.8 VP, his biggest | 10 to 40 turns | The siege oracle as the leaf: capture chance and turns per reachable city, priced by campaign value; forced-branch rollouts (`value_regret` pattern, 24 reseeded games per option) | Increment 7, week of 11-30 |
| War declaration and Peace timing | +1.2 declare, +1.0 refuse Peace (wide intervals); Swiss T147 refusal +25 pp | 20 to 60 turns, ~5 decisions a game | Forced branches for 20 turns, play out; $5 to $50 per 2,000 decisions by horizon | Increment 7 |
| Founding sooner, site choice | +5.8 VP | 30 turns | Forced branches: found now against route or wait, scored on cities and VP at +30 and +60 | Increment 8 |
| Upgrades, research order | research 0.00 measured | long | Not worth labelling until the rows above move | Later |

The army-target row is where increment 1 pays twice: the fitted siege estimate is both the student for the commit gate and the leaf for the campaign search.

## 7. Learning from his games

**Score his orders with the search: yes, offline.** His tape replays hash-exact on its own tag, so after each Training game the box runs the searcher on every engagement of his seat and logs his set against the teacher's best, same schema (`source: designer-game`). Three outcomes per engagement:

- **His set ≈ best.** The teacher is validated on a human-made position; the count is its held-out score.
- **His realised outcome beats the teacher's expectation.** The teacher's defender or candidate model is missing something: the richest signal in the program, straight to pattern mining.
- **His set < best.** A coach line: "at Lugano T58 the search gives storm 71%, you waited (44%)". Not every human move is right.

For the coach, the window diffs get a tactical layer per engagement and the takeover curve a per-engagement regret series, so "where did the game turn" and "which order turned it" are one report. **Don't score his seat in the app:** log its positions, score after the game. That removes a thread pool's work from the app and keeps his turns fast.

**His games train the AI (designer, 2026-11-07: "yes my games should always be used to train the AI. it just shouldn't overfit or assume that the way that I played is the only way that the AI should play for that faction").** There never was a "never train on his games" rule; that was a mistake in the first memo. His engagements are labelled like any others and enter the student's training data. The guard is against overfitting: they are weighted alongside self-play, not above it; no faction's policy is fitted to his play alone (check that the per-faction strategic diversity in the census doesn't collapse); and his newest one or two games each round are held back as the test. What this unlocks now: a **scripted designer-defender** as the third rollout policy (Walls first everywhere, one or two units, buy a shooter the turn an army arrives, sortie on the wounded: the coach's pattern table). The teacher's biggest blind spot is that nobody in its rollouts defends the way he does.

**Training games are not Hard games.** The opponent plays omniscient-search moves, so a result against it says nothing about Hard. Mark them `mode: training` in the archive index and keep them out of the verdict table, "Hard in his seat" and the 50% target.

## 8. Self-play curriculum on spot (within the monthly cap)

1. **Hardest positions first.** The fixed lab cells, 35 war scenarios and every Scenario Lab cut where Hard scores 0/50 (`history.jsonl`: scenario mean 18%). A few hundred positions, under $1, and the ones that matter.
2. **Exploiter seats in self-play.** Hard-vs-Hard never attacks like a human; Proxy v6 beats Hard 75% (learnings 10-06), the `storm` seat and the `peace_cycler` seat exist. Put one exploiter in a quarter of self-play games so the teacher labels defences against human-style attacks.
3. **A league in the gate, against overfitting to Hard.** Keep the two previous Hard versions and the exploiters as frozen opponents; a round passes only if the new Hard is non-inferior against the whole population, not just the current Hard. This is what the 10-27 plan's increment 4 meant, started early and cheap.
4. **Budget line per round in the ledger**: labels $25 to $40, gates $5 to $15, league $5. Two rounds a week in a heavy week is about $120; the nightly is $0.29 a day. Spot preemption: slice batches short and rerun lost seed ranges (`tools/sim/value-generate.sh` already does).

## 9. How we know it works

Per round, one ledger row and a `training` block in `docs/sim-results/nightly/history.jsonl`, both on the fixed engagement set:

| Metric | Direction | Where |
|---|---|---|
| Teacher capture on fixed cells (omniscient) | up when the search improves, flat otherwise | nightly `training.teacher_capture` |
| Student capture, fogged, with the gene | up | nightly `training.student_capture`; state page section 1 |
| Transfer gap (teacher − student) | down | nightly |
| Deviation rate and mean regret on the fixed set | **down round over round**: the convergence metric | nightly; state page section 1 |
| `hidden_dependent` share of deviations | reported; if it rises, add determinized labels for those positions | round report |
| Full-vs-local agreement (the 1-in-20 sample) | ≥ 80% | round report |
| War suite, human-seat gate, best/worst ratio | the existing guards | existing rows |
| Teacher calibration on realised outcomes (Brier) | down | round report; his games separately |

The nightly already stamps `rules_version`; add the fixed set to what it refuses to run stale, so a bump turns the training row red the same morning. The state page gets two section-1 rows (deviation rate, transfer gap) and a section-5 freshness row for the fixed set. His drift concern is answered by the fixed set, the ledger row per round, and the rule that a round without a row didn't happen.

## 10. Changes to the two specs, for the builders now

`training-mode-claude.md`:

1. Log score **components** and the per-unit order profile, not one integer.
2. Common random numbers across candidates; paired regret with an interval.
3. Coordinate descent with **five restarts** (Hard, the four lab policies); log the winning restart and passes.
4. Continuation policy is a stance mixture with the stance searched, not plain Hard.
5. Log `hidden_dependent`, the horizon used, and the defender-split scores.
6. Engagement id plus a **realised outcome** record closed N turns later.
7. Include the defender's city orders (buy Walls, buy a unit) in the searched set.
8. Don't search his seat in the app; log its positions, score offline from the tape.
9. Mark Training games `mode: training` in the archive index.
10. Every line carries `schema`, `rules_version`, `ai_defaults_sha256`, `engine_commit`.

`siege-oracle.md`:

11. The feature vector is **one exported Rust function** that the log, the lab fit and the live gate all call; the fitted weights live in one JSON data file.
12. Report calibration on realised self-play outcomes as well as lab cells.

## 11. Risks and things not to do

- `fable.md` §8 stands: no search as a shipped level, no rules from clusters, no second simulator, no fitting on his games.
- Don't argmax without common random numbers; the deviation rate will read 36% and mean nothing, as in October.
- Don't let the rollouts' continuation be the policy being improved; the teacher is then Hard plus one ply.
- Don't fit on fields the live AI can't compute from `PlayerView`; `hidden_dependent` is the line.
- Don't read a flat human-seat gate as failure: expansion and war timing dominate it (focus fire moved it +0.20 pp). The lab, the suite and the deviation rate are the instruments; the gate is the guard.
- Don't let a rules bump through mid-round; if one must land, the round restarts and the ledger says so.
- Stop rule: if the deviation rate on the fixed set hasn't fallen after two rounds with the student on, the features are wrong; write it up before a third.

## 12. Roadmap (wall clock with agents, from Sat 11-07)

| # | Increment | Usable result | Measured by | Lands | Bottleneck |
|---|---|---|---|---|---|
| 1 | Siege oracle (running) | Hard commits to winnable sieges | lab 29% → 38%, suite +5 pp, calibration | Tue 11-10 | review rounds, ~half a day each |
| 2a | Searcher with the ten spec changes; schema v1 | Deviation log that a fitter can read | fixed set labelled; CRN regret with intervals | Thu 11-12 | one builder; the restart and stance work is the Opus part |
| 2b | Fixed engagement set, nightly `training` block, state-page rows, first round's ledger row and cluster report | Nothing drifts from here on | teacher capture, deviation rate, `hidden_dependent` share, agreement ≥ 80% | Fri 11-13 | the 1-in-20 full rollouts ($3) |
| 3 | Student v1: stance head and per-unit order value, fogged; designer-defender policy; one expert-iteration round (relabel with the student as continuation) | Hard storms, waits or pulls back on its own valuation | lab 38% → 42%, transfer gap, deviation rate down, league non-inferior | Wed 11-18 | fit-play-relabel, two self-play batches (~$60) |
| 4 | Training mode in the app, with offline scoring of his seat and `mode: training` | He plays it; his engagements enter the training data | two games logged; per-engagement regret in the coach's window diff | Fri 11-20, games the weekend 11-21/22 | his playtests |
| 5 | Field battles, retreat and heal; second iteration round | Wider trigger | same gates | Wed 11-25 | rounds |
| 6 | Settler escort, route and snipe oracle | The +35 pp window closed from the AI side | India T13–T17 window; `settler_escort` replaced by a priced term | Fri 11-27 | arena generalisation |
| 7 | Army target and war timing via forced branches with the siege oracle as leaf | Hard marches on the right city at the right time | war suite goals 14 → 17 of 34; "army routes" row shrinks | Fri 12-04 | full-game rollout cost; the frozen-control gate |
| 8 | Founding timing; exploiter league in every gate | | patterns table rows 2 and 6 | week of 12-07 | his reviews |

Total compute through increment 5 under $150, well inside the monthly cap. The real bottlenecks, in order: the rules freeze holding (six bumps this week), review rounds, his playtests (a few a week), and spot preemption on the long label batches. Reviews and gates fire off the ledger; if a round's row is missing on the state page, the round is not done.

## Addendum: data and cost (designer, 2026-11-07)
The designer rejected $25–40 cloud label batches ("too much"). The data sources become:
1. his local Training-mode playtests, where the AI seats search during his turn on his Mac;
2. overnight re-search of each of his battles, plus perturbed variants (move a unit, change HP, add a reinforcement), at low priority on his idle Mac;
3. combat-lab sweeps (about $0.10).

Cloud runs keep only Hard-vs-Hard balance and the gates. Any cloud training run over about $5 needs his OK. Expect fewer labels per round, so favour the small explainable fits (siege estimate, stance tables) over larger learned models until the data supports them.

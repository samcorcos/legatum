# Review 2: how the AI is improved and measured

Reviewed at master `a9eb46864`, 2026-11-09. Read-only. "Inferred" marks anything I did not verify in a file or by running it.

## Short answer

He is right. There is no training loop. There is a measurement system (good, mostly automated) wrapped around a manual process in which agents read his games, hand-write a new branch behind a gene, and send it to a gate that cannot resolve the size of change they make. Nothing in the repo takes "Hard wins this scenario 0 of 50" and automatically works on it until that number moves. The one automatic optimiser (CMA-ES over genes) has been stalled since 10-23 and the nightly Lab score that could drive one is computed and then read by nobody.

## 1. The loop as it runs today

**What actually happens for one improvement** (from `plans/ai-war.md`, `plans/ai-war-progress.txt`, `.claude/skills/legatum-workflow/SKILL.md`, the ledger):

1. He plays a game (person). 23–24 reviewed human games exist in total.
2. The coach agent reviews the tape and names mistakes (automated once triggered).
3. The lead or an agent picks one mistake and writes Rust: a new code path in `crates/ai/src` behind a default-off gene in `params.rs` (agent, hand-authored logic).
4. Cloud gate: human-seat benchmark (8 maps x 256 salts = 2,048 games per arm, seat-only and rivals-only), then designer-seat score, war suite, sometimes census (automated, about $2–3 and 15–30 box-minutes per batch).
5. An agent reads the result and decides: land, hold, or "land by judgment" (agent; e.g. `runaway_coalition` was turned on "by the lead's judgment rather than the gate", `docs/status/state-of-the-ai.md`).
6. Docs reviewer, static merge gate, landing queue (automated, serial).

**Automated:** measurement, gates, reviews, landing. **Not automated:** generating the candidate change. Every candidate is hand-written code. **Waits on a person:** new tapes, cloud spend over ~$5 for Training, rule changes, judgment landings.

**Cycle time:** about half a day to a day per change, wall clock (inferred from ledger timestamps and progress log; the gate itself is under an hour). The real delays are the serial landing queue (stalled 6 h overnight on a hung box job, `plans/ai-war-progress.txt`) and re-pinning baselines after rules bumps: `RULES_VERSION` went 7 to 25 between 10-31 and 11-08, and the bump to 25 silently invalidated 13 of 14 Lab baselines (`docs/learnings.md`).

**What landed in two weeks (10-26 to 11-09):** 2,340 commits, 477 touching `crates/ai`, about 220 touching `params.rs`. The ledger (`docs/sim-results/ai-quality/ledger.jsonl`) has 122 rows in that window, about 60 gated arms. Statistically confirmed strength gains: roughly 6 to 8.
- Seat-only: `border_explained+fallback_risk` +4.35 pp [2.5, 6.2] (10-31); Swiss playbook +1.07 and +0.73 (11-05).
- Rivals harder: settler bundle -3.37, `walls_garrison` -4.15, `march_past_reach` -2.54, siege stance -1.86 [-3.61, -0.10].
- AI-only faction fixes (Mongol horde, India, Germany 6.2 to 15.9%), which are balance, not strength against him.

So about one confirmed gain per 60 AI commits, and none of them large. Every item in the plan's tracker (W1, W2, W3, CHECK, W4, R1) is still `passes: false`.

## 2. Is the measurement sound?

**The headline has not moved.** Human-seat control: 16.2% (10-28), 17.8% (10-31), then 487, 468, 470, 472 of 2,048 on 11-01 to 11-06: flat at 23% for eight days against a 50% target. The runs on 11-07 (505, 460, 439, 406) are on regenerated maps and are not comparable. Designer-seat held out: 57.4, 56.0, 56.0 (`docs/sim-results/designer-seat/history.jsonl`, three points). His real record against Hard is 8–0 since 10-27. Rate toward 50%: about +7 pp in the first four days, zero since. At the current rate there is no date.

**The gate cannot see the changes being made.** Across 72 seat-only delta rows, the median absolute effect is 0.65 pp and the median CI half-width is 1.9 pp. 10 rows are significantly positive and 6 significantly negative; at 72 tests about 4 would be significant by chance alone. The workflow skill already says it: "The 2,048-game gate rates nearly every change neutral." To resolve 0.65 pp on this benchmark needs roughly 9 times the games, and that still would not help because:

**The games are not independent.** `docs/learnings.md` (11-08, 10-18): on fixed dice, 37 of 64 salts of a map were identical games; a map is "close to one coin flip". The benchmark is 8 maps. The honest sample size is far below 2,048 (inferred: tens, not thousands), so the printed CIs are too narrow and the significant rows above deserve less trust than they look.

**Every instrument is Hard against Hard.** This is the main soundness problem, and the 11-07 strategy memo names it (`docs/design/ai-strategy/fable-2026-11-07.md`). The "human seat" is a seat-swapped Hard, not him. A gene that helps against opponents who make Hard's mistakes scores well; a gene that would beat him but loses to Hard's mirror scores badly. `siege_oracle` is the example on record: better estimate, braver, -7.3 pp.

**The designer-seat score measures the wrong direction.** Hard takes over *his* seat on *his* tapes and plays against Hard rivals. It answers "can Hard finish his position", not "can Hard beat him". The T60 and T80 cells are near 48 of 48 (finishing a won game), Celts 8.9.26 sits at 88%, and the page itself says it "does not include rivals getting harder against him" (`docs/status/designer-seat.md`). The held-out set was redefined on the day it was created. It can rise while he keeps winning 8–0, and it can stay flat while a real fix lands: walled-chain-push took Nicomedia from 0/50 to 20/50 and was not landed because designer seat was flat.

**Metrics that can rise without the AI getting better against him:** seat-only human-seat (exploits Hard's mirror), designer-seat training tapes (48 runs on a fixed tape, the takeover replays his opening), census faction ratio (balance), and any Lab card score if cards are tuned on directly (25 cards, a few hundred genes).

**Selection bias is documented and uncorrected.** `docs/learnings.md`: a +10 pp best-of-window result appeared in 64% of windows and held on fresh samples 27–41% of the time. Determinized rollouts are optimistic (7/16 vs 20/48).

**An inconsistency to resolve:** `docs/sim-results/nightly/scenario-lab.json` reports a Lab mean win of 18.07%, but the baselines stored in `scenarios/lab/*.json` are 0% on 19 of 24 measured cards and average under 2%. One of these is stale or they cover different card sets. I did not resolve it.

**Gap between gate and goal.** No gate contains a player who plays like him, and no gate is his actual result. The only ground truth is 8–0 and it arrives once every day or two.

## 3. What is missing, against how strong game AIs are built

Throughput, measured: 16 eight-player Hard games in 4.04 s wall, 44.3 CPU-s on this Mac with other agents running, so 2.8 CPU-s per game, about 4 games/s (1.23 CPU-s per game on a quiet machine per `docs/learnings.md`, about 13 games/s). A 64-vCPU spot box at about $0.73/h does about 25–50 full games/s (inferred from the same per-game cost). The ES pilot ran 1,360 sixty-turn 1v1 games/s on a c8g.16xlarge. A Lab card is 5 to 20 turns from a saved state: about 0.1–0.3 CPU-s (inferred), so 50–150 card runs/s on the Mac, several hundred per second on a box. The monthly cloud budget buys several hundred box-hours: tens of millions of full games or about a billion card runs. Compute is not the constraint. It is the cheapest resource in the project and the least used: the Codex pending queue is empty (0 tasks, about 19 parked in `hold/`), CMA-ES last ran 10-23.

| Method | Fit | Why |
| --- | --- | --- |
| Search at decision time (MCTS or beam with rollouts) | Good for tactics, poor for whole turns | Simultaneous turns, 8 players, hundreds of orders per turn. But battles and sieges are small. Training round 2 already showed it: war suite +3.7, combat lab 29.8 to 42.9%. 36% of battles hit the 15 s cap, so it is offline teacher only for now |
| Offline search then distil into the live policy | Best fit | This is the missing engine. Fast deterministic sim, saved states, cheap rollouts |
| Imitation from his games | Good as an opponent, weak as a policy | 24 games is far too few to learn to play, enough to fit a "plays like him" opponent in specific situations and to seed search with his orders |
| Automated curriculum from failures | Best fit, mostly built | The Lab is this. It is not connected to an optimiser |
| Exploiter league | Later | W4 in the plan, not started. Needs a policy space richer than genes or it exploits nothing |
| PBT or CMA-ES over genes | Only after the above | Tried: 12 of 12 runs failed. Genes are scalars on fixed code paths; no gene value writes "garrison the bought unit" |
| Learned value net | Low return now | Regret study: a better estimate changed 17.8% of picks and saved 0.04 pp |

**The specific missing pieces:**

1. **A closed loop from a failing scenario to a behaviour change.** Hold Biel 0/50 was traced by hand to three concrete tactical errors (buy ranking, fire targeting, bought units leaving). That diagnosis is exactly what search can produce automatically: find an order sequence that wins the card, diff it against Hard's orders at the first divergence. Nobody runs that.
2. **A search that is allowed to find things Hard cannot propose.** Operation search failed its kill check four times (2/22, 5/22, 0/21, 3/38 windows vs a 30% mark) and the stated cause is the candidate set: it chooses among Hard's own plans. His recorded orders are an existence proof and the right seed. That has not been tried.
3. **An opponent that is him.** Without it "rivals harder" means harder for Hard.
4. **A dense score.** Win/lose on a 0/50 card has no gradient. The cards already record `median`, `p10`, `p90` objective scores. Those are climbable.
5. **A rules freeze for the AI.** 18 rules versions in 8 days means every baseline is rebuilt before it is used twice.
6. **Somewhere for found behaviour to go.** If the only output format is "a gene", most search findings cannot be expressed. The loop needs an agent step that turns a diff into code, and a test that the code generalises.

## 4. The lead's three-step proposal

**Step 1, the whole Lab becomes one score the tuner climbs, his games as guardrail.** Weakest step as written.
- The tuner has nothing to climb with. CMA-ES over genes failed 12 of 12 and ten-plus hand genes all landed within ±2 pp. The parameter space does not contain the winning behaviour.
- 19 of 24 cards are at 0%. A mean of zeros is flat. It must be the shaped objective score, normalised per card against his result.
- 25 cards and a few hundred genes will overfit in hours. Split the cards (train and held out) before the first run and never tune on the held-out half. `scenarios are disposable` in the workflow skill works against this: the held-out half must be stable.
- The proposed guardrail (designer-seat) is flat, half ceiling, and measures finishing his lead. Use it as a noninferiority check only, as the 11-08 rule already allows.
- Worth keeping: one number, published nightly. It exists; fix the 18.07% discrepancy and make it the headline.

**Step 2, offline search finds the winning line in each failing scenario.** Right core, and the thing he is asking for. Conditions, from the four failed kill checks: search at the order level, seeded from his recorded orders, not over Hard's plan candidates; score the card objective; confirm every found line on fresh dice (the selection-maximum and determinized-rollout traps are both on record). And it is half a step: a winning line on one card is a replay, not a skill. The proposal says nothing about how the line becomes general behaviour. That conversion is the hard part and needs its own gate: the fix must raise sibling cards it was not derived from.

**Step 3, Codex works the queue continuously.** Fine, but it speeds up the part that is not slow. Implementation is not the bottleneck; 477 AI commits in two weeks produced 6–8 confirmed gains. The queue is empty today because nobody generates tasks, not because nobody executes them. It becomes valuable once step 2 emits tasks automatically ("card X: Hard diverges at turn 3, does A, the winning line does B"). Then it is the right worker.

**Missing from the proposal:** the opponent model of him, the rules freeze, and a measurement fix. Without those, the loop will climb a score and he will still win.

## Recommended loop, in usable increments

Each increment is usable alone. Times are wall clock with agents.

**A. Skateboard: one card, end to end, by hand-cranked tooling. About 1 day (by 11-10).**
Take Hold Biel or Nicomedia. Order-level search from the card's saved state, seeded with his orders, objective = card score. Output: a line that wins on 50 fresh dice seeds, and the first-divergence diff against Hard. An agent turns the diff into a general code change. Accept if the card moves from 0/50 to at least 15/50, two sibling cards it was not built from do not drop, and designer-seat passes noninferiority. Runs on the Mac (card runs are sub-second). Bottleneck: writing the order-level candidate generator, and one agent fix round. Kill check: if search seeded with his own orders cannot win the card on fresh dice, the card depends on dice or on his opponent's errors, and that is worth knowing in a day.

**B. Scooter: the same thing as a nightly queue over every failing card. About 2 more days (by 11-12).**
Nightly: run all cards, search every card below his score, emit one Codex task per divergence into `.progress/astra-queue/pending`, re-score. Headline number: mean shaped score on held-out cards, plus count of cards at parity with him. Fix the card split now. Cost: under $5 a night. Bottleneck: the serial landing queue and rules bumps. Ask him for a rules freeze on AI-relevant mechanics, or batch rule changes weekly with an automatic re-baseline.

**C. Bicycle: cards mint themselves from every game he plays. About 2 days (by 11-14).**
After each of his games, automatically cut a card at every point where a takeover by Hard loses ground against his line (the takeover curve tooling exists). This is the curriculum, and it removes the person from scenario authoring. Bottleneck: his play rate, one game a day or two. Also mine the 624-game archive for AI games where Hard lost a lead.

**D. Motorcycle: an opponent that plays like him, and a gate built on it. About 3–4 days (by 11-18).**
Per-situation imitation from his 24 tapes plus scripted versions of his known openings and strikes, installed as exploiter seats in the rivals-only gate. Replace the 8-map full-game benchmark as primary instrument with many short scenarios on independent dice (thousands of independent samples instead of about eight); keep the full-game gate weekly as a guardrail. Bottleneck: tape volume; validate the model by predicting his orders on a held-out game.

**E. Car: tuning and distillation on top. Open-ended, start after 11-18.**
Only now run CMA-ES or PBT over the shaped held-out score, distil battle search into the live tactical valuation, and start the exploiter league (W4). These failed before because there was no dense score and no expressive space; A–D supply both.

**The test of the whole thing** is unchanged and should be on the status page next to the Lab score: his win rate over his last 10 games against Hard.

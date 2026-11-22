# Laya journal

A running record of Laya training runs and what each one taught us. Newest entries at the bottom. Engineering gotchas still go in `docs/learnings.md`; this file holds the Laya results and the lessons drawn from them.

**What Laya is here:** a small model (`convaiinnovations/laya`, typed-decisions head, v0.3.26) that we fine-tune to make the strategic decisions an AI seat faces every 5 turns. These are posture, production focus, research, threat response, war target, and accepting peace or alliances. The goal is an AI that decides better without cheating (pillar 6), at a fraction of an LLM's cost per turn.

**Pipeline:**
1. A teacher LLM plays one seat in a full game of `legatum-sim` (Quick speed, 8 empires, Hard scripted opponents), answering each decision (`tools/llm-seat/gen-data.sh`).
2. Hard decisions escalate to a stronger tier.
3. The teacher also files "findings": bugs, rule inconsistencies and design flags it noticed in the game state.
4. `tools/llm-seat/laya-train.py` fine-tunes Laya's decision head on the teacher's answers (frozen encoder, MPS GPU), holding out whole games for evaluation.

Data lives in `~/LegatumRuns/laya/` and checkpoints in `~/LegatumRuns/laya/checkpoints/<date>`.

---

## 2026-10-05: first Claude-teacher run (Sonnet 5.5, escalating to Opus 5.5)

**Setup.** 4 teacher games planned at Quick speed. The teacher is Claude Sonnet 5.5; hard decisions escalate to Claude Opus 5.5. Codex (the earlier teacher) was out of budget.

**Getting it to run took three fixes** (details in `docs/learnings.md`):
1. macOS bash 3.2 aborts on an empty array under `set -u`, so the overnight run died in its first second.
2. A leftover check accepted only Codex tier names (`luna`/`sol`) and rejected every Claude answer.
3. The skip-recorded-games check failed under `set -e` when no file matched.

The run also lost about 11 hours to two Mac session crashes caused by Playwright WebKit (unrelated to Laya).

**Teacher data so far:**

| Game | Faction | Result | Turns | Decisions | Escalated to Opus | Seconds deciding |
|---|---|---|---|---|---|---|
| 0 | Switzerland | lost, VP 2, rank 6 | 125 | 33 | 24 (73%) | 1,102 |
| 1 | Germany | lost, VP 1, rank 4 | 94 | 21 | 7 (33%) | 386 |
| 2 | Sparta | in progress | | 31+ | | |

**What we learned so far:**
- **Escalation rate varies a lot by game** (73% vs 33%). Game 0 was a messy war-and-bankruptcy game. Escalation drives cost and time, so before scaling up, measure whether Sonnet-only answers are good enough targets (the audits give a first hint below).
- **Audits:** in game 1, Opus re-answered 5 of Sonnet's decisions, one each of posture, production focus, research, threat and war target, and agreed with all 5. That's a small sample, but it suggests Sonnet alone may be a good enough teacher for many question types.
- **The teacher seat lost both games.** That's expected: the LLM only sets strategy every 5 turns, and the scripted AI executes it. It does mean the data teaches "what a strong player would choose here", not "what wins". The labels are only as good as the teacher's judgement.
- **The teacher is a very good bug finder.** Game 0 filed 64 findings and game 1 filed 29. These are the teacher's claims from the state it was shown, unverified; some may be gaps in the seat's view rather than game bugs. Recurring themes:
  - **Idle build queues.** Cities, including the capital, "build nothing" for 9 to 20 turns while sites are known and the empire is at war. Reported in both games. If real, this is a significant AI bug, possibly in how the LLM-seat directives hand back to the scripted AI.
  - **Units vanishing with no combat.** Military counts drop 5→3 or 3→2 at peace, with no event logged. Possibly upkeep disbands that the seat's event log doesn't show, which would break pillar 5 (no hidden math) for human players too.
  - **War declared twice on the same target, with no peace logged** between, in both games.
  - **Wars with no fighting.** A declared war with an undefended target 4 hexes away, and no army moves for 5 to 11 turns.
  - **Research left idle** while Science accrues.
  - **View or format issues in the seat's prompt:**
    - the turn-1 state shows no city and no Settler;
    - the yes/no question type is labelled `noul`;
    - alliance guidance reads both "refuses" and "accepts at 0+".
  - **Design flags:**
    - runaway leaders (one empire holding 4 of 5 titles, 9/10 VP);
    - faction goals of very uneven difficulty (Switzerland' 200 Gold against +1/t income);
    - the Six Technologies milestone separates no one;
    - Walls' buy cost is flat at 140 Gold however close they are to done.
- **Next:** verify the top bug claims (idle queues, vanishing units, double declarations) against the tapes before acting on them.

**Training:** pending until all 4 games finish. With 4 games, the by-game holdout is a single game, so the first evaluation will be very noisy. Read it as "does training run and move in the right direction", not as a quality measure.

### Results (run finished 20:12)

**Teacher data, all 4 games:**

| Game | Faction | Result | Turns | Decisions | Escalated to Opus | Seconds deciding | Findings |
|---|---|---|---|---|---|---|---|
| 0 | Switzerland | lost, VP 2, rank 6 | 125 | 33 | 24 (73%) | 1,102 | 64 |
| 1 | Germany | lost, VP 1, rank 4 | 94 | 21 | 7 (33%) | 386 | 29 |
| 2 | Sparta | lost, VP 5, rank 5 | 147 | 37 | 28 (76%) | 1,111 | 64 |
| 3 | Japan | lost, VP 1, rank 6 | 145 | 43 | 28 (65%) | 1,291 | 64 |

134 decisions in total. One decision asks several questions (posture, threat, research and so on), so they became 1,296 training questions: 856 train (games 0–2) and 440 held out (game 3). Audits: Opus re-answered 14 of Sonnet's decisions and agreed with 12 (5/5 and 7/9).

**Training:** 300 steps on MPS, about 70 seconds. The encoder is frozen; only the decision head trains. 300 steps is less than one pass over the 856 training questions. Mean training loss was 2.45. Laya warned that the base checkpoint ships invalid temperatures, so its confidences are uncalibrated.

**Evaluation (first 100 held-out questions):**

| | Agreement with the teacher |
|---|---|
| **Fine-tuned Laya** | **12%** |
| Random guess (average 1/options) | 28% |
| "Always give the most common training answer for this question type" | 65% |
| The scripted AI's own default | 67% (on the 69 questions that have one) |

Calibration error (ECE) was 0.22, which is poor.

**What we learned:**
- **The first fine-tune is worse than random.** Below chance usually means a bug, not just too little data: a label or index mismatch between training and evaluation (the "score" question type maps answers differently from "choice"), or the base model's broken temperatures. It needs debugging before more teacher data is worth buying. Being below chance at 12% isn't explained by "too few steps".
- **The teacher mostly agrees with the scripted AI.** Sonnet/Opus picked the scripted default 54–67% of the time. So a model that just learned "do what the scripted AI would" would score about 65%. Laya has to beat that baseline to be worth anything; beating chance isn't enough.
- **The interesting signal is the other third:** the cases where the teacher overrides the scripted AI. Future evaluation should report agreement separately on "teacher agreed with scripted" and "teacher overrode scripted". The overrides are what we want Laya to learn.
- **Cost:** about 65% of decisions escalated to Opus in the long games, and each Quick game took 6–21 minutes of LLM time. The audits (12 of 14 agreed) suggest Sonnet alone would usually do, which would roughly halve the cost.
- **The teacher keeps finding real-looking bugs** in every game: idle build queues, units vanishing at peace, wars declared twice. This may be the most valuable output of the run so far.

**Next:**
1. Debug the below-chance result:
   - evaluate the untrained base model;
   - check the label mapping per question type;
   - train for more steps;
   - report agreement on overrides versus non-overrides.
2. Verify the top teacher bug claims against the tapes.
3. Only then generate more teacher data, probably Sonnet-only.

**Decision (user, 2026-10-05):** future teacher data uses Sonnet alone, with no Opus escalation. `~/LegatumRuns/laya/overnight.sh` now points the escalation command at Sonnet too.

## 2026-10-05: verifying the teacher's bug claims

Verified against exact replays of games 1-3 (the runs saved no tape, so I replayed each game by feeding the logged answers back through a decider that makes no LLM call; the decision states match byte for byte) and against core events. Game 0 does not replay (it diverges at turn 7), so its claims rest on the logged states plus the same patterns in its replay. Full report: `.reviews/laya-findings-verify-2026-10-05.md`. The diagnostics are on branch `claude/laya-findings-verify`.

**Almost none of the "bugs" are game bugs. They are gaps in the seat's briefing.** Three claims rest on the briefing being a turn-start snapshot with no history, and a fourth on the teacher not being able to see fog:

| Claim | Verdict | What is really happening |
|---|---|---|
| Idle build queues for 9-20 turns | not a game bug | Core shows a city with nothing queued only on single turns, always right after an item completed or the city was founded (435 of 435 completions, apart from one non-seat city); the scripted AI queues the next item the same turn and nothing is lost. The briefing prints "builds nothing" and the teacher read non-consecutive snapshots as one long spell. |
| Research idle while Science accrues | not a game bug | Same artefact: every "Research: none" is the turn after a tech completed (70 of 70 over the four games). |
| Units vanish at peace | not a game bug | Each loss has a logged event: barbarian kills (games 1 and 3), Sparta desertion (game 2: 10 `UnitDeserted`, the faction drawback, which the seat text never mentions), Gold-runway disbands (game 0). The briefing never reports own losses, barbarians or the Sparta drawback. The real UI does tell the player. |
| War declared twice, no peace logged | not a game bug | Every re-declaration follows a logged `PeaceEstablished`; the seat's memory records declarations but never peace. |
| Declared wars where no army moves | real, but not a movement bug | All 11 wars the seat declared came from the `war_target` directive (scripted answer was always "none"). None ever started a campaign: under the Hard plan layer campaigns start only from the plan, and the forecast said the target could not be taken (`insufficient_army`). "Undefended" was fog, not fact: the target had a Soldier and 3 Rangers. The LLM can only start paper wars. |
| Turn 1 shows no Settler or city; "noul"; "refuses" with "accepts at 0+" | seat-view bugs | The Settler exists and founds the capital in turn 1's orders; "noul" is Laya's name for yes/no; "n/a (refuses)" means the scripted AI refused without scoring, so the "accepts at 0+" suffix contradicts it. |

**Consequences for Laya.** The teacher's training questions carry these mislabelled states: "builds nothing", "Research: none", wars with no way to prosecute them, and an Sparta that is not told it bleeds units at peace. Fix the briefing before generating more teacher data, or the student learns to react to artefacts. Top fixes, all in `crates/ai/src/llm/` and none touching game rules:
1. show "just finished X; the scripted AI queues Y" instead of "builds nothing" and "Research: none";
2. add a recent-losses line with causes, barbarians in "seen near us", the Sparta drawback, and "peace since tN" lines;
3. make `war_target` and `focus_target` report campaign feasibility and fog honestly, and gate the declaration on it;
4. small prompt fixes (Settlers and Scouts on turn 1, a "noul" legend, the "refuses" wording);
5. expose the plan and campaign events from `LlmSeatBrain` so run records can show them.

**Lesson.** A teacher that only sees text will report every gap in the text as a game bug. Diagnose a claim with a replay before filing it; the same replay tool also checks any directive change for faithfulness (its orders match the tape on every turn).

## 2026-10-05: LLM-seat briefing fixes (branch `claude/llm-seat-briefing`)

Fixed the briefing gaps found by the bug-claim verification, in `crates/ai/src/llm/` only (plus the plan layer's read-only forecast helpers). No game rules changed, and scripted seats are byte-identical (`tools/sim/repin.sh` shows no diff).

**Briefing.**
- A city or research slot with nothing queued now says why and what happens next: "idle at turn start (just finished Workshop; 12 Production banked; the scripted AI queues Settler this turn)", and "Research: none queued (just finished X; the scripted AI picks Y this turn)".
- Turn 1 shows "Settlers 1, Scouts 1" and that the Settler founds the capital.
- New lines: "Recent losses (own units)" with an inferred cause, "Barbarians" (units and camps near our cities), the Sparta Restless Regiments drawback and its desertion countdown, and "Peace since tN" / "peace with X (war since tN)".
- Fogged cities say "defenders unknown", not "0 defenders".
- The prompt explains `noul` (yes/no), and an unscored offer reads "Not scored: the scripted AI refuses it".
- The PlayerView carries no events, so `SeatMemory` infers losses, finished items and peace from the change between turns. Causes are heuristic: a loss near a hostile unit is combat or barbarians; otherwise Sparta desertion due that turn; otherwise a treasury that could not cover its net; our own `DisbandUnit`; else "cause unknown".

**War targets.**
- Under the plan layer, `war_target` offers only wars with a feasible campaign: the plan's forecast takes a known city, capture odds are at least 15%, the army is not holding a captured city, and a force already in reach also passes the campaign's own gate. Each option shows the forecast turns and odds; the Rivals lines show the reason when there is none.
- A feasible seat war becomes the plan on its decision turn (`PlanReason::Directive`), so the war is declared and fought. A war with no feasible campaign is not offered and, if named anyway, is not declared.
- The 15% floor comes from data. With a bare "forecast is Some" gate, 6 mock-seat wars (4 Hard Quick games, `mock:warlike`) gave 3 paper wars: those at 3% and 12% odds were dropped by the plan within 2 to 23 turns, and every war at 15% or more started a campaign. Re-run with the floor: 5 seat wars, 4 started a campaign (the fifth was declared at turn 143 of 148). `laya_plan_trace` on game 1 replays the tape with 0 order differences and shows the same plan and campaign events.
- Replaying teacher games 2 and 3 with the new briefing: none of the wars the teacher declared (t17, t46, t102, t107, ...) is offered any more, so none is declared.
- `LlmSeatBrain` now exposes `plan_events`, `campaign_events` and `blocked_expenses`, so the run JSON records them for the seat.
- New `mock:warlike` decider (scripted answers, but war on the first offered target) for end-to-end runs with no LLM.

**Consequences for Laya.** Teacher data recorded before this change carries the old briefing ("builds nothing", paper wars). Generate new teacher data after it.

### Why "Hard AI beats the LLM" isn't the conclusion yet (2026-10-05)
The user asked how the scripted Hard AI can beat a Laya + frontier-LLM combo. It hasn't been shown to:
- **Laya hasn't played a game.** The 12% is an imitation quiz, and being below chance points to a pipeline bug that is still being debugged.
- **The LLM seat played with handicaps.** It sets strategy only, every 5 turns, while the scripted AI executes everything. It read a misleading briefing. And its declared wars were dropped by the plan layer (paper wars).
- **The sample is tiny.** 4 games as 1 seat of 8, where the fair share of wins is about 12.5%. It placed 4th–6th.

**Plan, in order:**
1. Fix the briefing and the paper wars (`claude/llm-seat-briefing`).
2. Run a fair A/B: the same seeds and seat, a Sonnet seat against a plain Hard seat; compare win rate, VP and rank. Only if the LLM general beats Hard is imitating it with Laya worth more data.
3. Then more Sonnet-only teacher data, and Laya retraining against the majority and scripted baselines.

## 2026-10-06: fair A/B, a Sonnet LLM general against the plain Hard seat

**Setup.** Landed `claude/llm-seat-briefing` first (master `b55586e8`): fmt, clippy, `cargo test --workspace --release` and `repin.sh` (no diff) all pass, and the 50-game Standard smoke is clean (0 panics, 0 stalls, second-city-never 15 of 400 seats, 3.8%).

A/B: Hard, Quick, 8 empires, 8 games with seeds 3711001 + 1009 x i, seat i and the faction lineup rotated as `gen-data.sh` does (seat 0 Switzerland through seat 7 Mongols).
- **A:** that seat is an LLM seat, Sonnet only (`claude-sonnet-5-5` for both `--llm-cmd` and `--llm-escalate-cmd`), a decision every 5 turns, audits off.
- **B:** the same seeds, seat and factions with a plain Hard seat.
- The games share maps and factions but diverge in play once the seat's first directive lands, so this is not the same game twice. Game 4 is the exception: the seat's answers changed nothing and A and B are identical.

**Cost and reliability (A).** 283 decisions and 497 calls (low-confidence answers re-ask the same model). Fallbacks 1 of 283 (0.4%), failed calls 1 of 497 (0.2%), far under the 10% stop line.

| Seed | Seat faction | A rank / VP | B rank / VP | A cities at t50 / t100 / end | B cities at t50 / t100 / end | A captured / lost | B captured / lost | A wars declared / campaigns | B wars declared / campaigns |
|---|---|---|---|---|---|---|---|---|---|
| 3711001 | Switzerland | 7 / 1 | 6 / 1 | 2 / 2 / 2 | 1 / 3 / 3 | 0 / 1 | 0 / 0 | 0 / 0 | 0 / 0 |
| 3712010 | Germany | 5 / 3 | 5 / 5 | 2 / 3 / 3 | 2 / 2 / 2 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 3713019 | Sparta | 5 / 1 | 4 / 1 | 3 / 1 / 1 | 2 / 0 / 0 | 0 / 2 | 0 / 2 | 2 / 1 | 0 / 0 |
| 3714028 | Japan | 5 / 2 | 7 / 1 | 3 / 3 / 4 | 3 / 3 / 1 | 0 / 0 | 0 / 2 | 0 / 0 | 0 / 0 |
| 3715037 | Celts | 3 / 5 | 3 / 5 | 3 / 4 / 4 | 3 / 4 / 4 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| 3716046 | Athens | 2 / 6 | 2 / 8 | 3 / 5 / 5 | 2 / 5 / 5 | 0 / 0 | 0 / 0 | 1 / 1 | 2 / 0 |
| 3717055 | Byzantium | **1 / 11 (won)** | **1 / 10 (won)** | 3 / 4 / 4 | 3 / 6 / 7 | 1 / 0 | 3 / 1 | 1 / 1 | 2 / 3 |
| 3718064 | Mongols | 2 / 5 | 4 / 1 | 3 / 6 / 8 | 2 / none / 2 | 3 / 0 | 0 / 0 | 3 / 2 | 0 / 0 |
| **Total / mean** | | wins 1, rank 3.75, VP 4.25 | wins 1, rank 4.00, VP 4.00 | 22 / 28 / 31 | 18 / 23 / 24 | 4 / 3 | 3 / 5 | 7 / 5 | 4 / 3 |

("none" is B's seed 3718064 game, which ended at turn 88 before turn 100.)

**Read.**
- **Win rate:** 1 of 8 each (the fair share is 12.5%). The 95% Wilson interval for 1 of 8 is about 2% to 47%, so the two arms are statistically indistinguishable, and so would be arms differing by 30 points.
- **VP and rank:** paired differences A minus B are VP 0, -2, 0, +1, 0, -2, +1, +4 (mean +0.25, standard error about 0.6, t about 0.4) and rank better by 0.25 places on average. Nothing is significant.
- **Expansion:** the one direction that shows across games is more cities for the LLM seat (22, 28 and 31 summed over the 8 games at turns 50, 100 and the end, against 18, 23 and 24). That is +4 cities by turn 50 on 8 games; it is suggestive, not established. Part of it is that B's game 7 ended early.
- **War:** the LLM seat declared 7 wars and started 5 campaigns (4 captures); the plain Hard seat declared 4 wars and started 3 campaigns (3 captures). Of the 7 seat wars, 5 started a campaign and 2 did not (seed 3713019 declared 2 wars and started 1 campaign; seed 3718064 declared 3 and started 2), so the gate removes most paper wars but not all; the 2 were not examined (wars declared counts every declaration the seat made, directive or scripted).

**Honest conclusion.** With 8 games, no. A Sonnet general setting strategy every 5 turns is **not shown to be better than the Hard plan layer, and not shown to be worse**: it is at about parity on wins, VP and rank, with a mild lean towards more cities and more (feasible) wars. Both arms lost 7 of 8 games as a seat of eight, in line with the 12.5% fair share, so neither is strong. A difference small enough to matter for the next step (a few points of win rate) needs on the order of 100+ games per arm; 8 per arm only rules out a large effect.

**What this means for Laya.** The experiment does not justify imitating the LLM general yet, because the teacher is not demonstrably better than the scripted default. What it does show is that the teacher plays no worse and costs about 62 Sonnet calls per game. The cheaper question to answer before buying more teacher data is whether a larger A/B (say 32 games per arm, about 2,000 calls) shows the city and war lean turning into wins.

## 2026-10-05: Debugging the below-chance fine-tune (it was not a label bug)

**Short version:** the 12% was the *untrained* head's behaviour, and the first run (300 steps, batch 1, lr 2e-5) hardly moved it. The label mapping is correct for all three question types. With a proper run (several epochs, batch 8, lr 1e-4) Laya reaches about 50% agreement on all 440 held-out questions. That equals the "most common answer per question family" baseline (50.5%) and is below the scripted AI's default (54.4%). So the fine-tune is not yet useful, but the pipeline is sound.

**What was wrong:**
1. **No bug in the label mapping.** Checked end to end for `choice`, `score` and `noul` (`tools/llm-seat/test_laya_train.py`):
   - The training path's logits row equals the inference path's raw option logits, so the loss and `system_one` use the same option order (criteria order; for `noul`, index 0 = false, 1 = true).
   - After overfitting a few examples whose targets are never option 0, the decoded pick from `system_one` equals the target for every type.
   - The `score` concern was unfounded: the answer is a level label, `criteria.index(label)` gives the level index, and `system_one` returns probability keys "0".."n-1", which `int(key)` reads back.
2. **The base head is anti-correlated with the teacher here.** On all 440 held-out questions the untrained model scores 23.4%, below chance (29.3%). It is worst on `score` (8%, picks "medium/high" threat while the teacher says "none/low") and on `war` (5%, picks the war option while the teacher says no war), and 0% on `research` and `treaty`.
3. **The old checkpoint is essentially the base model.** Evaluated on all 440 it scores 23.4%, the same as the base, with identical per-type numbers. Its 12% on the first 100 questions is that same behaviour on early turns, where the teacher mostly says "no war / low threat / keep the default". 300 steps at batch 1 and lr 2e-5 moved the last scorer layer by only about 20% and flipped almost no argmax. "Below chance" came from a head that starts wrong plus too little training, not from training in the wrong direction.
4. **The old evaluation hid this.** The first 100 questions are all from early turns. The majority and scripted baselines on those 100 (68% and 67%) are far higher than on the whole game (50% and 54%), and the overall picture is very different.

**Pipeline changes** (`tools/llm-seat/laya-train.py`):
- Evaluates all held-out questions, per type and per family, against chance, the teacher-majority-per-family baseline (learned from the training games) and the scripted default.
- Splits agreement into "teacher agreed with scripted" and "teacher overrode scripted".
- `--epochs` (0 evaluates the untrained base), batched training, hard labels by default (`--soft-labels` to use the teacher's stated confidence), `--unfreeze N` for the last N encoder layers, and `--out` must be passed explicitly to save.
- With a frozen encoder it caches the encoder output once (about 10 minutes when other jobs share the GPU, then epochs are fast), and confirms the final number with Laya's own `system_one`.

**Results** (game 3 held out, all 440 questions; trained on games 0 to 2, 856 questions):

| | Agreement with teacher | Choice (280) | Score (124) | Yes/no (36) | Teacher agreed w/ scripted (172) | Teacher overrode scripted (144) |
|---|---|---|---|---|---|---|
| Random chance | 29.3% | 28.6% | 25.0% | 50.0% | 29.5% | 32.8% |
| Untrained base Laya | 23.4% | 27.9% | 8.1% | 41.7% | 15.1% | 46.5% |
| First fine-tune (300 steps) | 23.4% | 27.9% | 8.1% | 41.7% | 14.5% | 47.2% |
| 6 epochs, head only, lr 1e-4 | 49.1% | 45.7% | 56.5% | 50.0% | 45.9% | 46.5% |
| 6 epochs, last 4 encoder layers unfrozen | 50.2% | 46.8% | 58.1% | 50.0% | 59.3% | 32.6% |
| 20 epochs, head only, hard labels | 46.4% | 45.0% | 48.4% | 50.0% | 39.0% | 53.5% |
| 20 epochs, head only, soft labels | 50.2% | n/a | n/a | n/a | 54.1% | 47.2% |
| 30 epochs, head only, lr 3e-4 | 50.7% | 48.2% | 55.7% | 52.8% | 54.7% | 41.7% |
| **Teacher's most common answer per question family** | **50.5%** | 47.1% | 58.1% | 50.0% | 65.7% | 25.7% |
| **Scripted AI's default (316 questions that have one)** | **54.4%** | 53.9% | n/a | 58.3% | 100% | 0% |

The agreement numbers move by about 2 points between epochs (440 questions, one game), so 49 to 51% should be read as "the same". Training agreement reaches 79% after 20 epochs while held-out falls to 46%, so more epochs over-fit.

**What the model learned.** It learned the per-family answer habits: threat is mostly "low", war is mostly "none", treaty offers are mostly "none". That alone earns 50%. It does not read the state well enough to improve on those habits: `research` (pick the tech from the option text, which includes numbers such as "score 7600, 7 turns") stays at 5% to 33% against 37% for the majority answer, and `focus` (which city to focus) is at or below chance. Where the teacher overrode the scripted AI, the model got 33 to 54% depending on the run, which is mostly the same habits and not understanding.

**What limits it.**
- **Data size:** 856 training questions from 3 games, and a single held-out game. Run-to-run noise is about 2 points, so none of the variants can be ranked.
- **Frozen encoder and the option format:** questions whose answer depends on reading numbers or comparing options in the text (research, focus) need more than a head on frozen features. Unfreezing 4 of the 28 layers did not change that in 6 epochs.
- **Labels:** the teacher agrees with the scripted AI 54% of the time and its stated confidence is mostly 45 to 90%. Teacher answers on the `production` question differ from the scripted default in 131 of 134 cases, which is consistent but only one habit to learn.
- **The ceiling is low for the same reason as before:** the teacher lost all 4 games, so these labels mean "what a strong player would pick", not "what wins".

**Next:**
1. Do not generate more teacher data on this evidence alone; first decide what Laya is for. If it only needs to reproduce the scripted AI's default plus the teacher's habits, the family-majority baseline already does that for free.
2. If we want Laya to beat the baselines, the cheapest next experiments are: give the head the option facts as structured numbers, or reformulate `research` and `focus` as score questions per option; and add data in games with more varied states (the 4 games are all lost games with similar mid-game shapes).
3. Always report against the family-majority and scripted baselines on all held-out questions; never on the first N.

## 2026-10-06 — Concluded

Concluded 2026-10-06: removed in favour of search-based AI; see the design doc decisions log. The Laya scripts, the LLM seat and its deciders are deleted from the repo (they remain in git history), as are `~/LegatumRuns/laya/` and the Hugging Face cache of the model. This journal stays as the record of what was tried.

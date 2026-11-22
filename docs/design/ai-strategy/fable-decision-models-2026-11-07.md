# Hosted decision models and LLM teachers for Hard (Fable, 2026-11-07)

Decision memo, second research round. Written against master `274953523` (RULES_VERSION 21) and the operation-search branch `claude/operation-search` at `2e4ed6e4d`, whose first round was running while this was written (12 of 22 windows done). Companion to `fable-2026-11-07.md` (the plan this memo slots into). Internal numbers name their file; external claims carry a link in section 10. Dates are wall-clock with agents, from Sat 2026-11-07.

**What changed since the first memo.** By "system 1" you meant hosted decision models: TypeSafe's Jev and OpenAI's Decisions API. The first memo read it as a learned policy and argued against that. This memo takes the models as they are. Two facts from the lead reshape it: we hold no OpenAI Platform key and no TypeSafe key (Codex runs gpt-6-luna, sol and astra under your ChatGPT subscription only; Claude Haiku, Sonnet and Opus run under the Claude subscription through our agents), and you asked whether Luna or Haiku could simply be the teacher with no decision model at all.

## 0. One page

**What the decision models are.** Jev (TypeSafe, launched 2026-09-15, GA) and the Decisions API (OpenAI, public beta 2026-10-06, now GA, `POST /v1/decisions`, `gpt-6-luna` only) are **classifiers over a described situation**. You send a state as text (or JSON, or an image) and typed questions; they return a choice from your list with per-option probabilities and a confidence, a yes/no probability, or a score on a rubric. They generate no text. Jev: $0.042 per million input tokens, output free, p50 0.21 s. Decisions: $0.10 per million input, output free, about 0.15 s on OpenAI's slide. Independent tests (CMU's JEV-as-a-Judge; BTInsights' survey-coding study) put them within a few points of a frontier LLM when the answer can be read off the text, and clearly behind when it has to be derived (math, code, logic). Both warn that accuracy falls as irrelevant state grows, and Jev's probabilities move when option order changes. The "Laya" in the third-party comparison is the same `convaiinnovations/laya` model we fine-tuned and removed on 10-06: open weights, 421M parameters, 33 ms, self-hostable, and at our scale it learned 46–51% agreement with its teacher against a 54% scripted baseline.

**Verdict.** For an **offline teacher** their two advantages, 150 ms latency and sub-cent cost, buy us nothing: we have about 22 decisive windows on three tapes, not a million tickets, and the operation search takes 70–260 s per window anyway. What they cannot do is the thing we most need: **propose**. They choose from a menu; our measured cap since 10-06 is the menu. So: **no decision-model API in the first run.** Use the general models we already pay for, Luna through Codex and Haiku through our Claude agents, in two roles the search cannot fill, with **the operation search as referee** wherever it can score the answer:

1. **Proposer.** Read the window as text plus your reasoning notes, and name operations the generator missed, in the generator's own grammar, so the search can price them. This attacks the candidate-set cap directly.
2. **Long-horizon strategist.** Whole-game posture, Alliance offers, Peace with the hold leader, which neighbour to settle away from: judgements beyond a 30-turn rollout, where your stated reasons are already in words.

Add Jev and the Decisions API later as cheap **choosers** inside the same harness if keys appear (under $1 for the whole set); they are a side measurement, not the plan.

**This is not Laya again.** Laya's LLM set posture every five turns for a seat that then lost all four games; its labels were the teacher's opinion with no referee; its student was a text classifier over a briefing that was itself buggy; its score was agreement with the teacher. Here the LLM never issues an order, its picks are scored against the search's rollouts and your own recorded choice, the student is the integer value net plus the operation generator in Rust, and the score is the designer-seat score (held out 29.0%, training 44.4%, frozen control `516cca016`).

**The experiment (Sun 11-08 to Tue 11-10, $0 extra).** The 22 windows the operation search is analysing on Sparta 8.9.26 (T18–30), Swiss 8.9.26 (T30–40) and India 8.9.25 (T13–22, T148–152). For each: a text description of the seat's own `PlayerView`, the search's candidate list, and your choice on the tape. Luna and Haiku each answer three times in chooser mode (pick and rank the candidates) and proposer mode (name up to three operations not on the list); a strategist question at the five takeover turns of each tape. About 500 calls, about 2M input tokens in all, covered by the subscriptions; at API list prices about $0.25 per model. Metrics: agreement with the search's confirmed best, paired regret against Hard's plan, agreement with your tape choice, consistency across repeats, and how many proposals the search then confirms at +10 pp. **Success:** either role clears its bar (section 6). **Kill:** neither does, and we drop the LLM to offline review, where it already earns its keep.

**Where this fits.** It feeds the first memo's loop; it replaces nothing. The proposer widens the skateboard's generator (increment 1, Tue 11-10 to Fri 11-13); the strategist's recorded picks become a prior term in the value refit (increment 2) and the Peace and Alliance operations of increment 3. The first memo's dates and the 11-27 checkpoint stand.

**What you must provide.** Nothing for the first run. For the second: an OpenAI Platform key and a TypeSafe key, stored in a git-ignored `.env` or `~/.config/legatum/keys.env`, never committed (the hygiene gate and gitleaks already scan for them). And a decision on whether a rules freeze holds through the run, as the first memo asked.

---

## 1. What these models are, and what they are good at

### 1.1 The facts, as of 2026-11-07

| | OpenAI Decisions API | TypeSafe Jev | Laya (open weights) |
|---|---|---|---|
| Status | Public beta 2026-10-06; OpenAI's guide now says GA (200 → 400 questions per request) | Launched 2026-09-15, GA, no waitlist | Apache 2.0, three checkpoints released Sept 2026 |
| What it returns | `predicate` (probability), `choice` (choice, per-option probabilities, confidence), `score` (2–10 ordered levels, weighted score, probabilities, confidence); a `refusal` answer type exists | `choice`, `score`, `noul` (yes/no probability), `box` (image location); all questions in a request answered in parallel against one state | Typed answers (choice, score, yes/no) from a single forward pass |
| Input | Text, JSON (`input_json`), images; `gpt-6-luna` only | Text, JSON, images per the docs (third-party reports said text only in September) | Text, 512 or 1,024 token context |
| Price | $0.10 per 1M input, cache hits discounted, output free; 10% regional uplift; input rate doubles over 272K tokens | $0.042 per 1M input, output free (OpenRouter and TypeSafe list price) | $0 marginal, self-hosted |
| Latency | 150 ms on OpenAI's DevDay slide against 1.6 s for a normal Luna call; no published benchmark | p50 0.21 s, p95 0.34 s, p99 0.75 s (OpenRouter telemetry, 2026-10-01) | about 33 ms self-reported |
| Limits | Up to 400 questions a request; no published maximum of options, context length, rate limits, or determinism controls (no seed) | 64K context per TypeSafe's docs (32K on OpenRouter); no published seed | 421M parameters (ModernBERT-large) |
| Independent accuracy | None published by OpenAI. BTInsights (a vendor): F1 77.1 against Jev's 75.4 on survey coding, Decisions about 7× Jev's cost per code | CMU JEV-as-a-Judge (arXiv 2609.26550): within 3 points of GPT-6 where the verdict can be read off the text, behind where it must be derived (math, code, logic); 0.36% of GPT-6's fee; a confidence-gated cascade matches GPT-6 | Self-reported 0.766 on a typed-decisions benchmark; our own fine-tune: 46–51% held out |
| Known weaknesses | Beta documentation gaps; no cache discount on question content | Accuracy falls as irrelevant state grows (TypeSafe's own docs); probabilities shift with option order; a nine-point drop on style-adversarial inputs | Weak zero-shot (0.362 against 0.318 random on the base checkpoint); needs labels |

Sources in section 10. Several numbers come from third-party blogs and vendors; the ones I would trust are OpenAI's own guide (schema, GA, pricing), TypeSafe's docs (question types), OpenRouter's telemetry (Jev latency) and the CMU paper.

### 1.2 What they are good at, honestly

They answer one kind of question well: **given a described situation and a fixed set of options, which option, and how sure.** That makes them good at routing, classification, guardrails on agent actions (the TechCrunch demo: $2.94 to monitor an agent run against $372 with a frontier LLM), survey coding and judging. They are calibrated enough that a confidence threshold separates the cases they get right from the ones to escalate (the CMU cascade result). They are cheap and fast enough to run on every event in a production system.

### 1.3 Where they would beat or complement search plus value net

- **Reading rules stated in words.** "Don't settle toward an aggressive neighbour", "research the reveal tech first", "a two-front war near a win is a distraction", "an Alliance with an ally at 0 HP is a trap" (`docs/playtests/human-reasoning.md`, 2026-11-07). These are predicates over a described situation. A 30-turn rollout cannot see the T24 Alliance's consequences; a classifier told the ally's capital is at 0 HP can.
- **Long horizons.** Founding pays after the search horizon (the diff's Settler-route rows are negative inside 30 turns, positive after: `docs/playtests/diffs/patterns-2026-11.md`). Peace with the hold leader decides games 20 to 60 turns later. The value net was fit on self-play and prices positions, not intentions.
- **Cheap consistency checks at scale**, if we ever had scale: a decision model could score every plan review point of every self-play game for the price of a coffee. We do not have that problem yet.

### 1.4 Where they fail, for us

- **They cannot propose.** No text out, so no new operation, no "march on Karakorum, it has one Spearman" unless that is already an option. The 10-06 tactical search and the 10-27 plan search both failed on the candidate set, and the first memo's kill criterion for the skateboard is the same. A chooser cannot fix a chooser's cap.
- **Spatial and tactical reasoning.** Which pass a lookout should hold, whether five Hoplites two tiles out can storm a walled city with one Spearman, whether the Celts' four units at Gergovia can reach an empty Tlatelolco 11 to 15 tiles away: these are distance, terrain and combat arithmetic. The engine computes them exactly in microseconds; a model reading a text rendering guesses. The CMU result (behind wherever the verdict must be derived) is this failure named.
- **Long state descriptions.** A fogged `PlayerView` at T30 has 10 to 30 units, 2 to 6 cities, 3 to 7 known rivals, wars, offers, treasury, research and a map. Rendered honestly it is 1,500 to 3,000 tokens; TypeSafe says accuracy falls with irrelevant state, and Laya's teacher taught us that whatever the briefing omits or garbles becomes a wrong label (learnings 2026-10-05: of its top bug claims, one was real; the rest described the briefing).
- **Consistency.** No seed on either API. Jev's probabilities move with option order. A teacher that answers differently on repeats is a noisy label; the experiment measures this directly.
- **Latency for a live AI is a non-issue either way.** Offline, 150 ms against 10 s does not matter. Live, pillar 7 gives us seconds while you plan, so even a general LLM would fit the clock; what rules both out live is the internet dependency and the determinism cost, which section 4.5 handles by recording picks in the tape.

## 2. The options, now that we know what the models are

| | (A) Search plus value net only (the first memo) | (B) A decision model as the teacher's chooser | (C) A general LLM as proposer and strategist, search as referee (recommended) | (D) A live hosted-model level in the shipped game |
|---|---|---|---|---|
| What it adds | Nothing new; the loop in `fable-2026-11-07.md` §4 | A cheap, consistent-ish pick among search candidates; calibrated probabilities as a prior | New candidates the generator lacks; judgement at horizons the search can't reach; readable reasons | A "Teacher" difficulty that asks a model each plan review |
| Needs | Mac nights | A Platform or TypeSafe key; a state renderer; a strict option grammar | Codex (Luna) and Claude (Haiku, Sonnet) budgets we already have; the same renderer and grammar | Internet at play time, keys per player, tape recording of every pick |
| Ground truth | The search itself and the designer-seat score | The search's confirmed best (where its interval is clear) and your tape choice | Same, plus the proposals the search confirms | Only the designer-seat score, slowly |
| Main risk | The candidate set is the cap (10-06 again) | Buys speed we don't need; cannot widen the set; rendering bugs become labels | Hallucinated operations the executors can't run; inconsistency; prompt-level overfitting to your three tapes | Pillar 7 and determinism costs, internet, and we'd be shipping something we haven't shown is better |
| Cost | $0 | under $1 per run at list prices | $0 extra (subscription) | per-player API spend |
| Fits "no rewrite until we know it's better" | Yes | Yes | Yes | No |

(C) is (A) with one more source of candidates and one more labeller, both scored by (A)'s referee. (B) is a measurement we can add to (C) for pennies when keys exist. (D) is the one thing not to do yet.

## 3. The architecture: a separate engine that trains the main engine

Your phrase was "a separate engine that we use to train the main rules engine." The honest version: the **operation search on the real engine is already that separate engine** (`crates/sim/examples/operation_search.rs`): it enumerates operations, rolls each forward on determinized states, and logs where it beats Hard. What it lacks is imagination (its templates) and horizon (30 turns). The LLM teacher supplies both, and the search keeps its job as referee.

```
your tapes ──► decisive windows ──► STATE RENDERER (Rust, PlayerView → text + JSON, public info only)
                                            │
            ┌───────────────────────────────┼─────────────────────────────┐
            ▼                               ▼                             ▼
   PROPOSER (Luna/Haiku):          CHOOSER (later: Jev,         STRATEGIST (Sonnet/Luna):
   new operations in the           Decisions; now Haiku):       posture, Alliance, Peace,
   generator's grammar             pick + rank the list         settle direction, war aim
            │                               │                             │
            ▼                               ▼                             ▼
   OPERATION SEARCH (referee): prices every candidate, old and proposed, 64 determinized samples,
   common random numbers, confirm on fresh samples ── logs picks, prompts, hashes, outcomes
                                            │
                                            ▼
   STUDENT (Rust, integer, no internet): generator templates + value-net refit + operation prior
                                            │
                                            ▼
   designer-seat score on held-out tapes (primary); 8-map gate (secondary)
```

### 3.1 Where each model belongs in the hierarchy

| Layer | Decision | Horizon | Who decides in the lab | Who judges | Ships as |
|---|---|---|---|---|---|
| **Strategy** | Posture (expand, rush, turtle, race); accept or refuse an Alliance; Peace with the hold leader; which neighbour to settle away from; which war to close | 20 to 100+ turns | LLM strategist, reading your reasoning notes | Designer-seat score and takeover curve after distillation; your tape choice now | Value-net prior terms and Peace/Alliance operations (increment 3) |
| **Operations** | March on X and strike; found now; escort or snipe; buy then strike; lookouts; wall up; small veteran army | 5 to 30 turns | Operation search; LLM **proposes**, decision model or LLM **chooses** as a tie-break | The search's rollouts (confirmed, interval clear of zero) | Templates in `generate()`; value-net refit; gene `operations` |
| **Tactics** | Per-unit orders in a battle; storm or wait; which target | 1 to 10 turns | Training-mode search, siege oracle (`docs/design/training-mode/`) | The battle searcher | Fitted siege valuation, per-unit order value |

No language model touches tactics. Decision models, if they ever earn a place, sit in the operations row as choosers.

### 3.2 Prompt and state encoding

One Rust function renders a window from the seat's `PlayerView` only (`crates/core/src/view.rs`: `CityView`, `UnitView`, `TileView`, known players, wars, offers, treasury, research). **Public information only**: never the true state, never fogged garrisons. The same rule the search follows (`search::determinize` from the view, "an operation chosen for what the seat can't see would teach the wrong reason", `operation_search.rs` header). Determinizing is the search's job after the pick; the LLM reasons on what you would have seen.

Contents, in this order so the stable part caches: (1) the rules the model needs, condensed from `legatum-rules` (one order per unit per turn, simultaneous resolution, Walls as the garrison, era resources, the VP track); (2) **your reasoning notes, condensed to about 40 lines** from `human-reasoning.md` (neighbour's faction as a threat prior; information is worth a unit; don't settle toward the aggressive neighbour; conquest is the fastest economy; keep veterans alive; close fronts you don't need; reveal tech first); (3) the window: turn, era, your faction and its kit, cities with population, Walls, what they build, treasury and income, research, your units with positions as named places ("two tiles east of Sparta, on the pass toward Karakorum"), known rivals with public army and city counts, wars and offers, visible enemy units with distances to your cities, recent events inferred from view diffs (the LLM-seat lesson: the view has no events, so diff consecutive views and say "a city finished an item" rather than "builds nothing", learnings 2026-10-05); (4) the candidates as the search names them (`hard`, `plan:Economy`, `strike:Karakorum`, `buy_strike:Sarai`, `expand_now`, `escort_settlers`, `snipe_settlers`, `lookouts`, `wall_up`, `small_army_economy`, `refuse_peace`, `accept_peace:T`, and generator 2's `faction_kit`, `fortress_economy`, `expand_walled_escorted`, `veteran_strike:X`, `short_strike:X`), each with one line saying what the executors will do; (5) the question.

Two variants per window: **with** and **without** your notes, so we measure what your words are worth. Option order shuffled per repeat (Jev's order sensitivity is the reason). Output is JSON with a fixed schema (choice, ranking, up to three proposals in the grammar `template(target, modifier)`, one sentence of reason), parsed strictly; a proposal outside the grammar is logged and dropped, never executed.

### 3.3 Distillation: how picks become a Rust AI with no internet

1. **Proposals → templates.** A proposed operation that the search confirms at +10 pp or more on two windows becomes a permanent template in `generate()`. This is the whole point of the proposer: it writes candidates, the search validates them, Rust keeps them. Nothing hosted survives into the shipped game.
2. **Strategist picks → a prior term.** Each window's pick, with its probabilities, joins the value-net rows (`tools/sim/train-value.py`, 111 s a fit) as a "teacher agreement" feature on operation-level candidates, fit as integer weights in the same JSON the value net already loads (`crates/ai/src/value_net.rs`, 7 µs a call). `choose_plan` prices candidates by the refit value; the prior moves the price, it never overrides the search. This is the move-prior half of the first memo's "system 1 = evaluator and move prior".
3. **Strategy-level rules → operations with a gate.** Peace with the hold leader, Alliance with an ally at 0 HP, settle direction: each becomes a priced operation (`refuse_peace` already exists) whose price the refit learns from the strategist's labels and the search's outcomes. Not a hard-coded guard; a priced candidate, as the design skill requires.

### 3.4 How picks are judged

| Judge | Measures | Available |
|---|---|---|
| The operation search's confirmed best | Agreement with the best; **paired regret** = win share of the best minus win share of the pick, on the same determinized samples | Now, where the confirm interval is clear of zero; the first round at 16 samples has intervals of about ±25 pp (Sparta T20 `lookouts` +31.3 [1.7, 60.8]; Swiss T38 `buy_strike:Gytheio` +43.8 [18.6, 68.9]; most others straddle zero). The referee needs 64 samples to judge a teacher; section 6 budgets that |
| Hard's own plan | Does the pick beat Hard's plan on the same samples | Now |
| **Your choice on the tape** | Did the model pick what you did; where it disagrees, does the search side with you or with it | Now, free, for every window. The one judge with no horizon problem and no noise |
| Designer-seat score, held out | After distillation only: does Hard convert your positions more often | Weekly |

### 3.5 Cost per game and per training batch

| Item | Tokens | Via subscriptions | At API list price |
|---|---|---|---|
| One window, chooser mode (state about 2,000, candidates 500, notes 1,200, answer 200) | about 3.9k | $0 | Haiku 5.5 ($0.10 / $0.50 per M): $0.0005. Luna ($0.10 / $0.50): $0.0005. Decisions ($0.10 in, free out): $0.0004. Jev ($0.042): $0.0002 |
| One window, proposer mode (answer 400) | about 4.1k | $0 | Haiku or Luna $0.0006; Sonnet 5.5 ($2 / $10): $0.012 |
| One whole game labelled every 2 turns to T150 (75 windows, both modes, one model) | about 600k | $0 | Haiku $0.08; Sonnet $1.60 |
| A 100-game training batch (the first memo's bicycle scale) | about 60M | strains the Codex or Claude daily budget (Codex ran out on 10-27) | Haiku about $8; Decisions as chooser only about $5; Sonnet about $160, which is why Sonnet is for judgement calls, not bulk |

Prices from the Claude pricing table cached 2026-10-06 (Haiku 5.5 $0.10 / $0.50, Sonnet 5.5 $2 / $10, Opus 5.5 $4 / $20 per million) and OpenAI's and TypeSafe's published rates. The referee's compute dwarfs all of it: one window at 64 samples is 5 to 17 minutes of the Mac.

### 3.6 Latency per turn

Offline: irrelevant; `codex exec` has 5 to 15 s of process start per call and that is fine overnight. Live, if a "Teacher" level ever existed: Decisions 0.15 s and Jev 0.21 s fit inside the human's planning time with room to spare; Haiku about 1 to 3 s does too; a Sonnet strategist at 5 to 15 s only at plan review points. Pillar 7 is satisfiable. The internet dependency and recording are the real costs, not the clock.

### 3.7 Determinism, with picks in the tape

Tapes record orders; the engine recomputes everything else, so coach reviews, takeover curves, regression pins and bug reproduction need exact replay. A hosted model is a non-deterministic oracle (no seed on either API), so it is treated like dice that were rolled once: every pick is recorded, and replay reads the record instead of calling. One record per pick: `{turn, seat, decision_id, prompt_hash, model, model_version, response_hash, choice, probabilities}`; the prompt itself goes to a sidecar log, not the tape. Offline this is already how the Training log works (`<app data>/training/<id>.jsonl`, `mode: training`). A game replayed without the sidecar still replays, because orders are what the tape holds. Shipped Hard is untouched: integer weights from JSON. Engineering, not a blocker, as the lead said.

### 3.8 Failure modes

1. **The briefing is wrong and the label inherits it.** Laya's teacher filed 64 findings a game and most described its own briefing. Guard: a renderer test suite against known states, view-diff events, and a one-in-ten human read of prompts. A model that says "Karakorum has one Spearman" is reading our text; if the text is wrong, so is the label.
2. **Proposals the executors can't carry.** A proposed "siege Karakorum from the hills with the Archers" is not an operation Hard can run. Guard: the strict grammar; anything outside it is dropped and counted (the drop rate is itself a metric).
3. **Inconsistency.** Three repeats with shuffled options; a window where the model disagrees with itself is not a label.
4. **A noisy referee.** At 16 samples the search cannot tell a teacher from a coin. 64 samples and common random numbers before any judgement.
5. **Overfitting to three tapes and one player.** Your three held-out tapes stay untouched; your own standing rule ("don't assume the way I played is the only way"); the census ratio as a secondary gate.
6. **Information leak through the harness.** `codex exec` is an agent, not a raw model call. Run it with `-s read-only` in an empty scratch directory holding only the prompt, so it cannot open the tape's true state. Same for `claude -p`.
7. **Model drift.** Hosted models change under us; the record carries `model_version`, and a round's labels are compared teacher-to-teacher on the fixed window set, as `learning-loop.md` §2 does for the search.
8. **Rules drift.** Twenty-one `RULES_VERSION` bumps in eight days; labels carry `rules_version` and die with it. The freeze the first memo asked for.

## 4. Why this is not the Laya pipeline removed on 10-06

| | Laya pipeline (removed 2026-10-06) | This |
|---|---|---|
| The LLM's role | **Actor's strategist**: set posture, focus, research, war target every 5 turns for a live seat; the scripted AI executed | **Proposer and labeller**: names candidates and judges them; never drives a seat |
| Ground truth | None. Labels were the teacher's opinion; the seat lost all four games | The operation search's rollouts on the real engine, your tape choice, the designer-seat score |
| Student | A 421M text encoder classifying a text briefing (`convaiinnovations/laya`) | The existing integer value net and the Rust operation generator; weights in JSON |
| Metric | Agreement with the teacher (46–51% held out, below the 54% scripted baseline) | Regret against the search's best; designer-seat score held out |
| Decisions labelled | Habit-level questions every 5 turns (posture, focus) in every game | Operation and strategy choices at decisive windows only (22 now), where the takeover curve says the game turned |
| Cost shape | 73% escalation to Opus; 1,100 s of LLM time per Quick game | Haiku and Luna for bulk, Sonnet for a handful of judgement calls; the Mac does the expensive part |
| What carried over and still holds | Replay before believing an LLM's bug report; the view has no events, so diff views; fix the briefing before generating data | All three, built into section 3.2 and 3.8 |

The design skill's rejection ("capped by the teacher; the LLM strategist only tied Hard") stands against imitation. Here the teacher is capped by the search, and the search is capped by what it is given to price. The LLM's job is to give it more.

## 5. The cheap side experiment

### 5.1 Decision points

The windows the operation search is analysing, with `--every 2`: Sparta 8.9.26 T18–30 (7 review turns), Swiss 8.9.26 T30–40 (6), India 8.9.25 T13–22 (6: 13, 15, 17, 19, 21, 22) and T148–152 (3). **22 windows.** Each comes with Hard's plan, 9 to 15 candidates, the search's selection and confirmed gains, and your actual orders on the tape. Round 1 (16 samples) is running; the referee rerun at 64 samples with `--confirm 64` runs Sun night.

### 5.2 Candidate lists and the text description

The candidates are the search's own (`generate()` and `widen()`, section 3.2), named exactly as the JSONL names them so answers join on the name. The description is the renderer of section 3.2: public view only, named places, distances, view-diff events, the faction kit, your condensed notes in the "with notes" variant. One example, Sparta T20 (the window where `lookouts` confirmed +31.3 pp): "You are Sparta, turn 20, Ancient era. Two cities: Sparta (pop 3, no Walls, building a Hoplite) and Gytheio (pop 1, no Walls, building a Settler). Treasury 46 Gold, +4 a turn. Five Hoplites: three two tiles west of Karakorum (Mongol capital, pop 2, no Walls; one Spearman seen last turn), two at Sparta. Known rivals: Mongols (2 cities, army small, at War with you since T17), Germany (2 cities, army small, Peace), Celts (not met). The Mongols offered Peace this turn. Hard's current plan: Campaign against the Mongols, target Sarai."

### 5.3 Both roles, both models, the call count

| Run | Model and route | Windows × repeats × variants | Calls | Input tokens |
|---|---|---|---|---|
| Chooser | Luna via `codex exec -m gpt-6-luna -s read-only` in an empty dir | 22 × 3 × 2 | 132 | 0.5M |
| Chooser | Haiku 5.5 via `claude -p --model haiku` (as the removed `claude-decider.sh` did) | 22 × 3 × 2 | 132 | 0.5M |
| Proposer | Luna and Haiku | 22 × 3 × 2 models (notes on) | 132 | 0.5M |
| Strategist | Sonnet 5.5 and Luna, at T2/T20/T40/T60/T80 of the three training tapes: posture, who to attack or settle away from, Peace and Alliance stance | 15 × 2 × 2 | 60 | 0.3M |
| **Total, first run** | | | **about 460** | **about 1.8M**, all under the subscriptions; at list prices about $0.25 per cheap model and $1 for Sonnet |
| Second run, if keys appear | Decisions API (`choice` and `predicate`) and Jev (`choice`) as choosers, same prompts | 22 × 3 × 2 each | 264 | 1.0M: Decisions about $0.07, Jev about $0.03 |

Well under the $20 cap. The expensive item is the referee: 22 windows at 64 samples, roughly 3 to 6 Mac-hours, one night.

### 5.4 Metrics

1. **Agreement with the search's confirmed best**, counted only on windows whose confirm interval is clear of zero (expect 6 to 10 of 22 at 64 samples).
2. **Paired regret against Hard's plan**: win share of the model's pick minus Hard's plan, on the same samples, with its interval. A chooser earns its place if its mean regret is positive and its worst pick is not a disaster.
3. **Agreement with your tape choice**, all 22 windows, and a three-way table (you, the model, the search's best).
4. **Consistency**: share of windows where three repeats with shuffled options agree on the top choice; mean rank correlation across repeats.
5. **Proposer yield**: proposals in grammar; distinct new operations; the number the search then confirms at +10 pp with the interval clear of zero.
6. **Strategist**: agreement with your stated reasoning at the same turn (the notes exist for these games), and whether its Peace and Alliance calls match the coach's measured branches (Swiss 8.9.26: declining the T24 Alliance keeps Zürich 48/48 against 0/48).
7. **Latency and tokens** per call, for the record.

### 5.5 Success and kill criteria, written before the run

- **Chooser success:** agreement with the confirmed best at least 60% where the referee is clear, consistency at least 80%, mean regret against Hard positive. Then the chooser is worth a prior term in increment 2. **Kill:** under 45% agreement or under 60% consistency: no better than Hard's own plan choice, drop the chooser role.
- **Proposer success:** at least two distinct proposed operations confirmed at +10 pp that the generator lacked. Then the proposer runs on every new coached tape and its confirmed proposals become templates. **Kill:** zero confirmed in 22 windows, or a drop rate over 50% (it cannot speak the grammar).
- **Strategist success:** matches your stated reason or the coach's measured branch in at least 10 of 15 questions, with the "with notes" variant clearly ahead of "without" (your words carry information). **Kill:** no better with the notes than without.
- **Global kill:** all three fail. The LLM stays in offline review and coaching, where it already works, and the first memo's plan runs unchanged.

### 5.6 What you must provide

- **First run: nothing.** Luna runs under the Codex subscription; Haiku and Sonnet under the Claude subscription through the agents. The run is about 1.8M tokens, a small share of either daily budget, paced by the ops supervisor.
- **Second run:** an OpenAI Platform API key (Decisions is billed through the API, not ChatGPT) and a TypeSafe API key (`api.typesafe.ai`, or via OpenRouter). Expected spend under $1.
- **Key storage:** in the repo's git-ignored `.env` (`.gitignore` already lists `.env` and `.env.*`, with `.env.example` tracked holding empty names), or in `~/.config/legatum/keys.env`, loaded by the harness with `set -a; source`. Never committed, never in a tape or sidecar (those hold prompt and response hashes), never echoed into `.progress` logs. The merge gate's `tools/gates/hygiene.py` already matches `sk-proj-…`, `sk-ant-…` and `api_key = "…"` patterns, and `tools/release/export-public.sh` runs gitleaks before any public export. The open-source memory note applies: nothing personal or secret may reach the public repo.
- **Your time:** about 20 minutes to read the three-way agreement table and tell us where the model's disagreement with you was right.

## 6. How this fits the first memo's plan

It **feeds** the learned value model and **runs alongside** the search; it replaces neither.

| Increment (first memo) | Dates | What this adds |
|---|---|---|
| 0. Instrument: designer-seat score | Sun 11-08 to Mon 11-09 (landed early on `claude/designer-seat`: held out 29.0%, training 44.4%) | Nothing; the score is the final judge of anything distilled here |
| **Side experiment** | **Sun 11-08 to Tue 11-10** | Renderer and harness (one Sonnet agent, about a day); referee at 64 samples Sun night; model runs Mon; report Tue in `docs/sim-results/ai-quality/llm-teacher-1110/README.md` with the three-way table. **Bottleneck:** the referee rerun and the subscriptions' daily budgets |
| 1. Skateboard: operation search | Tue 11-10 to Fri 11-13 | Confirmed proposals become templates before the kill check, so the generator the skateboard is judged on is the widened one |
| 2. Scooter: distil behind a gene | Sat 11-14 to Wed 11-18 | The chooser's and strategist's recorded picks enter the value-net rows as a prior term; measured by the held-out designer-seat score as planned (+5 pp go, +3 pp kill) |
| 3. Bicycle: iterate, league | Thu 11-19 to Tue 11-24 | Peace, Alliance and settle-direction operations priced from strategist labels plus search outcomes; the proposer runs on every new coached tape |
| Checkpoint | Fri 11-27 | Unchanged: you lose one of three, or held-out score 40% |
| Second run (keys) | Any day after Wed 11-11, if you open the accounts | Decisions and Jev as choosers on the same 22 windows; a table, not a plan change |

Nothing hosted ships. The shipped Hard stays integer Rust reading JSON.

## 7. Push back

**Where the idea is weak.**

1. **A decision model solves a problem we don't have.** Its edge is cost and latency at scale. We have 22 windows and a referee that takes minutes per window. Cheaper and faster labels for a referee that cannot keep up are worth nothing. "Luna or Haiku as the teacher" is the right instinct, and the lead's version, proposer plus strategist with search as referee, is the right shape.
2. **"A separate engine that trains the main engine" already exists**; it is the operation search, and its outputs have ground truth. A hosted model without a referee is Laya with a bigger teacher, and Laya's result (below the scripted baseline) came from the missing referee, not the small model. Anything the LLM says must be priced on the engine or scored against your tape before it becomes a label.
3. **"Eventually these models can be hosted locally"** is true and already tested: Laya is open weights and ran on the Mac at 33 ms. A locally hosted system 1 is exactly as good as its labels. The hard part is the labels; hosting is the easy part. So the sequence is labels first, and this memo is about labels.
4. **Choosers cannot fix the cap.** Since 10-06 every search result has been capped by its candidate set. The only role that attacks the cap is proposer, and the decision models cannot fill it by construction.

**Where it is strong.**

1. **Your reasoning is in words, and the search cannot read words.** "Don't settle toward Sparta", "reveal tech first", "close fronts you don't need near a win", "an Alliance offer that joins two wars is a trap": these are the decisions that turned your 8.9.26 games (the coach's T24 Alliance branch, 48/48 against 0/48), and no 30-turn rollout sees them. A language model with your notes does. That is a real, measurable gap and the experiment measures it.
2. **The proposer role is the first cheap attack on the candidate cap.** Every previous widening was a human writing templates. A model reading your diff categories and the window can propose in minutes what took a Codex branch a day, and the search keeps only what wins.
3. **Readable reasons.** Pillar 5 is for players, but the same need bites us when Hard blunders in your game: a teacher's one-sentence reason attached to a confirmed operation is the first `--decision-trace` for strategy.
4. **"No full rewrite until we know it's better"** is right and the experiment is that gate: two agent-days, $0, a yes or no by Tue 11-10.
5. **Zero marginal cost.** The subscriptions already cover Luna and Haiku, so the question "is a language teacher worth anything here" costs us a Mac night and a day of Sonnet, not a budget line.

## 8. Decisions I need from you

1. Go on the first run (Luna and Haiku, $0) on Sun 11-08.
2. Whether to open an OpenAI Platform account and a TypeSafe account for the second run (under $1 of spend; optional).
3. The rules freeze through the checkpoint (combat, siege, Walls, VP), as the first memo asked. Labels made this week die with the next bump.
4. Whether a Sonnet strategist may read your condensed reasoning notes verbatim in prompts sent to OpenAI (Luna) as well as to Claude. They are your words about your play; nothing personal, but your call.

## 9. What I am not sure about

- Whether 64 samples make the referee clear on more than a third of the windows. If fewer, the chooser metric is underpowered and only the proposer and your-tape agreement metrics carry the run.
- Whether Luna through `codex exec` behaves like a raw model call. It is an agent harness; the empty-directory sandbox should neutralise its tools, but the first smoke test must confirm it reads nothing but the prompt.
- Whether the strategist's picks can be distilled as a prior over the existing 20 value facts, or need strategy-level inputs (relations, offers, hold state). I expect the latter, as the first memo expected for operations.

## 10. Sources

Internal: `docs/design/ai-strategy/fable-2026-11-07.md`; `docs/status/state-of-the-ai.md`; `docs/status/designer-seat.md` (branch `claude/designer-seat`); `docs/design/training-mode/fable.md`, `learning-loop.md`; `crates/sim/examples/operation_search.rs` and `tools/sim/operation-search-report.py` (branch `claude/operation-search`); round-1 output in the lead's scratchpad (12 windows, 16 samples); `docs/playtests/human-reasoning.md` (entries of 2026-11-07); `docs/playtests/reviews/2026-11-07-8.9.26-sparta.md`, `2026-11-07-8.9.26-switzerland-win.md`, `2026-11-06-8.9.25-mongols.md`, `2026-11-05-8.9.25-india.md`; `docs/playtests/diffs/patterns-2026-11.md`; `docs/learnings.md` (2026-10-05 Laya entries, 2026-10-06 removal); `docs/sim-results/laya/journal.md`; `.claude/skills/legatum-design/SKILL.md` ("AI strategy is search, not imitation"); `.claude/skills/legatum-rules/references/game-rules.md` §1 (one order per unit per turn, simultaneous resolution); `tools/gates/hygiene.py`; `tools/release/export-public.sh`; `tools/astra/runner.sh`; `crates/core/src/view.rs`.

External:

- OpenAI, Decisions API guide (schema, question types, 400 questions per request, caching, pricing, GA): https://developers.openai.com/api/docs/guides/decisions
- The New Stack, "OpenAI answers TypeSafe's Jev with a Decision API built on Luna": https://thenewstack.io/openai-decision-api-luna/
- TechCrunch, 2026-09-30, Altman's DevDay aside, Jev's $2.94 against $372 monitoring demo, TypeSafe's CEO quotes: https://techcrunch.com/2026/09/30/openais-jev-clone-could-help-the-frontier-lab-stop-its-swarming-agents/
- Firecrawl, Decisions API vs Jev (Jev $0.042 per 1M, OpenRouter latency p50 0.21 s, 64K context, accuracy falls with irrelevant state, order sensitivity): https://www.firecrawl.dev/blog/openai-decisions-api-vs-jev
- BTInsights (vendor), survey-coding F1 and the 7× cost claim: https://btinsights.ai/blog/jev-vs-openai-decisions-api-coding-accuracy/
- Flowtivity, Decisions vs Jev vs Laya (Laya Apache 2.0, 421M, 33 ms, self-reported 0.766): https://flowtivity.ai/blog/decisions-api-vs-jev-vs-laya/
- Li, Miao, Krishnan, Padman, "JEV-as-a-Judge: Accept When Confident, Escalate When Unsure" (arXiv 2609.26550): https://arxiv.org/abs/2609.26550
- TypeSafe docs, Jev question types (choice, score, noul, box; parallel evaluation): https://docs.typesafe.ai/
- Decisions API public beta 2026-10-06, pricing and question types (secondary): https://smartscope.blog/en/blog/openai-decisions-api-use-pricing-public-beta-october-2026/ ; cost analysis of rubric-heavy traffic and ZDR terms: https://www.beri.net/article/openai-decisions-api-public-beta-gpt-6-luna-pricing-cache-discount-vs-jev-clef-zdr-residency
- Laya model family and checkpoints (secondary): https://www.llmreference.com/model-family/laya ; https://www.orcarouter.ai/blog/laya-decision-model-explained
- TypeSafe funding report (secondary, unverified): https://www.techtimes.com/articles/328875/20261013/typesafe-ai-raises-870m-jev-decision-model-that-never-generates-text.htm

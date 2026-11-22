# AI optimization program: hill-climbing toward the user's own milestones

The user, 2026-10-18: "shouldn't you be able to run a zillion simulations (maybe on a small map with 4 civs) with particular civs to hit certain economy or military benchmarks that you know are possible because I've done them myself... I really need you to become an expert on this." And, the same day: "make sure whatever strategy we pick doesn't burn through too many tokens on a go-forward basis... CPU and GPU are way cheaper."

The goal is unchanged: a Hard AI the user usually can't beat, without cheating (pillar 6). This document is the plan for getting there systematically: what the research says, the milestone suite built from his games, the first unattended climb on AWS spot, the gaps that search can't reach, and a staged roadmap with cloud and token costs.

**Design rule for the whole program: no LLM in the inner loop.** Every optimizer below is a script on CPU (spot fleets) or GPU: it proposes genes, plays games, scores them against his milestones, validates on held-out games and redraws its own chart. Claude or Codex appear only at checkpoints: choosing what to optimize next, reading a one-page summary, judging design fit, and writing structural code.

## 1. What the research says

The full sweep (with links) is in the appendix. In short:

| Method | What it is | Fit for Legatum | Verdict |
|---|---|---|---|
| **CMA-ES** (pycma 4.5, Optuna 5.0 `CmaEsSampler`, our dependency-free `tools/evolve/cmaes.py`) | Evolves a population of gene vectors, learning which directions and gene combinations help | Our genes are ~150 bounded integers; one population is one cloud batch; the sim is deterministic so paired, common-random-number comparisons are free | **Use now.** Large populations (24 to a few hundred) and paired controls handle the noise |
| Optuna TPE / Bayesian | Models which settings score well, proposes one at a time | Better under ~20 genes and small budgets; weaker at batch-parallel 30+ dims | Use for small, targeted questions (a 3-5 gene sweep) |
| Nevergrad (NGOpt, TBPSA) | Portfolio that picks an optimizer from dimension and noise | Its own choice at our size and noise is CMA; maintenance has slowed | Not needed |
| **NTBEA** (Lucas et al.) | Bandit over small gene combinations, built for noisy game-AI tuning | Good for discrete switches (our 0/1 genes) | Worth adding for the switch genes |
| **MAP-Elites / CMA-ME / CMA-MAE** (pyribs, `ribs` 0.12) | Keeps the best AI for each *style* (e.g. war-start turn × cities at T100), not one champion | Directly serves pillar 10 (no dominant strategy) and gives the AI several committed strategy bundles to pick from (the user's "pick one and commit") | **Stage 3.** Used for Hearthstone deck and balance search (Fontaine et al. 2019) |
| **League / exploiters** (AlphaStar, PBT) | Champions play a pool of past champions; "exploiter" runs search only for ways to beat the champion | No neural nets needed: the pool is parameter files, exploiters are CMA runs against the current Hard | **Stage 3**, as our red team (ideas #4) |
| **Start from demonstration states** (Salimans & Chen 2018, Go-Explore) | Train from states taken out of an expert's game, phase by phase | We already have this: war scenarios cut from his saves, `extract_state`, `cut.sh` | **Use now** for the opening and war phases |
| **Milestone (potential-based) scoring** (Ng 1999) | Reward progress toward sub-goals without changing what "winning" means | His milestones are exactly such potentials | **Use now** (the suite below) |
| **Behaviour cloning** from his games | Train a policy on his decisions | About 10 games is orders of magnitude too few to train a policy (AlphaStar used ~1M replays) | Use his games as **targets, start states and tie-breaker priors**, not training data |
| **MCTS / plan-level search** (portfolio, puppet, combinatorial bandits, ISMCTS) | Roll candidate plans forward on the engine | We have it (`search`: 3 candidates, 20 turns, 4 samples). Its limit is candidate recall and the leaf evaluation, not depth (AI review #9) | **Stage 4:** better candidates and a learned evaluation, not more depth |
| **Deep RL** (Gymnasium/PettingZoo via PyO3, SB3, RLlib; burn/candle) | Learn a policy from millions of games | CivRealm (Freeciv) RL "struggles to make substantial progress in the full game" after 2M steps; microRTS needed ~60 GPU-hours for one small map; it would also need a second action interface | **Not for the full game.** Possible later for one narrow sub-decision |
| Commercial 4X AIs (Civ V flavours, Civ VI agendas, Freeciv "want" values) | Weighted utility plus difficulty bonuses (cheats) | We already do value scoring without cheats; Freeciv's own doc calls its AI "still too easy for experienced players" | Confirms the approach; nobody ships a learned 4X AI |

What the literature and our own history agree on:
- **Noise kills naive evolution.** Our 2026-09-28 run (25 generations × 16 candidates) matched stock Hard exactly on held-out seeds; the 09-30 league gave each opening about 19 CMA steps against the hundreds a 48-gene noisy problem needs, and the "best" candidate's gain shrank from +0.05 to +0.02 out of sample (winner's curse). Fixes: judge the **population mean**, not the best sample; put the built-in AI in **every batch on the same games** (paired); reseed the dice per salt; keep **held-out salts and maps**; and run far more generations.
- **Parameters can't fix missing behaviour.** The same run's conclusion ("personality knobs can't fix behavioral gaps") and the AI code review's: the chosen plan doesn't drive staging or production for most factions. Search tunes only what the genes express (section 4).
- **Start states beat long games.** Short scenarios from his saves concentrate thousands of trials on the moments where the AI loses, at a hundredth of the cost of a 250-turn game.

## 2. The human milestone suite ("known possible")

`crates/sim/data/milestone-suite.json`, built by `tools/ai-quality/milestones/extract.py` from his saves (read only). The AI takes his seat from turn 1 on the same setup (same seed, map size, factions, difficulty), every rival plays the built-in Hard, and each salt plays its own AI seeds **and its own dice**. A game's score is the mean over the map's milestones of min(AI value ÷ his value, 1): 1.0 means it matched him on every milestone by its turn.

| Map | Role | Horizon | His milestones |
|---|---|---|---|
| **Sparta, 4-player Small** (8.9.4, won T40 by elimination) | train | T40 | cities 2 / 3 / 5 at T20 / T30 / T40; rivals eliminated 1 / 2 / 3 by T20 / T30 / T40 (Athens T20, AI Sparta T29, the Mongols T40) |
| **Sparta, 8 players** (8.9.2, T87 save) | held out | T60 | Nicaea taken T22 (2 cities); 4 cities T30, 6 T40, 7 T60; rivals eliminated 1 by T22, 2 by T35 |
| **Switzerland, 8 players** (8.9.2, won T202) | train | T100 | cities 3 / 4 / 5 / 6 at T30 / T50 / T75 / T100 (10 at T150); Gold 919 at T75 and 1,706 at T100; 4 VP at T100 |
| **Switzerland, 8 players** (8.9.4, won T246) | held out | T100 | cities 3 at T60 and 4 at T100 (6 at T150, 9 at T175); 533 Gold at T75 |

The suite also records, for every checkpoint, his VP, Gold, army strength, techs and population, so new targets need no new extraction. The eight older maps of the human-seat benchmark (`hard-vs-human/maps.json`, 8.3 to 8.6) stay the full-game no-regression check.

Caveat: the map comes from his seed on today's code; terrain changes since his release can move a city site or a hill, so a milestone is "his number on a near-identical map", not an exact replay. The war scenarios (`scenarios/war/`) are the exact-replay complement.

Run it: `milestone_suite --ai-params ARMS.json --games 64 --role all` (an example in `crates/sim`; on the cloud, `tools/cloud/run-batch.sh --bin milestone_suite --ai-params ARMS.json -- --games 64 --seed 1`).

## 3. The skateboard: an unattended climb on AWS spot

`tools/evolve/milestone_climb.py`: CMA-ES (the in-repo dependency-free implementation, so the runner needs no installs) over a gene set (`tools/evolve/genesets/*.json`), with no model calls anywhere.
- **One generation is one batch.** The population, the current CMA mean and the built-in AI are arms of the same `milestone_suite` batch on the same fresh salts, so "mean minus built-in" is a paired estimate and the curve is not a lucky draw.
- **Where it runs:** `--backend auto` (default) queues the batch on the build box's idle cores (`tools/cloud/filler.sh`, already paid for, lowest CPU priority) and falls back to AWS spot if the box hasn't finished it within `--filler-wait-min`; `--backend spot` goes straight to spot and retries while the account's spot quota is full.
- **Held-out check every few generations (`--holdout-every`, default 4) and at the end:** the mean and the built-in AI on held-out salts (900001+) and on the held-out maps, never seen by the optimizer. **Stop condition:** two held-out checks in a row without a gain, or the budget.
- **Outputs, per run** (`docs/sim-results/ai-quality/milestones/runs/<run-id>/`): `log.jsonl`, `state.json` (resumable), `best.json` (an AiParams file for the gates) and `curve.png`, redrawn every generation.
- **Why CMA-ES over Optuna:** a whole population fits in one batch (Optuna's TPE is sequential at heart), it is invariant to gene scaling, our implementation round-trips its state to JSON for resumable runs, and pycma or Optuna's CMA can replace it later without changing the evaluator. Optuna stays the tool for small targeted sweeps.

### First runs (2026-10-18)

Two climbs, 40 candidates per generation, 64 fresh salts per candidate per generation (about 2,700 games a generation), held-out checks on 128 salts every 5 generations. Both were stopped at their plateau: Sparta after 25 generations, Switzerland after 17. Charts: `milestones/runs/2026-10-18-sparta/curve.png` and `-swiss/curve.png`.

**Where it ran.** Today's climbs used spot because the box was full; the filler path was then smoke-tested end to end (a generation and a held-out check on the box's idle cores, about a minute each, $0). **What it costs.** A generation is about 15 seconds of compute on one 64-core spot server; the minute it takes to launch the server costs more than the games. Both climbs, the baseline and every held-out check together came to 50 batches and about $0.43 of server time by the launcher's estimate (under $1 on the bill). The AWS account's spot quota (448 vCPUs, shared with other agents' batches) was the real limit: generations waited for a free server about a third of the time.

**Baseline (the built-in Hard in his seat, 64 salts).** Milestone score 0.21 on Sparta's 4-player map, 0.31 on Sparta's 8-player map, 0.35 and 0.40 on the two Swiss maps. In plain numbers: on the 4-player map, where he had eliminated all three rivals by T40, the AI eliminates one in about 2% of games and holds 1.5 cities at T40 against his 5. As Switzerland it has 3.3 cities at T100 against his 6, and 105 Gold against his 1,706.

| | Sparta 4p (trained) | Sparta 8p (held-out map) | Swiss 8.9.2 | Swiss 8.9.4 (held-out map) |
|---|---|---|---|---|
| Built-in Hard, held-out salts | 0.199 | 0.305 | 0.356 | 0.389 |
| **Sparta climb**, generation 25 mean | **0.245** (+0.046) | **0.416** (+0.111) | 0.388 (+0.032) | 0.317 (−0.072) |
| **Swiss climb**, generation 15 mean | 0.242 (+0.043) | 0.417 (+0.113) | **0.390** (+0.035) | 0.316 (−0.073) |

What the numbers mean:
- **The gains are real and transfer to new games.** Every held-out check, on salts the optimizer never saw, kept the training gain (Sparta +0.046 to +0.047 at all five checks).
- **They transfer to the other Sparta map** (+0.11, more than on the map trained on).
- **They don't transfer to the 8.9.4 Swiss map** (−0.07 from both climbs). There, more Settlers meant more Settlers lost and fewer cities (1.2 against 1.6 at T60). That is the per-map problem of ideas #11: one gene set doesn't fit every map.
- **The climb found expansion, not his rush.** The Sparta gain is almost all cities from Settlers: about 2.0 cities at T40 against 1.5, but still no rival eliminated by T40 (0.00 against his 3) and about 0.4 captures. Across generations the climb kept a few settings: the Settler pipeline at 3 (both climbs found this), Sparta's military knob at 100, its expansion knob near 0, peace credibility near 0 and more march patience. The war genes (siege window, storm forecast, strike HP floors) wandered without effect, and no setting produced a T20 capture. **This is the structural gap of section 4, measured:** no gene sets "six Hoplites before anything else", and no setting of the 34 war and economy genes gets the AI's army to a city in time. The Swiss climb found what he does (more Settlers, farther sites, a longer Settler chain, less science), and Gold rose from about 105 to 176 at T100. That is still a tenth of his hoard, because no gene sets a treasury target past the goal line.
- **Both climbs plateaued within 5 generations** and then wandered: the population's step size grew instead of shrinking. The genes that matter were found at once, and the rest of the 34 were noise. More generations of the same gene set won't help; new knobs will.

**The full-game gate says don't ship either set.** These are whole games on the human-seat benchmark's eight older maps (8.3 to 8.6), 64 salts each, every salt with its own dice, in one batch with paired arms (`milestones/runs/2026-10-18-gates/report.txt`, about $0.40):

| Arm | Human-seat wins | Change, 95% interval |
|---|---|---|
| Built-in Hard in his seat | 31.8% | |
| Sparta genes in his seat | 27.0% | **−4.9 pp [−9.2, −0.4]** (worse) |
| Swiss genes in his seat | 30.5% | −1.4 pp [−5.9, +3.3] (noise) |
| Sparta genes in every rival (lower is better) | 36.3% | +4.5 pp [+0.0, +9.0] (rivals got weaker) |
| Swiss genes in every rival | 34.0% | +2.1 pp [−2.7, +7.0] (noise) |

Why a real milestone gain turned into a full-game loss, and what changes next:
1. **The objective rewarded the short horizon.** "More cities by T40" (or T100) is cheap to buy with Settlers and an empty army; the full game punishes it later. Fix: put full games in the objective, not only at the gate. A generation of 40 candidates × 8 maps × 16 salts is about 5,000 full games, roughly $0.80 on spot, so a 30-generation run with a full-game term is about $25. Or use the same maps cut at T100 and T150 (`hard_vs_human --until-turn`), about 15 times cheaper.
2. **One decision set was applied to every faction.** In the gate, the Sparta-tuned set also played Switzerland, Japan, the Celts, the Mongols and Germany. Per-faction decision overrides (section 4) are a precondition for tuning one faction at a time.
3. **The war milestones need code, not genes.** No gene setting came near his eliminations. The first structural change worth making is the opening army (`opening_troops`, `muster_turn`); then rerun the Sparta climb.

Nothing from these runs is switched on. The machinery works: unattended, paired, held out, gated, and cheap. The next run changes the objective, not the optimizer.

## 4. What search can't reach (structural gaps)

A gene search can only move what a gene controls. These are the places where the AI's behaviour is fixed in code, so no amount of search reaches his milestones (sources: the AI code review `.reviews/astra-review-ai.md`, the coach playbook, and this run).

| Gap | Where | Why search can't fix it | Who is on it | The knob search needs |
|---|---|---|---|---|
| **The chosen plan doesn't drive staging and production** (review #1) | `military.rs` takes `planned_city` from the Campaign only for Sparta; `production.rs` funds the army mainly for `conquest_city` | A searched target can differ from the city the army walks to | The war follow-through work (`claude/eu4-wars`: `campaign_persist`, `war_siege`, `city_defence_weight`) | One routed campaign objective (target, stage, required force, funding share) with genes for the force ratio and the funding share |
| **The early army size is a formula, not a choice** | `production.rs::uncapped_army`: cities + 5 with a conquest target, cities + 4 for Rush | His 8.9.4 opening was six Hoplites on T2 and six more on T17 with nothing else until T21. No gene sets "how many troops before the first Settler", so the knob and production-percent genes barely move the early army (coach, 2026-10-18) | Nobody yet | `opening_troops` (units queued before the first Settler or building) and `muster_turn`, valued by the plan rather than forced |
| **Post-search overrides replace the searched plan** (review #2) | `plan.rs`: `sparta_force_campaign` and the follow-up rule, logged as `Better` | The searched choice is thrown away; tuning search genes can't win against an override | Partly the war follow-through work | Price capture healing, desertion and the follow-up chain inside the plan value; keep a feasibility floor |
| **Hard-coded target bans** (review #3) | `conquest.rs`: no conquest nominee for Germany, or for any non-Sparta AI with fewer than two cities of its own; early Sparta excludes walled cities and Celts outright | A city that's never a candidate can't be chosen at any gene value | Nobody yet | Replace bans with priced costs (walls, owner, distance, hoard), each a gene weight |
| **The siege forecast is miscalibrated** (review #4; combat lab) | `opportunity.rs`, `city_assault.rs` (four-round window) | 23% of winnable fights are called hopeless, 34% of "capture" calls fail; `siege_window` alone also raises false alarms | Combat lab, the Military Engineer AI (`claude/military-engineer`) | A capture-odds table fitted to the combat lab's 30,000 situations (data, refitted on rule changes) instead of a hand formula |
| **Defence doesn't scale with the army at the door** (coach #23) | `threat.rs::exposure_to` returns 0/1/2; stops at Walls or one shooter | Athens built a Granary with a Hoplite two hexes away; no gene says "this army takes the city in N turns" | `claude/eu4-wars` (`city_defence_weight`) in part | Run the storm forecast from the defender's side; gene for the turns-to-fall threshold that switches production |
| **Production gates return before values are compared** (review #7) | `production.rs` emergency, opening, goal and faction gates | Gene multipliers on `item_value` never see items the gates pick | Nobody yet | Log which gate chose each item first; then fold gates into comparable values |
| **Decision genes are global** | `AiParams.decisions` has no per-faction override | A Sparta-tuned set is applied to every faction that uses the file; a climb on Sparta can hurt Switzerland | Small change, Luna-safe | `faction_overrides: {Faction: {gene: value}}` overrides in `AiParams` |
| **The AI Swiss treats 1,000 Gold as a ceiling** (coach, 8.9.2) | Swiss reserve logic | He banks well past it so war spending can't knock off the goal; no gene sets the buffer | Nobody yet | A `hoard_buffer_percent` gene (target treasury above the goal line) |

## 5. What would truly move the needle

Ranked by expected gain per cost. Every step runs unattended on CPU; the token column is what a Claude or Codex checkpoint costs per cycle (launch, read the one-page summary and chart, decide), not the compute.

| Stage | What | When | Cloud cost | Tokens per cycle |
|---|---|---|---|---|
| 1 | **Fix the objective, then climb per faction.** First the two lessons of the first run: (a) per-faction decision overrides in `AiParams` (a small Luna change), so a Sparta climb only changes Sparta; (b) a full-game term in the fitness: the eight benchmark maps cut at T100 and T150 next to the milestone maps, with the full-game gate unchanged at the end. Then climb Sparta and Switzerland again, then Athens, the Mongols and the rest as his games add maps | Days | $5–25 per climb (the full-game term is most of it) | ~50k (Haiku or Sonnet launches and reads; Opus only for the keep-or-drop call) |
| 2 | **Open the knobs search can't reach** (section 4), in this order: `opening_troops` and `muster_turn` (the rush his Sparta games are made of); priced target costs instead of bans; the routed campaign objective (the war follow-through work); a Swiss treasury buffer. Each lands off by default with a gene, then stage 1 tunes it | Weeks; one change at a time | Stage 1 again per change | The expensive part: 0.3–1.5M tokens per structural change (an implementing agent plus review). Do these on Luna where the spec is mechanical |
| 3 | **Strategy diversity with MAP-Elites** (pillar 10): keep the best genes for each cell of (first war turn × cities at T100) per faction, so Hard has several committed whole-game bundles (rush, wide, turtle, economic) and picks one per game from faction, map and neighbours. A pure-Python archive on top of the same evaluator (pyribs can't be installed under Homebrew's Python; a venv would allow it) | 1–2 weeks | $10–30 per faction archive | ~50k per archive |
| 3b | **Exploiter league** (AlphaStar's idea without neural nets): a climb whose only goal is to beat the current Hard from his seat; each exploiter that wins becomes a benchmark opponent and a regression scenario (ideas #4, #10) | With stage 3 | $3–5 per exploiter | ~50k |
| 4 | **A learned evaluation** for plan search: fit the win chance from position features (cities, economy, army, VP, wars, era) on hundreds of thousands of positions from sim games, as an integer model compiled into the AI (the `winchance_fit` and `learned_macro` paths already exist). Search's leaf score stops being a hand formula fitted on 10-04 rules, and is refitted automatically after each rule change | 1–2 weeks | $5–10 per refit (games) plus minutes of CPU or the Mac's GPU | ~100k per refit review |
| 4b | **Imitation from his saves as priors**, not a trained policy: his macro choices (when he armed, settled, declared, made peace) become tie-breaker priors and agreement metrics, and every save becomes milestone maps and war scenarios (the start-state method) | Ongoing, automatic per playtest | Cents per save | ~20k per save (the coach already reads it) |
| 5 | **Deep RL** for one narrow sub-decision (e.g. tactical siege moves in combat-lab arenas) if stages 1–4 plateau | Not before stage 4 | $50+ and GPU time | High; not planned |

Honest expectations:
- **In days**, stage 1 will tune the economy side (expansion, Gold) more than the war side, because the economy genes are live and the war behaviour is gated by the structural gaps. The first run already showed this: +0.04 to +0.11 of milestone score that held up on new games and a second Sparta map, all of it from Settlers, and a full-game loss when the same genes played every faction. Expect a few points of full-game win rate per faction from stage 1, not human parity.
- **In weeks**, with the stage 2 knobs in place, the same loop can chase his conquest milestones (Sparta's T40 sweep) for real. That is where the user's "Hard I usually can't beat" is decided: by the AI executing a rush or a siege it already chooses.
- **What doesn't scale:** an agent per experiment. The loop above spends tokens per cycle and per structural change only, so running ten times more games costs ten times more CPU and the same tokens.

## Appendix: research sources

- CMA-ES: Hansen's tutorial https://arxiv.org/abs/1604.00772; pycma (integer variables, `NoiseHandler`, Hansen et al. 2009 uncertainty handling) https://github.com/CMA-ES/pycma; learning-rate-adapted CMA-ES for noisy problems https://arxiv.org/abs/2304.03473; CMA-ES with margin for integers https://arxiv.org/abs/2205.13482.
- Optuna `CmaEsSampler`/TPE https://github.com/optuna/optuna (5.0.0); Nevergrad NGOpt https://github.com/facebookresearch/nevergrad.
- NTBEA for noisy game-AI tuning https://arxiv.org/abs/1802.05991, compared with SMAC on Planet Wars https://arxiv.org/abs/1901.00723.
- Quality-diversity: MAP-Elites for Hearthstone decks and balance https://arxiv.org/abs/1904.10656; surrogate-assisted MAP-Elites https://arxiv.org/abs/2112.03534; CMA-ME https://arxiv.org/abs/1912.02400; CMA-MAE https://arxiv.org/abs/2205.10752; pyribs https://github.com/icaros-usc/pyribs (PyPI `ribs` 0.12; the `pyribs` name on PyPI is a squatter).
- Population-based training https://arxiv.org/abs/1711.09846; AlphaStar league and exploiters https://www.nature.com/articles/s41586-019-1724-z; open-ended learning in non-transitive games https://arxiv.org/abs/1901.08106.
- Imitation: DAgger https://arxiv.org/abs/1011.0686; StarCraft macro datasets https://arxiv.org/abs/1710.03131; microRTS behaviour cloning then RL https://arxiv.org/abs/2402.08112.
- Search: combinatorial bandits for RTS https://arxiv.org/abs/1710.04805; Monte-Carlo planning in Civilization II https://arxiv.org/abs/1401.5390; information-set MCTS https://arxiv.org/abs/2005.07156.
- RL cost: CivRealm (Freeciv) https://arxiv.org/abs/2401.10568; Gym-microRTS (~60 hours, one GPU, one map) https://arxiv.org/abs/2105.13807; OpenAI Five https://arxiv.org/abs/1912.06680.
- Shipped AIs: Freeciv AI notes https://raw.githubusercontent.com/freeciv/freeciv/main/doc/README.AI; Civ V flavours (Community Patch source) https://github.com/LoneGazebo/Community-Patch-DLL. Old World, Humankind and Galactic Civilizations: no primary source found.
- Shaping and start states: potential-based shaping (Ng, Harada & Russell 1999); Montezuma's Revenge from one demonstration https://arxiv.org/abs/1812.03381; Go-Explore https://arxiv.org/abs/1901.10995.
- Overfitting a noisy validation score https://arxiv.org/abs/2506.19540.

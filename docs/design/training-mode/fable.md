# Training mode: tactical search, distilled into Hard

Decision memo, 2026-11-07. Written against master `cf4a9167e` (RULES_VERSION 19). Every number names its file; paths are from the repo root.

The designer's idea: stop ignoring "Hard + Search", but shrink it. In a "Training" mode the normal Hard AIs play, and whenever a city is attacked the exact combat is replayed in the combat simulator a thousand times, the winning move is played, and the gap between it and Hard's move is logged so the AI can be trained from the logs at the end. Slower turns are fine.

## 1. Push back first

**The strongest case against it is that we already built it, and it didn't win.** `crates/ai/src/search/tactics.rs` (search option `x`, 2026-10-06) does what the idea says at a city attack: the engaged units get four candidate order sets (Hard's own, focus fire, spread, wait), each is resolved this turn on determinized states, played 3 more turns, and scored. It chose differently from Hard in 36% of 4,666 decisions and gained **+0.4 pp [−1.6, +2.3]** over 512 paired 1v1 games. A capture-weighted score (+0.6), a 6-turn horizon (+0.4) and more samples (+1.0 on top of d=4) did not change that (`docs/learnings.md`, 2026-10-06 "Tactical levers"). "Deviates from Hard" is not "beats Hard".

The other objections, in order of weight:

| Objection | What the evidence says |
|---|---|
| **A local winning move is not the right move in the game.** The arena doesn't know whether this city is the campaign target, whether a relief army is two turns out, whether these units are the home front's only defence, or what the war is worth. | Real, and measurable. The lab's "storm" wins 39 of 48 situation classes but loses 2 to 3 times the Production of Hard's own policy (`docs/sim-results/combat-lab/results.md`). The 10-06 search did the opposite: its 3-turn material score kept units alive and captured fewer cities (396 against 417). The scoring metric is the whole problem, and no purely local metric gets it right. The fix is to price the local outcome with Hard's own valuations (the target's campaign value, unit cost, the door-defence need) and to check a sample of local verdicts against a full-game rollout. |
| **Hidden information.** The combat lab builds its arena from the true state. A Training mode that does the same teaches the AI to act on garrisons it can't see. | Must-have, and already solved for the whole game: `search/determinize.rs` builds a full `GameState` from one seat's `PlayerView` in 0.8 ms (unseen garrison shares placed from public army counts; a fogged city gets a sampled garrison, not the real one). The lab's own fog rows show the cost of ignoring this: the forecast assumes one fortified Soldier in any unseen city, so an empty city and one with three archers get the same answer. Rule: the oracle chooses from determinized samples only; the true state is used afterwards to score regret, never to choose. The omniscient flag stays a diagnostic. |
| **The opponent inside the rollouts.** Rivals in rollouts are scripted Hard (`rollout_first` → `rival_brain`), and the lab's defender "mostly holds" while the assault check assumes every defender strikes (learnings 2026-10-17 (4)). | A search against a passive defender learns to overextend against a human who sorties and shoots the wounded. Search could not defend against Proxy v6 for the same reason (−0.5 pp, learnings 2026-10-06). Needs a second defender model (sortie, focus the wounded, buy Walls or a unit with the city's Gold, the Swiss same-turn Walls) and a score that averages both. |
| **Variance and "1000 runs".** | Dice are cheap; fog is not. The capture rate of a policy over n dice draws has standard error √(p(1−p)/n): 64 draws give ±6 pp, and the policies the lab separates differ by 15 pp or more in the classes that matter. 1,000 draws of one determinization is wasted; 32 dice × 4 determinizations × 2 defender models is the right shape. Cost: a 12-turn arena game is 4.2 ms of CPU (6.65 M fights in 437 s on one c8g.16xlarge, `results.md`), so one engagement with 8 candidates is about 9 s of CPU, a quarter second on 16 cores. In the app the engine runs as single-threaded WASM (learnings 2026-10-06), so an in-app Training turn with three engaged seats is 30 s or more. "Longer turns" must mean a worker pool and a cap, not a frozen UI. |
| **Why Hard + Search was dropped.** | Three reasons. (1) Too slow: 2.3 to 5.0 s p95 late in the game in WASM against a 1 s budget, with plain Hard at 31 ms (`docs/sim-results/ai-war/baseline.md`); irrelevant offline, decisive for a shipped level. (2) Capped by its candidate set: three or four plans, orders byte-identical to Hard's on all 91 turns of the 8.3 game, a flat budget curve from 30 to 960 rollout turns. Applies directly: option `x` had four candidates and gained nothing. The move generator is the design problem. (3) Rivals modelled as Hard. Applies, see above. |

**Verdict: yes, with three changes to the idea.** The tactical layer is where the unexploited headroom is: in the nightly lab Hard captures 29.4% of the fixed cells where plain focus fire and storm capture 44.1% (`docs/sim-results/nightly/history.jsonl`, 11-07); his T34 to T38 army orders before the Byzantine siege were worth +18.6 pp [+10.5, +26.8] over Hard's, and the diff classifies them as execution gaps in `plan_military` / `coordinate_city_assaults` ("Hard stages and does not shoot the walls", `docs/playtests/reviews/2026-11-06-8.9.25-mongols.md`); the decision-diff pattern table ranks army routes and targets as his biggest edge over 23 games. The changes: (a) it is an **offline labelling pipeline first** and a playable mode second; (b) the search must generate **moves Hard doesn't**, or it repeats the 10-06 result; (c) the output is **a distilled valuation**, never the search itself as the shipped AI. Eighty percent of the pipeline exists: the combat lab is a Training-mode oracle over synthetic positions, `determinize` gives fogged positions from live state, and `search/tactics.rs` already logs per-option means and the chosen set.

## 2. What gets searched

**Trigger** (from the seat's view, both sides of a siege first): an own military unit within 2 hexes of a visible at-war enemy city, or a visible at-war enemy military unit within 2 hexes of an own city. Increment 3 widens it to any own unit within 2 of an enemy unit (field battles, retreat and heal, focus-fire targeting).

**The arena from live state.** Determinize the view (4 samples, common random numbers across candidates). Cut a window of radius 7 around the engagement: terrain, cities and units inside it, the two teams' relations, the cities' treasuries as determinized. Units outside the window with a goto that enters it within the horizon are kept (reinforcements in transit). Resolve with `resolve_turn` on the cut state so city orders, Walls purchases, healing and overflow all run as in the game, not `resolve_tactical_turn`, which skips city processing. One in twenty engagements also runs the full determinized state for 20 turns (`search::rollout`, about 260 ms a sample at T40 with 8 seats) to measure how often the local verdict disagrees with the full one. That number is the test of the premise and it is logged, not assumed.

**Candidates.** Per engaged unit, the legal atomic orders this turn: shoot the city, shoot each enemy unit in range, charge the city (melee and mounted only; a shooter is bounced, `RANGED_CAPTURE`), charge an adjacent unit, step to each reachable tile adjacent to the city or outside every enemy's reach, hold. Joint sets are not enumerated (6 units × 8 options is too many). The set is: Hard's own orders; the four lab policies; and single-unit swaps from Hard's orders ranked by the exact kill-chance pricing in `focus.rs` (prefixes of up to six attackers, 41^k rolls), keeping the best four swaps and their pairwise combinations. About 8 to 12 candidates, all generated from the view, none from Hard's commit gates.

**Score** at a 6-turn horizon, in Hard's currency: captured city × its campaign value (`plan.rs` pricing), own city kept × its value, units lost and killed at unit cost, city HP delta, against two defender models (Hard's city planner, and the sortie defender). Capture-by-deadline is logged beside it so the distillation can use either. The engine's rules do the rest (ranged fire before movement, the split counterattack, city HP shielding the garrison and overflow, Walls halving non-Siege hits: `legatum-rules` §4).

## 3. How the result becomes a better AI

| Route | For | Against | Place |
|---|---|---|---|
| (a) Cluster the logged deviations into patterns; a human or Codex turns each into a valuation change behind a gene | Cheap; readable; it is how the decision diff already found "stages and does not shoot the walls"; every fix is a priced term, not a rule | It is the hand-designed-gene loop that failed 10 of 10 in the week of 10-11 and plateaued under CMA-ES; the labels are at scale now, the humans aren't | Companion: the report that says which valuation to fit, and the one that catches a label bug |
| (b) A learned tactical value function fitted to the oracle's labels, replacing the analytic forecast; expert iteration (fit, play, relabel) | The forecast is wrong in both directions today (23% of winnable sieges called hopeless, 34% false alarms, and Hard commits only when it sees a capture within 3 turns); a 20-feature fixed-point model is microseconds live, integer math, weights in one data file, one engine; it is "distil what search found into the live valuation" (`legatum-design`) | Laya was capped by its teacher at 46 to 51% held-out. Here the teacher is the engine's own search, which beats Hard by 15 pp in the lab; the cap is the target | **First.** Its smallest form is a calibrated `forecast_siege`: capture chance and turn from features (force by class and HP, era, faction, garrison, Walls tier, city HP, ground, distance, relief within 6). The second head picks a stance (storm, focus, wait for shooters, hold), which Hard's order passes execute: a valuation knob, not a rule |
| (c) Ship the search live as "Hard + tactical search" | No distillation step; always current | The 10-06 result; single-threaded WASM at 9 s of CPU per engagement; a difficulty level that only exists with a thread pool | Not as a shipped level until (b) wins in the lab and suite. As **Training mode for data collection** from his games, yes, in increment 4 |

(b) first because it attacks the measured defect (a mis-calibrated forecast gating the commit) with the measured oracle, inside the existing value scoring, and the loop closes without a human reading clusters. (a) runs alongside on the same logs. (c) exists today behind option `x` and stays a diagnostic.

## 4. Where the data comes from

| Source | Volume | Distribution | Use |
|---|---|---|---|
| Lab grid (`combat_lab.rs`, 41,580 cells × 32 dice, 7 min, about $0.10) | huge | synthetic, balanced, exact factor coverage | training set for the forecast model; fixed subset is the gate |
| Headless self-play on spot (`legatum-sim --tactical-oracle DIR`, 4,096 games, about $1 to $5 per batch; the nightly battery costs $0.29 for 4,096 games plus lab and scenarios) | about 100k engagements a batch | Hard against Hard: real maps, real reinforcements, but Hard never attacks like a human | training set for the stance head and the full-versus-local check |
| War and scenario suites (35 cases in `scenarios/war/`, cut from his saves) | small | his wars, with written goals | held-out test and a gate |
| His playtests in Training mode | 50 to 100 engagements a game, a few games | positions a human creates; the ones that matter most | **held-out test only.** Too few to train on (the Laya lesson), too valuable to contaminate |

Mix: lab grid plus self-play for fitting; suites and his games for testing. Each `RULES_VERSION` bump invalidates every label (six bumps this week), so combat, siege and Walls rules freeze while a round is in flight.

## 5. Skateboard first

Each increment is usable on its own and gated before the next starts.

| # | Increment | What it delivers | Go / no-go |
|---|---|---|---|
| 1 | **Calibrated forecast from the lab** (gene `siege_oracle`): rerun the lab sweep on v19, fit capture chance and turn to the cell features, replace the analytic capture call in `forecast_siege` and the assault commit | Hard commits to the sieges the oracle wins and skips the false alarms. No new search code | Fixed-subset AI capture 29% → 38% or better on held-out cells (stride); war suite capture-by-deadline +5 pp, losses per capture no worse; seat-only non-inferior on the human-seat gate, rivals-only ≤ 0 |
| 2 | **The oracle on live positions**: arena from the determinized view, deviation log (position fingerprint, Hard's orders, best set, gain, full-versus-local agreement) in self-play, the cluster report | The first real Training-mode data, and the answer to "is the local move right in the game" | Local and full-game verdicts agree in 80% or more of sampled engagements; the report names at least three clusters worth more than +10 pp each |
| 3 | **Wider moves and the stance head**: the swap generator, the sortie defender, the stance model fitted to self-play labels; one expert-iteration round | Hard storms, waits for shooters or pulls back on its own valuation | Lab 38% → 42%; war suite exchange 1.20 → 1.35, goals met 14 → 17 of 34; gate as above |
| 4 | **Training mode in the app**: the oracle in a Web Worker pool with a 15 s per-turn cap, playing the oracle's move, logging his games to `~/LegatumRuns/training/` | He plays against it; his positions join the training data (with a few games held back for testing) | Two of his games logged; the coach's army-window diffs on those games shrink against the 8.9.25 baseline |
| 5 | Field battles, retreat and heal, focus targeting; a second iteration round | Same loop, wider trigger | Same gates |

## 6. Measurement

Existing gates, in the order they should move: `tools/sim/combat-lab.sh check` and a fresh sweep (the lab is STALE at v7, so increment 1 starts by rerunning it); the war suite (`tools/sim/war-scenarios.sh`, exchange ratio, losses per capture, goals met); the human-seat gate (`tools/sim/hard-vs-human.sh --cloud`, seat-only and rivals-only arms); the AI-only census (best/worst ratio must not worsen, since a tactical gain that only Sparta and the Mongols can use widens it); then his games through the coach's window diffs. Expectation set now: every step-2 tactical change so far was neutral on the human-seat gate (focus fire +0.20 [−1.67, +2.06]) while moving the exchange ratio; the gate is dominated by expansion and war timing. The lab and suite are where this program proves itself; the gate is the regression guard.

Anti-overfitting: held-out lab cells by stride; held-out census seed 8241001; a third of the 35 war scenarios held out by save; his 8.9.25 Mongol and India windows never enter training. A fitted model that beats the oracle in-sample but not on the held-out cells is discarded.

## 7. Cost and time

Wall clock with agents, from Sat 11-07. Sonnet 5.5 or Codex Sol for the implementation; Opus for the move generator and the value features.

| Increment | Build | Compute | Lands | Bottleneck |
|---|---|---|---|---|
| 1 | 4 to 6 h, one agent | lab sweep 7 min ($0.10); war suite 30 min; two gate batches about $5 | Mon 11-09 to Tue 11-10 | one or two review rounds, each about half a day |
| 2 | about a day, two agents (arena builder; log and report) | one self-play batch, about $3 | Wed 11-11 to Thu 11-12 | the full-versus-local sample run |
| 3 | about two days | two self-play batches, two gates, about $15 | Mon 11-16 to Wed 11-18 | fit-play-relabel rounds, his read of the cluster report |
| 4 | one to two days | none beyond his games | Thu 11-19 to Fri 11-20, then two playtests | his playtests |

Total compute under $40, a small part of the monthly cap. If increment 1 fails its lab number twice, stop (the plan's stop rule) and write up why before any of the rest.

## 8. Risks and what not to do

- Do not ship the search as a difficulty level before it beats Hard in the lab and the suite. That is the 10-06 result waiting to repeat.
- Do not label from the true state, even "just for training". Omniscient labels teach confidence the live AI can't have.
- Do not spend samples on dice. Spend them on determinizations and defender models.
- Do not let a local metric be the whole score. The lab's storm bleeds 2 to 3 times the Production; the 10-06 material score hoarded units. Price in Hard's currency and keep the full-game agreement check.
- Do not write rules from clusters ("melee first", "always storm at turn 3"). Fit valuations behind genes and gate them on held-out positions.
- ~~Do not train on his games.~~ **Corrected by the designer (2026-11-07): his games are always used to train the AI.** The guard is against overfitting: don't assume his way of playing a faction is the only good way. Hold back a few recent games as a test, and weight his positions alongside self-play, not above it.
- Do not expect the human-seat gate to move more than a point or two from tactics. Read the lab and the suite first; a flat gate with a moving suite is expected, a flat lab after two rounds is a stop.
- Do not build a second simulator. The arena is the engine's `blank_game` and `resolve_turn`, as the lab does it.
- Freeze combat, siege and Walls rules while a round is in flight; a bump invalidates the labels.

## Addendum: the designer's corrections (2026-11-07)

1. **Scope is the battle, 5–20 turns.** Evaluate each searched move over the battle's horizon: until the city falls or the attack is broken, capped at 20 turns. Include every unit of either side within reach of the fight, so reinforcements two tiles away are in the scenario. Opportunity cost elsewhere is out of scope by design.
2. **The teacher may be omniscient.** Training search is not the shipped AI, so it isn't cheating. Label from the true state; it's simpler and cheaper than determinizing. The guard moves to the student: the distilled valuation reads only `PlayerView` features. Report the transfer gap each round (the teacher's capture rate against fogged Hard with the lesson). If the gap is large on fogged positions, add determinized labels for those positions only.
3. **Distillation is the real design work,** as the memo says: increment 1 (the fitted siege estimate) is its first concrete form.
4. **Search broadly; don't hand-pick candidates (designer, 2026-11-07: "there are a finite number of moves... why are we not just testing basically all of them (and only prune the ones that are obviously bad)? battle sims are super cheap").** October's search compared four hand-written whole-army plans, and that was the cap. The full joint move set is too large to enumerate: about 20 legal orders per unit gives 20^8 ≈ 2.6 × 10^10 joint orders for 8 units in a single turn, before the enemy's simultaneous reply and later turns. So enumerate every legal order **per unit**: move tiles in reach, attack targets, fortify, heal, hold back. Prune only the plainly bad ones (suicide moves, illegal stacking). Search unit by unit with the others fixed, repeating until no single change improves the result (coordinate descent). That's about 8 × 20 = 160 candidates per pass, at about 64 rollouts each and 4 ms a rollout: about 40 s on one core, about 3 s on 16. Multi-turn plans such as "shoot the walls from range 2, then ride in" emerge because the rollouts continue the battle with a policy that itself improves each round (expert iteration). Use at least two defender policies in the rollouts: Hard, and a sortie defender.

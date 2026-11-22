# Review 1: the live AI's decision code (`crates/ai/src`)

Master `a9eb46864`, read-only review, 2026-11-09. Paths are relative to `crates/ai/src`. Line numbers were read from the code; anything not verified is marked "inferred".

## 1. How a Hard turn is decided

Entry: `scripted.rs::orders_staged` (798). Per turn, over a fogged `PlayerView`:

1. **Budget and posture.** `supply::affordable`, then `plan_staged` (872): `assess_opponents` (`opponents.rs`, every 5 turns: sightings, distances, garrisons; a threat and target sheet, not a model of what the rival intends).
2. **Strategy / plan** (`planner_through_plan`, 401): `prepare_hard_defense`, `start_second_front`, `reconcile_campaign` (`campaign.rs:759`, ends the single active campaign by rule), then `choose_plan` (`plan.rs:3552`). Plans are `Economy, Peace, Hold, Campaign{against,target}, Contain, Snipe, FieldStrike`, priced by `plan_candidates` (`plan.rs:3077`) in win-chance points from `winchance.rs` (a linear model over team shares). The current plan stands unless a rival beats it by `switch_margin`.
3. **Everything else, in fixed order** (`planner_with`, 358): diplomacy, levy, snipe, field strike, research, yields, hold, siege buys, `plan_cities` (production and Gold purchases), raze, settlers, scouts, `choose_stance`, then `plan_military`. Purchases are decided before the army has moved.
4. **Military** (`military.rs::plan_military`, 63-283): about 60 named passes in a fixed order. Each pass claims the units it wants; a claimed unit is gone for every later pass.
5. **Post-passes that can rewrite orders**: `goal_actions`, `objective_deck`, the air module (`air::air_orders_with_prizes`, separate from the land plan), `supply::within_budget` (drops orders over budget).

Hand-offs: the plan layer passes one target city to production (army size, siege buys) and to the `campaign` military passes. Learnings line 2729 states it plainly: only the plan layer and search consult the win-chance model; "production, military, Settlers and diplomacy are rule-driven."

## 2. Structural weaknesses, ranked by cost against a competent human

### F1. The army is run by a first-claim cascade, not by assignment to objectives

**Evidence.** `military.rs:63-283`. The plan's own passes (`campaign`, three variants) run only under a hard-coded condition list (179-203) and after about 35 earlier passes: `recover_wounded`, `counterattack`, `attack_targets` x3, `avoid_free_damage`, `defend_cities`, `hold_captures`, `reserve_front_guards`, `sortie_garrisons` and more. Each of those can take a campaign unit for a local reason with no comparison against the value of the campaign. `attack_targets` (2732) ranks by local force ratio against a threshold (`scripted.rs:1443`), skips enemy cities (2752), and has a literal `loss_chance < 25` for occupation (2803).

**Impact.** This is the designer's largest measured edge: army routes and targets +6.8 VP, garrisons and escorts +4.7 (state-of-the-AI decision diff). It is also why attacks arrive "one or two at a time" (gene comment for `joint_assault`, `params.rs:486`), why the operation planner lost when it was executed (tactics below it are the same cascade), and why every coach review ends in one more pass. The 60 passes are 60 patches.

**Change.** Replace the cascade with one assignment step: enumerate objectives (take city X, kill stack Y, hold city Z, escort Settler S, heal), price each (unit, objective) pair in one currency, solve greedily with a marginal-value rule (the second unit on a siege is worth more than the first until the kill threshold). Existing passes become the executors of an assigned objective, not competitors for units.

### F2. War is priced as one static city, one campaign at a time, on a 10-turn horizon

**Evidence.**
- `plan.rs:742`: `plan_horizon()` returns the constant 10. Not a gene.
- `plan.rs:668` `capture_odds`: a hand formula clamped to 0..70, calibrated on Hard-vs-Hard cohorts; `price_campaign` (1352) caps again at 70.
- `price_campaign` comment near 1375: the value is the output gained and lost "before either empire compounds from its new city count". No term for the second and third city of the same war, except an elimination path in 1v1 (`base.teams.len()==2`).
- `campaign.rs`: `CampaignMemory.active: Option<Campaign>`. One campaign. `reconcile_campaign` returns early while `held_capture` is set (912-917), so the army that just took a city may not start on the next one.
- `conquest.rs:57` nominates the target with a separate hand heuristic (`path.len*12`, `strength*3`, `+80` top threat), not the win-chance currency. Two valuations of the same target.

**Impact.** The designer's wins are chains: Aztecs conquest at T116, two captures in one turn at Germany T144. The AI cannot represent a chain, so a war is never worth an army big enough to win one. "Attacks rare and weak" and "leaders unchecked" in the open-problems list follow directly. The design skill says conquest is the fastest economy and compounding must be counted; the code does not count it.

**Change.** Price a war, not a city: value = sum over an ordered list of reachable enemy cities of P(take k | took k-1) x value, with the enemy's remaining defence shrinking as cities fall and our production growing. Horizon 30 to 40 turns for war plans. One currency for nomination and choice (delete the `conquest.rs` score).

### F3. Vetoes before valuation: targets the AI may not consider

Worst hard gates no gene can fully override (file:line):

| Gate | Where |
| --- | --- |
| "A march opens no second front": no new campaign while at war with any other living team; narrow exceptions tied to Sparta/India genes and a literal 6 hexes | `plan.rs:1195-1222` |
| Walled city needs a `shooter_only` unit in the force, else no forecast | `plan.rs:1232-1238` |
| Force must include a `charges` unit | `plan.rs` `campaign_forecast_with` (1127 ff.) |
| Candidates come from nominators only; all enemy cities are priced only under Hard+Search (`FRONT_TARGETS = 3`) | `plan.rs:1850` |
| `missing_plan_patience` requires `!c.walls`, distance 14, force >= 2 | `plan.rs:3477-3500` |
| New campaign blocked while holding a capture | `campaign.rs:912-917` |
| Campaign ends by rule: `WallsBeforeArrival`, `InsufficientArmy`, `Deadline`, with literals (start+6, >=3 units within 3, >=4 within 6, 16 turns) | `campaign.rs:759 ff.` |
| Walled conquest gates: hp >= 70, distance <= 5, >= 2 local units | `conquest.rs:17` |
| FieldStrike: enemy within three hexes, at least two units; Snipe: Settler unguarded | `plan.rs` trace gates in `choose_plan` |
| Upgrades only when a unit is idle and holding, and Gold stays above reserve | `military.rs:1539` |
| Garrison floor bought before any value comparison (`GARRISON_FLOOR_VALUE`; learnings 4220), army size is a formula | `production.rs:1045`, `uncapped_army` 1286 |

The Nicomedia 0% scenario was one of these; the others are the same shape and will each cost a scenario in turn. Stage 1 of the climb program turned many gates into genes (`params.rs:155-241`), but a gene that defaults to the old behaviour and is tuned on Hard-vs-Hard stays where it was.

**Change.** Every gate becomes a cost in the plan currency (a wall is extra turns and losses, a second war is a retaliation term that already exists at `retaliation_permille`). Rule: a gate may only remove an option that is illegal, never one that is unwise.

### F4. The valued plan is overridden after the fact

**Evidence.** `choose_plan` sorts by value, then Sparta-specific code replaces the winner (`plan.rs:3649-3707`: `sparta_force_campaign`, follow-up and chain targets sorted by `(!is_capital, distance, id)`), then `operation_candidate` may override again. `plan_candidates` (3090-3170) inserts a "forced expedition" campaign with `value: 0` and a literal distance 18. After planning, `goal_actions`, `objective_deck` and `within_budget` drop or replace orders. This breaks the design rule "no rule may silently override a plan".

**Change.** One decision point. Faction behaviour enters as a value term or a candidate, never as a post-sort replacement. `within_budget` should be unnecessary if purchases are priced against the plan; log every order it drops and treat a non-zero count as a bug.

### F5. The value function is stale, fitted on weak self-play, and blind to what the human exploits

**Evidence.** `winchance.rs`: `WEIGHTS` fitted 2026-10-04 on 300 Hard-vs-Hard games; `HOLD_HAZARD = 100` "an assumption". Learnings 3839 (10-31): on current master it is worse than a uniform guess (Brier skill -33%, -80% at T1-60). Learnings 3871: a better-calibrated net did not improve play because "plan options are still limited by what the execution layer can carry out". Learnings 4220: no term for tempo, time bought or defensive depth; it rewards army share. Fixed charges in chance units (`peace_credibility` 300, `campaign_commit`) sit on top of an uncalibrated scale.

**Impact.** Second-order today (F1-F3 bind first), first-order the moment the option set widens: a wider candidate set scored by a model that is wrong early will pick wrong wars. All training data is Hard vs Hard; the only player who exploits the AI is not in it.

**Change.** Refit on games that include his tapes and the human-seat benchmark, add opponent-relative and tempo features, and validate on decisions (the regret harness), not Brier.

### F6. No opponent model; reactions are thresholds

**Evidence.** `opponents.rs` records sightings and priors (`rush_capable` by unit stats, line 19); nothing predicts where a rival's army goes next or when its timing push lands. `runaway.rs` reacts once a public VP share crosses a threshold (`conquest_runaway_percent` 45, `runaway_coalition` bit flags). Defence (`threat.rs`, `hard_defense.rs`, `door_defense`) weighs visible armies within three turns; unseen armies do not exist. Diplomacy runs first in the turn with its own score (`diplomacy.rs:2196 wants_peace`), only partly bound to the plan (`peace_follows_plan`).

**Impact.** A human wins by doing the thing the AI will not see until it is three turns out. The Aztecs T116 "nobody stopped me" result is a runaway that crossed no threshold in time.

**Change.** A cheap forward projection per rival (army strength trend, nearest target, turns to arrive, VP slope) feeding the same plan currency: "team X wins by turn T unless stopped" becomes a negative term on every plan that ignores X.

### F7. Gold and upgrades

`gold_reserve` (`scripted.rs:1458-1488`) is a runway rule plus faction special cases; `buy_hold` 3 was added because Hard made 80 to 396 purchases a game against his 0 to 7; `decisive_buy` was added after one Bomber purchase. Upgrades fire from `hold_or_upgrade` only for idle units, plus `return_for_modernization` as the last military pass (`military.rs:282`). Gold is not priced in the plan currency, so "save for the strike" and "spend the hoard now" (his +3.9 VP edge) are both rules. Fix inside F2: a war plan includes its bill (upgrades, siege, reinforcements) and reserves it.

### Missing human behaviours, status

| Behaviour | Status |
| --- | --- |
| Focus fire | Present (`focus.rs:103`), field units only, runs mid-cascade |
| Escorts | Present as passes (`settler_escort`, `guard_exposed_settlers`), compete with everything else |
| Timing push before walls | Absent as a concept; `WallsBeforeArrival` cancels instead of accelerating |
| React to runaway | Threshold flags; coalition fix already queued |
| Defend by threat | Visible armies within three turns only |
| Upgrade obsolete units | Idle-only, reserve-gated |
| Use Gold | Rule stack, not valued |
| Multi-city war | Absent |
| Air with land | Separate post-pass |

## 3. Can patching reach the north star?

No. Evidence from the project's own results: CMA-ES over genes failed 12 of 12; many genes test null; a better value net did not change play; search is capped by the candidate generators; the operation planner lost because execution below it is the cascade. Designer-seat held-out is 57.4% but the 8-map human-seat number is 23.0% against a 50% target, and he is 8-0 since 10-27. Each scenario fix adds a gene and a pass (375 genes, 144 flags, 121 defaulting to off; 753 `Faction::` mentions in live files), and the next scenario finds the next gate. The parameter space is not where the gain is; the structure fixes what options exist.

What has to change, in order of leverage:

1. **Unit-to-objective assignment by value** (F1). This is the layer every higher idea has died on.
2. **War valued as a sequence over a longer horizon, all cities as candidates** (F2, F3).
3. **Opponent projection** (F6), then a value function refit on games that include a human (F5).

Full unit-level tree search is not required first. The assignment step gives most of the coordination; a shallow search over assignments (not over unit moves) can come after.

**Smallest test that would settle it (the skateboard).** Behind one gene, in war turns only, replace passes from `press_strike` through `campaign` with a greedy value assignment over three objective types: take the plan's city, kill a visible enemy stack, hold an own city under threat. Reuse existing forecasts (`campaign_can_take`, `focus::best_focus_plan`, `threat::score`) for prices. Measure on the two 0%-win scenario cards, the war suite (20% goals met today) and the Training lab capture rate (42.9% today). Pass bar: war-suite goals met at least doubles with no loss on designer-seat held-out. Wall-clock: about 1 day to build with one Opus 5.5 design pass and Sonnet 5.5 or gpt-6-sol implementation, plus one overnight local sim run; a verdict by 2026-11-11. Bottleneck is the sim run and his review, not the code. If it fails the bar, patching is no worse off; if it passes, the cascade is retired pass by pass over roughly the following week, then F2 (about 2 days plus one overnight census).

## 4. Code health

- **Size.** `crates/ai` is 113,517 lines (`src` 101,692; the brief's 190k is wrong for this crate). Six files exceed 3,000 lines: `production.rs` 6,656, `plan.rs` 5,061, `military.rs` 4,302, `training.rs` 3,760, `diplomacy.rs` 3,698, `civilians.rs` 3,328. `price_campaign` alone is about 290 lines; `choose_item_inner` about 375; `item_base_value` about 750.
- **Seven generations of proxy brains** (`proxy.rs`, `proxy_v4..v7`, `proxy_v8_mongols`, `proxy_v8_sparta`), about 10,700 lines, all still exported (`lib.rs:161-166`). Live users found: `proxy_v4` in `crates/wasm/src/session.rs:61`, `proxy_v6` as the search rival (`search/mod.rs:544`), v7/v8 in one sim example. v5 has no user outside tests (by grep). With the repo going public and the "one engine, no dead code" rule, keep one and delete the rest.
- **Genes.** 375 in `params.rs`; 121 default to 0, 57 are off flags, 30 are named after a faction. None is fully unread (crude grep), but an off gene is an untested branch on master. Each gene comment is a paragraph of history; the file is a changelog. Delete genes that tested null and move the history to learnings.
- **Faction logic in shared code.** `Faction::` 753 times in live files (`params.rs` 289, `production.rs` 109); `sparta_*` identifiers throughout `plan.rs`, `campaign.rs`, `military.rs`. This is the homogenisation problem in reverse: the shared decision code cannot be reasoned about without knowing seven special cases.
- **Literals beside genes.** Distances and counts (3, 5, 6, 14, 16, 18 hexes or turns; hp 30/70/80; `loss_chance < 25`) sit inline next to gene-driven code; a tuner cannot reach them.
- **Tests** are mostly saved-position unit tests per patch (`tests.rs` 69, `production.rs` 39). They pin current pass behaviour, so they resist the restructuring in section 3. No test asserts a plan-level invariant such as "the units the plan rostered were ordered toward its target". Add that one first; it will fail today and is the cheapest proof of F1 and F4.
- **Order of operations.** Purchases before movement, diplomacy before plan-bound checks, air after everything. These orderings are accidents of history in `planner_with` and are a source of contradictions on their own.

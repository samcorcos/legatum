# Hard's strategy layer: how it chooses and executes today, and what a committed strategy needs (2026-10-31)

Read-only analysis of master `b7e8f69b4`. The designer wants Hard to pick one winning strategy per game (science rush to a power-spike unit, early army rush, or big economy that buys what it needs) and execute it: "pick one and go with it" (2026-10-09), and "if you're planning to attack, you should build a big army and use it" (2026-10-30; `.claude/skills/legatum-design/SKILL.md:32`, `:125`).

**Verdict.** Shipped Hard has no strategy layer. It has a *war-objective* layer (`plan.rs`) that picks one war or no war on a 10-turn horizon, set on top of a fixed per-faction personality: five knobs plus about 250 banked genes. Build, research, army size and Gold policy are not chosen per game. They come from that personality, nudged by a few multipliers, and each pass reads the view on its own. The `Strategy` enum (Rush, Wide, Science, Turtle, Economic) is not used by Hard in the app. The one component with phases, triggers and abort reasons is the opening machine, which runs only in Hard v2, and Hard v2 is off in the app.

## 1. The real decision chain

**Construction (once per game).** The app builds Hard as `ScriptedBrain::with_difficulty` (`crates/wasm/src/session.rs:42-67`), and that sets `strategy: None` (`crates/ai/src/scripted.rs:172`). `with_strategy` (`scripted.rs:277-288`) is reached only from the simulator's `--strategy` flag (`crates/sim/src/runner.rs:356`), search rollout variant 3 (`search/mod.rs:546`) and the Hard v2 opening brain. Hard v2 is opt-in only (`hard_v2.rs:1-3`; `session.rs:53`), "because validation found Hard v2 easier for a strong player". What a seat does get is `Knobs::for_faction` (`knobs.rs:41-58`), scaled by difficulty, plus `BUILTIN_FACTION_DECISIONS` (`params.rs:439-696`: 256 overrides of about 300 shared genes; Switzerland 55, Germany 54, Byzantium 52, Athens 48, Sparta 31, Mongols 10, Celts 5, Japan 1). The climb banked these against each faction's *whole-game* win rate, so each set is one tuned average per faction, not a bundle per strategy.

**Each turn** (`ScriptedBrain::planner_with`, `scripted.rs:318-374`):

| Step | What it sets | Intent or execution |
|---|---|---|
| `assess_opponents` → `Adaptation` (`adaptation.rs:37-72`) | Balanced / Expand / DefendedExpand; clamps knobs, e.g. DefendedExpand forces military ≥ 65 (`:78-93`) | Reactive lean |
| `Posture::apply` (`scripted.rs:319`; `plan.rs:387-409`) | Military / Expand / Economy knob clamps | **Never live**: only Hard+Search with `postures`, which defaults off even in the app's Search (`search/mod.rs:135`) |
| `choose_plan` (`scripted.rs:355`; `plan.rs:2996-3205`) | One `PlanKind`: Economy/Peace, Campaign on a city, Contain the leader, Snipe, FieldStrike (`plan.rs:58-93`) | **The only intent layer** |
| `plan_research` (`:361`; `research.rs:89-185`) | Era queue by value per Science | Mostly independent of the plan |
| `plan_cities` (`:365`; `production.rs:476-560`) | Rescue, door, modernization, threat buys, faction goal, victory, then `plan_buy` and per-city scoring | Mostly independent of the plan |
| `plan_military` (`:371`; `military.rs:68-175`) | About 40 passes in fixed order | Execution |

**How the plan is chosen.** Each candidate is priced in win-chance hundredths (`winchance.rs`) over a fixed 10-turn horizon (`plan.rs:589-591`). A challenger must beat the running plan by `switch_margin` 40, doubled before the review turn (`params.rs:175`; `plan.rs:3060-3075`). Abort reasons are concrete for Campaign, Snipe and FieldStrike (`plan.rs:2832-2932`). Contain ends only when the target dies or Peace is signed, and Economy and Peace never end (`:2835`, `:2869-2880`).

**How the plan reaches action** (the full list):
- `start_planned_campaign` turns a Campaign plan, or a Contain plan with a big enough local force, into a `Campaign` (`plan.rs:3796-3840`).
- Research: +150 value on military techs during a war plan, +60 on growth techs during peace (`research.rs:135-140`; `params.rs:188-189`).
- Army quota: `cities + army_war_extra` during a conquest or Contain war (`production.rs:803-811`).
- A few military passes name the plan's enemy: `siege_war` (`siege.rs:27-42`), the containment march (`military.rs:289-293`), `leave_strike_range` (`military.rs:996-999`) and `war_garrison` (`military.rs:1937-1947`).

Everything else runs whatever the plan says. The threat tier puts military techs first when urgent (`research.rs:89-91`, `:152-159`), rescue, door and threat purchases run before `plan_buy` (`production.rs:478-555`), and `defend_cities` and `garrison_cities` run before the staging passes (`military.rs:150-170`). The faction goal, `plan_victory` and the era objectives (`crowns.rs`) each read the view separately.

So "build X" is the faction personality's item scores (`production.rs:1349-1640`, `item_value` `:1764`) after every defensive pass has had its pick, "research Y" is value per Science with a threat override, and "attack Z" is the plan, *if* a campaign can open. Only the third is decided for this game.

## 2. Is there a strategy with a win condition, a timing, phases and an abort?

**Not in shipped Hard.** The plan has a target and an abort for its war objective. It has no build path, research path, army plan, timing or win condition beyond "this war raises win chance over 10 turns". The "strategies" in the code are multipliers that get blended:
- **`Strategy`:** ×4 weights on Settlers, units, Laboratories, Walls or Markets (`strategy.rs:57-87`) and ×4 on a few techs each (`strategy.rs:105-120`). The only fixed path is Rush's Ancient research sequence (`strategy.rs:93-103`, used at `research.rs:164`).
- **The opening brain averages the base and strategy knobs field by field** (`openings.rs:889-898`): the "average of all strategies" in code form.
- `Adaptation` and `Posture` clamp the same five knobs again.

**Closest prior art:** `OpeningBrain` (`openings.rs`). Its phase machine covers Opening, Expand, Conquer, TurtleTech, Defend and Close, with triggers (`phase_for`, `openings.rs:594`). It also records completion and abort reasons: `RushCaptured`, `RushAbandoned`, `CityTarget`, `SnowballReady` and an 80-turn cap (`openings.rs:470-501`). Its build quotas and war army (`war_army`, `production.rs:812-818`) are tuned by the league (`tuned_openings.rs`). That is the right shape, but it covers only the first 80 turns, picks uniformly at random rather than for the faction or map (`hard_v2.rs:23-35`), and is off in the app.

**Power-spike techs.** Under the strict era gate (`.claude/skills/legatum-rules/references/game-rules.md:122`) an empire researches only its current era's five techs, with no prerequisites inside the era. "Beeline Flight" therefore means "reach the Modern era first and research Flight first within it". Today research ranks value per Science (`research.rs:117-185`); Steel and Flight are on `is_military_tech` (`research.rs:33-52`) and get a bonus only from a live war plan or a threat; `advanced_denial_bonus` chases the Most Advanced crown (`research.rs:209-260`). The plan's `era_terms` adds +30 per era of lead to a war against that target (`plan.rs:700-726`). That reacts to a lead Hard happens to have; nothing plans for one. **No code names a target era or unit, forecasts when it arrives, or times a war to it.** Medieval cavalry, Industrial siege and Modern Flight and Steel all arrive by accident.

## 3. Where intent fails to become action

The same pattern sits under all five failures. The plan doesn't own any units or Gold. Earlier passes spend both on local readings of the view before the plan's execution passes run.

1. **Contain never marches.** A Contain plan opens a campaign only when some enemy city has at least two units in its `campaign_force` (`plan.rs:3818-3836`). Outside Sparta, `campaign_force` counts only units within 4 hexes of the target and not standing in an own city (`campaign.rs:476-489`). `rushing()` is false for shipped Hard (`openings.rs:1121-1125`), so units must leave home *before* a campaign exists. They never do, because no campaign exists to send them.
   - A chicken-and-egg loop: the Swiss held Contain for 80 turns, Byzantium for 73 (`docs/playtests/2026-10-30-mongols-review.md` §4). The forecast path adds its own veto, "a march opens no second front" (`plan.rs:1016-1030`, `:1498`), and Contain has no abort for "no progress" (`plan.rs:2869-2880`).
2. **Every city reads as threatened, so units garrison.**
   - `threat_exposure` returns 1 for *every* city while at war with the top threat (`threat.rs:852-874`).
   - At war with the Mongols, every city gets a garrison first (`military.rs:97-117`).
   - `defend_cities` (`military.rs:1693`) and `garrison_cities` (`:1921`) run before the siege and staging passes.
   - In the 8.9.17 save, 5 of the 6 Swiss units stood in Swiss cities (`docs/learnings.md`, 2026-10-30 W2.4). `hollow_march` 3 freed them: Chur went from 34% to 59%, but the full gate was neutral, so it stays off.
3. **The Gold freeze.** `buy_frontline_defender` returns `danger = true` when an enemy unit is near and nothing is affordable above `keep` (`production.rs:3496-3570`; `keep` at `:3536-3542`). `plan_buy` then returns at once (`production.rs:3264-3266`), so upgrades, Great Works and every other buy are skipped that turn.
   - The Swiss `gold_reserve` is the 5,000 hoard (`scripted.rs:951-968`). Their treasury grew from 768 to 3,769, unspent, at war.
   - Underneath the freeze is income: Hard makes about 13 Gold a turn and can't afford the door's buy on 95% of the turns it wants one. A standing war chest cost attack power instead (`docs/sim-results/ai-war/war-chest.md`).
4. **Dead wars nobody fights.**
   - `war_pressure` charges only wars *we* declared, and never a war on the leader (`plan.rs:795-813`).
   - The plan's "Peace with X" label isn't what accepts an incoming Peace. `treaty_score` runs `wants_peace` (`diplomacy.rs:1399`, `:2017-2110`) and older refusals.
   - The override that lets the plan's Peace win requires `!self.campaigns.plan.searched.is_empty()` (`diplomacy.rs:1411-1416`), so it applies only to Hard+Search seats, never to plain Hard.
   - The Celts' plan said Peace with Athens for 85 turns while they refused 11 Athenian offers.
5. **Idle armies paying upkeep.** Army size follows a quota, not intent.
   - At peace with no threat: `cities + army_hard_extra` (2), a military bonus and conquest pressure (`production.rs:830-848`); any threat restores the full quota, and DefendedExpand forces military ≥ 65 (`adaptation.rs:85-87`).
   - So a non-attacker pays for 5 to 9 units: 225 to 405 Gold of upkeep by T100, against the designer's 76 (Mongols review §3).
   - An attacker's war quota is only `cities + army_war_extra`, banked at 1 for Switzerland (`params.rs:592`). Neither "smallest defensive army" nor "big army, used" exists.

## 4. What a committed strategy layer needs

One `StrategyPlan` per seat, chosen at game start, kept in `CampaignMemory` next to `PlanMemory` (so it is saved), and re-evaluated at fixed reviews and on triggers.

| Piece | What it holds | Exists and can be reused | Missing |
|---|---|---|---|
| **Goal** | VP route and win condition: conquer a named neighbour; crowns plus era objectives; first into an era and win by army; faction goal | `PlanKind` war objectives; `winchance` and the step 3 learned value net (gene `learned_value`, off); `victory.rs`, `crowns.rs` | A per-game route that outlives the 10-turn plan horizon |
| **Timing** | Spike window: e.g. Medieval entry plus N Knights by T70, or first into the Modern era with Bombers by T150; a deadline | `era_terms` (`plan.rs:700`); public era entry and `era_firsts`; the Gold runway (`funding.rs`) | A forecast of when the spike arrives (science rate to era entry, production to unit count) against rivals, and a "declare at spike" trigger |
| **Build path** | Production shares per phase (Settlers, economy, units); Gold policy (spend, hoard, or buy the army at the spike) | `Strategy::production_weight`; opening build quotas (`openings.rs` `BuildStep`); `item_value` gene percents (`production.rs:1775-1787`) | Phase quotas in place of the ×4 multipliers; a Gold policy the buy passes obey |
| **Research path** | Era pace and the priority inside each era (e.g. Flight first in the Modern era) | `research_weight`, `opening_sequence` (`strategy.rs:93-120`); `research_order` (`research.rs:117`) | A target era and tech per strategy; science investment tied to the spike |
| **Army plan** | Peace: the defensive minimum. Attack: a strike force of size N, its composition and a staging point | `desired_army` (`production.rs:776`); `hollow_march` garrison code (`claude/hollow-attack` 5069dcfbf); the army-intent agent's work; W1 operation planner (gene `operation_planner`); focus fire (on) | **Units owned by the strategy:** a muster that leaves garrisons and stages before a campaign exists (this fixes failures 1 and 2) |
| **Triggers** | Phase changes: army ready → declare; capture → next target; spike reached → war | OpeningBrain's `phase_for` and `Phase` (`openings.rs:71`, `:594`); Search `Signals::fired` (`plan.rs:411-479`) | Triggers for the whole game, not just 80 turns |
| **Abort** | Deadline missed; X% of the strike force lost; target walled or ahead in era; a new threat. Then re-select, with the reason logged | `HandoffReason` (`openings.rs:87-98`); `PlanReason` and `PlanEvent` logging (`plan.rs:247-302`); `StrategySwitch` | Numeric abort rules per strategy, and a re-selection step |
| **Selector** | Faction, map and neighbours → strategy | Hard v2's draw (uniform, `hard_v2.rs:23`); Search rollouts per opening (`search/mod.rs:530-555`); the value net | A feature-based choice; one climb per strategy, not per faction |

**Two more changes it needs to work:**
- **The strategy owns its budget.** The rescue, door and threat passes may override it only for a city that falls this turn. Today the plan owns only `reserve` Gold (`plan.rs:229-230`), and `plan_buy` returns early.
- **Diplomacy follows the strategy.** Extend the `plan_peace` override to plain Hard (`diplomacy.rs:1411-1416`), so a strategy that wants Peace accepts it.

Per the one-engine rule, `Strategy`, `OpeningBrain` and `Adaptation` should fold into this one layer, and the leftover parts should be deleted rather than kept alongside.

**Skateboard.** Two strategies behind one gene:
- **Conquer:** army to a Classical or Medieval spike, declare at the spike, capture, chain to the next target.
- **Economy then buy:** the defensive minimum, settle, Markets, bank Gold, then turn Gold into an army around T100-150, as the designer does (`docs/learnings.md` 2026-10-30, why he out-economies Hard).

A simple rule picks between them, and each aborts into the other. Measure on the war suite, an idle-army upkeep census, the hollow Mongols fixtures (0% today) and the 2,048-game human-seat gate.

**Then:** a science spike (first into the Modern era, Flight and air), a hoard strategy (Swiss), a learned selector, and per-strategy climbs on the box.

**Estimates (agent wall-clock).** The bottlenecks are build-test-fix rounds (about half a day each, and history says many first designs fail), box gates (about 1 hour and under $0.50 each), the step 3 data run (value-v1 failed and is being relaunched) and the designer's playtests.

| Increment | Wall-clock | Target date |
|---|---|---|
| Skateboard, one Opus agent: muster, army by intent, budget ownership, Peace override, two strategies, abort; reuses the army-intent, gold-freeze and dead-wars agents' genes | 1.5-2 days | Sun 11-01 to Mon 11-02; playtest after the gate |
| Science-spike and hoard strategies | +1-2 days | |
| Selector on the value net | +1-2 days | Fri 11-06 to Mon 11-09, in line with increment 3 of `plans/ai-war.md` |
| Per-strategy climbs | One to two overnight runs each | |

**Risks:** fixed openings were easier to beat (Hard v2), so triggers and aborts matter as much as the bundles. Holding resources back has failed twice (the 10-27 standing army, today's standing war chest): a strategy must spend at its spike. The gate's ±2 pp noise leaves close calls to the designer's fixtures and playtests.

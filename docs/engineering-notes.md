# Engineering notes

Lessons from the development of a deterministic Rust strategy game, selected from the private engineering log. Each dated note records the observation, the evidence that made it credible, and the resulting code, test, or measurement change. Dates describe when the finding was recorded; measured outcomes belong to that version of the game.

## 2026-10-23: Alliances cost Gold again, and an ally's own war is a call to arms

- **Rule version, not a revert.** `DiplomacyState::calls` is `Some` in new games and tapes of version 6 (`TAPE_VERSION`), `None` in older saves and tapes, which keep free Alliances and `share_wars`. The fixed-order tapes of versions 4 and 5 replayed to the same hashes after the change, which shows the legacy path equals the earlier rules; only the mixed tapes, the alliance tape and the pins moved.
- **Timing chosen:** a declaration makes defenders join in the same turn (a surprise war starts the war at the start of resolution); the call appears in the next planning turn's inbox, stays open two turns, and Join resolves at the boundary like a declaration (at War from the turn after the order). A lapse is a decline.
- **Peace needs only the teams at war.** Under whole-side Peace, an ally that declined the war would have had to agree to a Peace it was never part of (`peace_parties`).

## 2026-10-23: The Celts rework: two rules deleted, the sack grows, the share does not move

- **Result:** balanced league Celts 7.6% -> 7.0% over 4,096 games (paired -0.6 pp +-1.0); the Celtic-seat map of the playtester benchmark 26.6% -> 15.6% (+-9). The seat gets weaker without its land and hidden archers: eliminated 4% -> 9%, cities 4.4 -> 3.9.
- **Where a rule hid in the AI:** the Celts' rules were also hard-coded in other factions' AI (skip Celtic cities as targets, +120 cost on Celtic land, a "viable target" test, `prey_skips_celts` added by the gate-to-gene work the same day). Deleting a rule means grepping every `Faction::Celts` in `crates/ai` and every `celts`/`hunter` in the retired TypeScript parity client.
- **A latent AI bug surfaced:** `mixed_opening_orders_validate_in_real_fogged_games` failed on the changed trajectory with a move that ended in an ally's city (refused by the engine; the opening brains' orders were only filtered for stays). `drop_stays_in_allied_cities` now also drops such moves for every brain.

## 2026-10-23: The AI movement preflight cancelled turns the resolver would have taken in turn; the war check did not move

- **Cause.** `movement.rs::movement_conflict` simulates a turn's movement round by round and, when two of our movers would end a round on one tile, returned the lower-priority one, whose whole order became a Skip. The resolver (`rules::movement`, step 5) already settles that case: claimants of one tile that share an owner take turns by unit id, the lower id advances and the other waits a round with its moves intact, and both still arrive.
- **Fix (gene `movement_wait`, default 1).** The simulated round does what the resolver does: the higher id of two movers that enter one tile in the same round waits (position and moves kept), a mover entering a tile whose occupant waits also waits, and a waiter adds a simulated round (up to double the rounds).
- **Re-measure of `war-golden-horde-thin`: no recovery of the 95%.** On the move-through tree (`8d7bec73c`, 256 runs): thin 69% with the old preflight, 71% with the new (the 95% to 67% drop stays the engine rule, as the move-through entry says); Golden Horde 84% to 65%, the Swiss hold 14% to 33%.

## 2026-10-23: Rare stalls: a game past its expected length stopped planning faction goals, so a 9-VP Mongols never built its Hall

- **What stalled.** Era objectives step 1 (2 of 256 games at the 600-turn cap) and the opening-ramp removal (1 of 400 at 1,200).
- **Who holds the target, and who breaks the hold.** Nobody holds it for hundreds of turns: three or four empires sit at 8 and 9 VP, and when one reaches 10 a rival takes a crown from it within one to four turns (Largest Economy and Strongest Army flip at the margin; 9322, 5201637, 5202437, 5203228, 5200484).
- **The fix.** Gene `goal_open_end` (default 1): a game with no limit has no last turn; lateness (`is_late`) is still measured against the expected length. Gene `hall_decisive` (default 1): when the Hall's 2 VP take the team to its target the build needs neither a garrison nor a calm border, is worth 4,000 per VP (it was 1,500) and replaces the unit the Hall city repeats.

## 2026-10-23: The Celtic gates are gone and the sack is priced, but the Celts' share did not move

- **The premise had expired.** The benchmark measured the AI Celts at 2.4% on the 8.9.7 tag and 5.2% with the city-attack gates lifted. By the time the gates were deleted, the control Celts were already at **8.3%** (7.3% on a fresh seed) after the Celts camp-count fix (`e840acb4b`, "the crown-upkeep stall was a Celts camp-count bug").
- **Result: sacking happens, winning does not change.** Sacks per Celtic seat 0.01 -> 2.0 (tape replay, `crates/sim/examples/celts_wars.rs`), but the balanced 8-empire league reads 8.3% control, 7.7% priced, 6.5% with kept-city pricing, and +0.7 pp on a fresh seed (each ±1.5). The Celtic seat of the human-seat benchmark rises 10.9% -> 20.3% (±8.5); no other map can move because the changes only run for a Celts AI.
- **Where the AI Celts lose:** a seat that never sacked won 2.1% against the control's 6.2% in the same games, seats with three or more sacks won 14 to 18%. Wars that cannot be paid for in sacks cost more than they bring; `sack_commit` (a funded siege route before a Contain war, like the Sparta) did not move the share on its own (7.6%).

## 2026-10-23: The war-suite attack check fell to 0% because of free Alliances, not stale scenarios

- **Bisect of `war-golden-horde-thin` (64 Hard runs per commit):** 89% through 10-22 01:21, 67% after move-through, era objectives step 1, supply AI and the ramp removal, then 0% from the free-Alliances merge (`72fc33073` alone on its old base: 89% → 54%). An earlier version of this entry blamed scenarios cut under old rules; that was wrong for this drop.
- **Mechanism (plan traces, unconfirmed by the bloc census still running):** with Alliances free, an allied bloc forms around the Swiss; the Mongol campaign on Aarau then prices negative and the plan flips to "Contain" a bloc member, so the Mongols never attack. Likely a design consequence of free Alliances plus "allies fight together", not a bug; the balanced-game bloc census decides it.
- **Bloc census (512 eight-player Hard games, old tree 0ac97a0f8 vs free Alliances 72fc33073):** alliances formed per game 6.3 → 19.8; largest bloc at T100 1.3 → 5.3 teams of 8 (up to 7); wars opened per game 36 → 18; captures 23.2 → 20.9; the share of wars with a capture unchanged (32% → 35%). Free Alliances plus "allies fight together" made a five-to-six-empire bloc from about T100: fewer wars, not more stalled ones.

## 2026-10-22: Each era is defined by its resource: one key building per era, not every building; old scenario states break the attack check

- **Rule:** Copper (Ancient, Market), Iron (Classical and Medieval), Niter, Coal, Oil, Lithium; land units from the Classical era on and each era's key building use 1; an upgrade switches resource.
- **Why only one key building per era.** With every new building of an era using its resource, a mature city needs three or four supply per era (Ancient alone: Workshop, Market, Library), and a five-city empire 15 Copper: no deposit count that leaves resources scarce can feed that, and the opening's Production and Science would stall.
- **Census (32 Hard games, Standard 48x32, 8 empires; a 32-game supply census):** 15 deposits of each resource a map, about 1.9 nearest each start. A typical empire (4-5 cities, land army 6-9) supplies a median 2-3 of its era's resource against 4-9 wanted (its users plus older land units that could upgrade into it); 88-92% of empires want more than they supply at every snapshot from T50 to T250.

## 2026-10-22: Passing through your own units is a hop, not a stop; the benchmark did not move

- **Rule:** a unit may pass through a same-class friend of its team, or its own garrisoned city, but never ends a move on one.
- **Gotchas.** The `blocked` predicate must answer for the goal too (the search exempts the goal itself), or a route crosses a friend to attack a goal two tiles away. `path_turns` must count a hop as one move that waits a turn when it does not fit, or the planner compares routes by the wrong turn count.
- **Effect on Hard (2026-10-22, master `99918a75` as control, the same build for both):** the human-seat benchmark, reseeded dice, 8 maps x 256 salts (2,048 games), Hard in every seat: the seat wins 35.8% -> 35.5% (-0.2 points, 95% interval -2.7 to +2.2).

## 2026-10-22: The AI values supply

- **The Forge was not unvalued, it was outbid.** In 64 census games a Forge-ready city offered a Forge scored about 290 and lost to the due-turn ladder (Granary 1,100, Laboratory 1,400, College 800, Market 900) and to Settlers and troops (2,000 and more). A flat `industry_priority` of 300 per Production (a Forge 900, a Factory 1,500) doubles Forges (0.73 to 1.26 at T250) and Factories (0.30 to 0.55); 600 gives 1.55 and 0.64.
- **The priority must yield to a stalled-growth city** (`faction_tests::industry_payback_does_not_delay_stalled_growth_at_either_speed`, the Mill test): the same food test gates the boost.
- **A Forge started is often abandoned: 46% of Forge builds at priority 600, 42% at 300 (34% at baseline).** The city's build is replaced by military production (Military Engineers, Drones, Artillery, Soldiers). Progress is banked, so it is not all lost, but the Forge's timing is at the mercy of war production.

## 2026-10-21: Era objectives count gains from the last era score (R1, R2, `objective_seek`): the old counts cannot be kept, and the calibration is coupled

- **R1 and R2 on the level counts broke pacing:** median 479 turns, 46 of 256 games at the 600-turn cap (control then 303, 0), though era scores per empire barely moved (2.52 against 2.61). Master's winners were made by stacking: 310 of 2,048 empires scored all six eras and the best empire of a game did in 188 of 256 games (with R1: 100 and 87).
- **The calibration is coupled across eras.** Scoring any era restarts the count of every older open era, so easing late eras (pace) lowered era-2 scoring from 51% to 46% with era 2's own pool unchanged. Four passes of `tools/sim/era-objectives.sh` (3 minutes each) landed at median 318 turns, 49% of entrants scoring, 0 same-turn double scores.
- **The AI's objective pursuit barely changes what is met** (drawn and pursued minus drawn and not pursued: about 0 for Expand and Fight, +11 at most for one Build kind, Castle), so about 27% of the pool is met by accident, 34% for Fight. It does cost game length: `objective_seek` 0 is 18 turns slower (336 against 318).

## 2026-10-21: Peace-then-war is 28% of all AI wars, not only Sparta's capture cycle; pricing the redeclaration cut it by 40%

- **The 8.9.5 cycle.** Sparta's plan log: `captured -> peace [desertion -124]`, next turn `peace -> campaign [broken peace -190, peace credibility -280]` at +1,000.
- **The cycle is general.** `tools/sim/flip-metrics.sh` (192 to 256 eight-player Hard games in the simulation, `war_metrics` on the tapes): 47 wars and 56 Peaces per game, 28% of wars declared within 10 turns of a Peace with the same target (Sparta 30%, every faction 13 to 32%), 36% of captures of a city captured in the previous 20 turns.
- **What did and did not move it.** The hold alone (`peace_redeclare` 1): 13.3 -> 13.0 flips per game, nothing (the hold is a fifth of flips and the price lets some Peaces through). Extended to any war the AI fought in the last 8 turns (2): wars per game 47.2 -> 36.3, flips within 10 turns 13.4 -> 8.1 per game, captures per war 0.57 -> 0.68.

## 2026-10-20: Strategic supply replaces the stockpile

- **The AI barely builds the buildings supply gates.** `crates/sim/examples/supply_census.rs` (64 eight-player Standard Hard games): Hard empires have a median 4-5 cities and 0.3 Forges at T150 (0.7 at T250, Factories 0.3, Pikemen about 1). So a deposit's economic value to the AI is small in this increment, and resource wars will matter more once Hard grows wider or the Copper/Silver lines land.
- **Deposits already change hands in AI wars.** 7.3 strategic deposits per game passed with a captured city before the AI valued them, and 8.4 with the valuation. Shortages (an idle unit) hit 53 empires in 36 of 64 games.
- **The map is already scarce enough:** 11 Iron and 10 Coal deposits per 48x32 map, 1.4 Iron and 1.25 Coal nearest each start. Empires of 8-12 cities hold a median 4-6 Iron supply, so 2-3 of 66-80 such empires could put a Forge in every city.

## 2026-10-18: The Golden Horde attack check fell from 100% to 0% because the Swiss defender can now see (territory sight and the attack reveal), not because the attacker broke

- **Bisect in the simulation (64 Hard runs a probe, one job):** `ac975214c` 100%, `097af665d` 100%, `2a76649b` 100%, `5c27218d` 0%, `430210c48` 0%, `897570d2` 6%, `dbf1395e7` ("Make owned territory and border ring always visible") 6%. The commit adds two sight rules for every team, AI included: the team's own tiles plus one adjacent ring are always visible, and an ordinary unit that attacked a team is visible to it through its next planning turn.

## 2026-10-18: Siege lab: a Catapult is a worse archer, walls are a number not a kind of defence, and a Military Engineer closes the gap

- **Per Production a Catapult does 7% more wall damage than Composite Archers (34 against 32 per 100 Production against a pop-3 walled city) and dies in essentially every fight** (1.0 lost per fight in the lab). Per Production, plain melee is the game's best wall-breaker (55 to 60 per 100), the opposite of Civ.

## 2026-10-18: Why the siege broke Sparta and the Celts: it takes their units before their own war passes

- **With the faction exclusions removed, Sparta's seat on the 8.4 Sparta map falls from 66% to 23%** (128 salts, map-only arms), the Celts' on 8.4 Celts from 11% to 6%. Germany no longer collapses on the control version (28% against 29%). (2) **An ablation mask (`siege_parts`) found the cause:** every part that moves Spartan melee or runs before its own passes hurts: the melee screen, the early pass on the campaign city, the field-army raid, and keeping a failed campaign's army at the front.

## 2026-10-18: Japan's 3-tile spacing helps Japan and its rivals alike; read the benchmark by which maps hold Japan

- **Benchmark, Hard, 8 maps x 64 salts, all seats:** 31.2% on the control version against 27.1% with the rule, but the movement is all on the maps that contain Japan. As the human seat (map 8.4 Japan) Japan wins 53% of runs against 36% before (+17 points, a skill gain, not a regression).

## 2026-10-18: Reading the runaway moves armies; requiring a staged force before declaring takes the pressure off the leader

- **The race term is the lever.** With `runaway_read` (crowns held 30+ turns, a live goal, the public Power lead, the race from 30% of the target, no distance discount for the strict leader) the Mongols take Aarau in the Golden Horde scenario in 64 of 64 runs instead of 43%, and as opponents the Swiss hold all five cities in 1% instead of 56%; Sparta presses the Mongols' walled cities in Behind the Walls (they held 7% instead of 100%).
- **"Declare only with a force staged for a named city" backfired.** Extending `contain_commit` to every faction (`leader_commit` 2) cut wars on the leader from about 9.6 to 3.9 per game in the Swiss what-ifs and raised the playtester's win chance (T126 Hard seat 23% to 38%). Even unsuccessful wars imposed a cost: forced spending delayed the reserve goal by 27 turns.
- **Zero-battle wars are a minority in AI-only games.** The new counters in `hard_vs_human` (8 maps x 256 salts): 15.4 wars a game, 16.8% with no battle, 0.59 captures per war, and in a third of the pairs of empires at war with the same target both attacked it within 2 turns of each other. The coalition genes moved these by under a point.

## 2026-10-18: The Military Engineer (siege rework built)

- **The rule:** Walls halve every attack on a city's HP unless a Military Engineer stands on the attacker's tile; the Engineer's siege tier (Mathematics Catapult, Machinery Trebuchet, Metallurgy Bombard) cancels Walls up to its tier, and against higher walls it cancels the lower tiers and half the halving. Catapult and Bombard retired (old saves load them as Engineers), the Mongols' half city damage is gone.
- **Coverage and price came from the lab, not taste:** one Engineer per unit on its tile captured about as often as one per four units beside them (50% against 46%, Classical and Renaissance pooled, against 27% with none), so "the unit it's with" was not too weak; it costs 3 more Engineers per four units, which is why the price is 40, not 60 (970 Production per capture at 40, 2,450 with no Engineer).
- **A partial cancel needs a middle value:** against a Castle city with a tier-1 Engineer, 25% damage captured 20% (worse than no Engineer, 49%), 50% 57%, 75% 64%, 100% 80%. Anything at or below the uncovered 50% makes a lower tier pointless; 100% makes the tiers pointless. 75% it is.

## 2026-10-18: AI fixes validated on one replay failed on a new map

- **Phase 1 and 2 AI defence was checked by replaying a playtester's 8.9.2 Sparta orders on an 8-player map, where captures fell from 7 to 2. On a later 4-player small map the playtester still swept by T40.** A fixed replay tests one game's openings; the playtester adapts.

## 2026-10-18: aircraft reused ground rules they shouldn't

- **In 8.9.5 a Bomber's strike failed with "No line of sight: the Forest... blocks the shot".** The air strike path reused the ground ranged-attack validation, including line of sight. A bought Bomber also never entered the Return queue, because the air UI queued aircraft only when a target was already in reach at war.

## 2026-10-17: AI Sparta's rush failed before the assault: closed borders, a walling first target and a forecast that called a 4-turn storm 9 turns away

- **Closed borders make the march slow.** At Peace an army cannot enter the target's land, so a Sparta that waits to declare until a siege is forecast (`early_campaign_muster`) stands at the border, 3 to 5 hexes out, while the target walls up. A playtester declared with a playtester's four Hoplites one to three hexes from Nicaea and stormed it four turns later.
- **The first target is the first city the scout finds.** From turn 1 AI Sparta knew only Japan's Kyoto on T13; Japan had seen a rush faction next door and walled up by T14. Scoring targets by value (`value_first_target`) cannot pick a city it has not seen.
- **The siege forecast, not the target, blocked the storm.** With a playtester's T18 army in place, plain Hard took Nicaea in 0 of 64 runs; pricing the split counterattack and Sparta's kill heal on unwalled cities (`storm_forecast`) made it 64 of 64 (median 5 turns). In the full game the same genes moved captures by T87 only from 1.2 to about 1.4.

## 2026-10-17: The first AI offensive that beat a playtester: what made it work, and what did not matter

- **What it was:** the 8.9.2 Mongol AI took Switzerland's Aarau on T147 after a 70-turn war (declared T78, a failed first strike, 58 turns without a battle and without peace, then six turns of concentrated fire from three or four shooters at range 2 while the melee waited, and a Phalangite walked into the city after the shooters killed the garrison).
- **The genes everyone suspected did nothing.** `troop_runway`, `capture_economy_percent`, `price_bought_walls`, `staging_damage_percent` (50 and 200) and `war_commit_ratio` (250) give run-for-run identical outcomes in 128 runs of the scenario (an identical result across arms can be a dead parameter file, so a positive control ran: Mongols `aggression` 0 gives 25% and 100 gives 50%, so the seat's parameters do apply). `restless_far_war` is Sparta-only.
- **A scenario objective must not depend on the defender's expansion.** The first version ("the Swiss own at most 4 cities") scored a captured Aarau as a failure whenever the Swiss AI founded a sixth city in the meantime; "own Aarau" (`OwnCity`) is robust. Use `cities_captured` in the report to cross-check an objective.

## 2026-10-17: faction win rates on the control version (8 Hard AIs, 512 games)

- **`legatum-sim --out X` doesn't put the report at X.** It always writes `docs/sim-results/<date>-<label>.{md,json}`, so `remote.sh --artifact X` comes back empty. Fetch those two paths instead.
- **AI-only faction spread is 6.8x** (target 1.5x): - Byzantium 19.7%, Athens 18.6%, Sparta 17.6%; - Germany 11.5%, Switzerland 11.1%, Celts 10.9%; - Mongols 7.6%, Japan 2.9%.
- The 95% range is about ±3 pp. A playtester has said balance waits for a competent AI, so read this as a pointer, not a tuning target.

## 2026-10-17: An AI that sues for peace and attacks again is usually a candidate-list gap

- The 8.9.2 Mongols proposed Peace to the VP leader on T168 and declared on T172, with the same Contain war scored +200 on every other turn. Cause: when a Campaign ends, the plan falls back to the best candidate that is not the ended objective, and Contain on the same enemy was missing that turn because the target list (`contain_candidates`) follows the threat model's top threat, which flickered 72 to 75 between two rivals.

## 2026-10-11: Hard's open-field stalls are a sight problem, not a planner objective

- `tactical-close-to-contact` gave the tactical layer a visible-enemy objective. Of the 896 mutual all-hold episodes in the combat-scenario grid, 672 are open fights at distance four where neither army sees the other at the start, so a fallback that honestly reads only the player view can't act (896 → 832).

## 2026-10-11: The eight-map combined gate hides large opposite per-map effects

- The capital-guard gene was rejected as "no effect" (Hard seat-only 27.73% → 27.73%). Per map, it moved 8.3 Switzerland 1 → 18/64 and 8.4 Sparta 30 → 11/64, which cancel in the equal-weight mean.

## 2026-10-11: A lead penalty should wait for a sustained lead, and only hit empires ahead

- A playtester was paying crown upkeep (-3 Gold, 2 crowns) at turn 7, and the live game's one-city Germany at 0 Gold lost four units to crown upkeep from turn 32. Crowns are won in the opening by luck (one more ruin), and VP v2 charged them from the turn after they were won, whoever held them.

## 2026-10-11: The war scenarios show Hard defends like a playtester but can't attack like him

- On the war-scenario suite cut from a playtester's playtests, Hard meets the playtester's goal 95–98% of the time on the two defences but 0–3% on the attacks (Hard+Search does better only on Sparta's First Blood). Three failures: plain-Hard Sparta never declares with 5 Spearmen at T29 and marches home; Hard proposes Peace the turn after it is attacked, even while winning; Search turns down a war Hard half-wins.

## 2026-10-11: The Settle lens hid legal sites because it reused the planner's caution

- A playtester (playtest 8.8.1, T46) saw only a few tiles tinted on the Settle lens while many visibly legal tiles had nothing. The lens listed exactly the sites the AI planner may send a Settler to: `settlement_scores` ran `analyze_sites` through `SiteBasics::facts`, which drops every site flagged by `border_city_risks`.
- **Fix (core, `settlement.rs`):** the lens lists every `known_legal_site` tile; a risky one carries `border_risk: true` (dashed rim, and the hover says a foreign city may stand unseen nearby). The planner still skips risky sites (`settlement_candidates_with`, `rank_sites`, the top-three markers).
- **The scale was relative, so it hid how good a site is:** the old tiers were "within 20% of the best reachable score" and "at least half", from a score that included travel, "found without moving ×110%", threats and the Settler's route. The same tile read Great on one turn and Poor the next.

## 2026-10-11: Era objectives are the HUD, and the race panel is zero-sum with the dock

- A playtester played 8.8.1 and found the VP objectives only on the era-up card ("it was entirely unclear to me that there were VP goals in the first era"). The first era has none, so turn 1 showed nothing and later eras' three objectives lived on a takeover and a sheet.
- **The right column cannot grow.** The race panel and the This Turn dock share the column height (the minimap takes the rest), so a block added to one is taken from the other's rows. The first draft (labels, a bar under each line, a sub line, a note) was 250 px and left the race panel with no rows at all, even at 1440x900.
- **The dock's own objective row was removed**, not kept beside the block: the same rule twice costs 34 px of dock, and the race panel already shows every objective with its progress.

## 2026-10-10: Era overhaul A1 (eras, strict gate, boosts): how the pacing came out, and what surprised us

- **Pacing is set by Science income, not by the early costs.** Cutting Ancient and Classical costs by 15% moved the winners' Classical entry by only 3 to 4 turns, and the late eras moved even less, because Science grows with cities and Libraries and the cheap early techs are a small share of the total. Each pass moved era entry by about a quarter of the cost change; it took five passes (and the AI's boost and threat ordering) to land.
- **Result (Standard, Hard+Search, 256 games per lobby; `docs/sim-results/era-overhaul/`).** Median winner enters Classical/Medieval/Renaissance/Industrial/Modern/Future at T32/63/97/129/161/191 in 4-player games (median length 178) and T29/63/100/134/169/203 in 8-player games (median 204).
- **The "idle research" number lies unless the era-entry turn is excluded.** An empire has nothing legal to queue on the turn it enters an era (the gate forbids queueing next-era techs), so the first telemetry showed about 1,000 idle turns per 64 games. The real stalls were Easy seats lapsing their research order; the plan is now exempt when empty.

## 2026-10-10: The human-seat benchmark penalised changes that improve every AI

- `hard_vs_human` ran the candidate in all seats. Settler safety (losses −32%) and Sparta war (wins 17.7% → 26.0%) both lowered the playtester-seat score, because they also made the opponents stronger.

## 2026-10-10: An AI war cycle lasts nearly three eras

- The war-cycle study (Hard + Search, Standard) measured p75 86 turns from building an army to a second city capture, and it recommended eras of about 130 turns. The era core's pacing gives eras of 27–37 turns (a median game of 220 turns, in line with Civ 6's 200–250).

## 2026-10-09: League training: one early gain, then a plateau; validate mid-run

- A 14-generation league promoted once, at generation 4 (+4.5 points against its opponents, confidence interval above zero).
- **Don't wait for the end-of-run validation.** Test the current champion on held-out human-seat maps as soon as it has been stable for a few generations, and stop the run if it doesn't transfer.
- **Tuning thresholds can't create a move the AI lacks.** The playtest gaps (Settler-capture rate, buying defence just in time instead of keeping a standing army, Markets first, contested land first) are moves.

## 2026-10-09: Faction win rates in the Hard + Search late-game study: Mongols dominates, Germany is weak

- In 512 Standard games with Hard + Search in every seat, Mongols won 60.4% of 4-player games and 40.2% of 8-player games; Germany won 17.8% and 5.1%.
- A fair share is 25% in 4-player games and 12.5% in 8-player. The best-to-worst ratio is about 4.8x against the pillar-8 target of 1.5x.

## 2026-10-07: A policy trained for turn-60 survival loses full games even when gated to turn 60

- The ES rush-defence policy (A1) lifted turn-60 capital held from 58.6% to 90.2% (512 held-out seeds, Hard vs Proxy v7 Mongols). Ported ungated it collapsed full-game results (duel matrix vs Proxy v7, 64 games: 5 Hard wins against 20 without it).

## 2026-10-07: Faction goals increment 3 core and AI: a held goal needs the unit's identity, and the goal pass claims idle cities

- **A held goal is a unit, so identity lives on the unit** (`Unit::heir_of`, public as `UnitView::heir_of`), not in a Player counter: a civilian is captured in place (the owner changes; id and `heir_of` stay), so whoever owns the unit scores (`goals::holders`). Two Byzantines in one test game each have their own Heir.
- **Tests that used `faction_goal_done = true` as "this empire has 2 VP" break for the Byzantium**: a held goal scores only from the unit (coalition fixtures led by a Byzantium lost their leader; the telemetry score test too). Use a Permanent faction (Japan) when the fixture only needs VP.
- **A new civilian unit kind needs the invariant oracles checked**: the fuzz oracle called every non-Settler "military" and rejected an Heir sharing a tile with a soldier; Mongols's +1 move and Sparta healing tests keyed on `Settler` rather than `military`. All three now read `data::unit_stats(kind).military`.

## 2026-10-07: Stalls after the goal redesign: a 9-VP team with nothing to steal, and rich empires that never buy the title (goals increment 3)

- 100 Standard Hard games on seeds 5300001+ (goals-3 on the control version with hold-to-win): 4 stalled at the 1,200-turn cap (seeds 5300017, 5300035, 5300070, 5300086); master's baseline on the same seeds had none, and goals-3's smoke on the previous master had 1 in 50.
- **No hold ever starts.** In all four, the best team stands at exactly 9 VP (Japan once, Byzantium twice, Sparta once) and nobody reaches 10, so hold-to-win is not the blocker: there is nothing to hold. The 9-VP team has every milestone, its goal and two titles; the tenth point is a title held by a rival.
- **The world is frozen at peace.** By turn 300 every unit has a `HoldUnit` order and nobody declares war; in 5300017 the Army title is 2,715 against 2,714, the Populous title 107 against 101, and neither moves in 900 turns.

## 2026-10-07: Endgame stalls: four AI holes, fixed behind one gene; a walled late game remains

- 1,024 Standard Hard games on seeds 5300001+: stalls at the 1,200-turn cap fell from 11 to 4 with `endgame_push` (on by default; `vp_trace` example prints a game's VP race from its tape). Causes found in the stalled games:
- **Idle armies blocked every purchase:** a city needs a free spawn tile (its own tile or ring 1; `cities::spawn_tile`), and 800 turns of `HoldUnit` filled them all, so a 9-VP empire with 23,000 Gold got `NoSpawnTile` on every unit. Resting units now step off ring 1 when fewer than two spawn tiles are left.
- **The Army title was priced by Production turns** (`victory.rs::production_points`); a treasury that covers the whole gap now buys a unit a turn per city, past the routine purchase cap.

## 2026-10-07: Under the new rules search beats Hard by 4 points but still not the proxies

- **Against plain Hard search now pays:** default k3 d4 +3.7 [+2.2, +5.3], 12 plan-by-overlay composites with 40-turn rollouts +4.5 [+2.6, +6.4] (was +1.7 under the old rules). Hold-to-win makes the plan layer's choices count for more.
- **Against the proxies it is flat:** Proxy v6 -1.3 to -0.1 at every budget (30 to 1,920 rollout turns), Proxy v7 -0.2 to +0.3; the Proxy v6 rival in half the rollouts (`a=2`) doesn't change that. Rollouts that model the opponent as scripted Hard (or v6) don't foresee the proxy's assaults.

## 2026-10-07: Goal calibration: two AI bugs mattered more than the knobs

- 8-player Standard Hard, 1,024 games per setting (seeds 5300001+, final checked on 5400001+). Goal reached (share of seats, median turn), before -> after: Japan 23% (250) -> 33% (225), Byzantium 67% (115) -> 57% (144), Sparta 0.5% -> 28% (223) at level 2, since reverted to 4; Switzerland 53%, Celts 55% (210), Germany 37% (175), Mongols 39% (214), Athens 45% (255) unchanged.
- **Sparta: one unit per tile.** The AI retired only when all three owed veterans stood in War Shrine cities in one turn, but a city tile holds one military unit, so with one shrine that never happened. Retiring them as they arrive (progress is kept) was needed; even then level 3 reached only 4% of seats, so the level went to 2 (28% of seats). **Reverted the same day (designer): the level stays 4**, "level 2 is just way too easy to achieve, and units are cheap"; getting units to level 4 is AI work.
- **Deadlines past the expected length.** `goal_actions::last_turn` was the expected length, so once a game ran past it every "can it finish and hold in time" test failed for good; a 9-VP Mongols never started its Hall in 900 turns. It now keeps a tenth of the expected length ahead.

## 2026-10-07: Learning-AI pilot: ES discovers rush defence, but not sentries

- Evolution strategies over a 150-parameter fixed-point linear policy (`es_defence.rs`; report `docs/sim-results/rl-pilot/README.md`), overriding only unit moves (keep Hard's order, fortify, home, watch their city, midway, picket) and city builds (keep, best melee, best ranged, Walls) on top of plain Hard, against Proxy v7 Mongols in the 1v1 Small Standard lobby, 60 turns, no hand-written strategy:
- **It works for defence.** Three independent runs, held-out seeds 9100001+ (2,048 paired games each): capital held at turn 60 goes 54.4% → 83.4-86.9% (+29 to +33 points, CIs ±2.3), more cities (1.34 → 1.41-1.61), twice the enemy units killed. At turn 100 (past the training horizon): 33% → 54-62%.
- **It didn't find sentries.** Sentry turns and turns seeing their city barely moved; the warning lead grew 1-1.7 turns because attacks came later, not from sight. With the city head frozen (unit moves only), ES gained +5 to +15 points by pulling units back (picket 3 out, home), and sentry turns fell.

## 2026-10-07: A captured city left its attackers' move orders alive

- Five Sparta Soldiers ordered `MoveUnit` into a fallen city; one entered and captured it, and the other four, which attacked but did not move in, kept `goto` = the city tile. They showed a "move into this city" arrow and were not counted among the units needing orders (the client counts a unit as ready only when it has no `goto`, `needs_orders` is false otherwise).
- **Fix:** `capture_city` now calls `end_orders_into_city`: every unit of the captor's team or an ally with a goto or route ending at the city (not standing on it) is cleared and gets `needs_orders`; Celts razing is unchanged. A dead or own-side-captured attack target uses `void_pursuit` (clear plus `needs_orders`).
- **Gotcha:** the mock's `coreFixture` did not pass `attack_target` to the core, so WASM parity tests of attack orders must set it (now carried) or use orders.

## 2026-10-07: Earlier Walls gave no measurable defence gain

- Walls under way are now finished or bought, never swapped for an emergency unit (`keep_walls`; the 8.4 Byzantium capital swapped Walls at 86 of 91 Production for a Ranger and fell), and a rival within 12 hexes with twice our public army is a border threat at any time (`army_ratio_walls`). Against Proxy v7, 2,048 paired duels: +0.1 [-1.1, +1.3]; 50.1% against the old Hard.

## 2026-10-07: The learned rush defence wins its narrow test and loses full games

- Ported exactly as gene `learned_defence`. It reproduces the pilot: capital held at turn 60 in 90.2% of games against Hard's 58.6%.
- **Rule:** a policy learned on a narrow objective must be gated to the states it was trained on (here: early turns, under threat), or trained on the full-game outcome. Always measure on full games before it becomes a default.

## 2026-10-07: Measure whether rivals actually contest the VP leader

- Measured with the new `contest_leader` example. A trigger is a team at 7+ VP at T80 or starting a hold; it is answered when within 15 turns a rival at war with it has 2+ units within 4 hexes of one of its cities (one away from home) or an AI starts a campaign on one of its cities.
- **Contain had no body, so rollouts couldn't tell it from Peace.** Its only effect was a longer march radius in `seize_open_city`, which needs an unguarded city; against a garrisoned leader Contain issued the same orders as Peace, and when the leader (modelled as Hard) refused peace both rollouts were the same game.
- **A 7-VP leader at T80 was almost never fought:** 37 of 275 answered (13%) with plain Hard; rivals at war with it averaged 0.4. Its Contain price was negative (the "VP denied" assumption is worth 0-1); a *leader's race* term (we gain our share of the win chance the war takes from the leader) plus the strict leader at 70% as a candidate fixed that: 174 of 274 (64%).

## 2026-10-07: Settlers were boxed in, not under-built

- Branch. Tools: `settler_loss_recovery` (scenario), `economy_t80`, `settler_life` (per-Settler positions from a tape), all in `crates/sim/examples/`.
- **The finding.** In the baseline, 43% of Settler-turns before T60 were spent idle in a city and a quarter more idle on the map; only 31% walked. Cause: `hard_defense::prepare_hard_defense` marks every tile a *peaceful* rival's military unit could reach as Settler danger (the surprise-war snipe defence), and a Scout is a military unit.
- **Four cities at T50 is not reachable by AI choices.** A forced Settler-first AI (every gate but pop, site and time bypassed) reaches 2.89 at T50 and 4.07 at T75; the capital makes a Settler about every ten turns (cost 76 to 150 and one citizen each), and the playtester's 5 at T50 came from Farm and Granary first and no Ranger. Settler priority, pipeline 3, open-cities 8, travel penalty, site distance, commit, patience and production weights all moved T50 by under 0.05.

## 2026-10-07: The AI was too weak to balance with

- On a playtester's 8.5 map (seed 874654782136188), 512 AI salts each, original seats and mirror, the Mongols and Sparta human proxies won 0 and 0 in the original seats and 42 and 8 in the mirror; Hard + Search seats won the rest with VP holds around T190. The runs differed only in the AI RNG salt, so they tested one map rather than map-wide strength.
- **Fidelity gaps:** the Mongols proxy missed the T28 Scout raid that stole a Settler and the 5-cities-by-T50 pace. The Sparta proxy reached 4 cities at T62 against the playtester's 7.
- **Rule:** validate a proxy on the decisive milestones before trusting a head-to-head.

## 2026-10-06: "Winner at or above the target" no longer means a VP win; the sim classifies by a completed hold

- With the hold-to-win rule (`GameSpeed::vp_hold_turns`: 10 Standard, 5 Quick), a team can stand at its VP target at the turn limit and win on highest VP without having completed its hold. The old classifiers (`stats::classify`, `wasm/src/history.rs` outcome "vp") used `team_vp >= team_target` and would have called that a VP win.

## 2026-10-06: Search AI step 1: rollouts beat the analytic plan layer in 1v1

- **Determinization from one seat's view is close enough to trust.** On a real Standard game the determinized state reproduces the seat's own units, cities, treasury and research exactly (own city yields within 1-2 per city, from guessed tile assignments), every known city, and each rival's public city count, army and population titles within a few percent.
- **Rollouts must seed every unknown, including a rival with no city yet.** A first version gave such rivals no units at all, so turn-1 rollouts played against an empty seat. Now they get their starting Settlers at the sampled map's start for their seat.
- **A far Japan city made the scripted work plan divide by zero** (`investment::work_plan`: a Japan building's network gain is negative far from the capital, which drove the rate to 0). Only odd determinized states hit it, but it's a real latent bug; rates are now floored at 1, and plain-Hard tapes are unchanged.

## 2026-10-06: Sim speed, increment 1: where a game's time goes and what was waste

- **Measure with instructions, not seconds.** At load 30-90, timings move ±30% and a single-threaded run lands on E-cores. `/usr/bin/time -l` reports instructions retired and cycles; instructions move under 1% run to run.
- **State cloning isn't the rollout cost.** `GameState::clone` takes 10 µs (duel) to 60 µs (eight players, turn 100), against 2 ms to 40 ms per rollout turn. The time is the per-turn loop: AI planning (about 65%), building a PlayerView per seat (20-30%), and resolve_turn (12-16%).
- **AI "what if" projections were most of the waste.** `diplomatic_campaign`, `choose_strike`, `forecast_siege` and `city_risk` clone the whole PlayerView to flip one relation or move a few units. `diplomatic_campaign` then builds a complete second Planner, which scored every tile for settling.

## 2026-10-06: Search gains are capped by the candidate set, not by information or evaluation

- In 512 paired 1v1 games per setting, information triggers gained 0.2 points and rival-economy calibration lost 0.9 points against the search control. More rollout information could not compensate for missing offensive candidates.

## 2026-10-06: Sim speed, increment 2: battle forecasts were quadratic, and LTO pays

- **`forecast_attack_mode` looped over all 41×41 roll pairs.** Damage to the defender depends only on the attacker's roll and vice versa, so every pair total factors into one sum per side times the other side's roll count (exact integers). That alone was about 8% of an eight-player game.
- **Memoize per planning pass what reads only fixed facts:** `exposure_key` and `safe_campaign_tile` read only the view and the planner's enemy lists, and A* asked them for the same tiles hundreds of times.
- **A distance bound beats a smarter search:** a step costs at least 1 and a turn walks at most `max_moves` steps, so a strike tile further than `approach × max_moves` can skip its A* with an identical result.

## 2026-10-06: Sim speed, increment 3: a simulation-only fast path (player: build only what decisions read)

- **The AI seats' views are now decision-only (`ViewDetail::Decisions`) on every target.** They're never shown to anyone, so `Session::ai_orders` (native and WASM alike, which keeps cross-target tapes identical) and the runner skip the labelled yield sources and focus reasons, tech unlocks' growth forecasts, income-change previews, buy quotes, upkeep previews and population unlocks. Humans' views stay `Full`.
- **The engine built a whole PlayerView (city details, research, economy, forecasts) per player per turn just to check move orders against that player's map.** `ViewDetail::Map` (tiles, units, cities, players, diplomacy) does the same checks: 2–4% of a turn.
- **Some human-looking text is a decision input; audit before cutting.** The Byzantium salvage planner parses the "Tech diffusion (N...)" label of tech costs, and buy and production blockers are read as conditions. The rule for any new cut is in `view::ViewDetail`'s docs: grep `crates/ai` for every read, then check the bench digests and repin.

## 2026-10-06: New core view fields need the mock in the same change (parity break fixed)

- AI speed pass 1 added `ProductionOption::buy_block` and `TechOption::known_by` and was gated on cargo only, so `app/src/test/diplomacy.wasm.test.ts` (2 parity tests) failed on the control version. The retired TypeScript parity client produced both through the same rule order as `economy::buy_block_with_reason` and `economy::tech_known_by`; `buy_block` is omitted when unset, like core's `skip_serializing_if`.
- **Search step 2, skateboard (1v1, paired against base search k3 h20 d2 w on the post-tile-improvements game, 512 games each): none of the wider choices helped.** - Hold-city candidates (`g`): +1.7 [−0.8, +4.1]. - A target per front (`f`): +0.0.
- **The tile-improvements merge changed the game:** the same base search on the same seeds changed 104 of 512 outcomes. Ledger rows before it are a different game version; the chart marks the break.

## 2026-10-06: Sim speed, increments 5–6: cheap view clones, shared settlement facts, PGO

- **A PlayerView clone is now mostly reference counts.** Units, cities, diplomacy, players, titles, milestones and standings joined research, economy, city detail and tiles as copy-on-write `view::Shared` parts, so an AI projection copies only what it edits. Test code that edits views needs `.into()` for whole-field assignments.
- **Every settler re-derived the same per-view settlement facts**, and so did the planner's site scores: border risks, enemy reach, and each tile's legality and potential. `settlement::SiteBasics` computes them once per view.
- **PGO**  is 3–8% fewer cycles on top of fat LTO. Duels gain most and eight players about 1%.

## 2026-10-06: Hard + Search in the app: the engine is WASM, so the budget shrinks with seats

- **How the app runs AI turns:** the Tauri app is a shell around the web UI, and the whole engine (`Game::end_turn`, every AI seat's planning) runs as WASM in the webview on one thread. Tauri has no native engine (only archive commands); `legatum-server` (LAN) is native but runs the same `Session`.
- **One budget for every path.** Determinism across native and WASM needs the same search config everywhere, so a faster native path can't have a bigger budget. `SearchConfig::in_app(players)` depends only on the player count: 2 players k3 h20 d4 w, 3-4 players k3 h20 d2 w, 5-8 players k3 h10 d2 w, all with `z` (stagger).
- **At 8 players the spike was the problem, not the mean.** One native thread, CPU time (`crates/wasm/examples/app_search_time.rs`, 7 search seats): the full 1v1 budget costs 1.5 s per search (max 3.2 s), 446 ms per turn on average, and p95 2.7 s, max 8.8 s, because the first plan's review is on the same turn for every seat, so 5 searches coincide. h10 d2 cuts a search to 360 ms but 6 still coincided (max 2.6 s).

## 2026-10-06: Sim speed, increment 7: cross-turn city yields cache

- **`yields_cache::ViewCache` keeps each city's yield results across view builds while an explicit key holds:** position, owner, faction, population, resistance, buildings, the owner's techs, terrain/resource/improvement of the centre and the city's own tiles within 3, squatters, Byzantium Salvage Works camps, and the owner's city layout (capital, Japan network).
- **The hit rate is about 80% per city build, and the gain is smaller than the profile promised: 3–5% of a game.** Most misses are real changes: border growth, population growth and new techs. What a miss costs (recomputing all ~30 building previews) outweighs what a hit saves.
- **Debug builds recompute every cached read and assert equality.** The full debug test suite and a 60-game debug sim batch (duels, small maps, 4-player mixed brains, 8-player Hard and rotated difficulties) passed with 0 panics. Leaving `population` out of the key on purpose made 2 of 4 debug duels panic, so the cross-check does catch a missing input.

## 2026-10-06: Search can't defend against an opponent it doesn't model

- With Proxy v6 now playable in 1v1, it beats plain Hard 75.1% [71.2, 78.6] over 512 games. Hard+Search does no better (−0.5 pp, only 16 of 512 outcomes change).

## 2026-10-06: Sim speed, increment 8: checks that don't word their answer

- **Blocks were worded even when only their existence mattered.** Production, building, improvement, buy, resource and tile-purchase checks are now generic over `economy::Reason`: `String` words the block, `Blocked` only reports it. AI seats' views keep every block present but empty: `buy_blocker`, `resource_blocker`, unavailable reasons, `buy_tile_blocker` and unlock blockers.
- **`buy_cost_for` built the whole labelled buy quote to read its total.** The quote's "Game speed" line brings the total to the full cost, so the price is `(full + first-Settler discount) × percent`. A debug assert compares it with the quote.
- **Result:** 4–5% fewer instructions in every scenario. Digests and full views are unchanged, and the debug suite and the debug sim batch passed.

## 2026-10-06: Sim speed, increment 9: sight remembered per map

- **Native sight was recomputed for every team on every query:** each movement round's `update_explored`, remembered threats, the views and combat. `visibility::SightMemo` lives on `Map` (serde-skipped, ignored by equality, shared by clones).
- **Checking terrain per source cost almost as much as line of sight itself** (it gained 1%). A whole-map terrain compare per update, shared by all teams, plus whole-team reuse, gained 3–4% of a game.
- **Debug builds recompute every reused team sight and footprint.** Sabotage check: matching team sight on source count alone made debug duels panic.

## 2026-10-06: Sim speed, increment 10: AI seats plan in parallel within one native game

- **Each seat decides only from its own view and brain**, so `Session::ai_orders` plans the seats on the rayon pool and gathers orders by seat id. Each seat now has its own yields cache.
- **One 8-player game's wall time drops about 2.4x** (0.51 → 0.20 s and 0.90 → 0.38 s at load 30). A duel gains nothing.

## 2026-10-06: The desktop app runs the engine in WASM, so native speedups don't reach the player

- Parallel AI seats (increment 10) cut one 8-player game's AI time about 2.5× on native, but the Tauri app runs the engine as single-threaded WASM, so a playtester's games see neither the parallelism nor native code speed. Simulations and the LAN server do.

## 2026-10-06: The release check exited 0 with a failed Playwright step

- The release-check script ran every step and logs `exit N` for each, but the script's own exit code came from its last command, so the playtest-8.3 check (release-p83) ended "exit 0" in bgjob while its main Chromium Playwright step had exited 1. A green job status was not a green check.

## 2026-10-06: Sim speed, increment 11: a search-game bench; rollouts keep a view cache

- **The primary metric is now a search game** (`sim_bench search`): 1v1 Standard on the Small map, seat 0 Hard+Search at `SearchConfig::default()` plus `w` (k3 h20 d4 w), seat 1 plain Hard, seeds 7,000,001.. (4 games). Its digest covers every final state, turn count and search decision (options, sample results, choice), so a speedup is proven not to change what the search picks.
- **A search game is mostly ordinary scripted turns.** One rollout thread, xctrace on the control version 76e1cb52: 89% of the time is inside rollouts, and within them scripted planning is 55%, views 21% and `resolve_turn` 18%; the real game's own turns are about 11%. So engine and scripted-AI speedups carry straight over to search, and nothing search-specific is a large share.
- **Rollouts built every view without the city yields cache** (`player_view_detail` makes a fresh one per call). Each thread now keeps one `ViewCache` for all its rollouts and seats (thread-local, `search::with_rollout_views`): the cache compares every input by value, so it serves any state.

## 2026-10-06: Proxy v7 (conqueror): the playbook beats Proxy v6, but only with a Settler wave after two Soldiers

- `--proxy-version 7` / `--duel-opponent proxy7` (`proxy_v7.rs`, forked from frozen v6; v6 and v5 untouched, repin shows no pin change).
- Measured with the same binary for v6 and v7 (side A is plain Hard, so lower is a stronger proxy):
- 1v1 Small, 512 games, all 64 faction pairs: seed 7100001 v7 24.4% [20.9, 28.3] vs v6 35.6% [31.5, 39.8]; seed 8100001 v7 26.8% [23.1, 30.8] vs v6 38.2% [34.1, 42.5].

## 2026-10-06: Sim speed, increment 14: cheaper route searches

- **A* allocated and filled two map-sized arrays per search** and compared 16-byte tuple keys: 175,000 searches in 4 search games, about 20 expansions each. `path::find_path_by_step` now reuses per-thread buffers stamped per search, a neighbour table, and one 64-bit key per open-list entry ((f, g, rank in `Hex` order), which sorts exactly as the tuple did), so the expansion order and every path are unchanged.
- **Movement declare reads occupancy by tile index** instead of a tree lookup per route edge: −1%.
- **Campaign approaches skip searches whose answer is already known.** `campaign_staging_spot` tries ring spots one by one; once a unit's safe flood fill exists, a spot outside it has no all-safe path, so the answer is None whatever the unfiltered route is (the unfiltered A* still ran for every such spot). And with danger near the unit the answer is always the safe route's first step, so the unfiltered search is skipped there too (no unfiltered route means no safe one).

## 2026-10-06: Sim speed, increment 15: decision views skip unread unlock previews

- **Tech unlocks worked out every city's yields with every tech-gated building installed** (`UnlockCity::yields_after`), on each view, for buildings already built or long researched. The only brain read is the Machine Shop's and Factory' Production gain (`legatum_ai::production`).

## 2026-10-06: Playtest 8.3: the Mongols AI never started its goal, and search played exactly like plain Hard

- **The Mongols goal pass skips every Market in a city without a Workshop** (`crates/ai/src/victory/goals.rs:583-595`), and nothing queues the Workshop. The production scorer always ranked a Workshop (1,295) under a Soldier (2,400) or Settler (3,000+).
- **Hard + Search issued byte-identical orders to plain Hard on all 91 turns** when both were fed the same trajectory (scratch `p83_ai.rs`: two brains side by side on the tape). Its candidates (Economy, Contain, Campaign, Peace) can't touch production, research or goals, which is where tthe recorded game was lost.

## 2026-10-06: AI goal prerequisites (playtest 8.3 follow-up)

- **Fix.** The goal pass (`victory/goals.rs::plan_faction_goal`) skipped a Mongols Market in any city without a Workshop and nothing ever queued the Workshop (the scorer ranked it 1,295 under a Soldier's 2,400). It now walks each goal item's requirement chain (`goal_need`: the foundation building, a lower tier, the unlocking technology) and starts the first startable piece.
- **Audit of every goal.** Switzerland (Gold), Japan (cities), Byzantium (Science) and Celts (a Granary, buildable from turn 1) have nothing gated. Germany's four techs are known by the end in 98%+ of 8-player seats (median turn 43 to 53) and Sparta's Fortification and Ballistics in 99%+ (median turn 46 and 68), so their misses are population and war, not an unlock.

## 2026-10-06: Hold-to-win: the AI side

- Titles are the only VP that can be taken away (milestones and faction goals are permanent in core today), so breaking a hold means taking `margin + 1` titles. `hold.rs::capture_title_losses` and `kill_takes_army_title` estimate that from public title values; foreign city income is private, so Largest Economy is apportioned by population.
- The goal-driven plan didn't fight a holder at first because Contain's reach is measured from our cities (`nearest_known`), and a field army next to the holder's capital counted for nothing. For holders our army's distance now counts too.
- AI probes (20 games each, Hard, seeds 9500001+, core hold rule before vs this AI after): every game ended by a completed hold in both. Holds started / broken: 2p Quick 23/3 → 26/6, 2p Standard 23/3 → 28/8, 4p Quick 37/15 → 38/16, 4p Standard 38/16 → 47/27 (mean length 156 → 163).

## 2026-10-05: Removing a rule variant breaks every older save

- Removing Pact and Influence made 7a saves fail to load: unknown variant `Pact`, missing `protected_until`, and a reshaped alliance `security` record. The archive can't be replayed either, because its engine commit and rules fingerprint differ.

## 2026-10-05: A forecast-feasible campaign can still never begin, so a seat-declared war can sit on paper

- The plan layer's `campaign_forecast` (a siege rollout over the units in reach) and the campaign's own start gate (`campaign_can_take`, over units within 4 hexes, plus "no new campaign while the army holds a captured city") disagree. Forcing a seat-chosen war into the plan on a bare forecast gave 3 paper wars in 6, all at 12% capture odds or less; the plan dropped them within 2 to 23 turns and the war ended in peace.

## 2026-10-05: The PlayerView has no events: infer own losses, finished items and peace from turn-to-turn diffs

- The LLM seat sees only the PlayerView, which carries no event log, so the verification's advice to "show losses from the events" cannot be followed literally. `SeatMemory::observe_changes` diffs consecutive views instead (own units gone, `producing` Some to None, research current to None, War to Peace) and names the cause heuristically.

## 2026-10-05: Population is territory: the rules, the retune and what broke

- **Rules as built.** A city works every owned tile within 3 hexes; each population above 1 claims one tile, nearest ring first and the best by focus within a ring (including beyond 3 hexes, where tiles are claimed but unworked and hatched on the map). Losing population releases the worst unworked tile, else the latest.
- **The opening got far faster, and the spread wider.** Working all of ring one at population 1 took a 48-game Hard Standard probe (seed 9100, 60 turns) to a median first Settler at T11 and second city at T17 (target T18/T25). Levers, in probes: capital +0 Production and 65% first Settler T12/T19; capital +0 and 100% T16/T23 (30 Food first growth) or T18/T24 (45); capital +2, 100%, T17/T23 at 60 Food, T19/T25 at 75.
- **Forecasts hold territory fixed.** The growth and tall-empire forecasts don't add the tile a growth will claim, so they undershoot growth slightly. Tests that compare a forecast to real resolution must box the city in (own every other tile), or the real city grows faster than forecast.

## 2026-10-04: Pathfinding bounces

- A melee unit attacking an adjacent target is a "mover" that never leaves its tile. Treating every friend with a pending order as passable sent units through attackers, so they bounced.

## 2026-10-04: Greying out the pop-1 Settler costs the AI about 3 opening turns

- Requiring pop 2 to *start* a Settler (Civ 6) pushed the Hard AI's first Settler from T18 to T21 at Standard, even with first growth cut to 10 Food (pop 2 by T4). The cause: Production before pop 2 now goes into a filler item instead of the Settler.
- Any AI filler unit costs Switzerland 3 Gold a turn and cuts their goal rate; prefer upkeep-free fillers (Monument).

## 2026-10-03: AI upkeep bankruptcy: a deliberate-disband safety net, not stricter spending

- Scripted AIs never checked upkeep before buying or building, so `rules::cities::pay_gold` force-disbanded their costliest unit about 11 times per game (200 Hard Standard games). Tightening production and purchase solvency cut disbands to 2.4 per game but suppressed armies enough to reopen a title-race stall (`coalition_stall`, seed 910030).

## 2026-10-03: Captures per game is not an AI quality metric

- A playtester pointed out that aggregate captures mix factions that should conquer (Sparta, Mongols) with ones that shouldn't (Celts razes; Switzerland turtle). Report conquest per faction against its identity, and judge AI combat skill by efficiency (captures per war declared, losses per capture, failed walled assaults) rather than volume.

## 2026-10-03: Orders that look accepted but fail

- In playtest 6 a ranged order on a city at Peace queued, drew its icon on the map, and was then rejected by core, with a misleading "no enemy there" message. Moves already asked "Declare War?" first; ranged orders never checked the relationship.

## 2026-10-03: War events: two timings, and how ally calls appear

- `WarDeclared` has two timings. For a surprise war at Peace, `relation.effective_turn` equals the resolved turn; when a Pact or Alliance is broken, it's the resolved turn + 1.

## 2026-10-03: Passive ranged garrison: a faction-conditional pass ate the unit's one order

- In playtest 6 the Athens' Ranger in Resonator held for three turns and died without firing. `plan_military` runs an early `garrison_cities` pass, only when a Mongols empire is at war with us, and it gave the garrison `hold_or_upgrade`.
- **Holds become shots:** `fire_from_holds` converts any ranged `HoldUnit` with a target in range into a shot, at every difficulty.
- **Target choice is value-based:** threat removed × kill chance, with a capture threat counting the city's HP. Shooting would-be captors first comes from the scoring.

## 2026-10-03: Scenario baselines: what the Hard AI does at five decision points

- Fifty Hard runs per scenario showed distinct failures:
- **Glass at the Gate (rush defence):** the AI proposes peace, only fortifies, never sallies against Rangers in the open, and loses the city in 50 of 50 runs.
- **The Last Bridge (rush offence, Sparta):** it captures in 49 of 50 runs, but declares war about 6 turns late.

## 2026-10-01: AI route search: unexplored tiles must count as passable

- `AiContext::move_cost` returns None for unexplored tiles, while `PlayerView::movement_path` treats unknown terrain as passable Grassland. The new settler-escort shared route used the former, so pairs at the fog frontier held forever.

## 2026-10-01: A human-playbook benchmark (Proxy v5) gets 17%; humans still win far more

- Proxy v5 (`--proxy-version 5`) plays a playtester's stated plan: about 3 cities, army, early conquest, gold into armies, then turtle and tech. It wins 17.3% (CI 13.9-21.3%) against 7 scripted Hard AIs in 8-player Standard games, against v4's 1.75% and a GreedyExpand challenger's 12.75%.

## 2026-10-01: Human-playbook benchmark v6: 31% against 7 Hard

- Proxy v6 (`--proxy-version 6`, opt-in) adds coordinated captures, earlier army assembly, purchase bursts, settler raids and consolidation to v5's plan. It wins 31.3% (CI 26.9-36.0%) against 7 scripted Hard AIs in 8-player Standard games, against v5's 15.0% on the same opponents.

## 2026-09-30: Mock ranged Battle events must report HP after the whole ranged phase

- The former TypeScript client applied ranged damage shot by shot and put the running HP in each Battle event, while core (`battle::apply_stage`) sums the whole ranged phase's damage first and reports HP after all of it. Parity tests never had two shots on one target, so nobody noticed.

## 2026-09-30: Built-in Rush can't rush either: primitive war planning shadows the opening's war logic

- 40 built-in Rush vs GreedyExpand duels: Rush won 2.5%, declared war in 20% of games (always after abandoning the rush, mean turn 127), captured nothing, and stayed at 1 city for ~160 turns while its 160-turn timeout held the economy hostage. Causes in crates/ai/src:
- `plan_primitive_war()` always claims declarations because `PrimitiveParams::default()` weights are non-zero, so `plan_opening_war()`'s muster/`war_army` logic is dead for every non-Default opening, and primitives have no minimum-force gate.
- `strike_target` is only read by gold buying. Marching uses `campaign_target_for()`, gated on aggression and unaware of the declared target.

## 2026-09-29: A human-seat benchmark exposed an expansion gap

- **The same map is effectively one game.** Re-seeding every scripted brain's tie-break jitter (16 salts) left seat 0 identical through turn 136, and outcomes diverged only after about turn 150. To get a spread, measure across maps, not across brain seeds.
- **The human was far ahead of both bots in its own seat.** On the playtest-3 map a Hard AI in that seat had 3 cities and 2 VP at turn 136, against the playtester's 8 cities and 7 VP. It won 0 of 16 games.
- **The master gap is unchanged.** On 200 rotated Standard maps with barbarians on, Proxy v4 still beats 7 Hard AIs 63.0% of the time, the same as around playtest 2. G8's pacing levers only shorten games: 60.5%, with the median win moving from turn 211 to 190.

## 2026-09-29: Playtest-3 AI post-mortem: barbarians still change builds, and the coalition never fights

- Replaying `playtest3-final-1849` with a fresh Hard `ScriptedBrain` on each seat's view reproduced all 945 taped AI turns exactly, so a playtest tape is a complete, re-runnable record of AI reasoning. `crates/sim/examples/playtest_ai_trace.rs` dumps per-turn state/events and checks this; test-only probes inside `production` and `diplomacy` print option values and war-declaration inputs at dumped turns.
- G10's "barbarians never change what a city builds" only covers `item_base_value`. `plan_emergency_production` still uses `ctx.threat` (barbarians included): 9 civilian builds were converted, including 291 Settler Production (Germany's first Settler at 147/190 at T50; it stayed at 1 city all game).
- `choose_item`'s time discount (`value × 60 / (turns + 60)` at Standard) makes a 50–64-turn Settler lose to a Workshop with lower value (Germany T81: 912 vs 1037). At the 3-Production floor the Settler rarely wins again.

## 2026-09-29: Barbarians (G10): a reserved owner, and the cost shows up in AI expansion, not survival

- Barbarian units are ordinary units owned by a reserved `PlayerId::BARBARIAN` (u8::MAX) on `TeamId::BARBARIAN`, outside `GameState::players`. `GameState::player` returns a shared read-only record for it (neutral Japan unit stats); `player_mut` has none.
- Anything that holds per-team or per-owner state has to accept an owner outside `players`: threat memory (`threat_observations`), `PlayerView::team_of`/`faction_of`, the fuzz occupancy oracle, and the UI's `atWar`.
- Sims: a barbarian-free AI would have looked fine on eliminations (0.04 per game) and game length (unchanged). The real cost was 5 to 7% of empires never founding a second city.

## 2026-09-29: G20: surprise war at plain Peace starts before movement; sleepers need the new enemies removed from their baseline

- A declaration with no Pact or Alliance now takes effect at the start of resolution (`diplomacy::declare_surprise_wars`, before barbarian planning and unit orders), so an attack in the same order list resolves that turn; the boundary skips those declarations. Breaches keep the next-turn timing and G2's AwaitingWar hold.

## 2026-09-28: Promotions (G9): the fixtures never fight, and the mock's ranged phase had drifted

- The three 40-turn determinism tapes (and every other committed tape) came out byte-identical after adding combat XP: games start at Peace and nobody fights in 40 turns. A rule that only acts in combat needs its own tape; `promotion_tape.rs` starts two empires at War (`legacy_all_war`) and fights until both Soldiers promote.
- The mock's ranged phase used its own RNG stream (`this.rng([COMBAT, 1, id])`), reported a defender roll, and let a shot unit heal the same turn; core uses `Rng::derive(seed, [turn, COMBAT, stage 0, attacker, target])`, reports defender roll 0, and marks every damaged unit as having acted. Only the new WASM promotion parity test (which compares whole results, not chosen fields) exposed it.
- AI games are sparse on fighting (65 battles per 8-empire Quick game, 117 at Standard), so with Civ 6's per-fight XP and thresholds (halved at Quick) only ~13% / 8% of surviving units are promoted at game end, mostly level 1. Battles per game do not double at Standard although thresholds do, so Standard promotes fewer units.

## 2026-09-28: Wake-on-sight makes Fortify a no-op next to a visible enemy

- `wake_threatened_sleepers` runs before the first movement round, so a unit re-fortified while an enemy military unit is in sight within its movement range wakes again in the same resolution. Played in a browser: a garrison Soldier and Scout two hexes from an enemy Soldier woke every turn ("woke: enemy military in sight within movement range"), and F could not keep them out of the cycle.

## 2026-09-28: Frozen state snapshots must hash the schema they were archived in

- AI regression tests that load an archived `GameState` (campaign_memory.rs, defense_retreat.rs) pin its hash for provenance. Adding a serialized field (the city `production_queue` and `last_purchase_turn`) changes `state.hash()` even though the archive is unchanged, so after a schema change strip the new default-valued fields from the JSON before hashing (`state::fnv1a`) instead of re-pinning a new hash; the counterfactual turns then check current-schema save/load separately.

## 2026-09-28: A passing tape can preserve old orders

- At master `78d81d1`, `legatum-sim --games 3 --seed 18446744073709551613 --turn-limit 40 --brain mixed` produces different orders from turns 8, 9 and 10 than the committed tapes: later defense-AI changes were refreshed with the order-preserving `standing_orders_tapes` example, which keeps old orders and only rewrites hashes. Replay still passes, but the tapes no longer describe what the current AI does.

## 2026-09-28: A lifetime cap on AI purchases brings back hoarded gold

- production-queue-buy-2 caps each AI at 4 or 6 military purchases per game (`Player.military_purchases`, and wars are permanent) and only buys non-military items the city is already building. Against master at `78d81d1` over 24 Quick seeds, purchases per AI seat fell from 8.3 to 1.4 a game and median gold at game end rose from 40 to 118.

## 2026-09-28: Fog made AI settlers walk between two illegal sites for 150 turns

- The shared settle scorer infers a hidden foreign city only from borders visible right now. A settler approaching site A saw the border, switched to site B; walking away it forgot A's border, so A looked legal and better again; near B the same happened.

## 2026-09-28: Diplomacy merge: hysteresis on both sides, and Peace breaks "every foreigner is an enemy"

- After giving notice on a pact, ACCEPTING a new pact needs the same cooldown as proposing it; otherwise the loop re-forms through the other side whenever the partner's progress flickers across the threshold (seed 700254 re-signed 11 pacts with one team).
- Games now start at Peace, so any code that treats every foreign unit as an enemy breaks: wake-on-sight (sleepers woke for peaceful neighbours), scout threat avoidance, siege test fixtures. Review each master merge for team-vs-War assumptions; threats are armies of teams at War with you.

## 2026-09-28: Peace starts broke the benchmark opponent

- Proxy v2 has no diplomacy code and never declares war, so after the diplomacy merge (games start at Peace) it only fought AIs that attacked it: 25% wins vs 7 Hard and 0.44 captures per game, against 83% and ~6 before. Any rules change that alters how conflict starts must be re-checked against every benchmark and harness.

## 2026-09-28: Capping simultaneous AI offers needs a public schedule, not a per-AI limit

- Each AI already sent at most one offer per turn, yet a human received three pacts on turn 2: AI brains plan independently from the same state, so no per-brain rule bounds the total. What does bound it without coordination is a schedule every AI can compute from public information: each alive AI team owns a turn slot (turn mod the number of AI teams) for new offers to a given human.

## 2026-09-28: All-war Proxy starts reproduce the old benchmark; 128-game screens overstate levers

- With `--proxy-start war` (a `GameSetup.legacy_all_war` field, omitted from JSON when false so old tapes and hashes are unchanged) the post-diplomacy Proxy benchmark reproduces the pre-diplomacy results exactly on the same seeds (Hard 186/300). Separately, the best early-defense lever looked like −9 points on a 128-game screen and was −2 on the 300-game run: screen with 128, but only report the 300.

## 2026-09-28: Evolution found no strength gain but exposed a measurement gap

- Run 2026-09-28-0313 (25 generations × 16 candidates, 128 Proxy + 32 league Quick games each, 40 personality genes): on held-out seeds the evolved AI matched the stock Hard exactly on the Proxy benchmark (65.2% vs 65.2%), so it isn't adopted. Two lessons: the per-candidate fitness was too noisy to steer (the "best" jumped around and the gen-20 checkpoint was worse than gen 10), and personality knobs can't fix behavioral gaps.

## 2026-09-28: A coalition rule with a tie-break becomes a permanent dogpile in a late pack

- Held-out coalition game 910030 (rotation 4) ran to the 1,200-turn cap with 0 captures. By turn 300 every team had banked its seven permanent VP (milestones + faction goal), four of the five titles were locked for hundreds of turns, and only Strongest Army moved.

## 2026-09-28: Wide dominates, and the first four counter levers didn't fix it

- After diplomacy, the forced-strategy study flipped: Wide (not Economic) finishes ahead of every other policy, and frontier-city defense, market plunder, earlier walls and early siege (alone and combined) didn't create a counter. An independent search found the same result.

## 2026-09-28: Faction balance after diplomacy: 5.2x at Quick, fixed by drawbacks to 1.3x

- On master after diplomacy, coalitions and the AI work, the best/worst faction win ratio was 5.2x at Quick (Germany 21% / Sparta 4%) and 2.8–2.9x at Standard (Celts 18% / Byzantium 6%). The V9 bundle of compensating drawbacks and payoffs brought it to 1.28–1.36x on 2,000 games per speed (tuning and held-out), without touching signature strengths.

## 2026-09-28: "Attack bounces" in playtest were moves onto units at Peace

- All teams start at Peace. At Peace, `enemyAt()` in app/src/commands.ts returns null, so clicking a foreign unit issued a plain MoveUnit, not an attack.

## 2026-09-28: G8: the Standard opening is slow for everyone because capitals sit at the 3-Production floor

- Replaying the playtest-3 save's tape showed the capital at exactly 3 Production (the floor) from turn 2 to 70, so an 80-cost Soldier took 26 turns and a 150-cost Settler 50. It wasn't a player mistake: over 400 Standard games, Normal AIs finish their first Settler on turn 62 and average 2 cities at turn 100 (Civ 6: first Settler turns 25–35, 5–7 cities at turn 100).

## 2026-09-28: G2 fix: attacks follow the unit, and "why nothing happened" never hides in Movement

- Follow-up to the Peace-bounce entry above. The report had filed the explaining MoveBlocked line under Movement, a category hidden by default, so a line that explains why an order did nothing belongs in Orders or Combat.

## 2026-09-27: Ruin rewards are a string, so the UI cannot celebrate them truthfully

- Plan: a structured `RuinReward` enum (docs/design/hooked-loop.md, increment 1). This changes the event's data, not the rules.

## 2026-09-27: AI stalemates came from policy gates, not impossible scores

- About 1–2 of every 1,000 games stuck at 9 of 10 VP because of these AI policy gates:
- Army quotas blocked buying a title-flipping unit.
- Campaigns ignored remembered (fogged) cities.

## 2026-09-27: The former TypeScript client drifts from core rules; test parity, don't patch cases

- the former TypeScript engine duplicates the rules for UI development and tests (real games use the WASM core). Its purchases completed instantly in the order phase, while core funds production and completes it during city processing, where a blocked spawn waits.

## 2026-09-27: `upgrade_cost: null` means "can't now", not "not in territory"

- The view's `upgrade_cost` is `check_upgrade(...).ok()`: null for a missing technology, for standing outside your own tile, and for too little Gold alike. The unit panel inferred "Only inside your territory" from null and misled a player with 59 of 60 Gold.

## 2026-09-27: Different thresholds for ending and starting a relationship make AIs oscillate

- Diplomacy gave notice on pacts when the partner reached 70% of its victory target but still proposed and accepted pacts below 75%. VP moves in steps of 10% on a 10-VP target, so every team at 7 VP triggered sign → notice → minimum term ends → re-sign, up to about 250 pacts in one game; pact count correlated with game length at r = 0.75, and each cycle's "honored" trust suppressed wars.

## 2026-09-27: A remembered route must be re-checked every turn, like a fresh path

- Movement-contention made every multi-hex human move a confirmed `MoveUnitPath` and replayed its steps blindly on later turns. Anything that appeared on the route afterwards changed the meaning of the order: an enemy on the path turned the move into an unforecast attack, and a parked friend blocked it forever (MoveUnit repaths around both).

## 2026-09-27: Compare experiments against the same control version

- Movement-contention reported captures going 1,402 → 1,471 → 1,059 over 500 games. The drop wasn't the change: the last two cohorts were built on a master that had since merged AI combat response (scouts flee, garrisons counterattack), which cuts captures.

## 2026-09-27: `RemoteClient.validate()` sends orders to the LAN server

- In app/src/remote.ts, `validate()` is also the sync path: it calls `setOrders` → `pushOrders`, so the server's copy of the seat's orders is replaced and the seat is un-readied. Its only callers pass the seat's real order list, so it isn't a live bug, but any preview or "what if" call over LAN would silently overwrite the player's orders.

## 2026-09-27: Endgame stalemates: gold piles up while no one takes the last VP

- Long Quick games (555 and 596 turns against a ~150 target) share a pattern: the tech tree ends around turn 280, all eight AIs sit at 5–9 of 10 VP in a war of attrition, and gold goes into endless military purchases (783 in one game) or sits unspent (Celts at 9 VP on 5,033 gold). The AI needs an endgame plan that converts resources into the nearest VP source; buying defenders "whenever threatened" makes it worse.

## 2026-09-27: First strategic-diversity measurement: Economic has no counter

- 480 Quick games with six forced scripted strategies (docs/sim-results/strategy-diversity-2026-09-27.md): every strategy wins some games (winner entropy 0.97), but Economic finishes ahead of each other strategy 57–71% of the time, and Rush loses to all five. Natural (unforced) AI play has winner entropy 0.68, below the proposed 0.80 floor.

## 2026-09-27: A benchmark opponent must first reproduce the result it stands in for

- The first human-proxy opponent won only 24% against 7 Hard AIs (18% vs Normal, 40% vs Easy) at Standard, while the playtester it models beat 7 Hard AIs easily by turn 197. Measuring AI improvements against it would have said "the AI is already competitive" when it isn't.

## 2026-09-26: Cross-target replay coverage

- A matching final save hash does not verify continued play after loading. Both native and WASM tape checks now replay continuously and from a save/load at every turn boundary, including the initial and final states.
- The original WASM tape check returned only turn hashes, so it missed a wrong `initial_hash`. `Game.replay_tape` now verifies the initial hash too; the native fixture suite includes a deliberately corrupted initial hash to keep that rejection covered.
- Fixed mixed-brain tapes under `crates/sim/tests/fixtures/` pin 120 turns for seeds near `u64::MAX`, independently of subsequent AI changes. Their README documents intentional regeneration and cross-target checks.

## 2026-09-26: Determinism and WASM

- Never cast a `u64` seed or hash to `usize` in game logic: `usize` is 32-bit on wasm32, so `seed as usize` truncates there and native and WASM diverge (found in `GameSetup::standard`/`all_ai` faction assignment, which also overflowed in debug for large seeds). Reduce in `u64` before casting, as in `(seed % 3) as usize`.
- On wasm32-unknown-unknown, `catch_unwind` compiles but catches nothing: a Rust panic surfaces in JS as `RuntimeError: unreachable`, with the real message only in the panic hook's `console.error`. The UI must recover by rebuilding the `Game` from its last save.
- Save JSON contains `u64` bitset words as plain JSON numbers. JavaScript must treat the save string as opaque (never `JSON.parse` and re-serialize it) or precision is lost.

## 2026-09-26: Map generation

- Setting sea level by percentile (sort tiles by elevation, take the top N%) gives an exact land share every time, and rank-normalizing climate noise over land tiles turns terrain thresholds into shares. Together they made terrain tuning predictable.
- A global flood fill per candidate mountain tile is fast enough (<1 ms per 48x32 map in release) and guarantees ridges never split the continent.

## 2026-09-26: Testing the rules

- Test states with more than 2 military units pay upkeep, and a broke empire disbands a unit per turn. Fund the treasury in spawn and stacking tests, or units silently vanish.
- Two single-empire teams can never reach their VP targets on the same turn with the current numbers (each can bank at most 7 VP without titles, and there are only 5 titles), so simultaneous-winner tiebreak tests need teams of 3+.

## 2026-09-26: The server keeps its own faction list

- `crates/server/src/staging.rs` `FactionChoice` duplicates `Faction` and adds `Random`. The factions merge added five factions to core and the UI but not here, so rooms returned HTTP 400 and wouldn't start.

## 2026-09-26: Phase-1 choices must not change the same turn's battles

- City orders apply in phase 1, before ranged combat and movement. Attack forecasts can only see the state the turn started from, so if a phase-1 order changes anything combat reads (city or unit strength), the battle no longer matches its forecast.

## 2026-09-26: Removing a state field keeps old saves loading

- `GameState`, `Player` and the view structs don't use `deny_unknown_fields`, so serde ignores a field that no longer exists. Removing Recovery charters left old saves with a `"doctrine"` key; they still load and the key disappears on the next save (`research_tree_is_exposed_and_old_charter_saves_still_load` in `crates/wasm`).

## 2026-09-26: Vitest only ran `.test.ts`, so `.test.tsx` suites were silently skipped

- `app/vite.config.ts` included only `src/**/*.test.ts`. Two component suites (`CityPanel.test.tsx` and `SeatRelease.test.tsx`) had never run and were broken: tooltips portal into `document.body`, not the render container, and happy-dom has no `window.confirm` to spy on, so use `vi.stubGlobal('confirm', …)`.

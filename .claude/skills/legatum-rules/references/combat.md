# Combat and capture

Part of the Legatum rules reference (index: `index.md`). Answers: How are battles resolved (damage, melee, ranged, pursuit, walls, siege, city strike), and how do capture, the sack, raze, the occupation lock, healing and promotions work?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

**Occupation lock (the rule in every game since rules 31, designer 2026-11-13; proposed 2026-11-09, built 2026-11-11 as an experiment):** there is no switch any more, and no lobby or simulator option. `battle::capture_city` records the civ it took the city from in `City::occupied_from` (none for a city its captor's team is retaking, none for barbarians). `rules::occupation::lock`: a military unit (aircraft excluded) inside a city its owner holds with a record cannot leave while its owner's team is at war with that civ and that civ is alive; Peace or its elimination releases it, and clears the record for good (`occupation::clear_released`, run every turn boundary after diplomacy resolves and before `hold_locked_units`, so a Peace is always seen before a later declaration): a later war with that civ neither locks the city's units nor freezes it (user, 2026-11-11, Almaliq); otherwise the record stays until the city changes hands. The lock is the city's, so a replacement unit that moves in is locked too. Locked units can still fortify, heal, skip and shoot (`RangedAttack`, auto-fire); `MoveUnit`/`MoveUnitPath`/`SetAutomation` and `DisbandUnit` are refused at validation ("<Unit> is holding <city> down and cannot leave until Peace with <people>") and again at resolution; a standing route is dropped each movement round, so a melee sortie cannot start either. Upgrades happen in place and stay allowed. The player views carry `CityView::occupation_lock` (the civ a unit of yours would be locked against by ending a move there, including by capturing a hostile city) and `UnitView::occupation_lock`; the tile card warns before entry or capture, the attack forecast repeats that warning whenever the city can be captured this turn (`forecastModel.lockWarning`, shown while the capture chance is above 0), and a locked unit is the city's garrison, held automatically: at the end of every turn `occupation::hold_locked_units` gives a locked unit with no standing order `Sleep` (fortify) if melee or `AutoFire` if ranged (the session then fires at the best visible target each turn, so the lock never costs a shot), clears any route or attack target it carried, and clears `needs_orders`, so it is never in the Return/needs-orders cycle or a pending decision and end-turn never asks about it (Heal is kept until healed, then it is held again). Locked units never wake for approaching enemies. What a locked unit still may do: fortify, heal, skip, `RangedAttack` and auto-fire, upgrade in place; a melee attack out of the city is a move and is refused. Nothing happens if a locked unit dies except that the next unit to move in is locked too (no revolt). Its map token wears a padlock badge (`mark-lock`, top-left, both zooms); its tile card says "Garrison of <city> · held here until Peace with <people>"; its unit card leads with "Garrisoning <city>: holding the city down until Peace with <people>." and a muted line "<City>'s people will not accept your rule while <people> still fights. This unit cannot leave or disband.", and offers no Move, Disband, Fortify or Skip. After Peace the unit stays fortified in the core, but the app asks for it once (designer, 2026-11-15; app only, no rules change): the turn the lock lifts on a unit of yours (locked in the view before the turn, not in the view now: `app/src/releasedGarrison.ts`), it is a pending decision in the Return queue labelled "<City> is secure, needs orders", with one turn-report line ("<Unit>: <City> is secure. Needs orders."); Fortify, Skip this turn or any other order answers it, and a unit still under the lock, one fortified by choice or a teammate's is never asked. A game loaded on the release turn has no earlier view, so it shows no prompt. A right-click or hotkey move is refused at the click (`commands.ts::lockedRefusal`) and never queued. A frozen city is never a pending decision (the wasm `city_automation_orders` skips it, `idleCitiesNeedingProduction` excludes it), its map banner shows "Held" with the Fortify icon instead of the red "Build", its tile card and city panel say "Occupied: <city> builds and buys nothing until Peace with <people>", and the city panel replaces the build list with that note (the existing queue stays visible). **City freeze (designer decision 2026-11-11, same switch):** while `occupied_from` is set, its owner's team is at War with that civ and that civ is alive (`occupation::city_freeze`, the unit lock's condition), the city makes no Production (`process_cities` skips `produce`, so nothing is banked and the yield breakdown shows "Occupied: builds nothing until Peace with <people>") and refuses every purchase and Company hire ("Occupied: <city> builds and buys nothing until Peace with <people>", through `economy::buy_check` and `company::blocker`, so validation and resolution agree; the freeze is checked before resistance, so a city that is both gives the Occupied reason, the one that outlasts resistance). Food, Gold, Science and growth continue. Queue changes are still allowed. Capture resistance (`City::resistance`, `CAPTURE_RESISTANCE_TURNS`) is stronger and independent: it zeroes every yield, stops growth and border growth, blocks buys ("is resisting occupation") and ticks down; the freeze starts counting only from the same capture and lasts until Peace or elimination, so the two overlap for the first turns and the freeze outlasts resistance. `CityView::occupation_lock` on an own city is exactly the freeze, and the city panel says so. Peace needs both teams to agree, so a loser that refuses keeps the unit lock and the freeze in place until it is eliminated. Once Peace is made the record is gone: the former owner cannot re-freeze the city by declaring war again. AI: `drop_leaving_orders_of_locked_units` and `drop_purchases_in_frozen_cities` remove refused orders; nothing else is adapted (no valuation changes; the Peace logic `wants_peace` does not know about the lock).

## 4. Combat

**Formula**: `base_damage = round(30·e^((own_str−other_str)/25))` from the table `combat::DAMAGE_TABLE` (the strength gap is clamped to ±50, so base damage runs from 4 to 222), × an independent roll per side in
**80–120%**, attacker first. Strength = base + promotion + terrain defense (defender only) +
fortify (defender only) − wounded penalty (missing-HP/10 units, /20 cities, unless faction
`ignores_wounds`). `UNIT_HP=100`, `CITY_HP=200`.

**Melee**: stepping onto a hostile tile is the attack; both sides can take damage. **Meeting**
engagements (both sides moving into the same tile) get no terrain/fortify bonus to either side. A
unit that fought ends its turn — no further move or re-attack.

**Pursuit, catching a retreat, and advancing on a kill** (verified 2026-10-11 against the p881 horseman report). A melee order onto a visible hostile unit's tile is an attack on that *unit* (`rules/units.rs::explicit_attack_target`, `Unit::attack_target`): normally each movement round the pursuer's goto is re-aimed at where the target stands now (`rules/movement.rs`, `struct Pursuit`). **A target that dies before movement** (ranged fire in phase 2) passes the attack to the next visible hostile unit still on its tile, military before civilian, then the lowest id (`movement.rs::next_target_on`, Sam 2026-11-21, RULES_VERSION 47: his Keshig's Horseman target fell to another Keshig's shot and the Settler on the same tile walked away); with nothing hostile left there the order ends with `MoveBlocked` "its target was destroyed before it could attack" (`TARGET_GONE`; before rules 47 it ended silently). Outrider is the exception: distant melee stays aimed at a pre-Modern horse's turn-start tile and stops if the horse rides away (`rules/movement.rs::aim_pursuits`, `finish_pursuits`). Every unit takes one step per round, whatever its moves, so a fast unit outruns a slow pursuer only once the pursuer has run out of moves in an ordinary pursuit. A unit attacked in melee while stepping away takes the hit where it stands without fortification (terrain counts) and completes its step if it survives (G21, the movement.rs module doc); the class bonus still applies. A melee winner that kills its defender advances into the defender's tile (module doc step 5), Civ 6 style; there is no "attack without moving in" for melee or mounted units, so a kill can leave the attacker next to the target's screen and within two tiles of a walled city. The core forecast (`forecast.rs::forecast_attack_mode`) describes only the fight and this turn's city shot from where the attacker starts; the app's forecast card adds the **"after the attack" line** for a melee attack that can win against a unit (no rule change, view-only, `app/src/aftermath.ts::attackAftermath`, shown by `ui/ForecastCard.tsx`): at the defender's tile, unfortified, with the winner's HP after the fight (an Sparta winner at 100), it lists every known enemy walled city within two tiles, regardless of terrain blocking the shot (`forecast_city_strike` mirrored: the city's public `strength` against the winner's defense) and every visible enemy military unit that can shoot it from where it stands or reach it in one turn (, the Threat lens's reach; fogged units are never counted), with expected damage at average rolls including the class bonus, applied in turn order (city, ranged, melee) so each wound weakens the next defense. The risk word is Safe (nothing can reach it), Exposed (it survives the expected total) or Likely lost (the expected total is at least its HP); Likely lost, or Exposed with half its HP gone, goes in the card's headline. The kill chance assumes a fortified defender stays fortified, and the card says so ("Assumes the Archer stays fortified (+6 Defense)"): a ranged defender that fires first loses the bonus (`rules/battle.rs`: an attacker's fortify is cleared), so the card also gives the kill chance without it. The p881 T30 example of a Spearman chasing and striking a retreating Horseman from beyond turn-start adjacency described the former rule; under Outrider that horse escapes.

**Split counterattack** (user decision 2026-10-04; `rules/battle.rs::split_counterattacks`): a
defender (a unit, or a city's melee defense) fights back once per turn. In the first movement
stage in which it is attacked in melee, its counterattack is computed once from the snapshot (its
retaliation against the strongest melee attacker: highest attack strength, then lowest unit id)
and split evenly by integer division among every melee attacker that hits it in that stage; the
strongest also takes the remainder (21 among three: 7 each; 22: 8, 7, 7). Later stages that turn
take no counterattack from it. A city and its garrison are one defending side: a garrison whose
city was assaulted in an earlier stage does not fight back, and overflow never adds a second
counterattack, except that each supporter of an assaulted city adds its own reply to the
pool (supported defence, §4). Every attacker still deals its full hit; overkill on a unit is wasted. Unchanged:
ranged attacks (no retaliation), the wall strike (once per turn), meeting engagements (each pair
still fights once, and they do not spend the counterattack). The result is the same whatever
order the orders were submitted in.

**Ranged**: no retaliation; range 1-2; the Bomber's +36 vs. cities (`data::siege_city_bonus`, design 20 on the era curve). Resolves entirely before
movement/melee. **A shot fights as the ranged class** (`combat::fight_class`, `fight_class_bonus`, used by
`rules/battle.rs::plan_fight_on` and `forecast.rs::forecast_attack_mode`; mirrored in `app/src/combat.ts`): a
mounted unit firing its bow (the Keshig) gets no class bonus for the shot, a melee-line target gets none against it
(arrows are not a charge), and a mounted target gets no "Mounted against ranged" against any shot, an Archer's included (designer, 2026-11-10; the bonus is for charging). Charging
and defending, the Keshig is mounted. Every other unit fights as its own class, so nothing else changed. At range 2, ground ranged units, including those with the Siege upgrade, need a clear line of sight (`los::ranged_clear_with_tech`), except a unit standing in its own walled city (any wall tier) or one with indirect fire. Aircraft ignore terrain blockers. Adjacent shots always work. Walled city strikes also ignore terrain blockers (§4 City strike), while team visibility still limits every target. On a hostile city's tile the city absorbs the shot while it has HP (below). A shot at a city hits the city while it has HP (`rules/battle.rs::ranged_target`).

**Walls and Siege (siege study 2026-10-18; Siege replaced the Military Engineer, user decision 2026-10-22, decisions log; `docs/design/research/siege-rework-2026-10-18.md`, results `docs/sim-results/siege-engineer/`, `docs/sim-results/siege-upgrade/`).**
- **Walls halve every attack on a city's HP** (`combat::wall_damage_percent`, `WALLS_DAMAGE_PERCENT` 50, ranged and melee alike), except attacks from a unit with Siege, bought or built in. A city's wall tier is `data::wall_tier`: 0 none, 1 Walls (the Swiss Alpine Redoubt too), 2 a Castle, 3 a Star Fort. Hits on a unit, or on a garrison once the city is at 0 HP, are never halved; air strikes are not halved. The Walls' strength and the city strike are unchanged.
  The city-fire forecast applies that same percentage to the attack after the city's shot wounds the attacker (`forecast.rs::city_fire_outcome`); the card and its "or N with Siege" hint use the reduced damage.
- **Siege is built in from the Industrial era** (designer, 2026-10-28; `data::built_in_siege`, `BUILT_IN_SIEGE_ERA` 4, the 0-based `unit_era_tier`). Every land military unit whose own era is Industrial or later (Rifleman, Cavalry, Field Cannon and on; not aircraft) fights walled cities as if Siege covered every tier: no wall strength and full damage, whatever its owner knows (`data::siege_cover` returns `MAX_WALL_TIER` 3). It is never offered: `can_have_siege` is false, `UnitView::siege_offer` is None, and an `UpgradeSiege` order is refused with "Rifleman needs no Siege: it ignores Walls, Castles and Star Forts (Industrial-era guns)" (`BUILT_IN_SIEGE_NOTE`, also `rules.generated.json` `siege.builtInNote`). `Unit::siege` stays the *bought* flag and is always false on those units: an upgrade into an Industrial or later kind clears it (`rules/units.rs`, and the order list's supply check in `rules/mod.rs`), so the unit stops using the Siege Iron. The one helper is `data::siege_cover(kind, bought_with)` (with `data::has_siege(kind, bought)` for the AI): `rules/battle.rs::siege_cover`, `forecast.rs::siege_cover` and `ai/siege_buy.rs` all call it. Walls keep their strength against everything else and the city strike; air strikes unchanged; Mongol rules unchanged.
- **Siege is a unit upgrade bought with Gold** (`rules/siege.rs`, `Order::UpgradeSiege {unit}`, `Unit::siege`, `Event::SiegeBought`). Any land military unit except a Scout, before the Industrial era (`data::can_have_siege`), once its owner knows Mathematics (`data::siege_tier` > 0), in its owner's own territory, for `data::siege_gold(kind)`: `SIEGE_BASE_GOLD` 160 Gold (the Engineer's old 80 Production at the upgrade rate of 2 Gold a Production) scaled by the unit's era on the units' own curve (`era_scaled`: about 160 through the Medieval era, 185 Renaissance, 222 Industrial, 285 Modern, 383 Future), halved by Military Engineering like every upgrade. One order per unit per turn, and the unit acts next turn. It is permanent and is kept through unit upgrades, until an upgrade into an Industrial or later kind (built in from then); the check is `siege::check` (the same words in the order validation, the resolution and `UnitView::siege_offer {price, blocker}`, which the app prints on the unit card for own units once Mathematics is known).
- **Iron.** Each Siege unit uses 1 Iron supply while it exists, as the Pikeman does (`resources::SupplyUser::Siege`, `siege_need`; a Siege Pikeman uses two): the purchase needs 1 spare Iron (counted with the Pikeman upgrades in an order list, `siege::iron_blocker`), and short of supply the newest users go idle and fight `UNSUPPLIED_STRENGTH_PERCENT` weaker (an idle Siege unit keeps its Siege). **The one switch** is `data::SIEGE_USES_IRON` (the user was "not sure" about a resource): `false` makes Siege cost Gold only, everywhere (core, `rules.generated.json` `siege.usesIron`, the app .
- **What it does** (`rules/battle.rs::siege_cover`, `forecast.rs::siege_cover`, `UnitView::siege`, public): the owner's highest siege technology is the tier (`data::siege_tier`: Mathematics 1, Machinery 2, Metallurgy 3; the tier follows the technology, so nothing is re-bought). Against walls of its tier or lower the unit fights as if the city had no Walls: the tiers' strength is left out (`combat::city_defense_covered`, `city_defense_vs`) and the damage is full. Against higher walls it leaves out the tiers it reaches and the damage is 75% (`SIEGE_PARTIAL_PERCENT`). The forecast carries `BattleForecast::walls` (`WallsForecast {tier, covered, percent, built_in}`); the app's Walls line names Siege (`ui/forecastWarnings.ts::wallsLine`), and for built-in Siege prints "Siege built in: your Rifleman ignores Walls, Castles and Star Forts (Industrial-era guns)." (core's note after the unit's name). Mirrored in `app/src/combat.ts` (`siegeCover`, `data.ts::builtInSiege`). The unit wears siege gear of its own period on the map (`app/src/siegeProps.ts`, rivals see it too): the gear is capped by the unit's own era, whatever the tier (designer, 2026-10-28: an Ancient Archer with Metallurgy wore a bombard): Ancient and Classical ranged units a catapult, Medieval ranged units a catapult or a trebuchet (tier 2 and up), Renaissance ranged units a bombard; every other unit ladders (tier 1) or a siege tower (tiers 2 and 3) through Medieval, and ladders in the Renaissance; Industrial and later units nothing (no gear and no badge: the designer rejected a badge). The prop is cosmetic: the combat tier still comes from the technologies, and the combat animation's shot at a city follows the drawn prop (`combatFx.ts::shotKindFor`). The unit card's siege note reads "Siege built in: ..." for them, with no Siege upgrade row.
- **The Military Engineer retired** (user, 2026-10-22): the unit, its build card, art, upkeep, stacked cover and capture rule are gone, with the AI's Engineer planning.
- **Mongols** (user, 2026-10-19 and 2026-10-22) raise Walls, Castles and Star Forts **from the Medieval era** (the owner's era, `data::MONGOL_WALLS_ERA` 2, through `available_to_player(faction, era, item)` and `faction_excluded_building_at`) and buy Siege from then too (`data::siege_available`); before it neither. Greyed reasons: "The Mongols raise Walls from the Medieval era" and, on the unit card, `MONGOL_SIEGE_REASON` ("No Siege before the Medieval era."). Their old half damage to city HP was removed earlier. Walls are Ancient (Masonry), so the gate is the era, not the technology.
- Roadmap, not built: units laying roads (Engineering) and building Forts (Feudalism).

**Cities shield their garrisons** (Civ 6; user decision 2026-10-03, after playtest 6). One shared
function decides what an attack on a hostile city's tile damages:
`combat::city_tile_defender(city_hp, garrisoned)` (`crates/core/src/combat.rs::city_tile_defender`).
- **City HP > 0:** every attack on the tile (ranged or melee) is aimed at the city's HP, halved by Walls unless the attacker has Siege.
  Melee retaliation uses the city's own strength:
  `combat::city_defense_for_rules` = 10 + 2 × population + Walls (10; Swiss Alpine Redoubt
  15) + Castle 5 + Star Fort 5, each line scaled to its owner's era by `data::era_scaled` (x1.00 Ancient, 1.00 Classical,
  0.96 Classical, 1.00 Medieval, 1.16 Renaissance, 1.39 Industrial, 1.51 Machine Age, 1.78 World War, 2.14 Modern, 2.88 Future (`MELEE_LINE` over `DESIGN_MELEE_LINE`); the argument is the owner's 0-based era,
  public in the view), − (200 − HP)/20, plus the opt-in `frontier-cities`/`wall-masonry` levers. **There
  is no garrison bonus:** the garrison adds nothing to city strength; it is a second layer. A stage
  that takes the city to 0 HP captures only if its overflow also kills the garrison (below).
- **City HP = 0:** ranged fire hits the visible garrison (unit defense: terrain of the city tile,
  fortify, promotion, wounds); a melee attack fights the garrison, and if the garrison dies the
  attacker advances and captures. With no garrison, a melee unit entering captures as before;
  ranged fire at an empty fallen city is held for whoever comes in (next bullet).
- **Held shots at a fallen city** (Sam, 2026-11-21, RULES_VERSION 47, from his turn-67 Geneva
  siege: five Keshig volleys hit the empty 0-HP city and did nothing, then a Swiss Spearman
  slipped in as garrison and only met the charge; his rule: "1) slip into garrison (with
  support) 2) ranged attack 3) melee attack"; `rules/battle.rs::HeldShot`, `held_city`,
  `release_held_shots`, `rules/movement.rs::movement_round`, `reserves`). A ground unit's shot
  at a hostile city that is at 0 HP with no garrison in the ranged-phase snapshot is not fired
  then: the unit spends its turn as usual and the shot waits. In the first movement round in
  which a military unit of the city's side stands on the tile (a unit slipping in under
  "Reinforcing your own city goes first", or one that arrived in an earlier round), every
  held shot on that city strikes it as a ranged hit at the head of that round's stage, before
  the round's melee hits: aimed at the garrison, shared with supporters as in "Supported
  defence", no retaliation. A held shot is itself an attack on the tile, so a unit stepping in
  under it is a reserve (garrison first, then supporters) even when no melee attacker comes.
  The melee of that round then fights what is left; the volley's and the melee's damage land
  together (the stage is one snapshot), so a garrison the volley kills still gives that
  round's melee its counterattack. If nobody comes in all turn, the held shots land on the
  empty city after movement exactly as before (a `Battle` on the city, no damage); a city
  captured or no longer hostile by then takes nothing. A city razed during movement (a Celtic capture) leaves its held shots with no city: they besiege nothing (`movement.rs::reserves` reads the city with `cities.get`, `held_shot_tests::a_held_shot_whose_city_was_razed_does_not_stop_a_later_relief`) and are dropped. A held shooter counts toward its side's
  force in a rival assault. Aircraft never hold (they return to their hangars after the ranged
  phase). Replaying his tape at rules 47, the five volleys kill the Spearman (29 + 32 + 27 + 27
  + 28) and his Keshig takes Geneva. The forecast card does not describe the held shot yet.
  Tests: `rules/held_shot_tests.rs`.
- **Overflow on a city tile is per hit** (user decision 2026-10-04, made per hit by the designer
  2026-11-11, RULES_VERSION 27; `rules/battle.rs::carried_damage`, `combat::carried_share`): the
  hits of one stage (the ranged phase, or one movement round) go one by one in resolution order.
  Each is judged against what it hits: against the city's HP it is the hit's damage to the city
  (Walls and Siege included); a hit that kills the city's HP with damage to spare carries on to the
  garrison only its unused share, `floor(direct × leftover / hit_damage)`, where `direct` is what
  the same attacker, with the same roll, deals to the garrison itself (its defence, fortification,
  class bonuses such as Melee line against mounted; no Walls), all from the stage snapshot (integer
  math). The old rule pooled the stage's damage and converted the excess by `city_defense /
  garrison_defense`; for a city without supporters that changes the garrison's wound only when the
  hits differ in class or the defences differ from the city's. Later hits in the stage strike
  whatever layer is current (the garrison once the city is at 0 HP), computed against it directly.
  Excess past the garrison's HP is wasted; with supporters, what gets past the city is shared (supported defence, below). A garrison
  stepping out of the city that round is not reached. The garrison's defence takes the same
  terrain, fortification, promotion, wound, phalanx and resource-shortage modifiers as direct
  combat (`rules/battle.rs::unit_defense_on`); a Plains Pikeman without Iron defends at 24.
- **The forecast shows the whole fight** (user, 2026-10-19, T187 Lugano; `forecast/assault.rs::forecast_city_assault`, `CityAssaultForecast`, surfaced through `GameClient.assault` and the wasm/online `assault` call, never TS rules): one card with the city's HP before and after, the garrison's HP after the overflow (with `AssaultGarrison::carried`, the expected HP lost to the carried part of the hit that empties the city, shown as a note on the city's row, "carries −M to the garrison"), the chance the city is captured this turn, and your unit's HP, **pooling every attack of yours queued on the same city** (`AlongsideAttack`). It runs the stages as resolution does (ranged stage, then the melee round: hits one by one on the city HP, the hit that empties it carrying its unused share to the garrison (`CarriedShare`), a fallen city's garrison taking hits directly), with the primary attacker's and defender's independent rolls varied over 80-120% and the other queued attacks at 100%. A melee attacker the city's own shot would destroy is left out of that roll. A shot alone never captures (`can_capture` false). The headline "Decisive" follows the capture chance, not the city's HP alone.
  Attack strength uses the same city modifiers as resolution, including the mounted charge penalty. The melee stage splits one counterattack among its attackers; capture requires at least one melee attacker to survive it (`combat::attack_strength_rules`, `forecast/assault.rs::run`).
- **Phase order:** ranged fire resolves before movement. Its overflow can damage or kill the
  garrison but never captures; a later melee stage can then fight the exposed garrison or walk
  into the emptied city. In a movement round, a combined melee assault can break the city, kill
  the garrison through overflow and capture immediately: the strongest surviving melee attacker
  that fought on that tile this round advances (highest attack strength, then lowest unit id;
  `rules/movement.rs`, "If this very stage broke the city"). A round that starts with the city at
  0 HP hits its garrison directly.
- **Reinforcing your own city goes first** (user decision 2026-10-04, from a turn-107 bounce;
  `rules/movement.rs::reserves`): inside a movement
  round, after steps are declared and classified and before any fight is planned, a plain move
  by a non-ranged military unit into its **owner's** city resolves first when the tile holds no
  military unit and an enemy attack targets that tile this round (a shot held for the fallen
  city counts, "Held shots at a fallen city" above). Per city tile the one that can
  soak the most goes first (`battle::soak`, next bullet; ties to the lowest unit id); the others
  are its supporters (next bullet). The unit moves, pays its
  step, emits `UnitMoved` then `CityReinforced`, and is the garrison for the rest of the round:
  enemy attacks on that tile are reclassified against it. The city's HP still absorbs first and
  overflow reaches the new garrison (it can die, and then the assault captures); at 0 HP the
  attack fights the new garrison as a unit (a melee, not a meeting engagement). Too late if the
  city was captured in an earlier round (the move then fails as before). Owner only: an ally's
  unit gets no priority (the old contest rules apply). No priority for ranged units, for a unit
  an enemy attacks where it stands this round, or when nobody attacks the city (an ordinary
  move). Order-independent: intents are keyed by unit id. UI: the report line "Your Legionary
  reached Keystone Wreck first and held it" (or "...fought as its garrison, but the city fell"),
  `app/src/format.ts`; a tile-card note on your own threatened, ungarrisoned city when a unit is
  selected (`app/src/ui/forecastWarnings.ts::reinforcementNote`). Tests:
  `rules/reinforcement_tests.rs`, parity in `app/src/test/cityStrike.wasm.test.ts`.
- **Supported defence: the garrison and its supporters share the damage and all fight back; a
  city falls only when all of them are dead** (designer, 2026-11-15, replacing "relief chains in"
  of 2026-11-11, RULES_VERSION 37; `rules/movement.rs::reserves`,
  `rules/battle.rs::carried_damage`, `split_counterattacks`, `soak`). His words: the walls absorb
  first, then the damage is spread among the units supporting the city; "it's basically the same
  result as if you put all the attacking units on one hex and all the defending units on one hex".
  - **Who supports.** All of the owner's non-ranged military units stepping into their own city
    (a plain move) in a round in which an enemy attacks that tile. Owner only. A unit an enemy
    attacks where it stands that round gives no support (cut support: pin it). If nobody attacks
    the city, it is an ordinary move. Supporters are never claimants of the tile: an enemy's claim
    never bounces them, and they never bounce the enemy's.
  - **Shared damage.** The city's HP (halved by Walls unless Siege) absorbs first, as above. What
    gets past it in a stage is shared among the living defenders: the garrison and every
    supporter. Per hit, in resolution order: a hit of damage B with P of it past the city (all of
    it for a hit on the garrison or on a city already emptied that stage) gives each of the n
    living defenders P / n by integer division, the remainder to the garrison (without one, the
    lowest unit id). Each takes that share of what the same attacker, with the same roll, deals
    to it directly, `floor(direct × share / B)` (its defence on the city's terrain, class bonuses;
    a supporter is never fortified), so damage is still judged against the unit type it hits.
    Overkill on one defender is wasted; later hits are shared among those still alive. A garrison
    stepping out of a city with HP is not reached; one leaving a fallen city takes its parting hit
    first (G21) and what gets past it is shared among the supporters. With no supporters this is
    the per-hit overflow above, unchanged.
  - **Supporters stay where they are.** They take their share on their own tiles and stay there,
    wounded, with `MoveBlocked` "another of your units holds the city" while the garrison lives.
  - **Backfill.** If the garrison dies (or steps out), the supporter left alive that can soak the
    most enters as the new garrison (`UnitMoved`, `CityReinforced`; enemy claimants get
    `MoveBlocked` "the city was reinforced"). Soak (`battle::soak`, designer 2026-11-15: "a
    warrior in the late game might have the most HP, but they can't absorb it") is the unit's HP
    over the damage the worst of the round's attackers deals it at an average roll, on the city's
    terrain, unfortified; ties to the lowest unit id. A full-HP Spearman loses to a 45-HP Rifleman
    against Riflemen. The same ranking picks which unit enters an empty attacked city first
    (the "goes first" bullet above).
  - **Pooled reply.** Every living defender fights back: the city's melee defence while the city
    has HP (the garrison only once the city is at 0 HP, as before), plus each supporter's own
    counterattack against the same strongest melee attacker with the same defender roll (its
    defence on the city's terrain, unfortified, class bonuses). The replies are added and split
    among the melee attackers as in "Split counterattack" (even, remainder to the strongest).
    Still once per turn per defender; ranged attacks still draw no reply; the wall strike is
    unchanged. Supporters fight no Battle of their own: their replies show in the attackers'
    `damage_to_attacker`.
  - **Capture.** The city is captured that round only when the garrison and every supporter are
    dead and a melee attacker took part (the usual captor choice). A supporter killed this way
    counts as a kill for Spartan healing and Flower Wars by everyone who hit its city's tile that
    round.
  - **Worked example** (`rules/reinforcement_tests.rs::rifleman_worked_example_prints_turn_by_turn`,
    average rolls): a walled size-5 Industrial city with a Rifleman garrison, three enemy
    Riflemen assaulting it. Alone, the garrison falls on turn 2. With three Riflemen supporting
    it every turn, the city holds and the three attackers are dead by turn 3 (under the old rule
    the supporters went in one at a time and the city fell on turn 5). With three more enemy
    Riflemen pinning the supporters, it falls on turn 3.
  - **Forecast.** `forecast/assault.rs::forecast_city_assault_supported` runs the same fight with
    the supporters the caller names (`CityAssaultForecast::supporters`, each `AssaultSupporter`'s
    expected HP and death chance, and `reply`, the attacker's expected share of the pooled
    reply). The attacker cannot see the defender's orders, so the app's card still calls
    `forecast_city_assault` (no supporters) for now.
  Tests: `rules/reinforcement_tests.rs` (`supporters_share_*`, `shared_damage_splits_*`,
  `the_backfill_is_*`, `every_supporter_adds_*`, `a_supporter_attacked_*`, `bukhara_t67_*`),
  `forecast.rs::a_supported_assault_shares_the_damage_and_pools_the_reply_as_resolution_does`.
- **Rival assaults on one city: the stronger side walks in** (designer, 2026-11-11, Lucerne: his
  city at 0/200, two empires at war with each other both ordering units in and bouncing every
  turn; `rules/movement.rs::settle_by_force`, `rules/battle.rs::city_force`). When military
  claimants of a city tile all belong to players hostile to its owner and are not all one
  permanent team (**at war with each other or not**: since 2026-11-14, Tara, India and Sparta
  both at war with the Celts but at Peace with each other bounced off an empty city for three
  turns, because the rule had only run for claimants at war with each other), each unit's **force** is the damage its attack would deal to the city's HP at an
  average (100%) roll, not limited by the HP left (the city may be at 0 HP with no garrison):
  every unit that moved onto the tile this round (melee strength, including a shooter that
  stepped on) and every unit of those players that shot at the tile this turn (ranged
  strength, aircraft included), with the city's current defence, Walls and Siege as in
  resolution. A side is one permanent team (`GameState::same_team`): Alliance partners of other teams and attackers at Peace with each other are separate sides and do not add together, and the stronger walks in. Only a side
  with a melee claimant that can advance (alive, not pinned) can win; a ranged-only side's force
  counts but never captures. It runs after the round's damage, shared with any supporters: it decides
  only who enters once the city is at 0 HP and every defender is dead (or the garrison stepped
  out). The winning side's captor is chosen as for one team; the others stay where they stood
  with `MoveBlocked` "<winner> took the city with force W against your L" (shown even for units
  that fought the garrison). The two attacking sides never fight each other over the city tile
  (the old meeting engagement at an empty 0-HP city between two of its enemies is gone; a unit
  on the owner's side still meets an attacker there as before). An exact tie in force between
  the strongest sides is drawn by seed, turn and tile (rules 51, Sam 2026-11-22: "in that case,
  random"; it used to bounce everyone; `rng::purpose::UNIT_CONTEST`, as for peaceful contests on
  open ground, movement.md): the losers read "<winner> brought equal force (N each) and won the
  draw for the city". The assault forecast does not show rival force yet. Tests:
  `rules/rival_assault_tests.rs`.
- **Garrison moving out the same turn:** during the ranged phase it is still shielded. In a
  movement round where it steps out while an attacker steps in, a city with HP takes the hit and
  the garrison leaves unharmed (no overflow reaches it); at 0 HP it takes a parting hit without fortification (G21). The
  attacker does not advance that round (the snapshot holder blocks it).
- **Civilians in the city** are never targets; on capture they are captured as before.
- **Visibility:** at 0 HP a shot hits
  the empty-looking city instead (`rules/battle.rs::ranged_target`). Melee in movement uses the
  real state.
- **Non-hostile city:** a hostile unit standing in a city the attacker is not at war with gets no
  shield (it is hit as a field unit).
- **City HP regeneration** (changed 2026-10-06, playtest 8.2: "too slowly") ignores the garrison:
  a city below 200 HP regains `data::CITY_HEAL` = **+20 HP** at the end of a turn (10 turns from
  empty to full), unless it took battle damage that turn (`City.last_attacked_turn == state.turn`,
  set in `rules/battle.rs::apply_stage` when a stage deals it damage; battles resolve before
  healing, so a hit on turn T blocks T's heal and T+1 heals again) or an at-war military unit is
  adjacent (`rules/cities.rs::heal_blocked_by_adjacent_enemy`, `heal_cities`; a barbarian never counts against a Celtic city, `rules::is_threat`). It was +5 with
  only the adjacency test. A 0-HP city is not "damaged" by attacks on its garrison, so it heals
  (+20) unless an enemy is adjacent. A fallen city that regains HP shields its garrison again.
  The `city-heal` rules experiment is gone (this is the default). The city card shows "Heals +20 a
  turn · full in N turns" or "Not healing: an enemy military unit is adjacent"
  (`CityDetail.turns_to_heal`, `heal_blocked`), the health tooltip and Guide state the rule, and the attack forecast on a city
  with HP says damaging it stops its heal. The proxies' campaign forecasts (`proxy_v5/v6
  campaign.rs::needs_rotation`) project a struck city with no heal and an unstruck one with +20.
- **City-under-attack alerts** (2026-10-08, UI only; no rule changed): `CityDetail.attacked_last_turn`
  (owner-only, absent when false; core `view.rs`) is true when `City.last_attacked_turn
  + 1 == state.turn`, i.e. the city took battle damage in the turn that just resolved, and survives a
  save, unlike the turn report. The app's alerts (`app/src/siege.ts`) also read visible at-war military
  units (scouts excluded): beside the city, or ranged within their range, they can strike it next turn.
  The capture forecast uses the same damage table and strengths as combat at an average (100%) roll; it
  models "0 HP then the garrison then captured", so only a melee unit takes a city.
- **Pursuit:** a unit pursuing a quarry that enters its own city with HP stops beside it, since
  it could only hit the city (`rules/movement.rs::movement_round`).
Used by `rules/battle.rs::ranged_target`, the melee step classification in
`rules/movement.rs::movement_round`, and `forecast.rs::forecast_attack_mode`. `BattleForecast.garrison`
(`forecast.rs::GarrisonForecast`, omitted when absent) reports the visible garrison and
`shielded`; `forecast::city_attack_target(view, hex)` answers "what does an
attack on this city tile damage" for AI scoring. Client display uses `app/src/combat.ts::cityTileDefender`; WASM behavior checks in `app/src/test/cityStrike.wasm.test.ts`. UI: the forecast headline says "The city absorbs
damage first. The hit that empties its HP carries on to the garrison with the part of its damage the city did not absorb; later hits strike the garrison in full" (`app/src/ui/forecastWarnings.ts::garrisonHeadline`),
the win line says what a win does (`forecastWinLabel`), and the Guide's combat entry explains it.
Split counterattack and overflow in the UI (`app/src/ui/ForecastCard.tsx::forecastModel`): with
other friendly melee orders on the same defender this turn, the forecast says "2 others also
attacking: you take about −7 instead of −21" and counts the share in HP, loss odds and risk; when
this turn's friendly hits on a garrisoned city exceed its HP (100% rolls) it adds "This attack /
Combined friendly attacks this turn bring the city to 0 HP and reach the garrison." The turn report
(`app/src/format.ts::formatEvents`) gives one line per combined assault, named by place, and only
that line says destroyed: "… was destroyed at Tessitura in a combined assault by your 3
Oathbearers; its counterattack was split, −7 each"; a city taken with its garrison in that round
reads "Tessitura fell to a combined assault by …; its garrison was destroyed and the city
captured."

**City strike** (default on; **walls only** since 2026-10-03, user decision, as in Civ 6): only a
city with Walls fires. Any tier counts: Walls, its Castle and Star Fort upgrades, and the Swiss Alpine
Redoubt (their Walls, `data::wall_tier`). An unwalled city never
fires; its garrison and 200 HP are its only defence, so early rushes can take fresh cities. A
walled city with HP left auto-fires once per ranged phase at the lowest-(HP, id) hostile
military unit within 2 hexes that its team can see (`rules::is_threat`: a **Celtic city never fires at a barbarian**, 2026-11-03; there is no manual city strike, so only the ranged units inside it can shoot one, by order). Forest, Hills and Mountains do not block
the strike; terrain still affects team visibility (fog). It deals no damage to the city and grants
no XP to either side; a 0-HP city stops firing.
One shared function decides it: `combat::city_can_fire(walls, hp)`
(`crates/core/src/combat.rs::city_can_fire`), used by resolution (`rules/battle.rs::resolve_ranged`)
and through `forecast::city_fires` by
`forecast_city_strike` (so `BattleForecast.city_strike` is absent for unwalled cities) and
`city_strike_target`. Unit ranged fire uses `crates/core/src/los.rs::ranged_clear_with_tech`. A ranged unit standing in its own city with any wall tier (`data::wall_tier > 0`) ignores terrain LOS blockers, as the city strike does; it still needs range and team visibility. An unwalled city's garrison and units outside cities use ordinary LOS. City strikes use range and team visibility, with no terrain LOS check. Walls are public in `CityView.walls`. The client uses `app/src/combat.ts::cityCanFire` and `app/src/los.ts::shotBlockReason`.
UI: the map draws a wall overlay on walled towns (the owner's era sprite, `app/src/map/cityArt.ts::wallStem`; the Alpine Redoubt uses the star-fort Renaissance sprite, and a Castle or Star Fort draws as plain Walls); the city panel and tile card say "Fires
at units within 2 tiles: has Walls" or "Cannot fire: no Walls" (`app/src/format.ts::cityFireStatus`);
the attack forecast notes when the target cannot fire back.
**AI and the city shield** (2026-10-03): projections of committed fire and melee use
`analysis::damage_tile_defender` (`crates/ai/src/analysis.rs`: the stage's damage hits the city
first, then each hit's unused share carries to its garrison, estimated by the defense ratio) in `military.rs::melee_forecast`,
`city_assault.rs::city_assault_succeeds` and proxy v6 `after_volley`. Ganging up is cheaper:
`military.rs::queued_split_forecast` divides a melee forecast's counterattack (and recomputes the
loss chance) by the number of this turn's queued friendly melee orders on the same tile, and
`city_assault_succeeds` splits the defending side's counterattack once among the first round's
attackers, with none in later rounds. `military.rs::best_shot`
(used by `ranged_fire` and `fire_from_holds`) shoots garrisoned cities with HP (it no longer skips
them), scores shots at a shielded tile as capture progress against the city's remaining HP
(`tactics.rs::city_shot_value`, see "Ranged fire" in section 8), and wastes no shot past 0 HP. `tactics.rs::city_shield` (Full /
MeleeOnly / None) makes a garrison of a city that cannot be broken this turn safe in
`can_defend_at`, `garrison_doomed` and `unit_exposure_key`; when it can be broken, only melee
reaches the garrison. `purchase_defense.rs::city_risk_with` applies hits to the city first and
models a ranged garrison's own shot; `production.rs::buy_emergency_defender` can buy Archers
or Crossbowmen and prefers a shooter while the city has over half its HP, a melee garrison below.
Threatened cities with over half HP prefer a ranged garrison (`military.rs::prefers_ranged_garrison`, used by `defend_cities` and `garrison_cities`). Rush sieges bombard
garrisoned cities and send the capturer only when the volley exposes the garrison and the melee
forecast beats it. `targets()` keeps its force heuristic (garrison + city/4). Proxies v2-v5 are
frozen benchmarks and keep old-rule projections; v6 is rule-aware (exposure, volley scoring,
capture finishing, rotation).

**AI**: all city-fire exposure maths go through `city_fires` (`crates/ai/src/tactics.rs`,
`analysis.rs`, `exploration.rs`, `victory/goals.rs`, proxies v5/v6). Walls are valued more when a
rival with a city within 8 hexes is at war with us, has declared any war, or masses 2+ military
within 6 hexes (`development.rs::aggressive_neighbour`, used in `production.rs` Walls score,
`infrastructure_bonus`, and Masonry research), and for Hard when a watched or top
threat (§10, `threat.rs`) can reach the city. Rush targets prefer unwalled cities: a walled
city counts 6 hexes farther (`openings.rs::WALLED_DETOUR`, `opening_strike_city`, which also drops
a pre-war target that has since built Walls), and scripted campaigns weight walled targets 1.5x
(`military.rs::campaign_target_for`). AI exposure estimates also use the shared LOS function for
city and ranged-unit fire lanes (`crates/ai/src/tactics.rs::exposure_key,enemy_exposure_key`).
Confirmed scripted routes stop before a newly visible hostile unit or city instead of submitting
an invalid path through it (`crates/ai/src/scripted.rs::unit_order`,
`crates/core/src/rules/units.rs::check_known_path`).

**Capture**: a melee unit dropping an ungarrisoned city to 0 HP captures it, as does a melee unit
that destroys the garrison of a city already at 0 HP (above); ranged alone cannot. Capture halves
the city's population, rounded down (min 1; rules 31, designer 2026-11-13; a city coming home to its founder keeps it all), clears production, loses capital status, starts 6 turns of zero
output. Celts is the exception: capture sacks and burns the city (**unless the Celtic captor founded it: a city you founded comes home intact when you retake it, through the ordinary capture path with its full population, no sack Gold, no camp and no `CityRazed`; designer, 2026-11-14, Celts game turn 156: "if i take back my own city it shouldn't be razed". A city founded by anyone else, a teammate included, is still sacked.** `battle.rs::capture_city` checks `c.founder == captor`; `forecast.rs` shows no `sack_gold` for it; `CityView::founder` is public and the AI's `sack_gold_value`/`Planner::sacks_city` value such a retake as a kept city), releases its owned tiles, and transfers
**60 Gold per population of the city, nothing else** to the Celts (`economy::SACK_GOLD_PER_POPULATION`,
`economy::celts_sack_gold(population)`; the victim's treasury is not touched; designer, 2026-11-03: "simplify the gold
capture... that explanation in the rules is too complicated"), and raises a warband camp: one rite camp
(§7, wild camps) on a free tile within 3 of the ruins, first in the territory of a Celtic city with no camp yet, then
nearest (`barbarians::raise_sack_camp`). A sack takes no technology (the 2026-10-23 steal was removed 2026-11-03: it
fired in 0 of 2 sacks in the designer's 8.9.25 game and 12% of AI Celtic sacks). The 60 Gold value doubles the previous fixed sack payout (designer review, 2026-11-06).
`CityRazed` carries `sack_gold` and `camp`, both public: `forecast.rs::forecast_attack_mode` shows the exact amount
(`ForecastCard`: "Sack: +N Gold (60 per population)"), and `event_visibility.rs` no longer strips `sack_gold` (the
ordered Raze's 10 per population is public math too); there is no optional sack order. Capture through a garrison uses the same path, so the sack, Swiss bounty and resistance all apply unchanged. An
Sparta capture fully heals its military within two hexes (`crates/core/src/rules/battle.rs::capture_city`).

**Greece Captive (Athens, 2026-11-06):** when an empire captures a city Athens owns at that instant, the captor gains one free technology to choose. The free-tech prompt names the captured city as its source (`Player::free_techs_captured`, `FreeTechsView::captured`; a broadcast clears it). No Lyceum is required and even a Celtic capture that immediately sacks the city earns it. The captor can earn this once for that city; `City::greece_captive_paid` records paid empires across later changes of hands. Athens and its teammates gain nothing for taking it. The grant uses `Player::free_techs_pending += 1` and `free_techs_from = Athens`; `Action::ChooseFreeTechs` spends it under the same current-era gate and persistent prompt as Share the Learning (`rules/actions.rs::unknown_techs`, `eras::learn`). A fully researched captor receives no unspendable pick. `AthensCityCaptured` is public and the report names the city and free technology. `CityView::greece_captive_tech` tells a rival whether its capture still pays; Athens sees the risk on its own city card. `FactionRule::AthensGreeceCaptive` turns the reward off for ablation. Hard’s `greece_captive_attack` and `greece_captive_defend` genes value the research gain in city targeting, campaigns, and Athens’s hold decision (`ai/victory.rs`, `plan.rs`, `army_commit.rs`).

**Original-capital VP is held (Sam, 2026-11-22, via the Fable review; rules 50; replaces the 2026-11-01 permanent point):** an empire scores `scoring::CAPITAL_VP` (1) for each rival original capital it **holds now**: a city with `City::original_capital` whose founder is on another team (`scoring::capitals_held`, read by `scoring::vp` as `VpSource::CapitalTaken { capital }`; the variant keeps its old name). The point moves with the city exactly like a world wonder's: whoever owns the city that turn has it, a recapture by anyone (a third empire, the founder or a teammate of the founder) takes it away the same turn, and a razed or sacked capital scores for nobody. Taking it again scores it again. The founder and its teammates never score their own capital. Capturing does not score in `rules/battle.rs::capture_city` any more: end-of-turn scoring compares the holders at the start of the turn (`objectives::CaptureVpSnapshot::capitals`, from `scoring::capital_holders`; barbarians never hold one) with the holders now and reports `Event::CapitalLost { player, capital, name }` for the old holder, then `Event::CapitalTaken { captor, capital, name }` for the new one (`scoring::check_capitals`, run after the world wonders and before `check_victory`), both public. Because team VP is counted live, losing the point can break a victory hold the same turn (`update_vp_holds`), exactly as losing a wonder does. **The Celts' sack is the one exception (pillar 8, the Celts' signature kept):** a Celtic sack of a rival's original capital (founder on another team) burns it and keeps its 1 VP for the rest of the game, since nobody can take the city back. `rules/battle.rs::capture_city`'s sack path pushes `(city, name)` onto `Player::capitals_sacked` (serde default, skipped when empty; only that path adds to it), emits public `Event::CapitalSacked { captor, capital, name }` after `CityRazed`, and `scoring::vp` counts each as `VpSource::CapitalSacked { capital }` (1). `PlayerPublic::capitals_sacked` shows them; the ledger lists them as "(sacked)". A teammate's capital sacked scores nothing. The previous holder still loses its held point (`CapitalLost`), and the capital counts as fallen for the capitals victory. **An ordered Raze** of a held capital gives the point up. `PlayerPublic::capitals_taken` is the list of capitals held now (names included), so the ledger and the map star (hollow while you hold it) read it directly. The capitals victory (`holds_own_capital`) is unchanged. Hard: `victory.rs::capital_capture_vp` reads the held point, and the raze decisions count the held point they would give up (`raze.rs::own_capital_vp`, added to the keep value in `raze_by_value` and `production.rs`'s pressure raze).

**Raze order (2026-10-25 playtest 8.9.11; 2026-10-31 rule):** any empire may order `Raze` on a city it holds but did not found, beginning with the planning turn after capture. It is one city order for the turn and resolves before movement (`orders.rs::Raze`, `rules/cities.rs::check_order,apply_order`); other city orders for the same city then fail. It burns the city, releases all its tiles and clears their improvements and landmarks, as the Celtic burn does, and pays the razing empire population × 10 Gold. World wonders and faction goal buildings can be destroyed this way, along with their held VP. Aircraft based there are destroyed; tile units follow normal movement rules. Original capitals can be razed like any other city (designer ruling 2026-11-01, "let's make all cities razable"): the captor scores its first-capture point before it can raze, and razing denies rivals theirs. The owner's `raze_blocker` has only the founder and first-turn refusals. The owner view provides `CityDetail::captured` and the exact `raze_blocker` so the city panel can explain unavailable orders and forecast the Gold. `CityRazed { ordered: true }` records the result for the report, chronicle and map; Gold stays private to the former owner and razer through `event_visibility.rs`. An ordered raze does not count as another capture for objectives (`objectives::record_fights`) or postgame taken count. Hard AI (`production::plan_raze`, `raze::held_vp_at_city`) includes the VP a former holder could regain, and for an original capital the point a rival could still score by taking it (`raze::rival_capital_vp`, 1 VP while some empire off our team and not the founder has not scored it; razing denies it), when it prices razing and keeping a captured city, with value scoring under gene `raze_value` 1 (§10, "Raze and buys"). A city with a pending `Raze` order is never put in the Choose production queue (`app/src/ui/forecastWarnings.ts::idleCitiesNeedingProduction` skips it; 8.9.29 turn 102, 2026-11-08), so Ready is not blocked on a city that burns at the turn boundary. The Celts keep their automatic capture sack of population × 30 Gold and a camp.

**Capture ends the orders aimed at the city (2026-10-07, playtest 8.4 turn 31).** When a city
changes hands (not sacked), every unit of the captor's team or an ally whose goto or confirmed route
targets that city tile, other than units standing on it, loses goto, route, `path_stop_on_block`,
`blocked_turns` and `attack_target`, and gets `needs_orders`, so it returns to the order cycle with no
"move into this city" arrow. This happens at the moment of capture, so a unit still marching in a
later round of the same turn stops too. Rivals at War with the captor keep their orders (the city
is still hostile to them). A unit's attack order ends the same way when its target is gone
(destroyed, or captured by its own side): `end_pursuit` plus `needs_orders` (`movement.rs::void_pursuit`).
An attack on a Peace unit still just becomes an ordinary move (it is reported, not voided). Code:
`rules/battle.rs::end_orders_into_city` .

**Fortify/heal**: +3 fortify/turn to +6 max, cleared by any move/attack. Heal: 20 HP/turn in own
territory, 10 in the field; barbarians never heal. Spartan units rest-heal at these normal rates too (`rules/units.rs::resting_heal`; designer, 2026-11-06). Closed foreign land heals 0.
Destroying enemy military (including Scouts and barbarians) instead restores up to 100
HP shared evenly by the Sparta units that damaged the victim during this turn. Integer
remainders go to lower unit IDs; healing above max HP is redistributed to wounded participants.
All stage damage and deaths resolve before this healing, so mutual kills revive nobody
(`rules/battle.rs::apply_stage,heal_sparta_killers`, `rules/units.rs::resting_heal`).
A garrison killed by overflow damage (user decision 2026-10-04) counts as a kill: every Sparta
unit with a damaging Battle against the victim's own city tile this turn (ranged or melee, any
stage) joins the sharing, as if it had damaged the garrison itself.

**Promotions**: automatic, **no choice of tree** (`rules/promotion.rs`). Ordinarily 5 levels,
each +10% of base melee/ranged strength; XP from fights (ratio-based, capped at 8) and city
attacks (3, or 10 for the kill). All Sparta military spawn at level 1, gain XP 50% faster
(rounded up after the ordinary XP calculation), and alone may reach level 6, Iron Guard,
at 318 XP. This applies to Oathbearers once, not as a second strength bonus
(`game.rs::spawn_unit`, `rules/promotion.rs::threshold_for,combat_xp,level_for`).

**Zone of control**: no named mechanic; pathing avoids hostiles when routing.

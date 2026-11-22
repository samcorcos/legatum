# Turn structure and orders

Part of the Legatum rules reference (index: `index.md`). Answers: What orders exist, one order per unit, idle orders, automation, resolution phases, auto-fire, order UI?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## 1. Turn structure and orders

**Simultaneous turns**: all players submit orders for the same state; `resolve_turn` applies them
together (`crates/core/src/rules/mod.rs::resolve_turn`). No "active player."

**Order enum**, one per unit/city/research slot per turn (`crates/core/src/orders.rs`):
diplomacy (`DeclareWar{against,breach}`, `BreakTreaty` = leave an Alliance,
`ProposeTreaty`, `RespondToProposal`, `WithdrawProposal`); units (`SetAutomation`, `SetAutoFire`, `MoveUnit{to}`,
`MoveUnitPath{to,path,stop_on_block}`, `SleepUnit{until_healed}`, `SkipUnit`,
`FoundCity`, `DisbandUnit`, `RangedAttack`, `UpgradeUnit`, `UpgradeSiege`); aircraft (`AirStrike`, `AirRecon`, `Rebase`,
`UpgradeAircraft`: Eras and technologies, "The air layer"); city (`SetProduction`, `SetProductionQueue`, `BuyItem`, `BuyTile{tile}`, `HireCompany`
(Byzantium only), `Raze`, `SetCityAutomation`); research (`SetResearch`, `SetResearchQueue`,
`SetScienceConversion` after the last technology);
and **one-time empire actions** through a single variant, `Action { action }` (`orders.rs::Action`,
serde tag `"action"`: `{"type":"Action","action":{"action":"Sakoku","close":true}}`): `Swarm` (the Aztecs' New Fire),
`Sakoku{close}` (Japan closes or reopens the country), `Tomb{build}` (Egypt starts or pauses work on the Royal Tomb, rules 54), `Broadcast`,
`ChooseFreeTechs{techs}`, `RetireVeteran{unit}` (a unit action: several veterans may retire in one turn,
each once), and `CallTheWild` (`ClaimObjective`, the Demobilise card's order, was removed in rules 40); later goal increments add their variants here, never a new order shape.
Checks are `rules/actions.rs::validate_action` (run by `validate_orders`: each action once per turn;
retirements beyond what completes the goal are refused), resolution is `apply_actions` right after the
city orders (spending free picks first), and `PlayerView.actions` (`ActionsView`: `swarm`, `sakoku`, `tomb`, `colonia` (Rome's level-3 founders, a preview only), `broadcast`,
`free_techs`, `retire`, `wild`) says what is available, why not, and previews it.

**Team map pings (no rules change).** A human on a team with another human can ping a tile; the teammates see a ring there until the turn ends. It is not an order and is never in the game state, a save or a tape. Online it is a host message (`ClientMessage::MapPing`, `ServerMessage::MapPing`, `crates/wasm/src/host/mod.rs`, `outbox.rs::Sender::ping`), relayed only to seats on the sender's team, 24 per seat per planning turn; hot seat keeps it in the store (`app/src/teamPing.ts`). The host's `welcome` says `map_pings` so older hosts are never sent one.

**Co-op Gold gifts (rules 50, Sam 2026-11-22; `Order::GiveGold{to,amount}`, `rules/gifts.rs`).** An empire gives Gold to a
living teammate (same team, not itself, not Barbarians). Validation (`validate_gifts`): each gift at least 1, all of a
turn's gifts together at most the giver's Gold at the start of the turn ("you have N Gold to give"). Timing: gifts move
in `resolve_turn` right after `apply_actions`, i.e. after every empire's city orders (purchases) and actions and before
ranged fire, movement, city growth, income and upkeep. A gift is capped at the Gold the giver has left then, so its own
purchases this turn are paid first; with nothing left it fails with an `OrderFailed` ("Gold gift not given: no Gold left
after this turn's purchases"). A gift to someone no longer a living teammate at resolution fails the same way. Order:
player id, then the giver's order list. `Event::GoldGiven{from,to,amount}` is visible to the team only. The teammate
spends it from next turn. The AI never issues it (`ai/src/fixture.rs` panics on one). UI: "Your team" section of the
diplomacy panel (`app/src/ui/GiveGold.tsx`), with a forecast of both treasuries.

**One order per unit per turn.** A unit is marked `acted` after its order; a second order for it is
rejected. The UI never offers a multi-step per-unit queue. A unit that attacked stays where it fought (a melee winner that kills advances into the
defender's tile, §4) and needs new orders next turn (`rules/units.rs::validate_unit_order`: "already has an order this turn (one per unit)"; Guide combat entry, `app/src/ui/GuideContent.tsx`).
**Idle orders: two, `SleepUnit` (with an optional `until_healed`, below) and `SkipUnit` (user, 2026-10-20; before that `HoldUnit`, `SleepUnit`, `SkipUnit`, `HealUnit` and `WakeUnit`; recorded use in his 24 games: Hold 221, Skip 265, Heal 98).** Player-facing names: military units say **Fortify**; civilians (including Scouts, Settlers, Military Engineers and Heirs) say **Wait**. Both names use the same `SleepUnit` order and F key. `SkipUnit` remains **Skip this turn** (Space).
`SkipUnit` does nothing this turn: it clears the destination, automation and any Sleep, and the unit asks for orders again next turn. `SleepUnit` is the one
standing order for every unit: it clears the destination and automation, keeps the unit where it is and out of the order cycle until it is woken, given another order,
or a visible military unit of a team at War newly comes into sight within its movement radius (`units::wake_threatened_sleepers`, once per newly arrived enemy, so
re-sleeping beside an enemy already there holds; an `UnitAwakened` event and `needs_orders`). Healing and fortification are not orders: any unit that neither
moved, fought nor was ordered to act rests (`units::heal_and_fortify`: +3 fortify a turn to +6, heal 20 at home, 10 in the field, nothing in closed foreign
land), asleep or not, so a sleeper heals and fortifies and stays asleep at full HP.
Old Fortify never woke for enemies; a Sleeping garrison now does (an at-war military unit newly in sight within its move radius, `units::sleeping_threats`, which asks `rules::is_threat`: **barbarians never wake a Celtic sleeper**, 2026-11-03). **Every order wakes:** any new unit order replaces the standing one. **Hold fire:** core never fires
a unit on its own, only on an explicit `RangedAttack`, so a sleeping (or skipped, or unordered) ranged unit does not shoot; the AI turns its stay-put `SkipUnit`
into a shot when a target is in range (`military.rs::fire_from_holds`) and never sleeps a ranged unit it wants to fire. Recorded tapes with the previous idle order names still deserialize through the aliases in `orders.rs` and `types.rs`.
**Heal (user, 2026-11-01; "heals the unit until it's healed then brings them back into the action queue. i find that I 'f' them and then i forget they exist")** is `SleepUnit { until_healed: true }`, not a third order: the wire shape adds a flag (`#[serde(default, skip_serializing_if = "Not::not")]`), so a plain Sleep serializes byte for byte as before and old tapes (which wrote `HealUnit`, read as plain `SleepUnit` by the serde alias and `wasm/src/tape.rs`) replay unchanged. It sets `StandingOrder::Heal` and rests and wakes for a new enemy exactly like Sleep (`is_sleeping` counts both); right after `heal_and_fortify`, `units::wake_healed` clears the order the turn HP is full, sets `needs_orders` and emits `Event::UnitHealed {unit, owner, unit_kind, at}` (owner's team only, `event_visibility.rs`). The order is refused (`units::check_heal`, in validation and again at resolution) at full HP ("already at full health") and where the unit heals nothing (closed foreign land). A Heal unit stranded in closed land by a border change is not woken (it just keeps resting for 0). The change was additive when it landed (a v8 save had no Heal state); `RULES_VERSION` is now 9, and a save that holds a Heal order cannot load in an older build.
**Auto-explore (`SetAutomation`, `Automation::Explore`) is open to every land military unit as well as the Scout (designer, 2026-11-14, rules 32; before that Scout only).** `data::can_auto_explore` (Scout or a land military unit; Settlers, Heirs and Military Engineers never, aircraft never since they live in hangars, and there are no ships) is the one test, used by `units.rs::check_automation` ("only Scouts and military units can auto-explore") and by `legatum_ai::explore_step`. The route planner is the same for every kind and uses only the fogged `PlayerView` (client-side, `wasm::explore_step`; the Hard AI never calls it): known land only, never into or beside tiles a known enemy army or firing city can strike, never through a tile held by another military unit, and never across a border closed to the unit (`passage_allowed`: foreign land is closed at Peace for every kind, open at War, to an Alliance and across a Courier Hall holder's land). At War it therefore walks into enemy land but stops the moment a known army or firing city threatens it; inside a rival's land when Peace is signed it routes out first. A unit below 100 HP, or threatened, rests in safe ground (heals in place or walks home), and below 60 HP (`EXPLORE_MIN_HP`) or when threatened it stops the automation and asks for orders; with nothing reachable left to explore it stops with `Complete` and returns to the needs-orders queue. A unit held by the occupation lock refuses the order like any move. UI: Auto-explore (`E`) is the Scout's primary; every other military unit keeps Fortify as its idle order and shows Auto-explore beside Move (never primary), in the compact and the full unit panel, hidden (like Move) while the unit garrisons a city under the occupation lock.
UI: the unit card has Fortify for military units and Wait for civilians (the primary for units that do not lead with Auto-explore or Found City, `F`), **Heal (`H`) for a damaged unit** (hidden at full HP; off with its reason under the row where nothing heals; its tooltip gives the rate and turns to full from `resting_heal`) and Skip this turn (`Space`). A healing unit wears the `mark-heal` badge on the map and "Healing" in its status, and when it wakes the queue label reads "healed to full, needs orders" (`turnFlow.ts`, from the last resolution's `UnitHealed` event); `J` is gone.

**Moving onto a hostile unit/city IS the attack** — one fused step, no "attack-then-move"
(`rules/movement.rs`, G18: moves_left/fortify zeroed after a fight). Ranged units "cannot shoot and
move in the same turn" (Guide ranged entry, `app/src/ui/GuideContent.tsx`). **A unit with both attacks (the Mongol Keshig, §8)
chooses each turn:** a `RangedAttack` from where it stands (it does not move that turn) or a move onto
the enemy (a charge). Any military unit may make the move-onto attack, but only a unit that
`data::attacks_in_melee` (the melee line, mounted units including the Keshig, Scouts) can capture or
reinforce a city; a shooter that steps onto a hostile city is bounced (`rules/movement.rs`,
`RANGED_CAPTURE`).

**Resolution phases** (`rules/mod.rs::resolve_turn`): 0) surprise wars at plain Peace take effect now; 1)
barbarian planning, unit orders, then city orders, then empire actions; 2) ranged attacks incl. aircraft strikes and city-strike bombardment (`air::resolve_ranged_phase`; a ground unit's shot at a fallen, empty hostile city is held for phase 3, §4 "Held shots at a fallen city");
3) movement rounds with combat (loops until nobody advances; inside a round a unit entering its own besieged, ungarrisoned city resolves first, see "Reinforcing your own city goes first" in §4); 4) barbarian camp clearing; 5) city
founding; 6) elimination, heal/fortify, then Heal-order wake-ups (`units::wake_healed`); 7) city production/growth/Gold/upkeep, Company expiry, research, borders, roads (`roads::extend`, `extend_roadworks`) and the strategic-supply update; 8)
scoring/victory, the diplomacy boundary (treaties, calls to arms, Peace), turn advances.

**Automatic ranged garrison fire (2026-10-31, S13).** `Order::SetAutoFire` stores
`StandingOrder::AutoFire` on a ranged unit in one of its own cities (`rules/units.rs`).
Each `Session::step` chooses a target from the owner's visible view using the scripted AI's
`Planner::best_shot` (`legatum_ai::automatic_garrison_target`) and records the resulting ranged
order for deterministic replay. The shot goes through ordinary ranged validation and resolution;
when no legal hostile target exists, the unit holds. **A Celtic garrison skips barbarians** (`Planner::best_shot_automatic`, `PlayerView::is_threat`, 2026-11-03: barbarians never attack the Celts, so only an explicit `RangedAttack` shoots one). This standing order never moves, declares War
or spends Gold. A move, wait, skip or upgrade replaces it; a manual ranged shot chooses that turn’s target while the setting persists. The client unit card controls the toggle
(`ui/UnitPanel.tsx`, `commands.ts`); `orders.ts::unitsNeedingOrders` excludes it from Return.

**UI**: click/drag to move; clicking an enemy at Peace opens a "Declare War?" dialog, never a silent
attack; ranged click-to-target with range check; Sleep/Found/Disband/Upgrade/Skip
each issue one order. Production/research queues (city-panel code, not inspected): Shift-click
queues a technology (Guide, Controls: "Research: click / Shift+click", `app/src/ui/GuideContent.tsx`). **One purchase per city per
turn**, an item (`economy.rs::buy_blocker`, `City::last_purchase_turn`).

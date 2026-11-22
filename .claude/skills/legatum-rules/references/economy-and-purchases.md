# Economy, upkeep and purchases

Part of the Legatum rules reference (index: `index.md`). Answers: What do Gold, upkeep, buying items and tiles, treasury bounds, resources and strategic supply cost and do?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## 3. Economy

**Extreme-value bounds (2026-10-31):** The treasury stops accruing at 100,000,000 Gold
(`validation::MAX_GOLD`, `rules/cities.rs::pay_gold`, `validation::credit_gold`). A city stops
growing at population 1,000; stored Food, Production, and Science stop at 1,000,000
(`validation::MAX_POPULATION`, `validation::MAX_STORED`). Save loading rejects negative or
out-of-range amounts with the affected field named (`validation::check_state`), and refuses the references
that rules would index with (2026-11-22, `validation::check_references` and `check_state`): a player id past the seats,
a tile off the map, an entity whose own id does not match its key or is above `next_city_id`/`next_unit_id`, a map
tile naming a missing city, an aircraft in a missing city, a winner or VP hold naming an unseated team; all fail as
"The save is damaged: ...". Stale references load: a team id in relations, chronicle or memories of a team with no
empire left, and a city or unit id that has merely fallen (a razed Hall or Diwan city, camp rosters, memories, supply
records and air sightings are pruned or read with `.get`, never indexed), because real games produce them.
The save is parsed once and the same JSON tree is decoded into the state (`wasm::session::from_save_json`,
`json::state_from_value`); `GameState::hash` streams the JSON into FNV-1a with the same bytes and hash as before. These bounds
leave ordinary game pacing unchanged while keeping long-run arithmetic within range.

**Upkeep**: 2 free military units/empire; each extra costs 1 Gold/turn (3 for Switzerland).
**Byzantine thin levies** count upkeep-paying land units except hired Companies, plus aircraft based in owned cities (`economy.rs::levy_count`, `production_check`). Aircraft count toward the levy cap but pay no Gold upkeep and use none of the two free upkeep slots (`economy.rs::upkeep_unit_count`, `upkeep_breakdown`, `forecast/cities.rs::forecast_cities`, `view.rs::unit_upkeep_preview`); Scouts, Settlers and the Heir count toward neither. The public `PlayerPublic.levy` drives the Armed counter and production blockers.
**Scouts and civilians (Settlers) never pay upkeep and never use a free slot, in any faction
(user decision 2026-10-06, after a Switzerland playtest where an Archer, a Spearman and a ruin's free
Scout bankrupted a 0 Gold treasury on turn 6; Civ 6 Scouts are free too)**
(`data.rs::pays_upkeep`, used by `economy.rs::upkeep_breakdown`, the bankruptcy victim
`rules/cities.rs::bankruptcy_victim`, the forecast and the build-card preview `view.rs::unit_upkeep_preview`,
which is absent for a Scout so its card reads "No upkeep"). The scripted
AI counts only `pays_upkeep` units in its upkeep, runway and disband maths (`ai/production.rs::upkeep_unit_count`).
Each active Alliance adds 1 Gold/turn per owned city (minimum 1) in a game under the calls-to-arms rules (§5; `economy.rs::treaty_upkeep`, a "Treaty upkeep: Alliance with X" line in the Gold breakdown), except on a turn the Alliance is at War with a team close to winning (below, §5 "Alliance upkeep waiver"). No technology discounts military upkeep any more (Strategic Logistics was removed; Military Engineering halves unit upgrade Gold instead). A negative treasury can force a disband
(`crates/core/src/data.rs::paid_military_upkeep`, `economy.rs::upkeep_breakdown`,
`rules/cities.rs::pay_gold`). The bankruptcy forecast includes upkeep (`economy.rs::player_income`,
`forecast/cities.rs`).

**Buying** (`BuyItem`; tiles are `BuyTile`, below): 200% of Production cost in Gold (175% Switzerland), **one purchase per city
per turn**. Nobody may buy world wonders; they must be built with Production (`economy.rs::is_unbuyable`; the Mongols' Yam Relay Station is not a world wonder and can be bought). **Walls, Castle, Star Fort, and the Swiss Alpine Redoubt cannot be bought with Gold; every Walls-line building must be built with Production.** This blocks only `BuyItem`: Production can be banked while another item is built, and a half-built Walls-line building can be finished by switching production back to it. The city panel and production list show “Walls must be built with Production; they can't be bought.” on the disabled Buy control. The UI displays the core's quote and blocker. Strategic-resource cost still applies to legal buys; bought units act next turn (`crates/core/src/economy.rs::buy_cost_for`, `buy_quote`, `rules/cities.rs::complete_item`).
**Experiment `no_purchase_under_threat` (off by default; Sam, 2026-11-22, via the Fable review; to be judged on his Swiss and Mongol games; rules 50).** `RuleExperiments::no_purchase_under_threat`, CLI `--rules-experiment no-purchase-under-threat`: a city cannot buy a military unit (`data::unit_stats(kind).military`, aircraft included) while an at-War military unit stands within `economy::THREAT_BUY_RADIUS` (2) tiles of the city centre (`economy::enemy_near_city`, with `rules::is_threat`, so barbarians count except against the Celts). `buy_check` returns `BuyBlock::UnderThreat` and "<City> can't buy military units while an enemy unit is within 2 tiles", so validation refuses the order and the production list's disabled Buy shows that reason; buildings, tiles and Companies are unaffected. Off, it changes nothing. There is no lobby switch (only World Ages has one); the Guide adds a paragraph only when a game has it on.
Swiss interest adds one Gold per 100 Gold held at the economy phase and has **no maximum**
(user, 2026-10-17: the old cap of 5 is gone; interest is floor(treasury / 100) a turn, so a hoard
compounds at about 1% a turn). It is a line of its own in the Gold breakdown ("Switzerland:
interest") and **decadence cuts it** like any other income (user, 2026-10-17): the empire's rate on the
gross interest, to the nearest Gold, shown as its own line "Decadence −N% of interest: luxury" next to
the gross line when decadence is enabled. It is still not a city yield, so it does not count toward Power. Forecast and
resolution use the same current treasury and the same rate (`economy.rs::swiss_interest_gross`,
`swiss_interest_cut`, `swiss_interest` = gross minus cut, `player_income`,
`rules/cities.rs::process_cities`). A captured Swiss city pays the captor current treasury ×
city population / total owned population, doubled for the capital, capped at the whole treasury;
the city view exposes this public bounty (`economy.rs::swiss_capture_bounty`,
`rules/battle.rs::capture_city`, `view.rs::player_view`).

**Buying a tile** (`Order::BuyTile{city,tile}`; designer, 2026-11-20, rules 46; `rules/cities.rs::check_order`, `apply_order`, `buy_tile_check_among`, `tile_price`; it existed until rules 19 and returned as a rebuild, not a revert). Legal when the tile is explored by the buyer, unowned, touches the city's territory (a claim candidate) and lies within `CITY_WORK_RADIUS` (3) of the city; tiles may be bought out of ring order, and farther tiles come only from growth (`buy_candidates`). It uses the city's **one purchase per turn**, shared with `BuyItem` and `HireCompany` (`City::last_purchase_turn`), and is refused wherever those are: the same slot, the occupation freeze and resistance checks (`buy_tile_slot_check`, mirroring `economy::buy_check`). Price in Gold: `(TILE_GOLD_BASE 40 + TILE_GOLD_PER_TILE 20 x city.tiles_claimed) x ring percent / 100`, one division rounding down, with ring percent 100 / 125 / 150 for rings 1 / 2 / 3 (`TILE_RING_STEP_PERCENT` 25; `data.rs`). `tiles_claimed` counts every tile gained since founding, grown or bought, so each purchase raises the next price. There is **no Code of Laws discount** (Code of Laws only gives a new city its best ring-2 tile). Applying it pays the Gold, sets `last_purchase_turn`, gives the tile to the city (`claim_tile`, `tiles_claimed += 1`) and emits `Event::TileBought{city,owner,tile,gold}`, visible like `TileClaimed` (`event_visibility.rs`). A bought tile is **not** a growth claim: it is not in `grown_tiles`, population loss never releases it, and it does not use up a growth claim (settling still claims one tile per population above 1). It resolves with `BuyItem` in `apply_city_orders`, before movement; if two cities of one empire order the same tile in one turn, the first in its order list gets it and the second is refused as "already owned" (validation of one empire's own list sees this too). **Two empires bidding on one tile the same turn (rules 50, seeded):** `rules/cities.rs::defer_contested_tile_buys` draws the order the bids are tried in (`rng::seeded_order` with `purpose::TILE_CONTEST`, turn and tile), so the lower seat no longer always wins; the winner's order keeps its place, and each loser's order is tried after every empire's city orders, where it is refused as "already owned" (its Gold untouched) unless the winner's own order failed. AI seats never buy tiles yet: a decision view (`ViewDetail::Decisions`) leaves `buyable_tiles` empty and `buy_tile_blocker` None, and city automation never buys one. The app opens the shared tile-placement mode from the city panel's "Buy a tile" button: the map lights each buyable tile with its price, and a click on an affordable one queues the order; a click on one the treasury cannot pay is refused in place (the mode stays open). The button is disabled while an item or company purchase is queued in the city ("Cancel the Settler purchase first"), and a queued tile refuses an item the same way: neither replaces the other (core refuses whichever comes second; Guide, "Borders").

**Resources** (`resources.rs::Resource`): Wheat/Cattle/Stone/Copper are Bonus (always visible; Stone on
Grassland, Plains or Hills; Copper on Hills with +2 Gold when improved). Five Strategic resources begin
with Iron in the Classical era (`stats(r).revealed_by`; hidden until then):

| Era | Resource | Terrain | Tile bonus (improved) | Revealed by | Users (1 each while it exists) |
|---|---|---|---|---|---|
| Ancient | None (Copper is bonus) | Hills | Copper: +2 Gold | Copper always visible | No supply users; Market and Workshop are free |
| Classical | Iron | Hills, Tundra | +1 Production | Iron Working | Phalangite, Composite Archer, Cataphract (Keshig), Forge, Siege |
| Medieval | Iron (continues) | | | | Pikeman (Tagmatic Soldier), Crossbowman, Knight, University |
| Renaissance | Niter | Plains, Desert, Grassland | +1 Food +1 Production | Gunpowder | Musketman, Arquebusier, Cuirassier, Bank |
| Industrial | Coal | Hills, Forest | +2 Production | Industrialization | Rifleman, Field Cannon, Cavalry, Factory |
| Machine Age | Oil | Desert, Tundra, Plains | +1 Production +1 Gold | Combustion | Trench Infantry, Howitzer, Early Tank, Biplane (the Stock Exchange uses nothing) |
| World War | Oil (continues; user, 2026-11-14: "it's ok if oil covers 2 eras") | | | | Infantry, Artillery, Tank, Fighter, Bomber, Research Lab |
| Modern | Oil (continues; a placeholder, so Oil covers 3 eras: the ninth era's resource was not specified) | | | | Mechanised Infantry, Rocket Artillery, Main Battle Tank, Jet Fighter, Jet Bomber (the Supermarket and the Air Defence use nothing) |
| Future | Silicon | Desert, Hills | +1 Production +1 Science | Autonomous Systems | Hunter-Killer, Combat Drone, Autonomous Armour, Stealth Fighter, Stealth Bomber |
**Strategic supply begins in the Classical era** (user, 2026-10-25; `resources::ERA_RESOURCES`,
`era_resource` returns `None` for Ancient): an era's new land units and its key building use 1 of that era's
resource from Classical onward. `unit_need`: every land military unit (`data::unit_class` Melee/Ranged/Mounted) whose
`unit_era_tier` is Classical or later uses `era_resource(tier)`; the turn-1 units (Spearman, Archer,
Horseman) and civilians need nothing. Biplane, Fighter and Bomber use 1 Oil each while in a hangar; Stealth Fighter and Stealth Bomber use 1 Silicon each. Faction unique
units share their kind's need. **An upgrade switches resource** (`upgrade_need` is the target's need when
it differs): a Pikeman upgrading to a Musketman frees 1 Iron and needs 1 spare Niter, and its supply record
restarts (`since` = that turn, so it is the newest Niter user). `building_need`: the key building of each
era (`SUPPLY_TIERS`: Forge, University, Bank, Factory, Research Lab) uses its tech's era resource; the Machine Age's Stock Exchange is not one of them (lead's call, 2026-11-14: the era shares the World War era's Oil with its units and aircraft; the Modern era's Supermarket and Air Defence use nothing either);
a higher tier keeps its lower tiers' needs (a Bank city uses Niter, a Factory city Iron + Coal, a
Stock Exchange city Niter, a Research Lab city Iron + Oil). Free: Market, Workshop, Library, Granary, Brood Hall, Walls/Castle/Star Fort, Mill,
Roadworks, War Shrine, world wonders, faction landmarks. Siege (the Gold upgrade) uses 1 Iron in every era it can be bought, on top of the unit's own need (a Siege Pikeman uses 2 Iron, a Siege Musketman 1 Niter and 1 Iron; lead's pick, 2026-10-23); Industrial and later units have Siege built in and use no Iron for it (designer, 2026-10-28). `current_users` includes aircraft in city hangars; each Biplane, Fighter and Bomber uses 1 Oil, and each Stealth Fighter and Stealth Bomber uses 1 Silicon. The AI scores aircraft, ground units and buildings against the same scarce Oil. Its supply value counts Oil that would restore an idle plane's 25% strength penalty, or support an aircraft that the air production scorer values at a Flight-era city, within the empire's hangar capacity. This value enters Oil Well, settlement and capture choices. A new aircraft's production score pays for the lab or land upgrade its Oil slot would displace, at the existing industry and wartime strength weights (`ai/supply_value.rs::demand`, `aircraft_oil_opportunity`; `ai/production.rs::item_base_value`). Search and Opening brains forward aircraft orders from the scripted policy after their ground plan (`air_wrapper_orders`, default 1).
For a rival city with Oil, the AI's capture score also counts visible rival aircraft that would lose strength if that Oil vanished (`ai/supply_value.rs::rival_loss`).
**Each era's resource is revealed by a technology of the era it is used in** (user, 2026-11-07, "that makes it feel like more of a race"; rules version 25): Iron by Iron Working (Classical), Niter by Gunpowder (Renaissance), Coal by Industrialization (Industrial), Oil by Combustion (Machine Age; Oil is also Modern's resource), Silicon by Autonomous Systems (Future). Until then an era's units and building cannot be started ("Needs Niter: Gunpowder reveals Niter on the map") and its deposits give nothing. Learning the unit's tech and the reveal are separate choices inside the era (Rifling for Riflemen, Industrialization for Coal), so the order of research is the race. Hard adds gene `reveal_tech_bonus` to the revealing technology's research value. A newly learned reveal makes known deposits appear on the map with an ink moment when Map animations are On. `ResourcesRevealed` drives one card per revealed resource with the count of explored deposits within five hexes of the player's cities and the count inside their borders. Tech cards show "Reveals Iron on the map" with its medallion. The supply panel names each resource's users in the current era.
**Strategic supply, not a stockpile** (user, 2026-10-19, "Strategic resources become a core reason for war";
increment 1, the Production line). Nothing is stored and nothing is paid:
- **Supply:** each known deposit on a tile the empire owns supplies `SUPPLY_PER_DEPOSIT` = 2 of its resource, +`SUPPLY_PER_MINE` = 1 with the appropriate improvement or a city built on it (`resources::deposits`, `supplied`). Germany gets twice both amounts: 4 unimproved, 6 improved (`resources::deposit_supply`, `mine_supply_gain`), unless its `GermanyDoubleSupply` ablation switch is off. The supply chips, tile cards and improvement build cards use these owner-specific amounts. Worked or not, inside the work radius or not.
- **Users:** every building tier or unit that needs a resource uses 1 for as long as it exists
  (`building_need`, `unit_need`, `current_users`; table above): e.g. the **Forge** (`Forge`) uses 1 Iron; the
  **Factory** (`Factory`) adds 1 Coal and keeps the Forge's Iron (a Factory city holds the Forge tier, so it
  uses 1 Iron + 1 Coal); the **Pikeman** (`Pikeman`) uses 1 Iron. The Workshop and Library, the Food line
  (Granary, Mill), the Walls line and every other item need nothing. The Bank Silver / University Copper mapping of 2026-10-19 is superseded by the era rule.
- **Starting needs spare supply:** setting production to, buying, or upgrading into a user needs
  `spare` ≥ 1 = supplied − in use − **held**, where builds in progress in *other* cities hold 1 each until
  they finish (`reserved`; the city's own active build doesn't block itself, and replacing it frees its
  hold). Queue heads wait ("…skipped queued Pikeman: No Iron to spare…; kept in queue"); validation runs
  orders in list order, so the second of two Forges with one spare Iron is refused. Upgrades take supply
  in list order before city orders (`rules/mod.rs::validate_orders`, which builds the after-upgrade state
  for the city orders). Aircraft upgrades are paid later, in the ranged phase, in aircraft-id order, so they are
  checked last, against the Gold and supply the unit upgrades and city orders of the same list leave, each valid one
  applied before the next (`air::validate_orders` takes that projected state from
  `cities::validate_city_orders_projected`; the order list's order does not matter for who is refused). A Factory also needs this city's Forge working (`start_check`). Wording
  (`shortage_text`): "No Iron to spare: your deposits supply 6 and 5 are in use, plus 1 held for the Forge in
  Bern. Win another Iron deposit, or build a Mine on one (+1 Iron)"; none at all: "Needs 1 Iron: you have no
  Iron yet. Bring an Iron deposit inside your borders"; unrevealed: "Needs Iron: Iron Working reveals Iron on
  the map". Upgrades: "upgrading to Pikeman: no Iron to spare: …".
  The top-bar supply alert is red only for an active shortage or a build/upgrade currently blocked by supply; an upgrade whose technology is still unknown does not make a 0/0 chip red.
- **Short of supply, the newest users go idle** (`update_supply`, run once at the end of resolution, after
  borders and the ResourcesRevealed announcement): each empire keeps `Player::supply`, one `SupplyRecord` per
  user (`SupplyUser::Building{city, building}` or `Unit{unit}`, sorted) with `since` = the turn the empire
  first had it (built, upgraded into or **captured**: a captured Forge is new to its captor) and `idle`. Per
  resource, users sorted by (since, user; on the same turn buildings before units) beyond the supply are idle
  until supply returns at a later count. The record is stored state, so mid-turn changes (a capture, a Mine)
  take effect at the end of the turn.
  - **An idle building tier yields nothing:** its normal yield line stays and a second line cancels it,
    "Forge idle: no Iron" −3 Production (`economy.rs::yields_base`, `resources::idle_tier`; a Factory is idle
    for Coal, or for Iron while its Forge is). Decadence then cuts the net. The yields cache keys on the idle
    tiers.
  - **An idle unit fights `UNSUPPLIED_STRENGTH_PERCENT` = 25% weaker:** a breakdown line "No Iron supply"
    −25% of its base strength (of its ranged strength when it shoots) in every fight: attack, defence, a
    meeting, a city's shot at it, and every forecast (`resources::add_unsupplied`, `rules/battle.rs`,
    `forecast.rs`, `forecast/assault.rs`). Chosen as the simplest visible penalty (lead, 2026-10-20): one
    strength line the forecast already shows, the same rule for any future unit that needs Oil, and the unit
    stays useful (a flat percent, not "can't attack"). The AI's own tactical estimates (attack,
    defence, ranged and `power` of a visible unit) subtract it for units the view shows as `unsupplied`
    (`crates/ai/src/supply.rs::attack_total` and friends, 2026-10-21); the frozen proxy brains and the app's
    `aftermath.ts`, `siege.ts` and `unitEra.ts` don't.
  - **An idle building doesn't count toward a Patrons card** (2026-10-21, `objectives::Objective::progress`):
    "a Forge in N cities" counts a city only while its Forge tier isn't idle (`resources::idle_tier`).
- **Events** (team-private): `ResourcesRevealed{player, resource, city, sources, supply}` (field renamed from
  `per_turn`)  and `SupplyShortage{player, resource, idle, supplied, used}` whenever
  the idle count changes (idle 0: covered again).
- **View:** `PlayerView.supply: Vec<SupplyView{resource, revealed_by, revealed, supplied: Breakdown (one line
  per deposit, "Iron near Lugano, with a Mine" 3), used: Breakdown ("Forge in Lugano" 1, "Pikeman ×3" 3), idle:
  [labels, newest first], held: ["Forge in Bern"], spare}>`; `ProductionOption.supply_need` and
  `resource_blocker`; `UnitView.upgrade_need` (own) and `unsupplied` (any visible unit, so a rival's forecast
  is right); `CityDetail.idle_buildings: [{building, resource}]`. The top bar shows "Iron 5 / 6" (used /
  supplied) with the breakdown tooltip; numbers come from the view only. Until the current era's resource is revealed (`resources.rs` era resource; `revealed` false in `SupplyView`), the bar shows a muted chip "Niter: revealed by Gunpowder" instead (Coal by Industrialization, Oil by Combustion, Silicon by Autonomous Systems), with a tooltip of the units and key building that resource feeds that era (`app/src/ui/SupplyChips.tsx::chipSupplies`, `resources.ts::resourceUsersThisEra`). Only the current era's unrevealed resource gets one.
- **Map (2026-10-22):** every start gets a Copper deposit 2 hexes out and an Iron deposit 3-5 hexes out (each
  nearer to it than to any other start) and 2 bonus resources exactly 2 hexes out (ring 1 stays bare, see Balanced founding sites); Niter, Coal, Oil and Silicon sit
  only in the borderlands (at least 4 hexes from every start, at most 1.5x farther from the nearest rival start
  than the nearest start, at most 3 per nearest start, 3 apart); extra Copper and Iron by land share
  (`placement.rs`, `LAND_TILES_PER_DEPOSIT` 45: about 15 of each a Standard map). Census (32 Hard games,
  2026-10-22, `docs/sim-results/era-resources/`): a typical empire (4-5 cities, land army 6-9) supplies a
  median 2-3 of its era's resource against 4-9 wanted, and about 90% of empires want more than they supply at
  every snapshot T50-T250; about 36 deposits a game change hands, all with a captured city. Before the era rule: On the Standard 48x32 map with 8 empires
  (`crates/sim/examples/supply_census.rs`, 64 Hard games, 2026-10-20): 11 Iron and 10 Coal deposits a map,
  1.4 Iron and 1.25 Coal nearest each start (median 1 each, at most 4 Iron); empires of 8-12 cities hold a
  median 4-6 Iron supply, so they can't put a Forge in every city without expanding or fighting.
- Census numbers with the supply-aware AI are in `docs/learnings.md` (2026-10-20, "Strategic supply").

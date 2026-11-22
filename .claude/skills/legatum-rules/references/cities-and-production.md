# Cities, borders and production

Part of the Legatum rules reference (index: `index.md`). Answers: How do cities grow, work tiles, claim borders, bank production, automate, and build improvements; Settler rules?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

**City/Settler rules**: production floors at 3 before buildings/penalties. Growth needs
`30 + 16×(pop-1)` Food (`data::GROWTH_BASE`, `GROWTH_PER_POP`); **the first
growth (pop 1 → 2) costs `FIRST_GROWTH` = 65 Food** (retuned 2026-10-05 for
population is territory: ring one is worked from founding, about +10 Food a turn, so a new city
reaches population 2 and its first claimed tile around turn 7; it was 10 Food when a pop-1 city
worked one tile; `data.rs::growth_threshold`,
`economy.rs::growth_threshold`, breakdown line "First growth").
**A Settler needs city pop ≥2 to be chosen, queued, continued or bought** (as in Civ 6; user
decision 2026-10-03, `economy.rs::production_blocker`, `buy_blocker`), and consumes 1 pop on
completion. A pop-1 city sees it greyed out with the reason (`view.rs::CityDetail.unavailable`).
A queued Settler that can't start is dropped from the queue with the reason; an active Settler
whose city falls to pop 1 is banked (§3, Production progress) and the city asks for a new
choice. Cost starts at 70, +50/trained Settler; every Sparta Settler, the first included, pays the +50% non-military premium after escalation and before the opening ramp (`economy.rs::sparta_nonmilitary_premium`); **an empire's first Settler costs `data::FIRST_SETTLER_PERCENT` of that price: 100% since 2026-10-05** (it was 65% to hold the opening near turn 18; population is territory made the opening faster, so the discount went; the constant and its zero-valued quote line stay; `economy.rs::production_cost`).
New cities must be **more than 3 hexes** from every existing city, including allies'; India may found cities **3 hexes apart** (`data::min_city_gap`, `rules/cities.rs::check_found_city`).
The Settle lens, settler preview, Found City blocker and AI site scoring use the same city-spacing result as founding resolution even when a city center is unexplored. `view.rs::player_view_cached` exposes only explored, otherwise-foundable tiles inside the real city gap as `PlayerView.city_gap_blocked`; it does not reveal a hidden city's center, owner or name. `settlement.rs::known_legal_site`, `site_blocker` and `SiteBasics::known_legal_site` consume that mask. A hidden-city blocker says "Too close to another city to found one here"; terrain, borders, units and site quality remain fog-limited. The gap is 2 hexes for India and 3 for every other faction (`data::min_city_gap`).

**Population-is-territory retune (2026-10-05).** Working every ring-one tile from founding made the opening much faster (a 48-game Hard Standard probe, seed 9100, 60 turns: first Settler median T11, second city T17). Retuned to hit the first-Settler-T18 and second-city-T25 targets: the first Settler costs full price (the 65% discount is gone) and the first growth costs 65 Food. Result (48 Hard games, seed 9100, 60 turns): first Settler T18, second city T24 at Standard. Levers tried, all 48-game Standard probes: capital +0 Production and 65% gave T12/T19; capital +0 and 100% gave T16/T23 at 30 Food, T18/T24 at 45 Food ; capital +2 and 100% gave T14/T20 at 30 Food, T17/T23 at 60, T19/T25 at 75 and T20/T26 at 90 (65 chosen, the capital's +2 kept so a poor start still makes 5). Start Production now spans 5 to 14 (all of ring one is worked), so terrain matters more than before. No balance sims ran (the AI is not yet competent).

**Default pacing (2026-10-22).** The capital gets +2 Production (like Civ 6's Palace). There is no opening ramp: until 2026-10-22 every Production price phased in from half the listed price on turn 0 to the full price on turn 100 (`economy::ramp_price`, the old speed surcharge); it was removed with the game-speed layer ("delete any leftover machinery", user 2026-10-21) and the early prices were retuned instead so the opening holds: the Settler costs 70 (+50 per Settler trained) and the early buildings and improvements cost what the ramp made them cost in the first third (Granary and Brood Hall 50, Workshop and Market 60, Library 80, Walls 100 (raised 2026-11-06), Farm/Pasture/Quarry 30, Mine 40); units are at their listed prices (a Spearman is 80 from turn 1, 40 before). 8-player Hard, 400 games, seed 5200001: first Settler median T18 and second city T25 before and after (`docs/sim-results/2026-10-22-opening-ramp-removal.md`); the games run about 8% shorter (309 to 285 turns on average) because the cheap early buildings stay cheap. City strike is default, but since 2026-10-03 only walled cities fire (§4). Older notes in this file about T16/T24 or T21/T29 describe earlier tunings; the numbers above are the measured ones. Individual factions can be later than these aggregate medians. See `docs/sim-results/2026-10-04-production-banking-validation.md` for per-faction measurements of the earlier tuning. Verify exact constants in crates/core/src/data.rs and economy.rs.

**City building automation (2026-10-31; focuses redesigned rules 55, Sam 2026-11-22).** A human may set each owned city's saved
`City.build_automation` to Military, Economy, Gold, Science or Food (`CityBuildFocus`; Balanced is gone and old
"Balanced" JSON reads as Economy) with
`Order::SetCityAutomation`. Its optional `picked` item is refused when it names a tile off the map
(2026-11-22: the pick is stored on the city as sent, and an online guest could otherwise make the save unloadable). This is separate from the city's
tile claim focus. At Ready the client sends its own fogged `PlayerView` to WASM and
`ai::city_automation_choice_with_pipeline`, which values available production using Hard's item values
(`ai/production.rs::automated_choice`; it no longer calls the AI's own `choose_item`).
- **Gold, Science, Food (streams):** the city runs that project and nothing else, even with no garrison or an enemy
  next door (`CityBuildFocus::stream`). Nile Egypt has no Food project, so its Food stream runs Grow.
- **Military:** combat land units first, each with a value of at least 1 even when Hard would see no use for one. It never
  trains a unit when a better unit of the same line is buildable now, strategic resources included
  (`composition::best_buildable_in_line`, `outdated_by_buildable`: an option with no `resource_blocker`); without Iron the
  Spearman is the best melee unit left and stays allowed. Among the allowed units the newest generation wins
  (`composition::quality`), value decides within it. Buildings fill in only when no unit can start or be funded (`troop_unfunded`).
- **Economy:** buildings, Great Works and improvements; never a unit of any kind.
- Neither Military nor Economy builds Settlers, Scouts or other civilians (expansion and exploring are the player's call).
  When nothing is left, they run a project: Gold while the treasury's per-turn total is 0 or less, or once every technology
  is known, otherwise Science (`Planner::idle_project`). With no active build, the choice becomes a `SetProduction` order. If the
current build is forecast to finish this turn and the queue is empty, the choice is
queued behind it with `SetProductionQueue`. An existing player queue takes precedence.
If the chooser finds no build for a city that needs a choice, Ready stops before turn resolution
and puts that city in the normal Choose production queue. The city keeps automation on; a manual
production order handles this turn. When WASM is loaded, the city warns before Ready from the
same chooser result. The WASM plan returns `orders` and `unchosen_cities`; a later chooser change
can attach a per-city reason there (`wasm/src/lib.rs::city_automation_orders_json`,
`app/src/automation.ts::cityAutomationPreview`, `app/src/turnFlow.ts::pendingDecisions`).
**Gold, Science and Food projects (Gold and Science for every empire since 2026-11-22, Sam; Germany-only before; Food since rules 55).** `ProductionItem::Project {Gold|Science|Food}` is open to every faction except that Nile Egypt has no Food project ("Egyptian cities use Grow instead"; `data::available_to_faction_rules`). The Food project turns `data::PROJECT_FOOD_PERCENT` 25% of Production into Food for everyone, Germany included (sim arm `project-food=N`). It never completes and banks nothing; each turn it adds `RuleExperiments::project_percent(faction, project)` percent of the city's Production total (after the Nile, before decadence, rounded down) as Gold, Science or Food (`economy.rs::yields_for_item`, label "Gold project (25% of Production)"). The AI's own cities never pick the Food project (`idle_project` picks Gold or Science; RandomBrain skips it). Rates: Germany 50% Gold / 25% Science (`data::GERMANY_*_PROJECT_PERCENT`, its faction strength; with `germany_industry_projects` switched off Germany gets everyone's rate), everyone else `data::PROJECT_GOLD_PERCENT` 25% / `PROJECT_SCIENCE_PERCENT` 10% (sim arms `project-gold=N`, `project-science=N`, 1..100). Egypt has no Production: on a project its surplus Food still becomes build progress (the Nile), and the project converts that progress. The AI runs a project only when no item has positive value and idling would waste Production past the idle cap (`production.rs::choose_item`, `idle_project`); its Gold-seeking swaps (campaign Gold clock, paused troops, India's Diwan fallback) pick a project only for Germany (`gold_swap_allows`). Egypt still prefers Grow when growth pays back within `egypt_grow_payback`. Gold and Science (2026-11-22) needed no `RULES_VERSION` bump; the Food project and the new focuses are rules 55.
An active project has no completion; automation replaces it at Ready. Egypt's Grow project (rules 48) never completes either.
Changing or stopping automation keeps the active item and replaces or removes an
unstarted item that automation itself queued; manual queue edits replace that marker
(`City.auto_queued`).
No automation path issues `BuyItem`, changes tile claims or reads `GameState` to plan.
The focus and last automated choice are private city detail; capture clears both
(`core/state.rs::City`, `core/view.rs::city_detail`, `core/rules/battle.rs::capture_city`,
`wasm/src/lib.rs::city_automation_orders_json`, `ai/production.rs::city_automation_choice`).

| Yield | Sources |
|---|---|
| Food | City center (2) + terrain + Wheat/Cattle; grows population; German cities lose 1 per Production building on its own stat line (including retained upgrade tiers and the Pyramids) |
| Production | City center (1) + terrain + Stone/Iron/Coal; floor of 3; Workshop family |
| Gold | City center (1) + Coast/Copper/Oil + Market family; Switzerland +1 per 2 population "tolls" and treasury interest; buys items |
| Science | Capital +2 base; other cities +pop/2; Library family; cuts research cost. Every faction's city Science accrues (the Byzantium's did not before 2026-10-10). |

There are four yields. **Influence no longer exists** (removed 2026-10-03, user decision: fewer
currencies, every trade-off priced in Gold). Saves from another rules version do not load.

**Population is territory** (user decision 2026-10-04): a city works **every** tile it owns within
`CITY_WORK_RADIUS` = 3 hexes (`economy.rs::worked_tiles`), so there is no tile-worker assignment and
no citizen allocation. A new city owns its centre plus ring one (unowned tiles only), all worked from
founding. Population above 1 is territory: see Borders below. Yields don't depend on the build any
more; the active build only steers which tile the *next claim* takes (`auto_focus`).

**Production progress is banked per item** (Civ 6 style; user decision 2026-10-03, branch
claude/production-banking). `City.production_stored` is the active unit's or building's
progress. Switching away banks it in `City.production_banked` (one entry per item); the newly
chosen item starts from its own bank (0 if none), and switching back resumes it. A banked
Settler's progress goes to the next Settler that city builds. You can't move progress from one
item to another (`state.rs::City::switch_production`, `production_for`; used by
`rules/cities.rs::apply_order`, `advance_queue`, `defer_blocked_head`, `produce`).
- **Unassigned Production** (overflow from a completion, idle banking up to the idle cap,
  Production held while a queue waits) sits in `production_stored` while no unit or building
  is active, and joins whatever starts next, together with that item's own bank. So
  **overflow still carries into the next build** (at most one completion per city per turn).
  Germany projects never hold or bank progress.
- **Buying** never adds progress (purchases are separate full-price copies, since dd1fc7ea; the
  old `City.bought` record is always empty, so there is no gold-bought progress to forfeit).
  Buying a building that already has progress keeps it like overflow: if it was the active
  build, its progress joins the next queued item; if it was banked, the bank joins the current
  build (`rules/cities.rs::apply_order`, BuyItem).
- **Stale banks are dropped** at the start of the city's production phase: a building already
  standing (or a higher tier of its family), or a world wonder once the empire has completed its
  one (`rules/cities.rs::bank_is_stale`, `prune_stale_banks`). Units never become obsolete
  here (`production_blocker` checks only tech and the Settler's population), so a unit's bank
  never goes stale and nothing needs to carry to an upgrade.
- **A lost world wonder race** stops the city for a choice (§World wonder race); its queue waits
  one production phase, then resumes.
- **The active build becoming impossible** (a Settler's city fell to pop 1, a world wonder lost
  its support) banks its progress and reports "X can no longer produce Y: reason; its N
  Production stays banked"; the city asks for a new choice (`rules/cities.rs::produce`).
- **Capture clears** the city's banks with its production (`rules/battle.rs::capture_city`).
- **No decay**: banked progress keeps its value indefinitely (Settler escalation still applies
  to the item's remaining cost).
- View: `ProductionOption.production_stored` (progress if selected now),
  `ProductionOption.production_banked`, `CityDetail.production_banked`,
  `CityDetail.unavailable` (`view.rs::city_detail`).

**Faction-useless items are hidden** (user decision 2026-10-03): master's
`data.rs::available_to_faction(faction, item)` is the one shared rule (production choices, the
tech tree, the AI through its options, `app/src/data.ts::availableToFaction`).
`data.rs::faction_unavailable_reason(item, faction)` wraps it for validation and greyed-out
text ("Only Egyptian cities can choose Grow", "Workshops, Forges and Factories do nothing for Egypt, which has no Production", otherwise "Does nothing for the X"),
checked first in `economy.rs::production_blocker`; such items are not listed as unavailable,
just absent. Faction replacements are the same item under the faction's name
(`data.rs::building_name_for`, TS `factionItemName`).

**Borders: population is territory** (user decisions 2026-10-04; `rules/cities.rs`). The old per-city
border timer is gone. Each city holds `grown_tiles` = population − 1 tiles claimed by growth, oldest
first (`City::grown_tiles`; the founding ring is not counted).

- **Claim order, "nearest first, prioritizing quality"** (`next_claim`): the next growth claims a
  tile of the *current ring*, the smallest hex distance from the city among its claim candidates
  (unowned tiles touching its territory, no reach cap). Every claimable tile of a ring goes before
  any tile of the next. Within a ring the best `claim_score` wins: the tile's yields by the city's
  focus (`economy::tile_score` of `owner_tile_yields`, whether or not it will be worked) plus
  `CLAIM_PULL_WEIGHT` (4) × `resources::border_pull` toward known unclaimed Iron/Coal. Ties go to the
  lowest hex (q, then r). Tiles beyond 3 hexes are claimed the same way. They are **unworked** and
  hatched only if no other city of the same empire can work them after the end-of-turn handoff below.
- **Settling** (`settle_territory`, run for every city in city-id order by `process_borders`, and
  at once for a ruin's population gift): claims until `grown_tiles` = population − 1, one tile per
  population, as many per turn as it takes; a city with no free candidate catches up later. Losing
  population (a Settler leaving, starvation, capture) releases tiles one at a time
  (`next_release`): the worst-scoring *unworked* grown tile (beyond 3 hexes; strategic deposits
  score +100 to keep), else the most recent claim. Event
  `TileReleased{city,owner,tile}` beside `TileClaimed`.
- **Same-empire handoff** (`reassign_workable_territory`, at the end of `process_borders`): after every city has settled its claims, a tile beyond its owning city's 3-hex work radius passes to the nearest same-empire city within 3 hexes (lower city id breaks equal-distance ties). A tile already within its city's radius stays put; land never passes between empires this way. The old city's growth-claim record and `tiles_claimed` counter lose a transferred claim; the recipient gets existing land, not another growth claim. `TilesReassigned` groups the transferred tiles by old and new city for the turn report. The map hatches only tiles no own city can work; the claim preview and tile card name a pending recipient where there is one.
- **Founding** (`found_city`): the centre always becomes the new city's (it takes the tile from
  another own city's `grown_tiles` if need be); ring one takes only unowned tiles.
- **What became of the old border-growth sources** (user, 2026-10-05, after seeing the blanket
  "everything becomes Food"): India keeps Food (+1 in every Indian city,
  `data.rs::INDIA_NETWORK_FOOD`); the **Monument and Assembly are cut everywhere**; the Embassy and Academy are removed; **Code of Laws** gives every existing city its best available ring-2 tile when learned (`eras::learn`, `rules/cities.rs::claim_code_of_laws_tile`) and later cities the same on founding. This permanent claim is not one of the grown tiles, so population loss never releases it and it does not raise `tiles_claimed`; **Irrigation** is +1 Food in every city and
  the **Granary** (+2 Food) needs no tech. India Food is an ordinary yield, so its percentages apply.

**Territory sources:** borders grow through population, conquest and a Gold purchase. Population growth claims the next unowned tile touching the city's territory using the claim order above; conquest transfers existing territory with the captured city; a city can also buy a tile (below). The view carries `next_claim: ClaimPreview{tile,yields,worked}`, `territory`, and for the owner `tile_base_price`, `buyable_tiles: [BuyableTile{tile,ring,price}]` and `buy_tile_blocker` (`view.rs::CityDetail`, `buyable_tiles`).

**Tile improvements** (user decision 2026-10-04, playtest 8; `crates/core/src/improvements.rs`). There
are no Builder units: a city *builds an improvement as a production item* that names one tile
(`ProductionItem::Improvement { improvement, tile }`, in the same `SetProduction` / queue / `BuyItem`
orders as any item; the picker names it "Mine, Iron · 2 tiles NE").
- **A resource yields its bonus only once its tile is improved** (`Tile::improved`, `improvements::tile_resource_yields`;
  a city centre on a resource counts as improved). Before that the tile is plain land, and its bonus
  is not in the city's "Resources" line. The city breakdown names each improved resource with its
  improvement ("Resources: Wheat (Farm), Iron (Mine)").
- **Seven improvements cover the nine resources** (`kind_for`): Farm (Wheat), Pasture (Cattle), Quarry
  (Stone), Mine (Copper, Iron, Coal), Saltpetre Works (Niter), Oil Well (Oil), Quartz Mine (Silicon). Only resource tiles can be improved. Cost (with the Sparta non-military +50%): Farm, Pasture and Quarry 30,
  Mine, Saltpetre Works, Oil Well and Quartz Mine 40 Production (`base_cost`); buyable like any item (200% / 175% Gold), one purchase per city per turn.
- **Eligibility** (`improvements::blocker`, shared by validation, the picker, the AI and queue upkeep):
  the tile belongs to *this* city and lies within the 3-hex work radius (tiles claimed beyond it only block
  rivals, and the city centre can't be targeted), its resource is *known* to the owner (a hidden strategic
  deposit can't be targeted before its revealing technology), the improvement matches the resource, it
  isn't already improved. An item whose tile stops
  qualifying ("can no longer produce…") stops like any impossible build, and its progress stays banked.
- **Upgrades are automatic** (no re-building): the owner's technology decides the form
  (`upgrade_tech`): Crop Rotation turns Farm into Irrigated Farm and Pasture into Ranch;
  Masonry turns Quarry into Deep Quarry; Iron Working turns Mine, Saltpetre Works and Quartz Mine into their Deep forms (`upgrade_tech`); Combustion upgrades the Oil Well (+1 Gold). Railways adds another Production to Mines. An upgraded improvement adds
  one more of the resource's main yield (`upgrade_yields`: Food for Wheat, Cattle and Niter, Production for
  Stone, Iron, Coal and Silicon, Gold for Copper and Oil). The tech cards list it as an effect line.
- **Population loss keeps the released tile's improvement** (`rules/cities.rs::settle_territory`; training a Settler can release an improved tile). Razing and founding a city on an improved centre clear it; capture passes improved land to the captor intact. **Supply:** owning a known deposit is enough to supply 2; a Mine on Iron or
  Coal adds +1 supply (+2 for Germany) as well as the tile's yield.
- **Territory interaction:** claim scoring and release keep valuing a known resource at its improved
  worth (`economy::owner_tile_potential`), so claims still lean toward resources; the claim preview
  and the worked yields show only what the tile yields now. Improvements are banked per item like
  buildings (`City::holds_progress`, `switch_production`).
- **View:** `TileView::improvement` (kind, and `upgraded` on the viewer's own tiles only; remembered
  for fogged tiles through `TileMemory::improved`), `ProductionOption::improvement` (tile, resource,
  gain, and the upgrade still to come) and `CityDetail::options` lists every eligible tile. (Egypt's city view omits Quarry, see the Egypt Nile entry.)
- **AI** (`crates/ai/src/production.rs::improvement_value`): an improvement is valued by the yield it adds
  (Food 4, Production 3, Gold 2, Science 3; the upgrade counts half) over the 30-turn planning horizon
  against its cost, scaled by `knobs.economy`, and ranked with Settlers and units by the usual
  time-discounted production score; it waits while a city is threatened. WASM behavior checks in `app/src/test/improvements.wasm.test.ts`; core tests in `crates/core/src/improvements/tests.rs`.
- Not built: a delegation tech that auto-queues improvements (follow-up, per pillar 3).

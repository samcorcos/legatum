# AI: architecture

Part of the Legatum rules reference (index: `index.md`). Answers: How is the AI built (views, ScriptedBrain, difficulty, openings, threat response, conquest, proxies, victory goals)?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

- **Opponent assessment** (`crates/ai/src/opponents.rs`, `adaptation.rs`): scripted and opening
  brains retain per-rival observations from `PlayerView` (persisted in session saves), refreshed every five turns and on first
  physical contact. Public faction unit data supplies rush priors; known city walls, visible
  garrisons, known land routes, public city counts/declarations, treasury capture bounties and
  India links inform targets and border defense. Army changes are sighting deltas, and fogged
  garrisons remain unknown. A remote scout does not make a rival a homeland neighbor.
  A rush-capable neighbour within 10 hexes raises the military knob (`DefendedExpand`); only
  observed aggression (2+ Spearmen's strength at our border, a visible buildup or any war it
  declared: `OpponentAssessment::observed_aggression`) makes exposed cities research legal Walls
  prerequisites and build a garrisoned shooter, then Walls (`production.rs::choose_item`,
  `adaptation.rs::assessed_border_threat`); a faction prior alone never walls (units counter
  rushes before Walls).
  Early rush/conquest plans with known but unsuitable targets pivot to expansion; city combat
  forecasts still govern tactical execution. early walled campaigns are avoided except
  finishing an already badly damaged city. Strategy switches record player, turn, rival, old/new
  plan and reason through `Brain::strategy_switches`. The shared Hard conquest nomination
  gates army funding, staging and war persistence (`conquest.rs::assessed_conquest_allowed`):
  only early walled cities are ruled out (`adaptation.rs::assessed_campaign_allowed` likewise);
  Celtic land is no cost or ban any more (the +120 was deleted with the homeland damage on 2026-10-23); a strong or unseen garrison is a cost
  in `choose_conquest_city`'s score, not a ban (the earlier viable-garrison gate removed most
  AI-vs-AI captures). The weakened-city exception for those requires a visible target, an
  existing war and healthy local capture units. Unwalled targets need healthy infantry; firing cities additionally need ranged
  support. Simulator target-quality counters use
  observer-only pre-turn snapshots (`crates/sim/src/opponent_telemetry.rs`). No game rule changes.

- **Threat response** (`crates/ai/src/threat.rs`, AI only, no rule changes; playtest 6): every
  scripted brain keeps a per-turn `ThreatTracker` inside its opponent memory. From public facts
  only (each player's `alive` and city count, all diplomatic relations including those with
  eliminated empires, the Strongest Army title, VP) plus its own sight, it infers captures
  (public city counts moving between empires at war, known cities changing owner), who
  eliminated whom (a dead empire's war partner, preferring the declarer, so a loaded save
  without AI memory still sees past eliminations), wars declared, armies near its cities and
  their trend, city growth and distance. Each rival gets a score (eliminations 35, recent
  captures 15, declared and ongoing offensive wars 10, a surprise war by the Strongest Army
  holder +10/+20, growth, size, VP, nearby army; times a proximity factor). Wars declared on
  the common top threat do not count against the declarer. At 30 a rival is *watched*: Hard
  researches Masonry and values Walls in cities within 12 hexes of it. At 60 it is the
  *top threat*: Hard (`Planner::threat_aware`, plain Hard and Hard openings; Easy/Normal
  unchanged) treats it as the containment target (no pacts or alliances with it, notice on
  existing ones, coalition scoring), starts Walls with Production before the
  shooter when the arrival forecast allows it, otherwise puts a ranged unit in each exposed
  city first and then switches production to Walls. No faction buys a Walls-line building.
  It raises army investment through `conquest_pressure`, prefers ranged garrisons, and
  values its recently captured cities as campaign targets. Alliances against it are not
  abandoned when its VP dips, and AIs sharing it relax the Alliance trust minimum to 0. A war
  on it (second front) needs `second_front_value` ≥ 0: neutral for Celts (temperament 0 since 2026-10-23, once "they defend at home"), a
  treasury covering ten Standard turns of deficit, 80% of its public army together with teams
  already at war with it (half alone unless it is fighting elsewhere or allies are in), a
  known city of its within 12 hexes of one of ours on the same landmass, a concrete objective
  (such a city weakly held, or allies already fighting it; Sparta may go alone), and its visible army not outweighing our home defence; then the existing
  campaign and combined-strength checks. The threat stays top until its score falls 20 below
  60 (hysteresis), and a siege near our border counts as near even before its cities are known. Decisions are logged in
  `Brain::threat_log` (`ThreatEvent`: TopThreat, Walls, Treaty, SecondFront); the simulator
  records per-seat wars with captures and losses, treaties, Walls completions and
  `Elimination.by` (`crates/sim/src/openings.rs`, `runner.rs`; `scripts/threat_report.py`).

- `PlayerView.tiles` (`crates/core/src/view.rs::SharedTileViews`) shares the fogged tile vector
  across AI projection clones and copies it only on mutation, with the fogged terrain map cached
  alongside (`terrain_map`, used by `movement_path` and `Planner::movement_map`). `research`,
  `economy` and `CityView::detail` are copy-on-write too (`view::Shared`). These are storage and
  computation changes, not new AI decisions.
- **How much of a view is built** (`view::ViewDetail`): a human's view is `Full`. AI seats get
  `Decisions` (`wasm/src/session.rs::Session::ai_orders` on every target, and the simulator):
  the same values for everything a brain reads, without labelled yield sources, focus reasons,
  tech unlocks' growth forecasts, income-change previews, buy quotes, upkeep previews and
  population unlocks. The engine checks move orders against a `Map` view (tiles, units, cities,
  players, diplomacy). Viewer-independent parts (players, era scores, world wonders, standings) are
  built once per state (`public_view`, `player_view_with`). Before leaving another field out
  of `Decisions`, check that no brain reads it; some text is an input (the Byzantium planner parses
  the "Tech diffusion" line of tech costs).

- **ScriptedBrain** (`crates/ai/src/scripted.rs`): knob-weighted heuristic over a `PlayerView`
  (research → cities → civilians → army). **Every shipped difficulty runs on it** —
  `wasm/src/session.rs::make_brain` builds `ScriptedBrain::with_difficulty(..., difficulty)` for
  Easy/Normal/Hard, switching to `hard_v2::brain` only if `hard_v2_ai` is set, which the app never
  does (Hard v2 "made Hard easier for a strong player"). Since 2026-11-07 the app no longer builds
  `search::SearchBrain`: a Hard seat with `PlayerSetup::search` is a Training seat and plays plain
  Hard (see "AI levels" at the top; Hard + Search, below, is now a simulator and lab brain only,
  `legatum-sim --search`). App difficulty is `Normal`, not "Default" —
  `Default` is a value of the separate `Opening` enum (no opening policy, plain ScriptedBrain).
- **Difficulty** (`AiDifficulty::{Easy,Normal,Hard}`, `difficulty.rs`) is purely behavioral: Easy
  lowers expansion/military/aggression knobs plus a 20% seeded chance to silently drop a planned
  order; Hard only raises the military knob. No AI-only resource/yield/vision bonus. Hard-only
  effects elsewhere are timing, not cheating (pursues victory investments at 40% of game length vs.
  80%, gets `hard_defense` awareness, wants one extra standing-army unit at ≥3 cities).
- **Settler escort follow-through** (`hard_defense.rs::guard_exposed_settlers`,
  `civilians.rs::settler_trip_risk`): Hard prices visible route risk and requests a melee escort
  at risk 40 or higher. If no escort can cover a threatened arrival, it holds the Settler on its
  current tile when it cannot be captured this turn, even if that tile is already inside the
  conservative next-turn snipe radius. The risk gate remains separate from `peace_scout_danger`,
  so peaceful Scouts do not generally block settlement routes.
  Routine `escort_settlers` remains the non-hard-defense fallback.
- **Plain Hard's Settler guard reads more than this turn's sight** (AI decisions only, 2026-11-15,
  no new gene and no default changed; `hard_defense.rs::guard_exposed_settlers_duel,fog_wait_home`,
  `civilians.rs::recent_surprise_war`, `threat.rs::ThreatTracker::settler_snipers`; from the 8.9.40
  Byzantium review, idea #55). (1) A Settler held back for an unseen threat (a fog read from
  `snipe_read`, a rival city beside the route, an unseen flank near a rival border) waits only in
  one of our cities or on its escort's tile; on open ground it walks back to the nearest city and
  never waits alone. (2) An escort called to or beside a Settler is kept in
  `CampaignMemory::settler_escorts` (Settler to escort, saved only while non-empty) until the city
  is founded or the Settler is back in a city with no trip; a threat read of 0 does not release
  it, the escort walks to where the Settler stops, and a Settler whose escort cannot keep up
  waits (in its city or on the escort's tile) while the escort steps onto its tile. (3) A waiting
  Settler with no escort calls the nearest free troop that can reach it however far (it was
  capped at `escort_search_distance` 6). (4) A rival whose unit is, in our view, one of our former
  Settlers stays a known sniper for the rest of the game (the 20-turn `SNIPE_RECORD_TURNS` expiry
  applies only to a surprise war that took no Settler).
- **Opening policies/primitives** (`openings.rs`): `Opening::{Default, Rush, GreedyExpand,
  Snowball, Economic}` are opt-in, stateful, wrap a ScriptedBrain
  (`OpeningBrain`), with a phase machine (`Opening, Expand, Conquer, TurtleTech, Defend, Close`) and
  memory derived only from `PlayerView`s. Snowball and Economic escort settlers when threatened
  (`crates/ai/src/openings.rs:OpeningPolicy::builtin`). **Primitives** (`openings/primitives.rs`) are smaller
  composable behaviors an opening weights on/off (`escort_civilians`, `snipe_settlers`,
  `save_and_burst`, `weakest_target`, `coalition_march`, `siege_persistence`, `market_first_economy`).
- **hard_v2** (`hard_v2.rs`): draws one league-tuned opening per seed for a Hard seat —
  **opt-in only** via `hard_v2_ai`, never set by the app (validation found it easier for strong
  players). Its **hand-off** variant runs the tuned opening then reverts permanently to plain
  scripted Hard; simulator-only. The generated uniform table gives each of the five openings
  weight 1 (`crates/ai/src/tuned_openings.rs:TUNED`, `crates/ai/src/hard_v2.rs:draw_opening`).
- **Strategy biases** (`crates/ai/src/strategy.rs:Strategy::ALL`) are Rush, Wide, Science,
  Turtle, and Economic. The scripted planner no longer reserves growth investments for a
  compact-city plan (`crates/ai/src/investment.rs`, `crates/ai/src/production.rs`,
  `crates/ai/src/research.rs`). Age cards are scored from gains after the Age begins
  (`crates/core/src/objectives.rs::update`).
  Observed strategy telemetry uses classifier version 2 with Rush, Wide, Science,
  Turtle, Economic, and Generalist labels (`crates/sim/src/diversity.rs:CLASSIFIER_VERSION,Archetype::ACTIVE,classify`).
- **Value-based colonies** (`crates/ai/src/production.rs::choose_item`, `Tally`):
  normal scripted AI has no fixed city quota. A Settler competes with other production
  after checks for a reachable known site, nearby good sites, population, development,
  safety, army, pipeline, and enough time to repay the cost. The score discounts rising
  Settler cost and remaining game time. Hard can claim its first four sites before each
  colony develops; later Settlers come from cities with a Workshop and population or
  population at least four. At most two Settlers are in the pipeline (one for Normal
  below four cities). Opt-in strategy openings and Easy retain their target-city limits.
- **Expansion genes (2026-10-07, playtest 8.4; AI only, no rule change).** Hard's Settlers were
  boxed in more than they walked: `hard_defense` marked every tile a peaceful rival's unit could
  reach as Settler danger, Scouts included, and the commonest sentry or wanderer is a Scout.
  `peace_scout_danger` (0): a Scout of a rival at Peace no longer counts (at war it does; armed
  units at Peace still do). `keep_settler` (40): a Settler at least that percent built is finished,
  not swapped for an emergency unit, unless the city is forecast to fall or an armed enemy is
  within two hexes (`production.rs::plan_emergency_production`, mirroring `keep_walls`; it also
  holds in a lone city under `last_city_front`). `lone_city_radius` (4, was 8): how close a rival army
  at war must stand before a lone city builds no Settler. `peace_follows_plan` (1): a Hard + Search
  plan that chose Peace with a team accepts its offer, skipping the strike and containment-target
  vetoes (`diplomacy.rs::treaty_score`). `settler_pipeline` (2), `settler_open_cities` (4) and
  `settler_priority` (0, applied only at peace with every rival) are tuning genes, left at the old
  behaviour because they cost 1 to 2 points against Proxy v7 for a gain only after turn 75.
  Measured limits: even a forced Settler-first AI (all gates but pop, site and time ignored)
  reaches 2.9 cities at turn 50, because the capital makes a Settler about every ten turns and each
  costs a citizen; turn 75 and 100 are where expansion pays.
- **hard_defense** (`hard_defense.rs`): active only for plain Hard with no opening. Computes where a
  visible rival *could* move if it declared war this turn (even at Peace), to hold a defender back
  or escort an exposed settler — without seeing through fog.
  **Wartime Settlers (2026-10-07, `civilians.rs`):** while the empire is at war with anyone, a Settler never
  routes through, retreats through or picks a site in land owned by a team it fights (`enemy_war_tile`;
  military units may still enter it), never picks a site within `THREAT_RADIUS` of a visible enemy military
  unit (`enemy_army_near`; barbarians count), and no longer falls back to the highest-risk site: with no safe
  site it holds. Wartime routes near an enemy border go out as confirmed `MoveUnitPath` with stop-on-block, and
  retreats use `retreat_settler`; the reconciliation pass (`movement.rs::dedupe_first_steps`) keeps a replanned
  Settler out of enemy land too. At peace the paths are unchanged apart from the barbarian/enemy-army site filter.
  **Peace armies at settlement sites (2026-11-15, AI only; `peace_army_sites`, Hard default 1):**
  `Ctx::threat_units` still contains only wartime units, and a wartime army still blocks a site
  within three hexes. In `civilians.rs::choose_site` and `found_here_value`, a visible military
  unit of a non-allied empire at Peace instead discounts the site's value. The discount uses
  that empire's visible force within eight hexes against our nearby field units and a fresh
  city's defence. A visible patrol by its known city adds one sixth of its public army
  as a possible border force. Its public relative army strength, known city or border contact,
  and whether it fights another war scale the chance of attack. It is capped at 90% of the base site score,
  leaving the site available if its other value still wins. When that price erases a preferred
  site's edge, Hard compares its safe fallback instead of taking the preferred site anyway.
  Allied armies never count.
  The existing route and escort exposure reads already consider Peace units and are unchanged.
  Setting `peace_army_sites` to 0 restores the previous site valuations and orders;
  no core rule or `RULES_VERSION` changes (`analysis.rs::Ctx::build`, `civilians.rs`).
  **Search options c and y (2026-10-06, opt-in):** `c=N` searches (plan, overlay) composites:
  every plan with the current overlay, then each plan with the next overlay (Military, Expand,
  Economy, Goal, Victory, Build, Ranged, Melee) until N. Goal makes the faction-goal pass switch
  city production to its steps (`victory/goals.rs`); Victory runs the end-game title planner now
  (`victory.rs::pursuing_victory`); Build stops troops beyond the garrison in unthreatened cities.
  `y` re-plans when the win chance falls a third below the last search's
  (`plan.rs::search_decision`). Neither changes plain Hard or the default search.
  **Endgame push (2026-10-07, gene `endgame_push`, Hard):** a treasury that covers the whole Army
  title gap buys a unit a turn per city (`victory.rs::production_points`, past
  `military_purchase_available`); resting units leave ring 1 when a city has fewer than two
  spawn tiles (`military.rs::rest`); a held Swiss hoard bounds readiness and frontline buys
  while its points keep the team at target (`production.rs::live_goal_floor`); a racing team
  uses city growth and conquest to advance Age cards; one point short,
  rival cities whose capture scores a point join the campaign candidates (`plan.rs`).
  **Late capture value (2026-11-01, gene `late_capture_value`, Hard; AI only):** a feasible
  campaign scores the change in its own chance to complete a ten-turn VP hold and the rival's
  lost chance, including the discounted value of points still needed before a hold can start.
  This residual matters when the eventual-win softmax is already near 100% but the game has no
  winner. Celtic sacks count the live points they remove from a rival even though they do not
  transfer the city or its held wonder points. A duel also prices a feasible sequence of city
  captures ending in elimination, with capture odds compounded across the remaining rival
  cities. The same campaign price is used when a siege is reconsidered, and a one-point-short
  Celtic seat nominates rival cities whose sack removes live VP (`plan.rs::price_campaign`,
  `campaign.rs::campaign_value`, `victory.rs::sack_vp_denied`). All capture forecasts, combat,
  VP sources, target and hold rules remain the same.
  **Rush watch (2026-10-07, Hard, genes `sentry`, `rush_walls`):** next to a rush-capable or
  aggressive neighbour (`opponents::rush_capable`, `OpponentAssessment::aggressive_neighbor`) one
  Scout stands on the tile it may enter that is closest to that neighbour's nearest city
  (`civilians.rs::post_sentry`; not while the empire has under four cities and knows no good site, see "Explore before sentry"); and a rush-capable rival with a city within 12 hexes (or not yet
  placed, in lobbies of four or fewer, for the capital) counts as a border threat
  (`adaptation.rs::rush_walls_threat`), bringing the Walls tech, a shooter and Walls first.
  **Research items and the Sparta drill (2026-10-07, Hard genes):** `sentry` 2 posts one Scout per
  rival within 10 hexes (rushers first; default 1 = rushers only); `army_growth_trigger` (150%)
  makes a rival within 12 hexes a border threat before turn 40 when its public army (Strongest
  Army value) exceeds ours by that much (`adaptation.rs::rush_walls_threat`); `veteran_drill` (3)
  sends the Sparta's most experienced units below the retire level into favourable fights
  after the volleys, and with home quiet toward barbarians within 8 hexes
  (`military.rs::drill_veterans`).
  **City defence (2026-10-06, `duel_1v1_vs_proxy` 24.9% → 32.5%):** a rival army threatens only
  the own city it stands nearest to, plus any within `threat_aim_slack` (1) hexes of that
  distance, so a stack between two cities doesn't split their defenders
  (`hard_defense.rs::prepare_hard_defense`). A visible attacker near a known strategic deposit
  can also draw defenders to the city that works it, when losing that supply would displace
  valuable users; spare supply protects current users, while future demand can still make the
  deposit worth defending. A threatened city gathers defensive weight up to
  `defend_margin_percent` (200%) of the visible attack (`military.rs::defend_cities`). Own cities
  with a non-barbarian threat are *relief* cities (`Planner::relief`): healthy defenders within
  `relieve_radius` (3) of one don't retreat out of the attackers' reach
  (`tactics.rs::avoid_free_damage`), and a reserve that finds no safe tile beside the city closes
  in to within 3 of it instead of holding (`military.rs::defend`). All three are decision genes
  (`params.rs`); `--opponent-params` with `{"defend_margin_percent":100,"threat_aim_slack":5,
  "relieve_radius":0}` seats the pre-change Hard.
  **Playtest-8.2 behaviours (2026-10-06, Hard only, decision genes):** free field units within
  `open_city_radius` (6), or `contain_march_radius` (12) when the city belongs to our containment
  target, go for the most open enemy city at war (no soldier on or beside it, and either unwalled
  or at half HP or less; the runaway's first), when their attack weight beats the city's
  (`military.rs::seize_open_city`, after `defend_cities`). Units within 6 of an own city with an
  at-war enemy within 3 disband last for upkeep (`protect_defenders`, `funding.rs::defending`). A
  rush-capable neighbour by its public rules (`OpponentAssessment::aggressive_neighbor`: more
  moves, cheaper strength or infantry that ignore wounds) counts as a border threat before it
  attacks (`faction_threat_prior`, `adaptation.rs::assessed_border_threat`), so exposed cities
  take a shooter and Walls first. `--ai-params` and `--opponent-params` also accept inline JSON.
- **Hard unit concentration (2026-11-15, AI only):** `unit_concentration_turns` defaults to 10
  (`crates/ai/src/params.rs`). In routine production scoring, a city that would take more than
  ten turns to start a combat unit defers that kind when another eligible own city could finish
  it sooner (`production.rs::defer_slow_duplicate_unit`). Cities that can finish within ten
  turns may build the kind together. The gate does not penalize a unit already under way;
  emergency production,
  openings and purchases retain their existing priority. Zero disables the gate. The estimates
  and city choices come only from Hard's own `PlayerView`; combat and production rules do not change.
- **Terrain firing**: scripted combat forecasts and exposure checks use the same core LOS rule
  through `forecast.rs::forecast_attack_mode` and `los.rs::ranged_clear`, using only known terrain
  in `PlayerView`. Ranged staging seeks legal firing tiles, preferring Hills and then Forest cover
  among comparable safe positions (`crates/ai/src/military.rs::strike,campaign_staging_spot`,
  `crates/ai/src/city_assault.rs::coordinate_city_assaults`, `crates/ai/src/tactics.rs::ranged_unit_clear`).
  The ranged-garrison threat ranking also checks that a prospective enemy shot has LOS from
  visible terrain (`crates/ai/src/tactics.rs::enemy_threat`).

- **Hard conquest** (`conquest.rs`, `military.rs::campaign`, `production.rs::desired_army`): plain
  Hard selects one known, reachable city using observed defense, public VP, treasury Gold at risk, Swiss bounty,
  and India links. Celtic target value includes the Gold its sack would pay: population × 60 since 2026-11-06 (formerly × 30), the exact amount the forecast shows (`opportunity.rs::sack_gold_value`); Celts are no longer excluded from conquest targeting (`conquest.rs`). It funds home guards plus a mixed field force, stages before optional war,
  and preserves a local siege against routine peace offers. Gotos and observed positions/HP
  provide continuity; no hidden state or vision is added. Existing `city_assault` forecasts
  still decide whether the assembled force can enter city fire and capture. Germany's defensive
  policy remains separate; the Celts' razing is priced as a sack, below.
  Hard preflights siege movement, removes blocked participants, and repeats the capture
  forecast before committing the remaining group. Infantry can forecast a two-hex intermediate
  approach through rough terrain. Campaign production budgets queued upkeep; Switzerland
  complete their treasury goal before funding an optional expedition. Existing wars take
  precedence over staging against a neutral target (`city_assault.rs`, `movement.rs`, `conquest.rs`).
  Switzerland still banking for their treasury goal pick only a runaway's city: the public VP
  leader past 45% of its target (2026-10-04, The Courier King).
- **The Celtic sack, priced (2026-10-23, coach review of the user's 8.9.7 Celts win; AI only, no rule change; genes `sack_price` 1, `sack_commit` 1, `sack_loss_percent` 60).** Until now every city-attack path returned early for Celts, so the AI Celts sacked 10 cities in 2,048 seats on the 8.9.7 tag (the user sacked six in one game). The gates are deleted (the list above) and a capture by the Celts is priced as what it is, from public information only (`victory.rs::sacks`, `sack_vp_denied`, `plan.rs::price_campaign`, `opportunity.rs::strike`, `crowns.rs::objective_capture_value`):
  - **What the Celts gain:** the Gold the sack pays (population × 60 since 2026-11-06, previously × 30; `opportunity.rs::sack_gold_value`; before 2026-11-03 population × 25 plus a quarter of an estimated treasury), turned into the army it would buy (`plan.rs::gold_as_army`), and a "capture" objective if the city completes one. They gain **no** city, population, land, output, supply deposit, "four cities" or "20 population" milestone or title: none of `conquest_victory_value`'s keep terms, and no "Own N cities" or land count is credited by `objective_capture_value`.
  - **What the victim loses:** the city, its people, land and public output share (as for any capture), **and its VP**: the held faction goal the city props up (Swiss hoard, Germany Ruhr Foundries, the Heir, the Hall, a closed Japan's vow; `victory.rs::sack_vp_denied`). Age cards and milestones already scored are permanent and the progress a sack sets back on an unscored one is private to its owner, so neither counts.
  - **A strike's worth (Production):** a kept city is worth `city_keep_value` to its captor; a sacked one is worth `sack_loss_percent` of that (the victim's lost output) plus the Gold term; the race, hold and last-city terms (what the victim loses) are unchanged.
  - **`sack_commit`:** a new Celtic Contain war needs a funded siege route to a named city with at least `contain_min_odds` capture odds (the Sparta's `contain_commit`).
  - **Also lifted for the Celts:** `scout_front`, `diplomacy::border_raid_opportunity` and the "never sieges" test of `diplomatic_campaign` (Germany kept these gates until the 2026-11-05 rework), `peace_would_be_broken`, `second_front_temperament` (now 0). `war_siege` now includes the Celts; `siege_exempt` 1 restores the earlier exclusion for a comparison arm.
  - **Measured** (2026-10-23, master `de34c553a` as control; `docs/sim-results/ai-quality/celts-sack.md`): the 8.9.7 tag read Celts 2.4% (gates lifted 5.2%), but master had already moved to **8.3%** (the camp-count fix `e840acb4b`, 8.9.7 to now); balanced 8-empire Hard, 2,048 games: control 8.3%, sack priced 7.7%, gates lifted with keep pricing (`sack_price` 0) 6.5%, sack plus `sack_commit` 7.6%; on fresh seeds (9310001) control 7.3% against 8.1% (paired deltas -0.6 and +0.7 pp, each ±1.5: no change in win share). What changed is what the Celts do: sacks per seat 0.01 -> 2.0 to 2.7 (1,024 seats each, tape replay), wars declared 1.4 -> 1.1 to 1.9, era-objective VP 2.4 -> 2.8, Raise the Warbands held at the end 78% -> 67%, Gold taken about 300 a seat; seats with three or more sacks win 14 to 18%, seats with none 2%. Human-seat benchmark (8 maps x 128 salts, reseeded dice): every map without a Celtic seat is identical to the control (the changes touch only Celts), the Celtic seat's map 10.9% -> 20.3% (priced), 25.0% (keep pricing), +9.4 pp ±8.5; opponents-only is zero by construction (no Celtic rival on those maps). War suite 42% -> 42% (only `war-german-stand` 100% -> 96%).

- **The Celtic sack grows (2026-10-23, user decision; engine rule, AI pricing; `docs/sim-results/ai-quality/celts-rework.md`).** Besides the Gold, a sack raised a camp and (until 2026-11-03, when the steal was removed; `sack_tech_value` and `sack_take_value` are deleted, the Gold alone is `sack_gold_value`) stole a technology (§4 "Capture"), and the AI prices the camp: `victory.rs::sack_camp_vp` counts Raise the Warbands' 2 VP when the camp would be the last one a campless Celtic city needs (the goal not held, exactly one campless city of ours, a free tile in its land within the camp radius of the ruins); and `goal_sparta_wild.rs::wild_camp_denial` makes a rival go for a Celtic camp not only while the Celts score the goal but when the camps known in their land number at least their cities less one. The homeland damage and the hunters' concealment are deleted with their AI terms (the avoidance of Celtic land and cities in the other factions' targeting, `reveal_cost`, last-seen memory, the gene `prey_skips_celts`). Measured: league Celts 7.6% -> 7.0% over 4,096 games (paired -0.6 pp +-1.0), the Celtic-seat benchmark map 26.6% -> 15.6% (+-9), technologies stolen 0.29 a seat, camps raised 1.74 a seat, war suite 47% -> 47%.

- **Celts and the barbarians (2026-11-02, AI only; follows the rule in §7).** The Celtic AI no longer fears them: `analysis.rs` leaves barbarian units out of `threat` and the danger map (their tiles stay targets), `tactics.rs::enemy_threat` is 0 for them, and the Settler risk, snipe exposure, escort and city-under-threat checks (`civilians.rs`, `hard_defense.rs`, `goal_actions.rs`) treat barbarian units and camps as no risk, so a lone Celtic Settler walks past camps. Raising warbands stays valued by the VP window (`goal_actions.rs::wild_window`, unchanged). **A raid value for the call was tried and measured negative, so it is not shipped:** calling Raise the Warbands as soon as the camps would stand within 6 hexes of at least 2 rival cities (and keeping those camps uncleared) cut the Celts' league win rate from 9.0% to 3.7% (the same call only once expansion was done: 7.5%, with or without keeping the camps); camps cost the Celts their tiles' yields (a camp and the barbarian units around it deny them) and bring rival armies to clear them. A real value needs the denial cost priced first. Rival AIs clear camps near their own cities as before; `wild_camp_denial` still sends them farther for a camp of a Celts that scores the goal.
- **Contesting a hold with orders (2026-11-03, AI only, gene `contest_hold`, default 100; 8.9.25 Celts coach review, rec 1; `docs/sim-results/ai-quality/contest-hold-1103/`).** `crates/ai/src/contest.rs::contest_vp_sources` runs early in `military.rs::plan_military` (after the escorts, snipes, coalition march and barbarian guards, before `hold_captures`, door defence and every attack pass), Hard only. Its sources (`denial_sources`) are the known camps (`PlayerView::barbarian_camps`) in the land of a Celts at War with us whose public `vp_sources` carry its `FactionGoal`, while its team holds or is projected at its target and the goal's VP exceed its margin; each camp counts for the nearest known city of that Celts (territory is not in the view), and a city with more known camps splits the break chance. A removal is priced with the win estimate (`denial_gain`): the holder's own `WinFacts` chance now less its chance with its hold restarting the turn the force clears the camp (a re-call restores it, so the countdown resets), zero once the hold would complete first, times our plausible share (our estimate, at least an even share among the other teams with cities, as `plan.rs::leader_race`), the gene's percent and the break chance. The force (`denial_force`) is the free units at 50 HP or more that walk there within 8 turns, nearest first, up to 4: an unseen or empty camp takes the nearest unit; a seen guard takes units until their summed `combat::base_damage` kills it in time (it never heals), a melee unit must survive the reply, and a shooter-only force needs a turn to walk in. Each unit is charged its own job (`contest_cost`: the own city it alone keeps from falling this turn, `purchase_defense::city_risk_with`, or `army_commit.rs::capture_now_value`), and the best net removal per holding team goes ahead, a removal already under way (a unit's `goto` at the camp) first. Orders: walk onto an empty camp; at a guarded one shooters fire from range and melee units attack when the forecast wins or risks at most 30% (added to `assault_attacks`), else close in beside it; the units are exempt from `focus.rs::hold_stragglers` (`Planner::contest_units`). City-borne VP (Great Works, goal buildings, capitals) stay with the capture valuations (`victory.rs::capture_goal_swing`, `hold.rs::hold_city_points`) and a visible Heir with the attack passes. At Peace no armed unit can enter Celtic land, so only empires at War contest. The older late `barbarians.rs::clear_barbarian_camps` denial (`wild_camp_denial`, 12 hexes, 8 steps) is unchanged and takes what this pass leaves. War scenario `war-celts-wild-hold-t150` uses the new scenario condition `scenario::Condition::StopWin { rival }` ("Stop X winning": met unless that rival's team has won; as a keep condition it fails the moment it wins).
- **The Celts play their kit (2026-11-05, AI only, gene `celts_playbook`, a bit mask, Celts only, shared default 0, Hard Celts play 2 (the Wild fixes) via `BUILTIN_FACTION_DECISIONS` since 2026-11-05; gene `celts_wild_late`, default 70; branch `claude/celts-playbook`; `docs/sim-results/ai-quality/celts-playbook-1105/`).** From the designer's 8.9.25 win. Part 2, the Wild (`goal_actions.rs::wild_window`, `wild_now`): the call fires whenever a city lacks a camp and every city would hold one afterwards (`with_camp + sites >= cities`; the shared test asks a site of every city, but the core previews sites only for camp-less cities, so with gene 0 the AI never re-calls once any camp stands, or when a natural camp sits in one city's land); the window opens when the 2 VP leave at most one VP to the target with time to hold (`WILD_PLAYBOOK_SLACK`), or at `celts_wild_late` percent of the game; settling continues after a call except while the goal decides a hold (`goal_sparta_wild.rs::wild_allows_settling`); the Celts keep every camp in their land (`barbarians.rs::clear_barbarian_camps`) and `celts_playbook.rs::spare_own_camp_guards` drops those camps' guards from `Ctx::enemy_units`, so no attack pass takes them (with gene 0 the attack passes kill the 10-HP guards and walk in: the AI Celts hold Raise the Warbands 0.9 turns a game). Part 1, light garrisons (`celts_playbook.rs::sheltered_city`): a city with no threat, no city of an empire at War within 8 reachable hexes and no foreign military unit seen within 6 (barbarians aside; never the capital, a Wonder, Hall or Heir city) keeps no standing garrison (`military.rs::garrison_cities`) and leaves the garrison quotas (`garrisoned_city_count` in the production minimum, ranged quota and Hard peace army quota). Part 4, soft targets (`celtic_soft_target`): no Walls +60, no garrison seen +60, and up to +120 for an owner whose public army (Strongest Army values) is weaker than ours, in the conquest nominator and as a new-war term in `plan.rs::price_campaign`. Measured on 4,096 paired 8-empire games per seed: part 2 alone takes the AI Celts from 8.6% to 11.4% (+2.8 pp [+1.9, +3.6]; +2.9 on a second seed); parts 1 and 4 add nothing measurable; calling as soon as the points drew no lead, or at 45% of the game, measured worse. The rite places its own camps (farthest free tile), so the AI cannot aim them at a rival border.
- **Opportunistic strikes (every scripted difficulty that isn't legacy diplomacy, 2026-10-04,
  `opportunity.rs`)**: each turn the AI forecasts taking each rival city with the healthy units
  (50+ HP, not standing in our cities) that reach an assault tile within 2 turns, playing
  rounds of enemy fire (from round 1, including the approach) and then the volleys and melee
  of units already in position (4 rounds after the slowest arrives), through the engine's own
  attack forecasts; a
  fogged city is assumed to hold one fortified Spearman (PlayerView only). Gain = city value
  (`city_keep_value`) plus a second city's worth for a rival's last city, up to one more as
  the target, the public VP leader, goes from 30% to 55% of its target, bounty, title flips and conquest
  VP, times eagerness (60 + aggression knob + identity: Sparta +30, Mongols +25,
  Germany −20, Celts +10 for sack Gold, others 0). Cost = expected Production lost
  (dead units at full cost, damage at half its share since it heals), the larger of retaliation
  on our cities from their units within 6 hexes and a stronger enemy army (its strength divided
  among its wars with living rivals; both are the same army answering), our other wars, and the
  fading value of a recent peace with the target (15 Standard turns).
  Damage follows the city shield (`analysis::damage_tile_defender`: the city first, the
  garrison only once it is at 0 HP). For a Hard Sparta assault on a prized walled city,
  the forecast splits one melee counterattack among same-round attackers
  (`opportunity.rs::forecast_siege`), matching the combat rule. A strike needs gain > cost
  and gain ≥ 125% of cost (115%
  when the city is visible and the whole force is at most a turn from its tiles); it is then
  enough on its own for an optional (non-breach) declaration, alongside the older campaign score and
  `conquest_ready`'s two-ranged-two-melee recipe (`diplomacy.rs`), keeps peace and treaties
  with the target off the table unless we're endangered, and `military.rs::press_strike` moves the force the turn
  war is declared (Archers shoot the city tile, melee attacks through the city-assault
  forecast; where cities don't fire, a melee unit that can hit the city this turn with under 25%
  loss chance attacks, except a Mongols courier against a garrisoned city far from home). `dedupe_first_steps` treats borders as open after a same-turn declaration, as
  resolution does. The strike forecast gives a newly declared enemy no retroactive unit attack
  in the declaration round; automatic city fire still applies (`opportunity.rs::forecast_siege`).
  Tests: `crates/ai/tests/attack_opportunism.rs` (The Last Bridge, The Courier King).
- A profitable local strike can bypass the global 80% coalition army ratio, since its forecast
  already prices city fire, unit losses and retaliation. Broad second fronts still use the
  coalition and affordability tests. At war, Hard refuses routine peace while a visible,
  badly damaged enemy city near home can be retaken by a healthy local force. Incoming peace
  proposals use the same campaign assessment as outgoing offers. A nearby active siege, or a
  newly captured damaged city in resistance, also suppresses Settler production until the
  front is resolved. A lone surviving city treats visible or recently observed wartime
  military within eight hexes as a local front, so fog after another city falls does not
  turn its next build into a Settler (`production.rs::last_city_front`).
- **Holding captures** (`military.rs::hold_captures`): a city in resistance, or damaged with
  enemies about, keeps a defender (a ranged unit that reaches it this turn first) before
  campaigns march (gene `capture_hold`, on by default since 2026-10-31: only when the visible attack would retake it
  this turn, otherwise from the units the attack passes leave free; §10 "Army commitment"); the Hard conquest planner won't stage against a neutral city while a war
  near home is unresolved (`active_front`); `scout_front` sends one idle unit to find a
  neighbour's cities when its land is seen near home but none of its nearby cities is known.
  `retake_weakened_city` sends a local force against a visible, badly damaged city at war,
  subject to the shared assault forecast. During a bombardment, a survivable melee unit
  prefers an adjacent tile to stop the city's 20-HP end-turn regeneration. When the top threat
  has no currently visible city, `scout_threat_front` sends one spare field unit to refresh a
  remembered city on that front using only the AI's own fogged view.
- **Human-playbook proxies** (`proxy.rs`, `proxy_v5.rs`, `proxy_v6.rs`, `proxy_v7.rs`, `proxy_v8_sparta.rs`, `proxy_v8_mongols.rs`): fixed, non-knob benchmark
  brains, not a player difficulty, used to validate balance. v2: expand to 7 cities, then ranged+melee
  conquest. **v5** (forked from frozen v2): 3 cities, funded ranged campaign, then science/defense.
  **v6** (forked from frozen v5): adds economic "consolidation" (reinvest captured-city treasury)
  and a split tactics module. **v7** (forked from frozen v6, `proxy_v7.rs`, `--proxy-version 7`,
  `--duel-opponent proxy7`): the "conqueror" from playtest 8.2. Spearmen from turn 2 (one Settler
  wave only after two Spearmen exist; Sparta keep one Archer, others an Archer per two Spearmen),
  Settler snipes (declare war beside an unescorted Settler and take it the same turn), staging 2-3
  hexes from a city at Peace then declaring war and striking in the same turn (the planner runs a
  second pass on a view with the war applied), assaults on defended cities with 3+ Spearmen
  (more per visible defender; walled cities also need Archers), no consolidation (after a capture
  the next nearest or weakest city is the target), peace offered only to a rival whose war stalled
  (16 idle turns) so the army can switch fronts, one guard per threatened home city,
  and Sparta fight wounded (kills and captures heal). None is wired into the app's brain selection.
  The playtest-8.4 **Sparta** fork (`HumanSpartaBrain`, `--duel-opponent proxy8o`)
  suppresses early settlement, seeks at least four infantry before marching at T20, and uses
  an early Scout sweep to find the nearest neighbour, then searches the next corridor after
  committing to that war. From T20–60 it may declare with four
  infantry against an unwalled city within ten hexes of home, even before the assault stack is
  staged. Its Spearman target is five before T40, then seven plus two per city, and it defers
  Archers until T40 and researches Feudalism before Currency and Bronze Working. An early
  Wall aborts a weak assault; a later seasoned army can gather outside one and attacks only
  when the local forecast is favorable. Wounded Spearmen rotate home, and forward cities
  favor War Shrines after the first conquests.
  The playtest-8.5 **Mongols** fork (`HumanMongolsBrain`, `--duel-opponent proxy8g`)
  attempts one early mobile campaign or an exposed Settler capture, with one Scout watching a
  known rival's early settlement corridor. It queues Quarry and Granary at T2, then Farm and
  five Settlers from T12, and pivots to settlement from T35 or after a capture. It favors up
  to three concurrent Settlers and nine cities, keeps a one-unit baseline army until the
  late title race, starts capital Walls with Production when a rival army approaches, accepts offered peace after
  a raid, and favors a Courier Hall in its first expansion city from T60. Both forks still read only
  `PlayerView`; neither changes default AI or game rules. The four-player comparison
  is in `crates/sim/examples/human_proxy_duel.rs` and `docs/sim-results/human-proxies/README.md`.
- **Victory goals** (`victory.rs`, `victory/goals.rs`): compares endgame investments by public
  VP-per-turn from the AI's fogged view; `goals.rs` builds per-faction plans (e.g. Mongols's
  3-site Market trio), recomputed every turn.
  **Goal prerequisites (2026-10-06, playtest 8.3).** The paid goal pass (`goals.rs::plan_faction_goal`)
  never skips a goal item for something it could start itself: `goal_need` walks the item's
  requirement chain in that city and starts the first startable piece with the goal's priority
  (an idle city takes it before ordinary scoring; a legal purchase is bought like any goal item):
  the **Workshop under each Mongols Market** (the AI's own foundation rule, not an engine rule;
  pursued only once Currency is known and all three trio cities stand, and the Market that
  completes the trio is exempt; earlier it takes idle cities from Settlers, Granaries and Walls and
  cost 1.9 points against the proxy), a lower building tier (`data::building_predecessor`),
  then the unlocking technology. Before this, a Mongols city without a Workshop was skipped
  forever, because the ordinary scorer ranked a Workshop (about 1,295) under a Spearman (2,400):
  the AI built no Market in 91 turns of the 8.3 game and lost 10 to 3. A missing technology is
  research's job: `goal_techs` (Mongols: Currency and Bronze Working;
  Athens no longer has a gated item: its goal is the broadcast) get `faction_goal_vp x 150` research value
  (`research.rs::goal_tech_priority`) and lead the opening research plan
  (`research.rs::plan_research`). Audited and unchanged: Switzerland (Gold), Japan (Sakoku, an action),
  Byzantium (Science; since replaced by the Heir), Celts (Granary, no tech; since replaced by the warbands), Germany (four techs known by the end in 98%+ of
  8-player games; its misses are population) and Sparta (Masonry and Feudalism known
  in 99%+; misses are war). The goal Settler step still skips a city with no Workshop. Per-faction
  goal rate and median goal turn are in every sim report (`stats.rs::GoalProgress`, "Faction
  goal progress") and `tools/analysis/goal_rates.py`.
- **Scouts and the AI (2026-10-06):** Scouts are free, so the AI never disbands one for upkeep;
  a Scout with no frontier left holds (`civilians.rs::plan_scouts`).
- **Ranged fire (every scripted difficulty, 2026-10-03)**: targets are ranked by value, not by
  unit type (`military.rs::best_shot`). A unit shot is worth the threat it removes:
  `tactics.rs::enemy_threat` is the target's best expected hit on one of our units or cities in
  its reach next turn, plus the city's full HP (200) when its melee attack could capture that
  city; `shot_value` multiplies it by `removed_share` (the kill chance plus half the surviving
  damage share). Both follow the city shield: a unit in our city that its HP fully shields
  (`city_shield == Full`) is not harm, one in a city that may break counts melee only, civilians
  in a city are never harm, and the capture stake applies only to an ungarrisoned city, or when
  the melee hit would kill a garrison its city no longer shields (0 HP, or `city_shield ==
  MeleeOnly`: this turn's fire could break the city before the movement rounds; melee against a
  fully shielded garrisoned city only hits the city). A garrison on an enemy city tile with HP is never a unit target
  (`city_attack_target == City`, or the forecast's `garrison.shielded`): the shot is scored as
  city damage, `tactics.rs::city_shot_value` = `removed_share` of the city's remaining HP x 200
  when one of our melee units is within its move range of the city, else x 1. Committed damage is
  tracked per tile against the city's HP while it has HP, including overflow (a city with no HP
  left after this phase's committed shots draws no more). Any `SkipUnit` (the AI's one stay-put order) a ranged unit
  received from any phase (garrison, defense, rest) becomes a shot when a visible target is in range and passes the shared terrain LOS check (which a unit in its own walled city bypasses)
  (`los::ranged_clear_with_tech` via the forecast). Enemy ranged threat and city-shield pressure also
  require this shared LOS check through known terrain
  (`military.rs::fire_from_holds`, end of `plan_military`): shooting never moves the unit and
  draws no retaliation. Playtest-6 bug: the early Mongols garrison pass's hold used the
  garrison's one order (`crates/ai/tests/ranged_garrison.rs`).
- **Production with banking (2026-10-03)**: routine choices run only for an idle city, and
  every option's `turns` already counts that item's own bank plus unassigned Production; a
  banked item also scores up to +50% by its banked share, so started work gets finished
  (`production.rs::choose_item`). A population-1 city that wants a Settler builds a short item
  first: a Spearman when exposed and unguarded, else the quicker of a Granary
  or (only if it has none) a Scout, with a Spearman only as a last resort, which avoids extra unit
  upkeep (3 Gold for Switzerland)
  (`production.rs::pre_settler_build`). When such a Scout or Granary has more than a turn left,
  the city reaches population 2 and the planner would choose a Settler there, it switches to the
  Settler; the filler resumes from its bank later (`production.rs::start_settler_on_growth`).
  Emergency production and the Rush Market switch no longer get the old item's progress
  (`plan_emergency_production`, `fund_rush_army`); investment estimates use each step's own
  progress (`investment.rs`). Items useless to the faction never appear in
  the AI's options (`data::available_to_faction`).
- **Persistent war campaign** (`crates/ai/src/campaign.rs`, `scripted.rs`, `openings.rs`):
  a scripted seat retains one known target city, start turn, committed unit IDs, the first
  negative-Gold turn from its current economy forecast, expected capture and denial VP less
  forecast unit losses, and explicit end reasons in `OpponentMemory`. Visible city commitments
  use the same combat rollout as opportunistic strikes; the treasury clock compares its
  capture turn with the first insolvent turn. A campaign ends on
  capture, peace, a lost target, negative value, insufficient observed army, Walls before arrival,
  or its treasury deadline. A newly captured city keeps its committed holders while resistance
  or serious damage and war persist. Diplomacy refuses routine peace while a positive campaign
  is active or a visible local capture still forecasts positive value before the treasury deadline;
  an explicit withdrawal can propose peace. A peace-time strike is not a campaign
  until war actually begins, and a failed campaign waits for a material improvement in its
  force, target or treasury forecast before retrying. If a tactical pass starts a forecastable
  wartime city capture outside the nominated plan, the AI adopts that actual siege so it must carry
  the objective or record an abort on the next turn. Military staging retains the target,
  and an aborted army receives withdrawal orders. This is AI memory; core rules do not change.
  On Hard the goal-driven plan (below) owns this: campaigns start only for the plan's target,
  an adopted siege must be against the plan's enemy, and a campaign the plan drops ends with
  `plan_changed`.
  Hard Sparta can commit the entire reachable, forecasted force to a valuable walled city
  (capital, last city, title or conquest VP swing, or population 3+ as a public proxy for
  compounding yields). A newly finished Wall ends that campaign before arrival only when the
  updated siege forecast cannot capture it; otherwise it can accept the forecast losses and
  continue with the force reachable inside the six-turn march window. Value, funding and
  continuation use that same forecast, including expected losses and travel time
  (`campaign.rs::sparta_walled_prize,campaign_force,reconcile_campaign`,
  `opportunity.rs::campaign_siege_estimate`). The capture forecast and treasury deadline
  must still pass.
- **Upkeep budget / deliberate disband**: campaign planning first changes production toward
  Gold income or away from a nonessential troop completing before the treasury deadline.
  It can cut an uncommitted unit early when the current runway cannot cover both the forecast
  capture and the captured city's resistance. Routine purchases pause while a campaign runs
  a deficit (`production.rs::plan_buy`).
- **Reserve ledger (2026-10-04, `crates/ai/src/funding.rs`)**: every scripted seat computes its
  Gold runway from its own `PlayerView` economy (Gold ÷ −net Gold per turn; none while income
  covers upkeep) before any strategic commitment. The floor is 4 turns. A campaign's deadline is
  the end of that runway. Below the floor:
  a campaign continues only if its forecast capture lands inside the runway (assault now);
  otherwise it ends with `deadline`, its army withdraws and Peace values the army's healing
  (the pre-audit baseline scored +100). No campaign
  starts or adopts a siege on a short runway unless its capture comes first. After its
  production and purchases each turn, `plan_cities` closes the ledger: a non-front city
  (no threat, no resistance, no at-war army or city within 5) does not start or continue a
  troop whose upkeep leaves the runway under the floor unless the campaign's capture comes
  first, switching to its best-Gold non-unit option; on a short runway it also takes any
  option that raises Gold (a Gold project, Germany only: `production.rs::gold_swap_allows`; Egypt's Grow stays allowed). Optional purchases that would leave the
  runway short are withdrawn newest first (emergency and threat-defense purchases stand),
  and unit upgrades need the runway too. **Hard troop funding (`troop_runway`, default 10 turns,
  2026-10-11, `production.rs::troop_unfunded`):** away from the front a Hard
  city does not start a troop (any production path, including the Sparta and rush openings)
  whose unit upkeep leaves the runway under that floor; a troop not yet begun is paused by the same test
  in the ledger. Before it, the 8.4 Sparta seat trained a Spearman, disbanded it for upkeep and
  trained another for 80 turns without founding a city. If the runway is still short, the AI cuts its least
  valuable field units that are not at the front (more than 4 hexes from any at-war city),
  not committed and not garrisons. When the forecast still says this turn ends bankrupt,
  `settle_insolvency` covers the whole deficit itself with its least valuable units, far
  from the front first and committed units last; it never leaves a structural deficit to the
  engine, which would take the costliest unit and repeat that every turn. Unit value is cost
  times health plus the city it stands in (the whole city if threatened, damaged, in
  resistance or with enemies within 6 hexes; a quarter otherwise), plus half again near an
  enemy city. The core bankruptcy rule (`rules::cities::pay_gold`) is unchanged
  (`crates/ai/src/production.rs::settle_insolvency`).
- **Goal-driven plans, Hard only (2026-10-04, `crates/ai/src/plan.rs`, `winchance.rs`)**:
  every turn, after reconciling its campaign, a Hard scripted seat prices its strategic options
  in one currency, its estimated chance to win (an integer 0–10,000 from its own `PlayerView`:
  a conditional logit over teams' VP progress and public title shares plus its own runway
  shortfall, weights fitted on sim games, `winchance::WEIGHTS`). Options: Economy (at war:
  Peace with its enemies) worth 0; Campaign against one city (the
  siege forecast's odds, calibrated, times the capture's city, people, land, public
  Production-plus-Gold economy share and VP denied to the owner, against a failed siege,
  retaliation at home, forgone growth and the
  aggressor's risks for a new war; a march from up to 6 turns away uses the same rollout
  from its assault tiles and loses a tenth of the odds per turn on the road, and opens no
  second front); Contain a rival (the coalition or threat target) priced by the VP the war denies it, not by captures; Snipe an exposed rival
  Settler. The plan persists in `CampaignMemory::plan` (saved with the opponent memory); a
  challenger must beat it by 40 (80 before its review turn). An Sparta Campaign may keep
  marching for up to `campaign_march_patience` (6 Standard turns) through a missing
  prospective forecast while its known city remains unwalled and two healthy, reachable
  troops including infantry remain; other war plans survive one missed turn. It ends early
  only for a recorded reason (captured, target
  gone, peace signed, campaign ended, Settler taken or missed, declaration illegal,
  infeasible). On a runway under the floor no campaign is offered unless its capture comes
  first, and no new containment war. While a plan routes: `plan_diplomacy` declares only
  the war the plan names; every other war is one it offers and accepts peace for
  (`wants_peace`); new campaigns
  start only from the plan; the plan's Gold reserve withdraws discretionary purchases after
  the reserve ledger (`veto_plan_reserve`). Every switch (with its reason, value, chance
  and main terms) and every blocked purchase is logged (`Brain::plan_events`,
  `SeatRecord::plan_events`). Normal and Easy plan routing is unchanged. There is no
  external decision-maker seat: the LLM seat and its directives were removed (2026-10-06). The
  read-only forecast helpers `campaign_outlook` and `campaign_against` in `plan.rs` also
  gate Sparta Contain declarations and their march targets. An Sparta selected Campaign's city
  guides staging before a local siege can begin (`military.rs::campaign`). Its
  early Campaign on an unwalled city stages at peace and delays its declaration until a
  funded local siege forecast is ready, so the march need not warn the defender into
  building Walls. Normal siege and funding gates still decide whether to assault.
  A prized walled city can also be offered to Hard Sparta and Hard + Search when the
  existing siege rollout with its actual force predicts capture; an all-melee expedition is
  valid (`plan.rs::campaign_forecast`, `conquest.rs::conquest_ready`,
  `military.rs::campaign_target_for`). The early restraint on starting a war against intact
  Walls still applies (`conquest.rs::choose_conquest_city`,
  `adaptation.rs::assessed_campaign_allowed`); a walled target in an existing war may be
  finished when its forecast remains viable. Walls' costs, strength, fire, and capture rules
  are unchanged.
  On the archived 8.5 T14 army-edge state, 32 paired future seeds gave declarations before
  Walls 16 to 31 and captures 2 to 31; at T100 mean yields rose 120.6 to 206.8 and cities
  3.2 to 5.6. A separate 32-map cohort had visible unwalled neighbors but no early
  declarations under the broader opportunity definition, so this evidence applies to the
  archived favorable force and position rather than every open-city sighting.
- **Hard+Search (2026-10-06, `crates/ai/src/search/`)**: a Hard seat whose plan review points
  are decided by rollouts on the real engine. **Not in the app since 2026-11-07:** the lobby
  level "Hard + Search (experimental)" was replaced by Training (top of this file), and
  `make_brain` no longer builds a `SearchBrain`; it stays in the simulator and lab
  (`legatum-sim --search`, the examples). It was a Hard AI seat with `PlayerSetup::search`
  (omitted when false, so saves, tapes and hashes are unchanged; the flag now means Training;
  the in-app budget below, `SearchConfig::in_app(player count)`, is history). The in-app budget depends on the
  player count only, never on the platform, so native, WASM and online play the same game:
  2 players k3 h20 d4 w, 3-4 players k3 h20 d2 w, 5-8 players k3 h10 d2 w, all with `z`
  (stagger: the first plan's review is delayed by the seat number modulo the review interval so
  seats don't all search on one turn). The WASM engine is single-threaded (rayon rollouts are
  native-only), which is why the budget shrinks with seats. Simulator:
  `legatum-sim --search k=3,h=20,d=2[,o,oh=..,v=..,t,z,omniscient]` (`--search-rotation` for one
  seat per game, `--duel-matrix` with `--players 2` for the 1v1 schedule). Plain Hard is
  unchanged (its tapes and pins are identical).
  - *When it searches* (`plan.rs::search_decision`): the first plan, a plan that ended (its
    abort reason), the plan's review turn (every 10 Standard turns), an analytic challenger the
    last search didn't already weigh, and with `t` an information trigger (`plan::Signals`: a
    rival newly met, 2+ more visible rival military units within 6 hexes of our cities, a
    rival losing a city, a new war anywhere). Between those the plan stands; the analytic
    margin switch is off. A search with only one candidate is skipped.
  - *What it compares*: the running plan plus the top K candidates by analytic value (the plan
    plain Hard would pick is always among them). Each is played H turns forward (`h=end`: to
    the end of the game, capped at 400 Standard turns), D times, from D states determinized
    from the seat's own `PlayerView` (`search/determinize.rs`). In a rollout every seat is the
    scripted Hard brain (with `v=2..4`, rivals cycle through Hard, a full-game Rush, a
    GreedyExpand and a Turtle-biased Hard by sample); this seat is a copy of its own brain
    (same memory, no search) with the candidate forced (`PlanMemory::forced`) until it ends
    for its own reason or the horizon passes. Score: the seat's `winchance` at the end (a won
    game 10,000, a lost one 0), averaged over the samples; ties keep the running plan.
    The D samples are shared by every candidate (common random numbers); every seed derives
    from the brain seed, so a game replays exactly. Rollouts run in parallel with rayon
    (feature `parallel-search`, native only).
  - *Openings* (`o`, `search::SearchBrain`): on its first turn the seat rolls out every opening
    policy (Default, Rush, GreedyExpand, Snowball, Economic; each hands off to Hard when done,
    as in the Hard v2 hand-off) for `oh` turns and plays the best.
  - *Determinization (no cheating)*: own empire exact (treasury, research, cities with
    production and banks, units with orders); every explored tile, seen or remembered city and
    unit as known; public diplomacy, era scores and world wonders. Unknowns are sampled:
    unexplored terrain, ruins and resources from a map generated with the sample seed; unseen
    rival cities (public city count) on unseen land near what is known of their owner, sized
    from observed cities; a surviving rival with no city after losing it gets one plausible hidden Settler near its last known position or sampled start; remembered sightings stay where last seen and the
    rest of each rival's public army title is filled with its known unit kinds at its cities;
    rival Gold scales ours by the public economy value; rival technology is approximated from
    our known technologies; rival cities hold only the Walls and
    Castles we've seen (so rival economies are underestimated: about 20-40% under the public
    economy title mid-game). `omniscient` rolls out from the true state instead: a flagged
    diagnostic upper bound only.
  - Events: `Brain::search_events` (`SearchEvent`: turn, trigger, horizon, every option with
    its analytic value, mean and per-sample horizon chance, chosen and analytic pick),
    recorded in `SeatRecord::search_events`; decision time per turn in
    `SeatRecord::decision_ms` for games with a search seat. Results and the hill-climb ledger:
    `docs/sim-results/ai-quality/`.
  - Measured (1v1, 2026-10-06): the base `k=3,h=20,d=2,w` beats plain Hard by +3.1 points
    (paired, 512 games). h10 +2.3, h40 +4.1, h80 +3.7 (before the Byzantium fix). No option on
    top helped: `t` +0.2, `e` −0.9, `p` +0.4, `v=3` −0.3, `d=4` −1.0, `h=end` −1.6, `o` −3.5,
    omniscient +0.1.
- **Byzantine AI: Mercenary Companies and thin levies (2026-10-22, `ai/src/company.rs`, `levy.rs`).** Replaces the tribute AI (deleted with bought peace).
  - *Hiring* (`Planner::plan_company`, after `plan_cities`): only when at War with an offensive (conquest city, containment march or active campaign) or a threatened city; the want is `uncapped_army` less the armed units it has (Companies included), at most 2 hires a turn. Each offer in `PlayerView.company_offers` scores unit strength per Gold over the whole contract (`price + UPKEEP x COMPANY_TURNS`) plus the class-triangle value (`composition_bonus`), less a penalty for enriching a seller ahead of it in VP (40 per VP, 120 more for the leader). It hires in the most threatened city, else the nearest the target, never in a city that bought this turn, only from Gold above the reserve and with at least `runway_floor` turns of runway after the new upkeep.
  - *Thin levies* (`levy.rs`): `desired_army` is clamped to the public cap; a city producing an armed unit at the cap is re-chosen (progress banks); in quiet peace at the cap it disbands the weakest unit (never a Company) that has no upgrade offer when a clearly stronger (150%) unit it can train waits behind the cap.
  - *Rivals*: capture odds for each armed unit beyond a Byzantium's public cap (`levy_edge_odds`, 4 points each, at most 4 units). Rivals do not yet model a Company's expiry or "shutting its shop" (a follow-up).
- **Wars between allies in the AI (2026-10-23; replaces "shared wars in the AI", 2026-10-18).** An Alliance is scored by what it does to our wars (`DiplomaticAction.new_wars` of the offer or acceptance, `alliance_war_value`): a war the new ally's partner defends that we would join at once costs `join_war_value` (-20, +40 if the enemy is the diplomatic target, +60 if it is counting down to victory, half the clamped army edge of the combined side over the enemy's side, -20 per other war, -30 endangered, -80 when the combined side is under 80% of the enemy's side or no army can reach it, unless it is counting down, minus `inherited_front_cost`), and each ally who would join one of our wars is worth `10 + min(20, edge/5)` (+10 when endangered, scaled by `helper_reach_percent`). Wars the partner declared are calls, not costs: the offer's explanation says "Wars we would be asked to join (we may decline)". The AI answers a call with `call_value` (join at 0 or more). Declarations compare committed participants (`participating_strength`) in `plan_diplomacy` and `plan.rs::contain_candidates`: direct allies defend automatically, while offensive calls use `ally_offensive_join_percent` to weight likely joiners. Incoming offers are the proposals whose `needs` include the team, answered against the other named team (`DiplomacyView::counterparty`); an ally's separate Peace requires no answer from us. `coalition_disengage` lets go of a war an ally brought it into (`joined_war`) once the target falls behind. The AI may make a separate Peace while staying allied; leaving an Alliance still follows its existing exit conditions. The human-proxy benchmark opponents (`proxy_v4`..`v8`) answer calls with the proxy rule (target at 80% progress, or an affordable reachable campaign).
- **Peace flips (2026-10-17, playtest 8.9.2; `peace_credibility` on by default for Hard, `contain_after_campaign` off):** a Hard AI that proposes Peace and declares on the same empire within 10 turns happens about 5.5 times per game. Cause: after a Campaign ends, the plan falls back to the best other candidate, and the Contain war on the same enemy is not on offer unless that enemy is still the diplomatic target (`plan.rs::contain_candidates`; the "war in progress stays on offer" clause covers only a running Contain plan). Peace is priced 0 (`quiet_candidate`), so it wins and `diplomacy.rs::wants_peace` offers it. Gene `contain_after_campaign` keeps a running Campaign's enemy on offer; gene `peace_credibility` is on by default for Hard (weight 300): it charges a new Contain or Campaign on a team we made Peace with after a war, falling to 0 over `scale_cost(15)` turns, never against a team counting down to victory (`plan.rs::peace_credibility_term`).
- **Sparta 8.9.10 war audit (2026-10-25; AI preferences, no combat-rule change):** `raid_arrival_weight` defaults to 100 after the gate (range 0–300). Positive values make a field raid count only at least two units that can reach distinct known firing/screen tiles this turn, subtract weighted forecast reply losses, and reserve their coordinated movement; 100 fully prices the reply. Zero restores the prior six-hex nominal-force raid. `founding_survival_weight` stays opt-in at default 0 (range 0–400): a Settler at war discounts a city's site value by forecast ranged HP damage before Walls, from visible enemies, known traversable routes and terrain-aware line of sight; when no site beats preserving the Settler, it returns to a friendly city. Historical fogged fixtures cover Japan's T142 isolated Phalangite and T129 exposed Kyoto (`docs/sim-results/ai-quality/sparta-game-1025.md`); the paired gate and its uncertainty are in `docs/sim-results/ai-quality/gate-genes-1025/README.md`.
- **Rush conversion and kept Walls (2026-10-17, playtest 8.9.2 Sparta; AI only, no rule change).** On by default for Hard: `value_first_target` 1 (`conquest.rs::choose_conquest_city`, `public_softness`): the conquest target is the best value-for-cost rival city in reach, whatever its faction; public softness adds +120 for an owner's last city, +60 for a levy-capped owner (`PlayerPublic.levy`), +60 for a city seen with no garrison, +30 for each other war the owner fights (up to 3), and the owner's visible army 4 to 8 hexes out as a cost. For Sparta it replaces the one-city Japan/Byzantium filter; the `military.rs` sorties, the `production.rs` assault-wing prizes, `campaign.rs::sparta_opening_byzantium` (7 to 10 hexes) and the forced expedition (`plan.rs::plan_candidates`) follow the nomination. `storm_forecast` 1 (`opportunity.rs::forecast_siege`): an Sparta melee storm on an unwalled city is forecast with the split counterattack and its kill heal (up to 100 HP shared by the attackers that killed the garrison); 2 applies the split for every faction. `campaign_commit` 300 (`plan.rs::campaign_commitment`): any plan that walks away from a Campaign war we declared, while its target has no Walls, is charged 300, and the forced expedition keeps that target. `walls_under_way` 1 (`threat.rs::plan_threat_defense`, `production.rs::choose_item_inner`): Walls in progress in a threatened city are finished with Production, never swapped for a shooter, and a shooter within two hexes counts as covering the city. `walls_before_shooter` 1 (`threat.rs::walls_beat_arrival`): when scouted hostiles can be beaten to the city by Walls completion, the AI switches production to Walls before training a shooter. No AI path buys a Walls-line building; the old `price_bought_walls` forecast and Gold purchase logic were removed. Off, opt-in: `prior_needs_army` (the rush-faction prior counts only against a rival whose public army is this percent of ours, `adaptation.rs::assessed_border_threat`), `assault_wing_forecast` (the early Sparta wing trains melee until `campaign.rs::assault_wing_ready` says it takes the prize, and declares once staged within 6 hexes), `siege_window` (rounds played past arrival by `forecast_siege` and `city_assault.rs::city_assault_succeeds`; 0 keeps four).
- **Runaway response and coalition fronts (2026-10-18, playtest 8.9.2 Switzerland; AI only, no rule change).** On by default for Hard: `runaway_read` 1 (`threat.rs::runaway_signs`, the threat model's race term, `plan.rs::contain_leader`): the strict VP leader's public signs of an economic runaway (12 for a live faction goal; a quarter of its public Power lead over the mean, `PlayerPublic.lead_percent`) add to its race score. The crown term was removed from this signal when the crowns were deleted on 2026-11-11. The race starts at 30% of the target instead of 40%, the leader is not discounted by distance (proximity 100), and a leader with signs of 20 or more is containable from 30% of its target instead of `contain_leader_percent`. `coalition_front` 1 (`plan.rs::coalition_fronts`, `military.rs::open_front`): a Contain candidate gains half its leader's-race term per other team whose visible army stands within four hexes of one of the target's cities (up to two), and a containment war with no city in marching range stages on the target's nearest known city to open a front (the assault still needs its forecast). `walled_quota` 4 (`production.rs::walled_quota_value`): while the routed plan's war marches on a walled city, a shooter is worth a counter-class unit until four stand in the army, plus a melee escort of two. Off, opt-in: `leader_commit` (1: a plan that ends our war with the strict leader at half its target pays one VP of the leader's times the war's odds, `plan.rs::leader_peace_forfeit`; 2 also asks every faction for a funded siege route before a new Contain war, which cut wars on the leader and helped him in the Swiss what-ifs). The human-seat benchmark records war-quality counters per game in `diag` (`wars`, `wars_no_battle`, `wars_captured`, `war_captures`, `coalition_pairs`, `coalition_joint`, each also `_on_seat`; `hard_vs_human.rs::close_wars`).
- **Wars are fought: sieges and follow-through (AI only).** `war_siege` 100 and `campaign_persist` 100 remain the shared defaults. Sparta no longer has built-in `siege_exempt` or `plan_persist_skip`, so its army can siege and a viable campaign can persist like any faction's. The genes remain available for comparisons. `siege_parts` 15 controls the early campaign siege, late war siege, field raid and holding the front. With `siege_storm_first` 1, Sparta's early pass runs after its sorties, `rush_siege` and assault passes, and its late pass runs after campaign staging; those passes can commit the melee wing before the siege orders unused troops. `siege_early_shooters` 1 makes its early pass use only unassigned shooters. Other factions keep the earlier siege pass and its melee screen. The early pass attacks only the active campaign city; the late pass follows every routed war. `celtic_campaign` 1 allows Celtic campaigns and assaults; 0 restores the old gate. These are AI order choices, not engine rule changes (`military.rs`, `siege.rs`, `campaign.rs`, `plan.rs`).
- **Sparta's early rush is valued, not forbidden by Walls (AI only, 2026-10-31).** Its `early_campaign_value` 160 adds campaign plan value in the Ancient era and half that in the Classical era, then expires as the veteran infantry edge fades. `target_wall_cost` 180 raises the cost of nominating a walled conquest target, and `campaign_wall_cost` 120 prices the extra burden of a walled campaign in the plan choice. The shared values are zero. Sparta's built-in `early_walled_skip` is removed. The shared early Wall assessment now admits a city when the actual force has a viable capture forecast even without Siege equipment; that city still has to win the target and plan value comparisons. Engine combat and Wall strength are unchanged (`crates/ai/src/params.rs`, `conquest.rs::assessed_conquest_allowed`, `adaptation.rs::assessed_campaign_allowed`, `plan.rs::price_campaign`).
- **A valued wall siege stays the war plan (AI, 2026-10-25).** When a campaign ends its local forecast but the same walled target remains a positive plan candidate, is still at war, and a local force meets the siege strength ratio, Hard continues that Campaign plan (`plan.rs::keeps_walled_siege`, `choose_plan`). Its late siege pass can keep firing instead of opening another war for one transient forecast miss. A lost city, peace, or a weaker force still ends the plan normally.
- **Existing walled wars can stage a siege (AI, 2026-10-25).** If a short capture forecast fails after wall fire, Hard still prices an already declared war on a walled city when a reachable field force inside `siege_reach` has at least two shooters, a melee unit and enough strength. `plan.rs::wall_staging_forecast` estimates the march, city damage and losses; it offers a Campaign objective so the later siege pass can assemble the army. It never opens a new war on this estimate. City strikes still ignore terrain LOS and require team sight.
- **Value-scored scouting (2026-10-18, playtest 8.9.2 Sparta; AI only, no rule change).** On by default for Hard: `scout_find` 200 (`civilians.rs::ValueFrontier`, used by `choose_frontier`): while some rival capital is still unseen and before turn 40 (twenty half-pace turns, `SCOUT_FIND_UNTIL`), a frontier tile's worth to the Scout adds, to the usual 3 points per unexplored tile it would reveal within sight, `scout_find`% of a point per hex of reach (`clamp(1.3 x sqrt(map area / living empires), 10, 24)`, 24 on the 48x32 eight-empire map) that tile lies inside our nearest city, so the land around home is explored before the far edge (ruins keep their usual bonus; once every rival has a known city, or after turn 40, the plain `frontier_score` applies). It made the AI expand about 8% faster (3.54 cities at turn 100 against 3.26) and won 58% of a fresh-seed 8-player league against 50% (4 players 54.5%, 2 players no difference); it does not change who AI Sparta attacks or how many cities it takes. Intel genes tried and not kept (`claude/ai-value-scouting-experiments`): a sentry only beside a rival with a threatening public army, a spotter scout on the army's target, waiting for a fresh sighting before declaring on an unwalled city, value-scored re-scouting, a stale-sighting cost on the conquest target, and an explored border counting as seeing a rival. A Scout (sight 2) cannot enter a rival's tiles at Peace (including the Mongols since 2026-11-06), so once a rival's border is two or more hexes deep it cannot see the capital; the border itself, public on explored land, is often all it learns.
- **Reachable leader wars (AI, 2026-11-15; `plan.rs::choose_plan`, `runaway.rs::reachable_war_force`, gene `reachable_wars` 1 on Hard):** before a new Contain, runaway-leader Campaign, or coalition declaration, Hard needs two own field units within eight hexes of the same known target city, with a staging route traversable after the proposed declaration. Their local combat value must exceed nearby visible defenders, a per-city share of the target's unseen public army, and a one-turn estimate of units its public Gold could buy near that city. Purchases are capped at one per public city; undiscovered cities add one possible nearby purchase. The price uses the strongest current-era unit Hard has seen that empire field, or its era's melee line if none has been seen. Hard defers a fresh Contain or Campaign declaration for 15 turns after committing a Campaign to another empire, and a Contain declaration during another war also needs a funded capture forecast. A second-front operation likewise needs a funded forecast on its target city. Coalition treaty and join bonuses require the reachable force before rewarding a new war. The estimate reads only `PlayerView` and uses integer arithmetic. Once fighting or striking the public runaway leader, Hard keeps its Campaign plan on that leader. No game rule, save shape, or `RULES_VERSION` changes.
- **Urgent leader war candidate (AI, 2026-11-21; gene `leader_war`, default on):** a Contain that leads the alternatives by at least 100 may pass `reachable_wars`' one-front filters when its target is publicly counting down to a VP win or reads as a containable VP contender (at least 40% of its target and within two VP of the public leader). Two own military units still have to route to the same known city within the public hold countdown, or ten turns before a hold. This route uses prospective war entry and movement turns. The force must beat visible nearby defenders, the per-city share of unseen public army, and one purchase per nearby city using public Gold. When Swiss public Gold is within one affordable defender purchase of the 5,000 Reserve threshold, Gold below 5,000 is treated as VP committed to the Reserve; spending it to defend also delays the win. Other declarations retain the existing eight-tile, one-front gate. This changes only Hard's decision; orders, combat, saves and `RULES_VERSION` are unchanged.
- **Closing in on a fallen own city (AI, 2026-11-21; gene `held_shot_stepin`, default 1, Hard; `military.rs::may_close_into`, used by the `relief` branch of `Planner::defend`; `claude/hc-stepin`):** when every safe tile beside a besieged own city is taken or under fire, a free unit closes in on a nearer free tile. That branch no longer includes the city tile itself when the city is at 0 HP and `can_defend_at` is false for the unit (it would die to the volley and the charge with the city lost anyway). AI only; no rule, save or `RULES_VERSION` change. 0 restores the old step-in. Learnings 2026-11-21.
- **Storming a fallen enemy city (AI, 2026-11-22; gene `storm_fallen`, default 1, Hard; `military.rs::storm_fallen`, a pass before `withdraw_failed_campaign`; `claude/ai-storm-fallen`):** the pass handles a visible enemy city at war at 0 HP only when every land tile beside it is held. A city with a free tile beside it is left to the ordinary passes. For a held city, a free charger beside it storms it when `too_risky` passes. Otherwise the nearest charger within `open_city_radius` (at least 4) works in through our own non-capturing units (`storm_lane`), swapping places each turn with the one on its next step; the engine moves a closed friendly cycle together. Units within 4 of an own city with an at-war enemy within 3 stay home, unless already beside the target. With the gene off (0), nothing in Hard moves its own shooter off a fallen city's only approach. `strike` charges only from 2 hexes or nearer, its staging (`campaign_staging_spot`, `storm_ring_spot`) needs a free tile beside the city, and `seize_open_city` skips a city with any enemy unit beside it. Rules-50 seed 21999 ran to the 1,200-turn cap that way: Karakorum is coastal with two land neighbours, Egypt's CombatDrone held one, and the city sat at 0 HP for 888 turns. AI only; no rule, save or `RULES_VERSION` change. Learnings 2026-11-22.
- **Containing the leader (2026-10-07, playtest 8.4, Hard genes `contain_leader_percent` 70,
  `contain_share` 250, `contain_campaign`, `search_race`; `plan.rs`, `military.rs::contain_target`)**:
  in the Celts game two AIs at war with the human leader under Contain made 0 campaigns in 94 turns,
  and every search scored Contain like Peace. Now: (1) the strict public VP leader at 70% of its
  target is a Contain candidate before any hold or coalition rule names it (`contain_leader`);
  (2) Contain is priced with a *leader's race* term: a war at full strength (coalition army ≥ theirs)
  and reach takes `contain_share` permille of the target's win chance (its logit chance from our
  projection). The AI's share of the chance reopened by stopping that leader is the larger of its
  current chance and an equal share among the surviving nonleader teams; a rounded-to-zero solo
  chance therefore does not erase a viable coalition (`plan.rs::leader_race`, 2026-11-01). Every
  other empire already at War contributes its public Strongest Army force only in proportion to
  visible military reach within 15 hexes of a known leader city. The AI's own field army must also
  reach a known leader city; its share of the combined public force scales the race benefit, so a
  small exposed partner cannot claim the whole coalition's payoff. Other visible armies add a
  joint-strike bonus only when they threaten a different city theatre (cities more than four hexes
  apart; `plan.rs::coalition_fronts`). "Contest the hold" likewise uses the AI's own army-to-city distance and its
  public share of the coalition force; a city near home without an army is not a hold-breaking
  front (`plan.rs::contain_candidates`). This is a plan score
  from `PlayerView`, not a combat bonus. A leader's capital (strategic economy target, no intrinsic
  capture VP under these rules) and any city whose loss takes held VP receive extra campaign and
  containment target value from public force near that city (`campaign.rs::coalition_city_push`,
  `military.rs::campaign_target_for`). A Campaign candidate counts its capture odds times that
  coalition prize only if the capture can land before a holder finishes (`plan.rs::price_campaign`);
  the siege forecast and tactical gates still apply. Our field
  army's distance counts for reach against every public target, not only holders; (3) a Contain war
  has a body: while at war its free field units (after home defence and garrisons) march on the
  target's city with the most VP at stake for the least defence and distance
  (`campaign_target_for(.., contain = true)`: +400 per title its capture takes, the Heir it shelters,
  plus the ordinary denial, hoard and bounty terms; for Sparta, `contain_commit` (default on)
  requires `campaign_against` to find a funded, reachable forecast with at least
  `contain_min_odds` (30%) calibrated capture odds before a new declaration. Its
  march chooses only forecastable cities, including a softer unwalled city when one qualifies;
  range 20 measured from our cities or the marching army; the early Sparta restraint on walled or
  Celts cities is lifted; Germany may march, and so may Celts since 2026-10-23). Once two or more units stand within 4 of
  one of its cities and the campaign gates pass there, the plan starts a campaign on it
  (`start_planned_campaign`); the assault keeps every forecast. A Contain war funds a field force
  like a campaign (`contain_war`: `desired_army` = cities + `army_war_extra` (1; Athens 0) and the campaign unit scores in
  `production.rs`); (4) Hard + Search adds each Contain option's leader's race to its rollout mean
  when choosing (`SearchOption::beyond`): the 10-turn rollouts can't see a war's payoff, and in 8.4
  15 of 17 searches had identical Contain and Peace samples. Remaining identical pairs are seats
  with no army to march (none built) facing a leader that refuses peace: there the two plans
  issue the same orders. Measured (`contest_leader` example, 8-player Standard, seeds 9600001+,
  against master ae36e888): a 7-VP-at-T80 or holding leader answered by a march or campaign within
  15 turns 54% → 80% (plain Hard, 1,024 games) and 69% → 72% (all Hard + Search, 256 games);
  coalitions that never allied 23% → 5%, treasury refusals 1,478 → 47, stalls 15 → 7; identical
  Contain/Peace rollouts 45% → 31%; Proxy v7 duel unchanged (paired 0.0 [-1.2, +1.2], 2,048 games).
- **Hold-to-win AI (2026-10-06, `crates/ai/src/hold.rs`, `winchance.rs` VERSION 4)**: every
  AI reads the public countdown (`TeamStanding::victory_hold`). Win chance values a hold as the
  chance of surviving its remaining turn ends, not a win: per-turn break hazard
  `winchance::HOLD_HAZARD` 10% at the target (an unmeasured assumption), divided by 1 + VP
  margin, rising linearly to certain one VP below the target; a projection that loses a title
  the holder leads costs a VP of margin; a team projected at its target starts a full window.
  The holder's completion chance wins outright and the logit shares the rest. Against a rival
  holder: it is the threat tracker's top threat (race +100, more as the clock runs), never gets
  peace (`wants_peace`, all difficulties), Alliances that join a war on it are worth signing, Normal declares whenever an army can reach it, Hard's plan offers Contain
  against every holder (our field army's distance counts for reach; no runway gate; a breach is
  allowed) and campaigns on its cities whose capture takes a title (`capture_title_losses`); a
  capture forecast to land after its countdown ends is priced as worthless. Fire, attack, city
  campaign, conquest and opportunity targets of a holder get `denial_priority` (more as the
  clock runs, when the hit takes a title, and when the margin is thin). A holding AI: at margin
  0 with Strongest Army it keeps the army home (no scripted campaign without an active one) and
  attacks at 5/4 the threshold (`guards_army_title`); it never disbands for runway or campaign
  funding the unit whose loss would cost Strongest Army (`unit_keeps_our_title`); incoming peace
  scores +40. Openings close as soon as anyone holds.
- **Win-chance weights per player count (2026-10-06, `winchance::WEIGHTS_DUEL`, VERSION 3)**:
  1v1 has its own fitted weights (army and science shares near zero; economy, late VP progress
  and cities dominate). Search rollouts are always scored with the weights for the game's team
  count (`WinFacts::search_chance`); the plan layer prices with the 8-player `WEIGHTS` unless
  the search option `w` sets `WinFacts::fitted`.
- **Settler snipe (Hard plan; genes `settler_raid`, `settler_raid_range`, `settler_chain`, and
  `settler_chain_priority`)**: with `settler_raid` on, a rival Settler at plain Peace with a
  legal non-breach declaration, not in a city and
  with no owner military adjacent, can be targeted by a healthy melee unit that can reach it
  in up to two movement turns. The plan declares and routes toward it; if adjacent, G20 makes
  the war effective before movement, which pins and captures the Settler. If it is one turn
  farther away, the plan is retained for one turn so it can finish the capture if the target
  remains reachable. When the target is Sparta and a Byzantium player is alive, the AI offers
  only immediate adjacent captures by non-Scout melee units; the two-turn pursuit remains
  available against other empires. This is an AI target gate, not a change to capture or war
  resolution. The candidate is priced as 0.8 of a city for us against 0.8 for them,
  less retaliation and aggressor risk; after capture the next plan can seek
  peace. The movement reconciler treats a declared team's units as hostile
  (`plan.rs::snipe_targets,execute_snipe`, `movement.rs::movement_conflict`). With
  `settler_chain` on, a capital below four cities prioritizes Settlers after its Granary when a
  good site remains and Hard's existing threat and siege assessments are quiet. The existing
  `settler_pipeline` limit still caps simultaneous Settlers; `settler_chain_priority` controls
  the extra production score. The normal Walls and defense genes and threat gates still apply
  (`production.rs::choose_item`).
- **Settler trip risk (Hard multiplayer)**: `peace_settler_capture` (default 0 since the 2026-10-24 gate, which found opponents softer at 100 (docs/sim-results/ai-quality/gate-1024); re-gated 2026-11-03 with `settler_safety` 1 after `settler_stalk`: rivals-only +3.12 pp softer, seat-only -4.39 pp, docs/sim-results/ai-quality/settler-safety-1103; range
  0–200) activates the route and escort policy; 0 restores the prior Hard policy. An
  enemy already at War or a barbarian contributes 100 exposure. A rival at Peace with a
  legal surprise declaration contributes its estimated declaration chance, scaled by the
  gene: the chance is higher for a remote rival whose cities cannot readily be struck in
  return, for a human or aggressive faction, and for the immediate gain of a free Settler;
  it is zero for an ally or a blocked declaration. With `snipe_read` a rival with a recent
  surprise war is at least that gene's percent, with `peace_settler_capture` 0 or not. The chance is multiplied by route
  interception reach, then by twice the site's value to price our lost city and their gained
  city. A peaceful Scout also counts when this chance reaches the escort threshold, because
  Scouts can capture civilians after declaring War. Remembered threats, camps and known borders
  still add smaller priors. A candidate
  at `settler_risk_avoid` exposure (default 100) becomes a last resort; at
  `settler_risk_escort` (default 40) the Settler seeks a guard, waits or detours rather than
  finish a move within capture reach. Both thresholds are genes. The separate
  `settler_safety` gene can still activate the route policy when `peace_settler_capture` is
  zero (`civilians.rs::settler_trip_risk,peace_settler_capture_chance,move_settler`,
  `hard_defense.rs::prepare_hard_defense,guard_exposed_settlers`).

**UI note (2026-10-03, claude/attack-at-peace):** ranged targeting of a unit or city at Peace asks "Declare War?" (the declaration and the attack resolve the same turn at plain Peace). Clicking a foreign city opens its card with Declare War, Diplomacy and Attack with <selected unit>. Core's error for a real target at Peace reads "you are not at War with its owner; declare War first" (`rules/battle.rs`).

**Goal calibration (2026-10-07).** 1,024 8-player Standard Hard games per setting (cloud, seeds 5300001+, checked on 5400001+): Swarm 50 -> 30 population, Heir 200 -> 400 Production (half-points under the old opening ramp; today's prices are doubled, `data::HEIR_COST` 800), Sparta AI retire window 70% -> 55% (the level was 4 in this calibration; it became 3 on 2026-11-06 after the user reported rarely reaching level 4) of the game (`goal_actions.rs::RETIRE_LATE_PERCENT`); Courier Hall 300 (today `data::YAM_RELAY_STATION_COST` 600) and the 45%/55%/70% Hall, forge and Wild cutoffs unchanged. AI: veterans retire one at a time as they reach a War Shrine city once enough exist to finish (one unit per tile made all-at-once almost impossible), and with no turn limit `goal_actions::last_turn` keeps a tenth of the expected length ahead once the game runs past it, so goal deadlines don't all close.

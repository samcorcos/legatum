# Climb program stage 1: hard-coded behaviour gates in crates/ai

Stage 1 turns hard-coded behaviour gates in `crates/ai/src` into genes (`decision_params!` in `params.rs`), with today's value as the default, so the climber (`tools/evolve`) can explore them. A faction gate becomes a gene whose shared default is what most factions get, plus `BUILTIN_FACTION_DECISIONS` entries for the factions that were gated. Default behaviour is unchanged by construction (every pin and fixture hash is unchanged).

## Counts

- Found (rows below): 117
- Converted: 101 rows, 96 genes (new or reused), in three landings, each verified (tests, clippy, fmt, 50-game smoke, no pin diff)
- Skipped: 16 rows (rule knowledge, already genes, structural, frozen proxies, weights)

Sites that share one gene count as separate rows only when they are separate decisions. The city-attack gates of the Celts were being removed by another agent (celts-sack) while landings 1 and 2 were made; landing 3 converted what remained after it landed.

Notes for the climber:

- Flipped genes: `planned_stage`, `sparta_force_campaign`, `early_campaign_muster`, `campaign_march_patience` had a shared default that only Sparta used. The shared default is now 0 (off) and Sparta carries the old value in `BUILTIN_FACTION_DECISIONS`, so Sparta plays exactly as before and any other faction can switch them on. A parameter file that set one of them to its old shared value now turns it on for every faction in that run.
- The Sparta-only behaviour genes (`lone_city_strike`, `chain_press`, `open_followup`, `assault_wing`, `walled_prize`, `engaged_assault_keep`, `early_prize_opening`, `army_item_bonus`, `war_shrine_drive`) are packages with their own turn and radius genes; turn them on for another faction with a faction override.
- `strike_eagerness` has per-faction entries including Celts (a number, not an attack gate).

## Converted

| Site | What it did | Kind | Gene | Shared default | Per-faction defaults | Status |
|---|---|---|---|---|---|---|
| conquest.rs::assessed_conquest_allowed | A walled city above a third HP is no conquest target unless a campaign forecast takes it, for Mongols only | faction gate | conquest_walled_forecast | 0 | Mongols 1 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Germany nominates no conquest city | faction gate | conquest_enabled | 1 | Germany 0 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Fewer than two cities: no nominee, except Sparta | exclusion | conquest_min_cities | 2 | Sparta 0 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Switzerland bank for the treasury goal: nominee only against a runaway | faction gate | conquest_bank_goal | 0 | Switzerland 1 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Runaway = public leader at 45% of its VP target | threshold | conquest_runaway_percent | 45 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | One-city Sparta: only an open Japan or Byzantium city is a nominee (and the Byzantium window) | faction gate | lone_city_strike | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | The Byzantium window around the lone capital is 7 hexes | threshold | lone_city_strike_radius | 7 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city, military.rs::campaign_target_for | Early Sparta (first third of the game) skips walled cities and Celtic land (the Celtic clause is kept inside the gene, not removed) | exclusion | early_walled_skip | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Nominee must be within 14 hexes of our nearest city | threshold | conquest_home_distance | 14 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Nominee path at most 20 steps | threshold | conquest_path_len | 20 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Value weight: already at war with the owner (80) | threshold (weight) | conquest_war_bonus | 80 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Value weight: city visibly damaged (100) | threshold (weight) | conquest_wounded_bonus | 100 |  | converted, landing 1 (21bd3b81d) |
| conquest.rs::choose_conquest_city | Value weight per own unit within 5 hexes (20) | threshold (weight) | conquest_nearby_weight | 20 |  | converted, landing 1 (21bd3b81d) |
| siege.rs::siege_enemy | war_siege never sieges for Germany and Sparta (the Celtic clause was converted in landing 3) | faction gate | siege_exempt | 0 | Germany 1, Sparta 1 | converted, landing 1 (21bd3b81d) |
| military.rs::plan_military, campaign (3 sites) | Sparta stages the field force toward a chosen Campaign city before the siege opens | faction gate | planned_stage (existing gene, shared default 1 -> 0) | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| military.rs::campaign_target_for (contain branch) | A containment march is staged only on a city the campaign outlook can take, Sparta only | faction gate | contain_march_outlook | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| military.rs::campaign_target_for | Containment march priority + 35 per own unit near the target, Sparta only | faction gate (weight) | contain_stack_weight | 0 | Sparta 35 | converted, landing 1 (21bd3b81d) |
| military.rs::sparta_nearby_sortie | Lone-capital sortie on a nearby open Byzantium city, Sparta only | faction gate | lone_city_strike (shared with the nominee gate) | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| military.rs::sparta_nearby_sortie | Sortie window ends turn 48 | threshold | lone_city_sortie_turn | 48 |  | converted, landing 1 (21bd3b81d) |
| military.rs::sparta_followup_sortie | Two-city follow-up sortie after a first capture, Sparta only | faction gate | open_followup | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| plan.rs::campaign_forecast (march second-front test) | A march ignores a second war against a team with no cities, Sparta only | faction gate | march_ignores_dead_war | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| plan.rs::campaign_forecast (march second-front test) | A march on a nearby open target ignores a second war whose army is far from home, Sparta only | faction gate | march_ignores_second_war | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| plan.rs::grind_forecast | No grind-past-a-forecast-miss pricing for a running campaign, Sparta only | faction gate | plan_persist_skip | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| plan.rs::plan_candidates, choose_plan (2 sites) | Sparta keeps a feasible Campaign through plan and search valuations | faction gate | sparta_force_campaign (existing gene, shared default 1 -> 0) | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| plan.rs::missing_plan_patience | Sparta lets a chosen unwalled Campaign stage through missing forecasts (12 turns) | faction gate | campaign_march_patience (existing gene, shared default 12 -> 0) | 0 | Sparta 12 | converted, landing 1 (21bd3b81d) |
| plan.rs::choose_plan, missing_plan_patience, diplomacy.rs::plan_diplomacy (3 sites) | Sparta stages an early Campaign at Peace before declaring | faction gate | early_campaign_muster (existing gene, shared default 1 -> 0) | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_chain_target | A field wing presses a newly founded open city (three cities or more), Sparta only | faction gate | chain_press | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_chain_target | Chain press window ends turn 130 | threshold | chain_press_turn | 130 |  | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_open_followup | Two-city follow-up on the next open city, Sparta only | faction gate | open_followup | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_open_followup | Follow-up window ends turn 84 | threshold | open_followup_turn | 84 |  | converted, landing 1 (21bd3b81d) |
| campaign.rs::wing_target | The early assault wing is trained for a nearby open prize, Sparta only (needs assault_wing_forecast) | faction gate | assault_wing | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_opening_byzantium | A lone first city strikes an open Byzantium or other open city, Sparta only | faction gate | lone_city_strike | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_opening_byzantium | Opening strike window ends turn 70 | threshold | opening_strike_turn | 70 |  | converted, landing 1 (21bd3b81d) |
| campaign.rs::sparta_walled_prize | A walled city whose capture changes the race is worth paying through the wall, Sparta only | faction gate | walled_prize | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| campaign.rs (engaged_open_assault) | A funded assault already engaged on a damaged open city is kept, Sparta only | faction gate | engaged_assault_keep | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| production.rs::choose_item_inner (6 sites) | The opening plays for an early open prize city (Settler timing, assault wing, melee first), Sparta only | faction gate | early_prize_opening | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| production.rs::choose_item_inner | Melee-first window ends turn 110 | threshold | early_prize_turn | 110 |  | converted, landing 1 (21bd3b81d) |
| production.rs::item_base_value (army unit) | Army unit below the army quota: +100 value, Sparta only | faction gate (weight) | army_item_bonus | 0 | Sparta 100 | converted, landing 1 (21bd3b81d) |
| production.rs::item_base_value (War Shrine) | War Shrine worth 240 + military while the army is under quota, Sparta only | faction gate | war_shrine_drive | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| production.rs::uncapped_army | War or containment: cities + 5 | threshold | army_war_extra | 5 |  | converted, landing 1 (21bd3b81d) |
| production.rs::uncapped_army | Rush strategy: cities + 4 (+1 after turn 120) | threshold | army_rush_extra | 4 |  | converted, landing 1 (21bd3b81d) |
| production.rs::uncapped_army | Hard at peace: cities + 2, Sparta cities + 4 | faction gate | army_hard_extra | 2 | Sparta 4 | converted, landing 1 (21bd3b81d) |
| production.rs::uncapped_army | Hard at peace: +1 when the military knob is 70 or more, not for Sparta | faction gate | army_military_bonus | 1 | Sparta 0 | converted, landing 1 (21bd3b81d) |
| production.rs::uncapped_army | Hard at peace: Switzerland keep max(cities, 2) plus one under pressure | faction gate | army_banker | 0 | Switzerland 1 | converted, landing 1 (21bd3b81d) |
| production.rs::item_base_value (Market) | Market +60 for Switzerland | faction gate (weight) | toll_market_bonus | 0 | Switzerland 60 | converted, landing 1 (21bd3b81d) |
| production.rs::item_base_value (Scout) | First Scout +150 for Byzantium, Mongols, Athens | faction gate (weight) | scout_priority_bonus | 0 | Byzantium 150, Mongols 150, Athens 150 | converted, landing 1 (21bd3b81d) |
| production.rs, diplomacy.rs (4 sites) | A Rush strategy window ends turn 120 (rush production, no neighbour Alliance, army extra) | threshold | rush_window_turn | 120 |  | converted, landing 1 (21bd3b81d) |
| production.rs::choose_item_inner | The peaceful opening Settler chain counts as early before turn 80 | threshold | opening_chain_turn | 80 |  | converted, landing 1 (21bd3b81d) |
| investment.rs::work_bonus, research.rs, production.rs (3 sites) | Great Work plans steer play from turn 120 | threshold | work_plan_turn | 120 |  | converted, landing 1 (21bd3b81d) |
| opportunity.rs::identity_eagerness | Per-faction eagerness added to an early strike: Celts +10, Germany -20, Sparta +30, Mongols +25 | faction gate (table) | strike_eagerness | 0 | Celts 10, Germany -20, Sparta 30, Mongols 25 | converted, landing 1 (21bd3b81d) |
| exploration.rs::frontier_score, civilians.rs (2 sites) | A ruin tile is worth 80 (Byzantium, Athens) or 40 to a Scout | faction gate (table) | ruin_bonus | 40 | Byzantium 80, Athens 80 | converted, landing 1 (21bd3b81d) |
| civilians.rs::choose_frontier | Sparta reveals the approach around home before chasing distant ruins while a Byzantium player lives | faction gate | approach_scout | 0 | Sparta 1 | converted, landing 1 (21bd3b81d) |
| civilians.rs::choose_frontier | Approach scouting ends turn 24 (window radius shares lone_city_strike_radius) | threshold | approach_scout_turn | 24 |  | converted, landing 1 (21bd3b81d) |
| civilians.rs::choose_frontier | Distant reconnaissance weight 2 per hex from home (cap 12), Mongols only | faction gate (weight) | recon_weight | 0 | Mongols 2 | converted, landing 1 (21bd3b81d) |
| adaptation.rs::rush_walls_threat | A rival army growth counts as a rush before turn 40 | threshold | rush_read_turn | 40 |  | converted, landing 1 (21bd3b81d) |
| adaptation.rs::rush_walls_threat | A rival city within 12 hexes is a neighbour | threshold | rival_radius | 12 |  | converted, landing 1 (21bd3b81d) |
| victory.rs::pursuing_victory | Within 4 VP of the target from turn 80 counts as pursuing victory | threshold | victory_close_turn | 80 |  | converted, landing 1 (21bd3b81d) |
| victory.rs::endgame_victory_planning | Endgame victory plan from turn 160 | threshold | endgame_victory_turn | 160 |  | converted, landing 1 (21bd3b81d) |
| military.rs::campaign_target_for | A staged campaign picks its target within CAMPAIGN_RANGE 14 hexes (plus a tenth of the pressure) | threshold | campaign_range | 14 |  | converted, landing 1 (21bd3b81d) |
| military.rs::campaign_target_for | A containment march adds CONTAIN_EXTRA_RANGE 6 hexes | threshold | contain_extra_range | 6 |  | converted, landing 1 (21bd3b81d) |
| plan.rs::choose_plan | A new plan must beat the running one by SWITCH_MARGIN 40 (doubled before its review turn) | threshold | switch_margin | 40 |  | converted, landing 1 (21bd3b81d) |
| plan.rs::campaign_candidates (outlook cutoff) | A campaign under MIN_SEAT_WAR_ODDS 15 percent capture odds is not offered | threshold | min_seat_war_odds | 15 |  | converted, landing 1 (21bd3b81d) |
| plan.rs::peace_would_be_broken | A war fought within RECENT_FIGHT_TURNS 8 turns counts as active | threshold | recent_fight_turns | 8 |  | converted, landing 1 (21bd3b81d) |
| plan.rs::campaign_forecast, campaign.rs | Field army counts as in reach within MARCH_TURNS 6 turns | threshold | march_turns | 6 |  | converted, landing 1 (21bd3b81d) |
| opportunity.rs::forecast_siege | SIEGE_ROUNDS 4: rounds the siege forecast plays when siege_window is 0 | threshold | siege_rounds | 4 |  | converted, landing 1 (21bd3b81d) |
| opportunity.rs::choose_strike, could_strike | APPROACH_TURNS 2: turns a unit may take to reach a struck city | threshold | approach_turns | 2 |  | converted, landing 1 (21bd3b81d) |
| opportunity.rs::evaluate_strike | RETALIATION_RADIUS 6: enemy army within this many hexes of home counts as retaliation | threshold | retaliation_radius | 6 |  | converted, landing 1 (21bd3b81d) |
| siege.rs (2 sites) | SIEGE_REACH 8: our units this close count toward a siege | threshold | siege_reach | 8 |  | converted, landing 1 (21bd3b81d) |
| siege.rs (2 sites) | RAID_REACH 6: enemy field units this close are raid targets | threshold | raid_reach | 6 |  | converted, landing 1 (21bd3b81d) |
| diplomacy.rs (5 sites) | ALLY_PROGRESS_LIMIT 70: leave or refuse an Alliance with a team this far along | threshold | ally_progress_limit | 70 |  | converted, landing 1 (21bd3b81d) |
| diplomacy.rs (2 sites) | ALLY_OFFER_PROGRESS 60: no new Alliance with a team this far along | threshold | ally_offer_progress | 60 |  | converted, landing 1 (21bd3b81d) |
| research.rs::research_order | WAR_LEAN 150: military tech value added while the plan is war | threshold (weight) | war_lean | 150 |  | converted, landing 1 (21bd3b81d) |
| research.rs::research_order | PEACE_LEAN 60: growth tech value added while the plan is peace | threshold (weight) | peace_lean | 60 |  | converted, landing 1 (21bd3b81d) |
| research.rs::research_order | HYSTERESIS_PERCENT 125: a new research choice must beat the current one by this much | threshold | research_hysteresis | 125 |  | converted, landing 1 (21bd3b81d) |
| civilians.rs::value_frontier | SCOUT_FIND_UNTIL 40: value-scored scouting window | threshold | scout_find_turns | 40 |  | converted, landing 1 (21bd3b81d) |
| military.rs::strike pass, targets (2 sites) | Couriers raid civilians but take no field battle far from home, Mongols only | faction gate | courier_raid_only | 0 | Mongols 1 | converted, landing 2 (01696bd0e) |
| production.rs::goal_building_bonus | Walls +150 within 8 hexes of an enemy city, Switzerland only | faction gate (weight) | walls_near_enemy_bonus | 0 | Switzerland 150 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value (faction_priority table) | Bronze Working bias: Germany 100, Celts 70 | faction gate (table) | bronze_bias | 0 | Germany 100, Celts 70 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Irrigation bias: Celts 70 | faction gate (table) | irrigation_bias | 0 | Celts 70 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Currency bias: Mongols 100 | faction gate (table) | currency_bias | 0 | Mongols 100 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Banking bias: Mongols 60 | faction gate (table) | banking_bias | 0 | Mongols 60 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Feudalism bias: Sparta 180, Byzantium 60 | faction gate (table) | feudalism_bias | 0 | Sparta 180, Byzantium 60 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Chivalry bias: Sparta 120 | faction gate (table) | chivalry_bias | 0 | Sparta 120 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Writing, Code of Laws, Philosophy bias: Athens 140 | faction gate (table) | scribe_bias | 0 | Athens 140 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Currency +20 for Switzerland | faction gate (weight) | currency_tech_bonus | 0 | Switzerland 20 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Iron Working and Industrialization +100 for Germany | faction gate (weight) | industry_tech_bonus | 0 | Germany 100 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Code of Laws +80 for Japan | faction gate (weight) | law_tech_bonus | 0 | Japan 80 | converted, landing 2 (01696bd0e) |
| research.rs::tech_value | Crop Rotation +100 for Celts | faction gate (weight) | crop_tech_bonus | 0 | Celts 100 | converted, landing 2 (01696bd0e) |
| production.rs::unique_unit_bonus | Veteran infantry +100 for Byzantium | faction gate (weight) | veteran_infantry_bonus | 0 | Byzantium 100 | converted, landing 2 (01696bd0e) |
| production.rs::unique_unit_bonus | Hunter units +90 for Celts | faction gate (weight) | hunter_bonus | 0 | Celts 90 | converted, landing 2 (01696bd0e) |
| production.rs::unique_unit_bonus | Infantry +100 for Sparta | faction gate (weight) | infantry_bonus | 0 | Sparta 100 | converted, landing 2 (01696bd0e) |
| production.rs::roadworks_value | Roadworks base value before a target is committed: Sparta 1000, Mongols 240 | faction gate (table) | roadworks_war_base | 0 | Sparta 1000, Mongols 240 | converted, landing 2 (01696bd0e) |
| production.rs::roadworks_value | Roadworks value per enemy city in range (max 4): Sparta 100, Mongols 40 | faction gate (table) | roadworks_war_per_enemy | 0 | Sparta 100, Mongols 40 | converted, landing 2 (01696bd0e) |
| campaign.rs (3), plan.rs (2), production.rs (3), military.rs | An open enemy city within 10 hexes is an early prize (follow-up, wing, expedition force, early prize opening, march test, nearby sortie) | threshold | open_prize_radius | 10 |  | converted, landing 2 (01696bd0e) |
| campaign.rs, production.rs, military.rs (3 sites) | The Byzantium window of 7 hexes in the opening prize, sortie and early prize tests | threshold | lone_city_strike_radius (existing from landing 1) | 7 |  | converted, landing 2 (01696bd0e) |
| diplomacy.rs::border_raid_opportunity | Optional wars may open on a valuable local field target, Germany only | faction gate | border_raid | 0 | Germany 1 | converted, landing 3 |
| diplomacy.rs::diplomatic_campaign_uncached, military.rs::campaign_target_for | Germany takes no campaign target except a containment war | faction gate | campaign_contain_only | 0 | Germany 1 | converted, landing 3 |
| military.rs::scout_front | Germany sends no unit to find a neighbour's cities | faction gate | scout_front | 1 | Germany 0 | converted, landing 3 |
| threat.rs::second_front_temperament | Second-front temperament table: Sparta +25, Mongols +10, Switzerland -15, Germany -10 | faction gate (table) | second_front_temperament | 0 | Sparta 25, Mongols 10, Switzerland -15, Germany -10 | converted, landing 3 |
| siege.rs::siege_enemy | war_siege never sieges for the Celts (Celts clause, now a built-in entry next to Germany and Sparta) | faction gate | siege_exempt | 0 | Celts 1 (with Germany 1, Sparta 1) | converted, landing 3 |
| campaign.rs (4), production.rs, plan.rs, diplomacy.rs (3) | Celtic cities are not preferred prey: excluded from the open-prize tests, plan targets and early picks, ranked last otherwise | exclusion | prey_skips_celts | 1 |  | deleted 2026-10-23 with the Celtic hunters and homeland damage (celts rework) |

## Skipped

| Site | What it is | Kind | Why skipped |
|---|---|---|---|
| conquest.rs::choose_conquest_city, military.rs::campaign_target_for (Celtic clause inside early_walled_skip); tactics.rs::reveal_cost; opportunity.rs::sack_gold_value; hold.rs; military.rs (Celts land check); opponents.rs; civilians.rs | Celtic clauses that remain: the early Sparta skip of Celtic land (kept inside early_walled_skip), hunters' reveal cost, the sack Gold, titles a Celtic captor keeps, rival-Celt threat priors | faction gate | Rule knowledge (hunters, sack, hoard rules, rival public rules) or inside a converted gene |
| victory.rs, production.rs, goal_*.rs (Switzerland hoard; Germany heir and Forge; Mongols Hall; Japan network; Athens relay; Byzantium heir) | Faction goal logic and the AI valuing its own goal | faction gate | Rule knowledge: a faction's own goal rules, not a behaviour choice |
| production.rs::swiss_market_priority | Hard Switzerland with a pending goal add a Market bonus | faction gate | A gene already prices it (300); an existing test asserts the faction gate, so it stays until that test is revised |
| diplomacy.rs (sparta_restless_target, sparta_iron_target, sparta_needs_war, restless_far_war), plan.rs (desertion terms), military.rs (iron target) | Sparta desertion in peace and the iron price | faction gate | Rule knowledge: Sparta regiments desert in peace and pay iron (`SPARTA_DESERTION_INTERVAL`) |
| military.rs::march_min_hp, drill_veterans; tactics.rs (rest, counterattack, kill-heal value); opportunity.rs (strike min HP, capture heal forecast, storm kill heal); plan.rs, campaign.rs expedition HP; params.rs for_faction (recover_hp) | Sparta regiments never heal by resting, so wounded ones march, strike and drill at lower HP | faction gate | Rule knowledge: no healing by rest; Honour the Fallen is a goal (veteran_drill) |
| military.rs (courier garrison and cover), production.rs, threat.rs, adaptation.rs (Mongols Engineers, no Walls) | Mongols Engineer training and no wall building | faction gate | Rule knowledge: Engineer and wall rules (courier raids are converted: courier_raid_only) |
| military.rs (slow_core), production.rs (Germany fallback Industry project), victory.rs (engine goal) | Germany slow machines and Industry projects | faction gate | Rule knowledge: Germany unique rules |
| tribute.rs, plan.rs (tribute), opponents.rs, investment.rs | Byzantium pays tribute; rival-faction rules (Celts attrition, Japan links) | faction gate | Rule knowledge: tribute is the Byzantium mechanic; the rest model other factions' public rules |
| production.rs::goal_building_bonus (Germany Workshop, Athens Laboratory and Workshop, Byzantium Scriptorium) | Value of the buildings a faction's goal needs | faction table | Rule knowledge: faction goal logic |
| knobs.rs (Knobs::for_faction) | Per-faction personality knobs | faction table | Already tunable: `factions` in a parameter file and knob.* genes |
| opportunity.rs (storm_forecast 1) and conquest.rs::value_first_target (value 1) | Tri-state genes whose value 1 means Sparta only | faction gate | Already genes: a per-faction override of 2 turns them on for any faction |
| civilians.rs (scout threat radius and prior by the owner's faction) | How far a rival faction's units are feared | faction table | Keyed by the rival's faction (its public rules), not ours: not expressible as a gene of our own faction |
| adaptation.rs::early (turn < 120) | Early game cut-off used by the opponent model and conquest | threshold | Structural: a view-only helper called from the opponent model; a gene needs plumbing through opponents.rs |
| proxy*.rs, proxy_v*/ | Frozen human-proxy benchmark policies | frozen opponent | Not live AI policy: fixed on purpose (the benchmark must not move) |
| threat.rs (TOP_THREAT 60, WATCH_THREAT 30), barbarians.rs, engineers.rs, levy.rs, goal_*.rs constants, research.rs ERA_* | Other named threshold constants | threshold | Opponent-model structs have no access to decision genes (plumbing needed), goal constants are the faction goals' rule knowledge, the rest are minor; candidates for a later stage. The former civilians.rs SETTLER_RISK_* thresholds are now decision genes `settler_risk_escort` and `settler_risk_avoid`. |
| Scoring weights across production.rs, military.rs, plan.rs, threat.rs, conquest.rs (cost terms, 100 / 25 / 12 weights), ~130 distance and HP literals | Value-function weights and local radii of 3 to 10 hexes | threshold (weight) | Not gates; each needs its function read to name it. Candidates for a later stage (the big radii and turn cutoffs are done) |

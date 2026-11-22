//! Tuned decision parameters for the scripted AI, loaded from a JSON file (`--ai-params`).
//!
//! Pillar 6: this file can hold personality knobs and bounded tactical/macro decision weights.
//! These parameters feed the planner's preferences and nothing else; the
//! brain still sees only its fogged `PlayerView` and every order goes through the core's
//! validation. There is no field for yields, costs, vision, prices or any other rule, and
//! unknown fields are rejected, so a genome cannot change the game, only how the AI decides.

use crate::difficulty;
use crate::knobs::Knobs;
use legatum_core::{AiDifficulty, Faction};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PARAMS_SCHEMA: u32 = 1;

/// Knob values as written in a parameter file: all five required, each 0-100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnobValues {
    pub expansion: i32,
    pub military: i32,
    pub science: i32,
    pub economy: i32,
    pub aggression: i32,
}

impl From<KnobValues> for Knobs {
    fn from(v: KnobValues) -> Self {
        Knobs::new(v.expansion, v.military, v.science, v.economy, v.aggression)
    }
}

impl From<Knobs> for KnobValues {
    fn from(k: Knobs) -> Self {
        Self {
            expansion: k.expansion,
            military: k.military,
            science: k.science,
            economy: k.economy,
            aggression: k.aggression,
        }
    }
}

/// Shared decision genes. Each default is the former literal at the decision site.
/// Integer ranges constrain preferences, never the core's combat or economic rules.
#[derive(Clone, Debug, Serialize)]
pub struct DecisionGene {
    pub name: &'static str,
    pub min: i32,
    pub max: i32,
    pub default: i32,
    pub description: &'static str,
}

macro_rules! decision_params {
    ($($(#[$field_attr:meta])* $field:ident: $default:expr, $min:expr, $max:expr, $description:expr;)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct DecisionParams { $($(#[$field_attr])* pub $field: i32,)* }

        impl Default for DecisionParams {
            fn default() -> Self { Self { $($field: $default,)* } }
        }
        #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct DecisionOverrides { $($(#[$field_attr])* #[serde(skip_serializing_if = "Option::is_none")] pub $field: Option<i32>,)* }

        impl DecisionParams {
            pub fn genes() -> Vec<DecisionGene> {
                vec![$(DecisionGene { name: concat!("decision.", stringify!($field)),
                    min: $min, max: $max, default: $default, description: $description },)*]
            }
            fn validate(&self) -> Result<(), String> {
                $(if !($min..=$max).contains(&self.$field) {
                    return Err(format!("decisions.{} must be within {}..={}",
                        stringify!($field), $min, $max));
                })*
                Ok(())
            }

            /// Set one gene by its bare name (`war_siege`); false when no such gene exists.
            fn set_named(&mut self, name: &str, value: i32) -> bool {
                match name {
                    $(stringify!($field) => { self.$field = value; true })*
                    _ => false,
                }
            }

            fn get_named(&self, name: &str) -> Option<i32> {
                match name {
                    $(stringify!($field) => Some(self.$field),)*
                    _ => None,
                }
            }

            fn with_overrides(mut self, overrides: &DecisionOverrides) -> Self {
                $(if let Some(value) = overrides.$field { self.$field = value; })*
                self
            }
        }

        impl DecisionOverrides {
            fn validate(&self, faction: Faction) -> Result<(), String> {
                $(if let Some(value) = self.$field {
                    if !($min..=$max).contains(&value) {
                        return Err(format!("faction_overrides.{faction:?}.{} must be within {}..={}",
                            stringify!($field), $min, $max));
                    }
                })*
                Ok(())
            }
        }
    }
}

decision_params! {
    mongol_horde: 0, 0, 3, "Hard Mongols: 1 mounted war force and early counter plus Iron Working, 2 funded veteran Keshig upgrades and retreat, 3 ranged siege of walled cities before capture";
    walls_garrison_value: 0, 0, 200, "Shared Walls-as-garrison valuation, percent: compare expected city damage saved and dealt per Production with a troop's defence plus its offensive service under the current plan (0: legacy garrison floor)";
    modernization_value: 100, 0, 200, "Weight squared combat strength against observed opponents in unit production and purchases (0: old valuation)";
    hard_purchase_scorer: 1, 0, 1, "Hard: score military purchases by combat against visible foes, survival, mobility and avoided refit cost; hold non-rescue buys for a due upgrade when the city survives (0: original purchase orders)";
    siege_survival_clock: 0, 0, 1, "Hard: forecast a city's survival through eight turns of visible siege with engine combat forecasts, and use its capture or provisional breach clock as the deadline for defensive builds (0: the former one-turn danger read and production timing). Default 0 since 2026-11-21: on the held-out Athens 8.9.47 and Rome 8.9.67 games (12 hybrid cells, 96 paired runs) the clock cost the AI 17 points of Sam score (8.6 on against 25.6 off, the GitHub master level) while training moved 25.3 to 26.0; it is needed with `leader_war` for the Swiss S131 hold (runs/heldout/genes-table.md). Set 1 to measure the clock";
    production_threat_agreement: 1, 0, 1, "Hard: routine production values a ranged defender after normal weights when emergency or threat exposure sees a nearby visible army; emergency leaves shooter-only choices to routine scoring and routine defers Settlers under visible threat (0: former alternating passes)";
    purchase_generation_floor: 1, 0, 1, "Hard: buy the newest trainable unit in the needed upgrade line; only its immediate predecessor when the newest lacks strategic supply; save Gold when neither is affordable (0: prior orders)";
    modernization_upgrade: 100, 0, 200, "Value funded upgrade chains before optional purchases (0: idle upgrades only)";
    modernization_return: 100, 0, 200, "Value returning to own territory for a funded refit, charging travel and lost service (0: no refit routing)";
    named_fund: 0, 0, 1, "Off after the 2026-11-15 rules-40 purchase gate: from Sparta 8.9.57 T40 the designer seat won 47/48 against fund-on rivals versus 41/48 against fund-off rivals; Swiss hollow-Mongols war card fell 31/64 to 0/64. Hard: save toward one plan-named Gold fund for a campaign, upgrade wave or visible defence (0: ordinary purchase and income-turn hold policy)";
    capture_vp_accounting: 1, 0, 1, "Price city captures using the captor Germany Foundry goal (0: legacy projection)";
    declare_and_strike: 1, 0, 1, "Attack on the same turn as a valid surprise War declaration (0: wait until the next turn, for A/B comparison)";
    recover_hp: 60, 25, 90, "Recover field troops below this HP";
    ranged_survival_weight: 200, 0, 400, "Production-equivalent HP value of retreating or resting a shooter before committing fire (0: legacy fire-first and overwrite held rests)";
    hold_exchange_percent: 100, 50, 175, "Incoming damage tolerated relative to retaliation when defending";
    garrison_doom_percent: 100, 50, 150, "Retreat when certain incoming damage reaches this percent of garrison HP";
    supported_defence: 1, 0, 1, "Hard: hold a threatened city and order useful melee supporters into it (0: former orders)";
    war_commit_ratio: 100, 100, 250, "Campaign commitment multiplier on the personality power ratio, hundredths (100 = 1.0)";
    attack_ratio_percent: 100, 70, 150, "Multiplier on personality-derived local attack power ratio";
    counterattack_loss_percent: 25, 0, 60, "Maximum forecast loss chance for a counterattack or garrison sortie";
    melee_response_weight: 100, 0, 300, "Charge a melee duel for the enemy response at its post-kill and plausible pursuit positions (0: immediate duel only)";
    counterattack_win_percent: 75, 40, 100, "Forecast win chance sufficient for a sortie";
    staging_damage_percent: 100, 50, 200, "Safety charge on forecast damage during assault staging";
    escort_home_distance: 2, 0, 6, "Escort settlement sites farther than this from home";
    escort_search_distance: 6, 2, 12, "Maximum distance to recruit a free settler escort";
    site_terrain_weight: 20, 0, 200, "Hard: percent added to a candidate site's settlement value per point of terrain Production on its explored ring-one tiles that are unowned or ours (every owned ring-one tile is worked, so the whole ring is the new city's early Production, not just its best two tiles; the designer's new cities in recorded games out-produce Hard's through ring-one Hills and Forest); 0: off";
    settler_min_site_score: 60, 20, 120, "Minimum preferred site score when training and moving settlers";
    settler_max_site_distance: 10, 4, 20, "Maximum preferred settlement distance from home";
    settler_travel_penalty: 0, 0, 20, "Extra site score penalty per travel turn";
    settler_commit_percent: 150, 100, 225, "Hysteresis multiplier on an existing safe settlement destination";
    settler_commit_bonus: 20, 0, 80, "Additional score for an existing safe settlement destination";
    settle_patience: 20, 8, 32, "Standard turns before an unfounded settler chooses a nearby safe site";
    border_raid: 0, 0, 1, "Hard: optional wars may open on a valuable local field target near our cities instead of a city campaign (unless endangered); former hard-coded gate (climb program stage 1): Germany";
    campaign_contain_only: 0, 0, 1, "Hard: no campaign target except for a containment war (so no city campaign or diplomatic campaign); former hard-coded gate (climb program stage 1): Germany";
    scout_front: 1, 0, 1, "Hard: send one healthy idle unit to find a neighbour's nearby cities when none are known (0: never); former hard-coded gate (climb program stage 1): Germany 0";
    second_front_temperament: 0, -60, 60, "Percent added to the value of opening a second front against the top threat; former hard-coded per-faction table (climb program stage 1)";
    second_front_siege_price: 1, 0, 1, "Hard: price the least costly reachable siege against a walled top threat instead of requiring an open or damaged city (0: old soft-city veto)";
    neighbour_fall: 100, 0, 100, "Hard: percent of the recent neighbour-fall warning from the conqueror's public army used in city threat, defender reserves and alliance value while its wars continue (0: former defence orders)";
    neighbour_alarm: 1, 0, 1, "Hard (survive-him, 2026-11-20): the neighbour-fall warning also starts from a capture we only learn from public city counts plus the conqueror's units seen near us, grows by half for each capture he made in the last 20 turns, and stays on once his army reaches our border; used only by production (Walls, then defenders, ahead of economy builds); positioning, reserves and diplomacy keep the former warning (0: former production)";
    busy_leader_front: 1, 0, 1, "Hard: Scout toward the busy leader's seen border, then open a reachable containment front without a funded siege when its army is away and our cities are safe (0: former orders)";
    ally_defense_percent: 100, 0, 200, "Hard: keep Walls valuable against a stronger ally massing near our city, scaling the army's threat; 0 ignores allied armies as before";
    ally_vision_cost: 10, 0, 100, "Hard: score each own city whose vision an Alliance shares with a prospective partner; 0 keeps the old free-vision score";
    ally_passage_cost: 15, 0, 100, "Hard: price a prospective ally's passage into our land and the free exit after ten turns; 0 keeps the old free-passage score";
    ally_exit_discount_percent: 50, 0, 100, "Hard: discount Alliance security because either partner may leave free after ten turns; 0 keeps the old permanent-security valuation";
    ally_commitment_model: 1, 0, 1, "Hard: count actual war participants and direct defensive entrants; offensive allies join only by probability, rather than counting the whole Alliance chain (0: old whole-side army)";
    ally_offensive_join_percent: 50, 0, 100, "Hard: chance-weight the army of a direct ally asked to join a new offensive war";
    open_prize_radius: 10, 5, 16, "Hexes from our cities within which an open enemy city is an early prize (follow-up, assault wing, expedition force, early prize opening, march second-front test); former hard-coded threshold (climb program stage 1)";
    courier_raid_only: 0, 0, 1, "Hard: units raid exposed civilians but take no field battle more than 3 hexes from home, and a city strike skips a remote enemy-held target; former hard-coded gate (climb program stage 1): Mongols";
    walls_near_enemy_bonus: 0, 0, 400, "Added production value of Walls in a city within 8 hexes of an enemy city (the faction-bonus slot); former hard-coded bonus (climb program stage 1): Switzerland 150";
    bronze_bias: 0, 0, 300, "Research value added to Bronze Working; former hard-coded faction bias (climb program stage 1): Germany 100, Celts 70";
    irrigation_bias: 0, 0, 300, "Research value added to Irrigation; former hard-coded faction bias (climb program stage 1): Celts 70";
    currency_bias: 0, 0, 300, "Research value added to Currency (faction priority slot); former hard-coded faction bias (climb program stage 1): Mongols 100";
    banking_bias: 0, 0, 300, "Research value added to Banking; former hard-coded faction bias (climb program stage 1): Mongols 60";
    feudalism_bias: 0, 0, 300, "Research value added to Feudalism; former hard-coded faction bias (climb program stage 1): Sparta 180, Byzantium 60";
    chivalry_bias: 0, 0, 300, "Research value added to Chivalry; former hard-coded faction bias (climb program stage 1): Sparta 120";
    flower_wars_value: 100, 0, 300, "Aztec captive value as a percent of the nearest city growth cost; rivals value denying captives with ranged units and Walls";
    egypt_food_buildings: 0, 0, 1, "Hard Egypt: value buildings by turns of surplus Food and lost growth, and wait for growth after the Royal Granary (1: on)";
    egypt_grow_payback: 30, 0, 200, "Egypt on the Nile: a city chooses Grow instead of a building, wonder or improvement when the Food the next population costs is repaid by its extra surplus within this many turns (0: never grows by choice)";
    scribe_bias: 0, 0, 300, "Research value added to Writing, Code of Laws and Philosophy; former hard-coded faction bias (climb program stage 1): Athens 140";
    currency_tech_bonus: 0, 0, 200, "Research value added to Currency in its tech value; former hard-coded faction bonus (climb program stage 1): Switzerland 20";
    industry_tech_bonus: 0, 0, 300, "Research value added to Iron Working and Industrialization; former hard-coded faction bonus (climb program stage 1): Germany 100";
    reveal_tech_bonus: 150, 0, 400, "Hard: research value added to the technology that reveals a strategic resource (Iron Working, Gunpowder, Industrialization, Combustion, Autonomous Systems), so the empire learns where the era's deposits are early in the era, as the designer does (2026-11-07)";
    reveal_tech_first: 1, 0, 1, "Hard: the technology that reveals a strategic resource is researched in the first tier with the unit and defence technologies when the empire is threatened, instead of after them (the designer learns each era's reveal early; 2026-11-07)";
    law_tech_bonus: 0, 0, 300, "Research value added to Code of Laws in its tech value; former hard-coded faction bonus (climb program stage 1): India 80";
    crop_tech_bonus: 0, 0, 300, "Research value added to Crop Rotation; former hard-coded faction bonus (climb program stage 1): Celts 100";
    veteran_infantry_bonus: 0, 0, 300, "Production value added to a veteran-infantry unit while the army is below quota; former hard-coded faction bonus (climb program stage 1): Byzantium 100";
    infantry_bonus: 0, 0, 300, "Production value added to an infantry unit while the army is below quota; former hard-coded faction bonus (climb program stage 1): Sparta 100";
    roadworks_war_base: 0, 0, 2000, "Roadworks value added before a target is committed; former hard-coded faction value (climb program stage 1): Sparta 1000, Mongols 240";
    roadworks_war_per_enemy: 0, 0, 300, "Roadworks value per enemy city in range (at most 4); former hard-coded faction value (climb program stage 1): Sparta 100, Mongols 40";
    campaign_range: 14, 6, 24, "Hard and others: hexes (plus a tenth of the conquest pressure) within which a staged campaign picks its target city; former hard-coded constant (climb program stage 1)";
    early_campaign_value: 0, 0, 400, "Hard: campaign plan value added in the Ancient era, half in the Classical era, and none later; a faction's early veteran advantage can make a funded rush worth its opportunity cost";
    target_wall_cost: 0, 0, 500, "Hard: extra defence cost of a city with Walls when nominating a conquest target; value of the city can still outweigh this cost";
    campaign_wall_cost: 0, 0, 400, "Hard: extra plan-value cost of attacking a city with Walls; a valuable and winnable siege can still win the plan choice";
    contain_extra_range: 6, 0, 12, "Hexes added to campaign_range for a containment march; former hard-coded constant (climb program stage 1)";
    switch_margin: 40, 0, 120, "Plan value a new plan must beat the running one by (doubled before its review turn); former hard-coded constant (climb program stage 1)";
    min_seat_war_odds: 15, 0, 50, "Capture odds in percent below which a campaign is not offered by the plan layer; former hard-coded constant (climb program stage 1)";
    recent_fight_turns: 8, 2, 20, "A war whose units fought within this many turns counts as active for the peace redeclaration rule; former hard-coded constant (climb program stage 1)";
    march_turns: 6, 2, 12, "Turns of march the campaign forecast counts a field army as in reach of an enemy city; former hard-coded constant (climb program stage 1)";
    siege_rounds: 4, 1, 8, "Rounds past the arrival the siege forecast plays when siege_window is 0; former hard-coded constant (climb program stage 1)";
    approach_turns: 2, 1, 4, "Turns of approach the strike forecast allows a unit to reach a city; former hard-coded constant (climb program stage 1)";
    retaliation_radius: 6, 3, 10, "Hexes from home within which an enemy army counts as retaliation for a strike; former hard-coded constant (climb program stage 1)";
    siege_reach: 8, 4, 14, "Our units this close to an enemy city count toward a siege of it (war_siege); former hard-coded constant (climb program stage 1)";
    raid_arrival_weight: 100, 0, 300, "Hard field raids: count only troops arriving together and price their forecast losses before comparing force with the target; 100 fully prices the reply (0: nearby nominal force)";
    raid_reach: 6, 2, 12, "Enemy field units this close to our army are raid targets when no city siege is on; former hard-coded constant (climb program stage 1)";
    ally_progress_limit: 70, 40, 95, "Leave an Alliance with a team at this public victory progress, and never sign one with it; former hard-coded constant (climb program stage 1)";
    ally_offer_progress: 60, 30, 90, "Never sign a new Alliance with a team at or above this victory progress; former hard-coded constant (climb program stage 1)";
    bloc_size_penalty: 60, 0, 200, "Alliance score lost for each team an Alliance adds to our bloc beyond a pair (the bloc-size cost, 2026-10-23; 0 turns it off)";
    war_lean: 150, 0, 400, "Research value added to a military tech while the plan is a war; former hard-coded constant (climb program stage 1)";
    peace_lean: 60, 0, 300, "Research value added to a growth tech while the plan is peace; former hard-coded constant (climb program stage 1)";
    research_hysteresis: 125, 100, 200, "Percent a new research choice must beat the current one by to switch; former hard-coded constant (climb program stage 1)";
    scout_find_turns: 40, 10, 100, "Last turn of value-scored scouting for a rival's first city (scout_find); former hard-coded constant (climb program stage 1)";
    rush_window_turn: 120, 40, 250, "Last turn of a Rush strategy's opening window (no Alliance with a neighbour, rush production, army extra after it); former hard-coded threshold (climb program stage 1)";
    opening_chain_turn: 80, 30, 200, "Turn before which the capital's peaceful opening Settler chain counts as early; former hard-coded threshold (climb program stage 1)";
    rush_read_turn: 40, 10, 100, "Turn before which a rival's public army growth is read as a rush (with army_growth_trigger); former hard-coded threshold (climb program stage 1)";
    rival_radius: 12, 6, 20, "Hexes within which a rival's city counts as a neighbour for the rush and walls reads; former hard-coded threshold (climb program stage 1)";
    work_plan_turn: 120, 60, 250, "Turn from which a Great Work plan steers production, research and purchases; former hard-coded threshold (climb program stage 1)";
    victory_close_turn: 80, 30, 200, "Turn from which being within four VP of the target counts as pursuing victory; former hard-coded threshold (climb program stage 1)";
    endgame_victory_turn: 160, 80, 300, "Turn from which the endgame victory plan starts when within two VP of the target; former hard-coded threshold (climb program stage 1)";
    conquest_war_bonus: 80, 0, 300, "Conquest city value added when we are already at war with its owner; former hard-coded weight (climb program stage 1)";
    conquest_wounded_bonus: 100, 0, 300, "Conquest city value added when its visible HP is below full; former hard-coded weight (climb program stage 1)";
    conquest_nearby_weight: 20, 0, 80, "Conquest city value per own military unit within 5 hexes of it; former hard-coded weight (climb program stage 1)";
    strike_eagerness: 0, -60, 60, "Percent added to the eagerness for an early strike (60 plus aggression plus this); former hard-coded per-faction table (climb program stage 1)";
    ruin_bonus: 40, 0, 120, "Known ruin value for Scouts and available units (Scout frontier scores are lower-is-better); former hard-coded gate (climb program stage 1): Byzantium and Athens 80";
    ruin_collect: 1, 0, 1, "Hard: available military units and idle Settlers collect a known ruin in or beside their own territory (0: only Scouts do)";
    approach_scout: 0, 0, 1, "Hard: a lone first-city seat reveals the approach around home before chasing distant ruins while a Byzantium player lives; former hard-coded gate (climb program stage 1): Sparta";
    approach_scout_turn: 24, 4, 60, "Last turn of approach_scout; former hard-coded threshold (climb program stage 1)";
    recon_weight: 0, 0, 6, "Scout frontier score per hex from home (capped at 12): distant reconnaissance is worth this much; former hard-coded gate (climb program stage 1): Mongols 2";
    scout_priority_bonus: 0, 0, 400, "Added production value of the first Scout in a city with unexplored land; former hard-coded bonus (climb program stage 1): Byzantium, Mongols and Athens 150";
    toll_market_bonus: 0, 0, 200, "Added production value of a Market; former hard-coded bonus (climb program stage 1): Switzerland 60";
    army_war_extra: 1, 0, 10, "Armed units wanted beyond one per city while a conquest or containment war is on; former hard-coded constant (climb program stage 1). Shared default 1 since 2026-11-06 (was 5; strategy-gap quota ablation, docs/sim-results/strategy-gap-1105/: 4,096 eight-empire Hard games, own effect Byzantium +3.2 pp, Celts +2.5, Mongols +1.6, Sparta and India null, less upkeep; with walls_garrison the human-seat gate's rivals arm -4.15 pp [-6.35, -1.90]). Athens keeps its built-in 0";
    army_rush_extra: 4, 0, 10, "Armed units wanted beyond one per city for a Rush strategy (one more after turn 120); former hard-coded constant (climb program stage 1)";
    army_hard_extra: 2, 0, 8, "Hard at peace with no threat: armed units wanted beyond one per city; former hard-coded constant (climb program stage 1): Sparta 4";
    army_military_bonus: 1, 0, 3, "Hard at peace with no threat: units added to that quota when the military knob is 70 or more; former hard-coded constant (climb program stage 1): Sparta 0";
    army_banker: 0, 0, 1, "Hard at peace with no threat: the army quota is max(cities, 2) plus one under pressure, to leave the treasury alone; former hard-coded gate (climb program stage 1): Switzerland";
    chain_press_turn: 130, 40, 300, "Last turn of chain_press; former hard-coded threshold (climb program stage 1)";
    open_followup_turn: 84, 30, 200, "Last turn of open_followup; former hard-coded threshold (climb program stage 1)";
    opening_strike_turn: 70, 20, 150, "Last turn a lone first city strikes an open Byzantium city (lone_city_strike); former hard-coded threshold (climb program stage 1)";
    lone_city_sortie_turn: 48, 10, 120, "Last turn of the lone-city sortie (lone_city_strike); former hard-coded threshold (climb program stage 1)";
    early_prize_turn: 110, 30, 250, "Last turn the capital builds melee first for early_prize_opening; former hard-coded threshold (climb program stage 1)";
    conquest_enabled: 1, 0, 1, "Hard: the AI nominates a conquest city at all (0: never); former hard-coded gate (climb program stage 1): Germany never conquered";
    conquest_min_cities: 2, 0, 6, "Hard: own cities needed before a conquest city is nominated; former hard-coded gate (climb program stage 1): every faction but Sparta needed two";
    conquest_bank_goal: 0, 0, 1, "Hard: bank for the treasury goal and nominate a conquest city only against a runaway; former hard-coded gate (climb program stage 1): Switzerland";
    conquest_runaway_percent: 45, 20, 90, "Hard: public VP share of its target at which a leader counts as a runaway for conquest_bank_goal; former hard-coded threshold (climb program stage 1)";
    conquest_walled_forecast: 0, 0, 1, "Hard: a walled city above a third HP is not a conquest target unless a campaign forecast takes it; former hard-coded gate (climb program stage 1): Mongols";
    lone_city_strike: 0, 0, 1, "Hard: with one city, strike a nearby open Byzantium or India city before settling (conquest nominee, sortie); former hard-coded gate (climb program stage 1): Sparta";
    lone_city_strike_radius: 7, 3, 12, "Hard: hexes within which a lone first city looks for an open Byzantium city (conquest nominee, sortie, early prize, approach scouting); former hard-coded threshold (climb program stage 1)";
    early_walled_skip: 0, 0, 1, "Hard: in the first third of the game, no conquest nominee or containment march on a walled city; former hard-coded gate (climb program stage 1): Sparta";
    conquest_home_distance: 14, 6, 24, "Hard: a conquest city is nominated only within this many hexes of our nearest city; former hard-coded threshold (climb program stage 1)";
    conquest_path_len: 20, 8, 32, "Hard: and only if the path to it is at most this many steps; former hard-coded threshold (climb program stage 1)";
    siege_exempt: 0, 0, 1, "Hard: with war_siege, a war's army never sieges (stays on the storm and raid policy); 1 restores the former Germany, Sparta or Celtic exclusion for a comparison arm";
    contain_march_outlook: 0, 0, 1, "Hard: a containment march is staged only on a city the campaign outlook can take; former hard-coded gate (climb program stage 1): Sparta";
    contain_stack_weight: 0, 0, 100, "Hard: containment march priority per own military unit within 2 hexes of the target city; former hard-coded weight (climb program stage 1): Sparta 35";
    plan_persist_skip: 0, 0, 1, "Hard: the plan layer prices no grind past a forecast miss for a running campaign (campaign_persist still acts elsewhere); former hard-coded gate (climb program stage 1): Sparta";
    march_ignores_dead_war: 0, 0, 1, "Hard: a campaign march ignores a second war against a team with no cities when judging a second front; former hard-coded gate (climb program stage 1): Sparta";
    march_ignores_second_war: 0, 0, 1, "Hard: a march on a nearby open target (chain press, valued or India city) ignores a second war whose army is not near home; former hard-coded gate (climb program stage 1): Sparta";
    chain_press: 0, 0, 1, "Hard: with three cities or more, a field wing presses a newly founded open city while one regiment holds the last capture; former hard-coded gate (climb program stage 1): Sparta";
    open_followup: 0, 0, 1, "Hard: with two cities after a first capture, the field army follows up on the next nearby open city (campaign, sortie); former hard-coded gate (climb program stage 1): Sparta";
    assault_wing: 0, 0, 1, "Hard: the early assault wing is trained for a nearby open prize (needs assault_wing_forecast); former hard-coded gate (climb program stage 1): Sparta";
    walled_prize: 0, 0, 1, "Hard: a walled city whose capture can change the race (capital, population 3, title, last city) is worth paying through the wall; former hard-coded gate (climb program stage 1): Sparta";
    engaged_assault_keep: 0, 0, 1, "Hard: a funded assault already engaged on a damaged open city is kept while its capture force is present; former hard-coded gate (climb program stage 1): Sparta";
    early_prize_opening: 0, 0, 1, "Hard: the opening plays for an early open prize city (settler timing, assault wing, melee first) instead of the ordinary muster; former hard-coded gate (climb program stage 1): Sparta";
    army_item_bonus: 0, 0, 300, "Added production value of an army unit while the army is below its quota; former hard-coded bonus (climb program stage 1): Sparta 100";
    war_shrine_drive: 0, 0, 1, "A War Shrine is worth 240 plus military weight while the army is below its quota; former hard-coded gate (climb program stage 1): Sparta";
    reserve_deficit_turns: 10, 0, 40, "Standard-turn deficit runway kept before discretionary purchases";
    surplus_upkeep_turns: 60, 10, 120, "Standard-turn upkeep runway required for surplus army purchases";
    surplus_purchase_percent: 200, 100, 350, "Gold required relative to a surplus unit purchase, plus reserves";
    purchase_min_value: 0, 0, 150, "Minimum production decision value for a discretionary purchase";
    purchase_savings_turns: 0, 0, 12, "Save for a higher-value legal building purchase reachable within this many turns (0: current affordable-only buying)";
    frontline_purchase_radius: 5, 2, 9, "Visible enemy approach distance that triggers defensive purchases";
    research_military_percent: 100, 40, 200, "Weight military technology preferences";
    research_economy_percent: 100, 40, 200, "Weight economic technology preferences";
    research_science_percent: 100, 40, 200, "Weight science technology preferences";
    production_military_percent: 100, 40, 200, "Weight military unit production preferences";
    production_settler_percent: 100, 40, 200, "Weight settler production preferences";
    production_building_percent: 100, 40, 200, "Weight building production preferences";
    leader_pressure_hard_percent: 40, 15, 75, "Public VP progress that starts Hard's leader pressure";
    leader_pressure_percent: 60, 25, 90, "Public VP progress that starts other difficulties' leader pressure";
    leader_siege_percent: 60, 30, 90, "Public VP progress that permits a riskier Hard siege";
    defend_margin_percent: 200, 100, 300, "Hard: defensive weight a threatened city gathers, percent of its attackers' weight";
    relieve_radius: 3, 0, 5, "Hard: healthy defenders this close to an attacked city stand in its attackers' reach (0: off)";
    open_city_radius: 6, 0, 12, "Hard: free field units this close go for an unguarded enemy city at war (0: off)";
    contain_march_radius: 12, 0, 20, "Hard: the same for the target of a containment war";
    protect_defenders: 1, 0, 1, "Hard: units defending an attacked city disband last for upkeep";
    faction_threat_prior: 1, 0, 1, "Hard: a rush-capable neighbour by its public rules triggers border defence";
    endgame_push: 1, 0, 1, "Hard endgame: buy a title the treasury covers, keep spawn room, guard a live hoard, war for a title one point short";
    late_capture_value: 100, 0, 200, "Hard: value a capture that starts our VP hold or breaks a rival hold even if the win-chance softmax is saturated, and value a feasible city-by-city elimination path in a late duel; percent of the secondary race value (0: off)";
    sentry: 1, 0, 2, "Hard: a Scout stands sentry by the nearest city of each rush-capable or aggressive neighbour (2: of every bordering rival)";
    sentry_needs_sites: 1, 0, 1, "Hard: while an empire with fewer than four cities knows no good city site, its Scouts explore instead of standing sentry or watching approaches (0: off; Mongols 8.9.73.1 review: the Swiss sentry stood by his capital T7-T63 and the Swiss never settled)";
    army_growth_trigger: 150, 0, 400, "Hard: before turn 40, a rival within 12 hexes whose public army exceeds ours by this percent is a border threat (0: off)";
    rush_walls: 1, 0, 1, "Hard: a rush-capable rival within 12 hexes (or unplaced, in small lobbies) brings the Walls tech, a shooter and Walls first";
    veteran_drill: 3, 0, 6, "Hard Sparta: this many units nearest the retire level take favourable fights and hunt barbarians (0: off)";
    roman_founders: 1, 0, 1, "Hard Rome: seek the first melee promotion, train replacement melee units, found immediately on a safe legal site with a spare veteran, and keep the last threatened-city defender";
    roman_soldier_route: 1, 0, 1, "Hard Rome (with roman_founders): a veteran founder walks to its site as a soldier, not a Settler: no Settler capture-danger detour and no trip-risk price (it cannot be captured); the site's own danger gates still apply";
    roman_veteran_retreat: 1, 0, 1, "Hard Rome (with roman_founders): a wounded veteran (level 1+) under threat steps out of reach to heal, since it can found only at full health, unless it is the threatened city's last defender";
    roman_promotion_value: 100, 0, 300, "Hard Rome (with roman_founders): each first promotion a siege should earn (3 XP per city attack a turn, 10 for the capture) is priced as the city it founds, in strike gains and campaign plans; percent of a new city's value (0: off)";
    roman_hunt_radius: 8, 0, 30, "Hard Rome (with roman_founders): promotion hunts stay within this many hexes of an own city, there is no frontier reconnaissance, and the hunt leaves the army to a planned campaign, so a strike force can gather (0: the old unbounded hunt and reconnaissance)";
    roman_early_war: 1, 0, 1, "Hard Rome (with roman_founders): with no level-1 melee veteran, one city is enough to nominate a conquest city, so the campaign planner prices an early war on the softest reachable neighbour (its forecast and the veteran founders it earns) instead of waiting for a second city Rome can only found with a veteran (0: Rome needs `conquest_min_cities` like everyone)";
    roman_found_first: 1, 0, 1, "Hard Rome (with roman_founders): supported-defence staging leaves this turn's veteran founder to its founding order or route (0: staging recalls it to the city, overwriting FoundCity; rome-founders trace 2026-11-21, game 930027 T44 and T63)";
    roman_finish_siege: 1, 0, 1, "Hard Rome (with roman_founders): while a campaign is live the one-city conquest gate stays open after the first promotion, and while its capture forecast holds the committed units neither step out to heal nor leave to found unless the capture still forecasts without them (0: the first promotion drops the conquest city and the veterans walk home mid-siege; rome-siege repro 2026-11-21: 76 of 186 one-city sieges abandoned, 69 of the departing orders the veteran retreat)";
    rome_colonia: 1, 0, 1, "Hard Rome (with roman_founders; Rules 54 goal, 2026-11-22; `goal_colonia.rs`): while the Colonia Victrix is open, a level-3 melee veteran is never spent on an ordinary city: it founds the Colonia on the most defensible reachable site beside a rival's border (near Rome's own army and cities, away from rival armies), or waits as a soldier until one opens; while no unit is level 3 the most experienced level-2 melee unit is kept from founding; the founded Colonia is garrisoned, walled and defended first like the Courier Hall's city (0: veterans found ordinary cities as before and the Colonia comes only by chance)";
    egypt_tomb: 1, 0, 1, "Egypt (Rules 54 goal, 2026-11-22; `goal_tomb.rs`): start the royal tomb when its 2 VP take the team to the target with time to hold them or the game is 60% gone, the approaches are clear and the would-be tomb city is garrisoned; pause it while an enemy army is near a city (unless two turn ends from standing) and resume when it clears; guard the tomb city first like the Courier Hall's city (0: the tomb is never started)";
    goal_city_capture: 100, 0, 300, "Every empire (Rules 54 goals, 2026-11-22; `goal_cities.rs`): percent weight on the 2 VP a public Colonia Victrix or royal tomb city swings when captured (the Courier Hall's valuation, its founder's retake higher), on a working Egypt's cities that build nothing until the tomb stands, and on the city its tomb will stand in (priced as a Great Work under construction) (0: these cities are priced as ordinary cities)";
    keep_walls: 1, 0, 1, "Hard: Walls under way are finished, never swapped for an emergency unit (playtest 8.4)";
    army_ratio_walls: 200, 0, 400, "Hard: a rival within 12 hexes whose public army is this percent of ours is a border threat at any time (0: off; playtest 8.4)";
    threat_aim_slack: 1, 0, 5, "Hard: an army threatens own cities this many hexes beyond its nearest one (5: all in reach)";
    keep_settler: 40, 0, 100, "Hard: a Settler this percent built is finished, not swapped for an emergency unit, unless the city is actually threatened (0: off; playtest 8.4)";
    settler_pipeline: 2, 1, 4, "Hard: Settlers alive or in production at once before another is started";
    settler_open_cities: 4, 2, 8, "Hard: cities plus Settlers in the pipeline below this claim sites before developing each colony";
    settler_priority: 0, 0, 4000, "Hard at peace with every rival: added to a routine Settler's value, over the army and building quotas it competes with";
    settler_chain: 1, 0, 1, "Hard: capital prioritizes a Settler pipeline when sites are open and its existing threat assessment is quiet";
    settler_chain_priority: 500, 0, 10000, "Hard: extra capital production score for each quiet-opening Settler after Granary";
    settler_raid: 1, 0, 1, "Hard: consider declaring war to capture an exposed rival Settler within its configured reach";
    settler_raid_range: 2, 0, 2, "Hard: exposed rival Settler raid reach (0: adjacent; 1: this turn; 2: this turn or next)";
    settler_safety: 0, 0, 1, "Hard multiplayer: forecast interception and guard or recall exposed Settlers";
    founding_survival_weight: 0, 0, 400, "Percent of a new city's site value charged for projected pre-Walls ranged damage from a visible war enemy (0: established site scores)";
    peace_settler_capture: 0, 0, 200, "Hard multiplayer: scale the chance of a surprise war for a free Settler; 0 keeps the prior settler routing policy";
    settler_risk_escort: 40, 1, 200, "Trip exposure at which an exposed Settler seeks an escort or waits";
    settler_risk_avoid: 100, 1, 300, "Trip exposure at which a site becomes a last resort";
    heir_route_risk_weight: 500, 0, 1000, "Heir refuge score penalty for exposed tiles on the actual movement route, valuing a lost Heir as a VP swing (0: destination-only refuge)";
    lone_city_radius: 4, 2, 8, "A lone surviving city builds no Settler while an army of a rival at war stands this close (playtest 8.4: eight kept a one-city empire at one city for 94 turns)";
    peace_follows_plan: 1, 0, 2, "Hard + Search: a searched plan that chose Peace with a team makes peace with it: no older veto, and the score floor is zero (playtest 8.4). 2: plain Hard's plan too (it never searches; playtest 8.9.17: 6 of the Celts' 11 refusals of Athens's Peace came under their own Peace plan, on the score)";
    peace_scout_danger: 0, 0, 1, "Hard: a Scout of a rival at Peace marks the tiles it could reach as dangerous to Settlers (0: only if surprise-war capture risk reaches settler_risk_escort; at war always counts)";
    peace_exposure_percent: 0, 0, 300, "Weight a peaceful rival's visible army by its chance of declaring this turn in tactical exposure and city pressure (0: war enemies only)";
    threat_horizon_turns: 0, 0, 4, "Forecast rival army arrival by passable path and movement turns when funding city defence (0: the old five-hex trigger)";
    assessment_interval: 0, 0, 1, "Refresh a rival assessment each turn when its visible army is within two movement turns of a city (0: every five turns)";
    fog_army_forward_percent: 70, 0, 100, "Share of unseen public army strength sampled toward the active front of a wartime rival or the top threat (0: all unseen units at home)";
    fog_escort_percent: 0, 0, 300, "Settler interception risk from unseen route tiles near a known rival border, priced into escorts and site choice (0: visible and remembered units only)";
    army_threat_weight: 0, 0, 300, "Uncapped border-army threat relative to our field strength and the public army title (0: old 20-point cap)";
    field_strike: 0, 0, 300, "Win-chance value of preempting a massed rival field army near our city when the local battle forecast pays for a new war (0: no FieldStrike plan)";
    scout_threat: 0, 0, 300, "Count enemy Scouts by their low threat_power and their ability to capture broken cities and civilians (0: exclude Scouts from city threat sums)";
    walled_defense_percent: 0, 0, 300, "Value continued defenders for a walled city when the visible siege forecast still takes it (0: Walls end threat-driven production)";
    intel_value: 0, 0, 300, "Value fresh knowledge of a rival's army, increasing with public strength and age of its last sighting (0: no explicit information value or four-turn warning)";
    sentry_posts: 0, 0, 300, "Post Scouts on likely threat approaches where sight and hills reveal an army before arrival, and revisit stale sightings (0: only the old rush-neighbour sentries)";
    counter_composition: 0, 0, 300, "Extra production value for units that counter the visible or recently seen composition of an approaching rival army (0: old class mix only)";
    #[serde(alias = "tollkeeper_market_priority")]
    swiss_market_priority: 300, 0, 600, "Hard Switzerland: add this production value to Markets while the treasury goal is pending (0: off)";
    strain_percent: 20, 0, 100, "A rival whose weighted decadence reaches this percent is straining: its cities rank higher as campaign targets and are worth a slightly riskier siege (0: off)";
    strain_priority: 400, 0, 2000, "Campaign priority added per hundred percent of a straining rival's weighted decadence";
    decadence_weight: 200, 0, 1000, "Percent of a rival's decadence counted as burden (200 means 10% decadence counts as 20% burden; with decadence off, burden is zero)";
    movement_wait: 1, 0, 1, "The movement preflight lets two movers that claim one tile in one round take turns as the resolver does (the lower unit id first, the other waits a round with its moves); 0 cancels the lower-priority mover's whole turn instead";
    goal_open_end: 1, 0, 1, "A game with no turn limit has no last turn to plan a goal against: 1 plans as if the game goes on until someone wins; 0 keeps the old fiction (the end a tenth of the expected length ahead, so past turn 200 no build of more than 22 turns pays for a faction goal)";
    hall_decisive: 1, 0, 1, "Hard Mongols: when the Courier Hall's 2 VP take the team to the target its build starts at once, in the city that repeats a unit too, whatever the open land's exposure (0: only with the Hall city garrisoned, no army or fire outmatching ours, and an idle city)";
    objective_horizon: 160, 20, 600, "World Ages: Age cards needing more than this many turns of effort are not pursued";
    objective_seek: 1, 0, 1, "World Ages: 0 ignores Age cards entirely (no production bonus, no effort estimate, no capture value for a live card), so how often empires meet them by accident can be measured; 1 pursues them";
    objective_bonus_percent: 60, 0, 300, "World Ages: added to the value of a production item that advances the chosen Age card (the production bonus uses at least 500, as World Ages always did; the capture weight uses this value / 20)";
    cheapest_objective: 1, 0, 1, "Hard: rank every open, unscored Age card by its one VP over estimated completion time and production cost from PlayerView; value each still-needed build against the city's other production choices (0: legacy effort and percentage bonus). Default 1 since 2026-11-04 (lead decision: Codex left it off as inconclusive, but the bar is rivals not significantly weaker, as for found_now): in the 2,048-game human-seat benchmark rivals-only 465 against control 468 wins, seat-only 486; the Aztec Lab median 770 -> 840 and AI-only era scores +6% (old era-objective rules) (3,772 -> 4,002 in 200 paired games)";
    capture_completes_count: 1, 0, 1, "VP v2: a capture that completes a live city-count or territory objective (one city short; the city's own tiles cover the tiles still needed) counts as that objective's VP in campaign and conquest target value, like a Fight objective (playtest 8.9.6 Swiss: India sat at 9 VP for T199-T202, 2 tiles short of the Industrial territory count and 11 of 12 cities, with the strongest army; 0: off; default 1 since 2026-10-21: human-seat benchmark seat-only -0.4 pp [-1.0, +0.1], opponents-only -1.0 [-1.9, -0.1], in the 8.9.6 Swiss branches with every AI on the gene his seat wins 46/60 -> 0 from T160 and 50/60 -> 0 from T175)";
    greece_captive_attack: 100, 0, 300, "Hard: percent of the research value of the free technology gained by capturing an Athenian city in city target and campaign scores";
    greece_captive_defend: 100, 0, 300, "Hard Athens: percent of the free technology a rival gains in the value of holding a threatened Athenian city";
    vow_hold: 0, 0, 1, "Hard India: while the closed country's vow holds or scores within 15 turns, accept and offer Peace with anyone at war (a closed country starts no war of its own; a treaty ends a costly war), and while the vow takes the empire to, or one point from, its target keep two defenders per city (playtest 8.9.6 Swiss: India refused Athens's Peace six turns before the vow scored, lost Himeji; 0: off)";
    era_entry_percent: 25, 0, 100, "VP v2: added to a technology's value when it opens the next era (its objectives)";
    era_transition_scoring: 0, 0, 1, "Value the last missing technology in the current era as the actual era transition (0: legacy candidate-era comparison)";
    contain_leader_percent: 70, 0, 100, "Hard: the public VP leader at this share of its target, strictly ahead of every other team, can be contained (0: off; playtest 8.4)";
    contain_share: 250, 0, 1000, "Hard: permille of the leader's win chance a containment war at full strength and reach takes away";
    contain_campaign: 1, 0, 1, "Hard: a containment war marches on the target's softest VP-bearing city and runs a campaign there (playtest 8.4)";
    coalition_alliance: 1, 0, 1, "Hard: an Alliance between two teams at war with the same leader weighs upkeep against the leader's win instead of applying the 20-turn treasury veto (playtest 8.4)";
    search_race: 1, 0, 1, "Hard+Search: a containment option's rollout mean gains its analytic leader's race, the part of its worth past the rollout horizon (playtest 8.4)";
    campaign_march_patience: 0, 0, 24, "Hard Sparta: turns a chosen, still-unwalled Campaign may stage despite a temporarily missing siege forecast";
    planned_stage: 0, 0, 1, "Hard Sparta: stage the field force toward a chosen Campaign city before its local siege can open";
    contain_commit: 1, 0, 1, "Hard Sparta: a new Contain war requires a funded siege route and its march uses forecastable targets";
    contain_min_odds: 30, 15, 70, "Hard Sparta: minimum calibrated capture odds for declaring a new Contain war";
    extent_guard: 1, 0, 1, "Hard: a plan that projects a captured, lost or founded city moves its owners' population and territory only when the win estimate actually reads those facts (under VP v2 it does not: both are 0 for every team, and the projection used to hand the capturer a 100% share of both, pricing every Campaign as a near-win; 1 also allocates a captured city's economy over the owner's seen cities; 0: the old projection; idea 24d)";
    stale_intel: 15, 0, 60, "Hard: a rival city not in sight for more than this many turns (the clock starts when the AI first notes the city) is priced as aged (Walls once its owner is past the first era, population grown from the turns away, full HP) and cannot be declared on until the AI sees it again; Scouts revisit it first (0: off; idea 24a, 24b)";
    stale_nominee: 1, 0, 1, "Hard: nominate a fresh reachable rival city ahead of a stale one; if the best value nominee is stale, send an idle Scout or the fastest idle military unit (cheapest on a speed tie) to a safe sighting tile (0: preserve the old nomination and orders)";
    strike_range_exit: 1, 0, 1, "Hard: when no plan attacks a firing enemy city, units within its two-hex strike range walk out of it at once instead of standing there; not for Sparta or Celts (0: off; idea 24c)";
    capture_economy_percent: 200, 0, 200, "Hard: credit a captured city's share of its owner's public Production plus Gold economy in a campaign forecast";
    sack_price: 1, 0, 1, "Hard, Celts: a capture is priced as the sack it is (0: as a kept city, the Celts' old way). The Gold taken (population x 60, the forecast amount) buys units, the victim loses its city, people, land and output, and the VP its held goals lose with the city count; the Celts gain no city, people, land or output, and no milestone (user 2026-10-22, coach review of the 8.9.7 Celts win)";
    sack_commit: 1, 0, 1, "Hard, Celts: a new Contain war is declared only with a funded siege route to a city the siege forecast can take, as Sparta's contain_commit does (0: any Contain war the plan prices well; coach review of the 8.9.7 Celts win: Celtic seats that declared wars and never sacked won 2% against 6% without the wars)";
    sack_loss_percent: 60, 0, 200, "Hard, Celts, strike value: percent of a city's keep value a sack counts as the victim's lost output, in place of the keep value itself (a kept city is worth that to its captor; a razed one is worth the harm to its owner)";
    raze_value: 1, 0, 1, "Hard (decision audit 2026-10-31: Hard razed captured cities in four of the designer's five wins; he kept them all): keep or raze a captured city by value, its output over 40 turns from our yield per population, growth and the live gain-cities objectives it counts for, less the Decadence its output adds, against the Gold to defend it or the forecast loss to its former owner; razing pays population x 10 Gold (0: the pressure gate and city_keep_value comparison); default 1 since 2026-10-31: 2,048-game gate (batch w29-1031-013833, control 358/2,048) with buy_hold 3: seat-only +5.03 pp [+2.98, +7.08], opponents-only +3.47 [+1.48, +5.46]; alone, opponents-only +2.10 [+0.44, +3.76]";
    buy_hold: 3, 0, 3, "Hard (decision audit 2026-10-31: in the designer's seat Hard made 80 to 396 purchases a game, he made 0 to 7): a purchase competes with keeping the Gold. Routine purchases (buildings, improvements, Settlers, surplus army) spend only the Gold above its later uses, the upgrade bill for units whose next form is researched or due within 10 turns and, at war or under threat, a defender for each threatened city (two at most), capped at 25 turns of income; upgrades, rescues, door defense, threat, victory, goal and hoard purchases don't see it (2: readiness and frontline defenders also leave it unless an attack can reach the city this turn; 3: it also banks the army spike, the dearest unit we can buy now for each of up to four cities); default 3 since 2026-10-31: 2,048-game gate (batch w29-1031-013833, control 358/2,048) with raze_value 1: seat-only +5.03 pp [+2.98, +7.08], opponents-only +3.47 [+1.48, +5.46]; alone, opponents-only +2.54 [+0.53, +4.55]";
    raze_pressure_percent: 130, 0, 400, "Hard: enemy counterattack pressure relative to the local garrison above which a captured city may be razed after loot, denial and keep value are compared (0 permits consideration under any pressure)";
    supply_value_percent: 100, 0, 300, "Strategic supply (user, 2026-10-19: resources are a reason for war): a known strategic deposit (each era's resource) is worth this percent of the Production its supply would put to work, idle users first, then cities ready for the era's key building and units waiting to upgrade; counted in campaign targets (and the owner's loss), city sites and Mines (0: deposits are only tiles)";
    settlement_claim_timeline: 0, 0, 1, "Count only newly claimed deposits at a settlement, discounting outer rings until border growth (0: legacy full-radius immediate supply)";
    industry_priority: 300, 0, 1000, "Strategic supply: a Forge or Factory with the supply it uses spare is worth this many production points per Production per turn it yields, on top of its payback value, so a city with a Workshop and spare Iron (or a Forge and spare Coal) builds it ahead of lesser buildings (0: payback value only)";
    supply_war_percent: 800, 0, 2000, "Strategic supply: where every spare Iron is already wanted by a Forge-ready city or an idle Forge, a Pikeman (or an upgrade into one) takes it only if its strength premium over a Phalangite, counted this percent at war (100 = no extra), beats the Production a Forge would put to work with it (0: a Pikeman always takes the Iron)";
    supply_term: 400, 0, 2000, "Strategic supply: a campaign's value gains this many win-chance points (1/10000) per Production per turn the target city's deposits would put to work for us (half the owner's loss), times the capture odds (0: the public economy share alone, which is too small to move a target choice)";
    supply_race_cap: 1, 0, 1, "Hard: cap a campaign's supply term at its largest positive capture, race, or denial term, so a deposit cannot outweigh the candidate's win-chance value (0: uncapped supply term)";
    supply_shortage_percent: 300, 100, 1000, "Strategic supply: a seat short of supply (an idle user, or its era's resource wanted with none to spare) counts that supply term this percent, so a shortage is a reason to take a deposit city";
    relative_city_percent: 100, 50, 200, "Hard and Search: weight city and population gaps against rivals in the win race";
    relative_economy_percent: 100, 50, 250, "Hard and Search: weight public Production plus Gold economy gaps against rivals";
    relative_vp_percent: 100, 50, 200, "Hard and Search: weight VP and projected title gaps against rivals";
    vp_race: 100, 0, 300, "Value our VP relative to the public leader and closest rival, increasing as they approach the target (0: fitted win chance alone)";
    vp_denial: 100, 0, 300, "Project live faction-goal VP lost or transferred by a city capture and price public objective-count denial (0: old city projection)";
    vp_hold_break: 100, 0, 300, "Extra value for breaking a rival's public victory hold and for a coalition fighting it (0: old hold valuation)";
    vp_peace: 100, 0, 300, "Cost of making Peace with a holder or a team one point from holding, offset by the danger of continuing the war (0: old peace policy)";
    vp_two_step: 100, 0, 300, "Prefer a reachable second VP after the next Age card (0: one-point planning)";
    early_campaign_muster: 0, 0, 1, "Hard Sparta: stage a chosen early unwalled Campaign before declaring, until its local funded siege forecast is ready";
    #[serde(alias = "oathbound_conquest_bundle")]
    sparta_conquest_bundle: 1, 0, 1, "Hard Sparta: train an assault wing for a known nearby unwalled city";
    #[serde(alias = "oathbound_force_campaign")]
    sparta_force_campaign: 0, 0, 1, "Hard Sparta: choose a feasible Campaign despite plan/search valuation veto";
    sparta_valued_expedition: 1, 0, 1, "Hard Sparta: only a positive Campaign plan may compel an expedition or declaration (0: legacy zero-value forced expeditions)";
    #[serde(alias = "oathbound_search_only")]
    sparta_search_only: 0, 0, 1, "Apply new Sparta war genes only to Hard + Search (including its rollout copies); 0 also applies them to plain Hard (default 0 since 2026-10-11: Hard seat-only +4.1 pp with troop_runway, and Sparta's First Blood 0% to 100%)";
    #[serde(alias = "oathbound_strike_min_hp")]
    sparta_strike_min_hp: 50, 30, 60, "Hard Sparta: minimum HP for a wounded regiment to join a forecast city strike";
    #[serde(alias = "oathbound_expedition_min_hp")]
    sparta_expedition_min_hp: 60, 30, 70, "Hard Sparta: minimum HP counted toward a nearby war expedition or open-city follow-up";
    #[serde(alias = "oathbound_march_min_hp")]
    sparta_march_min_hp: 20, 10, 50, "Hard Sparta: minimum HP to keep a wounded regiment marching with its campaign";
    #[serde(alias = "oathbound_capture_heal_forecast")]
    sparta_capture_heal_forecast: 1, 0, 2, "Hard Sparta: price capture healing in siege losses (0: off, 1: erase new wounds, 2: also credit repaired old wounds)";
    #[serde(alias = "oathbound_peace_guard")]
    troop_runway: 10, 0, 80, "Hard: away from the front, no troop is started whose upkeep, leaves under this many turns of Gold runway; such troops were trained, then disbanded for upkeep (0: off, the 2-turn ledger floor only)";
    unit_concentration_turns: 10, 0, 30, "Hard: defer a new combat-unit build slower than this many turns when another eligible city can finish the same unit sooner (0: off); cities that can finish within the window may build in parallel";
    force_mix: 0, 0, 6, "A city not under threat trains combat units toward a class mix, melee/ranged/mounted (experiments 1: line 8/1/1, 2: mounted 4/2/4, 3: ranged 3/5/2, 4: screen and gunline 5/5/0, 5: combined 5/2/3; 6: the combat simulator's best response for our era to the armies we face; 0: off)";
    peace_retaliation: 0, 0, 1, "Hard: the Peace plan with an enemy at war carries the same retaliation term as fighting on, times the chance the enemy refuses (certain if it declared, refused our last offer or is our top threat; else half), so a capable AI doesn't sue for peace the turn the enemy army arrives (0: off; playtest 8.8.1)";
    rival_redeclare_percent: 100, 0, 200, "Hard: price the rival's public declaration record and army left at our border when accepting Peace; 0 keeps the old free-Peace estimate";
    peace_withdraw_price: 1, 0, 1, "Hard: replace the flat Peace bonus after a campaign withdraws with the value of healing its wounded army; 0 keeps the old +100";
    contain_after_campaign: 0, 0, 1, "Hard: a Contain war on the enemy of our running Campaign stays on offer, so when the campaign ends the plan falls back to the containment war it served, not to the free Peace plan (playtest 8.9.2: the Mongols sued the VP leader for peace the turn a siege fell short and declared again four turns later; 0: off)";
    peace_credibility: 300, 0, 1000, "Hard: value charged to a new Contain or Campaign war on a team we signed Peace with after a war, falling to 0 over the next 30 turns; never charged against a team counting down to victory (default 300 since 2026-10-17: seat-only +0.4 pp [-1.8, +2.5], opponents-only -3.3 pp, war suite unchanged; 0: off; playtest 8.9.2)";
    value_first_target: 1, 0, 2, "Hard: choose the conquest target by value among every reachable rival city, scoring public softness (an empty or levy-capped garrison, the owner's other wars, a last city), never by the owner's faction (1: Sparta, which drops its one-city India/Byzantium filter; 2: every conquest faction; 0: off; playtest 8.9.2 Sparta; default 1 since 2026-10-17: seat-only +2.0 pp [+0.2, +3.7] with storm_forecast and campaign_commit, opponents-only 0.0)";
    storm_forecast: 1, 0, 2, "Hard: siege forecasts price the engine's split counterattack and Sparta kill healing on unwalled cities too, so a melee wing commits to a city it can storm (1: Sparta, 2: every faction for the split; 0: off; playtest 8.9.2 Sparta; default 1 since 2026-10-17, war scenario Sparta's Nicaea Strike 0% to 100%)";
    split_counterattack: 1, 0, 1, "Forecast one city counterattack split among simultaneous melee attackers for every faction (0: legacy Sparta-only split)";
    walls_under_way: 1, 0, 1, "Hard: threat defence never swaps Walls under way for a shooter and counts a shooter within two hexes as present (0: off; playtest 8.9.2 Sparta; default 1 since 2026-10-17: seat-only +2.9 pp [-0.8, +6.6], opponents-only +3.5 [-0.6, +7.6]; the user's own 8.9.2 rush replayed against it stalls after two cities)";
    walls_before_shooter: 1, 0, 1, "Hard: a city facing a rusher or a threat starts Walls before the shooter when the Walls finish before the nearest hostile army can arrive (visible armies by their moves, else the rival's nearest city plus training time; 0: off, shooter first; user 2026-10-17)";
    prior_needs_army: 0, 0, 200, "Hard: a rush-capable neighbour by its faction alone (no army massing, no war) triggers border defence only when its public army is at least this percent of ours (0: always, the old prior; playtest 8.9.2: AI Sparta walled up against a one-unit Mongol neighbour instead of rushing)";
    assault_wing_forecast: 0, 0, 1, "Hard Sparta: the early assault wing keeps training melee until the siege forecast, with the regiments placed at the prize, takes it within three rounds (eight at most), instead of a fixed count (0: off; playtest 8.9.2 Sparta)";
    campaign_commit: 300, 0, 1000, "Hard: value charged to any plan that walks away from a Campaign war we declared while its target city stands open (no Walls), for peace or another war (0: off; playtest 8.9.2 Sparta; default 300 since 2026-10-17)";
    peace_redeclare: 2, 0, 2, "Hard: while the army holds a city it just captured (2: or fought the enemy in the last 8 turns), no Peace (offered or accepted) with an enemy whose charges for declaring again (aggressor risk, second front, broken peace, peace credibility) exceed the retaliation it can bring meanwhile, unless the empire is endangered (playtest 8.9.5 Athens: Sparta signed Peace after every capture and declared again, Nagoya changed hands 11 times; 0: off; default 2 since 2026-10-21: all-AI wars per game 47.2 -> 36.3, declarations within 10 turns of a Peace 13.4 -> 8.1 per game, human-seat benchmark seat-only -0.6 pp [-3.2, +2.1], opponents-only -1.2 [-4.5, +2.1])";
    peace_pair_scope: 1, 0, 1, "All difficulties: price Peace for the named enemy only, preserving other wars; value removing one member of an enemy Alliance while its ally still fights (0: legacy global-danger valuation)";
    siege_window: 0, 0, 8, "Hard: rounds past the arrival that the siege forecast and the city-assault check play out (0: the built-in four; combat lab 2026-10-17: a four-round window calls a third of winnable cities hopeless, melee storms often take five or six)";
    runaway_read: 1, 0, 1, "Hard: read an economic runaway from public signs: the threat race starts at 30% of the target and adds a live faction goal and the public Power lead; the strict VP leader is not discounted by distance; such a leader is containable from 30% (0: off; playtest 8.9.2 Switzerland; default 1 since 2026-10-18 with coalition_front and walled_quota 4: seat-only +1.5 pp [-0.4, +3.3], opponents-only -0.5 [-2.7, +1.8] at 256 salts; Golden Horde 43% -> 100%)";
    leader_commit: 0, 0, 2, "Hard: wars on the leader are commitments: a plan that ends our war with the strict VP leader at half its target pays the VP that war could still take; 2 also makes every faction declare a Contain war only with a funded siege route to a named city (contain_commit) (0: off; playtest 8.9.2 Switzerland, user 2026-10-17)";
    leader_peace_horizon: 1, 0, 1, "Hard: price an ongoing campaign against a strict leader one point from its target before its hold starts; 0 keeps the old active-hold-only protection";
    coalition_front: 1, 0, 1, "Hard: coalition members value striking the leader while another member's army is at one of its cities (a separate front), and a containment war with no city in marching range stages on the target's nearest city to open a front (0: off; user 2026-10-17)";
    coalition_push: 1, 0, 1, "Hard: price reachable coalition pressure and leader city prizes; 0 restores the prior coalition strength, race, hold and front scoring";
    walled_quota: 4, 0, 8, "Hard: while our plan's war marches on a walled city, shooters are worth as much as a counter-class unit until this many stand in our army, with a melee escort of two (0: off; the Golden Horde finding, playtest 8.9.2)";
    coalition_own_read: 0, 0, 1, "Hard: the war target we organise against is our own top threat (the threat read, with its runaway signs); the formal coalition target decides only when its threat score is within 10 of the top (0: the formal target overrides from 80% of its VP target; the lead's recommendation, user 2026-10-18)";
    celtic_campaign: 1, 0, 1, "Hard Celts: run city campaigns, sieges and assaults like everyone else, valued by the sack's Gold and possible technology, the warband camp goal, and the city its owner loses, since they burn what they take (0: the Celts run no city campaign; idea #25, the 8.9.4 Swiss review)";
    celt_sack_target: 1, 0, 1, "Hard Celts: nominate reachable cities by sack_gold_value, penalizing distance, garrisons and Walls more heavily so funded wars can actually sack (0: old target score)";
    #[serde(alias = "japan_network_guard")]
    india_network_guard: 1, 0, 1, "Hard India: linked cities keep a guard against a single arriving attacker and fund one extra defender per two linked cities (0: normal defense)";
    india_remittance_value: 400, 0, 600, "Hard India: production value per Gold per turn remitted by a new provincial Revenue Office (70: former valuation; 0: no remittance value)";
    india_office_goal_value: 800, 0, 1000, "Hard India: after turn 60, extra value for each missing Revenue Office while the held Imperial Diwan goal is pending; emergency defense still takes precedence (0: no goal preparation)";
    india_last_office_value: 0, 0, 2000, "Hard India: extra goal value divided among the last two missing Revenue Offices, so the final prerequisite can complete the Diwan chain (0: no final-office priority)";
    india_diwan_commit: 0, 0, 1, "Hard India: once every city has an Office, start the Imperial Diwan in a safe productive city, keeping emergency defense and the army minimum first (0: routine production scoring only)";
    india_emergency_defense: 1, 0, 1, "Hard India: allow a troop needed at a threatened city even when its upkeep leaves under ten turns of treasury runway (0: the former unconditional veto)";
    india_prewar_guard: 1, 0, 1, "Hard India: treat a city exposed to a rush-capable neighbour as a funding front before War, so its anticipatory defender is not vetoed by the ten-turn Gold runway (0: only active War and visible armies count)";
    india_early_office: 1, 0, 1, "Hard India: start a Revenue Office in a safe provincial city before the opening queue resumes (0: opening queue keeps precedence)";
    india_early_office_min_gold: 3, 1, 6, "Minimum estimated Gold per turn remitted by an early provincial Revenue Office";
    #[serde(alias = "japan_objective_reach")]
    india_objective_reach: 1, 0, 1, "Hard India: a reachable conquest nominee can complete a Gain cities Age card when nearby settlement sites run out (0: settlement sites only)";
    siege_timeline: 1, 0, 1, "Only count Siege equipment affordable from current Gold and allow one action turn to upgrade before moving (0: six-turn future income is spent immediately)";
    siege_for_walls: 1, 0, 1, "Hard: forecast a walled city with Siege support";
    war_siege: 100, 0, 300, "Hard: every war our plan fights (Contain or Campaign) has an active objective: when the force near one of the enemy's cities is at least this percent of its local defence (two thirds of it while another coalition member's army stands at the enemy's cities), shooters take firing positions in range of the city, accepting its strike, and melee screens them; with no city in reach, the army goes for the enemy's field units near it on the same test. A failed campaign's army stays to siege instead of walking home. Per-faction siege_exempt 1 restores a faction's old exclusion (0: off; user 2026-10-18, EU4-style wars: \"they pretty much always attack with whatever they have\")";
    siege_scope: 0, 0, 1, "Hard, with war_siege: 0 sieges in every war the plan fights; 1 only in wars on the strict VP leader, a team counting down to victory or our diplomatic target";
    siege_floor: 0, 0, 90, "Hard, with war_siege: a sieging unit may stand where it keeps at least this HP after an average strike from the city (0: anywhere it survives the strongest strike)";
    siege_parts: 15, 0, 15, "Hard, with war_siege: which parts run, as a sum: 1 the early pass on the running campaign's city, 2 the late pass for every plan war, 4 the raid on field units when no city qualifies, 8 keeping a failed campaign's army at the front";
    siege_melee: 1, 0, 1, "Hard, with war_siege: melee screen the sieging shooters (0: only shooters besiege; melee stay with the storm and campaign passes)";
    siege_early_shooters: 1, 0, 1, "Hard Sparta, with war_siege: the early siege pass takes only unused shooters (0: it may also take melee)";
    siege_storm_first: 1, 0, 1, "Hard Sparta, with war_siege: run the early siege after Sparta's sorties, rush siege and assault passes (0: before recovery and those passes)";
    siege_home_radius: 2, 0, 8, "Hard, with war_siege: units within this many hexes of a home under real threat (more than half its strength) stay to defend it instead of besieging";
    siege_shooters: 1, 1, 6, "Hard, with war_siege: the fewest shooters near a city to besiege it";
    siege_walled_only: 0, 0, 1, "Hard, with war_siege: besiege only walled cities and leave open ones to the storm (the assault coordinator)";
    campaign_persist: 100, 0, 300, "Hard: a running campaign whose committed force at the city is still at least this percent of the city's local defence doesn't end when the capture forecast misses (a defender buying a garrison unit each turn breaks it every turn, and pays for each one); it is priced as a grind: shooters' fire takes the city's HP down, then one turn to storm it, at a strike's loss a turn; not for Sparta (0: off; playtest 8.9.2 Golden Horde, coordinator 2026-10-18; default 100 since 2026-10-18 with war_siege 100)";
    war_garrison: 0, 0, 1, "Hard, with war_siege: while the plan fights a war, a city no rival can reach soon (no threat on it, no enemy city within 8 hexes, no enemy unit within 5) keeps no standing garrison, so its unit can go to the front (0: off; user 2026-10-18)";
    war_pressure: 0, 0, 50, "Hard: a war we declared whose units have not attacked the enemy for more than eight turns loses this much plan value per idle turn (prosecute it or make peace), and an idle war's peace offer scores one point more per idle turn, up to 30; never for a war on the strict VP leader, a team counting down or our diplomatic target, where peace helps the leader (0: off; user 2026-10-18: no token wars)";
    dual_attack_choice: 1, 0, 1, "A unit with both a charge and a bow (the Mongol Keshig) takes whichever is worth more each turn: its best shot against its best charge, by kill value, retaliation and where the charge leaves it (0: it shoots whenever it has a target)";
    fighter_strike: 1, 0, 1, "A Fighter strikes the best unit or city within 5 tiles instead of always staying on guard, unless a rival at war in the Flight era could send Bombers near (0: always on guard; user 2026-10-17: Fighters may strike, with shorter range; default 1 after the human-seat benchmark showed no drop, docs/sim-results/hard-vs-human/journal.md)";
    air_wrapper_orders: 1, 0, 1, "Search and Opening wrappers dispatch the scripted air policy after their ground plan (0: legacy omission of aircraft actions)";
    air_marginal_strikes: 1, 0, 1, "Allocate each aircraft against target HP left after planned ground and air fire (0: each independently values the original full target)";
    air_defense: 100, 0, 300, "Price Fighter production and guard duty by uncovered cities and troops within reach of public Flight-era rival bases (percent; 0: bomber-count production and fixed guard value)";
    air_stage_read: 1, 0, 1, "Use visible rival hangars and reported air battles as aircraft evidence in decisive Bomber buys; treat a Modern rival without Fighter evidence as uncertain rather than certainly defended. At War, price a Fighter guarding an exposed own capital or Great Work city when the rival is known to have Flight (sighted or reported aircraft, or a later public era). 0: prior era proxy and ordinary air defense; default 1 after the 2026-11-05 India gate";
    air_rebase: 1, 0, 1, "Rebase idle aircraft to useful own hangars and evacuate bases under immediate capture threat (0: never rebase)";
    air_commitment: 0, 0, 1, "Preserve a useful aircraft already under construction during a ground emergency unless the city is forecast to fall now (0: aircraft builds are interruptible civilian work)";
    phalanx_formation: 1, 0, 1, "Hard Sparta: Spartan Hoplites stage and march next to each other, since a Hoplite beside another fights with the phalanx bonus (0: ignore adjacency)";
    scout_find: 200, 0, 2000, "Hard: value-scored scouting. In the opening (before turn 40) while a rival capital is unseen, the Scout's frontier choice adds, to the usual 3 points a tile it would reveal, this percent of a point per hex of reach (the map's expected spacing between capitals, about 24 on the 48x32 map) the tile lies inside our nearest city: the land around home is explored before the far edge, so sites and neighbours are known sooner (0: off; default 200 since 2026-10-18: league of 9,000 fresh 8-player games 58% against 51% for the control, 8 seats 57.7% on held-out seeds, 4 seats 54.5%, cities at turn 100 3.54 against 3.26; the fixed-map benchmark with each salt's own dice, seat-only +1.5 pp [-1.7, +4.5]; saturates from 200 to 1600)";
    military_scout: 0, 0, 300, "Hard: value of a safe frontier tile's unexplored land and ruin to an idle military unit, as a percent of a Scout's valuation; distance, lost fortification, exposure, health and distance from home reduce it (0: off)";
    learned_macro: 0, 0, 3,"Hard+Search distilled integer model: 0 off, 1 plans and openings, 2 plans only, 3 openings only";
    learned_value: 0, 0, 2, "Hard: the plan's chance-to-win estimate scores each team with 1: the learned value network (crates/ai/data/value_net.json, a small integer MLP trained on Hard self-play outcomes), 2: the same linear formula with its weights refitted on those games (the recalibration alone), instead of the fitted linear WEIGHTS; projected title transfers, the VP hold overlay and the VP race term stay rule-based (0: off, the hand formula; AI war plan step 3)";
    focus_fire: 100, 0, 300, "Hard: battle tactics that concentrate on kills (AI war plan step 2). Before the ordinary passes pick each unit's attack alone, the free shooters and healthy chargers that can reach a visible enemy field unit this turn are priced as one plan: the exact kill chance of their summed rolls, a kill worth the target's cost plus the threat it carries (this percent), against the chargers' split counterattack, death chance and the fire they stand in after the kill, and each attacker's value as a lone hit; the best plan goes first until none pays (0: off)";
    gather_hold: 50, 0, 200, "Hard: stragglers wait for the army (AI war plan step 2). The last military pass: a move that would end where the visible enemy can hit the unit next turn, with its expected share of that fire (split among our units ending beside it) at least this percent of its HP, stops at the last tile of its route below that, or stays; attacks, retreats, coordinated assaults and moves into or beside our own cities are exempt (0: off)";
    door_defense: 100, 0, 300, "Hard: defend where the enemy army is (idea #23, AI war plan W2). A visible enemy field army within three turns' march of a city is weighed (squared strength, this percent) against the defenders near it; the gap is closed with the era's best unit bought there, Gold upgrades of the units already there, the nearest free unit into an empty threatened city and its production, and favourable focused strikes on the attackers' few or wounded units, and a weakened lost city nearby is retaken while lightly held. Everything follows the visible threat and stops when it leaves (0: off)";
    peace_army_sites: 1, 0, 1, "Hard: discount a settlement site near visible military of a non-allied empire at Peace by its local force against defenders Hard can bring, and its public incentive to turn on us (relative army strength, another war, and known border contact); 0 restores prior site orders";
    army_door_production: 1, 0, 1, "Hard: while a visible at-war army leaves a city's door front short, build the newest trainable line defender there and in reachable own cities within four hexes until the gap closes; replace weaker units, buildings and wonders (0: old production choice)";
    door_parts: 29, 0, 63, "With door_defense: which parts act, a bit mask (1 buys, 2 upgrades, 4 production, 8 focused strikes, 16 garrisons for empty threatened cities, 32 retaking a weakened enemy city beside ours; 63: all). Default 29 leaves out upgrades and the retake: on the war suite (2026-10-28) upgrades cost 0.6 points of the mean (Japan T142, Athens T112, German air early) and the retake fired but helped no case and cost Japan T142 84% to 73%";
    war_reaim: 1, 0, 2, "Hard: re-aim, don't quit (AI war plan W2, 8.9.15 Swiss review rec 1). When the plan's campaign against an enemy ends (insufficient army, Walls before arrival, negative value, target gone) or its target drops out of reach, the same enemy's other cities are ranked by value over defence and march time and the best few are priced as campaigns, with the field army's march counted even while another war is on (the war on this enemy is already open); the plan stands down to Peace only when none is winnable. 2: also price an empty target as holding one defender bought there, since the owner can buy one unit per city per turn and its treasury is hidden (0: off; playtest 8.9.15: one bought Arquebusier in Chur ended the Mongol war on the empty, rich leader); default 1 since 2026-10-29 with hollow_read, persist_hollow and peace_own_danger 2: AWS gate rivals-only +0.34 pp [-1.38, +2.07], all seats +0.88 [-0.97, +2.73], Chur T137 3% -> 34%";
    hollow_read: 1, 0, 1, "Hard: read the hollow leader (8.9.15 Swiss review rec 2). An empire whose public army per city is at most half the median empire's, and that shows riches in public (a live goal, a Power lead, an era lead, a visible Swiss capture bounty), adds its hollowness weighted by its cities to its runaway signs without a strict VP lead (nobody more than a tenth of the target ahead of it); such a team is containable from a fifth of its target and its visibly empty cities rank higher as targets (0: off; playtest 8.9.15: one unit for eight cities, 3,600 Gold read 30 to 44 at a VP tie); default 1 since 2026-10-29: AWS gate rivals-only +0.34 pp [-1.39, +2.07], seat-only +1.07 [-0.18, +2.33], Chur T137 3% -> 16%";
    persist_hollow: 1, 0, 1, "Hard: war_reaim and peace_own_danger act only against an enemy that reads as a hollow leader (public army per city at most half the median empire's, and rich in public: threat::hollow_bonus); against anyone else a war ends as before (0: they act against every enemy; W2.2 found blanket persistence cost the AIs +2.1 pp in full games); default 1 since 2026-10-29 with war_reaim 1 and peace_own_danger 2";
    hollow_march: 0, 0, 15, "Hard: execute a war on a hollow leader (AI war plan W2.4, playtest 8.9.17), a bit mask. While the plan's war target reads hollow (threat::hollow_bonus): 1, an own city with no visible foreign military unit within 6 reachable hexes keeps no standing garrison and its units count toward the campaign forecast's march force (the capital keeps its garrison); 2, the capital too; 4, the war army quota is at least 5 units beyond one per city; 8, while the forecast reads every enemy city out of reach, Gold above 150 buys one march unit a turn up to that quota, Swiss hoard savings notwithstanding. Door defense still answers any army that comes into view (0: off; the Swiss kept five of six units in their cities through 40 turns of war on the designer's ten-city, four-Horseman Mongols, wanted one unit beyond a garrison each and banked 1,400 Gold for their Reserve; with a two-round siege forecast every Mongol city read army too weak)";
    hoard_at_war: 1, 0, 1, "Hard Switzerland: while its plan fights a war and the unheld hoard would not win, Gold saved toward the goal is spendable on the war (0: keep the reserve; default 1).";
    decisive_buy: 1, 0, 2, "Hard (designer-decision patterns 2026-11, fix 2: at the 8.9.24 Swiss T140 he bought a Bomber the turn after Flight for 805 of 4,358 Gold, +2.38 VP in the paired diff; Hard kept the Gold, its plan read Peace and the 5,000 hoard savings were the reserve): a purchase whose forecast effect on a live war is worth more VP than the reserve holds spends the reserve, whatever the plan is called. For every enemy city at war whose capture brings VP (an original capital, a live goal, a Fight objective, a title), the siege forecast of the field force that can reach it (aircraft striking in the volley) runs without and with the bought unit; the gain is the added capture odds times that VP, the price the VP the Gold puts at risk (a Swiss hoard still to bank arrives later by the income turns the purchase costs, out of the turns left; a held hoard is never spent). The purchase leaves the deficit reserve and its upkeep payable, at most one a turn; a Bomber goes to the qualifying city whose range holds the most of the war. 1: Bombers (from the next turn, out of reach of a rival in the Flight era); 2: land units too, marching from the city that buys them (0: off; routine and peacetime buys keep the reserve either way); default 1 since 2026-11-03: the 8.9.24 T140 Bomber fires under the recorded view, in his city (Shangdu), and Hard with it from T140 scores +1.71 VP [+1.14, +2.27] by T161 over 48 paired runs; 2,048-game gate (batch spend-hoard-gate-1103-113520) identical to the control at 1 in every game, rivals-only and seat-only (no benchmark game reaches a decisive Bomber), and neutral at 2 (rivals-only +0.05 pp [-0.98, +1.12], seat-only -0.49 [-1.22, +0.24])";
    decisive_buy_min: 20, 0, 200, "With decisive_buy: the least net gain, in VP percent (added capture odds in percent times the capture's VP, less the price), that releases the reserve";
    peace_own_danger: 2, 0, 2,"Hard: Peace follows the plan (8.9.15 Swiss review rec 3). Toward the enemy our plan fights, only danger from that enemy, or a city of ours at half HP or less with an enemy beside it, makes us want Peace; pressure from a third war no longer does. 1: that enemy's danger is the usual local test (its army near a city against our units within 2); 2: its army must outweigh all of ours within 5 hexes, or the city be at half HP (0: off, any enemy's pressure near our cities ends every war; the Mongols, pressed by Japan, offered the Swiss Peace mid-campaign on T161 and mid-hold on T175); default 2 since 2026-10-29, limited by persist_hollow";
    peace_siege_value: 0, 0, 200, "Hard: price the visible, favorable siege or nearly fallen original capital lost during Peace's ten-turn truce, even when the routed plan fights elsewhere. The price scales with the campaign's capture value; danger from the offerer's army at our cities still permits relief (0: off; 100: measured candidate, left off after the rivals-only gate was inconclusive)";
    peace_prize: 0, 0, 200, "Hard: price the prize in reach against Peace (fable-scenarios-1115, Game 4 T126: Peace taken one turn after Chittor fell, the capital two moves away, -1.8 VP at +20 turns). At war, the best enemy city (remembered sightings included, a fogged city assumed to hold one Soldier) that the forces already in place take within the ten-turn truce, by the campaign planner's own forecast and price, is a cost of Peace: capture value x gene / 250, at most 200, against the Peace terms (0: off; 100: the candidate the study measured; the cost is waived while the enemy army stands within three hexes of our original capital)";
    dead_war_peace: 12, 0, 60, "Hard: end a war nobody fights (8.9.17 Mongols review rec 2). A war at least this many turns old counts as dead when the plan fights someone else (or nobody), no campaign or strike is aimed at that enemy, none of our units stands within two hexes of its cities, its army does not endanger us, and it is neither the leader the plan contains, a team counting down, our diplomatic target nor a hollow leader (W2.3's persistence stands). A dead war's Peace drops the redeclaration veto (no plan would declare again) and the truce's lost pressure (nothing is pressed), and scores a point per turn past this age, up to 30, plus 20 (10 if out of reach) while a rival leader is the war that matters (playtest 8.9.17: the Celts, strongest army on the map, refused 11 Peace offers from Athens over 85 turns of a war their own plan wanted ended; 0: off); default 12 since 2026-10-31: 2,048-game gate (batch w28-1030-233855, control 352) neutral, all seats +0.34 pp [-1.58, +2.27], rivals-only -0.34 [-2.28, +1.59], seat-only +0.24 [-1.35, +1.83]; on the designer's 8.9.17 Mongols game (replays from T96) the Celts declare on him in 32 of 32 with the gene on (12 of 32 off) and Hard's win rate in his seat falls 24/32 -> 19/32";
    capture_hold: 1, 0, 1, "Hard (decision audit 2026-10-31: after a capture the designer keeps his few units attacking; Hard pulled the nearest one back into each city in resistance or damaged with enemies about, before any attack pass ran): a captured or damaged front city takes a holder first only when the visible attack would take it empty this turn, a holder would stop that, and the holder has no capture this turn worth more (enemy city population x its capture chance against ours x 90%); otherwise its holder comes from the units the attack passes left free (0: every front city takes the nearest unit within 3 first); default 1 since 2026-10-31: 2,048-game gate (batch w30-1031-032745, control 407/2,048), opponents-only +2.64 pp [+0.90, +4.37], seat-only +0.05 (noise; the 512-game quarter gate's +3.91 seat-only did not replicate)";
    strike_kill: 1, 0, 1, "Hard (decision audit 2026-10-31: the designer's units kill the wounded or exposed enemy beside them on the way to a city; Hard's opportunity strike sent the same units to their staging tiles or held them there): a unit of the strike force that can kill a visible enemy field unit this turn alone (forecast kill chance at least 50%) for more than it risks (the target's cost times the kill chance against its own cost times its loss chance) is left to the attack passes instead of staging (0: the strike force always stages); default 1 since 2026-10-31: 2,048-game gate (batch w30-1031-032745, control 407/2,048) with capture_hold 1: opponents-only +3.32 pp [+1.51, +5.13], seat-only +1.17 [-0.44, +2.78]";
    retake_hold: 0, 0, 20, "Hard: for this many turns after a campaign begins against a city owned by this player within the previous 20 turns, keep the campaign through an insufficient_army forecast while the committed force near the city has at least half the city's and visible garrison's strength. Other end reasons still apply (0: off)";
    found_here: 0, 0, 2, "Hard (decision audit round 3, 2026-10-31: on 15 of the designer's 39 mid-game founding turns Hard's Settler, standing on the tile he founded on, walked to a site 2 to 10 hexes away, went home or retreated: the site search drops a tile for hazards to a Settler on a trip, an enemy army within the threat radius, danger, its trip risk, and for a foreign border that could hide a city): a Settler on a legal site of the preferred quality (foreign-border risk included) founds there when the site's value, less its forecast loss as a fresh city (the visible at-war attack that can reach it by next turn against a new city's defence, both sides of the swing), is at least the value of the site the search chose, or when it would retreat or go home instead. The Settler must be safe from capture this turn (founding resolves after movement; a first version without this lost the 8.3 Switzerland gate map 18 -> 0 of 64), and a found order that failed on that tile the turn before falls back to the search, which rescouts. 2: the tile's land quality need only be positive, not settler_min_site_score, since the Settler is still compared with the search's far fallback or going home (0: off; round 3: audit whole policy -0.013 cities @20, settling share -0.007; census cities at T100 +0.30 ±0.21; quarter gate seat-only -0.98 pp, rivals-only -1.17, both noise)";
    found_now: 10, 0, 100, "Hard (decision diff patterns across 23 of the designer's games, 2026-11-03, pattern 2: on 62 turns in 20 games he founded on the Settler's tile while Hard routed it on, retreated or went home, +5.79 VP [+1.28, +10.30]; at 8.3 Tollkeepers T36 and 8.9.5 Signalwrights T58 the only threat was a barbarian camp's Scout or guard, which the rules never let capture, and at T44 a Ranger of a rival at Peace): the Settler's own tile is weighed as `found_here` weighs it, but the chance it is captured before the found order resolves is priced per unit that can reach it this turn instead of ruling the tile out: an empire at war or a raiding barbarian 100 (still a veto), a barbarian Scout 5 (it only wanders and reports), a barbarian standing on its camp 0 (the guard holds the camp), a rival at Peace that can declare its `peace_settler_capture_chance`; barbarians never assault a city, so only empires at war count toward the fresh city's loss. The tile must also pass the site search's border verdicts (remembered and visible foreign borders that could hide a city; the 8.9.24 Swiss T145 rejection). The search's own choice is discounted by the chance its land is gone before the Settler founds there: a visible foreign Settler that can reach within its city gap of the site by then, 50, and each other empire whose nearest known city lies within a Settler's reach of the site (twice its city gap plus 2), this gene's percent a turn times how close the site lies, over the travel turns plus one (0: off)";
    border_explained: 1, 0, 1, "Hard (decision audit round 4, 2026-10-31: on 30 Settler-turns of the designer's four wins Hard's site search left out the tile he later founded on with that Settler, as a site a foreign border could hide a city near; in 29 of them every foreign tile behind that verdict lay within 2 of a city the AI already knew, so no unseen city was implied, and Hard sent the Settler to a site 2 to 10 hexes farther, a weak far fallback or home): a visible foreign border tile within a known city's work radius of a city of that owner is explained by it and implies no unseen city, as for remembered borders (`remembered_border_risks`); only unexplained borders put a site at risk (0: every visible foreign border tile is evidence, `settlement::border_city_risks`); default 1 since 2026-10-31: 2,048-game gate (batch w31-1031-070814, control 364/2,048), both genes: seat-only +4.35 pp [+2.47, +6.22], opponents-only +1.42 [-0.55, +3.38]";
    fallback_risk: 1, 0, 1, "Hard (decision audit round 4, 2026-10-31: with the border gene on, the tile the designer later founded on was dropped for trip risk on 26 Settler-turns, and on 15 of them Hard sent the Settler to a far fallback site, which the search never checks for trip risk: a near site rejected for one enemy unit near the route lost to a site 6 to 11 hexes away past the same units): when every preferred site is out and the search would take a fallback, the fallback's own trip risk is compared with the least risky preferred site dropped for trip risk, and a fallback no safer loses to that site (0: fallback sites are never charged trip risk); default 1 since 2026-10-31: 2,048-game gate (batch w31-1031-070814, control 364/2,048), both genes: seat-only +4.35 pp [+2.47, +6.22], opponents-only +1.42 [-0.55, +3.38]";
    shot_sweep: 0, 0, 1, "Hard (decision audit round 4, 2026-10-31: of the kills the designer made that Hard skipped in his seat, the largest shared cause was a shooter given a move by a pass before the fire passes, the routine garrison, leaving a city's strike range, the wounded's retreat or staging, 12 of 37): after every pass, a shooter ordered to move that can kill a visible enemy field unit from where it stands, given the shots already ordered (kill chance at least 50%, the target's cost times the kill chance above its own cost times the chance the rest of the visible enemy kills it if it stays), fires instead, unless it is the holder that keeps an own city from falling this turn (0: off)";
    capture_alarm: 0, 0, 1, "Hard (decision audit round 6): a damaged neighboring capture's resistance alone no longer triggers emergency unit production within eight hexes; the existing visible enemy within six hexes condition still does. Local threat, ranged relief and lone-city defense are unchanged (0: off)";
    peace_strong: 0, 0, 50, "Hard (decision audit round 5, 2026-10-31: the designer refused 11 Peace offers in his four wins that Hard in his seat would have signed; in 8 his army was at least half again the proposer's, and in 7 of those the score's +30 for endangered, an enemy unit near a thinly held or freshly captured city, outweighed the army-advantage and pressing terms; he kept taking the proposer's cities): when our army is at least (100 + this) percent of the proposer's (the army-advantage term counts at most 50), danger to our cities adds nothing to the Peace score; the plan's gates, the other terms and the VP cost are unchanged (0: off)";
    settler_shelter: 0, 0, 1, "Hard (decision audit round 5, 2026-10-31: on 12 of the 28 Settler-turns where Hard's Settler retreated as exposed in the designer's seat it stood in one of his own cities, which he left it in until the way was clear; a civilian's exposure counted every enemy that could reach the tile, as if the city were open ground, so Hard walked it out of a held city toward another one): a Settler in an own city that the worst-case attack this turn cannot break (the city shield military garrisons already use) is not exposed there, so it neither retreats from that city nor avoids it as a retreat tile (0: off)";
    colony_compound: 100, 0, 300, "Hard (8.9.23 Germany coach review, 2026-11-01: the designer had 3 cities at T23 and 4 at T32, Hard on his map T43 and T56; from his start, 641 of the capital's 918 Settler-ready choices to T60 went to a garrison or army quota worth more than the Settler): a Settler also carries a new city's compounding return, the best reachable site's early worked yield over a 40-turn window less the citizen it takes, raised by 1/cities for the Settlers the new city will train, per Production spent, scaled by this percent; with it on, one defender (not two) lets an early Settler start and a second troop must outbid it. Not while a conquest target is set (0: off)";
    settler_escort: 2, 0, 2, "Hard (8.9.23 Germany coach review, 2026-11-01: from his start, 96 of the 106 Settlers rivals took were taken by a Scout, 88 by a surprise war from Peace, 105 with no escort beside them; the escort search skipped every unit standing in a city, the only troop most young empires have): a garrison may escort a threatened Settler when its city has no threat and no enemy at war within the threat radius (0: field units only). 2 (8.9.24 Swiss coach review, 2026-11-02: the Mongol escort turned back at T38 when the Swiss Soldier stood in fog, the Settler walked on alone two hexes past Thun and was taken at T41): a route within two hexes of a city of a rival that can declare a surprise war also calls for an escort, and a Settler with no escort this turn but one within two turns' reach waits where nobody can take it this turn while the escort comes";
    settler_stalk: 3, 0, 4, "Hard (8.9.23 Germany coach review, 2026-11-01: the designer declared on Byzantium for an unescorted Settler four hexes from his capital, took it three turns later and Byzantium never founded a second city; Hard's snipe only looks one move past this turn): a Scout or field troop closes on an exposed rival Settler at Peace up to this many turns away, without declaring, while no own city is threatened, so the Snipe plan can price the war once the capture is in reach (0: off)";
    finish_kill: 50, 0, 100, "Hard (8.9.24 Swiss coach review, 2026-11-02: the Mongols had unwalled Lugano at 61 of 200 HP on T61, 30 on T62, when two bought archers killed their two melee units; the campaign ended for an insufficient army, the city healed to 50 and built Walls before the next campaign): a campaign whose city is at most this percent of its HP, with a committed shooter still on it, does not end for want of a capturer while a melee-capable unit (a quiet city's garrison included) can arrive within march_turns; those units join the force and the siege goes on while the campaign's value with them is positive (0: off). Default 50 since 2026-11-02: 2,048-game gate on top of the Settler genes, rivals-only -0.59 pp [-1.51, +0.29], seat-only +0.44 [-0.05, +0.93]; the T58 replay of his game is unchanged (Lugano 0 of 32)";
    stalk_contact: 1, 0, 1, "Hard (8.9.24 Swiss coach review, 2026-11-02: Hard in the designer's seat from T40 declared on the Mongols at T41 in 48 of 48 runs and took nothing: the Snipe sent a Scout three hexes off whose view moves were last turn's leftover, and a moving Settler outruns an equally fast pursuer; the designer shadowed it for three turns and declared from the next hex): with settler_stalk on, the Snipe plan declares only on contact (a melee-capable unit beside the Settler, which pins it), the stalk closes on every Settler not yet beside a unit of ours, on the side away from its owner's nearest city, with this turn's moves, and the garrison of a quiet city may stalk (0: the 8.9.23 snipe reach and stalk)";
    settler_slow_turns: 0, 0, 60, "Hard (8.9.23 Germany, round 2, 2026-11-02: from T41 to T80 the pipeline cap blocked 2,150 Settler-ready choices on his map while Hard trained 2 Settlers a game to his 5; slots were held by Settlers small cities would take 20 to 40 turns to finish at 220 Production and up): another city's Settler in production this many turns or more from finishing does not count against settler_pipeline in the Settler gate (0: every Settler in production counts). Off by default: at 12 the 2,048-game gate made rivals weaker (designer's seat +1.56 pp rivals-only) although they founded more cities";
    land_contest: 0, 0, 200, "Hard (8.9.23 Germany, land race, 2026-11-02: 4 of the designer's 9 cities at T81 stand on the land toward Byzantium, which never left one city; Hard founds them on whatever is nearest and leaves the land between it and a neighbour for the neighbour): a site also carries what taking it denies the nearest known rival city's empire, this percent of its value times how close it lies to that city inside a rival Settler's reach (twice the rival's city gap plus 2), so contested land is claimed first and safe land behind it later (0: off)";
    site_denial: 100, 0, 200, "Hard (placement strategy, 2026-11-07: Biel and Bern cut India off from land and Niter it needed later): inside the nearest rival's Settler reach, a site gains this percent of its value at full denial, scaled by how much open land lies behind it from the rival's side and by the strategic deposits it would hold, and by how close it lies to the rival. Default 100 since 2026-11-07 (docs/sim-results/ai-quality/settle-strategy-1107/: 4,096 eight-Hard games per seed, best/worst 2.50 to 2.48 on 9311003 and 2.57 to 2.16 held out on 8241001, cities lost in their first 30 turns -0.05 and -0.24 a game; human-seat gate seat +1.03 pp [-0.77, +2.82], rivals -0.39 [-2.47, +1.69]) (0: off)";
    camp_land: 0, 0, 300, "Hard (8.9.23 Germany, land race, 2026-11-02: a third of the good sites hidden within 8 hexes of Hard's cities lie within 3 of a barbarian camp): a known camp is worth clearing for the land behind it: the known good legal sites within its reach and 3 points per unexplored land tile within 3, this percent; camps are cleared in that order and up to 4 hexes farther out when the land is worth at least a site (0: nearest camp first, within 8)";
    contest_hold: 100, 0, 300, "Hard (8.9.25 Celts coach review, rec 1, 2026-11-03: he won on a ten-turn hold that rested on Raise the Warbands; four Hard AIs at War with him cleared one of his ten camps in 32 turns, a Mongol unit stood beside an unguarded goal camp for 8 turns, and the Aztec plan scored contest the hold at +1,000 with no unit sent): an early military pass removes a public VP source of a rival team whose loss takes it below its target (a known camp in the land of a Celts at War with us that scores Raise the Warbands, while its team holds or is projected at its target with no more than the goal's VP to spare). Each camp is priced by the win estimate: our chance with the hold restarting the turn the force clears it (the Celts call again, so the countdown resets) less our chance now, zero once the hold would complete first, this percent, times the chance the camp is its city's only known one. The force is the free units that arrive soonest within 8 turns, sized to kill a seen guard in time (it never heals) and step onto the camp; each unit's own job is charged against it (an own city it alone keeps from falling this turn, a capture it can make this turn), and the best net removal per holding team goes ahead (0: off). Default 100 since 2026-11-03: from his T130, T140 and T148 positions his own orders (re-calling the Wild whenever it lapses) win 0 of 32 each, against 32 of 32 with the gene off, his goal lapsing 10 to 15 times a run against 1 to 3; 0 of 32 from T148 with every camp guarded and his cities no longer shooting the guards (32 of 32 off), and 0 of 32 under four neutral perturbations that leave him at 91 to 100% off; war scenario war-celts-wild-hold-t150 (Byzantium, his orders replayed) 20% to 82% of 64 runs with the gene in the seat, 100% in every AI seat; 2,048-game human-seat gate identical game for game in rivals-only and seat-only (no benchmark game has a Celtic team at its target on the Wild); AI-only 8-empire league, 2,048 games, AI Celts 8.4% both ways (88 games differ)";
    celts_playbook: 0, 0, 7, "Hard Celts play the designer's 8.9.25 kit (factions skill, user 2026-11-05: the AI must play the kit; AI Celts 5.9% of 4,096 held-out games while he won easily), a bit mask, Celts only. 1, light garrisons: barbarians never attack the Celts, so a city with no threat, no city of an empire at War within 8 reachable hexes and no foreign military unit seen within 6 reachable hexes (barbarians aside; not the capital, a Wonder or Hall city) keeps no standing garrison and leaves the garrison quotas (the minimum, the ranged quota and the peace-time army quota count only the other cities), so its production goes to Settlers, buildings and the field army; door defense and emergency buys still answer an army that comes into view. 2, the Wild as a valued, repeatable call: a re-call adds a camp only to the cities without one, so the call fires whenever every city would hold a camp afterwards (the old test asked every city for room and never re-called once any camp stood), settling stays open after a call except while the goal decides the hold (the new city is covered by the next call), the Celts keep the camps in their land and no attack pass targets their guards (every pass took the 10-HP guards as free kills, so the Celts cleared their own camps the turn after a call and held the goal about one turn a game), and the window opens when the 2 VP leave at most one VP to the target with time to hold it, or at `celts_wild_late` percent of the game (calling while the points drew no lead was measured and rejected: a median T12 call cost 4.3 points of win share in 256 paired games). 4, war targets: the sack target is scored by what the designer looked at, the city's Walls, its visible garrison and the owner's visible army against ours (0: off). Measured 2026-11-05 (docs/sim-results/ai-quality/celts-playbook-1105/; 4,096 paired eight-empire Hard AI-only games per arm and seed, AI Celts win share against 8.6% off on seed 9311003 and 8.7% on 9500001): part 2 alone +2.8 pp [+1.9, +3.6] and +2.9 [+2.0, +3.8]; 3 +3.2 and +2.3; 7 +3.4 and +2.5; part 1 alone +0.2 [-0.8, +1.1]; parts 1 and 4 add nothing measurable to part 2. Human-seat gate (seed 310001, 256 salts x 8 maps): rivals-only identical game for game (no benchmark rival is the Celts), seat-only 470 to 475 wins (3) and 471 (7), all on the Celts map. Hard Celts play 2 (the Wild fixes) by default since 2026-11-05 (`BUILTIN_FACTION_DECISIONS`; parts 1 and 4 measured neutral and stay off)";
    celts_wild_late: 70, 30, 95, "Hard Celts with `celts_playbook` part 2: the share of the game (percent of the turn limit, else 300 turns) after which the Wild is called whatever the standing (the shared rule waits for 70%, T210, while 8-empire AI games end near T190; the designer called at T127 in 8.9.25). 45 calls in 95% of games at T135 but measured worse than 70: +2.1 pp against +3.2 with part 1 and 2, 4,096 paired games";
    march_past_reach: 2, 0, 2, "Hard (8.9.24 Swiss coach review, swiss-strongbox, 2026-11-03: the Mongols declared in 29 of 200 runs; on 10 to 15 of 25 turns a run the march force, a melee unit with it, forecast taking unwalled Lugano, but the Rangers within the two-turn approach had no melee unit among them, and a force in reach without a capturer ended the forecast, so no campaign was ever offered): a campaign forecast with no committed force whose in-reach units cannot take the city alone is judged by the march window's field army instead (1: only for a new war; 2: also a city we are already at war with); a Campaign plan then stages its field force toward the city (as planned_stage: at Peace, and at war every free unit marches, even alone) and, in the first third of the game against an open city, declares only once the force in reach takes it (as early_campaign_muster). The forecast alone declared in 200 of 200 runs at 7% capture odds and took Lugano in 1%; with the staging, 85% (0: off). Default 2 since 2026-11-03: 2,048-game gate rivals-only -2.54 pp [-4.35, -0.68], seat-only +1.46 [-0.05, +3.08] (1: -1.95 [-3.66, -0.24] and +0.49); in the designer's game from T58, his orders and purchases replayed, Lugano falls in 31 of 32 runs (0 of 32 without; the marching reinforcements do it, staging only at Peace gives 0); costs the late Mongol war scenarios Solothurn (14% to 0%) and Chur (32% to 23%)";
    upgrade_wait: 0, 0, 1, "Hard (decision audit round 3, 2026-10-31: on 24 unit-turns Hard upgraded a unit the designer attacked with or marched, losing its action for the turn; upgrades take all moves): the modernization pass leaves a unit to the military passes this turn when it has an attack they would take (the best profitable shot for a ranged unit, a favourable strike or counterattack on a visible enemy for a melee one), and upgrades it on a later turn (0: off; round 3: audit unchanged, quarter gate seat-only +3.71 pp [+0.98, +6.45] all on two maps, rivals-only -0.78)";
    joint_assault: 0, 0, 2, "Hard (8.9.25 Celts coach review, 2026-11-03: over 50 turns at War the Aztecs, with four times his army, walked about 15 units one or two at a time into the strike range of his walled Lugdunum, where the city and one Marksman killed each in 1 to 3 turns; 35 of the 45 Aztec moves into that range came from the siege pass, which sends every unit within siege_reach to a firing or screening tile, so the far ones arrive alone): the siege pass sends units into a walled city's strike range only together, when the siege forecast (opportunity.rs::forecast_siege) of the units that reach their tiles in range this turn takes the city; otherwise every planned unit stages just outside the range (three or four hexes, a safe tile) until the force that takes it can step in on one turn; 2: also when every planned unit steps in this turn, whatever the forecast (waiting would bring no one; the siege's old wear-down policy for a force that arrives together) (0: off)";
    joint_window: 0, 0, 10, "With joint_assault: the rounds past the arrival that its siege forecast plays (0: the siege forecast's own window, siege_window or four)";
    finish_fallen: 50, 0, 100, "Hard (8.9.25 Celts coach review, 2026-11-03: a Mongol Cataphract and Phalangite brought walled Celts Colony 7 to 0 HP on T92; on T93 the campaign was withdrawn and two full-HP Rangers two hexes from the city walked away while the Marksman inside, at 61 HP, survived): an enemy city at war at or below this percent of its HP, with a unit of ours within four hexes, gets the city-assault coordinator (its multi-turn capture forecast) before any withdrawal pass takes the units, whatever campaign is running (0: off). Default 50 since 2026-11-03: from his T93 position with Hard in his seat the Mongols take Colony 7 in 32 of 32 runs (13 of 32 without); 2,048-game gate rivals-only +0.68 pp [-0.68, +2.05], seat-only +0.78 [-0.39, +1.95]; war suite unchanged (docs/sim-results/ai-quality/joint-assault-1103/)";
    snipe_read: 90, 0, 100, "Hard (8.9.25 India coach review, 2026-11-03: India declared on Germany at T16 and took its Settler at T17; the next turn the Celts, who had not yet met India and could see none of its units, sent their Settler out alone and lost it to his Scout at T18, while Hard priced every surprise war at the flat 10% prior): a rival whose team alone declared a war still running within the last 20 turns is a known Settler sniper. Its surprise-war chance against a Settler becomes this percent (in every use of `peace_settler_capture_chance`: founding in place, trip risk), and its Scouts mark settler danger like its troops. On plain Hard's escort path (`settler_safety` 0): a Settler trip with an unseen tile within two hexes of its route counts as threatened while such a rival is not known to be far (none of its known cities and units within 12 hexes of the route; one never seen counts), so it takes an escort or waits up to two turns for one; an escort that cannot reach where the Settler would stop makes the Settler wait for it; two or more rival troops (not Scouts) in reach of where an escorted Settler would stop send it back to a city it reaches this turn (one escort holds a tile against one attacker); and a threatened Settler falling back into one of our cities keeps its escort beside it even from a tile a rival could strike (8.9.25 T15: the German escort stood still while the Settler turned home). On by default since 2026-11-03 although the benchmark (2,048 games, seed 310001) read rivals with it +0.63 pp softer [-1.51, +2.69] and their T100 cities 5.35 to 5.28, both within noise: that benchmark's Hard seat rarely snipes, while the designer sniped in most of his games (Germany 8.9.23 T16, Swiss 8.9.24 T41, India 8.9.25 T16-T18), and on his 8.9.25 orders from T12 this gene alone keeps both the German and the Celtic Settler (his T37 1 city and 14 Production instead of 4 and 46; `capital_guard` alone changes nothing there) (0: the flat prior, no record read)";
    capitals: 1, 0, 1, "Hard (the capitals victory, rules 37, 2026-11-15; the designer said he would beeline for the capital): the AI plays the rule. Our standing original capital with a known rival military unit within six hexes (seen at war or under a legal declaration, or remembered) keeps a troop (a shooter first) out of every other pass, calls the nearest free troop if empty, is never skipped as sheltered or spared, holds the escort-path guard whatever `capital_guard` says, and values its garrison and Walls at 4,000 above every other build; rival original capitals are worth more in target values as fewer rivals still hold one (4, 2, 2, 1 points, the last standing one 10 while ours stands), our team's fallen capital 10 to retake, and razing a captured capital keeps its founder from retaking it. 0: Hard ignores the rule in those places (the win chance still reads each team's public capital flag). On by default: the rule is always live; the switch exists for ablation and for tests of recorded older behaviour";
    capital_guard: 0, 0, 1, "Hard (8.9.25 India coach review, 2026-11-03: Germany's only Soldier escorted its first Settler out of Leipzig while Indian Soldiers closed on the empty capital; India killed the escort at T17, took the Settler and walked into Leipzig at T22): while a rival army stands within three hexes of the capital and can walk to it (two or more troops, not Scouts, of one empire at war or at Peace with a legal declaration, or one troop of an empire at war or with a recent surprise war; barbarians never assault a city), the capital keeps its last troop. A lone defender in it does not escort, and a threatened Settler left without an escort stays in, or goes back to, a city it reaches this turn; an empty capital calls its Settlers within four hexes home (their escorts walk back with them), and the troop beside a Settler that gets home this turn goes home with it. One troop of a quiet neighbour is not enough: in the Lab position `india-take-the-settler` a Spartan Soldier parked three hexes from Leipzig for twenty turns, and holding the garrison for it cut Germany's cities at T36 from 2.90 to 1.87 over 200 runs. Plain Hard's escort path only (`settler_safety` 0). Off by default: the 2026-11-03 benchmark (2,048 games, seed 310001) read rivals with it +0.78 pp softer [-0.73, +2.29] (0: garrisons escort whenever the city has no threat at war). Independent of this gene, under the capitals victory (rules 37, gene `capitals`) a known rival unit within six hexes of our standing original capital, seen or remembered, holds the guard (`capitals.rs`)";
    swiss_playbook: 0, 0, 3, "Hard Switzerland (the designer's Swiss playbook, 8.9.24 win; docs/sim-results/ai-quality/swiss-playbook-1105/README.md). 1: the Redoubt is a city's garrison: a walled city needs no troop of its own, so the garrison floor and the peacetime army quota count only unwalled cities beyond the two free units, a founded city's Redoubt is priced as the troop it replaces, and Masonry as the technology every new city's garrison waits on; 2: also lets the faction's expansion genes play the shared defaults where the climb set them lower (Settler chain priority, site floor, Settler weight); 3: raises the quiet-opening Settler chain priority to at least 1500 and accepts sites scoring at least 40. This prices earlier walled colonies without raising the general Settler production weight (0: off).";
    swiss_hoard_any_war: 0, 0, 1, "Hard Switzerland: release unfinished Reserve Gold for ordinary purchases during an actual war even when the routed plan is Peace; a held or immediately winning hoard remains protected. Swiss builtin 1 since 2026-11-06: eight-Hard AI-only Swiss wins 388 to 404 on seed 9311003 (+0.44 pp [-0.66,+1.52]), 374 to 422 held out on 8241001 (+1.33 [+0.25,+2.35]); 256-salt human-seat gate noninferior for both seat and rivals (0: off)";
    escort_buy: 90, 0, 100, "Hard (8.9.25 India coach review, 2026-11-03: Germany spent its 140 Gold of ruin and wonder rewards on a Pasture and a Mine at T2-T4 with one troop and its first Settler due, then had to choose between escorting the Settler and holding the capital): with at most one troop, a Settler out or in production and a rival troop (not a Scout) that can declare, or is at war, within six hexes of it or of the city training it, a second melee troop is worth the Settler's quote times this percent (the chance the visible army takes a Settler nobody guards); the cheapest troop worth that is bought before any routine purchase, and when it is five turns of income or less away routine purchases leave its price. Benchmark 2026-11-03 (rivals-only, 2,048 games): 0.00 pp [-0.49, +0.54]; it fires rarely, since a 160-Gold troop is rarely affordable when the army is in sight (0: off)";
    aztecs_playbook: 0, 0, 7, "Hard Aztecs (the designer's two Playtest 8.9.22 wins, 2026-11-05; docs/sim-results/ai-quality/aztecs-playbook-1105/README.md): bit flags for his plan, each a valuation on the Aztec seat only. 1: Walls first in every frontier city (an unallied rival empire's known city within open_prize_radius), and Walls before the shooter whenever no visible army can arrive first (the stock rule also assumes one from the rival's nearest city, and waits for the opening's hand-off); he queued Walls first in 26 of his new or captured cities while Hard in his seat lost its fresh unwalled colonies (Tlacopan T77, Xochimilco T123) to neighbours at war. 2: the New Fire as the closer: when the rite starts the VP hold, only the cities it shrinks must be clear of an enemy army (he kindled it at War with four empires on T144, 8 to 10 VP, and won on that hold; the stock rite waits while any city has an enemy army within three hexes). 4: a lean war army: a conquest or containment war wants one unit per city plus 1, not plus army_war_extra (5); he fought every war with 4 to 9 units for up to 20 cities, and Hard in his seat held 9 to 12 units for 6 cities (AI-only screening, Aztec seats only: plus 2 +1.6 pp, plus 1 +2.9 pp, plus 0 +2.7 pp of Aztec win rate) (0: off; 7: all three). Hard Aztecs play 7 by default since 2026-11-05 (`BUILTIN_FACTION_DECISIONS`)";
    walls_garrison: 1, 0, 1, "Hard, every faction (strategy-gap quota ablation, 2026-11-05; docs/design/strategy-gap/fable.md arm A3): the `swiss_playbook` 1 rule without its faction test. A city's Walls are its garrison: a walled city (or one with Walls under way or buildable) needs no troop of its own, so the garrison floor counts only the other cities beyond the two free units, a founded city's Walls are priced as the troop they replace, Walls come before the anticipatory shooter, and Masonry gains the Swiss Redoubt's research bonus. The Hard peacetime quota still counts every city except under `army_banker` (0: off). Shared default 1 since 2026-11-06 (docs/sim-results/strategy-gap-1105/: 4,096 eight-empire Hard games per arm, own effect +1.7 to +5.0 pp for each of the eight factions it reaches, held out on seed 8241001; human-seat gate rivals arm -3.03 pp [-5.37, -0.78], seat arm +1.90 [-0.20, +4.10])";
    walls_peace_quota: 0, 0, 1, "Hard at peace with no threat, under walls_garrison (strategy-gap follow-up, 2026-11-06): the peacetime army quota counts only the cities a troop must garrison (troop_garrison_cities, at least the two free units) where it counted every city, the upkeep half of the Swiss Redoubt rule that army_banker gives the Swiss; a Celtic seat keeps the smaller of that and its sheltered count (0: off)";
    india_playbook: 0, 0, 6, "Hard India plays the designer's 8.9.25 plan (2026-11-05 review, section 4d; docs/sim-results/ai-quality/india-playbook-1105/): bit flags, India only. Bit 1 (Walls as a city's garrison) is retired: every Hard faction plays it through `walls_garrison` since 2026-11-06. 2: remove India's extra network guard and use the shared defend margin (200) and door defense (100). 4: while at War with the strict VP leader at 60% of its target, refuse Peace. Shared default 0; Hard India plays 2 via BUILTIN_FACTION_DECISIONS. Measured on the pre-`walls_garrison` AI: bits 1+2 gained 6.03 percentage points [+4.79, +7.27] of India win share and 5.27 [+4.03, +6.52] on held-out seed 8241001, bit 1 alone 4.39 and bit 2 alone 1.06 [-0.01, +2.13]; bit 4 alone measured -0.05 and stays off";
    operations: 0, 0, 1, "Hard (AI strategy reset increment 1b, 2026-11-07): honour a committed operation (`operation.rs`): its rostered units take orders from the operation executor before every other military pass (a home city actually under attack releases the units near it, logged), its plan outranks the plan layer and faction rules such as Sparta's campaign rule until it completes or aborts, a strike declares War when assembled and refuses Peace while it runs, and a bought strike spends the treasury where the strike assembles. Nothing in live play creates an operation yet; offline only (`operation_search`) (0: off)";
    breach: 0, 0, 400, "Hard (Training round 2 distilled, 2026-11-08; the designer's 8.9.27 Germany T144: two Bombers took Xochimilco and Chalco to 0 HP and an Infantry and a Cavalry walked in, two captures in one turn; `breach.rs`, a pass right after door defence): Hard prices a breach on every visible enemy city at war. The shots already ordered at the city count first; then its free shooters in range and its aircraft in range fire at it (largest expected damage first, ground before air, until the city reaches 0 HP 95% of the time or every shot is used), and the nearest, then cheapest, free melee unit that reaches the city this turn walks in (ranged fire resolves before movement); its route is reserved. The chance the shots zero the city is exact over the uniform rolls (`breach.rs::zero_chance`); a garrison, or an owner's melee unit that can step in first (owners reinforce their attacked cities before movement), multiplies it by the walker's win chance against the strongest of them on the fallen city. The breach is ordered when that capture chance times the city's value (400 + 80 per population, Production) times this percent, kept by the walker against the visible enemy power that can reach the city, beats the walker's expected loss if the city keeps HP. 0: off; 100: the city at its value";
    forecast_garrison_holds: 1, 0, 2, "Hard (siege forecast calibration, 2026-11-08; `opportunity.rs::forecast_siege_rounds`): the strike forecast no longer has the besieged city's own garrison charge out at the weakest attacker every round; it holds the city and only shoots if it can. In the combat lab Hard garrisons made 0 melee sorties in 403 siege turns, and the old assumption made the forecast call winnable storms hopeless. 1: only in the siege stance's storm test (`breach.rs`); 2: in every siege forecast, including the plan layer's campaign and Peace valuations (on India 8.9.25 from T80, level 2 kept a losing war going: seat −12.5 pp). 0: off. Default on since 2026-11-08 with forecast_garrison_holds 1, siege_stance 6, stance_home_guard 6 and stance_plan_war 1 (docs/sim-results/ai-quality/siege-forecast-1108/: combat lab 29.8% to 41.7%, war suite +5.6, designer seat at 200 salts +1.2 [-0.5, +2.9] (held out +0.5 [-2.0, +3.0]), human seat seat-only +1.46 [-0.05, +2.98] and rivals-only -1.86 [-3.61, -0.10])";
    stance_plan_war: 1, 0, 1, "Hard, with siege_stance (2026-11-08, from the India 8.9.25 designer-seat losses): the storm and siege stance only take on cities of the enemy the plan fights (its Campaign or Contain war) and the running campaign's or conquest's city. A stance on any other war's city turned a dead war into a pressing one, so Hard refused the Peace it signs without the stance. 0: off. Default on since 2026-11-08 with forecast_garrison_holds 1, siege_stance 6, stance_home_guard 6 and stance_plan_war 1 (docs/sim-results/ai-quality/siege-forecast-1108/: combat lab 29.8% to 41.7%, war suite +5.6, designer seat at 200 salts +1.2 [-0.5, +2.9] (held out +0.5 [-2.0, +3.0]), human seat seat-only +1.46 [-0.05, +2.98] and rivals-only -1.86 [-3.61, -0.10])";
    stance_home_guard: 6, 0, 12, "Hard, with siege_stance (2026-11-08, from the Mongols 8.9.25 and India 8.9.25 designer-seat losses): units within this many hexes of an own city under real threat (more than half its strength) are left out of the storm and siege stance on any city farther than this many hexes from that home, so the army doesn't leave a threatened home to take a distant city (a city near the home is still fair game; without that exception Hollow Swiss T76 fell from 100% to 3%). 0: off. Default on since 2026-11-08 with forecast_garrison_holds 1, siege_stance 6, stance_home_guard 6 and stance_plan_war 1 (docs/sim-results/ai-quality/siege-forecast-1108/: combat lab 29.8% to 41.7%, war suite +5.6, designer seat at 200 salts +1.2 [-0.5, +2.9] (held out +0.5 [-2.0, +3.0]), human seat seat-only +1.46 [-0.05, +2.98] and rivals-only -1.86 [-3.61, -0.10])";
    siege_stance: 6, 0, 12, "Hard (Training round 2 stances, distilled 2026-11-08; `breach.rs::siege_stance_cities`, a pass right after the city-assault coordinator): for each visible enemy city at war with our free units near it, a storm when the strike forecast (`forecast_siege_rounds` on the free units, this many rounds) takes the city for fewer Production lost than its value (400 + 80 per population): the units that hit it this turn do (shooters in range, melee that reach it), the rest stay with the other passes; otherwise a siege when the shooters near it and our aircraft zero it within this many turns at their expected damage per turn: the shooters take firing tiles (`siege_city`) and the melee hold out of the city's strike within a move of it, for the breach (gene `breach`). The same test replaces the open-city seizure's wait while a walled city still fires above half HP (`military.rs::seize_open_city`). 0: off (the old half-HP wait). Default on since 2026-11-08 with forecast_garrison_holds 1, siege_stance 6, stance_home_guard 6 and stance_plan_war 1 (docs/sim-results/ai-quality/siege-forecast-1108/: combat lab 29.8% to 41.7%, war suite +5.6, designer seat at 200 salts +1.2 [-0.5, +2.9] (held out +0.5 [-2.0, +3.0]), human seat seat-only +1.46 [-0.05, +2.98] and rivals-only -1.86 [-3.61, -0.10])";
    capture_horizon: 0, 0, 400, "Hard (Lab card Ride to Dyrrachium, 2026-11-10; `plan.rs::capture_turns`, `plan.rs::price_campaign`): a campaign is priced by when its capture lands. The forecast turns of a siege on a walled city whose force has no Siege and none it could buy count at this percent after the march (the forecast halves every attack for the Walls, but not what a slow siege under the city's strike costs in execution: the Mongol army forecast Nicomedia in 4 turns at 45% and took it within 4 turns in 7 of 50 runs, about what 8 turns give), and the supply term counts only the share of the 10-turn plan horizon left after that capture (a deposit put to work on turn 9 is worth a tenth of one put to work now). 0: off. 200: on, walled sieges without Siege count double";
    strike_upgrade_wait: 0, 0, 1, "Hard (Lab cards Ride to Dyrrachium and Two Cities, 2026-11-10; `modernization.rs::in_strike`, `upgrade_waits_for_attack`): a unit the running strike counts on (the active campaign's committed force, a running operation's roster, or a unit reaching an assault tile of the routed Campaign's city within `march_turns`) doesn't upgrade: an upgrade takes all its moves. The card loop found Horseman #42's T37 upgrade decides Dyrrachium; on Two Cities the Cavalry upgraded to a Tank on T144 mid-strike. 0: off";
    short_objective: 0, 0, 1, "Hard (Lab cards, 2026-11-10; `short_objective.rs`): a short-horizon objective step before the plan is chosen. It nominates every enemy city the forces in reach take within 5 turns by the siege forecast and every own city a visible enemy army is forecast to take within 5 turns, prices each capture with `price_campaign` under `capture_horizon` 200 (or the gene's own value when set) less the expected loss of the home cities forecast to fall, and holds the best through a pinned operation: a Campaign on the city, or a Hold on the most endangered home city when no offence is worth its home risk. It changes course when the pinned capture's odds stay under 15% for two turns (that city is then left alone for 5 turns), when a better objective beats it by `switch_margin`, or when it is met. Reads only the seat's view. 0: off";
    relief_forecast: 0, 0, 1, "Hard (Fable seat, After the Riflemen, 2026-11-13, lesson 1; `opportunity.rs::relief_units`): the capture forecasts (`forecast_siege_rounds`, `city_assault.rs::city_assault_succeeds`) model the core's relief chain: the city owner's melee units that can step into the city this turn hold to relieve it instead of attacking, and when an attack clears the garrison (or walks into the empty fallen city) the healthiest steps in and is fought by the next attacker, so a lone melee attacker's capture fails while one stands by. The coordinator's lone walk-in of an empty fallen city, and its trim to one capturer, apply only when no relief stands by. 0: off";
    charge_preview: 0, 0, 1, "Hard (Fable seat, After the Riflemen, 2026-11-13, lesson 3; `breach.rs`, `city_assault.rs`): a melee charge at a city that the forecast does not have capturing it this turn is held when the engine's preview (`melee_forecast`, this turn's ordered shots counted) has it dying more often than not or taking more damage than it deals (the teacher held a Rifleman on 'deal 22, take 39' and let the shooters work). 0: off";
    relief_kill: 0, 0, 200, "Hard (Fable seat, After the Riflemen, 2026-11-13, lesson 2; `tactics.rs::relief_stake`): kill the units beside a city before walking in. An enemy melee unit that can step into its owner's city (a relief, `opportunity::relief_units`) while one of our melee units can reach that city adds this percent of the city's value (400 + 80 per population) to the harm its removal averts (`enemy_threat`, which prices shots and sorties), scaled by the share of the city's HP already gone. 0: off";
    capturer_upgrade: 0, 0, 400, "Hard (Fable seat, After the Riflemen, 2026-11-13, lesson 4; `modernization.rs::capturer`): Gold goes to the unit that can take the city. A melee unit within 4 hexes of a visible enemy city at war has its refit score (combat gain per Gold) raised by this percent, and when that refit ranks first but is not yet affordable while Gold within `march_turns` of income covers it, its cost is kept in hand (no lesser refit, and no later purchase that would dip into it) until it is (the teacher skipped a 58 Gold Marksman refit to upgrade a Soldier to a Rifleman for 210 and took the city with it). 0: off";
    emergency_bombardment_only: 0, 0, 1, "Hard (production alternation, 2026-11-15, docs/sim-results/ai-quality/production-alternation-1115/; `production.rs::plan_emergency_production`): the emergency production pass no longer fires on `needs_ranged` alone, the standing want for a shooter when an enemy unit stands within 3 hexes of a garrisoned city with no shooter within 1. It fires only for bombardment, an exposed capture or threat above local defence. Without it the pass swapped the build for a shooter and the routine pass swapped it back next turn (3.2 times in 20 turns in the G4 scenario). Removes the one interrupt for a shooter with an enemy at 3 hexes. 0: off";
    refit_before_siege: 1, 0, 3, "Hard (Lab card After the Riflemen, 2026-11-13, docs/sim-results/ai-quality/campaign-holds-1113/; `siege_buy.rs::wall_proof_refits`): Gold goes to a refit that brings Siege built in before Siege is bought. The force near the walled city our war marches on: each unit whose refit gives a Siege-built-in kind, orderable now or unlocked by the technology we finish within this many turns, keeps its refit Gold in hand (cheapest first, as far as Gold plus that many turns of income reaches); Siege is bought only beyond it, never for a unit about to refit, and nothing is saved for Siege while a refit waits. In 26 of 50 runs Hard bought Siege for a Horseman on T135, a turn before Cannonry, and could not refit two Arquebusiers to Field Cannons (74 Gold each, Siege built in); 24 of those 26 lost. 0: off. Default 1 since 2026-11-14 (designer approved after the full-game gate, docs/sim-results/ai-quality/refit-gate-1114/: 1,024 paired human-seat games, win 33.1% to 34.9%, +1.8 pp [0.0, +3.7], no measure worse)";
    regen_strike: 0, 0, 100, "Hard (Fable seat, After the Riflemen, 2026-11-13, lesson 5; `tactics.rs::regen_strike_at`): a walled enemy city at 0 HP with none of our military units beside it heals 20 HP at the end of the turn and strikes in the next ranged phase, before movement; a tile within 2 of it (not adjacent, which blocks the heal) is charged that strike at this percent in `city_damage_at`, which places shooters, siege holds and assault tiles. Mounted units leading a charge (the other half of the lesson) are left to `charge_preview`: the engine's preview already carries their -33% against cities. 0: off";
    runaway_coalition: 55, 0, 63, "Hard (designer's Horde scenario and the 8.9.26 reviews, 2026-11-07; `runaway.rs`): bit flags against a runaway leader read from public information only (the strict VP leader with a VP lead, an army lead, cities captured recently, empires eliminated, or a hold). 1: diplomacy: an Alliance with anyone at war with the runaway is valued by how much of the runaway's army the pair can bring, a call to arms against it is answered, and no Peace is offered to or accepted from it while it runs away (unless one of our cities is about to fall). 2: second front: an empire at war with the runaway, or allied with one that is, commits an operation (`operation.rs`, the executor honoured for it) on the runaway's weakest city in reach when half the runaway's public army is visibly 8+ tiles from that city 4: standing bloc (A1, 2026-11-08, designer's wariness design): each empire keeps a hidden wariness toward every other from public signs (recent captures, 3 to 5 a roll-over and 1 to 2 almost nothing; eliminations; visible army build-up; VP lead; all scaled by nearness), and values an Alliance, at Peace included, with a partner wary of the same empire by the weaker wariness, the share of that empire's army the pair can bring and adjacency, chosen by a seeded draw; the bloc-size cost is waived for a bloc wary of the same empire. 16: coalition wars fight at once: at war with an empire we are wary of that another empire also fights, an operation strikes its weakest reachable city with no army-away test, keeping one defender in each city its army is near. 32: wary guard: one extra home troop per city within 10 tiles of the empire we fear most, by seeded draw (docs/design/ai-strategy/eu4-coalitions-2026-11-08.md). 8: strike a hollow runaway (A2): a runaway counting down or reading as hollow is struck on any weakly held city in reach by an operation that needs no army away and no war under way (it declares when assembled). Default 55 since 2026-11-08 (bits 1, 2, 4, 16, 32; bit 8 off; main's call after the 55 arm: seat +0.39 pp, designer seat held out 57.4 -> 58.5; docs/sim-results/ai-quality/coalition-bloc-1108/); bits 1 and 2 were on (3) since 2026-11-07 (lead's call): Horde scenario Byzantine starting cities held 0.50 -> 3.12, Peace with the Mongols 1.25 -> 0.59 a run, Sparta's second front in 41 of 64 runs; full games barely move (human-seat rivals arm +0.54 pp [-0.88, +1.90], census best/worst 2.45 -> 2.37 and 2.51 -> 2.46): the gate's Hard seat is rarely the runaway, the designer usually is (docs/sim-results/ai-quality/coalition-front-1107/)";
    reachable_wars: 1, 0, 1, "Hard (Germany 8.9.43 review, 2026-11-15): a new Contain, runaway-leader Campaign, or coalition war needs two own field units near the same known target city and enough local army to meet nearby defenders, a per-city share of unseen public army and one-turn nearby purchases from public Gold. A new Contain or Campaign war waits after a recent Campaign against another empire; a Contain declaration during another war and a second-front operation need a funded campaign forecast. 0: old declaration pricing";
    leader_war: 1, 0, 1, "Hard (Swiss 8.9.72 review, idea 58): a VP contender's clearly best Contain war may replace a different front when two units can reach a known city before the race closes. The reach estimate counts purchases near that city; Swiss Gold within one defender purchase of the Reserve is valued as VP at risk. 0: keep reachable_wars without the exception";
    held_shot_stepin: 1, 0, 1, "Hard (hill-climb candidate D2, a bug on every rules version made worse by rules 47's held shots; `military.rs::may_close_into`): a free unit closing in on its own besieged city (the `relief` branch of `defend`, taken when every safe tile beside the city is taken or under fire) no longer steps into the city itself when the city is at 0 HP and the unit cannot survive there (`can_defend_at` false). Traced on M51-geneva: the Swiss Horseman at 100 HP stepped into empty Geneva on T67 with six Cataphracts in range and six in melee reach, so the held volley and the charge killed it with the city lost anyway. 0: it may step in";
    storm_fallen: 1, 0, 1, "Hard (rules-50 stall trace, seed 21999: Egypt held Mongol Karakorum at 0 HP for 888 turns to the 1,200-turn cap with 21 units within 8 hexes and 3 melee attacks in all; the coastal city has two land neighbours, one held by Egypt's own CombatDrone shooting from it, so every charger's staging search found no tile and held, and an adjacent Mongol unit kept the open-city capture off it; `military.rs::storm_fallen`): before withdrawals and strikes, a fallen (0 HP) enemy city at war whose every land neighbour is held is stormed by a free charger beside it whose forecast is not too risky; otherwise the nearest charger in the open-city radius works in through our own non-capturing units, swapping places each turn with the one on its next step. Units near an own threatened city stay home. A city with a free land tile beside it is left to the ordinary passes (an earlier version that also staged chargers there moved B40-zürich-139 by -29 pp) (0: off)";
    german_industry_food: 1, 0, 1, "Hard Germany prices the Food lost to each Production building, especially in a small city; 0 restores the former Ruhr Foundry-only Food valuation";
}

/// Built-in per-faction decision defaults: `(faction, gene, value)`, applied on top of the shared
/// defaults for that faction's AI. This is where the climb program banks a faction-specific winner
/// and where a former hard-coded faction gate keeps its old value as a gene
/// (`docs/sim-results/ai-quality/climb-program/README.md`). A parameter file still wins: its own
/// `faction_overrides`, and any shared gene it sets away from the shared default.
/// One entry per line in exactly this form; `tools/evolve/climb_program.py` reads them.
pub const BUILTIN_FACTION_DECISIONS: &[(Faction, &str, i32)] = &[
    (Faction::Sparta, "conquest_min_cities", 0),
    (Faction::Mongols, "conquest_walled_forecast", 1),
    (Faction::Sparta, "lone_city_strike", 1),
    (Faction::Sparta, "early_campaign_value", 160),
    (Faction::Sparta, "target_wall_cost", 180),
    (Faction::Sparta, "campaign_wall_cost", 120),
    (Faction::Sparta, "contain_march_outlook", 1),
    (Faction::Sparta, "contain_stack_weight", 35),
    (Faction::Sparta, "march_ignores_dead_war", 1),
    (Faction::Sparta, "march_ignores_second_war", 1),
    (Faction::Sparta, "chain_press", 1),
    (Faction::Sparta, "open_followup", 1),
    (Faction::Sparta, "assault_wing", 1),
    (Faction::Sparta, "walled_prize", 1),
    (Faction::Sparta, "engaged_assault_keep", 1),
    (Faction::Sparta, "early_prize_opening", 1),
    (Faction::Sparta, "army_item_bonus", 100),
    (Faction::Sparta, "war_shrine_drive", 1),
    (Faction::Sparta, "planned_stage", 1),
    (Faction::Sparta, "sparta_force_campaign", 1),
    (Faction::Sparta, "early_campaign_muster", 1),
    (Faction::Sparta, "campaign_march_patience", 12),
    (Faction::Celts, "strike_eagerness", 10),
    (Faction::Sparta, "strike_eagerness", 30),
    (Faction::Mongols, "strike_eagerness", 25),
    (Faction::Mongols, "mongol_horde", 3),
    (Faction::Sparta, "approach_scout", 1),
    (Faction::Mongols, "recon_weight", 2),
    (Faction::Mongols, "scout_priority_bonus", 150),
    (Faction::Sparta, "army_hard_extra", 4),
    (Faction::Sparta, "army_military_bonus", 0),
    (Faction::Mongols, "courier_raid_only", 1),
    (Faction::Celts, "bronze_bias", 70),
    (Faction::Celts, "irrigation_bias", 70),
    (Faction::Mongols, "currency_bias", 100),
    (Faction::Mongols, "banking_bias", 60),
    (Faction::Sparta, "feudalism_bias", 180),
    (Faction::Sparta, "chivalry_bias", 120),
    (Faction::India, "law_tech_bonus", 80),
    (Faction::India, "campaign_contain_only", 1),
    (Faction::India, "second_front_temperament", -60),
    (Faction::India, "leader_commit", 2),
    (Faction::India, "defend_margin_percent", 250),
    (Faction::India, "door_defense", 150),
    (Faction::Celts, "crop_tech_bonus", 100),
    (Faction::Sparta, "infantry_bonus", 100),
    (Faction::Sparta, "roadworks_war_base", 1000),
    (Faction::Mongols, "roadworks_war_base", 240),
    (Faction::Sparta, "roadworks_war_per_enemy", 100),
    (Faction::Mongols, "roadworks_war_per_enemy", 40),
    (Faction::Sparta, "second_front_temperament", 25),
    (Faction::Mongols, "second_front_temperament", 10),
    (Faction::Athens, "army_growth_trigger", 277),
    (Faction::Athens, "army_rush_extra", 6),
    (Faction::Athens, "army_war_extra", 0),
    (Faction::Athens, "attack_ratio_percent", 103),
    (Faction::Athens, "campaign_commit", 631),
    (Faction::Athens, "campaign_persist", 98),
    (Faction::Athens, "campaign_range", 10),
    (Faction::Athens, "capture_economy_percent", 118),
    (Faction::Athens, "conquest_home_distance", 19),
    (Faction::Athens, "conquest_min_cities", 1),
    (Faction::Athens, "conquest_path_len", 14),
    (Faction::Athens, "conquest_runaway_percent", 43),
    (Faction::Athens, "era_entry_percent", 49),
    (Faction::Athens, "keep_settler", 89),
    (Faction::Athens, "min_seat_war_odds", 2),
    (Faction::Athens, "objective_bonus_percent", 73),
    (Faction::Athens, "open_city_radius", 7),
    (Faction::Athens, "opening_chain_turn", 44),
    (Faction::Athens, "peace_credibility", 481),
    (Faction::Athens, "peace_lean", 0),
    (Faction::Athens, "production_building_percent", 70),
    (Faction::Athens, "production_military_percent", 72),
    (Faction::Athens, "production_settler_percent", 131),
    (Faction::Athens, "recover_hp", 82),
    (Faction::Athens, "relative_city_percent", 102),
    (Faction::Athens, "research_economy_percent", 74),
    (Faction::Athens, "research_military_percent", 96),
    (Faction::Athens, "research_science_percent", 94),
    (Faction::Athens, "reserve_deficit_turns", 6),
    (Faction::Athens, "ruin_bonus", 80),
    (Faction::Athens, "rush_window_turn", 125),
    (Faction::Athens, "scout_find", 390),
    (Faction::Athens, "scout_priority_bonus", 150),
    (Faction::Athens, "scribe_bias", 140),
    (Faction::Athens, "settler_chain_priority", 496),
    (Faction::Athens, "settler_max_site_distance", 14),
    (Faction::Athens, "settler_min_site_score", 90),
    (Faction::Athens, "siege_window", 2),
    (Faction::Athens, "staging_damage_percent", 80),
    (Faction::Athens, "strike_eagerness", -60),
    (Faction::Athens, "surplus_purchase_percent", 259),
    (Faction::Athens, "surplus_upkeep_turns", 88),
    (Faction::Athens, "switch_margin", 117),
    (Faction::Athens, "troop_runway", 26),
    (Faction::Athens, "walled_quota", 6),
    (Faction::Athens, "war_lean", 137),
    (Faction::Athens, "war_pressure", 3),
    (Faction::Athens, "war_siege", 0),
    (Faction::Byzantium, "army_growth_trigger", 14),
    (Faction::Byzantium, "army_hard_extra", 8),
    (Faction::Byzantium, "army_rush_extra", 8),
    (Faction::Byzantium, "attack_ratio_percent", 138),
    (Faction::Byzantium, "campaign_commit", 845),
    (Faction::Byzantium, "campaign_persist", 0),
    (Faction::Byzantium, "campaign_range", 24),
    (Faction::Byzantium, "capture_economy_percent", 83),
    (Faction::Byzantium, "chain_press", 1),
    (Faction::Byzantium, "conquest_home_distance", 6),
    (Faction::Byzantium, "conquest_min_cities", 0),
    (Faction::Byzantium, "conquest_path_len", 17),
    (Faction::Byzantium, "conquest_runaway_percent", 25),
    (Faction::Byzantium, "era_entry_percent", 0),
    (Faction::Byzantium, "escort_home_distance", 0),
    (Faction::Byzantium, "feudalism_bias", 60),
    (Faction::Byzantium, "keep_settler", 0),
    (Faction::Byzantium, "min_seat_war_odds", 40),
    (Faction::Byzantium, "objective_bonus_percent", 0),
    (Faction::Byzantium, "open_city_radius", 5),
    (Faction::Byzantium, "opening_chain_turn", 30),
    (Faction::Byzantium, "peace_credibility", 331),
    (Faction::Byzantium, "peace_lean", 214),
    (Faction::Byzantium, "planned_stage", 1),
    (Faction::Byzantium, "production_building_percent", 172),
    (Faction::Byzantium, "production_military_percent", 95),
    (Faction::Byzantium, "production_settler_percent", 132),
    (Faction::Byzantium, "purchase_min_value", 25),
    (Faction::Byzantium, "recover_hp", 45),
    (Faction::Byzantium, "relative_city_percent", 99),
    (Faction::Byzantium, "research_economy_percent", 171),
    (Faction::Byzantium, "research_military_percent", 200),
    (Faction::Byzantium, "research_science_percent", 107),
    (Faction::Byzantium, "reserve_deficit_turns", 14),
    (Faction::Byzantium, "ruin_bonus", 80),
    (Faction::Byzantium, "rush_window_turn", 139),
    (Faction::Byzantium, "scout_find", 497),
    (Faction::Byzantium, "scout_priority_bonus", 150),
    (Faction::Byzantium, "settler_chain_priority", 0),
    (Faction::Byzantium, "settler_max_site_distance", 4),
    (Faction::Byzantium, "settler_min_site_score", 33),
    (Faction::Byzantium, "settler_open_cities", 5),
    (Faction::Byzantium, "staging_damage_percent", 126),
    (Faction::Byzantium, "strike_eagerness", 14),
    (Faction::Byzantium, "surplus_purchase_percent", 265),
    (Faction::Byzantium, "surplus_upkeep_turns", 90),
    (Faction::Byzantium, "switch_margin", 39),
    (Faction::Byzantium, "troop_runway", 41),
    (Faction::Byzantium, "veteran_infantry_bonus", 100),
    (Faction::Byzantium, "walled_quota", 2),
    (Faction::Byzantium, "war_lean", 242),
    (Faction::Byzantium, "war_siege", 300),
    (Faction::Switzerland, "army_banker", 1),
    (Faction::Switzerland, "army_growth_trigger", 111),
    (Faction::Switzerland, "army_hard_extra", 1),
    (Faction::Switzerland, "army_item_bonus", 31),
    (Faction::Switzerland, "army_rush_extra", 7),
    (Faction::Switzerland, "attack_ratio_percent", 93),
    (Faction::Switzerland, "campaign_commit", 48),
    (Faction::Switzerland, "campaign_march_patience", 7),
    (Faction::Switzerland, "campaign_persist", 166),
    (Faction::Switzerland, "campaign_range", 11),
    (Faction::Switzerland, "capture_economy_percent", 79),
    (Faction::Switzerland, "conquest_bank_goal", 1),
    (Faction::Switzerland, "conquest_home_distance", 15),
    (Faction::Switzerland, "conquest_min_cities", 3),
    (Faction::Switzerland, "conquest_path_len", 18),
    (Faction::Switzerland, "conquest_runaway_percent", 36),
    (Faction::Switzerland, "currency_tech_bonus", 20),
    (Faction::Switzerland, "era_entry_percent", 11),
    (Faction::Switzerland, "escort_home_distance", 3),
    (Faction::Switzerland, "keep_settler", 37),
    (Faction::Switzerland, "min_seat_war_odds", 20),
    (Faction::Switzerland, "objective_bonus_percent", 116),
    (Faction::Switzerland, "open_city_radius", 5),
    (Faction::Switzerland, "opening_chain_turn", 86),
    (Faction::Switzerland, "peace_credibility", 344),
    (Faction::Switzerland, "peace_lean", 32),
    (Faction::Switzerland, "production_building_percent", 96),
    (Faction::Switzerland, "production_military_percent", 114),
    (Faction::Switzerland, "production_settler_percent", 85),
    (Faction::Switzerland, "recover_hp", 54),
    (Faction::Switzerland, "relative_city_percent", 95),
    (Faction::Switzerland, "research_economy_percent", 78),
    (Faction::Switzerland, "research_military_percent", 59),
    (Faction::Switzerland, "research_science_percent", 122),
    (Faction::Switzerland, "reserve_deficit_turns", 20),
    (Faction::Switzerland, "rush_window_turn", 162),
    (Faction::Switzerland, "scout_find", 514),
    (Faction::Switzerland, "second_front_temperament", -15),
    (Faction::Switzerland, "settler_chain_priority", 0),
    (Faction::Switzerland, "settler_max_site_distance", 12),
    (Faction::Switzerland, "settler_min_site_score", 84),
    (Faction::Switzerland, "siege_window", 2),
    (Faction::Switzerland, "staging_damage_percent", 113),
    (Faction::Switzerland, "strike_eagerness", -44),
    (Faction::Switzerland, "surplus_purchase_percent", 169),
    (Faction::Switzerland, "surplus_upkeep_turns", 88),
    (Faction::Switzerland, "switch_margin", 45),
    (Faction::Switzerland, "toll_market_bonus", 60),
    (Faction::Switzerland, "troop_runway", 21),
    (Faction::Switzerland, "walled_quota", 6),
    (Faction::Switzerland, "walls_near_enemy_bonus", 150),
    (Faction::Switzerland, "war_lean", 198),
    (Faction::Switzerland, "war_pressure", 9),
    (Faction::Switzerland, "war_siege", 170),
    (Faction::Germany, "army_growth_trigger", 226),
    (Faction::Germany, "army_item_bonus", 15),
    (Faction::Germany, "army_rush_extra", 2),
    (Faction::Germany, "attack_ratio_percent", 102),
    (Faction::Germany, "bronze_bias", 100),
    (Faction::Germany, "campaign_commit", 714),
    (Faction::Germany, "campaign_persist", 147),
    (Faction::Germany, "campaign_range", 7),
    (Faction::Germany, "capture_economy_percent", 78),
    (Faction::Germany, "conquest_home_distance", 13),
    (Faction::Germany, "conquest_min_cities", 3),
    (Faction::Germany, "conquest_path_len", 15),
    (Faction::Germany, "era_entry_percent", 22),
    (Faction::Germany, "escort_home_distance", 1),
    (Faction::Germany, "industry_tech_bonus", 100),
    (Faction::Germany, "keep_settler", 36),
    (Faction::Germany, "min_seat_war_odds", 14),
    (Faction::Germany, "objective_bonus_percent", 93),
    (Faction::Germany, "open_city_radius", 9),
    (Faction::Germany, "opening_chain_turn", 36),
    (Faction::Germany, "peace_credibility", 514),
    (Faction::Germany, "peace_lean", 86),
    (Faction::Germany, "production_building_percent", 128),
    (Faction::Germany, "production_military_percent", 123),
    (Faction::Germany, "production_settler_percent", 121),
    (Faction::Germany, "purchase_min_value", 2),
    (Faction::Germany, "recover_hp", 30),
    (Faction::Germany, "relative_city_percent", 149),
    (Faction::Germany, "research_economy_percent", 133),
    (Faction::Germany, "research_military_percent", 86),
    (Faction::Germany, "research_science_percent", 96),
    (Faction::Germany, "reserve_deficit_turns", 0),
    (Faction::Germany, "scout_find", 252),
    (Faction::Germany, "second_front_temperament", -10),
    (Faction::Germany, "settler_chain_priority", 0),
    (Faction::Germany, "settler_max_site_distance", 9),
    (Faction::Germany, "settler_min_site_score", 37),
    (Faction::Germany, "settler_open_cities", 5),
    (Faction::Germany, "staging_damage_percent", 81),
    (Faction::Germany, "storm_forecast", 2),
    (Faction::Germany, "surplus_purchase_percent", 234),
    (Faction::Germany, "surplus_upkeep_turns", 67),
    (Faction::Germany, "switch_margin", 45),
    (Faction::Germany, "troop_runway", 25),
    (Faction::Germany, "walled_quota", 0),
    (Faction::Germany, "war_commit_ratio", 131),
    (Faction::Germany, "war_lean", 108),
    (Faction::Germany, "war_siege", 218),
    (Faction::Switzerland, "swiss_playbook", 3),
    (Faction::Switzerland, "swiss_hoard_any_war", 1),
    (Faction::Celts, "celts_playbook", 2),
    (Faction::Aztecs, "aztecs_playbook", 7),
    (Faction::India, "india_playbook", 2),
    (Faction::Egypt, "egypt_food_buildings", 1),
];

impl DecisionParams {
    /// The shared defaults with this faction's built-in entries applied.
    pub fn builtin_for(faction: Faction) -> Self {
        Self::default().with_builtin_faction(faction)
    }

    /// Apply the faction's built-in entries to every gene still at its shared default.
    fn with_builtin_faction(mut self, faction: Faction) -> Self {
        let shared = Self::default();
        for &(f, name, value) in BUILTIN_FACTION_DECISIONS {
            if f == faction && self.get_named(name) == shared.get_named(name) {
                let known = self.set_named(name, value);
                debug_assert!(known, "unknown gene {name} in BUILTIN_FACTION_DECISIONS");
            }
        }
        self
    }

    /// The decisions a faction actually plays with.
    pub fn for_faction(mut self, faction: Faction) -> Self {
        if faction == Faction::Sparta {
            self.recover_hp = self.recover_hp.min(self.sparta_march_min_hp);
        }
        // Gene `swiss_playbook` 2: a walled colony costs no troop, so the Swiss expand at the
        // shared pace where the climb (round 1, 2026-10-24, before the Redoubt garrison) set
        // their expansion genes below it.
        if faction == Faction::Switzerland && self.swiss_playbook >= 2 {
            let shared = Self::default();
            self.settler_chain_priority = self
                .settler_chain_priority
                .max(shared.settler_chain_priority);
            self.settler_min_site_score = self
                .settler_min_site_score
                .min(shared.settler_min_site_score);
            self.production_settler_percent = self
                .production_settler_percent
                .max(shared.production_settler_percent);
        }
        if faction == Faction::Switzerland && self.swiss_playbook >= 3 {
            self.settler_chain_priority = self.settler_chain_priority.max(1500);
            self.settler_min_site_score = self.settler_min_site_score.min(40);
        }
        // India playbook bit 2 removes the extra guards that its remittance cannot fund.
        // Keep the older India values in BUILTIN_FACTION_DECISIONS for gene-off controls.
        if faction == Faction::India && self.india_playbook & 2 != 0 {
            let shared = Self::default();
            self.india_network_guard = 0;
            self.defend_margin_percent = shared.defend_margin_percent;
            self.door_defense = shared.door_defense;
        }
        self
    }
}

/// A versioned set of AI decision parameters. Anything missing falls back to the built-in
/// defaults, so an empty file (`{"schema": 1, "name": "builtin"}`) is the stock AI.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiParams {
    pub schema: u32,
    /// Identity recorded in sim reports (`scripted@<name>`).
    pub name: String,
    /// Free text: where the parameters came from (run id, generation, candidate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Base knobs per faction, replacing `Knobs::for_faction`. Difficulty scaling still
    /// applies on top, so Easy < Normal < Hard stays ordered.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub factions: BTreeMap<Faction, KnobValues>,
    /// Final knobs for one difficulty and faction, used as-is instead of scaling the base.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub difficulty: BTreeMap<AiDifficulty, BTreeMap<Faction, KnobValues>>,
    #[serde(default)]
    pub decisions: DecisionParams,
    /// Partial per-faction decision genes; present values override the shared defaults.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub faction_overrides: BTreeMap<Faction, DecisionOverrides>,
}

impl AiParams {
    /// The built-in personalities written out in full: every faction's base knobs. The
    /// evolution harness starts from this, so the defaults live only in `Knobs::for_faction`.
    pub fn builtin() -> Self {
        Self {
            schema: PARAMS_SCHEMA,
            name: "builtin".into(),
            source: Some("Knobs::for_faction".into()),
            factions: Faction::ALL
                .into_iter()
                .map(|f| (f, Knobs::for_faction(f).into()))
                .collect(),
            difficulty: BTreeMap::new(),
            decisions: DecisionParams::default(),
            faction_overrides: BTreeMap::new(),
        }
    }

    /// Reject anything a decision parameter can't be: wrong schema, a blank name, or a knob
    /// outside 0-100 (values are never silently clamped, so a file means what it says).
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != PARAMS_SCHEMA {
            return Err(format!(
                "AI params schema {} is not supported (expected {PARAMS_SCHEMA})",
                self.schema
            ));
        }
        if self.name.trim().is_empty()
            || !self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            return Err("AI params name must be nonempty ASCII letters, digits, - _ or .".into());
        }
        let check = |what: String, v: &KnobValues| {
            let k = Knobs::from(*v);
            if k == k.clamped() {
                Ok(())
            } else {
                Err(format!("{what}: every knob must be within 0..=100"))
            }
        };
        for (f, v) in &self.factions {
            check(format!("factions.{f:?}"), v)?;
        }
        for (d, by_faction) in &self.difficulty {
            for (f, v) in by_faction {
                check(format!("difficulty.{d:?}.{f:?}"), v)?;
            }
        }
        self.decisions.validate()?;
        for (faction, overrides) in &self.faction_overrides {
            overrides.validate(*faction)?;
        }
        Ok(())
    }

    /// The faction's base personality before difficulty scaling.
    pub fn base(&self, faction: Faction) -> Knobs {
        self.factions
            .get(&faction)
            .map_or_else(|| Knobs::for_faction(faction), |v| (*v).into())
    }

    /// The knobs a brain of this faction and difficulty plays with.
    pub fn knobs(&self, faction: Faction, d: AiDifficulty) -> Knobs {
        self.difficulty
            .get(&d)
            .and_then(|m| m.get(&faction))
            .map_or_else(
                || difficulty::scale_knobs(self.base(faction), d),
                |v| (*v).into(),
            )
    }

    /// Shared decision defaults with this faction's explicitly present genes applied.
    pub fn decisions(&self, faction: Faction) -> DecisionParams {
        let decisions = self.decisions.with_builtin_faction(faction);
        self.faction_overrides
            .get(&faction)
            .map_or(decisions, |overrides| decisions.with_overrides(overrides))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Brain, ScriptedBrain};
    use legatum_core::{GameSetup, PlayerId, new_game, player_view};

    #[test]
    fn gate_gene_arms_apply_effective_values_after_faction_builtins() {
        let arms: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tools/sim/fixtures/gene-arms-builtins.json"
        ))
        .unwrap();
        let names = [
            "swiss_playbook",
            "celts_playbook",
            "aztecs_playbook",
            "india_playbook",
            "walls_garrison",
            "army_war_extra",
        ];
        for arm in arms["arms"].as_array().unwrap() {
            let params: AiParams = serde_json::from_value(arm["params"].clone()).unwrap();
            let off = arm["label"] != "all";
            assert_eq!(params.faction_overrides.len(), Faction::ALL.len());
            for faction in Faction::ALL {
                let faction_name = format!("{faction:?}");
                assert!(
                    params.faction_overrides.contains_key(&faction),
                    "every arm must explicitly set {faction:?}"
                );
                let effective = params.decisions(faction);
                for name in names {
                    let expected = if off {
                        0
                    } else {
                        BUILTIN_FACTION_DECISIONS
                            .iter()
                            .find(|(f, gene, _)| *f == faction && *gene == name)
                            .map_or_else(
                                || DecisionParams::default().get_named(name).unwrap(),
                                |(_, _, value)| *value,
                            )
                    };
                    assert_eq!(
                        effective.get_named(name),
                        Some(expected),
                        "{} {faction:?} {name}",
                        arm["label"]
                    );
                    assert_eq!(
                        arm["params"]["faction_overrides"][faction_name.as_str()][name].as_i64(),
                        Some(i64::from(expected)),
                        "{} must explicitly set {faction:?} {name}",
                        arm["label"]
                    );
                }
            }
        }
    }

    #[test]
    fn old_japan_decision_names_load() {
        let decisions: DecisionParams = serde_json::from_value(serde_json::json!({
            "japan_network_guard": 0,
            "japan_objective_reach": 0
        }))
        .unwrap();
        assert_eq!(decisions.india_network_guard, 0);
        assert_eq!(decisions.india_objective_reach, 0);
    }

    #[test]
    fn india_playbook_defaults_to_two_and_gene_off_restores_the_extra_guards() {
        let builtin = AiParams::default().decisions(Faction::India);
        assert_eq!(builtin.india_playbook, 2);
        let on = builtin.for_faction(Faction::India);
        assert_eq!(
            (
                on.india_network_guard,
                on.defend_margin_percent,
                on.door_defense
            ),
            (0, 200, 100)
        );

        let params: AiParams = serde_json::from_str(
            r#"{"schema":1,"name":"india-off","faction_overrides":{"India":{"india_playbook":0}}}"#,
        )
        .unwrap();
        let off = params.decisions(Faction::India).for_faction(Faction::India);
        assert_eq!(
            (
                off.india_playbook,
                off.india_network_guard,
                off.defend_margin_percent,
                off.door_defense
            ),
            (0, 1, 250, 150)
        );
        assert_eq!(
            AiParams::default()
                .decisions(Faction::Switzerland)
                .india_playbook,
            0
        );
    }

    #[test]
    fn round_two_lists_germany_gates_and_dormant_commitments_without_changing_defaults() {
        let set: serde_json::Value =
            serde_json::from_str(include_str!("../../../tools/evolve/genesets/climb-r2.json"))
                .unwrap();
        let genes: Vec<&str> = set["genes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        let mut unique = genes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), genes.len());
        for name in [
            "conquest_enabled",
            "siege_exempt",
            "campaign_contain_only",
            "scout_front",
            "leader_commit",
            "war_pressure",
            "war_garrison",
            "peace_retaliation",
            "settler_safety",
            "peace_scout_danger",
        ] {
            assert!(
                genes.contains(&format!("decision.{name}").as_str()),
                "{name}"
            );
        }
        let germany = DecisionParams::builtin_for(Faction::Germany);
        assert_eq!(
            (
                germany.conquest_enabled,
                germany.siege_exempt,
                germany.campaign_contain_only,
                germany.scout_front
            ),
            (1, 0, 0, 1)
        );
        let shared = DecisionParams::default();
        assert_eq!(
            (
                shared.leader_commit,
                shared.war_pressure,
                shared.war_garrison,
                shared.peace_retaliation,
                shared.settler_safety,
                shared.peace_scout_danger
            ),
            (0, 0, 0, 0, 0, 0)
        );
    }

    #[test]
    fn builtin_faction_decisions_name_real_genes_in_range() {
        let genes = DecisionParams::genes();
        for &(faction, name, value) in BUILTIN_FACTION_DECISIONS {
            let gene = genes
                .iter()
                .find(|g| g.name == format!("decision.{name}"))
                .unwrap_or_else(|| panic!("{faction:?}: unknown gene {name}"));
            assert!(
                (gene.min..=gene.max).contains(&value),
                "{faction:?}.{name}={value}"
            );
            assert_ne!(
                value, gene.default,
                "{faction:?}.{name} repeats the shared default"
            );
            let p = AiParams {
                schema: 1,
                name: "t".into(),
                ..AiParams::default()
            };
            assert_eq!(p.decisions(faction).get_named(name), Some(value));
            assert_eq!(
                DecisionParams::builtin_for(faction).get_named(name),
                Some(value)
            );
        }
        let mut d = DecisionParams::default();
        assert!(d.set_named("siege_window", 3) && d.siege_window == 3);
        assert!(!d.set_named("vision", 1));
    }

    #[test]
    fn empty_params_are_the_builtin_ai() {
        let p = AiParams {
            schema: 1,
            name: "builtin".into(),
            ..Default::default()
        };
        p.validate().unwrap();
        assert_eq!(p.decisions, DecisionParams::default());
        assert!(p.faction_overrides.is_empty());
        assert!(
            serde_json::to_value(&p)
                .unwrap()
                .get("faction_overrides")
                .is_none()
        );
        let state = new_game(&GameSetup::all_ai(99));
        for d in [AiDifficulty::Easy, AiDifficulty::Normal, AiDifficulty::Hard] {
            for f in Faction::ALL {
                assert_eq!(
                    p.knobs(f, d),
                    difficulty::scale_knobs(Knobs::for_faction(f), d)
                );
            }
            for player in &state.players {
                let id = player.id;
                let view = player_view(&state, id);
                let mut stock = ScriptedBrain::with_difficulty(player.faction, 7, id, d);
                let mut tuned = ScriptedBrain::with_params(player.faction, 7, id, d, &p);
                assert_eq!(stock.orders(&view), tuned.orders(&view));
                let mut replay = ScriptedBrain::with_params(player.faction, 7, id, d, &p);
                let mut replay_again = ScriptedBrain::with_params(player.faction, 7, id, d, &p);
                assert_eq!(replay.orders(&view), replay_again.orders(&view));
            }
        }
    }

    #[test]
    fn builtin_lists_every_faction_default() {
        let b = AiParams::builtin();
        b.validate().unwrap();
        assert_eq!(b.factions.len(), Faction::ALL.len());
        for f in Faction::ALL {
            assert_eq!(b.base(f), Knobs::for_faction(f));
        }
        let text = serde_json::to_string(&b).unwrap();
        assert_eq!(serde_json::from_str::<AiParams>(&text).unwrap(), b);
    }

    #[test]
    fn base_knobs_scale_by_difficulty_and_overrides_win() {
        let mut p: AiParams = serde_json::from_str(
            r#"{"schema":1,"name":"t","factions":{"India":{"expansion":10,"military":20,
            "science":30,"economy":40,"aggression":50}},"difficulty":{"Easy":{"Celts":
            {"expansion":1,"military":2,"science":3,"economy":4,"aggression":5}}}}"#,
        )
        .unwrap();
        p.validate().unwrap();
        let base = Knobs::new(10, 20, 30, 40, 50);
        assert_eq!(p.base(Faction::India), base);
        assert_eq!(p.knobs(Faction::India, AiDifficulty::Normal), base);
        assert_eq!(
            p.knobs(Faction::India, AiDifficulty::Hard),
            difficulty::scale_knobs(base, AiDifficulty::Hard)
        );
        assert_eq!(
            p.knobs(Faction::Celts, AiDifficulty::Easy),
            Knobs::new(1, 2, 3, 4, 5)
        );
        let brain =
            ScriptedBrain::with_params(Faction::India, 1, PlayerId(0), AiDifficulty::Hard, &p);
        assert_eq!(brain.base, base);
        assert_eq!(brain.knobs, p.knobs(Faction::India, AiDifficulty::Hard));
        p.factions.get_mut(&Faction::India).unwrap().military = 101;
        assert!(p.validate().is_err());
    }

    #[test]
    fn every_decision_gene_is_bounded_and_defaults_round_trip() {
        let defaults = AiParams::builtin();
        for gene in DecisionParams::genes() {
            let field = gene.name.strip_prefix("decision.").unwrap();
            let mut json = serde_json::to_value(&defaults).unwrap();
            assert_eq!(json["decisions"][field], gene.default);
            for bad in [gene.min - 1, gene.max + 1] {
                json["decisions"][field] = bad.into();
                assert!(
                    serde_json::from_value::<AiParams>(json.clone())
                        .unwrap()
                        .validate()
                        .is_err()
                );
            }
            for good in [gene.min, gene.max] {
                json["decisions"][field] = good.into();
                serde_json::from_value::<AiParams>(json.clone())
                    .unwrap()
                    .validate()
                    .unwrap();
            }
        }
    }

    #[test]
    fn faction_decisions_override_global_genes_and_leave_other_genes_and_factions_alone() {
        let p: AiParams = serde_json::from_str(
            r#"{"schema":1,"name":"t","decisions":{"siege_window":4,"recover_hp":55},
               "faction_overrides":{"Sparta":{"siege_window":7}}}"#,
        )
        .unwrap();
        p.validate().unwrap();
        let sparta = p.decisions(Faction::Sparta);
        assert_eq!(sparta.siege_window, 7);
        assert_eq!(sparta.recover_hp, 55);
        assert_eq!(p.decisions(Faction::India).siege_window, 4);
        assert_eq!(p.decisions(Faction::India).recover_hp, 55);
        assert_eq!(
            serde_json::to_value(&p).unwrap()["faction_overrides"]["Sparta"],
            serde_json::json!({"siege_window": 7})
        );
        let sparta_brain =
            ScriptedBrain::with_params(Faction::Sparta, 1, PlayerId(0), AiDifficulty::Hard, &p);
        assert_eq!(sparta_brain.decisions.siege_window, 7);
        assert_eq!(sparta_brain.decisions.recover_hp, 55);
        assert!(
            serde_json::from_str::<AiParams>(
                r#"{"schema":1,"name":"t","faction_overrides":{"Sparta":{"siege_window":9}}}"#,
            )
            .unwrap()
            .validate()
            .is_err()
        );
    }

    #[test]
    fn retired_gene_names_still_load_in_current_faction_parameters() {
        let p: AiParams = serde_json::from_str(
            r#"{"schema":1,"name":"legacy",
               "decisions":{"oathbound_force_campaign":1,
                            "tollkeeper_market_priority":321},
               "faction_overrides":{
                 "Switzerland":{"oathbound_search_only":1},
                 "India":{"siege_window":2},
                 "Byzantium":{"siege_window":3},
                 "Germany":{"siege_window":4},
                 "Celts":{"siege_window":5},
                 "Mongols":{"siege_window":6},
                 "Sparta":{"oathbound_force_campaign":1},
                 "Athens":{"siege_window":8}}}"#,
        )
        .unwrap();

        assert_eq!(p.decisions.sparta_force_campaign, 1);
        assert_eq!(p.decisions.swiss_market_priority, 321);
        assert_eq!(
            p.faction_overrides.len(),
            8,
            "parameter files may omit faction overrides"
        );
        // No override for the Aztecs: the shared decisions plus their built-in entries.
        assert_eq!(
            p.decisions(Faction::Aztecs),
            p.decisions.with_builtin_faction(Faction::Aztecs)
        );
        assert_eq!(p.decisions(Faction::Switzerland).sparta_search_only, 1);
        assert_eq!(p.decisions(Faction::Sparta).sparta_force_campaign, 1);

        let saved = serde_json::to_value(&p).unwrap();
        assert!(saved["faction_overrides"].get("Switzerland").is_some());
        assert_eq!(saved["decisions"]["sparta_force_campaign"], 1);
        assert!(saved["decisions"].get("oathbound_force_campaign").is_none());
    }

    /// Pillar 6: the file format has no way to express rule changes.
    #[test]
    fn rules_cannot_be_smuggled_in() {
        for bad in [
            r#"{"schema":1,"name":"x","yields":{"food":9}}"#,
            r#"{"schema":1,"name":"x","decisions":{"vision":9}}"#,
            r#"{"schema":1,"name":"x","decisions":{"unit_cost":1}}"#,
            r#"{"schema":1,"name":"x","decisions":{"food_yield":9}}"#,
            r#"{"schema":1,"name":"x","factions":{"India":{"expansion":1,"military":1,
               "science":1,"economy":1,"aggression":1,"vision":9}}}"#,
            r#"{"schema":1,"name":"x","factions":{"India":{"expansion":1}}}"#,
            r#"{"schema":1,"name":"x","factions":{"Atlantis":{"expansion":1,"military":1,
               "science":1,"economy":1,"aggression":1}}}"#,
            r#"{"schema":1,"name":"x","faction_overrides":{"Sparta":{"vision":1}}}"#,
            r#"{"schema":1,"name":"x","faction_overrides":{"Atlantis":{"siege_window":1}}}"#,
        ] {
            assert!(serde_json::from_str::<AiParams>(bad).is_err(), "{bad}");
        }
        let p: AiParams = serde_json::from_str(r#"{"schema":2,"name":"x"}"#).unwrap();
        assert!(p.validate().is_err());
        let p: AiParams = serde_json::from_str(r#"{"schema":1,"name":"a/b"}"#).unwrap();
        assert!(p.validate().is_err());
    }
}

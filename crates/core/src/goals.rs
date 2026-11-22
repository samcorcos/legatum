//! Faction goals: each faction's one objective, worth `data::FACTION_GOAL_VP` (2) for every
//! faction (user rule, 2026-10-06). This module is the single source for what a goal is
//! (`spec`), whether it is met (`reached`) and how far along it is (`progress`); scoring, the
//! views and the UI export (`crates/ai/tests/ui_rules.rs`) all read it.
//!
//! A goal is one of three kinds:
//! - **Permanent**: earned once, kept for good.
//! - **Live**: scores only while its condition stands; it is lost the turn it stops holding and
//!   regained when it holds again (the Switzerland' hoard).
//! - **Held**: a capturable item; whoever holds it scores (Byzantium's Heir; the
//!   Yam Relay Station).

use crate::data;
use crate::experiments::{FactionRule, RuleExperiments};
use crate::state::{GameState, Unit};
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalKind {
    Permanent,
    Live,
    Held,
}

/// Which rule a faction's goal follows. A goal is a rule, not a faction: the Swarm was a
/// population goal and is now the Aztecs' New Fire; Sakoku waits for Japan. `default_rule` gives
/// each faction its rule; `Player::goal_rule` overrides it (scenarios and tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalRule {
    /// Egypt and Rome have no faction goal in their first playable increment.
    None,
    /// Switzerland: hold the hoard of Gold (Live).
    Hoard,
    /// Aztecs: offer population in the New Fire (Permanent, an action).
    Swarm,
    /// Reserved for Japan: close the country and hold it closed (Live).
    Sakoku,
    /// India: hold the Imperial Diwan's city (Held).
    Diwan,
    /// Byzantium: hold the Heir (Held).
    Heir,
    /// Germany: own five Ruhr Foundries (Live).
    Factories,
    /// Celts: a wild camp in every city (Live).
    Wild,
    /// Sparta: retire veterans (Permanent, an action).
    Retire,
    /// Mongols: hold the Yam Relay Station (Held).
    Hall,
    /// Athens: share the learning (Permanent, an action).
    Broadcast,
}

/// Switzerland: the treasury a hoard needs.
pub const SWISS_HOARD_GOLD: i32 = 5_000;
/// The Swarm (`GoalRule::Swarm`): population sacrificed (calibrated 2026-10-07: 30, about a
/// third of seats), now the Aztecs' New Fire.
pub const AZTEC_NEW_FIRE_POPULATION: i32 = 30;
/// Japan, Sakoku (`GoalRule::Sakoku`): turn ends the country stays closed before it holds its VP.
pub const SAKOKU_TURNS: i32 = 30;
/// Athens, Share the Signal: free technologies each rival chooses.
pub const SIGNAL_FREE_TECHS: u8 = 3;
/// Sparta, Honour the Fallen: veterans to retire in an Agoge city.
pub const SPARTA_RETIRE_COUNT: i32 = 3;
/// Sparta, Honour the Fallen: the promotion level a retiring veteran needs.
pub const SPARTA_RETIRE_LEVEL: u8 = 3;

/// The rules facts of a faction's goal. Text (names, rule sentences) lives in the app's
/// `FACTION_GOAL`, which reads these numbers from the generated rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoalSpec {
    pub rule: GoalRule,
    pub kind: GoalKind,
    pub vp: u32,
    /// The number the goal counts toward.
    pub target: i32,
    /// What `progress` counts, for a meter's label ("Gold in the treasury").
    pub label: &'static str,
}

/// The rule each faction's goal follows.
pub fn default_rule(faction: Faction) -> GoalRule {
    match faction {
        Faction::Switzerland => GoalRule::Hoard,
        Faction::India => GoalRule::Diwan,
        Faction::Byzantium => GoalRule::Heir,
        Faction::Germany => GoalRule::Factories,
        Faction::Celts => GoalRule::Wild,
        Faction::Sparta => GoalRule::Retire,
        Faction::Mongols => GoalRule::Hall,
        Faction::Athens => GoalRule::Broadcast,
        Faction::Aztecs => GoalRule::Swarm,
        Faction::Egypt => GoalRule::None,
        Faction::Rome => GoalRule::None,
    }
}

/// The rule empire `p`'s goal follows right now.
pub fn rule(state: &GameState, p: PlayerId) -> GoalRule {
    let player = state.player(p);
    if p.is_barbarian() {
        // The barbarian record borrows a faction for combat; it has no goal.
        return player.goal_rule.unwrap_or(GoalRule::Broadcast);
    }
    player
        .goal_rule
        .unwrap_or_else(|| default_rule(player.faction))
}

/// The reason a goal action gives while an ablation has switched its goal off.
pub const SWITCHED_OFF: &str = "This faction goal is switched off";

/// The faction rule that switches a goal off for ablation (`--rule-off`), if it has one.
pub fn goal_switch(rule: GoalRule) -> Option<FactionRule> {
    match rule {
        GoalRule::None => None,
        GoalRule::Hoard => Some(FactionRule::SwissGoal),
        GoalRule::Diwan => Some(FactionRule::IndiaGoal),
        GoalRule::Heir => Some(FactionRule::ByzantiumGoal),
        GoalRule::Factories => Some(FactionRule::GermanyGoal),
        GoalRule::Wild => Some(FactionRule::CeltsGoal),
        GoalRule::Hall => Some(FactionRule::MongolsGoal),
        GoalRule::Retire => Some(FactionRule::SpartaGoal),
        GoalRule::Broadcast => Some(FactionRule::AthensGoal),
        GoalRule::Swarm => Some(FactionRule::AztecsGoal),
        GoalRule::Sakoku => None,
    }
}

/// Whether `rule`'s goal is switched off in `rules`: it scores nothing and cannot be reached.
pub fn rule_switched_off(rules: RuleExperiments, rule: GoalRule) -> bool {
    goal_switch(rule).is_some_and(|r| rules.rule_off(r))
}

/// Whether empire `p`'s own goal is switched off.
pub fn switched_off(state: &GameState, p: PlayerId) -> bool {
    rule_switched_off(state.setup.rule_experiments, rule(state, p))
}

/// The goal facts of a faction's own rule, as played under `rules`: a switched-off goal is worth
/// no VP, so valuations that read `.vp` never plan around it.
pub fn spec_in(rules: RuleExperiments, faction: Faction) -> GoalSpec {
    let mut spec = spec(faction);
    if rule_switched_off(rules, spec.rule) {
        spec.vp = 0;
    }
    spec
}

/// The goal facts of a rule.
pub fn spec_for(rule: GoalRule) -> GoalSpec {
    let goal = |kind, target, label| GoalSpec {
        rule,
        kind,
        vp: data::FACTION_GOAL_VP,
        target,
        label,
    };
    match rule {
        GoalRule::None => GoalSpec {
            rule,
            kind: GoalKind::Permanent,
            vp: 0,
            target: 0,
            label: "no faction goal yet",
        },
        GoalRule::Hoard => goal(GoalKind::Live, SWISS_HOARD_GOLD, "Gold in the treasury"),
        GoalRule::Swarm => goal(
            GoalKind::Permanent,
            AZTEC_NEW_FIRE_POPULATION,
            "population to spare",
        ),
        // Sakoku: live, held once the country stays closed for the required turns.
        GoalRule::Sakoku => goal(GoalKind::Live, SAKOKU_TURNS, "turns closed"),
        GoalRule::Diwan => goal(GoalKind::Held, 1, "Imperial Diwan held"),
        // The Heir: held, not counted.
        GoalRule::Heir => goal(GoalKind::Held, 1, "Heir to the Throne held"),
        // Five owned cities with a Ruhr Foundry; other cities do not affect the goal.
        GoalRule::Factories => goal(GoalKind::Live, data::GERMANY_GOAL_CITIES, "Ruhr Foundries"),
        // Call the warbands: live, a wild camp in the territory of every city owned (no minimum).
        GoalRule::Wild => goal(GoalKind::Live, 1, "cities with a wild camp"),
        // Honour the Fallen: three veterans retired in an Agoge city.
        GoalRule::Retire => goal(GoalKind::Permanent, SPARTA_RETIRE_COUNT, "veterans retired"),
        // The Yam Relay Station: held, not counted.
        GoalRule::Hall => goal(GoalKind::Held, 1, "Yam Relay Station held"),
        GoalRule::Broadcast => goal(GoalKind::Permanent, 1, "learning shared"),
    }
}

/// The goal facts of a faction's own rule.
pub fn spec(faction: Faction) -> GoalSpec {
    spec_for(default_rule(faction))
}

/// The goal facts of empire `p` (its `rule`).
pub fn spec_of(state: &GameState, p: PlayerId) -> GoalSpec {
    spec_for(rule(state, p))
}

/// The goal's target.
pub fn target(faction: Faction) -> i32 {
    rule_target(default_rule(faction))
}

/// `target` for a rule.
pub fn rule_target(rule: GoalRule) -> i32 {
    spec_for(rule).target
}

/// Whether the empire's own goal condition stands this turn (Permanent and Live goals).
pub fn reached(state: &GameState, p: PlayerId) -> bool {
    if switched_off(state, p) {
        return false;
    }
    let player = state.player(p);
    match rule(state, p) {
        GoalRule::None => false,
        GoalRule::Hoard => player.gold >= rule_target(GoalRule::Hoard),
        // the New Fire, the broadcast and the retirement are actions: the action itself sets
        // `faction_goal_done`.
        GoalRule::Swarm | GoalRule::Broadcast | GoalRule::Retire => player.faction_goal_done,
        GoalRule::Sakoku => sakoku_holding(state, p),
        GoalRule::Diwan => diwan_holder(state, p) == Some(p),
        GoalRule::Heir => heir_holder(state, p) == Some(p),
        GoalRule::Factories => factory_cities(state, p) >= data::GERMANY_GOAL_CITIES,
        GoalRule::Wild => {
            let (with_camp, owned) = wild_camp_cities(state, p);
            owned >= 1 && with_camp == owned
        }
        GoalRule::Hall => hall_holder(state, p) == Some(p),
    }
}

/// Turn ends the country must stay closed before Sakoku holds.
pub fn sakoku_target() -> u32 {
    rule_target(GoalRule::Sakoku) as u32
}

/// Whether empire `p` has closed the country (Sakoku): it can't declare war or join an
/// Alliance, learns nothing from abroad (no tech diffusion), and no foreign unit stands in its
/// land at Peace (`GameState::passage_allowed`). Barbarians are never closed.
pub fn is_closed(state: &GameState, p: PlayerId) -> bool {
    !p.is_barbarian() && state.player(p).sakoku_turns.is_some()
}

/// Whether a team has a member that has closed the country. Alliances are between teams, so one
/// closed member keeps the whole team out of them.
pub fn team_closed(state: &GameState, team: TeamId) -> bool {
    state
        .players
        .iter()
        .any(|q| q.team == team && q.alive && q.sakoku_turns.is_some())
}

/// Sakoku scores after the country has stayed closed for the full countdown.
pub fn sakoku_holding(state: &GameState, p: PlayerId) -> bool {
    state
        .player(p)
        .sakoku_turns
        .is_some_and(|n| n >= sakoku_target())
        && !p.is_barbarian()
}

/// Celts, Call the warbands: `(cities of `p` with a barbarian camp in their territory, cities owned)`.
/// Natural camps never stand inside borders, so a camp in a city's territory is a rite camp.
pub fn wild_camp_cities(state: &GameState, p: PlayerId) -> (i32, i32) {
    let camps: Vec<_> = state
        .barbarians
        .iter()
        .flat_map(|b| b.camps.iter().map(|c| c.pos))
        .collect();
    let mut with_camp = 0;
    let mut owned = 0;
    for c in state.cities_of(p) {
        owned += 1;
        if camps
            .iter()
            .any(|h| state.map.get(*h).is_some_and(|t| t.city == Some(c.id)))
        {
            with_camp += 1;
        }
    }
    (with_camp, owned)
}

/// Population the empire can offer: every city keeps at least 1.
pub fn spare_population(state: &GameState, p: PlayerId) -> i32 {
    state.cities_of(p).map(|c| (c.population - 1).max(0)).sum()
}

/// What the New Fire takes from each city, as `(city, population lost)` in city-id order: one
/// population at a time from the currently biggest city (ties: the lower city id), every city
/// keeping at least 1. Short of `AZTEC_NEW_FIRE_POPULATION` in total when the empire can't spare
/// that much (the action is then invalid). Validation, the view's preview and resolution all
/// read this.
pub fn swarm_plan(state: &GameState, p: PlayerId) -> Vec<(CityId, i32)> {
    let mut left: Vec<(CityId, i32)> = state.cities_of(p).map(|c| (c.id, c.population)).collect();
    let mut loss: std::collections::BTreeMap<CityId, i32> = Default::default();
    for _ in 0..AZTEC_NEW_FIRE_POPULATION {
        // `max_by_key` keeps the last maximum, so iterate ids in reverse to prefer the lower id.
        let Some(biggest) = left
            .iter_mut()
            .rev()
            .filter(|(_, pop)| *pop > 1)
            .max_by_key(|(_, pop)| *pop)
        else {
            break;
        };
        biggest.1 -= 1;
        *loss.entry(biggest.0).or_default() += 1;
    }
    loss.into_iter().collect()
}

/// Cities of `p` with a Ruhr Foundry standing, including cities in resistance.
pub fn factory_cities(state: &GameState, p: PlayerId) -> i32 {
    state
        .cities_of(p)
        .filter(|c| data::has_building(&c.buildings, BuildingKind::Forge))
        .count() as i32
}

/// The Heir of Byzantium empire `of`, wherever he stands (at most one exists: he can be retrained
/// only once none does). A captured Heir keeps `heir_of`.
pub fn heir(state: &GameState, of: PlayerId) -> Option<&Unit> {
    state
        .units
        .values()
        .find(|u| u.kind == UnitKind::HeirToTheThrone && u.heir_of == Some(of))
}

/// Who holds `of`'s Heir and so scores the Byzantium goal: his owner, unless barbarians took him
/// (then nobody scores until a camp is cleared and he is freed).
pub fn heir_holder(state: &GameState, of: PlayerId) -> Option<PlayerId> {
    heir(state, of)
        .map(|u| u.owner)
        .filter(|o| !o.is_barbarian())
}

/// The city holding Mongols empire `of`'s Yam Relay Station: the city it was built in, if it still
/// stands with the Station in it (a Celts capture razes the city and its wonder tile). A captured
/// Station normally stays in its city; a released tile can be claimed by another city.
pub fn hall_city(state: &GameState, of: PlayerId) -> Option<&crate::state::City> {
    state
        .player(of)
        .hall_city
        .and_then(|id| state.cities.get(&id))
        .filter(|c| data::has_building(&c.buildings, BuildingKind::YamRelayStation))
}

/// Who holds `of`'s Yam Relay Station and so scores the Mongols goal: the owner of its tile.
pub fn hall_holder(state: &GameState, of: PlayerId) -> Option<PlayerId> {
    if let Some((_, holder)) = crate::landmarks::site(state, of, BuildingKind::YamRelayStation) {
        return holder.filter(|o| !o.is_barbarian());
    }
    hall_city(state, of)
        .map(|c| c.owner)
        .filter(|o| !o.is_barbarian())
}

pub fn diwan_holder(state: &GameState, of: PlayerId) -> Option<PlayerId> {
    state
        .player(of)
        .diwan_city
        .and_then(|id| state.cities.get(&id))
        .filter(|c| data::has_building(&c.buildings, BuildingKind::ImperialDiwan))
        .map(|c| c.owner)
        .filter(|p| !p.is_barbarian())
}

pub fn imperial_diwans(state: &GameState) -> Vec<crate::view::ImperialDiwanView> {
    state
        .players
        .iter()
        .filter_map(|p| {
            let city = p.diwan_city?;
            Some(crate::view::ImperialDiwanView {
                of: p.id,
                holder: diwan_holder(state, p.id)?,
                city,
            })
        })
        .collect()
}

/// Whether `owner` holds a Yam Relay Station: every empire may then cross all of its land at Peace
/// (`GameState::passage_allowed`).
pub fn holds_open_road(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .iter()
        .any(|p| p.hall_city.is_some() && hall_holder(state, p.id) == Some(owner))
}

/// Every standing Yam Relay Station, for the public view.
pub fn courier_halls(state: &GameState) -> Vec<crate::view::CourierHallView> {
    state
        .players
        .iter()
        .filter_map(|p| {
            let city = hall_city(state, p.id)?;
            Some(crate::view::CourierHallView {
                of: p.id,
                holder: hall_holder(state, p.id)?,
                city: city.id,
                tile: crate::landmarks::find(state, p.id, BuildingKind::YamRelayStation)
                    .map(|(h, _)| h),
            })
        })
        .filter(|h| !h.holder.is_barbarian())
        .collect()
}

/// The empire that scores a Held goal of `of` right now, if any.
pub fn held_by(state: &GameState, of: PlayerId) -> Option<PlayerId> {
    if switched_off(state, of) {
        return None;
    }
    match rule(state, of) {
        GoalRule::Heir => heir_holder(state, of),
        GoalRule::Hall => hall_holder(state, of),
        GoalRule::Diwan => diwan_holder(state, of),
        _ => None,
    }
}

/// Who scores each goal right now: `(of, holder)` for every empire whose goal scores. A
/// Permanent or Live goal scores for its own empire (`Player::faction_goal_done`); a Held goal
/// scores for whoever holds the item.
pub fn holders(state: &GameState) -> Vec<(PlayerId, PlayerId)> {
    state
        .players
        .iter()
        .filter_map(|p| match spec_of(state, p.id).kind {
            GoalKind::Held => held_by(state, p.id).map(|h| (p.id, h)),
            _ => p.faction_goal_done.then_some((p.id, p.id)),
        })
        .collect()
}

/// How far a faction goal is: a value out of a target and a plain-words label, for its owner's
/// view only (rivals see done or not done). A Permanent goal has none once it is done; a Live goal
/// always has one, so its owner sees how far above or below it the empire stands. `value` and
/// `target` measure the same conditions as [`reached`], one number per condition met.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionGoalView {
    pub value: i32,
    pub target: i32,
    pub label: String,
    /// A Live goal scores only while it holds, and can be lost.
    #[serde(default)]
    pub live: bool,
    /// It scores now.
    #[serde(default)]
    pub done: bool,
}

fn thousands(n: i32) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

pub fn view(state: &GameState, p: PlayerId) -> Option<FactionGoalView> {
    if rule(state, p) == GoalRule::None {
        return None;
    }
    if switched_off(state, p) {
        return None;
    }
    let player = state.player(p);
    let rule = rule(state, p);
    let kind = spec_for(rule).kind;
    let live = kind == GoalKind::Live;
    // A held goal can be lost (the Heir captured), so its line stays like a live goal's.
    if player.faction_goal_done && !live && kind != GoalKind::Held {
        return None;
    }
    let (value, target, label) = match rule {
        GoalRule::None => unreachable!("no goal has no view"),
        GoalRule::Hoard => {
            let target = rule_target(GoalRule::Hoard);
            (
                player.gold.clamp(0, target),
                target,
                format!("{} of {} Gold", thousands(player.gold), thousands(target)),
            )
        }
        GoalRule::Swarm => {
            let spare = spare_population(state, p);
            (
                spare.min(AZTEC_NEW_FIRE_POPULATION),
                AZTEC_NEW_FIRE_POPULATION,
                format!("{spare} of {AZTEC_NEW_FIRE_POPULATION} population to spare"),
            )
        }
        GoalRule::Heir => {
            // Held: whoever owns the Heir scores; the owner's own meter says only whether it does.
            let holds = heir_holder(state, p) == Some(p);
            let label = match heir(state, p).map(|u| u.owner) {
                None => "The Heir to the Throne is not trained".to_string(),
                Some(o) if o == p => "The Heir to the Throne is yours".to_string(),
                Some(o) if o.is_barbarian() => "Barbarians hold the Heir".to_string(),
                Some(_) => "A rival holds the Heir".to_string(),
            };
            (i32::from(holds), 1, label)
        }
        GoalRule::Factories => {
            let factories = factory_cities(state, p);
            let target = data::GERMANY_GOAL_CITIES;
            (
                factories,
                target,
                format!("{factories} of {target} Ruhr Foundries"),
            )
        }
        GoalRule::Wild => {
            let (with_camp, owned) = wild_camp_cities(state, p);
            let target = owned.max(1);
            (
                with_camp,
                target,
                format!("{with_camp} of {target} cities with a wild camp in their territory"),
            )
        }
        GoalRule::Hall => {
            // Held: whoever owns the Station's city scores; the owner's own meter says only whether it does.
            let holds = hall_holder(state, p) == Some(p);
            let label = match hall_city(state, p).map(|c| (c.owner, c.name.as_str())) {
                None => "The Yam Relay Station is not built".to_string(),
                Some((o, name)) if o == p => format!("The Yam Relay Station stands in {name}"),
                Some((_, name)) => format!("A rival holds the Yam Relay Station in {name}"),
            };
            (i32::from(holds), 1, label)
        }
        GoalRule::Diwan => {
            let holds = diwan_holder(state, p) == Some(p);
            let label = state
                .player(p)
                .diwan_city
                .and_then(|id| state.cities.get(&id))
                .filter(|c| data::has_building(&c.buildings, BuildingKind::ImperialDiwan))
                .map_or("The Imperial Diwan is not built".to_string(), |c| {
                    if c.owner == p {
                        format!("The Imperial Diwan stands in {}", c.name)
                    } else {
                        format!("A rival holds the Imperial Diwan in {}", c.name)
                    }
                });
            (i32::from(holds), 1, label)
        }
        GoalRule::Retire => {
            let retired = i32::from(player.veterans_retired).min(SPARTA_RETIRE_COUNT);
            (
                retired,
                SPARTA_RETIRE_COUNT,
                format!("{retired} of {SPARTA_RETIRE_COUNT} veterans retired"),
            )
        }
        GoalRule::Sakoku => {
            let target = sakoku_target() as i32;
            match player.sakoku_turns {
                None => (0, target, format!("Not closed yet · 0 of {target} turns")),
                Some(n) if (n as i32) < target => {
                    (n as i32, target, format!("{n} of {target} turns closed"))
                }
                Some(_) => (
                    target,
                    target,
                    "The country has stayed closed: holding".to_string(),
                ),
            }
        }
        GoalRule::Broadcast => (0, 1, "The Learning is not shared yet".to_string()),
    };
    Some(FactionGoalView {
        value,
        target,
        label,
        live,
        done: player.faction_goal_done,
    })
}

#[cfg(test)]
#[path = "goals_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sakoku_tests.rs"]
mod sakoku_tests;

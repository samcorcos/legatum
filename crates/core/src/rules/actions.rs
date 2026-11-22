//! One-time empire and unit actions (`Order::Action`): the Swarm (the Aztecs' New Fire),
//! Japan's reserved Sakoku vow, the Athens' broadcast with its free-technology picks, Sparta's retirement of veterans and the
//! Celts's Call the warbands. Validation (`validate_action`), resolution
//! (`apply_actions`, phase 1 beside the city orders) and the `ActionsView` the UI and the AI read
//! all share the same checks.

use crate::data;
use crate::events::{Event, SwarmLoss};
use crate::goals::{self, GoalRule};
use crate::hex::Hex;
use crate::orders::{Action, Order, OrderError};
use crate::rules::promotion;
use crate::state::GameState;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Why `p` can't unleash the New Fire now, or None.
pub fn swarm_blocker(state: &GameState, p: PlayerId) -> Option<String> {
    let player = state.player(p);
    if goals::switched_off(state, p) {
        return Some(goals::SWITCHED_OFF.into());
    }
    if goals::rule(state, p) != GoalRule::Swarm {
        return Some("Only an empire whose goal is the New Fire can kindle it".into());
    }
    if player.faction_goal_done {
        return Some("The New Fire has already been kindled".into());
    }
    let spare = goals::spare_population(state, p);
    if spare < goals::AZTEC_NEW_FIRE_POPULATION {
        return Some(format!(
            "The New Fire needs {} population to offer, and every city keeps 1; you can spare {spare}",
            goals::AZTEC_NEW_FIRE_POPULATION
        ));
    }
    None
}

/// Why `p` can't close (`close`) or reopen the country now, or None. Closing needs Peace with
/// the world's Alliances behind it (a closed country can't be allied) and no captured Courier
/// Hall (the open road contradicts the vow).
pub fn sakoku_blocker(state: &GameState, p: PlayerId, close: bool) -> Option<String> {
    if goals::rule(state, p) != GoalRule::Sakoku {
        return Some("Only an empire with Sakoku can close the country".into());
    }
    let closed = goals::is_closed(state, p);
    if !close {
        return (!closed).then(|| "The country is already open".to_string());
    }
    if closed {
        return Some("The country is already closed".into());
    }
    if let Some(ally) = crate::diplomacy::allied_teams(state, state.team_of(p))
        .into_iter()
        .next()
    {
        return Some(format!(
            "Leave your Alliance with {} first: a closed country can't be allied",
            crate::scoring::team_label(state, ally)
        ));
    }
    if goals::holds_open_road(state, p) {
        return Some(
            "You hold a Courier Hall: its open road contradicts closing the country".into(),
        );
    }
    None
}

/// Why `p` can't broadcast now, or None.
pub fn broadcast_blocker(state: &GameState, p: PlayerId) -> Option<String> {
    let player = state.player(p);
    if goals::switched_off(state, p) {
        return Some(goals::SWITCHED_OFF.into());
    }
    if player.faction != Faction::Athens {
        return Some("Only Athens can share the learning".into());
    }
    if player.faction_goal_done {
        return Some("The learning has already been shared".into());
    }
    if broadcast_receivers(state, p).is_empty() {
        return Some("No rival empire is left to receive it".into());
    }
    None
}

/// A unit's promotion level under Sparta's ladder (they climb one level higher).
pub fn veteran_level(state: &GameState, unit: &crate::state::Unit) -> u8 {
    promotion::level_for_rules(
        state.setup.rule_experiments.faction_rules_off,
        Faction::Sparta,
        unit.xp,
    )
}

/// Why `p` can't retire `unit` now, or None. A veteran is a military unit of promotion level
/// `goals::SPARTA_RETIRE_LEVEL` or higher, standing in an own city with a War Shrine.
pub fn retire_blocker(state: &GameState, p: PlayerId, unit: UnitId) -> Option<String> {
    let player = state.player(p);
    if goals::switched_off(state, p) {
        return Some(goals::SWITCHED_OFF.into());
    }
    if player.faction != Faction::Sparta {
        return Some("Only Sparta can honour its veterans".into());
    }
    if player.faction_goal_done {
        return Some("Honour the Veterans is already complete".into());
    }
    let Some(u) = state.units.get(&unit).filter(|u| u.owner == p) else {
        return Some("That unit is not yours".to_string());
    };
    if !data::unit_stats(u.kind).military || u.kind == UnitKind::Scout {
        return Some("Only soldiers retire with honour".into());
    }
    let level = veteran_level(state, u);
    if level < goals::SPARTA_RETIRE_LEVEL {
        return Some(format!(
            "Only veterans of promotion level {} or higher retire; this one is level {level}",
            goals::SPARTA_RETIRE_LEVEL
        ));
    }
    let Some(city) = state.city_at(u.pos).filter(|c| c.owner == p) else {
        return Some("A veteran retires in one of your cities with an Agoge".into());
    };
    if !data::has_building(&city.buildings, BuildingKind::Agoge) {
        return Some(format!(
            "{} has no Agoge: build one to retire veterans here",
            city.name
        ));
    }
    None
}

/// Why `p` can't raise the warbands now, or None.
pub fn wild_blocker(state: &GameState, p: PlayerId) -> Option<String> {
    if goals::switched_off(state, p) {
        return Some(goals::SWITCHED_OFF.into());
    }
    if state.player(p).faction != Faction::Celts {
        return Some("Only the Celts can raise the warbands".into());
    }
    if state.cities_of(p).next().is_none() {
        return Some("Raising warbands needs a city to raise its camps in".into());
    }
    if crate::rules::barbarians::rite_plan(state, p).is_empty() {
        return Some(
            "No city needs a wild camp or has room for one: it needs a free tile of its territory, other than the city's own".into(),
        );
    }
    None
}

/// Every living empire outside `p`'s team: the broadcast's receivers.
pub fn broadcast_receivers(state: &GameState, p: PlayerId) -> Vec<PlayerId> {
    let team = state.team_of(p);
    state
        .players
        .iter()
        .filter(|q| q.alive && q.team != team)
        .map(|q| q.id)
        .collect()
}

/// Technologies `p` could take as free picks right now: the unknown ones of its current era.
pub fn free_tech_choices(state: &GameState, p: PlayerId) -> Vec<TechId> {
    TechId::ALL
        .iter()
        .copied()
        .filter(|t| crate::economy::tech_available(state, p, *t))
        .collect()
}

/// Unknown technologies across all eras: a grant carries into the next era if needed.
pub(crate) fn unknown_techs(state: &GameState, p: PlayerId) -> usize {
    TechId::ALL
        .iter()
        .filter(|tech| !state.player(p).techs.contains(tech))
        .count()
}

/// Territory tiles a city of `pop_after` population releases: what `settle_territory` will take,
/// in order (`next_release` on the shrinking grown-tile list).
pub fn released_tiles(state: &GameState, city: CityId, pop_after: i32) -> Vec<Hex> {
    let mut c = state.cities[&city].clone();
    let target = (pop_after - 1).max(0) as usize;
    let mut out = Vec::new();
    c.grown_tiles
        .retain(|h| state.map.get(*h).is_some_and(|t| t.city == Some(city)));
    while c.grown_tiles.len() > target {
        let Some(tile) = crate::rules::cities::next_release(state, &c) else {
            break;
        };
        c.grown_tiles.retain(|h| *h != tile);
        out.push(tile);
    }
    out
}

/// Check one action order against `state`.
pub fn validate_action(state: &GameState, player: PlayerId, action: &Action) -> Result<(), String> {
    let Some(p) = state.players.get(player.0 as usize) else {
        return Err(format!("Unknown player #{}", player.0));
    };
    if !p.alive {
        return Err(format!("{} has been eliminated", p.name));
    }
    match action {
        Action::Swarm => swarm_blocker(state, player).map_or(Ok(()), Err),
        Action::Sakoku { close } => sakoku_blocker(state, player, *close).map_or(Ok(()), Err),
        Action::Broadcast => broadcast_blocker(state, player).map_or(Ok(()), Err),
        Action::RetireVeteran { unit } => retire_blocker(state, player, *unit).map_or(Ok(()), Err),
        Action::CallTheWild => wild_blocker(state, player).map_or(Ok(()), Err),
        Action::ChooseFreeTechs { techs } => {
            if p.free_techs_pending == 0 {
                return Err("You have no free technologies to choose".into());
            }
            if techs.is_empty() {
                return Err("Choose at least one technology".into());
            }
            if techs.len() > usize::from(p.free_techs_pending) {
                return Err(format!(
                    "You can choose at most {} free technologies",
                    p.free_techs_pending
                ));
            }
            let mut chosen = BTreeSet::new();
            for tech in techs {
                let name = data::tech_name(*tech);
                if p.techs.contains(tech) {
                    return Err(format!("{name} is already researched"));
                }
                if !chosen.insert(*tech) {
                    return Err(format!("{name} is chosen more than once"));
                }
                if !crate::eras::researchable(p, *tech) {
                    return Err(format!(
                        "{name} is not in your era: free technologies come from the {} era",
                        crate::eras::era_name(crate::eras::current_era(&p.techs))
                    ));
                }
            }
            Ok(())
        }
    }
}

/// What makes two action orders the same order: the kind, and the unit of a unit action (several
/// veterans may retire in one turn, each once).
fn action_key(action: &Action) -> (std::mem::Discriminant<Action>, Option<UnitId>) {
    let unit = match action {
        Action::RetireVeteran { unit } => Some(*unit),
        _ => None,
    };
    (std::mem::discriminant(action), unit)
}

/// Validate every `Order::Action` in a player's list; each sees the effects of none of the others
/// (they are independent: one rite per faction and a choice that spends a pending grant), except
/// that retirements beyond what completes the goal are refused.
pub fn validate_actions(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError> {
    let mut errors = Vec::new();
    // gate-allow: duplicate check, lookups only, never iterated
    let mut seen = std::collections::HashSet::new();
    let mut retiring = i32::from(state.player(player).veterans_retired);
    for (index, order) in orders.iter().enumerate() {
        let Order::Action { action } = order else {
            continue;
        };
        if !seen.insert(action_key(action)) {
            errors.push(OrderError {
                index,
                message: "That action is already ordered this turn".into(),
            });
            continue;
        }
        if let Err(message) = validate_action(state, player, action) {
            errors.push(OrderError { index, message });
            continue;
        }
        // The vow resolves after this turn's declarations and treaty acceptances were checked
        // against an open country: refuse a close that rides with one.
        if matches!(action, Action::Sakoku { close: true })
            && let Some(message) = crate::diplomacy::vow_conflict(state, orders)
        {
            errors.push(OrderError { index, message });
            continue;
        }
        if matches!(action, Action::RetireVeteran { .. }) {
            if retiring >= goals::SPARTA_RETIRE_COUNT {
                errors.push(OrderError {
                    index,
                    message:
                        "Honour the Veterans is already covered by the veterans ordered to retire"
                            .into(),
                });
            }
            retiring += 1;
        }
    }
    errors
}

/// Resolve a player's actions: spending free techs first (a broadcast resolving this turn gives
/// receivers a choice for next turn), then the rites. Invalid actions are skipped and reported as
/// `OrderFailed`. Called once per player in id order by `resolve_turn`.
pub fn apply_actions(
    state: &mut GameState,
    player: PlayerId,
    orders: &[Order],
    events: &mut Vec<Event>,
) {
    // gate-allow: duplicate check, lookups only, never iterated
    let mut seen = std::collections::HashSet::new();
    let mut actions: Vec<&Action> = Vec::new();
    for order in orders {
        let Order::Action { action } = order else {
            continue;
        };
        if seen.insert(action_key(action)) {
            actions.push(action);
        } else {
            events.push(Event::OrderFailed {
                player,
                reason: "That action is already ordered this turn".into(),
            });
        }
    }
    // Spending free techs first (stable sort keeps list order otherwise).
    actions.sort_by_key(|a| !matches!(a, Action::ChooseFreeTechs { .. }));
    for action in actions {
        if let Err(reason) = validate_action(state, player, action) {
            events.push(Event::OrderFailed { player, reason });
            continue;
        }
        match action {
            Action::Swarm => swarm(state, player, events),
            Action::Sakoku { close } => sakoku(state, player, *close, events),
            Action::Broadcast => broadcast(state, player, events),
            Action::RetireVeteran { unit } => retire_veteran(state, player, *unit, events),
            Action::CallTheWild => call_the_wild(state, player, events),
            Action::ChooseFreeTechs { techs } => choose_free_techs(state, player, techs, events),
        }
    }
}

fn complete_goal(state: &mut GameState, player: PlayerId, events: &mut Vec<Event>) {
    let faction = state.player(player).faction;
    let p = state.player_mut(player);
    p.faction_goal_done = true;
    p.faction_goal_reached_before = true;
    events.push(Event::FactionGoalCompleted {
        player,
        faction,
        vp: goals::spec_of(state, player).vp,
        again: false,
    });
}

fn swarm(state: &mut GameState, player: PlayerId, events: &mut Vec<Event>) {
    let plan = goals::swarm_plan(state, player);
    let mut cities = Vec::new();
    for (id, lost) in plan {
        let c = state.cities.get_mut(&id).expect("planned city");
        c.population -= lost;
        c.food_stored = 0;
        let population = c.population;
        let before = state.cities[&id].grown_tiles.len();
        crate::rules::cities::settle_territory(state, id, events);
        let after = state.cities[&id].grown_tiles.len();
        cities.push(SwarmLoss {
            city: id,
            lost,
            population,
            tiles: before.saturating_sub(after) as u32,
        });
    }
    events.push(Event::SwarmUnleashed { player, cities });
    complete_goal(state, player, events);
}

fn sakoku(state: &mut GameState, player: PlayerId, close: bool, events: &mut Vec<Event>) {
    if close {
        state.player_mut(player).sakoku_turns = Some(0);
        events.push(Event::SakokuClosed { player });
    } else {
        let turns = state.player_mut(player).sakoku_turns.take().unwrap_or(0);
        events.push(Event::SakokuReopened { player, turns });
    }
}

fn broadcast(state: &mut GameState, player: PlayerId, events: &mut Vec<Event>) {
    let mut receivers = Vec::new();
    for q in broadcast_receivers(state, player) {
        // Nothing to give an empire that already knows everything.
        let unknown = unknown_techs(state, q);
        let grant = unknown.min(usize::from(goals::SIGNAL_FREE_TECHS)) as u8;
        if grant == 0 {
            continue;
        }
        let r = state.player_mut(q);
        r.free_techs_pending = r.free_techs_pending.saturating_add(grant);
        r.free_techs_from = Some(player);
        r.free_techs_captured = None;
        receivers.push(q);
    }
    events.push(Event::SignalBroadcast {
        from: player,
        receivers,
        techs_each: goals::SIGNAL_FREE_TECHS,
    });
    complete_goal(state, player, events);
}

fn retire_veteran(state: &mut GameState, player: PlayerId, unit: UnitId, events: &mut Vec<Event>) {
    let u = state.units.remove(&unit).expect("validated unit");
    let city = state.city_at(u.pos).expect("validated city").id;
    let p = state.player_mut(player);
    p.veterans_retired = p.veterans_retired.saturating_add(1);
    let retired = p.veterans_retired;
    events.push(Event::VeteranRetired {
        player,
        unit,
        kind: u.kind,
        city,
        retired,
        of: goals::SPARTA_RETIRE_COUNT as u8,
    });
    if i32::from(retired) >= goals::SPARTA_RETIRE_COUNT {
        complete_goal(state, player, events);
    }
}

fn call_the_wild(state: &mut GameState, player: PlayerId, events: &mut Vec<Event>) {
    let raised = crate::rules::barbarians::call_the_wild(state, player, events);
    events.push(Event::WildCalled {
        player,
        camps: raised.len() as u32,
    });
}

fn choose_free_techs(
    state: &mut GameState,
    player: PlayerId,
    techs: &[TechId],
    events: &mut Vec<Event>,
) {
    let from = state.player(player).free_techs_from;
    for tech in techs {
        crate::eras::learn(state, player, *tech, events);
    }
    let p = state.player_mut(player);
    p.free_techs_pending -= techs.len() as u8;
    // A grant that can no longer be spent (nothing left to learn) is dropped, never a stuck prompt.
    if crate::eras::all_researched(&p.techs) {
        p.free_techs_pending = 0;
    }
    if p.free_techs_pending == 0 {
        p.free_techs_from = None;
        p.free_techs_captured = None;
    }
    events.push(Event::FreeTechsChosen {
        player,
        from,
        techs: techs.to_vec(),
    });
}

/// One city's part of the New Fire, previewed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwarmCityView {
    pub city: CityId,
    pub lost: i32,
    /// Population after the sacrifice.
    pub population: i32,
    /// The territory tiles it would release with it.
    pub tiles: Vec<Hex>,
}

/// The reserved Swarm: whether it can be ordered, why not, and what it would take.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwarmView {
    /// Population sacrificed (`goals::AZTEC_NEW_FIRE_POPULATION`).
    pub population: i32,
    /// Population the empire can spare now (every city keeps 1).
    pub spare: i32,
    /// Why the New Fire can't be ordered now.
    pub blocker: Option<String>,
    pub cities: Vec<SwarmCityView>,
}

/// Japan's Sakoku vow: the state of the country, what ordering the next move needs, and what
/// the vow blocks. Own view only; rivals see the closed state and countdown on the public player.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SakokuView {
    /// The country is closed.
    pub closed: bool,
    /// Turn ends spent closed (0 while open).
    pub turns: u32,
    /// Turn ends the country must stay closed before it holds its VP.
    pub target: u32,
    /// The VP scores now: the countdown is done.
    pub holding: bool,
    /// Why the next move (closing, or reopening while closed) can't be ordered now.
    pub blocker: Option<String>,
}

/// The Athens' broadcast: whether it can be ordered, why not, and who receives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BroadcastView {
    pub techs_each: u8,
    pub blocker: Option<String>,
    pub receivers: Vec<PlayerId>,
}

/// Free technologies the viewer has to choose (a rival's broadcast).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreeTechsView {
    pub pending: u8,
    /// The empire whose broadcast (or captured city) granted them.
    pub from: Option<PlayerId>,
    /// Athens' Greece Captive: the city whose capture granted them; None for a broadcast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured: Option<CityId>,
    /// Technologies pickable now (prerequisites known); a pick can unlock more.
    pub choices: Vec<TechId>,
}

/// A veteran the Sparta could retire, and why he can't yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetireVeteranView {
    pub unit: UnitId,
    pub level: u8,
    /// Why this veteran can't retire where he stands (the action is offered only when None).
    pub blocker: Option<String>,
}

/// The Sparta's Honour the Fallen: progress, the cities where veterans retire and the
/// veterans themselves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetireView {
    pub retired: u8,
    /// Veterans to retire (`goals::SPARTA_RETIRE_COUNT`).
    pub target: u8,
    /// The promotion level a veteran needs (`goals::SPARTA_RETIRE_LEVEL`).
    pub level: u8,
    /// Own cities with a War Shrine: where veterans retire.
    pub shrines: Vec<CityId>,
    /// Every unit of the required level, in id order.
    pub veterans: Vec<RetireVeteranView>,
}

/// One city's wild camp, as Call the warbands would raise it now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildSiteView {
    pub city: CityId,
    pub at: Hex,
}

/// The Celts's Call the warbands: whether it can be ordered, why not, and where it would raise camps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WildView {
    pub blocker: Option<String>,
    /// The camps the call would raise now, one per city with room.
    pub sites: Vec<WildSiteView>,
    /// Cities with a camp in their territory now, and cities owned.
    pub with_camp: i32,
    pub cities: i32,
}

/// The viewer's one-time actions (their own view only).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionsView {
    pub swarm: Option<SwarmView>,
    pub sakoku: Option<SakokuView>,
    pub broadcast: Option<BroadcastView>,
    pub free_techs: Option<FreeTechsView>,
    pub retire: Option<RetireView>,
    pub wild: Option<WildView>,
}

pub fn view(state: &GameState, me: PlayerId) -> ActionsView {
    let player = state.player(me);
    let mut out = ActionsView::default();
    if !player.alive {
        return out;
    }
    if goals::rule(state, me) == GoalRule::Sakoku {
        let closed = goals::is_closed(state, me);
        out.sakoku = Some(SakokuView {
            closed,
            turns: player.sakoku_turns.unwrap_or(0),
            target: goals::sakoku_target(),
            holding: goals::sakoku_holding(state, me),
            blocker: sakoku_blocker(state, me, !closed),
        });
    }
    if goals::rule(state, me) == GoalRule::Swarm && !player.faction_goal_done {
        let cities = goals::swarm_plan(state, me)
            .into_iter()
            .map(|(city, lost)| {
                let population = state.cities[&city].population - lost;
                SwarmCityView {
                    city,
                    lost,
                    population,
                    tiles: released_tiles(state, city, population),
                }
            })
            .collect();
        out.swarm = Some(SwarmView {
            population: goals::AZTEC_NEW_FIRE_POPULATION,
            spare: goals::spare_population(state, me),
            blocker: swarm_blocker(state, me),
            cities,
        });
    }
    if player.faction == Faction::Athens && !player.faction_goal_done {
        out.broadcast = Some(BroadcastView {
            techs_each: goals::SIGNAL_FREE_TECHS,
            blocker: broadcast_blocker(state, me),
            receivers: broadcast_receivers(state, me),
        });
    }
    if player.faction == Faction::Sparta && !player.faction_goal_done {
        out.retire = Some(RetireView {
            retired: player.veterans_retired,
            target: goals::SPARTA_RETIRE_COUNT as u8,
            level: goals::SPARTA_RETIRE_LEVEL,
            shrines: state
                .cities_of(me)
                .filter(|c| data::has_building(&c.buildings, BuildingKind::Agoge))
                .map(|c| c.id)
                .collect(),
            veterans: state
                .units_of(me)
                .filter(|u| {
                    data::unit_stats(u.kind).military
                        && u.kind != UnitKind::Scout
                        && veteran_level(state, u) >= goals::SPARTA_RETIRE_LEVEL
                })
                .map(|u| RetireVeteranView {
                    unit: u.id,
                    level: veteran_level(state, u),
                    blocker: retire_blocker(state, me, u.id),
                })
                .collect(),
        });
    }
    if player.faction == Faction::Celts {
        let (with_camp, cities) = goals::wild_camp_cities(state, me);
        out.wild = Some(WildView {
            blocker: wild_blocker(state, me),
            sites: crate::rules::barbarians::rite_plan(state, me)
                .into_iter()
                .map(|(city, at)| WildSiteView { city, at })
                .collect(),
            with_camp,
            cities,
        });
    }
    if player.free_techs_pending > 0 {
        out.free_techs = Some(FreeTechsView {
            pending: player.free_techs_pending,
            from: player.free_techs_from,
            captured: player.free_techs_captured,
            choices: free_tech_choices(state, me),
        });
    }
    out
}

#[cfg(test)]
#[path = "actions_tests.rs"]
mod tests;

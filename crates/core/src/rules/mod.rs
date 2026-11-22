//! Simultaneous turn resolution (see plans/first-playable.md for the fixed phase order).
//!
//! `resolve_turn` runs the phases in order. Unit phases live in [`units`] (orders, ruins,
//! founding conflicts, elimination, healing), [`battle`] (combat, ranged phase, city capture)
//! and [`movement`] (simultaneous intent resolution); city, research and border phases live in
//! [`cities`].

use crate::data;
use crate::events::Event;
use crate::hex::Hex;
use crate::orders::{Order, OrderError};
use crate::scoring;
use crate::state::GameState;
use crate::types::{Faction, PlayerId, TeamId, UnitId, UnitKind};
use crate::visibility;
use std::collections::{BTreeMap, BTreeSet};

pub mod actions;
pub mod air;
pub mod barbarians;
pub mod battle;
pub mod cities;
pub mod gifts;
pub mod movement;
pub mod occupation;
#[cfg(test)]
mod occupation_tests;
pub mod promotion;
pub mod siege;
pub mod units;

/// Hostility is separate from permanent team identity.
pub fn hostile(state: &GameState, a: PlayerId, b: PlayerId) -> bool {
    state.at_war(a, b)
}

/// Whether `other` is a threat to `observer`: hostile, except that barbarians threaten nothing
/// of the Celts (`barbarians::spares`). Everything automatic or advisory on `observer`'s behalf
/// (a walled city's shot, a sleeper's wake-up, alarms and warnings) asks this, never `hostile`.
/// Orders that attack are `hostile`: the Celts may still fight barbarians when they choose to.
pub fn is_threat(state: &GameState, observer: PlayerId, other: PlayerId) -> bool {
    hostile(state, observer, other)
        && !(other.is_barbarian() && barbarians::spares(state, observer))
}

/// An armed unit on the map: aircraft are armed but based in hangars, never a garrison, a target
/// on a tile or a defender (`rules::air`).
pub fn is_military(kind: UnitKind) -> bool {
    data::unit_stats(kind).military && !data::is_aircraft(kind)
}

/// Check a player's orders without applying them or revealing unexplored terrain.
/// Movement validation uses the player's knowledge; resolution checks the actual map.
pub fn validate_orders(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError> {
    let mut errors = Vec::new();
    let known = (player.0 as usize) < state.players.len();
    if !known || !state.player(player).alive {
        let message = if known {
            "your empire has been eliminated"
        } else {
            "no such player"
        };
        return (0..orders.len())
            .map(|index| OrderError {
                index,
                message: message.into(),
            })
            .collect();
    }

    let mut acted = BTreeSet::new();
    let start_gold = state.player(player).gold;
    let mut vision = None;
    let mut view = None;
    let diplomacy_errors = crate::diplomacy::validate_orders(state, player, orders);
    let mut gold = start_gold;
    // An attack ordered with this turn's valid War declaration may target that team's land.
    let declared: BTreeSet<TeamId> =
        units::declared_wars(state, &BTreeMap::from([(player, orders)]))
            .into_iter()
            .filter_map(|(team, other)| (team == state.team_of(player)).then_some(other))
            .collect();
    // Resolution opens valid surprise wars before ranged fire. Validate ranged and air orders against
    // that same relationship snapshot, without changing the state used by other orders.
    let mut ranged_state = None;
    if let Some(diplomacy) = &state.diplomacy
        && !state.finished
    {
        let team = state.team_of(player);
        let surprise: BTreeSet<_> = declared
            .iter()
            .copied()
            .filter(|other| {
                diplomacy
                    .relation(team, *other)
                    .is_some_and(|r| crate::diplomacy::surprise_war(r, state.turn))
            })
            .collect();
        if !surprise.is_empty() {
            let mut projected = state.clone();
            if let Some(d) = &mut projected.diplomacy {
                for relation in &mut d.relations {
                    if (relation.a == team && surprise.contains(&relation.b))
                        || (relation.b == team && surprise.contains(&relation.a))
                    {
                        relation.state = crate::diplomacy::RelationState::War;
                    }
                }
            }
            ranged_state = Some(projected);
        }
    }
    for (index, order) in orders.iter().enumerate() {
        if units::order_unit(order).is_none() {
            continue;
        }
        let result = units::validate_unit_order(
            if matches!(order, Order::RangedAttack { .. }) {
                ranged_state.as_ref().unwrap_or(state)
            } else {
                state
            },
            player,
            order,
            &declared,
            &mut acted,
            &mut gold,
            &mut vision,
            &mut view,
        );
        if let Err(message) = result {
            errors.push(OrderError { index, message });
        }
    }
    // Upgrades also take strategic supply, in list order; production sees what is left.
    let mut upgraded: Vec<(UnitId, UnitKind)> = Vec::new();
    let mut taken: BTreeMap<crate::resources::Resource, i32> = BTreeMap::new();
    for (index, order) in orders.iter().enumerate() {
        let Order::UpgradeUnit { unit } = order else {
            continue;
        };
        if errors.iter().any(|e| e.index == index) {
            continue;
        }
        let Some(u) = state.units.get(unit) else {
            continue;
        };
        let Some((to, _, _)) = units::upgrade_plan(state, player, u.kind) else {
            continue;
        };
        if let Some(need) = crate::resources::unit_upgrade_need_to(u, to) {
            let earlier = taken.entry(need).or_default();
            if let Some(reason) = crate::resources::upgrade_blocker(state, player, u, to, *earlier)
            {
                errors.push(OrderError {
                    index,
                    message: format!(
                        "upgrading to {}: {}",
                        data::unit_name(to),
                        units::lowercase_first(&reason)
                    ),
                });
                continue;
            }
            *earlier += 1;
        }
        upgraded.push((*unit, to));
    }
    // Siege uses supply too: the purchases are counted after the upgrades, in list order.
    for (index, order) in orders.iter().enumerate() {
        let Order::UpgradeSiege { .. } = order else {
            continue;
        };
        if errors.iter().any(|e| e.index == index) {
            continue;
        }
        if let Some(need) = crate::resources::siege_need() {
            let earlier = taken.entry(need).or_default();
            if let Some(reason) = siege::iron_blocker(state, player, *earlier) {
                errors.push(OrderError {
                    index,
                    message: format!("Siege: {}", units::lowercase_first(&reason)),
                });
                continue;
            }
            *earlier += 1;
        }
    }
    // Upgrades are paid before city orders at resolution, so purchases see the gold left.
    let disbanded: Vec<_> = orders
        .iter()
        .enumerate()
        .filter_map(|(i, o)| match o {
            Order::DisbandUnit { unit } if !errors.iter().any(|e| e.index == i) => Some(*unit),
            _ => None,
        })
        .collect();
    let sieged: Vec<UnitId> = orders
        .iter()
        .enumerate()
        .filter_map(|(i, o)| match o {
            Order::UpgradeSiege { unit } if !errors.iter().any(|e| e.index == i) => Some(*unit),
            _ => None,
        })
        .collect();
    let supply_unchanged = taken.is_empty() && sieged.is_empty();
    // The city check also projects the state after the valid unit and city orders, which aircraft
    // upgrades (paid later, in the ranged phase) spend from.
    let (city_errors, after_cities) =
        if gold == start_gold && disbanded.is_empty() && supply_unchanged {
            cities::validate_city_orders_projected(state, player, orders)
        } else {
            let mut after = state.clone();
            after.player_mut(player).gold = gold;
            for (unit, to) in &upgraded {
                if let Some(u) = after.units.get_mut(unit) {
                    u.kind = *to;
                    u.siege &= !crate::data::built_in_siege(*to);
                }
            }
            for unit in sieged {
                if let Some(u) = after.units.get_mut(&unit) {
                    u.siege = true;
                }
            }
            for unit in disbanded {
                after.units.remove(&unit);
            }
            cities::validate_city_orders_projected(&after, player, orders)
        };
    errors.extend(city_errors);
    errors.extend(actions::validate_actions(state, player, orders));
    errors.extend(gifts::validate_gifts(state, player, orders));
    // An air strike resolves in the same ranged phase, so it sees the same surprise wars.
    errors.extend(air::validate_orders(
        ranged_state.as_ref().unwrap_or(state),
        player,
        orders,
        after_cities,
    ));
    errors.extend(diplomacy_errors);
    errors.sort_by_key(|e| e.index);
    errors
}

/// Orders from players that exist and are alive, in player-id order.
fn live_orders<'a>(
    state: &GameState,
    orders: &'a BTreeMap<PlayerId, Vec<Order>>,
) -> BTreeMap<PlayerId, &'a [Order]> {
    orders
        .iter()
        .filter(|(p, _)| (p.0 as usize) < state.players.len() && state.player(**p).alive)
        .map(|(p, os)| (*p, os.as_slice()))
        .collect()
}

/// Phases 1 (unit part) to 3: reset, unit orders, ranged attacks, movement and combat.
/// City orders are applied by the caller between unit orders and ranged attacks.
fn unit_phases_before_cities(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) -> units::UnitPlan {
    units::reset_turn(state);
    units::apply_unit_orders(state, orders, events)
}

/// Resolve one turn from every player's orders. Invalid orders are skipped (and reported as
/// events), never fatal. Returns the next state and the turn report.
pub fn resolve_turn(
    state: &GameState,
    orders: &BTreeMap<PlayerId, Vec<Order>>,
) -> (GameState, Vec<Event>) {
    if state.turn >= u32::MAX - crate::validation::COUNTER_HEADROOM
        || state.next_unit_id >= u32::MAX - crate::validation::COUNTER_HEADROOM
        || state.next_city_id >= u32::MAX - crate::validation::COUNTER_HEADROOM
    {
        return (state.clone(), Vec::new());
    }
    let mut s = state.clone();
    let mut events = Vec::new();
    if s.finished {
        return (s, events);
    }
    let capture_vp = crate::objectives::CaptureVpSnapshot::new(state);
    let turn_start: BTreeMap<UnitId, Hex> = state.units.values().map(|u| (u.id, u.pos)).collect();
    // A landmark already in starting sight pays through this first turn's visible report.
    crate::wonders::discover(&mut s, &mut events);
    let orders = live_orders(&s, orders);

    visibility::remember_threats(&mut s);
    let resource_sources = crate::resources::source_counts(&s);
    // 0. Surprise wars at plain Peace begin now, before ranged fire and movement (G20).
    let surprise = crate::diplomacy::declare_surprise_wars(&mut s, &orders, &mut events);
    // 1. Orders (barbarians choose their moves from the same start-of-turn state).
    let barbarian_shots = barbarians::plan_turn(&mut s, &mut events);
    let mut plan = unit_phases_before_cities(&mut s, &orders, &mut events);
    plan.ranged.extend(barbarian_shots);
    let mut sleeping_threats = units::sleeping_threats(&s);
    // A surprise declaration makes an already-visible army a new threat: its units are not in
    // the baseline, so they wake sleepers in the first movement round.
    for (id, contacts) in &mut sleeping_threats {
        let team = s.team_of(s.units[id].owner);
        contacts.retain(|enemy| {
            let other = s.team_of(s.units[enemy].owner);
            !surprise.contains(&crate::diplomacy::pair(team, other))
        });
    }
    // Same-turn bids on one tile are tried in a seeded order, not by empire id.
    match cities::defer_contested_tile_buys(&s, &orders) {
        None => {
            for (p, os) in &orders {
                cities::apply_city_orders(&mut s, *p, os, &mut events);
            }
        }
        Some((kept, deferred)) => {
            for (p, os) in &kept {
                cities::apply_city_orders(&mut s, *p, os, &mut events);
            }
            for (p, order) in deferred {
                cities::apply_city_orders(&mut s, p, std::slice::from_ref(&order), &mut events);
            }
        }
    }
    for (p, os) in &orders {
        actions::apply_actions(&mut s, *p, os, &mut events);
    }
    // Gifts between teammates move after every purchase, so a teammate spends them next turn.
    gifts::apply_gifts(&mut s, &orders, &mut events);
    // 2. Ranged attacks.
    let held = air::resolve_ranged_phase(
        &mut s,
        &orders,
        std::mem::take(&mut plan.ranged),
        &mut events,
    );
    // 3-4. Movement rounds with combat (held shots at fallen cities fire at whoever enters).
    movement::resolve_movement_with_paths(
        &mut s,
        &plan.automation_paths,
        &plan.awaiting_war,
        &mut sleeping_threats,
        Some(&turn_start),
        held,
        &mut events,
    );
    barbarians::clear_camps(&mut s, &mut events);
    // 5. Found cities.
    for settler in units::found_city_candidates(&s, &plan.found, &mut events) {
        let owner = s.units[&settler].owner;
        if let Err(reason) = cities::found_city(&mut s, settler, &mut events) {
            let founder = if s.player(owner).faction == Faction::Rome
                && s.units[&settler].kind != UnitKind::Settler
            {
                "Roman founder"
            } else {
                "Settler"
            };
            events.push(Event::OrderFailed {
                player: owner,
                reason: format!("{founder} {} could not found a city: {reason}", settler.0),
            });
        }
    }
    // Successful founding consumes the settler. A surviving, still-owned settler in this
    // plan received an OrderFailed, whether from site validation or a simultaneous clash.
    // Remember only the attempted location, not any information about the hidden cause.
    for (player, settler) in &plan.found {
        if let Some(unit) = s.units.get(settler).filter(|u| u.owner == *player) {
            let pos = unit.pos;
            s.player_mut(*player).failed_city_sites.insert(pos);
        }
    }
    units::check_elimination(&mut s, &mut events);
    units::heal_and_fortify(&mut s);
    units::wake_healed(&mut s, &mut events);
    units::finish_automation(&mut s, &plan);
    // 6-10. Cities (including city healing), gold and upkeep, research, borders.
    cities::process_cities(&mut s, &mut events);
    // A Company's contract ends after its last turn's upkeep is paid.
    crate::company::expire(&mut s, &mut events);
    cities::process_research(&mut s, &mut events);
    // Empires entering the same era this turn: the first is drawn, not the lowest id.
    crate::eras::draw_same_turn_firsts(&mut s, &mut events);
    cities::process_borders(&mut s, &mut events);
    crate::roads::extend(&mut s, &mut events);
    crate::roads::extend_roadworks(&mut s);
    crate::resources::announce(&resource_sources, &s, &mut events);
    crate::resources::update_supply(&mut s, &mut events);
    visibility::update_explored(&mut s);
    crate::wonders::discover(&mut s, &mut events);
    units::wake_threatened_sleepers(&mut s, &mut sleeping_threats, &mut events);
    // 11. Scoring, then the next turn.
    scoring::update_with_capture_snapshot(&mut s, &mut events, &capture_vp);
    crate::diplomacy::resolve_boundary(state, &mut s, &orders, &mut events);
    // An attack reveal lasts one planning turn.
    visibility::expire_attack_reveals(&mut s);
    // Newly allied sight must enter observation memory before its sources can disappear.
    // update_explored records borrowed tiles without awarding native exploration credit.
    visibility::update_explored(&mut s);
    crate::wonders::discover(&mut s, &mut events);
    // A declaration makes an already-visible peaceful army a new threat. Reuse the
    // movement baseline so re-slept units still accept enemies they already knew.
    units::wake_threatened_sleepers(&mut s, &mut sleeping_threats, &mut events);
    barbarians::end_of_turn(state, &mut s, &mut events);
    // Peace and expanding borders can strand units; no displacement, but old fortification
    // cannot remain active in territory where fortifying is forbidden.
    let closed: Vec<_> = s
        .units
        .values()
        .filter(|u| !s.unit_in_open_land(u))
        .map(|u| u.id)
        .collect();
    for id in closed {
        s.units.get_mut(&id).expect("unit").fortify = 0;
    }
    occupation::clear_released(&mut s);
    occupation::hold_locked_units(&mut s);
    remember_air_contacts(state, &mut s, &events);
    crate::history::record(state, &mut s, &events);
    s.turn = s.turn.checked_add(1).expect("turn has headroom");
    (s, events)
}

/// Resolve a tactical arena turn using the game's unit phases, without economy, research,
/// borders, diplomacy updates, or victory scoring. The caller supplies an already hostile
/// state and owns its tactical end condition. This is useful for short combat studies; every
/// attack and move still runs through the ordinary order and battle resolvers.
pub fn resolve_tactical_turn(
    state: &GameState,
    orders: &BTreeMap<crate::PlayerId, Vec<Order>>,
) -> (GameState, Vec<Event>) {
    if state.turn >= u32::MAX - crate::validation::COUNTER_HEADROOM {
        return (state.clone(), Vec::new());
    }
    let mut s = state.clone();
    let mut events = Vec::new();
    let turn_start: BTreeMap<UnitId, Hex> = state.units.values().map(|u| (u.id, u.pos)).collect();
    let orders = live_orders(&s, orders);
    let plan = unit_phases_before_cities(&mut s, &orders, &mut events);
    let mut sleeping_threats = units::sleeping_threats(&s);
    let held = air::resolve_ranged_phase(&mut s, &orders, plan.ranged.clone(), &mut events);
    movement::resolve_movement_with_paths(
        &mut s,
        &plan.automation_paths,
        &plan.awaiting_war,
        &mut sleeping_threats,
        Some(&turn_start),
        held,
        &mut events,
    );
    let _ = units::found_city_candidates(&s, &plan.found, &mut events);
    units::heal_and_fortify(&mut s);
    units::wake_healed(&mut s, &mut events);
    units::finish_automation(&mut s, &plan);
    remember_air_contacts(state, &mut s, &events);
    s.turn = s.turn.checked_add(1).expect("turn has headroom");
    (s, events)
}

/// Keep only aircraft facts in a battle report visible to a participating or sighted team.
/// Rival hangars, technology and battles outside sight remain private.
fn remember_air_contacts(before: &GameState, after: &mut GameState, events: &[Event]) {
    let mut sights = BTreeMap::new();
    for event in events {
        let Event::Battle {
            attacker_owner,
            defender,
            at,
            air_strike,
            ..
        } = event
        else {
            continue;
        };
        let defender_owner = match defender {
            crate::events::Defender::Unit { owner, .. }
            | crate::events::Defender::City { owner, .. } => *owner,
        };
        let defending_air = match defender {
            crate::events::Defender::Unit {
                owner, unit_kind, ..
            } if data::is_aircraft(*unit_kind) => Some((*owner, *unit_kind)),
            _ => None,
        };
        if !air_strike && defending_air.is_none() {
            continue;
        }
        // Battle reports reach the participants and any team that saw the battle tile before
        // or after resolution. Use that same visibility boundary for remembered air facts.
        let observers: Vec<_> = after
            .players
            .iter()
            .filter(|p| {
                p.team == before.team_of(*attacker_owner)
                    || p.team == before.team_of(defender_owner)
                    || before.map.index(*at).is_some_and(|i| {
                        let (start, end) = sights.entry(p.team).or_insert_with(|| {
                            (
                                visibility::visible_tiles(before, p.team),
                                visibility::visible_tiles(after, p.team),
                            )
                        });
                        start.get(i) || end.get(i)
                    })
            })
            .map(|p| p.id)
            .collect();
        for observer in observers {
            if *air_strike {
                // The public Battle report identifies an air strike, but not a rival striker's
                // kind when its hangar is hidden. Do not turn a private roster read into intel.
                remember_air_contact(after, observer, *attacker_owner, None, before.turn, None);
            }
            if let Some((owner, kind)) = defending_air {
                remember_air_contact(after, observer, owner, Some(kind), before.turn, None);
            }
        }
    }
}

pub(crate) fn remember_air_contact(
    state: &mut GameState,
    observer: PlayerId,
    owner: PlayerId,
    kind: Option<UnitKind>,
    turn: u32,
    city: Option<crate::CityId>,
) {
    if state.same_team(observer, owner) {
        return;
    }
    let team = state.team_of(observer);
    let recipients: Vec<_> = state
        .players
        .iter()
        .filter(|p| p.team == team)
        .map(|p| p.id)
        .collect();
    for id in recipients {
        let contacts = state.air_contacts.entry(id).or_default();
        if let Some(contact) = contacts
            .iter_mut()
            .find(|c| c.owner == owner && c.kind == kind)
        {
            if city.is_some() || contact.turn < turn {
                contact.city = city;
            }
            contact.turn = turn;
        } else {
            contacts.push(crate::state::AirContact {
                owner,
                kind,
                turn,
                city,
            });
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::hex::Hex;
    use crate::state::{City, GameSetup, Map, PlayerSetup};
    use crate::types::*;

    /// A small all-grassland game with `teams.len()` players on the given teams.
    pub fn game(teams: &[u8]) -> GameState {
        let players = teams
            .iter()
            .enumerate()
            .map(|(i, t)| PlayerSetup {
                name: format!("P{i}"),
                faction: Faction::Switzerland,
                team: TeamId(*t),
                is_human: false,
                difficulty: Default::default(),
                search: false,
            })
            .collect();
        let setup = GameSetup {
            seed: 7,
            width: 12,
            height: 10,
            players,
            turn_limit: Some(150),
            legacy_all_war: false,
            barbarians: false,
            rule_experiments: Default::default(),
        };
        {
            let mut state = crate::game::blank_game(&setup, Map::new(12, 10, Terrain::Grassland));
            // These pre-diplomacy tactical fixtures assume every foreign team is at War.
            for relation in &mut state.diplomacy.as_mut().unwrap().relations {
                relation.state = crate::diplomacy::RelationState::War;
            }
            state
        }
    }

    pub fn hex(col: i32, row: i32) -> Hex {
        Hex::from_offset(col, row)
    }

    /// Gives `player` every technology of the eras before `era`, so `era` is their current era
    /// and its techs pass the research gate (`eras::researchable`).
    pub fn know_eras_before(s: &mut GameState, player: PlayerId, era: u8) {
        s.player_mut(player).techs.extend(
            TechId::ALL
                .into_iter()
                .filter(|t| crate::eras::era_of_tech(*t) < era),
        );
    }

    pub fn set_terrain(s: &mut GameState, h: Hex, t: Terrain) {
        s.map.get_mut(h).unwrap().terrain = t;
    }

    pub fn unit(s: &mut GameState, owner: u8, kind: UnitKind, at: Hex) -> UnitId {
        crate::game::spawn_unit(s, PlayerId(owner), kind, at)
    }

    pub fn goto(s: &mut GameState, u: UnitId, to: Hex) {
        s.units.get_mut(&u).unwrap().goto = Some(to);
    }

    /// Insert a city directly, owning its radius-1 tiles.
    pub fn city(s: &mut GameState, owner: u8, at: Hex, pop: i32) -> CityId {
        let id = s.alloc_city_id();
        s.cities.insert(
            id,
            City {
                last_attacked_turn: None,
                id,
                owner: PlayerId(owner),
                founder: PlayerId(owner),
                name: format!("City {}", id.0),
                pos: at,
                population: pop,
                food_stored: 5,
                production_stored: 7,
                producing: None,
                bought: None,
                production_queue: Vec::new(),
                build_automation: None,
                last_auto_pick: None,
                auto_queued: None,
                production_banked: Vec::new(),
                production_queue_notified: Default::default(),
                wonder_lost: None,
                last_purchase_turn: None,
                buildings: Default::default(),
                hp: data::CITY_HP,
                grown_tiles: Vec::new(),
                tiles_claimed: 0,
                is_capital: true,
                original_capital: true,
                greece_captive_paid: Default::default(),
                occupied_from: None,
                founded_turn: 1,
                resistance: 0,
            },
        );
        for h in at.within(1) {
            if let Some(t) = s.map.get_mut(h) {
                t.owner = Some(PlayerId(owner));
                t.city = Some(id);
            }
        }
        id
    }

    /// A city with Walls: only walled cities fire (city strike).
    pub fn walled_city(s: &mut GameState, owner: u8, at: Hex, pop: i32) -> CityId {
        let id = city(s, owner, at, pop);
        s.cities
            .get_mut(&id)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        id
    }

    /// Every unit phase of `resolve_turn`, skipping the city/research/border/scoring calls
    /// (owned by rules-cities) and founding (only conflict filtering runs). Elimination is
    /// skipped too, so test empires need no cities or settlers; it is tested directly.
    pub fn resolve_units(
        state: &GameState,
        orders: &BTreeMap<PlayerId, Vec<Order>>,
    ) -> (GameState, Vec<Event>) {
        let mut s = state.clone();
        let mut events = Vec::new();
        let turn_start: BTreeMap<UnitId, Hex> =
            state.units.values().map(|u| (u.id, u.pos)).collect();
        let orders = live_orders(&s, orders);
        let plan = unit_phases_before_cities(&mut s, &orders, &mut events);
        let mut sleeping_threats = units::sleeping_threats(&s);
        let held = air::resolve_ranged_phase(&mut s, &orders, plan.ranged.clone(), &mut events);
        movement::resolve_movement_with_paths(
            &mut s,
            &plan.automation_paths,
            &plan.awaiting_war,
            &mut sleeping_threats,
            Some(&turn_start),
            held,
            &mut events,
        );
        let _ = units::found_city_candidates(&s, &plan.found, &mut events);
        units::heal_and_fortify(&mut s);
        units::wake_healed(&mut s, &mut events);
        units::finish_automation(&mut s, &plan);
        s.turn += 1;
        (s, events)
    }

    pub fn no_orders() -> BTreeMap<PlayerId, Vec<Order>> {
        BTreeMap::new()
    }

    pub fn orders(p: u8, os: Vec<Order>) -> BTreeMap<PlayerId, Vec<Order>> {
        let mut m = BTreeMap::new();
        m.insert(PlayerId(p), os);
        m
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::types::*;

    #[test]
    fn tactical_turn_matches_full_turn_unit_phases() {
        let mut s = game(&[0, 1]);
        let attacker = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
        let _defender = unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
        // Keep both empires alive in the full resolver, which also processes elimination.
        unit(&mut s, 0, UnitKind::Settler, hex(1, 1));
        unit(&mut s, 1, UnitKind::Settler, hex(10, 8));
        let os = orders(
            0,
            vec![Order::MoveUnit {
                unit: attacker,
                to: hex(5, 4),
            }],
        );
        let (tactical, tactical_events) = resolve_tactical_turn(&s, &os);
        let (full, full_events) = resolve_turn(&s, &os);
        assert_eq!(tactical.units, full.units);
        let battles = |events: Vec<Event>| {
            events
                .into_iter()
                .filter(|e| matches!(e, Event::Battle { .. }))
                .collect::<Vec<_>>()
        };
        assert_eq!(battles(tactical_events), battles(full_events));
    }

    #[test]
    fn tactical_turn_matches_full_turn_city_assault() {
        let mut s = game(&[0, 1]);
        let city = walled_city(&mut s, 1, hex(5, 5), 3);
        unit(&mut s, 1, UnitKind::Soldier, hex(5, 5));
        let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(4, 5));
        let siege = unit(&mut s, 0, UnitKind::Ranger, hex(3, 5));
        unit(&mut s, 0, UnitKind::Settler, hex(1, 1));
        let os = orders(
            0,
            vec![
                Order::RangedAttack {
                    unit: siege,
                    target: hex(5, 5),
                },
                Order::MoveUnit {
                    unit: soldier,
                    to: hex(5, 5),
                },
            ],
        );
        let (tactical, tactical_events) = resolve_tactical_turn(&s, &os);
        let (full, full_events) = resolve_turn(&s, &os);
        assert_eq!(tactical.units, full.units);
        assert_eq!(tactical.cities[&city].hp, full.cities[&city].hp);
        let battles = |events: Vec<Event>| {
            events
                .into_iter()
                .filter(|e| matches!(e, Event::Battle { .. }))
                .collect::<Vec<_>>()
        };
        assert_eq!(battles(tactical_events), battles(full_events));
    }

    fn errors(s: &GameState, p: u8, os: &[Order]) -> Vec<(usize, String)> {
        validate_orders(s, PlayerId(p), os)
            .into_iter()
            .map(|e| (e.index, e.message))
            .collect()
    }

    #[test]
    fn validate_accepts_good_unit_orders() {
        let mut s = game(&[0, 1]);
        let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(2, 2));
        let settler = unit(&mut s, 0, UnitKind::Settler, hex(3, 3));
        let scout = unit(&mut s, 0, UnitKind::Scout, hex(6, 6));
        let os = vec![
            Order::MoveUnit {
                unit: soldier,
                to: hex(8, 2),
            },
            Order::FoundCity { unit: settler },
            Order::SkipUnit { unit: scout },
        ];
        assert!(errors(&s, 0, &os).is_empty());
    }

    #[test]
    fn move_validation_does_not_reveal_unknown_terrain_or_connectivity() {
        let mut s = game(&[0, 1]);
        let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(1, 4));
        let target = hex(9, 4);
        let order = Order::MoveUnit {
            unit: soldier,
            to: target,
        };
        let original_view = crate::player_view(&s, PlayerId(0));
        assert_eq!(original_view.tile(target).unwrap().terrain, None);
        for terrain in [Terrain::Grassland, Terrain::Ocean, Terrain::Mountains] {
            set_terrain(&mut s, target, terrain);
            assert_eq!(crate::player_view(&s, PlayerId(0)), original_view);
            assert!(errors(&s, 0, std::slice::from_ref(&order)).is_empty());
            if terrain != Terrain::Grassland {
                let (_, events) = resolve_turn(&s, &orders(0, vec![order.clone()]));
                let reasons: Vec<_> = events
                    .iter()
                    .filter_map(|event| match event {
                        Event::OrderFailed {
                            player: PlayerId(0),
                            reason,
                        } => Some(reason.as_str()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(reasons, [format!("Move of unit {} failed", soldier.0)]);
            }
        }

        set_terrain(&mut s, target, Terrain::Grassland);
        // Even a known destination must not disclose a wall in unexplored territory.
        let index = s.map.index(target).unwrap();
        s.player_mut(PlayerId(0)).explored.set(index);
        let original_view = crate::player_view(&s, PlayerId(0));
        for row in 0..s.map.height {
            set_terrain(&mut s, hex(6, row), Terrain::Mountains);
        }
        assert_eq!(crate::player_view(&s, PlayerId(0)), original_view);
        assert!(errors(&s, 0, std::slice::from_ref(&order)).is_empty());
        let (_, events) = resolve_turn(&s, &orders(0, vec![order.clone()]));
        assert!(events.contains(&Event::OrderFailed {
            player: PlayerId(0),
            reason: format!("Move of unit {} failed", soldier.0),
        }));

        // Once the wall is known, its connectivity is legitimate validation feedback.
        for row in 0..s.map.height {
            let index = s.map.index(hex(6, row)).unwrap();
            s.player_mut(PlayerId(0)).explored.set(index);
        }
        assert!(errors(&s, 0, &[order])[0].1.contains("reach"));
    }

    #[test]
    fn validate_rejects_bad_unit_orders() {
        let mut s = game(&[0, 1]);
        let soldier = unit(&mut s, 0, UnitKind::Soldier, hex(2, 2));
        let enemy = unit(&mut s, 1, UnitKind::Soldier, hex(9, 8));
        let ranger = unit(&mut s, 0, UnitKind::Ranger, hex(5, 5));
        set_terrain(&mut s, hex(10, 2), Terrain::Mountains);
        // Wall off the corner (11, 0) with mountains.
        for h in hex(11, 0).neighbors() {
            if s.map.contains(h) {
                set_terrain(&mut s, h, Terrain::Mountains);
            }
        }
        let os = vec![
            Order::MoveUnit {
                unit: enemy,
                to: hex(8, 8),
            }, // 0: not ours
            Order::MoveUnit {
                unit: UnitId(999),
                to: hex(1, 1),
            }, // 1: missing
            Order::MoveUnit {
                unit: soldier,
                to: hex(40, 40),
            }, // 2: off map
            Order::SkipUnit { unit: soldier }, // 3: second action for soldier
            Order::FoundCity { unit: ranger }, // 4: not a settler
        ];
        let e = errors(&s, 0, &os);
        assert_eq!(
            e.iter().map(|x| x.0).collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
        assert!(e[3].1.contains("one"), "{}", e[3].1);

        let s2 = s.clone();
        let mut s3 = s.clone();
        let other = unit(&mut s3, 0, UnitKind::Soldier, hex(3, 3));
        // Terrain/connectivity errors are available only for known tiles.
        for index in 0..s3.map.len() {
            s3.player_mut(PlayerId(0)).explored.set(index);
        }
        let e = errors(
            &s3,
            0,
            &[
                Order::MoveUnit {
                    unit: soldier,
                    to: hex(10, 2),
                }, // impassable
                Order::MoveUnit {
                    unit: other,
                    to: hex(11, 0),
                }, // unreachable
            ],
        );
        assert_eq!(e.len(), 2);
        assert!(e[0].1.contains("impassable"), "{}", e[0].1);
        assert!(e[1].1.contains("reach"), "{}", e[1].1);

        // Ranged: non-ranged unit, out of range, no enemy there.
        let e = errors(
            &s2,
            0,
            &[
                Order::RangedAttack {
                    unit: soldier,
                    target: hex(3, 2),
                },
                Order::RangedAttack {
                    unit: ranger,
                    target: hex(9, 8),
                },
            ],
        );
        assert_eq!(e.len(), 2);
        let mut s4 = s2.clone();
        let _friend = unit(&mut s4, 0, UnitKind::Soldier, hex(6, 5));
        let e = errors(
            &s4,
            0,
            &[Order::RangedAttack {
                unit: ranger,
                target: hex(6, 5),
            }],
        );
        assert_eq!(e.len(), 1, "cannot shoot a friendly tile");
        let _foe = unit(&mut s4, 1, UnitKind::Soldier, hex(7, 5));
        assert!(
            errors(
                &s4,
                0,
                &[Order::RangedAttack {
                    unit: ranger,
                    target: hex(7, 5)
                }]
            )
            .is_empty()
        );
    }

    #[test]
    fn validate_upgrade_rules() {
        let mut s = game(&[0, 1]);
        let soldier = unit(&mut s, 0, UnitKind::Phalangite, hex(2, 2));
        let scout = unit(&mut s, 0, UnitKind::Scout, hex(2, 3));
        let up = |u| vec![Order::UpgradeUnit { unit: u }];
        // No tech, no territory, no gold.
        let e = errors(&s, 0, &up(soldier));
        assert!(e[0].1.contains("Feudalism"), "{}", e[0].1);
        s.player_mut(PlayerId(0)).techs.insert(TechId::Feudalism);
        // Phalangite→Pikeman also uses 1 Iron supply; this test is about Gold.
        s.player_mut(PlayerId(0))
            .techs
            .extend([TechId::BronzeWorking, TechId::IronWorking]);
        let e = errors(&s, 0, &up(soldier));
        assert!(e[0].1.contains("territory"), "{}", e[0].1);
        city(&mut s, 0, hex(2, 2), 1);
        s.map.get_mut(hex(3, 2)).unwrap().resource = Some(crate::Resource::Iron);
        let e = errors(&s, 0, &up(soldier));
        assert!(e[0].1.contains("Gold"), "{}", e[0].1);
        s.player_mut(PlayerId(0)).gold = 84;
        assert!(errors(&s, 0, &up(soldier)).is_empty());
        assert_eq!(errors(&s, 0, &up(scout)).len(), 1, "scouts cannot upgrade");
        // Running gold: two upgrades need 168.
        let second = unit(&mut s, 0, UnitKind::Phalangite, hex(3, 2));
        let e = errors(
            &s,
            0,
            &[
                Order::UpgradeUnit { unit: soldier },
                Order::UpgradeUnit { unit: second },
            ],
        );
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].0, 1);
    }

    #[test]
    fn validate_eliminated_player_rejects_everything() {
        let mut s = game(&[0, 1]);
        let u = unit(&mut s, 0, UnitKind::Scout, hex(1, 1));
        s.player_mut(PlayerId(0)).alive = false;
        assert_eq!(errors(&s, 0, &[Order::SkipUnit { unit: u }]).len(), 1);
        assert_eq!(errors(&s, 9, &[Order::SkipUnit { unit: u }]).len(), 1);
    }

    #[test]
    fn finished_game_is_unchanged() {
        let mut s = game(&[0, 1]);
        s.finished = true;
        let (next, events) = resolve_turn(&s, &no_orders());
        assert_eq!(next, s);
        assert!(events.is_empty());
    }

    /// A busy skirmish: movement, meeting engagements, ranged fire, captures, ruins.
    fn skirmish() -> (GameState, BTreeMap<PlayerId, Vec<Order>>) {
        let mut s = game(&[0, 1, 2]);
        set_terrain(&mut s, hex(5, 5), Terrain::Hills);
        set_terrain(&mut s, hex(6, 4), Terrain::Forest);
        s.map.get_mut(hex(7, 7)).unwrap().ruin = true;
        city(&mut s, 2, hex(9, 2), 3);
        let a = unit(&mut s, 0, UnitKind::Soldier, hex(2, 5));
        let b = unit(&mut s, 1, UnitKind::Soldier, hex(8, 5));
        let c = unit(&mut s, 2, UnitKind::Soldier, hex(5, 8));
        let r = unit(&mut s, 0, UnitKind::Ranger, hex(3, 4));
        let st = unit(&mut s, 1, UnitKind::Settler, hex(7, 6));
        let sc = unit(&mut s, 2, UnitKind::Scout, hex(9, 7));
        let _g = unit(&mut s, 2, UnitKind::Soldier, hex(9, 2));
        for (u, to) in [
            (b, hex(2, 5)),
            (c, hex(5, 2)),
            (st, hex(1, 9)),
            (sc, hex(7, 7)),
        ] {
            goto(&mut s, u, to);
        }
        let mut orders = BTreeMap::new();
        orders.insert(
            PlayerId(0),
            vec![
                Order::MoveUnit {
                    unit: a,
                    to: hex(9, 2),
                },
                Order::SkipUnit { unit: r },
            ],
        );
        (s, orders)
    }

    #[test]
    fn unit_phases_are_deterministic() {
        let (s, orders) = skirmish();
        let run = |mut st: GameState| {
            let mut hashes = Vec::new();
            let mut all = Vec::new();
            for turn in 0..6 {
                let os = if turn == 0 {
                    orders.clone()
                } else {
                    no_orders()
                };
                let (next, ev) = resolve_units(&st, &os);
                hashes.push(next.hash());
                all.extend(ev);
                st = next;
            }
            (hashes, all)
        };
        let (h1, e1) = run(s.clone());
        let (h2, e2) = run(s);
        assert_eq!(h1, h2);
        assert_eq!(e1, e2);
        assert!(
            e1.iter().any(|e| matches!(e, Event::Battle { .. })),
            "the skirmish should involve combat"
        );
    }

    /// Full turn through rules-cities and scoring.
    #[test]
    fn full_turn_resolves_and_is_deterministic() {
        let (s, orders) = skirmish();
        let (a, ea) = resolve_turn(&s, &orders);
        let (b, eb) = resolve_turn(&s, &orders);
        assert_eq!(a.hash(), b.hash());
        assert_eq!(ea, eb);
        assert_eq!(a.turn, s.turn + 1);
    }

    /// Simple scripted orders for a real game: found on turn 1, then wander toward a
    /// seed-derived target (explores, meets enemies, fights).
    fn scripted_orders(s: &GameState) -> BTreeMap<PlayerId, Vec<Order>> {
        let mut out: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for u in s.units.values() {
            let os = out.entry(u.owner).or_default();
            if u.kind == UnitKind::Settler {
                os.push(Order::FoundCity { unit: u.id });
            } else if u.goto.is_none() {
                let mut rng =
                    crate::rng::Rng::derive(s.setup.seed, &[s.turn as u64, 99, u.id.0 as u64]);
                let target = crate::hex::Hex::from_offset(
                    rng.range(0, (s.map.width - 1) as i64) as i32,
                    rng.range(0, (s.map.height - 1) as i64) as i32,
                );
                if validate_orders(
                    s,
                    u.owner,
                    &[Order::MoveUnit {
                        unit: u.id,
                        to: target,
                    }],
                )
                .is_empty()
                {
                    os.push(Order::MoveUnit {
                        unit: u.id,
                        to: target,
                    });
                }
            }
        }
        for c in s.cities.values() {
            let os = out.entry(c.owner).or_default();
            if c.producing.is_none() {
                os.push(Order::SetProduction {
                    city: c.id,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Soldier,
                    },
                });
            }
        }
        for (p, os) in out.iter_mut() {
            let bad: BTreeSet<usize> = validate_orders(s, *p, os)
                .into_iter()
                .map(|e| e.index)
                .collect();
            let kept: Vec<Order> = os
                .iter()
                .enumerate()
                .filter(|(i, _)| !bad.contains(i))
                .map(|(_, o)| o.clone())
                .collect();
            *os = kept;
        }
        out
    }

    #[test]
    fn full_games_run_deterministically() {
        let setup = crate::state::GameSetup::all_ai(11);
        let play = || {
            let mut s = crate::game::new_game(&setup);
            for r in &mut s.diplomacy.as_mut().unwrap().relations {
                r.state = crate::diplomacy::RelationState::War;
            }
            let mut hashes = Vec::new();
            let mut battles = 0;
            for _ in 0..40 {
                let os = scripted_orders(&s);
                let (next, events) = resolve_turn(&s, &os);
                battles += events
                    .iter()
                    .filter(|e| matches!(e, Event::Battle { .. }))
                    .count();
                // Invariant: at most one military and one civilian unit per tile, one team.
                let mut seen: BTreeMap<(crate::hex::Hex, bool), PlayerId> = BTreeMap::new();
                for u in next.units.values() {
                    let prev = seen.insert((u.pos, is_military(u.kind)), u.owner);
                    assert!(prev.is_none(), "two units of a class on {:?}", u.pos);
                }
                for u in next.units.values() {
                    for v in next.units_at(u.pos) {
                        assert!(
                            !hostile(&next, u.owner, v.owner),
                            "enemies share {:?}",
                            u.pos
                        );
                    }
                    assert!(next.map.move_cost(u.pos).is_some());
                }
                hashes.push(next.hash());
                s = next;
            }
            (hashes, battles, s)
        };
        let (h1, b1, end) = play();
        let (h2, _, _) = play();
        assert_eq!(h1, h2);
        assert!(!end.cities.is_empty());
        assert!(b1 > 0, "40 turns of wandering should produce some combat");
    }
}

//! The Modern and Future air layer (era overhaul phase A2, `UnitClass::Air`).
//!
//! Aircraft live in a city's hangar (`GameState::aircraft`, two slots or three after Stealth
//! Technology), never on a map tile: they are not garrison, cannot be attacked on a tile, never
//! capture, and are lost with their city. Four orders, one per aircraft per turn:
//! - `Order::AirStrike`: strike a visible enemy unit or city within `data::BOMBER_RANGE` (Bomber)
//!   or `data::FIGHTER_STRIKE_RANGE` (Fighter, its own strength, no city bonus) of its city. It
//!   resolves in the ranged phase with no retaliation, like any shot.
//! - `Order::Rebase`: move to any other own city's hangar with room, at any distance, at the end of
//!   the ranged phase (the aircraft cannot strike that turn).
//! - `Order::AirRecon`: scout any tile within the aircraft's strike range of its city: the tiles
//!   within `data::RECON_RADIUS` of it are in the team's sight for the next turn only, so a strike
//!   can be ordered then. Recorded in `settle_recon`, the last step of the ranged phase; the
//!   aircraft has used its order (no strike that turn) but a Fighter still guards.
//! - `Order::UpgradeAircraft`: upgrade a based aircraft to its newest researched form (a Biplane
//!   to a Fighter or a Bomber, the player's choice; a Fighter or Bomber to its Stealth form).
//!
//! A Fighter on guard intercepts: a strike on a tile within `data::FIGHTER_RANGE` of an enemy
//! Fighter's city does not land. Strikes are taken in aircraft-id order and each Fighter intercepts
//! one per turn, lowest id first; the striker and the Fighter then trade damage like a melee
//! exchange (the striker's strength against the Fighter's). A Fighter with a valid strike order has
//! used its turn and cannot intercept (one order per aircraft). Deterministic: ids and the usual
//! derived rolls.
//!
//! An Air Defence (`BuildingKind::AirDefence`, the Modern era) intercepts a strike on its own
//! city's tile the same way, one strike per turn, with no hangar slot: a Jet Fighter's strength
//! (`AIR_DEFENCE_INTERCEPTOR`) takes the Fighter's place in the duel, and it takes no damage
//! (a building is not shot down). A strike on that tile meets the Air Defence before any Fighter.
//! Stealth Bombers pass it as they pass every Fighter but a Stealth Fighter.
//!
//! Resolution reuses the unit machinery: an aircraft that acts this turn becomes a temporary map
//! unit at its city for the ranged phase (same id, `rules::is_military` is false for aircraft so it
//! is never a garrison or target) and is written back to the hangar afterwards.

use super::{battle, hostile};
use crate::data;
use crate::events::Event;
use crate::game::unit_with_id;
use crate::hex::Hex;
use crate::orders::Order;
use crate::state::{AirRecon, Aircraft, GameState};
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// The interceptor an Air Defence fields in a duel: a Jet Fighter's stats at full HP, standing in
/// its city for the ranged phase. It is neither a hangar aircraft nor harmed by the duel.
pub const AIR_DEFENCE_INTERCEPTOR: UnitKind = UnitKind::JetFighter;

/// Roll stream of the aircraft duel stage (the ranged stage is 0, movement rounds count up).
pub const STAGE_AIR: u64 = 900;

/// The aircraft in `city`'s hangar.
pub fn hangar(state: &GameState, city: CityId) -> impl Iterator<Item = &Aircraft> {
    state.aircraft.iter().filter(move |a| a.city == city)
}

pub fn find(state: &GameState, id: UnitId) -> Option<&Aircraft> {
    state.aircraft.iter().find(|a| a.id == id)
}

/// The owner of an aircraft: the owner of its city.
pub fn owner_of(state: &GameState, plane: &Aircraft) -> Option<PlayerId> {
    state.cities.get(&plane.city).map(|c| c.owner)
}

/// Why `city` cannot base another aircraft now (its hangar is full).
pub fn hangar_blocker(state: &GameState, city: CityId) -> Option<String> {
    let cap = state.cities.get(&city).map_or(data::AIR_BASE_CAP, |c| {
        data::air_base_cap(&state.player(c.owner).techs)
    });
    (hangar(state, city).count() >= cap)
        .then(|| format!("The hangar is full: a city bases {} aircraft", cap))
}

/// A new aircraft completed in `city`'s hangar.
pub fn base_new(state: &mut GameState, city: CityId, kind: UnitKind) {
    let id = state.alloc_unit_id();
    let owner = state.cities[&city].owner;
    let xp = super::promotion::starting_xp(state, owner, kind);
    state.aircraft.push(Aircraft {
        id,
        kind,
        city,
        hp: data::UNIT_HP,
        xp,
    });
}

/// The aircraft of a city that falls or is razed are lost with it.
pub fn lose_city_aircraft(state: &mut GameState, city: CityId, events: &mut Vec<Event>) {
    let Some(c) = state.cities.get(&city) else {
        return;
    };
    let (owner, at) = (c.owner, c.pos);
    let lost: Vec<Aircraft> = state
        .aircraft
        .iter()
        .filter(|a| a.city == city)
        .cloned()
        .collect();
    state.aircraft.retain(|a| a.city != city);
    for a in lost {
        events.push(Event::UnitDestroyed {
            unit: a.id,
            owner,
            unit_kind: a.kind,
            at,
        });
    }
}

/// Check `player`'s air orders. The caller projects valid same-turn surprise wars into `state`
/// before checking strikes, as it does for ranged attacks.
///
/// Upgrades are checked against `economy`, the state after this list's valid unit and city
/// orders (`validate_orders` in `rules`): resolution pays for them in the ranged phase, after
/// unit upgrades and city purchases, in aircraft-id order. Each valid upgrade is applied to
/// `economy` in that same order, so the Gold and supply a second upgrade sees are what is left
/// after the first, and an order resolution would refuse is refused here.
pub fn validate_orders(
    state: &GameState,
    player: PlayerId,
    orders: &[Order],
    mut economy: GameState,
) -> Vec<crate::orders::OrderError> {
    let mut errors = Vec::new();
    let mut acted = BTreeSet::new();
    let mut rebased_into: BTreeMap<CityId, usize> = BTreeMap::new();
    let mut vision = None;
    let mut upgrades: Vec<(UnitId, Option<UnitKind>, usize)> = Vec::new();
    for (index, order) in orders.iter().enumerate() {
        let plane = match order {
            Order::AirStrike { plane, .. }
            | Order::Rebase { plane, .. }
            | Order::AirRecon { plane, .. }
            | Order::UpgradeAircraft { plane, .. } => *plane,
            _ => continue,
        };
        if acted.contains(&plane) {
            errors.push(crate::orders::OrderError {
                index,
                message: format!(
                    "Your {} already has an order this turn (one per aircraft)",
                    find(state, plane).map_or("aircraft", |a| crate::data::unit_name_for(
                        state.player(player).faction,
                        a.kind
                    ))
                ),
            });
            continue;
        }
        let result = match order {
            Order::AirStrike { target, .. } => {
                validate_strike(state, player, plane, *target, &mut vision)
            }
            Order::Rebase { to, .. } => {
                validate_rebase(state, player, plane, *to, &mut rebased_into)
            }
            Order::AirRecon { target, .. } => validate_recon(state, player, plane, *target),
            Order::UpgradeAircraft { to, .. } => {
                // Resolution takes the first order for each aircraft, so this one holds it.
                acted.insert(plane);
                upgrades.push((plane, *to, index));
                continue;
            }
            _ => unreachable!(),
        };
        if let Err(message) = result {
            errors.push(crate::orders::OrderError { index, message });
        } else {
            acted.insert(plane);
        }
    }
    if !upgrades.is_empty() {
        upgrades.sort();
        for (plane, via, index) in upgrades {
            match validate_upgrade(&economy, player, plane, via) {
                Ok((to, cost)) => {
                    economy.player_mut(player).gold -= cost;
                    if let Some(a) = find_mut(&mut economy, plane) {
                        a.kind = to;
                    }
                }
                Err(message) => errors.push(crate::orders::OrderError { index, message }),
            }
        }
        errors.sort_by_key(|e| e.index);
    }
    errors
}

/// The form this aircraft upgrades to along the line `via` (None: the default line) and its
/// Gold price.
pub fn upgrade_price(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
    via: Option<UnitKind>,
) -> Result<(UnitKind, i32), String> {
    let (aircraft, _) = own_plane(state, player, plane)?;
    if let Some(choice) = via
        && !data::upgrade_choices(aircraft.kind).contains(&choice)
    {
        return Err(format!(
            "{} cannot upgrade to {}",
            data::unit_name(aircraft.kind),
            data::unit_name(choice)
        ));
    }
    let (to, _, price) = super::units::upgrade_plan_via(state, player, aircraft.kind, via)
        .ok_or_else(|| {
            let next = via.or(data::unit_stats(aircraft.kind).upgrades_to);
            match next.and_then(|n| data::unit_stats(n).requires) {
                Some(tech) => format!(
                    "{} needs {}",
                    data::unit_name(aircraft.kind),
                    data::tech_name(tech)
                ),
                None => format!("{} has no upgrade", data::unit_name(aircraft.kind)),
            }
        })?;
    Ok((to, price.total))
}

fn validate_upgrade(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
    via: Option<UnitKind>,
) -> Result<(UnitKind, i32), String> {
    let (to, cost) = upgrade_price(state, player, plane, via)?;
    if state.player(player).gold < cost {
        return Err(format!(
            "Not enough Gold to upgrade to {}: costs {cost}",
            data::unit_name(to)
        ));
    }
    if let Some(need) = crate::resources::unit_need(to)
        && crate::resources::unit_need(find(state, plane).expect("owned aircraft").kind)
            != Some(need)
        && let Some(reason) = crate::resources::shortage::<String>(state, player, need, None)
    {
        return Err(reason);
    }
    Ok((to, cost))
}

fn own_plane(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
) -> Result<(&Aircraft, &crate::state::City), String> {
    let a = find(state, plane).ok_or_else(|| "No such aircraft".to_string())?;
    let city = state
        .cities
        .get(&a.city)
        .filter(|c| c.owner == player)
        .ok_or_else(|| "That aircraft is not yours".to_string())?;
    Ok((a, city))
}

fn validate_strike(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
    target: Hex,
    vision: &mut Option<crate::bitset::BitSet>,
) -> Result<(), String> {
    let (a, city) = own_plane(state, player, plane)?;
    let stats = data::unit_stats(a.kind);
    if stats.ranged <= 0 {
        return Err(format!("{} cannot strike", data::unit_name(a.kind)));
    }
    let dist = city.pos.distance(target);
    if dist < 1 || dist > stats.range {
        return Err(format!(
            "target is out of range: {dist} tiles from {}, range {}",
            city.name, stats.range
        ));
    }
    battle::check_visible(state, player, target, vision)?;
    battle::target_at(state, player, target).map(|_| ())
}

/// A Recon needs an own aircraft and a tile on the map within its strike range of its city (the
/// same reach as its strike, seen or not: scouting is how a target becomes seen).
fn validate_recon(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
    target: Hex,
) -> Result<(), String> {
    if find(state, plane).is_none()
        && let Some(unit) = state.units.get(&plane)
    {
        return Err(format!(
            "{} is not an aircraft: only aircraft can scout from the air",
            data::unit_name(unit.kind)
        ));
    }
    let (a, city) = own_plane(state, player, plane)?;
    let range = data::unit_stats(a.kind).range;
    let dist = city.pos.distance(target);
    if state.map.index(target).is_none() {
        return Err("that tile is off the map".to_string());
    }
    if dist < 1 {
        return Err(format!("pick a tile away from {}", city.name));
    }
    if dist > range {
        return Err(format!(
            "target is out of range: {dist} tiles from {}, range {range}",
            city.name
        ));
    }
    Ok(())
}

/// The turn's reconnaissance, the last step of the ranged phase (after strikes, rebases and
/// upgrades): each valid `Order::AirRecon` records a reveal of its target for the next turn
/// (`visibility::add_recon_sight`: live only when `state.turn` is one past the recorded turn, so
/// nothing changes this turn's movement or fire). Reveals older than the turn just played are
/// dropped here.
fn settle_recon(
    state: &mut GameState,
    mut recons: Vec<(UnitId, PlayerId, Hex)>,
    events: &mut Vec<Event>,
) {
    let turn = state.turn;
    state
        .air_recons
        .retain(|r| r.turn.saturating_add(1) >= turn);
    recons.sort();
    for (plane, player, target) in recons {
        match validate_recon(state, player, plane, target) {
            Ok(()) => {
                let recon = AirRecon {
                    owner: player,
                    target,
                    turn,
                };
                if !state.air_recons.contains(&recon) {
                    state.air_recons.push(recon);
                }
            }
            Err(reason) => events.push(Event::OrderFailed {
                player,
                reason: format!("Recon by aircraft {} failed: {reason}", plane.0),
            }),
        }
    }
}

fn validate_rebase(
    state: &GameState,
    player: PlayerId,
    plane: UnitId,
    to: CityId,
    rebased_into: &mut BTreeMap<CityId, usize>,
) -> Result<(), String> {
    let (_, from) = own_plane(state, player, plane)?;
    let dest = state
        .cities
        .get(&to)
        .filter(|c| c.owner == player)
        .ok_or_else(|| "aircraft rebase only to your own cities".to_string())?;
    if dest.id == from.id {
        return Err("it is already based there".into());
    }
    let added = rebased_into.entry(to).or_default();
    let cap = data::air_base_cap(&state.player(player).techs);
    if hangar(state, to).count() + *added >= cap {
        return Err(format!("{}'s hangar is full ({} aircraft)", dest.name, cap));
    }
    *added += 1;
    Ok(())
}

/// The ranged phase with its air part: interceptions, then every strike that stands joins the
/// unit orders in `battle::resolve_ranged` (one stage, no retaliation); afterwards aircraft return
/// to their hangars and the turn's rebases happen.
pub fn resolve_ranged_phase(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    mut ranged: Vec<(PlayerId, UnitId, Hex)>,
    events: &mut Vec<Event>,
) -> Vec<battle::HeldShot> {
    let mut strikes = Vec::new();
    let mut rebases = Vec::new();
    let mut upgrades = Vec::new();
    let mut recons = Vec::new();
    let mut seen = BTreeSet::new();
    for (player, list) in orders {
        for order in list.iter() {
            match order {
                Order::AirStrike { plane, target } if seen.insert(*plane) => {
                    strikes.push((*plane, *player, *target));
                }
                Order::Rebase { plane, to } if seen.insert(*plane) => {
                    rebases.push((*plane, *player, *to));
                }
                Order::AirRecon { plane, target } if seen.insert(*plane) => {
                    recons.push((*plane, *player, *target));
                }
                Order::UpgradeAircraft { plane, to } if seen.insert(*plane) => {
                    upgrades.push((*plane, *player, *to));
                }
                _ => {}
            }
        }
    }
    if strikes.is_empty() && rebases.is_empty() && upgrades.is_empty() {
        let held = battle::resolve_ranged(state, &ranged, events);
        settle_recon(state, recons, events);
        return held;
    }
    strikes.sort();
    let mut vision: BTreeMap<TeamId, Option<crate::bitset::BitSet>> = BTreeMap::new();
    let mut ghosts: BTreeSet<UnitId> = BTreeSet::new();
    // Validate all strikes before interception so a valid Fighter strike uses its order
    // even when that Fighter's id comes after the incoming striker's id.
    let valid_strikes: Vec<_> = strikes
        .into_iter()
        .filter(|(plane, player, target)| {
            let v = vision.entry(state.team_of(*player)).or_default();
            if let Err(reason) = validate_strike(state, *player, *plane, *target, v) {
                events.push(Event::OrderFailed {
                    player: *player,
                    reason: format!("Air strike by aircraft {} failed: {reason}", plane.0),
                });
                false
            } else {
                true
            }
        })
        .collect();
    let mut used_fighters: BTreeSet<UnitId> = valid_strikes
        .iter()
        .filter(|(plane, _, _)| find(state, *plane).is_some_and(|a| data::is_fighter(a.kind)))
        .map(|(plane, _, _)| *plane)
        .collect();
    let mut duels = Vec::new();
    let mut defences: BTreeSet<CityId> = BTreeSet::new();
    let mut defence_units: BTreeSet<UnitId> = BTreeSet::new();
    for (plane, player, target) in valid_strikes {
        let striker_kind = find(state, plane).map(|a| a.kind);
        let defence = if striker_kind == Some(UnitKind::StealthBomber) {
            None
        } else {
            air_defence_at(state, player, target, &defences)
        };
        if let Some((city, owner)) = defence {
            defences.insert(city);
            let id = state.alloc_unit_id();
            let pos = state.cities[&city].pos;
            let interceptor = unit_with_id(state, id, owner, AIR_DEFENCE_INTERCEPTOR, pos);
            state.units.insert(id, interceptor);
            defence_units.insert(id);
            ghost(state, plane, &mut ghosts);
            duels.push((plane, id));
            continue;
        }
        let interceptor = state
            .aircraft
            .iter()
            .filter(|f| {
                data::is_fighter(f.kind)
                    && (find(state, plane)
                        .is_none_or(|striker| striker.kind != UnitKind::StealthBomber)
                        || f.kind == UnitKind::StealthFighter)
                    && !used_fighters.contains(&f.id)
                    && owner_of(state, f).is_some_and(|o| hostile(state, player, o))
                    && state.cities.get(&f.city).is_some_and(|c| {
                        c.pos.distance(target) <= data::fighter_guard_radius(f.kind)
                    })
            })
            .map(|f| f.id)
            .min();
        ghost(state, plane, &mut ghosts);
        match interceptor {
            Some(fighter) => {
                used_fighters.insert(fighter);
                ghost(state, fighter, &mut ghosts);
                duels.push((plane, fighter));
            }
            None => ranged.push((player, plane, target)),
        }
    }
    if !duels.is_empty() {
        let fights: Vec<_> = duels
            .iter()
            .map(|(bomber, fighter)| {
                let mut fight = battle::plan_fight(
                    state,
                    STAGE_AIR,
                    *bomber,
                    battle::Target::Unit(*fighter),
                    battle::FightKind::Melee,
                );
                if defence_units.contains(fighter) {
                    fight.damage_to_defender = 0;
                }
                fight
            })
            .collect();
        battle::apply_fights(state, &fights, events);
        for id in &defence_units {
            state.units.remove(id);
        }
    }
    let held = battle::resolve_ranged(state, &ranged, events);
    // Back to the hangars: survivors keep their wounds; the destroyed are gone.
    for id in ghosts {
        match state.units.remove(&id) {
            Some(u) => {
                if let Some(a) = state.aircraft.iter_mut().find(|a| a.id == id) {
                    a.hp = u.hp;
                    a.xp = u.xp;
                }
            }
            None => state.aircraft.retain(|a| a.id != id),
        }
    }
    rebases.sort();
    for (plane, player, to) in rebases {
        let ok = validate_rebase(state, player, plane, to, &mut BTreeMap::new());
        let (from, kind) = match find(state, plane) {
            Some(a) if ok.is_ok() => (a.city, a.kind),
            _ => {
                if let Err(reason) = ok {
                    events.push(Event::OrderFailed {
                        player,
                        reason: format!("Rebase of aircraft {} failed: {reason}", plane.0),
                    });
                }
                continue;
            }
        };
        if let Some(a) = state.aircraft.iter_mut().find(|a| a.id == plane) {
            a.city = to;
        }
        events.push(Event::AircraftRebased {
            plane,
            owner: player,
            kind,
            from,
            to,
        });
    }
    upgrades.sort();
    for (plane, player, via) in upgrades {
        match validate_upgrade(state, player, plane, via) {
            Ok((to, cost)) => {
                state.player_mut(player).gold -= cost;
                find_mut(state, plane).expect("validated aircraft").kind = to;
                events.push(Event::UnitUpgraded {
                    unit: plane,
                    owner: player,
                    to,
                    gold: cost,
                });
            }
            Err(reason) => events.push(Event::OrderFailed {
                player,
                reason: format!("Upgrade of aircraft {} failed: {reason}", plane.0),
            }),
        }
    }
    settle_recon(state, recons, events);
    held
}

/// The city whose Air Defence meets a strike by `player` on `target`: a city standing on that tile
/// with an Air Defence, hostile to the striker, that has not intercepted yet this turn.
pub fn air_defence_at(
    state: &GameState,
    player: PlayerId,
    target: Hex,
    used: &BTreeSet<CityId>,
) -> Option<(CityId, PlayerId)> {
    state
        .cities
        .values()
        .find(|c| {
            c.pos == target
                && !used.contains(&c.id)
                && data::has_building(&c.buildings, BuildingKind::AirDefence)
                && hostile(state, player, c.owner)
        })
        .map(|c| (c.id, c.owner))
}

fn find_mut(state: &mut GameState, id: UnitId) -> Option<&mut Aircraft> {
    state.aircraft.iter_mut().find(|a| a.id == id)
}

/// A temporary map unit for an aircraft that acts this turn (same id, at its city).
fn ghost(state: &mut GameState, plane: UnitId, ghosts: &mut BTreeSet<UnitId>) {
    if !ghosts.insert(plane) {
        return;
    }
    let a = find(state, plane).expect("validated aircraft").clone();
    let city = &state.cities[&a.city];
    let (owner, pos) = (city.owner, city.pos);
    let mut u = unit_with_id(state, a.id, owner, a.kind, pos);
    u.hp = a.hp;
    u.xp = a.xp;
    state.units.insert(a.id, u);
}

#[cfg(test)]
mod tests {
    use super::super::promotion;
    use super::super::test_support::*;
    use super::*;
    use crate::rules::resolve_tactical_turn;

    const P0: PlayerId = PlayerId(0);
    const P1: PlayerId = PlayerId(1);

    /// Two empires, P0's city at (3,4) with a Scout for sight, P1's city at (7,4).
    fn arena() -> (GameState, CityId, CityId) {
        let mut s = game(&[0, 1]);
        let a = city(&mut s, 0, hex(3, 4), 3);
        let b = city(&mut s, 1, hex(7, 4), 3);
        (s, a, b)
    }

    /// The air check alone, with nothing else in the order list spending first.
    fn validate_orders(
        s: &GameState,
        player: PlayerId,
        orders: &[Order],
    ) -> Vec<crate::orders::OrderError> {
        super::validate_orders(s, player, orders, s.clone())
    }

    fn strike(plane: UnitId, at: Hex) -> BTreeMap<PlayerId, Vec<Order>> {
        orders(0, vec![Order::AirStrike { plane, target: at }])
    }

    fn plane_hp(s: &GameState, id: UnitId) -> Option<i32> {
        find(s, id).map(|a| a.hp)
    }

    #[test]
    fn hangar_aircraft_count_toward_army_strength_with_the_same_formula() {
        let (mut s, city, _) = arena();
        let before = crate::scoring::army_strength(&s, P0);
        base_new(&mut s, city, UnitKind::Bomber);
        let plane = &s.aircraft[0];
        let stats = data::unit_stats(plane.kind);
        let expected = crate::army::combat_value(stats.strength, stats.ranged, plane.hp, 0, false);
        assert_eq!(crate::army::aircraft_value(&s, plane), expected);
        assert_eq!(crate::scoring::army_strength(&s, P0), before + expected);
        assert_eq!(
            crate::army::breakdown(&s, P0).last().unwrap().value,
            expected
        );
    }

    #[test]
    fn a_bomber_strikes_an_enemy_unit_in_range_with_no_retaliation() {
        let (mut s, a, _b) = arena();
        let victim = unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        assert!(validate_orders(&s, P0, &strike(plane, hex(6, 4))[&P0]).is_empty());
        let (next, events) = resolve_tactical_turn(&s, &strike(plane, hex(6, 4)));
        assert!(
            next.units.get(&victim).is_none_or(|u| u.hp < 100),
            "the strike landed"
        );
        assert_eq!(plane_hp(&next, plane), Some(100), "no retaliation");
        assert!(
            next.units
                .values()
                .all(|u| !crate::data::is_aircraft(u.kind)),
            "aircraft never stay on the map"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: true, .. }))
        );
    }

    #[test]
    fn an_oil_short_bomber_has_the_same_visible_strength_penalty_in_forecast_and_combat() {
        let (mut s, a, _) = arena();
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::Fighter);
        let plane = s.aircraft[0].id;
        crate::resources::update_supply(&mut s, &mut Vec::new());
        let view = crate::view::player_view(&s, P0);
        for aircraft in &view.aircraft {
            assert_eq!(aircraft.unsupplied, Some(crate::resources::Resource::Oil));
            let forecast =
                crate::forecast::forecast_air_strike(&view, aircraft.id, hex(6, 4)).unwrap();
            assert!(
                forecast
                    .attacker_strength
                    .parts
                    .iter()
                    .any(|(label, loss)| label == "No Oil supply" && *loss < 0)
            );
        }
        let (_, events) = resolve_tactical_turn(&s, &strike(plane, hex(6, 4)));
        assert!(events.iter().any(|e| matches!(e, Event::Battle { attacker_strength, .. }
            if attacker_strength.parts.iter().any(|(label, loss)| label == "No Oil supply" && *loss < 0))));
    }

    #[test]
    fn a_strike_needs_range_sight_war_and_a_bomber_and_each_plane_acts_once() {
        let (mut s, a, _b) = arena();
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 1, UnitKind::Soldier, hex(13, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::Fighter);
        let (bomber, fighter) = (s.aircraft[0].id, s.aircraft[1].id);
        let errs = |os: Vec<Order>| validate_orders(&s, P0, &os);
        // Out of range (the Bomber reaches 8 hexes from its city).
        assert!(
            !errs(vec![Order::AirStrike {
                plane: bomber,
                target: hex(13, 4)
            }])
            .is_empty()
        );
        // Nothing there.
        assert!(
            !errs(vec![Order::AirStrike {
                plane: bomber,
                target: hex(5, 5)
            }])
            .is_empty()
        );
        // A Fighter strikes, but only 5 hexes from its city; a stranger's plane is not yours.
        assert!(
            errs(vec![Order::AirStrike {
                plane: fighter,
                target: hex(6, 4)
            }])
            .is_empty()
        );
        assert!(
            !errs(vec![Order::AirStrike {
                plane: fighter,
                target: hex(9, 4)
            }])
            .is_empty(),
            "a Fighter reaches {} hexes, this target is 6 away",
            crate::data::FIGHTER_STRIKE_RANGE
        );
        // A stranger's plane is not yours.
        assert!(
            !validate_orders(
                &s,
                P1,
                &[Order::AirStrike {
                    plane: bomber,
                    target: hex(3, 5)
                }]
            )
            .is_empty()
        );
        // One order per aircraft.
        assert_eq!(
            errs(vec![
                Order::AirStrike {
                    plane: bomber,
                    target: hex(6, 4)
                },
                Order::AirStrike {
                    plane: bomber,
                    target: hex(6, 4)
                },
            ])
            .len(),
            1
        );
    }

    #[test]
    fn a_bomber_hits_a_city_and_an_enemy_fighter_intercepts_the_first_strike_only() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, b, UnitKind::Fighter);
        let (first, second) = (s.aircraft[0].id, s.aircraft[1].id);
        let fighter = s.aircraft[2].id;
        let os = orders(
            0,
            vec![
                Order::AirStrike {
                    plane: second,
                    target: hex(7, 4),
                },
                Order::AirStrike {
                    plane: first,
                    target: hex(7, 4),
                },
            ],
        );
        let (next, events) = resolve_tactical_turn(&s, &os);
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Battle {
                air_strike: true,
                ..
            }
        )));
        // The lowest-id Bomber was intercepted: the Fighter and it traded damage, its strike did not land.
        let duel = events.iter().find_map(|e| match e {
            Event::Battle {
                attacker,
                defender: crate::events::Defender::Unit { unit, .. },
                ranged: false,
                damage_to_attacker,
                damage_to_defender,
                ..
            } if attacker.unit() == Some(first) && *unit == fighter => {
                Some((*damage_to_attacker, *damage_to_defender))
            }
            _ => None,
        });
        let (taken, dealt) = duel.expect("the Fighter met the first Bomber");
        assert!(taken > 0 && dealt > 0, "they trade damage");
        assert!(plane_hp(&next, first).is_none_or(|hp| hp < 100));
        assert!(plane_hp(&next, fighter).is_none_or(|hp| hp < 100));
        assert!(
            next.air_contacts[&crate::PlayerId(0)]
                .iter()
                .any(|c| c.owner == crate::PlayerId(1)
                    && c.kind == Some(UnitKind::Fighter)
                    && c.turn == s.turn)
        );
        assert!(
            next.air_contacts[&crate::PlayerId(1)]
                .iter()
                .any(|c| c.owner == crate::PlayerId(0) && c.kind.is_none() && c.turn == s.turn)
        );
        // The second Bomber's strike landed on the city: one interception per Fighter per turn.
        assert!(next.cities[&b].hp < 200);
        assert_eq!(plane_hp(&next, second), Some(100));
        // A Fighter within 4 hexes of the target protects it; one farther away does not.
        let mut far = s.clone();
        far.aircraft.retain(|p| p.kind != UnitKind::Fighter);
        let c = city(&mut far, 1, hex(11, 9), 2);
        assert!(hex(11, 9).distance(hex(7, 4)) > crate::data::FIGHTER_RANGE);
        base_new(&mut far, c, UnitKind::Fighter);
        let (hit, _) = resolve_tactical_turn(&far, &strike(first, hex(7, 4)));
        assert!(
            hit.cities[&b].hp < 200,
            "a Fighter too far from the target cannot intercept"
        );
    }

    #[test]
    fn only_stealth_fighters_intercept_stealth_bombers() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::StealthBomber);
        base_new(&mut s, b, UnitKind::Fighter);
        let bomber = s.aircraft[0].id;
        let (ordinary, events) = resolve_tactical_turn(&s, &strike(bomber, hex(7, 4)));
        assert!(ordinary.cities[&b].hp < s.cities[&b].hp);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: false, .. }))
        );
        s.aircraft[1].kind = UnitKind::StealthFighter;
        let (guarded, events) = resolve_tactical_turn(&s, &strike(bomber, hex(7, 4)));
        assert_eq!(guarded.cities[&b].hp, s.cities[&b].hp);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: false, .. }))
        );
    }

    #[test]
    fn an_air_defence_meets_one_strike_a_turn_on_its_city_and_a_stealth_bomber_passes_it() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::StealthBomber);
        let (first, second, stealth) = (s.aircraft[0].id, s.aircraft[1].id, s.aircraft[2].id);
        s.cities
            .get_mut(&b)
            .unwrap()
            .buildings
            .insert(BuildingKind::AirDefence);
        let hangar_before = s.aircraft.len();
        let city_hp = s.cities[&b].hp;
        // One Bomber: the Air Defence meets it, the strike does not land and the two trade damage
        // (the Bomber takes some, the building none and no hangar aircraft is hit).
        let (met, events) = resolve_tactical_turn(&s, &strike(first, hex(7, 4)));
        assert_eq!(met.cities[&b].hp, city_hp, "the strike did not land");
        let duel = events.iter().find_map(|e| match e {
            Event::Battle {
                attacker,
                defender: crate::events::Defender::Unit { unit_kind, .. },
                ranged: false,
                damage_to_attacker,
                damage_to_defender,
                ..
            } if attacker.unit() == Some(first) => {
                Some((*unit_kind, *damage_to_attacker, *damage_to_defender))
            }
            _ => None,
        });
        let (kind, taken, dealt) = duel.expect("the Air Defence met the strike");
        assert_eq!(kind, super::AIR_DEFENCE_INTERCEPTOR);
        assert!(taken > 0, "the Bomber takes damage");
        assert_eq!(dealt, 0, "the Air Defence takes none");
        assert!(plane_hp(&met, first).is_none_or(|hp| hp < 100));
        assert_eq!(
            met.aircraft.len(),
            hangar_before - usize::from(plane_hp(&met, first).is_none())
        );
        assert!(
            met.units
                .values()
                .all(|u| u.kind != super::AIR_DEFENCE_INTERCEPTOR),
            "the interceptor leaves no unit on the map"
        );
        // Two Bombers: one interception a turn, the second strike lands.
        let both = orders(
            0,
            vec![
                Order::AirStrike {
                    plane: first,
                    target: hex(7, 4),
                },
                Order::AirStrike {
                    plane: second,
                    target: hex(7, 4),
                },
            ],
        );
        let (two, _) = resolve_tactical_turn(&s, &both);
        assert!(two.cities[&b].hp < city_hp, "the second strike landed");
        // A Stealth Bomber passes the Air Defence exactly as it passes an ordinary Fighter.
        let (passed, events) = resolve_tactical_turn(&s, &strike(stealth, hex(7, 4)));
        assert!(passed.cities[&b].hp < city_hp);
        assert_eq!(plane_hp(&passed, stealth), Some(100));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: false, .. }))
        );
        // A strike on a tile beside the city is not met: the Air Defence guards its own city only.
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        let (beside, _) = resolve_tactical_turn(&s, &strike(first, hex(6, 4)));
        assert!(
            !beside
                .units
                .values()
                .any(|u| u.kind == UnitKind::Soldier && u.hp == 100),
            "the strike on the tile beside the city landed"
        );
        assert_eq!(plane_hp(&beside, first), Some(100));
    }

    #[test]
    fn the_forecast_of_a_strike_on_a_city_with_an_air_defence_shows_the_duel_but_not_for_stealth() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::StealthBomber);
        let (bomber, stealth) = (s.aircraft[0].id, s.aircraft[1].id);
        let forecast = |s: &GameState, plane: UnitId| {
            let view = crate::player_view(s, P0);
            crate::forecast_attack_mode(&view, plane, hex(7, 4), crate::AttackMode::Ranged)
                .expect("a forecast")
        };
        assert!(
            forecast(&s, bomber).air_defence.is_none(),
            "no Air Defence yet"
        );
        s.cities
            .get_mut(&b)
            .unwrap()
            .buildings
            .insert(BuildingKind::AirDefence);
        let f = forecast(&s, bomber);
        let duel = f.air_defence.expect("the forecast shows the Air Defence");
        assert!(duel.expected_damage_to_attacker > 0);
        assert!(duel.interceptor_strength > 0 && duel.striker_strength > 0);
        assert!(forecast(&s, stealth).air_defence.is_none());
    }

    #[test]
    fn the_forecast_of_a_strike_is_the_ranged_math_from_the_hangar() {
        let (mut s, a, b) = arena();
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let view = crate::player_view(&s, P0);
        let f = crate::forecast_attack_mode(&view, plane, hex(6, 4), crate::AttackMode::Ranged)
            .expect("a forecast");
        assert_eq!(f.attacker_strength.total, crate::data::era_scaled(64, 6));
        assert!(
            f.ranged && f.expected_damage_to_attacker == 0 && f.expected_damage_to_defender > 0
        );
        // Out of range and a rival's plane have none.
        assert!(
            crate::forecast_attack_mode(&view, plane, hex(11, 4), crate::AttackMode::Ranged)
                .is_none()
        );
        base_new(&mut s, b, UnitKind::Fighter);
        let view = crate::player_view(&s, P0);
        let rival = s.aircraft[1].id;
        assert!(
            crate::forecast_attack_mode(&view, rival, hex(6, 4), crate::AttackMode::Ranged)
                .is_none()
        );
        // Your own Fighter: its own strength, only within its short strike range.
        base_new(&mut s, a, UnitKind::Fighter);
        let view = crate::player_view(&s, P0);
        let mine = s.aircraft[2].id;
        let f = crate::forecast_attack_mode(&view, mine, hex(6, 4), crate::AttackMode::Ranged)
            .expect("a Fighter forecast");
        assert_eq!(f.attacker_strength.total, crate::data::era_scaled(58, 6));
        assert!(f.ranged && f.expected_damage_to_defender > 0);
        assert!(
            crate::forecast_attack_mode(&view, mine, hex(8, 4), crate::AttackMode::Ranged)
                .is_none(),
            "5 hexes is a Bomber's reach, not a Fighter's"
        );
    }

    #[test]
    fn a_fighter_hits_units_without_a_city_bonus_and_a_striking_fighter_does_not_intercept() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        let victim = unit(&mut s, 1, UnitKind::Soldier, hex(5, 5));
        unit(&mut s, 0, UnitKind::Soldier, hex(6, 4));
        base_new(&mut s, a, UnitKind::Fighter);
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, b, UnitKind::Fighter);
        let (mine, bomber, guard) = (s.aircraft[0].id, s.aircraft[1].id, s.aircraft[2].id);
        // The Fighter strikes a unit in its short range with no retaliation (no enemy Fighter on guard).
        let mut calm = s.clone();
        calm.aircraft.retain(|p| p.id != guard);
        let (next, events) = resolve_tactical_turn(&calm, &strike(mine, hex(5, 5)));
        assert!(next.units.get(&victim).is_none_or(|u| u.hp < 100));
        assert_eq!(plane_hp(&next, mine), Some(100), "no retaliation");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: true, .. }))
        );
        // An enemy Fighter on guard meets a Fighter's strike too, within its 4 hexes: it is an air strike.
        let (met_f, _) = resolve_tactical_turn(&s, &strike(mine, hex(5, 5)));
        assert!(
            met_f.units.get(&victim).is_none_or(|u| u.hp == 100),
            "intercepted: the strike did not land"
        );
        // The enemy Fighter on guard meets the Bomber's strike on its own city...
        let (met, _) = resolve_tactical_turn(&s, &strike(bomber, hex(7, 4)));
        assert!(plane_hp(&met, guard).is_none_or(|hp| hp < 100));
        assert_eq!(
            met.cities[&b].hp, 200,
            "intercepted: the strike did not land"
        );
        // ...but not when that Fighter strikes this turn: one order per aircraft.
        let mut both = strike(bomber, hex(7, 4));
        both.insert(
            P1,
            vec![Order::AirStrike {
                plane: guard,
                target: hex(6, 4),
            }],
        );
        let (busy, _) = resolve_tactical_turn(&s, &both);
        assert!(
            busy.cities[&b].hp < 200,
            "a striking Fighter is not on guard"
        );
    }

    #[test]
    fn aircraft_ignore_line_of_sight_but_ground_shooters_do_not() {
        let (mut s, a, _b) = arena();
        let victim = unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(6, 5));
        for h in [hex(4, 4), hex(5, 4)] {
            set_terrain(&mut s, h, Terrain::Forest);
        }
        set_terrain(&mut s, hex(5, 5), Terrain::Hills);
        let terrain = |h: Hex| s.map.get(h).map(|t| t.terrain);
        assert!(
            !crate::los::ranged_clear(UnitKind::Ranger, hex(3, 4), hex(6, 4), terrain),
            "the forest blocks a ground shooter"
        );
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, a, UnitKind::Fighter);
        let (bomber, fighter) = (s.aircraft[0].id, s.aircraft[1].id);
        for plane in [bomber, fighter] {
            let order = Order::AirStrike {
                plane,
                target: hex(6, 4),
            };
            let errors = validate_orders(&s, P0, std::slice::from_ref(&order));
            assert!(
                errors.is_empty(),
                "validation ignores line of sight: {errors:?}"
            );
        }
        let (next, events) = resolve_tactical_turn(&s, &strike(bomber, hex(6, 4)));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. })),
            "{events:?}"
        );
        assert!(
            next.units.get(&victim).is_none_or(|u| u.hp < 100),
            "the strike landed over the forest"
        );
        // The forecast reads the same table.
        let view = crate::view::player_view(&s, P0);
        assert!(crate::forecast::forecast_air_strike(&view, bomber, hex(6, 4)).is_some());
        assert!(crate::forecast::forecast_air_strike(&view, fighter, hex(6, 4)).is_some());
    }

    fn at_peace(s: &mut GameState) {
        for r in &mut s.diplomacy.as_mut().unwrap().relations {
            r.state = crate::diplomacy::RelationState::Peace;
        }
    }

    #[test]
    fn a_strike_on_the_turn_war_is_declared_resolves_like_a_ground_attack() {
        let (mut s, a, _b) = arena();
        at_peace(&mut s);
        let victim = unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let hit = Order::AirStrike {
            plane,
            target: hex(6, 4),
        };
        let war = Order::DeclareWar {
            against: TeamId(1),
            breach: false,
        };
        // At Peace with no declaration: both validation and resolution refuse it.
        let plain = orders(0, vec![hit.clone()]);
        assert!(!crate::validate_orders(&s, P0, &plain[&P0]).is_empty());
        let (kept, events) = crate::rules::resolve_turn(&s, &plain);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
        assert_eq!(kept.units[&victim].hp, 100);
        // With the declaration in the same list the strike is valid and lands this turn.
        let surprise = orders(0, vec![war.clone(), hit.clone()]);
        assert!(crate::validate_orders(&s, P0, &surprise[&P0]).is_empty());
        let (next, events) = crate::rules::resolve_turn(&s, &surprise);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. })),
            "{events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: true, attacker, .. } if attacker.unit() == Some(plane)))
        );
        assert!(next.units.get(&victim).is_none_or(|u| u.hp < 100));
        // Fighters use the same order availability as Bombers.
        base_new(&mut s, a, UnitKind::Fighter);
        let fighter_hit = Order::AirStrike {
            plane: s.aircraft[1].id,
            target: hex(6, 4),
        };
        assert!(!crate::validate_orders(&s, P0, std::slice::from_ref(&fighter_hit)).is_empty());
        assert!(crate::validate_orders(&s, P0, &[war.clone(), fighter_hit]).is_empty());
        // The declaration comes first or last in the list: the same war opens before the ranged phase.
        let reversed = orders(0, vec![hit, war]);
        assert!(crate::validate_orders(&s, P0, &reversed[&P0]).is_empty());
    }

    #[test]
    fn a_fighter_with_an_invalid_strike_still_intercepts() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        base_new(&mut s, b, UnitKind::Fighter);
        let (bomber, fighter) = (s.aircraft[0].id, s.aircraft[1].id);
        let mut os = strike(bomber, hex(7, 4));
        os.insert(
            P1,
            vec![Order::AirStrike {
                plane: fighter,
                target: hex(1, 4), // Six tiles away; a Fighter can strike only five.
            }],
        );
        let (next, events) = resolve_tactical_turn(&s, &os);
        assert_eq!(next.cities[&b].hp, 200, "the Bomber strike was intercepted");
        assert!(events.iter().any(|e| matches!(
            e,
            Event::OrderFailed { player, reason }
                if *player == P1 && reason.contains("out of range")
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Battle {
                attacker,
                defender: crate::events::Defender::Unit { unit, .. },
                ranged: false,
                ..
            } if attacker.unit() == Some(bomber) && *unit == fighter
        )));
    }

    #[test]
    fn rebasing_moves_a_plane_to_a_city_with_room_within_range() {
        let (mut s, a, _b) = arena();
        let c = city(&mut s, 0, hex(8, 8), 2);
        base_new(&mut s, a, UnitKind::Fighter);
        let plane = s.aircraft[0].id;
        let os = orders(0, vec![Order::Rebase { plane, to: c }]);
        assert!(validate_orders(&s, P0, &os[&P0]).is_empty());
        let (next, events) = resolve_tactical_turn(&s, &os);
        assert_eq!(find(&next, plane).map(|p| p.city), Some(c));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::AircraftRebased { to, .. } if *to == c))
        );
        // Not to a rival's city, and not into a full hangar.
        let rival = Order::Rebase { plane, to: _b };
        assert!(!validate_orders(&s, P0, &[rival]).is_empty());
        base_new(&mut s, c, UnitKind::Bomber);
        base_new(&mut s, c, UnitKind::Bomber);
        assert!(!validate_orders(&s, P0, &os[&P0]).is_empty());
    }

    #[test]
    fn rebasing_works_at_any_distance() {
        let (mut s, a, _b) = arena();
        let far = city(&mut s, 0, hex(3, 60), 2);
        assert!(s.cities[&a].pos.distance(s.cities[&far].pos) > 50);
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let os = orders(0, vec![Order::Rebase { plane, to: far }]);
        assert!(validate_orders(&s, P0, &os[&P0]).is_empty());
        let (next, _) = resolve_tactical_turn(&s, &os);
        assert_eq!(find(&next, plane).map(|p| p.city), Some(far));
    }

    #[test]
    fn duplicate_rebase_does_not_reserve_an_extra_hangar_slot() {
        let (mut s, a, _enemy) = arena();
        let other = city(&mut s, 0, hex(4, 5), 2);
        let x = city(&mut s, 0, hex(8, 8), 2);
        base_new(&mut s, a, UnitKind::Fighter);
        base_new(&mut s, other, UnitKind::Bomber);
        let (plane_a, plane_b) = (s.aircraft[0].id, s.aircraft[1].id);
        let orders = vec![
            Order::Rebase {
                plane: plane_a,
                to: x,
            },
            Order::Rebase {
                plane: plane_a,
                to: x,
            },
            Order::Rebase {
                plane: plane_b,
                to: x,
            },
        ];
        let errors = validate_orders(&s, P0, &orders);
        assert_eq!(
            errors.len(),
            1,
            "only the duplicate should fail: {errors:?}"
        );
        assert_eq!(errors[0].index, 1);
        assert!(errors[0].message.contains("already has an order"));

        let (next, events) = crate::rules::resolve_turn(&s, &self::orders(0, orders));
        let in_x: BTreeSet<_> = next
            .aircraft
            .iter()
            .filter(|plane| plane.city == x)
            .map(|plane| plane.id)
            .collect();
        assert_eq!(in_x, BTreeSet::from([plane_a, plane_b]));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::AircraftRebased { to, .. } if *to == x))
                .count(),
            2
        );
    }

    #[test]
    fn a_hangar_holds_two_and_aircraft_are_lost_with_their_city() {
        let (mut s, a, _b) = arena();
        assert!(hangar_blocker(&s, a).is_none());
        base_new(&mut s, a, UnitKind::Fighter);
        base_new(&mut s, a, UnitKind::Bomber);
        assert!(hangar_blocker(&s, a).is_some());
        let item = ProductionItem::Unit {
            unit: UnitKind::Bomber,
        };
        assert!(crate::rules::cities::spawn_tile(&s, P0, hex(3, 4), UnitKind::Bomber).is_none());
        assert!(crate::economy::placement_blocker(&s, &s.cities[&a], item).is_some());
        let mut events = Vec::new();
        lose_city_aircraft(&mut s, a, &mut events);
        assert!(s.aircraft.is_empty());
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn aircraft_are_not_a_garrison_or_upkeep() {
        let (mut s, a, _b) = arena();
        unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
        unit(&mut s, 0, UnitKind::Soldier, hex(4, 5));
        assert_eq!(crate::economy::upkeep(&s, P0), 0, "two free land units");
        base_new(&mut s, a, UnitKind::Fighter);
        assert!(!crate::rules::is_military(UnitKind::Fighter));
        assert!(!crate::data::pays_upkeep(UnitKind::Bomber));
        assert_eq!(crate::economy::upkeep_unit_count(&s, P0), 2);
        assert_eq!(crate::economy::levy_count(&s, P0), 3);
        assert_eq!(
            crate::economy::upkeep(&s, P0),
            0,
            "the Fighter has no Gold bill"
        );
        let v = crate::player_view(&s, P0);
        assert_eq!(v.aircraft.len(), 1);
        assert!(
            crate::player_view(&s, P1).aircraft.is_empty(),
            "rivals never see them"
        );
        unit(&mut s, 0, UnitKind::Soldier, hex(5, 4));
        assert_eq!(
            crate::economy::upkeep(&s, P0),
            data::military_upkeep(s.player(P0).faction),
            "the third land unit pays"
        );
    }

    fn xp_of(s: &GameState, id: UnitId) -> u32 {
        find(s, id).expect("the aircraft survived").xp
    }

    fn stats(s: &GameState, kind: UnitKind) -> data::UnitStats {
        data::unit_stats_for(s.player(P0).faction, kind)
    }

    #[test]
    fn a_strike_on_a_unit_or_a_city_earns_the_ground_unit_xp() {
        let (mut s, a, b) = arena();
        let victim = unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let (next, _) = resolve_tactical_turn(&s, &strike(plane, hex(6, 4)));
        let died = !next.units.contains_key(&victim);
        assert_eq!(
            xp_of(&next, plane),
            promotion::unit_fight_xp(
                stats(&s, UnitKind::Bomber).ranged,
                stats(&s, UnitKind::Soldier).strength,
                died,
                true,
                true
            )
        );
        assert!(xp_of(&next, plane) > 0);

        let (next, _) = resolve_tactical_turn(&s, &strike(plane, hex(7, 4)));
        assert_eq!(
            xp_of(&next, plane),
            promotion::city_attack_xp(s.cities[&b].hp, next.cities[&b].hp)
        );
    }

    /// A garrison killed by the strike's overflow (`battle::carried_damage`, rules 27) is no
    /// fight of its own: the plane earns the city-attack XP, as any ground attacker would.
    #[test]
    fn a_garrison_killed_by_a_strikes_overflow_earns_the_city_attack_xp() {
        let (mut s, a, b) = arena();
        s.cities.get_mut(&b).unwrap().hp = 1;
        let garrison = unit(&mut s, 1, UnitKind::Soldier, hex(7, 4));
        s.units.get_mut(&garrison).unwrap().hp = 1;
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let (next, _) = resolve_tactical_turn(&s, &strike(plane, hex(7, 4)));
        assert!(
            !next.units.contains_key(&garrison),
            "the overflow killed it"
        );
        assert_eq!(
            xp_of(&next, plane),
            promotion::city_attack_xp(1, next.cities[&b].hp)
        );
        assert!(xp_of(&next, plane) > 0);
    }

    #[test]
    fn an_interception_gives_both_aircraft_xp() {
        let (mut s, a, b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        base_new(&mut s, a, UnitKind::Fighter);
        base_new(&mut s, b, UnitKind::Fighter);
        let (striker, guard) = (s.aircraft[0].id, s.aircraft[1].id);
        let (next, events) = resolve_tactical_turn(&s, &strike(striker, hex(6, 4)));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Battle { ranged: false, .. })),
            "the guard met the striker"
        );
        let strength = stats(&s, UnitKind::Fighter).strength;
        assert_eq!(
            xp_of(&next, striker),
            promotion::unit_fight_xp(strength, strength, false, false, true)
        );
        assert_eq!(
            xp_of(&next, guard),
            promotion::unit_fight_xp(strength, strength, false, false, false)
        );
    }

    #[test]
    fn aircraft_level_up_at_the_ground_thresholds_and_announce_it() {
        let (mut s, a, _b) = arena();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Fighter);
        let plane = s.aircraft[0].id;
        let first = promotion::threshold(1).unwrap();
        let (fresh, _) = resolve_tactical_turn(&s, &strike(plane, hex(7, 4)));
        let gain = xp_of(&fresh, plane);
        s.aircraft[0].xp = first - gain - 1;
        let (next, events) = resolve_tactical_turn(&s, &strike(plane, hex(7, 4)));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::UnitPromoted { .. }))
        );
        assert_eq!(crate::player_view(&next, P0).aircraft[0].level, 0);
        s.aircraft[0].xp = first - gain;
        let (next, events) = resolve_tactical_turn(&s, &strike(plane, hex(7, 4)));
        assert!(events.iter().any(|e| matches!(e, Event::UnitPromoted {
            unit, level: 1, unit_kind: UnitKind::Fighter, ..
        } if *unit == plane)));
        let view = &crate::player_view(&next, P0).aircraft[0];
        assert_eq!((view.level, view.xp), (1, first));
        assert_eq!(view.xp_next, promotion::threshold(2));
    }

    #[test]
    fn the_level_bonus_is_in_the_forecast_and_the_resolved_strike_alike() {
        let (mut s, a, _b) = arena();
        unit(&mut s, 1, UnitKind::Soldier, hex(6, 4));
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let target = hex(6, 4);
        let plain =
            crate::forecast::forecast_air_strike(&crate::player_view(&s, P0), plane, target)
                .unwrap()
                .attacker_strength;
        s.aircraft[0].xp = promotion::threshold(3).unwrap();
        let view = crate::player_view(&s, P0);
        let forecast = crate::forecast::forecast_air_strike(&view, plane, target).unwrap();
        let bonus = forecast.attacker_strength.total - plain.total;
        assert!(bonus > 0);
        assert!(
            forecast
                .attacker_strength
                .parts
                .iter()
                .any(|(label, value)| label == "Level 3: +30%" && *value == bonus)
        );
        let (_, events) = resolve_tactical_turn(&s, &strike(plane, target));
        let resolved = events
            .iter()
            .find_map(|e| match e {
                Event::Battle {
                    air_strike: true,
                    attacker_strength,
                    ..
                } => Some(attacker_strength),
                _ => None,
            })
            .expect("the strike");
        assert_eq!(resolved, &forecast.attacker_strength);
    }

    #[test]
    fn xp_is_kept_through_a_rebase_and_the_stealth_upgrade() {
        let (mut s, a, _b) = arena();
        let c = city(&mut s, 0, hex(8, 8), 2);
        base_new(&mut s, a, UnitKind::Fighter);
        let plane = s.aircraft[0].id;
        s.aircraft[0].xp = 50;
        let os = orders(0, vec![Order::Rebase { plane, to: c }]);
        let (next, _) = resolve_tactical_turn(&s, &os);
        assert_eq!(find(&next, plane).map(|p| (p.city, p.xp)), Some((c, 50)));

        s.player_mut(P0).techs.extend([
            TechId::JetEngines,
            TechId::StealthTechnology,
            TechId::AutonomousSystems,
        ]);
        let mine = s.map.get_mut(hex(4, 5)).unwrap();
        mine.resource = Some(crate::resources::Resource::Silicon);
        mine.improved = true;
        mine.owner = Some(P0);
        mine.city = Some(a);
        s.player_mut(P0).gold = 10_000;
        let os = orders(0, vec![Order::UpgradeAircraft { plane, to: None }]);
        let (next, events) = resolve_tactical_turn(&s, &os);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::UnitUpgraded { .. }))
        );
        let upgraded = find(&next, plane).unwrap();
        assert_eq!((upgraded.kind, upgraded.xp), (UnitKind::StealthFighter, 50));
    }

    /// Two Fighters in `a` that can each upgrade to a Stealth Fighter, with Silicon for both.
    fn two_fighters_ready_for_stealth() -> (GameState, CityId, UnitId, UnitId, i32) {
        let (mut s, a, _b) = arena();
        base_new(&mut s, a, UnitKind::Fighter);
        base_new(&mut s, a, UnitKind::Fighter);
        let (p1, p2) = (s.aircraft[0].id, s.aircraft[1].id);
        s.player_mut(P0).techs.extend([
            TechId::JetEngines,
            TechId::StealthTechnology,
            TechId::AutonomousSystems,
        ]);
        let silicon = crate::resources::Resource::Silicon;
        for h in hex(3, 4).neighbors() {
            if crate::resources::spare(&s, P0, silicon, None) >= 2 {
                break;
            }
            let t = s.map.get_mut(h).unwrap();
            t.resource = Some(silicon);
            t.improved = true;
            t.owner = Some(P0);
            t.city = Some(a);
        }
        assert!(crate::resources::spare(&s, P0, silicon, None) >= 2);
        let (to, price) = upgrade_price(&s, P0, p1, None).unwrap();
        assert_eq!(to, UnitKind::StealthFighter);
        assert_eq!(upgrade_price(&s, P0, p2, None).unwrap().1, price);
        (s, a, p1, p2, price)
    }

    #[test]
    fn a_second_aircraft_upgrade_is_checked_against_the_gold_the_first_leaves() {
        let (mut s, _a, p1, p2, price) = two_fighters_ready_for_stealth();
        s.player_mut(P0).gold = 2 * price - 1;
        let up = |plane| Order::UpgradeAircraft { plane, to: None };
        // Resolution pays in aircraft-id order, so the lower id is upgraded whatever the list
        // order, and the other order is the one refused.
        let os = vec![up(p2), up(p1)];
        let errors = crate::validate_orders(&s, P0, &os);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].index, 0);
        assert!(errors[0].message.contains("Not enough Gold"), "{errors:?}");
        let (next, events) = resolve_tactical_turn(&s, &orders(0, os));
        assert_eq!(find(&next, p1).unwrap().kind, UnitKind::StealthFighter);
        assert_eq!(find(&next, p2).unwrap().kind, UnitKind::Fighter);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
        // With Gold for both, both pass.
        s.player_mut(P0).gold = 2 * price;
        assert!(crate::validate_orders(&s, P0, &[up(p2), up(p1)]).is_empty());
        // With Silicon for only one, the second is refused for supply.
        let silicon = crate::resources::Resource::Silicon;
        let spare_base = city(&mut s, 0, hex(8, 8), 2);
        while crate::resources::spare(&s, P0, silicon, None) > 1 {
            base_new(&mut s, spare_base, UnitKind::StealthFighter);
        }
        let os = vec![up(p1), up(p2)];
        let errors = crate::validate_orders(&s, P0, &os);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].index, 1);
        let (next, _) = resolve_tactical_turn(&s, &orders(0, os));
        assert_eq!(find(&next, p1).unwrap().kind, UnitKind::StealthFighter);
        assert_eq!(find(&next, p2).unwrap().kind, UnitKind::Fighter);
    }

    #[test]
    fn an_aircraft_upgrade_sees_the_gold_a_city_purchase_spent() {
        let (mut s, a, p1, _p2, price) = two_fighters_ready_for_stealth();
        let buy = Order::BuyItem {
            city: a,
            item: ProductionItem::Unit {
                unit: UnitKind::Scout,
            },
        };
        s.player_mut(P0).gold = 10_000;
        let (buy_errors, after) = super::super::cities::validate_city_orders_projected(
            &s,
            P0,
            std::slice::from_ref(&buy),
        );
        assert!(buy_errors.is_empty(), "{buy_errors:?}");
        let cost = 10_000 - after.player(P0).gold;
        assert!(cost > 0);
        let up = Order::UpgradeAircraft {
            plane: p1,
            to: None,
        };
        s.player_mut(P0).gold = cost + price - 1;
        assert!(crate::validate_orders(&s, P0, std::slice::from_ref(&up)).is_empty());
        let errors = crate::validate_orders(&s, P0, &[buy.clone(), up.clone()]);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].index, 1);
        assert!(errors[0].message.contains("Not enough Gold"), "{errors:?}");
        let (next, _) = crate::resolve_turn(&s, &orders(0, vec![buy.clone(), up.clone()]));
        assert_eq!(find(&next, p1).unwrap().kind, UnitKind::Fighter);
        s.player_mut(P0).gold = cost + price;
        assert!(crate::validate_orders(&s, P0, &[buy, up]).is_empty());
    }

    #[test]
    fn spartan_aircraft_start_at_level_one_and_old_aircraft_load_without_xp() {
        let (mut s, a, _b) = arena();
        base_new(&mut s, a, UnitKind::Fighter);
        assert_eq!(s.aircraft[0].xp, 0);
        s.players[0].faction = Faction::Sparta;
        base_new(&mut s, a, UnitKind::Bomber);
        assert_eq!(s.aircraft[1].xp, promotion::threshold(1).unwrap());
        let old: Aircraft =
            serde_json::from_str(r#"{"id":7,"kind":"Bomber","city":1,"hp":60}"#).unwrap();
        assert_eq!((old.xp, old.hp), (0, 60));
        assert!(!serde_json::to_string(&old).unwrap().contains("xp"));
    }
}

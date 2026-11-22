//! The AI's air arm (era overhaul phase A2): when to base aircraft and where to strike.
//!
//! Aircraft are a hangar mechanic (`legatum_core::rules::air`): a Bomber strikes the best target
//! within eight hexes of its city every turn (no retaliation, so it always shoots when it can), and
//! the AI bases a Bomber where a war is within reach, plus a Fighter to cover its Bombers once it
//! has some. A Fighter either strikes (five hexes, units and cities) or stays on guard to intercept (one
//! order per aircraft): it guards while a rival at war that has reached the Flight era has a city
//! near enough for its Bombers to strike around this one, unless a strike is worth more than the
//! guard. Targets are scored by value: damage dealt in Production-equivalent HP (cost-weighted),
//! with a bonus for a kill and for chipping a city whose capture is in play; the forecast supplies
//! the numbers, so a Bomber prefers soft or dear targets over a wall of armour.

use legatum_core::{AttackMode, Order, PlayerView, TechId, UnitKind, data, forecast_attack_mode};
use std::collections::BTreeMap;

mod coverage;

/// What a Fighter on guard is worth while an enemy Bomber could strike near its city, in the same
/// units as a strike's value (damage times the target's cost, per hundred).
const GUARD_VALUE: i32 = 120;

/// Whether an enemy Bomber could strike a tile an interceptor based at `city` covers: a rival at
/// war (not barbarians) in the Flight era or later with a city within its strike range plus this
/// city's interception radius. Rival hangars are hidden, so the era stands in for "has Bombers".
fn bombers_likely(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    fighter: UnitKind,
) -> bool {
    // `PlayerView::era_of` is zero-based; `data::era_index` is one-based.
    let flight = data::era_index(TechId::Flight) - 1;
    let air_power = data::era_index(TechId::AirPower) - 1;
    let jet_engines = data::era_index(TechId::JetEngines) - 1;
    view.cities.iter().any(|c| {
        let era = view.era_of(c.owner);
        // Before Air Power a rival's only aircraft is the short-ranged Biplane.
        let bomber_range = if era >= legatum_core::eras::LAST_ERA {
            data::unit_stats(UnitKind::StealthBomber).range
        } else if era >= jet_engines {
            data::JET_BOMBER_RANGE
        } else if era >= air_power {
            data::BOMBER_RANGE
        } else {
            data::BIPLANE_STRIKE_RANGE
        };
        c.owner != view.me
            && !c.owner.is_barbarian()
            && view.at_war(view.me, c.owner)
            && era >= flight
            && c.pos.distance(city.pos) <= bomber_range + data::fighter_guard_radius(fighter)
    })
}

/// Bomber and Fighter orders for this turn: one strike per Bomber at its best target in range,
/// and per Fighter a strike or none (on guard).
#[cfg(test)]
pub fn air_orders(view: &PlayerView, fighters_strike: bool) -> Vec<Order> {
    air_orders_with_commitments(view, fighters_strike, true, &[])
}

/// Plan air fire against HP still standing after the submitted ground volley and earlier planes.
#[cfg(test)]
pub fn air_orders_with_commitments(
    view: &PlayerView,
    fighters_strike: bool,
    marginal_strikes: bool,
    ground_orders: &[Order],
) -> Vec<Order> {
    air_orders_with_policy(
        view,
        fighters_strike,
        marginal_strikes,
        ground_orders,
        0,
        false,
    )
}

#[cfg(test)]
pub(crate) fn air_orders_with_policy(
    view: &PlayerView,
    fighters_strike: bool,
    marginal_strikes: bool,
    ground_orders: &[Order],
    defense: i32,
    rebase: bool,
) -> Vec<Order> {
    air_orders_with_prizes(
        view,
        fighters_strike,
        marginal_strikes,
        ground_orders,
        defense,
        rebase,
        false,
    )
}

pub(crate) fn air_orders_with_prizes(
    view: &PlayerView,
    fighters_strike: bool,
    marginal_strikes: bool,
    ground_orders: &[Order],
    defense: i32,
    rebase: bool,
    prize_guard: bool,
) -> Vec<Order> {
    let mut orders = Vec::new();
    let mut committed: BTreeMap<legatum_core::Hex, i32> = BTreeMap::new();
    if marginal_strikes {
        for order in ground_orders {
            if let Order::RangedAttack { unit, target } = order
                && let Some(f) = forecast_attack_mode(view, *unit, *target, AttackMode::Ranged)
            {
                *committed.entry(*target).or_default() += f.expected_damage_to_defender;
            }
        }
    }
    for plane in view.aircraft.iter().filter(|a| a.owner == view.me) {
        if let Some(order) = hold_or_upgrade(view, plane) {
            orders.push(order);
            continue;
        }
        let stats = data::unit_stats(plane.kind);
        if stats.ranged <= 0 || (data::is_fighter(plane.kind) && !fighters_strike) {
            continue;
        }
        let Some(city) = view.cities.iter().find(|c| c.id == plane.city) else {
            continue;
        };
        let mut best: Option<(i32, legatum_core::Hex, i32)> = None;
        let mut consider = |pos: legatum_core::Hex, worth: i32, hp: i32| {
            if pos.distance(city.pos) > stats.range {
                return;
            }
            if let Some(f) = forecast_attack_mode(view, plane.id, pos, AttackMode::Ranged) {
                let remaining = if marginal_strikes {
                    (hp - committed.get(&pos).copied().unwrap_or_default()).max(0)
                } else {
                    hp
                };
                let damage = f.expected_damage_to_defender.min(remaining);
                let mut value = damage * worth / 100;
                if f.win_chance > 0 && (!marginal_strikes || (damage >= remaining && remaining > 0))
                {
                    value += f.win_chance as i32 * worth / 100;
                }
                if best.is_none_or(|(v, p, _)| (value, p) > (v, pos)) && value > 0 {
                    best = Some((value, pos, damage));
                }
            }
        };
        for u in view
            .units
            .iter()
            .filter(|u| u.military && view.at_war(view.me, u.owner) && !u.owner.is_barbarian())
            .filter(|u| {
                !marginal_strikes || !view.cities.iter().any(|c| c.pos == u.pos && c.hp > 0)
            })
        {
            consider(u.pos, crate::tuned(data::unit_stats(u.kind).cost), u.hp);
        }
        // A city is worth its HP in attacking it again: the assault that follows captures it.
        for c in view
            .cities
            .iter()
            .filter(|c| view.at_war(view.me, c.owner) && c.visible)
        {
            consider(c.pos, 120, c.hp);
        }
        if let Some((value, target, damage)) = best {
            let guard = if defense > 0 {
                coverage::guard_value(
                    view,
                    city,
                    Some(plane.id),
                    &ground_orders
                        .iter()
                        .chain(orders.iter())
                        .cloned()
                        .collect::<Vec<_>>(),
                ) * defense
                    / 100
            } else if bombers_likely(view, city, plane.kind) {
                GUARD_VALUE
            } else {
                0
            };
            let guard = guard.max(if prize_guard && data::is_fighter(plane.kind) {
                coverage::prize_guard_value(view, city, Some(plane.id), &orders)
            } else {
                0
            });
            let guarding = data::is_fighter(plane.kind) && value <= guard;
            if !guarding {
                orders.push(Order::AirStrike {
                    plane: plane.id,
                    target,
                });
                *committed.entry(target).or_default() += damage;
            }
        }
    }
    if rebase {
        coverage::rebase(
            view,
            &mut orders,
            ground_orders,
            defense,
            fighters_strike,
            prize_guard,
        );
    }
    orders
}

/// Upgrade a based aircraft when the stronger range and Stealth interception are worth the
/// Gold this turn. This is the hangar counterpart of the ground planner's hold_or_upgrade.
/// The new aircraft needs a spare of its resource only when it differs from the old one's (a
/// Biplane and a Fighter both use Oil; a Stealth aircraft Silicon).
fn hold_or_upgrade(view: &PlayerView, plane: &legatum_core::view::AircraftView) -> Option<Order> {
    let cost = plane.upgrade_cost?;
    let to = data::unit_stats(plane.kind).upgrades_to?;
    let need = legatum_core::resources::unit_need(to);
    let supplied = need == legatum_core::resources::unit_need(plane.kind)
        || need.is_none_or(|resource| {
            view.supply
                .iter()
                .any(|s| s.resource == resource && s.spare > 0)
        });
    if view.economy.gold >= cost
        && supplied
        && (data::unit_stats(to).range > data::unit_stats(plane.kind).range
            || (to == UnitKind::StealthFighter
                && view.cities.iter().any(|c| view.at_war(view.me, c.owner))))
    {
        Some(Order::UpgradeAircraft {
            plane: plane.id,
            to: None,
        })
    } else {
        None
    }
}

/// How much a city wants to base `unit` (an aircraft) now, in the production scorer's units.
/// Zero when the hangar is full or no war is within reach.
#[cfg(test)]
pub fn production_value(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    unit: UnitKind,
) -> i32 {
    production_value_with_policy(view, city, unit, 0, &[])
}

pub(crate) fn production_value_with_policy(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    unit: UnitKind,
    defense: i32,
    orders: &[Order],
) -> i32 {
    production_value_with_prizes(view, city, unit, defense, orders, false)
}

pub(crate) fn production_value_with_prizes(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    unit: UnitKind,
    defense: i32,
    orders: &[Order],
    prize_guard: bool,
) -> i32 {
    let me = view.me;
    let based = view
        .aircraft
        .iter()
        .filter(|a| a.owner == me && a.city == city.id)
        .count()
        + usize::from(city_builds_air(city));
    if based
        >= if view.research.known.contains(&TechId::StealthTechnology) {
            data::STEALTH_AIR_BASE_CAP
        } else {
            data::AIR_BASE_CAP
        }
    {
        return 0;
    }
    let own = |kind: UnitKind| {
        view.aircraft
            .iter()
            .filter(|a| a.owner == me && a.kind == kind)
            .count()
    };
    // One aircraft per city, so a small empire is not all hangar.
    let cities = view.cities.iter().filter(|c| c.owner == me).count();
    if own(UnitKind::Biplane)
        + own(UnitKind::Bomber)
        + own(UnitKind::JetBomber)
        + own(UnitKind::StealthBomber)
        + own(UnitKind::Fighter)
        + own(UnitKind::JetFighter)
        + own(UnitKind::StealthFighter)
        >= cities.max(1)
        && !(data::is_fighter(unit)
            && (defense > 0 && coverage::guard_value(view, city, None, orders) > 0
                || prize_guard && coverage::prize_guard_value(view, city, None, orders) > 0))
    {
        return 0;
    }
    let reach = data::unit_stats(unit).range + 1;
    let war_in_reach = view.units.iter().any(|u| {
        u.military
            && view.at_war(me, u.owner)
            && !u.owner.is_barbarian()
            && u.pos.distance(city.pos) <= reach
    }) || view
        .cities
        .iter()
        .any(|c| view.at_war(me, c.owner) && c.pos.distance(city.pos) <= reach);
    let legacy = match unit {
        kind if data::is_bomber(kind) && war_in_reach => 1800,
        // The Machine Age's only aircraft both strikes and guards: worth basing for a war in its
        // short reach, below a Bomber.
        UnitKind::Biplane if war_in_reach => 1200,
        kind if data::is_fighter(kind)
            && own(UnitKind::Bomber) + own(UnitKind::JetBomber) + own(UnitKind::StealthBomber)
                > own(UnitKind::Fighter)
                    + own(UnitKind::JetFighter)
                    + own(UnitKind::StealthFighter) =>
        {
            900
        }
        _ => 0,
    };
    if data::is_fighter(unit) && (defense > 0 || prize_guard) {
        // Same production units as the existing 1800/900 air priorities. Defense buys
        // repeated interception of an exposed asset, independently of owning Bombers.
        let ordinary = coverage::guard_value(view, city, None, orders) * 10 * defense / 100;
        let prize = if prize_guard {
            coverage::prize_guard_value(view, city, None, orders) * 10
        } else {
            0
        };
        legacy.max(ordinary).max(prize)
    } else {
        legacy
    }
}

fn city_builds_air(city: &legatum_core::view::CityView) -> bool {
    city.detail.as_ref().is_some_and(|d| {
        matches!(d.producing, Some(legatum_core::ProductionItem::Unit { unit }) if data::is_aircraft(unit))
    })
}

/// A committed aircraft has a job when it finishes; do not count its own queue as coverage.
pub(crate) fn useful_build(
    view: &PlayerView,
    city: &legatum_core::view::CityView,
    unit: UnitKind,
) -> bool {
    match unit {
        kind if data::is_fighter(kind) => {
            coverage::guard_value(view, city, Some(legatum_core::UnitId(u32::MAX)), &[]) > 0
        }
        kind if data::is_bomber(kind) => {
            let range = data::unit_stats(kind).range;
            view.cities
                .iter()
                .any(|c| view.at_war(view.me, c.owner) && c.pos.distance(city.pos) <= range)
                || view.units.iter().any(|u| {
                    u.military
                        && !u.owner.is_barbarian()
                        && view.at_war(view.me, u.owner)
                        && u.pos.distance(city.pos) <= range
                })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::diplomacy::{Relation, RelationState};
    use legatum_core::resources::SupplyView;
    use legatum_core::{Breakdown, Resource};
    use legatum_core::{Faction, PlayerId, UnitId};

    const RIVAL: PlayerId = PlayerId(2);

    #[test]
    fn seen_flight_at_war_prices_a_guard_for_an_exposed_capital() {
        let mut view = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(5, 8), 3)
            .city(2, RIVAL, off(12, 8), 3)
            .era_of(2, data::era_index(TechId::Flight))
            .build();
        let capital = view.cities.iter().find(|c| c.owner == ME).unwrap().clone();
        assert_eq!(coverage::prize_guard_value(&view, &capital, None, &[]), 0);
        view.players[2].era += 1;
        assert_eq!(
            coverage::prize_guard_value(&view, &capital, None, &[]),
            300,
            "a later public era proves Flight under the strict gate"
        );
        view.players[2].era -= 1;
        view.air_sightings.push(legatum_core::view::AirSighting {
            owner: RIVAL,
            kind: UnitKind::Bomber,
            city: legatum_core::CityId(2),
        });
        assert_eq!(coverage::prize_guard_value(&view, &capital, None, &[]), 300);
        assert!(
            production_value_with_prizes(&view, &capital, UnitKind::Fighter, 100, &[], true)
                > production_value_with_prizes(&view, &capital, UnitKind::Fighter, 100, &[], false)
        );
    }

    fn view_with_war() -> PlayerView {
        ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(5, 8), 3)
            .city(2, RIVAL, off(14, 8), 3)
            .unit(1, ME, UnitKind::Scout, off(8, 8))
            .unit(2, RIVAL, UnitKind::Spearman, off(9, 8))
            .aircraft(30, UnitKind::Bomber, 1)
            .build()
    }

    #[test]
    fn a_bomber_strikes_the_best_target_in_range_every_turn() {
        let v = view_with_war();
        let orders = air_orders(&v, true);
        assert_eq!(
            orders,
            vec![Order::AirStrike {
                plane: UnitId(30),
                target: off(9, 8)
            }]
        );
        // No strike with nothing in range.
        let mut quiet = v.clone();
        quiet.units = vec![].into();
        assert!(air_orders(&quiet, true).is_empty());
    }

    #[test]
    fn a_bomber_strikes_with_a_surprise_declaration() {
        let mut view = view_with_war();
        view.diplomacy.legacy = false;
        let against = view.team_of(RIVAL);
        view.diplomacy.relations = vec![Relation {
            a: view.team,
            b: against,
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 0,
            security: None,
            war_id: None,
            declarers: vec![],
        }];
        let declaration = [Order::DeclareWar {
            against,
            breach: false,
        }];
        let current = crate::scripted::projected_attack_view(&view, &declaration, true);
        assert_eq!(
            air_orders(&current, true),
            vec![Order::AirStrike {
                plane: UnitId(30),
                target: off(9, 8),
            }]
        );
        let control = crate::scripted::projected_attack_view(&view, &declaration, false);
        assert!(air_orders(&control, true).is_empty());
    }

    #[test]
    fn two_bombers_assign_only_one_strike_to_a_certain_kill() {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(5, 8), 3)
            .city(2, RIVAL, off(14, 8), 3)
            .unit(2, RIVAL, UnitKind::Tank, off(9, 8))
            .unit(3, RIVAL, UnitKind::Spearman, off(9, 9))
            .aircraft(30, UnitKind::Bomber, 1)
            .aircraft(31, UnitKind::Bomber, 1)
            .build();
        v.units.iter_mut().find(|u| u.id == UnitId(2)).unwrap().hp = 1;
        let orders = air_orders(&v, true);
        let targets: Vec<_> = orders
            .iter()
            .filter_map(|o| match o {
                Order::AirStrike { target, .. } => Some(*target),
                _ => None,
            })
            .collect();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets.iter().filter(|p| **p == off(9, 8)).count(), 1);
        assert!(targets.contains(&off(9, 9)));
    }

    #[test]
    fn bomber_uses_the_ground_volleys_remaining_target_hp() {
        let mut v = ViewBuilder::new(24, 16)
            .faction(Faction::India)
            .city(1, ME, off(5, 8), 3)
            .city(2, RIVAL, off(14, 8), 3)
            .unit(1, ME, UnitKind::Archer, off(7, 8))
            .unit(2, RIVAL, UnitKind::Tank, off(9, 8))
            .unit(3, RIVAL, UnitKind::Spearman, off(9, 9))
            .aircraft(30, UnitKind::Bomber, 1)
            .build();
        v.units.iter_mut().find(|u| u.id == UnitId(2)).unwrap().hp = 1;
        let ground = [Order::RangedAttack {
            unit: UnitId(1),
            target: off(9, 8),
        }];
        let air = air_orders_with_commitments(&v, true, true, &ground);
        assert_eq!(
            air,
            vec![Order::AirStrike {
                plane: UnitId(30),
                target: off(9, 9)
            }]
        );
    }

    #[test]
    fn a_fighter_strikes_a_near_target_unless_enemy_bombers_are_likely() {
        // The rival's Spearman stands `soldier_x - 5` hexes from the Fighter's city (a Fighter
        // strikes 3), its city `city_x - 5` (Bombers strike 5, a Fighter intercepts 4).
        let build = |spearman_x: i32, city_x: i32, rival_era: u8| {
            ViewBuilder::new(24, 16)
                .faction(Faction::India)
                .city(1, ME, off(5, 8), 3)
                .city(2, RIVAL, off(city_x, 8), 3)
                .unit(1, ME, UnitKind::Scout, off(7, 8))
                .unit(2, RIVAL, UnitKind::Spearman, off(spearman_x, 8))
                .aircraft(30, UnitKind::Fighter, 1)
                .era_of(2, rival_era)
                .build()
        };
        let flight = data::era_index(TechId::Flight);
        let strike = |x: i32| {
            vec![Order::AirStrike {
                plane: UnitId(30),
                target: off(x, 8),
            }]
        };
        assert_eq!(
            air_orders(&build(8, 16, 1), true),
            strike(8),
            "no Bombers likely: it strikes"
        );
        assert_eq!(
            air_orders(&build(8, 20, flight), true),
            strike(8),
            "a far rival does not keep it home"
        );
        assert!(
            air_orders(&build(8, 12, flight), true).is_empty(),
            "a Flight-era rival close enough for its Bombers: on guard"
        );
        assert_eq!(
            air_orders(&build(8, 12, 1), true),
            strike(8),
            "an earlier era has no Bombers"
        );
        assert!(
            air_orders(&build(11, 16, 1), true).is_empty(),
            "6 hexes is out of a Fighter's reach"
        );
        assert!(
            air_orders(&build(8, 16, 1), false).is_empty(),
            "with the gene off a Fighter never strikes"
        );
    }

    #[test]
    fn a_war_in_reach_makes_a_city_want_a_bomber_and_peace_does_not() {
        let v = view_with_war();
        let city = v
            .cities
            .iter()
            .find(|c| c.id == legatum_core::CityId(1))
            .unwrap();
        // The hangar already holds a plane and the empire has one city: one aircraft per city.
        assert_eq!(production_value(&v, city, UnitKind::Bomber), 0);
        let mut fresh = v.clone();
        fresh.aircraft.clear();
        let city = fresh
            .cities
            .iter()
            .find(|c| c.id == legatum_core::CityId(1))
            .unwrap();
        assert!(production_value(&fresh, city, UnitKind::Bomber) > 0);
        assert_eq!(
            production_value(&fresh, city, UnitKind::Fighter),
            0,
            "a Fighter covers Bombers"
        );
        // Without the legacy all-war rule and with no War relation, everyone is at peace.
        fresh.diplomacy.legacy = false;
        fresh.diplomacy.relations.clear();
        let city = fresh
            .cities
            .iter()
            .find(|c| c.id == legatum_core::CityId(1))
            .unwrap();
        assert_eq!(production_value(&fresh, city, UnitKind::Bomber), 0);
    }

    #[test]

    fn oil_demand_does_not_veto_a_valuable_bomber() {
        let mut view = view_with_war();
        view.aircraft.clear();
        view.supply = vec![SupplyView {
            resource: Resource::Oil,
            revealed_by: None,
            revealed: true,
            supplied: Breakdown {
                total: 3,
                parts: vec![],
            },
            deposits: vec![],
            used: Breakdown {
                total: 2,
                parts: vec![],
            },
            idle: vec!["Research Lab".into(), "Tank".into()],
            held: vec![],
            spare: 1,
        }];
        assert!(crate::supply_value::unmet(&view, Resource::Oil) > 0);
        let city = view
            .cities
            .iter()
            .find(|c| c.id == legatum_core::CityId(1))
            .unwrap();
        assert!(production_value(&view, city, UnitKind::Bomber) > 0);
    }
}

#[cfg(test)]
mod japan_regressions;

//! Public air-risk estimates: rival hangars remain hidden. An era and a known city are a
//! possibility of a strike, not evidence that a particular rival owns a Bomber.
use legatum_core::view::{AircraftView, CityView};
use legatum_core::{
    AttackMode, CityId, Hex, Order, PlayerView, ProductionItem, UnitId, UnitKind, data,
    forecast_attack_mode,
};

fn threatened(view: &PlayerView, target: Hex) -> bool {
    view.cities.iter().any(|base| {
        !base.owner.is_barbarian()
            && !view.same_team(view.me, base.owner)
            && !view.allied(view.me, base.owner)
            && view.era_of(base.owner)
                >= legatum_core::eras::era_of_tech(legatum_core::TechId::Flight)
            && base.pos.distance(target)
                <= if view.era_of(base.owner) >= legatum_core::eras::LAST_ERA {
                    data::unit_stats(UnitKind::StealthBomber).range
                } else if view.era_of(base.owner)
                    >= legatum_core::eras::era_of_tech(legatum_core::TechId::JetEngines)
                {
                    data::JET_BOMBER_RANGE
                } else {
                    data::BOMBER_RANGE
                }
    })
}

/// VP-bearing cities deserve a guard when Flight is known, rather than merely possible from
/// Modern-era entry. Rival techs and fogged hangars are private: a sighted plane proves Flight,
/// and entering a later era proves every Modern technology was researched under the strict gate.
fn flight_known(view: &PlayerView, owner: legatum_core::PlayerId) -> bool {
    view.era_of(owner) > legatum_core::eras::era_of_tech(legatum_core::TechId::Flight)
        || view.air_sightings.iter().any(|a| a.owner == owner)
        || view.air_contacts.iter().any(|a| a.owner == owner)
}

pub(super) fn prize_guard_value(
    view: &PlayerView,
    base: &CityView,
    plane: Option<UnitId>,
    orders: &[Order],
) -> i32 {
    let radius = plane
        .and_then(|id| view.aircraft.iter().find(|a| a.id == id).map(|a| a.kind))
        .map(data::fighter_guard_radius)
        .unwrap_or(data::FIGHTER_RANGE);
    view.cities
        .iter()
        .filter(|c| c.owner == view.me && (c.is_capital || !c.great_works.is_empty()))
        .filter(|c| base.pos.distance(c.pos) <= radius && !covered(view, c.pos, plane, orders))
        .filter(|c| {
            view.cities.iter().any(|enemy| {
                enemy.owner != view.me
                    && !enemy.owner.is_barbarian()
                    && view.at_war(view.me, enemy.owner)
                    && flight_known(view, enemy.owner)
                    && enemy.pos.distance(c.pos)
                        <= if view.era_of(enemy.owner) >= legatum_core::eras::LAST_ERA
                            || view.air_sightings.iter().any(|a| {
                                a.owner == enemy.owner && a.kind == UnitKind::StealthBomber
                            })
                        {
                            data::unit_stats(UnitKind::StealthBomber).range
                        } else if view.era_of(enemy.owner)
                            >= legatum_core::eras::era_of_tech(legatum_core::TechId::JetEngines)
                        {
                            data::JET_BOMBER_RANGE
                        } else {
                            data::BOMBER_RANGE
                        }
            })
        })
        .map(|c| 300 * (i32::from(c.is_capital) + c.great_works.len() as i32))
        .max()
        .unwrap_or(0)
}

fn covers(view: &PlayerView, city: CityId, target: Hex, kind: UnitKind) -> bool {
    view.cities
        .iter()
        .any(|c| c.id == city && c.pos.distance(target) <= data::fighter_guard_radius(kind))
}

fn covered(view: &PlayerView, target: Hex, except: Option<UnitId>, orders: &[Order]) -> bool {
    view.aircraft.iter().any(|a| {
        data::is_fighter(a.kind)
            && a.owner == view.me
            && except.is_none_or(|id| a.id < id)
            && !orders
                .iter()
                .any(|o| matches!(o, Order::AirStrike { plane, .. } if *plane == a.id))
            && covers(
                view,
                orders
                    .iter()
                    .find_map(|o| match o {
                        Order::Rebase { plane, to } if *plane == a.id => Some(*to),
                        _ => None,
                    })
                    .unwrap_or(a.city),
                target,
                a.kind,
            )
    }) || (except.is_none()
        && view.cities.iter().filter(|c| c.owner == view.me).any(|c| {
            let selected = orders
                .iter()
                .rev()
                .find_map(|o| match o {
                    Order::SetProduction { city, item } | Order::BuyItem { city, item }
                        if *city == c.id =>
                    {
                        Some(*item)
                    }
                    _ => None,
                })
                .or_else(|| c.detail.as_ref().and_then(|d| d.producing));
            selected.is_some_and(|item| matches!(item, ProductionItem::Unit { unit } if data::is_fighter(unit) && covers(view, c.id, target, unit)))
        }))
}

/// Maximum exposed asset value under this guard circle, in the strike scorer's units.
/// Production counts Fighters already in the pipeline; guard decisions count existing planes.
/// The single-interception rule makes summing every protected asset overstate one Fighter.
pub(super) fn guard_value(
    view: &PlayerView,
    base: &CityView,
    plane: Option<UnitId>,
    orders: &[Order],
) -> i32 {
    let radius = plane
        .and_then(|id| view.aircraft.iter().find(|a| a.id == id).map(|a| a.kind))
        .map(data::fighter_guard_radius)
        .unwrap_or_else(|| {
            if view
                .research
                .known
                .contains(&legatum_core::TechId::StealthTechnology)
            {
                data::fighter_guard_radius(UnitKind::StealthFighter)
            } else {
                data::FIGHTER_RANGE
            }
        });
    let exposed = |pos| {
        base.pos.distance(pos) <= radius
            && threatened(view, pos)
            && !covered(view, pos, plane, orders)
    };
    let cities = view
        .cities
        .iter()
        .filter(|c| c.owner == view.me && exposed(c.pos))
        .map(|c| 120 + c.population.max(0) * 20);
    let units = view
        .units
        .iter()
        .filter(|u| u.owner == view.me && u.military && exposed(u.pos))
        .map(|u| crate::tuned(data::unit_stats(u.kind).cost) * u.hp.clamp(0, 100) / 100);
    cities.chain(units).max().unwrap_or(0)
}

fn endangered(view: &PlayerView, base: &CityView) -> bool {
    (base.hp <= 50 || threatened(view, base.pos))
        && view.units.iter().any(|u| {
            u.military
                && view.at_war(view.me, u.owner)
                && u.pos.distance(base.pos) <= 1
                && data::unit_class(u.kind) != Some(legatum_core::UnitClass::Air)
                && data::unit_stats(u.kind).ranged == 0
        })
}

fn strike_value(view: &PlayerView, plane: &AircraftView, base: CityId) -> i32 {
    let mut projected = view.clone();
    projected
        .aircraft
        .iter_mut()
        .find(|a| a.id == plane.id)
        .unwrap()
        .city = base;
    let units = view
        .units
        .iter()
        .filter(|u| u.military && view.at_war(view.me, u.owner) && !u.owner.is_barbarian())
        .map(|u| (u.pos, crate::tuned(data::unit_stats(u.kind).cost), u.hp));
    let cities = view
        .cities
        .iter()
        .filter(|c| c.visible && view.at_war(view.me, c.owner))
        .map(|c| (c.pos, 120, c.hp));
    units
        .chain(cities)
        .filter_map(|(pos, worth, hp)| {
            forecast_attack_mode(&projected, plane.id, pos, AttackMode::Ranged)
                .map(|f| f.expected_damage_to_defender.min(hp.max(0)) * worth / 100)
        })
        .max()
        .unwrap_or(0)
}

/// One-hop moves only; project occupancy in the engine's aircraft-id order, including allies
/// in hangars. Validation does not credit departures, so never spend a vacated slot.
/// Require a strict improvement to avoid shuffling equally useful bases.
pub(super) fn rebase(
    view: &PlayerView,
    orders: &mut Vec<Order>,
    ground: &[Order],
    defense: i32,
    fighters_strike: bool,
    prize_guard: bool,
) {
    let mut occupancy = std::collections::BTreeMap::<CityId, usize>::new();
    for a in &view.aircraft {
        *occupancy.entry(a.city).or_default() += 1;
    }
    let mut planes: Vec<_> = view
        .aircraft
        .iter()
        .filter(|a| a.owner == view.me)
        .collect();
    planes.sort_by_key(|a| a.id);
    for plane in planes {
        let Some(current) = view.cities.iter().find(|c| c.id == plane.city) else {
            continue;
        };
        let evacuate = endangered(view, current);
        if !evacuate
            && orders
                .iter()
                .any(|o| matches!(o, Order::AirStrike { plane: id, .. } if *id == plane.id))
        {
            continue;
        }
        let combined: Vec<_> = ground.iter().chain(orders.iter()).cloned().collect();
        let value = |base: &CityView| {
            let guard = if data::is_fighter(plane.kind) {
                let ordinary = guard_value(view, base, Some(plane.id), &combined) * defense / 100;
                ordinary.max(if prize_guard {
                    prize_guard_value(view, base, Some(plane.id), &combined)
                } else {
                    0
                })
            } else {
                0
            };
            guard.max(if data::is_bomber(plane.kind) || fighters_strike {
                strike_value(view, plane, base.id)
            } else {
                0
            })
        };
        let now = value(current);
        let best = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me && c.id != current.id)
            .filter(|c| {
                occupancy.get(&c.id).copied().unwrap_or(0)
                    < if view
                        .research
                        .known
                        .contains(&legatum_core::TechId::StealthTechnology)
                    {
                        data::STEALTH_AIR_BASE_CAP
                    } else {
                        data::AIR_BASE_CAP
                    }
            })
            .filter(|c| !endangered(view, c))
            .map(|c| (value(c), c.id))
            .filter(|(v, _)| evacuate || *v > now)
            .max_by_key(|(v, id)| (*v, std::cmp::Reverse(*id)));
        if let Some((_, to)) = best {
            orders.retain(|o| !matches!(o, Order::AirStrike { plane: id, .. } if *id == plane.id));
            orders.push(Order::Rebase {
                plane: plane.id,
                to,
            });
            *occupancy.entry(to).or_default() += 1;
        }
    }
}

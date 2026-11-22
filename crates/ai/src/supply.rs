//! Strategic supply: every brain sees only production it has supply for, and its final order
//! list never starts more supply users than the view has spare. This is a rules requirement,
//! not tuning: a start, purchase or upgrade without spare supply is an invalid order.

use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    Faction, Order, PlayerView, ProductionItem, Resource, Terrain, combat, data, resources,
};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// The view with supply-blocked production options removed from own cities, so no brain
/// chooses a Pikeman it has no Iron for. Borrowed unchanged when nothing is blocked.
pub(crate) fn affordable(view: &PlayerView) -> Cow<'_, PlayerView> {
    let blocked = view
        .cities
        .iter()
        .filter_map(|c| c.detail.as_ref())
        .any(|d| d.options.iter().any(|o| o.resource_blocker.is_some()));
    if !blocked {
        return Cow::Borrowed(view);
    }
    let mut view = view.clone();
    for city in &mut view.cities {
        if let Some(d) = city.detail.as_mut() {
            d.options.retain(|o| o.resource_blocker.is_none());
        }
    }
    Cow::Owned(view)
}

fn city(view: &PlayerView, id: legatum_core::CityId) -> Option<&CityView> {
    view.cities
        .iter()
        .find(|c| c.id == id && c.owner == view.me)
}

/// What a unit's lack of supply costs it: `UNSUPPLIED_STRENGTH_PERCENT` of its base strength
/// (of its ranged strength when `shooting`), nothing while it is supplied. The view shows any
/// visible unit's supply status, so this reads public facts only.
fn unsupplied_loss(faction: Faction, u: &UnitView, shooting: bool) -> i32 {
    if u.unsupplied.is_none() {
        return 0;
    }
    let stats = data::unit_stats_for(faction, u.kind);
    resources::unsupplied_penalty(if shooting {
        stats.ranged
    } else {
        stats.strength
    })
}

/// `combat::unit_attack_total` less the penalty core applies to an unsupplied unit's attack.
pub(crate) fn attack_total(faction: Faction, u: &UnitView, hp: i32) -> i32 {
    combat::unit_attack_total(faction, u.kind, hp, u.level) - unsupplied_loss(faction, u, false)
}

/// `combat::unit_ranged_total` less the unsupplied penalty on its ranged strength.
pub(crate) fn ranged_total(faction: Faction, u: &UnitView, hp: i32) -> i32 {
    combat::unit_ranged_total(faction, u.kind, hp, u.level) - unsupplied_loss(faction, u, true)
}

/// `combat::unit_defense_total` less the unsupplied penalty (a defender is weakened too).
pub(crate) fn defense_total(
    faction: Faction,
    u: &UnitView,
    hp: i32,
    terrain: Terrain,
    fortify: i32,
) -> i32 {
    combat::unit_defense_total(faction, u.kind, hp, terrain, fortify, u.level)
        - unsupplied_loss(faction, u, false)
}

/// A unit's base strength (the larger of melee and ranged), after the unsupplied penalty and
/// scaled by health: the tactical weight of a visible unit.
pub(crate) fn power(u: &UnitView) -> i32 {
    let loss = |base: i32| {
        if u.unsupplied.is_some() {
            resources::unsupplied_penalty(base)
        } else {
            0
        }
    };
    (u.strength - loss(u.strength))
        .max(u.ranged - loss(u.ranged))
        .max(0)
        * u.hp.clamp(0, data::UNIT_HP)
        / data::UNIT_HP
}

/// Spare supply per resource, as the view shows it.
pub(crate) fn spare(view: &PlayerView) -> BTreeMap<Resource, i32> {
    view.supply.iter().map(|s| (s.resource, s.spare)).collect()
}

/// Drop orders that together would start more supply users than are spare, in resolution
/// order: unit and Siege upgrades first, then city orders in list order. Keeping a city's current build
/// is free; replacing it gives back the supply it held.
pub(crate) fn within_budget(view: &PlayerView, orders: Vec<Order>) -> Vec<Order> {
    if view.supply.is_empty() {
        return orders;
    }
    let mut spare = spare(view);
    let mut keep = vec![true; orders.len()];
    let take = |spare: &mut BTreeMap<Resource, i32>, need: Option<Resource>| {
        let Some(r) = need else { return true };
        let left = spare.entry(r).or_default();
        if *left < 1 {
            return false;
        }
        *left -= 1;
        true
    };
    for (i, order) in orders.iter().enumerate() {
        let need = match order {
            Order::UpgradeUnit { unit } => view
                .units
                .iter()
                .find(|u| u.id == *unit)
                .and_then(|u| u.upgrade_need),
            _ => continue,
        };
        keep[i] = take(&mut spare, need);
    }
    // Core reserves supply for unit upgrades before Siege purchases, regardless of their
    // position in the order list. The Siege pass must spend from the same remaining budget.
    for (i, order) in orders.iter().enumerate() {
        if matches!(order, Order::UpgradeSiege { .. }) {
            keep[i] = take(&mut spare, resources::siege_need());
        }
    }
    // Which build each city holds supply for, as earlier orders change it.
    let mut producing: BTreeMap<legatum_core::CityId, Option<ProductionItem>> = BTreeMap::new();
    for (i, order) in orders.iter().enumerate() {
        let (id, item, buy) = match order {
            Order::SetProduction { city, item } => (*city, Some(*item), false),
            Order::SetProductionQueue { city, queue } => (*city, queue.first().copied(), false),
            Order::BuyItem { city, item } => (*city, Some(*item), true),
            _ => continue,
        };
        let Some(current) = city(view, id)
            .and_then(|c| c.detail.as_ref())
            .map(|d| *producing.entry(id).or_insert(d.producing))
        else {
            continue;
        };
        // Buying the unit a city is training does not end that build: the city trains another,
        // which keeps its hold, so the bought unit needs supply of its own.
        if current == item && !(buy && matches!(item, Some(ProductionItem::Unit { .. }))) {
            continue;
        }
        if buy {
            keep[i] = take(&mut spare, item.and_then(resources::item_need));
            continue;
        }
        // Replacing a held build gives its supply back before the new one takes some.
        let freed = current.and_then(resources::item_need);
        if let Some(r) = freed {
            *spare.entry(r).or_default() += 1;
        }
        keep[i] = take(&mut spare, item.and_then(resources::item_need));
        if keep[i] {
            producing.insert(id, item);
        } else if let Some(r) = freed {
            *spare.entry(r).or_default() -= 1;
        }
    }
    orders
        .into_iter()
        .zip(keep)
        .filter_map(|(o, k)| k.then_some(o))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::view::SiegeOffer;
    use legatum_core::{Breakdown, BuildingKind, UnitId, UnitKind};

    #[test]
    fn siege_and_city_orders_share_iron_after_a_build_in_progress_holds_its_one() {
        let mut view = ViewBuilder::new(20, 20)
            .city(1, ME, off(4, 4), 3)
            .city(2, ME, off(8, 4), 3)
            .unit(1, ME, UnitKind::Spearman, off(4, 4))
            .build();
        view.supply.push(resources::SupplyView {
            resource: Resource::Iron,
            revealed_by: None,
            revealed: true,
            supplied: Breakdown {
                total: 6,
                parts: vec![],
            },
            deposits: vec![],
            used: Breakdown {
                total: 4,
                parts: vec![],
            },
            idle: vec![],
            held: vec!["University in Bellinzona".into()],
            spare: 1,
        });
        view.units
            .iter_mut()
            .find(|u| u.id == UnitId(1))
            .unwrap()
            .siege_offer = Some(SiegeOffer {
            price: Breakdown {
                total: 160,
                parts: vec![],
            },
            blocker: None,
            waits_for_gold: false,
        });
        let siege = Order::UpgradeSiege { unit: UnitId(1) };
        let university = Order::SetProduction {
            city: legatum_core::CityId(2),
            item: ProductionItem::Building {
                building: BuildingKind::University,
            },
        };
        assert_eq!(
            within_budget(&view, vec![siege.clone(), university]),
            vec![siege.clone()]
        );
        view.supply[0].spare = 0;
        assert!(within_budget(&view, vec![siege]).is_empty());
    }

    #[test]
    fn siege_purchase_cannot_spend_iron_reserved_by_a_unit_upgrade() {
        let mut view = ViewBuilder::new(12, 12)
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .unit(2, ME, UnitKind::Spearman, off(6, 5))
            .build();
        view.units[0].upgrade_need = Some(Resource::Iron);
        view.supply.push(resources::SupplyView {
            resource: Resource::Iron,
            revealed_by: None,
            revealed: true,
            supplied: Default::default(),
            deposits: Vec::new(),
            used: Default::default(),
            idle: Vec::new(),
            held: Vec::new(),
            spare: 1,
        });
        let orders = vec![
            Order::UpgradeSiege {
                unit: legatum_core::UnitId(2),
            },
            Order::UpgradeUnit {
                unit: legatum_core::UnitId(1),
            },
        ];
        assert_eq!(
            within_budget(&view, orders),
            vec![Order::UpgradeUnit {
                unit: legatum_core::UnitId(1)
            }]
        );
    }

    #[test]
    fn an_unsupplied_unit_is_weighed_a_quarter_weaker() {
        let mut v = ViewBuilder::new(20, 20)
            .unit(1, ENEMY, UnitKind::Pikeman, off(5, 5))
            .unit(2, ME, UnitKind::Spearman, off(6, 5))
            .build();
        let faction = v.faction_of(ENEMY);
        let supplied = v.units.iter().find(|u| u.id.0 == 1).unwrap().clone();
        let base = data::unit_stats_for(faction, UnitKind::Pikeman).strength;
        let mut short = supplied.clone();
        short.unsupplied = Some(Resource::Iron);
        assert_eq!(
            attack_total(faction, &supplied, 100) - attack_total(faction, &short, 100),
            resources::unsupplied_penalty(base)
        );
        assert_eq!(
            defense_total(faction, &supplied, 100, Terrain::Plains, 0)
                - defense_total(faction, &short, 100, Terrain::Plains, 0),
            resources::unsupplied_penalty(base)
        );
        assert!(power(&short) < power(&supplied));
        assert_eq!(
            power(&short),
            power(&supplied) - resources::unsupplied_penalty(base)
        );
        // A unit that needs nothing is never docked.
        let soldier = v.units.iter_mut().find(|u| u.id.0 == 2).unwrap();
        let before = power(soldier);
        soldier.unsupplied = None;
        assert_eq!(power(soldier), before);
    }
}

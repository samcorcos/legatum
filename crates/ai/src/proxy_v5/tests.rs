use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use legatum_core::{CityId, UnitId};

fn economy() -> PlayerView {
    ViewBuilder::new(24, 20)
        .known_techs(&[TechId::Irrigation, TechId::Currency, TechId::BronzeWorking])
        .city(1, ME, off(4, 5), 3)
        .city(2, ME, off(4, 11), 3)
        .city(3, ME, off(10, 11), 3)
        .city(4, ENEMY, off(11, 5), 2)
        .city(5, ENEMY, off(17, 5), 2)
        .unit(1, ME, UnitKind::Soldier, off(7, 5))
        .build()
}

#[test]
fn opens_with_settlers_without_waiting_for_escorts_or_spending_reserve() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 2)
        .gold(400)
        .build();
    let orders = HumanPlaybookBrain::default().orders(&view);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Settler
        },
    }));
    assert!(!orders.iter().any(|o| matches!(o, Order::BuyItem { .. })));
}

#[test]
fn buys_a_funded_burst_in_multiple_cities_and_preserves_economic_builds() {
    let mut view = economy();
    view.economy.gold = 700;
    for city in view.cities.iter_mut().filter(|c| c.owner == ME) {
        let d = city.detail.as_mut().unwrap();
        d.producing = Some(ProductionItem::Building {
            building: BuildingKind::Market,
        });
        for o in &mut d.options {
            if o.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Ranger,
                })
            {
                o.buy_cost = Some(175);
            }
        }
    }
    let orders = HumanPlaybookBrain::default().orders(&view);
    let buys: Vec<_> = orders
        .iter()
        .filter_map(|o| match o {
            Order::BuyItem { city, item } => Some((*city, *item)),
            _ => None,
        })
        .collect();
    assert!(buys.len() >= 2, "{orders:?}");
    assert!(buys.iter().all(|(_, item)| matches!(
        item,
        ProductionItem::Unit {
            unit: UnitKind::Ranger
        }
    )));
    let spent: i32 = buys
        .iter()
        .map(|(id, item)| {
            view.cities
                .iter()
                .find(|c| c.id == *id)
                .unwrap()
                .detail
                .as_ref()
                .unwrap()
                .options
                .iter()
                .find(|o| o.item == *item)
                .unwrap()
                .buy_cost
                .unwrap()
        })
        .sum();
    assert!(spent <= view.economy.gold - 30);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { .. }))
    );
}

#[test]
fn volley_then_melee_captures_instead_of_attacking_independently() {
    let target = off(8, 5);
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .city(2, ENEMY, target, 2)
        .unit(1, ME, UnitKind::Ranger, off(6, 5))
        .unit(2, ME, UnitKind::Soldier, off(7, 5))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.id == CityId(2))
        .unwrap()
        .hp = 5;
    let orders = HumanPlaybookBrain::default().orders(&view);
    assert!(orders.contains(&Order::RangedAttack {
        unit: UnitId(1),
        target
    }));
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(2), to, .. } if *to == target)),
        "{orders:?}"
    );
}

#[test]
fn takes_exposed_settler_without_a_suicidal_melee_attack() {
    let target = off(7, 5);
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .unit(1, ME, UnitKind::Soldier, off(6, 5))
        .unit(2, ENEMY, UnitKind::Settler, target)
        .build();
    let orders = HumanPlaybookBrain::default().orders(&view);
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(1), to, .. } if *to == target))
    );
}

#[test]
fn view_memory_is_repeatable_and_captures_trigger_consolidation() {
    let view = economy();
    let mut a = HumanPlaybookBrain::default();
    let mut b = HumanPlaybookBrain::default();
    assert_eq!(a.orders(&view), b.orders(&view));
    let mut captured = view.clone();
    let detail = captured.cities[0].detail.clone();
    captured.cities[3].owner = ME;
    captured.cities[3].detail = detail;
    captured.turn += 1;
    assert_eq!(a.orders(&captured), b.orders(&captured));
    assert_eq!(a.captured_cities.len(), 1);
    let orders = a.orders(&captured);
    assert_eq!(orders, a.orders(&captured));
    assert_eq!(a.captured_cities.len(), 1);
    captured.cities[4].owner = ME;
    captured.cities[4].detail = captured.cities[0].detail.clone();
    a.orders(&captured);
    assert_eq!(a.phase, Phase::Consolidate);
    assert_eq!(a.target, None);
    captured.finished = true;
    assert!(a.orders(&captured).is_empty());
}

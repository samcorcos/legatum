//! Exercise the production dispatch: complete OpeningBrain plans with primitives enabled.
use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid};
use legatum_core::{AiDifficulty, Faction, Hex, data};

fn h(q: i32, r: i32) -> Hex {
    Hex::new(q, r)
}
fn brain(opening: Opening) -> OpeningBrain {
    let b = OpeningBrain::new(
        ScriptedBrain::with_difficulty(Faction::Switzerland, 7, ME, AiDifficulty::Hard),
        OpeningPolicy::builtin(opening),
    );
    assert!(b.primitives_enabled);
    b
}
fn quote(v: &mut PlayerView) {
    for c in &mut v.cities {
        if let Some(d) = &mut c.detail {
            let d = &mut **d;
            d.options.retain(|o| match o.item {
                ProductionItem::Building { building: b } => {
                    !data::has_building(&d.buildings, b)
                        && data::building_predecessor(b)
                            .is_none_or(|p| data::has_building(&d.buildings, p))
                }
                _ => true,
            });
            for o in &mut d.options {
                o.buy_cost = Some(o.cost * 2);
            }
        }
    }
}
fn invests(orders: &[Order], item: ProductionItem) -> bool {
    orders.iter().any(|o| matches!(o, Order::BuyItem { item: i, .. } | Order::SetProduction { item: i, .. } if *i == item))
}

#[test]
fn economic_purchase_falls_through_to_the_next_market_tier_and_respects_gold_save() {
    let mut v = ViewBuilder::new(28, 20)
        .known_techs(&TechId::ALL)
        .city(1, ME, h(5, 5), 6)
        .gold(1000)
        .build();
    v.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .push(BuildingKind::Market);
    quote(&mut v);
    let mut b = brain(Opening::Economic);
    b.policy.numbers.settler_start = 300;
    let expected = Order::BuyItem {
        city: CityId(1),
        item: building(BuildingKind::Clearinghouse),
    };
    let orders = b.orders(&v);
    assert_valid(&v, &orders);
    assert!(orders.contains(&expected), "{orders:?}");
    b.policy.numbers.gold_save = 3000;
    assert!(!b.orders(&v).contains(&expected));
}

#[test]
fn unaffordable_primitive_settler_falls_through_to_an_affordable_market() {
    let mut v = ViewBuilder::new(28, 20)
        .known_techs(&[TechId::Currency])
        .city(1, ME, h(5, 5), 5)
        .gold(400)
        .build();
    v.turn = 20;
    // A costly second Settler (no first-Settler discount) stays out of reach while
    // the Market remains affordable at Standard treasury levels.
    for o in &mut v.cities[0].detail.as_mut().unwrap().options {
        if o.item
            == (ProductionItem::Unit {
                unit: UnitKind::Settler,
            })
        {
            o.cost = 250;
        }
    }
    quote(&mut v);
    let orders = brain(Opening::Economic).orders(&v);
    assert!(
        orders.contains(&Order::BuyItem {
            city: CityId(1),
            item: building(BuildingKind::Market),
        }),
        "{orders:?}"
    );
    assert_valid(&v, &orders);
}

#[test]
fn all_settler_paths_share_start_interval_target_and_queued_commitments() {
    let mut v = ViewBuilder::new(40, 28)
        .city(1, ME, h(5, 5), 5)
        .city(2, ME, h(5, 13), 5)
        .gold(2000)
        .build();
    quote(&mut v);
    let mut b = brain(Opening::GreedyExpand);
    b.policy.numbers.settler_start = 40;
    b.policy.numbers.settler_interval = 20;
    b.policy.numbers.settler_target = 3;
    v.turn = 59; // One founded expansion: next admission is at 40 + 20.
    assert!(!invests(&b.orders(&v), unit(UnitKind::Settler)));
    v.turn = 60;
    let orders = b.orders(&v);
    assert!(invests(&orders, unit(UnitKind::Settler)), "{orders:?}");
    assert_eq!(
        orders
            .iter()
            .filter(|o| matches!(o,
        Order::BuyItem { item, .. } | Order::SetProduction { item, .. }
        if *item == unit(UnitKind::Settler)))
            .count(),
        1,
        "same-turn buy consumes the cadence"
    );
    // Every queue entry counts, even two settlers behind a non-settler build.
    v.turn = 200;
    v.cities[1].detail.as_mut().unwrap().production_queue = vec![unit(UnitKind::Settler); 2];
    assert!(!invests(&b.orders(&v), unit(UnitKind::Settler)));
    v.cities[1]
        .detail
        .as_mut()
        .unwrap()
        .production_queue
        .clear();
    v = ViewBuilder { view: v }
        .unit(1, ME, UnitKind::Settler, h(9, 10))
        .unit(2, ME, UnitKind::Settler, h(13, 10))
        .build();
    assert!(!invests(&b.orders(&v), unit(UnitKind::Settler)));
}

#[test]
fn greedy_none_escort_does_not_rendezvous_with_a_settler() {
    let mut v = ViewBuilder::new(28, 20)
        .city(1, ME, h(5, 5), 3)
        .unit(1, ME, UnitKind::Settler, h(6, 5))
        .unit(2, ME, UnitKind::Soldier, h(5, 5))
        .gold(0)
        .build();
    // A high perceived threat would trigger the primitive even without an enemy.
    let mut b = brain(Opening::GreedyExpand);
    b.policy.primitives.escort_civilians_threat = 0;
    v.turn = 40;
    let orders = b.orders(&v);
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::MoveUnit {
                unit: UnitId(1),
                ..
            } | Order::MoveUnitPath {
                unit: UnitId(1),
                ..
            }
        )),
        "{orders:?}"
    );
    assert!(
        !orders.contains(&Order::MoveUnit {
            unit: UnitId(2),
            to: h(6, 5)
        }),
        "{orders:?}"
    );
}

#[test]
fn greedy_optional_budget_counts_queues_and_losses_but_never_blocks_emergency_production() {
    let mut v = ViewBuilder::new(28, 20)
        .known_techs(&[TechId::BronzeWorking])
        .city(1, ME, h(5, 5), 3)
        .city(2, ME, h(5, 12), 3)
        .unit(1, ME, UnitKind::Soldier, h(5, 12))
        .gold(1000)
        .build();
    v.units[0].created_turn = Some(2);
    v.cities[1].detail.as_mut().unwrap().production_queue = vec![unit(UnitKind::Ranger); 2];
    quote(&mut v);
    let mut b = brain(Opening::GreedyExpand);
    b.policy.numbers.settler_target = 0;
    for step in &mut b.policy.builds {
        step.count = 0;
    }
    let optional = b.orders(&v);
    for kind in [UnitKind::Soldier, UnitKind::Ranger] {
        assert!(!invests(&optional, unit(kind)), "{optional:?}");
    }
    v.units.clear(); // Previously completed investment still consumes the budget.
    v.turn += 1;
    let optional = b.orders(&v);
    for kind in [UnitKind::Soldier, UnitKind::Ranger] {
        assert!(!invests(&optional, unit(kind)), "{optional:?}");
    }
    v = ViewBuilder { view: v }
        .unit(99, ENEMY, UnitKind::Ranger, h(7, 5))
        .build();
    v.economy.gold = 0;
    v.turn += 1;
    let emergency = b.orders(&v);
    assert!(invests(&emergency, unit(UnitKind::Ranger)), "{emergency:?}");
}

#[test]
fn snowball_schedules_both_settlers_before_commerce_and_builds_colony_industry() {
    let mut v = ViewBuilder::new(32, 24)
        .known_techs(&[TechId::BronzeWorking, TechId::Currency])
        .city(1, ME, h(5, 5), 5)
        .city(2, ME, h(5, 13), 3)
        .unit(1, ME, UnitKind::Settler, h(5, 5))
        .gold(0)
        .build();
    v.turn = 70;
    v.units[0].created_turn = Some(2);
    v.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .push(BuildingKind::Workshop);
    let mut b = brain(Opening::Snowball);
    b.orders(&v); // Observe the first trained settler.
    v.units.clear(); // It is now the existing first colony.
    v.turn += 1;
    let orders = b.orders(&v);
    assert!(
        orders.contains(&Order::SetProduction {
            city: CityId(1),
            item: unit(UnitKind::Settler)
        }),
        "{orders:?}"
    );
    assert!(
        orders.contains(&Order::SetProduction {
            city: CityId(2),
            item: building(BuildingKind::Workshop)
        }),
        "{orders:?}"
    );
}

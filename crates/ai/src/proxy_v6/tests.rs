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
        .unit(1, ME, UnitKind::Spearman, off(7, 5))
        .build()
}

#[test]
fn opens_with_settlers_without_waiting_for_escorts_or_spending_reserve() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 2)
        .gold(400)
        .build();
    let orders = HumanPlaybookV6Brain::default().orders(&view);
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
    view.economy.gold = 6000;
    for city in view.cities.iter_mut().filter(|c| c.owner == ME) {
        let d = city.detail.as_mut().unwrap();
        d.producing = Some(ProductionItem::Building {
            building: BuildingKind::Market,
        });
        for o in &mut d.options {
            if matches!(
                o.item,
                ProductionItem::Unit {
                    unit: UnitKind::Archer | UnitKind::Spearman
                }
            ) {
                o.buy_cost = Some(350);
            }
        }
    }
    let orders = HumanPlaybookV6Brain::default().orders(&view);
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
            unit: UnitKind::Archer | UnitKind::Spearman
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
        .unit(1, ME, UnitKind::Archer, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(7, 5))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.id == CityId(2))
        .unwrap()
        .hp = 5;
    let orders = HumanPlaybookV6Brain::default().orders(&view);
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
        .unit(1, ME, UnitKind::Spearman, off(6, 5))
        .unit(2, ENEMY, UnitKind::Settler, target)
        .build();
    let orders = HumanPlaybookV6Brain::default().orders(&view);
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(1), to, .. } if *to == target))
    );
}

#[test]
fn view_memory_is_repeatable_and_two_captures_do_not_end_a_campaign() {
    let view = economy();
    let mut a = HumanPlaybookV6Brain::default();
    let mut b = HumanPlaybookV6Brain::default();
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
    assert_ne!(a.phase, Phase::Consolidate);
    captured.turn = 230;
    a.orders(&captured);
    assert_eq!(a.phase, Phase::Consolidate);
    assert_eq!(a.target, None);
    captured.finished = true;
    assert!(a.orders(&captured).is_empty());
}

#[test]
fn researches_siege_as_soon_as_the_campaign_prerequisites_are_complete() {
    let view = ViewBuilder::new(24, 20)
        .known_techs(&[
            TechId::Irrigation,
            TechId::BronzeWorking,
            TechId::Currency,
            TechId::Writing,
            TechId::Masonry,
            TechId::Feudalism,
            TechId::IronWorking,
        ])
        .city(1, ME, off(4, 5), 3)
        .build();
    assert!(
        HumanPlaybookV6Brain::default()
            .orders(&view)
            .contains(&Order::SetResearch {
                tech: TechId::Mathematics,
            })
    );
}

#[test]
fn idles_instead_of_building_unwanted_scouts() {
    let mut view = economy();
    for c in view.cities.iter_mut().filter(|c| c.owner == ME) {
        c.detail.as_mut().unwrap().options.retain(|o| {
            matches!(
                o.item,
                ProductionItem::Unit {
                    unit: UnitKind::Scout
                }
            )
        });
    }
    let orders = HumanPlaybookV6Brain::default().orders(&view);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { .. })),
        "{orders:?}"
    );
}

fn conquering() -> HumanPlaybookV6Brain {
    HumanPlaybookV6Brain {
        phase: Phase::Conquer,
        ..Default::default()
    }
}

/// The same view as a two-player lobby: me and the one rival.
fn duel(mut view: PlayerView) -> PlayerView {
    let players: Vec<_> = view
        .players
        .iter()
        .filter(|p| p.id == ME || p.id == ENEMY)
        .cloned()
        .collect();
    view.players = players.into();
    view
}

#[test]
fn duel_campaign_never_consolidates_into_peace_with_its_only_rival() {
    let mut view = duel(economy());
    view.turn = 230 + 20;
    let mut brain = HumanPlaybookV6Brain::default();
    brain.orders(&view);
    assert_ne!(brain.phase, Phase::Consolidate);
    // The eight-player schedule is unchanged.
    let mut crowded = economy();
    crowded.turn = view.turn;
    let mut brain = HumanPlaybookV6Brain::default();
    brain.orders(&crowded);
    assert_eq!(brain.phase, Phase::Consolidate);
}

#[test]
fn duel_marches_on_a_rival_city_beyond_the_crowded_lobby_range() {
    let view = ViewBuilder::new(48, 20)
        .city(1, ME, off(4, 5), 3)
        .city(2, ENEMY, off(36, 5), 3)
        .unit(1, ME, UnitKind::Spearman, off(5, 5))
        .build();
    let mut brain = conquering();
    brain.orders(&duel(view.clone()));
    assert_eq!(brain.target, Some(CityId(2)));
    let mut crowded = conquering();
    crowded.orders(&view);
    assert_eq!(crowded.target, None);
}

#[test]
fn duel_sends_one_infantryman_to_find_a_rival_nobody_has_seen() {
    let view = ViewBuilder::new(30, 16)
        .city(1, ME, off(4, 5), 3)
        .unit(1, ME, UnitKind::Spearman, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(6, 6))
        .unit(3, ME, UnitKind::Archer, off(6, 7))
        .unexplored_from_col(12)
        .build();
    // How far from the city each unit ends its move.
    let reach = |orders: &[Order], id| {
        orders.iter().find_map(|o| match o {
            Order::MoveUnitPath { unit, to, .. } if *unit == UnitId(id) => {
                Some(to.distance(off(4, 5)))
            }
            _ => None,
        })
    };
    let mut brain = conquering();
    let orders = brain.orders(&duel(view.clone()));
    let seeker = reach(&orders, 1).expect("the lowest infantry id explores");
    assert!(seeker >= 3, "{orders:?}");
    assert!(reach(&orders, 2).is_none_or(|d| d < seeker), "{orders:?}");
    assert!(reach(&orders, 3).is_none_or(|d| d < seeker), "{orders:?}");
    // A crowded lobby keeps its army home when it has no target.
    let mut crowded = conquering();
    let orders = crowded.orders(&view);
    assert!(reach(&orders, 1).is_none_or(|d| d < seeker), "{orders:?}");
}

#[test]
fn duel_replaces_a_lost_scout_while_the_rival_is_unknown() {
    let mut view = ViewBuilder::new(30, 16)
        .city(1, ME, off(4, 5), 3)
        .unit(1, ME, UnitKind::Spearman, off(6, 5))
        .build();
    for c in view.cities.iter_mut().filter(|c| c.owner == ME) {
        c.detail.as_mut().unwrap().options.retain(|o| {
            matches!(
                o.item,
                ProductionItem::Unit {
                    unit: UnitKind::Scout
                }
            )
        });
    }
    let scout = Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Scout,
        },
    };
    let mut brain = conquering();
    assert!(brain.orders(&duel(view.clone())).contains(&scout));
    let mut crowded = conquering();
    assert!(!crowded.orders(&view).contains(&scout));
}

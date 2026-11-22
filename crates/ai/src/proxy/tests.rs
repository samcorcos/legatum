use super::ProxyV2Brain as ProxyBrain;
use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use legatum_core::{CityId, UnitId};

#[test]
fn proxy_is_stateless_deterministic_and_handles_finished_games() {
    let view = ViewBuilder::new(20, 16)
        .city(1, ME, off(4, 5), 3)
        .city(2, ENEMY, off(10, 5), 2)
        .unit(1, ME, UnitKind::Archer, off(8, 5))
        .unit(2, ME, UnitKind::Spearman, off(9, 4))
        .unit(3, ME, UnitKind::Settler, off(4, 5))
        .build();
    let mut proxy = ProxyBrain;
    let expected = proxy.orders(&view);
    assert_eq!(expected, proxy.orders(&view));
    assert_eq!(expected, ProxyBrain.orders(&view));
    let mut finished = view.clone();
    finished.finished = true;
    assert!(proxy.orders(&finished).is_empty());
    assert_eq!(expected, proxy.orders(&view));
}

#[test]
fn ranged_harasses_garrison_without_melee_suicide() {
    let city = off(8, 5);
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .city(2, ENEMY, city, 2)
        .unit(1, ME, UnitKind::Archer, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(7, 5))
        .unit(3, ENEMY, UnitKind::Spearman, city)
        .build();
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(2))
        .unwrap()
        .hp = 25;
    let orders = ProxyBrain.orders(&view);
    assert!(orders.contains(&Order::RangedAttack {
        unit: UnitId(1),
        target: city
    }));
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit: UnitId(2), to, .. } if *to == city))
    );
}

#[test]
fn one_melee_occupies_a_defeated_city_while_archers_stop_firing() {
    let city = off(8, 5);
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .city(2, ENEMY, city, 2)
        .unit(1, ME, UnitKind::Archer, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(7, 5))
        .unit(3, ME, UnitKind::Spearman, off(8, 4))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.id == CityId(2))
        .unwrap()
        .hp = 0;
    let orders = ProxyBrain.orders(&view);
    assert_eq!(
        orders
            .iter()
            .filter(|o| matches!(o, Order::MoveUnitPath { to, .. } if *to == city))
            .count(),
        1
    );
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { target, .. } if *target == city))
    );
}

#[test]
fn remembered_city_is_scouted_but_never_shot_through_fog() {
    let mut view = ViewBuilder::new(20, 16)
        .city(1, ME, off(3, 5), 3)
        .city(2, ENEMY, off(8, 5), 2)
        .unit(1, ME, UnitKind::Archer, off(6, 5))
        .build();
    view.cities
        .iter_mut()
        .find(|c| c.id == CityId(2))
        .unwrap()
        .visible = false;
    assert!(
        !ProxyBrain
            .orders(&view)
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { .. }))
    );
}

#[test]
fn opening_fields_ranged_support_and_funds_science() {
    let view = ViewBuilder::new(24, 20)
        .city(1, ME, off(8, 8), 3)
        .unit(90, ME, UnitKind::Spearman, off(9, 8))
        .build();
    let orders = ProxyBrain.orders(&view);
    assert!(orders.contains(&Order::SetResearch {
        tech: TechId::Irrigation
    }));
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Archer
        }
    }));
    let view = ViewBuilder::new(24, 20)
        .known_techs(&[TechId::BronzeWorking, TechId::Writing])
        .city(1, ME, off(4, 8), 3)
        .city(2, ME, off(10, 8), 3)
        .city(3, ME, off(16, 8), 3)
        .unit(1, ME, UnitKind::Settler, off(4, 8))
        .unit(2, ME, UnitKind::Settler, off(10, 8))
        .unit(3, ME, UnitKind::Settler, off(16, 8))
        .unit(90, ME, UnitKind::Spearman, off(5, 8))
        .unit(91, ME, UnitKind::Spearman, off(6, 8))
        .unit(92, ME, UnitKind::Archer, off(7, 8))
        .unit(93, ME, UnitKind::Archer, off(8, 8))
        .build();
    // After the opening, replacing losses must not starve every city's science base.
    let mut view = view;
    view.turn = 100;
    let detail = view
        .cities
        .iter_mut()
        .find(|c| c.id == CityId(1))
        .unwrap()
        .detail
        .as_mut()
        .unwrap();
    // Workshop built; the Granary (buildable from turn 1) is out of the picture too.
    detail.buildings.push(BuildingKind::Workshop);
    detail.options.retain(|o| {
        !matches!(
            o.item,
            ProductionItem::Building {
                building: BuildingKind::Workshop | BuildingKind::Granary,
            }
        )
    });
    let orders = ProxyBrain.orders(&view);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Building {
            building: BuildingKind::Library
        }
    }));
}

#[test]
fn disconnected_frontier_is_not_scored_as_a_finite_distance() {
    let mut builder = ViewBuilder::new(20, 16)
        .unexplored_from_col(18)
        .city(1, ME, off(4, 5), 2)
        .unit(1, ME, UnitKind::Scout, off(6, 5));
    for row in 0..16 {
        builder = builder.terrain(off(10, row), Terrain::Mountains);
    }
    let view = builder.build();
    assert!(
        ProxyBrain
            .orders(&view)
            .contains(&Order::SkipUnit { unit: UnitId(1) })
    );
}

#[test]
fn v2_settlement_does_not_inherit_new_scripted_border_inference() {
    let site = off(10, 8);
    let hidden = off(13, 8);
    let mut view = ViewBuilder::new(24, 16)
        .city(1, ME, off(4, 8), 3)
        .unit(1, ME, UnitKind::Settler, site)
        .owner(off(12, 8), ENEMY)
        .build();
    let i = view.index(hidden).unwrap();
    view.tiles[i].visible = false;
    let i = view.index(site).unwrap();
    assert!(Ctx::new(&view).site_score[i].is_none());
    let legacy = v2_context(&view);
    // Thirty-seven plains in radius three, one unavailable foreign tile:
    // 36 * (food 1*4 + production 1*3) - 8.
    assert_eq!(legacy.site_score[i], Some(244));
    assert_eq!(PROXY_V2_VERSION, 2);
}

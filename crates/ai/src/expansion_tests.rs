//! Expansion and economy decisions (playtest 8.4: the AIs out-expand nobody).

use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::{AiParams, Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Faction, Order, PlayerView, ProductionItem, UnitKind};

const SETTLER: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Settler,
};

/// Seat ME's capital builds a Settler `percent` done; an enemy Spearman stands `gap` hexes away.
fn settler_city(percent: i32, gap: i32) -> PlayerView {
    let home = off(6, 8);
    let mut v = ViewBuilder::new(28, 16)
        .faction(Faction::Celts)
        .city(1, ME, home, 3)
        .unit(9, ENEMY, UnitKind::Spearman, off(6 + gap, 8))
        .producing(1, SETTLER, 0)
        .build();
    let city = v.cities.iter_mut().find(|c| c.owner == ME).unwrap();
    let detail = city.detail.as_mut().unwrap();
    let option = detail
        .options
        .iter_mut()
        .find(|o| o.item == SETTLER)
        .unwrap();
    option.production_banked = option.cost * percent / 100;
    v
}

fn orders(v: &PlayerView, keep_settler: i32) -> Vec<Order> {
    let mut params = AiParams::builtin();
    params.decisions.keep_settler = keep_settler;
    params.decisions.army_door_production = 0;
    ScriptedBrain::with_params(Faction::Celts, 7, ME, AiDifficulty::Hard, &params).orders(v)
}

fn swapped(orders: &[Order]) -> bool {
    orders.iter().any(
        |o| matches!(o, Order::SetProduction { city, item } if city.0 == 1 && *item != SETTLER),
    )
}

#[test]
fn a_settler_well_under_way_survives_a_distant_threat() {
    let v = settler_city(60, 3);
    let kept = orders(&v, 40);
    assert!(!swapped(&kept), "60% built: finish it: {kept:?}");
    assert!(
        swapped(&orders(&v, 0)),
        "with the gene off the Settler is swapped"
    );
}

#[test]
fn a_settler_barely_started_is_still_swapped() {
    assert!(swapped(&orders(&settler_city(20, 3), 40)));
}

#[test]
fn a_settler_is_given_up_when_the_enemy_is_at_the_gates() {
    assert!(swapped(&orders(&settler_city(60, 2), 40)));
}

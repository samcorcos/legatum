//! Genes `capture_hold` and `strike_kill` (`army_commit.rs`).
use crate::fixture::{ENEMY, ME, ViewBuilder, planner};
use legatum_core::{Hex, Order, PlayerView, UnitId, UnitKind};

const CAPTURED: Hex = Hex { q: 8, r: 8 };
const HOLDER: u32 = 1;

/// Our Pikeman stands in a city we just took (resistance 3, 0 HP); the enemy's next city is
/// four hexes east. `enemies` adds enemy units.
fn captured(enemies: &[(u32, UnitKind, Hex)]) -> PlayerView {
    let mut b = ViewBuilder::new(24, 18)
        .city(1, ME, CAPTURED, 3)
        .city(2, ME, Hex::new(2, 8), 4)
        .city(3, ENEMY, Hex::new(12, 8), 3)
        .unit(HOLDER, ME, UnitKind::Pikeman, CAPTURED);
    for (id, kind, at) in enemies {
        b = b.unit(*id, ENEMY, *kind, *at);
    }
    let mut view = b.build();
    view.cities[0].hp = 0;
    view.cities[0].is_capital = false;
    view.cities[0].detail.as_mut().unwrap().resistance = 3;
    view
}

fn holds(view: &PlayerView, gene: i32) -> (bool, bool) {
    let mut p = planner(view, 7);
    p.decisions.capture_hold = gene;
    let army: Vec<_> = p.ctx.own_units.clone();
    p.hold_captures(&army);
    let early = p.orders.contains(&Order::SkipUnit {
        unit: UnitId(HOLDER),
    });
    p.hold_captures_late(&army);
    let late = p.orders.contains(&Order::SkipUnit {
        unit: UnitId(HOLDER),
    });
    (early, late)
}

#[test]
fn a_capture_no_visible_attack_can_retake_leaves_its_holder_to_the_attack_passes() {
    let view = captured(&[]);
    // Off: the holder is held before any attack pass runs.
    assert_eq!(holds(&view, 0), (true, true));
    // On: it is held only if the attack passes leave it free.
    assert_eq!(holds(&view, 1), (false, true));
}

#[test]
fn a_capture_an_adjacent_soldier_would_retake_empty_keeps_its_holder_first() {
    let view = captured(&[(20, UnitKind::Spearman, Hex::new(9, 8))]);
    assert_eq!(holds(&view, 1), (true, true));
}

#[test]
fn the_holder_goes_when_its_own_capture_this_turn_is_worth_more() {
    // An empty enemy city at 0 HP beside the holder (population 6) against our population-3
    // capture that an adjacent Spearman would retake.
    let mut view = captured(&[(20, UnitKind::Spearman, Hex::new(7, 9))]);
    let i = view.cities.iter().position(|c| c.id.0 == 3).unwrap();
    view.cities[i].pos = Hex::new(9, 8);
    view.cities[i].population = 6;
    view.cities[i].hp = 0;
    assert!(!holds(&view, 1).0);
    // A population-1 prize is worth less than the city it would cost.
    view.cities[i].population = 1;
    assert!(holds(&view, 1).0);
}

#[test]
fn a_strike_unit_beside_a_wounded_enemy_kills_it_instead_of_staging() {
    let wounded = ViewBuilder::new(24, 18)
        .city(1, ME, Hex::new(2, 8), 4)
        .unit(1, ME, UnitKind::Pikeman, Hex::new(8, 8))
        .unit_hp(20, ENEMY, UnitKind::Spearman, Hex::new(9, 8), 20)
        .build();
    let p = planner(&wounded, 7);
    assert!(p.kill_beside(p.ctx.own_units[0]));
    // A healthy enemy of the same strength is not a forecast kill.
    let healthy = ViewBuilder::new(24, 18)
        .city(1, ME, Hex::new(2, 8), 4)
        .unit(1, ME, UnitKind::Pikeman, Hex::new(8, 8))
        .unit(20, ENEMY, UnitKind::Pikeman, Hex::new(9, 8))
        .build();
    let p = planner(&healthy, 7);
    assert!(!p.kill_beside(p.ctx.own_units[0]));
}

/// Gene `shot_sweep`: an Archer an earlier pass sent home stands two hexes from an enemy Spearman.
fn ranger_sent_home(enemy_hp: i32, guards: &[Hex]) -> (PlayerView, Order) {
    let mut b = ViewBuilder::new(24, 18)
        .city(1, ME, Hex::new(2, 8), 4)
        .unit(1, ME, UnitKind::Archer, Hex::new(8, 8))
        .unit_hp(20, ENEMY, UnitKind::Spearman, Hex::new(10, 8), enemy_hp);
    for (i, h) in guards.iter().enumerate() {
        b = b.unit(30 + i as u32, ENEMY, UnitKind::Pikeman, *h);
    }
    let home = Order::MoveUnit {
        unit: UnitId(1),
        to: Hex::new(3, 8),
    };
    (b.build(), home)
}

fn swept(view: &PlayerView, home: &Order, gene: i32) -> Vec<Order> {
    let mut p = planner(view, 7);
    p.difficulty = legatum_core::AiDifficulty::Hard;
    p.decisions.shot_sweep = gene;
    p.unit_order(UnitId(1), home.clone());
    p.shot_sweep();
    p.orders
}

#[test]
fn a_shooter_sent_home_kills_the_wounded_enemy_in_range_instead() {
    let (view, home) = ranger_sent_home(10, &[]);
    assert_eq!(swept(&view, &home, 0), vec![home.clone()]);
    assert_eq!(
        swept(&view, &home, 1),
        vec![Order::RangedAttack {
            unit: UnitId(1),
            target: Hex::new(10, 8),
        }]
    );
}

#[test]
fn a_shot_that_would_not_kill_leaves_the_move() {
    let (view, home) = ranger_sent_home(100, &[]);
    assert_eq!(swept(&view, &home, 1), vec![home]);
}

#[test]
fn a_shooter_the_rest_of_the_enemy_would_kill_if_it_stayed_keeps_its_move() {
    // Three healthy Pikemen beside it would finish an Archer that stays to shoot.
    let guards = [Hex::new(9, 7), Hex::new(7, 8), Hex::new(8, 9)];
    let (view, home) = ranger_sent_home(10, &guards);
    assert_eq!(swept(&view, &home, 1), vec![home]);
}

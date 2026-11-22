//! Genes `focus_fire` and `gather_hold` (`focus.rs`).
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid};
use crate::scripted::Planner;
use legatum_core::view::UnitView;
use legatum_core::{AiDifficulty, Hex, Order, PlayerView, UnitKind};

fn planner(v: &PlayerView, focus: i32, gather: i32) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.focus_fire = focus;
    p.decisions.gather_hold = gather;
    p
}

fn army(v: &PlayerView) -> Vec<&UnitView> {
    v.units
        .iter()
        .filter(|u| u.owner == ME && u.military)
        .collect()
}

fn shots(orders: &[Order]) -> Vec<Hex> {
    orders
        .iter()
        .filter_map(|o| match o {
            Order::RangedAttack { target, .. } => Some(*target),
            _ => None,
        })
        .collect()
}

/// Two Archers within range of two wounded Spearmen: one shot wounds, two kill.
fn two_targets() -> PlayerView {
    ViewBuilder::new(20, 12)
        .city(1, ME, Hex::new(2, 6), 3)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(10, ME, UnitKind::Archer, Hex::new(8, 5))
        .unit(11, ME, UnitKind::Archer, Hex::new(8, 7))
        .unit_hp(20, ENEMY, UnitKind::Spearman, Hex::new(10, 5), 45)
        .unit_hp(21, ENEMY, UnitKind::Spearman, Hex::new(10, 7), 45)
        .build()
}

#[test]
fn focus_fire_off_orders_nothing() {
    let v = two_targets();
    let a = army(&v);
    let mut p = planner(&v, 0, 0);
    p.focus_fire(&a);
    assert!(p.orders.is_empty(), "{:?}", p.orders);
}

#[test]
fn focus_fire_puts_both_shots_on_one_target_when_together_they_kill() {
    let v = two_targets();
    let a = army(&v);
    let mut p = planner(&v, 100, 0);
    p.focus_fire(&a);
    let s = shots(&p.orders);
    assert_eq!(s.len(), 2, "{:?}", p.orders);
    assert_eq!(s[0], s[1], "shots spread: {:?}", p.orders);
    assert_valid(&v, &p.orders);
    // Deterministic: the same view gives the same orders.
    let mut q = planner(&v, 100, 0);
    q.focus_fire(&a);
    assert_eq!(p.orders, q.orders);
}

#[test]
fn focus_fire_leaves_a_hopeless_target_to_the_ordinary_passes() {
    // One Archer against a full-HP Spearman: no kill is possible, so no plan pays.
    let v = ViewBuilder::new(20, 12)
        .city(1, ME, Hex::new(2, 6), 3)
        .city(2, ENEMY, Hex::new(18, 6), 3)
        .unit(10, ME, UnitKind::Archer, Hex::new(8, 5))
        .unit(20, ENEMY, UnitKind::Spearman, Hex::new(10, 5))
        .build();
    let a = army(&v);
    let mut p = planner(&v, 100, 0);
    p.focus_fire(&a);
    assert!(p.orders.is_empty(), "{:?}", p.orders);
}

#[test]
fn a_lone_straggler_stops_short_of_the_enemy_but_a_group_goes_on() {
    // A wounded Spearman walking alone next to three enemy Spearmen stops; with two friends
    // ending beside it the fire is shared and it goes on.
    let enemies = [
        (Hex::new(12, 5), 30),
        (Hex::new(12, 6), 31),
        (Hex::new(12, 7), 32),
    ];
    let build = |friends: bool| {
        let mut b = ViewBuilder::new(20, 12)
            .city(1, ME, Hex::new(2, 6), 3)
            .city(2, ENEMY, Hex::new(18, 6), 3)
            .unit_hp(10, ME, UnitKind::Spearman, Hex::new(8, 6), 70);
        if friends {
            b = b.unit(11, ME, UnitKind::Spearman, Hex::new(10, 5)).unit(
                12,
                ME,
                UnitKind::Spearman,
                Hex::new(10, 7),
            );
        }
        for (h, id) in enemies {
            b = b.unit(id, ENEMY, UnitKind::Spearman, h);
        }
        b.build()
    };
    for friends in [false, true] {
        let v = build(friends);
        let a = army(&v);
        let mut p = planner(&v, 0, 50);
        let u = a.iter().find(|u| u.id.0 == 10).unwrap();
        p.unit_order(
            u.id,
            Order::MoveUnit {
                unit: u.id,
                to: Hex::new(10, 6),
            },
        );
        p.hold_stragglers(&a);
        let held = !p.orders.iter().any(
            |o| matches!(o, Order::MoveUnit { unit, to } if unit.0 == 10 && *to == Hex::new(10, 6)),
        );
        assert_eq!(held, !friends, "friends {friends}: {:?}", p.orders);
    }
}

//! The Mongol Keshig, a horse archer with a charge and a bow (`dual.rs`): our own picks the more
//! valuable attack each turn, and rivals read it as both a melee and a ranged threat.

use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use crate::scripted::Planner;
use legatum_core::{
    AttackMode, Faction, Hex, Order, PlayerView, Terrain, UnitId, UnitKind, combat, data,
};

const KESHIG: UnitId = UnitId(1);
const AT: Hex = Hex::new(8, 8);

/// A Mongol view: our Keshig at `AT`, a home city to the west, and the enemy's units.
fn mongol(enemy: impl FnOnce(ViewBuilder) -> ViewBuilder) -> PlayerView {
    let b = ViewBuilder::new(24, 16)
        .faction(Faction::Mongols)
        .city(1, ME, Hex::new(2, 8), 2)
        .unit(KESHIG.0, ME, data::KESHIG_KIND, AT);
    enemy(b).build()
}

fn plan(view: &PlayerView, dual_choice: bool) -> Vec<Order> {
    let mut p = Planner::new(
        view,
        Knobs::for_faction(Faction::Mongols),
        Faction::Mongols,
        7,
    );
    p.decisions.dual_attack_choice = i32::from(dual_choice);
    p.decisions.capitals = 0;
    p.plan_military();
    p.orders
}

/// The Keshig's own choice when it is free (`dual_fire`, after the plan's moves): the shot or
/// the charge, by value.
fn choose(view: &PlayerView, dual_choice: bool) -> Vec<Order> {
    let mut p = Planner::new(
        view,
        Knobs::for_faction(Faction::Mongols),
        Faction::Mongols,
        7,
    );
    p.decisions.dual_attack_choice = i32::from(dual_choice);
    let army: Vec<&legatum_core::view::UnitView> = p.ctx.own_units.clone();
    p.dual_fire(&army);
    p.orders
}

fn keshig_order(orders: &[Order]) -> Option<Order> {
    orders
        .iter()
        .find(|o| match o {
            Order::MoveUnit { unit, .. }
            | Order::MoveUnitPath { unit, .. }
            | Order::RangedAttack { unit, .. }
            | Order::SkipUnit { unit }
            | Order::UpgradeUnit { unit } => *unit == KESHIG,
            _ => false,
        })
        .cloned()
}

#[test]
fn the_keshig_is_a_mounted_unit_with_a_bow() {
    let view = mongol(|b| b);
    let k = view.units.iter().find(|u| u.id == KESHIG).unwrap();
    assert!(k.ranged > 0 && k.range > 0);
    assert!(crate::composition::dual(k.kind, k.ranged));
    assert!(crate::composition::charges(k.kind));
    assert!(!crate::composition::shooter_only(k.kind, k.ranged));
    // Every other unit is one or the other, as before.
    for kind in [
        UnitKind::Ranger,
        UnitKind::Soldier,
        UnitKind::Horseman,
        UnitKind::Cataphract,
    ] {
        let s = data::unit_stats_for(Faction::Switzerland, kind);
        assert!(!crate::composition::dual(kind, s.ranged), "{kind:?}");
        assert_eq!(crate::composition::charges(kind), s.ranged == 0, "{kind:?}");
    }
}

#[test]
fn an_own_keshig_charges_an_exposed_archer() {
    let archer = Hex::new(10, 8);
    let view = mongol(|b| b.unit(50, ENEMY, UnitKind::Ranger, archer));
    // Within its bow's range too, but the charge (mounted against ranged) is worth more.
    assert_eq!(
        keshig_order(&choose(&view, true)),
        Some(Order::MoveUnit {
            unit: KESHIG,
            to: archer
        })
    );
    // Gene off: the bow always wins.
    assert_eq!(
        keshig_order(&choose(&view, false)),
        Some(Order::RangedAttack {
            unit: KESHIG,
            target: archer
        })
    );
}

#[test]
fn an_own_keshig_shoots_a_fortified_spearman_instead_of_charging_it() {
    let spear = Hex::new(10, 8);
    let mut view = mongol(|b| b.unit(50, ENEMY, UnitKind::Soldier, spear));
    view.units
        .iter_mut()
        .find(|u| u.id == UnitId(50))
        .unwrap()
        .fortify = data::FORTIFY_MAX;
    assert_eq!(
        keshig_order(&choose(&view, true)),
        Some(Order::RangedAttack {
            unit: KESHIG,
            target: spear
        })
    );
}

#[test]
fn an_own_keshig_rides_into_a_fallen_city_rather_than_shoot() {
    let city = Hex::new(9, 8);
    let archer = Hex::new(6, 8);
    let mut view = mongol(|b| {
        b.city(9, ENEMY, city, 2)
            .unit(50, ENEMY, UnitKind::Ranger, archer)
    });
    view.cities.iter_mut().find(|c| c.pos == city).unwrap().hp = 0;
    assert_eq!(
        keshig_order(&plan(&view, true)),
        Some(Order::MoveUnit {
            unit: KESHIG,
            to: city
        })
    );
}

#[test]
fn a_keshig_gives_up_its_bow_only_for_a_better_unit() {
    let k = data::unit_stats_for(Faction::Mongols, data::KESHIG_KIND);
    let c = data::unit_stats_for(Faction::Mongols, UnitKind::Knight);
    let keshig = crate::composition::worth(data::KESHIG_KIND, k.strength, k.ranged);
    let knight = crate::composition::worth(UnitKind::Knight, c.strength, c.ranged);
    // Worth is the stronger attack plus half the other: the bow counts, but as an option.
    assert_eq!(
        keshig,
        k.strength.max(k.ranged) + k.strength.min(k.ranged) / 2
    );
    assert_eq!(knight, c.strength);
}

/// A Swiss rival facing a Mongol Keshig.
fn rival(build: impl FnOnce(ViewBuilder) -> ViewBuilder) -> PlayerView {
    let mut b = ViewBuilder::new(24, 16);
    b.view.players[ENEMY.0 as usize].faction = Faction::Mongols;
    build(b).build()
}

fn threat_of(view: &PlayerView, enemy: UnitId) -> i32 {
    let p = Planner::new(
        view,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        7,
    );
    let e = p.ctx.enemy_units.iter().find(|e| e.id == enemy).unwrap();
    p.enemy_threat(e, e.hp)
}

#[test]
fn a_rival_counts_an_adjacent_keshig_as_a_melee_threat_to_its_city() {
    let city = Hex::new(8, 8);
    let mut view = rival(|b| {
        b.city(1, ME, city, 2)
            .unit(50, ENEMY, data::KESHIG_KIND, Hex::new(9, 8))
    });
    view.cities[0].hp = 10;
    // It can ride in and take the empty, broken city (its bow alone never could).
    assert!(threat_of(&view, UnitId(50)) >= data::CITY_HP);
    let c = view.cities[0].clone();
    let attacks = crate::purchase_defense::city_attacks(&view, &c, &|_, _| true);
    assert_eq!(attacks, vec![(1, UnitId(50), AttackMode::Melee)]);
}

#[test]
fn a_rival_reads_a_keshig_shot_without_the_spear_bonus() {
    let view = rival(|b| {
        b.unit(1, ME, UnitKind::Soldier, Hex::new(8, 8)).unit(
            50,
            ENEMY,
            data::KESHIG_KIND,
            Hex::new(10, 8),
        )
    });
    // Arrows fight as the ranged class: a spear line gets no bonus against them.
    assert_eq!(
        crate::tactics::fight_bonuses(data::KESHIG_KIND, UnitKind::Soldier, true),
        (0, 0)
    );
    assert_eq!(
        crate::tactics::fight_bonuses(data::KESHIG_KIND, UnitKind::Soldier, false),
        (
            0,
            combat::class_bonus_strength(data::unit_stats(UnitKind::Soldier).strength)
        )
    );
    let shot = crate::tactics::expected_damage(
        combat::unit_ranged_total(Faction::Mongols, data::KESHIG_KIND, 100, 0),
        combat::unit_defense_total(
            Faction::Switzerland,
            UnitKind::Soldier,
            100,
            Terrain::Plains,
            0,
            0,
        ),
        100,
    );
    assert!(shot > 0);
    // Since the stronger riders (2026-10-31) the charge into the spears outweighs the arrows; the
    // threat is the worse of the two, each with its own class lines.
    let charge = crate::tactics::expected_damage(
        combat::unit_attack_total(Faction::Mongols, data::KESHIG_KIND, 100, 0),
        combat::unit_defense_total(
            Faction::Switzerland,
            UnitKind::Soldier,
            100,
            Terrain::Plains,
            0,
            0,
        ) + combat::class_bonus_strength(data::unit_stats(UnitKind::Soldier).strength),
        100,
    );
    assert_eq!(threat_of(&view, UnitId(50)), shot.max(charge));
}

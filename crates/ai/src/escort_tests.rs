use super::*;
use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use legatum_core::{PlayerView, UnitId};

const HOME: Hex = Hex::new(3, 8);
const SITE: Hex = Hex::new(7, 8);
const ESCORT: UnitId = UnitId(1);

fn view() -> ViewBuilder {
    ViewBuilder::new(24, 16)
        .faction(Faction::Mongols)
        .city(1, ME, HOME, 2)
        .unit(1, ME, UnitKind::Spearman, SITE)
        .unit(2, ME, UnitKind::Spearman, HOME)
}

fn planner(view: &PlayerView) -> Planner<'_> {
    let mut planner = Planner::new(
        view,
        Knobs::for_faction(Faction::Mongols),
        Faction::Mongols,
        7,
    );
    // This test exercises the escort reservation with the idle-upgrade policy (a plain
    // affordable upgrade), not the modernization refit policy that is now on by default.
    planner.decisions.modernization_upgrade = 0;
    planner.decisions.modernization_return = 0;
    // Exercise military phase priority even when civilian planning requests this site.
    planner.settle_targets.push(SITE);
    planner.escort_sites.push(SITE);
    planner
}

#[test]
fn holding_or_upgrading_an_arrived_escort_reserves_its_position() {
    for gold in [29, 30] {
        let mut view = view().gold(gold).build();
        view.units[0].upgrade_cost = Some(30);
        let mut p = planner(&view);
        p.plan_military();
        assert!(p.mil_dest.contains(&SITE));
        assert!(p.has_acted(ESCORT));
        assert!(p.orders.contains(&if gold == 30 {
            Order::UpgradeUnit { unit: ESCORT }
        } else {
            Order::SkipUnit { unit: ESCORT }
        }));
        assert_eq!(p.gold_left, if gold == 30 { 0 } else { 29 });
    }
}

#[test]
fn tactical_retreat_preempts_an_active_escort_assignment() {
    let mut view = view()
        .unit(50, ENEMY, UnitKind::Pikeman, Hex::new(9, 8))
        .build();
    view.units[0].hp = 20;
    let mut p = planner(&view);
    p.plan_military();
    assert!(p.retreats.contains(&ESCORT));
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, to } if *unit == ESCORT && *to != SITE))
    );
    assert!(!p.mil_dest.contains(&SITE));
}

#[test]
fn immediate_combat_preempts_an_active_escort_assignment() {
    let target = Hex::new(8, 8);
    let view = view()
        .unit_hp(50, ENEMY, UnitKind::Scout, target, 1)
        .build();
    let mut p = planner(&view);
    p.plan_military();
    assert!(p.orders.contains(&Order::MoveUnit {
        unit: ESCORT,
        to: target
    }));
    assert!(!p.orders.contains(&Order::SkipUnit { unit: ESCORT }));
}

#[test]
fn city_garrison_preempts_an_active_escort_assignment() {
    let mut view = view().build();
    view.units.retain(|u| u.id == ESCORT);
    let mut p = planner(&view);
    p.plan_military();
    assert!(p.orders.contains(&Order::MoveUnit {
        unit: ESCORT,
        to: HOME
    }));
    assert!(!p.mil_dest.contains(&SITE));
}

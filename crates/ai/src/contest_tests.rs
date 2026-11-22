//! Gene `contest_hold` on hand-built views: a rival Celts holds its target on Raise the
//! Warbands, and our units can reach one of its camps.

use super::*;
use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
use legatum_core::view::{BarbarianCampView, PlayerView, TeamStanding, VictoryHold};
use legatum_core::{AiDifficulty, Order, PlayerId, UnitId, UnitKind};

const CAMP: (i32, i32) = (12, 6);

fn standing(team: u8, vp: u32, target: u32, held: Option<u32>) -> TeamStanding {
    TeamStanding {
        team: TeamId(team),
        members: if team == 0 {
            vec![ME, PlayerId(1)]
        } else {
            vec![PlayerId(team)]
        },
        vp,
        target,
        victory_hold: held.map(|turns_held| VictoryHold {
            turns_held,
            turns_needed: 10,
        }),
    }
}

/// Our city at (3,6) and a Celtic city at (16,6) whose land holds a known camp at `CAMP`; the
/// Celts score Raise the Warbands (2 VP) and stand at `vp` of a 10 target, holding `held` turns.
fn holding_celts(vp: u32, held: Option<u32>) -> ViewBuilder {
    let mut b = ViewBuilder::new(30, 14)
        .faction(Faction::Switzerland)
        .city(1, ME, off(3, 6), 4)
        .city(2, ENEMY, off(16, 6), 5);
    for h in off(16, 6).within(4) {
        b = b.owner(h, ENEMY);
    }
    b.view.players[2].faction = Faction::Celts;
    b.view.players[2].cities = 1;
    b.view.players[0].cities = 1;
    b.view.players[2].vp = vp;
    b.view.players[2].vp_sources = vec![(VpSource::FactionGoal { of: ENEMY }, 2)];
    b.view.standings = vec![standing(0, 4, 10, None), standing(2, vp, 10, held)].into();
    b.view.barbarian_camps = vec![BarbarianCampView {
        pos: off(CAMP.0, CAMP.1),
        visible: true,
    }];
    b.view.turn = 150;
    b
}

fn planner(v: &PlayerView, gene: i32) -> Planner<'_> {
    let mut p = Planner::new(
        v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        7,
    );
    p.difficulty = AiDifficulty::Hard;
    p.decisions.contest_hold = gene;
    p
}

fn run(v: &PlayerView, gene: i32) -> Vec<Order> {
    let mut p = planner(v, gene);
    let army: Vec<&UnitView> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military && u.kind != UnitKind::Scout)
        .collect();
    p.contest_vp_sources(&army);
    p.orders
}

fn ordered(orders: &[Order], id: u32) -> Option<&Order> {
    orders.iter().find(|o| match o {
        Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::RangedAttack { unit, .. }
        | Order::SkipUnit { unit } => *unit == UnitId(id),
        _ => false,
    })
}

fn heads_to_camp(orders: &[Order], id: u32) -> bool {
    let camp = off(CAMP.0, CAMP.1);
    matches!(ordered(orders, id), Some(Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. })
        if to.distance(camp) <= 2)
        || matches!(ordered(orders, id), Some(Order::RangedAttack { target, .. }) if *target == camp)
}

#[test]
fn a_free_unit_walks_into_an_empty_goal_camp_during_the_hold() {
    let v = holding_celts(10, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    let orders = run(&v, 100);
    assert!(
        matches!(ordered(&orders, 70), Some(Order::MoveUnit { to, .. }) if *to == off(CAMP.0, CAMP.1)),
        "{orders:?}"
    );
    assert_valid(&v, &orders);
    assert!(run(&v, 0).is_empty(), "gene off: nothing");
}

#[test]
fn a_camp_that_would_not_break_the_hold_is_left_alone() {
    // Two VP of margin: losing the goal's 2 VP still leaves the Celts on target.
    let v = holding_celts(12, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    assert!(run(&v, 100).is_empty());
    // Below the target and not holding: the Celts would call again before it matters.
    let v = holding_celts(8, None)
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    assert!(run(&v, 100).is_empty());
}

#[test]
fn at_peace_or_against_a_teammate_nothing_is_sent() {
    let mut v = holding_celts(10, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    v.players[2].team = TeamId(0);
    assert!(run(&v, 100).is_empty());
}

#[test]
fn the_gain_grows_as_the_countdown_runs_and_is_zero_past_it() {
    let early = holding_celts(10, Some(1))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    let late = holding_celts(10, Some(8))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .build();
    let s = |v: &PlayerView| planner(v, 100).denial_sources()[0].clone();
    let g_early = planner(&early, 100).denial_gain(&s(&early), 1);
    let g_late = planner(&late, 100).denial_gain(&s(&late), 1);
    assert!(g_late > g_early && g_early > 0, "{g_early} {g_late}");
    // Two turns left: a force three turns out arrives after the win.
    assert_eq!(planner(&late, 100).denial_gain(&s(&late), 3), 0);
}

#[test]
fn a_guarded_camp_gets_a_force_that_can_kill_the_guard() {
    // A barbarian Spearman holds the camp. One Spearman alone is a coin flip it may lose; with a
    // Archer and a second Spearman the guard dies and a melee unit steps in.
    let v = holding_celts(10, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(9, 6))
        .unit(71, ME, UnitKind::Spearman, off(9, 7))
        .unit(72, ME, UnitKind::Archer, off(9, 5))
        .unit(
            90,
            PlayerId::BARBARIAN,
            UnitKind::Spearman,
            off(CAMP.0, CAMP.1),
        )
        .build();
    let orders = run(&v, 100);
    let sent = [70, 71, 72]
        .iter()
        .filter(|id| heads_to_camp(&orders, **id))
        .count();
    assert!(sent >= 2, "the force is more than one unit: {orders:?}");
    assert_valid(&v, &orders);
}

#[test]
fn the_holder_of_a_city_about_to_fall_stays_home() {
    // Our only unit holds our broken city (0 HP) with a wounded Celtic Spearman beside it:
    // empty, the city falls to it; held, it does not.
    let mut v = holding_celts(10, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(3, 6))
        .unit_hp(80, ENEMY, UnitKind::Spearman, off(4, 6), 40)
        .build();
    v.cities[0].hp = 0;
    v.barbarian_camps[0].pos = off(6, 6);
    for h in off(6, 6).within(1) {
        if let Some(i) = v.index(h) {
            v.tiles[i].owner = Some(ENEMY);
        }
    }
    let p = planner(&v, 100);
    let holder = p.ctx.own_units.iter().find(|u| u.id == UnitId(70)).unwrap();
    assert!(
        p.contest_cost(holder, TeamId(2)) > 0,
        "the holder's job has a price"
    );
    let orders = run(&v, 100);
    assert!(
        !matches!(ordered(&orders, 70), Some(Order::MoveUnit { to, .. }) if *to == off(6, 6)),
        "{orders:?}"
    );
}

#[test]
fn the_brain_orders_stay_valid_with_the_gene() {
    let v = holding_celts(10, Some(4))
        .unit(70, ME, UnitKind::Spearman, off(8, 6))
        .unit(71, ME, UnitKind::Archer, off(4, 6))
        .build();
    let mut params = crate::AiParams::default();
    params.decisions.contest_hold = 100;
    let mut brain =
        crate::ScriptedBrain::with_params(Faction::Switzerland, 7, ME, AiDifficulty::Hard, &params);
    let orders = crate::Brain::plan(&mut brain, &v);
    assert_valid(&v, &orders);
}

#[test]
fn a_guard_that_cannot_die_before_the_hold_completes_is_not_attacked() {
    // One turn left and a lone Spearman against a strong guard: the fight outlasts the hold.
    let v = holding_celts(10, Some(9))
        .unit(70, ME, UnitKind::Spearman, off(11, 6))
        .unit(
            90,
            PlayerId::BARBARIAN,
            UnitKind::Musketman,
            off(CAMP.0, CAMP.1),
        )
        .build();
    assert!(run(&v, 100).is_empty(), "{:?}", run(&v, 100));
    // Early in the countdown an Archer alone still goes: the guard never heals or replies to a
    // shot, so it wears down over several turns and the Archer walks in.
    let v = holding_celts(10, Some(1))
        .unit(71, ME, UnitKind::Archer, off(10, 6))
        .unit(
            90,
            PlayerId::BARBARIAN,
            UnitKind::Musketman,
            off(CAMP.0, CAMP.1),
        )
        .build();
    let p = planner(&v, 100);
    let army: Vec<&UnitView> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    let s = p.denial_sources()[0].clone();
    let (force, turns) = p.denial_force(&army, &s).expect("a force");
    assert_eq!(force.len(), 1);
    assert!(
        turns > 2,
        "several turns of shots and one to walk in: {turns}"
    );
}

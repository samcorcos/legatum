//! Hold-to-win behaviour on hand-built views: every AI treats a team counting down to victory
//! as the top threat, and a holding AI defends what keeps its VP.

use crate::fixture::*;
use crate::scripted::Planner;
use crate::{Brain, Knobs};
use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
use legatum_core::view::{TeamStanding, VictoryHold};
use legatum_core::*;

/// A standing; `held` turns of a 10-turn Standard hold when holding.
fn standing(team: u8, vp: u32, target: u32, held: Option<u32>) -> TeamStanding {
    TeamStanding {
        team: TeamId(team),
        members: if team == 0 {
            vec![ME, MATE]
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

fn relation(team: u8, state: RelationState) -> Relation {
    Relation {
        a: TeamId(0),
        b: TeamId(team),
        state,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: (state == RelationState::War).then_some(1),
        declarers: if state == RelationState::War {
            vec![TeamId(team)]
        } else {
            vec![]
        },
    }
}

fn action(order: Order) -> DiplomaticAction {
    DiplomaticAction {
        order,
        effective_turn: 11,
        cost: 0,
        upkeep: 0,
        betrayal: false,
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        blocked_reason: None,
    }
}

fn orders(view: &PlayerView, difficulty: AiDifficulty) -> Vec<Order> {
    let mut b = brain(view, 7, difficulty);
    let orders = b.orders(view);
    assert_valid(view, &orders);
    orders
}

#[test]
fn the_coalition_target_is_the_holder_even_on_a_vp_tie() {
    let mut v = ViewBuilder::new(24, 16).city(1, ME, off(3, 8), 3).build();
    v.diplomacy.legacy = false;
    v.standings = vec![
        standing(0, 4, 17, None),
        standing(2, 10, 10, Some(2)),
        standing(3, 10, 10, None),
    ]
    .into();
    assert_eq!(
        legatum_core::diplomacy::coalition::target_view(&v),
        Some(TeamId(2))
    );
    let p = Planner::new(
        &v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        1,
    );
    assert_eq!(p.diplomatic_target(), Some(TeamId(2)));
    // Without the hold the tie has no target.
    v.standings[1].victory_hold = None;
    assert_eq!(legatum_core::diplomacy::coalition::target_view(&v), None);
}

/// Our city and army at home, a rival city 8 hexes away, real diplomacy (not the legacy
/// all-war fixture).
fn rival_next_door(state: RelationState) -> PlayerView {
    let mut v = ViewBuilder::new(28, 16)
        .city(1, ME, off(3, 8), 3)
        .unit(1, ME, UnitKind::Spearman, off(3, 8))
        .unit(2, ME, UnitKind::Spearman, off(4, 8))
        .unit(3, ME, UnitKind::Archer, off(4, 9))
        .city(9, ENEMY, off(11, 8), 3)
        .unit(50, ENEMY, UnitKind::Spearman, off(11, 8))
        .build();
    v.diplomacy.legacy = false;
    v.diplomacy.relations.push(relation(2, state));
    v.diplomacy
        .relations
        .push(relation(3, RelationState::Peace));
    v.standings = vec![
        standing(0, 4, 17, None),
        standing(2, 6, 10, None),
        standing(3, 2, 10, None),
    ]
    .into();
    v
}

#[test]
fn every_ai_declares_on_a_holder_and_prices_peace_against_survival() {
    let mut v = rival_next_door(RelationState::Peace);
    v.diplomacy.actions.push(action(Order::DeclareWar {
        against: TeamId(2),
        breach: false,
    }));
    let declares = |v: &PlayerView| {
        orders(v, AiDifficulty::Normal).contains(&Order::DeclareWar {
            against: TeamId(2),
            breach: false,
        })
    };
    assert!(!declares(&v), "a rival at 6 of 10 VP is no reason for war");
    v.standings[1].vp = 10;
    v.standings[1].victory_hold = Some(VictoryHold {
        turns_held: 1,
        turns_needed: 10,
    });
    assert!(declares(&v), "a holder is declared on");

    // At war with a holder and a force that can contest its points, Peace gives the
    // countdown away. An overrun home may rationally need Peace to survive.
    let mut contest = rival_next_door(RelationState::War);
    crate::fixture::set_armies(&mut contest, &[(ME, 80), (ENEMY, 90)]);
    contest.standings[1].vp = 10;
    contest.standings[1].victory_hold = Some(VictoryHold {
        turns_held: 3,
        turns_needed: 10,
    });
    let strong = Planner::new(
        &contest,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        1,
    );
    assert!(strong.vp_peace_cost(TeamId(2)) > 0);
    assert!(!strong.wants_peace(TeamId(2)));
    contest.standings[1].vp = 9;
    contest.standings[1].victory_hold = None;
    let one_short = Planner::new(
        &contest,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        1,
    );
    assert!(one_short.vp_peace_cost(TeamId(2)) > 0);
    assert!(!one_short.wants_peace(TeamId(2)));
    let mut war = ViewBuilder {
        view: rival_next_door(RelationState::War),
    }
    .unit(60, ENEMY, UnitKind::Pikeman, off(5, 8))
    .unit(61, ENEMY, UnitKind::Pikeman, off(5, 7))
    .unit(62, ENEMY, UnitKind::Pikeman, off(5, 9))
    .build();
    let wants = |v: &PlayerView, d| {
        let mut p = Planner::new(
            v,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            1,
        );
        p.difficulty = d;
        // Prices the hold against survival; `runaway_coalition` has its own Peace tests.
        p.decisions.runaway_coalition = 0;
        p.wants_peace(TeamId(2))
    };
    assert!(
        wants(&war, AiDifficulty::Normal),
        "overrun: peace without a hold"
    );
    assert!(wants(&war, AiDifficulty::Hard));
    war.standings[1].vp = 10;
    war.standings[1].victory_hold = Some(VictoryHold {
        turns_held: 3,
        turns_needed: 10,
    });
    // No useful force remains in range, so keeping this war does not break the hold.
    assert!(wants(&war, AiDifficulty::Hard));
    let mut old = Planner::new(
        &war,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        1,
    );
    old.decisions.vp_peace = 0;
    assert!(!old.wants_peace(TeamId(2)));
}

#[test]
fn a_hit_that_takes_points_off_a_holders_margin_is_worth_more() {
    let mut v = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 4)
        .city(9, ENEMY, off(12, 8), 6)
        .build();
    v.standings = vec![standing(0, 4, 17, None), standing(2, 10, 10, Some(1))].into();
    let hold = crate::hold::rival_hold(&v).unwrap();
    assert_eq!(
        (hold.team, hold.turns_left, hold.margin()),
        (TeamId(2), 9, 0)
    );
    assert!(
        crate::hold::denial_priority(&v, ENEMY, || 1)
            > crate::hold::denial_priority(&v, ENEMY, || 0)
    );
    assert_eq!(crate::hold::denial_priority(&v, PlayerId(3), || 1), 0);
}

#[test]
fn army_value_matches_core_for_ranged_promoted_and_unsupplied_units() {
    let view = ViewBuilder::new(12, 12)
        .faction(Faction::Sparta)
        .unit(1, ME, UnitKind::Artillery, off(3, 3))
        .build();
    let mut unit = view.units[0].clone();
    unit.military = true;
    unit.strength = 77;
    unit.ranged = 98;
    unit.hp = 50;
    unit.level = 2;
    unit.unsupplied = None;
    assert_eq!(crate::hold::army_value(&unit), 342);
    unit.unsupplied = Some(legatum_core::resources::Resource::Oil);
    assert_eq!(
        crate::hold::army_value(&unit),
        legatum_core::army::combat_value(77, 98, 50, 2, true)
    );
    unit.military = false;
    assert_eq!(crate::hold::army_value(&unit), 0);
}

//! Gene `dead_war_peace` (`diplomacy.rs::dead_war`): a war nobody fights is worth ending,
//! the more so while a rival leader is the war that matters (8.9.17 Mongols review rec 2: the
//! Celts refused 11 Peace offers from Athens over 85 turns of a war their own plan wanted over).
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::plan::{Plan, PlanKind};
use crate::scripted::Planner;
use legatum_core::diplomacy::{Relation, RelationState};
use legatum_core::view::TeamStanding;
use legatum_core::{AiDifficulty, PlayerId, PlayerView, TeamId, UnitKind};

const THIRD: PlayerId = PlayerId(3);

fn relation(a: TeamId, b: TeamId, state: RelationState, since: u32) -> Relation {
    Relation {
        a,
        b,
        state,
        effective_turn: since,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: (state == RelationState::War).then_some(1),
        declarers: if state == RelationState::War {
            vec![b]
        } else {
            Vec::new()
        },
    }
}

/// Turn 80, at War with the enemy since turn 20 (it declared), twice its public army, at
/// Peace with a third empire that leads the VP race (6 of 10) and stands within 12 hexes.
fn dead_view() -> PlayerView {
    let mut v = ViewBuilder::new(40, 20)
        .city(1, ME, off(4, 8), 3)
        .city(2, ME, off(4, 12), 3)
        .city(10, ENEMY, off(32, 6), 3)
        .city(11, ENEMY, off(32, 10), 3)
        .city(12, ENEMY, off(32, 14), 3)
        .city(20, THIRD, off(14, 4), 3)
        .city(21, THIRD, off(14, 16), 3)
        .unit(1, ME, UnitKind::Spearman, off(5, 8))
        .unit(2, ME, UnitKind::Spearman, off(5, 12))
        .build();
    v.turn = 80;
    v.diplomacy.legacy = false;
    let (me, enemy, third) = (v.team, v.team_of(ENEMY), v.team_of(THIRD));
    v.diplomacy.relations = vec![
        relation(me, enemy, RelationState::War, 20),
        relation(me, third, RelationState::Peace, 0),
        relation(enemy, third, RelationState::Peace, 0),
    ];
    for p in &mut v.players {
        p.cities = v.cities.iter().filter(|c| c.owner == p.id).count() as u32;
    }
    crate::fixture::set_armies(&mut v, &[(ME, 300), (ENEMY, 150), (THIRD, 300)]);
    v.standings = [(me, ME, 3), (enemy, ENEMY, 2), (third, THIRD, 6)]
        .map(|(team, member, vp)| TeamStanding {
            team,
            members: vec![member],
            vp,
            target: 10,
            victory_hold: None,
        })
        .to_vec()
        .into();
    v
}

fn planner(v: &PlayerView, gene: i32, plan: PlanKind) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.plan_routing = true;
    p.decisions.dead_war_peace = gene;
    p.campaigns.plan.active = Some(Plan {
        kind: plan,
        start_turn: v.turn - 5,
        review_turn: v.turn + 5,
        reserve: 0,
        value: 0,
        terms: Vec::new(),
        breach: false,
        misses: 0,
    });
    p
}

fn peace_plan(v: &PlayerView) -> PlanKind {
    PlanKind::Peace {
        with: v.team_of(ENEMY),
    }
}

#[test]
fn a_war_the_plan_does_not_fight_scores_peace() {
    let v = dead_view();
    let enemy = v.team_of(ENEMY);
    let off = planner(&v, 0, peace_plan(&v)).peace_diagnosis(enemy);
    assert!(off.wants, "the plan wants this war over: {off:?}");
    assert!(
        off.score.is_some_and(|s| s < 0),
        "gene off: twice the enemy's army refuses its Peace: {off:?}"
    );
    // Gene `peace_follows_plan` 2: plain Hard's own Peace plan floors the score at zero.
    let mut follows = planner(&v, 0, peace_plan(&v));
    follows.decisions.peace_follows_plan = 2;
    assert_eq!(follows.peace_diagnosis(enemy).score, Some(0));
    let on = planner(&v, 12, peace_plan(&v)).peace_diagnosis(enemy);
    assert!(on.wants);
    assert!(
        on.score.is_some_and(|s| s >= 0),
        "gene on: a dead war's Peace is accepted: {on:?}"
    );
    assert!(on.terms.contains(&("dead war", 30)), "{on:?}");
    assert!(
        on.terms.contains(&("rival leader", 20)),
        "the VP leader within reach is the war that matters: {on:?}"
    );
    assert!(
        !on.terms.iter().any(|(t, _)| *t == "truce forgoes pressing"),
        "a dead war presses nothing: {on:?}"
    );
}

#[test]
fn a_recent_skirmish_does_not_veto_a_dead_wars_peace() {
    let v = dead_view();
    let enemy = v.team_of(ENEMY);
    let mut off = planner(&v, 0, peace_plan(&v));
    off.campaigns.plan.war_fights.insert(enemy, v.turn - 2);
    assert_eq!(
        off.wants_peace_why(enemy),
        (false, "peace would be broken"),
        "gene off: one recent skirmish vetoes the Peace the plan wants"
    );
    let mut on = planner(&v, 12, peace_plan(&v));
    on.campaigns.plan.war_fights.insert(enemy, v.turn - 2);
    assert!(on.dead_war(enemy).is_some());
    assert!(on.wants_peace(enemy), "{:?}", on.wants_peace_why(enemy));
}

#[test]
fn a_war_still_fought_or_worth_keeping_is_not_dead() {
    let v = dead_view();
    let enemy = v.team_of(ENEMY);
    // The plan fights this enemy.
    let p = planner(&v, 12, PlanKind::Contain { against: enemy });
    assert_eq!(p.dead_war(enemy), None);
    // Too young.
    let p = planner(&v, 61, peace_plan(&v));
    assert_eq!(p.dead_war(enemy), None);
    // Our army stands at one of its cities.
    let mut at_city = dead_view();
    at_city.units[0].pos = off(30, 10);
    let p = planner(&at_city, 12, peace_plan(&at_city));
    assert_eq!(p.dead_war(enemy), None);
    // W2.3 stands: a rich, hollow enemy (a Power lead, an army per city at most half the
    // median's) is a war worth keeping.
    let mut hollow = dead_view();
    hollow.players[ENEMY.0 as usize].lead_percent = 64;
    crate::fixture::set_armies(&mut hollow, &[(ME, 300), (ENEMY, 30), (THIRD, 300)]);
    let p = planner(&hollow, 12, peace_plan(&hollow));
    assert!(crate::threat::hollow_bonus(&hollow, enemy) > 0);
    assert_eq!(p.dead_war(enemy), None);
    // A co-belligerent: the enemy is itself at war with the VP leader, so its army is already
    // on the front that matters (Chur T137: an early offer to Japan turned Japan's Peace offer
    // to the Swiss).
    let mut busy = dead_view();
    let third = busy.team_of(THIRD);
    busy.diplomacy.relations[2].state = RelationState::War;
    let p = planner(&busy, 12, peace_plan(&busy));
    assert!(busy.teams_at_war(enemy, third));
    assert_eq!(p.dead_war(enemy), None);
    // The VP leader: even an idle war costs it (`leader_war`).
    let mut leader = dead_view();
    let third = leader.team_of(THIRD);
    leader.standings = [(leader.team, ME, 3), (enemy, ENEMY, 8), (third, THIRD, 4)]
        .map(|(team, member, vp)| TeamStanding {
            team,
            members: vec![member],
            vp,
            target: 10,
            victory_hold: None,
        })
        .to_vec()
        .into();
    let p = planner(&leader, 12, peace_plan(&leader));
    assert!(p.leader_war(enemy));
    assert_eq!(p.dead_war(enemy), None);
}

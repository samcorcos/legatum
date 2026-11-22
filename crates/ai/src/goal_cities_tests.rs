use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, off, planner};
use legatum_core::view::PlayerView;
use legatum_core::{AiDifficulty, CityId, Hex, TeamId, UnitKind, VpSource};

const ROME: PlayerId = PlayerId(3);

/// Us (Switzerland), a rival holding city 5, a third empire `ROME` (a Rome) with city 7.
fn world() -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 5)
        .city(5, ENEMY, off(20, 10), 5)
        .city(6, ENEMY, off(26, 16), 3)
        .city(7, ROME, off(34, 4), 4)
        .build();
    v.players[3].faction = Faction::Rome;
    v
}

fn goal_city(of: PlayerId, holder: PlayerId, city: u32, at: Hex) -> GoalCityView {
    GoalCityView {
        of,
        holder,
        city: CityId(city),
        at,
        name: "City".into(),
    }
}

fn city(v: &PlayerView, id: u32) -> &CityView {
    v.cities.iter().find(|c| c.id == CityId(id)).unwrap()
}

fn hard(v: &PlayerView) -> Planner<'_> {
    let mut p = planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p
}

#[test]
fn a_rivals_colonia_or_tomb_city_is_worth_its_two_vp_swing() {
    let mut v = world();
    // A rival holds Rome's Colonia (it captured it): capturing it moves the 2 VP to us.
    v.colonias = vec![goal_city(ROME, ENEMY, 5, off(20, 10))];
    let p = hard(&v);
    assert_eq!(p.goal_city_in_city_value(city(&v, 5)), 200);
    assert_eq!(p.goal_city_swing(city(&v, 5)), (2, 2));
    assert_eq!(p.capture_goal_swing(city(&v, 5)), (2, 2));
    assert!(p.hoard_denial_value(city(&v, 5)) >= 200);
    // Another city of the same rival carries nothing.
    assert_eq!(p.goal_city_in_city_value(city(&v, 6)), 0);
    // A royal tomb is priced the same way.
    v.colonias.clear();
    v.royal_tombs = vec![goal_city(ENEMY, ENEMY, 6, off(26, 16))];
    let p = hard(&v);
    assert_eq!(p.goal_city_in_city_value(city(&v, 6)), 200);
    assert_eq!(p.capture_goal_swing(city(&v, 6)), (2, 2));
    // Gene off: an ordinary city again.
    let mut off_gene = hard(&v);
    off_gene.decisions.goal_city_capture = 0;
    assert_eq!(off_gene.goal_city_in_city_value(city(&v, 6)), 0);
    assert_eq!(off_gene.capture_goal_swing(city(&v, 6)), (0, 0));
}

#[test]
fn the_founder_values_retaking_its_own_goal_city_higher() {
    let mut v = world();
    v.players[0].faction = Faction::Rome;
    v.colonias = vec![goal_city(ME, ENEMY, 5, off(20, 10))];
    assert_eq!(hard(&v).goal_city_in_city_value(city(&v, 5)), 500);
    // A goal city we hold is no capture target of ours.
    v.colonias = vec![goal_city(ROME, ME, 1, off(4, 4))];
    assert_eq!(hard(&v).goal_city_in_city_value(city(&v, 1)), 0);
}

#[test]
fn a_holder_scoring_a_rome_or_egypt_goal_is_a_reason_for_war() {
    let mut v = world();
    v.players[2].vp_sources = vec![(VpSource::FactionGoal { of: ROME }, 2)];
    assert_eq!(hard(&v).held_goal_denial(TeamId(2)), 200);
    // A Switzerland's hoard is not a city goal.
    v.players[3].faction = Faction::Switzerland;
    assert_eq!(hard(&v).held_goal_denial(TeamId(2)), 0);
}

#[test]
fn we_garrison_and_defend_a_goal_city_we_hold_first() {
    let mut v = world();
    v.players[0].faction = Faction::Rome;
    v.colonias = vec![goal_city(ME, ME, 1, off(4, 4))];
    let p = hard(&v);
    assert!(p.holds_goal_city(city(&v, 1)));
    assert!(p.goal_city_unguarded(city(&v, 1)));
    assert_eq!(p.hall_defense_percent(city(&v, 1)), 150);
    let v = ViewBuilder { view: v }
        .unit(10, ME, UnitKind::Spearman, off(4, 4))
        .unit(11, ME, UnitKind::Spearman, off(5, 4))
        .build();
    assert!(!hard(&v).goal_city_unguarded(city(&v, 1)));
    // A captured one is ours to hold too; with the Rome gene off our own is ordinary.
    let mut captured = world();
    captured.royal_tombs = vec![goal_city(ENEMY, ME, 1, off(4, 4))];
    assert!(hard(&captured).holds_goal_city(city(&captured, 1)));
    let mut off_gene = hard(&v);
    off_gene.decisions.rome_colonia = 0;
    assert!(!off_gene.holds_goal_city(city(&v, 1)));
}

#[test]
fn a_working_egypt_is_weak_and_its_tomb_city_about_to_score() {
    let mut v = world();
    v.players[2].faction = Faction::Egypt;
    let p = hard(&v);
    assert_eq!(p.tomb_weakness_value(city(&v, 5)), 0, "not working");
    assert_eq!(p.tomb_site_denial(city(&v, 5)), 0);
    v.players[2].tomb_work = true;
    v.players[2].tomb_turns = 5;
    let p = hard(&v);
    // Ten turns without builds left in every city.
    assert_eq!(p.tomb_weakness_value(city(&v, 5)), 40);
    assert_eq!(p.tomb_weakness_value(city(&v, 6)), 40);
    assert_eq!(p.swarm_weakness_value(city(&v, 6)), 40);
    // Its capital (city 5) is where the tomb will stand: a Great Work under construction.
    assert_eq!(p.tomb_site_denial(city(&v, 5)), 100 + 200 * 5 / 15);
    assert_eq!(p.tomb_site_denial(city(&v, 6)), 0);
    assert!(p.rival_objective_denial(city(&v, 5)) >= 166);
    // Our own cities are never targets.
    assert_eq!(p.tomb_weakness_value(city(&v, 1)), 0);
}

#[test]
fn egypt_guards_the_city_its_tomb_will_stand_in_while_the_work_is_wanted() {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Egypt)
        .city(1, ME, off(4, 4), 6)
        .city(2, ME, off(4, 12), 4)
        .build();
    v.turn = 200;
    v.turn_limit = None;
    v.standings = vec![legatum_core::view::TeamStanding {
        team: TeamId(0),
        members: vec![ME],
        vp: 5,
        target: 10,
        victory_hold: None,
    }]
    .into();
    v.actions.tomb = Some(legatum_core::rules::actions::TombView {
        working: false,
        turns: 0,
        target: goals::TOMB_TURNS,
        site: Some(CityId(1)),
        tomb: None,
        blocker: None,
    });
    let p = hard(&v);
    assert!(p.goal_city_guard(city(&v, 1)));
    assert!(p.goal_city_unguarded(city(&v, 1)));
    assert!(!p.goal_city_guard(city(&v, 2)));
    // Early and far from the target the tomb is not wanted: no city is singled out.
    v.turn = 60;
    assert!(!hard(&v).goal_city_guard(city(&v, 1)));
}

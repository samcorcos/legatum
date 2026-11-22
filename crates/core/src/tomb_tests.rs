//! Egypt's royal tomb (`GoalRule::Tomb`): the action, what the work stops, the count, the tomb's
//! city, and holding and losing its VP.

use super::*;
use crate::events::Event;
use crate::orders::{Action, Order};
use crate::rules::actions::{apply_actions, tomb_blocker};
use crate::rules::cities::tests::{P0, P1, at, found, test_game};
use crate::scoring;

fn world() -> (GameState, CityId, CityId) {
    let mut s = test_game(&[(Faction::Egypt, 0), (Faction::Athens, 1)]);
    let capital = found(&mut s, P0, at(4, 4));
    let second = found(&mut s, P0, at(12, 4));
    found(&mut s, P1, at(15, 15));
    (s, capital, second)
}

fn order(s: &mut GameState, build: bool) -> Vec<Event> {
    let mut ev = Vec::new();
    apply_actions(
        s,
        P0,
        &[Order::Action {
            action: Action::Tomb { build },
        }],
        &mut ev,
    );
    ev
}

fn end_turn(s: &mut GameState) -> Vec<Event> {
    let mut ev = Vec::new();
    scoring::update(s, &mut ev);
    ev
}

#[test]
fn only_egypt_builds_the_tomb_and_the_work_is_public() {
    let (mut s, ..) = world();
    assert!(tomb_blocker(&s, P1, true).is_some());
    assert_eq!(tomb_blocker(&s, P0, true), None);
    assert!(tomb_blocker(&s, P0, false).is_some(), "nothing to pause");
    let ev = order(&mut s, true);
    assert!(matches!(
        ev[..],
        [Event::TombStarted {
            player: P0,
            turns: 0
        }]
    ));
    assert!(tomb_working(&s, P0));
    assert!(tomb_blocker(&s, P0, true).is_some(), "already building");
    let rival = crate::view::player_view(&s, P1);
    let egypt = rival.players.iter().find(|p| p.id == P0).unwrap();
    assert!(egypt.tomb_work);
    let own = crate::view::player_view(&s, P0);
    let tomb = own.actions.tomb.unwrap();
    assert!(tomb.working && tomb.blocker.is_none());
    assert_eq!(tomb.target, TOMB_TURNS);
}

#[test]
fn while_the_tomb_is_built_no_city_builds_or_grows() {
    let (mut s, capital, second) = world();
    for id in [capital, second] {
        let c = s.cities.get_mut(&id).unwrap();
        c.population = 3;
    }
    let before = crate::economy::city_yields(&s, &s.cities[&capital]).total;
    assert!(before.food > crate::data::FOOD_PER_POP * 3 || before.production > 0);
    order(&mut s, true);
    for id in [capital, second] {
        let city = &s.cities[&id];
        let y = crate::economy::city_yields(&s, city);
        assert_eq!(y.total.production, 0);
        assert!(y.total.food <= crate::data::FOOD_PER_POP * city.population);
        assert_eq!(
            y.total.gold,
            crate::economy::city_yields(
                &{
                    let mut open = s.clone();
                    open.player_mut(P0).tomb_work = false;
                    open
                },
                city
            )
            .total
            .gold,
            "Gold is untouched"
        );
    }
    // Through real turns: population and build progress stand still.
    let stored: Vec<_> = [capital, second]
        .iter()
        .map(|id| (s.cities[id].population, s.cities[id].production_stored))
        .collect();
    for _ in 0..3 {
        s = crate::resolve_turn(&s, &Default::default()).0;
    }
    let after: Vec<_> = [capital, second]
        .iter()
        .map(|id| (s.cities[id].population, s.cities[id].production_stored))
        .collect();
    assert_eq!(stored, after);
    assert_eq!(s.player(P0).tomb_turns, 3);
}

#[test]
fn the_count_survives_a_pause_and_the_tomb_stands_in_the_capital() {
    let (mut s, capital, _) = world();
    order(&mut s, true);
    for _ in 0..5 {
        end_turn(&mut s);
    }
    let ev = order(&mut s, false);
    assert!(matches!(
        ev[..],
        [Event::TombPaused {
            player: P0,
            turns: 5
        }]
    ));
    end_turn(&mut s);
    assert_eq!(s.player(P0).tomb_turns, 5, "no count while paused");
    order(&mut s, true);
    let mut raised = Vec::new();
    for _ in 5..TOMB_TURNS {
        assert!(tomb_city(&s, P0).is_none());
        raised = end_turn(&mut s);
    }
    assert!(raised.iter().any(|e| matches!(
        e,
        Event::TombRaised { player: P0, city, .. } if *city == capital
    )));
    assert!(
        raised
            .iter()
            .any(|e| matches!(e, Event::FactionGoalCompleted { player: P0, .. }))
    );
    assert!(!tomb_working(&s, P0), "the work ends with the tomb");
    assert_eq!(tomb_holder(&s, P0), Some(P0));
    assert!(holders(&s).contains(&(P0, P0)));
    assert!(tomb_blocker(&s, P0, true).is_some(), "one tomb at a time");
    // Public, with its place, on a rival's view.
    let rival = crate::view::player_view(&s, P1);
    assert_eq!(rival.royal_tombs.len(), 1);
    assert_eq!(rival.royal_tombs[0].city, capital);
    assert_eq!(rival.royal_tombs[0].at, at(4, 4));
}

#[test]
fn a_captured_tomb_scores_for_its_captor_and_a_razed_one_can_be_rebuilt() {
    let (mut s, capital, _) = world();
    s.player_mut(P0).tomb_city = Some(capital);
    end_turn(&mut s);
    crate::rules::battle::capture_city(&mut s, capital, P1, &mut Vec::new());
    let ev = end_turn(&mut s);
    assert_eq!(tomb_holder(&s, P0), Some(P1));
    assert!(holders(&s).contains(&(P0, P1)));
    assert!(
        ev.iter()
            .any(|e| matches!(e, Event::FactionGoalLost { player: P0, .. }))
    );
    assert!(
        tomb_blocker(&s, P0, true).is_some(),
        "the tomb still stands"
    );
    s.cities.remove(&capital);
    assert_eq!(tomb_holder(&s, P0), None);
    assert_eq!(tomb_blocker(&s, P0, true), None, "Egypt may build another");
}

#[test]
fn without_a_capital_the_tomb_stands_in_the_largest_city() {
    let (mut s, capital, second) = world();
    s.cities.get_mut(&capital).unwrap().is_capital = false;
    s.cities.get_mut(&second).unwrap().population = 9;
    assert_eq!(tomb_site(&s, P0), Some(second));
}

#[test]
fn the_egypt_goal_switch_turns_the_tomb_off() {
    let (mut s, ..) = world();
    s.setup
        .rule_experiments
        .faction_rules_off
        .insert(crate::experiments::FactionRule::EgyptGoal);
    assert!(tomb_blocker(&s, P0, true).is_some());
    assert!(crate::view::player_view(&s, P0).actions.tomb.is_none());
}

#[test]
fn rivals_see_where_the_tomb_under_construction_will_stand() {
    let (mut s, capital, _) = world();
    assert!(
        crate::view::player_view(&s, P1).tomb_sites.is_empty(),
        "no work, no site"
    );
    order(&mut s, true);
    let rival = crate::view::player_view(&s, P1);
    assert_eq!(rival.tomb_sites.len(), 1);
    assert_eq!(rival.tomb_sites[0].of, P0);
    assert_eq!(rival.tomb_sites[0].city, capital);
    assert_eq!(rival.tomb_sites[0].at, at(4, 4));
    s.player_mut(P0).tomb_turns = 6;
    order(&mut s, false);
    assert_eq!(s.player(P0).tomb_turns, 6, "the pause keeps the count");
    assert_eq!(
        crate::view::player_view(&s, P1).tomb_sites.len(),
        1,
        "a paused tomb with turns counted keeps its site"
    );
    s.player_mut(P0).tomb_turns = 0;
    assert!(
        crate::view::player_view(&s, P1).tomb_sites.is_empty(),
        "no work and no count, no site"
    );
}

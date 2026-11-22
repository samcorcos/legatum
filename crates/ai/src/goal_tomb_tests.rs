use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
use crate::{Brain, ScriptedBrain};
use legatum_core::orders::Order;
use legatum_core::view::TeamStanding;
use legatum_core::{AiDifficulty, Faction, PlayerId, TeamId, UnitKind};

const CAPITAL: CityId = CityId(1);

fn tomb(working: bool, turns: u32) -> TombView {
    TombView {
        working,
        turns,
        target: goals::TOMB_TURNS,
        site: Some(CAPITAL),
        tomb: None,
        blocker: None,
    }
}

/// An Egypt at `turn` with `vp` of `target`: a capital garrisoned by two Medjay (one on the
/// tile), a second city, and a quiet rival far away.
fn egypt(turn: u32, vp: u32, target: u32) -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Egypt)
        .city(1, ME, off(4, 4), 6)
        .city(2, ME, off(4, 12), 4)
        .city(3, ENEMY, off(34, 20), 5)
        .unit(10, ME, UnitKind::Spearman, off(4, 4))
        .unit(11, ME, UnitKind::Spearman, off(5, 4))
        .build();
    v.turn = turn;
    v.turn_limit = None;
    v.standings = vec![TeamStanding {
        team: TeamId(0),
        members: vec![ME, PlayerId(1)],
        vp,
        target,
        victory_hold: None,
    }]
    .into();
    v.actions.tomb = Some(tomb(false, 0));
    v
}

fn tomb_orders(v: &PlayerView) -> (bool, bool) {
    let o = crate::goal_actions::goal_actions(v);
    (
        o.contains(&Order::Action {
            action: Action::Tomb { build: true },
        }),
        o.contains(&Order::Action {
            action: Action::Tomb { build: false },
        }),
    )
}

fn with_enemy(v: PlayerView, id: u32, pos: legatum_core::Hex) -> PlayerView {
    ViewBuilder { view: v }
        .unit(id, ENEMY, UnitKind::Spearman, pos)
        .build()
}

#[test]
fn the_tomb_starts_when_its_points_reach_the_target_or_the_game_is_late() {
    // 8 of 10: the 2 VP (and one more earned meanwhile) reach the target with time to hold.
    assert_eq!(tomb_orders(&egypt(100, 8, 10)), (true, false));
    // 7 of 10 with the lead: still the target.
    assert_eq!(tomb_orders(&egypt(100, 7, 10)), (true, false));
    // Far from the target in mid-game: 15 turns of no builds are not worth 2 VP yet.
    assert_eq!(tomb_orders(&egypt(60, 3, 10)), (false, false));
    assert_eq!(tomb_orders(&egypt(150, 5, 10)), (false, false));
    // 60% of the game gone: the points are banked.
    assert_eq!(tomb_orders(&egypt(180, 5, 10)), (true, false));
    // Too late to finish before a turn limit: never. (With no limit the game ends when
    // someone wins, so there is no "too late".)
    let mut limited = egypt(290, 8, 10);
    limited.turn_limit = Some(300);
    assert_eq!(tomb_orders(&limited), (false, false));
}

#[test]
fn the_tomb_waits_for_its_city_to_be_garrisoned() {
    let mut v = egypt(180, 5, 10);
    v.units.retain(|u| u.id.0 != 11);
    assert_eq!(tomb_orders(&v), (false, false), "one guard of two");
    v.units.retain(|u| u.id.0 != 10);
    assert_eq!(tomb_orders(&v), (false, false), "no guard");
    // Wanted, so the site is the city to guard first.
    assert_eq!(tomb_guard_site(&v, false), Some(CAPITAL));
    // When the 2 VP decide the game one guard on the tile is enough.
    let mut decisive = egypt(100, 8, 10);
    decisive.units.retain(|u| u.id.0 != 11);
    assert_eq!(tomb_orders(&decisive), (true, false));
    // Guards beside the city but none on it: not held.
    let mut beside = egypt(180, 5, 10);
    beside.units.retain(|u| u.id.0 != 10);
    beside = ViewBuilder { view: beside }
        .unit(12, ME, UnitKind::Spearman, off(4, 5))
        .build();
    assert_eq!(tomb_orders(&beside), (false, false));
}

#[test]
fn an_enemy_army_on_the_approaches_holds_the_start_and_pauses_the_work() {
    // An at-war army four hexes from the second city: the work would leave it without builds.
    let v = with_enemy(egypt(180, 5, 10), 50, off(8, 12));
    assert_eq!(tomb_orders(&v), (false, false));
    // Working with that army five hexes out: not yet at the gates, the work goes on.
    let mut far = with_enemy(egypt(180, 5, 10), 50, off(9, 12));
    far.actions.tomb = Some(tomb(true, 4));
    assert_eq!(tomb_orders(&far), (false, false));
    // At the gates: pause so the cities build defenders again.
    let mut near = with_enemy(egypt(180, 5, 10), 50, off(6, 12));
    near.actions.tomb = Some(tomb(true, 4));
    assert_eq!(tomb_orders(&near), (false, true));
    // Two turn ends from standing: finish it.
    near.actions.tomb = Some(tomb(true, goals::TOMB_TURNS - 2));
    assert_eq!(tomb_orders(&near), (false, false));
    // Ten turns in, one unit at the gates does not stop it; three do.
    near.actions.tomb = Some(tomb(true, 10));
    assert_eq!(tomb_orders(&near), (false, false));
    let mut army = with_enemy(with_enemy(near, 51, off(6, 13)), 52, off(5, 14));
    assert_eq!(tomb_orders(&army), (false, true));
    army.actions.tomb = Some(tomb(true, 5));
    assert_eq!(
        tomb_orders(&army),
        (false, true),
        "one tolerated at five turns"
    );
}

#[test]
fn a_paused_count_resumes_when_the_approaches_clear() {
    // Mid-game and far from the target, a fresh tomb waits; a count already started is
    // finished once it is safe, since its turns are sunk.
    let mut v = egypt(120, 4, 10);
    assert_eq!(tomb_orders(&v), (false, false));
    v.actions.tomb = Some(tomb(false, 6));
    assert_eq!(tomb_orders(&v), (true, false));
    // Still an army near: it stays paused.
    let near = with_enemy(v.clone(), 50, off(7, 4));
    assert_eq!(tomb_orders(&near), (false, false));
}

#[test]
fn the_work_pauses_when_the_game_ends_before_it_can_finish() {
    let mut v = egypt(250, 5, 10);
    v.turn_limit = Some(260);
    v.actions.tomb = Some(tomb(true, 2));
    assert_eq!(tomb_orders(&v), (false, true));
    v.actions.tomb = Some(tomb(true, 10));
    assert_eq!(
        tomb_orders(&v),
        (false, false),
        "five turn ends left, ten turns"
    );
}

#[test]
fn barbarians_that_spare_no_one_count_only_at_two_hexes() {
    let v = ViewBuilder {
        view: egypt(180, 5, 10),
    }
    .unit(
        60,
        legatum_core::PlayerId::BARBARIAN,
        UnitKind::Spearman,
        off(4, 8),
    )
    .build();
    // Four hexes from both cities: a raider that far is no threat to the work.
    assert_eq!(tomb_orders(&v), (true, false));
    let v = ViewBuilder {
        view: egypt(180, 5, 10),
    }
    .unit(
        60,
        legatum_core::PlayerId::BARBARIAN,
        UnitKind::Spearman,
        off(4, 10),
    )
    .build();
    assert_eq!(tomb_orders(&v), (false, false));
}

#[test]
fn nothing_is_ordered_when_the_tomb_stands_is_blocked_or_the_gene_is_off() {
    let mut v = egypt(180, 5, 10);
    v.actions.tomb.as_mut().unwrap().tomb = Some(CAPITAL);
    assert_eq!(tomb_orders(&v), (false, false));
    let mut v = egypt(180, 5, 10);
    v.actions.tomb.as_mut().unwrap().blocker = Some("no".into());
    assert_eq!(tomb_orders(&v), (false, false));
    let v = egypt(180, 5, 10);
    let off_gene = crate::DecisionParams {
        egypt_tomb: 0,
        ..Default::default()
    };
    assert_eq!(tomb_action(&v, false, &off_gene), None);
    assert_eq!(
        tomb_action(&v, false, &crate::DecisionParams::default()),
        Some(Action::Tomb { build: true })
    );
}

#[test]
fn the_brain_orders_a_valid_start() {
    let v = egypt(180, 5, 10);
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        let orders = ScriptedBrain::with_difficulty(Faction::Egypt, 7, ME, difficulty).orders(&v);
        assert_valid(&v, &orders);
        assert!(
            orders.contains(&Order::Action {
                action: Action::Tomb { build: true }
            }),
            "{difficulty:?}: {orders:?}"
        );
    }
}

#[test]
fn a_rival_sees_where_a_working_egypts_tomb_will_stand() {
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 6)
        .city(5, ENEMY, off(30, 10), 3)
        .city(6, ENEMY, off(30, 18), 8)
        .build();
    assert_eq!(rival_tomb_site(&v, ENEMY), Some(CityId(5)), "the capital");
    v.cities
        .iter_mut()
        .find(|c| c.id == CityId(5))
        .unwrap()
        .is_capital = false;
    assert_eq!(
        rival_tomb_site(&v, ENEMY),
        Some(CityId(6)),
        "else the largest"
    );
}

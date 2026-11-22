use super::*;
use crate::fixture::{ENEMY, MATE, ME, ViewBuilder, off};
use crate::openings::{HandoffReason, Phase};
use legatum_core::{BuildingKind, PlayerView, ProductionItem, UnitKind};

fn base(v: &PlayerView) -> ScriptedBrain {
    ScriptedBrain::with_difficulty(v.faction_of(ME), 7, ME, AiDifficulty::Hard)
}

fn quiet_view() -> PlayerView {
    ViewBuilder::new(40, 24)
        .city(0, ME, off(5, 5), 2)
        .unit(0, ME, UnitKind::Spearman, off(5, 5))
        .build()
}

fn assert_handoff(b: &mut OpeningBrain, v: &PlayerView, reason: HandoffReason) {
    assert_eq!(b.orders(v), base(v).orders(v));
    let h = b.opening_handoff().unwrap();
    assert_eq!((h.turn, h.reason), (v.turn, reason));
    assert!(b.primitive_coalition_march().is_none());
}

#[test]
fn handoff_cap_applies_to_every_opening_and_permanently_uses_exact_default_orders() {
    for opening in Opening::ALL.into_iter().filter(|o| *o != Opening::Default) {
        let cap = 80;
        let mut v = quiet_view();
        v.turn = cap - 1;
        let mut b = OpeningBrain::new(base(&v), policy(opening)).with_handoff();
        let mut full = OpeningBrain::new(base(&v), policy(opening));
        assert_eq!(b.orders(&v), full.orders(&v));
        assert_eq!(b.opening_handoff(), None, "{opening:?}");
        v.turn = cap;
        assert_handoff(&mut b, &v, HandoffReason::TurnCap);
        let handoff = b.opening_handoff();
        // Losses and changed resources cannot reactivate the opening.
        v.turn += 1;
        v.units.clear();
        v.economy.gold = 700;
        assert_eq!(b.orders(&v), base(&v).orders(&v));
        assert_eq!(b.opening_handoff(), handoff);
        assert_eq!(b.opening(), Some(opening));
        full.orders(&v);
        assert_eq!(full.opening_handoff(), None);
    }
}

#[test]
fn expansion_handoff_requires_owned_completed_cities_at_each_policys_target() {
    for opening in [Opening::GreedyExpand] {
        let p = policy(opening);
        let target = p.numbers.settler_target as u32 + 1;
        let mut fixture = ViewBuilder::new(48, 24);
        for id in 0..target {
            fixture = fixture.city(
                id,
                if id == target - 1 { MATE } else { ME },
                off(3 + id as i32 * 7, 5),
                2,
            );
        }
        let mut v = fixture.unit(0, ME, UnitKind::Settler, off(8, 12)).build();
        v.cities[0].detail.as_mut().unwrap().production_queue = vec![
            ProductionItem::Unit {
                unit: UnitKind::Settler
            };
            2
        ];
        let mut b = OpeningBrain::new(base(&v), p).with_handoff();
        b.orders(&v);
        assert_eq!(b.opening_handoff(), None, "{opening:?}");
        // Founding/capture counts; allies, travelling settlers and queues do not.
        v.cities.last_mut().unwrap().owner = ME;
        v.turn += 1;
        assert_handoff(&mut b, &v, HandoffReason::CityTarget);
        v.cities.truncate(1);
        v.turn += 1;
        assert_eq!(b.orders(&v), base(&v).orders(&v));
        assert_eq!(b.opening_handoff().unwrap().turn, v.turn - 1);
    }
}

#[test]
fn zero_expansion_target_still_requires_the_capital() {
    let mut v = ViewBuilder::new(24, 18)
        .unit(0, ME, UnitKind::Settler, off(5, 5))
        .build();
    let mut p = policy(Opening::GreedyExpand);
    p.numbers.settler_target = 0;
    let mut b = OpeningBrain::new(base(&v), p).with_handoff();
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None);
    v = ViewBuilder { view: v }.city(0, ME, off(5, 5), 2).build();
    v.turn += 1;
    assert_handoff(&mut b, &v, HandoffReason::CityTarget);
}

#[test]
fn economic_core_counts_completed_units_once_excluding_starting_units_and_queues() {
    let mut v = quiet_view();
    v = ViewBuilder { view: v }
        .unit(1, ME, UnitKind::Settler, off(5, 5))
        .unit(2, MATE, UnitKind::Settler, off(5, 5))
        .unit(3, ME, UnitKind::Scout, off(5, 5))
        .build();
    v.units[1].created_turn = Some(1);
    v.units[2].created_turn = Some(2);
    v.units[3].created_turn = Some(2);
    let d = v.cities[0].detail.as_mut().unwrap();
    d.buildings = vec![
        BuildingKind::Market,
        BuildingKind::Workshop,
        BuildingKind::Library,
    ];
    d.production_queue = vec![
        ProductionItem::Unit {
            unit: UnitKind::Settler
        };
        3
    ];
    let p = policy(Opening::Economic);
    assert_eq!(p.builds[1].count, 3);
    let mut b = OpeningBrain::new(base(&v), p).with_handoff();
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None);
    for id in 10..13 {
        v.turn += 1;
        v = ViewBuilder { view: v }
            .unit(id, ME, UnitKind::Settler, off(5, 5))
            .build();
        v.units.last_mut().unwrap().created_turn = Some(v.turn);
        if id < 12 {
            // Replanning and observing the same unit next turn never counts twice.
            b.orders(&v);
            b.orders(&v);
            v.turn += 1;
            b.orders(&v);
            assert_eq!(b.opening_handoff(), None);
            v.units.pop(); // Founding or losing it does not erase its completed build.
        } else {
            assert_handoff(&mut b, &v, HandoffReason::CoreBuild);
        }
    }
}

#[test]
fn economic_handoff_also_waits_for_the_last_core_building() {
    let mut v = quiet_view();
    let mut p = OpeningPolicy::builtin(Opening::Economic);
    p.builds[1].count = 0; // A legal genome without a core unit quota.
    v.cities[0].detail.as_mut().unwrap().buildings =
        vec![BuildingKind::Market, BuildingKind::Workshop];
    let mut b = OpeningBrain::new(base(&v), p).with_handoff();
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None);
    v.turn += 1;
    v.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .push(BuildingKind::Library);
    assert_handoff(&mut b, &v, HandoffReason::CoreBuild);
}

#[test]
fn snowball_handoff_requires_both_capital_workshop_and_completed_expansion() {
    for workshop_first in [false, true] {
        let mut v = quiet_view();
        let p = policy(Opening::Snowball);
        let colonies = p.numbers.settler_target as u32;
        if workshop_first {
            v.cities[0]
                .detail
                .as_mut()
                .unwrap()
                .buildings
                .push(BuildingKind::Workshop);
        } else {
            for id in 1..=colonies {
                v = ViewBuilder { view: v }
                    .city(id, ME, off(3 + id as i32 * 6, 15), 2)
                    .build();
            }
            // A colony workshop cannot substitute for the opening's capital build.
            v.cities[1]
                .detail
                .as_mut()
                .unwrap()
                .buildings
                .push(BuildingKind::Workshop);
        }
        let mut b = OpeningBrain::new(base(&v), p).with_handoff();
        b.orders(&v);
        assert_eq!(b.opening_handoff(), None);
        if workshop_first {
            for id in 1..=colonies {
                v = ViewBuilder { view: v }
                    .city(id, ME, off(3 + id as i32 * 6, 15), 2)
                    .build();
            }
        } else {
            v.cities[0]
                .detail
                .as_mut()
                .unwrap()
                .buildings
                .push(BuildingKind::Workshop);
        }
        v.turn += 1;
        assert_handoff(&mut b, &v, HandoffReason::SnowballReady);
    }
}

#[test]
fn temporary_defense_never_completes_an_opening() {
    for opening in Opening::ALL.into_iter().filter(|o| *o != Opening::Default) {
        let mut v = quiet_view();
        for id in 10..14 {
            v = ViewBuilder { view: v }
                .unit(id, ENEMY, UnitKind::Pikeman, off(6, 5))
                .build();
        }
        let mut b = OpeningBrain::new(base(&v), policy(opening)).with_handoff();
        b.orders(&v);
        assert_eq!(b.phase, Phase::Defend, "{opening:?}");
        assert_eq!(b.opening_handoff(), None, "{opening:?}");
    }
}

#[test]
fn rush_keeps_starting_spearman_and_ready_army_until_its_first_capture() {
    let state = legatum_core::new_game(&legatum_core::GameSetup::all_ai(5));
    let mut v = legatum_core::player_view(&state, ME);
    let mut b = OpeningBrain::new(base(&v), policy(Opening::Rush)).with_handoff();
    let mut full = OpeningBrain::new(base(&v), policy(Opening::Rush));
    assert_eq!(policy(Opening::Rush).numbers.war_army, 1);
    for turn in 1..=3 {
        v.turn = turn;
        assert_eq!(b.orders(&v), full.orders(&v));
        assert_eq!(b.phase, Phase::Conquer);
        assert_eq!(b.opening_handoff(), None);
    }
    v = ViewBuilder { view: v }
        .city(0, ME, off(5, 5), 2)
        .city(1, ENEMY, off(15, 5), 2)
        .unit(99, ME, UnitKind::Spearman, off(5, 5))
        .build();
    // This hand-off fixture requires a viable target: known land and an attackable,
    // non-attrition rival. Unknown routes and Celts now intentionally abandon a rush.
    for tile in &mut v.tiles {
        tile.explored = true;
        tile.visible = true;
        tile.terrain = Some(legatum_core::Terrain::Plains);
    }
    v.diplomacy.legacy = true;
    v.players
        .iter_mut()
        .find(|p| p.id == ENEMY)
        .unwrap()
        .faction = legatum_core::Faction::Athens;
    v.units.last_mut().unwrap().created_turn = Some(3);
    v.turn = 4;
    b.orders(&v);
    assert_eq!(
        b.opening_handoff(),
        None,
        "a trained unit is not a resolved rush"
    );
    v.cities[1].owner = ME;
    v.turn = 5;
    assert_handoff(&mut b, &v, HandoffReason::RushCaptured);
}

#[test]
fn rush_handoff_uses_its_standard_abandonment_clock() {
    let mut v = quiet_view();
    v.turn = 1;
    let mut b = OpeningBrain::new(base(&v), policy(Opening::Rush)).with_handoff();
    b.orders(&v);
    v.turn = 60;
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None);
    v.turn = 61; // Ready after 60 Standard turns.
    assert_handoff(&mut b, &v, HandoffReason::RushAbandoned);
}

#[test]
fn rush_handoff_uses_its_casualty_failure_without_waiting_for_cap() {
    let mut v = quiet_view();
    v = ViewBuilder { view: v }
        .unit(1, ME, UnitKind::Spearman, off(6, 5))
        .unit(2, ME, UnitKind::Spearman, off(7, 5))
        .unit(3, ME, UnitKind::Archer, off(8, 5))
        .build();
    let mut p = policy(Opening::Rush);
    p.numbers.war_army = 4;
    p.numbers.losses_limit = 2;
    let mut b = OpeningBrain::new(base(&v), p).with_handoff();
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None);
    v.turn += 1;
    v.units.pop();
    b.orders(&v);
    assert_eq!(b.opening_handoff(), None, "one casualty is not failure");
    v.turn += 1;
    v.units.truncate(1);
    assert_handoff(&mut b, &v, HandoffReason::RushAbandoned);
}

#[test]
fn handoff_factory_preserves_draw_and_full_opening_orders_on_turn_one() {
    let state = legatum_core::new_game(&legatum_core::GameSetup::all_ai(5));
    let v = legatum_core::player_view(&state, ME);
    for opening in Opening::ALL {
        let seed = (0..10_000)
            .find(|seed| draw_opening(*seed) == opening)
            .unwrap();
        let mut full = brain(v.faction_of(ME), seed, ME);
        let mut handoff = handoff_brain(v.faction_of(ME), seed, ME);
        assert_eq!(handoff.opening(), full.opening());
        assert_eq!(handoff.orders(&v), full.orders(&v), "{opening:?}");
        assert_eq!(handoff.opening_handoff(), None);
    }
}

#[test]
fn historical_handoff_reasons_remain_readable() {
    for reason in [HandoffReason::ArmyTarget, HandoffReason::PhaseEnded] {
        let json = format!(r#"{{"turn":2,"reason":"{reason:?}"}}"#);
        let old: crate::openings::OpeningHandoff = serde_json::from_str(&json).unwrap();
        assert_eq!(old.reason, reason);
        assert_eq!(serde_json::to_string(&old).unwrap(), json);
    }
}

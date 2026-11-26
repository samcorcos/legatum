//! Ranged units never take a city (`RANGED_CAPTURE`). A report said Archers took a city by
//! shooting its last defender; these scenarios pin every way a shooter could reach that city:
//! a shot that kills the last defender, a move onto the emptied city (the same turn and the
//! next), a confirmed path (`MoveUnitPath`, what the map's route preview sends), a long goto,
//! a fight onto a garrisoned tile, and Egypt's Chariot Archer. A melee unit or the Keshig
//! (mounted, it charges) still captures.
use super::*;
use crate::rules::resolve_turn;
use crate::rules::test_support::*;
use crate::{Order, PlayerId};

fn two(p0: Vec<Order>, p1: Vec<Order>) -> BTreeMap<PlayerId, Vec<Order>> {
    BTreeMap::from([(PlayerId(0), p0), (PlayerId(1), p1)])
}

fn shoot(unit: UnitId, target: Hex) -> Order {
    Order::RangedAttack { unit, target }
}

fn move_to(unit: UnitId, to: Hex) -> Order {
    Order::MoveUnit { unit, to }
}

/// P1's city at (5,4) at 0 HP with one badly hurt Spearman inside; P0 has two Archers next to
/// it. Both empires keep a Settler far away so the full resolver does not eliminate anyone.
struct Siege {
    s: GameState,
    city: CityId,
    city_at: Hex,
    archers: [UnitId; 2],
    last_defender: UnitId,
}

fn siege() -> Siege {
    let mut s = game(&[0, 1]);
    let city_at = hex(5, 4);
    let city = city(&mut s, 1, city_at, 3);
    s.cities.get_mut(&city).unwrap().hp = 0;
    let last_defender = unit(&mut s, 1, UnitKind::Spearman, city_at);
    s.units.get_mut(&last_defender).unwrap().hp = 5;
    let archers = [
        unit(&mut s, 0, UnitKind::Archer, city_at.neighbor(2)),
        unit(&mut s, 0, UnitKind::Archer, city_at.neighbor(4)),
    ];
    unit(&mut s, 0, UnitKind::Settler, hex(0, 0));
    unit(&mut s, 1, UnitKind::Settler, hex(11, 9));
    Siege {
        s,
        city,
        city_at,
        archers,
        last_defender,
    }
}

fn owner(s: &GameState, c: CityId) -> PlayerId {
    s.cities[&c].owner
}

fn assert_held(s: &GameState, events: &[Event], c: CityId) {
    assert_eq!(
        owner(s, c),
        PlayerId(1),
        "the city changed hands: {events:#?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CityCaptured { .. } | Event::CityRazed { .. })),
        "no capture event: {events:#?}"
    );
}

fn bounced(events: &[Event], unit: UnitId) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::MoveBlocked { unit: u, reason, .. }
            if *u == unit && reason == RANGED_CAPTURE)
    })
}

#[test]
fn shooting_the_last_defender_does_not_take_the_city() {
    let Siege {
        s,
        city,
        city_at,
        archers,
        last_defender,
    } = siege();
    let os = two(
        vec![shoot(archers[0], city_at), shoot(archers[1], city_at)],
        vec![],
    );
    let (next, events) = resolve_turn(&s, &os);
    assert!(
        !next.units.contains_key(&last_defender),
        "the shots kill it: {events:#?}"
    );
    assert_held(&next, &events, city);
    // Nothing happens to it on the following turns either.
    let (later, events) = resolve_turn(&next, &no_orders());
    assert_held(&later, &events, city);
    assert!(later.units.values().all(|u| u.pos != city_at));
}

#[test]
fn an_archer_moving_onto_the_emptied_city_is_bounced() {
    let Siege {
        s,
        city,
        city_at,
        archers,
        ..
    } = siege();
    // Same turn: one shoots the defender dead, the other walks onto the tile.
    let os = two(
        vec![shoot(archers[0], city_at), move_to(archers[1], city_at)],
        vec![],
    );
    let (next, events) = resolve_turn(&s, &os);
    assert_held(&next, &events, city);
    assert_ne!(next.units[&archers[1]].pos, city_at);
    // Next turn, onto the empty city.
    let (empty, _) = resolve_turn(
        &s,
        &two(
            vec![shoot(archers[0], city_at), shoot(archers[1], city_at)],
            vec![],
        ),
    );
    assert!(empty.units.values().all(|u| u.pos != city_at));
    let (after, events) = resolve_turn(
        &empty,
        &two(
            vec![move_to(archers[0], city_at), move_to(archers[1], city_at)],
            vec![],
        ),
    );
    assert_held(&after, &events, city);
    assert!(bounced(&events, archers[0]) && bounced(&events, archers[1]));
    assert!(archers.iter().all(|a| after.units[a].pos != city_at));
}

#[test]
fn an_archer_attacking_onto_the_last_defender_does_not_take_the_city() {
    let Siege {
        s,
        city,
        city_at,
        archers,
        last_defender,
    } = siege();
    // A move onto the garrisoned tile is an attack, and the hurt Spearman dies in it.
    let (next, events) = resolve_turn(&s, &two(vec![move_to(archers[0], city_at)], vec![]));
    assert!(!next.units.contains_key(&last_defender), "{events:#?}");
    assert_held(&next, &events, city);
    assert_ne!(next.units[&archers[0]].pos, city_at);
}

#[test]
fn a_confirmed_path_or_a_long_goto_into_the_city_is_bounced() {
    let mut s = game(&[0, 1]);
    let city_at = hex(5, 4);
    let city = city(&mut s, 1, city_at, 3);
    s.cities.get_mut(&city).unwrap().hp = 0;
    unit(&mut s, 0, UnitKind::Settler, hex(0, 0));
    unit(&mut s, 1, UnitKind::Settler, hex(11, 9));
    // The map's route preview sends a confirmed path; two tiles out, ending on the city.
    let near = city_at.neighbor(3);
    let start = near.neighbor(3);
    let pathed = unit(&mut s, 0, UnitKind::Archer, start);
    let far = unit(&mut s, 0, UnitKind::Crossbowman, hex(1, 4));
    let os = two(
        vec![
            Order::MoveUnitPath {
                unit: pathed,
                to: city_at,
                path: vec![near, city_at],
                stop_on_block: false,
            },
            move_to(far, city_at),
        ],
        vec![],
    );
    let mut st = s;
    for _ in 0..4 {
        let (next, events) = resolve_turn(&st, &os);
        assert_held(&next, &events, city);
        st = next;
    }
    assert!(st.units.values().all(|u| u.pos != city_at));
}

#[test]
fn egypts_chariot_archer_does_not_take_the_city() {
    let Siege {
        mut s,
        city,
        city_at,
        archers,
        ..
    } = siege();
    s.players[0].faction = Faction::Egypt;
    let (empty, _) = resolve_turn(&s, &two(vec![shoot(archers[0], city_at)], vec![]));
    assert!(empty.units.values().all(|u| u.pos != city_at));
    let (next, events) = resolve_turn(&empty, &two(vec![move_to(archers[1], city_at)], vec![]));
    assert_held(&next, &events, city);
    assert!(bounced(&events, archers[1]));
}

#[test]
fn a_melee_unit_moving_in_after_the_shots_does_take_it() {
    let Siege {
        mut s,
        city,
        city_at,
        archers,
        ..
    } = siege();
    let spear = unit(&mut s, 0, UnitKind::Spearman, city_at.neighbor(0));
    let os = two(
        vec![
            shoot(archers[0], city_at),
            shoot(archers[1], city_at),
            move_to(spear, city_at),
        ],
        vec![],
    );
    let (next, events) = resolve_turn(&s, &os);
    assert_eq!(owner(&next, city), PlayerId(0), "{events:#?}");
    assert_eq!(next.units[&spear].pos, city_at);
}

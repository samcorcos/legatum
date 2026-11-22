//! Sam's game, 2026-11-21. Turn 67, Geneva: five Keshig volleys hit the empty, fallen city and
//! did nothing, then a Swiss Spearman slipped in as its garrison and only met the Keshig charge.
//! His rule: "the order of events should be 1) slip into garrison (with support) 2) ranged
//! attack 3) melee attack". A shot at a fallen, empty city now waits for whoever comes in.
//! Turn 75: a Keshig ordered onto a Horseman and Settler lost its order when another Keshig's
//! shot killed the Horseman, and the Settler walked away. The charge now goes on against the
//! next enemy on the tile.
use super::*;
use crate::events::Defender;
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

/// Ranged hits on `unit`, in event order.
fn shots_on(events: &[Event], unit: UnitId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(e, Event::Battle { defender: Defender::Unit { unit: u, .. }, ranged: true, .. }
                if *u == unit)
        })
        .count()
}

/// P1's fallen, empty city at (5,4); P0 has two Archers in range, a Spearman west of the city;
/// P1's Spearman waits east of it.
struct Fallen {
    s: GameState,
    city: CityId,
    city_at: Hex,
    rangers: [UnitId; 2],
    attacker: UnitId,
    defender: UnitId,
}

fn fallen() -> Fallen {
    let mut s = game(&[0, 1]);
    let city_at = hex(5, 4);
    let c = city(&mut s, 1, city_at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let rangers = [
        unit(&mut s, 0, UnitKind::Archer, city_at.neighbor(2)),
        unit(&mut s, 0, UnitKind::Archer, city_at.neighbor(4)),
    ];
    let attacker = unit(&mut s, 0, UnitKind::Spearman, city_at.neighbor(3));
    let defender = unit(&mut s, 1, UnitKind::Spearman, city_at.neighbor(0));
    Fallen {
        s,
        city: c,
        city_at,
        rangers,
        attacker,
        defender,
    }
}

#[test]
fn geneva_t67_a_unit_slipping_into_a_fallen_city_takes_the_volley_before_the_charge() {
    let Fallen {
        s,
        city,
        city_at,
        rangers,
        attacker,
        defender,
    } = fallen();
    let charge_only = two(
        vec![move_to(attacker, city_at)],
        vec![move_to(defender, city_at)],
    );
    let (without, _) = resolve_units(&s, &charge_only);
    let full = two(
        vec![
            shoot(rangers[0], city_at),
            shoot(rangers[1], city_at),
            move_to(attacker, city_at),
        ],
        vec![move_to(defender, city_at)],
    );
    let (next, events) = resolve_units(&s, &full);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CityReinforced { unit, .. } if *unit == defender)),
        "it still slips in first: {events:#?}"
    );
    assert_eq!(
        shots_on(&events, defender),
        2,
        "both shots hit it: {events:#?}"
    );
    // Order of the report: in, shot, then the charge.
    let moved = events
        .iter()
        .position(|e| matches!(e, Event::UnitMoved { unit, .. } if *unit == defender))
        .unwrap();
    let first_shot = events
        .iter()
        .position(|e| matches!(e, Event::Battle { ranged: true, .. }))
        .unwrap();
    assert!(moved < first_shot);
    let hp = |st: &GameState| st.units.get(&defender).map_or(0, |u| u.hp);
    assert!(
        hp(&next) < hp(&without),
        "the volley wounds it beyond the charge alone: {} vs {}",
        hp(&next),
        hp(&without)
    );
    // No volley was wasted on the empty city.
    assert!(!events.iter().any(|e| matches!(e,
        Event::Battle { defender: Defender::City { city: c, .. }, .. } if *c == city)));
}

#[test]
fn the_volley_can_kill_the_slipping_unit_and_the_charge_captures() {
    let Fallen {
        mut s,
        city,
        city_at,
        rangers,
        attacker,
        defender,
    } = fallen();
    s.units.get_mut(&defender).unwrap().hp = 20;
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![
                shoot(rangers[0], city_at),
                shoot(rangers[1], city_at),
                move_to(attacker, city_at),
            ],
            vec![move_to(defender, city_at)],
        ),
    );
    assert!(!next.units.contains_key(&defender), "{events:#?}");
    assert_eq!(next.cities[&city].owner, PlayerId(0), "captured");
    assert_eq!(next.units[&attacker].pos, city_at);
}

#[test]
fn held_shots_alone_besiege_the_city_so_the_unit_slipping_in_is_hit() {
    let Fallen {
        s,
        city,
        city_at,
        rangers,
        defender,
        ..
    } = fallen();
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![shoot(rangers[0], city_at)],
            vec![move_to(defender, city_at)],
        ),
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CityReinforced { unit, .. } if *unit == defender)),
        "{events:#?}"
    );
    assert_eq!(shots_on(&events, defender), 1);
    assert!(next.units[&defender].hp < 100);
    assert_eq!(next.units[&defender].pos, city_at);
    assert_eq!(next.cities[&city].owner, PlayerId(1));
}

#[test]
fn with_nobody_coming_in_the_shot_lands_on_the_empty_city_as_before() {
    let Fallen {
        s,
        city,
        city_at,
        rangers,
        ..
    } = fallen();
    let (next, events) = resolve_units(&s, &two(vec![shoot(rangers[0], city_at)], vec![]));
    assert!(events.iter().any(|e| matches!(e,
        Event::Battle { defender: Defender::City { city: c, .. }, ranged: true, .. } if *c == city)));
    assert_eq!(next.cities[&city].hp, 0);
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    let r = &next.units[&rangers[0]];
    assert!(
        r.acted && r.moves_left == 0,
        "the shot still used the unit's turn"
    );
}

#[test]
fn a_shot_at_a_city_with_hp_is_not_held() {
    let Fallen {
        mut s,
        city,
        city_at,
        rangers,
        defender,
        ..
    } = fallen();
    s.cities.get_mut(&city).unwrap().hp = 100;
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![shoot(rangers[0], city_at)],
            vec![move_to(defender, city_at)],
        ),
    );
    assert!(
        next.cities[&city].hp < 100,
        "the city's HP absorbs it in the ranged phase"
    );
    assert_eq!(shots_on(&events, defender), 0);
}

#[test]
fn bibracte_t75_the_charge_goes_on_to_the_settler_when_a_shot_kills_its_escort() {
    let mut s = game(&[0, 1]);
    let tile = hex(5, 4);
    let horse = unit(&mut s, 1, UnitKind::Horseman, tile);
    let settler = unit(&mut s, 1, UnitKind::Settler, tile);
    s.units.get_mut(&horse).unwrap().hp = 1;
    let archer = unit(&mut s, 0, UnitKind::Archer, tile.neighbor(2));
    let rider = unit(&mut s, 0, UnitKind::Horseman, tile.neighbor(3));
    let away = tile.neighbor(0).neighbor(0);
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![shoot(archer, tile), move_to(rider, tile)],
            vec![move_to(settler, away)],
        ),
    );
    assert!(
        !next.units.contains_key(&horse),
        "the shot killed the escort"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::SettlerCaptured { .. })),
        "the charge took the Settler: {events:#?}"
    );
    assert!(
        next.units
            .get(&settler)
            .is_none_or(|u| u.owner == PlayerId(0)),
        "the Settler did not get away"
    );
}

#[test]
fn a_charge_whose_target_died_with_nothing_left_says_so() {
    let mut s = game(&[0, 1]);
    let tile = hex(5, 4);
    let horse = unit(&mut s, 1, UnitKind::Horseman, tile);
    s.units.get_mut(&horse).unwrap().hp = 1;
    let archer = unit(&mut s, 0, UnitKind::Archer, tile.neighbor(2));
    let rider = unit(&mut s, 0, UnitKind::Horseman, tile.neighbor(3));
    let (_, events) = resolve_units(
        &s,
        &two(vec![shoot(archer, tile), move_to(rider, tile)], vec![]),
    );
    assert!(events.iter().any(|e| matches!(e,
        Event::MoveBlocked { unit, reason, .. }
            if *unit == rider && reason == TARGET_GONE)));
}

/// Hill-climb suite, 2026-11-21: a Celtic capture razes the fallen city in round 0 while a shot
/// held for it is still pending, and a later round's relief into another of the defender's
/// cities looked the razed city up and panicked. A held shot whose city is gone besieges nothing.
#[test]
fn a_held_shot_whose_city_was_razed_does_not_stop_a_later_relief() {
    let Fallen {
        mut s,
        city,
        city_at,
        rangers,
        attacker,
        ..
    } = fallen();
    s.player_mut(PlayerId(0)).faction = crate::Faction::Celts;
    s.cities.get_mut(&city).unwrap().population = 1;
    // P1's second city three tiles east, and a Spearman two steps from it heading home.
    let home_at = hex(8, 4);
    let home = crate::rules::test_support::city(&mut s, 1, home_at, 3);
    let walker = unit(&mut s, 1, UnitKind::Spearman, hex(10, 4));
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![shoot(rangers[0], city_at), move_to(attacker, city_at)],
            vec![move_to(walker, home_at)],
        ),
    );
    assert!(
        !next.cities.contains_key(&city),
        "the Celts raze it: {events:#?}"
    );
    assert_eq!(next.cities[&home].owner, PlayerId(1));
    assert_eq!(next.units[&walker].pos, home_at, "{events:#?}");
}

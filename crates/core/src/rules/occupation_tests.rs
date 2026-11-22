//! Occupation lock (`rules::occupation`): the rules, one test each.
use super::battle::capture_city;
use super::*;
use crate::diplomacy::RelationState;
use crate::rules::test_support::*;
use crate::state::GameState;
use crate::types::{CityId, StandingOrder};

const AT: (i32, i32) = (5, 5);

/// Player 0 captures player 1's city; a Soldier of player 0 holds it.
fn occupied() -> (GameState, CityId, UnitId) {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(AT.0, AT.1), 3);
    capture_city(&mut s, c, PlayerId(0), &mut Vec::new());
    let u = unit(&mut s, 0, UnitKind::Soldier, hex(AT.0, AT.1));
    (s, c, u)
}

fn step(u: UnitId) -> Vec<Order> {
    vec![Order::MoveUnit {
        unit: u,
        to: hex(AT.0 + 1, AT.1),
    }]
}

fn make_peace(s: &mut GameState) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::Peace;
    }
}

#[test]
fn a_move_out_is_refused_with_the_reason_and_the_unit_stays() {
    let (s, c, u) = occupied();
    assert_eq!(s.cities[&c].occupied_from, Some(PlayerId(1)));
    let errors = validate_orders(&s, PlayerId(0), &step(u));
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message,
        "Spearman is holding City 1 down and cannot leave until Peace with Switzerland"
    );
    let (next, _) = resolve_units(&s, &orders(0, step(u)));
    assert_eq!(next.units[&u].pos, hex(AT.0, AT.1));
    // A standing route is dropped too.
    let mut routed = s.clone();
    goto(&mut routed, u, hex(AT.0 + 2, AT.1));
    let (next, _) = resolve_units(&routed, &no_orders());
    assert_eq!(next.units[&u].pos, hex(AT.0, AT.1));
    assert_eq!(next.units[&u].goto, None);
}

#[test]
fn disband_is_refused_so_it_cannot_dodge_the_lock() {
    let (s, _, u) = occupied();
    let errors = validate_orders(&s, PlayerId(0), &[Order::DisbandUnit { unit: u }]);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("is holding City 1 down"));
}

#[test]
fn fortify_heal_and_shoot_are_allowed() {
    let (mut s, _, soldier) = occupied();
    let archer = unit(&mut s, 0, UnitKind::Ranger, hex(AT.0, AT.1));
    s.units.get_mut(&soldier).unwrap().hp = 60;
    let enemy = unit(&mut s, 1, UnitKind::Soldier, hex(AT.0 + 2, AT.1));
    let ok = |os: Vec<Order>| validate_orders(&s, PlayerId(0), &os).is_empty();
    assert!(ok(vec![Order::SkipUnit { unit: soldier }]));
    assert!(ok(vec![Order::SleepUnit {
        unit: soldier,
        until_healed: true
    }]));
    assert!(ok(vec![Order::MoveUnit {
        unit: soldier,
        to: hex(AT.0, AT.1)
    }]));
    assert!(ok(vec![Order::RangedAttack {
        unit: archer,
        target: hex(AT.0 + 2, AT.1)
    }]));
    let (next, events) = resolve_units(
        &s,
        &orders(
            0,
            vec![Order::RangedAttack {
                unit: archer,
                target: hex(AT.0 + 2, AT.1),
            }],
        ),
    );
    assert!(next.units[&enemy].hp < s.units[&enemy].hp, "{events:?}");
    assert_eq!(next.units[&archer].pos, hex(AT.0, AT.1));
}

#[test]
fn peace_releases_the_unit() {
    let (mut s, _, u) = occupied();
    make_peace(&mut s);
    assert!(validate_orders(&s, PlayerId(0), &step(u)).is_empty());
    let (next, _) = resolve_units(&s, &orders(0, step(u)));
    assert_eq!(next.units[&u].pos, hex(AT.0 + 1, AT.1));
}

#[test]
fn elimination_of_the_civ_releases_the_unit() {
    let (mut s, _, u) = occupied();
    s.players[1].alive = false;
    assert!(validate_orders(&s, PlayerId(0), &step(u)).is_empty());
    let (next, _) = resolve_units(&s, &orders(0, step(u)));
    assert_eq!(next.units[&u].pos, hex(AT.0 + 1, AT.1));
}

#[test]
fn a_replacement_unit_that_moves_in_is_locked_too() {
    let (mut s, _, u) = occupied();
    s.units.remove(&u);
    let relief = unit(&mut s, 0, UnitKind::Soldier, hex(AT.0 - 1, AT.1));
    goto(&mut s, relief, hex(AT.0, AT.1));
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&relief].pos, hex(AT.0, AT.1));
    let errors = validate_orders(&next, PlayerId(0), &step(relief));
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn a_city_the_civ_retakes_carries_no_record() {
    let (mut s, c, _) = occupied();
    capture_city(&mut s, c, PlayerId(1), &mut Vec::new());
    assert_eq!(s.cities[&c].owner, PlayerId(1));
    assert_eq!(s.cities[&c].occupied_from, None);
    let again = unit(&mut s, 1, UnitKind::Soldier, hex(AT.0, AT.1));
    assert!(validate_orders(&s, PlayerId(1), &step(again)).is_empty());
}

#[test]
fn a_city_captured_onward_is_keyed_to_the_civ_just_taken_from() {
    let mut s = game(&[0, 1, 2]);
    let c = city(&mut s, 1, hex(AT.0, AT.1), 3);
    capture_city(&mut s, c, PlayerId(0), &mut Vec::new());
    assert_eq!(s.cities[&c].occupied_from, Some(PlayerId(1)));
    capture_city(&mut s, c, PlayerId(2), &mut Vec::new());
    assert_eq!(s.cities[&c].occupied_from, Some(PlayerId(0)));
    // Peace with the founder frees nobody: player 2's unit is locked against player 0.
    let u = unit(&mut s, 2, UnitKind::Soldier, hex(AT.0, AT.1));
    make_peace(&mut s);
    assert!(validate_orders(&s, PlayerId(2), &step(u)).is_empty());
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::War;
    }
    assert_eq!(validate_orders(&s, PlayerId(2), &step(u)).len(), 1);
}

#[test]
fn the_views_warn_before_a_unit_enters_or_captures() {
    let (mut s, c, u) = occupied();
    let view = crate::view::player_view(&s, PlayerId(0));
    let mine = view.cities.iter().find(|v| v.id == c).unwrap();
    assert_eq!(mine.occupation_lock, Some(PlayerId(1)));
    let held = view.units.iter().find(|v| v.id == u).unwrap();
    assert_eq!(held.occupation_lock, Some(PlayerId(1)));
    // A rival city the viewer could capture would lock the capturer.
    let rival = city(&mut s, 1, hex(2, 2), 3);
    unit(&mut s, 0, UnitKind::Soldier, hex(2, 3));
    let view = crate::view::player_view(&s, PlayerId(0));
    let target = view.cities.iter().find(|v| v.id == rival).unwrap();
    assert_eq!(target.occupation_lock, Some(PlayerId(1)));
}

#[test]
fn the_same_seed_and_orders_give_the_same_state_hash() {
    let (s, _, u) = occupied();
    let (a, _) = resolve_units(&s, &orders(0, step(u)));
    let (b, _) = resolve_units(&s, &orders(0, step(u)));
    assert_eq!(a.hash(), b.hash());
}

fn building(s: &mut GameState, c: CityId) {
    let city = s.cities.get_mut(&c).unwrap();
    city.resistance = 0;
    city.producing = Some(crate::ProductionItem::Unit {
        unit: UnitKind::Soldier,
    });
    city.production_stored = 0;
    s.player_mut(PlayerId(0)).gold = 5000;
}

fn stored(s: &GameState, c: CityId) -> i32 {
    s.cities[&c].production_stored
}

fn tick(s: &mut GameState) {
    crate::rules::cities::process_cities(s, &mut Vec::new());
}

fn buy(c: CityId) -> Vec<Order> {
    vec![Order::BuyItem {
        city: c,
        item: crate::ProductionItem::Unit {
            unit: UnitKind::Soldier,
        },
    }]
}

#[test]
fn a_frozen_city_makes_no_progress_while_at_war_but_keeps_its_food() {
    let (mut s, c, _) = occupied();
    building(&mut s, c);
    let food = s.cities[&c].food_stored;
    tick(&mut s);
    assert_eq!(stored(&s, c), 0);
    assert!(s.cities[&c].food_stored > food, "Food still accrues");
    let y = crate::economy::city_totals(&s, &s.cities[&c]);
    assert_eq!(y.production, 0);
    assert!(y.food > 0);
}

#[test]
fn a_frozen_city_refuses_purchases_with_the_reason() {
    let (mut s, c, _) = occupied();
    building(&mut s, c);
    let errors = validate_orders(&s, PlayerId(0), &buy(c));
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].message,
        "Occupied: City 1 builds and buys nothing until Peace with Switzerland"
    );
    // Resolution refuses it too.
    let (next, _) = crate::resolve_turn(&s, &orders(0, buy(c)));
    assert!(next.cities[&c].bought.is_none());
    let view = crate::view::player_view(&s, PlayerId(0));
    assert_eq!(
        view.cities
            .iter()
            .find(|v| v.id == c)
            .unwrap()
            .occupation_lock,
        Some(PlayerId(1))
    );
}

#[test]
fn a_frozen_city_still_resisting_gives_the_occupied_reason_not_the_resistance_one() {
    let (mut s, c, _) = occupied();
    building(&mut s, c);
    s.cities.get_mut(&c).unwrap().resistance = 3;
    let errors = validate_orders(&s, PlayerId(0), &buy(c));
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].message,
        "Occupied: City 1 builds and buys nothing until Peace with Switzerland"
    );
}

#[test]
fn peace_or_elimination_resumes_production_and_buying() {
    for release in [0, 1] {
        let (mut s, c, _) = occupied();
        building(&mut s, c);
        if release == 0 {
            make_peace(&mut s);
        } else {
            s.players[1].alive = false;
        }
        tick(&mut s);
        assert!(stored(&s, c) > 0, "release {release}");
        assert!(
            validate_orders(&s, PlayerId(0), &buy(c)).is_empty(),
            "release {release}"
        );
    }
}

/// The relief chain meets the lock (rules 27): a relief unit that takes over an assaulted occupied
/// city as its garrison is locked there from then on, and the city keeps its record when it holds.
#[test]
fn a_relief_unit_that_becomes_the_garrison_of_an_occupied_city_is_locked() {
    let (mut s, c, garrison) = occupied();
    let at = hex(AT.0, AT.1);
    s.cities.get_mut(&c).unwrap().hp = 0;
    s.units.get_mut(&garrison).unwrap().hp = 4;
    let relief = unit(&mut s, 0, UnitKind::Phalangite, at.neighbor(1));
    goto(&mut s, relief, at);
    for dir in [3, 4, 5] {
        let a = unit(&mut s, 1, UnitKind::Phalangite, at.neighbor(dir));
        goto(&mut s, a, at);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(
        events.iter().any(|e| matches!(e,
            Event::CityReinforced { unit, city, .. } if *unit == relief && *city == c)),
        "the relief takes over: {events:#?}"
    );
    assert_eq!(next.cities[&c].owner, PlayerId(0), "the city holds");
    assert_eq!(next.cities[&c].occupied_from, Some(PlayerId(1)));
    assert_eq!(next.units[&relief].pos, at);
    let errors = validate_orders(&next, PlayerId(0), &step(relief));
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message.contains("is holding City 1 down"));
}

#[test]
fn a_locked_garrison_is_never_a_relief_for_another_city() {
    let (mut s, _, locked) = occupied();
    let other = city(&mut s, 0, hex(AT.0 + 3, AT.1), 3);
    let a = unit(&mut s, 1, UnitKind::Phalangite, hex(AT.0 + 4, AT.1 + 1));
    goto(&mut s, a, hex(AT.0 + 3, AT.1));
    goto(&mut s, locked, hex(AT.0 + 3, AT.1));
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&locked].pos, hex(AT.0, AT.1));
    assert_eq!(next.cities[&other].owner, PlayerId(0));
}

#[test]
fn a_locked_unit_is_held_by_the_turn_and_never_asks_for_orders() {
    let (mut s, _, soldier) = occupied();
    // The civ it was taken from stays alive, or the elimination releases the lock.
    s.player_mut(PlayerId(0)).gold = 500;
    city(&mut s, 1, hex(AT.0 + 8, AT.1 + 8), 3);
    let archer = unit(&mut s, 0, UnitKind::CompositeArcher, hex(AT.0, AT.1));
    let far = unit(&mut s, 0, UnitKind::Soldier, hex(AT.0 + 3, AT.1));
    for id in [soldier, archer] {
        let u = s.units.get_mut(&id).unwrap();
        u.needs_orders = true;
        u.goto = Some(hex(AT.0 + 2, AT.1));
    }
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    let held = &next.units[&soldier];
    assert_eq!(held.standing_order, Some(StandingOrder::Sleep));
    assert_eq!((held.needs_orders, held.goto), (false, None));
    // A ranged unit takes auto-fire so the lock never costs a shot.
    assert_eq!(
        next.units[&archer].standing_order,
        Some(StandingOrder::AutoFire)
    );
    assert!(!next.units[&archer].needs_orders);
    // A unit outside the city is untouched.
    assert_eq!(next.units[&far].standing_order, None);
}

fn set_war(s: &mut GameState) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::War;
    }
}

#[test]
fn peace_clears_the_record_so_a_new_war_neither_locks_nor_freezes() {
    let (mut s, c, u) = occupied();
    building(&mut s, c);
    city(&mut s, 1, hex(12, 12), 3); // keeps the civ alive, so only Peace clears the record
    make_peace(&mut s);
    let (mut s, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(s.cities[&c].occupied_from, None);
    set_war(&mut s);
    assert!(validate_orders(&s, PlayerId(0), &step(u)).is_empty());
    assert_eq!(occupation::city_freeze(&s, &s.cities[&c]), None);
    assert!(validate_orders(&s, PlayerId(0), &buy(c)).is_empty());
}

#[test]
fn a_record_at_war_is_kept_across_turns() {
    let (mut s, c, _) = occupied();
    city(&mut s, 1, hex(12, 12), 3); // keeps the civ alive
    let (s, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(s.cities[&c].occupied_from, Some(PlayerId(1)));
}

#[test]
fn the_civ_being_eliminated_clears_the_record() {
    let (mut s, c, u) = occupied();
    s.players[1].alive = false;
    let (s, _) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(s.cities[&c].occupied_from, None);
    assert!(validate_orders(&s, PlayerId(0), &step(u)).is_empty());
}

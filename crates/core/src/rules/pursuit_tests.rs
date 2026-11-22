//! Regressions for playtest G2: a melee attack targets the unit, not its tile, and a move
//! onto a unit of a team at Peace is never a silent bounce.
use super::*;
use crate::diplomacy::RelationState;
use crate::rules::test_support::*;
use crate::{Order, resolve_turn};

fn move_to(unit: UnitId, to: Hex) -> Order {
    Order::MoveUnit { unit, to }
}

fn two(p0: Vec<Order>, p1: Vec<Order>) -> BTreeMap<PlayerId, Vec<Order>> {
    BTreeMap::from([(PlayerId(0), p0), (PlayerId(1), p1)])
}

fn battles_on(events: &[Event], target: UnitId) -> Vec<Hex> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Battle {
                defender: crate::events::Defender::Unit { unit, .. },
                at,
                ..
            } if *unit == target => Some(*at),
            _ => None,
        })
        .collect()
}

fn pursuit(events: &[Event], attacker: UnitId) -> Option<(PursuitOutcome, Option<Hex>)> {
    events.iter().find_map(|e| match e {
        Event::AttackPursuit {
            unit,
            outcome,
            target_at,
            ..
        } if *unit == attacker => Some((*outcome, *target_at)),
        _ => None,
    })
}

fn at_peace(s: &mut GameState) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::Peace;
    }
}

/// Straight line a -> m -> b, so the attacker's first step toward b is always m.
fn line(start: Hex, dir: usize) -> (Hex, Hex, Hex) {
    let m = start.neighbor(dir);
    (start, m, m.neighbor(dir))
}

#[test]
fn outrider_stationary_horse_is_hit_but_moving_horse_escapes() {
    let mut s = game(&[0, 1]);
    let (a, m, b) = line(hex(3, 5), 0);
    let away = b.neighbor(0);
    let spear = unit(&mut s, 0, UnitKind::Spearman, a);
    let horse = unit(&mut s, 1, UnitKind::Horseman, b);
    s.units.get_mut(&horse).unwrap().fortify = 6;
    let (stationary, hits) = resolve_units(&s, &two(vec![move_to(spear, b)], vec![]));
    assert_eq!(battles_on(&hits, horse), vec![b]);
    assert_eq!(stationary.units[&spear].pos, m);

    let (escaped, events) = resolve_units(
        &s,
        &two(vec![move_to(spear, b)], vec![move_to(horse, away)]),
    );
    assert!(battles_on(&events, horse).is_empty(), "{events:?}");
    assert_eq!(escaped.units[&spear].pos, m);
    assert_eq!(escaped.units[&horse].pos, away);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, reason, .. }
        if *unit == spear && reason == "The Swiss Horseman rode away before your Spearman reached it.")),
        "{events:?}"
    );
    assert_eq!(escaped.units[&spear].attack_target, None);
}

#[test]
fn outrider_adjacent_spear_still_hits_a_leaving_horse() {
    let mut s = game(&[0, 1]);
    let (_, b, away) = line(hex(3, 5), 0);
    let spear = unit(&mut s, 0, UnitKind::Spearman, b);
    let horse = unit(&mut s, 1, UnitKind::Horseman, away);
    let further = away.neighbor(0);
    let (_, events) = resolve_units(
        &s,
        &two(vec![move_to(spear, away)], vec![move_to(horse, further)]),
    );
    assert!(!battles_on(&events, horse).is_empty(), "{events:?}");
}

#[test]
fn outrider_horse_escapes_when_spear_detours_around_a_friend() {
    let mut s = game(&[0, 1]);
    let (a, m, b) = line(hex(3, 5), 0);
    let spear = unit(&mut s, 0, UnitKind::Spearman, a);
    unit(&mut s, 0, UnitKind::Spearman, m);
    let horse = unit(&mut s, 1, UnitKind::Horseman, b);
    let away = b.neighbor(0);
    let (next, events) = resolve_units(
        &s,
        &two(vec![move_to(spear, b)], vec![move_to(horse, away)]),
    );
    assert!(battles_on(&events, horse).is_empty(), "{events:?}");
    assert_eq!(next.units[&horse].pos, away);
    assert_eq!(next.units[&spear].pos.distance(a), 1);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, reason, .. }
        if *unit == spear && reason.contains("rode away"))),
        "{events:?}"
    );
}

#[test]
fn g2_target_that_sidesteps_within_reach_is_attacked_where_it_ends() {
    // Before G2 the Spearman walked into the vacated tile and no battle happened.
    let mut s = game(&[0, 1]);
    let (a, m, b) = line(hex(3, 5), 0);
    let c = m.neighbor(1);
    assert_eq!((b.distance(c), m.distance(c)), (1, 1));
    let spearman = unit(&mut s, 0, UnitKind::Spearman, a);
    let scout = unit(&mut s, 1, UnitKind::Scout, b);
    let (next, events) = resolve_units(
        &s,
        &two(vec![move_to(spearman, b)], vec![move_to(scout, c)]),
    );
    assert_eq!(battles_on(&events, scout), vec![c], "{events:?}");
    assert_eq!(
        pursuit(&events, spearman),
        Some((PursuitOutcome::Attacked, Some(c)))
    );
    assert_eq!(next.units[&spearman].pos, m, "attacking ends movement");
    if next.units.contains_key(&scout) {
        // One attack per order (G18): it stays where it fought and needs orders.
        let u = &next.units[&spearman];
        assert_eq!((u.goto, u.attack_target), (None, None));
        assert!(u.path.is_empty());
    }
}

#[test]
fn g2_pursuit_is_the_same_from_either_seat() {
    // Pillar 7: swapping who attacks (seat and unit ids) gives the mirror result.
    for (atk, def) in [(0u8, 1u8), (1, 0)] {
        let mut s = game(&[0, 1]);
        let (a, m, b) = line(hex(3, 5), 0);
        let c = m.neighbor(1);
        let (target, spearman) = if atk == 1 {
            let t = unit(&mut s, def, UnitKind::Scout, b);
            (t, unit(&mut s, atk, UnitKind::Spearman, a))
        } else {
            let so = unit(&mut s, atk, UnitKind::Spearman, a);
            (unit(&mut s, def, UnitKind::Scout, b), so)
        };
        let mut orders = BTreeMap::new();
        orders.insert(PlayerId(atk), vec![move_to(spearman, b)]);
        orders.insert(PlayerId(def), vec![move_to(target, c)]);
        let (_, events) = resolve_units(&s, &orders);
        assert_eq!(battles_on(&events, target), vec![c], "attacker seat {atk}");
    }
}

#[test]
fn g18_target_that_outruns_the_attacker_is_followed_then_the_order_ends() {
    let mut s = game(&[0, 1]);
    let (a, _, b) = line(hex(2, 5), 0);
    let far = b.neighbor(0).neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, a);
    let scout = unit(&mut s, 1, UnitKind::Scout, b);
    let (s1, e1) = resolve_units(
        &s,
        &two(vec![move_to(spearman, b)], vec![move_to(scout, far)]),
    );
    assert!(battles_on(&e1, scout).is_empty());
    assert_eq!(s1.units[&scout].pos, far);
    assert_eq!(
        pursuit(&e1, spearman),
        Some((PursuitOutcome::Followed, Some(far)))
    );
    let u = &s1.units[&spearman];
    assert_eq!(u.pos, b, "followed into the vacated tile");
    assert_eq!(
        (u.goto, u.attack_target),
        (None, None),
        "an attack lasts one turn: the unit needs orders (G18)"
    );
    // Next turn, with no new order, it does not attack.
    let (s2, e2) = resolve_units(&s1, &no_orders());
    assert!(battles_on(&e2, scout).is_empty());
    assert_eq!(s2.units[&spearman].pos, b);
}

#[test]
fn g2_target_that_leaves_sight_ends_the_order_and_says_so() {
    let mut s = game(&[0, 1]);
    let (a, _, b) = line(hex(1, 5), 0);
    let gone = b.neighbor(0).neighbor(0).neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, a);
    let scout = unit(&mut s, 1, UnitKind::Scout, b);
    let (s1, e1) = resolve_units(
        &s,
        &two(vec![move_to(spearman, b)], vec![move_to(scout, gone)]),
    );
    assert_eq!(s1.units[&scout].pos, gone);
    assert_eq!(
        pursuit(&e1, spearman),
        Some((PursuitOutcome::LostSight, None))
    );
    let u = &s1.units[&spearman];
    assert_eq!((u.goto, u.attack_target), (None, None), "no dangling route");
    assert!(battles_on(&e1, scout).is_empty());
}

#[test]
fn g2_pursuit_never_attacks_an_escort_it_was_not_ordered_to_attack() {
    let mut s = game(&[0, 1]);
    let (a, m, b) = line(hex(3, 5), 0);
    let c = m.neighbor(1);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, a);
    let settler = unit(&mut s, 1, UnitKind::Settler, b);
    let escort = unit(&mut s, 1, UnitKind::Spearman, c);
    let (next, events) = resolve_units(
        &s,
        &two(vec![move_to(spearman, b)], vec![move_to(settler, c)]),
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::Battle { .. })),
        "{events:?}"
    );
    assert_eq!(next.units[&escort].hp, 100);
    assert_eq!(next.units[&settler].owner, PlayerId(1));
    assert_eq!(
        pursuit(&events, spearman),
        Some((PursuitOutcome::Followed, Some(c)))
    );
}

#[test]
fn g2_plain_move_never_pursues() {
    // An empty destination is a move, not an attack order: no target to follow.
    let mut s = game(&[0, 1]);
    let (a, m, b) = line(hex(3, 5), 0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, a);
    let scout = unit(&mut s, 1, UnitKind::Scout, m.neighbor(1).neighbor(1));
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![move_to(spearman, b)],
            vec![move_to(scout, m.neighbor(1))],
        ),
    );
    assert_eq!(next.units[&spearman].attack_target, None);
    assert!(pursuit(&events, spearman).is_none());
}

#[test]
fn g2_ranged_fire_resolves_before_the_target_can_move() {
    // Ranged attacks resolve from the turn-start snapshot (phase 2), so a target cannot slip
    // out of range before the shot.
    let mut s = game(&[0, 1]);
    let (a, _, b) = line(hex(3, 5), 0);
    let archer = unit(&mut s, 0, UnitKind::Archer, a);
    let scout = unit(&mut s, 1, UnitKind::Scout, b);
    let away = b.neighbor(0).neighbor(0);
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![Order::RangedAttack {
                unit: archer,
                target: b,
            }],
            vec![move_to(scout, away)],
        ),
    );
    assert_eq!(battles_on(&events, scout), vec![b]);
    if let Some(sc) = next.units.get(&scout) {
        assert!(sc.hp < 100);
    }
}

#[test]
fn g2_move_onto_a_peaceful_unit_names_it_in_the_report() {
    // The playtest case: the Spearman was ordered onto an adjacent Scout of a team at Peace.
    // The report identifies the peaceful unit that blocks the move.
    let mut s = game(&[0, 1]);
    at_peace(&mut s);
    let here = hex(3, 5);
    let there = here.neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let scout = unit(&mut s, 1, UnitKind::Scout, there);
    let (next, events) = resolve_units(
        &s,
        &two(
            vec![move_to(spearman, there)],
            vec![move_to(scout, there.neighbor(1))],
        ),
    );
    assert_eq!(
        next.units[&spearman].attack_target, None,
        "no War: no attack"
    );
    let want = format!("{PEACE_BLOCK_PREFIX}P1's Scout");
    assert!(
        events.iter().any(|e| matches!(e, Event::MoveBlocked { unit, reason, .. } if *unit == spearman && *reason == want)),
        "{events:?}"
    );
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
}

/// A full-turn fixture (diplomacy resolves at the turn boundary), with a city per empire.
fn peace_game() -> GameState {
    let mut s = game(&[0, 1]);
    at_peace(&mut s);
    city(&mut s, 0, hex(1, 1), 1);
    city(&mut s, 1, hex(10, 8), 1);
    s
}

#[test]
fn g20_ranged_attack_on_city_validates_and_fires_with_same_turn_surprise_war() {
    let mut s = peace_game();
    let target = hex(10, 8);
    let archer = unit(&mut s, 0, UnitKind::Archer, hex(8, 8));
    crate::visibility::update_explored(&mut s);
    let shot = Order::RangedAttack {
        unit: archer,
        target,
    };
    let war = Order::DeclareWar {
        against: TeamId(1),
        breach: false,
    };
    // Without the declaration the error says why (Peace), not that nothing is there.
    let errors = crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&shot));
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains("not at War"),
        "{:?}",
        errors[0].message
    );
    let empty = Order::RangedAttack {
        unit: archer,
        target: hex(9, 9),
    };
    let errors = crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&empty));
    assert_eq!(
        errors.first().map(|e| e.message.as_str()),
        Some("no enemy unit or city to attack there")
    );
    let orders = orders(0, vec![war, shot]);
    assert!(crate::validate_orders(&s, PlayerId(0), &orders[&PlayerId(0)]).is_empty());
    let (_, events) = resolve_turn(&s, &orders);
    assert!(
        events.iter().any(|e| matches!(e, Event::Battle {
        attacker, defender: crate::events::Defender::City { .. }, ..
    } if *attacker == archer)),
        "{events:?}"
    );
}

fn war_declared(events: &[Event]) -> Option<u32> {
    events.iter().find_map(|e| match e {
        Event::WarDeclared { relation } => Some(relation.effective_turn),
        _ => None,
    })
}

#[test]
fn g20_declaring_war_at_plain_peace_attacks_this_turn() {
    // Civ 6 surprise war: no treaty, so the War starts now and the attack resolves now.
    let mut s = peace_game();
    let here = hex(4, 5);
    let there = here.neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let scout = unit(&mut s, 1, UnitKind::Scout, there);
    let step = there.neighbor(1);
    let orders = two(
        vec![
            Order::DeclareWar {
                against: TeamId(1),
                breach: false,
            },
            move_to(spearman, there),
        ],
        vec![move_to(scout, step)],
    );
    let (s1, e1) = resolve_turn(&s, &orders);
    assert!(
        !e1.iter().any(|e| matches!(e, Event::OrderFailed { .. })
            || matches!(e, Event::MoveBlocked { unit, .. } if *unit == spearman)),
        "{e1:?}"
    );
    assert_eq!(war_declared(&e1), Some(s.turn), "War starts this turn");
    let declared_at = e1
        .iter()
        .position(|e| matches!(e, Event::WarDeclared { .. }))
        .unwrap();
    let battle_at = e1
        .iter()
        .position(|e| matches!(e, Event::Battle { .. }))
        .expect("the attack resolves this turn");
    assert!(declared_at < battle_at, "declared before the battle");
    assert!(!battles_on(&e1, scout).is_empty(), "{e1:?}");
    assert_ne!(
        pursuit(&e1, spearman).map(|p| p.0),
        Some(PursuitOutcome::AwaitingWar)
    );
    assert!(s1.at_war(PlayerId(0), PlayerId(1)));
    if s1.units.contains_key(&scout) {
        let u = &s1.units[&spearman];
        assert_eq!((u.goto, u.attack_target), (None, None), "one attack (G18)");
    }
}

fn sign_alliance(s: &mut GameState) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::Alliance;
        r.security = Some(crate::diplomacy::SecurityTreaty {
            id: 1,
            started_turn: 0,
            protected_until: 0,
            coalition_target: None,
        });
    }
}

#[test]
fn g20_war_on_an_ally_still_holds_the_attack_for_the_first_turn_of_war() {
    let mut s = peace_game();
    sign_alliance(&mut s);
    let here = hex(4, 5);
    let there = here.neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let scout = unit(&mut s, 1, UnitKind::Scout, there);
    let step = there.neighbor(1);
    let orders = two(
        vec![
            Order::DeclareWar {
                against: TeamId(1),
                breach: true,
            },
            move_to(spearman, there),
        ],
        vec![move_to(scout, step)],
    );
    let (s1, e1) = resolve_turn(&s, &orders);
    assert_eq!(
        war_declared(&e1),
        Some(s.turn + 1),
        "war on an ally takes effect next turn"
    );
    assert!(battles_on(&e1, scout).is_empty());
    assert_eq!(s1.units[&spearman].pos, here, "holds until War begins");
    assert_eq!(
        pursuit(&e1, spearman),
        Some((PursuitOutcome::AwaitingWar, Some(step)))
    );
    let u = &s1.units[&spearman];
    assert_eq!((u.goto, u.attack_target), (Some(step), Some(scout)));
    // War has begun: the held attack fires once at the Scout's new tile.
    let (s2, e2) = resolve_turn(&s1, &no_orders());
    assert_eq!(battles_on(&e2, scout), vec![step], "{e2:?}");
    let (_, e3) = resolve_turn(&s2, &no_orders());
    assert!(battles_on(&e3, scout).is_empty(), "one attack (G18)");
}

#[test]
fn g2_attack_on_a_peaceful_unit_without_declaring_war_does_not_wait() {
    let mut s = peace_game();
    let here = hex(4, 5);
    let there = here.neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let _scout = unit(&mut s, 1, UnitKind::Scout, there);
    let (s1, e1) = resolve_turn(&s, &orders(0, vec![move_to(spearman, there)]));
    assert_eq!(s1.units[&spearman].attack_target, None);
    assert!(pursuit(&e1, spearman).is_none());
    assert!(e1.iter().any(
        |e| matches!(e, Event::MoveBlocked { reason, .. } if reason.starts_with(PEACE_BLOCK_PREFIX))
    ));
}

#[test]
fn g2_an_attack_declared_this_turn_may_target_a_unit_inside_closed_borders() {
    let mut s = peace_game();
    let their_land = hex(9, 8);
    let here = their_land.neighbor(3).neighbor(3);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let _guard = unit(&mut s, 1, UnitKind::Spearman, their_land);
    for h in their_land.within(1) {
        if let Some(t) = s.map.get_mut(h) {
            t.owner = Some(PlayerId(1));
        }
    }
    crate::visibility::update_explored(&mut s);
    let attack = move_to(spearman, their_land);
    assert_eq!(
        crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&attack)).len(),
        1,
        "at Peace the borders are closed"
    );
    let declare = Order::DeclareWar {
        against: TeamId(1),
        breach: false,
    };
    assert_eq!(
        crate::validate_orders(&s, PlayerId(0), &[declare, attack]),
        vec![]
    );
}

#[test]
fn g20_declaring_war_with_a_move_onto_their_city_marches_now_and_attacks_once() {
    let mut s = peace_game();
    let their_city = hex(10, 8);
    let here = their_city.neighbor(3).neighbor(3);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    crate::visibility::update_explored(&mut s);
    let orders = orders(
        0,
        vec![
            Order::DeclareWar {
                against: TeamId(1),
                breach: false,
            },
            move_to(spearman, their_city),
        ],
    );
    assert_eq!(
        crate::validate_orders(&s, PlayerId(0), &orders[&PlayerId(0)]),
        vec![]
    );
    let (s1, e1) = resolve_turn(&s, &orders);
    assert!(
        !e1.iter()
            .any(|e| matches!(e, Event::MoveBlocked { .. } | Event::OrderFailed { .. })),
        "{e1:?}"
    );
    assert_ne!(
        s1.units[&spearman].pos, here,
        "War opened the borders this turn"
    );
    let (s2, e2) = resolve_turn(&s1, &no_orders());
    let (s3, e3) = resolve_turn(&s2, &no_orders());
    let attacks = |events: &[Event]| {
        events
            .iter()
            .filter(|e| {
                matches!(e, Event::Battle {
                attacker,
                defender: crate::events::Defender::City { .. },
                ..
            } if *attacker == spearman)
            })
            .count()
    };
    // The march reaches the city and attacks it once (G18).
    assert_eq!(
        attacks(&e1) + attacks(&e2) + attacks(&e3),
        1,
        "{e1:?} {e2:?} {e3:?}"
    );
    // The city fires back each turn (city strike), so the Spearman may not survive.
    assert!(
        s3.units.get(&spearman).is_none_or(|u| u.goto.is_none()),
        "the attack ended the order"
    );
}

// Playtest G18: an attack order is one attack. The user's Spearman attacked a camp guard on
// turn 17, kept its goto on the guard's tile and re-attacked on turns 18 and 19 with no new
// orders until it died.

fn fought(events: &[Event], attacker: UnitId) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Battle { attacker: a, .. } if *a == attacker))
        .count()
}

#[test]
fn g18_an_attack_whose_defender_survives_ends_the_order_and_never_repeats() {
    let mut s = game(&[0, 1]);
    let here = hex(4, 5);
    let there = here.neighbor(0);
    set_terrain(&mut s, there, crate::types::Terrain::Hills);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, here);
    let guard = unit(&mut s, 1, UnitKind::Spearman, there);
    s.units.get_mut(&guard).unwrap().fortify = 6;
    let (s1, e1) = resolve_units(&s, &orders(0, vec![move_to(spearman, there)]));
    assert_eq!(fought(&e1, spearman), 1);
    assert!(s1.units.contains_key(&guard), "the guard survived");
    let u = &s1.units[&spearman];
    assert_eq!(u.pos, here, "stays where it fought");
    assert_eq!((u.goto, u.attack_target), (None, None), "needs orders");
    assert!(u.path.is_empty() && !u.path_stop_on_block);
    // No new orders: no second attack, on this turn or the next.
    let (s2, e2) = resolve_units(&s1, &no_orders());
    let (_, e3) = resolve_units(&s2, &no_orders());
    assert_eq!(fought(&e2, spearman) + fought(&e3, spearman), 0);
}

#[test]
fn g18_a_meeting_both_survive_ends_both_orders() {
    let mut s = game(&[0, 1]);
    let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
    let b = unit(&mut s, 1, UnitKind::Spearman, hex(5, 4));
    goto(&mut s, a, hex(8, 4));
    goto(&mut s, b, hex(0, 4));
    let (s1, e1) = resolve_units(&s, &no_orders());
    assert_eq!(fought(&e1, a) + fought(&e1, b), 1, "{e1:?}");
    for id in [a, b] {
        assert_eq!(s1.units[&id].goto, None, "the clash ended the move");
    }
}

#[test]
fn g18_an_attack_on_a_city_that_stands_ends_the_order() {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(6, 5), 3);
    let at = s.cities[&c].pos;
    let spearman = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(3));
    crate::visibility::update_explored(&mut s);
    let (s1, e1) = resolve_units(&s, &orders(0, vec![move_to(spearman, at)]));
    assert_eq!(fought(&e1, spearman), 1);
    assert!(s1.cities[&c].hp > 0);
    assert_eq!(s1.units[&spearman].goto, None);
    let (_, e2) = resolve_units(&s1, &no_orders());
    assert_eq!(fought(&e2, spearman), 0);
}

#[test]
fn g18_a_target_out_of_reach_this_turn_is_approached_then_the_order_ends() {
    // An attack never carries over: the Spearman moves toward it and asks for orders.
    let mut s = game(&[0, 1]);
    let start = hex(2, 5);
    let target = start.neighbor(0).neighbor(0).neighbor(0).neighbor(0);
    let spearman = unit(&mut s, 0, UnitKind::Spearman, start);
    let guard = unit(&mut s, 1, UnitKind::Spearman, target);
    // A spotter off the route keeps the target in sight, so the order is an attack.
    unit(&mut s, 0, UnitKind::Scout, target.neighbor(2).neighbor(2));
    crate::visibility::update_explored(&mut s);
    let (s1, e1) = resolve_units(&s, &orders(0, vec![move_to(spearman, target)]));
    assert_eq!(fought(&e1, spearman), 0);
    assert_eq!(
        pursuit(&e1, spearman),
        Some((PursuitOutcome::OutOfReach, Some(target)))
    );
    let u = &s1.units[&spearman];
    assert!(
        u.pos.distance(target) < start.distance(target),
        "approached"
    );
    assert_eq!((u.goto, u.attack_target), (None, None));
    let (_, e2) = resolve_units(&s1, &no_orders());
    assert!(battles_on(&e2, guard).is_empty());
}

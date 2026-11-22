//! Regressions for playtest F8 (own traffic) and F12 (capturable cities).
use super::*;
use crate::rules::test_support::*;
use crate::{AttackMode, Order, forecast_attack_mode, player_view};

#[test]
fn f8_two_rangers_with_clean_alternatives_both_arrive() {
    let mut s = game(&[0, 1]);
    let middle = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Archer, middle.neighbor(0));
    let b = unit(&mut s, 0, UnitKind::Archer, middle.neighbor(5));
    let goals = [middle.neighbor(2), middle.neighbor(1)];
    for (u, to) in [(a, goals[0]), (b, goals[1])] {
        assert_eq!(
            path::land_path(&s.map, s.units[&u].pos, to).unwrap()[0],
            middle
        );
        goto(&mut s, u, to);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!([next.units[&a].pos, next.units[&b].pos], goals);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Bounce { .. } | Event::MoveBlocked { .. }))
    );
    assert_eq!(resolve_units(&s, &no_orders()), (next, events));
}

#[test]
fn own_traffic_takes_turns_through_a_single_hex_choke() {
    let mut s = game(&[0, 1]);
    let middle = hex(5, 5);
    let starts = [middle.neighbor(0), middle.neighbor(5)];
    let goals = [middle.neighbor(2), middle.neighbor(3)];
    for tile in &mut s.map.tiles {
        tile.terrain = Terrain::Mountains;
    }
    for h in starts.into_iter().chain(goals).chain([middle]) {
        set_terrain(&mut s, h, Terrain::Grassland);
    }
    let a = unit(&mut s, 0, UnitKind::Archer, starts[0]);
    let b = unit(&mut s, 0, UnitKind::Archer, starts[1]);
    goto(&mut s, a, goals[0]);
    goto(&mut s, b, goals[1]);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!([next.units[&a].pos, next.units[&b].pos], goals);
    let through: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::UnitMoved { unit, to, .. } if *to == middle => Some(*unit),
            _ => None,
        })
        .collect();
    assert_eq!(through, [a, b]);
}

#[test]
fn allied_unit_is_transit_only_in_a_single_hex_choke() {
    let mut s = game(&[0, 1]);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = crate::diplomacy::RelationState::Alliance;
    }
    let middle = hex(5, 5);
    let start = middle.neighbor(0);
    let beyond = middle.neighbor(3);
    for tile in &mut s.map.tiles {
        tile.terrain = Terrain::Mountains;
    }
    for h in [start, middle, beyond] {
        set_terrain(&mut s, h, Terrain::Grassland);
    }
    let ally = unit(&mut s, 1, UnitKind::Spearman, middle);
    let mover = unit(&mut s, 0, UnitKind::Spearman, start);
    goto(&mut s, mover, beyond);
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&mover].pos, beyond);
    assert_eq!(next.units[&ally].pos, middle);

    goto(&mut s, mover, middle);
    let (blocked, _) = resolve_units(&s, &no_orders());
    assert_ne!(blocked.units[&mover].pos, middle);
}

#[test]
fn same_owner_shared_final_destination_terminates_without_stacking() {
    let mut s = game(&[0, 1]);
    let target = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Archer, target.neighbor(0));
    let b = unit(&mut s, 0, UnitKind::Archer, target.neighbor(3));
    goto(&mut s, a, target);
    goto(&mut s, b, target);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, target);
    assert_ne!(next.units[&b].pos, target);
    assert!(events.len() < 10);
}

#[test]
fn allied_city_is_transit_only_and_an_existing_occupant_can_leave() {
    let mut s = game(&[0, 1]);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = crate::diplomacy::RelationState::Alliance;
    }
    let city_pos = hex(5, 5);
    let city = city(&mut s, 1, city_pos, 3);
    let start = city_pos.neighbor(0);
    let beyond = city_pos.neighbor(3);
    let scout = unit(&mut s, 0, UnitKind::Scout, start);
    let transit = Order::MoveUnitPath {
        unit: scout,
        to: beyond,
        path: vec![city_pos, beyond],
        stop_on_block: false,
    };
    assert!(crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&transit)).is_empty());

    let stop = Order::MoveUnit {
        unit: scout,
        to: city_pos,
    };
    assert_eq!(
        crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&stop))[0].message,
        format!(
            "Allied units can pass through {} but can't stop there.",
            s.cities[&city].name
        )
    );

    s.units.get_mut(&scout).unwrap().pos = city_pos;
    assert!(crate::scenario::validate_state(&s).is_empty());
    let leave = Order::MoveUnit {
        unit: scout,
        to: beyond,
    };
    assert!(crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&leave)).is_empty());
    let (next, _) = resolve_units(&s, &orders(0, vec![leave]));
    assert_eq!(next.units[&scout].pos, beyond);
}

#[test]
fn a_unit_never_stops_in_an_allied_city_when_it_cannot_pay_for_the_exit() {
    let mut s = game(&[0, 1]);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = crate::diplomacy::RelationState::Alliance;
    }
    let city_pos = hex(5, 5);
    city(&mut s, 1, city_pos, 3);
    let start = city_pos.neighbor(0);
    let beyond = city_pos.neighbor(3);
    let scout = unit(&mut s, 0, UnitKind::Scout, start);
    s.units.get_mut(&scout).unwrap().moves_left = 1;
    let move_through = Order::MoveUnitPath {
        unit: scout,
        to: beyond,
        path: vec![city_pos, beyond],
        stop_on_block: false,
    };
    let (next, events) = resolve_units(&s, &orders(0, vec![move_through]));
    assert_ne!(next.units[&scout].pos, city_pos);
    assert!(!events.iter().any(|event| matches!(event,
        Event::UnitMoved { unit, to, .. } if *unit == scout && *to == city_pos
    )));
}

fn contested_city(holding: bool, defender_hp: i32) -> (GameState, CityId, UnitId, UnitId) {
    let mut s = game(&[0, 1]);
    let t = hex(5, 5);
    let c = city(&mut s, 1, t, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let a = unit(&mut s, 0, UnitKind::Pikeman, t.neighbor(0));
    let b = unit(
        &mut s,
        1,
        UnitKind::Spearman,
        if holding { t } else { t.neighbor(3) },
    );
    s.units.get_mut(&b).unwrap().hp = defender_hp;
    goto(&mut s, a, t);
    if !holding {
        goto(&mut s, b, t);
    }
    visibility::update_explored(&mut s);
    (s, c, a, b)
}

#[test]
fn f12_incoming_defender_fights_and_surviving_winner_captures() {
    let (s, c, a, b) = contested_city(false, 10);
    let (next, events) = resolve_units(&s, &no_orders());
    let fights: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::Battle { .. }))
        .collect();
    // The incoming defender reinforces its own city first (user, 2026-10-04), so the attack
    // fights it as the garrison, a melee rather than a meeting; the ruin absorbs nothing.
    assert_eq!(fights.len(), 1);
    assert!(matches!(
        fights[0],
        Event::Battle {
            meeting: false,
            damage_to_attacker: 1..,
            damage_to_defender: 1..,
            ..
        }
    ));
    assert!(!next.units.contains_key(&b));
    assert!(next.units[&a].hp > 0 && next.units[&a].hp < 100);
    assert_eq!(next.units[&a].pos, next.cities[&c].pos);
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    assert_eq!(resolve_units(&s, &no_orders()), (next, events));
}

#[test]
fn f12_garrison_battle_matches_melee_forecast_and_captures_on_kill() {
    let (mut s, c, a, b) = contested_city(true, 10);
    let target = s.cities[&c].pos;
    set_terrain(&mut s, target, Terrain::Hills);
    s.units.get_mut(&b).unwrap().fortify = 6;
    let forecast = forecast_attack_mode(
        &player_view(&s, PlayerId(0)),
        a,
        s.cities[&c].pos,
        AttackMode::Melee,
    )
    .unwrap();
    let (next, events) = resolve_units(&s, &no_orders());
    // City strike is default, but a 0-HP city doesn't fire: the only battle is the melee.
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Battle { .. }))
            .count(),
        1
    );
    let fight = events
        .iter()
        .find(|e| matches!(e, Event::Battle { .. }))
        .unwrap();
    let Event::Battle {
        attacker_strength,
        defender_strength,
        meeting,
        ..
    } = fight
    else {
        unreachable!()
    };
    assert!(!meeting);
    assert_eq!(*attacker_strength, forecast.attacker_strength);
    assert_eq!(*defender_strength, forecast.defender_strength);
    assert_eq!(forecast.win_chance, 100);
    assert!(!next.units.contains_key(&b));
    assert_eq!(next.cities[&c].owner, PlayerId(0));
}

#[test]
fn contested_zero_hp_city_cannot_repeat_a_no_damage_stalemate() {
    for holding in [false, true] {
        let (mut s, c, a, b) = contested_city(holding, 100);
        let mut resolved = false;
        let target = s.cities[&c].pos;
        for turn in 0..10 {
            if turn > 0 {
                // One attack per order (G18): both sides order the contest again each turn.
                goto(&mut s, a, target);
                if !holding {
                    goto(&mut s, b, target);
                }
            }
            let hp = s.units[&a].hp + s.units[&b].hp;
            let (next, events) = resolve_units(&s, &no_orders());
            assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
            let remaining: i32 = [a, b]
                .iter()
                .filter_map(|u| next.units.get(u).map(|u| u.hp))
                .sum();
            assert!(remaining < hp, "every contested turn deals real damage");
            if !next.units.contains_key(&a) || !next.units.contains_key(&b) {
                if next.units.contains_key(&a) {
                    assert_eq!(next.cities[&c].owner, PlayerId(0));
                }
                resolved = true;
                break;
            }
            s = next;
        }
        assert!(
            resolved,
            "combat must finish within a bounded number of turns"
        );
    }
}

#[test]
fn mutual_kill_does_not_capture_and_two_own_survivors_do_not_block_capture() {
    let (mut s, c, a, b) = contested_city(false, 1);
    s.units.get_mut(&a).unwrap().hp = 1;
    let (next, _) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&a) && !next.units.contains_key(&b));
    assert_eq!(next.cities[&c].owner, PlayerId(1));

    let (mut s, c, a, _) = contested_city(false, 10);
    let target = s.cities[&c].pos;
    let ally = unit(&mut s, 0, UnitKind::Spearman, target.neighbor(1));
    goto(&mut s, ally, target);
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    // The garrison fell to the two attackers; one of them takes the city, the other waits.
    let inside = [a, ally]
        .into_iter()
        .filter(|u| next.units[u].pos == target)
        .count();
    assert_eq!(inside, 1);
}

#[test]
fn confirmed_route_survives_save_load_and_follows_each_previewed_step() {
    let mut s = game(&[0, 1]);
    let start = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Archer, start);
    // Deliberate three-step detour to a tile only one step from the start.
    let path = vec![
        start.neighbor(2),
        start.neighbor(2).neighbor(0),
        start.neighbor(0),
    ];
    let to = *path.last().unwrap();
    visibility::update_explored(&mut s);
    let os = orders(
        0,
        vec![Order::MoveUnitPath {
            unit: a,
            to,
            path: path.clone(),
            stop_on_block: false,
        }],
    );
    assert!(crate::validate_orders(&s, PlayerId(0), &os[&PlayerId(0)]).is_empty());
    let (first, mut events) = resolve_units(&s, &os);
    assert_eq!(first.units[&a].path, path[2..]);
    let saved: GameState = serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
    let (last, next_events) = resolve_units(&saved, &no_orders());
    events.extend(next_events);
    let walked: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::UnitMoved { unit, to, .. } if *unit == a => Some(*to),
            _ => None,
        })
        .collect();
    assert_eq!(walked, path);
    assert_eq!(last.units[&a].pos, to);
    assert!(last.units[&a].path.is_empty());
}

#[test]
fn confirmed_routes_are_private_and_validation_cannot_probe_fog() {
    let mut s = game(&[0, 1]);
    let (start, target) = (hex(5, 5), hex(6, 5));
    let a = unit(&mut s, 0, UnitKind::Archer, start);
    let b = unit(&mut s, 1, UnitKind::Spearman, target);
    s.units.get_mut(&b).unwrap().path = vec![hex(7, 5)];
    goto(&mut s, b, hex(7, 5));
    visibility::update_explored(&mut s);
    assert!(
        player_view(&s, PlayerId(0))
            .units
            .iter()
            .find(|u| u.id == b)
            .unwrap()
            .path
            .is_empty()
    );
    let before = forecast_attack_mode(&player_view(&s, PlayerId(0)), a, target, AttackMode::Melee);
    goto(&mut s, b, start);
    assert_eq!(
        before,
        forecast_attack_mode(&player_view(&s, PlayerId(0)), a, target, AttackMode::Melee),
        "enemy intentions never enter the forecast"
    );
    let bad = Order::MoveUnitPath {
        unit: a,
        to: hex(10, 9),
        path: vec![hex(10, 9)],
        stop_on_block: false,
    };
    assert!(!crate::validate_orders(&s, PlayerId(0), &[bad]).is_empty());
    let hidden = hex(10, 9);
    let route = path::land_path(&s.map, start, hidden).unwrap();
    let os = [Order::MoveUnitPath {
        unit: a,
        to: hidden,
        path: route,
        stop_on_block: false,
    }];
    let errors = crate::validate_orders(&s, PlayerId(0), &os);
    assert!(!errors.is_empty());
    set_terrain(&mut s, hidden, Terrain::Mountains);
    assert_eq!(errors, crate::validate_orders(&s, PlayerId(0), &os));
}

#[test]
fn automated_scout_waits_for_own_traffic_without_skipping_its_next_step() {
    let mut s = game(&[0, 1]);
    let middle = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Archer, middle.neighbor(0));
    let b = unit(&mut s, 0, UnitKind::Scout, middle.neighbor(5));
    visibility::update_explored(&mut s);
    let goals = [middle.neighbor(2), middle.neighbor(3)];
    let os = orders(
        0,
        vec![
            Order::MoveUnitPath {
                unit: a,
                to: goals[0],
                path: vec![middle, goals[0]],
                stop_on_block: false,
            },
            Order::SetAutomation {
                unit: b,
                automation: Some(Automation::Explore),
                path: vec![middle, goals[1]],
                stop_reason: None,
            },
        ],
    );
    assert!(crate::validate_orders(&s, PlayerId(0), &os[&PlayerId(0)]).is_empty());
    let (next, events) = resolve_units(&s, &os);
    assert_eq!([next.units[&a].pos, next.units[&b].pos], goals);
    assert_eq!(next.units[&b].automation, Some(Automation::Explore));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::AutomationStopped { .. }))
    );
}

#[test]
fn manual_confirmed_path_cancels_auto_explore() {
    let mut s = game(&[0, 1]);
    let start = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Scout, start);
    s.units.get_mut(&a).unwrap().automation = Some(Automation::Explore);
    visibility::update_explored(&mut s);
    let to = start.neighbor(0).neighbor(0);
    let os = orders(
        0,
        vec![Order::MoveUnitPath {
            unit: a,
            to,
            path: vec![start.neighbor(0), to],
            stop_on_block: false,
        }],
    );
    let (next, _) = resolve_units(&s, &os);
    assert_eq!(next.units[&a].pos, to);
    assert_eq!(next.units[&a].automation, None);
}

// Review regressions: first submit a legal route, then change its next tile between turns.
#[test]
fn constrained_route_keeps_its_goal_until_replaced_after_a_block_and_save_load() {
    let mut s = game(&[0, 1]);
    let a = unit(&mut s, 0, UnitKind::Settler, hex(2, 4));
    for i in 0..s.map.len() {
        s.players[0].explored.set(i);
    }
    let goal = hex(8, 4);
    let order = Order::MoveUnitPath {
        unit: a,
        to: goal,
        path: (3..=8).map(|col| hex(col, 4)).collect(),
        stop_on_block: true,
    };
    let order = serde_json::from_str(&serde_json::to_string(&order).unwrap()).unwrap();
    s = resolve_units(&s, &orders(0, vec![order])).0;
    let at = s.units[&a].pos;
    let remaining = s.units[&a].path.clone();
    // Two friends in a row: crossing both costs more than a Settler's moves, so the
    // confirmed step cannot hop over them and the route stops (one friend would be crossed).
    unit(&mut s, 0, UnitKind::Settler, remaining[0]);
    unit(&mut s, 0, UnitKind::Settler, remaining[1]);
    s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    s = resolve_units(&s, &no_orders()).0;
    let u = &s.units[&a];
    assert_eq!(u.pos, at);
    assert_eq!(u.goto, Some(goal));
    assert_eq!(u.path, remaining);
    assert!(u.path_stop_on_block && u.needs_orders);
    s = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    // An explicit destination-only replacement opts back into ordinary rerouting.
    s = resolve_units(&s, &orders(0, vec![Order::MoveUnit { unit: a, to: goal }])).0;
    assert_ne!(s.units[&a].pos, at);
    assert!(!s.units[&a].path_stop_on_block);
    for _ in 0..4 {
        s = resolve_units(&s, &no_orders()).0;
    }
    assert_eq!(s.units[&a].pos, goal);
    assert_eq!(s.units[&a].goto, None);
}

#[test]
fn constrained_route_blocked_by_allied_settler_requests_orders_then_stops() {
    let mut s = game(&[0, 0, 1]);
    let start = hex(4, 4);
    let goal = hex(5, 4);
    let a = unit(&mut s, 0, UnitKind::Settler, start);
    let ally = unit(&mut s, 1, UnitKind::Settler, goal);
    visibility::update_explored(&mut s);
    let order = Order::MoveUnitPath {
        unit: a,
        to: goal,
        path: vec![goal],
        stop_on_block: true,
    };
    assert!(
        super::super::validate_orders(&s, PlayerId(0), std::slice::from_ref(&order)).is_empty()
    );
    let (first, events) = resolve_units(&s, &orders(0, vec![order]));
    let u = &first.units[&a];
    assert_eq!(u.pos, start);
    assert_eq!(u.goto, Some(goal));
    assert_eq!(u.path, [goal]);
    assert!(u.path_stop_on_block);
    assert!(
        u.needs_orders,
        "occupancy blocks must request orders immediately"
    );
    assert_eq!(u.blocked_turns, 1);
    assert!(events.iter().any(
        |e| matches!(e, Event::MoveBlocked { unit, toward, reason, .. }
        if *unit == a && *toward == goal && reason == "blocked by P1's Settler, which did not move")
    ));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. }))
    );

    // The retained order and blocked-turn count survive save/load, then the usual recovery fires.
    let first: GameState = serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
    let (second, events) = resolve_units(&first, &no_orders());
    let u = &second.units[&a];
    assert_eq!(u.pos, start);
    assert_eq!(u.goto, None);
    assert!(u.path.is_empty());
    assert!(!u.path_stop_on_block);
    assert!(u.needs_orders);
    assert_eq!(u.blocked_turns, 0);
    assert_eq!(second.units[&ally].pos, goal);
    let reason = format!("{MOVE_STOPPED_PREFIX}blocked by P1's Settler at 5,4");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, reason: r, .. }
        if *unit == a && *r == reason))
    );
    let (third, events) = resolve_units(&second, &no_orders());
    assert_eq!(third.units[&a].pos, start);
    assert!(third.units[&a].needs_orders);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, .. } if *unit == a))
    );
}

fn changed_route(blocker: &str, kind: UnitKind) {
    let mut s = game(&[0, 1]);
    let a = unit(&mut s, 0, kind, hex(2, 4));
    for i in 0..s.map.len() {
        s.players[0].explored.set(i);
    }
    let goal = hex(8, 4);
    let route: Vec<_> = (3..=8).map(|col| hex(col, 4)).collect();
    let os = orders(
        0,
        vec![Order::MoveUnitPath {
            unit: a,
            to: goal,
            path: route,
            stop_on_block: false,
        }],
    );
    assert!(crate::validate_orders(&s, PlayerId(0), &os[&PlayerId(0)]).is_empty());
    s = resolve_units(&s, &os).0;
    let at = s.units[&a].path[0];
    match blocker {
        "city" => {
            city(&mut s, 1, at, 3);
        }
        "friend" => {
            unit(&mut s, 0, kind, at);
        }
        "civilian" => {
            unit(&mut s, 1, UnitKind::Settler, at);
        }
        _ => {
            unit(&mut s, 1, UnitKind::Spearman, at);
        }
    }
    let mut plain = s.clone();
    plain.units.get_mut(&a).unwrap().path.clear();
    for _ in 0..4 {
        let (next, events) = resolve_units(&s, &no_orders());
        // The blocking city may fire at us (city strike); our unit must never attack.
        assert!(
            !events.iter().any(|e| matches!(
                e,
                Event::Battle {
                    attacker: crate::events::Attacker::Unit(_),
                    ..
                } | Event::SettlerCaptured { .. }
            )),
            "a route must not become an attack: {events:?}"
        );
        s = next;
        plain = resolve_units(&plain, &no_orders()).0;
    }
    assert_eq!(plain.units[&a].pos, goal, "MoveUnit control arrives");
    assert_eq!(
        s.units[&a].pos, goal,
        "confirmed route must reroute around {blocker}"
    );
    assert!(s.units[&a].path.is_empty());
}

#[test]
fn review_route_never_attacks_new_enemy_unit() {
    changed_route("enemy", UnitKind::Spearman);
}
#[test]
fn review_route_never_attacks_new_enemy_city() {
    changed_route("city", UnitKind::Spearman);
}
#[test]
fn review_route_never_captures_intermediate_civilian() {
    changed_route("civilian", UnitKind::Spearman);
}
#[test]
fn review_route_reroutes_around_parked_friend() {
    changed_route("friend", UnitKind::Spearman);
}
#[test]
fn review_settler_route_reroutes_around_enemy() {
    changed_route("enemy", UnitKind::Settler);
}

#[test]
fn review_known_hostiles_are_rejected_inside_routes_but_allowed_as_goals() {
    for blocker in ["city", "unit", "civilian"] {
        let mut s = game(&[0, 1]);
        let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
        let at = hex(4, 4);
        if blocker == "city" {
            city(&mut s, 1, at, 3);
        } else {
            unit(
                &mut s,
                1,
                if blocker == "unit" {
                    UnitKind::Spearman
                } else {
                    UnitKind::Settler
                },
                at,
            );
        }
        visibility::update_explored(&mut s);
        let order = Order::MoveUnitPath {
            unit: a,
            to: hex(5, 4),
            path: vec![at, hex(5, 4)],
            stop_on_block: false,
        };
        assert_eq!(
            crate::validate_orders(&s, PlayerId(0), std::slice::from_ref(&order)).len(),
            1,
            "known {blocker}"
        );
        let (_, events) = resolve_units(&s, &orders(0, vec![order]));
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
        assert!(
            crate::validate_orders(
                &s,
                PlayerId(0),
                &[Order::MoveUnitPath {
                    unit: a,
                    to: at,
                    path: vec![at],
                    stop_on_block: false,
                }]
            )
            .is_empty()
        );
    }
}

#[test]
fn review_ranged_units_cannot_capture_in_any_entry_case() {
    for kind in [
        UnitKind::Archer,
        UnitKind::Crossbowman,
        UnitKind::CompositeArcher,
    ] {
        for defender in ["none", "incoming", "garrison"] {
            let mut s = game(&[0, 1]);
            let at = hex(5, 5);
            let c = city(&mut s, 1, at, 3);
            s.cities.get_mut(&c).unwrap().hp = 0;
            let a = unit(&mut s, 0, kind, at.neighbor(0));
            if defender != "none" {
                let b = unit(
                    &mut s,
                    1,
                    UnitKind::Spearman,
                    if defender == "incoming" {
                        at.neighbor(3)
                    } else {
                        at
                    },
                );
                s.units.get_mut(&b).unwrap().hp = 1;
                if defender == "incoming" {
                    goto(&mut s, b, at);
                }
            }
            goto(&mut s, a, at);
            let (next, events) = resolve_units(&s, &no_orders());
            assert_eq!(next.cities[&c].owner, PlayerId(1), "{kind:?}, {defender}");
            assert_ne!(next.units[&a].pos, at);
            assert!(events.iter().any(|e| matches!(e, Event::MoveBlocked { unit, reason, .. } if *unit == a && reason == "Ranged units can't capture cities; move a melee unit in")), "{events:?}");
        }
    }
}

fn team_capture(seed: u64, reverse_ids: bool, reverse_seats: bool, hp: [i32; 2]) -> Hex {
    let mut s = game(&[0, 0, 1]);
    s.setup.seed = seed;
    let at = hex(5, 5);
    let c = city(&mut s, 2, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let starts = [at.neighbor(0), at.neighbor(3)];
    for k in if reverse_ids { [1, 0] } else { [0, 1] } {
        let a = unit(
            &mut s,
            if reverse_seats { 1 - k } else { k } as u8,
            UnitKind::Spearman,
            starts[k],
        );
        s.units.get_mut(&a).unwrap().hp = hp[k];
        goto(&mut s, a, at);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_ne!(
        next.cities[&c].owner,
        PlayerId(2),
        "teammates must capture immediately"
    );
    assert!(!events.iter().any(|e| matches!(e, Event::Bounce { .. })));
    let captor = next.units.values().find(|u| u.pos == at).unwrap();
    assert_eq!(next.cities[&c].owner, captor.owner);
    let best_hp = next.units.values().map(|u| u.hp).max().unwrap();
    assert_eq!(captor.hp, best_hp, "highest HP after resolution wins");
    s.units[&captor.id].pos
}

#[test]
fn review_teammates_capture_with_the_healthiest_survivor() {
    for reverse in [false, true] {
        assert_eq!(
            team_capture(7, reverse, false, [100, 60]),
            hex(5, 5).neighbor(0)
        );
        assert_eq!(
            team_capture(7, reverse, false, [60, 100]),
            hex(5, 5).neighbor(3)
        );
    }
}

#[test]
fn review_teammate_capture_coin_uses_seed_turn_tile_not_ids_or_seats() {
    let mut winners = BTreeSet::new();
    for seed in 0..32 {
        let expected = team_capture(seed, false, false, [100, 100]);
        for (ids, seats) in [(true, false), (false, true), (true, true)] {
            assert_eq!(team_capture(seed, ids, seats, [100, 100]), expected);
        }
        winners.insert(expected);
    }
    assert_eq!(winners.len(), 2, "both directions win across seeds");
}

#[test]
fn review_waiting_traffic_does_not_retry_because_another_owner_moved() {
    let mut s = game(&[0, 1]);
    let at = hex(5, 5);
    unit(&mut s, 0, UnitKind::Spearman, at);
    let a = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(0));
    let b = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(3));
    let other = unit(&mut s, 1, UnitKind::Spearman, hex(9, 8));
    goto(&mut s, a, at);
    goto(&mut s, b, at);
    goto(&mut s, other, hex(10, 8));
    let mut stopped = BTreeSet::new();
    let mut events = Vec::new();
    assert!(movement_round(
        &mut s,
        0,
        &BTreeMap::new(),
        &mut stopped,
        &mut Pursuit::default(),
        &mut Vec::new(),
        &mut events
    ));
    assert_eq!(s.units[&other].pos, hex(10, 8));
    assert!(
        stopped.contains(&b),
        "unrelated progress must not restart waiting traffic"
    );
}

#[test]
fn review_terrain_fallback_cannot_turn_a_move_into_an_attack() {
    let mut s = game(&[0, 1]);
    let a = unit(&mut s, 0, UnitKind::Spearman, hex(3, 4));
    let b = unit(&mut s, 1, UnitKind::Spearman, hex(4, 4));
    for t in &mut s.map.tiles {
        t.terrain = Terrain::Mountains;
    }
    for col in 3..=6 {
        set_terrain(&mut s, hex(col, 4), Terrain::Grassland);
    }
    goto(&mut s, a, hex(6, 4));
    s.units.get_mut(&a).unwrap().path = (4..=6).map(|c| hex(c, 4)).collect();
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(3, 4));
    assert_eq!(next.units[&b].hp, 100);
    assert!(next.units[&a].path.is_empty());
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, .. } if *unit == a))
    );
}

#[test]
fn review_teammates_use_hp_after_contested_ruin_combat() {
    for seed in 0..16 {
        let mut s = game(&[0, 0, 1]);
        s.setup.seed = seed;
        let at = hex(5, 5);
        let c = city(&mut s, 2, at, 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let a = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(0));
        let b = unit(&mut s, 1, UnitKind::Spearman, at.neighbor(2));
        let enemy = unit(&mut s, 2, UnitKind::Spearman, at.neighbor(4));
        s.units.get_mut(&enemy).unwrap().hp = 1;
        for u in [a, b, enemy] {
            goto(&mut s, u, at);
        }
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(!next.units.contains_key(&enemy));
        // The city's owner reinforces first: both teammates fight it as the garrison.
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Battle { meeting: false, .. }))
                .count(),
            2
        );
        let winner = [a, b]
            .into_iter()
            .find(|u| next.units[u].pos == at)
            .unwrap();
        assert_eq!(
            next.units[&winner].hp,
            next.units[&a].hp.max(next.units[&b].hp)
        );
        assert_eq!(next.cities[&c].owner, next.units[&winner].owner);
    }
}

#[test]
fn review_defeated_city_forecast_has_no_retaliation() {
    let mut s = game(&[0, 1]);
    let at = hex(5, 5);
    let c = city(&mut s, 1, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let a = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(0));
    visibility::update_explored(&mut s);
    let f = forecast_attack_mode(&player_view(&s, PlayerId(0)), a, at, AttackMode::Melee).unwrap();
    assert_eq!(
        (
            f.expected_damage_to_attacker,
            f.expected_damage_to_defender,
            f.loss_chance
        ),
        (0, 0, 0)
    );
    goto(&mut s, a, at);
    let (next, events) = resolve_units(&s, &no_orders());
    // Nor does it fire back (city strike needs HP above 0 in the round's snapshot).
    assert_eq!(next.units[&a].hp, 100);
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
}

/// A Peace-time corridor of Grassland through a foreign city's land: `start` (P1's land, where
/// P0's unit was stranded), the city `c`, `beyond` (P1's land) and `exit` (unowned, open), plus
/// the way round the city when `around` is set. Everything else is Mountains.
fn stranded_beside_a_foreign_city(around: bool) -> (GameState, UnitId, Hex, Hex) {
    let mut s = game(&[0, 1]);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = crate::diplomacy::RelationState::Peace;
    }
    for tile in &mut s.map.tiles {
        tile.terrain = Terrain::Mountains;
    }
    let start = hex(4, 5);
    let c = start.neighbor(0);
    let beyond = c.neighbor(0);
    let exit = beyond.neighbor(0);
    let mut open = vec![start, c, beyond, exit];
    if around {
        open.extend([start.neighbor(1), c.neighbor(1)]);
    }
    for h in open {
        set_terrain(&mut s, h, Terrain::Grassland);
    }
    city(&mut s, 1, c, 3);
    assert_eq!(s.map.get(start).unwrap().owner, Some(PlayerId(1)));
    assert_eq!(s.map.get(exit).unwrap().owner, None);
    let ranger = unit(&mut s, 0, UnitKind::Archer, start);
    (s, ranger, c, exit)
}

/// The state is legal apart from units stranded in closed land, which Peace leaves standing.
fn only_stranded(s: &GameState) -> bool {
    crate::scenario::validate_state(s)
        .iter()
        .all(|p| p.contains("territory without being at war"))
}

#[test]
fn a_stranded_unit_never_walks_into_a_peaceful_foreign_city_on_its_way_out() {
    // War suite `war-india-chur-ring` run 41, T155: a Swiss Archer stranded in Japanese land by
    // the Peace made that turn took the city as the stepping stone to the exit and ended the
    // turn inside Hiroshima. The egress route counted the city's tile as passable.
    let (mut s, ranger, c, exit) = stranded_beside_a_foreign_city(false);
    for goal in [c, exit] {
        goto(&mut s, ranger, goal);
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(next.units[&ranger].pos, s.units[&ranger].pos, "{goal:?}");
        assert!(
            !events.iter().any(|e| matches!(e,
                Event::UnitMoved { unit, to, .. } if *unit == ranger && *to == c)),
            "{goal:?}"
        );
        assert!(only_stranded(&next));
    }
}

#[test]
fn a_stranded_unit_still_walks_out_around_a_peaceful_foreign_city() {
    let (mut s, ranger, c, exit) = stranded_beside_a_foreign_city(true);
    goto(&mut s, ranger, exit);
    for _ in 0..6 {
        let (next, events) = resolve_units(&s, &no_orders());
        assert!(!events.iter().any(|e| matches!(e,
            Event::UnitMoved { unit, to, .. } if *unit == ranger && *to == c)));
        assert!(only_stranded(&next));
        s = next;
    }
    assert_eq!(s.units[&ranger].pos, exit);
}

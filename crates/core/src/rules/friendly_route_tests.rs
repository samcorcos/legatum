//! Playtest-6.2 "The Last Bridge": routes prefer free tiles over friends that will not leave.
use super::*;
use crate::Order;
use crate::rules::test_support::*;

/// City `c`, a friend `m` beside it, a free tile `f` beside it, and a soldier `r` two tiles
/// away that touches both `m` and `f`: two equally short approaches. Every rotation is tried,
/// so the hex-order tie-break picks the friend's tile in at least one of them.
fn last_bridge(dir: usize) -> (GameState, [Hex; 4]) {
    let mut s = game(&[0, 1]);
    let c = hex(6, 5);
    let m = c.neighbor(dir);
    let f = c.neighbor(dir + 1);
    let r = Hex::new(m.q + f.q - c.q, m.r + f.r - c.r);
    assert_eq!((r.distance(c), r.distance(m), r.distance(f)), (2, 1, 1));
    city(&mut s, 1, c, 3);
    (s, [c, m, f, r])
}

fn attacked_city(events: &[Event], attacker: UnitId) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::Battle { attacker: crate::events::Attacker::Unit(a), .. } if *a == attacker)
    })
}

fn halted(events: &[Event], who: UnitId) -> bool {
    events.iter().any(|e| match e {
        Event::MoveBlocked { unit, .. } => *unit == who,
        Event::Bounce { units, .. } => units.contains(&who),
        _ => false,
    })
}

#[test]
fn attack_routes_around_a_friend_attacking_in_place() {
    for dir in 0..6 {
        let (mut s, [c, m, f, r]) = last_bridge(dir);
        let friend = unit(&mut s, 0, UnitKind::Soldier, m);
        let rightmost = unit(&mut s, 0, UnitKind::Soldier, r);
        // The confirmed route the old preview built through the friend, and a plain goto.
        for confirmed in [true, false] {
            let order = if confirmed {
                Order::MoveUnitPath {
                    unit: rightmost,
                    to: c,
                    path: vec![m, c],
                    stop_on_block: false,
                }
            } else {
                Order::MoveUnit {
                    unit: rightmost,
                    to: c,
                }
            };
            let (next, events) = resolve_units(
                &s,
                &orders(
                    0,
                    vec![
                        Order::MoveUnit {
                            unit: friend,
                            to: c,
                        },
                        order,
                    ],
                ),
            );
            assert!(
                attacked_city(&events, friend),
                "dir {dir}: the friend attacks"
            );
            assert!(
                attacked_city(&events, rightmost),
                "dir {dir} confirmed {confirmed}: the rightmost soldier attacks from {f:?}: {events:?}"
            );
            assert!(!halted(&events, rightmost), "dir {dir}: no bounce");
            assert_eq!(next.units[&rightmost].pos, f, "dir {dir}");
            assert_eq!(next.units[&friend].pos, m);
        }
    }
}

#[test]
fn a_parked_friend_is_routed_around_by_tie_and_by_a_slightly_longer_route() {
    for dir in 0..6 {
        let (mut s, [c, m, f, r]) = last_bridge(dir);
        // Free tile `f` is forest: the free route costs one more move but still arrives
        // this turn for a 2-move Soldier (the step into the city is always allowed at 1 move).
        set_terrain(&mut s, f, Terrain::Forest);
        let _parked = unit(&mut s, 0, UnitKind::Soldier, m);
        let rightmost = unit(&mut s, 0, UnitKind::Soldier, r);
        let (next, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![Order::MoveUnit {
                    unit: rightmost,
                    to: c,
                }],
            ),
        );
        assert!(!halted(&events, rightmost), "dir {dir}: {events:?}");
        assert_eq!(next.units[&rightmost].pos, f, "dir {dir}");
    }
}

#[test]
fn a_moving_friend_is_a_tie_break_not_a_wall() {
    for dir in 0..6 {
        let (mut s, [c, m, f, r]) = last_bridge(dir);
        // The friend leaves `m` away from the city: following it is legal (chains move
        // together), but on an equal route the free tile is preferred.
        let friend = unit(&mut s, 0, UnitKind::Soldier, m);
        let away = m.neighbor(dir);
        let rightmost = unit(&mut s, 0, UnitKind::Soldier, r);
        let (next, events) = resolve_units(
            &s,
            &orders(
                0,
                vec![
                    Order::MoveUnit {
                        unit: friend,
                        to: away,
                    },
                    Order::MoveUnit {
                        unit: rightmost,
                        to: c,
                    },
                ],
            ),
        );
        assert!(attacked_city(&events, rightmost), "dir {dir}: {events:?}");
        assert_eq!(next.units[&rightmost].pos, f, "dir {dir}: free tile first");
    }
}

#[test]
fn a_row_attacking_one_city_takes_distinct_approach_tiles() {
    let mut s = game(&[0, 1]);
    let c = hex(6, 4);
    city(&mut s, 1, c, 3);
    // Three soldiers two tiles south of the city, side by side.
    let row: Vec<Hex> = c
        .within(2)
        .into_iter()
        .filter(|h| h.distance(c) == 2 && h.r == c.r + 2)
        .collect();
    assert_eq!(row.len(), 3);
    let ids: Vec<UnitId> = row
        .iter()
        .map(|h| unit(&mut s, 0, UnitKind::Soldier, *h))
        .collect();
    let os = ids
        .iter()
        .map(|&u| Order::MoveUnit { unit: u, to: c })
        .collect();
    let (next, events) = resolve_units(&s, &orders(0, os));
    let ends: BTreeSet<Hex> = ids.iter().map(|u| next.units[u].pos).collect();
    assert_eq!(ends.len(), 3, "distinct tiles");
    // Only two approach tiles are one step away for the row; the third soldier detours to
    // another face of the city and attacks next turn, but nobody bounces off a friend.
    for u in &ids {
        assert_eq!(next.units[u].pos.distance(c), 1, "{events:?}");
        assert!(!halted(&events, *u), "{u:?}: {events:?}");
    }
    let attackers = ids.iter().filter(|u| attacked_city(&events, **u)).count();
    assert_eq!(attackers, 2, "{events:?}");
    let detoured = ids.iter().find(|u| !attacked_city(&events, **u)).unwrap();
    assert_eq!(next.units[detoured].goto, Some(c), "keeps its attack order");
    let (_, events) = resolve_units(&next, &no_orders());
    assert!(attacked_city(&events, *detoured), "{events:?}");
}

/// A one-tile corridor along row 4 (mountains elsewhere, 12 columns).
fn corridor() -> GameState {
    let mut s = game(&[0, 1]);
    for col in 0..12 {
        for row in 0..10 {
            if row != 4 {
                set_terrain(&mut s, hex(col, row), Terrain::Mountains);
            }
        }
    }
    s
}

#[test]
fn a_unit_passes_through_a_friend_that_stays_but_never_ends_on_it() {
    // Civ 6 (user, 2026-10-21): "it should be reasonable for this unit to be able to walk
    // through my unit". The only route runs through a parked friend: the mover passes it in
    // one move and lands on the tile beyond, paying for both.
    let mut s = corridor();
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(2, 4));
    let friend = unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    goto(&mut s, a, hex(8, 4));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(4, 4), "{events:?}");
    assert_eq!(next.units[&a].moves_left, 0, "both tiles were paid for");
    assert_eq!(next.units[&friend].pos, hex(3, 4));
    assert!(!halted(&events, a), "{events:?}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitMoved { unit, from, to, .. }
            if *unit == a && *from == hex(2, 4) && *to == hex(4, 4))),
        "one hop, never a stop on the friend: {events:?}"
    );
    // The rest of the way next turn, with the friend still where it was.
    let (next, _) = resolve_units(&next, &no_orders());
    assert_eq!(next.units[&a].pos, hex(6, 4));
    let (next, _) = resolve_units(&next, &no_orders());
    assert_eq!(next.units[&a].pos, hex(8, 4));
    assert_eq!(next.units[&friend].pos, hex(3, 4));
}

#[test]
fn a_unit_passes_through_its_owngarrisoned_city() {
    // The Kanazawa case: a unit routed round its own city instead of through it.
    let mut s = corridor();
    city(&mut s, 0, hex(4, 4), 3);
    let garrison = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    goto(&mut s, a, hex(8, 4));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(5, 4), "{events:?}");
    assert!(!halted(&events, a), "{events:?}");
    assert_eq!(next.units[&garrison].pos, hex(4, 4));
}

#[test]
fn a_move_that_would_end_on_a_friend_stops_on_the_last_free_tile() {
    // The goal is a friend's tile: the unit comes up beside it and says what blocks it.
    let mut s = corridor();
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(2, 4));
    unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    goto(&mut s, a, hex(3, 4));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(2, 4));
    assert_eq!(next.units[&a].goto, Some(hex(3, 4)));
    let want = "blocked by P0's Spearman, which did not move";
    assert!(
        events.iter().any(
            |e| matches!(e, Event::MoveBlocked { unit, reason, .. } if *unit == a && reason == want)
        ),
        "{events:?}"
    );
    // Two friends in a row cost more to cross than a Soldier has moves: it stops on the
    // last free tile it reached rather than on the first friend.
    let mut s = corridor();
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(1, 4));
    unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    goto(&mut s, a, hex(8, 4));
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(2, 4));
    // Once the first friend walks on down the pass, the follower moves with it as a chain.
    let mut s = corridor();
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(2, 4));
    let lead = unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    goto(&mut s, a, hex(6, 4));
    goto(&mut s, lead, hex(8, 4));
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(!halted(&events, a), "{events:?}");
    assert_eq!(next.units[&a].pos, hex(4, 4));
}

#[test]
fn a_hop_that_does_not_fit_the_moves_left_waits_for_next_turn() {
    // One move already spent when the friend comes up: crossing it and landing needs two,
    // so the unit waits on the last free tile it passed and hops at the start of next turn.
    let mut s = corridor();
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(0, 4));
    unit(&mut s, 0, UnitKind::Soldier, hex(2, 4));
    goto(&mut s, a, hex(8, 4));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&a].pos, hex(1, 4), "{events:?}");
    assert_eq!(next.units[&a].goto, Some(hex(8, 4)));
    let (next, _) = resolve_units(&next, &no_orders());
    assert_eq!(next.units[&a].pos, hex(3, 4));
}

#[test]
fn nothing_ends_a_turn_on_a_friend_in_a_crowded_pass() {
    // Six soldiers through a one-tile pass, three of them parked in it: every turn ends
    // with one unit per tile, whatever the order they declare in.
    let mut s = corridor();
    let movers: Vec<UnitId> = (0..3)
        .map(|i| unit(&mut s, 0, UnitKind::Soldier, hex(i, 4)))
        .collect();
    for col in [4, 5, 6] {
        unit(&mut s, 0, UnitKind::Soldier, hex(col, 4));
    }
    for (i, &m) in movers.iter().enumerate() {
        goto(&mut s, m, hex(9 + i as i32 - 2, 4));
    }
    for _ in 0..6 {
        s = resolve_units(&s, &no_orders()).0;
        let mut seen = BTreeSet::new();
        for u in s.units.values() {
            assert!(seen.insert(u.pos), "two soldiers on {:?}", u.pos);
        }
    }
}

/// Playtest 8.6 turn 77 ("units moved backwards away from the goal"): six soldiers sent
/// through a one-tile pass to a goal beyond it, with a long way round. Two causes made the
/// followers turn back, then return next turn:
/// - a friend slowed by hills ends its turn on a pass tile with no moves and a standing goto;
///   the soldier behind treated it as a wall and set off the long way round;
/// - a lower-id friend's declared step reserved a tile two steps ahead, which forced the same
///   detour in round one.
fn pass_with_long_way_round() -> (GameState, Hex) {
    let mut s = game(&[0, 1]);
    for col in 0..12 {
        for row in 0..10 {
            set_terrain(&mut s, hex(col, row), Terrain::Mountains);
        }
    }
    // The pass along row 4, and a way round: down column 1, along row 8, up column 10.
    for col in 1..=10 {
        set_terrain(&mut s, hex(col, 4), Terrain::Grassland);
        set_terrain(&mut s, hex(col, 8), Terrain::Grassland);
    }
    for row in 5..=7 {
        set_terrain(&mut s, hex(1, row), Terrain::Grassland);
        set_terrain(&mut s, hex(10, row), Terrain::Grassland);
    }
    set_terrain(&mut s, hex(6, 4), Terrain::Hills);
    (s, hex(10, 4))
}

/// Steps along the terrain from `from` to `to`, ignoring units.
fn steps_to(s: &GameState, from: Hex, to: Hex) -> i32 {
    crate::path::land_path(&s.map, from, to).map_or(i32::MAX, |p| p.len() as i32)
}

/// Two soldiers (a lone follower has room to turn back, which is how the bug shows), and a
/// column of four. Each has its own tile beyond the pass.
#[test]
fn a_friend_that_moves_on_later_is_waited_for_not_circled() {
    for starts in [vec![5, 4], vec![5, 4, 3, 2]] {
        let (mut s, goal) = pass_with_long_way_round();
        let ids: Vec<UnitId> = starts
            .iter()
            .map(|&col| unit(&mut s, 0, UnitKind::Soldier, hex(col, 4)))
            .collect();
        let goals: Vec<Hex> = (0..ids.len() as i32).map(|i| hex(10 - i, 4)).collect();
        for (&id, &to) in ids.iter().zip(&goals) {
            goto(&mut s, id, to);
        }
        let mut walled = s.clone();
        for col in 5..=9 {
            set_terrain(&mut walled, hex(col, 4), Terrain::Mountains);
        }
        let round = crate::path::land_path(&walled.map, hex(4, 4), goal);
        assert!(
            round.is_some_and(|p| p.len() > 12),
            "a long way round exists"
        );
        for turn in 1..=10 {
            let before: Vec<i32> = ids
                .iter()
                .zip(&goals)
                .map(|(id, to)| steps_to(&s, s.units[id].pos, *to))
                .collect();
            s = resolve_units(&s, &no_orders()).0;
            for ((id, to), was) in ids.iter().zip(&goals).zip(before) {
                let now = steps_to(&s, s.units[id].pos, *to);
                // Waiting behind a slow friend costs the round, not the ground already made.
                let progress = now < was || was == 0 || starts.len() > 2;
                assert!(
                    now <= was && progress,
                    "{starts:?} turn {turn}: {id:?} went from {was} to {now} steps from its goal, at {:?}",
                    s.units[id].pos
                );
            }
        }
        for (id, to) in ids.iter().zip(&goals) {
            assert_eq!(s.units[id].pos, *to, "{starts:?}: {id:?} arrives");
        }
    }
}

#[test]
fn two_spearmen_follow_through_a_reserved_pass_without_detouring() {
    let (mut s, _) = pass_with_long_way_round();
    let pass = hex(5, 4);
    // The lower-id Spearman enters the only mountain pass from above. The follower's
    // direct first step is the same tile; the alternative goes down to row 8 and back.
    set_terrain(&mut s, hex(4, 3), Terrain::Grassland);
    set_terrain(&mut s, hex(6, 4), Terrain::Grassland);
    let lead = unit(&mut s, 0, UnitKind::Soldier, hex(4, 3));
    let follower = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    goto(&mut s, lead, hex(9, 4));
    goto(&mut s, follower, hex(8, 4));

    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&lead].pos, hex(6, 4), "{events:?}");
    assert_eq!(next.units[&follower].pos, pass, "{events:?}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitMoved { unit, to, .. }
        if *unit == lead && *to == pass)),
        "lead enters pass: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitMoved { unit, to, .. }
        if *unit == follower && *to == pass)),
        "follower enters after lead: {events:?}"
    );
    let moves: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::UnitMoved { unit, to, .. } if [lead, follower].contains(unit) => {
                Some((*unit, *to))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        moves,
        vec![(lead, pass), (lead, hex(6, 4)), (follower, pass)],
        "the follower waits round one, then enters the vacated pass"
    );
    assert_ne!(next.units[&lead].pos, next.units[&follower].pos);
    for id in [lead, follower] {
        assert_eq!(
            next.units[&id].pos.r, pass.r,
            "no mountain detour: {events:?}"
        );
    }
    let (next, _) = resolve_units(&next, &no_orders());
    assert!(next.units[&follower].pos.q > pass.q);
    let (next, _) = resolve_units(&next, &no_orders());
    assert_eq!(next.units[&lead].pos, hex(9, 4));
    assert_eq!(next.units[&follower].pos, hex(8, 4));
}

#[test]
fn a_shared_final_tile_still_has_one_claimant() {
    let mut s = game(&[0, 1]);
    let goal = hex(5, 4);
    let first = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    let second = unit(&mut s, 0, UnitKind::Soldier, hex(4, 3));
    goto(&mut s, first, goal);
    goto(&mut s, second, goal);
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&first].pos, goal);
    assert_eq!(next.units[&second].pos, hex(4, 3));
    assert_eq!(next.units[&second].goto, Some(goal));
}

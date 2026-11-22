//! Units of empires not at war stepping onto the same tile in the same round (Sam, 2026-11-22:
//! a Spartan unit and a weaker one both went for a ruin and both bounced). The side with the
//! most combined strength moves in, the others stay where they were, and nobody is attacked.
use super::*;
use crate::diplomacy::RelationState;
use crate::rules::test_support::*;

/// Three teams, all at Peace with each other.
fn at_peace(teams: &[u8]) -> GameState {
    let mut s = game(teams);
    for relation in &mut s.diplomacy.as_mut().unwrap().relations {
        relation.state = RelationState::Peace;
    }
    s
}

fn reason(events: &[Event], unit: UnitId) -> Option<String> {
    events.iter().find_map(|e| match e {
        Event::MoveBlocked {
            unit: u, reason, ..
        } if *u == unit => Some(reason.clone()),
        _ => None,
    })
}

fn no_fights(events: &[Event]) -> bool {
    !events
        .iter()
        .any(|e| matches!(e, Event::Battle { .. } | Event::Bounce { .. }))
}

#[test]
fn the_stronger_unit_takes_a_contested_ruin_and_the_weaker_stays_put() {
    let mut s = at_peace(&[0, 1]);
    let ruin = hex(5, 5);
    s.map.get_mut(ruin).unwrap().ruin = true;
    let scout = unit(&mut s, 0, UnitKind::Scout, ruin.neighbor(0));
    let spear = unit(&mut s, 1, UnitKind::Soldier, ruin.neighbor(3));
    goto(&mut s, scout, ruin);
    goto(&mut s, spear, ruin);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&spear].pos, ruin);
    assert_eq!(next.units[&scout].pos, ruin.neighbor(0));
    assert_eq!(next.units[&scout].hp, 100);
    assert!(
        !next.map.get(ruin).unwrap().ruin,
        "the winner explored the ruin"
    );
    assert!(no_fights(&events));
    assert_eq!(
        reason(&events, scout).as_deref(),
        Some("outmatched: P1's Spearman is stronger (20 against 5) and moved in")
    );
    // Seat order does not matter: the same holds with the roles swapped.
    let mut s = at_peace(&[0, 1]);
    let spear = unit(&mut s, 0, UnitKind::Soldier, ruin.neighbor(0));
    let scout = unit(&mut s, 1, UnitKind::Scout, ruin.neighbor(3));
    goto(&mut s, scout, ruin);
    goto(&mut s, spear, ruin);
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&spear].pos, ruin);
    assert_eq!(next.units[&scout].pos, ruin.neighbor(3));
}

#[test]
fn damage_counts_and_a_civilian_never_wins_against_a_military_unit() {
    let mut s = at_peace(&[0, 1]);
    let tile = hex(5, 5);
    let hurt = unit(&mut s, 0, UnitKind::Soldier, tile.neighbor(0));
    s.units.get_mut(&hurt).unwrap().hp = 40;
    let fresh = unit(&mut s, 1, UnitKind::Soldier, tile.neighbor(3));
    goto(&mut s, hurt, tile);
    goto(&mut s, fresh, tile);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(
        next.units[&fresh].pos, tile,
        "the unhurt Spearman is stronger"
    );
    assert_eq!(next.units[&hurt].pos, tile.neighbor(0));
    assert!(no_fights(&events));

    let mut s = at_peace(&[0, 1]);
    let settler = unit(&mut s, 0, UnitKind::Settler, tile.neighbor(0));
    let scout = unit(&mut s, 1, UnitKind::Scout, tile.neighbor(3));
    goto(&mut s, settler, tile);
    goto(&mut s, scout, tile);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&scout].pos, tile);
    assert_eq!(next.units[&settler].pos, tile.neighbor(0));
    assert_eq!(next.units[&settler].owner, PlayerId(0), "never captured");
    assert!(no_fights(&events));
}

#[test]
fn equal_strength_is_drawn_by_seed_and_the_same_on_every_replay() {
    let tile = hex(5, 5);
    let mut winners = BTreeSet::new();
    for seed in 0..16 {
        let mut s = at_peace(&[0, 1]);
        s.setup.seed = seed;
        let a = unit(&mut s, 0, UnitKind::Soldier, tile.neighbor(0));
        let b = unit(&mut s, 1, UnitKind::Soldier, tile.neighbor(3));
        goto(&mut s, a, tile);
        goto(&mut s, b, tile);
        let (next, events) = resolve_units(&s, &no_orders());
        assert_eq!(
            resolve_units(&s, &no_orders()),
            (next.clone(), events.clone())
        );
        let won = [a, b]
            .into_iter()
            .filter(|u| next.units[u].pos == tile)
            .collect::<Vec<_>>();
        assert_eq!(won.len(), 1, "exactly one moves in");
        let lost = if won[0] == a { b } else { a };
        assert!(
            reason(&events, lost)
                .unwrap()
                .ends_with("is just as strong (20 each) and won the draw to move in")
        );
        assert!(no_fights(&events));
        winners.insert(next.units[&won[0]].owner);
    }
    assert_eq!(winners.len(), 2, "neither seat always wins a tie");
}

#[test]
fn co_op_partners_are_one_side_against_a_stronger_unit() {
    // P0 and P1 share a team; P2 is at Peace with both. Their Settler and Scout add up to 5,
    // less than P2's Spearman.
    let mut s = at_peace(&[0, 0, 1]);
    let tile = hex(5, 5);
    let settler = unit(&mut s, 0, UnitKind::Settler, tile.neighbor(0));
    let scout = unit(&mut s, 1, UnitKind::Scout, tile.neighbor(1));
    let spear = unit(&mut s, 2, UnitKind::Soldier, tile.neighbor(3));
    for u in [settler, scout, spear] {
        goto(&mut s, u, tile);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&spear].pos, tile);
    assert_eq!(next.units[&settler].pos, tile.neighbor(0));
    assert_eq!(next.units[&scout].pos, tile.neighbor(1));
    assert!(no_fights(&events));

    // With the partners' Spearman stronger than the rival's Scout, the team moves in: its
    // Spearman and Settler share the tile, as one team's units may.
    let mut s = at_peace(&[0, 0, 1]);
    let settler = unit(&mut s, 0, UnitKind::Settler, tile.neighbor(0));
    let spear = unit(&mut s, 1, UnitKind::Soldier, tile.neighbor(1));
    let scout = unit(&mut s, 2, UnitKind::Scout, tile.neighbor(3));
    for u in [settler, spear, scout] {
        goto(&mut s, u, tile);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&spear].pos, tile);
    assert_eq!(next.units[&settler].pos, tile);
    assert_eq!(next.units[&scout].pos, tile.neighbor(3));
    assert!(no_fights(&events));
}

#[test]
fn at_war_the_two_units_still_fight_a_meeting_engagement() {
    let mut s = game(&[0, 1]);
    let tile = hex(5, 5);
    let a = unit(&mut s, 0, UnitKind::Soldier, tile.neighbor(0));
    let b = unit(&mut s, 1, UnitKind::Scout, tile.neighbor(3));
    goto(&mut s, a, tile);
    goto(&mut s, b, tile);
    let (_, events) = resolve_units(&s, &no_orders());
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert!(reason(&events, b).is_none_or(|r| !r.starts_with(OUTMATCHED_PREFIX)));
}

#[test]
fn several_units_of_one_side_add_up_against_a_single_stronger_unit() {
    // As at a fallen city (`settle_by_force`), force is summed per side: a hurt Spearman and a
    // Scout together outweigh one fresh Spearman that beats either alone.
    let mut s = at_peace(&[0, 1]);
    let tile = hex(5, 5);
    let hurt = unit(&mut s, 0, UnitKind::Soldier, tile.neighbor(0));
    s.units.get_mut(&hurt).unwrap().hp = 60;
    let scout = unit(&mut s, 0, UnitKind::Scout, tile.neighbor(1));
    let fresh = unit(&mut s, 1, UnitKind::Soldier, tile.neighbor(3));
    let strength = |kind, hp| combat::unit_meeting_strength(kind, hp).total;
    let (a, b, c) = (
        strength(UnitKind::Soldier, 60),
        strength(UnitKind::Scout, 100),
        strength(UnitKind::Soldier, 100),
    );
    assert!(a < c && b < c && a + b > c, "{a} + {b} against {c}");
    for u in [hurt, scout, fresh] {
        goto(&mut s, u, tile);
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.units[&fresh].pos, tile.neighbor(3));
    assert_eq!(
        reason(&events, fresh).as_deref(),
        Some(
            format!(
                "outmatched: P0's units are stronger ({} against {c}) and moved in",
                a + b
            )
            .as_str()
        )
    );
    // One of P0's two units takes the tile (lowest id first); the other waits behind it.
    assert_eq!(next.units[&hurt].pos, tile);
    assert!(no_fights(&events));
}

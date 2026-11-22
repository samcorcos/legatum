//! Rival assaults on one city (user, 2026-11-11, Lucerne: his city at 0/200, two empires at war
//! with each other both ordering units in and bouncing every turn). The side with the most force
//! walks in; an exact tie still bounces everyone. Since 2026-11-14 (Tara: India and Sparta, both at
//! war with the Celts but at Peace with each other, bounced off an empty city for three turns) the
//! same rule decides it for attackers of different teams at Peace with each other.
use super::*;
use crate::rules::test_support::*;
use crate::{Order, resolve_turn};

fn reasons(events: &[Event], unit: UnitId) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::MoveBlocked {
                unit: u, reason, ..
            } if *u == unit => Some(reason.clone()),
            _ => None,
        })
        .collect()
}

fn battles_between_attackers(events: &[Event], a: PlayerId, b: PlayerId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(e, Event::Battle { attacker_owner, defender: crate::events::Defender::Unit { owner, .. }, .. }
                if (*attacker_owner == a && *owner == b) || (*attacker_owner == b && *owner == a))
        })
        .count()
}

/// Lucerne: P0's city at (5,4), 0 HP, no garrison; P1 and P2, at war with P0 and each other.
fn lucerne() -> (GameState, CityId) {
    let mut s = game(&[0, 1, 2]);
    let c = city(&mut s, 0, hex(5, 4), 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    (s, c)
}

fn send(s: &mut GameState, owner: u8, kind: UnitKind, dir: usize) -> UnitId {
    let u = unit(s, owner, kind, hex(5, 4).neighbor(dir));
    goto(s, u, hex(5, 4));
    u
}

#[test]
fn lucerne_the_side_with_more_force_takes_the_ruined_city() {
    let (mut s, c) = lucerne();
    let a1 = send(&mut s, 1, UnitKind::Soldier, 0);
    let a2 = send(&mut s, 1, UnitKind::Soldier, 1);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let force = battle::city_force(&s, a1, c, false);
    assert!(force > 0);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(1));
    assert_eq!(
        [a1, a2]
            .iter()
            .filter(|u| next.units[*u].pos == hex(5, 4))
            .count(),
        1
    );
    assert_eq!(next.units[&b].pos, hex(5, 4).neighbor(3), "the loser stays");
    assert_eq!(next.units[&b].hp, 100, "no fight between the two attackers");
    assert_eq!(
        battles_between_attackers(&events, PlayerId(1), PlayerId(2)),
        0
    );
    let p1 = s.player(PlayerId(1)).name.clone();
    assert_eq!(
        reasons(&events, b),
        vec![format!(
            "{p1} took the city with force {} against your {force}",
            2 * force
        )]
    );
}

#[test]
fn lucerne_equal_force_still_bounces_everyone() {
    let (mut s, c) = lucerne();
    let a = send(&mut s, 1, UnitKind::Soldier, 0);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let force = battle::city_force(&s, a, c, false);
    assert_eq!(force, battle::city_force(&s, b, c, false));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    assert_eq!(next.units[&a].pos, hex(5, 4).neighbor(0));
    assert_eq!(next.units[&b].pos, hex(5, 4).neighbor(3));
    assert_eq!(
        battles_between_attackers(&events, PlayerId(1), PlayerId(2)),
        0
    );
    let name = |p: u8| s.player(PlayerId(p)).name.clone();
    assert_eq!(
        reasons(&events, a),
        vec![format!(
            "{} brought equal force ({force} each), so nobody took the city",
            name(2)
        )]
    );
    assert_eq!(
        reasons(&events, b),
        vec![format!(
            "{} brought equal force ({force} each), so nobody took the city",
            name(1)
        )]
    );
}

#[test]
fn lucerne_a_ranged_side_with_more_force_cannot_take_it_and_the_melee_side_does() {
    let (mut s, c) = lucerne();
    // A whole turn: keep both rivals in the game and able to pay upkeep.
    for p in [1u8, 2] {
        city(&mut s, p, hex(1 + 8 * (p as i32 - 1), 1), 3);
        s.players[p as usize].gold = 500;
    }
    // P1: two Rangers shoot the ruin and one more steps onto it (bounced: ranged can't capture).
    let r1 = unit(&mut s, 1, UnitKind::Ranger, hex(5, 4).neighbor(0));
    let r2 = unit(&mut s, 1, UnitKind::Ranger, hex(5, 4).neighbor(1));
    let r3 = send(&mut s, 1, UnitKind::Ranger, 2);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let shots: i32 = [r1, r2]
        .iter()
        .map(|r| battle::city_force(&s, *r, c, true))
        .sum::<i32>()
        + battle::city_force(&s, r3, c, false);
    assert!(
        shots > battle::city_force(&s, b, c, false),
        "P1 brings more force"
    );
    let shoot = |unit| Order::RangedAttack {
        unit,
        target: hex(5, 4),
    };
    let orders = BTreeMap::from([(PlayerId(1), vec![shoot(r1), shoot(r2)])]);
    let (next, events) = resolve_turn(&s, &orders);
    assert_eq!(next.cities[&c].owner, PlayerId(2));
    assert_eq!(next.units[&b].pos, hex(5, 4));
    assert_ne!(next.units[&r3].pos, hex(5, 4));
    let p2 = s.player(PlayerId(2)).name.clone();
    assert_eq!(
        reasons(&events, r3),
        vec![format!(
            "{p2} took the city with force {} against your {shots}",
            battle::city_force(&s, b, c, false)
        )]
    );
}

#[test]
fn rivals_behind_a_relief_chain_settle_by_force_once_every_defender_is_dead() {
    let (mut s, c) = lucerne();
    let g = unit(&mut s, 0, UnitKind::Soldier, hex(5, 4));
    s.units.get_mut(&g).unwrap().hp = 5;
    let relief = unit(&mut s, 0, UnitKind::Soldier, hex(5, 4).neighbor(5));
    s.units.get_mut(&relief).unwrap().hp = 5;
    goto(&mut s, relief, hex(5, 4));
    let a1 = send(&mut s, 1, UnitKind::Soldier, 0);
    let a2 = send(&mut s, 1, UnitKind::Soldier, 1);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&g));
    assert!(!next.units.contains_key(&relief), "the relief fell too");
    assert_eq!(
        next.cities[&c].owner,
        PlayerId(1),
        "then the stronger side walked in"
    );
    assert!([a1, a2].iter().any(|u| next.units[u].pos == hex(5, 4)));
    assert_ne!(next.units[&b].pos, hex(5, 4));
    assert_eq!(
        battles_between_attackers(&events, PlayerId(1), PlayerId(2)),
        0
    );
    let blocked = reasons(&events, b);
    assert_eq!(blocked.len(), 1, "{events:#?}");
    assert!(
        blocked[0].contains(" took the city with force "),
        "{blocked:?}"
    );
}

/// Tara: P0's ruined city, with P1 and P2 at war with P0 but at Peace with each other.
fn tara() -> (GameState, CityId) {
    let (mut s, c) = lucerne();
    make_peace(&mut s, 1, 2);
    (s, c)
}

fn make_peace(s: &mut GameState, a: u8, b: u8) {
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        if (r.a, r.b) == (TeamId(a), TeamId(b)) || (r.a, r.b) == (TeamId(b), TeamId(a)) {
            r.state = crate::diplomacy::RelationState::Peace;
        }
    }
}

fn bounces(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, Event::Bounce { .. }))
        .count()
}

#[test]
fn tara_attackers_at_peace_with_each_other_settle_by_force() {
    let (mut s, c) = tara();
    // A whole turn: keep both rivals in the game and able to pay upkeep.
    for p in [1u8, 2] {
        city(&mut s, p, hex(1 + 8 * (p as i32 - 1), 1), 3);
        s.players[p as usize].gold = 500;
    }
    let phalanx = send(&mut s, 1, UnitKind::Phalangite, 0);
    let horse = send(&mut s, 2, UnitKind::Horseman, 3);
    let archer = unit(&mut s, 2, UnitKind::Ranger, hex(5, 4).neighbor(4));
    let f1 = battle::city_force(&s, phalanx, c, false);
    let f2 = battle::city_force(&s, horse, c, false) + battle::city_force(&s, archer, c, true);
    assert_ne!(f1, f2, "the fixture needs a stronger side");
    let (winner, loser, wf, lf, lost) = if f2 > f1 {
        (2u8, 1u8, f2, f1, phalanx)
    } else {
        (1u8, 2u8, f1, f2, horse)
    };
    let orders = BTreeMap::from([(
        PlayerId(2),
        vec![Order::RangedAttack {
            unit: archer,
            target: hex(5, 4),
        }],
    )]);
    let (next, events) = resolve_turn(&s, &orders);
    assert_eq!(next.cities[&c].owner, PlayerId(winner));
    assert_eq!(bounces(&events), 0, "{events:#?}");
    assert_eq!(
        battles_between_attackers(&events, PlayerId(1), PlayerId(2)),
        0
    );
    let name = s.player(PlayerId(winner)).name.clone();
    assert_eq!(
        reasons(&events, lost),
        vec![format!(
            "{name} took the city with force {wf} against your {lf}"
        )]
    );
    let _ = loser;
}

#[test]
fn tara_both_peaceful_attackers_kill_the_garrison_and_the_stronger_enters_that_round() {
    let (mut s, c) = tara();
    let g = unit(&mut s, 0, UnitKind::Soldier, hex(5, 4));
    s.units.get_mut(&g).unwrap().hp = 5;
    let a1 = send(&mut s, 1, UnitKind::Soldier, 0);
    let a2 = send(&mut s, 1, UnitKind::Soldier, 1);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&g), "the garrison fell");
    assert_eq!(next.cities[&c].owner, PlayerId(1), "that same round");
    assert!([a1, a2].iter().any(|u| next.units[u].pos == hex(5, 4)));
    assert_ne!(next.units[&b].pos, hex(5, 4));
    assert_eq!(bounces(&events), 0, "{events:#?}");
    let blocked = reasons(&events, b);
    assert_eq!(blocked.len(), 1, "{events:#?}");
    assert!(
        blocked[0].contains(" took the city with force "),
        "{blocked:?}"
    );
}

#[test]
fn tara_equal_force_says_so_instead_of_bouncing_silently() {
    let (mut s, c) = tara();
    let a = send(&mut s, 1, UnitKind::Soldier, 0);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let force = battle::city_force(&s, a, c, false);
    assert_eq!(force, battle::city_force(&s, b, c, false));
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    assert_eq!(bounces(&events), 0, "{events:#?}");
    let name = |p: u8| s.player(PlayerId(p)).name.clone();
    assert_eq!(
        reasons(&events, a),
        vec![format!(
            "{} brought equal force ({force} each), so nobody took the city",
            name(2)
        )]
    );
    assert_eq!(
        reasons(&events, b),
        vec![format!(
            "{} brought equal force ({force} each), so nobody took the city",
            name(1)
        )]
    );
}

#[test]
fn tara_a_third_attacker_at_peace_with_two_warring_attackers_stays_its_own_side() {
    let mut s = game(&[0, 1, 2, 3]);
    let c = city(&mut s, 0, hex(5, 4), 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    make_peace(&mut s, 1, 3);
    make_peace(&mut s, 2, 3);
    let a = send(&mut s, 1, UnitKind::Soldier, 0);
    let b = send(&mut s, 2, UnitKind::Soldier, 3);
    let t1 = send(&mut s, 3, UnitKind::Soldier, 1);
    let t2 = send(&mut s, 3, UnitKind::Soldier, 2);
    let force = battle::city_force(&s, a, c, false);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(
        next.cities[&c].owner,
        PlayerId(3),
        "two units outweigh one each"
    );
    assert!([t1, t2].iter().any(|u| next.units[u].pos == hex(5, 4)));
    let name = s.player(PlayerId(3)).name.clone();
    for loser in [a, b] {
        assert_eq!(
            reasons(&events, loser),
            vec![format!(
                "{name} took the city with force {} against your {force}",
                2 * force
            )]
        );
    }
}

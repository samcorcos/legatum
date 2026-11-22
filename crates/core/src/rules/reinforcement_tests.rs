//! Reinforcing your own city goes first (user, 2026-10-04, turn 107: a unit ordered into its
//! own threatened city bounced while the enemy captured it). Within a movement round a unit
//! stepping into its owner's own empty city, which an enemy attacks that round, resolves
//! before the attack and is the garrison the attack meets. The owner's further units stepping
//! in are its supporters (designer, 2026-11-15, after the relief chain of 2026-11-11, Bukhara
//! T67): they share the damage past the city's HP with the garrison, each adds its own
//! counterattack, the one that soaks most backfills a fallen garrison, and the city is captured
//! only when all of them are dead.
use super::*;
use crate::events::Defender;
use crate::rules::test_support::*;
use crate::{Order, resolve_turn};

fn reinforced(events: &[Event]) -> Vec<(UnitId, CityId)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::CityReinforced { unit, city, .. } => Some((*unit, *city)),
            _ => None,
        })
        .collect()
}

fn index_of(events: &[Event], f: impl Fn(&Event) -> bool) -> usize {
    events
        .iter()
        .position(f)
        .unwrap_or_else(|| panic!("event missing from {events:#?}"))
}

/// P1 owns a city at (5,4); P0's attacker stands west of it and P1's defender east.
struct Siege {
    s: GameState,
    city: CityId,
    attacker: UnitId,
    defender: UnitId,
}

fn siege(city_hp: i32) -> Siege {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(5, 4), 3);
    s.cities.get_mut(&c).unwrap().hp = city_hp;
    let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
    let defender = unit(&mut s, 1, UnitKind::Spearman, hex(6, 4));
    goto(&mut s, attacker, hex(5, 4));
    goto(&mut s, defender, hex(5, 4));
    Siege {
        s,
        city: c,
        attacker,
        defender,
    }
}

#[test]
fn reinforcement_beats_a_same_round_capture_and_the_enemy_fights_the_garrison() {
    let Siege {
        s,
        city,
        attacker,
        defender,
    } = siege(0);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(reinforced(&events), vec![(defender, city)]);
    assert_eq!(next.cities[&city].owner, PlayerId(1), "the city holds");
    assert_eq!(next.units[&defender].pos, hex(5, 4));
    assert_eq!(
        next.units[&attacker].pos,
        hex(4, 4),
        "the attack bounced off"
    );
    // The garrison arrived first; the attack then fought it as a unit (the city is at 0 HP).
    let moved = index_of(
        &events,
        |e| matches!(e, Event::UnitMoved { unit, .. } if *unit == defender),
    );
    let fought = index_of(&events, |e| {
        matches!(e, Event::Battle { defender: Defender::Unit { unit, .. }, meeting: false, .. }
            if *unit == defender)
    });
    assert!(moved < fought, "{events:#?}");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Battle { meeting: true, .. })),
        "no meeting engagement over the ruin"
    );
    assert!(next.units[&defender].hp < 100, "the garrison took the hit");
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { .. } | Event::Bounce { .. }))
    );
}

#[test]
fn a_city_with_hp_absorbs_and_overflow_wounds_the_new_garrison() {
    let Siege {
        s, city, defender, ..
    } = siege(5);
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(reinforced(&events), vec![(defender, city)]);
    assert!(events.iter().any(|e| matches!(e,
        Event::Battle { defender: Defender::City { city: c, .. }, .. } if *c == city)));
    assert_eq!(next.cities[&city].hp, 0);
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    assert!(
        next.units[&defender].hp > 0 && next.units[&defender].hp < 100,
        "overflow reached the garrison"
    );
}

#[test]
fn overflow_can_still_kill_the_new_garrison_and_capture() {
    let Siege {
        mut s,
        city,
        attacker,
        defender,
    } = siege(1);
    s.units.get_mut(&defender).unwrap().hp = 1;
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(reinforced(&events), vec![(defender, city)]);
    assert!(!next.units.contains_key(&defender), "overflow killed it");
    assert_eq!(next.cities[&city].owner, PlayerId(0), "captured");
    assert_eq!(next.units[&attacker].pos, hex(5, 4));
}

#[test]
fn a_city_captured_in_an_earlier_round_means_the_reinforcement_is_too_late() {
    let Siege {
        mut s,
        city,
        attacker,
        defender,
    } = siege(0);
    // The defender starts two steps away: the attacker takes the empty ruin in round one.
    s.units.get_mut(&defender).unwrap().pos = hex(7, 4);
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(reinforced(&events).is_empty(), "{events:#?}");
    assert_eq!(next.cities[&city].owner, PlayerId(0));
    assert_eq!(next.units[&attacker].pos, hex(5, 4));
    assert_ne!(next.units[&defender].pos, hex(5, 4));
}

#[test]
fn allies_get_no_priority() {
    // P0 and P1 are one team; P2 attacks P0's city and P1's unit hurries in.
    let mut s = game(&[0, 0, 1]);
    let c = city(&mut s, 0, hex(5, 4), 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let attacker = unit(&mut s, 2, UnitKind::Spearman, hex(4, 4));
    let ally = unit(&mut s, 1, UnitKind::Spearman, hex(6, 4));
    goto(&mut s, attacker, hex(5, 4));
    goto(&mut s, ally, hex(5, 4));
    let (_, events) = resolve_units(&s, &no_orders());
    assert!(reinforced(&events).is_empty(), "{events:#?}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Battle { meeting: true, .. })),
        "the ally and the attacker contest the ruin as before"
    );
}

#[test]
fn no_siege_no_priority_and_ranged_units_do_not_reinforce() {
    // Nobody attacks: an ordinary move, no event.
    let Siege {
        mut s,
        city,
        attacker,
        defender,
    } = siege(0);
    s.units.get_mut(&attacker).unwrap().goto = None;
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(reinforced(&events).is_empty());
    assert_eq!(next.units[&defender].pos, hex(5, 4));
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    // A ranged unit cannot be the garrison that holds a ruin against the capture rule.
    let Siege {
        mut s, defender, ..
    } = siege(0);
    s.units.get_mut(&defender).unwrap().kind = UnitKind::Archer;
    let (_, events) = resolve_units(&s, &no_orders());
    assert!(reinforced(&events).is_empty(), "{events:#?}");
}

#[test]
fn two_reinforcers_resolve_most_soak_then_lowest_id_first_in_any_order() {
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(5, 4), 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    city(&mut s, 0, hex(1, 1), 3);
    let attacker = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
    let r1 = unit(&mut s, 1, UnitKind::Spearman, hex(6, 4));
    let r2 = unit(&mut s, 1, UnitKind::Spearman, hex(6, 3));
    let mv = |unit, to| Order::MoveUnit { unit, to };
    let enemy = vec![mv(attacker, hex(5, 4))];
    let mut results = Vec::new();
    for mine in [
        vec![mv(r1, hex(5, 4)), mv(r2, hex(5, 4))],
        vec![mv(r2, hex(5, 4)), mv(r1, hex(5, 4))],
    ] {
        let orders = BTreeMap::from([(PlayerId(0), enemy.clone()), (PlayerId(1), mine)]);
        results.push(resolve_turn(&s, &orders));
    }
    assert_eq!(
        results[0], results[1],
        "order of submission changes nothing"
    );
    let (next, events) = &results[0];
    assert_eq!(
        reinforced(events),
        vec![(r1, c)],
        "equally strong: the lower unit id garrisons"
    );
    assert_eq!(next.units[&r1].pos, hex(5, 4));
    assert_ne!(next.units[&r2].pos, hex(5, 4), "no stacking behind it");
    assert_eq!(next.cities[&c].owner, PlayerId(1));
    // Both beside the city: the healthier unit goes in first, whatever its id or tile.
    let mut s = s.clone();
    s.units.get_mut(&r2).unwrap().pos = hex(5, 4).neighbor(1);
    s.units.get_mut(&r1).unwrap().hp = 60;
    let orders = BTreeMap::from([
        (PlayerId(0), enemy),
        (PlayerId(1), vec![mv(r1, hex(5, 4)), mv(r2, hex(5, 4))]),
    ]);
    let (next, events) = resolve_turn(&s, &orders);
    assert_eq!(reinforced(&events), vec![(r2, c)]);
    assert_eq!(next.units[&r2].pos, hex(5, 4));
    assert_ne!(next.units[&r1].pos, hex(5, 4), "the second waits outside");
}

#[test]
fn the_turn_107_shape_a_pikeman_beats_two_attackers_to_its_ruined_city() {
    // The user's save (turn 107): a wounded Pikeman ordered into its own 0 HP city with no
    // garrison, two enemy Spearmen ordered in from the other side; the city fell and the
    // Pikeman was bounced. The save comes from an older engine (Pacts and Influence) that
    // current code cannot load, so this rebuilds its shape.
    let mut s = game(&[0, 1]);
    let c = city(&mut s, 1, hex(5, 4), 2);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let a1 = unit(&mut s, 0, UnitKind::Spearman, hex(4, 4));
    let a2 = unit(&mut s, 0, UnitKind::Spearman, hex(4, 5));
    s.units.get_mut(&a2).unwrap().hp = 46;
    let pikeman = unit(&mut s, 1, UnitKind::Pikeman, hex(6, 4));
    s.units.get_mut(&pikeman).unwrap().hp = 76;
    for u in [a1, a2, pikeman] {
        goto(&mut s, u, hex(5, 4));
    }
    let (next, events) = resolve_units(&s, &no_orders());
    assert_eq!(reinforced(&events), vec![(pikeman, c)]);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Bounce { .. } | Event::MoveBlocked { .. })),
        "{events:#?}"
    );
    let fought: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(e, Event::Battle { defender: Defender::Unit { unit, .. }, meeting: false, .. }
                if *unit == pikeman)
        })
        .collect();
    assert_eq!(fought.len(), 2, "both attackers fight the garrison");
    // Whoever wins, the Pikeman was there first: it holds the city unless it fell in the fight.
    let holds = next.units.contains_key(&pikeman);
    assert_eq!(
        next.cities[&c].owner,
        if holds { PlayerId(1) } else { PlayerId(0) }
    );
}

// Relief chains into an assaulted city, never bounces (user, 2026-11-11, Bukhara T67).

fn blocked_reasons(events: &[Event], unit: UnitId) -> Vec<String> {
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

fn no_contest(events: &[Event]) {
    assert!(
        !events.iter().any(|e| matches!(e, Event::Bounce { .. })
            || matches!(e, Event::MoveBlocked { reason, .. } if reason.contains("contested"))),
        "{events:#?}"
    );
}

/// Bukhara T67: Bukhara (Mongol) at 0 HP; its 4-HP Keshig steps out while a fresh Phalangite
/// steps in; three Swiss Phalangites move onto the city.
struct Bukhara {
    s: GameState,
    city: CityId,
    keshig: UnitId,
    relief: UnitId,
    attackers: Vec<UnitId>,
}

fn bukhara() -> Bukhara {
    let mut s = game(&[0, 1]);
    s.player_mut(PlayerId(0)).faction = Faction::Switzerland;
    s.player_mut(PlayerId(1)).faction = Faction::Mongols;
    let at = hex(5, 4);
    let c = city(&mut s, 1, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let keshig = unit(&mut s, 1, data::KESHIG_KIND, at);
    s.units.get_mut(&keshig).unwrap().hp = 4;
    goto(&mut s, keshig, at.neighbor(0));
    let relief = unit(&mut s, 1, UnitKind::Phalangite, at.neighbor(1));
    goto(&mut s, relief, at);
    let attackers = [3, 4, 5]
        .map(|dir| {
            let u = unit(&mut s, 0, UnitKind::Phalangite, at.neighbor(dir));
            goto(&mut s, u, at);
            u
        })
        .to_vec();
    Bukhara {
        s,
        city: c,
        keshig,
        relief,
        attackers,
    }
}

#[test]
fn bukhara_t67_shape_with_a_stronger_garrison_the_relief_takes_the_leftover_and_holds() {
    let Bukhara {
        mut s,
        city,
        keshig,
        relief,
        attackers,
    } = bukhara();
    s.units.get_mut(&keshig).unwrap().hp = 70;
    let (next, events) = resolve_units(&s, &no_orders());
    no_contest(&events);
    assert!(
        !next.units.contains_key(&keshig),
        "the parting hits kill it"
    );
    assert_eq!(reinforced(&events), vec![(relief, city)]);
    let r = &next.units[&relief];
    assert_eq!(r.pos, hex(5, 4), "the relief is the garrison");
    assert!(r.hp < 100, "it took what the Keshig did not absorb");
    assert_eq!(next.cities[&city].owner, PlayerId(1), "the city holds");
    for a in &attackers {
        assert_ne!(next.units[a].pos, hex(5, 4));
        assert_eq!(
            blocked_reasons(&events, *a),
            vec![CITY_REINFORCED.to_string()]
        );
    }
    // Only the Keshig was fought: overflow into the relief adds no Battle and no counterattack.
    assert!(events.iter().all(|e| !matches!(e,
        Event::Battle { defender: Defender::Unit { unit, .. }, .. } if *unit == relief)));
}

#[test]
fn bukhara_t67_each_hit_carries_only_its_unused_share_so_the_relief_holds() {
    // The replayed shape: three Phalangite hits (51, 43, 52 against the Keshig) on a 4-HP
    // Keshig. Only the first one's unused 47/51 reaches the relief, as that share of what the
    // same Phalangite deals to a Phalangite; the next two find the Keshig dead and strike the
    // relief directly. The pooled 146 of the first rule killed it; now it survives, wounded.
    let Bukhara {
        s,
        city,
        keshig,
        relief,
        attackers,
    } = bukhara();
    let (next, events) = resolve_units(&s, &no_orders());
    no_contest(&events);
    assert!(!next.units.contains_key(&keshig));
    assert_eq!(reinforced(&events), vec![(relief, city)]);
    let r = &next.units[&relief];
    assert_eq!(r.pos, hex(5, 4), "the relief is the garrison");
    assert!((5..=15).contains(&r.hp), "about 10 HP left, got {}", r.hp);
    assert_eq!(next.cities[&city].owner, PlayerId(1), "the city holds");
    for a in &attackers {
        assert_ne!(next.units[a].pos, hex(5, 4));
    }
}

#[test]
fn a_relief_unit_weak_to_the_attackers_class_dies_and_the_assault_captures() {
    // The same 95-HP relief, judged by what the Phalangites deal to its own type: a Phalangite
    // holds on 5 HP, while a mounted unit (Melee line against mounted, +40% on every hit that
    // reaches it) takes a little more and dies inside the city.
    let held = {
        let Bukhara { mut s, relief, .. } = bukhara();
        s.units.get_mut(&relief).unwrap().hp = 95;
        let (next, _) = resolve_units(&s, &no_orders());
        next.units.get(&relief).map(|u| u.hp)
    };
    assert!(
        held.is_some_and(|hp| hp > 0),
        "the Phalangite relief holds: {held:?}"
    );
    let Bukhara {
        mut s,
        city,
        keshig,
        relief,
        ..
    } = bukhara();
    s.units.get_mut(&relief).unwrap().kind = data::KESHIG_KIND;
    s.units.get_mut(&relief).unwrap().hp = 95;
    let (next, events) = resolve_units(&s, &no_orders());
    no_contest(&events);
    assert!(!next.units.contains_key(&keshig));
    assert!(
        !next.units.contains_key(&relief),
        "the mounted relief falls inside the city"
    );
    assert_eq!(next.cities[&city].owner, PlayerId(0), "captured that step");
}

/// A fallen city at (5,4) with a Spearman garrison on `garrison_hp`, P1's Spearmen beside it on
/// `supporter_hps` ordered in, and P0's Spearmen west of it ordered in.
struct Supported {
    s: GameState,
    city: CityId,
    garrison: UnitId,
    supporters: Vec<UnitId>,
    attackers: Vec<UnitId>,
}

fn supported(garrison_hp: i32, supporter_hps: &[i32], attackers: usize) -> Supported {
    let mut s = game(&[0, 1]);
    let at = hex(5, 4);
    let c = city(&mut s, 1, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let garrison = unit(&mut s, 1, UnitKind::Spearman, at);
    s.units.get_mut(&garrison).unwrap().hp = garrison_hp;
    let supporters = supporter_hps
        .iter()
        .enumerate()
        .map(|(d, hp)| {
            let u = unit(&mut s, 1, UnitKind::Spearman, at.neighbor(d));
            s.units.get_mut(&u).unwrap().hp = *hp;
            goto(&mut s, u, at);
            u
        })
        .collect();
    let attackers = [3, 4, 5][..attackers]
        .iter()
        .map(|d| {
            let u = unit(&mut s, 0, UnitKind::Spearman, at.neighbor(*d));
            goto(&mut s, u, at);
            u
        })
        .collect();
    Supported {
        s,
        city: c,
        garrison,
        supporters,
        attackers,
    }
}

#[test]
fn supporters_share_the_hits_with_the_garrison_and_stay_on_their_tiles() {
    // Alone, the 40-HP garrison falls to two Spearmen; shared with two supporters it holds.
    let alone = supported(40, &[], 2);
    let (next, _) = resolve_units(&alone.s, &no_orders());
    assert!(!next.units.contains_key(&alone.garrison), "alone it falls");
    assert_eq!(next.cities[&alone.city].owner, PlayerId(0));
    let Supported {
        s,
        city,
        garrison,
        supporters,
        attackers,
    } = supported(40, &[30, 29], 2);
    let (next, events) = resolve_units(&s, &no_orders());
    no_contest(&events);
    let g = &next.units[&garrison];
    assert!(
        g.hp > 0 && g.hp < 40,
        "the garrison took its share: {}",
        g.hp
    );
    for (u, start) in supporters.iter().zip([30, 29]) {
        let x = &next.units[u];
        assert!(x.hp < start, "each supporter took a share");
        assert_ne!(x.pos, hex(5, 4), "it stays on its own tile");
        assert_eq!(blocked_reasons(&events, *u), vec![CITY_HELD.to_string()]);
    }
    assert!(reinforced(&events).is_empty(), "nobody needed to enter");
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    for a in &attackers {
        assert_ne!(next.units[a].pos, hex(5, 4));
    }
    // Supporters fight no Battle of their own.
    assert!(events.iter().all(|e| !matches!(e,
        Event::Battle { defender: Defender::Unit { unit, .. }, .. } if supporters.contains(unit))));
}

#[test]
fn shared_damage_splits_each_hit_evenly_with_the_remainder_to_the_garrison() {
    // One Spearman hit on the garrison of a fallen city, shared by three Spearmen of equal HP
    // and the same terrain: the garrison (fortified) takes the remainder, each supporter
    // `floor(direct × (B / 3) / B)` of what that Spearman deals to it unfortified.
    let Supported {
        s,
        city,
        garrison,
        supporters,
        attackers,
    } = supported(100, &[100, 100], 1);
    let f = battle::plan_fight(
        &s,
        1,
        attackers[0],
        Target::Unit(garrison),
        FightKind::Melee,
    );
    let b = f.damage_to_defender;
    let leaving: BTreeSet<UnitId> = supporters.iter().copied().collect();
    let reserves = BTreeMap::from([(city, supporters.clone())]);
    let carry = battle::carried_damage(&s, std::slice::from_ref(&f), &leaving, &reserves);
    assert_eq!(carry.hit.get(&0), Some(&(b / 3 + b % 3)));
    for u in &supporters {
        let share = carry.damage[u];
        assert!(
            share >= b / 3,
            "unfortified, each takes at least its part: {share} of {b}"
        );
    }
    assert_eq!(carry.relief.len(), 2);
}

#[test]
fn the_garrison_falls_and_the_healthiest_supporter_takes_its_place_in_any_order() {
    let Supported {
        s,
        city,
        garrison,
        supporters,
        attackers,
    } = supported(5, &[50, 90], 2);
    let mv = |unit, to| Order::MoveUnit { unit, to };
    let enemy: Vec<Order> = attackers.iter().map(|a| mv(*a, hex(5, 4))).collect();
    let mut results = Vec::new();
    for mine in [
        vec![mv(supporters[0], hex(5, 4)), mv(supporters[1], hex(5, 4))],
        vec![mv(supporters[1], hex(5, 4)), mv(supporters[0], hex(5, 4))],
    ] {
        let orders = BTreeMap::from([(PlayerId(0), enemy.clone()), (PlayerId(1), mine)]);
        results.push(resolve_turn(&s, &orders));
    }
    assert_eq!(
        results[0], results[1],
        "order of submission changes nothing"
    );
    let (next, events) = &results[0];
    assert!(!next.units.contains_key(&garrison));
    assert_eq!(reinforced(events), vec![(supporters[1], city)]);
    assert_eq!(next.units[&supporters[1]].pos, hex(5, 4));
    assert!(
        next.units[&supporters[1]].hp < 90,
        "it took its share first"
    );
    assert_ne!(next.units[&supporters[0]].pos, hex(5, 4));
    assert!(next.units[&supporters[0]].hp < 50);
    assert_eq!(
        blocked_reasons(events, supporters[0]),
        vec![CITY_HELD.to_string()]
    );
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    for a in &attackers {
        assert_eq!(
            blocked_reasons(events, *a),
            vec![CITY_REINFORCED.to_string()]
        );
    }
}

#[test]
fn the_city_falls_only_when_the_garrison_and_every_supporter_are_dead() {
    let Supported {
        s,
        city,
        garrison,
        supporters,
        attackers,
    } = supported(4, &[4, 4], 3);
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&garrison));
    for u in &supporters {
        assert!(!next.units.contains_key(u), "{events:#?}");
    }
    assert_eq!(next.cities[&city].owner, PlayerId(0), "captured that round");
    assert!(attackers.iter().any(|a| next.units[a].pos == hex(5, 4)));
}

fn damage_taken(events: &[Event], attacker: UnitId) -> i32 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Battle {
                attacker: crate::events::Attacker::Unit(a),
                damage_to_attacker,
                ..
            } if *a == attacker => Some(*damage_to_attacker),
            _ => None,
        })
        .sum()
}

#[test]
fn every_supporter_adds_its_own_counterattack_to_the_pooled_reply() {
    // Two attackers on a garrison with two supporters: the garrison's reply plus one reply per
    // supporter (each against the strongest attacker, same roll), split evenly with the
    // remainder to the strongest.
    let alone = supported(100, &[], 2);
    let (_, events) = resolve_units(&alone.s, &no_orders());
    let alone_total: i32 = alone
        .attackers
        .iter()
        .map(|a| damage_taken(&events, *a))
        .sum();
    let Supported { s, attackers, .. } = supported(100, &[100, 100], 2);
    let (_, events) = resolve_units(&s, &no_orders());
    let taken: Vec<i32> = attackers
        .iter()
        .map(|a| damage_taken(&events, *a))
        .collect();
    let total: i32 = taken.iter().sum();
    assert!(
        total > 2 * alone_total,
        "three defenders reply, not one: {total} vs {alone_total}"
    );
    assert!(taken[0].abs_diff(taken[1]) <= 1, "split evenly: {taken:?}");
    // Ranged attacks still draw no reply, supported or not.
    let Supported {
        mut s,
        garrison,
        attackers,
        ..
    } = supported(100, &[100, 100], 1);
    s.units.get_mut(&attackers[0]).unwrap().goto = None;
    let archer = unit(&mut s, 0, UnitKind::Archer, hex(3, 4));
    let shot = BTreeMap::from([(
        PlayerId(0),
        vec![Order::RangedAttack {
            unit: archer,
            target: s.units[&garrison].pos,
        }],
    )]);
    let (_, events) = resolve_units(&s, &shot);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Battle { ranged: true, .. })),
        "{events:#?}"
    );
    assert_eq!(damage_taken(&events, archer), 0);
}

#[test]
fn the_backfill_is_the_supporter_that_soaks_most_not_the_one_with_most_hp() {
    // Designer, 2026-11-15: "a warrior in the late game might have the most HP, but they can't
    // absorb it". A full-HP Spearman and a wounded Rifleman support a dying garrison against
    // Riflemen; the Rifleman, which takes far less per hit, enters the city.
    let mut s = game(&[0, 1]);
    for p in [0, 1] {
        know_eras_before(&mut s, PlayerId(p), 4);
    }
    let at = hex(5, 4);
    let c = city(&mut s, 1, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let garrison = unit(&mut s, 1, UnitKind::Rifleman, at);
    s.units.get_mut(&garrison).unwrap().hp = 3;
    let spearman = unit(&mut s, 1, UnitKind::Spearman, at.neighbor(0));
    let rifleman = unit(&mut s, 1, UnitKind::Rifleman, at.neighbor(1));
    s.units.get_mut(&rifleman).unwrap().hp = 45;
    let attackers: Vec<UnitId> = [3]
        .map(|d| unit(&mut s, 0, UnitKind::Rifleman, at.neighbor(d)))
        .to_vec();
    for u in [spearman, rifleman].iter().chain(&attackers) {
        goto(&mut s, *u, at);
    }
    let terrain = Terrain::Grassland;
    assert!(
        battle::soak(&s, rifleman, &attackers, terrain)
            > battle::soak(&s, spearman, &attackers, terrain),
        "a 45-HP Rifleman soaks more than a 100-HP Spearman"
    );
    let (next, events) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&garrison));
    assert!(
        next.units[&spearman].hp > next.units[&rifleman].hp,
        "the Spearman still has more HP: {} vs {}",
        next.units[&spearman].hp,
        next.units[&rifleman].hp
    );
    assert_eq!(reinforced(&events), vec![(rifleman, c)]);
    assert_eq!(next.units[&rifleman].pos, at);
    assert_eq!(next.cities[&c].owner, PlayerId(1));
}

#[test]
fn a_supporter_attacked_where_it_stands_gives_no_support() {
    let Supported {
        mut s,
        garrison,
        supporters,
        ..
    } = supported(100, &[100], 1);
    let cutter = unit(
        &mut s,
        0,
        UnitKind::Spearman,
        hex(5, 4).neighbor(0).neighbor(0),
    );
    let sup_at = s.units[&supporters[0]].pos;
    goto(&mut s, cutter, sup_at);
    let supported_hp = {
        let (next, _) = resolve_units(&supported(100, &[100], 1).s, &no_orders());
        next.units[&garrison].hp
    };
    let (next, _) = resolve_units(&s, &no_orders());
    assert!(
        next.units[&garrison].hp < supported_hp,
        "the cut garrison takes the whole hit: {} vs {supported_hp}",
        next.units[&garrison].hp
    );
}

#[test]
fn a_garrison_that_survives_its_parting_hit_hands_the_city_to_the_relief_unharmed() {
    let Bukhara {
        mut s,
        city,
        keshig,
        relief,
        attackers,
    } = bukhara();
    s.units.get_mut(&keshig).unwrap().hp = 100;
    for a in &attackers[1..] {
        s.units.get_mut(a).unwrap().goto = None;
    }
    let (next, events) = resolve_units(&s, &no_orders());
    no_contest(&events);
    assert!(next.units[&keshig].hp < 100, "parting hit (G21)");
    assert_eq!(next.units[&keshig].pos, hex(5, 4).neighbor(0));
    assert_eq!(next.units[&relief].pos, hex(5, 4));
    assert_eq!(next.units[&relief].hp, 100, "nothing was left over for it");
    assert_eq!(reinforced(&events), vec![(relief, city)]);
    assert_eq!(next.cities[&city].owner, PlayerId(1));
    assert_eq!(
        blocked_reasons(&events, attackers[0]),
        vec![CITY_REINFORCED.to_string()]
    );
}

// The designer's worked example (2026-11-15): a walled Industrial city with a Rifleman garrison,
// three enemy Riflemen assaulting it in melee, three friendly Riflemen beside it on the far side
// ordered in every turn. Average rolls; run turn by turn until the city falls or the attack is
// spent.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    NoSupport,
    Supported,
    /// Three more enemy Riflemen each attack one supporter where it stands.
    SupportCut,
}

struct Example {
    rows: Vec<String>,
    /// (turn, captured) at the end.
    end: (u32, bool),
}

fn rifleman_example(variant: Variant) -> Example {
    battle::AVERAGE_ROLLS.with(|c| c.set(true));
    let mut s = game(&[0, 1]);
    for p in [0, 1] {
        know_eras_before(&mut s, PlayerId(p), 4);
    }
    let at = hex(5, 4);
    let c = walled_city(&mut s, 1, at, 5);
    let garrison = unit(&mut s, 1, UnitKind::Rifleman, at);
    s.units.get_mut(&garrison).unwrap().fortify = 0;
    let supporters: Vec<UnitId> = if variant == Variant::NoSupport {
        Vec::new()
    } else {
        [0, 1, 2]
            .map(|d| unit(&mut s, 1, UnitKind::Rifleman, at.neighbor(d)))
            .to_vec()
    };
    let attackers: Vec<UnitId> = [3, 4, 5]
        .map(|d| unit(&mut s, 0, UnitKind::Rifleman, at.neighbor(d)))
        .to_vec();
    let cutters: Vec<(UnitId, UnitId)> = if variant == Variant::SupportCut {
        supporters
            .iter()
            .enumerate()
            .map(|(d, sup)| {
                let h = at.neighbor(d).neighbor(d);
                (unit(&mut s, 0, UnitKind::Rifleman, h), *sup)
            })
            .collect()
    } else {
        Vec::new()
    };
    let hp = |s: &GameState, u: UnitId| s.units.get(&u).map_or(0, |u| u.hp);
    let mut rows = vec!["turn | city HP | garrison | supporters | attackers | owner".to_string()];
    let row = |s: &GameState, turn: u32| {
        let g = s
            .units
            .values()
            .find(|u| u.pos == at && u.owner == PlayerId(1))
            .map_or("-".to_string(), |u| format!("#{} {}", u.id.0, u.hp));
        let sup: Vec<String> = supporters
            .iter()
            .map(|u| match s.units.get(u) {
                Some(x) if x.pos == at => format!("#{} in city", u.0),
                Some(x) => format!("#{} {}", u.0, x.hp),
                None => format!("#{} dead", u.0),
            })
            .collect();
        let att: Vec<String> = attackers
            .iter()
            .chain(cutters.iter().map(|(u, _)| u))
            .map(|u| format!("{}", hp(s, *u)))
            .collect();
        format!(
            "{turn:>4} | {:>7} | {:>8} | {} | {} | P{}",
            s.cities[&c].hp,
            g,
            sup.join(", "),
            att.join(", "),
            s.cities[&c].owner.0
        )
    };
    rows.push(row(&s, 0));
    let mut end = (0, false);
    for turn in 1..=12 {
        for a in &attackers {
            if s.units.contains_key(a) {
                goto(&mut s, *a, at);
            }
        }
        for (cut, sup) in &cutters {
            if s.units.contains_key(cut) {
                let to = s
                    .units
                    .get(sup)
                    .filter(|u| u.pos != at)
                    .map_or(at, |u| u.pos);
                goto(&mut s, *cut, to);
            }
        }
        for u in &supporters {
            if s.units.get(u).is_some_and(|x| x.pos != at) {
                goto(&mut s, *u, at);
            }
        }
        s = resolve_units(&s, &no_orders()).0;
        rows.push(row(&s, turn));
        let captured = s.cities[&c].owner == PlayerId(0);
        let spent = attackers
            .iter()
            .chain(cutters.iter().map(|(u, _)| u))
            .all(|u| !s.units.contains_key(u));
        end = (turn, captured);
        if captured || spent {
            break;
        }
    }
    battle::AVERAGE_ROLLS.with(|c| c.set(false));
    Example { rows, end }
}

#[test]
fn rifleman_worked_example_prints_turn_by_turn() {
    // Run with --nocapture for the table. Alone the garrison falls on turn 2; with three
    // supporters the city holds and the three attackers are spent by turn 3; with every
    // supporter pinned by its own attacker the city falls on turn 3.
    let mut ends = Vec::new();
    for v in [Variant::NoSupport, Variant::Supported, Variant::SupportCut] {
        let e = rifleman_example(v);
        println!("{v:?}: ends turn {} captured {}", e.end.0, e.end.1);
        for r in &e.rows {
            println!("  {r}");
        }
        ends.push(e.end);
    }
    assert_eq!(ends, vec![(2, true), (3, false), (3, true)]);
}

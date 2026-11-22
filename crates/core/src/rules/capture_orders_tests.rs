//! Playtest 8.4, turn 31 (user): after five soldiers took Vestige, the four that did not move in
//! still showed a "move into this city" order. A city that changes hands ends every order of the
//! captor's team that aims at it; an attack whose target is gone ends too and the unit is ready
//! for orders.
use super::*;
use crate::rules::test_support::*;

fn no_order(s: &GameState, id: UnitId) -> bool {
    let u = &s.units[&id];
    u.goto.is_none() && u.path.is_empty() && !u.path_stop_on_block && u.attack_target.is_none()
}

#[test]
fn five_attackers_take_the_city_and_the_rest_are_ready_for_orders() {
    let mut s = game(&[0, 1]);
    let at = hex(5, 4);
    let c = city(&mut s, 1, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let garrison = unit(&mut s, 1, UnitKind::Soldier, at);
    s.units.get_mut(&garrison).unwrap().hp = 1;
    let attackers: Vec<UnitId> = at.neighbors()[..5]
        .iter()
        .map(|h| {
            let id = unit(&mut s, 0, UnitKind::Soldier, *h);
            goto(&mut s, id, at);
            id
        })
        .collect();
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    let entered: Vec<_> = attackers
        .iter()
        .filter(|a| next.units[*a].pos == at)
        .collect();
    assert_eq!(entered.len(), 1, "one soldier took the city: {entered:?}");
    for id in &attackers {
        assert!(no_order(&next, *id), "{id:?} still ordered at the city");
        let u = &next.units[id];
        assert_eq!(
            u.needs_orders,
            u.pos != at,
            "{id:?}: ready unless it moved in"
        );
        assert!(u.goto.is_none());
    }
}

#[test]
fn a_teammate_or_ally_capturing_the_city_ends_a_distant_route_into_it() {
    for ally in [false, true] {
        let mut s = game(&if ally { vec![0, 1, 2] } else { vec![0, 0, 1] });
        if ally {
            for r in &mut s.diplomacy.as_mut().unwrap().relations {
                if r.a == TeamId(0) && r.b == TeamId(1) {
                    r.state = crate::diplomacy::RelationState::Alliance;
                }
            }
        }
        let at = hex(5, 4);
        let target_owner = 2;
        let c = city(&mut s, target_owner, at, 3);
        s.cities.get_mut(&c).unwrap().hp = 0;
        let g = unit(&mut s, target_owner, UnitKind::Soldier, at);
        s.units.get_mut(&g).unwrap().hp = 1;
        let captor = unit(&mut s, 1, UnitKind::Soldier, hex(4, 4));
        goto(&mut s, captor, at);
        // Far away, with a confirmed route ending at the city.
        let far = unit(&mut s, 0, UnitKind::Soldier, hex(1, 4));
        let route = vec![hex(2, 4), hex(3, 4), at];
        {
            let u = s.units.get_mut(&far).unwrap();
            u.goto = Some(at);
            u.path = route;
            u.path_stop_on_block = true;
        }
        let (next, _) = resolve_units(&s, &no_orders());
        assert_eq!(next.cities[&c].owner, PlayerId(1), "ally={ally}");
        assert!(
            no_order(&next, far),
            "ally={ally}: route into the captured city"
        );
        assert!(next.units[&far].needs_orders, "ally={ally}");
    }
}

#[test]
fn an_enemy_of_the_captor_keeps_its_order_at_the_city() {
    let mut s = game(&[0, 1, 2]);
    let at = hex(5, 4);
    let c = city(&mut s, 2, at, 3);
    s.cities.get_mut(&c).unwrap().hp = 0;
    let g = unit(&mut s, 2, UnitKind::Soldier, at);
    s.units.get_mut(&g).unwrap().hp = 1;
    let captor = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    goto(&mut s, captor, at);
    let rival = unit(&mut s, 1, UnitKind::Soldier, hex(2, 4));
    goto(&mut s, rival, at);
    let (next, _) = resolve_units(&s, &no_orders());
    assert_eq!(next.cities[&c].owner, PlayerId(0));
    assert!(
        next.units[&rival].goto.is_some(),
        "a rival's war order still stands"
    );
}

#[test]
fn an_attack_on_a_unit_that_died_before_movement_ends_with_needs_orders() {
    let mut s = game(&[0, 1]);
    let victim = unit(&mut s, 1, UnitKind::Scout, hex(6, 4));
    let a = unit(&mut s, 0, UnitKind::Soldier, hex(4, 4));
    {
        let u = s.units.get_mut(&a).unwrap();
        u.goto = Some(hex(6, 4));
        u.attack_target = Some(victim);
    }
    s.units.remove(&victim);
    let (next, _) = resolve_units(&s, &no_orders());
    assert!(no_order(&next, a));
    assert!(next.units[&a].needs_orders);
}

#[test]
fn an_attack_on_a_unit_a_teammate_destroys_this_turn_ends_with_needs_orders() {
    let mut s = game(&[0, 0, 1]);
    // The victim is at the edge of a teammate's reach: it dies in the first round.
    let victim = unit(&mut s, 2, UnitKind::Scout, hex(6, 4));
    s.units.get_mut(&victim).unwrap().hp = 1;
    let killer = unit(&mut s, 1, UnitKind::Soldier, hex(5, 4));
    let pursuer = unit(&mut s, 0, UnitKind::Soldier, hex(3, 4));
    {
        let u = s.units.get_mut(&pursuer).unwrap();
        u.goto = Some(hex(6, 4));
        u.attack_target = Some(victim);
    }
    goto(&mut s, killer, hex(6, 4));
    s.units.get_mut(&killer).unwrap().attack_target = Some(victim);
    let (next, _) = resolve_units(&s, &no_orders());
    assert!(!next.units.contains_key(&victim), "the scout fell");
    assert!(no_order(&next, pursuer));
    assert!(next.units[&pursuer].needs_orders);
}

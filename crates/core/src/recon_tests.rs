//! Aircraft sight and the Recon order (user, 2026-11-15: a Bomber could not see targets to strike).
use crate::data;
use crate::rules::air;
use crate::rules::test_support::{city, game, hex};
use crate::visibility::visible_tiles;
use crate::*;

const P0: PlayerId = PlayerId(0);

fn seen(s: &GameState, h: Hex) -> bool {
    s.map
        .index(h)
        .is_some_and(|i| visible_tiles(s, TeamId(0)).get(i))
}

#[test]
fn every_aircraft_sees_four_tiles_around_its_city_over_any_terrain() {
    assert_eq!(data::AIRCRAFT_SIGHT, 4);
    for kind in UnitKind::ALL.into_iter().filter(|k| data::is_aircraft(*k)) {
        let mut s = game(&[0, 1]);
        let at = hex(6, 5);
        let home = city(&mut s, 0, at, 3);
        let outer = at.ring(data::AIRCRAFT_SIGHT);
        let beyond = at.ring(data::AIRCRAFT_SIGHT + 1);
        assert!(outer.iter().all(|h| s.map.index(*h).is_some()));
        for h in &outer {
            crate::rules::test_support::set_terrain(&mut s, *h, Terrain::Mountains);
        }
        assert!(
            outer.iter().all(|h| !seen(&s, *h)),
            "{kind:?}: no aircraft, no sight at 4"
        );
        air::base_new(&mut s, home, kind);
        assert!(
            outer.iter().all(|h| seen(&s, *h)),
            "{kind:?} sees every tile 4 from its city, mountains or not"
        );
        assert!(
            beyond.iter().all(|h| !seen(&s, *h)),
            "{kind:?} does not see 5 tiles out"
        );
    }
}

#[test]
fn a_rival_aircraft_gives_no_sight() {
    let mut s = game(&[0, 1]);
    let theirs = city(&mut s, 1, hex(6, 5), 3);
    air::base_new(&mut s, theirs, UnitKind::Bomber);
    assert!(!seen(&s, hex(6, 5).ring(4)[0]));
}

mod recon {
    use super::*;
    use crate::bitset::BitSet;
    use crate::orders::Order;
    use crate::rules::resolve_tactical_turn;
    use crate::rules::test_support::{orders, unit};
    use std::collections::BTreeMap;

    const P1: PlayerId = PlayerId(1);

    /// P0's city at (3,4) (sight 3, aircraft sight 4), P1's city at (7,4); P0 has one aircraft.
    fn arena(kind: UnitKind) -> (GameState, CityId, UnitId) {
        let mut s = game(&[0, 1]);
        let a = city(&mut s, 0, hex(3, 4), 3);
        city(&mut s, 1, hex(7, 4), 3);
        air::base_new(&mut s, a, kind);
        let plane = s.aircraft[0].id;
        (s, a, plane)
    }

    fn recon(plane: UnitId, target: Hex) -> Order {
        Order::AirRecon { plane, target }
    }

    fn errors(s: &GameState, os: Vec<Order>) -> Vec<String> {
        air::validate_orders(s, P0, &os, s.clone())
            .into_iter()
            .map(|e| e.message)
            .collect()
    }

    fn disc(s: &GameState, centre: Hex, radius: i32) -> BitSet {
        let mut tiles = BitSet::new(s.map.len());
        for h in centre.within_iter(radius) {
            if let Some(i) = s.map.index(h) {
                tiles.set(i);
            }
        }
        tiles
    }

    fn with_disc(s: &GameState, team: TeamId, centre: Hex) -> BitSet {
        let mut tiles = visible_tiles(s, team);
        for i in disc(s, centre, data::RECON_RADIUS).iter_ones() {
            tiles.set(i);
        }
        tiles
    }

    fn without_recons(s: &GameState) -> GameState {
        let mut bare = s.clone();
        bare.air_recons.clear();
        bare
    }

    #[test]
    fn a_recon_shows_exactly_the_tiles_around_its_target_on_the_next_turn_only() {
        let (s, _, plane) = arena(UnitKind::Bomber);
        let target = hex(8, 4);
        assert_eq!(data::RECON_RADIUS, 2);
        const { assert!(data::RECON_RADIUS < data::AIRCRAFT_SIGHT) };
        assert!(errors(&s, vec![recon(plane, target)]).is_empty());
        let (next, _) =
            crate::rules::test_support::resolve_units(&s, &orders(0, vec![recon(plane, target)]));
        assert_eq!(next.turn, s.turn + 1);
        assert_eq!(
            visible_tiles(&next, TeamId(0)),
            with_disc(&without_recons(&next), TeamId(0), target),
            "this turn's sight plus the 2-tile disc, nothing else"
        );
        assert!(
            !visible_tiles(&without_recons(&next), TeamId(0)).get(next.map.index(target).unwrap()),
            "the target was out of sight before"
        );
        assert_eq!(
            visible_tiles(&next, TeamId(1)),
            visible_tiles(&without_recons(&next), TeamId(1)),
            "the rival team sees nothing of it"
        );
        let mut same_turn = next.clone();
        same_turn.turn = s.turn;
        assert_eq!(
            visible_tiles(&same_turn, TeamId(0)),
            visible_tiles(&without_recons(&same_turn), TeamId(0)),
            "nothing is revealed on the turn it is ordered"
        );
        let (later, _) = crate::rules::test_support::resolve_units(
            &next,
            &crate::rules::test_support::no_orders(),
        );
        assert_eq!(later.turn, s.turn + 2);
        assert_eq!(
            visible_tiles(&later, TeamId(0)),
            visible_tiles(&without_recons(&later), TeamId(0)),
            "the reveal is gone after the turn it served"
        );
    }

    #[test]
    fn a_recon_lets_the_next_turns_strike_hit_a_target_that_was_unseen() {
        let (mut s, _, plane) = arena(UnitKind::Bomber);
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(10, 4));
        let strike = Order::AirStrike {
            plane,
            target: hex(10, 4),
        };
        assert!(
            !errors(&s, vec![strike.clone()]).is_empty(),
            "the target is in range but unseen"
        );
        let (next, _) = crate::rules::test_support::resolve_units(
            &s,
            &orders(0, vec![recon(plane, hex(9, 4))]),
        );
        assert!(errors(&next, vec![strike.clone()]).is_empty());
        let (hit, _) = resolve_tactical_turn(&next, &orders(0, vec![strike]));
        assert!(hit.units.get(&victim).is_none_or(|u| u.hp < 100));
    }

    #[test]
    fn allies_share_a_recon_and_each_player_sees_only_their_team() {
        let mut s = game(&[0, 0, 1]);
        let a = city(&mut s, 0, hex(3, 4), 3);
        city(&mut s, 1, hex(3, 8), 3);
        city(&mut s, 2, hex(9, 8), 3);
        air::base_new(&mut s, a, UnitKind::Bomber);
        let plane = s.aircraft[0].id;
        let target = hex(8, 4);
        let (next, _) =
            crate::rules::test_support::resolve_units(&s, &orders(0, vec![recon(plane, target)]));
        let i = next.map.index(target).unwrap();
        assert!(visible_tiles(&next, TeamId(0)).get(i));
        assert!(!visible_tiles(&next, TeamId(1)).get(i));
        assert_eq!(next.air_recons.len(), 1);
    }

    #[test]
    fn every_aircraft_recons_within_its_own_range_and_is_refused_beyond_it() {
        for kind in UnitKind::ALL.into_iter().filter(|k| data::is_aircraft(*k)) {
            let (s, _, plane) = arena(kind);
            let range = data::unit_stats(kind).range;
            let at = |d: i32| {
                hex(3, 4)
                    .ring(d)
                    .into_iter()
                    .find(|h| s.map.index(*h).is_some())
            };
            let inside = at(range).expect("a tile at range");
            assert!(
                errors(&s, vec![recon(plane, inside)]).is_empty(),
                "{kind:?} at {range}"
            );
            let outside = at(range + 1).expect("a tile past range");
            let refused = errors(&s, vec![recon(plane, outside)]);
            assert_eq!(refused.len(), 1, "{kind:?}");
            assert!(refused[0].contains("out of range"), "{}", refused[0]);
        }
    }

    #[test]
    fn a_recon_that_cannot_work_says_why() {
        let (mut s, _, plane) = arena(UnitKind::Bomber);
        let scout = unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        let rival = {
            let b = *s.cities.keys().nth(1).unwrap();
            air::base_new(&mut s, b, UnitKind::Bomber);
            s.aircraft.last().unwrap().id
        };
        let say = |os: Vec<Order>| errors(&s, os).join("; ");
        assert!(say(vec![recon(scout, hex(6, 4))]).contains("is not an aircraft"));
        assert!(say(vec![recon(rival, hex(6, 4))]).contains("not yours"));
        assert!(say(vec![recon(plane, hex(3, 4))]).contains("pick a tile away from"));
        assert!(say(vec![recon(plane, hex(-40, 4))]).contains("off the map"));
        assert!(say(vec![recon(plane, hex(11, 9))]).contains("out of range"));
    }

    #[test]
    fn a_recon_uses_the_aircrafts_one_order_so_it_cannot_also_strike() {
        let (mut s, _, plane) = arena(UnitKind::Bomber);
        let victim = unit(&mut s, 1, UnitKind::Spearman, hex(6, 4));
        let strike = Order::AirStrike {
            plane,
            target: hex(6, 4),
        };
        let both = vec![recon(plane, hex(9, 4)), strike.clone()];
        let refused = errors(&s, both.clone());
        assert_eq!(refused.len(), 1);
        assert!(refused[0].contains("already has an order this turn"));
        let (next, _) = resolve_tactical_turn(&s, &orders(0, both));
        assert_eq!(next.units[&victim].hp, 100, "the strike did not happen");
        assert_eq!(next.air_recons.len(), 1, "the recon did");
        let (struck, events) =
            resolve_tactical_turn(&s, &orders(0, vec![strike, recon(plane, hex(9, 4))]));
        assert!(struck.units.get(&victim).is_none_or(|u| u.hp < 100));
        assert!(
            struck.air_recons.is_empty(),
            "the strike came first, so no recon"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. }))
        );
    }

    #[test]
    fn a_fighter_that_recons_still_guards() {
        let (mut s, _, bomber) = arena(UnitKind::Bomber);
        let b = *s.cities.keys().nth(1).unwrap();
        unit(&mut s, 0, UnitKind::Scout, hex(5, 4));
        air::base_new(&mut s, b, UnitKind::Fighter);
        let fighter = s.aircraft.last().unwrap().id;
        let mut both = BTreeMap::new();
        both.insert(
            P0,
            vec![Order::AirStrike {
                plane: bomber,
                target: hex(7, 4),
            }],
        );
        both.insert(P1, vec![recon(fighter, hex(10, 4))]);
        let (next, _) = resolve_tactical_turn(&s, &both);
        assert_eq!(
            next.cities[&b].hp, s.cities[&b].hp,
            "the Fighter intercepted"
        );
        assert_eq!(next.air_recons.len(), 1);
    }

    #[test]
    fn recon_orders_round_trip_through_json() {
        let order = recon(UnitId(9), hex(2, 3));
        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"type\":\"AirRecon\""), "{json}");
        assert_eq!(serde_json::from_str::<Order>(&json).unwrap(), order);
    }
}

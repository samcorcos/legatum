use crate::events::Event;
use crate::game::spawn_unit;
use crate::hex::Hex;
use crate::orders::Order;
use crate::rules::cities::tests::{P0, P1, at, found, test_game};
use crate::rules::resolve_turn;
use crate::state::GameState;
use crate::types::*;
use crate::{Faction, roads};
use std::collections::BTreeMap;

fn mongols_world() -> (GameState, CityId, Vec<CityId>) {
    let mut s = test_game(&[(Faction::Mongols, 0), (Faction::India, 1)]);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::Peace;
    }
    let hall = found(&mut s, P0, at(3, 3));
    let others = vec![
        found(&mut s, P0, at(14, 3)),
        found(&mut s, P1, at(16, 15)),
        found(&mut s, P1, at(4, 16)),
    ];
    (s, hall, others)
}

fn turn(s: &GameState) -> (GameState, Vec<Event>) {
    resolve_turn(s, &BTreeMap::new())
}

fn give_hall(s: &mut GameState, city: CityId) {
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::CourierHall);
    s.player_mut(P0).hall_city = Some(city);
}

#[test]
fn no_roads_without_a_hall() {
    let (s, _, _) = mongols_world();
    let (next, events) = turn(&s);
    assert!(!next.map.has_roads());
    assert!(!events.iter().any(|e| matches!(e, Event::RoadsLaid { .. })));
}

#[test]
fn a_new_hall_links_every_city_of_every_empire_once() {
    let (mut s, hall, others) = mongols_world();
    give_hall(&mut s, hall);
    let (next, events) = turn(&s);
    let laid: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::RoadsLaid {
                owner,
                city,
                linked,
                ..
            } => Some((*owner, *city, *linked)),
            _ => None,
        })
        .collect();
    assert_eq!(laid, vec![(P0, hall, 3)]);
    assert_eq!(next.map.road_hub, Some(next.cities[&hall].pos));
    for c in others.iter().chain([&hall]) {
        assert!(next.map.is_road(next.cities[c].pos), "{c:?}");
    }
    // Announced once: the next turn lays nothing and says nothing.
    let (again, events) = turn(&next);
    assert_eq!(again.map.tiles, next.map.tiles);
    assert!(!events.iter().any(|e| matches!(e, Event::RoadsLaid { .. })));
}

#[test]
fn cities_founded_later_join_and_roads_outlive_the_hall() {
    let (mut s, hall, _) = mongols_world();
    give_hall(&mut s, hall);
    let (mut s, _) = turn(&s);
    let late = found(&mut s, P1, at(18, 8));
    assert!(!s.map.is_road(s.cities[&late].pos));
    let (s, events) = turn(&s);
    assert!(s.map.is_road(s.cities[&late].pos));
    assert!(!events.iter().any(|e| matches!(e, Event::RoadsLaid { .. })));
    // Destroy the Hall: every road stays, and later cities are not joined.
    let mut gone = s.clone();
    gone.cities.get_mut(&hall).unwrap().buildings.clear();
    let tiles = gone.map.tiles.clone();
    let after = found(&mut gone, P1, at(10, 17));
    let (gone, _) = turn(&gone);
    assert!(!gone.map.is_road(gone.cities[&after].pos));
    for (a, b) in tiles.iter().zip(&gone.map.tiles) {
        assert_eq!(a.road, b.road);
    }
}

/// Soldier (2 moves) at the west end of a row of Hills, bound for the east end.
fn hills_row(road: bool, from_road: bool) -> (GameState, UnitId, Hex) {
    let mut s = test_game(&[(Faction::Switzerland, 0), (Faction::India, 1)]);
    let (a, b) = (at(2, 5), at(5, 5));
    // A corridor: Mountains above and below.
    for c in 0..9 {
        for r in [4, 6] {
            s.map.get_mut(at(c, r)).unwrap().terrain = Terrain::Mountains;
        }
    }
    for c in 2..=6 {
        let h = at(c, 5);
        let t = s.map.get_mut(h).unwrap();
        t.terrain = Terrain::Hills;
        if road && (from_road || c > 2) {
            t.road = roads::NODE;
        }
    }
    let u = spawn_unit(&mut s, P0, UnitKind::Soldier, a);
    spawn_unit(&mut s, P1, UnitKind::Settler, at(18, 18));
    spawn_unit(&mut s, P0, UnitKind::Settler, at(1, 18));
    (s, u, b)
}

fn walk(s: &GameState, u: UnitId, to: Hex) -> Hex {
    let orders = BTreeMap::from([(P0, vec![Order::MoveUnit { unit: u, to }])]);
    let (next, _) = resolve_turn(s, &orders);
    next.units[&u].pos
}

#[test]
fn road_tiles_cost_one_and_a_unit_starting_on_a_road_has_one_more_move() {
    // Off road, Hills cost 2: a 2-move Soldier takes one step.
    let (s, u, to) = hills_row(false, false);
    assert_eq!(walk(&s, u, to), at(3, 5));
    // On road tiles it starts on, +1 move and 1 per tile: three steps arrive.
    let (s, u, to) = hills_row(true, true);
    assert_eq!(walk(&s, u, to), at(5, 5));
    // Starting off the road gets no bonus, but entering road tiles still costs 1 each.
    let (s, u, to) = hills_row(true, false);
    assert_eq!(walk(&s, u, to), at(4, 5));
}

#[test]
fn roads_do_not_open_a_foreign_border() {
    let mut s = test_game(&[(Faction::Switzerland, 0), (Faction::India, 1)]);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = crate::diplomacy::RelationState::Peace;
    }
    for c in 2..=10 {
        s.map.get_mut(at(c, 5)).unwrap().road = roads::NODE;
        if c >= 6 {
            s.map.get_mut(at(c, 5)).unwrap().owner = Some(P1);
        }
    }
    s.map.roaded = true;
    let u = spawn_unit(&mut s, P0, UnitKind::Soldier, at(2, 5));
    spawn_unit(&mut s, P1, UnitKind::Settler, at(18, 18));
    spawn_unit(&mut s, P0, UnitKind::Settler, at(1, 18));
    assert!(!s.passage_allowed(P0, UnitKind::Soldier, Some(P1)));
    let end = walk(&s, u, at(10, 5));
    assert!(end.to_offset().0 <= 5, "stopped at the border, got {end:?}");
}

fn give_roadworks(s: &mut GameState, city: CityId) {
    s.cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::Roadworks);
}

#[test]
fn roadworks_link_every_city_within_eight_hexes_of_any_empire_and_only_those() {
    let mut s = test_game(&[(Faction::Sparta, 0), (Faction::India, 1)]);
    let home = found(&mut s, P0, at(3, 3));
    let centre = s.cities[&home].pos;
    // A city exactly eight hexes away is in range; one nine away is not.
    let at_distance = |d: i32, skip: &[Hex]| {
        (0..20)
            .flat_map(|r| (0..20).map(move |c| at(c, r)))
            .find(|h| h.distance(centre) == d && skip.iter().all(|k| k.distance(*h) >= 4))
            .unwrap()
    };
    let near_at = at_distance(5, &[]);
    let edge_at = at_distance(8, &[near_at]);
    let far_at = at_distance(9, &[near_at, edge_at]);
    let near = found(&mut s, P1, near_at);
    let edge = found(&mut s, P0, edge_at);
    let far = found(&mut s, P1, far_at);
    give_roadworks(&mut s, home);
    let (s, _) = turn(&s);
    assert!(s.map.is_road(s.cities[&home].pos));
    assert!(s.map.is_road(s.cities[&near].pos));
    assert!(s.map.is_road(s.cities[&edge].pos));
    assert!(!s.map.is_road(s.cities[&far].pos), "out of range");
    // The Hall's one-time announcement is not made for a player's roads.
    assert!(s.map.road_hub.is_none());
    // A city founded later within range joins the next turn; a far one never does.
    let mut s = s;
    let taken = [near_at, edge_at, far_at, centre];
    let late_at = (0..20)
        .flat_map(|r| (0..20).map(move |c| at(c, r)))
        .find(|h| h.distance(centre) == 6 && taken.iter().all(|k| k.distance(*h) >= 4))
        .unwrap();
    let late_far_at = (0..20)
        .flat_map(|r| (0..20).map(move |c| at(c, r)))
        .find(|h| {
            h.distance(centre) == 11 && taken.iter().chain([&late_at]).all(|k| k.distance(*h) >= 4)
        })
        .unwrap();
    let late = found(&mut s, P1, late_at);
    let late_far = found(&mut s, P1, late_far_at);
    let (s, events) = turn(&s);
    assert!(s.map.is_road(s.cities[&late].pos));
    assert!(!s.map.is_road(s.cities[&late_far].pos));
    assert!(!events.iter().any(|e| matches!(e, Event::RoadsLaid { .. })));
}

#[test]
fn roadworks_yield_one_gold_flat_whatever_the_connections() {
    let mut s = test_game(&[(Faction::Sparta, 0), (Faction::India, 1)]);
    let home = found(&mut s, P0, at(3, 3));
    let base = crate::economy::city_yields(&s, &s.cities[&home]).total.gold;
    give_roadworks(&mut s, home);
    let alone = crate::economy::city_yields(&s, &s.cities[&home]).total.gold;
    for (c, r) in [(9, 3), (3, 10), (11, 9)] {
        found(&mut s, P1, at(c, r));
    }
    let linked = crate::economy::city_yields(&s, &s.cities[&home]).total.gold;
    assert_eq!((alone - base, linked - base), (1, 1));
}

#[test]
fn hall_roads_and_roadworks_share_one_road_network() {
    let (mut s, hall, others) = mongols_world();
    // A Roadworks city far from the Hall's roads joins its own neighbours first; the Hall then
    // joins everything, the Roadworks cluster included.
    give_roadworks(&mut s, others[1]);
    let (mut s, _) = turn(&s);
    assert!(!s.map.has_roads() || s.map.road_hub.is_none());
    give_hall(&mut s, hall);
    let (s, events) = turn(&s);
    assert!(events.iter().any(|e| matches!(e, Event::RoadsLaid { .. })));
    let net = roads::network_steps_for_tests(&s.map, s.cities[&hall].pos);
    for c in s.cities.values() {
        assert!(
            net[s.map.index(c.pos).unwrap()] >= 0,
            "{:?} is on the Hall's network",
            c.id
        );
    }
}

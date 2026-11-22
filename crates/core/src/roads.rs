//! Roads: the Yam Relay Station's open road.
//!
//! When the first Yam Relay Station completes, every city on the map (all empires, enemy land
//! included) is joined to the Station's city by roads, and a city founded later is joined the next
//! turn while a Station stands (`extend`). Roads are land, not a city's property: a captured or
//! razed Station, or a razed city, takes none of them away, and nothing ever removes one.
//!
//! Movement: entering a road tile costs 1 move whatever the terrain (`Map::move_cost`), and a
//! unit that starts its turn on a road has +1 move (`rules::units::reset_turn`). Roads grant no
//! passage: Peace borders still apply (`GameState::passage_allowed`).
//!
//! The layout is deterministic integer arithmetic. Cities join one at a time, nearest to the
//! Station first. Each runs a Dijkstra search over land and ends at the road tile (a terminal)
//! where its new road plus the trip along the existing network to the Station is cheapest, so
//! branches share trunks and every road leads towards the Station (a shortest-path tree grown
//! outward). Costs, in 64ths of a move: a new road tile costs 64 x its terrain cost (water and
//! mountains are impassable, so roads use passes), a step along existing road costs 48 (joining
//! a trunk beats building a parallel one), and a tile off the straight line from the city to
//! the Station costs a little more (about 1 per 0.4 tile of distance), so a road runs as a
//! straight line rather than a zig-zag or an L.

use crate::hex::Hex;
use crate::state::Map;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// `Tile::road` bit marking a road tile that has no link yet (the Station's own tile at first).
pub const NODE: u8 = 1 << 6;

const NEW_ROAD: i32 = 64;
const ALONG_ROAD: i32 = 48;
/// Straight-line penalty: 1 per this many plane units off the line (a tile is about 3464 wide).
const DEVIATION_UNIT: i64 = 1500;

/// Flat plane coordinates, doubled and scaled so hex centres are integers (pointy-top odd-r:
/// x = sqrt(3) (q + r/2), y = 1.5 r, both times 2000).
fn plane(h: Hex) -> (i64, i64) {
    (1732 * (2 * h.q as i64 + h.r as i64), 3000 * h.r as i64)
}

fn dir_of(from: Hex, to: Hex) -> usize {
    from.neighbors()
        .iter()
        .position(|&n| n == to)
        .expect("adjacent hexes")
}

/// Links `a` and `b` (adjacent) both ways, marking both as road.
fn link(map: &mut Map, a: Hex, b: Hex) {
    map.roaded = true;
    let d = dir_of(a, b);
    map.get_mut(a).expect("on map").road |= NODE | (1 << d);
    map.get_mut(b).expect("on map").road |= NODE | (1 << ((d + 3) % 6));
}

/// Roads reach `to` from `from` as one link of the network (both tiles are road).
pub fn linked(map: &Map, from: Hex, to: Hex) -> bool {
    from.distance(to) == 1
        && map
            .get(from)
            .is_some_and(|t| t.road & (1 << dir_of(from, to)) != 0)
}

/// Steps along the existing network from every road tile to `hub` (BFS over links); -1 where
/// a road tile is not joined to it.
fn network_steps(map: &Map, hub: Hex) -> Vec<i32> {
    let mut steps = vec![-1; map.len()];
    let Some(start) = map.index(hub) else {
        return steps;
    };
    steps[start] = 0;
    let mut queue = std::collections::VecDeque::from([hub]);
    while let Some(h) = queue.pop_front() {
        let here = steps[map.index(h).expect("on map")];
        let mask = map.get(h).expect("on map").road;
        for (d, nb) in h.neighbors().into_iter().enumerate() {
            if mask & (1 << d) == 0 {
                continue;
            }
            if let Some(i) = map.index(nb)
                && steps[i] < 0
            {
                steps[i] = here + 1;
                queue.push_back(nb);
            }
        }
    }
    steps
}

#[cfg(test)]
pub fn network_steps_for_tests(map: &Map, hub: Hex) -> Vec<i32> {
    network_steps(map, hub)
}

/// Lays the road from `city` to the network rooted at `hub`: the cheapest new road plus network
/// trip (see the module doc). Returns false when no road tile is reachable over land.
fn connect_one(map: &mut Map, hub: Hex, city: Hex, net: &[i32]) -> bool {
    let n = map.len();
    let (ax, ay) = plane(city);
    let (hx, hy) = plane(hub);
    let (dx, dy) = (hx - ax, hy - ay);
    let len = ((dx * dx + dy * dy) as u64).isqrt() as i64;
    let deviation = |h: Hex| -> i32 {
        if len == 0 {
            return 0;
        }
        let (x, y) = plane(h);
        let cross = (dx * (y - ay) - dy * (x - ax)).abs();
        (cross / len / DEVIATION_UNIT) as i32
    };

    let start = map.index(city).expect("city on map");
    let mut g = vec![i32::MAX; n];
    let mut prev = vec![usize::MAX; n];
    let mut open = BinaryHeap::new();
    g[start] = 0;
    open.push(Reverse((0, start)));
    let mut best: Option<(i32, usize)> = None;
    while let Some(Reverse((cost, i))) = open.pop() {
        if cost > g[i] {
            continue;
        }
        if best.is_some_and(|(total, _)| cost >= total) {
            break;
        }
        let here = map.hex_at(i);
        if i != start && map.tiles[i].is_road() && net[i] >= 0 {
            // A terminal: finish along the existing network. (A road tile of another network
            // is walked over for free below, and the walk links it in.)
            let total = cost + ALONG_ROAD * net[i];
            if best.is_none_or(|(b, bi)| (total, i) < (b, bi)) {
                best = Some((total, i));
            }
            continue;
        }
        for nb in here.neighbors() {
            let Some(j) = map.index(nb) else { continue };
            let Some(terrain) = map.tiles[j].terrain.move_cost() else {
                continue;
            };
            let step = if map.tiles[j].is_road() {
                0
            } else {
                NEW_ROAD * terrain + deviation(nb)
            };
            let next = cost + step;
            if next < g[j] {
                g[j] = next;
                prev[j] = i;
                open.push(Reverse((next, j)));
            }
        }
    }
    let Some((_, mut i)) = best else {
        return false;
    };
    while prev[i] != usize::MAX {
        let p = prev[i];
        link(map, map.hex_at(p), map.hex_at(i));
        i = p;
    }
    true
}

/// Joins every city of `cities` that is not yet on a road to the network rooted at `hub`
/// (marked as road itself), nearest to the hub first. Returns how many it joined.
pub fn connect(map: &mut Map, hub: Hex, cities: &[Hex]) -> usize {
    if let Some(t) = map.get_mut(hub) {
        t.road |= NODE;
        map.roaded = true;
    }
    let mut pending: Vec<Hex> = cities.to_vec();
    pending.sort_by_key(|&c| (c.distance(hub), c));
    let mut joined = 0;
    for city in pending {
        let net = network_steps(map, hub);
        // Already on the hub's network (a road laid for an earlier city may pass through it).
        if map.index(city).is_some_and(|i| net[i] >= 0) {
            continue;
        }
        if connect_one(map, hub, city, &net) {
            joined += 1;
        }
    }
    joined
}

/// Every standing Roadworks city links itself to every city within `ROADWORKS_RANGE` hexes, any
/// empire's, that its road network does not reach yet. Run every turn, so a city founded or
/// captured later within range is linked automatically. Roads outlive their Roadworks.
pub fn extend_roadworks(state: &mut crate::state::GameState) {
    let works: Vec<(Hex, i32)> = state
        .cities
        .values()
        .filter(|c| crate::data::has_building(&c.buildings, crate::types::BuildingKind::Roadworks))
        .map(|c| {
            (
                c.pos,
                if state
                    .player(c.owner)
                    .techs
                    .contains(&crate::types::TechId::Railways)
                {
                    crate::data::RAILWAYS_ROADWORKS_RANGE
                } else {
                    crate::data::ROADWORKS_RANGE
                },
            )
        })
        .collect();
    if works.is_empty() {
        return;
    }
    let cities: Vec<Hex> = state.cities.values().map(|c| c.pos).collect();
    for (hub, range) in works {
        let near: Vec<Hex> = cities
            .iter()
            .copied()
            .filter(|c| *c != hub && c.distance(hub) <= range)
            .collect();
        connect(&mut state.map, hub, &near);
    }
}

/// Every turn, while a Yam Relay Station stands: the first one's completion lays the roads (rooted at
/// its city, announced with `Event::RoadsLaid`), and any city without a road (founded since, or
/// the city of a later Station) is joined to the network. With no Station standing nothing is laid, and
/// nothing is ever removed.
pub fn extend(state: &mut crate::state::GameState, events: &mut Vec<crate::events::Event>) {
    let Some((owner, city, pos)) = state
        .players
        .iter()
        .filter_map(|p| crate::goals::hall_city(state, p.id).map(|c| (p.id, c.id, c.pos)))
        .next()
    else {
        return;
    };
    let first = state.map.road_hub.is_none();
    let hub = state.map.road_hub.unwrap_or(pos);
    let cities: Vec<Hex> = state.cities.values().map(|c| c.pos).collect();
    if !first && state.map.roaded {
        let net = network_steps(&state.map, hub);
        if cities
            .iter()
            .all(|&c| state.map.index(c).is_some_and(|i| net[i] >= 0))
        {
            return;
        }
    }
    state.map.road_hub = Some(hub);
    let linked = connect(&mut state.map, hub, &cities);
    if first {
        events.push(crate::events::Event::RoadsLaid {
            owner,
            city,
            at: pos,
            linked: linked as u32,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Terrain;

    fn road_tiles(map: &Map) -> Vec<Hex> {
        map.hexes().filter(|&h| map.is_road(h)).collect()
    }

    #[test]
    fn two_cities_get_one_straight_road() {
        let mut map = Map::new(30, 12, Terrain::Grassland);
        let hub = Hex::from_offset(3, 6);
        let far = Hex::from_offset(25, 6);
        assert_eq!(connect(&mut map, hub, &[hub, far]), 1);
        // A road along a row: exactly the 23 tiles, hub to far, no wobble.
        let tiles = road_tiles(&map);
        assert_eq!(tiles.len(), 23);
        assert!(tiles.iter().all(|h| h.to_offset().1 == 6));
        assert!(map.has_roads());
    }

    #[test]
    fn diagonal_road_is_a_straight_hex_line() {
        let mut map = Map::new(40, 40, Terrain::Grassland);
        let hub = Hex::from_offset(5, 5);
        let far = Hex::from_offset(30, 25);
        connect(&mut map, hub, &[far]);
        let tiles = road_tiles(&map);
        // A shortest path (no step wasted).
        assert_eq!(tiles.len() as i32, hub.distance(far) + 1);
        // Within a tile of the straight line between the two centres.
        let (ax, ay) = plane(hub);
        let (bx, by) = plane(far);
        let len = (((bx - ax).pow(2) + (by - ay).pow(2)) as u64).isqrt() as i64;
        for h in tiles {
            let (x, y) = plane(h);
            let d = ((bx - ax) * (y - ay) - (by - ay) * (x - ax)).abs() / len;
            assert!(d < 3464, "{h:?} is {d} off the line");
        }
    }

    #[test]
    fn roads_go_around_mountains_and_water() {
        let mut map = Map::new(30, 20, Terrain::Grassland);
        for r in 2..18 {
            let h = Hex::from_offset(15, r);
            map.get_mut(h).unwrap().terrain = if r % 2 == 0 {
                Terrain::Mountains
            } else {
                Terrain::Ocean
            };
        }
        let hub = Hex::from_offset(3, 10);
        let far = Hex::from_offset(27, 10);
        assert_eq!(connect(&mut map, hub, &[far]), 1);
        for h in road_tiles(&map) {
            assert!(map.get(h).unwrap().terrain.move_cost().is_some());
        }
    }

    #[test]
    fn a_pass_is_used_instead_of_a_detour() {
        // A ridge of Mountains with one Hills gap.
        let mut map = Map::new(30, 21, Terrain::Grassland);
        for r in 0..21 {
            let h = Hex::from_offset(15, r);
            map.get_mut(h).unwrap().terrain = if r == 10 {
                Terrain::Hills
            } else {
                Terrain::Mountains
            };
        }
        let hub = Hex::from_offset(3, 10);
        let far = Hex::from_offset(27, 10);
        connect(&mut map, hub, &[far]);
        assert!(map.is_road(Hex::from_offset(15, 10)));
    }

    #[test]
    fn later_cities_join_a_trunk_instead_of_a_parallel_road() {
        let mut map = Map::new(40, 14, Terrain::Grassland);
        let hub = Hex::from_offset(2, 7);
        let first = Hex::from_offset(30, 7);
        let near = Hex::from_offset(31, 8);
        connect(&mut map, hub, &[first]);
        let before = road_tiles(&map).len();
        assert_eq!(connect(&mut map, hub, &[near]), 1);
        assert!(road_tiles(&map).len() <= before + 2, "a short spur only");
    }

    #[test]
    fn layout_is_deterministic_and_idempotent() {
        let make = || {
            let mut map = Map::new(36, 24, Terrain::Grassland);
            for (c, r) in [(10, 5), (11, 6), (20, 14), (21, 14)] {
                map.get_mut(Hex::from_offset(c, r)).unwrap().terrain = Terrain::Hills;
            }
            let cities: Vec<_> = [(4, 4), (30, 5), (31, 20), (6, 21), (18, 12)]
                .iter()
                .map(|&(c, r)| Hex::from_offset(c, r))
                .collect();
            connect(&mut map, cities[4], &cities);
            (map, cities)
        };
        let (a, cities) = make();
        let (mut b, _) = make();
        assert_eq!(a.tiles, b.tiles);
        let before = b.tiles.clone();
        assert_eq!(connect(&mut b, cities[4], &cities), 0);
        assert_eq!(b.tiles, before);
        for c in &cities {
            assert!(a.is_road(*c));
        }
    }

    #[test]
    fn an_unreachable_city_is_left_alone() {
        let mut map = Map::new(20, 10, Terrain::Grassland);
        for r in 0..10 {
            map.get_mut(Hex::from_offset(10, r)).unwrap().terrain = Terrain::Ocean;
        }
        let hub = Hex::from_offset(3, 5);
        let island = Hex::from_offset(15, 5);
        assert_eq!(connect(&mut map, hub, &[island]), 0);
        assert!(!map.is_road(island));
    }
}

#[cfg(test)]
#[path = "roads_tests.rs"]
mod state_tests;

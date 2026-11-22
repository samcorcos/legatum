//! A* pathfinding on the hex map. Deterministic: ties break on hex order.

use crate::hex::Hex;
use crate::state::Map;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Cheapest path from `from` to `to`, excluding `from`. `cost(h)` is the cost to enter `h`
/// (`None` = impassable). Returns `None` if unreachable.
pub fn find_path(
    map: &Map,
    from: Hex,
    to: Hex,
    cost: impl Fn(Hex) -> Option<i32>,
) -> Option<Vec<Hex>> {
    if from != to {
        map.index(from)?;
        map.index(to)?;
        cost(to)?;
    }
    find_path_by_step(map, from, to, |_, h| cost(h))
}

/// Edge-aware pathfinding also supports mandatory progress toward a closed-border exit.
pub fn find_path_by_step(
    map: &Map,
    from: Hex,
    to: Hex,
    cost: impl Fn(Hex, Hex) -> Option<i32>,
) -> Option<Vec<Hex>> {
    if from == to {
        return Some(Vec::new());
    }
    let goal = map.index(to)?;
    let start = map.index(from)?;
    // No search can reach a goal with no legal incoming edge. Unlike cost(from,to),
    // this preserves egress paths whose permissions depend on the preceding tile.
    if !to
        .neighbors()
        .into_iter()
        .any(|h| map.contains(h) && cost(h, to).is_some())
    {
        return None;
    }

    if map.has_roads() {
        // Equal-cost routes prefer road tiles, so units use the roads laid beside a plain
        // route. Costs scale by `ROAD_SCALE` and a road tile takes one off, which never outweighs
        // a whole step on a path shorter than the scale; the heuristic scales with them.
        let scaled =
            |a: Hex, b: Hex| cost(a, b).map(|c| c * ROAD_SCALE - i32::from(map.is_road(b)));
        return run_search(map, from, to, start, goal, &scaled, ROAD_SCALE - 1);
    }
    run_search(map, from, to, start, goal, &cost, 1)
}

/// Cost scale of a map with roads (`find_path_by_step`): more than any sensible path length.
const ROAD_SCALE: i32 = 256;

fn run_search(
    map: &Map,
    from: Hex,
    to: Hex,
    start: usize,
    goal: usize,
    cost: &impl Fn(Hex, Hex) -> Option<i32>,
    hscale: i32,
) -> Option<Vec<Hex>> {
    let fast = SEARCH.with(|scratch| {
        let mut scratch = scratch.try_borrow_mut().ok()?;
        scratch.search(map, (from, to), (start, goal), cost, hscale)
    });
    let path = match fast {
        Some(path) => path,
        None => search_generic(map, from, to, start, goal, cost, hscale),
    };
    debug_assert_eq!(
        path,
        search_generic(map, from, to, start, goal, cost, hscale),
        "packed A* from {from:?} to {to:?}"
    );
    path
}

/// The reference A*: the order `Scratch::search` reproduces exactly.
fn search_generic(
    map: &Map,
    from: Hex,
    to: Hex,
    start: usize,
    goal: usize,
    cost: &impl Fn(Hex, Hex) -> Option<i32>,
    hscale: i32,
) -> Option<Vec<Hex>> {
    let n = map.len();
    let mut g = vec![i32::MAX; n];
    let mut prev = vec![usize::MAX; n];
    let mut open = BinaryHeap::new();
    g[start] = 0;
    open.push(Reverse((from.distance(to) * hscale, 0, from)));

    while let Some(Reverse((_, cur_g, cur))) = open.pop() {
        let ci = map.index(cur).expect("on map");
        if ci == goal {
            let mut path = vec![to];
            let mut i = goal;
            while prev[i] != start {
                i = prev[i];
                path.push(map.hex_at(i));
            }
            path.reverse();
            return Some(path);
        }
        if cur_g > g[ci] {
            continue;
        }
        for nb in cur.neighbors() {
            let Some(ni) = map.index(nb) else { continue };
            let Some(step) = cost(cur, nb) else { continue };
            let ng = cur_g + step;
            if ng < g[ni] {
                g[ni] = ng;
                prev[ni] = ci;
                open.push(Reverse((ng + nb.distance(to) * hscale, ng, nb)));
            }
        }
    }
    None
}

thread_local! {
    /// This thread's A* buffers, reused across searches (`Scratch::search`).
    static SEARCH: std::cell::RefCell<Scratch> = std::cell::RefCell::new(Scratch::default());
}

/// Bits of a packed open-list key: f and g (`KEY_BITS` each) above a tile's rank in hex order.
const KEY_BITS: u32 = 20;
const RANK_BITS: u32 = 24;

/// A* buffers for one map size: per-tile costs and parents stamped by search (so a search
/// touches only the tiles it reaches), a neighbour table, and an open list of packed keys.
#[derive(Default)]
struct Scratch {
    width: i32,
    height: i32,
    /// Neighbour tile index per tile and direction (`Hex::neighbors` order); u32::MAX off map.
    neighbors: Vec<[u32; 6]>,
    hexes: Vec<Hex>,
    /// Each tile's position in `Hex` order (q, then r), and the tile at each rank.
    rank: Vec<u32>,
    tile_at_rank: Vec<u32>,
    g: Vec<i32>,
    prev: Vec<u32>,
    stamp: Vec<u32>,
    epoch: u32,
    open: BinaryHeap<Reverse<u64>>,
}

impl Scratch {
    fn prepare(&mut self, map: &Map) -> bool {
        let n = map.len();
        if n != (map.width.max(0) as usize) * (map.height.max(0) as usize) || n >= 1 << RANK_BITS {
            return false;
        }
        if self.width != map.width || self.height != map.height || self.hexes.len() != n {
            self.width = map.width;
            self.height = map.height;
            self.hexes = (0..n).map(|i| map.hex_at(i)).collect();
            self.neighbors = self
                .hexes
                .iter()
                .map(|h| {
                    let nb = h.neighbors();
                    std::array::from_fn(|d| map.index(nb[d]).map_or(u32::MAX, |i| i as u32))
                })
                .collect();
            let mut order: Vec<u32> = (0..n as u32).collect();
            order.sort_by_key(|&i| self.hexes[i as usize]);
            self.rank = vec![0; n];
            for (r, &i) in order.iter().enumerate() {
                self.rank[i as usize] = r as u32;
            }
            self.tile_at_rank = order;
            self.g = vec![0; n];
            self.prev = vec![0; n];
            self.stamp = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.stamp.iter_mut().for_each(|s| *s = 0);
            self.epoch = 1;
        }
        self.open.clear();
        true
    }

    /// `search_generic`, with every open-list entry packed into one integer that sorts the
    /// same way ((f, g, hex) ascending). None when the map or a cost doesn't fit the packing;
    /// the caller then runs `search_generic`.
    fn search(
        &mut self,
        map: &Map,
        (from, to): (Hex, Hex),
        (start, goal): (usize, usize),
        cost: &impl Fn(Hex, Hex) -> Option<i32>,
        hscale: i32,
    ) -> Option<Option<Vec<Hex>>> {
        if !self.prepare(map) {
            return None;
        }
        const LIMIT: i32 = 1 << KEY_BITS;
        let pack = |f: i32, g: i32, rank: u32| -> u64 {
            ((f as u64) << (KEY_BITS + RANK_BITS)) | ((g as u64) << RANK_BITS) | u64::from(rank)
        };
        let epoch = self.epoch;
        let h0 = from.distance(to).saturating_mul(hscale);
        if h0 >= LIMIT {
            return None;
        }
        self.stamp[start] = epoch;
        self.g[start] = 0;
        self.open.push(Reverse(pack(h0, 0, self.rank[start])));
        while let Some(Reverse(key)) = self.open.pop() {
            let ci = self.tile_at_rank[(key & ((1 << RANK_BITS) - 1)) as usize] as usize;
            let cur_g = ((key >> RANK_BITS) & ((1 << KEY_BITS) - 1)) as i32;
            if ci == goal {
                let mut path = vec![to];
                let mut i = goal;
                while self.prev[i] as usize != start {
                    i = self.prev[i] as usize;
                    path.push(self.hexes[i]);
                }
                path.reverse();
                return Some(Some(path));
            }
            // Stamped this search: an entry is only pushed after its tile's g is set.
            if cur_g > self.g[ci] {
                continue;
            }
            let cur = self.hexes[ci];
            for d in 0..6 {
                let ni = self.neighbors[ci][d];
                if ni == u32::MAX {
                    continue;
                }
                let ni = ni as usize;
                let nb = self.hexes[ni];
                let Some(step) = cost(cur, nb) else { continue };
                let ng = cur_g + step;
                if self.stamp[ni] != epoch || ng < self.g[ni] {
                    let f = ng.saturating_add(nb.distance(to).saturating_mul(hscale));
                    if !(0..LIMIT).contains(&ng) || !(0..LIMIT).contains(&f) {
                        return None;
                    }
                    self.stamp[ni] = epoch;
                    self.g[ni] = ng;
                    self.prev[ni] = ci as u32;
                    self.open.push(Reverse(pack(f, ng, self.rank[ni])));
                }
            }
        }
        Some(None)
    }
}

/// Minimum number of passable steps to any legally enterable tile, ignoring units.
/// Using steps (not movement points) makes every egress order visibly approach an exit.
pub fn exit_distances(map: &Map, open: impl Fn(Hex) -> bool) -> Vec<i32> {
    exit_distances_around(map, open, |_| false)
}

/// `exit_distances` where a closed tile that `city` calls a city centre is a wall, not a
/// stepping stone: nobody walks into a foreign city at Peace, so no egress route runs through
/// one (`Map` has no city centres: callers pass `GameState::city_at` or the view's known
/// cities). Open city tiles are exits already, whatever `city` says.
pub fn exit_distances_around(
    map: &Map,
    open: impl Fn(Hex) -> bool,
    city: impl Fn(Hex) -> bool,
) -> Vec<i32> {
    use std::collections::VecDeque;
    let mut distances = vec![i32::MAX; map.len()];
    let mut queue = VecDeque::new();
    for (i, distance) in distances.iter_mut().enumerate() {
        let h = map.hex_at(i);
        if map.move_cost(h).is_some() && open(h) {
            *distance = 0;
            queue.push_back(h);
        }
    }
    while let Some(h) = queue.pop_front() {
        let next = distances[map.index(h).expect("on map")] + 1;
        for nb in h.neighbors() {
            if let Some(i) = map.index(nb)
                && map.move_cost(nb).is_some()
                && !city(nb)
                && next < distances[i]
            {
                distances[i] = next;
                queue.push_back(nb);
            }
        }
    }
    distances
}

pub fn border_step(
    map: &Map,
    from: Hex,
    to: Hex,
    open: impl Fn(Hex) -> bool,
    exits: Option<&[i32]>,
) -> bool {
    if open(to) {
        return true;
    }
    if open(from) {
        return false;
    }
    exits.is_some_and(|d| match (map.index(from), map.index(to)) {
        (Some(a), Some(b)) => d[b] < d[a],
        _ => false,
    })
}

/// The landing of a hop across friends: when `path` starts on a `crossable` tile (a friend of
/// the mover's class that stays put), the index of the first tile after the run of crossable
/// tiles. None when the path does not start on one, or the run reaches the goal (a unit never
/// ends a move on a friend, so there is nothing to land on).
pub fn crossing_end(path: &[Hex], crossable: impl Fn(Hex) -> bool) -> Option<usize> {
    let run = path.iter().take_while(|h| crossable(**h)).count();
    (run > 0 && run < path.len()).then_some(run)
}

/// The hop a unit makes when its route starts across friends that stay put: it passes them in
/// one move and lands on the first tile beyond, paying every tile it crossed. Returns the
/// landing and the total cost, or None when the route does not start on a crossable tile (or
/// ends on one: the goal itself is never crossed).
pub fn hop(
    path: &[Hex],
    from: Hex,
    cost: impl Fn(Hex, Hex) -> Option<i32>,
    crossable: impl Fn(Hex) -> bool,
) -> Option<(Hex, i32)> {
    let end = crossing_end(path, crossable)?;
    let mut prev = from;
    let mut total = 0;
    for &h in &path[..=end] {
        total += cost(prev, h)?;
        prev = h;
    }
    Some((path[end], total))
}

/// Turns needed to walk `path` under the engine's step rule (`rules::movement::can_step`):
/// a step needs moves ≥ its cost, except that a unit at full moves may always take one step.
/// A hop across crossable friends (`hop`) is one atomic move that needs moves ≥ its whole cost:
/// short of that the unit waits for the next turn. Mirrored by `app/src/path.ts:pathTurns`.
pub fn path_turns(
    path: &[Hex],
    mut step_cost: impl FnMut(Hex) -> i32,
    (from, moves_left, max_moves): (Hex, i32, i32),
    on_road: impl Fn(Hex) -> bool,
    crossable: impl Fn(Hex) -> bool,
) -> i32 {
    let mut turns = 1;
    let mut moves = moves_left;
    let mut at = from;
    let mut i = 0;
    while i < path.len() {
        let end = crossing_end(&path[i..], &crossable).map_or(i, |k| i + k);
        let cost: i32 = path[i..=end].iter().map(|&h| step_cost(h)).sum();
        let affordable = if end == i {
            crate::rules::movement::can_step(moves, max_moves, cost)
        } else {
            moves >= cost
        };
        if !affordable {
            turns += 1;
            // A unit that starts its turn on a road has +1 move (`rules::units::reset_turn`).
            moves = max_moves + i32::from(on_road(at));
        }
        moves = (moves - cost).max(0);
        at = path[end];
        i = end + 1;
    }
    turns
}

/// How friendly occupancy shapes a land route. The one costing used by movement resolution
/// (`rules::movement::next_step`), the AI's movement preflight (`legatum_ai::movement`) and,
/// mirrored, the client preview and mock (`app/src/path.ts:routeAroundFriends`).
///
/// - `blocked` tiles are never entered on the way (the goal itself is always allowed): hostile
///   or neutral occupants. Pass the answer for the goal too: a crossing
///   cannot land on a blocked goal.
/// - `passable` tiles hold a friend of the same class that stays put this turn (parked, out of
///   moves, or attacking from where it stands). A unit passes through them but never ends its
///   move on one (Civ 6, user 2026-10-21): `hop` crosses the whole run in one move and lands
///   on the first tile beyond, paying for every tile. A crossing is only planned where that
///   landing exists (not a blocked goal, not the end of the route) and the unit can pay for it
///   in one turn (`crossing_faults`); otherwise the first tile of the run is a wall for that
///   search.
/// - `waiting` is the part of `passable` that clears on a later turn: a friend out of moves
///   for now that still has a destination. When its crossing cannot be made (a friend that sits
///   on hills, a landing dearer than the unit's moves) the unit waits behind it, and walks
///   round only if the way round takes fewer turns (playtest 8.6, turn 77: a follower turned
///   back for a friend that moved on next turn).
/// - `risky` tiles hold a friend of the same class with a move out this turn. Following it
///   works when it leaves first (friendly chains move together), but its step may fail, so a
///   route that avoids every risky tile wins whenever it arrives in as few turns.
/// - `reserved` tiles are claimed by a lower-id friend in this round. Entering one waits a
///   movement round under the ordinary claim rules, without spending moves. Prefer following
///   it when that still arrives in no more turns than a route avoiding the reservation.
/// - The route through friends that stay wins unless walking round them arrives in fewer
///   turns (a hop that does not fit the moves left waits for the next turn: `path_turns`).
/// - With no route around the blocked tiles at all, the plain terrain route is returned: the
///   unit waits in a choke behind whatever blocks it rather than losing its destination.
///
/// Deterministic: every search breaks ties on hex order.
#[allow(clippy::too_many_arguments)]
pub fn route_around_friends(
    map: &Map,
    from: Hex,
    to: Hex,
    cost: impl Fn(Hex, Hex) -> Option<i32>,
    blocked: impl Fn(Hex) -> bool,
    passable: impl Fn(Hex) -> bool,
    waiting: impl Fn(Hex) -> bool,
    risky: impl Fn(Hex) -> bool,
    (moves_left, max_moves): (i32, i32),
    reserved: impl Fn(Hex) -> bool,
) -> Option<Vec<Hex>> {
    // Only fall back to plain terrain if occupancy actually pruned a legal edge. Terrain or
    // border disconnection cannot be repaired by the same second search.
    let pruned = std::cell::Cell::new(false);
    // Which tiles a search treats as walls: the blocked ones; friends that stay unless
    // `cross`; the first tile of any crossing that cannot be made (unless that friend is
    // waited for); risky ones when asked. Also whether the route waits behind a friend.
    let around =
        |cross: bool, avoid_risky: bool, avoid_reserved: bool| -> Option<(Vec<Hex>, bool)> {
            let mut walls: Vec<Hex> = Vec::new();
            // Every retry walls a new tile, so this ends; the cap keeps a crowded route cheap.
            for _ in 0..6 {
                let path = find_path_by_step(map, from, to, |a, b| {
                    cost(a, b).filter(|_| {
                        let avoid = b != to
                            && (blocked(b)
                                || (!cross && passable(b))
                                || walls.contains(&b)
                                || (avoid_risky && risky(b))
                                || (avoid_reserved && reserved(b)));
                        if avoid {
                            pruned.set(true);
                        }
                        !avoid
                    })
                })?;
                let faults = crossing_faults(&path, from, &cost, &blocked, &passable, max_moves);
                match faults.iter().find(|f| !waiting(**f)) {
                    None => return Some((path, !faults.is_empty())),
                    Some(&first) => walls.push(first),
                }
            }
            None
        };
    let walk = |p: &[Hex], waits: bool| {
        let mut prev = from;
        path_turns(
            p,
            |h| {
                let c = cost(prev, h).unwrap_or(1);
                prev = h;
                c
            },
            (from, moves_left, max_moves),
            |h| map.is_road(h),
            // Waiting behind a friend is plain steps: it clears before the unit gets there.
            |h| !waits && passable(h),
        )
    };
    let mut cross = true;
    let mut found = around(true, false, false);
    if found.is_none() {
        found = around(false, false, false);
        cross = false;
    }
    // Passing through friends that stay usually wins; walking round them only when that
    // arrives in fewer turns (a hop that does not fit the moves left waits a turn), or, for a
    // friend waited for, in no more turns than waiting behind it.
    if cross
        && let Some((through, waits)) = &found
        && interior(through).iter().any(|h| passable(*h))
        && let Some((round, _)) = around(false, false, false)
    {
        let (round_turns, through_turns) = (walk(&round, false), walk(through, *waits));
        if if *waits {
            round_turns <= through_turns
        } else {
            round_turns < through_turns
        } {
            found = Some((round, false));
            cross = false;
        }
    }
    let Some((mut path, mut waits)) = found else {
        return pruned
            .get()
            .then(|| find_path_by_step(map, from, to, &cost))
            .flatten();
    };
    // A claim is temporary. The first claim can be deferred, but it consumes no movement
    // budget and the next round can follow the friend through the pass. The comparison is
    // in turns; a one-round pause only matters if it makes arrival a later turn.
    if interior(&path).iter().any(|h| reserved(*h))
        && let Some((free, free_waits)) = around(cross, false, true)
        && walk(&free, free_waits) < walk(&path, waits)
    {
        path = free;
        waits = free_waits;
    }
    if !interior(&path).iter().any(|h| risky(*h)) {
        return Some(path);
    }
    match around(cross, true, false) {
        Some((free, free_waits)) if walk(&free, free_waits) <= walk(&path, waits) => Some(free),
        _ => Some(path),
    }
}

/// A route without its goal.
fn interior(path: &[Hex]) -> &[Hex] {
    &path[..path.len().saturating_sub(1)]
}

/// The first tile of every crossing that `path` cannot make: a run of friends with no landing
/// (it reaches the goal), a landing on a blocked goal (an attack cannot start from two tiles
/// away), or a hop that costs more than a unit's full moves. A friend that is the goal on its
/// own is no crossing: the unit waits beside it.
pub fn crossing_faults(
    path: &[Hex],
    from: Hex,
    cost: impl Fn(Hex, Hex) -> Option<i32>,
    blocked: impl Fn(Hex) -> bool,
    crossable: impl Fn(Hex) -> bool,
    max_moves: i32,
) -> Vec<Hex> {
    let mut faults = Vec::new();
    let Some(last) = path.len().checked_sub(1) else {
        return faults;
    };
    let mut i = 0;
    while i < path.len() {
        if !crossable(path[i]) || i == last {
            i += 1;
            continue;
        }
        let Some(end) = crossing_end(&path[i..], &crossable).map(|k| i + k) else {
            faults.push(path[i]);
            break;
        };
        let mut prev = if i == 0 { from } else { path[i - 1] };
        let mut total = 0;
        for &h in &path[i..=end] {
            total += cost(prev, h).unwrap_or(1);
            prev = h;
        }
        if (end == last && blocked(path[end])) || total > max_moves {
            faults.push(path[i]);
        }
        i = end + 1;
    }
    faults
}

/// Standard land-unit movement costs from terrain.
pub fn land_path(map: &Map, from: Hex, to: Hex) -> Option<Vec<Hex>> {
    find_path(map, from, to, |h| map.move_cost(h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Terrain;

    #[test]
    fn impossible_goal_returns_before_searching_the_map() {
        let map = Map::new(100, 100, Terrain::Grassland);
        let calls = std::cell::Cell::new(0);
        let goal = Hex::from_offset(80, 80);
        assert!(
            find_path(&map, Hex::from_offset(1, 1), goal, |h| {
                calls.set(calls.get() + 1);
                (h != goal).then_some(1)
            })
            .is_none()
        );
        assert_eq!(calls.get(), 1);
        calls.set(0);
        assert!(
            find_path_by_step(&map, Hex::from_offset(1, 1), goal, |_, h| {
                calls.set(calls.get() + 1);
                (h != goal).then_some(1)
            })
            .is_none()
        );
        assert_eq!(calls.get(), 6);
    }

    #[test]
    fn edge_dependent_goal_uses_its_neighbors_not_the_start() {
        let map = Map::new(6, 6, Terrain::Grassland);
        let from = Hex::from_offset(1, 1);
        let goal = Hex::from_offset(4, 1);
        let permitted = Hex::from_offset(3, 1);
        let path = find_path_by_step(&map, from, goal, |a, b| {
            (b != goal || a == permitted).then_some(1)
        })
        .unwrap();
        assert_eq!(path[path.len() - 2], permitted);
        assert_eq!(path.last(), Some(&goal));
    }

    #[test]
    fn friends_shape_routes_blocked_risky_and_choke() {
        let map = Map::new(10, 10, Terrain::Grassland);
        // Two tiles apart off-axis: two equally short routes, through either common neighbor.
        let to = Hex::from_offset(5, 5);
        let (m, f) = (to.neighbor(0), to.neighbor(1));
        let from = Hex::new(m.q + f.q - to.q, m.r + f.r - to.r);
        let terrain = |_: Hex, b: Hex| map.move_cost(b);
        let direct = land_path(&map, from, to).unwrap();
        assert_eq!(direct.len(), 2);
        let mid = direct[0];
        let route = |blocked: &dyn Fn(Hex) -> bool,
                     passable: &dyn Fn(Hex) -> bool,
                     risky: &dyn Fn(Hex) -> bool| {
            route_around_friends(
                &map,
                from,
                to,
                terrain,
                blocked,
                passable,
                |_| false,
                risky,
                (2, 2),
                |_| false,
            )
        };
        // A wall (an enemy): routed around, with an equally short free tile.
        let around = route(&|h| h == mid, &|_| false, &|_| false).unwrap();
        assert_eq!(around.len(), 2);
        assert_ne!(around[0], mid);
        // A friend that stays is crossed, not circled: the straight route wins the tie.
        let through = route(&|_| false, &|h| h == mid, &|_| false).unwrap();
        assert_eq!(through, direct);
        // A friend that leaves: the free route arrives in as few turns, so it wins.
        let free = route(&|_| false, &|_| false, &|h| h == mid).unwrap();
        assert_eq!(free, around);
        // The goal itself is never avoided.
        let onto = route_around_friends(
            &map,
            from,
            mid,
            terrain,
            |h| h == mid,
            |h| h == mid,
            |_| false,
            |h| h == mid,
            (2, 2),
            |_| false,
        );
        assert_eq!(onto, Some(vec![mid]));
        // A one-tile corridor keeps the plain route through a wall (it waits there).
        let mut corridor = Map::new(10, 10, Terrain::Mountains);
        for col in 0..10 {
            corridor.get_mut(Hex::from_offset(col, 4)).unwrap().terrain = Terrain::Grassland;
        }
        let cost = |_: Hex, b: Hex| corridor.move_cost(b);
        let (start, through, end) = (
            Hex::from_offset(2, 4),
            Hex::from_offset(3, 4),
            Hex::from_offset(4, 4),
        );
        let choke = route_around_friends(
            &corridor,
            start,
            end,
            cost,
            |h| h == through,
            |_| false,
            |_| false,
            |_| false,
            (2, 2),
            |_| false,
        );
        assert_eq!(choke, Some(vec![through, end]));
        // A risky route is kept when the only free one takes an extra turn.
        let mut detour = Map::new(10, 10, Terrain::Grassland);
        for nb in from.neighbors() {
            if nb != mid {
                detour.get_mut(nb).unwrap().terrain = Terrain::Forest;
            }
        }
        let cost = |_: Hex, b: Hex| detour.move_cost(b);
        let kept = route_around_friends(
            &detour,
            from,
            to,
            cost,
            |_| false,
            |_| false,
            |_| false,
            |h| h == mid,
            (2, 2),
            |_| false,
        );
        assert_eq!(kept, Some(vec![mid, to]));
        assert_eq!(
            path_turns(
                &[mid, to, Hex::from_offset(5, 4)],
                |_| 1,
                (from, 2, 2),
                |_| false,
                |_| false
            ),
            2
        );
        assert_eq!(
            path_turns(&[mid], |_| 3, (from, 2, 2), |_| false, |_| false),
            1,
            "first step is always allowed"
        );
    }

    /// A one-tile pass along row 4 with a way round through row 8, a friend in the pass.
    fn pass_with_way_round() -> (Map, Hex, Hex, Hex) {
        let mut map = Map::new(12, 10, Terrain::Mountains);
        for col in 1..=10 {
            for row in [4, 8] {
                map.get_mut(Hex::from_offset(col, row)).unwrap().terrain = Terrain::Grassland;
            }
        }
        for row in 5..=7 {
            for col in [1, 10] {
                map.get_mut(Hex::from_offset(col, row)).unwrap().terrain = Terrain::Grassland;
            }
        }
        (
            map,
            Hex::from_offset(4, 4),
            Hex::from_offset(5, 4),
            Hex::from_offset(10, 4),
        )
    }

    #[test]
    fn a_friend_that_stays_in_a_pass_is_crossed_not_circled() {
        let (map, from, friend, to) = pass_with_way_round();
        let cost = |_: Hex, b: Hex| map.move_cost(b);
        let route = route_around_friends(
            &map,
            from,
            to,
            cost,
            |_| false,
            |h| h == friend,
            |_| false,
            |_| false,
            (2, 2),
            |_| false,
        )
        .unwrap();
        assert_eq!(route, land_path(&map, from, to).unwrap());
        assert_eq!(route[0], friend);
        // As a wall (an enemy) it is walked round the long way.
        let round = route_around_friends(
            &map,
            from,
            to,
            cost,
            |h| h == friend,
            |_| false,
            |_| false,
            |_| false,
            (2, 2),
            |_| false,
        )
        .unwrap();
        assert!(round.len() > 12, "{round:?}");
        // The hop over it: landed beyond, both tiles paid.
        assert_eq!(
            hop(&route, from, cost, |h| h == friend),
            Some((route[1], 2))
        );
        assert_eq!(hop(&route[1..], friend, cost, |h| h == friend), None);
    }

    #[test]
    fn a_round_reservation_is_followed_through_a_pass() {
        let (map, from, reserved, to) = pass_with_way_round();
        let cost = |_: Hex, b: Hex| map.move_cost(b);
        let route = route_around_friends(
            &map,
            from,
            to,
            cost,
            |_| false,
            |_| false,
            |_| false,
            |_| false,
            (2, 2),
            |h| h == reserved,
        )
        .unwrap();
        assert_eq!(route, land_path(&map, from, to).unwrap());
        assert_eq!(route[0], reserved);
        let round = route_around_friends(
            &map,
            from,
            to,
            cost,
            |h| h == reserved,
            |_| false,
            |_| false,
            |_| false,
            (2, 2),
            |_| false,
        )
        .unwrap();
        assert!(round.len() > 12);
    }

    #[test]
    fn a_crossing_that_cannot_land_or_be_paid_for_is_walled() {
        let (map, from, friend, to) = pass_with_way_round();
        let cost = |_: Hex, b: Hex| map.move_cost(b);
        // The goal itself is a friend that stays: it cannot be ended on, so the route still
        // comes up beside it (a run that reaches the goal is no crossing).
        let beside = Hex::from_offset(9, 4);
        assert_eq!(
            crossing_faults(&[beside, to], beside, cost, |_| false, |_| true, 2),
            vec![beside]
        );
        assert!(
            crossing_faults(&[to], beside, cost, |_| false, |h| h == to, 2).is_empty(),
            "the goal alone: the unit waits beside it"
        );
        // A hostile goal beyond the friend: an attack cannot start two tiles away.
        let two = [friend, Hex::from_offset(6, 4)];
        let at_goal = |h: Hex| h == two[1];
        assert_eq!(
            crossing_faults(&two, from, cost, at_goal, |h| h == friend, 2),
            vec![friend]
        );
        assert!(crossing_faults(&two, from, cost, |_| false, |h| h == friend, 2).is_empty());
        // A hop dearer than a unit's full moves can never be paid: walled, so the route goes
        // round, or, for a friend that moves on later, waits behind it when that is no slower.
        let mut hills = map.clone();
        hills.get_mut(two[1]).unwrap().terrain = Terrain::Hills;
        let hill = |_: Hex, b: Hex| hills.move_cost(b);
        assert_eq!(
            crossing_faults(&two, from, hill, |_| false, |h| h == friend, 2),
            vec![friend]
        );
        assert!(crossing_faults(&two, from, hill, |_| false, |h| h == friend, 3).is_empty());
        let route = |waiting: bool| {
            route_around_friends(
                &hills,
                from,
                to,
                hill,
                |_| false,
                |h| h == friend,
                |h| waiting && h == friend,
                |_| false,
                (2, 2),
                |_| false,
            )
            .unwrap()
        };
        assert!(route(false).len() > 12, "walks round: {:?}", route(false));
        // Playtest 8.6: the way round takes more turns than waiting behind a friend that
        // moves on later, so the unit waits.
        assert_eq!(route(true), land_path(&hills, from, to).unwrap());
    }

    #[test]
    fn a_hop_that_does_not_fit_the_moves_left_waits_a_turn() {
        let (map, from, friend, to) = pass_with_way_round();
        let route = land_path(&map, from, to).unwrap();
        let cost = |h: Hex| map.move_cost(h).unwrap();
        let turns = |moves: i32| {
            path_turns(
                &route[..2],
                cost,
                (from, moves, 2),
                |_| false,
                |h| h == friend,
            )
        };
        assert_eq!(turns(2), 1, "both tiles paid in one move");
        assert_eq!(turns(1), 2, "one move left: the hop waits for next turn");
    }

    #[test]
    fn straight_path_on_open_ground() {
        let map = Map::new(10, 10, Terrain::Grassland);
        let from = Hex::from_offset(1, 1);
        let to = Hex::from_offset(5, 1);
        let p = land_path(&map, from, to).unwrap();
        assert_eq!(p.len(), 4);
        assert_eq!(*p.last().unwrap(), to);
    }

    #[test]
    fn routes_around_mountains_and_fails_when_walled() {
        let mut map = Map::new(10, 10, Terrain::Grassland);
        let from = Hex::from_offset(1, 5);
        let to = Hex::from_offset(8, 5);
        for row in 0..9 {
            map.get_mut(Hex::from_offset(4, row)).unwrap().terrain = Terrain::Mountains;
        }
        assert!(land_path(&map, from, to).unwrap().len() > 7);
        map.get_mut(Hex::from_offset(4, 9)).unwrap().terrain = Terrain::Mountains;
        assert!(land_path(&map, from, to).is_none());
    }
}

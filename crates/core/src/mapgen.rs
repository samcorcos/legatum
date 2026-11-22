//! Procedural planet generation (story S2).
//!
//! One continent per planet. Elevation is integer fractal value noise minus an elliptical edge
//! falloff; the sea level is the percentile that makes `LAND_PERCENT` of the tiles land. Mountain
//! ridges come from "ridged" noise and are placed one tile at a time, skipping any tile whose
//! mountain would split a passable land region. Climate (tundra, desert, forest, grassland,
//! plains, hills) comes from latitude plus rank-normalized noise layers. Starts are searched
//! in quality bands after scattering ruins, with optional nearby teammates. A candidate
//! that fails validation is discarded and the next attempt uses fresh derived streams.
//!
//! Everything is integer math and every random draw comes from
//! `Rng::derive(seed, &[purpose::MAPGEN, attempt, layer, ...])`.

use crate::hex::Hex;
use crate::rng::{Rng, purpose};
use crate::state::{GameSetup, Map};
use crate::types::Terrain;

/// Share of tiles that start as land, out of 100 (spec: about 40-50%).
const LAND_PERCENT: usize = 45;
/// Accepted land share after generation, out of 100.
pub const MIN_LAND_PERCENT: usize = 40;
pub const MAX_LAND_PERCENT: usize = 50;
/// The largest land component must hold at least this share of all land, out of 100.
pub const MAIN_LAND_PERCENT: usize = 85;
/// Largest separate island allowed (islands never hold starts).
pub const MAX_ISLAND: usize = 15;
/// Tiles this close to the map edge are always water.
const EDGE_MARGIN: i32 = 2;
/// Mountains as a share of land, out of 100 (upper bound; connectivity can veto some).
const MOUNTAIN_PERCENT: usize = 6;
/// Only tiles this far up the ridge noise (0..=1000) can become mountains, so they form lines.
const RIDGE_MIN: i64 = 700;
pub const START_MIN_DISTANCE: i32 = 7;
pub const START_MIN_WORKABLE: usize = 12;
pub const START_RADIUS: i32 = 2;
/// A reliable early production floor: two nearby Forest/Hills tiles or equivalent yield.
pub const START_MIN_PRODUCTION_UNITS: usize = 2;
/// Starts closer than this to each other are acceptable but not preferred.
const START_PREFERRED_DISTANCE: i32 = 10;
/// At most four radius-2 plains tiles' weighted food/production between the best and worst
/// start. Ruins and rival breathing room must fit in this same budget, not a separate allowance.
pub const START_MAX_QUALITY_SPREAD: i32 = 24;
/// Two-thirds of an all-plains radius-3 site (150). Do not equalize everyone onto barren land.
pub const START_MIN_QUALITY: i32 = 100;
/// Every empire's founding territory (start tile plus ring 1, the seven tiles a new city owns
/// and works) totals exactly this much Food and Production (designer, 2026-11-07). Ring 1
/// terrain is sculpted after the starts are picked; strategic and bonus resources stay out of
/// ring 1 so the count never changes when they are revealed.
pub const START_FOOD: i32 = 8;
pub const START_PRODUCTION: i32 = 8;
/// Water tiles in ring 1 are never changed, so a start with more of them cannot reach the target.
const START_MAX_RING_WATER: usize = 2;
const RIVAL_QUALITY_RANGE: i32 = 2 * (START_PREFERRED_DISTANCE - START_MIN_DISTANCE);
const START_SEARCH_BUDGET: usize = 1000;
pub const LAND_TILES_PER_RUIN: usize = 60;
const RUIN_MIN_SPACING: i32 = 4;
const MAX_ATTEMPTS: u64 = 1000;

// Independent random streams inside one attempt.
const LAYER_ELEVATION: u64 = 1;
const LAYER_RIDGE: u64 = 2;
const LAYER_MOISTURE: u64 = 3;
const LAYER_TEMPERATURE: u64 = 4;
const LAYER_HILLS: u64 = 5;
const LAYER_STARTS: u64 = 6;
const LAYER_RUINS: u64 = 7;
const LAYER_BALANCE: u64 = 8;

/// Start placement options, kept separate from the serialized game setup. Ordinary generation
/// enables nearby teammates; callers doing map previews/experiments can explicitly opt out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartOptions {
    /// Teams form connected groups of ally links shorter than either end's nearest rival.
    /// Single-member teams impose no additional placement constraint.
    pub teams_start_near: bool,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            teams_start_near: true,
        }
    }
}

/// Generate the map and one start position per player (same order as `setup.players`).
pub fn generate(setup: &GameSetup) -> (Map, Vec<Hex>) {
    generate_counted(setup).0
}

/// Generate with an explicit teammate placement policy.
pub fn generate_with_options(setup: &GameSetup, options: StartOptions) -> (Map, Vec<Hex>) {
    generate_counted_with_options(setup, options).0
}

/// `generate`, also returning how many attempts were discarded before one validated.
pub fn generate_counted(setup: &GameSetup) -> ((Map, Vec<Hex>), u64) {
    generate_counted_with_options(setup, StartOptions::default())
}

/// `generate_with_options`, also returning the number of discarded attempts.
pub fn generate_counted_with_options(
    setup: &GameSetup,
    options: StartOptions,
) -> ((Map, Vec<Hex>), u64) {
    try_generate_counted_with_options(setup, options).unwrap_or_else(|e| panic!("{e}"))
}

/// Fallible entry point for untrusted setups. Generation itself is unchanged.
pub fn try_generate_counted_with_options(
    setup: &GameSetup,
    options: StartOptions,
) -> Result<((Map, Vec<Hex>), u64), String> {
    crate::validation::check_setup(setup)?;
    for attempt in 0..MAX_ATTEMPTS {
        if let Some(result) = try_generate(setup, options, attempt) {
            return Ok((result, attempt));
        }
    }
    Err(format!(
        "mapgen: no valid {}x{} map for {} players after {} attempts (seed {}); the map is too small for the start rules",
        setup.width,
        setup.height,
        setup.players.len(),
        MAX_ATTEMPTS,
        setup.seed
    ))
}

/// One candidate map, or `None` if it fails validation.
fn try_generate(setup: &GameSetup, options: StartOptions, attempt: u64) -> Option<(Map, Vec<Hex>)> {
    let (w, h) = (setup.width, setup.height);
    let n = (w * h) as usize;
    let seed = setup.seed;
    let noise = |layer, cell, octaves| fractal_noise(seed, attempt, layer, w, h, cell, octaves);

    // 1. Land and sea.
    let elevation = elevation(&noise(LAYER_ELEVATION, 12_000, 4), w, h);
    let mut order: Vec<usize> = (0..n).filter(|&i| !in_margin(i, w, h)).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(elevation[i]), i));
    let mut map = Map::new(w, h, Terrain::Ocean);
    for &i in order.iter().take(n * LAND_PERCENT / 100) {
        map.tiles[i].terrain = Terrain::Plains;
    }

    let (land_comp, land_sizes) = components(&map, |t| t.is_land());
    let main = largest(&land_sizes)?;
    let land: usize = land_sizes.iter().sum();
    if land_sizes[main] * 100 < land * MAIN_LAND_PERCENT {
        return None;
    }
    if land_sizes
        .iter()
        .enumerate()
        .any(|(c, &s)| c != main && s > MAX_ISLAND)
    {
        return None;
    }

    // 2. Mountain ridges that never split a passable region.
    let is_land: Vec<bool> = map.tiles.iter().map(|t| t.terrain.is_land()).collect();
    let ridge: Vec<i64> = rank_normalize(&noise(LAYER_RIDGE, 6_000, 3), &is_land)
        .into_iter()
        .map(|v| 1000 - 2 * (v - 500).abs())
        .collect();
    let mut peaks: Vec<usize> = (0..n)
        .filter(|&i| land_comp[i] == main as u32 && ridge[i] >= RIDGE_MIN)
        .collect();
    peaks.sort_by_key(|&i| (std::cmp::Reverse(ridge[i] * 2 + elevation[i]), i));
    let mut mountains = 0;
    let mut regions = components(&map, passable).1.len();
    for i in peaks {
        if mountains >= land * MOUNTAIN_PERCENT / 100 {
            break;
        }
        map.tiles[i].terrain = Terrain::Mountains;
        let now = components(&map, passable).1.len();
        if now > regions {
            map.tiles[i].terrain = Terrain::Plains;
        } else {
            regions = now;
            mountains += 1;
        }
    }

    // 3. Climate.
    let moisture = rank_normalize(&noise(LAYER_MOISTURE, 6_000, 3), &is_land);
    let temperature = rank_normalize(&noise(LAYER_TEMPERATURE, 8_000, 2), &is_land);
    let hills = rank_normalize(&noise(LAYER_HILLS, 3_000, 2), &is_land);
    for i in 0..n {
        if map.tiles[i].terrain != Terrain::Plains {
            continue;
        }
        let hex = map.hex_at(i);
        let near_mountain = hex
            .neighbors()
            .iter()
            .any(|&nb| map.get(nb).map(|t| t.terrain) == Some(Terrain::Mountains));
        map.tiles[i].terrain = climate(
            latitude(hex.r, h),
            moisture[i],
            temperature[i],
            hills[i],
            near_mountain,
        );
    }

    // 4. Coast: water next to land.
    for i in 0..n {
        if !map.tiles[i].terrain.is_land()
            && map
                .hex_at(i)
                .neighbors()
                .iter()
                .any(|&nb| map.get(nb).is_some_and(|t| t.terrain.is_land()))
        {
            map.tiles[i].terrain = Terrain::Coast;
        }
    }

    // 5. Starts and ruins.
    let (pass_comp, pass_sizes) = components(&map, passable);
    let home = largest(&pass_sizes)? as u32;
    // Ruins precede the start search so their actual locations contribute to quality. Starts
    // must stay at least two tiles away; subsequent optimization cannot change ruin access.
    place_ruins(&mut map, &pass_comp, home, seed, attempt);
    let mut starts = pick_starts(&map, &pass_comp, home, setup, options, attempt)?;
    assign_india_starts(&map, &mut starts, setup, options);
    balance_founding_sites(&mut map, &starts, seed, attempt)?;
    // 6. Resources (G11): a separate seeded layer, placed after starts so it can be fair.
    crate::resources::placement::place(&mut map, &starts, setup, attempt)?;
    crate::wonders::place(&mut map, &starts, setup, attempt);

    validate(&map, &starts).ok()?;
    crate::resources::placement::validate_rivals(&map, &starts, setup).ok()?;
    Some((map, starts))
}

/// Terrain for a land tile from its climate inputs (all 0..=1000 except latitude 0..=100).
fn climate(lat: i64, moisture: i64, temperature: i64, hills: i64, near_mountain: bool) -> Terrain {
    // 100 at the equator, 0 at the poles, nudged by up to +-20 of noise.
    let temp = 100 - lat + (temperature - 500) / 25;
    if hills >= 900 || (near_mountain && hills >= 740) {
        return Terrain::Hills;
    }
    if temp < 27 {
        return if moisture >= 780 {
            Terrain::Forest
        } else {
            Terrain::Tundra
        };
    }
    // Dry mid-latitudes (the subtropical belt) grow deserts.
    let desert_line = if (28..=68).contains(&lat) { 230 } else { 50 };
    if moisture < desert_line && temp >= 35 {
        return Terrain::Desert;
    }
    if moisture >= 790 {
        Terrain::Forest
    } else if moisture >= 470 {
        Terrain::Grassland
    } else {
        Terrain::Plains
    }
}

/// 0 at the middle row, 100 at the top and bottom rows.
fn latitude(row: i32, height: i32) -> i64 {
    let span = (height - 1).max(1) as i64;
    (2 * row as i64 - span).abs() * 100 / span
}

fn passable(t: Terrain) -> bool {
    t.move_cost().is_some()
}

fn in_margin(i: usize, w: i32, h: i32) -> bool {
    let (col, row) = (i as i32 % w, i as i32 / w);
    col < EDGE_MARGIN || row < EDGE_MARGIN || col >= w - EDGE_MARGIN || row >= h - EDGE_MARGIN
}

/// Tile center in fixed-point units: hex width 1000, row spacing 866 (pointy-top).
fn tile_pos(col: i32, row: i32) -> (i64, i64) {
    (col as i64 * 1000 + (row as i64 & 1) * 500, row as i64 * 866)
}

/// Noise minus an elliptical falloff that pushes the map rim under water.
fn elevation(noise: &[i64], w: i32, h: i32) -> Vec<i64> {
    let (cx, cy) = tile_pos(w, h - 1);
    let (cx, cy) = ((cx / 2).max(1), (cy / 2).max(1));
    noise
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let (px, py) = tile_pos(i as i32 % w, i as i32 / w);
            let dx = (px - cx).abs() * 1000 / cx;
            let dy = (py - cy).abs() * 1000 / cy;
            // 0 at the center, 1000 on the ellipse touching the map edges.
            let r2 = (dx * dx + dy * dy) / 1000;
            v - r2 * r2 / 2200
        })
        .collect()
}

/// Integer fractal value noise, one value per tile (row-major), roughly 0..=1000.
fn fractal_noise(
    seed: u64,
    attempt: u64,
    layer: u64,
    w: i32,
    h: i32,
    base_cell: i64,
    octaves: u32,
) -> Vec<i64> {
    let n = (w * h) as usize;
    let mut sum = vec![0i64; n];
    let mut total_amp = 0;
    let (max_x, max_y) = tile_pos(w, h);
    for octave in 0..octaves {
        let cell = (base_cell >> octave).max(1);
        let amp = 1i64 << (octaves - 1 - octave);
        total_amp += amp;
        let nx = (max_x / cell + 2) as usize;
        let ny = (max_y / cell + 2) as usize;
        let mut rng = Rng::derive(seed, &[purpose::MAPGEN, attempt, layer, octave as u64]);
        let lattice: Vec<i64> = (0..nx * ny).map(|_| rng.range(0, 1000)).collect();
        for (i, s) in sum.iter_mut().enumerate() {
            let (px, py) = tile_pos(i as i32 % w, i as i32 / w);
            let (ix, iy) = ((px / cell) as usize, (py / cell) as usize);
            let fx = smooth((px % cell) * 1024 / cell);
            let fy = smooth((py % cell) * 1024 / cell);
            let at = |x: usize, y: usize| lattice[y * nx + x];
            let top = at(ix, iy) * (1024 - fx) + at(ix + 1, iy) * fx;
            let bottom = at(ix, iy + 1) * (1024 - fx) + at(ix + 1, iy + 1) * fx;
            *s += ((top * (1024 - fy) + bottom * fy) >> 20) * amp;
        }
    }
    sum.into_iter().map(|s| s / total_amp).collect()
}

/// Integer smoothstep on 0..=1024.
fn smooth(f: i64) -> i64 {
    f * f * (3 * 1024 - 2 * f) / (1024 * 1024)
}

/// Replace the values of `include`d tiles by their rank among them, scaled to 0..=1000 (ties by
/// index), so thresholds read as shares of those tiles. Other tiles get 0.
fn rank_normalize(values: &[i64], include: &[bool]) -> Vec<i64> {
    let mut order: Vec<usize> = (0..values.len()).filter(|&i| include[i]).collect();
    order.sort_by_key(|&i| (values[i], i));
    let top = (order.len().max(2) - 1) as i64;
    let mut out = vec![0; values.len()];
    for (rank, i) in order.into_iter().enumerate() {
        out[i] = rank as i64 * 1000 / top;
    }
    out
}

/// Connected components of tiles matching `pred`. Returns a component id per tile
/// (`u32::MAX` for non-matching tiles) and each component's size. Ids follow index order.
pub fn components(map: &Map, pred: impl Fn(Terrain) -> bool) -> (Vec<u32>, Vec<usize>) {
    let mut comp = vec![u32::MAX; map.len()];
    let mut sizes = Vec::new();
    let mut stack = Vec::new();
    for start in 0..map.len() {
        if comp[start] != u32::MAX || !pred(map.tiles[start].terrain) {
            continue;
        }
        let id = sizes.len() as u32;
        let mut size = 0;
        comp[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            size += 1;
            for nb in map.hex_at(i).neighbors() {
                if let Some(j) = map.index(nb)
                    && comp[j] == u32::MAX
                    && pred(map.tiles[j].terrain)
                {
                    comp[j] = id;
                    stack.push(j);
                }
            }
        }
        sizes.push(size);
    }
    (comp, sizes)
}

/// Index of the biggest component (lowest id on ties).
fn largest(sizes: &[usize]) -> Option<usize> {
    (0..sizes.len()).max_by_key(|&c| (sizes[c], std::cmp::Reverse(c)))
}

/// Workable land (land, not mountains) within the start radius, including the tile itself.
pub fn workable_tiles(map: &Map, at: Hex) -> usize {
    at.within(START_RADIUS)
        .into_iter()
        .filter(|&h| {
            map.get(h)
                .is_some_and(|t| t.terrain.is_land() && t.terrain != Terrain::Mountains)
        })
        .count()
}

fn ring_water(map: &Map, at: Hex) -> usize {
    at.neighbors()
        .iter()
        .filter(|&&nb| map.get(nb).is_some_and(|t| !t.terrain.is_land()))
        .count()
}

/// Food and Production of the founding territory (start tile plus ring 1), counting visible
/// bonus resources as a founded city would work them.
pub fn founding_yields(map: &Map, at: Hex) -> (i32, i32) {
    at.within(1)
        .into_iter()
        .filter_map(|hex| map.get(hex))
        .map(visible_yields)
        .fold((0, 0), |(f, p), y| (f + y.food, p + y.production))
}

/// Sculpt each start's ring 1 so its founding territory hits `START_FOOD`/`START_PRODUCTION`.
/// Water never changes; land becomes Grassland, Plains, Forest or Hills (the start tile only
/// Grassland or Plains). Among exact solutions the fewest changed tiles win, ties broken by a
/// seeded draw. `None` when a start cannot reach the target (the attempt is discarded).
fn balance_founding_sites(map: &mut Map, starts: &[Hex], seed: u64, attempt: u64) -> Option<()> {
    const RING: [Terrain; 4] = [
        Terrain::Grassland,
        Terrain::Plains,
        Terrain::Forest,
        Terrain::Hills,
    ];
    let mut rng = Rng::derive(seed, &[purpose::MAPGEN, attempt, LAYER_BALANCE]);
    for &start in starts {
        let (mut water_food, mut land) = (0, Vec::new());
        for nb in start.neighbors() {
            let Some(tile) = map.get(nb) else { continue };
            if tile.terrain.is_land() {
                land.push(nb);
            } else {
                water_food += tile.terrain.yields().food;
            }
        }
        let tiles: Vec<Hex> = std::iter::once(start).chain(land).collect();
        let combos = 2 * 4usize.pow(tiles.len() as u32 - 1);
        let mut best_cost = usize::MAX;
        let mut best = Vec::new();
        for code in 0..combos {
            let (mut rest, mut food, mut production, mut cost) = (code, water_food, 0, 0);
            let mut picked = Vec::with_capacity(tiles.len());
            for (i, &hex) in tiles.iter().enumerate() {
                let base = if i == 0 { 2 } else { 4 };
                let terrain = RING[rest % base];
                rest /= base;
                let y = terrain.yields();
                food += y.food;
                production += y.production;
                cost += usize::from(map.get(hex).unwrap().terrain != terrain);
                picked.push(terrain);
            }
            if food == START_FOOD && production == START_PRODUCTION && cost <= best_cost {
                if cost < best_cost {
                    best.clear();
                    best_cost = cost;
                }
                best.push(picked);
            }
        }
        if best.is_empty() {
            return None;
        }
        let choice = &best[rng.range(0, best.len() as i64 - 1) as usize];
        for (&hex, &terrain) in tiles.iter().zip(choice) {
            map.get_mut(hex).unwrap().terrain = terrain;
        }
    }
    Some(())
}

fn passable_neighbors(map: &Map, at: Hex) -> usize {
    at.neighbors()
        .iter()
        .filter(|&&nb| map.move_cost(nb).is_some())
        .count()
}

/// Production-yielding tiles, counting visible bonus resources when present. Strategic
/// deposits are hidden at game start. Resource placement follows the start search, so the
/// search uses terrain alone and cannot rely on a bonus that might never be placed.
pub fn production_tiles(map: &Map, at: Hex) -> usize {
    at.within(START_RADIUS)
        .into_iter()
        .filter(|&hex| {
            map.get(hex)
                .is_some_and(|tile| passable(tile.terrain) && visible_yields(tile).production >= 2)
        })
        .count()
}

/// Production above a Plains tile's baseline: two Forest/Hills tiles each contribute one,
/// while a sufficiently rich visible bonus on one tile can contribute the same two units.
fn early_production_units(map: &Map, at: Hex) -> i32 {
    at.within(START_RADIUS)
        .into_iter()
        .filter_map(|hex| map.get(hex).filter(|tile| passable(tile.terrain)))
        .map(|tile| (visible_yields(tile).production - 1).max(0))
        .sum()
}

fn visible_yields(tile: &crate::state::Tile) -> crate::types::Yields {
    tile.terrain.yields()
        + tile
            .resource
            .filter(|r| !crate::resources::is_strategic(*r))
            .map_or(crate::types::Yields::ZERO, |r| {
                crate::resources::stats(r).bonus
            })
}

/// Reassign only picked locations: an Indian seat gets the widest nearby land for a clustered network.
/// Keep the search's quality and ally-spacing guarantees after a seat swap.
fn assign_india_starts(map: &Map, starts: &mut [Hex], setup: &GameSetup, options: StartOptions) {
    let seats: Vec<_> = (0..starts.len()).collect();
    let ruins: Vec<_> = map.hexes().filter(|&h| map.get(h).unwrap().ruin).collect();
    let mut assigned = vec![false; starts.len()];
    for india in 0..starts.len() {
        if setup.players[india].faction != crate::types::Faction::India {
            continue;
        }
        let land: Vec<_> = starts
            .iter()
            .map(|start| {
                start
                    .within(8)
                    .into_iter()
                    .filter(|&h| map.move_cost(h).is_some())
                    .count()
            })
            .collect();
        let mut candidates: Vec<_> = (0..starts.len()).filter(|&seat| !assigned[seat]).collect();
        candidates.sort_by_key(|&seat| (std::cmp::Reverse(land[seat]), seat));
        for seat in candidates {
            starts.swap(india, seat);
            let qualities: Vec<_> = (0..starts.len())
                .map(|i| site_quality(map, starts[i], &ruins) + rival_quality(starts, setup, i))
                .collect();
            let band_ok = qualities
                .iter()
                .min()
                .is_some_and(|low| *low >= START_MIN_QUALITY)
                && qualities.iter().max().unwrap() - qualities.iter().min().unwrap()
                    <= START_MAX_QUALITY_SPREAD;
            if band_ok && layout_ok(starts.len(), |i| starts[i], &seats, setup, options) {
                break;
            }
            starts.swap(india, seat);
        }
        assigned[india] = true;
    }
}

fn start_terrain_ok(t: Terrain) -> bool {
    matches!(t, Terrain::Grassland | Terrain::Plains)
}

/// Faction-independent start value. Only passable land contributes: coastal food/gold cannot
/// replace room for land units and settlement. Radius 2 yields count three times, ring 3 once.
/// Hills and forest each earn up to six points, plus up to six for having both. Nearby ruins
/// earn up to twelve points (distance 2), and rival breathing room up to six (distance 10).
/// There is deliberately no fresh-water bonus: the map has no rivers or fresh-water rule.
pub fn start_quality(map: &Map, starts: &[Hex], setup: &GameSetup, seat: usize) -> i32 {
    let ruins: Vec<_> = map.hexes().filter(|&h| map.get(h).unwrap().ruin).collect();
    site_quality(map, starts[seat], &ruins) + rival_quality(starts, setup, seat)
}

fn site_quality(map: &Map, at: Hex, ruins: &[Hex]) -> i32 {
    // The founding territory (start tile plus ring 1) is sculpted to the same totals for every
    // start, so it contributes the same inner-ring weight to all of them.
    let (mut food, mut production) = (3 * START_FOOD, 3 * START_PRODUCTION);
    let (mut hills, mut forest) = (0, 0);
    for hex in at.within(3).into_iter().filter(|&h| at.distance(h) > 1) {
        let Some(tile) = map.get(hex).filter(|t| passable(t.terrain)) else {
            continue;
        };
        let inner = at.distance(hex) <= START_RADIUS;
        // Bonus resources are visible from the start; hidden strategic ones don't count.
        let yields = visible_yields(tile);
        food += yields.food * if inner { 3 } else { 1 };
        production += yields.production * if inner { 3 } else { 1 };
        if inner {
            hills += i32::from(tile.terrain == Terrain::Hills);
            forest += i32::from(tile.terrain == Terrain::Forest);
        }
    }
    // A one-yield site cannot substitute surplus Food for missing Production, or vice versa.
    let mut score = food + production - (food - production).abs() / 2;
    score += 2 * (hills.min(3) + forest.min(3) + hills.min(forest).min(3));
    if let Some(distance) = ruins.iter().map(|r| at.distance(*r)).min() {
        score += 2 * (8 - distance.clamp(2, 8));
    }
    score
}

fn rival_quality(starts: &[Hex], setup: &GameSetup, seat: usize) -> i32 {
    let distance = starts
        .iter()
        .enumerate()
        .filter(|&(i, _)| setup.players[i].team != setup.players[seat].team)
        .map(|(_, &hex)| starts[seat].distance(hex))
        .min()
        .unwrap_or(START_PREFERRED_DISTANCE);
    2 * (distance.clamp(START_MIN_DISTANCE, START_PREFERRED_DISTANCE) - START_MIN_DISTANCE)
}

#[derive(Clone, Copy)]
struct StartCandidate {
    hex: Hex,
    quality: i32,
    tie: u64,
}

/// Search narrow quality bands, highest first. Within a band, bounded backtracking preserves
/// space for later seats instead of permanently giving the first seat the best inland tile.
/// The static band leaves room for the entire rival-distance bonus, so every accepted layout
/// meets the final score bound, even when the seats belong to different-sized teams.
fn pick_starts(
    map: &Map,
    comp: &[u32],
    home: u32,
    setup: &GameSetup,
    options: StartOptions,
    attempt: u64,
) -> Option<Vec<Hex>> {
    let mut rng = Rng::derive(setup.seed, &[purpose::MAPGEN, attempt, LAYER_STARTS]);
    let ruins: Vec<_> = map.hexes().filter(|&h| map.get(h).unwrap().ruin).collect();
    let mut candidates = Vec::new();
    for (i, &c) in comp.iter().enumerate() {
        let hex = map.hex_at(i);
        if c != home
            || !start_terrain_ok(map.tiles[i].terrain)
            || workable_tiles(map, hex) < START_MIN_WORKABLE
            || passable_neighbors(map, hex) < 2
            || ring_water(map, hex) > START_MAX_RING_WATER
            || ruins.iter().any(|r| r.distance(hex) < 2)
        {
            continue;
        }
        let quality = site_quality(map, hex, &ruins);
        if quality >= START_MIN_QUALITY {
            candidates.push(StartCandidate {
                hex,
                quality,
                tie: rng.next_u64(),
            });
        }
    }

    // Assign entire teams in a seeded order, largest first. Members are also shuffled. This
    // avoids a fixed seat or team-ID advantage without destroying team geography afterwards.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for seat in 0..setup.players.len() {
        if let Some(group) = groups.iter_mut().find(|group| {
            options.teams_start_near && setup.players[group[0]].team == setup.players[seat].team
        }) {
            group.push(seat);
        } else {
            groups.push(vec![seat]);
        }
    }
    shuffle(&mut groups, &mut rng);
    groups.sort_by_key(|group| std::cmp::Reverse(group.len()));
    for group in &mut groups {
        shuffle(group, &mut rng);
    }
    let seats: Vec<_> = groups.into_iter().flatten().collect();
    let band = START_MAX_QUALITY_SPREAD - RIVAL_QUALITY_RANGE;
    let highest = candidates.iter().map(|c| c.quality).max()?;
    // Prefer a fresh map with the full floor over weakening a crowded map. Only after
    // repeated failed maps does the fallback relax this new rule; quality and spacing remain.
    let minimum_floor = if attempt < 64 {
        START_MIN_PRODUCTION_UNITS
    } else if attempt < 128 {
        1
    } else {
        0
    };
    for floor in (minimum_floor..=START_MIN_PRODUCTION_UNITS).rev() {
        for low in (START_MIN_QUALITY..=highest).rev().step_by(6) {
            let pool: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|c| {
                    (low..=low + band).contains(&c.quality)
                        && early_production_units(map, c.hex) >= floor as i32
                })
                .collect();
            let mut chosen = Vec::new();
            let mut budget = START_SEARCH_BUDGET;
            if search_starts(&pool, &seats, setup, options, &mut chosen, &mut budget) {
                improve_starts(&pool, &seats, setup, options, &mut chosen);
                let mut starts = vec![Hex::new(0, 0); seats.len()];
                for (&seat, candidate) in seats.iter().zip(chosen) {
                    starts[seat] = candidate.hex;
                }
                return Some(starts);
            }
        }
    }
    None
}

fn shuffle<T>(values: &mut [T], rng: &mut Rng) {
    for i in (1..values.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        values.swap(i, j);
    }
}

fn search_starts(
    pool: &[StartCandidate],
    seats: &[usize],
    setup: &GameSetup,
    options: StartOptions,
    chosen: &mut Vec<StartCandidate>,
    budget: &mut usize,
) -> bool {
    if chosen.len() == seats.len() {
        return true;
    }
    if *budget == 0 || pool.len() < seats.len() - chosen.len() {
        return false;
    }
    *budget -= 1;
    let seat = seats[chosen.len()];
    let mut next: Vec<_> = pool
        .iter()
        .copied()
        .filter(|candidate| placement_ok(chosen, *candidate, seats, setup, options))
        .collect();
    next.sort_by_key(|candidate| {
        let mut ally_gap = i32::MAX;
        let mut rival_gap = START_PREFERRED_DISTANCE;
        for (i, other) in chosen.iter().enumerate() {
            let gap = candidate.hex.distance(other.hex);
            if options.teams_start_near && setup.players[seats[i]].team == setup.players[seat].team
            {
                ally_gap = ally_gap.min(gap);
            } else {
                rival_gap = rival_gap.min(gap);
            }
        }
        (
            ally_gap,
            std::cmp::Reverse(rival_gap),
            std::cmp::Reverse(candidate.quality),
            candidate.tie,
            candidate.hex,
        )
    });
    for candidate in next {
        chosen.push(candidate);
        let available: Vec<_> = pool
            .iter()
            .copied()
            .filter(|other| other.hex.distance(candidate.hex) >= START_MIN_DISTANCE)
            .collect();
        if search_starts(&available, seats, setup, options, chosen, budget) {
            return true;
        }
        chosen.pop();
        if *budget == 0 {
            break;
        }
    }
    false
}

/// A team forms one connected group of short links: each link is shorter than either end's
/// nearest rival distance. Thus every member has a nearer teammate, and large teams cannot
/// split into unrelated pairs. Requiring *all* teammates to be nearer than *any* rival would
/// make crowded Small games (especially 7v1) needlessly impossible despite valid spacing.
fn placement_ok(
    chosen: &[StartCandidate],
    candidate: StartCandidate,
    seats: &[usize],
    setup: &GameSetup,
    options: StartOptions,
) -> bool {
    layout_ok(
        chosen.len() + 1,
        |i| {
            if i < chosen.len() {
                chosen[i].hex
            } else {
                candidate.hex
            }
        },
        seats,
        setup,
        options,
    )
}

fn layout_ok(
    count: usize,
    at: impl Fn(usize) -> Hex,
    seats: &[usize],
    setup: &GameSetup,
    options: StartOptions,
) -> bool {
    let team = |i: usize| setup.players[seats[i]].team;
    let mut rivals = vec![i32::MAX; count];
    for i in 0..count {
        for j in 0..i {
            let gap = at(i).distance(at(j));
            if gap < START_MIN_DISTANCE {
                return false;
            }
            if team(i) != team(j) {
                rivals[i] = rivals[i].min(gap);
                rivals[j] = rivals[j].min(gap);
            }
        }
    }
    if !options.teams_start_near {
        return true;
    }
    let mut visited = vec![false; count];
    for root in 0..count {
        if visited[root] {
            continue;
        }
        visited[root] = true;
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            for j in 0..count {
                if !visited[j]
                    && team(i) == team(j)
                    && at(i).distance(at(j)) < rivals[i].min(rivals[j])
                {
                    visited[j] = true;
                    stack.push(j);
                }
            }
        }
        if (0..count).any(|j| team(j) == team(root) && !visited[j]) {
            return false;
        }
    }
    true
}

/// Once a feasible band is found, minimize actual max-minus-min (including nearest rivals),
/// breaking equal spreads by the worst score, then total quality. A fixed number of passes
/// bounds work identically on native and WASM; no wall-clock or floating-point decisions.
fn improve_starts(
    pool: &[StartCandidate],
    seats: &[usize],
    setup: &GameSetup,
    options: StartOptions,
    chosen: &mut [StartCandidate],
) {
    let mut rank = layout_rank(chosen, seats, setup);
    for _ in 0..4 {
        let mut changed = false;
        for i in 0..chosen.len() {
            let mut best = chosen[i];
            for &candidate in pool {
                chosen[i] = candidate;
                let next_rank = layout_rank(chosen, seats, setup);
                if next_rank < rank
                    && layout_ok(chosen.len(), |j| chosen[j].hex, seats, setup, options)
                {
                    best = candidate;
                    rank = next_rank;
                    changed = true;
                }
            }
            chosen[i] = best;
        }
        if !changed {
            break;
        }
    }
}

fn layout_rank(
    chosen: &[StartCandidate],
    seats: &[usize],
    setup: &GameSetup,
) -> (i32, std::cmp::Reverse<i32>, std::cmp::Reverse<i32>) {
    let (mut low, mut high, mut sum) = (i32::MAX, i32::MIN, 0);
    for (i, candidate) in chosen.iter().enumerate() {
        let gap = chosen
            .iter()
            .enumerate()
            .filter(|&(j, _)| setup.players[seats[i]].team != setup.players[seats[j]].team)
            .map(|(_, other)| candidate.hex.distance(other.hex))
            .min()
            .unwrap_or(START_PREFERRED_DISTANCE);
        let quality = candidate.quality
            + 2 * (gap.clamp(START_MIN_DISTANCE, START_PREFERRED_DISTANCE) - START_MIN_DISTANCE);
        low = low.min(quality);
        high = high.max(quality);
        sum += quality;
    }
    (high - low, std::cmp::Reverse(low), std::cmp::Reverse(sum))
}

fn place_ruins(map: &mut Map, comp: &[u32], home: u32, seed: u64, attempt: u64) {
    let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
    let target = (land + LAND_TILES_PER_RUIN / 2) / LAND_TILES_PER_RUIN;
    let mut rng = Rng::derive(seed, &[purpose::MAPGEN, attempt, LAYER_RUINS]);
    let mut candidates: Vec<(u64, usize)> = (0..map.len())
        .filter(|&i| comp[i] == home)
        .map(|i| (rng.next_u64(), i))
        .collect();
    candidates.sort();
    let mut ruins: Vec<Hex> = Vec::new();
    for (_, i) in candidates {
        if ruins.len() >= target {
            break;
        }
        let hex = map.hex_at(i);
        if ruins.iter().all(|r| r.distance(hex) >= RUIN_MIN_SPACING) {
            map.tiles[i].ruin = true;
            ruins.push(hex);
        }
    }
}

/// Check every rule the spec places on a finished map. `Err` names the first broken rule.
pub fn validate(map: &Map, starts: &[Hex]) -> Result<(), String> {
    let n = map.len();
    let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
    if land * 100 < n * MIN_LAND_PERCENT || land * 100 > n * MAX_LAND_PERCENT {
        return Err(format!("land share {land}/{n} out of range"));
    }
    for i in 0..n {
        let (col, row) = (i as i32 % map.width, i as i32 / map.width);
        let edge = col == 0 || row == 0 || col == map.width - 1 || row == map.height - 1;
        if edge && map.tiles[i].terrain.is_land() {
            return Err(format!("land on the map edge at {:?}", map.hex_at(i)));
        }
        let t = map.tiles[i].terrain;
        let touches_land = map
            .hex_at(i)
            .neighbors()
            .iter()
            .any(|&nb| map.get(nb).is_some_and(|x| x.terrain.is_land()));
        if (t == Terrain::Coast) != (!t.is_land() && touches_land) {
            return Err(format!("coast mismatch at {:?}", map.hex_at(i)));
        }
    }
    let (land_comp, land_sizes) = components(map, |t| t.is_land());
    let main = largest(&land_sizes).ok_or("no land")?;
    if land_sizes[main] * 100 < land * MAIN_LAND_PERCENT {
        return Err("landmass is fragmented".into());
    }
    let (pass_comp, _) = components(map, passable);
    let Some(first) = starts.first() else {
        return Err("no starts".into());
    };
    let home = pass_comp[map.index(*first).ok_or("start off map")?];
    for (a, &s) in starts.iter().enumerate() {
        let i = map.index(s).ok_or("start off map")?;
        if !start_terrain_ok(map.tiles[i].terrain) {
            return Err(format!("start {s:?} on {:?}", map.tiles[i].terrain));
        }
        if pass_comp[i] != home || land_comp[i] != main as u32 {
            return Err(format!("start {s:?} not on the main passable region"));
        }
        if workable_tiles(map, s) < START_MIN_WORKABLE {
            return Err(format!("start {s:?} has too few workable tiles"));
        }
        if passable_neighbors(map, s) < 2 {
            return Err(format!("start {s:?} has no room for its escorts"));
        }
        if map.tiles[i].ruin {
            return Err(format!("ruin on start {s:?}"));
        }
        for hex in s.within(1) {
            if map.get(hex).is_some_and(|t| t.resource.is_some()) {
                return Err(format!("resource in the founding ring of {s:?}"));
            }
        }
        if founding_yields(map, s) != (START_FOOD, START_PRODUCTION) {
            return Err(format!(
                "start {s:?} founding yields {:?}, not {:?}",
                founding_yields(map, s),
                (START_FOOD, START_PRODUCTION)
            ));
        }
        for &o in &starts[a + 1..] {
            if s.distance(o) < START_MIN_DISTANCE {
                return Err(format!("starts {s:?} and {o:?} too close"));
            }
        }
    }
    for i in 0..n {
        if map.tiles[i].ruin {
            let hex = map.hex_at(i);
            if map.move_cost(hex).is_none() {
                return Err(format!("ruin on impassable {hex:?}"));
            }
            if starts.iter().any(|s| s.distance(hex) < 2) {
                return Err(format!("ruin at {hex:?} next to a start"));
            }
        }
    }
    crate::resources::placement::validate(map, starts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Faction;
    use std::collections::BTreeSet;

    fn setup(seed: u64) -> GameSetup {
        GameSetup::standard(seed, [Faction::Switzerland, Faction::India])
    }

    #[test]
    fn india_receives_the_most_passable_land_from_picked_starts() {
        let mut map = Map::new(30, 20, Terrain::Plains);
        let inland = Hex::from_offset(20, 10);
        let shore = Hex::from_offset(5, 5);
        for h in shore.within(8) {
            if shore.distance(h) >= 5
                && let Some(tile) = map.get_mut(h)
            {
                tile.terrain = Terrain::Ocean;
            }
        }
        let initial = vec![inland, shore];
        for human in [false, true] {
            let mut s = setup(17);
            s.players[1].is_human = human;
            let mut a = initial.clone();
            let mut b = initial.clone();
            assign_india_starts(&map, &mut a, &s, StartOptions::default());
            assign_india_starts(&map, &mut b, &s, StartOptions::default());
            assert_eq!(a, b);
            assert_eq!(a, [shore, inland]);
            assert!(start_quality(&map, &a, &s, 0) >= START_MIN_QUALITY);
            assert!(start_quality(&map, &a, &s, 1) >= START_MIN_QUALITY);
        }
    }

    #[test]
    fn quality_weights_land_rings_terrain_mix_ruins_and_rivals() {
        let mut map = Map::new(40, 26, Terrain::Ocean);
        let at = Hex::new(10, 10);
        for hex in at.within(3) {
            map.get_mut(hex).unwrap().terrain = Terrain::Plains;
        }
        // The founding territory counts as its fixed totals (8 + 8 at weight 3); the twelve
        // ring-2 tiles at weight 3 and eighteen outer tiles at weight 1 give two yields each.
        assert_eq!(site_quality(&map, at, &[]), 156);
        map.get_mut(at.neighbor(0)).unwrap().terrain = Terrain::Coast;
        assert_eq!(site_quality(&map, at, &[]), 156); // Ring 1 is sculpted later.
        let ring2 = Hex::new(12, 10);
        map.get_mut(ring2).unwrap().terrain = Terrain::Coast;
        assert_eq!(site_quality(&map, at, &[]), 150);
        map.get_mut(Hex::new(13, 10)).unwrap().terrain = Terrain::Ocean;
        assert_eq!(site_quality(&map, at, &[]), 148);
        map.get_mut(ring2).unwrap().terrain = Terrain::Forest;
        assert_eq!(site_quality(&map, at, &[]), 158); // Balanced yields and forest bonus.
        map.get_mut(Hex::new(10, 12)).unwrap().terrain = Terrain::Hills;
        assert_eq!(site_quality(&map, at, &[]), 159); // Hills and mixed terrain bonus.
        assert_eq!(site_quality(&map, at, &[Hex::new(12, 10)]), 171);
        assert_eq!(site_quality(&map, at, &[Hex::new(18, 10)]), 159);

        let mut setup = setup(1);
        setup.players.truncate(3);
        let starts = [at, Hex::new(17, 10), Hex::new(20, 10)];
        assert_eq!(rival_quality(&starts, &setup, 0), 6); // Nearby ally is not a rival.
        setup.players[1].team = setup.players[2].team;
        assert_eq!(rival_quality(&starts, &setup, 0), 0);
        map.get_mut(Hex::new(12, 10)).unwrap().ruin = true;
        assert_eq!(start_quality(&map, &starts, &setup, 0), 171);
    }

    #[test]
    fn one_yield_starts_rank_below_balanced_starts() {
        let at = Hex::new(10, 10);
        let mut map = Map::new(30, 20, Terrain::Grassland);
        let food_only = site_quality(&map, at, &[]);
        for hex in at.within(3) {
            map.get_mut(hex).unwrap().terrain = Terrain::Hills;
        }
        let production_only = site_quality(&map, at, &[]);
        for hex in at.within(3) {
            map.get_mut(hex).unwrap().terrain = Terrain::Plains;
        }
        let balanced = site_quality(&map, at, &[]);
        assert!(food_only < balanced, "{food_only} vs {balanced}");
        assert!(
            production_only < balanced,
            "{production_only} vs {balanced}"
        );
        assert_eq!(production_tiles(&map, at), 0);
        map.get_mut(at.neighbor(0)).unwrap().terrain = Terrain::Forest;
        map.get_mut(at.neighbor(1)).unwrap().terrain = Terrain::Hills;
        assert_eq!(production_tiles(&map, at), 2);
        assert!(early_production_units(&map, at) >= 2);
        map.get_mut(at.neighbor(0)).unwrap().terrain = Terrain::Plains;
        map.get_mut(at.neighbor(1)).unwrap().resource = Some(crate::resources::Resource::Stone);
        assert_eq!(production_tiles(&map, at), 1);
        assert_eq!(early_production_units(&map, at), 2);
    }

    #[test]
    fn founding_sites_reach_the_target_without_touching_water_or_adding_mountains() {
        for water in 0..=START_MAX_RING_WATER {
            for terrain in [Terrain::Grassland, Terrain::Plains] {
                for filler in [
                    Terrain::Desert,
                    Terrain::Tundra,
                    Terrain::Mountains,
                    Terrain::Forest,
                ] {
                    let at = Hex::new(10, 10);
                    let mut map = Map::new(40, 26, Terrain::Ocean);
                    for hex in at.within(3) {
                        map.get_mut(hex).unwrap().terrain = filler;
                    }
                    map.get_mut(at).unwrap().terrain = terrain;
                    for i in 0..water {
                        map.get_mut(at.neighbor(i)).unwrap().terrain = Terrain::Coast;
                    }
                    let before = map.clone();
                    balance_founding_sites(&mut map, &[at], 3, 0).unwrap();
                    assert_eq!(founding_yields(&map, at), (START_FOOD, START_PRODUCTION));
                    for hex in map.hexes() {
                        let (a, b) = (
                            before.get(hex).unwrap().terrain,
                            map.get(hex).unwrap().terrain,
                        );
                        assert_eq!(a.is_land(), b.is_land(), "{hex:?}");
                        assert!(
                            b != Terrain::Mountains || a == Terrain::Mountains,
                            "{hex:?}"
                        );
                        if at.distance(hex) > 1 {
                            assert_eq!(a, b, "{hex:?} beyond ring 1 changed");
                        }
                    }
                    assert!(matches!(
                        map.get(at).unwrap().terrain,
                        Terrain::Grassland | Terrain::Plains
                    ));
                }
            }
        }
    }

    #[test]
    fn hundred_seeds_are_valid() {
        let mut terrain_seen = BTreeSet::new();
        for seed in 1..=100 {
            let s = setup(seed);
            let (map, starts) = generate(&s);
            assert_eq!(map.width, 48, "seed {seed}");
            assert_eq!(map.height, 32, "seed {seed}");
            assert_eq!(starts.len(), s.players.len(), "seed {seed}");
            validate(&map, &starts).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
            for t in &map.tiles {
                terrain_seen.insert(t.terrain);
            }
        }
        // Across many planets every terrain type shows up.
        assert_eq!(terrain_seen.len(), Terrain::ALL.len());
    }

    #[test]
    fn land_ratio_edges_and_islands() {
        for seed in 1..=100 {
            let (map, starts) = generate(&setup(seed));
            let n = map.len();
            let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
            assert!(
                (40..=50).contains(&(land * 100 / n)),
                "seed {seed}: land {land}/{n}"
            );
            for h in map.hexes() {
                let (c, r) = h.to_offset();
                if c == 0 || r == 0 || c == map.width - 1 || r == map.height - 1 {
                    assert!(!map.get(h).unwrap().terrain.is_land(), "seed {seed}: {h:?}");
                }
            }
            let (comp, sizes) = components(&map, |t| t.is_land());
            let main = largest(&sizes).unwrap();
            for (c, &size) in sizes.iter().enumerate() {
                if c != main {
                    assert!(size <= MAX_ISLAND, "seed {seed}: island of {size}");
                }
            }
            for s in &starts {
                assert_eq!(comp[map.index(*s).unwrap()], main as u32, "seed {seed}");
            }
        }
    }

    #[test]
    fn starts_share_one_passable_region_and_meet_rules() {
        for seed in 1..=100 {
            let (map, starts) = generate(&setup(seed));
            let first = starts[0];
            for &s in &starts {
                let t = map.get(s).unwrap().terrain;
                assert!(matches!(t, Terrain::Grassland | Terrain::Plains));
                assert!(workable_tiles(&map, s) >= 12, "seed {seed}");
                // Reachable by land from the first start.
                assert!(
                    crate::path::land_path(&map, first, s).is_some(),
                    "seed {seed}: {s:?} unreachable from {first:?}"
                );
            }
            for (i, a) in starts.iter().enumerate() {
                for b in &starts[i + 1..] {
                    assert!(a.distance(*b) >= 7, "seed {seed}: {a:?} {b:?}");
                }
            }
        }
    }

    #[test]
    fn mountains_never_split_the_continent() {
        for seed in 1..=100 {
            let (map, _) = generate(&setup(seed));
            let (land_comp, land_sizes) = components(&map, |t| t.is_land());
            let (pass_comp, _) = components(&map, passable);
            let main = largest(&land_sizes).unwrap() as u32;
            let mountains = map
                .tiles
                .iter()
                .filter(|t| t.terrain == Terrain::Mountains)
                .count();
            assert!(mountains > 0, "seed {seed}: no mountains");
            // Every passable tile of the continent is in one passable region.
            let regions: BTreeSet<u32> = (0..map.len())
                .filter(|&i| land_comp[i] == main && passable(map.tiles[i].terrain))
                .map(|i| pass_comp[i])
                .collect();
            assert_eq!(regions.len(), 1, "seed {seed}: continent split");
        }
    }

    #[test]
    fn ruins_count_and_placement() {
        for seed in 1..=100 {
            let (map, starts) = generate(&setup(seed));
            let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
            let ruins: Vec<Hex> = map.hexes().filter(|&h| map.get(h).unwrap().ruin).collect();
            let expected = (land + 30) / 60;
            assert!(
                ruins.len() + 2 >= expected && ruins.len() <= expected,
                "seed {seed}: {} ruins for {land} land",
                ruins.len()
            );
            assert!((8..=14).contains(&ruins.len()), "seed {seed}");
            for r in &ruins {
                assert!(map.move_cost(*r).is_some());
                assert!(starts.iter().all(|s| s.distance(*r) >= 2));
            }
        }
    }

    #[test]
    fn deterministic_and_seed_sensitive() {
        for seed in [1, 2, 77, u64::MAX] {
            // Built by hand: the seed only matters to mapgen, not to faction assignment.
            let s = GameSetup { seed, ..setup(1) };
            let a = generate(&s);
            let b = generate(&s.clone());
            assert_eq!(a, b, "seed {seed}");
        }
        let a = generate(&setup(5));
        let b = generate(&setup(6));
        assert_ne!(a.0, b.0);
        assert_ne!(a.1, b.1);
    }

    #[test]
    fn noise_is_bounded_and_smooth() {
        let v = fractal_noise(9, 0, LAYER_ELEVATION, 48, 32, 8_000, 3);
        assert!(v.iter().all(|x| (0..=1000).contains(x)));
        // Neighbouring tiles differ far less than the full range.
        let map = Map::new(48, 32, Terrain::Ocean);
        let mut total = 0i64;
        let mut pairs = 0i64;
        for i in 0..map.len() {
            if let Some(j) = map.index(map.hex_at(i).neighbor(0)) {
                total += (v[i] - v[j]).abs();
                pairs += 1;
            }
        }
        assert!(
            total / pairs < 120,
            "mean neighbour delta {}",
            total / pairs
        );
        assert_eq!(smooth(0), 0);
        assert_eq!(smooth(1024), 1024);
        assert_eq!(smooth(512), 512);
    }

    #[test]
    fn climate_bands() {
        assert_eq!(climate(100, 300, 500, 100, false), Terrain::Tundra);
        assert_eq!(climate(45, 100, 500, 100, false), Terrain::Desert);
        assert_eq!(climate(0, 100, 500, 100, false), Terrain::Plains);
        assert_eq!(climate(10, 800, 500, 100, false), Terrain::Forest);
        assert_eq!(climate(10, 500, 500, 100, false), Terrain::Grassland);
        assert_eq!(climate(10, 500, 500, 950, false), Terrain::Hills);
        assert_eq!(climate(10, 500, 500, 800, true), Terrain::Hills);
        assert_eq!(latitude(0, 32), 100);
        assert_eq!(latitude(31, 32), 100);
        assert!(latitude(16, 32) <= 4);
    }

    #[test]
    fn retries_are_rare() {
        let mut total = 0;
        for seed in 1..=100 {
            total += generate_counted(&setup(seed)).1;
        }
        assert!(total <= 200, "{total} discarded attempts over 100 seeds");
    }
}

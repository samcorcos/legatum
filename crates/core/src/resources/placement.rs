//! Seeded resource placement, a mapgen layer that runs after starts are chosen so fairness
//! can be enforced per start, and the placement rules `mapgen::validate` checks.
//!
//! 1. Every start gets `START_BONUS` bonus resources exactly 2 hexes out (ring 1 stays
//!    bare: mapgen balances the founding territory by terrain alone), on flat land and only
//!    Food/Production ones, so every start's quality rises by the same amount (they count in
//!    `mapgen::start_quality`; Copper's bonus Gold and hidden strategic resources do not).
//! 2. Every start gets its early deposits (`GUARANTEES`): one bonus Copper 2 hexes out (inside the
//!    capital's first claims) and one Iron 3–5 hexes out, each nearer to it than to any other
//!    start, preferring its own terrain.
//! 3. The later eras' resources (`BORDERLAND`: Niter, Coal, Oil, Silicon) go in the
//!    borderlands: at least `BORDER_MIN_START_DISTANCE` from every start (so no capital's
//!    borders can reach them) and no more than 1.5x farther from the nearest rival start than
//!    from the nearest start. At most `BORDER_MAX_PER_START` of each per nearest start.
//! 4. Extra Copper, Iron and bonus resources fill the map by land share, never within
//!    `CLEAR_RADIUS` of a start, so they can't tilt start quality.
//!
//! One resource per tile, never on water, mountains, start tiles or ruins.

use super::{Resource, is_strategic, stats};
use crate::hex::Hex;
use crate::rng::{Rng, purpose};
use crate::state::{GameSetup, Map};
use crate::types::{TeamId, Terrain};

/// Mapgen random layer. Distinct from the terrain/start/ruin layers (1–7) and left clear of
/// the next few layer numbers other mapgen features may take.
pub const LAYER_RESOURCES: u64 = 20;
pub const START_BONUS: usize = 2;
pub const START_BONUS_RADIUS: i32 = 2;
pub const IRON_MIN_DISTANCE: i32 = 3;
pub const IRON_MAX_DISTANCE: i32 = 5;
pub const COPPER_DISTANCE: i32 = 2;
/// Each start's own deposits: (resource, nearest, farthest hexes out).
pub const GUARANTEES: [(Resource, i32, i32); 2] = [
    (Resource::Copper, COPPER_DISTANCE, COPPER_DISTANCE),
    (Resource::Iron, IRON_MIN_DISTANCE, IRON_MAX_DISTANCE),
];
/// The later eras' resources, placed only in the borderlands.
pub const BORDERLAND: [Resource; 4] = [
    Resource::Niter,
    Resource::Coal,
    Resource::Oil,
    Resource::Silicon,
];
/// Deposits other than a start's own guarantees stay farther than this from every start.
pub const CLEAR_RADIUS: i32 = 3;
/// Hard rule: outside every capital's work radius (`CITY_WORK_RADIUS` 3).
pub const BORDER_MIN_START_DISTANCE: i32 = 4;
/// Preferred, not required: crowded maps have too few tiles this far from every start.
pub const BORDER_PREFERRED_START_DISTANCE: i32 = 5;
/// Preferred, not required: two distant empires have no tile within 8 of both.
pub const BORDER_PREFERRED_REACH: i32 = 8;
pub const BORDER_MAX_PER_START: usize = 3;
/// Two deposits of the same resource stay at least this far apart.
pub const SPACING: i32 = 3;
pub const LAND_TILES_PER_BONUS: usize = 12;
/// Land tiles per deposit of each strategic resource, guaranteed ones included.
pub const LAND_TILES_PER_DEPOSIT: usize = 45;

/// Resources that can guarantee a start's Food/Production without tilting quality.
const START_BONUS_TYPES: [Resource; 3] = [Resource::Wheat, Resource::Cattle, Resource::Stone];

fn free(map: &Map, h: Hex, starts: &[Hex]) -> bool {
    map.get(h).is_some_and(|t| {
        t.terrain.move_cost().is_some() && !t.ruin && t.resource.is_none() && !starts.contains(&h)
    })
}

/// Nearest start (lowest seat on ties) and its distance.
pub fn nearest_start(starts: &[Hex], h: Hex) -> Option<(usize, i32)> {
    starts
        .iter()
        .enumerate()
        .map(|(seat, s)| (seat, s.distance(h)))
        .min_by_key(|&(seat, d)| (d, seat))
}

/// Distance to the second-nearest start, of any team.
fn second_distance(starts: &[Hex], h: Hex) -> Option<i32> {
    let mut d: Vec<i32> = starts.iter().map(|s| s.distance(h)).collect();
    d.sort();
    d.get(1).copied()
}

fn nearest_rival(starts: &[Hex], teams: &[TeamId], h: Hex, seat: usize) -> Option<i32> {
    starts
        .iter()
        .enumerate()
        .filter(|&(i, _)| teams[i] != teams[seat])
        .map(|(_, s)| s.distance(h))
        .min()
}

fn borderland(d1: i32, rival: Option<i32>) -> bool {
    d1 >= BORDER_MIN_START_DISTANCE && rival.is_none_or(|r| r * 2 <= d1 * 3)
}

fn shuffled(hexes: impl IntoIterator<Item = Hex>, rng: &mut Rng) -> Vec<Hex> {
    let mut keyed: Vec<(u64, Hex)> = hexes.into_iter().map(|h| (rng.next_u64(), h)).collect();
    keyed.sort();
    keyed.into_iter().map(|(_, h)| h).collect()
}

fn pick(options: &[Resource], rng: &mut Rng) -> Resource {
    options[rng.range(0, options.len() as i64 - 1) as usize]
}

fn bonus_types(terrain: Terrain) -> Vec<Resource> {
    // Copper is a bonus tile resource, but the start's two Food/Production deposits
    // and the ordinary bonus fill still use Wheat, Cattle and Stone. Copper has
    // its own guaranteed and extra placement pass below.
    START_BONUS_TYPES
        .into_iter()
        .filter(|r| stats(*r).terrains.contains(&terrain))
        .collect()
}

fn set(map: &mut Map, h: Hex, r: Resource) {
    map.get_mut(h).expect("on map").resource = Some(r);
}

fn count(map: &Map, r: Resource) -> Vec<Hex> {
    map.hexes()
        .filter(|&h| map.get(h).unwrap().resource == Some(r))
        .collect()
}

/// Place every resource. `None` if a guarantee can't be met (the attempt is discarded).
pub fn place(map: &mut Map, starts: &[Hex], setup: &GameSetup, attempt: u64) -> Option<()> {
    let stream = |step: u64| {
        Rng::derive(
            setup.seed,
            &[purpose::MAPGEN, attempt, LAYER_RESOURCES, step],
        )
    };
    let teams: Vec<TeamId> = setup.players.iter().map(|p| p.team).collect();
    let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();

    // 1. Two Food/Production bonus resources 2 hexes from every start.
    let mut rng = stream(1);
    for &s in starts {
        let near: Vec<Hex> = s
            .within(START_BONUS_RADIUS)
            .into_iter()
            .filter(|&h| s.distance(h) == START_BONUS_RADIUS && free(map, h, starts))
            .collect();
        let mut placed = 0;
        for h in shuffled(near.iter().copied(), &mut rng) {
            let options: Vec<Resource> = START_BONUS_TYPES
                .into_iter()
                .filter(|r| stats(*r).terrains.contains(&map.get(h).unwrap().terrain))
                .collect();
            if placed < START_BONUS && !options.is_empty() {
                let r = pick(&options, &mut rng);
                set(map, h, r);
                placed += 1;
            }
        }
        // Rare: no flat land nearby. Stone on hills keeps the promise of two resources.
        for h in shuffled(near, &mut rng) {
            if placed < START_BONUS && free(map, h, starts) {
                let options = bonus_types(map.get(h).unwrap().terrain);
                if !options.is_empty() {
                    let r = pick(&options, &mut rng);
                    set(map, h, r);
                    placed += 1;
                }
            }
        }
        if placed < START_BONUS {
            return None;
        }
    }

    // 2. Each start's own Copper and Iron, nearer to it than to any other start.
    let mut rng = stream(2);
    for (resource, min, max) in GUARANTEES {
        for (seat, &s) in starts.iter().enumerate() {
            let h = guarantee_site(map, starts, seat, s, resource, min, max, &mut rng)?;
            set(map, h, resource);
        }
    }

    // 3. The later eras' resources in the borderlands between rivals.
    for (step, resource) in BORDERLAND.into_iter().enumerate() {
        let mut rng = stream(10 + step as u64);
        let terrain = stats(resource).terrains;
        let mut candidates: Vec<(bool, bool, bool, u64, Hex, usize)> = map
            .hexes()
            .filter(|&h| free(map, h, starts))
            .filter_map(|h| {
                let (seat, d1) = nearest_start(starts, h)?;
                // With no rival team (everyone allied), balance between the two nearest starts.
                let rival = nearest_rival(starts, &teams, h, seat).or(second_distance(starts, h));
                borderland(d1, rival).then(|| {
                    (
                        d1 < BORDER_PREFERRED_START_DISTANCE,
                        rival.is_some_and(|r| r > BORDER_PREFERRED_REACH),
                        !terrain.contains(&map.get(h).unwrap().terrain),
                        rng.next_u64(),
                        h,
                        seat,
                    )
                })
            })
            .collect();
        candidates.sort();
        let target = border_target(land, starts.len());
        let mut per_start = vec![0usize; starts.len()];
        let mut placed: Vec<Hex> = Vec::new();
        for (_, _, _, _, h, seat) in candidates {
            if placed.len() >= target {
                break;
            }
            if per_start[seat] < BORDER_MAX_PER_START
                && placed.iter().all(|c| c.distance(h) >= SPACING)
            {
                set(map, h, resource);
                placed.push(h);
                per_start[seat] += 1;
            }
        }
        if placed.len() < border_minimum(starts.len()) {
            return None;
        }
    }

    // 4. Extra Copper and Iron on their own terrain away from the starts.
    for (step, (resource, _, _)) in GUARANTEES.into_iter().enumerate() {
        let mut rng = stream(4 + 16 * step as u64);
        let target = (land / LAND_TILES_PER_DEPOSIT).max(starts.len());
        let mut placed = count(map, resource);
        let open: Vec<Hex> = map
            .hexes()
            .filter(|&h| {
                free(map, h, starts)
                    && stats(resource)
                        .terrains
                        .contains(&map.get(h).unwrap().terrain)
                    && starts.iter().all(|s| s.distance(h) > CLEAR_RADIUS)
            })
            .collect();
        for h in shuffled(open, &mut rng) {
            if placed.len() >= target {
                break;
            }
            if placed.iter().all(|i| i.distance(h) >= SPACING) {
                set(map, h, resource);
                placed.push(h);
            }
        }
    }

    // 5. Extra bonus resources away from the starts, never adjacent to another deposit.
    let mut rng = stream(5);
    let target = (land / LAND_TILES_PER_BONUS).max(START_BONUS * starts.len());
    let mut bonus = map
        .tiles
        .iter()
        .filter(|t| t.resource.is_some_and(|r| START_BONUS_TYPES.contains(&r)))
        .count();
    let open: Vec<Hex> = map
        .hexes()
        .filter(|&h| {
            free(map, h, starts)
                && !bonus_types(map.get(h).unwrap().terrain).is_empty()
                && starts.iter().all(|s| s.distance(h) > CLEAR_RADIUS)
        })
        .collect();
    for h in shuffled(open, &mut rng) {
        if bonus >= target {
            break;
        }
        let lonely = h
            .neighbors()
            .iter()
            .all(|n| map.get(*n).is_none_or(|t| t.resource.is_none()));
        if lonely {
            let r = pick(&bonus_types(map.get(h).unwrap().terrain), &mut rng);
            set(map, h, r);
            bonus += 1;
        }
    }
    Some(())
}

/// The nearest free tile `min`..`max` hexes from start `seat` that is nearer to it than to any
/// other start, facing away from the neighbours, on the resource's own terrain if possible.
#[allow(clippy::too_many_arguments)]
fn guarantee_site(
    map: &Map,
    starts: &[Hex],
    seat: usize,
    s: Hex,
    resource: Resource,
    min: i32,
    max: i32,
    rng: &mut Rng,
) -> Option<Hex> {
    let terrain = stats(resource).terrains;
    let mut candidates: Vec<(i32, std::cmp::Reverse<i32>, bool, u64, Hex)> = s
        .within(max)
        .into_iter()
        .filter(|&h| {
            let d = s.distance(h);
            d >= min && free(map, h, starts) && own_start(starts, seat, h, d)
        })
        .map(|h| {
            // Face away from the neighbours, so a rival's second city is less likely to
            // claim it first; beyond 7 hexes every direction is equally safe.
            let rivals = starts
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != seat)
                .map(|(_, o)| o.distance(h).min(7))
                .min()
                .unwrap_or(7);
            (
                s.distance(h),
                std::cmp::Reverse(rivals),
                !terrain.contains(&map.get(h).unwrap().terrain),
                rng.next_u64(),
                h,
            )
        })
        .collect();
    candidates.sort();
    candidates.first().map(|c| c.4)
}

/// `h`, `d` hexes from start `seat`, is nearer to it than to any other start.
fn own_start(starts: &[Hex], seat: usize, h: Hex, d: i32) -> bool {
    starts
        .iter()
        .enumerate()
        .all(|(i, o)| i == seat || o.distance(h) > d)
}

/// Borderland deposits aimed for per resource: one per `LAND_TILES_PER_DEPOSIT` land tiles, at
/// least ⌈empires ÷ 2⌉ and at most `BORDER_MAX_PER_START` per start.
pub fn border_target(land: usize, starts: usize) -> usize {
    (land / LAND_TILES_PER_DEPOSIT).clamp(border_minimum(starts), BORDER_MAX_PER_START * starts)
}

pub fn border_minimum(starts: usize) -> usize {
    starts.div_ceil(2)
}

/// Placement rules that don't depend on teams. Placement also satisfies `validate_rivals`.
pub fn validate(map: &Map, starts: &[Hex]) -> Result<(), String> {
    for h in map.hexes() {
        let t = map.get(h).unwrap();
        let Some(r) = t.resource else { continue };
        if t.terrain.move_cost().is_none() {
            return Err(format!("{r:?} on {:?} at {h:?}", t.terrain));
        }
        if t.ruin {
            return Err(format!("{r:?} on a ruin at {h:?}"));
        }
        if starts.contains(&h) {
            return Err(format!("{r:?} on the start {h:?}"));
        }
        // A guaranteed Copper may fall back to another passable land tile when a
        // start has no free Hills at distance two; it keeps its bonus yield.
        if r != Resource::Copper && !is_strategic(r) && !stats(r).terrains.contains(&t.terrain) {
            return Err(format!("{r:?} on {:?} at {h:?}", t.terrain));
        }
    }
    for (seat, &s) in starts.iter().enumerate() {
        let bonus = s
            .within(START_BONUS_RADIUS)
            .into_iter()
            .filter(|&h| {
                map.get(h)
                    .and_then(|t| t.resource)
                    .is_some_and(|r| START_BONUS_TYPES.contains(&r))
            })
            .count();
        if bonus < START_BONUS {
            return Err(format!("start {s:?} has {bonus} bonus resources within 2"));
        }
        for (resource, min, max) in GUARANTEES {
            let found = s.within(max).into_iter().any(|h| {
                let d = s.distance(h);
                d >= min
                    && map.get(h).and_then(|t| t.resource) == Some(resource)
                    && own_start(starts, seat, h, d)
            });
            if !found {
                return Err(format!(
                    "start {s:?} has no {resource:?} {min}–{max} hexes out"
                ));
            }
        }
    }
    for resource in BORDERLAND {
        let placed = count(map, resource);
        if placed.len() < border_minimum(starts.len()) {
            return Err(format!(
                "{} {resource:?} deposits for {} starts",
                placed.len(),
                starts.len()
            ));
        }
        let mut per_start = vec![0usize; starts.len()];
        for &h in &placed {
            let (seat, d1) = nearest_start(starts, h).ok_or("no starts")?;
            if !borderland(d1, second_distance(starts, h)) {
                return Err(format!("{resource:?} at {h:?} is not in a borderland"));
            }
            per_start[seat] += 1;
            if per_start[seat] > BORDER_MAX_PER_START {
                return Err(format!(
                    "start {:?} is nearest to more than {BORDER_MAX_PER_START} {resource:?} deposits",
                    starts[seat]
                ));
            }
        }
    }
    Ok(())
}

/// Team-aware rules: the borderland resources lie between *rival* starts, and start quality
/// (including the guaranteed bonus resources) stays within `START_MAX_QUALITY_SPREAD`.
pub fn validate_rivals(map: &Map, starts: &[Hex], setup: &GameSetup) -> Result<(), String> {
    let teams: Vec<TeamId> = setup.players.iter().map(|p| p.team).collect();
    for resource in BORDERLAND {
        for h in count(map, resource) {
            let (seat, d1) = nearest_start(starts, h).ok_or("no starts")?;
            let rival = nearest_rival(starts, &teams, h, seat).or(second_distance(starts, h));
            if !borderland(d1, rival) {
                return Err(format!("{resource:?} at {h:?} is not between rivals"));
            }
        }
    }
    let scores: Vec<i32> = (0..starts.len())
        .map(|seat| crate::mapgen::start_quality(map, starts, setup, seat))
        .collect();
    let (low, high) = (
        scores.iter().copied().min().unwrap_or(0),
        scores.iter().copied().max().unwrap_or(0),
    );
    if high - low > crate::mapgen::START_MAX_QUALITY_SPREAD {
        return Err(format!(
            "start quality spread {} with resources",
            high - low
        ));
    }
    Ok(())
}

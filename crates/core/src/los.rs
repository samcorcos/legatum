//! Integer hex line of sight shared by exploration, combat and player-view forecasts.

use crate::{Hex, Terrain};

/// The first intervening tile that blocks a line. Unknown terrain is treated as opaque so a
/// player-view caller cannot infer hidden map data. Endpoints and adjacent tiles never block.
pub fn blocker(from: Hex, to: Hex, mut terrain: impl FnMut(Hex) -> Option<Terrain>) -> Option<Hex> {
    let distance = from.distance(to);
    if distance <= 1 {
        return None;
    }
    let high = terrain(from) == Some(Terrain::Hills);
    let opaque = |h: Hex, terrain: &mut dyn FnMut(Hex) -> Option<Terrain>| match terrain(h) {
        Some(Terrain::Mountains) | None => true,
        Some(Terrain::Forest | Terrain::Hills) => !high,
        _ => false,
    };
    if let Some(line) = line_offsets(to.q - from.q, to.r - from.r) {
        for &((aq, ar), (bq, br)) in line {
            let a = Hex::new(from.q + i32::from(aq), from.r + i32::from(ar));
            let b = Hex::new(from.q + i32::from(bq), from.r + i32::from(br));
            if opaque(a, &mut terrain) && (a == b || opaque(b, &mut terrain)) {
                return Some(a.min(b));
            }
        }
        return None;
    }
    for step in 1..distance {
        // Cube interpolation. Opposite integer nudges choose the two sides when the exact
        // segment follows a hex edge. Only two opaque edge tiles close that part of the line.
        let a = rounded_step(from, to, step, distance, 1);
        let b = rounded_step(from, to, step, distance, -1);
        if opaque(a, &mut terrain) && (a == b || opaque(b, &mut terrain)) {
            return Some(a.min(b));
        }
    }
    None
}

/// Lines up to this distance come from a precomputed table (`line_offsets`).
const LINE_TABLE_RADIUS: i32 = 12;

type LineStep = ((i8, i8), (i8, i8));

/// The (a, b) steps of `blocker` for a line from the origin to (dq, dr), relative to the
/// origin. The line is translation invariant: the nudges are never multiples of SCALE / 2,
/// so no interpolated coordinate is ever an exact half and rounding commutes with adding
/// whole tiles (`line_table_matches_rounded_steps` checks this).
fn line_offsets(dq: i32, dr: i32) -> Option<&'static [LineStep]> {
    static TABLE: std::sync::LazyLock<Vec<Vec<LineStep>>> = std::sync::LazyLock::new(|| {
        let side = 2 * LINE_TABLE_RADIUS + 1;
        let origin = Hex::new(0, 0);
        (0..side * side)
            .map(|i| {
                let to = Hex::new(i % side - LINE_TABLE_RADIUS, i / side - LINE_TABLE_RADIUS);
                let distance = origin.distance(to);
                (1..distance)
                    .map(|step| {
                        let a = rounded_step(origin, to, step, distance, 1);
                        let b = rounded_step(origin, to, step, distance, -1);
                        ((a.q as i8, a.r as i8), (b.q as i8, b.r as i8))
                    })
                    .collect()
            })
            .collect()
    });
    if dq.abs() > LINE_TABLE_RADIUS || dr.abs() > LINE_TABLE_RADIUS {
        return None;
    }
    let side = 2 * LINE_TABLE_RADIUS + 1;
    Some(&TABLE[((dr + LINE_TABLE_RADIUS) * side + dq + LINE_TABLE_RADIUS) as usize])
}

/// Geometry only: whether terrain leaves a clear line between two tiles. This does not
/// enforce a source's sight radius, team/alliance membership, fog, or unit concealment.
/// For current team sight use `visibility::visible_tiles`; for native team sources only
/// use `visibility::native_visible_tiles`. Apply unit concealment separately.
pub fn visible(from: Hex, to: Hex, terrain: impl FnMut(Hex) -> Option<Terrain>) -> bool {
    blocker(from, to, terrain).is_none()
}

/// Aircraft strike from the air (no terrain blocks them); every other ranged unit needs a clear
/// line at distance two.
pub fn ranged_clear(
    kind: crate::UnitKind,
    from: Hex,
    to: Hex,
    terrain: impl FnMut(Hex) -> Option<Terrain>,
) -> bool {
    matches!(crate::data::unit_class(kind), Some(crate::UnitClass::Air))
        || visible(from, to, terrain)
}

/// Indirect fire and a ranged garrison standing in its own walled city bypass terrain.
/// Target visibility remains the caller's responsibility.
pub fn ranged_clear_with_tech(
    kind: crate::UnitKind,
    knows_steel: bool,
    walled_garrison: bool,
    from: Hex,
    to: Hex,
    terrain: impl FnMut(Hex) -> Option<Terrain>,
) -> bool {
    walled_garrison
        || crate::data::indirect_fire(kind, knows_steel)
        || ranged_clear(kind, from, to, terrain)
}

const SCALE: i64 = 1024;

fn round_ratio(num: i64, den: i64) -> i64 {
    if num >= 0 {
        (num + den / 2) / den
    } else {
        -((-num + den / 2) / den)
    }
}

fn rounded_step(from: Hex, to: Hex, step: i32, distance: i32, side: i64) -> Hex {
    let n = i64::from(distance);
    let i = i64::from(step);
    let interpolate = |a: i32, b: i32, nudge: i64| {
        (i64::from(a) * (n - i) + i64::from(b) * i) * SCALE + nudge * side
    };
    let q = interpolate(from.q, to.q, 1);
    let r = interpolate(from.r, to.r, 2);
    let s = interpolate(from.s(), to.s(), -3);
    let den = n * SCALE;
    let mut cq = round_ratio(q, den);
    let mut cr = round_ratio(r, den);
    let mut cs = round_ratio(s, den);
    let eq = (cq * den - q).abs();
    let er = (cr * den - r).abs();
    let es = (cs * den - s).abs();
    if eq >= er && eq >= es {
        cq = -cr - cs;
    } else if er >= es {
        cr = -cq - cs;
    } else {
        cs = -cq - cr;
    }
    let _ = cs;
    Hex::new(cq as i32, cr as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_table_matches_rounded_steps() {
        let r = LINE_TABLE_RADIUS;
        for from in [
            Hex::new(0, 0),
            Hex::new(7, -3),
            Hex::new(-11, 23),
            Hex::new(40, -20),
        ] {
            for dq in -r..=r {
                for dr in -r..=r {
                    let to = Hex::new(from.q + dq, from.r + dr);
                    let distance = from.distance(to);
                    let line = line_offsets(dq, dr).unwrap();
                    assert_eq!(line.len(), (distance - 1).max(0) as usize);
                    for (step, ((aq, ar), (bq, br))) in (1..).zip(line) {
                        let a = rounded_step(from, to, step, distance, 1);
                        let b = rounded_step(from, to, step, distance, -1);
                        assert_eq!(
                            a,
                            Hex::new(from.q + i32::from(*aq), from.r + i32::from(*ar))
                        );
                        assert_eq!(
                            b,
                            Hex::new(from.q + i32::from(*bq), from.r + i32::from(*br))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn blocking_tile_and_neighbors_remain_visible() {
        let from = Hex::new(0, 0);
        let forest = Hex::new(1, 0);
        let terrain = |h| {
            Some(if h == forest {
                Terrain::Forest
            } else {
                Terrain::Plains
            })
        };
        assert!(visible(from, forest, terrain));
        assert_eq!(blocker(from, Hex::new(2, 0), terrain), Some(forest));
        assert!(visible(from, Hex::new(1, -1), terrain));
    }

    #[test]
    fn hill_viewer_sees_over_forest_but_not_mountain() {
        let from = Hex::new(0, 0);
        let to = Hex::new(2, 0);
        let middle = Hex::new(1, 0);
        assert!(visible(from, to, |h| Some(if h == from {
            Terrain::Hills
        } else if h == middle {
            Terrain::Forest
        } else {
            Terrain::Plains
        })));
        assert_eq!(
            blocker(from, to, |h| Some(if h == from {
                Terrain::Hills
            } else if h == middle {
                Terrain::Mountains
            } else {
                Terrain::Plains
            })),
            Some(middle)
        );
    }

    #[test]
    fn edge_line_requires_both_blockers() {
        let from = Hex::new(0, 0);
        let to = Hex::new(1, 1);
        let a = rounded_step(from, to, 1, 2, 1);
        let b = rounded_step(from, to, 1, 2, -1);
        assert_ne!(a, b);
        assert!(visible(from, to, |h| Some(if h == a {
            Terrain::Mountains
        } else {
            Terrain::Plains
        })));
        assert!(!visible(from, to, |h| Some(if h == a || h == b {
            Terrain::Mountains
        } else {
            Terrain::Plains
        })));
    }
}

use serde::{Deserialize, Serialize};

/// Axial hex coordinate for a pointy-top grid. The map stores tiles in "odd-r" offset rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

/// Neighbor directions, starting east and going counter-clockwise.
pub const DIRECTIONS: [Hex; 6] = [
    Hex::new(1, 0),
    Hex::new(1, -1),
    Hex::new(0, -1),
    Hex::new(-1, 0),
    Hex::new(-1, 1),
    Hex::new(0, 1),
];

impl Hex {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn s(self) -> i32 {
        -self.q - self.r
    }

    pub fn neighbor(self, dir: usize) -> Hex {
        let d = DIRECTIONS[dir % 6];
        Hex::new(self.q + d.q, self.r + d.r)
    }

    pub fn neighbors(self) -> [Hex; 6] {
        std::array::from_fn(|i| self.neighbor(i))
    }

    /// Hex distance. Computed in i64 and saturated, so hostile coordinates can't overflow.
    pub fn distance(self, other: Hex) -> i32 {
        let (q1, r1, q2, r2) = (self.q as i64, self.r as i64, other.q as i64, other.r as i64);
        let dq = q1 - q2;
        let dr = r1 - r2;
        let ds = (-q1 - r1) - (-q2 - r2);
        ((dq.abs() + dr.abs() + ds.abs()) / 2).min(i32::MAX as i64) as i32
    }

    /// All hexes within `radius` (including self), sorted for deterministic iteration.
    pub fn within(self, radius: i32) -> Vec<Hex> {
        // Built in (q, r) order, which is `Hex`'s `Ord`, so no sort is needed. Hot in yields.
        let n = radius.clamp(0, 1 << 12) as usize;
        let mut out = Vec::with_capacity(3 * n * (n + 1) + 1);
        for dq in -radius..=radius {
            let lo = (-radius).max(-dq - radius);
            let hi = radius.min(-dq + radius);
            for dr in lo..=hi {
                out.push(Hex::new(self.q + dq, self.r + dr));
            }
        }
        debug_assert!(out.is_sorted());
        out
    }

    /// `within(radius)` as an iterator, in the same order, without allocating.
    pub fn within_iter(self, radius: i32) -> impl Iterator<Item = Hex> {
        (-radius..=radius).flat_map(move |dq| {
            let lo = (-radius).max(-dq - radius);
            let hi = radius.min(-dq + radius);
            (lo..=hi).map(move |dr| Hex::new(self.q + dq, self.r + dr))
        })
    }

    /// Hexes at exactly `radius`, sorted.
    pub fn ring(self, radius: i32) -> Vec<Hex> {
        let mut out: Vec<Hex> = self
            .within(radius)
            .into_iter()
            .filter(|h| self.distance(*h) == radius)
            .collect();
        out.sort();
        out
    }

    /// Odd-r offset coordinates `(col, row)`. Saturates instead of overflowing for extreme
    /// coordinates (which are always off-map).
    pub fn to_offset(self) -> (i32, i32) {
        let (q, r) = (self.q as i64, self.r as i64);
        let col = q + (r - (r & 1)) / 2;
        (col.clamp(i32::MIN as i64, i32::MAX as i64) as i32, self.r)
    }

    pub fn from_offset(col: i32, row: i32) -> Hex {
        let q = col - (row - (row & 1)) / 2;
        Hex::new(q, row)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn within_iter_matches_within() {
        for radius in -1..=5 {
            let h = Hex::new(3, -7);
            assert_eq!(h.within_iter(radius).collect::<Vec<_>>(), h.within(radius));
        }
    }

    use super::*;

    #[test]
    fn offset_round_trip() {
        for row in 0..10 {
            for col in 0..10 {
                assert_eq!(Hex::from_offset(col, row).to_offset(), (col, row));
            }
        }
    }

    #[test]
    fn neighbors_are_distance_one() {
        let h = Hex::new(3, -2);
        for n in h.neighbors() {
            assert_eq!(h.distance(n), 1);
        }
    }

    #[test]
    fn within_counts() {
        let h = Hex::new(0, 0);
        assert_eq!(h.within(0).len(), 1);
        assert_eq!(h.within(1).len(), 7);
        assert_eq!(h.within(2).len(), 19);
        assert_eq!(h.ring(2).len(), 12);
    }
}

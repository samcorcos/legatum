//! Deterministic RNG. Every draw derives from `(stream seed, turn, purpose, entity ids)` so results
//! never depend on the order in which systems happen to call it. The default dice stream uses the
//! game seed; paired continuations may override it without changing map-seeded content.

/// Purposes keep independent random streams apart.
pub mod purpose {
    pub const MAPGEN: u64 = 1;
    pub const COMBAT: u64 = 2;
    pub const RUINS: u64 = 3;
    pub const AI: u64 = 4;
    pub const NAMES: u64 = 5;
    pub const SETUP: u64 = 6;
    pub const CITY_CAPTURE: u64 = 7;
    pub const BARBARIANS: u64 = 8;
    pub const OBJECTIVES: u64 = 9;
    /// Same-turn ties between empires (Sam, 2026-11-22, via the Fable review): no seat is
    /// favoured by its id. Two empires buying one tile, entering an era first, finishing a world
    /// wonder with the same overflow.
    pub const TILE_CONTEST: u64 = 10;
    pub const ERA_FIRST: u64 = 11;
    pub const WONDER_RACE: u64 = 12;
    /// Units of teams not at war, equally strong, stepping onto one tile (Sam, 2026-11-22).
    pub const UNIT_CONTEST: u64 = 13;
    /// Which AI teams start allied (`GameSetup::ai_alliance_pairs`).
    pub const AI_ALLIANCES: u64 = 14;
}

/// `items` in a seeded random order, the same for the same `(seed, parts)` whatever order they
/// came in: each gets an independent draw, and the order is by draw (then by the item itself).
pub fn seeded_order<T: Ord + Copy>(seed: u64, parts: &[u64], items: &[T]) -> Vec<T> {
    let mut sorted = items.to_vec();
    sorted.sort();
    let mut rng = Rng::derive(seed, parts);
    let mut keyed: Vec<(u64, T)> = sorted.into_iter().map(|t| (rng.next_u64(), t)).collect();
    keyed.sort();
    keyed.into_iter().map(|(_, t)| t).collect()
}

/// SplitMix64.
#[derive(Clone, Debug)]
pub struct Rng(u64);

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// A stream derived from a seed and a list of parts, e.g. `[turn, purpose::COMBAT, unit_id]`.
    pub fn derive(seed: u64, parts: &[u64]) -> Self {
        let mut s = mix(seed ^ 0x9e37_79b9_7f4a_7c15);
        for p in parts {
            s = mix(s ^ mix(p.wrapping_add(0x9e37_79b9_7f4a_7c15)));
        }
        Self(s)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0)
    }

    /// Uniform integer in `lo..=hi`.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(lo <= hi);
        // Wide intermediates so any valid i64 range works.
        let span = (hi as i128 - lo as i128 + 1) as u128;
        (lo as i128 + (self.next_u64() as u128 % span) as i128) as i64
    }

    /// Uniform integer in `0..100`.
    pub fn percent(&mut self) -> u32 {
        (self.next_u64() % 100) as u32
    }

    /// True with probability `pct`/100.
    pub fn chance(&mut self, pct: u32) -> bool {
        self.percent() < pct
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_streams_are_reproducible_and_distinct() {
        let a: Vec<u64> = (0..5)
            .map({
                let mut r = Rng::derive(42, &[1, 2]);
                move |_| r.next_u64()
            })
            .collect();
        let b: Vec<u64> = (0..5)
            .map({
                let mut r = Rng::derive(42, &[1, 2]);
                move |_| r.next_u64()
            })
            .collect();
        let c: Vec<u64> = (0..5)
            .map({
                let mut r = Rng::derive(42, &[1, 3]);
                move |_| r.next_u64()
            })
            .collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn range_is_inclusive() {
        let mut r = Rng::new(7);
        let mut seen = [false; 3];
        for _ in 0..200 {
            let v = r.range(80, 82);
            assert!((80..=82).contains(&v));
            seen[(v - 80) as usize] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }
}

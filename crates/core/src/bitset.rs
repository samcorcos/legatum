use serde::{Deserialize, Serialize};

/// Fixed-size bit set, used for per-player explored tiles.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitSet {
    len: usize,
    words: Vec<u64>,
}

impl BitSet {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            words: vec![0; len.div_ceil(64)],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Serde can construct storage that does not match the declared bit count.
    /// Check this before accepting an untrusted save or reading any of its bits.
    pub fn has_valid_storage(&self) -> bool {
        self.words.len() == self.len.div_ceil(64)
    }

    pub fn get(&self, i: usize) -> bool {
        i < self.len && self.words[i / 64] & (1 << (i % 64)) != 0
    }

    pub fn set(&mut self, i: usize) {
        if i < self.len {
            self.words[i / 64] |= 1 << (i % 64);
        }
    }

    /// Set indices in ascending order, excluding padding beyond the logical length.
    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(word_index, &word)| {
                let mut remaining = word;
                std::iter::from_fn(move || {
                    if remaining == 0 {
                        return None;
                    }
                    let bit = remaining.trailing_zeros() as usize;
                    remaining &= remaining - 1;
                    Some(word_index * 64 + bit)
                })
            })
            .take_while(|&i| i < self.len)
    }

    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;
    use crate::rng::Rng;

    #[test]
    fn set_bit_iteration_matches_get_across_word_boundaries_and_padding() {
        for len in [0, 1, 2, 63, 64, 65, 127, 128, 129, 1536, 3168] {
            for seed in 0..8 {
                let mut bits = BitSet::new(len);
                let mut rng = Rng::derive(seed, &[len as u64, 0xB13]);
                for i in 0..len {
                    if seed == 0 || (seed != 1 && rng.chance(35)) {
                        bits.set(i);
                    }
                }
                bits.set(len); // Out-of-range sets still do nothing.
                let before = serde_json::to_vec(&bits).unwrap();
                let expected: Vec<_> = (0..len).filter(|&i| bits.get(i)).collect();
                assert_eq!(bits.iter_ones().collect::<Vec<_>>(), expected);
                assert_eq!(bits.iter_ones().count(), bits.count());
                assert_eq!(serde_json::to_vec(&bits).unwrap(), before);
            }
        }
        // Deserialization can carry set padding. The iterator must agree with get,
        // which ignores it, without normalizing or changing the serialized words.
        let bits: BitSet = serde_json::from_str(
            r#"{"len":65,"words":[9223372036854775809,18446744073709551615]}"#,
        )
        .unwrap();
        assert_eq!(bits.iter_ones().collect::<Vec<_>>(), vec![0, 63, 64]);
    }
}

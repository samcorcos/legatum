/// Seat credentials use system entropy (the browser's `crypto.getRandomValues` in WASM),
/// independently of the game's deterministic RNG/seed.
pub(crate) fn token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    fill(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// A random game id: 128 bits of system entropy. It says nothing about the game's state or seed.
pub(crate) fn game_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    fill(&mut bytes)?;
    Ok(format!(
        "game-{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ))
}

/// A secret 128-bit key for [`masked_hash`], made once per hosted game and never sent or saved.
pub(crate) fn hash_key() -> Result<(u64, u64), String> {
    let mut bytes = [0u8; 16];
    fill(&mut bytes)?;
    let (a, b) = bytes.split_at(8);
    Ok((
        u64::from_le_bytes(a.try_into().expect("8 bytes")),
        u64::from_le_bytes(b.try_into().expect("8 bytes")),
    ))
}

/// The per-turn value players see in place of the state hash: SipHash-2-4 of the hash under the
/// game's secret key. Every seat gets the same value for a turn, but nobody can test a guess at
/// the hidden state (for example a short map seed) against it without the key.
pub(crate) fn masked_hash(key: (u64, u64), state_hash: u64) -> u64 {
    use std::hash::Hasher;
    #[allow(deprecated)] // the only keyed hash in std; SipHash-2-4 itself is sound
    let mut hasher = std::hash::SipHasher::new_with_keys(key.0, key.1);
    hasher.write_u64(state_hash);
    hasher.finish()
}

#[cfg(target_arch = "wasm32")]
fn fill(bytes: &mut [u8]) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(catch, js_namespace = crypto, js_name = getRandomValues)]
        fn get_random_values(bytes: &mut [u8]) -> Result<JsValue, JsValue>;
    }
    get_random_values(bytes)
        .map(|_| ())
        .map_err(|_| "Cannot create seat credential: no secure random source".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn fill(bytes: &mut [u8]) -> Result<(), String> {
    getrandom::fill(bytes).map_err(|e| format!("Cannot create seat credential: {e}"))
}

/// Compare a presented credential with a stored one in time that depends only on the length,
/// so response timing never reveals how many leading characters were right.
pub(crate) fn matches(stored: Option<&str>, presented: Option<&str>) -> bool {
    let (Some(stored), Some(presented)) = (stored, presented) else {
        return false;
    };
    let (a, b) = (stored.as_bytes(), presented.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let diff = a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    std::hint::black_box(diff) == 0
}

#[cfg(test)]
mod tests {
    use super::{matches, token};

    #[test]
    fn credentials_match_only_when_equal_and_present() {
        assert!(matches(Some("abc"), Some("abc")));
        assert!(!matches(Some("abc"), Some("abd")));
        assert!(!matches(Some("abc"), Some("ab")));
        assert!(!matches(None, Some("abc")));
        assert!(!matches(Some("abc"), None));
        assert!(!matches(None, None));
    }

    #[test]
    fn tokens_are_256_bit_hex_and_differ() {
        let (a, b) = (token().unwrap(), token().unwrap());
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}

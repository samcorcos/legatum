/// Seat credentials use system entropy (the browser's `crypto.getRandomValues` in WASM),
/// independently of the game's deterministic RNG/seed.
pub(crate) fn token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    fill(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
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

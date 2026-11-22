/// Seat credentials use OS entropy, independently of the game's deterministic RNG/seed.
pub(crate) fn token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| format!("Cannot create seat credential: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
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
    use super::matches;

    #[test]
    fn credentials_match_only_when_equal_and_present() {
        assert!(matches(Some("abc"), Some("abc")));
        assert!(!matches(Some("abc"), Some("abd")));
        assert!(!matches(Some("abc"), Some("ab")));
        assert!(!matches(None, Some("abc")));
        assert!(!matches(Some("abc"), None));
        assert!(!matches(None, None));
    }
}

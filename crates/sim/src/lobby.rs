//! Opt-in simulator lobby presets; no core rules or save fields are changed.

use legatum_core::Faction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum MapSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl MapSize {
    pub fn dimensions(self) -> (i32, i32) {
        match self {
            Self::Small => (40, 26),
            Self::Medium => (48, 32),
            Self::Large => (60, 40),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub players: usize,
    pub map_size: MapSize,
    pub factions: Option<[Faction; 8]>,
    /// Rotate explicit factions one seat per game for position-balanced smoke runs.
    pub rotate_factions: bool,
    pub mirror: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            players: 8,
            map_size: MapSize::Medium,
            factions: None,
            rotate_factions: false,
            mirror: false,
        }
    }
}

pub fn parse_faction(value: &str) -> Result<Faction, String> {
    Faction::ALL
        .into_iter()
        .find(|f| format!("{f:?}").eq_ignore_ascii_case(value))
        .ok_or_else(|| {
            format!(
                "unknown faction {value}; expected one of {:?}",
                Faction::ALL
            )
        })
}

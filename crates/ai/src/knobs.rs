//! The scripted AI's personality: five 0-100 knobs, with defaults per faction.

use legatum_core::Faction;
use serde::{Deserialize, Serialize};

/// Each knob is 0-100. Higher means "care more about this".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Knobs {
    /// Settler priority and target city count.
    pub expansion: i32,
    /// Army size and military production.
    pub military: i32,
    /// Research-oriented techs and buildings.
    pub science: i32,
    /// Growth, production and gold buildings and techs.
    pub economy: i32,
    /// Willingness to attack: sets the local strength ratio needed to attack, from 130%
    /// (aggression 0) down to 90% (aggression 100), and whether idle armies march on enemies.
    pub aggression: i32,
}

impl Knobs {
    pub const fn new(
        expansion: i32,
        military: i32,
        science: i32,
        economy: i32,
        aggression: i32,
    ) -> Self {
        Self {
            expansion,
            military,
            science,
            economy,
            aggression,
        }
    }

    /// Defaults per faction, leaning into each faction's bent rule and goal.
    pub fn for_faction(faction: Faction) -> Self {
        match faction {
            // Rich, cheap buying, goal is a 200 Gold hoard.
            Faction::Switzerland => Knobs::new(59, 57, 34, 100, 11),
            // Dense regional courts need a funded guard before optional wars.
            Faction::India => Knobs::new(70, 60, 40, 70, 40),
            // Double ruin rewards, goal needs Deep Survey and 100 explored land tiles.
            Faction::Byzantium => Knobs::new(13, 100, 100, 80, 77),
            Faction::Germany => Knobs::new(15, 59, 48, 100, 0),
            Faction::Celts => Knobs::new(46, 55, 40, 75, 30),
            Faction::Mongols => Knobs::new(65, 45, 45, 65, 55),
            Faction::Sparta => Knobs::new(40, 80, 35, 40, 85),
            // Finish the lab/Relay route before optional campaigns. Scoring investments can
            // still exceed the smaller routine army quota when a public title is reachable.
            Faction::Athens => Knobs::new(73, 39, 100, 65, 49),
            Faction::Aztecs => Knobs::new(75, 65, 60, 55, 65),
            Faction::Egypt => Knobs::new(70, 55, 45, 90, 35),
            Faction::Rome => Knobs::new(75, 65, 55, 55, 65),
        }
    }

    /// All knobs clamped to 0..=100.
    pub fn clamped(self) -> Self {
        let c = |v: i32| v.clamp(0, 100);
        Knobs::new(
            c(self.expansion),
            c(self.military),
            c(self.science),
            c(self.economy),
            c(self.aggression),
        )
    }

    /// Local strength ratio (percent) needed to attack: 130 at aggression 0, 90 at 100.
    pub fn attack_threshold(self) -> i32 {
        130 - 40 * self.aggression.clamp(0, 100) / 100
    }

    /// How many cities this empire wants before it stops training settlers.
    pub fn target_cities(self) -> i32 {
        3 + self.expansion.clamp(0, 100) / 20
    }

    /// Opening defense competes with expansion and faction buildings. Difficulty's existing
    /// military scaling makes Easy defer it more often and Hard invest sooner.
    pub(crate) fn opening_defense_value(self) -> i32 {
        400 + self.military * 20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_spans_130_to_90() {
        assert_eq!(Knobs::new(0, 0, 0, 0, 0).attack_threshold(), 130);
        assert_eq!(Knobs::new(0, 0, 0, 0, 100).attack_threshold(), 90);
        assert_eq!(Knobs::new(0, 0, 0, 0, 50).attack_threshold(), 110);
    }

    #[test]
    fn faction_defaults_are_in_range() {
        for f in Faction::ALL {
            let k = Knobs::for_faction(f);
            assert_eq!(k, k.clamped());
        }
    }
}

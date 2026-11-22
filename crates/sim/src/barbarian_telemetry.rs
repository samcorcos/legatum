//! Read-only barbarian observations per game, from the event log only.

use legatum_core::events::Defender;
use legatum_core::{Event, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// "Early" losses: before this resolved turn.
pub const EARLY_TURN: u32 = 50;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarbarianTelemetry {
    pub camps_spawned: u32,
    /// Cleared by a unit (with Gold).
    pub camps_cleared: u32,
    /// Dispersed by growing borders.
    pub camps_dispersed: u32,
    pub first_camp_turn: Option<u32>,
    pub scouts_spotted: u32,
    pub raids_launched: u32,
    pub barbarian_units_killed: u32,
    /// Empire units killed in any fight with barbarians (attacking or defending).
    pub empire_units_killed: u32,
    /// Of those, units that died attacking barbarians (the rest died defending).
    #[serde(default)]
    pub empire_units_killed_attacking: u32,
    /// Empire units killed by barbarians, by unit kind.
    #[serde(default)]
    pub empire_losses_by_kind: BTreeMap<String, u32>,
    /// Settlers (the only civilians) taken by barbarians, and taken back from them.
    pub settlers_captured: u32,
    pub settlers_rescued: u32,
    /// Barbarian attacks that left a city at 0 HP.
    pub cities_zeroed: u32,
    pub gold_awarded: i64,
    /// Per empire: units (military or civilian) lost to barbarians before `EARLY_TURN`.
    pub early_losses: BTreeMap<u8, u32>,
    /// Per empire: Settlers lost to barbarians before `EARLY_TURN`.
    pub early_settler_losses: BTreeMap<u8, u32>,
}

impl BarbarianTelemetry {
    pub fn observe(&mut self, turn: u32, event: &Event) {
        let early = turn < EARLY_TURN;
        let lost = |t: &mut Self, owner: PlayerId, settler: bool| {
            if early && !owner.is_barbarian() {
                *t.early_losses.entry(owner.0).or_default() += 1;
                if settler {
                    *t.early_settler_losses.entry(owner.0).or_default() += 1;
                }
            }
        };
        match event {
            Event::BarbarianCampSpawned { .. } => {
                self.camps_spawned += 1;
                self.first_camp_turn.get_or_insert(turn);
            }
            Event::BarbarianCampCleared { unit, gold, .. } => {
                if unit.is_some() {
                    self.camps_cleared += 1;
                } else {
                    self.camps_dispersed += 1;
                }
                self.gold_awarded += i64::from(*gold);
            }
            Event::BarbarianScoutSpotted { .. } => self.scouts_spotted += 1,
            Event::BarbarianRaidLaunched { .. } => self.raids_launched += 1,
            Event::SettlerCaptured { from, to, .. } if to.is_barbarian() => {
                self.settlers_captured += 1;
                lost(self, *from, true);
            }
            Event::SettlerCaptured { from, .. } if from.is_barbarian() => {
                self.settlers_rescued += 1;
            }
            Event::Battle {
                attacker,
                attacker_owner,
                defender,
                attacker_hp_after,
                defender_hp_after,
                ..
            } => {
                let (def_owner, def_kind) = match defender {
                    Defender::Unit {
                        owner, unit_kind, ..
                    } => (*owner, Some(*unit_kind)),
                    Defender::City { owner, .. } => (*owner, None),
                };
                if !attacker_owner.is_barbarian() && !def_owner.is_barbarian() {
                    return;
                }
                if attacker_owner.is_barbarian() && def_kind.is_none() && *defender_hp_after == 0 {
                    self.cities_zeroed += 1;
                }
                let attacker_kind = match event {
                    Event::Battle {
                        attacker_strength, ..
                    } => attacker_strength
                        .parts
                        .first()
                        .map(|p| p.0.clone())
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                for (owner, dead, attacking, kind) in [
                    (
                        *attacker_owner,
                        attacker.unit().is_some() && *attacker_hp_after == 0,
                        true,
                        attacker_kind,
                    ),
                    (
                        def_owner,
                        def_kind.is_some() && *defender_hp_after == 0,
                        false,
                        def_kind.map(|k| format!("{k:?}")).unwrap_or_default(),
                    ),
                ] {
                    if !dead {
                        continue;
                    }
                    if owner.is_barbarian() {
                        self.barbarian_units_killed += 1;
                    } else {
                        self.empire_units_killed += 1;
                        self.empire_units_killed_attacking += u32::from(attacking);
                        *self.empire_losses_by_kind.entry(kind).or_default() += 1;
                        lost(self, owner, false);
                    }
                }
            }
            _ => {}
        }
    }
}

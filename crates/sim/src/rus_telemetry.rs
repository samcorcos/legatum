//! Read-only observations of the Kyivan Rus' kit (`legatum_core::rus`), one record per Rus'
//! seat: Alliances held, the goal, units trained in an ally's version and raids on their cities.
//! They never touch orders, RNG, saves or hashes.

use legatum_core::{Event, Faction, GameState, PlayerId, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One Rus' seat over a game. Older reports omit it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RusTelemetry {
    /// Most Alliances held with living teams at a resolved turn.
    pub peak_alliances: i32,
    /// Sum over resolved turns of the Alliances held.
    pub alliance_turns: u32,
    /// First resolved turn the seat held an Alliance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_alliance_turn: Option<u32>,
    /// Resolved turns the goal scored, and the first of them.
    pub goal_turns: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_goal_turn: Option<u32>,
    /// The goal scored at the last resolved turn.
    pub goal_at_end: bool,
    /// Units seen owned by the seat in an ally's version (`Unit::origin`, not hired Companies).
    pub allied_units: u32,
    /// Those units by the people whose version they are.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub allied_by_faction: BTreeMap<String, u32>,
    /// Barbarian raids launched on the seat's cities.
    pub raids_on: u32,
}

/// The collector: per Rus' seat, plus the allied units already counted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observer {
    seats: BTreeMap<u8, RusTelemetry>,
    seen: BTreeSet<UnitId>,
}

impl Observer {
    pub fn observe_initial(&mut self, state: &GameState) {
        for p in &state.players {
            if p.faction == Faction::Rus {
                self.seats.insert(p.id.0, RusTelemetry::default());
            }
        }
    }

    pub fn observe_event(&mut self, event: &Event) {
        if let Event::BarbarianRaidLaunched { owner, .. } = event
            && let Some(t) = self.seats.get_mut(&owner.0)
        {
            t.raids_on += 1;
        }
    }

    pub fn observe_resolved_turn(&mut self, resolved_turn: u32, state: &GameState) {
        if self.seats.is_empty() {
            return;
        }
        for (&seat, t) in &mut self.seats {
            let p = PlayerId(seat);
            let held = legatum_core::rus::alliances(state, p);
            t.peak_alliances = t.peak_alliances.max(held);
            t.alliance_turns += held.max(0) as u32;
            if held > 0 {
                t.first_alliance_turn.get_or_insert(resolved_turn);
            }
            let done = state.player(p).faction_goal_done;
            if done {
                t.goal_turns += 1;
                t.first_goal_turn.get_or_insert(resolved_turn);
            }
            t.goal_at_end = done;
        }
        for u in state.units.values() {
            let Some(origin) = u.origin else { continue };
            if u.contract_until.is_some() || self.seen.contains(&u.id) {
                continue;
            }
            if let Some(t) = self.seats.get_mut(&u.owner.0)
                && state.player(u.owner).faction == Faction::Rus
            {
                self.seen.insert(u.id);
                t.allied_units += 1;
                *t.allied_by_faction
                    .entry(format!("{origin:?}"))
                    .or_default() += 1;
            }
        }
    }

    pub fn finish(&self) -> BTreeMap<u8, RusTelemetry> {
        self.seats.clone()
    }
}

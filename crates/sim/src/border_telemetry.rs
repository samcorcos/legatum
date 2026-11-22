//! Read-only border and Alliance observations per game: owned tiles at fixed turns, tiles grown
//! Alliances signed, what they cost in Gold, betrayals, wars between former allies
//! and forced disbands while paying for Alliances. Never participates in orders, RNG, saves or
//! hashes.

use legatum_core::diplomacy::RelationState;
use legatum_core::{Event, GameState, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Resolved turns at which every alive seat's owned tile count is recorded.
pub const SNAPSHOT_TURNS: [u32; 4] = [50, 100, 150, 200];
/// A war declared within this many turns of the pair last being allied is a war between
/// former allies.
pub const FORMER_ALLY_TURNS: u32 = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorderTelemetry {
    /// Resolved turn → seat → owned map tiles (alive seats only).
    pub owned_tiles_at: BTreeMap<u32, BTreeMap<u8, u32>>,
    /// Border tiles gained by growth (`Event::TileClaimed`), by seat.
    pub tiles_claimed: BTreeMap<u8, u32>,
    /// Alliances activated, counted once for each team in the pair (team id).
    pub treaties_signed: BTreeMap<u8, u32>,
    /// Gold spent on Alliances (upkeep and breaches), by seat.
    pub treaty_spend: BTreeMap<u8, i64>,
    /// Forced upkeep disbands in a resolved turn in which the seat paid for an Alliance.
    pub treaty_turn_disbands: BTreeMap<u8, u32>,
    /// Betrayals (`Event::AllianceBetrayed`) by the betraying team id.
    #[serde(default)]
    pub betrayals: BTreeMap<u8, u32>,
    /// Wars declared between teams allied within `FORMER_ALLY_TURNS` (or at declaration).
    #[serde(default)]
    pub former_ally_wars: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Collector {
    pub telemetry: BorderTelemetry,
    /// Seats that paid for an Alliance in the turn being resolved (breaches or upkeep).
    paid_treaty: BTreeSet<u8>,
    /// Seats in an Alliance at the last observation: they pay upkeep in the next turn.
    allied_seats: BTreeSet<u8>,
    disbanded: BTreeMap<u8, u32>,
    /// Last resolved turn each team pair (lower id first) was allied.
    last_allied: BTreeMap<(u8, u8), u32>,
}

impl Collector {
    pub fn observe_event(&mut self, resolved_turn: u32, event: &Event) {
        let t = &mut self.telemetry;
        match event {
            Event::TileClaimed { owner, .. } => *t.tiles_claimed.entry(owner.0).or_default() += 1,
            Event::TreatyActivated { relation, .. } => {
                *t.treaties_signed.entry(relation.a.0).or_default() += 1;
                *t.treaties_signed.entry(relation.b.0).or_default() += 1;
            }
            Event::TreatyPayment { player, gold, .. } => {
                *t.treaty_spend.entry(player.0).or_default() += i64::from(*gold);
                self.paid_treaty.insert(player.0);
            }
            Event::AllianceBetrayed { betrayal } => {
                *t.betrayals.entry(betrayal.by.0).or_default() += 1;
            }
            Event::WarDeclared { relation } => {
                let pair = (
                    relation.a.0.min(relation.b.0),
                    relation.a.0.max(relation.b.0),
                );
                if self
                    .last_allied
                    .get(&pair)
                    .is_some_and(|turn| resolved_turn <= turn + FORMER_ALLY_TURNS)
                {
                    t.former_ally_wars += 1;
                }
            }
            Event::UnitDisbanded { owner, reason, .. } if reason.contains("Treasury ran dry") => {
                *self.disbanded.entry(owner.0).or_default() += 1;
            }
            _ => {}
        }
    }

    /// Call once after each step, with the turn number from before the step.
    pub fn observe_resolved_turn(&mut self, resolved_turn: u32, state: &GameState) {
        for seat in &self.allied_seats {
            let player = PlayerId(*seat);
            if state.player(player).alive {
                let upkeep = legatum_core::diplomacy::TreatyKind::Alliance
                    .upkeep_for_cities(state.cities_of(player).count());
                *self.telemetry.treaty_spend.entry(*seat).or_default() += i64::from(upkeep);
            }
        }
        self.paid_treaty.extend(self.allied_seats.iter().copied());
        for (seat, n) in std::mem::take(&mut self.disbanded) {
            if self.paid_treaty.contains(&seat) {
                *self.telemetry.treaty_turn_disbands.entry(seat).or_default() += n;
            }
        }
        self.paid_treaty.clear();
        self.allied_seats.clear();
        if let Some(d) = &state.diplomacy {
            for r in d
                .relations
                .iter()
                .filter(|r| r.state == RelationState::Alliance)
            {
                self.last_allied.insert((r.a.0, r.b.0), resolved_turn);
                for p in state
                    .players
                    .iter()
                    .filter(|p| p.alive && (p.team == r.a || p.team == r.b))
                {
                    self.allied_seats.insert(p.id.0);
                }
            }
        }
        if SNAPSHOT_TURNS.contains(&resolved_turn) {
            let mut owned: BTreeMap<u8, u32> = state
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| (p.id.0, 0))
                .collect();
            for tile in &state.map.tiles {
                if let Some(n) = tile.owner.and_then(|o| owned.get_mut(&o.0)) {
                    *n += 1;
                }
            }
            self.telemetry.owned_tiles_at.insert(resolved_turn, owned);
        }
    }
}

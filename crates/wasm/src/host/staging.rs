//! An in-memory room, independent of the running game. Seat indices are stable slot IDs:
//! changing or closing one slot never moves an occupant into another slot.
use crate::host::ConnectionId;
use legatum_core::{AiDifficulty, Faction, GameSetup, PlayerSetup, TeamId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeatKind {
    Host,
    Open,
    Ai,
    Closed,
}

impl SeatKind {
    fn human(self) -> bool {
        matches!(self, Self::Host | Self::Open)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionChoice {
    Switzerland,
    India,
    Byzantium,
    Germany,
    Celts,
    Mongols,
    Sparta,
    Athens,
    Aztecs,
    Egypt,
    Rome,
    Random,
}

impl FactionChoice {
    fn resolve(self, seed: u64, slot: usize) -> Faction {
        match self {
            Self::Switzerland => Faction::Switzerland,
            Self::India => Faction::India,
            Self::Byzantium => Faction::Byzantium,
            Self::Germany => Faction::Germany,
            Self::Celts => Faction::Celts,
            Self::Mongols => Faction::Mongols,
            Self::Sparta => Faction::Sparta,
            Self::Athens => Faction::Athens,
            Self::Aztecs => Faction::Aztecs,
            Self::Egypt => Faction::Egypt,
            Self::Rome => Faction::Rome,
            // Standalone fallback; Room::setup rotates only the remaining factions.
            Self::Random => {
                let n = 9; // Preserve established seeded room rosters; Egypt and Rome are explicitly selectable.
                Faction::ALL[((seed % n as u64) as usize + slot) % n]
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingSeat {
    pub kind: SeatKind,
    pub name: String,
    pub faction: FactionChoice,
    pub team: u32,
    #[serde(default)]
    pub difficulty: AiDifficulty,
    /// Hard+Search (see `PlayerSetup::search`); omitted when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub search: bool,
}

/// Rule experiments a host can switch on in the lobby. None remain since World Ages became the
/// only Age system (rules 50); the field stays so a room from an older client, which may still
/// send `objectives_deck`, parses (unknown fields are ignored).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StagingRules {}

impl StagingRules {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingConfig {
    pub seed: String,
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub turn_limit: Option<u32>,
    /// Barbarian camps; on unless the host turns them off.
    #[serde(default = "barbarians_on")]
    pub barbarians: bool,
    /// The AI-only teams start in Alliances of two (`GameSetup::ai_alliances`). Off unless the
    /// host turns it on; omitted when off, so rooms from older clients are unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ai_alliances: bool,
    #[serde(default, skip_serializing_if = "StagingRules::is_default")]
    pub rule_experiments: StagingRules,
    pub seats: Vec<StagingSeat>,
}

fn barbarians_on() -> bool {
    true
}

impl StagingConfig {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !(2..=8).contains(&self.seats.len())
            || self.seats[0].kind != SeatKind::Host
            || self.seats[1..].iter().any(|s| s.kind == SeatKind::Host)
        {
            return Err("A room needs 2-8 seats with exactly one host in seat 0".into());
        }
        if self.seed.is_empty()
            || self.seed.len() > 20
            || !self.seed.bytes().all(|c| c.is_ascii_digit())
            || self.seed.parse::<u64>().is_err()
        {
            return Err("The planet seed must be a decimal u64 string".into());
        }
        if self.seats.iter().any(|s| s.name.chars().count() > 80) {
            return Err("Seat names cannot exceed 80 characters".into());
        }
        crate::host::check_dimensions(self.width, self.height, self.turn_limit)
    }
}

/// Public settings deliberately have no generation seed.
#[derive(Clone, Debug, Serialize)]
pub struct StagingSettings {
    pub width: i32,
    pub height: i32,
    pub turn_limit: Option<u32>,
    pub barbarians: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub ai_alliances: bool,
    #[serde(skip_serializing_if = "StagingRules::is_default")]
    pub rule_experiments: StagingRules,
    pub seats: Vec<StagingSeat>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Occupant {
    pub name: String,
    pub faction: FactionChoice,
    pub connected: bool,
    pub ready: bool,
    pub is_host: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct StagingState {
    pub room_id: String,
    pub config: StagingSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    pub occupants: Vec<Option<Occupant>>,
}

pub(crate) struct Reservation {
    name: String,
    faction: FactionChoice,
    pub(crate) connection: Option<ConnectionId>,
    ready: bool,
    pub(crate) token: String,
}

pub(crate) struct Room {
    pub(crate) id: String,
    pub(crate) config: StagingConfig,
    pub(crate) occupants: Vec<Option<Reservation>>,
}

pub(crate) struct RoomError {
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
}

impl RoomError {
    pub(crate) const NO_ROOM: Self = Self {
        code: "no_room",
        message: "No game room is open.",
    };
}

/// A guest's display name as stored everywhere (seat, staging, saved history): control and
/// format characters (bidi overrides, zero-width marks) removed, whitespace collapsed, trimmed
/// and cut to 80 characters. May be empty; callers choose their own fallback.
pub(crate) fn clean_name(value: &str) -> String {
    fn hidden(c: char) -> bool {
        c.is_control()
            || matches!(c, '\u{AD}' | '\u{61C}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}'
                | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{206F}' | '\u{FEFF}')
    }
    // Whitespace controls (tab, newline) separate words, so they become spaces before stripping.
    let spaced: String = value
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .filter(|c| !hidden(*c))
        .collect();
    let joined = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    joined
        .chars()
        .take(80)
        .collect::<String>()
        .trim()
        .to_string()
}

fn name(value: &str) -> String {
    let value = clean_name(value);
    if value.is_empty() {
        "Player".into()
    } else {
        value
    }
}

impl Room {
    pub(crate) fn new(id: String, config: StagingConfig) -> Self {
        Self {
            id,
            occupants: (0..config.seats.len()).map(|_| None).collect(),
            config,
        }
    }

    pub(crate) fn state(&self, host: bool) -> StagingState {
        StagingState {
            room_id: self.id.clone(),
            config: StagingSettings {
                width: self.config.width,
                height: self.config.height,
                turn_limit: self.config.turn_limit,
                barbarians: self.config.barbarians,
                ai_alliances: self.config.ai_alliances,
                rule_experiments: self.config.rule_experiments,
                seats: self.config.seats.clone(),
            },
            seed: host.then(|| self.config.seed.clone()),
            occupants: self
                .occupants
                .iter()
                .enumerate()
                .map(|(i, o)| {
                    o.as_ref().map(|o| Occupant {
                        name: o.name.clone(),
                        faction: o.faction,
                        connected: o.connection.is_some(),
                        ready: o.ready,
                        is_host: self.config.seats[i].kind == SeatKind::Host,
                    })
                })
                .collect(),
        }
    }

    /// Return the sockets whose slot disappeared or stopped being human.
    pub(crate) fn configure(&mut self, config: StagingConfig) -> Vec<ConnectionId> {
        if self.config == config {
            return vec![];
        }
        let mut lost = Vec::new();
        for (i, occupant) in self.occupants.iter_mut().enumerate() {
            if let Some(o) = occupant {
                if config
                    .seats
                    .get(i)
                    .is_none_or(|s| !s.kind.human() || s.kind != self.config.seats[i].kind)
                {
                    lost.extend(o.connection);
                    *occupant = None;
                } else {
                    o.ready = false;
                }
            }
        }
        self.occupants.resize_with(config.seats.len(), || None);
        self.config = config;
        lost
    }

    fn mine(&mut self, connection: ConnectionId) -> Option<&mut Reservation> {
        self.occupants
            .iter_mut()
            .flatten()
            .find(|o| o.connection == Some(connection))
    }

    pub(crate) fn detach(&mut self, connection: ConnectionId) {
        if let Some(o) = self.mine(connection) {
            o.connection = None;
            o.ready = false;
        }
    }

    pub(crate) fn leave(&mut self, connection: ConnectionId) {
        for occupant in &mut self.occupants {
            if occupant
                .as_ref()
                .is_some_and(|o| o.connection == Some(connection))
            {
                *occupant = None;
            }
        }
    }

    /// Release a reservation without moving the host out of their own slot.
    pub(crate) fn release(&mut self, local: bool, slot: usize) -> Result<(), RoomError> {
        if !local {
            return Err(RoomError {
                code: "host_only",
                message: "Only the loopback host can open a seat.",
            });
        }
        if self
            .config
            .seats
            .get(slot)
            .is_none_or(|s| s.kind != SeatKind::Open)
        {
            return Err(RoomError {
                code: "not_releasable",
                message: "Choose a network player seat.",
            });
        }
        if self.occupants[slot]
            .as_ref()
            .is_some_and(|o| o.connection.is_some())
        {
            return Err(RoomError {
                code: "seat_connected",
                message: "That player is connected. Only disconnected seats can be opened.",
            });
        }
        self.occupants[slot] = None;
        Ok(())
    }

    /// Whether `token` is the live reservation of `slot` (a rejoin, which needs no join code).
    pub(crate) fn owns(&self, slot: usize, token: Option<&str>) -> bool {
        self.occupants
            .get(slot)
            .and_then(|o| o.as_ref())
            .is_some_and(|o| crate::host::credentials::matches(Some(&o.token), token))
    }

    /// Whether `connection` holds a seat here.
    pub(crate) fn holds(&self, connection: ConnectionId) -> bool {
        self.occupants
            .iter()
            .flatten()
            .any(|o| o.connection == Some(connection))
    }

    /// Removes the player in a network seat: the seat keeps the name but gets a new credential
    /// nobody holds, so it stays closed to everyone until the host opens it with [`Room::release`].
    /// Returns the connection that held it.
    pub(crate) fn remove(&mut self, slot: usize) -> Result<Option<ConnectionId>, RoomError> {
        if self
            .config
            .seats
            .get(slot)
            .is_none_or(|s| s.kind != SeatKind::Open)
        {
            return Err(RoomError {
                code: "not_removable",
                message: "Choose a network player seat.",
            });
        }
        let token = crate::host::credentials::token().map_err(|_| RoomError {
            code: "credential_failed",
            message: "Cannot create seat credential.",
        })?;
        let Some(o) = self.occupants[slot].as_mut() else {
            return Err(RoomError {
                code: "not_removable",
                message: "Nobody is in that seat.",
            });
        };
        let old = o.connection.take();
        o.ready = false;
        o.token = token;
        Ok(old)
    }

    pub(crate) fn take(
        &mut self,
        connection: ConnectionId,
        local: bool,
        slot: usize,
        player_name: &str,
        token: Option<&str>,
    ) -> Result<(String, Option<ConnectionId>), RoomError> {
        let Some(seat) = self
            .config
            .seats
            .get(slot)
            .filter(|s| s.kind == SeatKind::Open || (s.kind == SeatKind::Host && local))
        else {
            return Err(RoomError {
                code: "not_open",
                message: "That seat is not open.",
            });
        };
        let occupant = self.occupants[slot].as_ref();
        let owns =
            occupant.is_some_and(|o| crate::host::credentials::matches(Some(&o.token), token));
        if occupant.is_some() && !owns && !local {
            return Err(RoomError {
                code: if occupant.is_some_and(|o| o.connection.is_some()) {
                    "seat_occupied"
                } else {
                    "seat_reserved"
                },
                message: "That seat requires its reconnect credential. Ask the host to open this seat.",
            });
        }
        // A stale reconnect must not silently reclaim a reservation the host just opened.
        // Fresh claims omit the token; only the current reservation can authenticate one.
        if !local && !owns && token.is_some() {
            return Err(RoomError {
                code: "seat_expired",
                message: "That reservation has expired. Take an open seat to join again.",
            });
        }
        let faction = occupant.map_or(seat.faction, |o| o.faction);
        let old = occupant
            .and_then(|o| o.connection)
            .filter(|c| *c != connection);
        let token = if owns {
            occupant.expect("credential owner").token.clone()
        } else {
            crate::host::credentials::token().map_err(|_| RoomError {
                code: "credential_failed",
                message: "Cannot create seat credential.",
            })?
        };
        self.leave(connection);
        self.occupants[slot] = Some(Reservation {
            name: name(player_name),
            faction,
            connection: Some(connection),
            ready: false,
            token: token.clone(),
        });
        Ok((token, old))
    }

    pub(crate) fn update(
        &mut self,
        connection: ConnectionId,
        player_name: Option<String>,
        faction: Option<FactionChoice>,
    ) {
        if let Some(o) = self.mine(connection) {
            if let Some(n) = player_name {
                o.name = name(&n);
            }
            if let Some(f) = faction {
                o.faction = f;
            }
            o.ready = false;
        }
    }

    pub(crate) fn ready(&mut self, connection: ConnectionId, ready: bool) {
        if let Some(o) = self.mine(connection) {
            o.ready = ready;
        }
    }

    pub(crate) fn can_start(&self) -> Result<(), RoomError> {
        if self.config.seats.iter().enumerate().any(|(i, s)| {
            s.kind == SeatKind::Open
                && !self.occupants[i]
                    .as_ref()
                    .is_some_and(|o| o.connection.is_some() && o.ready)
        }) {
            return Err(RoomError {
                code: "not_ready",
                message: "Every open seat must be taken and Ready.",
            });
        }
        Ok(())
    }

    /// Human slot IDs, then AI slot IDs; never use the compacted player ID for Random.
    pub(crate) fn player_slots(&self) -> Vec<usize> {
        let seats = &self.config.seats;
        (0..seats.len())
            .filter(|&i| seats[i].kind.human())
            .chain((0..seats.len()).filter(|&i| seats[i].kind == SeatKind::Ai))
            .collect()
    }

    pub(crate) fn setup(&self) -> GameSetup {
        let seed = self.config.seed.parse().expect("validated seed");
        let mut teams = Vec::new();
        let slots = self.player_slots();
        let used: Vec<Faction> = slots
            .iter()
            .filter_map(|&i| {
                let choice = self.occupants[i]
                    .as_ref()
                    .map_or(self.config.seats[i].faction, |o| o.faction);
                (choice != FactionChoice::Random).then(|| choice.resolve(seed, i))
            })
            .collect();
        let mut pool: Vec<_> = Faction::ALL[..9]
            .iter()
            .copied()
            .filter(|f| !used.contains(f))
            .collect();
        let offset = (seed % pool.len() as u64) as usize;
        pool.rotate_left(offset);
        let mut remaining = pool.into_iter();
        let players = self
            .player_slots()
            .into_iter()
            .map(|i| {
                let s = &self.config.seats[i];
                let o = self.occupants[i].as_ref();
                let team = match teams.iter().position(|t| *t == s.team) {
                    Some(team) => team,
                    None => {
                        teams.push(s.team);
                        teams.len() - 1
                    }
                };
                PlayerSetup {
                    name: o.map_or_else(|| name(&s.name), |o| o.name.clone()),
                    faction: {
                        let choice = o.map_or(s.faction, |o| o.faction);
                        if choice != FactionChoice::Random {
                            choice.resolve(seed, i)
                        } else {
                            remaining.next().expect("more factions than seats")
                        }
                    },
                    team: TeamId(team as u8),
                    is_human: s.kind.human(),
                    difficulty: s.difficulty,
                    search: s.search,
                }
            })
            .collect();
        GameSetup {
            seed,
            width: self.config.width,
            height: self.config.height,
            turn_limit: self.config.turn_limit,
            players,
            legacy_all_war: false,
            barbarians: self.config.barbarians,
            ai_alliances: self.config.ai_alliances,
            rule_experiments: legatum_core::experiments::RuleExperiments::default(),
        }
    }
}

#[cfg(test)]
mod faction_tests {
    use super::{FactionChoice, Room, StagingConfig};
    use legatum_core::Faction;

    #[test]
    fn every_faction_can_be_chosen_by_name() {
        for faction in Faction::ALL {
            let name = serde_json::to_value(faction).unwrap();
            let choice: FactionChoice = serde_json::from_value(name).unwrap();
            assert_eq!(choice.resolve(0, 0), faction);
        }
    }

    #[test]
    fn random_room_seats_exclude_human_choices_and_each_other() {
        use super::*;
        let seats = (0..8)
            .map(|i| StagingSeat {
                kind: if i == 0 { SeatKind::Host } else { SeatKind::Ai },
                name: format!("P{i}"),
                faction: if i == 0 {
                    FactionChoice::Aztecs
                } else {
                    FactionChoice::Random
                },
                team: i,
                difficulty: AiDifficulty::Hard,
                search: false,
            })
            .collect();
        let room = Room::new(
            "test".into(),
            StagingConfig {
                seed: u64::MAX.to_string(),
                width: 48,
                height: 32,
                turn_limit: None,
                barbarians: true,
                ai_alliances: false,
                rule_experiments: StagingRules::default(),
                seats,
            },
        );
        let setup = room.setup();
        assert_eq!(setup.players[0].faction, Faction::Aztecs);
        assert_eq!(
            setup
                .players
                .iter()
                .map(|p| p.faction)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            8
        );
    }

    #[test]
    fn random_rotates_legacy_factions_without_seed_overflow() {
        // The modulo is taken before adding the slot so u64::MAX never overflows.
        assert_eq!(FactionChoice::Random.resolve(3, 0), Faction::Germany);
        assert_eq!(
            FactionChoice::Random.resolve(u64::MAX, 1),
            Faction::ALL[((u64::MAX % 9) as usize + 1) % 9]
        );
        let picks: std::collections::BTreeSet<_> = (0..8)
            .map(|slot| FactionChoice::Random.resolve(5, slot))
            .collect();
        assert_eq!(picks.len(), 8);
    }

    #[test]
    fn an_older_rooms_world_ages_switch_is_ignored_and_rules_stay_default() {
        let room = |rules: serde_json::Value| {
            let mut config: StagingConfig = serde_json::from_value(serde_json::json!({
                "seed": "5", "width": 24, "height": 20,
                "seats": [
                    {"kind":"host", "name":"Host", "faction":"Random", "team":0},
                    {"kind":"open", "name":"Guest", "faction":"Random", "team":1}
                ]
            }))
            .unwrap();
            if !rules.is_null() {
                config.rule_experiments = serde_json::from_value(rules).unwrap();
            }
            Room::new("t".into(), config)
        };
        let off = room(serde_json::Value::Null);
        assert_eq!(
            off.setup().rule_experiments,
            legatum_core::experiments::RuleExperiments::default()
        );
        assert!(off.config.rule_experiments.is_default());
        // World Ages is the only Age system: an older client's switch parses and changes nothing.
        let ages = room(serde_json::json!({"objectives_deck": true}));
        assert!(ages.config.rule_experiments.is_default());
        assert_eq!(
            ages.setup().rule_experiments,
            legatum_core::experiments::RuleExperiments::default()
        );
    }
}

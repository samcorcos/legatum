//! Portable archive envelope and telemetry, shared by native and browser sessions.
use crate::{
    json,
    session::OrderMap,
    tape::{TAPE_VERSION, Tape, TapeTurn},
};
use legatum_core::{AiDifficulty, Faction, GameState, scoring};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const SCHEMA_VERSION: u32 = 1;
pub const ENGINE_COMMIT: &str = env!("LEGATUM_ENGINE_COMMIT");
pub const RULES_FINGERPRINT: &str = env!("LEGATUM_RULES_FINGERPRINT");
pub const CHECKPOINT_TURNS: u32 = 5;

pub fn timestamp() -> String {
    #[cfg(target_arch = "wasm32")]
    let nanos = (js_sys::Date::now() * 1_000_000.0) as i128;
    #[cfg(not(target_arch = "wasm32"))]
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i128;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .expect("current date")
        .format(&Rfc3339)
        .expect("UTC timestamp")
}

pub fn game_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        format!(
            "web-{}-{:016x}-{:016x}",
            js_sys::Date::now() as u64,
            (js_sys::Math::random() * u64::MAX as f64) as u64,
            (js_sys::Math::random() * u64::MAX as f64) as u64
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        format!(
            "native-{}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    HumanLocal,
    HumanHotseat,
    Lan,
    Sim,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seat {
    pub player: u8,
    pub name: String,
    pub faction: Faction,
    pub team: u8,
    pub difficulty: AiDifficulty,
    pub human: bool,
    pub final_vp: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub schema_version: u32,
    pub game_id: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub source: Source,
    /// Removed 2026-10-21 (Standard is the only speed): accepted and dropped so archives written
    /// before then still load.
    #[serde(default, skip_serializing)]
    pub speed: Option<String>,
    pub map: Map,
    pub seed: String,
    pub seats: Vec<Seat>,
    pub winner: Option<u8>,
    pub outcome: String,
    pub turn_count: u32,
    pub engine_commit: String,
    pub rules_fingerprint: String,
    /// The recording starts with the first turn captured by this engine.
    pub first_turn: u32,
    pub complete_history: bool,
    pub parent_game_id: Option<String>,
    /// `training` for a game with a Training-mode AI seat (searched battles): it is training
    /// data, kept out of the coach's "Hard in his seat" and verdict tables. Omitted otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map {
    pub generator: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerTelemetry {
    pub player: u8,
    pub vp: u32,
    pub cities: u32,
    pub population: i32,
    /// Science actually produced in the resolved turn (zero in the initial snapshot).
    pub science: i32,
    pub science_stored: i32,
    pub gold: i32,
    pub army_strength: i32,
    pub techs: u32,
    /// Living players on foreign teams currently at War with this player.
    pub wars: Vec<u8>,
    /// Living players on foreign teams at Peace or allied with this player.
    pub peace: Vec<u8>,
    pub alive: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnTelemetry {
    pub turn: u32,
    pub players: Vec<PlayerTelemetry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recording {
    pub metadata: Metadata,
    #[serde(with = "tape_json")]
    pub tape: Tape,
    /// Opaque JSON for a partial recording's start, absent for complete games.
    pub initial_state: Option<String>,
    pub telemetry: Vec<TurnTelemetry>,
    /// Wall-clock play time is client telemetry and never participates in replay hashes.
    #[serde(default)]
    pub turn_times: Vec<TurnPlayTime>,
    /// How smoothly the browser ran each turn (client telemetry, never in replay hashes): read by the perf supervisor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frame_times: Vec<TurnFrameTime>,
}

/// One turn's rendering and input timing in the browser, summed over the time the tab was visible.
/// Durations are whole milliseconds unless the name says tenths (`_ms10`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnFrameTime {
    pub turn: u32,
    /// Animation frames seen, their median and 95th-percentile gap (tenths of ms), the worst gap, and gaps over 50 ms.
    pub frames: u32,
    pub frame_p50_ms10: u32,
    pub frame_p95_ms10: u32,
    pub frame_max_ms: u32,
    pub slow_frames: u32,
    /// Pointer moves, presses and keys: time from the event to the next frame (95th percentile and worst), and how many.
    pub inputs: u32,
    pub input_p95_ms: u32,
    pub input_max_ms: u32,
    /// Main-thread tasks over 50 ms: how many, and their summed length.
    pub long_tasks: u32,
    pub long_task_ms: u32,
    /// Map canvas draws: how many, mean (tenths of ms) and worst.
    pub map_draws: u32,
    pub map_draw_mean_ms10: u32,
    pub map_draw_max_ms: u32,
    /// What was on the map at the end of the turn: units and cities in the viewer's view, and the zoom (x100).
    pub units: u32,
    pub cities: u32,
    pub zoom100: u32,
    /// The map canvas's size in device pixels, in tenths of a megapixel (what a frame has to fill). Nothing that
    /// tells players or devices apart is recorded: no browser, window size or screen.
    pub canvas_mp10: u32,
}

impl TurnFrameTime {
    fn valid(&self) -> bool {
        self.frame_p50_ms10 <= self.frame_p95_ms10
            && self.canvas_mp10 <= 2_000
            && self.frames <= 10_000_000
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnPlayTime {
    pub turn: u32,
    pub ended_at: String,
    pub active_seconds: u32,
    pub era: u8,
    /// A fresh id is generated when the app opens; it groups resumed play sessions.
    pub session_id: String,
}

mod tape_json {
    use super::*;

    /// Saves from before allies fought together record `CallToArms` orders, and saves from before
    /// Mercenary Companies record `PayTribute`: orders that no longer exist. Their archive is from other rules and is dropped on load (`compatible`),
    /// but it has to parse first: forget the retired orders.
    fn drop_retired_orders(tape: &mut serde_json::Value) {
        let turns = tape
            .get_mut("turns")
            .and_then(serde_json::Value::as_array_mut);
        for turn in turns.into_iter().flatten() {
            let orders = turn
                .get_mut("orders")
                .and_then(serde_json::Value::as_object_mut);
            for list in orders.into_iter().flat_map(|o| o.values_mut()) {
                if let Some(list) = list.as_array_mut() {
                    list.retain(|o| {
                        let kind = o.get("type").and_then(serde_json::Value::as_str);
                        let retired_item = |item: &serde_json::Value| {
                            item.get("kind").and_then(serde_json::Value::as_str) == Some("Building")
                                && item
                                    .get("building")
                                    .and_then(serde_json::Value::as_str)
                                    .is_some_and(|name| matches!(name, "AidStation" | "Reactor"))
                        };
                        !matches!(kind, Some("CallToArms" | "PayTribute"))
                            && !o.get("item").is_some_and(retired_item)
                    });
                    for order in list {
                        if let Some(queue) = order
                            .get_mut("queue")
                            .and_then(serde_json::Value::as_array_mut)
                        {
                            queue.retain(|item| {
                                !(item.get("kind").and_then(serde_json::Value::as_str)
                                    == Some("Building")
                                    && item
                                        .get("building")
                                        .and_then(serde_json::Value::as_str)
                                        .is_some_and(|name| {
                                            matches!(name, "AidStation" | "Reactor")
                                        }))
                            });
                        }
                    }
                }
            }
        }
    }
    pub fn serialize<S: serde::Serializer>(tape: &Tape, serializer: S) -> Result<S::Ok, S::Error> {
        let value: serde_json::Value =
            serde_json::from_str(&tape.to_json()).map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Tape, D::Error> {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        drop_retired_orders(&mut value);
        Tape::from_json(&value.to_string()).map_err(serde::de::Error::custom)
    }
}

impl Recording {
    pub fn new(state: &GameState) -> Self {
        let humans = state.players.iter().filter(|p| p.is_human).count();
        let complete = state.turn == 1
            && legatum_core::try_new_game(&state.setup)
                .is_ok_and(|initial| state.hash() == initial.hash());
        let mut this = Self {
            metadata: Metadata {
                schema_version: SCHEMA_VERSION,
                game_id: game_id(),
                created_at: timestamp(),
                finished_at: None,
                source: match humans {
                    0 => Source::Sim,
                    1 => Source::HumanLocal,
                    _ => Source::HumanHotseat,
                },
                speed: None,
                map: Map {
                    generator: "legatum-hex".into(),
                    width: state.setup.width,
                    height: state.setup.height,
                },
                seed: state.setup.seed.to_string(),
                seats: vec![],
                winner: None,
                outcome: "in-progress".into(),
                turn_count: 0,
                engine_commit: ENGINE_COMMIT.into(),
                rules_fingerprint: RULES_FINGERPRINT.into(),
                first_turn: state.turn,
                complete_history: complete,
                parent_game_id: None,
                mode: state
                    .setup
                    .players
                    .iter()
                    .any(|p| {
                        p.search && !p.is_human && p.difficulty == legatum_core::AiDifficulty::Hard
                    })
                    .then(|| "training".to_string()),
            },
            tape: Tape {
                version: TAPE_VERSION,
                setup: state.setup.clone(),
                initial_hash: state.hash(),
                turns: vec![],
            },
            initial_state: (!complete).then(|| json::state_to_json(state)),
            telemetry: vec![],
            turn_times: vec![],
            frame_times: vec![],
        };
        this.update_metadata(state);
        this.telemetry
            .push(snapshot(state.turn.saturating_sub(1), state));
        this
    }

    pub fn observe(&mut self, turn: u32, orders: &OrderMap, state: &GameState) {
        self.observe_with_hash(turn, orders, state, state.hash());
    }

    pub fn observe_with_hash(
        &mut self,
        turn: u32,
        orders: &OrderMap,
        state: &GameState,
        hash: u64,
    ) {
        self.tape.turns.push(TapeTurn {
            turn,
            orders: orders.clone(),
            hash: hash.to_string(),
        });
        self.telemetry.push(snapshot(turn, state));
        self.update_metadata(state);
    }

    pub fn record_turn_time(&mut self, time: TurnPlayTime) -> Result<(), String> {
        OffsetDateTime::parse(&time.ended_at, &Rfc3339).map_err(|_| "invalid turn ended_at")?;
        if time.turn < self.metadata.first_turn
            || time.turn >= self.metadata.first_turn + self.metadata.turn_count
            || time.active_seconds > 86_400
            || !(1..=7).contains(&time.era)
            || time.session_id.is_empty()
            || time.session_id.len() > 128
        {
            return Err("invalid turn play time".into());
        }
        if let Some(existing) = self.turn_times.iter_mut().find(|t| t.turn == time.turn) {
            *existing = time;
        } else {
            self.turn_times.push(time);
            self.turn_times.sort_by_key(|t| t.turn);
        }
        Ok(())
    }

    pub fn record_frame_time(&mut self, time: TurnFrameTime) -> Result<(), String> {
        if time.turn < self.metadata.first_turn
            || time.turn >= self.metadata.first_turn + self.metadata.turn_count
            || !time.valid()
        {
            return Err("invalid turn frame time".into());
        }
        if let Some(existing) = self.frame_times.iter_mut().find(|t| t.turn == time.turn) {
            *existing = time;
        } else {
            self.frame_times.push(time);
            self.frame_times.sort_by_key(|t| t.turn);
        }
        Ok(())
    }

    fn update_metadata(&mut self, state: &GameState) {
        self.metadata.turn_count = self.tape.turns.len() as u32;
        self.metadata.winner = state.winner.map(|t| t.0);
        self.metadata.seats = state
            .players
            .iter()
            .map(|p| Seat {
                player: p.id.0,
                name: self
                    .metadata
                    .seats
                    .iter()
                    .find(|seat| seat.player == p.id.0)
                    .map_or_else(|| p.name.clone(), |seat| seat.name.clone()),
                faction: p.faction,
                team: p.team.0,
                difficulty: state.setup.players[p.id.0 as usize].difficulty,
                human: p.is_human,
                final_vp: scoring::vp_total(state, p.id),
            })
            .collect();
        if state.finished {
            self.metadata.finished_at.get_or_insert_with(timestamp);
            self.metadata.outcome = match state.winner {
                None => "draw",
                Some(t)
                    if state
                        .players
                        .iter()
                        .filter(|p| p.alive)
                        .all(|p| p.team == t) =>
                {
                    "elimination"
                }
                Some(t)
                    if state
                        .vp_holds
                        .get(&t)
                        .is_some_and(|held| *held >= legatum_core::data::VP_HOLD_TURNS) =>
                {
                    "vp"
                }
                Some(_) => "turn-limit",
            }
            .into();
        }
    }

    /// Preserve a distinct history when explicitly loading a file to start another play.
    pub fn fork(&mut self) {
        self.metadata.parent_game_id =
            Some(std::mem::replace(&mut self.metadata.game_id, game_id()));
    }

    pub fn stop(&mut self, outcome: &str) {
        if self.metadata.finished_at.is_none() {
            self.metadata.finished_at = Some(timestamp());
            self.metadata.outcome = outcome.into();
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let m = &self.metadata;
        if m.schema_version != SCHEMA_VERSION {
            return Err("unsupported archive schema version".into());
        }
        if m.game_id.is_empty()
            || m.game_id.len() > 128
            || !m
                .game_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err("invalid archive game id".into());
        }
        OffsetDateTime::parse(&m.created_at, &Rfc3339).map_err(|_| "invalid created_at")?;
        if let Some(at) = &m.finished_at {
            OffsetDateTime::parse(at, &Rfc3339).map_err(|_| "invalid finished_at")?;
        }
        if m.turn_count as usize != self.tape.turns.len()
            || self.telemetry.len() != self.tape.turns.len() + 1
        {
            return Err("archive turn/telemetry counts differ".into());
        }
        if m.seats.is_empty()
            || m.seats.len() > 8
            || m.seats.len() != self.tape.setup.players.len()
            || m.seed != self.tape.setup.seed.to_string()
            || m.map.width != self.tape.setup.width
            || m.map.height != self.tape.setup.height
            || m.complete_history == self.initial_state.is_some()
            || m.first_turn == 0
            || (m.complete_history && m.first_turn != 1)
            || m.first_turn.checked_add(m.turn_count).is_none()
            || !(1..=128).contains(&m.map.width)
            || !(1..=128).contains(&m.map.height)
        {
            return Err("archive setup/metadata differ".into());
        }
        if (m.outcome == "in-progress") != m.finished_at.is_none()
            || ![
                "in-progress",
                "vp",
                "draw",
                "elimination",
                "turn-limit",
                "stalled",
                "panic",
                "abandoned",
            ]
            .contains(&m.outcome.as_str())
        {
            return Err("invalid archive outcome".into());
        }
        for (i, t) in self.tape.turns.iter().enumerate() {
            if t.turn != m.first_turn.checked_add(i as u32).ok_or("turn overflow")? {
                return Err("archive turns are not consecutive".into());
            }
            crate::tape::parse_hash(&t.hash)?;
        }
        let mut previous_turn = None;
        for time in &self.turn_times {
            OffsetDateTime::parse(&time.ended_at, &Rfc3339).map_err(|_| "invalid turn ended_at")?;
            if time.turn < m.first_turn
                || time.turn >= m.first_turn + m.turn_count
                || time.active_seconds > 86_400
                || !(1..=7).contains(&time.era)
                || time.session_id.is_empty()
                || time.session_id.len() > 128
                || previous_turn.is_some_and(|turn| turn >= time.turn)
            {
                return Err("invalid archive turn play times".into());
            }
            previous_turn = Some(time.turn);
        }
        let mut previous_turn = None;
        for time in &self.frame_times {
            if time.turn < m.first_turn
                || time.turn >= m.first_turn + m.turn_count
                || !time.valid()
                || previous_turn.is_some_and(|turn| turn >= time.turn)
            {
                return Err("invalid archive frame times".into());
            }
            previous_turn = Some(time.turn);
        }
        for (i, frame) in self.telemetry.iter().enumerate() {
            if frame.turn != m.first_turn - 1 + i as u32
                || frame.players.len() != m.seats.len()
                || frame
                    .players
                    .iter()
                    .enumerate()
                    .any(|(i, p)| p.player as usize != i)
            {
                return Err("invalid telemetry seats/turns".into());
            }
        }
        for (i, (seat, setup)) in m.seats.iter().zip(&self.tape.setup.players).enumerate() {
            if seat.player as usize != i
                || (m.complete_history
                    && (seat.faction != setup.faction
                        || seat.team != setup.team.0
                        || seat.human != setup.is_human))
                || seat.difficulty != setup.difficulty
            {
                return Err("archive seats/setup differ".into());
            }
            if seat.final_vp != self.telemetry.last().ok_or("missing telemetry")?.players[i].vp {
                return Err("archive VP/telemetry differ".into());
            }
        }
        Ok(())
    }

    pub fn compatible(&self) -> Result<(), String> {
        if self.metadata.rules_fingerprint != RULES_FINGERPRINT {
            return Err(format!(
                "incompatible engine: archive commit {}, rules {}; current commit {}, rules {}",
                self.metadata.engine_commit,
                self.metadata.rules_fingerprint,
                ENGINE_COMMIT,
                RULES_FINGERPRINT
            ));
        }
        Ok(())
    }

    pub fn replay(&self) -> Result<GameState, String> {
        self.validate()?;
        self.compatible()?;
        let mut state = match &self.initial_state {
            Some(s) => json::parse_state(s)?,
            None => self.tape.initial_state(),
        };
        if state.hash() != self.tape.initial_hash {
            return Err("first divergent turn: 0 (initial state)".into());
        }
        for t in &self.tape.turns {
            if state.turn != t.turn || state.finished {
                return Err(format!("invalid replay turn {}", t.turn));
            }
            state = legatum_core::resolve_turn(&state, &t.orders).0;
            if state.hash().to_string() != t.hash {
                return Err(format!("first divergent turn: {}", t.turn));
            }
        }
        Ok(state)
    }
}

/// The name the lobby gives a seat by default: "Player N" for a human, "AI N" for an AI empire (N = seat + 1).
pub fn default_seat_name(seat: usize, human: bool) -> String {
    format!("{} {}", if human { "Player" } else { "AI" }, seat + 1)
}

impl Recording {
    /// A copy that is safe to upload: every seat carries its default name, so names players typed never
    /// leave the device. Names are part of the state hash, so a renamed copy replays its orders to recompute
    /// every hash. A recording that already uses the default names comes back unchanged, with no replay.
    pub fn anonymized(&self) -> Result<Recording, String> {
        let mut out = self.clone();
        let mut typed: Vec<String> = Vec::new();
        let mut rename = |name: &mut String, seat: usize, human: bool| {
            let default = default_seat_name(seat, human);
            if *name != default {
                typed.push(std::mem::replace(name, default));
            }
        };
        for (i, p) in out.tape.setup.players.iter_mut().enumerate() {
            rename(&mut p.name, i, p.is_human);
        }
        for seat in &mut out.metadata.seats {
            rename(&mut seat.name, seat.player as usize, seat.human);
        }
        let mut state = match &self.initial_state {
            Some(s) => {
                let mut state = json::parse_state(s)?;
                for (i, p) in state.setup.players.iter_mut().enumerate() {
                    rename(&mut p.name, i, p.is_human);
                }
                for p in &mut state.players {
                    rename(&mut p.name, p.id.0 as usize, p.is_human);
                }
                state
            }
            None => out.tape.initial_state(),
        };
        if typed.is_empty() {
            return Ok(out);
        }
        if out.initial_state.is_some() {
            out.initial_state = Some(json::state_to_json(&state));
        }
        out.tape.initial_hash = state.hash();
        for t in &mut out.tape.turns {
            state = legatum_core::resolve_turn(&state, &t.orders).0;
            t.hash = state.hash().to_string();
        }
        // A partial recording's start state can quote a name elsewhere (a notice, a log line): never upload that.
        let text = serde_json::to_string(&out).map_err(|e| e.to_string())?;
        if typed
            .iter()
            .filter(|n| !n.trim().is_empty())
            .any(|n| text.contains(&serde_json::to_string(n.as_str()).unwrap_or_default()))
        {
            return Err("a typed player name is still in the recording".into());
        }
        Ok(out)
    }
}

pub fn snapshot(turn: u32, state: &GameState) -> TurnTelemetry {
    TurnTelemetry {
        turn,
        players: state
            .players
            .iter()
            .map(|p| PlayerTelemetry {
                player: p.id.0,
                vp: scoring::vp_total(state, p.id),
                cities: state.cities_of(p.id).count() as u32,
                population: state.cities_of(p.id).map(|c| c.population).sum(),
                science: p.science_history.last().copied().unwrap_or(0),
                science_stored: p.science_stored,
                gold: p.gold,
                army_strength: scoring::army_strength(state, p.id),
                techs: p.techs.len() as u32,
                alive: p.alive,
                wars: state
                    .players
                    .iter()
                    .filter(|q| p.alive && q.alive && state.at_war(p.id, q.id))
                    .map(|q| q.id.0)
                    .collect(),
                peace: state
                    .players
                    .iter()
                    .filter(|q| p.alive && q.alive && q.team != p.team && !state.at_war(p.id, q.id))
                    .map(|q| q.id.0)
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{BrainKind, Session};
    use legatum_core::{GameSetup, Order, PlayerId, TeamId, diplomacy::TreatyKind};

    #[test]
    fn vp_outcome_needs_a_completed_hold_not_just_the_target() {
        let mut state = legatum_core::new_game(&GameSetup::all_ai(7));
        state.finished = true;
        state.winner = Some(TeamId(0));
        let needed = legatum_core::data::VP_HOLD_TURNS;
        let mut recording = Recording::new(&state);
        recording.update_metadata(&state);
        assert_eq!(recording.metadata.outcome, "turn-limit");
        state.vp_holds.insert(TeamId(0), needed - 1);
        recording.update_metadata(&state);
        assert_eq!(recording.metadata.outcome, "turn-limit");
        state.vp_holds.insert(TeamId(0), needed);
        recording.update_metadata(&state);
        assert_eq!(recording.metadata.outcome, "vp");
    }

    #[test]
    fn turn_play_time_survives_archive_load_and_legacy_archive_defaults_empty() {
        let setup = GameSetup::standard(42, [Faction::India, Faction::Byzantium]);
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        session.step(&OrderMap::new());
        let history = session.history.as_mut().unwrap();
        history
            .record_turn_time(TurnPlayTime {
                turn: 1,
                ended_at: "2026-10-18T12:00:00Z".into(),
                active_seconds: 74,
                era: 1,
                session_id: "playtest-a".into(),
            })
            .unwrap();
        let saved = session.save_json();
        let loaded = Session::from_save_json(&saved, |_| BrainKind::Scripted).unwrap();
        assert_eq!(
            loaded.history.as_ref().unwrap().turn_times,
            vec![TurnPlayTime {
                turn: 1,
                ended_at: "2026-10-18T12:00:00Z".into(),
                active_seconds: 74,
                era: 1,
                session_id: "playtest-a".into(),
            }]
        );

        let mut old_save: serde_json::Value = serde_json::from_str(&saved).unwrap();
        old_save["archive"]
            .as_object_mut()
            .unwrap()
            .remove("turn_times");
        let old = Session::from_save_json(&old_save.to_string(), |_| BrainKind::Scripted).unwrap();
        assert!(old.history.as_ref().unwrap().turn_times.is_empty());
    }

    #[test]
    fn frame_times_survive_load_and_bad_ones_are_refused() {
        let setup = GameSetup::standard(42, [Faction::India, Faction::Byzantium]);
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        session.step(&OrderMap::new());
        let history = session.history.as_mut().unwrap();
        let time = TurnFrameTime {
            turn: 1,
            frames: 3600,
            frame_p50_ms10: 167,
            frame_p95_ms10: 334,
            frame_max_ms: 120,
            slow_frames: 4,
            inputs: 900,
            input_p95_ms: 40,
            input_max_ms: 130,
            long_tasks: 2,
            long_task_ms: 140,
            map_draws: 700,
            map_draw_mean_ms10: 52,
            map_draw_max_ms: 31,
            units: 40,
            cities: 12,
            zoom100: 200,
            canvas_mp10: 52,
        };
        history.record_frame_time(time.clone()).unwrap();
        assert!(
            history
                .record_frame_time(TurnFrameTime {
                    frame_p50_ms10: 400,
                    ..time.clone()
                })
                .is_err()
        );
        assert!(
            history
                .record_frame_time(TurnFrameTime {
                    turn: 9,
                    ..time.clone()
                })
                .is_err()
        );
        let saved = session.save_json();
        let loaded = Session::from_save_json(&saved, |_| BrainKind::Scripted).unwrap();
        assert_eq!(loaded.history.as_ref().unwrap().frame_times, vec![time]);
        // An archive from before frame times has none, and an archive without them writes no key.
        let mut old_save: serde_json::Value = serde_json::from_str(&saved).unwrap();
        old_save["archive"]
            .as_object_mut()
            .unwrap()
            .remove("frame_times");
        let old = Session::from_save_json(&old_save.to_string(), |_| BrainKind::Scripted).unwrap();
        assert!(old.history.as_ref().unwrap().frame_times.is_empty());
        assert!(!old.save_json().contains("frame_times"));
    }

    #[test]
    fn archive_tracks_peace_and_war_and_replays_after_load() {
        let mut setup = GameSetup::standard(42, [Faction::India, Faction::Byzantium]);
        setup.players.truncate(3);
        setup.players[2].is_human = true;
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        let check = |session: &Session, war: bool| {
            let history = session.history.as_ref().unwrap();
            assert_eq!(history.tape.version, TAPE_VERSION);
            let players = &history.telemetry.last().unwrap().players;
            for player in &players[..2] {
                assert_eq!(player.wars, if war { vec![2] } else { vec![] });
                assert_eq!(player.peace, if war { vec![] } else { vec![2] });
            }
            assert_eq!(players[2].wars, if war { vec![0, 1] } else { vec![] });
            assert_eq!(players[2].peace, if war { vec![] } else { vec![0, 1] });
            assert_eq!(history.replay().unwrap(), session.state);
            let loaded =
                Session::from_save_json(&session.save_json(), |_| BrainKind::Scripted).unwrap();
            assert_eq!(loaded.history, session.history);
            assert_eq!(loaded.state, session.state);
        };
        check(&session, false);
        let turns = [
            (
                0,
                Order::DeclareWar {
                    against: TeamId(1),
                    breach: false,
                },
            ),
            (
                0,
                Order::ProposeTreaty {
                    to: TeamId(1),
                    kind: TreatyKind::Peace,
                },
            ),
            (
                2,
                Order::RespondToProposal {
                    proposal: legatum_core::diplomacy::ProposalId(1),
                    accept: true,
                    explanation: None,
                },
            ),
        ];
        for (i, (player, order)) in turns.into_iter().enumerate() {
            session.step(&OrderMap::from([(PlayerId(player), vec![order])]));
            check(&session, i < 2);
        }
        assert_eq!(
            session.state.diplomacy.as_ref().unwrap().relations[0].state,
            legatum_core::diplomacy::RelationState::Peace
        );
    }

    #[test]
    fn anonymized_recording_drops_typed_names_and_still_replays() {
        let mut setup = GameSetup::standard(42, [Faction::India, Faction::Byzantium]);
        setup.players.truncate(3);
        setup.players[0].name = "Dana Smith".into();
        setup.players[1].is_human = false;
        setup.players[2].is_human = true;
        setup.players[2].name = "Alex".into();
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        for _ in 0..4 {
            session.step(&OrderMap::new());
        }
        let history = session.history.as_ref().unwrap();
        let public = history.anonymized().unwrap();
        let text = serde_json::to_string(&public).unwrap();
        assert!(!text.contains("Dana Smith") && !text.contains("\"Alex\""));
        let names: Vec<_> = public
            .metadata
            .seats
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, ["Player 1", "AI 2", "Player 3"]);
        // Same game, same orders: the replay ends in the original state except for the names.
        let mut expected = public.replay().unwrap();
        for (p, original) in expected.players.iter_mut().zip(&session.state.players) {
            p.name = original.name.clone();
        }
        expected.setup.players = session.state.setup.players.clone();
        assert_eq!(expected, session.state);
        // Default names need no replay and change nothing.
        assert_eq!(public.anonymized().unwrap(), public);
    }
}

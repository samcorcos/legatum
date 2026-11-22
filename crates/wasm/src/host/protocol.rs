use super::staging::{FactionChoice, StagingState};
use legatum_core::{
    AlongsideAttack, AttackMode, BattleForecast, CityAssaultForecast, Faction, Hex, Order,
    OrderError, PlayerId, PlayerView, TeamId, UnitId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ClientMessage {
    Join {
        seat: usize,
        name: String,
        token: String,
    },
    /// Claim an unassigned seat, or explicitly reassign it from loopback.
    GameTake {
        seat: usize,
        /// The join code of a locked game (at most 32 characters). Never stored or echoed.
        #[serde(default)]
        code: Option<String>,
    },
    /// Loopback host removes a remote player: the seat stays closed until the host opens it.
    GameRemove {
        seat: usize,
    },
    /// Loopback host opens a disconnected seat for a fresh claim.
    GameRelease {
        seat: usize,
    },
    Orders {
        game_id: String,
        turn: u32,
        orders: Vec<Order>,
    },
    Ready {
        game_id: String,
        turn: u32,
        ready: bool,
    },
    Forecast {
        req_id: Value,
        unit: UnitId,
        target: Hex,
        mode: AttackMode,
    },
    /// The combined fight at an enemy city, with the viewer's other attacks queued on it.
    Assault {
        req_id: Value,
        unit: UnitId,
        target: Hex,
        mode: AttackMode,
        #[serde(default)]
        alongside: Vec<AlongsideAttack>,
    },
    Ping,
    StagingWatch {
        #[serde(default)]
        host: bool,
    },
    StagingTake {
        seat: usize,
        name: String,
        token: Option<String>,
        /// The join code of a locked game (at most 32 characters). Never stored or echoed.
        #[serde(default)]
        code: Option<String>,
    },
    StagingRelease {
        seat: usize,
    },
    /// Loopback host removes a remote player from a staging seat; the seat stays closed.
    StagingRemove {
        seat: usize,
    },
    StagingUpdate {
        name: Option<String>,
        faction: Option<FactionChoice>,
    },
    StagingReady {
        ready: bool,
    },
    StagingLeave,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeatStatus {
    pub seat: usize,
    pub player: PlayerId,
    pub name: String,
    pub faction: Faction,
    pub connected: bool,
    pub reserved: bool,
    pub ready: bool,
    pub orders: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub mode: &'static str,
    pub has_game: bool,
    pub turn: u32,
    pub seats: Vec<SeatStatus>,
    pub is_local: bool,
    pub staging: Option<StagingState>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ServerMessage {
    Welcome {
        seat: usize,
        game_id: String,
        orders: Vec<Order>,
    },
    SeatToken {
        seat: usize,
        game_id: String,
        token: String,
    },
    View {
        view: Box<PlayerView>,
    },
    Seats {
        seats: Vec<SeatStatus>,
    },
    Validation {
        errors: Vec<OrderError>,
    },
    Turn {
        /// The new planning turn, matching the following view and autosave.
        turn: u32,
        events: Vec<crate::host::events::VisibleEvent>,
        hash: String,
        finished: bool,
        winner: Option<TeamId>,
    },
    Forecast {
        req_id: Value,
        forecast: Option<BattleForecast>,
    },
    Assault {
        req_id: Value,
        assault: Option<CityAssaultForecast>,
    },
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<&'static str>,
        message: String,
    },
    Pong,
    Staging {
        state: StagingState,
    },
    StagingStarted {
        game_id: String,
        seat: usize,
        token: String,
    },
    StagingTaken {
        room_id: String,
        seat: usize,
        token: String,
    },
    StagingLeft {
        room_id: String,
    },
    StagingSeatLost {
        message: String,
    },
    StagingClosed,
}

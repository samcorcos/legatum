//! The authoritative online host (plans/online-play.md, plans/game-protocol.md): one game, its
//! staging room, seat credentials and every connected player. It has no transport and does no
//! I/O: the browser host (`app/src/browserHost.ts`) feeds it messages through [`net::NetHost`]
//! and writes what it queues to the relay tunnel or to the host's own UI. Saves and game
//! histories are handed to JavaScript, which keeps them in the browser.

mod credentials;
mod events;
pub mod net;
mod outbox;
mod protocol;
mod staging;

pub use protocol::{SeatStatus, Status};
pub use staging::{
    FactionChoice, Occupant, SeatKind, StagingConfig, StagingSeat, StagingSettings, StagingState,
};

use crate::session::{BrainKind, Session};
use events::visible_events;
use legatum_core::{
    GameSetup, GameState, Order, PlayerId, forecast_attack_mode, forecast_city_assault,
    player_view, validate_orders,
};
use protocol::{ClientMessage, ServerMessage};
use serde::Serialize;
use staging::{Room, RoomError, clean_name};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) type ConnectionId = u64;

/// Wall-clock nanoseconds, for unique game and room ids (never for game rules).
fn now_nanos() -> u128 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() * 1_000_000.0) as u128
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }
}

/// The relay's lobby row for this host (the tunnel's `summary` frame).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub players: usize,
    pub humans: usize,
    pub open_seats: usize,
    pub phase: &'static str,
    pub turn: u32,
}

struct Connection {
    sender: outbox::Sender,
    local: bool,
    watching: bool,
    show_seed: bool,
}

struct Seat {
    player: PlayerId,
    name: String,
    connection: Option<ConnectionId>,
    joined: bool,
    token: Option<String>,
    ready: bool,
    orders: Vec<Order>,
    /// The connection and planning turn of this seat's last full join (welcome + view). A repeat
    /// join from the same connection in the same turn is answered with nothing.
    joined_at: Option<(ConnectionId, u32)>,
}

/// Most orders one seat may stage per turn. A real turn stages a few hundred at most.
const MAX_STAGED_ORDERS: usize = 2000;
/// Most validation errors sent back in one reply.
const MAX_VALIDATION_ERRORS: usize = 100;

struct Game {
    session: Session,
    id: String,
    seats: Vec<Seat>,
}

impl Game {
    fn from_session(mut session: Session) -> Result<Self, String> {
        let state = &session.state;
        check_state(state)?;
        let seats = state
            .players
            .iter()
            .filter(|p| p.is_human)
            .map(|p| Seat {
                player: p.id,
                name: session
                    .history
                    .as_ref()
                    .and_then(|h| h.metadata.seats.iter().find(|s| s.player == p.id.0))
                    .map_or_else(|| p.name.clone(), |s| s.name.clone()),
                connection: None,
                joined: false,
                token: None,
                ready: false,
                orders: Vec::new(),
                joined_at: None,
            })
            .collect();
        Ok(Self {
            id: format!("{}-{}", state.hash(), now_nanos()),
            session: {
                if let Some(h) = &mut session.history {
                    // A networked game (the archive's name for it predates online play).
                    h.metadata.source = crate::history::Source::Lan;
                }
                session
            },
            seats,
        })
    }

    fn view(&self, player: PlayerId) -> legatum_core::PlayerView {
        let mut view = player_view(&self.session.state, player);
        legatum_core::diplomacy::with_pending(
            &mut view,
            self.seats.iter().map(|s| (s.player, s.orders.as_slice())),
        );
        view
    }

    fn seat_status(&self) -> Vec<SeatStatus> {
        self.seats
            .iter()
            .enumerate()
            .map(|(i, s)| SeatStatus {
                seat: i,
                player: s.player,
                name: s.name.clone(),
                faction: self.session.state.player(s.player).faction,
                connected: s.connection.is_some(),
                reserved: s.token.is_some(),
                ready: s.ready,
                orders: s.orders.len(),
            })
            .collect()
    }

    fn can_resolve(&self) -> bool {
        let mut living_seats = self
            .seats
            .iter()
            .filter(|s| self.session.state.player(s.player).alive);
        if self.session.state.finished {
            return false;
        }
        if living_seats.clone().next().is_none() {
            // Every human is eliminated: connected observers pace the remaining AI turns.
            let mut observers = self.seats.iter().filter(|s| s.connection.is_some());
            return observers.clone().next().is_some() && observers.all(|s| s.ready);
        }
        living_seats.clone().any(|s| s.connection.is_some())
            && living_seats.all(|s| s.joined && (s.connection.is_none() || s.ready))
    }
}

pub(crate) struct Host {
    game: Option<Game>,
    room: Option<Room>,
    connections: BTreeMap<ConnectionId, Connection>,
    next_connection: ConnectionId,
    checkpoint_turns: u32,
    /// The latest save of the live game, waiting for the browser to keep it (`take_save`).
    save: Option<String>,
    /// Game histories (archive recordings, JSON) waiting for the browser to keep them.
    recordings: Vec<String>,
}

impl Host {
    /// A host with no game, or one resuming `save` (the browser's copy of an earlier online
    /// game). Credentials are session-only: a resumed game's seats are locked until the host
    /// explicitly opens or reassigns them, so a restart never opens private empires to guests.
    pub(crate) fn new(save: Option<&str>, checkpoint_turns: u32) -> Result<Self, String> {
        if checkpoint_turns == 0 {
            return Err("checkpoint turns must be positive".into());
        }
        let mut game = save
            .map(|save| {
                check_state(&crate::json::parse_state(save)?)?;
                Game::from_session(Session::from_save_json(save, |_| BrainKind::Scripted)?)
            })
            .transpose()
            .map_err(|e| format!("cannot resume the saved online game: {e}"))?;
        if let Some(game) = &mut game {
            for seat in &mut game.seats {
                // Resuming must not re-arm the first-join barrier for absent players.
                seat.joined = true;
                seat.token = Some(credentials::token()?);
            }
        }
        let mut host = Self {
            game,
            room: None,
            connections: BTreeMap::new(),
            next_connection: 0,
            checkpoint_turns,
            save: None,
            recordings: Vec::new(),
        };
        host.checkpoint();
        Ok(host)
    }

    pub(crate) fn status(&self, is_local: bool) -> Status {
        Status {
            mode: "server",
            has_game: self.game.is_some(),
            turn: self.game.as_ref().map_or(0, |g| g.session.state.turn),
            seats: self.game.as_ref().map_or_else(Vec::new, Game::seat_status),
            is_local,
            staging: self.room.as_ref().map(|room| room.state(false)),
        }
    }

    /// The relay's lobby row. An open room is shown as staging even while an older game runs.
    pub(crate) fn summary(&self) -> Summary {
        if let Some(room) = &self.room {
            let seats = &room.config.seats;
            return Summary {
                players: seats.iter().filter(|s| s.kind != SeatKind::Closed).count(),
                humans: seats
                    .iter()
                    .filter(|s| matches!(s.kind, SeatKind::Host | SeatKind::Open))
                    .count(),
                open_seats: seats
                    .iter()
                    .zip(&room.occupants)
                    .filter(|(s, o)| s.kind == SeatKind::Open && o.is_none())
                    .count(),
                phase: "staging",
                turn: 0,
            };
        }
        match &self.game {
            Some(game) => Summary {
                players: game.session.state.players.len(),
                humans: game.seats.len(),
                // Unassigned seats a guest can claim with `game_take`.
                open_seats: game.seats.iter().filter(|s| s.token.is_none()).count(),
                phase: "playing",
                turn: game.session.state.turn,
            },
            None => Summary {
                players: 0,
                humans: 0,
                open_seats: 0,
                phase: "staging",
                turn: 0,
            },
        }
    }

    pub(crate) fn new_game(&mut self, setup: GameSetup) -> Result<(), String> {
        check_setup(&setup)?;
        let session = Session::try_new(&setup, |_| BrainKind::Scripted)?;
        self.replace_game(session)
    }

    pub(crate) fn load_game(&mut self, save: &str) -> Result<(), String> {
        check_state(&crate::json::parse_state(save)?)?;
        let mut session = Session::from_save_json(save, |_| BrainKind::Scripted)?;
        if let Some(h) = &mut session.history {
            h.fork();
        }
        self.replace_game(session)
    }

    fn replace_game(&mut self, session: Session) -> Result<(), String> {
        // Validate before replacing the live game or disconnecting anyone.
        let game = Game::from_session(session)?;
        let previous = self.preserve_current_history();
        for (_, connection) in std::mem::take(&mut self.connections) {
            connection
                .sender
                .close(None, "The host replaced the game. Please rejoin.");
        }
        if let Some(room) = &mut self.room {
            let connections: Vec<_> = room
                .occupants
                .iter()
                .flatten()
                .filter_map(|o| o.connection)
                .collect();
            for connection in connections {
                room.detach(connection);
            }
        }
        self.save = Some(game.session.save_json());
        self.game = Some(game);
        self.finish_replaced_history(previous);
        self.checkpoint();
        Ok(())
    }

    pub(crate) fn save_game(&self) -> Result<String, String> {
        let game = self.game.as_ref().ok_or("No game in progress")?;
        Ok(game.session.save_json())
    }

    /// Opens every seat but the first for guests to take, as in a game just hosted from a save file
    /// (a guest sees its saved name and takes the seat; the host's own seat 0 stays its own).
    pub(crate) fn open_guest_seats(&mut self) {
        if let Some(game) = &mut self.game {
            for seat in game.seats.iter_mut().skip(1) {
                seat.token = None;
                seat.ready = false;
                seat.orders.clear();
            }
            self.broadcast_seats();
        }
    }

    /// The live game's save, once per change, for the browser to keep.
    pub(crate) fn take_save(&mut self) -> Option<String> {
        self.save.take()
    }

    /// Game histories to keep in the browser's archive, oldest first.
    pub(crate) fn take_recordings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.recordings)
    }

    fn checkpoint(&mut self) {
        if let Some(history) = self.game.as_ref().and_then(|g| g.session.history.as_ref()) {
            let json = serde_json::to_string(history).expect("recordings serialize");
            self.recordings.push(json);
        }
    }

    fn preserve_current_history(&mut self) -> Option<crate::history::Recording> {
        let previous = self
            .game
            .as_ref()
            .and_then(|g| g.session.history.as_ref())
            .filter(|h| h.metadata.finished_at.is_none())
            .cloned();
        if let Some(history) = &previous {
            self.record(history);
        }
        previous
    }

    fn finish_replaced_history(&mut self, previous: Option<crate::history::Recording>) {
        if let Some(mut history) = previous {
            history.stop("abandoned");
            self.record(&history);
        }
    }

    fn record(&mut self, recording: &crate::history::Recording) {
        self.recordings
            .push(serde_json::to_string(recording).expect("recordings serialize"));
    }

    pub(crate) fn connect(
        &mut self,
        sender: outbox::Sender,
        local: bool,
    ) -> Result<ConnectionId, String> {
        if self.connections.len() >= outbox::MAX_CONNECTIONS {
            return Err("The server has reached its connection limit".into());
        }
        self.next_connection += 1;
        self.connections.insert(
            self.next_connection,
            Connection {
                sender,
                local,
                watching: false,
                show_seed: false,
            },
        );
        Ok(self.next_connection)
    }

    fn send(&mut self, connection: ConnectionId, message: &ServerMessage) {
        self.send_batch(connection, std::slice::from_ref(message));
    }

    fn send_batch(&mut self, connection: ConnectionId, messages: &[ServerMessage]) {
        if let Some(sender) = self.connections.get(&connection)
            && sender.sender.send(messages).is_err()
        {
            if let Some(sender) = self.connections.remove(&connection) {
                sender.sender.close(
                    Some("slow_consumer"),
                    "Outbound queue exceeded its limit. Please rejoin.",
                );
            }
            self.detach(connection);
        }
    }

    fn broadcast_seats(&self) {
        let snapshot: Arc<str> = serde_json::to_string(&ServerMessage::Seats {
            seats: self.game.as_ref().map_or_else(Vec::new, Game::seat_status),
        })
        .expect("protocol serializes")
        .into();
        for sender in self.connections.values() {
            sender.sender.seats(&snapshot);
        }
    }

    pub(crate) fn disconnect(&mut self, connection: ConnectionId) {
        self.connections.remove(&connection);
        self.detach(connection);
        // A previously joined, disconnected seat does not hold up a ready teammate.
        self.resolve_if_ready();
        self.broadcast_seats();
        self.broadcast_staging();
    }

    fn detach(&mut self, connection: ConnectionId) {
        if let Some(room) = &mut self.room {
            room.detach(connection);
        }
        if let Some(game) = &mut self.game
            && let Some(seat) = game
                .seats
                .iter_mut()
                .find(|s| s.connection == Some(connection))
        {
            seat.connection = None;
            seat.ready = false;
        }
    }

    pub(crate) fn message(
        &mut self,
        connection: ConnectionId,
        message: ClientMessage,
    ) -> Result<(), String> {
        let result = self.handle_message(connection, message);
        // Account for queue evictions only after completing the current operation; never
        // recursively resolve a turn from inside send_batch.
        self.resolve_if_ready();
        self.broadcast_seats();
        self.broadcast_staging();
        result
    }

    fn handle_message(
        &mut self,
        connection: ConnectionId,
        message: ClientMessage,
    ) -> Result<(), String> {
        // Taking over a seat removes the old id under the same lock used for orders. A
        // queued message or late disconnect from the old socket cannot affect its successor.
        if !self.connections.contains_key(&connection) {
            return Err("This connection has been replaced. Please rejoin.".into());
        }
        if matches!(message, ClientMessage::Ping) {
            self.send(connection, &ServerMessage::Pong);
            return Ok(());
        }
        if matches!(
            message,
            ClientMessage::StagingWatch { .. }
                | ClientMessage::StagingTake { .. }
                | ClientMessage::StagingRelease { .. }
                | ClientMessage::StagingUpdate { .. }
                | ClientMessage::StagingReady { .. }
                | ClientMessage::StagingLeave
        ) {
            self.staging_message(connection, message);
            return Ok(());
        }
        let game = self.game.as_mut().ok_or("No game in progress")?;
        if let ClientMessage::Orders { game_id, turn, .. }
        | ClientMessage::Ready { game_id, turn, .. } = &message
            && (game_id != &game.id || *turn != game.session.state.turn)
        {
            return Err(
                "This frame is for a different game or planning turn. Refresh your view.".into(),
            );
        }
        if let ClientMessage::GameRelease { seat } = message {
            if !self.connections[&connection].local {
                return Err("Only the loopback host can open a seat.".into());
            }
            let s = game.seats.get_mut(seat).ok_or("Choose a human seat")?;
            // Check under the game mutex: a reconnect can win the race with confirmation.
            if s.connection.is_some() {
                return Err(
                    "That player is connected. Only disconnected seats can be opened.".into(),
                );
            }
            s.token = None;
            s.ready = false;
            s.orders.clear();
            return Ok(());
        }
        if let ClientMessage::GameTake { seat } = message {
            let local = self.connections[&connection].local;
            let s = game.seats.get_mut(seat).ok_or("Choose a human seat")?;
            if !local && (s.token.is_some() || seat == 0) {
                return Err("This seat is reserved. Use its reconnect credential or ask the host to open this seat.".into());
            }
            let token = crate::host::credentials::token()?;
            let previous = s.connection.take();
            if let Some(old) = previous
                && old != connection
                && let Some(old) = self.connections.remove(&old)
            {
                old.sender.close(
                    Some("seat_taken"),
                    "Your seat was claimed by another connection.",
                );
            }
            s.token = Some(token.clone());
            s.ready = false;
            s.orders.clear();
            let game_id = game.id.clone();
            if let Some(old) = previous.filter(|old| *old != connection) {
                self.detach(old);
            }
            self.send(
                connection,
                &ServerMessage::SeatToken {
                    seat,
                    game_id,
                    token,
                },
            );
            return Ok(());
        }
        if let ClientMessage::Join { seat, name, token } = message {
            if seat >= game.seats.len() {
                return Err(format!(
                    "Choose a human seat from 0 to {}",
                    game.seats.len() - 1
                ));
            }
            if game
                .seats
                .iter()
                .enumerate()
                .any(|(i, s)| i != seat && s.connection == Some(connection))
            {
                return Err("This browser already owns a seat; reconnect to change seats".into());
            }
            let s = &mut game.seats[seat];
            if !crate::host::credentials::matches(s.token.as_deref(), Some(token.as_str())) {
                self.send(
                    connection,
                    &ServerMessage::Error {
                        code: Some("invalid_seat_credential"),
                        message: "Invalid seat credential. Ask the host to open this seat if it was lost.".into(),
                    },
                );
                return Ok(());
            }
            let cleaned = clean_name(&name);
            let turn = game.session.state.turn;
            // A repeat join on the connection that already holds this seat this turn changes
            // nothing, so it is not worth a full view and a save.
            if s.connection == Some(connection)
                && s.joined
                && s.joined_at == Some((connection, turn))
                && (cleaned.is_empty() || cleaned == s.name)
            {
                return Ok(());
            }
            if let Some(old) = s.connection
                && old != connection
                && let Some(sender) = self.connections.remove(&old)
            {
                sender.sender.close(
                    Some("seat_taken"),
                    "Your seat was claimed by another connection.",
                );
            }
            let mut renamed = false;
            if !cleaned.is_empty() {
                renamed = s.name != cleaned;
                s.name = cleaned.clone();
                if !game.session.state.finished
                    && let Some(history) = &mut game.session.history
                {
                    // Display names are metadata; changing them must not change replay hashes.
                    let stored = &mut history.metadata.seats[s.player.0 as usize].name;
                    if *stored != cleaned {
                        *stored = cleaned;
                        renamed = true;
                    }
                }
            }
            s.connection = Some(connection);
            s.joined = true;
            s.joined_at = Some((connection, turn));
            s.ready = false;
            let welcome = ServerMessage::Welcome {
                seat,
                game_id: game.id.clone(),
                orders: s.orders.clone(),
            };
            let player = s.player;
            let view = ServerMessage::View {
                view: Box::new(game.view(player)),
            };
            // The joined name is part of the saved history; save only when it changed.
            if renamed {
                self.save = Some(game.session.save_json());
            }
            self.send_batch(connection, &[welcome, view]);
            // A game takeover may also displace a staging watcher/occupant.
            if let Some(room) = &mut self.room {
                let gone: Vec<_> = room
                    .occupants
                    .iter()
                    .flatten()
                    .filter_map(|o| o.connection)
                    .filter(|c| !self.connections.contains_key(c))
                    .collect();
                for c in gone {
                    room.detach(c);
                }
            }
            return Ok(());
        }
        let seat = game
            .seats
            .iter_mut()
            .find(|s| s.connection == Some(connection))
            .ok_or("Join a human seat first")?;
        match message {
            ClientMessage::Orders { orders, .. } => {
                if game.session.state.finished {
                    return Err("The game is finished".into());
                }
                if orders.len() > MAX_STAGED_ORDERS {
                    let errors = vec![legatum_core::OrderError {
                        index: MAX_STAGED_ORDERS,
                        message: format!("Too many orders (at most {MAX_STAGED_ORDERS})"),
                    }];
                    self.send(connection, &ServerMessage::Validation { errors });
                    return Ok(());
                }
                let mut errors = validate_orders(&game.session.state, seat.player, &orders);
                errors.truncate(MAX_VALIDATION_ERRORS);
                let old_diplomacy: Vec<_> = seat
                    .orders
                    .iter()
                    .filter(|o| legatum_core::diplomacy::is_diplomatic(o))
                    .collect();
                let new_diplomacy: Vec<_> = orders
                    .iter()
                    .filter(|o| legatum_core::diplomacy::is_diplomatic(o))
                    .collect();
                let changed = old_diplomacy != new_diplomacy;
                seat.orders = orders;
                seat.ready = false;
                let team = game.session.state.team_of(seat.player);
                let updates: Vec<_> = game
                    .seats
                    .iter()
                    .filter(|s| {
                        changed
                            && s.connection != Some(connection)
                            && game.session.state.team_of(s.player) == team
                    })
                    .filter_map(|s| {
                        s.connection.map(|c| {
                            (
                                c,
                                ServerMessage::View {
                                    view: Box::new(game.view(s.player)),
                                },
                            )
                        })
                    })
                    .collect();
                self.send(connection, &ServerMessage::Validation { errors });
                for (connection, view) in updates {
                    self.send(connection, &view);
                }
            }
            ClientMessage::Ready { ready, .. } => {
                if game.session.state.finished {
                    return Err("The game is finished".into());
                }
                seat.ready = ready;
            }
            ClientMessage::Forecast {
                req_id,
                unit,
                target,
                mode,
            } => {
                let view = player_view(&game.session.state, seat.player);
                let forecast = forecast_attack_mode(&view, unit, target, mode);
                self.send(connection, &ServerMessage::Forecast { req_id, forecast });
            }
            ClientMessage::Assault {
                req_id,
                unit,
                target,
                mode,
                alongside,
            } => {
                let view = player_view(&game.session.state, seat.player);
                let assault = forecast_city_assault(&view, unit, target, mode, &alongside);
                self.send(connection, &ServerMessage::Assault { req_id, assault });
            }
            _ => unreachable!("join, ping and staging handled above"),
        }
        Ok(())
    }

    fn resolve_if_ready(&mut self) {
        let Some(game) = &mut self.game else { return };
        if !game.can_resolve() {
            return;
        }
        let before: Vec<_> = game
            .seats
            .iter()
            .map(|s| player_view(&game.session.state, s.player))
            .collect();
        let mut orders = game.session.ai_orders();
        for seat in &mut game.seats {
            orders.insert(seat.player, std::mem::take(&mut seat.orders));
            seat.ready = false;
        }
        let events = game.session.step(&orders);
        let state = &game.session.state;
        self.save = Some(game.session.save_json());
        if let Some(history) = game.session.history.as_ref().filter(|h| {
            state.finished || h.metadata.turn_count.is_multiple_of(self.checkpoint_turns)
        }) {
            self.recordings
                .push(serde_json::to_string(history).expect("recordings serialize"));
        }
        let hash = state.hash().to_string();
        let mut messages = Vec::new();
        for (seat, before) in game.seats.iter().zip(&before) {
            if let Some(connection) = seat.connection {
                let view = player_view(state, seat.player);
                messages.push((
                    connection,
                    [
                        ServerMessage::Turn {
                            turn: state.turn,
                            events: visible_events(&events, before, &view),
                            hash: hash.clone(),
                            finished: state.finished,
                            winner: state.winner,
                        },
                        ServerMessage::View {
                            view: Box::new(view),
                        },
                    ],
                ));
            }
        }
        for (connection, message) in messages {
            self.send_batch(connection, &message);
        }
    }

    pub(crate) fn configure_staging(&mut self, config: StagingConfig) -> Result<(), String> {
        config.validate()?;
        let lost = if let Some(room) = &mut self.room {
            room.configure(config)
        } else {
            let id = format!("room-{}", now_nanos());
            self.room = Some(Room::new(id, config));
            vec![]
        };
        for connection in lost {
            self.send(
                connection,
                &ServerMessage::StagingSeatLost {
                    message: "The host changed your seat.".into(),
                },
            );
        }
        self.broadcast_staging();
        Ok(())
    }

    fn broadcast_staging(&self) {
        let snapshot = |host| -> Option<Arc<str>> {
            self.room.as_ref().map(|room| {
                serde_json::to_string(&ServerMessage::Staging {
                    state: room.state(host),
                })
                .expect("protocol serializes")
                .into()
            })
        };
        let public = snapshot(false);
        let private = snapshot(true);
        for connection in self.connections.values().filter(|c| c.watching) {
            connection.sender.staging(if connection.show_seed {
                private.as_ref()
            } else {
                public.as_ref()
            });
        }
        self.broadcast_admission();
    }

    fn broadcast_admission(&self) {
        for (&id, connection) in &self.connections {
            let admitted = self
                .game
                .as_ref()
                .is_some_and(|g| g.seats.iter().any(|s| s.connection == Some(id)))
                || self.room.as_ref().is_some_and(|r| {
                    r.occupants
                        .iter()
                        .flatten()
                        .any(|o| o.connection == Some(id))
                });
            connection.sender.admitted(admitted);
        }
    }

    pub(crate) fn close_staging(&mut self) {
        self.room = None;
        self.broadcast_staging(); // Clear obsolete snapshots before the terminal delivery.
        let watchers: Vec<_> = self
            .connections
            .iter()
            .filter(|(_, c)| c.watching)
            .map(|(&id, _)| id)
            .collect();
        for id in watchers {
            self.send(id, &ServerMessage::StagingClosed);
        }
    }

    pub(crate) fn staging_can_start(&self) -> Result<(), RoomError> {
        self.room.as_ref().ok_or(RoomError::NO_ROOM)?.can_start()
    }

    pub(crate) fn start_staging(&mut self) -> Result<(), String> {
        let room = self.room.as_ref().ok_or("No game room is open")?;
        room.can_start().map_err(|e| e.message)?;
        let setup = room.setup();
        check_setup(&setup)?;
        let mut game = Game::from_session(Session::try_new(&setup, |_| BrainKind::Scripted)?)?;
        // Bind every human seat before committing or notifying any socket. Even an absent
        // host stays reserved; only an explicit loopback reassignment can recover that seat.
        for (seat, slot) in game.seats.iter_mut().zip(room.player_slots()) {
            seat.token = Some(match &room.occupants[slot] {
                Some(occupant) => occupant.token.clone(),
                None => crate::host::credentials::token()?,
            });
        }
        let previous = self.preserve_current_history();
        self.save = Some(game.session.save_json());
        let room = self.room.take().expect("room exists");
        let started: Vec<_> = room
            .player_slots()
            .into_iter()
            .enumerate()
            .filter_map(|(seat, slot)| room.occupants[slot].as_ref()?.connection.map(|c| (c, seat)))
            .collect();
        let game_id = game.id.clone();
        self.game = Some(game);
        self.finish_replaced_history(previous);
        self.checkpoint();
        // Watchers must survive the game replacement to receive their handoff. Existing
        // game-only connections retain the established replacement/reconnect behavior.
        let old: Vec<_> = self
            .connections
            .iter()
            .filter(|(_, c)| !c.watching)
            .map(|(&id, _)| id)
            .collect();
        for id in old {
            if let Some(c) = self.connections.remove(&id) {
                c.sender
                    .close(None, "The host replaced the game. Please rejoin.");
            }
        }
        self.broadcast_staging();
        for (connection, seat) in started {
            self.send(
                connection,
                &ServerMessage::StagingStarted {
                    game_id: game_id.clone(),
                    seat,
                    token: self.game.as_ref().expect("committed game").seats[seat]
                        .token
                        .clone()
                        .expect("reserved seat"),
                },
            );
        }
        self.broadcast_seats();
        Ok(())
    }

    fn staging_message(&mut self, connection: ConnectionId, message: ClientMessage) {
        // Recover a handoff missed during reload; the secret identifies the compacted seat.
        if self.room.is_none()
            && let ClientMessage::StagingTake {
                token: Some(token), ..
            } = &message
            && let Some(game) = &self.game
            && let Some(seat) = game
                .seats
                .iter()
                .position(|s| crate::host::credentials::matches(s.token.as_deref(), Some(token)))
        {
            let message = ServerMessage::StagingStarted {
                game_id: game.id.clone(),
                seat,
                token: token.clone(),
            };
            self.send(connection, &message);
            return;
        }
        let client = self
            .connections
            .get_mut(&connection)
            .expect("live connection");
        if let ClientMessage::StagingWatch { host } = message {
            // Privileged room data is opt-in and requires the TCP loopback identity.
            client.show_seed = host && client.local;
            client.watching = true;
            // A repeated watch still gets a snapshot, even if nothing has changed.
            client.sender.staging(None);
            return;
        }
        let result = if let Some(room) = &mut self.room {
            match message {
                ClientMessage::StagingTake { seat, name, token } => {
                    match room.take(connection, client.local, seat, &name, token.as_deref()) {
                        Ok((token, old)) => {
                            client.watching = true;
                            client.sender.staging(None);
                            let room_id = room.id.clone();
                            if let Some(old) = old {
                                if let Some(client) = self.connections.remove(&old) {
                                    client.sender.close(
                                        Some("seat_taken"),
                                        "Your seat was claimed by another connection.",
                                    );
                                }
                                self.detach(old);
                            }
                            self.send(
                                connection,
                                &ServerMessage::StagingTaken {
                                    room_id,
                                    seat,
                                    token,
                                },
                            );
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                }
                ClientMessage::StagingUpdate { name, faction } => {
                    room.update(connection, name, faction);
                    Ok(())
                }
                ClientMessage::StagingRelease { seat } => room.release(client.local, seat),
                ClientMessage::StagingReady { ready } => {
                    room.ready(connection, ready);
                    Ok(())
                }
                ClientMessage::StagingLeave => {
                    room.leave(connection);
                    let room_id = room.id.clone();
                    self.send(connection, &ServerMessage::StagingLeft { room_id });
                    Ok(())
                }
                _ => unreachable!("staging messages only"),
            }
        } else {
            Err(RoomError::NO_ROOM)
        };
        if let Err(e) = result {
            self.send(
                connection,
                &ServerMessage::Error {
                    code: Some(e.code),
                    message: e.message.into(),
                },
            );
        }
    }
}

fn check_setup(setup: &GameSetup) -> Result<(), String> {
    legatum_core::validation::check_setup(setup)?;
    let humans = setup.players.iter().take_while(|p| p.is_human).count();
    if !(2..=8).contains(&setup.players.len())
        || humans == 0
        || setup.players[humans..].iter().any(|p| p.is_human)
    {
        return Err("Online games require 2-8 players with 1-8 humans in the first seats".into());
    }
    check_dimensions(setup.width, setup.height, setup.turn_limit)
}

pub(crate) fn check_dimensions(
    width: i32,
    height: i32,
    turn_limit: Option<u32>,
) -> Result<(), String> {
    if !(8..=128).contains(&width)
        || !(8..=128).contains(&height)
        || turn_limit.is_some_and(|limit| limit == 0 || limit == u32::MAX)
    {
        return Err("Invalid map dimensions or turn limit".into());
    }
    Ok(())
}

pub(crate) fn check_state(state: &GameState) -> Result<(), String> {
    legatum_core::validation::check_state(state)?;
    check_setup(&state.setup)?;
    // Exercise the same read path before accepting the save (also catches malformed bitsets).
    for p in &state.players {
        player_view(state, p.id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;
    use legatum_core::Faction;
    use serde_json::Value;

    /// Everything a connection would write now, parsed.
    fn drain(receiver: &outbox::Receiver) -> Vec<Value> {
        receiver
            .drain()
            .texts
            .iter()
            .map(|t| serde_json::from_str(t).unwrap())
            .collect()
    }

    /// As `drain`, without the seat snapshots every connection also gets.
    fn drain_room(receiver: &outbox::Receiver) -> Vec<Value> {
        let mut written = drain(receiver);
        written.retain(|m| m["type"] != "seats");
        written
    }

    #[test]
    fn online_history_is_handed_over_with_orders_names_and_hashes_across_restart() {
        let mut setup = GameSetup::standard(72, [Faction::India, Faction::Byzantium]);
        setup.turn_limit = Some(4);
        let mut host = Host::new(None, 2).unwrap();
        host.new_game(setup).unwrap();
        let (a, _a) = peer(&mut host);
        let (b, _b) = peer(&mut host);
        join(&mut host, a, 0);
        join(&mut host, b, 1);
        host.take_recordings();
        for _ in 0..2 {
            host.message(a, ready(&host, true)).unwrap();
            host.message(b, ready(&host, true)).unwrap();
        }
        // A checkpoint every second turn: none after turn 1, one after turn 2.
        let checkpoints = host.take_recordings();
        assert_eq!(checkpoints.len(), 1);
        let checkpoint: crate::history::Recording = serde_json::from_str(&checkpoints[0]).unwrap();
        assert_eq!(checkpoint.tape.turns.len(), 2);
        assert_eq!(checkpoint.metadata.source, crate::history::Source::Lan);
        let save = host.take_save().expect("the turn was saved");
        assert!(host.take_save().is_none(), "a save is handed over once");
        drop(host);

        let mut host = Host::new(Some(&save), 2).unwrap();
        // Resuming hands the history over again, unchanged.
        assert_eq!(host.take_recordings(), checkpoints);
        let (a, _a) = peer(&mut host);
        let (b, _b) = peer(&mut host);
        join(&mut host, a, 0);
        join(&mut host, b, 1);
        for _ in 0..2 {
            host.message(a, ready(&host, true)).unwrap();
            host.message(b, ready(&host, true)).unwrap();
        }
        let session = &host.game.as_ref().unwrap().session;
        let last = host.recordings.last().unwrap();
        let stored: crate::history::Recording = serde_json::from_str(last).unwrap();
        assert_eq!(stored.metadata.game_id, checkpoint.metadata.game_id);
        assert_eq!(stored.metadata.seats[0].name, "Human 0");
        assert!(stored.metadata.finished_at.is_some());
        assert_eq!(stored.tape.turns.len(), 4);
        assert_eq!(stored.replay().unwrap().hash(), session.state.hash());
    }

    #[test]
    fn replacing_an_unfinished_game_hands_over_its_history_as_abandoned() {
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        host.take_recordings();
        host.new_game(GameSetup::standard(8, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let outcomes: Vec<String> = host
            .take_recordings()
            .iter()
            .map(|r| {
                serde_json::from_str::<crate::history::Recording>(r)
                    .unwrap()
                    .metadata
                    .outcome
            })
            .collect();
        // The old game as it stood, then stopped, then the new game's first checkpoint.
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[1], "abandoned");
    }

    fn room_config() -> StagingConfig {
        serde_json::from_value(serde_json::json!({
            "seed": "5", "width": 24, "height": 20, "turn_limit": 100,
            "seats": [
                {"kind":"host", "name":"Host", "faction":"Random", "team":0},
                {"kind":"open", "name":"Guest", "faction":"Random", "team":1}
            ]
        }))
        .unwrap()
    }

    #[test]
    fn staging_accepts_optional_limits_rejects_invalid_values_and_ignores_a_legacy_speed() {
        let legacy = room_config();
        assert_eq!(legacy.turn_limit, Some(100));
        for limit in [None, Some(1), Some(900), Some(u32::MAX - 1)] {
            let mut config = legacy.clone();
            config.turn_limit = limit;
            assert!(config.validate().is_ok());
            let room = Room::new("limit-test".into(), config);
            assert_eq!(room.setup().turn_limit, limit);
        }
        for limit in [Some(0), Some(u32::MAX)] {
            let mut config = legacy.clone();
            config.turn_limit = limit;
            assert!(config.validate().is_err());
        }
        // A lobby config saved before the game speed was removed still loads and drops the key.
        for speed in ["Standard", "Quick"] {
            let mut json = serde_json::to_value(&legacy).unwrap();
            assert!(json.get("speed").is_none());
            json["speed"] = serde_json::json!(speed);
            let loaded: StagingConfig = serde_json::from_value(json).unwrap();
            assert_eq!(loaded, legacy, "{speed}");
            assert!(
                serde_json::to_value(&loaded)
                    .unwrap()
                    .get("speed")
                    .is_none()
            );
        }
    }

    #[test]
    fn staging_leave_acknowledgment_survives_a_coalesced_seat_reclaim() {
        let mut host = Host::new(None, 5).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (guest, messages) = peer(&mut host);
        host.message(
            guest,
            ClientMessage::StagingTake {
                seat: 1,
                name: "Alice".into(),
                token: None,
            },
        )
        .unwrap();
        host.message(guest, ClientMessage::StagingLeave).unwrap();
        assert!(host.status(true).staging.unwrap().occupants[1].is_none());
        let (next, _next_messages) = peer(&mut host);
        host.message(
            next,
            ClientMessage::StagingTake {
                seat: 1,
                name: "Bob".into(),
                token: None,
            },
        )
        .unwrap();
        // Drained only now: the room snapshots in between coalesced into the latest one.
        let written = drain_room(&messages);
        assert_eq!(written[0]["type"], "staging_taken");
        assert_eq!(
            written[1],
            serde_json::json!({
                "type": "staging_left", "room_id": written[0]["room_id"],
            })
        );
        assert_eq!(written[2]["type"], "staging");
        assert_eq!(written[2]["state"]["occupants"][1]["name"], "Bob");
        assert_eq!(written.len(), 3);
    }

    #[test]
    fn staging_is_coalesced_and_eviction_disconnects_the_reservation() {
        let mut host = Host::new(None, 5).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (watcher, watching) = peer(&mut host);
        host.message(watcher, ClientMessage::StagingWatch { host: false })
            .unwrap();
        let (guest, guest_messages) = peer(&mut host);
        host.message(
            guest,
            ClientMessage::StagingTake {
                seat: 1,
                name: "Guest".into(),
                token: None,
            },
        )
        .unwrap();
        for i in 0..128 {
            host.message(guest, ClientMessage::StagingReady { ready: i % 2 != 0 })
                .unwrap();
        }
        let written = drain_room(&watching);
        assert_eq!(written.len(), 1, "only the latest room snapshot");
        assert_eq!(written[0]["state"]["occupants"][1]["ready"], true);
        host.message(guest, ClientMessage::StagingReady { ready: true })
            .unwrap();
        assert!(
            drain_room(&watching).is_empty(),
            "an identical snapshot is not rewritten"
        );
        // The private credential acknowledgment is the guest's one reliable delivery so far;
        // seventeen pongs nobody reads overflow the queue.
        for _ in 0..17 {
            let _ = host.message(guest, ClientMessage::Ping);
        }
        let state = host.status(true).staging.unwrap();
        let occupant = state.occupants[1].as_ref().unwrap();
        assert!(!occupant.connected);
        assert!(!occupant.ready);
        assert!(host.staging_can_start().is_err());
        let evicted = guest_messages.drain();
        assert!(evicted.closed);
        assert!(evicted.texts[0].contains("slow_consumer"));
        drain_room(&watching);
        host.close_staging();
        let written = drain_room(&watching);
        assert_eq!(written.len(), 1, "a closed room leaves no stale snapshot");
        assert_eq!(written[0]["type"], "staging_closed");
    }

    #[test]
    fn only_the_hosts_own_connection_can_take_the_host_slot() {
        let mut host = Host::new(None, 5).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (sender, remote_messages) = outbox::channel();
        let remote = host.connect(sender, false).unwrap();
        host.message(
            remote,
            ClientMessage::StagingTake {
                seat: 0,
                name: "Imposter".into(),
                token: None,
            },
        )
        .unwrap();
        assert_eq!(drain_room(&remote_messages)[0]["code"], "not_open");
        assert!(host.status(true).staging.unwrap().occupants[0].is_none());
        host.message(
            remote,
            ClientMessage::StagingTake {
                seat: 1,
                name: "Guest".into(),
                token: None,
            },
        )
        .unwrap();
        host.message(remote, ClientMessage::StagingReady { ready: true })
            .unwrap();
        let (local, local_messages) = peer(&mut host);
        host.message(
            local,
            ClientMessage::StagingTake {
                seat: 0,
                name: "Host".into(),
                token: None,
            },
        )
        .unwrap();
        assert_eq!(drain_room(&local_messages)[0]["type"], "staging_taken");
        host.start_staging().unwrap();
        let written = drain_room(&local_messages);
        assert_eq!(written.len(), 1, "no room snapshot after the start");
        assert_eq!(written[0]["type"], "staging_started");
        assert_eq!(written[0]["seat"], 0);
    }

    fn peer(host: &mut Host) -> (ConnectionId, outbox::Receiver) {
        let (sender, receiver) = outbox::channel();
        (host.connect(sender, true).unwrap(), receiver)
    }

    fn ready(host: &Host, ready: bool) -> ClientMessage {
        let game = host.game.as_ref().unwrap();
        ClientMessage::Ready {
            game_id: game.id.clone(),
            turn: game.session.state.turn,
            ready,
        }
    }

    fn join(host: &mut Host, id: ConnectionId, seat: usize) {
        let token = host.game.as_mut().unwrap().seats[seat]
            .token
            .get_or_insert_with(|| credentials::token().unwrap())
            .clone();
        host.message(
            id,
            ClientMessage::Join {
                seat,
                name: format!("Human {seat}"),
                token,
            },
        )
        .unwrap();
    }

    #[test]
    fn oversized_saved_collections_preserve_live_game_and_autosave() {
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let saved = host.save_game().unwrap();
        assert_eq!(host.take_save().as_ref(), Some(&saved));
        let original: Value = serde_json::from_str(&saved).unwrap();
        let tiles = original["map"]["tiles"].as_array().unwrap().len();
        let history_limit = legatum_core::data::SCIENCE_HISTORY_TURNS;
        for (field, replacement) in [
            (
                "science_history",
                serde_json::json!(vec![0; history_limit + 1]),
            ),
            (
                "tile_memory",
                serde_json::json!(vec![None::<u8>; tiles + 1]),
            ),
            (
                "explored",
                serde_json::json!({"len": tiles, "words": vec![0; tiles.div_ceil(64) + 1]}),
            ),
            (
                "explored",
                serde_json::json!({"len": tiles, "words": vec![0; tiles.div_ceil(64) - 1]}),
            ),
        ] {
            let mut invalid = original.clone();
            invalid["players"][0][field] = replacement;
            let result = host.load_game(&invalid.to_string());
            assert!(result.is_err(), "accepted invalid {field}");
            assert_eq!(host.save_game().unwrap(), saved);
            assert!(host.take_save().is_none(), "a refused load saved nothing");
            // Resuming must enforce the same bounds as loading.
            assert!(Host::new(Some(&invalid.to_string()), 5).is_err());
        }
        // Legacy saves may omit observations; shorter histories/memory remain supported.
        let mut legacy = original;
        // A pre-archive save has no recorded hashes for its edited legacy fields.
        legacy.as_object_mut().unwrap().remove("archive");
        legacy["players"][0]
            .as_object_mut()
            .unwrap()
            .remove("tile_memory");
        legacy["players"][0]["science_history"] = serde_json::json!(vec![0; history_limit]);
        host.load_game(&legacy.to_string()).unwrap();
    }

    #[test]
    fn slow_observers_keep_only_the_latest_seat_snapshot() {
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let (observer, messages) = peer(&mut host);
        let (human, _human_messages) = peer(&mut host);
        join(&mut host, human, 0);
        for i in 0..128 {
            host.message(human, ready(&host, i % 2 == 0)).unwrap();
        }
        assert!(host.connections.contains_key(&observer));
        let written = drain(&messages);
        assert_eq!(written.len(), 1, "seat updates entered the reliable queue");
        assert_eq!(written[0]["seats"][0]["ready"], false);
    }

    #[test]
    fn reliable_queue_overflow_evicts_without_waiting_for_the_reader() {
        let mut host = Host::new(None, 5).unwrap();
        let (slow, messages) = peer(&mut host);
        for _ in 0..17 {
            let _ = host.message(slow, ClientMessage::Ping);
        }
        assert!(
            !host.connections.contains_key(&slow),
            "full queue kept the connection"
        );
        let evicted = messages.drain();
        assert!(evicted.closed);
        assert_eq!(
            drain_texts_code(&evicted.texts),
            Some("slow_consumer".into())
        );
    }

    fn drain_texts_code(texts: &[String]) -> Option<String> {
        let value: Value = serde_json::from_str(texts.first()?).ok()?;
        value["code"].as_str().map(str::to_owned)
    }

    #[test]
    fn connections_have_a_fixed_cap() {
        let mut host = Host::new(None, 5).unwrap();
        let mut readers = Vec::new();
        for _ in 0..outbox::MAX_CONNECTIONS {
            let (_, receiver) = peer(&mut host);
            readers.push(receiver);
        }
        let (sender, _receiver) = outbox::channel();
        assert!(host.connect(sender, true).is_err());
        host.disconnect(1);
        let (sender, _receiver) = outbox::channel();
        assert!(host.connect(sender, true).is_ok());
    }

    fn two_seat_host() -> Host {
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        host
    }

    fn join_as(host: &mut Host, id: ConnectionId, seat: usize, name: &str) {
        let token = host.game.as_mut().unwrap().seats[seat]
            .token
            .get_or_insert_with(|| credentials::token().unwrap())
            .clone();
        host.message(
            id,
            ClientMessage::Join {
                seat,
                name: name.into(),
                token,
            },
        )
        .unwrap();
    }

    #[test]
    fn a_repeat_join_on_the_same_connection_is_answered_with_nothing_but_a_new_connection_still_joins()
     {
        let mut host = two_seat_host();
        let (a, receiver) = peer(&mut host);
        join_as(&mut host, a, 0, "Ada");
        assert!(drain(&receiver).iter().any(|m| m["type"] == "view"));
        assert!(host.take_save().is_some());
        for _ in 0..3 {
            join_as(&mut host, a, 0, "Ada");
        }
        let again = drain(&receiver);
        assert!(
            again
                .iter()
                .all(|m| m["type"] != "view" && m["type"] != "welcome")
        );
        assert!(host.take_save().is_none(), "a repeat join must not save");
        // A reconnect on a new connection works as before.
        let (b, receiver_b) = peer(&mut host);
        join_as(&mut host, b, 0, "Ada");
        let joined = drain(&receiver_b);
        assert!(joined.iter().any(|m| m["type"] == "welcome"));
        assert!(joined.iter().any(|m| m["type"] == "view"));
        assert!(
            host.take_save().is_none(),
            "an unchanged name needs no save"
        );
        // A new name on the same connection is still applied.
        join_as(&mut host, b, 0, "Grace");
        assert_eq!(host.game.as_ref().unwrap().seats[0].name, "Grace");
        assert!(host.take_save().is_some());
    }

    #[test]
    fn guest_names_are_cleaned_the_same_in_the_seat_history_and_staging() {
        assert_eq!(
            clean_name("  Ada \u{202E}Lovelace\u{200B}\n\t Byron "),
            "Ada Lovelace Byron"
        );
        assert_eq!(clean_name("\u{FEFF}\u{AD}\u{2066}\u{61C}"), "");
        assert_eq!(clean_name(&"é".repeat(500)).chars().count(), 80);
        let mut host = two_seat_host();
        let (a, _a) = peer(&mut host);
        let dirty = format!("\u{202E}  Eve\u{200B}   Hacker  {}", "x".repeat(70_000));
        join_as(&mut host, a, 0, &dirty);
        let game = host.game.as_ref().unwrap();
        let history = game.session.history.as_ref().unwrap();
        assert_eq!(game.seats[0].name, history.metadata.seats[0].name);
        assert_eq!(game.seats[0].name.chars().count(), 80);
        assert!(game.seats[0].name.starts_with("Eve Hacker xxx"));
        // A name that cleans to nothing keeps the seat's name.
        join_as(&mut host, a, 0, "\u{200B}\u{202E}");
        assert_eq!(
            host.game.as_ref().unwrap().seats[0].name.chars().count(),
            80
        );
        // Staging uses the same cleaning and its own empty-name fallback.
        let mut room = Room::new("names".into(), room_config());
        assert!(
            room.take(7, false, 1, "\u{200B}  Zed\u{202E} ", None)
                .is_ok()
        );
        assert_eq!(room.state(false).occupants[1].as_ref().unwrap().name, "Zed");
        room.update(7, Some("\u{200B}".into()), None);
        assert_eq!(
            room.state(false).occupants[1].as_ref().unwrap().name,
            "Player"
        );
    }

    #[test]
    fn staged_orders_are_capped_per_turn_and_validation_errors_are_capped() {
        let mut host = two_seat_host();
        let (a, receiver) = peer(&mut host);
        join_as(&mut host, a, 0, "Ada");
        drain(&receiver);
        let game_id = host.game.as_ref().unwrap().id.clone();
        let turn = host.game.as_ref().unwrap().session.state.turn;
        let bogus = |n: usize| {
            let orders: Vec<Value> = (0..n)
                .map(|_| serde_json::json!({"type": "DisbandUnit", "unit": 4_000_000}))
                .collect();
            serde_json::from_value::<ClientMessage>(serde_json::json!({
                "type": "orders", "game_id": game_id, "turn": turn, "orders": orders
            }))
            .unwrap()
        };
        host.message(a, bogus(MAX_STAGED_ORDERS + 1)).unwrap();
        let reply = drain(&receiver);
        let validation = reply.iter().find(|m| m["type"] == "validation").unwrap();
        assert_eq!(validation["errors"].as_array().unwrap().len(), 1);
        assert!(host.game.as_ref().unwrap().seats[0].orders.is_empty());
        // Within the limit the orders are kept as before, with the error list capped.
        host.message(a, bogus(500)).unwrap();
        let reply = drain(&receiver);
        let validation = reply.iter().find(|m| m["type"] == "validation").unwrap();
        assert_eq!(
            validation["errors"].as_array().unwrap().len(),
            MAX_VALIDATION_ERRORS
        );
        assert_eq!(host.game.as_ref().unwrap().seats[0].orders.len(), 500);
    }

    #[test]
    fn orders_and_ready_require_the_current_game_and_planning_turn() {
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let (a, _a_messages) = peer(&mut host);
        let (b, _b_messages) = peer(&mut host);
        join(&mut host, a, 0);
        join(&mut host, b, 1);
        let game_id = host.game.as_ref().unwrap().id.clone();
        let unit = host
            .game
            .as_ref()
            .unwrap()
            .session
            .state
            .units_of(PlayerId(0))
            .next()
            .unwrap()
            .id;
        let decode = |v: Value| serde_json::from_value::<ClientMessage>(v).unwrap();
        for connection in [a, b] {
            host.message(connection, decode(serde_json::json!({"type": "ready", "game_id": game_id, "turn": 1, "ready": true}))).unwrap();
        }
        assert_eq!(host.status(true).turn, 2);
        let pending = vec![Order::SkipUnit { unit }];
        host.message(a, decode(serde_json::json!({"type": "orders", "game_id": game_id, "turn": 2, "orders": pending}))).unwrap();
        host.message(a, ready(&host, true)).unwrap();
        let saved = host.save_game().unwrap();
        for (game, turn) in [(&game_id, 1), (&game_id, 3), (&"other-game".to_string(), 2)] {
            for frame in [
                serde_json::json!({"type": "orders", "game_id": game, "turn": turn, "orders": [{"type": "DisbandUnit", "unit": unit}]}),
                serde_json::json!({"type": "ready", "game_id": game, "turn": turn, "ready": true}),
                serde_json::json!({"type": "ready", "game_id": game, "turn": turn, "ready": false}),
            ] {
                assert!(
                    host.message(a, decode(frame)).is_err(),
                    "accepted wrong game/turn"
                );
                assert_eq!(host.game.as_ref().unwrap().seats[0].orders, pending);
                assert!(host.status(true).seats[0].ready);
                assert_eq!(host.save_game().unwrap(), saved);
            }
        }
        for frame in [
            serde_json::json!({"type": "orders", "orders": []}),
            serde_json::json!({"type": "ready", "ready": true}),
            serde_json::json!({"type": "ready", "game_id": game_id, "ready": true}),
            serde_json::json!({"type": "ready", "turn": 2, "ready": true}),
        ] {
            assert!(serde_json::from_value::<ClientMessage>(frame).is_err());
        }
        host.message(a, decode(serde_json::json!({"type": "orders", "game_id": game_id, "turn": 2, "orders": [{"type": "SkipUnit", "unit": unit}]}))).unwrap();
        assert_eq!(host.status(true).seats[0].orders, 1);
    }

    #[test]
    fn stale_connection_cannot_submit_or_disconnect_replacement_and_save_resumes_identically() {
        let setup = GameSetup::standard(u64::MAX - 7, [Faction::India, Faction::Byzantium]);
        let mut host = Host::new(None, 5).unwrap();
        host.new_game(setup.clone()).unwrap();
        let (a, a_messages) = peer(&mut host);
        join(&mut host, a, 0);
        host.message(a, ready(&host, true)).unwrap();
        assert_eq!(host.status(true).turn, 1, "seat 1 has never joined");
        let (b, _b_messages) = peer(&mut host);
        join(&mut host, b, 1);
        let (replacement, messages) = peer(&mut host);
        join(&mut host, replacement, 0);
        let displaced = a_messages.drain();
        assert!(displaced.closed);
        assert_eq!(
            serde_json::from_str::<Value>(&displaced.texts[0]).unwrap(),
            serde_json::json!({
                "type": "error",
                "code": "seat_taken",
                "message": "Your seat was claimed by another connection."
            })
        );
        assert!(host.message(a, ready(&host, true)).is_err());
        host.disconnect(a);
        assert!(host.status(true).seats[0].connected);
        assert!(!host.status(true).seats[0].ready);
        host.message(b, ready(&host, true)).unwrap();
        assert_eq!(host.status(true).turn, 1);
        host.message(replacement, ready(&host, true)).unwrap();
        assert_eq!(host.status(true).turn, 2);
        let mut native = Session::new(&setup, |_| BrainKind::Scripted);
        let orders = native.ai_orders();
        native.step(&orders);
        assert_eq!(
            json::parse_state(&host.save_game().unwrap()).unwrap(),
            native.state
        );
        let written = drain(&messages);
        let at = written
            .iter()
            .position(|m| m["type"] == "turn")
            .expect("a turn report");
        assert_eq!(written[at]["turn"], 2);
        assert_eq!(written[at]["hash"], native.state.hash().to_string());
        assert_eq!(written[at + 1]["type"], "view");
        assert_eq!(written[at + 1]["view"]["me"], 0);
        assert_eq!(written[at + 1]["view"]["turn"], 2);
        let save = host.take_save().unwrap();
        assert_eq!(save, host.save_game().unwrap());
        let mut restored = Host::new(Some(&save), 5).unwrap();
        assert_eq!(restored.save_game().unwrap(), save);
        let (a, _a_messages) = peer(&mut restored);
        let (b, _b_messages) = peer(&mut restored);
        join(&mut restored, a, 0);
        join(&mut restored, b, 1);
        restored.message(a, ready(&restored, true)).unwrap();
        restored.disconnect(b);
        assert_eq!(restored.status(true).turn, 3);
        let orders = native.ai_orders();
        native.step(&orders);
        assert_eq!(
            json::parse_state(&restored.save_game().unwrap()).unwrap(),
            native.state
        );
    }
}

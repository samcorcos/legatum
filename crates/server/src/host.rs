use crate::{
    events::visible_events,
    outbox,
    protocol::{ClientMessage, SeatStatus, ServerMessage, Status},
    saves::Saves,
    staging::{Room, RoomError, StagingConfig},
};
use legatum_core::{
    GameSetup, GameState, Order, PlayerId, forecast_attack_mode, forecast_city_assault,
    player_view, validate_orders,
};
use legatum_wasm::session::{BrainKind, Session};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_SCENARIO_RUN_FILES: usize = 500;
const MAX_SCENARIO_RUN_BYTES: u64 = 100 * 1024 * 1024;

pub(crate) type ConnectionId = u64;

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
}

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
            })
            .collect();
        Ok(Self {
            id: format!(
                "{}-{}",
                state.hash(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            session: {
                if let Some(h) = &mut session.history {
                    h.metadata.source = legatum_archive::Source::Lan;
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
    saves: Saves,
    scenario_runs_dir: PathBuf,
    lan_urls: Vec<String>,
    connections: BTreeMap<ConnectionId, Connection>,
    next_connection: ConnectionId,
    /// `None` when the archive folder could not be opened: the game runs without a record.
    archive: Option<legatum_archive::Archive>,
    checkpoint_turns: u32,
}

impl Host {
    #[cfg(test)]
    pub(crate) fn open(saves_dir: PathBuf, lan_urls: Vec<String>) -> Result<Self, String> {
        let archive = legatum_archive::Archive::open(saves_dir.join("archive"))?;
        Self::open_with_archive(saves_dir, lan_urls, archive, 5)
    }

    pub(crate) fn open_with_archive(
        saves_dir: PathBuf,
        lan_urls: Vec<String>,
        archive: impl Into<Option<legatum_archive::Archive>>,
        checkpoint_turns: u32,
    ) -> Result<Self, String> {
        let archive = archive.into();
        if checkpoint_turns == 0 {
            return Err("checkpoint turns must be positive".into());
        }
        let scenario_runs_dir = saves_dir.join("scenarios");
        let saves =
            Saves::open(saves_dir).map_err(|e| format!("cannot open saves directory: {e}"))?;
        let mut game = saves.load_session()?.map(Game::from_session).transpose()?;
        // Credentials are session-only. A restart locks restored seats until the host
        // explicitly reassigns them; it never silently opens private empires to the LAN.
        if let Some(game) = &mut game {
            for seat in &mut game.seats {
                // Resuming must not re-arm the first-join barrier for absent players.
                seat.joined = true;
                seat.token = Some(crate::credentials::token()?);
            }
        }
        let host = Self {
            game,
            room: None,
            saves,
            scenario_runs_dir,
            lan_urls,
            connections: BTreeMap::new(),
            next_connection: 0,
            archive,
            checkpoint_turns,
        };
        // Retry a finish whose file/index write was interrupted before shutdown.
        host.checkpoint()?;
        Ok(host)
    }

    pub(crate) fn status(&self, is_local: bool) -> Status {
        Status {
            mode: "server",
            has_game: self.game.is_some(),
            turn: self.game.as_ref().map_or(0, |g| g.session.state.turn),
            seats: self.game.as_ref().map_or_else(Vec::new, Game::seat_status),
            lan_urls: self.lan_urls.clone(),
            is_local,
            staging: self.room.as_ref().map(|room| room.state(false)),
        }
    }

    /// The relay's lobby row. An open room is shown as staging even while an older game runs.
    pub(crate) fn summary(&self) -> crate::relay::Summary {
        use crate::staging::SeatKind;
        if let Some(room) = &self.room {
            let seats = &room.config.seats;
            return crate::relay::Summary {
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
            Some(game) => crate::relay::Summary {
                players: game.session.state.players.len(),
                humans: game.seats.len(),
                // Unassigned seats a guest can claim with `game_take`.
                open_seats: game.seats.iter().filter(|s| s.token.is_none()).count(),
                phase: "playing",
                turn: game.session.state.turn,
            },
            None => crate::relay::Summary {
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
        check_state(&legatum_wasm::json::parse_state(save)?)?;
        let mut session = Session::from_save_json(save, |_| BrainKind::Scripted)?;
        if let Some(h) = &mut session.history {
            h.fork();
        }
        self.replace_game(session)
    }

    fn replace_game(&mut self, session: Session) -> Result<(), String> {
        // Validate and persist before replacing the live game or disconnecting anyone.
        let game = Game::from_session(session)?;
        let previous = self.preserve_current_history()?;
        self.saves
            .write_session(&game.session)
            .map_err(|e| format!("cannot save game: {e}"))?;
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
        self.game = Some(game);
        self.finish_replaced_history(previous);
        if let Err(error) = self.checkpoint() {
            self.archive_warning(error);
        }
        Ok(())
    }

    pub(crate) fn save_game(&self) -> Result<String, String> {
        let game = self.game.as_ref().ok_or("No game in progress")?;
        Ok(game.session.save_json())
    }

    fn checkpoint(&self) -> Result<(), String> {
        if let Some(history) = self.game.as_ref().and_then(|g| g.session.history.as_ref()) {
            self.record(history)?;
        }
        Ok(())
    }

    fn preserve_current_history(&self) -> Result<Option<legatum_archive::Recording>, String> {
        let previous = self
            .game
            .as_ref()
            .and_then(|g| g.session.history.as_ref())
            .filter(|h| h.metadata.finished_at.is_none())
            .cloned();
        if let Some(history) = &previous {
            self.record(history)?;
        }
        Ok(previous)
    }

    fn finish_replaced_history(&mut self, previous: Option<legatum_archive::Recording>) {
        if let Some(mut history) = previous {
            history.stop("abandoned");
            if let Err(error) = self.record(&history) {
                self.archive_warning(error);
            }
        }
    }

    fn archive_warning(&mut self, error: String) {
        let message = format!("Game saved, but archive write failed: {error}");
        eprintln!("{message}");
        self.broadcast(&ServerMessage::Error {
            code: None,
            message,
        });
    }

    /// Write a recording to the archive, or do nothing when this host runs without one.
    fn record(&self, recording: &legatum_archive::Recording) -> Result<(), String> {
        record_in(self.archive.as_ref(), recording)
    }

    pub(crate) fn archive_authorized(&self, token: &str) -> bool {
        let token = Some(token).filter(|t| !t.is_empty());
        self.game.as_ref().is_some_and(|g| {
            g.seats
                .iter()
                .any(|s| crate::credentials::matches(s.token.as_deref(), token))
        }) || self.room.as_ref().is_some_and(|r| {
            r.occupants
                .iter()
                .flatten()
                .any(|o| crate::credentials::matches(Some(&o.token), token))
        })
    }

    /// Save a finished scenario run (format "legatum-scenario-run") into the saves directory.
    pub(crate) fn save_scenario_run(&self, json: &str) -> Result<PathBuf, String> {
        let name = scenario_run_name(json)?;
        prune_scenario_runs(&self.scenario_runs_dir, &name, json.len() as u64)
            .map_err(|e| format!("Could not prune scenario runs: {e}"))?;
        self.saves
            .write_scenario_run(&name, json)
            .map_err(|e| format!("Could not save the scenario run: {e}"))
    }

    pub(crate) fn upload_archive(
        &self,
        recording: &legatum_archive::Recording,
    ) -> Result<(), String> {
        self.record(recording)
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

    fn broadcast(&mut self, message: &ServerMessage) {
        for connection in self.connections.keys().copied().collect::<Vec<_>>() {
            self.send(connection, message);
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
            let token = crate::credentials::token()?;
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
            if !crate::credentials::matches(s.token.as_deref(), Some(token.as_str())) {
                self.send(
                    connection,
                    &ServerMessage::Error {
                        code: Some("invalid_seat_credential"),
                        message: "Invalid seat credential. Ask the host to open this seat if it was lost.".into(),
                    },
                );
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
            if !name.trim().is_empty() {
                s.name = name.trim().chars().take(80).collect();
                if !game.session.state.finished
                    && let Some(history) = &mut game.session.history
                {
                    // Display names are metadata; changing them must not change replay hashes.
                    history.metadata.seats[s.player.0 as usize].name = name;
                }
            }
            s.connection = Some(connection);
            s.joined = true;
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
            self.saves
                .write_session(&game.session)
                .map_err(|e| format!("cannot save joined player: {e}"))?;
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
                let errors = validate_orders(&game.session.state, seat.player, &orders);
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
        let save_error = self.saves.write_session(&game.session).err();
        let archive_error = game
            .session
            .history
            .as_ref()
            .filter(|h| {
                state.finished || h.metadata.turn_count.is_multiple_of(self.checkpoint_turns)
            })
            .and_then(|h| record_in(self.archive.as_ref(), h).err());
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
        if let Some(error) = save_error {
            let message = format!("Turn resolved, but autosave failed: {error}");
            eprintln!("{message}");
            self.broadcast(&ServerMessage::Error {
                code: None,
                message,
            });
        }
        if let Some(error) = archive_error {
            self.archive_warning(error);
        }
    }

    pub(crate) fn configure_staging(&mut self, config: StagingConfig) -> Result<(), String> {
        config.validate()?;
        let lost = if let Some(room) = &mut self.room {
            room.configure(config)
        } else {
            let id = format!(
                "room-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
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
                None => crate::credentials::token()?,
            });
        }
        // Commit first. A disk failure must preserve the room, old game, and all sockets.
        let previous = self.preserve_current_history()?;
        self.saves
            .write_session(&game.session)
            .map_err(|e| format!("cannot save game: {e}"))?;
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
        if let Err(error) = self.checkpoint() {
            self.archive_warning(error);
        }
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
                .position(|s| crate::credentials::matches(s.token.as_deref(), Some(token)))
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

/// Write a recording to the archive, or do nothing when the host runs without one.
fn record_in(
    archive: Option<&legatum_archive::Archive>,
    recording: &legatum_archive::Recording,
) -> Result<(), String> {
    match archive {
        Some(archive) => archive.write(recording).map(|_| ()),
        None => Ok(()),
    }
}

fn check_setup(setup: &GameSetup) -> Result<(), String> {
    legatum_core::validation::check_setup(setup)?;
    let humans = setup.players.iter().take_while(|p| p.is_human).count();
    if !(2..=8).contains(&setup.players.len())
        || humans == 0
        || setup.players[humans..].iter().any(|p| p.is_human)
    {
        return Err("LAN games require 2-8 players with 1-8 humans in the first seats".into());
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

/// Make room for the incoming file before writing it. Replacing an existing name counts as
/// one file with the new length; ties in modification time are broken by path.
fn prune_scenario_runs(dir: &Path, name: &str, incoming_bytes: u64) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let destination = dir.join(name);
    let mut files = Vec::new();
    let mut total_bytes = incoming_bytes;
    let mut total_files = 1;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let metadata = entry.metadata()?;
        if path == destination {
            continue;
        }
        total_bytes += metadata.len();
        total_files += 1;
        files.push((metadata.modified()?, path, metadata.len()));
    }
    files.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    for (_, path, bytes) in files {
        if total_files <= MAX_SCENARIO_RUN_FILES && total_bytes <= MAX_SCENARIO_RUN_BYTES {
            break;
        }
        fs::remove_file(path)?;
        total_files -= 1;
        total_bytes -= bytes;
    }
    if total_bytes > MAX_SCENARIO_RUN_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Scenario run exceeds the 100 MiB storage limit",
        ));
    }
    Ok(())
}

/// `<scenario id>-<start time>.json` for a run file (the finish time for a run without one), with
/// every character checked: the name comes from the client, and must never leave the scenarios
/// directory. One run keeps one name, so each turn's checkpoint is replaced by the next.
pub(crate) fn scenario_run_name(json: &str) -> Result<String, String> {
    let run: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid scenario run: {e}"))?;
    if run["format"] != "legatum-scenario-run" {
        return Err("Not a scenario run".into());
    }
    let id = run["scenario"]["id"].as_str().unwrap_or_default();
    let at = run["started_at"]
        .as_str()
        .or(run["finished_at"].as_str())
        .unwrap_or_default();
    let stamp: String = at
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches('Z')
        .chars()
        .map(|c| if c == ':' { '-' } else { c })
        .collect();
    let safe = |s: &str| {
        !s.is_empty()
            && s.len() <= 64
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    if !safe(id) || !safe(&stamp) {
        return Err("Scenario run has an invalid id or time".into());
    }
    Ok(format!("{id}-{stamp}Z.json"))
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
    use legatum_core::Faction;
    use legatum_wasm::json;

    const RUN: &str = r#"{"format":"legatum-scenario-run","scenario":{"id":"rush-offense"},"finished_at":"2026-10-03T20:25:09.123Z"}"#;

    #[test]
    fn scenario_run_names_come_from_the_run_and_stay_in_their_directory() {
        assert_eq!(
            scenario_run_name(RUN).unwrap(),
            "rush-offense-2026-10-03T20-25-09Z.json"
        );
        let started = RUN.replacen('{', r#"{"started_at":"2026-10-03T20:00:00.5Z","#, 1);
        assert_eq!(
            scenario_run_name(&started).unwrap(),
            "rush-offense-2026-10-03T20-00-00Z.json"
        );
        for id in ["../escape", "a/b", "", "x y"] {
            let run = RUN.replace("rush-offense", id);
            assert!(scenario_run_name(&run).is_err(), "{id:?} was accepted");
        }
        assert!(scenario_run_name(r#"{"format":"legatum-save"}"#).is_err());
        assert!(scenario_run_name("not json").is_err());
    }

    #[test]
    fn scenario_runs_are_saved_under_the_saves_directory() {
        let root = tempfile::tempdir().unwrap();
        let host = Host::open(root.path().join("saves"), vec![]).unwrap();
        let path = host.save_scenario_run(RUN).unwrap();
        assert!(path.starts_with(root.path().join("saves").join("scenarios")));
        assert_eq!(std::fs::read_to_string(path).unwrap(), RUN);
    }

    #[test]
    fn lan_archive_retains_orders_names_and_hashes_across_restart() {
        let root = tempfile::tempdir().unwrap();
        let archive = legatum_archive::Archive::open(root.path().join("archive")).unwrap();
        let mut setup = GameSetup::standard(72, [Faction::India, Faction::Byzantium]);
        setup.turn_limit = Some(4);
        let mut host =
            Host::open_with_archive(root.path().join("saves"), vec![], archive.clone(), 2).unwrap();
        host.new_game(setup).unwrap();
        let (a, _a) = peer(&mut host);
        let (b, _b) = peer(&mut host);
        join(&mut host, a, 0);
        join(&mut host, b, 1);
        for _ in 0..2 {
            host.message(a, ready(&host, true)).unwrap();
            host.message(b, ready(&host, true)).unwrap();
        }
        let history = host
            .game
            .as_ref()
            .unwrap()
            .session
            .history
            .as_ref()
            .unwrap()
            .clone();
        let checkpoint = archive
            .root()
            .join(legatum_archive::Archive::relative_path(&history).unwrap());
        assert_eq!(
            legatum_archive::read(&checkpoint).unwrap().tape.turns.len(),
            2
        );
        drop(host);
        let mut host =
            Host::open_with_archive(root.path().join("saves"), vec![], archive.clone(), 2).unwrap();
        assert_eq!(
            host.game
                .as_ref()
                .unwrap()
                .session
                .history
                .as_ref()
                .unwrap(),
            &history
        );
        let (a, _a) = peer(&mut host);
        let (b, _b) = peer(&mut host);
        join(&mut host, a, 0);
        join(&mut host, b, 1);
        for _ in 0..2 {
            host.message(a, ready(&host, true)).unwrap();
            host.message(b, ready(&host, true)).unwrap();
        }
        let session = &host.game.as_ref().unwrap().session;
        let final_history = session.history.as_ref().unwrap();
        let path = archive
            .root()
            .join(legatum_archive::Archive::relative_path(final_history).unwrap());
        let stored = legatum_archive::read(&path).unwrap();
        assert_eq!(stored.metadata.game_id, history.metadata.game_id);
        assert_eq!(stored.metadata.source, legatum_archive::Source::Lan);
        assert_eq!(stored.metadata.seats[0].name, "Human 0");
        assert_eq!(stored.tape.turns.len(), 4);
        assert_eq!(stored.replay().unwrap().hash(), session.state.hash());
        assert!(!checkpoint.exists());
        drop(host);
        Host::open_with_archive(root.path().join("saves"), vec![], archive.clone(), 2).unwrap();
        assert_eq!(
            std::fs::read_to_string(archive.root().join("index.jsonl"))
                .unwrap()
                .lines()
                .count(),
            1
        );
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
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (guest, mut messages) = peer(&mut host);
        host.message(
            guest,
            ClientMessage::StagingTake {
                seat: 1,
                name: "Alice".into(),
                token: None,
            },
        )
        .unwrap();
        let taken: serde_json::Value =
            serde_json::from_str(&messages.reliable.try_recv().unwrap().texts[0]).unwrap();
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
        let left: serde_json::Value =
            serde_json::from_str(&messages.reliable.try_recv().unwrap().texts[0]).unwrap();
        assert_eq!(
            left,
            serde_json::json!({
                "type": "staging_left", "room_id": taken["room_id"],
            })
        );
        let snapshot: serde_json::Value =
            serde_json::from_str(messages.staging.borrow().as_ref().unwrap()).unwrap();
        assert_eq!(snapshot["state"]["occupants"][1]["name"], "Bob");
    }

    #[test]
    fn staging_is_coalesced_and_eviction_disconnects_the_reservation() {
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (watcher, mut watching) = peer(&mut host);
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
        assert_eq!(watching.reliable.len(), 0);
        assert_eq!(guest_messages.reliable.len(), 1); // private credential acknowledgment
        let latest: serde_json::Value =
            serde_json::from_str(watching.staging.borrow_and_update().as_ref().unwrap()).unwrap();
        assert_eq!(latest["state"]["occupants"][1]["ready"], true);
        host.message(guest, ClientMessage::StagingReady { ready: true })
            .unwrap();
        assert!(!watching.staging.has_changed().unwrap());
        for _ in 0..17 {
            let _ = host.message(guest, ClientMessage::Ping);
        }
        let state = host.status(true).staging.unwrap();
        let occupant = state.occupants[1].as_ref().unwrap();
        assert!(!occupant.connected);
        assert!(!occupant.ready);
        assert!(host.staging_can_start().is_err());
        assert_eq!(
            guest_messages.close.borrow().as_ref().unwrap().code,
            Some("slow_consumer")
        );
        host.close_staging();
        assert!(
            watching.staging.borrow().is_none(),
            "closed room left a stale snapshot"
        );
        let event: serde_json::Value =
            serde_json::from_str(&watching.reliable.try_recv().unwrap().texts[0]).unwrap();
        assert_eq!(event["type"], "staging_closed");
    }

    #[test]
    fn only_loopback_sockets_can_take_the_host_slot() {
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.configure_staging(room_config()).unwrap();
        let (sender, mut remote_messages) = outbox::channel();
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
        let error: serde_json::Value =
            serde_json::from_str(&remote_messages.reliable.try_recv().unwrap().texts[0]).unwrap();
        assert_eq!(error["code"], "not_open");
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
        let (local, mut local_messages) = peer(&mut host);
        host.message(
            local,
            ClientMessage::StagingTake {
                seat: 0,
                name: "Host".into(),
                token: None,
            },
        )
        .unwrap();
        local_messages.reliable.try_recv().unwrap(); // staging_taken
        host.start_staging().unwrap();
        let started: serde_json::Value =
            serde_json::from_str(&local_messages.reliable.try_recv().unwrap().texts[0]).unwrap();
        assert_eq!(started["type"], "staging_started");
        assert_eq!(started["seat"], 0);
        assert!(local_messages.staging.borrow().is_none());
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
            .get_or_insert_with(|| crate::credentials::token().unwrap())
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
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let saved = host.save_game().unwrap();
        let committed = std::fs::read(root.path().join("autosave.json")).unwrap();
        let original: serde_json::Value = serde_json::from_str(&saved).unwrap();
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
            assert_eq!(
                std::fs::read(root.path().join("autosave.json")).unwrap(),
                committed
            );
            // Startup must enforce the same bounds as HTTP imports.
            let restart = tempfile::tempdir().unwrap();
            std::fs::write(restart.path().join("autosave.json"), invalid.to_string()).unwrap();
            assert!(Host::open(restart.path().to_owned(), vec![]).is_err());
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
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.new_game(GameSetup::standard(7, [Faction::India, Faction::Byzantium]))
            .unwrap();
        let (observer, messages) = peer(&mut host);
        let (human, _human_messages) = peer(&mut host);
        join(&mut host, human, 0);
        for i in 0..128 {
            host.message(human, ready(&host, i % 2 == 0)).unwrap();
        }
        assert!(host.connections.contains_key(&observer));
        assert_eq!(
            messages.reliable.len(),
            0,
            "seat updates entered the reliable queue"
        );
        let snapshot: serde_json::Value =
            serde_json::from_str(messages.seats.borrow().as_ref().unwrap()).unwrap();
        assert_eq!(snapshot["seats"][0]["ready"], false);
    }

    #[test]
    fn reliable_queue_overflow_evicts_without_waiting_for_the_reader() {
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        let (slow, messages) = peer(&mut host);
        for _ in 0..17 {
            let _ = host.message(slow, ClientMessage::Ping);
        }
        assert!(
            !host.connections.contains_key(&slow),
            "full queue kept the connection"
        );
        assert!(messages.reliable.len() <= 16);
        assert_eq!(
            messages.close.borrow().as_ref().unwrap().code,
            Some("slow_consumer")
        );
    }

    #[test]
    fn connections_have_a_fixed_cap() {
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
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

    #[test]
    fn orders_and_ready_require_the_current_game_and_planning_turn() {
        let root = tempfile::tempdir().unwrap();
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
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
        let decode = |v: serde_json::Value| serde_json::from_value::<ClientMessage>(v).unwrap();
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
        let root = tempfile::tempdir().unwrap();
        let setup = GameSetup::standard(u64::MAX - 7, [Faction::India, Faction::Byzantium]);
        let mut host = Host::open(root.path().to_owned(), vec![]).unwrap();
        host.new_game(setup.clone()).unwrap();
        let (a, mut a_messages) = peer(&mut host);
        join(&mut host, a, 0);
        host.message(a, ready(&host, true)).unwrap();
        assert_eq!(host.status(true).turn, 1, "seat 1 has never joined");
        let (b, _b_messages) = peer(&mut host);
        join(&mut host, b, 1);
        while a_messages.reliable.try_recv().is_ok() {}
        let (replacement, mut messages) = peer(&mut host);
        join(&mut host, replacement, 0);
        let outbox::Close { code, message } =
            a_messages.close.borrow().clone().expect("takeover close");
        assert_eq!(
            serde_json::to_value(ServerMessage::Error { code, message }).unwrap(),
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
        let mut turn_seen = false;
        while let Ok(batch) = messages.reliable.try_recv() {
            let message: serde_json::Value = serde_json::from_str(&batch.texts[0]).unwrap();
            if message["type"] == "turn" {
                assert_eq!(message["turn"], 2);
                assert_eq!(message["hash"], native.state.hash().to_string());
                turn_seen = true;
                let message: serde_json::Value = serde_json::from_str(&batch.texts[1]).unwrap();
                assert_eq!(message["type"], "view");
                assert_eq!(message["view"]["me"], 0);
                assert_eq!(message["view"]["turn"], 2);
            }
        }
        assert!(turn_seen);
        let mut restored = Host::open(root.path().to_owned(), vec![]).unwrap();
        assert_eq!(restored.save_game().unwrap(), host.save_game().unwrap());
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

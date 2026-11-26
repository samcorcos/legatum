//! The host's connections as the transport sees them: open one, hand it each text message,
//! close it, tick the clock, and drain what to write. The browser host (`app/src/browserHost.ts`)
//! drives this over the relay tunnel and the host's own in-tab connection; the tests drive it
//! directly. Times are milliseconds on any monotonic clock the caller chooses.
use super::{ConnectionId, Host, Status, Summary, outbox, protocol::ClientMessage, staging};
use crate::history::CHECKPOINT_TURNS;
use serde::Serialize;
use std::collections::BTreeMap;

/// An unseated connection must take a staging seat or join the game within this time.
/// Pings and repeated watches never extend it.
pub const PRE_JOIN_TIMEOUT_MS: f64 = 30_000.0;
/// A guest that sends nothing for this long is closed. Clients ping every 20 s; the margin
/// covers background tabs, whose timers browsers slow to about once a minute.
pub const SILENCE_TIMEOUT_MS: f64 = 120_000.0;
/// Guests at once (the relay's own cap per room is the same).
pub const MAX_GUESTS: usize = 8;
/// Largest guest message, as on the relay.
pub const MAX_GUEST_MESSAGE: usize = 64 * 1024;

struct Link {
    receiver: outbox::Receiver,
    /// The host's own UI in this tab; everyone else is a guest through the relay.
    local: bool,
    last_seen: f64,
    joined: bool,
    join_deadline: f64,
}

/// What to write to one connection.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Outgoing {
    pub conn: ConnectionId,
    pub texts: Vec<String>,
    /// The host is done with this connection: write `texts`, then close it. It is already
    /// forgotten here; do not call [`NetHost::disconnect`] for it.
    pub close: bool,
}

/// Why the room could not start, with the protocol's error code when there is one.
#[derive(Debug, Serialize)]
pub struct StartError {
    pub code: Option<&'static str>,
    pub message: String,
}

pub struct NetHost {
    host: Host,
    links: BTreeMap<ConnectionId, Link>,
    /// Connections ended by `tick`, reported by the next `drain`.
    closed: Vec<ConnectionId>,
}

impl NetHost {
    /// A host with no game, or one resuming the browser's saved copy of an online game.
    pub fn new(save: Option<&str>) -> Result<Self, String> {
        Self::with_checkpoints(save, CHECKPOINT_TURNS)
    }

    /// As [`NetHost::new`], handing over the game's history every `checkpoint_turns` turns.
    pub fn with_checkpoints(save: Option<&str>, checkpoint_turns: u32) -> Result<Self, String> {
        Ok(Self {
            host: guarded(|| Host::new(save, checkpoint_turns))?,
            links: BTreeMap::new(),
            closed: Vec::new(),
        })
    }

    /// Admits a connection. Guests are limited to [`MAX_GUESTS`], all connections to 32.
    pub fn connect(&mut self, local: bool, now: f64) -> Result<ConnectionId, String> {
        if !local && self.links.values().filter(|l| !l.local).count() >= MAX_GUESTS {
            return Err("The game is full".into());
        }
        let (sender, receiver) = outbox::channel();
        let id = guarded(|| self.host.connect(sender, local))?;
        self.links.insert(
            id,
            Link {
                receiver,
                local,
                last_seen: now,
                joined: false,
                join_deadline: now + PRE_JOIN_TIMEOUT_MS,
            },
        );
        Ok(id)
    }

    /// One text message from a connection. A bad message is answered with an `error`.
    pub fn receive(&mut self, conn: ConnectionId, text: &str, now: f64) {
        let Some(link) = self.links.get_mut(&conn) else {
            return;
        };
        link.last_seen = now;
        let result = if text.len() > MAX_GUEST_MESSAGE {
            Err("Message too large".to_owned())
        } else {
            match serde_json::from_str::<ClientMessage>(text) {
                Ok(message) => guarded(|| self.host.message(conn, message)),
                Err(error) => Err(format!("Invalid message: {error}")),
            }
        };
        if let Err(message) = result {
            self.host.send(
                conn,
                &super::protocol::ServerMessage::Error {
                    code: None,
                    message,
                },
            );
        }
    }

    /// The transport lost the connection.
    pub fn disconnect(&mut self, conn: ConnectionId) {
        if self.links.remove(&conn).is_some() {
            let _ = guarded(|| {
                self.host.disconnect(conn);
                Ok(())
            });
        }
    }

    /// Ends connections past their join deadline or silent for too long.
    pub fn tick(&mut self, now: f64) {
        self.track_admission(now);
        let expired: Vec<_> = self
            .links
            .iter()
            .filter(|(_, l)| {
                (!l.joined && now >= l.join_deadline)
                    || (!l.local && now - l.last_seen >= SILENCE_TIMEOUT_MS)
            })
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            self.disconnect(id);
            // The transport learns of it from `drain`: an empty, closing delivery.
            self.closed.push(id);
        }
    }

    /// Everything to write now, connection by connection. Closing a connection can change
    /// what others see (seats, the room), so this repeats until nothing is left.
    pub fn drain(&mut self, now: f64) -> Vec<Outgoing> {
        let mut out: Vec<Outgoing> = std::mem::take(&mut self.closed)
            .into_iter()
            .map(|conn| Outgoing {
                conn,
                texts: Vec::new(),
                close: true,
            })
            .collect();
        loop {
            self.track_admission(now);
            let mut closed = Vec::new();
            for (&conn, link) in &self.links {
                let drained = link.receiver.drain();
                if drained.closed {
                    closed.push(conn);
                }
                if drained.closed || !drained.texts.is_empty() {
                    match out.iter_mut().find(|o| o.conn == conn) {
                        Some(o) => {
                            o.texts.extend(drained.texts);
                            o.close |= drained.closed;
                        }
                        None => out.push(Outgoing {
                            conn,
                            texts: drained.texts,
                            close: drained.closed,
                        }),
                    }
                }
            }
            if closed.is_empty() {
                return out;
            }
            for conn in closed {
                self.disconnect(conn);
            }
        }
    }

    /// A seat gained restarts nothing; a seat lost starts a fresh join deadline.
    fn track_admission(&mut self, now: f64) {
        for link in self.links.values_mut() {
            if let Some(joined) = link.receiver.admitted() {
                if link.joined && !joined {
                    link.join_deadline = now + PRE_JOIN_TIMEOUT_MS;
                }
                link.joined = joined;
            }
        }
    }

    pub fn status(&self, is_local: bool) -> Status {
        self.host.status(is_local)
    }

    pub fn summary(&self) -> Summary {
        self.host.summary()
    }

    /// Starts a game from a setup (humans first). Everyone connected must rejoin.
    pub fn new_game(&mut self, setup_json: &str) -> Result<(), String> {
        guarded(|| self.host.new_game(crate::json::parse_setup(setup_json)?))
    }

    /// Loads an opaque save string as the online game.
    pub fn load_game(&mut self, save: &str) -> Result<(), String> {
        guarded(|| self.host.load_game(save))
    }

    /// Opens the staging room, or changes its settings.
    pub fn configure_staging(&mut self, config_json: &str) -> Result<(), String> {
        guarded(|| {
            let config: staging::StagingConfig =
                serde_json::from_str(config_json).map_err(|e| format!("Invalid room: {e}"))?;
            self.host.configure_staging(config)
        })
    }

    pub fn close_staging(&mut self) {
        let _ = guarded(|| {
            self.host.close_staging();
            Ok(())
        });
    }

    /// Starts the room's game; `not_ready` or `no_room` when it cannot start yet.
    pub fn start_staging(&mut self) -> Result<(), StartError> {
        if let Err(e) = self.host.staging_can_start() {
            return Err(StartError {
                code: Some(e.code),
                message: e.message.into(),
            });
        }
        guarded(|| self.host.start_staging()).map_err(|message| StartError {
            code: None,
            message,
        })
    }

    /// Sets or clears the join code (4 to 12 letters or digits; empty clears). It stays in this
    /// host: no status, summary, save or message carries it.
    pub fn set_join_code(&mut self, code: Option<&str>) -> Result<(), String> {
        self.host.set_join_code(code)
    }

    /// The live game's save, once per change.
    pub fn take_save(&mut self) -> Option<String> {
        self.host.take_save()
    }

    /// Game histories (archive recordings as JSON) to keep, oldest first.
    pub fn take_recordings(&mut self) -> Vec<String> {
        self.host.take_recordings()
    }

    /// Opens every seat but the host's own for guests (hosting from a save file).
    pub fn open_guest_seats(&mut self) {
        self.host.open_guest_seats();
    }

    /// The live game's save, for the host's own copy on demand.
    pub fn save_game(&self) -> Result<String, String> {
        self.host.save_game()
    }
}

fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    crate::guard(f)
}

/// [`NetHost`] for JavaScript (`app/src/browserHost.ts`): JSON strings in and out, connection
/// ids as numbers, times from `performance.now()`.
#[wasm_bindgen::prelude::wasm_bindgen]
pub struct BrowserHost(NetHost);

#[wasm_bindgen::prelude::wasm_bindgen]
impl BrowserHost {
    /// A host with no game, or one resuming a saved online game.
    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    pub fn new(save: Option<String>) -> Result<BrowserHost, wasm_bindgen::JsError> {
        crate::js(NetHost::new(save.as_deref()).map(Self))
    }

    pub fn connect(&mut self, local: bool, now: f64) -> Result<u32, wasm_bindgen::JsError> {
        crate::js(self.0.connect(local, now).map(|id| id as u32))
    }

    pub fn receive(&mut self, conn: u32, text: &str, now: f64) {
        self.0.receive(conn.into(), text, now);
    }

    pub fn disconnect(&mut self, conn: u32) {
        self.0.disconnect(conn.into());
    }

    pub fn tick(&mut self, now: f64) {
        self.0.tick(now);
    }

    /// `[{conn, texts, close}]`, see [`Outgoing`].
    pub fn drain(&mut self, now: f64) -> String {
        serde_json::to_string(&self.0.drain(now)).expect("outgoing serializes")
    }

    pub fn status(&self, is_local: bool) -> String {
        serde_json::to_string(&self.0.status(is_local)).expect("status serializes")
    }

    pub fn summary(&self) -> String {
        serde_json::to_string(&self.0.summary()).expect("summary serializes")
    }

    pub fn new_game(&mut self, setup_json: &str) -> Result<(), wasm_bindgen::JsError> {
        crate::js(self.0.new_game(setup_json))
    }

    pub fn load_game(&mut self, save: &str) -> Result<(), wasm_bindgen::JsError> {
        crate::js(self.0.load_game(save))
    }

    pub fn configure_staging(&mut self, config_json: &str) -> Result<(), wasm_bindgen::JsError> {
        crate::js(self.0.configure_staging(config_json))
    }

    pub fn close_staging(&mut self) {
        self.0.close_staging();
    }

    /// `""` once started, else the [`StartError`] as JSON.
    pub fn start_staging(&mut self) -> String {
        match self.0.start_staging() {
            Ok(()) => String::new(),
            Err(e) => serde_json::to_string(&e).expect("errors serialize"),
        }
    }

    pub fn take_save(&mut self) -> Option<String> {
        self.0.take_save()
    }

    /// Sets or clears the join code. Local only: it never leaves this engine.
    pub fn set_join_code(&mut self, code: Option<String>) -> Result<(), wasm_bindgen::JsError> {
        crate::js(self.0.set_join_code(code.as_deref()))
    }

    /// Opens every seat but the host's own for guests (hosting from a save file).
    pub fn open_guest_seats(&mut self) {
        self.0.open_guest_seats();
    }

    /// The live game's save right now, for the host's "Save to file" (empty when there is no game).
    pub fn save_game(&self) -> Result<String, wasm_bindgen::JsError> {
        crate::js(self.0.save_game())
    }

    /// A JSON array of recording JSON strings.
    pub fn take_recordings(&mut self) -> String {
        serde_json::to_string(&self.0.take_recordings()).expect("strings serialize")
    }

    /// As `take_recordings`, with every seat under its default name ("Player 1", "AI 2"): the copies a
    /// build that uploads games keeps and sends, as a single-player game does with `public_archive`.
    pub fn take_public_recordings(&mut self) -> String {
        serde_json::to_string(&public_recordings(&self.0.take_recordings()))
            .expect("strings serialize")
    }
}

thread_local! {
    /// Where the last renamed copy stopped, so the next checkpoint of the same game replays only new turns.
    static PUBLIC_RESUME: std::cell::RefCell<Option<crate::history::AnonymizedResume>> =
        const { std::cell::RefCell::new(None) };
}

/// Each recording with typed names replaced by the defaults. One that cannot be renamed safely (it does not
/// parse, or a typed name is still quoted somewhere in it) is left out: it must not be uploaded.
fn public_recordings(recordings: &[String]) -> Vec<String> {
    recordings
        .iter()
        .filter_map(|json| {
            let recording: crate::history::Recording = serde_json::from_str(json).ok()?;
            let public = PUBLIC_RESUME
                .with(|resume| recording.anonymized_resuming(&mut resume.borrow_mut()))
                .ok()?;
            serde_json::to_string(&public).ok()
        })
        .collect()
}

/// The rules version a guest must match to join this host (`RoomAnnounce::rules_version`).
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn online_rules_version() -> u32 {
    legatum_core::RULES_VERSION
}

#[cfg(test)]
mod public_recording_tests {
    use super::public_recordings;
    use crate::OrderMap;
    use crate::history::Recording;
    use crate::session::{BrainKind, Session};
    use legatum_core::{Faction, GameSetup};

    #[test]
    fn public_recordings_carry_default_names_across_checkpoints_and_drop_what_does_not_parse() {
        let mut setup = GameSetup::standard(42, [Faction::India, Faction::Byzantium]);
        setup.players.truncate(3);
        setup.players[0].name = "Dana Smith".into();
        setup.players[1].name = "Alex".into();
        setup.players[2].is_human = false;
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        // Two checkpoints of one game, as an online host hands them over, then a broken one.
        for turns in [2, 2] {
            for _ in 0..turns {
                session.step(&OrderMap::new());
            }
            let history = session.history.as_ref().unwrap();
            let own = serde_json::to_string(history).unwrap();
            assert!(own.contains("Dana Smith"));
            let public = public_recordings(&[own, "not a recording".into()]);
            assert_eq!(public.len(), 1);
            assert!(!public[0].contains("Dana Smith") && !public[0].contains("\"Alex\""));
            let recording: Recording = serde_json::from_str(&public[0]).unwrap();
            let names: Vec<_> = recording
                .metadata
                .seats
                .iter()
                .map(|s| s.name.as_str())
                .collect();
            assert_eq!(names, ["Player 1", "Player 2", "AI 3"]);
            assert_eq!(recording.metadata.game_id, history.metadata.game_id);
            // The renamed copy is a whole game of its own: its orders replay to its hashes.
            recording.replay().unwrap();
        }
    }
}

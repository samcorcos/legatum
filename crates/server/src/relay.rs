//! Online play through the relay (`plans/online-play.md`). The host dials out, holds one tunnel
//! WebSocket, and every guest is multiplexed over it. Each relayed guest runs the ordinary
//! connection loop as a non-loopback peer; the relay only forwards text and never sees game rules.
use crate::{Server, Transport, connection};
use axum::{body::Bytes, extract::ws::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, watch},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream, tungstenite::Message as TunnelMessage,
};

/// Bumped on any incompatible change to the online contract or the game protocol.
pub const ONLINE_PROTOCOL: u32 = 1;
/// Guest-bound texts longer than this many UTF-8 bytes travel as `chunk` messages.
pub const CHUNK_BYTES: usize = 256 * 1024;
/// The admission peer shared by every relayed guest: `100::1`, in the IPv6 discard-only prefix
/// (RFC 6666), so it can never be a real TCP peer or loopback. Relayed guests together get one
/// peer's quota (eight sockets), matching the relay's eight guests per room.
pub(crate) const RELAY_PEER: IpAddr = IpAddr::V6(Ipv6Addr::new(0x100, 0, 0, 0, 0, 0, 0, 1));

const OPEN: u8 = 0;
const DATA: u8 = 1;
const CLOSE: u8 = 2;
const STATUS: u8 = 3;
const SUMMARY: u8 = 4;
const BYE: u8 = 5;
/// The LAN socket's message limit, applied to relayed guests too.
const MAX_GUEST_MESSAGE: usize = 64 * 1024;
/// Guest messages waiting for their connection task; a guest that floods past it is closed.
const INBOX: usize = 64;
const TUNNEL_QUEUE: usize = 256;
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const PING_EVERY: Duration = Duration::from_secs(20);
const TUNNEL_SILENCE: Duration = Duration::from_secs(60);
const MIN_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// The lobby row a host publishes when it opens a room.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomAnnounce {
    /// The host's display name, at most 80 characters (longer names are cut).
    pub name: String,
    /// The app's build label, at most 40 characters (longer labels are cut).
    pub build: String,
    pub rules_version: u32,
    pub protocol: u32,
}

impl RoomAnnounce {
    /// A room for this build of the rules and protocol.
    pub fn new(name: impl Into<String>, build: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            build: build.into(),
            rules_version: legatum_core::RULES_VERSION,
            protocol: ONLINE_PROTOCOL,
        }
    }

    fn bounded(&self) -> Self {
        Self {
            name: self.name.chars().take(80).collect(),
            build: self.build.chars().take(40).collect(),
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct RelayConfig {
    /// The relay's base URL: `https://…` in production, `http://…` in tests (`wss`/`ws` work too).
    pub url: String,
    pub announce: RoomAnnounce,
}

/// The `summary` control frame: the relay's lobby row for this host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Summary {
    pub(crate) players: usize,
    pub(crate) humans: usize,
    pub(crate) open_seats: usize,
    pub(crate) phase: &'static str,
    pub(crate) turn: u32,
}

/// A running room. Dropping it closes the room like [`RelayHandle::stop`], without waiting.
pub struct RelayHandle {
    room: Arc<Mutex<Room>>,
    stop: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl RelayHandle {
    /// The current room id. It changes only if the relay forgot the room and it was re-announced.
    pub fn room(&self) -> String {
        self.room.lock().expect("room lock").id.clone()
    }

    /// Sends `bye` (when the tunnel is up) and stops reconnecting. Waits at most two seconds.
    pub async fn stop(self) {
        let _ = self.stop.send(true);
        let _ = timeout(Duration::from_secs(2), self.task).await;
    }
}

impl Server {
    /// Announces the room (`POST /rooms`), then keeps its tunnel open in the background,
    /// reconnecting with backoff. Fails only if the first announcement fails.
    pub async fn run_relay(&self, config: RelayConfig) -> Result<RelayHandle, String> {
        let relay = Relay::new(self.clone(), config)?;
        let room = relay.announce().await?;
        let room = Arc::new(Mutex::new(room));
        let (stop, stopped) = watch::channel(false);
        let task = tokio::spawn(
            Relay {
                room: room.clone(),
                ..relay
            }
            .run(stopped),
        );
        Ok(RelayHandle { room, stop, task })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Room {
    #[serde(rename = "room")]
    id: String,
    host_key: String,
}

struct Relay {
    server: Server,
    http_base: String,
    ws_base: String,
    announce: RoomAnnounce,
    http: reqwest::Client,
    tls: Arc<rustls::ClientConfig>,
    room: Arc<Mutex<Room>>,
}

enum Failure {
    /// The relay no longer knows the room; announce a new one.
    Gone,
    Other(String),
}

enum End {
    Stopped,
    Dropped(String),
}

impl Relay {
    fn new(server: Server, config: RelayConfig) -> Result<Self, String> {
        let base = config.url.trim_end_matches('/');
        let (http_base, ws_base) = if let Some(rest) = base.strip_prefix("https://") {
            (base.to_owned(), format!("wss://{rest}"))
        } else if let Some(rest) = base.strip_prefix("http://") {
            (base.to_owned(), format!("ws://{rest}"))
        } else if let Some(rest) = base.strip_prefix("wss://") {
            (format!("https://{rest}"), base.to_owned())
        } else if let Some(rest) = base.strip_prefix("ws://") {
            (format!("http://{rest}"), base.to_owned())
        } else {
            return Err(format!(
                "Relay URL must start with https:// or http://: {base}"
            ));
        };
        let tls = Arc::new(tls_config()?);
        // Direct connections only: the tunnel WebSocket cannot use an HTTP proxy either.
        let http = reqwest::Client::builder()
            .use_preconfigured_tls((*tls).clone())
            .no_proxy()
            .timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(|e| format!("Cannot create the relay client: {e}"))?;
        Ok(Self {
            server,
            http_base,
            ws_base,
            announce: config.announce.bounded(),
            http,
            tls,
            room: Arc::new(Mutex::new(Room {
                id: String::new(),
                host_key: String::new(),
            })),
        })
    }

    async fn announce(&self) -> Result<Room, String> {
        let response = self
            .http
            .post(format!("{}/rooms", self.http_base))
            .json(&self.announce)
            .send()
            .await
            .map_err(|e| format!("Cannot reach the relay: {e}"))?;
        if !response.status().is_success() {
            // e.g. 503 {"error":"relay is full"}
            let status = response.status();
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let reason = body["error"].as_str().unwrap_or("no reason given");
            return Err(format!("The relay refused the room ({status}): {reason}"));
        }
        response
            .json::<Room>()
            .await
            .map_err(|e| format!("Unexpected relay response: {e}"))
    }

    async fn run(self, mut stop: watch::Receiver<bool>) {
        let mut backoff = MIN_BACKOFF;
        loop {
            let attempt = tokio::select! {
                attempt = self.connect() => attempt,
                _ = stop.changed() => return,
            };
            match attempt {
                Ok(tunnel) => {
                    backoff = MIN_BACKOFF;
                    match self.session(tunnel, &mut stop).await {
                        End::Stopped => return,
                        End::Dropped(reason) => eprintln!("Relay tunnel dropped: {reason}"),
                    }
                }
                Err(Failure::Gone) => match self.announce().await {
                    Ok(room) => {
                        eprintln!("The relay forgot the room; reopened as {}", room.id);
                        *self.room.lock().expect("room lock") = room;
                    }
                    Err(error) => eprintln!("{error}"),
                },
                Err(Failure::Other(error)) => eprintln!("Relay tunnel failed: {error}"),
            }
            tokio::select! {
                () = sleep(backoff) => {}
                _ = stop.changed() => return,
            }
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    }

    async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Failure> {
        let url = {
            let room = self.room.lock().expect("room lock");
            format!(
                "{}/rooms/{}/host?key={}",
                self.ws_base, room.id, room.host_key
            )
        };
        let connecting = tokio_tungstenite::connect_async_tls_with_config(
            url,
            None,
            false,
            Some(Connector::Rustls(self.tls.clone())),
        );
        match timeout(CONNECT_TIMEOUT, connecting).await {
            Ok(Ok((tunnel, _))) => Ok(tunnel),
            Ok(Err(tokio_tungstenite::tungstenite::Error::Http(response)))
                if matches!(response.status().as_u16(), 404 | 410) =>
            {
                Err(Failure::Gone)
            }
            Ok(Err(error)) => Err(Failure::Other(error.to_string())),
            Err(_) => Err(Failure::Other("timed out connecting".into())),
        }
    }

    /// One tunnel's lifetime. Guest connections belong to it and end with it.
    async fn session(
        &self,
        tunnel: WebSocketStream<MaybeTlsStream<TcpStream>>,
        stop: &mut watch::Receiver<bool>,
    ) -> End {
        let (mut sink, mut stream) = tunnel.split();
        let (out, mut outgoing) = mpsc::channel::<TunnelMessage>(TUNNEL_QUEUE);
        let mut writer = tokio::spawn(async move {
            while let Some(message) = outgoing.recv().await {
                let last = matches!(message, TunnelMessage::Close(_));
                if !matches!(timeout(WRITE_TIMEOUT, sink.send(message)).await, Ok(Ok(()))) || last {
                    return;
                }
            }
        });
        let mut refreshed = self.server.refreshed.subscribe();
        let mut pushed = (String::new(), String::new());
        self.push(&out, &mut pushed).await;
        let mut guests: HashMap<u32, mpsc::Sender<Message>> = HashMap::new();
        let mut ping = tokio::time::interval(PING_EVERY);
        ping.tick().await;
        let mut last_seen = Instant::now();
        let end = loop {
            tokio::select! {
                _ = stop.changed() => {
                    // A dropped handle stops the room too.
                    guests.clear();
                    let _ = out.send(frame(0, BYE, b"")).await;
                    let _ = out.send(TunnelMessage::Close(None)).await;
                    let _ = timeout(Duration::from_secs(1), &mut writer).await;
                    break End::Stopped;
                }
                _ = refreshed.changed() => self.push(&out, &mut pushed).await,
                _ = ping.tick() => {
                    if last_seen.elapsed() >= TUNNEL_SILENCE {
                        break End::Dropped("no reply from the relay".into());
                    }
                    guests.retain(|_, inbox| !inbox.is_closed());
                    let _ = out.send(TunnelMessage::Ping(Bytes::new())).await;
                }
                _ = &mut writer => break End::Dropped("write failed".into()),
                incoming = stream.next() => {
                    last_seen = Instant::now();
                    match incoming {
                        Some(Ok(TunnelMessage::Binary(bytes))) => self.route(bytes, &mut guests, &out).await,
                        Some(Ok(TunnelMessage::Close(_))) | None => break End::Dropped("closed by the relay".into()),
                        Some(Err(error)) => break End::Dropped(error.to_string()),
                        Some(Ok(_)) => {}
                    }
                }
            }
        };
        writer.abort();
        end
    }

    /// Sends `status` and `summary` when they differ from what this tunnel last carried.
    async fn push(&self, out: &mpsc::Sender<TunnelMessage>, pushed: &mut (String, String)) {
        let status = self.server.status.read().ok().map(|status| {
            let mut status = status.clone();
            status.is_local = false;
            // Guests on the relay cannot use the host's LAN addresses; do not publish them.
            status.lan_urls.clear();
            serde_json::to_string(&status).expect("status serializes")
        });
        let summary = self
            .server
            .summary
            .read()
            .ok()
            .map(|summary| serde_json::to_string(&*summary).expect("summary serializes"));
        if let Some(status) = status
            && status != pushed.0
        {
            let _ = out.send(frame(0, STATUS, status.as_bytes())).await;
            pushed.0 = status;
        }
        if let Some(summary) = summary
            && summary != pushed.1
        {
            let _ = out.send(frame(0, SUMMARY, summary.as_bytes())).await;
            pushed.1 = summary;
        }
    }

    async fn route(
        &self,
        bytes: Bytes,
        guests: &mut HashMap<u32, mpsc::Sender<Message>>,
        out: &mpsc::Sender<TunnelMessage>,
    ) {
        if bytes.len() < 5 {
            return;
        }
        let conn = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let payload = bytes.slice(5..);
        match bytes[4] {
            OPEN if conn != 0 && !guests.contains_key(&conn) => {
                guests.retain(|_, inbox| !inbox.is_closed());
                let Some(permit) = self.server.admission.acquire(RELAY_PEER, false) else {
                    let _ = out.send(frame(conn, CLOSE, b"")).await;
                    return;
                };
                let (inbox, received) = mpsc::channel(INBOX);
                guests.insert(conn, inbox);
                let guest = Guest {
                    conn,
                    inbox: received,
                    out: out.clone(),
                    chunk_id: 0,
                };
                tokio::spawn(connection(self.server.clone(), guest, permit, false));
            }
            DATA => {
                let Some(inbox) = guests.get(&conn) else {
                    return;
                };
                let delivered = payload.len() <= MAX_GUEST_MESSAGE
                    && inbox
                        .try_send(match String::from_utf8(payload.to_vec()) {
                            Ok(text) => Message::Text(text.into()),
                            Err(_) => Message::Binary(payload),
                        })
                        .is_ok();
                if !delivered {
                    // Oversized, flooding or already finished: end it as the LAN socket would.
                    guests.remove(&conn);
                    let _ = out.send(frame(conn, CLOSE, b"")).await;
                }
            }
            CLOSE => {
                guests.remove(&conn);
            }
            _ => {}
        }
    }
}

fn tls_config() -> Result<rustls::ClientConfig, String> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    Ok(rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| format!("TLS setup failed: {e}"))?
    .with_root_certificates(roots)
    .with_no_client_auth())
}

fn frame(conn: u32, kind: u8, payload: &[u8]) -> TunnelMessage {
    let mut bytes = Vec::with_capacity(5 + payload.len());
    bytes.extend_from_slice(&conn.to_be_bytes());
    bytes.push(kind);
    bytes.extend_from_slice(payload);
    TunnelMessage::Binary(bytes.into())
}

/// Splits a guest-bound text longer than [`CHUNK_BYTES`] on character boundaries into `chunk`
/// messages; shorter texts pass through unchanged.
pub(crate) fn chunks(text: &str, id: u32) -> Vec<String> {
    if text.len() <= CHUNK_BYTES {
        return vec![text.to_owned()];
    }
    let mut pieces = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + CHUNK_BYTES).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        pieces.push(&text[start..end]);
        start = end;
    }
    let n = pieces.len();
    pieces
        .into_iter()
        .enumerate()
        .map(|(i, piece)| {
            let data = serde_json::to_string(piece).expect("strings serialize");
            format!(r#"{{"type":"chunk","id":{id},"i":{i},"n":{n},"data":{data}}}"#)
        })
        .collect()
}

/// One relayed guest: its messages arrive from the tunnel reader and leave as `data` frames.
struct Guest {
    conn: u32,
    inbox: mpsc::Receiver<Message>,
    out: mpsc::Sender<TunnelMessage>,
    chunk_id: u32,
}

impl Guest {
    async fn forward(&self, kind: u8, payload: &[u8]) -> bool {
        matches!(
            timeout(
                WRITE_TIMEOUT,
                self.out.send(frame(self.conn, kind, payload))
            )
            .await,
            Ok(Ok(()))
        )
    }
}

impl Transport for Guest {
    async fn send(&mut self, message: Message) -> bool {
        match message {
            Message::Text(text) if text.as_str().len() > CHUNK_BYTES => {
                self.chunk_id += 1;
                for piece in chunks(text.as_str(), self.chunk_id) {
                    if !self.forward(DATA, piece.as_bytes()).await {
                        return false;
                    }
                }
                true
            }
            Message::Text(text) => self.forward(DATA, text.as_bytes()).await,
            // The relay keeps the guest's socket alive; the client's `ping` messages keep the
            // connection loop's 60-second silence check satisfied.
            _ => true,
        }
    }

    async fn recv(&mut self) -> Option<Message> {
        self.inbox.recv().await
    }

    async fn finish_close(&mut self) {
        // A closed inbox means the relay (or the tunnel) already ended this guest.
        if !self.inbox.is_closed() {
            self.forward(CLOSE, b"").await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_split_on_character_boundaries_and_reassemble() {
        assert_eq!(chunks("short", 1), ["short"]);
        let text = format!(
            "{}é\"{}",
            "a".repeat(CHUNK_BYTES - 1),
            "b".repeat(CHUNK_BYTES)
        );
        let pieces = chunks(&text, 7);
        assert_eq!(pieces.len(), 3);
        let mut joined = String::new();
        for (i, piece) in pieces.iter().enumerate() {
            let value: serde_json::Value = serde_json::from_str(piece).unwrap();
            assert_eq!(value["type"], "chunk");
            assert_eq!(value["id"], 7);
            assert_eq!(value["i"], i);
            assert_eq!(value["n"], 3);
            let data = value["data"].as_str().unwrap();
            assert!(data.len() <= CHUNK_BYTES);
            joined.push_str(data);
        }
        assert_eq!(joined, text);
    }
}

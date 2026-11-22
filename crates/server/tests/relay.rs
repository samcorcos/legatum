//! Online play through an in-process fake relay that implements the relay side of
//! `plans/online-play.md`: `POST /rooms`, the binary host tunnel, guest sockets and status.
use axum::{
    Json, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{CloseFrame, Message as AxumMessage, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use legatum_core::{Faction, GameSetup};
use legatum_server::{RelayConfig, RelayHandle, RoomAnnounce, Server, embedded};
use legatum_wasm::json as game_json;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path as FsPath,
    sync::{Arc, Mutex},
    time::Duration,
};
use tempfile::TempDir;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

// ---------------------------------------------------------------------------------------------
// Fake relay

enum ToGuest {
    Text(String),
    Close(u16),
}

struct Tunnel {
    generation: u64,
    to_host: mpsc::UnboundedSender<Vec<u8>>,
    kill: Option<oneshot::Sender<()>>,
}

#[derive(Default)]
struct FakeRoom {
    key: String,
    announce: Value,
    status: Option<Value>,
    summary: Option<Value>,
    statuses: usize,
    summaries: usize,
    tunnels: usize,
    generation: u64,
    host: Option<Tunnel>,
    guests: HashMap<u32, mpsc::UnboundedSender<ToGuest>>,
    next_conn: u32,
}

impl FakeRoom {
    fn close_guests(&mut self, code: u16) {
        for (_, guest) in self.guests.drain() {
            let _ = guest.send(ToGuest::Close(code));
        }
    }
}

#[derive(Default)]
struct Rooms {
    rooms: HashMap<String, FakeRoom>,
    announced: usize,
    byes: Vec<String>,
}

type Shared = Arc<Mutex<Rooms>>;

struct FakeRelay {
    addr: SocketAddr,
    rooms: Shared,
    task: JoinHandle<()>,
}

impl Drop for FakeRelay {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl FakeRelay {
    async fn start() -> Self {
        let rooms = Shared::default();
        let app = Router::new()
            .route("/rooms", post(create_room))
            .route("/rooms/{id}/host", get(host_tunnel))
            .route("/rooms/{id}/ws", get(guest_socket))
            .route("/rooms/{id}/api/status", get(room_status))
            .with_state(rooms.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self { addr, rooms, task }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Waits until `check` sees the rooms in the expected state.
    async fn wait<T>(&self, what: &str, check: impl Fn(&Rooms) -> Option<T>) -> T {
        timeout(Duration::from_secs(20), async {
            loop {
                if let Some(found) = check(&self.rooms.lock().unwrap()) {
                    return found;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {what}"))
    }

    async fn summary(&self, room: &str, check: impl Fn(&Value) -> bool) -> Value {
        self.wait("summary", |rooms| {
            rooms.rooms[room]
                .summary
                .clone()
                .filter(|summary| check(summary))
        })
        .await
    }

    /// Drops the host's tunnel without a closing handshake, like a lost network.
    fn drop_tunnel(&self, room: &str) {
        let mut rooms = self.rooms.lock().unwrap();
        let kill = rooms
            .rooms
            .get_mut(room)
            .unwrap()
            .host
            .as_mut()
            .unwrap()
            .kill
            .take();
        kill.unwrap().send(()).unwrap();
    }

    async fn guest(&self, room: &str) -> Peer {
        let (socket, _) = connect_async(format!("ws://{}/rooms/{room}/ws", self.addr))
            .await
            .unwrap();
        Peer::new(socket)
    }
}

async fn create_room(State(rooms): State<Shared>, Json(announce): Json<Value>) -> Json<Value> {
    let mut rooms = rooms.lock().unwrap();
    rooms.announced += 1;
    let id = format!("room{:06}", rooms.announced);
    let key = format!("{}{}", "k".repeat(32), rooms.announced);
    rooms.rooms.insert(
        id.clone(),
        FakeRoom {
            key: key.clone(),
            announce,
            ..FakeRoom::default()
        },
    );
    Json(json!({"room": id, "host_key": key}))
}

async fn room_status(State(rooms): State<Shared>, Path(id): Path<String>) -> Response {
    let rooms = rooms.lock().unwrap();
    match rooms.rooms.get(&id) {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(room) => match (&room.host, &room.status) {
            (Some(_), Some(status)) => {
                let mut status = status.clone();
                status["is_local"] = json!(false);
                Json(status).into_response()
            }
            _ => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        },
    }
}

async fn host_tunnel(
    State(rooms): State<Shared>,
    Path(id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> Response {
    match rooms.lock().unwrap().rooms.get(&id) {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(room) if query.get("key") != Some(&room.key) => {
            return StatusCode::FORBIDDEN.into_response();
        }
        Some(_) => {}
    }
    ws.on_upgrade(move |socket| host_session(rooms, id, socket))
}

async fn host_session(rooms: Shared, id: String, mut socket: WebSocket) {
    let (to_host, mut outgoing) = mpsc::unbounded_channel::<Vec<u8>>();
    let (kill, mut killed) = oneshot::channel();
    let generation = {
        let mut rooms = rooms.lock().unwrap();
        let Some(room) = rooms.rooms.get_mut(&id) else {
            return;
        };
        room.close_guests(1012);
        room.generation += 1;
        room.tunnels += 1;
        room.host = Some(Tunnel {
            generation: room.generation,
            to_host,
            kill: Some(kill),
        });
        room.generation
    };
    loop {
        tokio::select! {
            _ = &mut killed => break,
            Some(bytes) = outgoing.recv() => {
                if socket.send(AxumMessage::Binary(bytes.into())).await.is_err() { break; }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(AxumMessage::Binary(bytes))) => host_frame(&rooms, &id, &bytes),
                Some(Ok(_)) => {}
                _ => break,
            },
        }
    }
    let mut rooms = rooms.lock().unwrap();
    if let Some(room) = rooms.rooms.get_mut(&id)
        && room
            .host
            .as_ref()
            .is_some_and(|t| t.generation == generation)
    {
        room.host = None;
        room.close_guests(1012);
    }
}

fn host_frame(rooms: &Shared, id: &str, bytes: &[u8]) {
    let mut rooms = rooms.lock().unwrap();
    let conn = u32::from_be_bytes(bytes[..4].try_into().unwrap());
    let payload = &bytes[5..];
    let kind = bytes[4];
    if kind == 5 {
        assert_eq!(conn, 0);
        if let Some(mut room) = rooms.rooms.remove(id) {
            room.close_guests(1000);
        }
        rooms.byes.push(id.to_owned());
        return;
    }
    let Some(room) = rooms.rooms.get_mut(id) else {
        return;
    };
    match kind {
        1 => {
            let text = std::str::from_utf8(payload).unwrap().to_owned();
            assert!(
                text.len() <= 1024 * 1024,
                "host-to-guest message over 1 MiB"
            );
            if let Some(guest) = room.guests.get(&conn) {
                let _ = guest.send(ToGuest::Text(text));
            }
        }
        2 => {
            if let Some(guest) = room.guests.remove(&conn) {
                let _ = guest.send(ToGuest::Close(1000));
            }
        }
        3 => {
            assert_eq!(conn, 0);
            room.status = Some(serde_json::from_slice(payload).unwrap());
            room.statuses += 1;
        }
        4 => {
            assert_eq!(conn, 0);
            room.summary = Some(serde_json::from_slice(payload).unwrap());
            room.summaries += 1;
        }
        other => panic!("unexpected tunnel frame kind {other}"),
    }
}

fn tunnel_frame(conn: u32, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = conn.to_be_bytes().to_vec();
    bytes.push(kind);
    bytes.extend_from_slice(payload);
    bytes
}

async fn guest_socket(
    State(rooms): State<Shared>,
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| guest_session(rooms, id, socket))
}

async fn guest_session(rooms: Shared, id: String, mut socket: WebSocket) {
    let (to_guest, mut outgoing) = mpsc::unbounded_channel();
    let attached = {
        let mut rooms = rooms.lock().unwrap();
        rooms.rooms.get_mut(&id).and_then(|room| {
            let to_host = room.host.as_ref()?.to_host.clone();
            room.next_conn += 1;
            room.guests.insert(room.next_conn, to_guest);
            Some((room.next_conn, to_host))
        })
    };
    let Some((conn, to_host)) = attached else {
        let _ = socket
            .send(AxumMessage::Close(Some(CloseFrame {
                code: 1013,
                reason: "".into(),
            })))
            .await;
        return;
    };
    let _ = to_host.send(tunnel_frame(conn, 0, b""));
    loop {
        tokio::select! {
            out = outgoing.recv() => match out {
                Some(ToGuest::Text(text)) => {
                    if socket.send(AxumMessage::Text(text.into())).await.is_err() { break; }
                }
                closing => {
                    let code = if let Some(ToGuest::Close(code)) = closing { code } else { 1000 };
                    let _ = socket
                        .send(AxumMessage::Close(Some(CloseFrame { code, reason: "".into() })))
                        .await;
                    return;
                }
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(AxumMessage::Text(text))) => {
                    let _ = to_host.send(tunnel_frame(conn, 1, text.as_bytes()));
                }
                Some(Ok(_)) => {}
                _ => break,
            },
        }
    }
    if let Some(room) = rooms.lock().unwrap().rooms.get_mut(&id) {
        room.guests.remove(&conn);
    }
    let _ = to_host.send(tunnel_frame(conn, 2, b""));
}

// ---------------------------------------------------------------------------------------------
// Host and clients

struct Host {
    addr: SocketAddr,
    server: Server,
    task: JoinHandle<()>,
    http: reqwest::Client,
}

impl Drop for Host {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Host {
    async fn start(root: &FsPath) -> Self {
        let static_dir = root.join("static");
        std::fs::create_dir_all(&static_dir).unwrap();
        std::fs::write(static_dir.join("index.html"), "<h1>Legatum</h1>").unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = Server::open_with_archive(
            root.join("saves"),
            addr.port(),
            vec![],
            root.join("archive"),
            5,
        )
        .unwrap();
        let app = server.clone().router(static_dir);
        let task = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .unwrap();
        });
        Self {
            addr,
            server,
            task,
            http: reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    async fn relay(&self, relay: &FakeRelay) -> RelayHandle {
        self.server
            .run_relay(RelayConfig {
                url: relay.url(),
                announce: RoomAnnounce::new("Sam", "test-build"),
            })
            .await
            .unwrap()
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    async fn post(&self, path: &str, body: &Value) -> Value {
        self.http
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn create(&self, setup: &GameSetup) -> Value {
        self.post("/api/game", &game_json::setup_to_value(setup))
            .await
    }

    async fn peer(&self) -> Peer {
        let (socket, _) = connect_async(format!("ws://{}/ws", self.addr))
            .await
            .unwrap();
        Peer::new(socket)
    }
}

struct Peer {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    game_id: String,
    turn: u32,
    token: String,
    /// Reassembled texts that arrived in more than one chunk.
    chunked: Vec<String>,
    /// The raw text of the last message returned by `next`.
    last_text: String,
}

impl Peer {
    fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            socket,
            game_id: String::new(),
            turn: 0,
            token: String::new(),
            chunked: Vec::new(),
            last_text: String::new(),
        }
    }

    async fn send(&mut self, value: Value) {
        self.socket
            .send(Message::Text(value.to_string().into()))
            .await
            .unwrap();
    }

    async fn text(&mut self) -> String {
        loop {
            match self.socket.next().await.expect("socket closed").unwrap() {
                Message::Text(text) => return text.to_string(),
                Message::Ping(data) => self.socket.send(Message::Pong(data)).await.unwrap(),
                Message::Pong(_) => {}
                message => panic!("unexpected frame: {message:?}"),
            }
        }
    }

    async fn next(&mut self) -> Value {
        timeout(Duration::from_secs(60), async {
            let mut text = self.text().await;
            let mut value: Value = serde_json::from_str(&text).unwrap();
            if value["type"] == "chunk" {
                // Mirrors app/src/relaySocket.ts: consecutive pieces of one id, in order.
                let id = value["id"].clone();
                let n = value["n"].as_u64().unwrap();
                assert!(n >= 2, "a chunked text has at least two pieces");
                let mut whole = String::new();
                for i in 0..n {
                    if i > 0 {
                        value = serde_json::from_str(&self.text().await).unwrap();
                    }
                    assert_eq!(value["type"], "chunk");
                    assert_eq!(value["id"], id);
                    assert_eq!(value["i"], i);
                    assert_eq!(value["n"], n);
                    whole.push_str(value["data"].as_str().unwrap());
                }
                self.chunked.push(whole.clone());
                text = whole;
                value = serde_json::from_str(&text).unwrap();
            }
            if value["type"] == "welcome" {
                self.game_id = value["game_id"].as_str().unwrap().into();
            }
            if value["type"] == "view" {
                self.turn = value["view"]["turn"].as_u64().unwrap() as u32;
            }
            self.last_text = text;
            value
        })
        .await
        .expect("timed out waiting for a message")
    }

    async fn until(&mut self, kind: &str) -> Value {
        loop {
            let message = self.next().await;
            if message["type"] == kind {
                return message;
            }
            assert_ne!(message["type"], "error", "unexpected error: {message}");
        }
    }

    async fn error(&mut self) -> Value {
        self.until_error().await
    }

    async fn until_error(&mut self) -> Value {
        loop {
            let message = self.next().await;
            if message["type"] == "error" {
                return message;
            }
        }
    }

    /// Reads until the socket closes; returns the close code.
    async fn closed(&mut self) -> Option<u16> {
        timeout(Duration::from_secs(20), async {
            loop {
                match self.socket.next().await {
                    Some(Ok(Message::Close(frame))) => return frame.map(|f| u16::from(f.code)),
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => return None,
                }
            }
        })
        .await
        .expect("socket stayed open")
    }

    async fn room(&mut self, check: impl Fn(&Value) -> bool) -> Value {
        loop {
            let message = self.until("staging").await;
            if check(&message["state"]) {
                return message["state"].clone();
            }
        }
    }

    async fn take(&mut self, seat: usize, name: &str) {
        self.send(json!({"type":"staging_take", "seat":seat, "name":name}))
            .await;
        self.token = self.until("staging_taken").await["token"]
            .as_str()
            .unwrap()
            .to_owned();
        self.room(|s| s["occupants"][seat]["connected"] == true)
            .await;
    }

    async fn game_take(&mut self, seat: usize) {
        self.send(json!({"type":"game_take", "seat":seat})).await;
        self.token = self.until("seat_token").await["token"]
            .as_str()
            .unwrap()
            .to_owned();
    }

    async fn join(&mut self, seat: usize) -> Value {
        self.send(
            json!({"type":"join", "seat":seat, "name":format!("Seat {seat}"), "token":self.token}),
        )
        .await;
        let welcome = self.until("welcome").await;
        let view = self.next().await;
        assert_eq!(view["type"], "view", "the view follows welcome");
        assert_eq!(welcome["seat"], seat);
        view
    }

    async fn ready(&mut self) {
        self.send(json!({"type":"ready", "game_id":self.game_id, "turn":self.turn, "ready":true}))
            .await;
    }
}

async fn connected(relay: &FakeRelay, room: &str) {
    relay
        .wait("the host tunnel", |rooms| {
            let room = rooms.rooms.get(room)?;
            (room.host.is_some() && room.status.is_some() && room.summary.is_some()).then_some(())
        })
        .await;
}

fn two_humans() -> GameSetup {
    let mut setup = GameSetup::standard(41, [Faction::India, Faction::Byzantium]);
    setup.players.truncate(3);
    setup
}

// ---------------------------------------------------------------------------------------------
// Tests

#[tokio::test(flavor = "multi_thread")]
async fn relayed_guest_takes_a_staging_seat_and_plays_a_turn_with_the_loopback_host() {
    let root = TempDir::new().unwrap();
    let relay = FakeRelay::start().await;
    let host = Host::start(root.path()).await;
    let handle = host.relay(&relay).await;
    let room = handle.room();
    connected(&relay, &room).await;
    {
        let rooms = relay.rooms.lock().unwrap();
        let announce = &rooms.rooms[&room].announce;
        assert_eq!(announce["name"], "Sam");
        assert_eq!(announce["build"], "test-build");
        assert_eq!(announce["rules_version"], legatum_core::RULES_VERSION);
        assert_eq!(announce["protocol"], legatum_server::ONLINE_PROTOCOL);
        assert_eq!(
            rooms.rooms[&room].status.as_ref().unwrap()["mode"],
            "server"
        );
    }
    relay
        .summary(&room, |s| {
            *s == json!({"players":0, "humans":0, "open_seats":0, "phase":"staging", "turn":0})
        })
        .await;

    let mut owner = host.peer().await;
    owner
        .send(json!({"type":"staging_watch", "host":true}))
        .await;
    host.post(
        "/api/staging",
        &json!({
            "seed": "77", "width": 24, "height": 20, "turn_limit": 100,
            "seats": [
                {"kind":"host", "name":"Host", "faction":"India", "team":0, "difficulty":"Normal"},
                {"kind":"open", "name":"Guest", "faction":"Byzantium", "team":0, "difficulty":"Normal"},
                {"kind":"ai", "name":"AI", "faction":"Germany", "team":1, "difficulty":"Easy"}
            ]
        }),
    )
    .await;
    owner.take(0, "Host").await;
    relay
        .summary(&room, |s| {
            *s == json!({"players":3, "humans":2, "open_seats":1, "phase":"staging", "turn":0})
        })
        .await;
    // Guests read the pushed status through the relay, never as the local host.
    let status: Value = host
        .http
        .get(format!("{}/rooms/{room}/api/status", relay.url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["is_local"], false);
    assert_eq!(status["staging"]["occupants"][0]["name"], "Host");
    assert!(status["staging"].get("seed").is_none());

    let mut guest = relay.guest(&room).await;
    guest
        .send(json!({"type":"staging_watch", "host":true}))
        .await;
    let watched = guest.until("staging").await;
    assert!(
        watched["state"].get("seed").is_none(),
        "a relayed guest is not loopback"
    );
    guest.take(1, "Guest").await;
    guest
        .send(json!({"type":"staging_ready", "ready":true}))
        .await;
    guest.room(|s| s["occupants"][1]["ready"] == true).await;
    relay.summary(&room, |s| s["open_seats"] == 0).await;

    host.post("/api/staging/start", &json!({})).await;
    let mut views = Vec::new();
    for (seat, peer) in [&mut owner, &mut guest].into_iter().enumerate() {
        let started = peer.until("staging_started").await;
        assert_eq!(started["seat"], seat);
        peer.token = started["token"].as_str().unwrap().to_owned();
        views.push(peer.join(seat).await);
    }
    assert_eq!(views[1]["view"]["me"], 1);
    for peer in [&mut owner, &mut guest] {
        peer.ready().await;
    }
    for (seat, peer) in [&mut owner, &mut guest].into_iter().enumerate() {
        let turn = peer.until("turn").await;
        assert_eq!(turn["turn"], 2);
        let view = peer.next().await;
        assert_eq!(view["type"], "view");
        assert_eq!(view["view"]["me"], seat);
    }
    relay
        .summary(&room, |s| {
            *s == json!({"players":3, "humans":2, "open_seats":0, "phase":"playing", "turn":2})
        })
        .await;
    assert!(guest.chunked.is_empty(), "small messages are never chunked");

    // Closing the room says bye; the relay drops it and its guests.
    handle.stop().await;
    relay
        .wait("bye", |rooms| rooms.byes.contains(&room).then_some(()))
        .await;
    assert_eq!(guest.closed().await, Some(1000));
}

#[tokio::test(flavor = "multi_thread")]
async fn relayed_guests_are_not_loopback_and_share_one_peer_quota() {
    let root = TempDir::new().unwrap();
    let relay = FakeRelay::start().await;
    let host = Host::start(root.path()).await;
    let handle = host.relay(&relay).await;
    let room = handle.room();
    connected(&relay, &room).await;
    host.create(&two_humans()).await;
    relay
        .summary(&room, |s| s["phase"] == "playing" && s["open_seats"] == 2)
        .await;

    let mut guest = relay.guest(&room).await;
    guest.send(json!({"type":"game_take", "seat":0})).await;
    assert!(
        guest.error().await["message"]
            .as_str()
            .unwrap()
            .contains("reserved")
    );
    guest.game_take(1).await;
    guest.join(1).await;
    guest.send(json!({"type":"game_release", "seat":0})).await;
    assert!(
        guest.error().await["message"]
            .as_str()
            .unwrap()
            .contains("loopback host")
    );
    relay.summary(&room, |s| s["open_seats"] == 1).await;

    // Every relayed guest counts against one admission peer: eight sockets at most.
    let mut extra = Vec::new();
    for _ in 0..7 {
        let mut watcher = relay.guest(&room).await;
        watcher.send(json!({"type":"ping"})).await;
        assert_eq!(watcher.until("pong").await["type"], "pong");
        extra.push(watcher);
    }
    let mut refused = relay.guest(&room).await;
    assert_eq!(refused.closed().await, Some(1000));
    // The loopback host still gets in.
    let mut owner = host.peer().await;
    owner.game_take(0).await;
    owner.join(0).await;
    handle.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_large_view_is_chunked_and_reassembles_to_the_lan_text() {
    let root = TempDir::new().unwrap();
    let relay = FakeRelay::start().await;
    let host = Host::start(root.path()).await;
    let handle = host.relay(&relay).await;
    let room = handle.room();
    connected(&relay, &room).await;
    let mut setup = two_humans();
    setup.width = 128;
    setup.height = 128;
    host.create(&setup).await;

    let mut guest = relay.guest(&room).await;
    guest.game_take(1).await;
    let relayed = guest.join(1).await;
    let relayed_text = guest.last_text.clone();
    assert!(
        relayed_text.len() > legatum_server::relay::CHUNK_BYTES,
        "the view must exceed one chunk: {} bytes",
        relayed_text.len()
    );
    assert_eq!(guest.chunked, std::slice::from_ref(&relayed_text));
    assert_eq!(relayed["view"]["me"], 1);

    // The same seat over the LAN socket receives the identical, unchunked text.
    let mut lan = host.peer().await;
    lan.token = guest.token.clone();
    lan.join(1).await;
    assert!(lan.chunked.is_empty());
    assert_eq!(lan.last_text, relayed_text);
    assert_eq!(guest.until_error().await["code"], "seat_taken");
    handle.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_dropped_tunnel_reconnects_to_the_same_room_and_game() {
    let root = TempDir::new().unwrap();
    let relay = FakeRelay::start().await;
    let host = Host::start(root.path()).await;
    let handle = host.relay(&relay).await;
    let room = handle.room();
    connected(&relay, &room).await;
    host.create(&two_humans()).await;
    let mut guest = relay.guest(&room).await;
    guest.game_take(1).await;
    guest.join(1).await;
    let game_id = guest.game_id.clone();
    relay.summary(&room, |s| s["open_seats"] == 1).await;
    let statuses = relay.rooms.lock().unwrap().rooms[&room].statuses;

    relay.drop_tunnel(&room);
    assert_eq!(guest.closed().await, Some(1012));
    relay
        .wait("the second tunnel", |rooms| {
            let room = &rooms.rooms[&room];
            (room.tunnels == 2 && room.host.is_some() && room.statuses > statuses).then_some(())
        })
        .await;
    assert_eq!(handle.room(), room);
    assert_eq!(relay.rooms.lock().unwrap().announced, 1);

    // The guest's saved credential rejoins the same game through the same room.
    let mut again = relay.guest(&room).await;
    again.token = guest.token.clone();
    again.join(1).await;
    assert_eq!(again.game_id, game_id);

    // A relay that forgot the room gets it announced again.
    let forgotten = relay.rooms.lock().unwrap().rooms.remove(&room).unwrap();
    forgotten.host.unwrap().kill.unwrap().send(()).unwrap();
    let reopened = relay
        .wait("a new room", |rooms| {
            let (id, room) = rooms.rooms.iter().next()?;
            room.host.is_some().then(|| id.clone())
        })
        .await;
    assert_ne!(reopened, room);
    assert_eq!(handle.room(), reopened);
    connected(&relay, &reopened).await;
    relay
        .summary(&reopened, |s| {
            s["phase"] == "playing" && s["open_seats"] == 1
        })
        .await;
    handle.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn embedded_server_serves_assets_on_loopback_and_opens_a_room() {
    let root = TempDir::new().unwrap();
    let relay = FakeRelay::start().await;
    let assets: embedded::Assets = Arc::new(|path: &str| match path {
        "/" => Some((b"<h1>Legatum</h1>".to_vec(), "text/html".into())),
        "/assets/app.js" => Some((b"console.log(1)".to_vec(), "text/javascript".into())),
        _ => None,
    });
    let config = embedded::EmbedConfig {
        saves_dir: root.path().join("saves"),
        archive_dir: root.path().join("archive"),
        assets,
        relay: Some(RelayConfig {
            url: relay.url(),
            announce: RoomAnnounce::new("Sam", "test-build"),
        }),
        bind_lan: false,
    };
    let handle = tokio::task::spawn_blocking(move || embedded::start(config))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(handle.url, format!("http://127.0.0.1:{}/", handle.port));
    let room = handle.room.clone().unwrap();
    connected(&relay, &room).await;
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let get = |path: &str| http.get(format!("{}{path}", handle.url)).send();
    let index = get("").await.unwrap();
    assert_eq!(index.status(), 200);
    assert_eq!(index.headers()["content-type"], "text/html");
    assert_eq!(index.text().await.unwrap(), "<h1>Legatum</h1>");
    assert_eq!(get("assets/app.js").await.unwrap().status(), 200);
    assert_eq!(get("assets/app.js?v=2").await.unwrap().status(), 200);
    assert_eq!(get("missing").await.unwrap().status(), 404);
    assert_eq!(get("assets/").await.unwrap().status(), 404);
    let status: Value = get("api/status").await.unwrap().json().await.unwrap();
    assert_eq!(status["is_local"], true);
    assert_eq!(status["lan_urls"], json!([]));
    let port = handle.port;
    let stopping = std::time::Instant::now();
    tokio::task::spawn_blocking(move || handle.stop())
        .await
        .unwrap();
    assert!(
        stopping.elapsed() < Duration::from_secs(3),
        "stop is bounded"
    );
    relay
        .wait("bye", |rooms| rooms.byes.contains(&room).then_some(()))
        .await;
    assert!(
        TcpStream::connect(("127.0.0.1", port)).await.is_err(),
        "stop closes the listener"
    );
}

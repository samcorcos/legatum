use futures_util::{SinkExt, StreamExt};
use legatum_core::{
    Faction, GameSetup, Order, PlayerId, PlayerView, TechId, UnitKind, player_view,
};
use legatum_server::Server;
use legatum_wasm::{
    json as game_json,
    session::{BrainKind, Session},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, net::SocketAddr, path::Path, sync::Mutex, time::Duration};
use tempfile::TempDir;
use tokio::{net::TcpStream, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, client_async, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

struct Running {
    addr: SocketAddr,
    task: JoinHandle<()>,
    http: reqwest::Client,
    tokens: Mutex<BTreeMap<usize, String>>,
}

#[tokio::test]
async fn archive_upload_requires_own_origin_and_loopback_or_lan_credentials() {
    let root = tempfile::tempdir().unwrap();
    let server = Running::start(root.path()).await;
    let mut setup = GameSetup::standard(23, [Faction::India, Faction::Byzantium]);
    setup.turn_limit = Some(1);
    server.create(&setup).await;
    let (peer, _) = server.join(0).await;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.step(&Default::default());
    let recording = session.history.unwrap();
    let request = || {
        server
            .http
            .post(server.url("/api/archive"))
            .json(&recording)
    };
    assert_eq!(
        request()
            .header("x-test-peer", "192.168.1.20:1234")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        request()
            .header("Origin", "https://evil.example")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        request()
            .header("Host", "evil.example")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        request()
            .header("x-test-peer", "192.168.1.20:1234")
            .bearer_auth("bad-token")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        request()
            .header("x-test-peer", "192.168.1.20:1234")
            .bearer_auth(&peer.token)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(request().send().await.unwrap().status(), 204);
    let index = std::fs::read_to_string(root.path().join("archive/index.jsonl")).unwrap();
    assert_eq!(index.lines().count(), 1);
    assert!(!index.contains(&peer.token));
    let mut invalid = recording;
    invalid.metadata.game_id = "../outside".into();
    assert_eq!(
        server
            .http
            .post(server.url("/api/archive"))
            .json(&invalid)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
}

impl Running {
    async fn start(root: &Path) -> Self {
        Self::start_with_peer(root, None).await
    }

    async fn start_with_peer(root: &Path, peer: Option<SocketAddr>) -> Self {
        Self::start_with_checkpoint_turns(root, peer, 5).await
    }

    async fn start_with_checkpoint_turns(
        root: &Path,
        peer: Option<SocketAddr>,
        checkpoint_turns: u32,
    ) -> Self {
        let static_dir = root.join("static");
        std::fs::create_dir_all(&static_dir).unwrap();
        std::fs::write(static_dir.join("index.html"), "<h1>Legatum</h1>").unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Server::open_with_archive(
            root.join("saves"),
            addr.port(),
            vec![format!("http://192.168.1.2:{}", addr.port())],
            root.join("archive"),
            checkpoint_turns,
        )
        .unwrap()
        .router(static_dir);
        // Exercise the full router with a non-loopback TCP identity without depending
        // on the test machine having a configured LAN interface.
        let app = match peer {
            Some(peer) => app.layer(axum::Extension(axum::extract::ConnectInfo(peer))),
            None => app,
        };
        // Test-only TCP identities allow local admin and hostile LAN peers on one server.
        let app = app.layer(axum::middleware::from_fn(
            |mut request: axum::extract::Request, next: axum::middleware::Next| async move {
                if let Some(peer) = request
                    .headers()
                    .get("x-test-peer")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<SocketAddr>().ok())
                {
                    request
                        .extensions_mut()
                        .insert(axum::extract::ConnectInfo(peer));
                }
                next.run(request).await
            },
        ));
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
            task,
            tokens: Mutex::new(BTreeMap::new()),
            http: reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(20))
                .build()
                .unwrap(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    async fn status(&self) -> Value {
        self.http
            .get(self.url("/api/status"))
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
        self.tokens.lock().unwrap().clear();
        self.http
            .post(self.url("/api/game"))
            .json(&game_json::setup_to_value(setup))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn save(&self) -> String {
        self.http
            .get(self.url("/api/save"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    async fn peer(&self) -> Peer {
        self.peer_as(None).await
    }

    async fn peer_as(&self, ip: Option<&str>) -> Peer {
        let (socket, _) = connect_async(self.ws_request(ip)).await.unwrap();
        Peer {
            socket,
            game_id: String::new(),
            turn: 0,
            token: String::new(),
            recv_timeout: Duration::from_secs(20),
        }
    }

    fn ws_request(&self, ip: Option<&str>) -> tokio_tungstenite::tungstenite::http::Request<()> {
        let mut request = format!("ws://{}/ws", self.addr)
            .into_client_request()
            .unwrap();
        if let Some(ip) = ip {
            request
                .headers_mut()
                .insert("x-test-peer", format!("{ip}:12345").parse().unwrap());
        }
        request
    }

    async fn join(&self, seat: usize) -> (Peer, PlayerView) {
        let mut peer = self.peer().await;
        let saved = self.tokens.lock().unwrap().get(&seat).cloned();
        peer.token = match saved {
            Some(token) => token,
            None => {
                peer.send(json!({"type":"game_take", "seat":seat})).await;
                let token = peer.until("seat_token").await["token"]
                    .as_str()
                    .unwrap()
                    .to_owned();
                self.tokens.lock().unwrap().insert(seat, token.clone());
                token
            }
        };
        peer.send(json!({"type": "join", "seat": seat, "name": format!("Human {seat}"), "token":peer.token}))
            .await;
        let welcome = peer.until("welcome").await;
        assert_eq!(welcome["seat"], seat);
        assert!(welcome["game_id"].is_string());
        let view = serde_json::from_value(peer.until("view").await["view"].clone()).unwrap();
        (peer, view)
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.task.abort();
    }
}

struct Peer {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    game_id: String,
    turn: u32,
    token: String,
    /// How long `next` waits for a frame; large-game tests raise it to their own budget.
    recv_timeout: Duration,
}

impl Peer {
    async fn close(&mut self) {
        self.socket.close(None).await.unwrap();
        timeout(Duration::from_secs(20), async {
            let mut acknowledged = false;
            while let Some(frame) = self.socket.next().await {
                if matches!(frame.expect("closing handshake failed"), Message::Close(_)) {
                    acknowledged = true;
                }
            }
            assert!(
                acknowledged,
                "server dropped TCP without acknowledging Close"
            );
        })
        .await
        .expect("timed out completing the closing handshake");
    }

    async fn send(&mut self, value: Value) {
        self.socket
            .send(Message::Text(value.to_string().into()))
            .await
            .unwrap();
    }

    async fn next(&mut self) -> Value {
        timeout(self.recv_timeout, async {
            loop {
                match self.socket.next().await.expect("socket closed").unwrap() {
                    Message::Text(text) => {
                        let value: Value = serde_json::from_str(&text).unwrap();
                        if value["type"] == "welcome" {
                            self.game_id = value["game_id"].as_str().unwrap().into();
                        }
                        if value["type"] == "view" {
                            self.turn = value["view"]["turn"].as_u64().unwrap() as u32;
                        }
                        return value;
                    }
                    Message::Ping(data) => self.socket.send(Message::Pong(data)).await.unwrap(),
                    Message::Pong(_) => {}
                    message => panic!("unexpected frame: {message:?}"),
                }
            }
        })
        .await
        .expect("timed out waiting for server message")
    }

    async fn until(&mut self, kind: &str) -> Value {
        loop {
            let message = self.next().await;
            if message["type"] == kind {
                return message;
            }
            assert_ne!(
                message["type"], "error",
                "unexpected server error: {message}"
            );
        }
    }

    async fn seats(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        loop {
            let message = self.until("seats").await;
            if predicate(&message["seats"]) {
                return message["seats"].clone();
            }
        }
    }

    async fn ready(&mut self, ready: bool) {
        self.send(
            json!({"type": "ready", "game_id": self.game_id, "turn": self.turn, "ready": ready}),
        )
        .await;
    }

    async fn orders(&mut self, orders: &[Order]) -> Value {
        self.send(
            json!({"type": "orders", "game_id": self.game_id, "turn": self.turn, "orders": orders}),
        )
        .await;
        self.until("validation").await
    }

    async fn turn(&mut self, expected: u32, me: u8) -> Value {
        let turn = self.until("turn").await;
        assert_eq!(turn["turn"], expected);
        assert!(turn["hash"].as_str().unwrap().parse::<u64>().is_ok());
        // The view must immediately follow this client's turn report.
        let message = self.next().await;
        assert_eq!(message["type"], "view");
        assert_eq!(message["view"]["turn"], expected);
        assert_eq!(message["view"]["me"], me);
        turn
    }
}

fn setup() -> GameSetup {
    GameSetup::standard(u64::MAX - 7, [Faction::India, Faction::Byzantium])
}

fn human_orders(view: &PlayerView) -> Vec<Order> {
    let capital = view.cities.iter().find(|c| c.owner == view.me).unwrap();
    vec![
        Order::SetProduction {
            city: capital.id,
            item: capital.detail.as_ref().unwrap().options[0].item,
        },
        Order::SetResearch {
            tech: if view.me == PlayerId(0) {
                TechId::BronzeWorking
            } else {
                TechId::Irrigation
            },
        },
    ]
}

// Build the cityless state these readiness tests need. New games now begin with capitals,
// so eliminating a seat by disbanding its opening Settler is no longer possible.
async fn remove_capitals(server: &Running, seats: &[PlayerId]) {
    let mut state = game_json::parse_state(&server.save().await).unwrap();
    state.cities.retain(|_, city| !seats.contains(&city.owner));
    for tile in &mut state.map.tiles {
        if tile.owner.is_some_and(|owner| seats.contains(&owner)) {
            tile.owner = None;
            tile.city = None;
        }
    }
    server
        .http
        .post(server.url("/api/load"))
        .header("content-type", "application/json")
        .body(game_json::state_to_json(&state))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn two_seats_resolve_native_turn_with_private_views_and_restartable_autosave() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let status = server.status().await;
    assert_eq!(status["mode"], "server");
    assert_eq!(status["is_local"], true);
    assert_eq!(status["has_game"], false);
    assert_eq!(status["seats"], json!([]));
    assert_eq!(
        status["lan_urls"],
        json!([format!("http://192.168.1.2:{}", server.addr.port())])
    );
    assert_eq!(
        server
            .http
            .get(server.url("/"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "<h1>Legatum</h1>"
    );
    let status = server.create(&setup()).await;
    assert_eq!(status["turn"], 1);
    assert_eq!(status["seats"].as_array().unwrap().len(), 2);
    let (mut a, first_a) = server.join(0).await;
    let (mut b, first_b) = server.join(1).await;
    assert_eq!((first_a.me, first_b.me), (PlayerId(0), PlayerId(1)));
    for peer in [&mut a, &mut b] {
        let seats = peer
            .seats(|s| s[0]["connected"] == true && s[1]["connected"] == true)
            .await;
        assert_eq!(seats[0]["name"], "Human 0");
        assert_eq!(seats[1]["faction"], "Byzantium");
    }
    let orders_a = human_orders(&first_a);
    let orders_b = human_orders(&first_b);
    assert_eq!(a.orders(&orders_a).await["errors"], json!([]));
    assert_eq!(b.orders(&orders_b).await["errors"], json!([]));
    a.seats(|s| s[0]["orders"] == 2 && s[1]["orders"] == 2)
        .await;
    a.ready(true).await;
    b.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false)
        .await;
    assert_eq!(server.status().await["turn"], 1);
    b.ready(true).await;
    let turn_a = a.until("turn").await;
    let view_a = a.next().await;
    let turn_b = b.until("turn").await;
    let view_b = b.next().await;
    assert_eq!(turn_a["turn"], 2);
    assert_eq!(turn_b["turn"], 2);
    assert_eq!(turn_a["hash"], turn_b["hash"]);
    assert!(turn_a["hash"].is_string());
    assert_eq!(turn_a["finished"], false);
    assert_eq!(turn_a["winner"], Value::Null);
    for view in [&view_a, &view_b] {
        assert_eq!(view["type"], "view");
        assert_eq!(view["view"]["turn"], 2);
    }

    // Compare the entire authoritative state and both private views to the existing native
    // Session, including six independently seeded AIs acting on their own PlayerViews.
    let mut native = Session::new(&setup(), |_| BrainKind::Scripted);
    let mut orders = native.ai_orders();
    orders.insert(PlayerId(0), orders_a);
    orders.insert(PlayerId(1), orders_b);
    native.step(&orders);
    assert_eq!(turn_a["hash"], native.state.hash().to_string());
    for (seat, view) in [(0, view_a), (1, view_b)] {
        let received: PlayerView = serde_json::from_value(view["view"].clone()).unwrap();
        assert_eq!(received, player_view(&native.state, PlayerId(seat)));
        assert!(
            received
                .cities
                .iter()
                .all(|c| c.detail.is_none() || c.owner == received.me)
        );
    }
    assert!(
        native
            .state
            .cities_of(PlayerId(0))
            .any(|c| c.producing.is_some())
    );
    a.seats(|s| {
        s[0]["ready"] == false
            && s[1]["ready"] == false
            && s[0]["orders"] == 0
            && s[1]["orders"] == 0
    })
    .await;
    let save = server.save().await;
    assert_eq!(game_json::parse_state(&save).unwrap(), native.state);
    for name in ["autosave.json", "turn-2.json"] {
        assert_eq!(
            std::fs::read_to_string(root.path().join("saves").join(name)).unwrap(),
            save
        );
    }
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(root.path()).await;
    assert_eq!(restarted.status().await["turn"], 2);
    assert_eq!(restarted.save().await, save);
    let (mut a, _) = restarted.join(0).await;
    let (mut b, _) = restarted.join(1).await;
    a.ready(true).await;
    b.ready(true).await;
    let turn = a.turn(3, 0).await;
    b.turn(3, 1).await;
    let orders = native.ai_orders();
    native.step(&orders);
    assert_eq!(
        turn["hash"],
        native.state.hash().to_string(),
        "recreated AI brains continue identically"
    );
}

#[tokio::test]
async fn restarted_host_reclaims_with_explicit_take_after_credential_rejection() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (host, _) = server.join(0).await;
    let expired = host.token.clone();
    drop(host);
    drop(server);

    let restarted = Running::start(root.path()).await;
    let mut host = restarted.peer().await;
    host.send(json!({ "type": "join", "seat": 0, "name": "Host", "token": expired }))
        .await;
    assert_eq!(host.until("error").await["code"], "invalid_seat_credential");
    let status = restarted.status().await;
    assert_eq!(status["seats"][0]["reserved"], true);
    assert_eq!(status["seats"][0]["connected"], false);
    let mut guest = restarted.peer_as(Some("192.0.2.10")).await;
    guest.send(json!({ "type": "game_take", "seat": 0 })).await;
    guest.until("error").await;
    host.send(json!({ "type": "game_take", "seat": 0 })).await;
    let fresh = host.until("seat_token").await["token"].clone();
    assert_ne!(fresh, expired);
    host.send(json!({ "type": "join", "seat": 0, "name": "Host", "token": fresh }))
        .await;
    assert_eq!(host.until("welcome").await["seat"], 0);
    assert_eq!(host.until("view").await["view"]["me"], 0);
}

#[tokio::test]
async fn readiness_waits_for_first_join_orders_clear_ready_and_disconnect_unblocks() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut a, _) = server.join(0).await;
    a.ready(true).await;
    a.seats(|s| s[0]["ready"] == true).await;
    assert_eq!(
        server.status().await["turn"],
        1,
        "a never-joined seat blocks resolution"
    );
    a.orders(&[]).await;
    a.seats(|s| s[0]["ready"] == false).await;
    let (mut b, _) = server.join(1).await;
    b.ready(true).await;
    a.seats(|s| s[1]["ready"] == true).await;
    assert_eq!(
        server.status().await["turn"],
        1,
        "replacing orders cleared seat 0 readiness"
    );
    b.ready(false).await;
    a.seats(|s| s[1]["ready"] == false).await;
    a.ready(true).await;
    a.seats(|s| s[0]["ready"] == true).await;
    b.socket.close(None).await.unwrap();
    a.turn(2, 0).await;
    a.seats(|s| s[1]["connected"] == false).await;
    let (mut b, view) = server.join(1).await;
    assert_eq!(view.turn, 2);
    assert_eq!(view.me, PlayerId(1));
    a.ready(true).await;
    b.ready(true).await;
    a.turn(3, 0).await;
    b.turn(3, 1).await;
}

#[tokio::test]
async fn readiness_ignores_connected_eliminated_seats_without_advancing_for_spectators() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    remove_capitals(&server, &[PlayerId(1)]).await;
    let (mut living, _) = server.join(0).await;
    let (mut eliminated, _) = server.join(1).await;
    living.ready(true).await;
    eliminated.ready(true).await;
    assert_eq!(living.turn(2, 0).await["finished"], false);
    eliminated.turn(2, 1).await;
    let state = game_json::parse_state(&server.save().await).unwrap();
    assert!(!state.player(PlayerId(1)).alive);
    for turn in 3..=4 {
        living.ready(true).await;
        living.turn(turn, 0).await;
        eliminated.turn(turn, 1).await;
        assert_eq!(server.status().await["seats"][1]["ready"], false);
    }

    living.socket.close(None).await.unwrap();
    eliminated.seats(|s| s[0]["connected"] == false).await;
    eliminated.ready(true).await;
    eliminated.send(json!({"type": "ping"})).await;
    eliminated.until("pong").await;
    assert_eq!(
        server.status().await["turn"],
        4,
        "an eliminated spectator cannot advance turns while all living humans are offline"
    );
}

#[tokio::test]
async fn observers_advance_ai_turns_once_every_human_is_eliminated() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    remove_capitals(&server, &[PlayerId(0), PlayerId(1)]).await;
    let (mut a, _) = server.join(0).await;
    let (mut b, _) = server.join(1).await;
    a.ready(true).await;
    b.ready(true).await;
    assert_eq!(a.turn(2, 0).await["finished"], false);
    b.turn(2, 1).await;
    let state = game_json::parse_state(&server.save().await).unwrap();
    assert!(!state.player(PlayerId(0)).alive && !state.player(PlayerId(1)).alive);
    assert!(!state.finished, "AI empires keep playing");
    a.ready(true).await;
    b.ready(true).await;
    a.turn(3, 0).await;
    b.turn(3, 1).await;
}

#[tokio::test]
async fn readiness_ignores_unjoined_eliminated_seats_on_load_and_restart() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut native = Session::new(&setup(), |_| BrainKind::Scripted);
    native
        .state
        .cities
        .retain(|_, city| city.owner != PlayerId(1));
    for tile in &mut native.state.map.tiles {
        if tile.owner == Some(PlayerId(1)) {
            tile.owner = None;
            tile.city = None;
        }
    }
    let orders = native.ai_orders();
    native.step(&orders);
    assert!(!native.state.player(PlayerId(1)).alive);
    assert!(!native.state.finished);
    server
        .http
        .post(server.url("/api/load"))
        .header("content-type", "application/json")
        .body(game_json::state_to_json(&native.state))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let (mut living, _) = server.join(0).await;
    living.ready(true).await;
    living.turn(3, 0).await;
    drop(living);
    drop(server);

    let restarted = Running::start(root.path()).await;
    let (mut living, _) = restarted.join(0).await;
    living.ready(true).await;
    living.turn(4, 0).await;
    let seats = restarted.status().await["seats"].clone();
    assert_eq!(seats[1]["connected"], false);
    assert_eq!(seats[1]["reserved"], true);
}

#[tokio::test]
async fn readiness_after_restart_waits_only_for_connected_living_seats() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut a, _) = server.join(0).await;
    let (mut b, _) = server.join(1).await;
    a.ready(true).await;
    b.ready(true).await;
    a.turn(2, 0).await;
    b.turn(2, 1).await;
    b.socket.close(None).await.unwrap();
    a.seats(|s| s[1]["connected"] == false).await;
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(root.path()).await;
    let (mut a, _) = restarted.join(0).await;
    for turn in 3..=4 {
        a.ready(true).await;
        a.turn(turn, 0).await;
    }
    let (mut b, view) = restarted.join(1).await;
    assert_eq!(view.turn, 4);
    a.ready(true).await;
    a.seats(|s| s[0]["ready"] == true && s[1]["connected"] == true && s[1]["ready"] == false)
        .await;
    assert_eq!(restarted.status().await["turn"], 4);
    b.ready(true).await;
    a.turn(5, 0).await;
    b.turn(5, 1).await;
    a.ready(true).await;
    a.seats(|s| s[0]["ready"] == true).await;
    b.socket.close(None).await.unwrap();
    a.turn(6, 0).await;
}

#[tokio::test]
async fn last_joiner_wins_and_old_disconnect_cannot_detach_successor() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut old, _) = server.join(0).await;
    let (mut teammate, _) = server.join(1).await;
    old.ready(true).await;
    old.seats(|s| s[0]["ready"] == true).await;
    let (mut replacement, _) = server.join(0).await;
    assert_eq!(
        old.until("error").await,
        json!({
            "type": "error",
            "code": "seat_taken",
            "message": "Your seat was claimed by another connection."
        })
    );
    let close = timeout(Duration::from_secs(5), old.socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(close, Message::Close(_)));
    drop(old);
    replacement.send(json!({"type": "ping"})).await;
    replacement.until("pong").await;
    teammate.ready(true).await;
    replacement
        .seats(|s| s[1]["ready"] == true && s[0]["ready"] == false)
        .await;
    assert_eq!(server.status().await["turn"], 1);
    replacement.ready(true).await;
    replacement.turn(2, 0).await;
    teammate.turn(2, 1).await;
    assert_eq!(server.status().await["seats"][0]["connected"], true);
}

#[tokio::test]
async fn ordinary_errors_and_game_replacement_do_not_report_seat_takeover() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut peer, _) = server.join(0).await;
    peer.send(json!({"type": "nonsense"})).await;
    let error = peer.until("error").await;
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("Invalid message")
    );
    assert_ne!(error["code"], "seat_taken");
    peer.send(json!({"type": "ping"})).await;
    peer.until("pong").await;

    server.create(&setup()).await;
    assert_eq!(
        peer.until("error").await,
        json!({"type": "error", "message": "The host replaced the game. Please rejoin."})
    );
    let close = timeout(Duration::from_secs(5), peer.socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(close, Message::Close(_)));
}

#[tokio::test]
async fn ranger_forecasts_use_the_requested_attack_mode() {
    use legatum_core::{AttackMode, Hex, Terrain, forecast_attack_mode, game::spawn_unit};

    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let state = legatum_core::new_game(&setup());
    let (mut state, _) = legatum_core::resolve_turn(
        &state,
        &BTreeMap::from([(
            PlayerId(0),
            vec![Order::DeclareWar {
                against: state.team_of(PlayerId(2)),
                breach: false,
            }],
        )]),
    );
    state.units.clear();
    for tile in &mut state.map.tiles {
        tile.terrain = Terrain::Grassland;
        tile.ruin = false;
    }
    let target = Hex::new(6, 5);
    let ranger = spawn_unit(&mut state, PlayerId(0), UnitKind::Ranger, Hex::new(5, 5));
    spawn_unit(&mut state, PlayerId(2), UnitKind::Soldier, target);
    state.units.get_mut(&ranger).unwrap().hp = 20;
    legatum_core::visibility::update_explored(&mut state);
    let loaded = server
        .http
        .post(server.url("/api/load"))
        .header("content-type", "application/json")
        .body(game_json::state_to_json(&state))
        .send()
        .await
        .unwrap();
    assert_eq!(loaded.status(), reqwest::StatusCode::OK);
    let (mut peer, view) = server.join(0).await;
    for mode in [AttackMode::Melee, AttackMode::Ranged] {
        peer.send(json!({"type": "forecast", "req_id": 7, "unit": ranger, "target": target, "mode": mode})).await;
        let reply = peer.until("forecast").await;
        assert_eq!(reply["req_id"], 7);
        let expected = forecast_attack_mode(&view, ranger, target, mode).unwrap();
        assert_eq!(expected.ranged, mode == AttackMode::Ranged);
        assert_eq!(reply["forecast"], serde_json::to_value(expected).unwrap());
    }
    // Missing/unknown modes are rejected instead of falling back to a different attack.
    for mode in [None, Some("Auto")] {
        let mut request =
            json!({"type": "forecast", "req_id": 8, "unit": ranger, "target": target});
        if let Some(mode) = mode {
            request["mode"] = json!(mode);
        }
        peer.send(request).await;
        assert_eq!(peer.until("error").await["type"], "error");
    }
}

#[tokio::test]
async fn bad_messages_validation_forecasts_and_host_load_preserve_protocol() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut unjoined = server.peer().await;
    unjoined
        .send(json!({"type": "join", "seat": 0, "name": "A", "token":"invalid"}))
        .await;
    assert!(
        unjoined.until("error").await["message"]
            .as_str()
            .unwrap()
            .contains("No game")
    );
    unjoined.send(json!({"type": "ping"})).await;
    unjoined.until("pong").await;
    // Exercise the numeric seed variant too.
    let response = server
        .http
        .post(server.url("/api/game"))
        .json(&GameSetup::standard(
            42,
            [Faction::India, Faction::Byzantium],
        ))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let mut unjoined = server.peer().await;
    for message in [
        json!({"type": "ready", "ready": true}),
        json!({"type": "join", "seat": 2, "name": "AI", "token":"invalid"}),
        json!({"type": "nonsense"}),
    ] {
        unjoined.send(message).await;
        unjoined.until("error").await;
    }
    let (mut a, view) = server.join(0).await;
    let enemy_order = Order::SkipUnit {
        unit: view
            .units
            .iter()
            .find(|u| u.owner == PlayerId(1))
            .unwrap()
            .id,
    };
    assert_eq!(
        a.orders(&[enemy_order]).await["errors"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    a.send(json!({"type": "forecast", "req_id": "example-7", "unit": 999999, "target": {"q": 0, "r": 0}, "mode": "Melee"})).await;
    let forecast = a.until("forecast").await;
    assert_eq!(forecast["req_id"], "example-7");
    assert_eq!(forecast["forecast"], Value::Null);
    let saved = server.save().await;
    let bad = server
        .http
        .post(server.url("/api/load"))
        .header("content-type", "application/json")
        .body("not a save")
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(server.save().await, saved);
    for body in [saved.clone(), serde_json::to_string(&saved).unwrap()] {
        let response = server
            .http
            .post(server.url("/api/load"))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let restored = server.save().await;
        assert_eq!(
            game_json::parse_state(&restored).unwrap(),
            game_json::parse_state(&saved).unwrap()
        );
        let before: Value = serde_json::from_str(&saved).unwrap();
        let after: Value = serde_json::from_str(&restored).unwrap();
        assert_eq!(after["archive"]["tape"], before["archive"]["tape"]);
        assert_ne!(
            after["archive"]["metadata"]["game_id"],
            before["archive"]["metadata"]["game_id"]
        );
        assert_eq!(
            after["archive"]["metadata"]["parent_game_id"],
            before["archive"]["metadata"]["game_id"]
        );
    }
    assert!(
        a.until("error").await["message"]
            .as_str()
            .unwrap()
            .contains("replaced")
    );
    assert_eq!(server.status().await["seats"][0]["connected"], false);
}

#[tokio::test]
async fn autosave_keeps_last_ten_turns_and_finished_games_do_not_advance() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut setup = setup();
    setup.turn_limit = Some(12);
    server.create(&setup).await;
    let (mut a, view_a) = server.join(0).await;
    let (mut b, view_b) = server.join(1).await;
    a.orders(&human_orders(&view_a)).await;
    b.orders(&human_orders(&view_b)).await;
    for turn in 2..=13 {
        a.ready(true).await;
        b.ready(true).await;
        let result = a.turn(turn, 0).await;
        b.turn(turn, 1).await;
        assert_eq!(result["finished"], turn == 13);
    }
    let mut turns: Vec<_> = std::fs::read_dir(root.path().join("saves"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter_map(|name| {
            name.strip_prefix("turn-")
                .and_then(|n| n.strip_suffix(".json"))
                .and_then(|n| n.parse::<u32>().ok())
        })
        .collect();
    turns.sort();
    assert_eq!(turns, (4..=13).collect::<Vec<_>>());
    a.ready(true).await;
    assert!(
        a.until("error").await["message"]
            .as_str()
            .unwrap()
            .contains("finished")
    );
    assert_eq!(server.status().await["turn"], 13);
}

#[tokio::test]
async fn sensitive_routes_reject_untrusted_origin_and_host_without_changing_the_game() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let saved = server.save().await;
    let host = server.addr.to_string();
    let origin = server.url("");
    let wrong_port = if server.addr.port() == 1 { 2 } else { 1 };
    let cases = [
        (host.clone(), "http://evil.example".into()),
        (host.clone(), "null".into()),
        (host.clone(), format!("http://localhost:{wrong_port}")),
        (host.clone(), format!("https://{host}")),
        (host.clone(), format!("{origin}/")),
        (host.clone(), format!("{origin}.evil.example")),
        (host.clone(), format!("{origin} http://evil.example")),
        (
            format!("evil.example:{}", server.addr.port()),
            origin.clone(),
        ),
        (format!("localhost:{wrong_port}"), origin.clone()),
        (
            format!("localhost.evil.example:{}", server.addr.port()),
            origin.clone(),
        ),
        (
            format!("localhost@evil.example:{}", server.addr.port()),
            origin.clone(),
        ),
    ];
    for (host, origin) in cases {
        for (path, method) in [
            ("/api/game", reqwest::Method::POST),
            ("/api/load", reqwest::Method::POST),
            ("/api/save", reqwest::Method::GET),
            ("/api/status", reqwest::Method::GET),
            ("/api/staging", reqwest::Method::POST),
            ("/api/staging", reqwest::Method::DELETE),
            ("/api/staging/start", reqwest::Method::POST),
        ] {
            // The guard rejects before consuming the body. Keep it small so a deliberate
            // early close cannot race a large upload and hide the response in BrokenPipe.
            let body = if path == "/api/game" {
                game_json::setup_to_value(&GameSetup::standard(
                    42,
                    [Faction::India, Faction::Byzantium],
                ))
                .to_string()
            } else {
                "{}".into()
            };
            let response = server
                .http
                .request(method, server.url(path))
                .header("host", &host)
                .header("origin", &origin)
                .header("content-type", "application/json")
                .header("x-forwarded-host", server.addr.to_string())
                .header("x-forwarded-for", "127.0.0.1")
                .body(body)
                .send()
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                reqwest::StatusCode::FORBIDDEN,
                "{path}: Host={host}, Origin={origin}"
            );
        }
        let mut request = format!("ws://{}/ws", server.addr)
            .into_client_request()
            .unwrap();
        request.headers_mut().insert("host", host.parse().unwrap());
        request
            .headers_mut()
            .insert("origin", origin.parse().unwrap());
        let result = client_async(request, TcpStream::connect(server.addr).await.unwrap()).await;
        match result {
            Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
                assert_eq!(response.status(), 403)
            }
            _ => panic!("websocket accepted Host={host}, Origin={origin}"),
        }
    }
    // Omitting Origin does not bypass the Host guard (non-browser clients still send Host).
    assert_eq!(
        server
            .http
            .get(server.url("/api/save"))
            .header("host", "evil.example")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::FORBIDDEN
    );
    for path in ["/api/staging", "/api/staging/start"] {
        for content_type in [None, Some("text/plain")] {
            let mut request = server.http.post(server.url(path));
            if let Some(value) = content_type {
                request = request.header("content-type", value);
            }
            assert_eq!(
                request.send().await.unwrap().status(),
                reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE
            );
        }
    }
    // Multiple origins must not be treated as the first, trusted value.
    assert_eq!(
        server
            .http
            .get(server.url("/api/save"))
            .header("origin", &origin)
            .header("origin", "http://evil.example")
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::FORBIDDEN
    );
    assert_eq!(server.save().await, saved);
}

#[tokio::test]
async fn own_origins_work_and_post_routes_require_json_content_type() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let saved = server.save().await;
    for name in ["localhost", "127.0.0.1", "192.168.1.2"] {
        let host = format!("{name}:{}", server.addr.port());
        let origin = format!("http://{host}");
        let response = server
            .http
            .get(server.url("/api/save"))
            .header("host", &host)
            .header("origin", &origin)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let mut request = format!("ws://{}/ws", server.addr)
            .into_client_request()
            .unwrap();
        request.headers_mut().insert("host", host.parse().unwrap());
        request
            .headers_mut()
            .insert("origin", origin.parse().unwrap());
        let (mut socket, _) = client_async(request, TcpStream::connect(server.addr).await.unwrap())
            .await
            .unwrap();
        socket.close(None).await.unwrap();
    }
    for (path, body) in [
        ("/api/game", game_json::setup_to_value(&setup()).to_string()),
        ("/api/load", saved.clone()),
    ] {
        for content_type in [
            None,
            Some("text/plain"),
            Some("application/x-www-form-urlencoded"),
            Some("application/jsonx"),
            Some("application/problem+json"),
        ] {
            let mut request = server.http.post(server.url(path)).body("{}");
            if let Some(content_type) = content_type {
                request = request.header("content-type", content_type);
            }
            assert_eq!(
                request.send().await.unwrap().status(),
                reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "{path}: {content_type:?}"
            );
        }
        for content_type in ["application/json", "application/json; charset=utf-8"] {
            assert!(
                server
                    .http
                    .post(server.url(path))
                    .header("origin", server.url(""))
                    .header("content-type", content_type)
                    .body(body.clone())
                    .send()
                    .await
                    .unwrap()
                    .status()
                    .is_success()
            );
        }
    }
}

#[tokio::test]
async fn welcome_restores_retained_orders_after_disconnect_and_takeover() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut old, view) = server.join(0).await;
    let (mut teammate, _) = server.join(1).await;
    let unit = view.units.iter().find(|u| u.owner == view.me).unwrap().id;
    let orders = [Order::DisbandUnit { unit }];
    assert_eq!(old.orders(&orders).await["errors"], json!([]));
    old.socket.close(None).await.unwrap();
    teammate.seats(|s| s[0]["connected"] == false).await;
    let mut rejoined = server.peer().await;
    let token = old.token.clone();
    rejoined
        .send(json!({"type": "join", "seat": 0, "name": "Rejoined", "token":token}))
        .await;
    let welcome = rejoined.until("welcome").await;
    assert_eq!(welcome["orders"], json!(orders));
    assert_eq!(rejoined.next().await["type"], "view");
    let mut replacement = server.peer().await;
    replacement
        .send(json!({"type": "join", "seat": 0, "name": "Replacement", "token":token}))
        .await;
    let welcome = replacement.until("welcome").await;
    assert_eq!(welcome["orders"], json!(orders));
    assert_eq!(replacement.next().await["type"], "view");
    replacement.orders(&[]).await;
    replacement.ready(true).await;
    teammate.ready(true).await;
    replacement.turn(2, 0).await;
    assert!(
        game_json::parse_state(&server.save().await)
            .unwrap()
            .units
            .contains_key(&unit)
    );
}

#[tokio::test]
async fn failed_game_replacement_leaves_live_and_restart_state_unchanged() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let saved = server.save().await;
    let snapshot = root.path().join("saves/turn-1.json");
    std::fs::remove_file(&snapshot).unwrap();
    std::fs::create_dir(&snapshot).unwrap();
    let replacement = game_json::setup_to_value(&GameSetup::standard(
        42,
        [Faction::India, Faction::Byzantium],
    ));
    assert_eq!(
        server
            .http
            .post(server.url("/api/game"))
            .json(&replacement)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    assert_eq!(server.save().await, saved);
    drop(server);
    let restarted = Running::start(root.path()).await;
    assert_eq!(restarted.save().await, saved);
}

#[tokio::test]
async fn trusted_headers_do_not_grant_lan_peers_admin_access() {
    let root = TempDir::new().unwrap();
    let server =
        Running::start_with_peer(root.path(), Some("192.168.1.9:1234".parse().unwrap())).await;
    for (path, method) in [
        ("/api/game", reqwest::Method::POST),
        ("/api/load", reqwest::Method::POST),
        ("/api/save", reqwest::Method::GET),
        ("/api/staging", reqwest::Method::POST),
        ("/api/staging", reqwest::Method::DELETE),
        ("/api/staging/start", reqwest::Method::POST),
    ] {
        let response = server
            .http
            .request(method, server.url(path))
            .header("origin", server.url(""))
            .header("x-forwarded-for", "127.0.0.1")
            .json(&game_json::setup_to_value(&setup()))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN, "{path}");
    }
    assert_eq!(server.status().await["is_local"], false);
    let mut peer = server.peer().await;
    peer.send(json!({"type": "ping"})).await;
    peer.until("pong").await;
}

#[tokio::test]
async fn unknown_move_targets_validate_and_fail_generically_at_resolution() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut a, view) = server.join(0).await;
    let (mut b, _) = server.join(1).await;
    let state = game_json::parse_state(&server.save().await).unwrap();
    let target = state
        .map
        .hexes()
        .find(|&h| view.tile(h).unwrap().terrain.is_none() && state.map.move_cost(h).is_none())
        .unwrap();
    let unit = view.units.iter().find(|u| u.owner == view.me).unwrap().id;
    assert_eq!(
        a.orders(&[Order::MoveUnit { unit, to: target }]).await["errors"],
        json!([])
    );
    assert_eq!(server.status().await["turn"], 1);
    a.ready(true).await;
    b.ready(true).await;
    let turn = a.turn(2, 0).await;
    let failures: Vec<_> = turn["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "OrderFailed" && e["player"] == view.me.0)
        .collect();
    assert_eq!(
        failures,
        [
            &json!({"type": "OrderFailed", "player": view.me.0, "reason": format!("Move of unit {} failed", unit.0)})
        ]
    );
}

#[tokio::test]
async fn replayed_turn_frames_cannot_change_pending_orders_or_readiness() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut a, view) = server.join(0).await;
    let (mut b, _) = server.join(1).await;
    let unit = view
        .units
        .iter()
        .find(|u| u.owner == PlayerId(0))
        .unwrap()
        .id;
    let stale_orders = json!({"type": "orders", "game_id": a.game_id, "turn": a.turn, "orders": [{"type": "DisbandUnit", "unit": unit}]});
    let stale_ready = json!({"type": "ready", "game_id": a.game_id, "turn": a.turn, "ready": true});
    a.ready(true).await;
    b.ready(true).await;
    a.turn(2, 0).await;
    b.turn(2, 1).await;
    a.orders(&[Order::SkipUnit { unit }]).await;
    let before = server.status().await;
    for frame in [
        stale_orders,
        stale_ready,
        json!({"type": "ready", "ready": true}),
    ] {
        a.send(frame).await;
        a.until("error").await;
        assert_eq!(server.status().await, before);
    }
    a.ready(true).await;
    b.ready(true).await;
    a.turn(3, 0).await;
    b.turn(3, 1).await;
    assert!(
        game_json::parse_state(&server.save().await)
            .unwrap()
            .units
            .contains_key(&unit)
    );
}

#[tokio::test]
async fn websocket_admission_has_a_cap_and_releases_it_after_disconnect() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut peers = Vec::new();
    for _ in 0..32 {
        let mut peer = server.peer().await;
        peer.send(json!({"type": "ping"})).await;
        peer.until("pong").await;
        peers.push(peer);
    }
    let url = format!("ws://{}/ws", server.addr);
    let error = connect_async(&url).await.unwrap_err();
    let tokio_tungstenite::tungstenite::Error::Http(response) = error else {
        panic!("expected HTTP rejection: {error}")
    };
    assert_eq!(response.status().as_u16(), 503);
    assert_eq!(server.status().await["has_game"], false);
    // A completed closing handshake and TCP EOF acknowledge release of the permit.
    peers.pop().unwrap().close().await;
    let mut replacement = server.peer().await;
    replacement.send(json!({"type": "ping"})).await;
    replacement.until("pong").await;
}

// Stable room slots deliberately interleave humans, AIs and closed seats.
fn room_config() -> Value {
    json!({
        "seed": "18446744073709551615", "width": 24, "height": 20, "turn_limit": 100,
        "seats": [
            {"kind":"host", "name":"Host", "faction":"Random", "team":40, "difficulty":"Normal"},
            {"kind":"ai", "name":"Easy AI", "faction":"Random", "team":90, "difficulty":"Easy"},
            {"kind":"closed", "name":"", "faction":"Random", "team":30, "difficulty":"Normal"},
            {"kind":"open", "name":"Alice", "faction":"Random", "team":40, "difficulty":"Normal"},
            {"kind":"ai", "name":"Hard AI", "faction":"Random", "team":20, "difficulty":"Hard"},
            {"kind":"open", "name":"Bob", "faction":"Random", "team":90, "difficulty":"Normal"}
        ]
    })
}

impl Running {
    async fn configure_room(&self, config: &Value) -> Value {
        self.http
            .post(self.url("/api/staging"))
            .json(config)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn start_room(&self) -> reqwest::Response {
        self.http
            .post(self.url("/api/staging/start"))
            .header("content-type", "application/json")
            .send()
            .await
            .unwrap()
    }
}

impl Peer {
    async fn room(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        loop {
            let message = self.until("staging").await;
            if predicate(&message["state"]) {
                return message["state"].clone();
            }
        }
    }

    async fn take(&mut self, seat: usize, name: &str) {
        self.send(json!({"type":"staging_take", "seat":seat, "name":name, "token":(!self.token.is_empty()).then_some(&self.token)}))
            .await;
        self.token = self.until("staging_taken").await["token"]
            .as_str()
            .unwrap()
            .to_owned();
        self.room(|s| {
            s["occupants"][seat]["name"] == name && s["occupants"][seat]["connected"] == true
        })
        .await;
    }

    async fn room_ready(&mut self, seat: usize) {
        self.send(json!({"type":"staging_ready", "ready":true}))
            .await;
        self.room(|s| s["occupants"][seat]["ready"] == true).await;
    }
}

#[tokio::test]
async fn staging_three_clients_start_in_human_order_and_resolve_native_first_turn() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    assert!(server.status().await["staging"].is_null());
    let mut host = server.peer().await;
    host.send(json!({"type":"staging_watch"})).await;
    let config = room_config();
    let status = server.configure_room(&config).await;
    assert_eq!(status["has_game"], false);
    let mut public_config = config.clone();
    public_config.as_object_mut().unwrap().remove("seed");
    // The room shows the barbarian option, on unless the host turned it off.
    public_config["barbarians"] = json!(true);
    assert_eq!(status["staging"]["config"], public_config);
    host.until("staging").await;
    host.take(0, "Host").await;
    let mut alice = server.peer_as(Some("192.0.2.3")).await;
    alice.take(3, "Alice").await;
    let not_ready = server.start_room().await;
    assert_eq!(not_ready.status(), reqwest::StatusCode::CONFLICT);
    assert_eq!(
        not_ready.json::<Value>().await.unwrap()["code"],
        "not_ready"
    );
    let mut bob = server.peer_as(Some("192.0.2.2")).await;
    bob.take(5, "Bob").await;
    alice
        .send(json!({"type":"staging_update", "name":"Alice II", "faction":"India"}))
        .await;
    alice
        .room(|s| s["occupants"][3]["name"] == "Alice II")
        .await;
    alice.room_ready(3).await;
    bob.room_ready(5).await;
    let status: Value = server
        .start_room()
        .await
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(status["staging"].is_null());
    assert_eq!(status["seats"].as_array().unwrap().len(), 3);
    assert_eq!(status["turn"], 1);
    let state = game_json::parse_state(&server.save().await).unwrap();
    let players = &state.setup.players;
    assert_eq!(state.setup.seed, u64::MAX);
    assert_eq!(
        players.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["Host", "Alice II", "Bob", "Easy AI", "Hard AI"]
    );
    assert_eq!(
        players.iter().map(|p| p.team.0).collect::<Vec<_>>(),
        [0, 0, 1, 1, 2]
    );
    assert_eq!(
        players.iter().map(|p| p.faction).collect::<Vec<_>>(),
        [
            Faction::Aztecs,
            Faction::India,
            Faction::Switzerland,
            Faction::Byzantium,
            Faction::Germany
        ]
    );
    assert_eq!(players[3].difficulty, legatum_core::AiDifficulty::Easy);
    assert_eq!(players[4].difficulty, legatum_core::AiDifficulty::Hard);
    let mut native = Session::new(&state.setup, |_| BrainKind::Scripted);
    let mut orders = native.ai_orders();
    let mut peers = [&mut host, &mut alice, &mut bob];
    let mut game_id = None;
    for (seat, peer) in peers.iter_mut().enumerate() {
        let started = peer.until("staging_started").await;
        assert_eq!(started["seat"], seat);
        if let Some(id) = &game_id {
            assert_eq!(&started["game_id"], id);
        }
        game_id = Some(started["game_id"].clone());
        peer.send(
            json!({"type":"join", "seat":seat, "name":players[seat].name, "token":peer.token}),
        )
        .await;
        let welcome = peer.until("welcome").await;
        assert_eq!(welcome["game_id"], started["game_id"]);
        assert_eq!(welcome["orders"], json!([]));
        let message = peer.next().await;
        assert_eq!(message["type"], "view");
        let view: PlayerView = serde_json::from_value(message["view"].clone()).unwrap();
        assert_eq!(view.me, PlayerId(seat as u8));
        let submitted = human_orders(&view);
        assert_eq!(peer.orders(&submitted).await["errors"], json!([]));
        orders.insert(view.me, submitted);
        peer.ready(true).await;
        if seat < 2 {
            peer.seats(|s| s[seat]["ready"] == true).await;
            assert_eq!(server.status().await["turn"], 1);
        }
    }
    native.step(&orders);
    for (seat, peer) in peers.iter_mut().enumerate() {
        assert_eq!(
            peer.turn(2, seat as u8).await["hash"],
            native.state.hash().to_string()
        );
    }
    assert_eq!(
        game_json::parse_state(&server.save().await).unwrap(),
        native.state
    );
    drop(server);
    let restarted = Running::start(root.path()).await;
    assert_eq!(
        restarted.status().await["seats"].as_array().unwrap().len(),
        3
    );
}

#[tokio::test]
async fn staging_settings_re_ready_reclaim_and_close_preserve_stable_seats_and_running_game() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let saved = server.save().await;
    let mut config = room_config();
    let original = server.configure_room(&config).await;
    assert_eq!(server.save().await, saved);
    let mut host = server.peer().await;
    host.take(0, "Host").await;
    let mut alice = server.peer_as(Some("192.0.2.3")).await;
    let mut bob = server.peer_as(Some("192.0.2.2")).await;
    alice.take(3, "Alice").await;
    bob.take(5, "Bob").await;
    alice.room_ready(3).await;
    bob.room_ready(5).await;
    // Idempotent settings do not unready players; an actual change does.
    let unchanged = server.configure_room(&config).await;
    assert_eq!(unchanged["staging"]["occupants"][3]["ready"], true);
    config["seed"] = json!("6");
    let updated = server.configure_room(&config).await;
    assert_eq!(
        updated["staging"]["room_id"],
        original["staging"]["room_id"]
    );
    assert_eq!(updated["staging"]["occupants"][3]["ready"], false);
    assert_eq!(updated["staging"]["occupants"][5]["ready"], false);
    assert_eq!(
        server.start_room().await.status(),
        reqwest::StatusCode::CONFLICT
    );
    bob.send(json!({"type":"staging_take", "seat":3, "name":"Thief"}))
        .await;
    assert_eq!(bob.until("error").await["code"], "seat_occupied");
    bob.send(json!({"type":"staging_take", "seat":1, "name":"AI thief"}))
        .await;
    assert_eq!(bob.until("error").await["code"], "not_open");
    assert_eq!(
        server.status().await["staging"]["occupants"][5]["name"],
        "Bob"
    );
    alice
        .send(json!({"type":"staging_update", "faction":"Byzantium"}))
        .await;
    alice
        .room(|s| s["occupants"][3]["faction"] == "Byzantium")
        .await;
    alice.socket.close(None).await.unwrap();
    host.room(|s| s["occupants"][3]["connected"] == false).await;
    let mut reloaded = server.peer_as(Some("192.0.2.3")).await;
    reloaded.token = alice.token.clone();
    reloaded.take(3, "Alice reloaded").await;
    let reclaimed = server.status().await;
    assert_eq!(reclaimed["staging"]["occupants"][3]["faction"], "Byzantium");
    assert_eq!(reclaimed["staging"]["occupants"][3]["ready"], false);
    config["seats"][3]["kind"] = json!("closed");
    server.configure_room(&config).await;
    reloaded.until("staging_seat_lost").await;
    let stable = bob
        .room(|s| s["config"]["seats"][3]["kind"] == "closed")
        .await;
    assert!(stable["occupants"][3].is_null());
    assert_eq!(stable["occupants"][5]["name"], "Bob");
    assert_eq!(stable["occupants"][5]["connected"], true);
    bob.send(json!({"type":"staging_leave"})).await;
    bob.room(|s| s["occupants"][5].is_null()).await;
    assert!(
        server
            .http
            .delete(server.url("/api/staging"))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    host.until("staging_closed").await;
    bob.until("staging_closed").await;
    assert!(server.status().await["staging"].is_null());
    assert_eq!(server.save().await, saved);
    reloaded
        .send(json!({"type":"staging_take", "seat":3, "name":"Alice"}))
        .await;
    assert_eq!(reloaded.until("error").await["code"], "no_room");
    assert_eq!(
        server.start_room().await.json::<Value>().await.unwrap()["code"],
        "no_room"
    );
}

#[tokio::test]
async fn invalid_staging_and_failed_start_preserve_room_live_game_and_restart_state() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let saved = server.save().await;
    let mut config = room_config();
    config["seats"][3]["kind"] = json!("closed");
    config["seats"][5]["kind"] = json!("closed");
    let original = server.configure_room(&config).await;
    for (pointer, value) in [
        ("/seed", json!("18446744073709551616")),
        ("/seed", json!("-1")),
        ("/seed", json!(5)),
        ("/width", json!(129)),
        ("/height", json!(0)),
        ("/turn_limit", json!(0)),
        ("/turn_limit", json!(4294967295_u32)),
        ("/seats/1/kind", json!("host")),
        ("/seats/0/kind", json!("open")),
        ("/seats/1/faction", json!("Unknown")),
        ("/seats/1/difficulty", json!("Impossible")),
        ("/seats/0/name", json!("x".repeat(81))),
        ("/seats", json!([])),
    ] {
        let mut invalid = config.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        let response = server
            .http
            .post(server.url("/api/staging"))
            .json(&invalid)
            .send()
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            reqwest::StatusCode::BAD_REQUEST,
            "{pointer}"
        );
        assert_eq!(server.status().await["staging"], original["staging"]);
    }
    let mut host = server.peer().await;
    host.take(0, "Host").await;
    let before_failure = server.status().await;
    let snapshot = root.path().join("saves/turn-1.json");
    std::fs::remove_file(&snapshot).unwrap();
    std::fs::create_dir(&snapshot).unwrap();
    assert_eq!(
        server.start_room().await.status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    assert_eq!(server.status().await, before_failure);
    assert_eq!(server.save().await, saved);
    assert_eq!(
        std::fs::read_to_string(root.path().join("saves/autosave.json")).unwrap(),
        saved
    );
    host.send(json!({"type":"ping"})).await;
    host.until("pong").await;
}

#[tokio::test]
async fn direct_host_path_accepts_one_through_eight_humans_and_rejects_interleaved_humans() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    for humans in 1..=8 {
        let mut setup = setup();
        for (i, p) in setup.players.iter_mut().enumerate() {
            p.is_human = i < humans;
        }
        assert_eq!(
            server.create(&setup).await["seats"]
                .as_array()
                .unwrap()
                .len(),
            humans
        );
        let (_peer, view) = server.join(humans - 1).await;
        assert_eq!(view.me, PlayerId((humans - 1) as u8));
    }
    for humans in [
        vec![false; 8],
        vec![true, false, true, false, false, false, false, false],
    ] {
        let mut setup = setup();
        for (p, human) in setup.players.iter_mut().zip(humans) {
            p.is_human = human;
        }
        assert_eq!(
            server
                .http
                .post(server.url("/api/game"))
                .json(&game_json::setup_to_value(&setup))
                .send()
                .await
                .unwrap()
                .status(),
            reqwest::StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test]
async fn staging_one_two_and_eight_humans_can_start_and_finish_the_first_turn() {
    for humans in [1, 2, 8] {
        let root = TempDir::new().unwrap();
        let server = Running::start(root.path()).await;
        let seats: Vec<_> = (0..8).map(|i| json!({
            "kind": if i == 0 { "host" } else if i < humans { "open" } else { "ai" },
            "name": format!("Player {i}"), "faction": "Random", "team": i, "difficulty": "Normal"
        })).collect();
        server
            .configure_room(
                &json!({"seed":"5", "width":40, "height":26, "turn_limit":100, "seats":seats}),
            )
            .await;
        let mut peers = Vec::new();
        for seat in 0..humans {
            let mut peer = server.peer().await;
            peer.take(seat, &format!("Player {seat}")).await;
            if seat != 0 {
                peer.room_ready(seat).await;
            }
            peers.push(peer);
        }
        assert_eq!(server.start_room().await.status(), reqwest::StatusCode::OK);
        for (seat, peer) in peers.iter_mut().enumerate() {
            assert_eq!(peer.until("staging_started").await["seat"], seat);
            peer.send(json!({"type":"join", "seat":seat, "name":format!("Player {seat}"), "token":peer.token}))
                .await;
            peer.until("welcome").await;
            peer.until("view").await;
            peer.ready(true).await;
        }
        for (seat, peer) in peers.iter_mut().enumerate() {
            peer.turn(2, seat as u8).await;
        }
        assert_eq!(
            server.status().await["seats"].as_array().unwrap().len(),
            humans
        );
    }
}

#[tokio::test]
async fn staging_credentials_survive_disconnect_handoff_and_authenticated_takeover() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut config = room_config();
    config["seats"][5]["kind"] = json!("closed");
    server.configure_room(&config).await;
    let mut host = server.peer().await;
    host.take(0, "Host").await;
    let mut alice = server.peer_as(Some("192.0.2.10")).await;
    alice.take(3, "Alice").await;
    let token = alice.token.clone();
    assert_eq!(token.len(), 64);
    assert!(token.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_ne!(token, host.token);
    let mut attacker = server.peer_as(Some("192.0.2.20")).await;
    attacker.send(json!({"type":"staging_watch"})).await;
    attacker.until("staging").await;
    for credential in [Value::Null, json!("forged"), json!(host.token)] {
        attacker
            .send(json!({"type":"staging_take", "seat":3, "name":"Thief", "token":credential}))
            .await;
        assert_eq!(attacker.until("error").await["code"], "seat_occupied");
    }
    alice.socket.close(None).await.unwrap();
    host.room(|s| s["occupants"][3]["connected"] == false).await;
    attacker
        .send(json!({"type":"staging_take", "seat":3, "name":"Thief"}))
        .await;
    assert_eq!(attacker.until("error").await["code"], "seat_reserved");
    let mut resumed = server.peer_as(Some("192.0.2.10")).await;
    resumed.token = token.clone();
    resumed.take(3, "Alice").await;
    assert_eq!(resumed.token, token);
    // The same credential can displace a still-connected stale staging socket.
    let mut replacement = server.peer_as(Some("192.0.2.10")).await;
    replacement.token = token.clone();
    replacement.take(3, "Alice").await;
    assert_eq!(resumed.until("error").await["code"], "seat_taken");
    replacement.room_ready(3).await;
    assert_eq!(server.start_room().await.status(), reqwest::StatusCode::OK);
    let started = replacement.until("staging_started").await;
    assert_eq!(started["seat"], 1); // Stable room slot 3 compacted to game seat 1.
    assert_eq!(started["token"], token);
    // Reservations already exist before any gameplay join, including the host seat.
    for seat in [0, 1] {
        for credential in [Value::Null, json!("forged")] {
            attacker
                .send(json!({"type":"join", "seat":seat, "name":"Thief", "token":credential}))
                .await;
            attacker.until("error").await;
        }
        attacker
            .send(json!({"type":"game_take", "seat":seat}))
            .await;
        attacker.until("error").await;
    }
    replacement.socket.close(None).await.unwrap();
    let mut game_peer = server.peer_as(Some("192.0.2.10")).await;
    // A reload that missed staging_started recovers the compacted seat using the token.
    game_peer
        .send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":token}))
        .await;
    assert_eq!(game_peer.until("staging_started").await["seat"], 1);
    game_peer
        .send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}))
        .await;
    game_peer.until("welcome").await;
    let view = game_peer.until("view").await;
    assert_eq!(view["view"]["me"], 1);
    let mut takeover = server.peer_as(Some("192.0.2.10")).await;
    takeover
        .send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}))
        .await;
    takeover.until("welcome").await;
    takeover.until("view").await;
    assert_eq!(game_peer.until("error").await["code"], "seat_taken");
    attacker
        .send(json!({"type":"join", "seat":1, "name":"Thief", "token":"forged"}))
        .await;
    attacker.until("error").await;
    takeover.send(json!({"type":"ping"})).await;
    takeover.until("pong").await;
    // Loopback explicitly reassigns the seat and invalidates the old credential.
    let mut admin = server.peer().await;
    admin.send(json!({"type":"game_take", "seat":1})).await;
    let fresh = admin.until("seat_token").await["token"].clone();
    assert_ne!(fresh, token);
    assert_eq!(takeover.until("error").await["code"], "seat_taken");
    attacker
        .send(json!({"type":"join", "seat":1, "name":"Old owner", "token":token}))
        .await;
    attacker.until("error").await;
    admin
        .send(json!({"type":"join", "seat":1, "name":"New owner", "token":fresh}))
        .await;
    admin.until("welcome").await;
}

#[tokio::test]
async fn game_release_is_loopback_only_clears_orders_and_invalidates_the_old_token() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    let (mut host, _) = server.join(0).await;
    let mut guest = server.peer_as(Some("192.0.2.10")).await;
    guest.send(json!({"type":"game_take", "seat":1})).await;
    let old_token = guest.until("seat_token").await["token"].clone();
    guest
        .send(json!({"type":"join", "seat":1, "name":"Alice", "token":old_token}))
        .await;
    guest.until("welcome").await;
    let view: PlayerView =
        serde_json::from_value(guest.until("view").await["view"].clone()).unwrap();
    guest.orders(&human_orders(&view)).await;
    guest.ready(true).await;
    let before = host.seats(|s| s[1]["ready"] == true).await;
    let saved = server.save().await;

    // A reconnect that beats the host's confirmation must not be kicked out.
    host.send(json!({"type":"game_release", "seat":1})).await;
    assert!(
        host.until("error").await["message"]
            .as_str()
            .unwrap()
            .contains("connected")
    );
    assert_eq!(server.status().await["seats"], before);
    host.send(json!({"type":"game_release", "seat":8})).await;
    host.until("error").await;
    assert_eq!(server.status().await["seats"], before);

    guest.socket.close(None).await.unwrap();
    let disconnected = host.seats(|s| s[1]["connected"] == false).await;
    assert_eq!(disconnected[1]["reserved"], true);
    assert_eq!(disconnected[1]["orders"], 2);
    let mut newcomer = server.peer_as(Some("192.0.2.20")).await;
    newcomer
        .send(json!({"type":"game_release", "seat":1}))
        .await;
    assert!(
        newcomer.until("error").await["message"]
            .as_str()
            .unwrap()
            .contains("loopback host")
    );
    assert_eq!(server.status().await["seats"], disconnected);
    newcomer.send(json!({"type":"game_take", "seat":1})).await;
    newcomer.until("error").await;

    host.send(json!({"type":"game_release", "seat":1})).await;
    let opened = host.seats(|s| s[1]["reserved"] == false).await;
    assert_eq!(opened[0], before[0]); // The host keeps their own seat.
    assert_eq!(opened[1]["connected"], false);
    assert_eq!(opened[1]["ready"], false);
    assert_eq!(opened[1]["orders"], 0);
    for key in ["seat", "player", "name", "faction"] {
        assert_eq!(opened[1][key], before[1][key]);
    }
    assert_eq!(server.status().await["seats"], opened);
    assert_eq!(server.save().await, saved);
    newcomer
        .send(json!({"type":"join", "seat":1, "name":"Old owner", "token":old_token}))
        .await;
    newcomer.until("error").await;
    newcomer.send(json!({"type":"game_take", "seat":1})).await;
    let token = newcomer.until("seat_token").await["token"].clone();
    assert_ne!(token, old_token);
    newcomer
        .send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}))
        .await;
    let welcome = newcomer.until("welcome").await;
    assert_eq!(welcome["orders"], json!([]));
    assert_eq!(welcome["game_id"], host.game_id);
    assert_eq!(newcomer.until("view").await["view"]["me"], 1);
    let mut old_owner = server.peer_as(Some("192.0.2.10")).await;
    old_owner
        .send(json!({"type":"join", "seat":1, "name":"Old owner", "token":old_token}))
        .await;
    old_owner.until("error").await;
    host.ready(true).await;
    host.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false)
        .await;
    assert_eq!(server.status().await["turn"], 1);
    newcomer.ready(true).await;
    host.turn(2, 0).await;
    newcomer.turn(2, 1).await;
}

#[tokio::test]
async fn game_release_opens_a_seat_locked_after_restart_for_a_lan_guest() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.create(&setup()).await;
    drop(server);
    let server = Running::start(root.path()).await;
    let (mut host, _) = server.join(0).await;
    let mut guest = server.peer_as(Some("192.0.2.10")).await;
    guest.send(json!({"type":"game_take", "seat":1})).await;
    guest.until("error").await;
    host.send(json!({"type":"game_release", "seat":1})).await;
    host.seats(|s| s[1]["reserved"] == false).await;
    // A restored seat stays nonblocking while disconnected, even after release.
    host.ready(true).await;
    host.turn(2, 0).await;
    guest.send(json!({"type":"game_take", "seat":1})).await;
    let token = guest.until("seat_token").await["token"].clone();
    guest
        .send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}))
        .await;
    guest.until("welcome").await;
    let view = guest.until("view").await;
    assert_eq!(view["view"]["me"], 1);
    assert_eq!(view["view"]["turn"], 2);
    host.ready(true).await;
    host.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false)
        .await;
    assert_eq!(server.status().await["turn"], 2);
    guest.ready(true).await;
    host.turn(3, 0).await;
    guest.turn(3, 1).await;
}

#[tokio::test]
async fn staging_release_opens_only_disconnected_network_seats_without_moving_the_host() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.configure_room(&room_config()).await;
    let mut host = server.peer().await;
    host.take(0, "Host").await;
    let mut alice = server.peer_as(Some("192.0.2.10")).await;
    alice.take(3, "Alice").await;
    let old_token = alice.token.clone();
    let mut bob = server.peer_as(Some("192.0.2.20")).await;
    bob.take(5, "Bob").await;
    alice.room_ready(3).await;
    bob.room_ready(5).await;
    let before = host
        .room(|s| s["occupants"][3]["ready"] == true && s["occupants"][5]["ready"] == true)
        .await;

    host.send(json!({"type":"staging_release", "seat":3})).await;
    assert_eq!(host.until("error").await["code"], "seat_connected");
    assert_eq!(server.status().await["staging"], before);
    alice.socket.close(None).await.unwrap();
    let disconnected = host.room(|s| s["occupants"][3]["connected"] == false).await;
    bob.send(json!({"type":"staging_release", "seat":3})).await;
    assert_eq!(bob.until("error").await["code"], "host_only");
    for seat in [0, 1, 2, 8] {
        host.send(json!({"type":"staging_release", "seat":seat}))
            .await;
        assert_eq!(host.until("error").await["code"], "not_releasable");
    }
    assert_eq!(server.status().await["staging"], disconnected);

    host.send(json!({"type":"staging_release", "seat":3})).await;
    let opened = host.room(|s| s["occupants"][3].is_null()).await;
    assert_eq!(opened["config"], before["config"]);
    assert_eq!(opened["occupants"][0], before["occupants"][0]);
    assert_eq!(opened["occupants"][5], before["occupants"][5]);
    assert_eq!(server.status().await["staging"], opened);
    assert!(!server.start_room().await.status().is_success());
    let mut old_owner = server.peer_as(Some("192.0.2.10")).await;
    old_owner
        .send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}))
        .await;
    assert_eq!(old_owner.until("error").await["code"], "seat_expired");
    assert_eq!(server.status().await["staging"], opened);
    let mut newcomer = server.peer_as(Some("192.0.2.30")).await;
    newcomer.take(3, "Alice II").await;
    assert_ne!(newcomer.token, old_token);
    assert_eq!(
        server.status().await["staging"]["occupants"][3]["ready"],
        false
    );
    old_owner
        .send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}))
        .await;
    assert_eq!(old_owner.until("error").await["code"], "seat_occupied");
    newcomer.socket.close(None).await.unwrap();
    host.room(|s| s["occupants"][3]["connected"] == false).await;
    old_owner
        .send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}))
        .await;
    assert_eq!(old_owner.until("error").await["code"], "seat_reserved");
}

#[tokio::test]
async fn only_host_subscriptions_receive_the_seed_and_status_is_always_public() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let config = room_config();
    let status = server.configure_room(&config).await;
    assert!(status["staging"].get("seed").is_none());
    assert!(status["staging"]["config"].get("seed").is_none());
    let mut host = server.peer().await;
    host.send(json!({"type":"staging_watch", "host":true}))
        .await;
    assert_eq!(host.until("staging").await["state"]["seed"], config["seed"]);
    for ip in [None, Some("192.0.2.10")] {
        let mut guest = server.peer_as(ip).await;
        guest
            .send(json!({"type":"staging_watch", "host":ip.is_some()}))
            .await;
        let state = guest.until("staging").await["state"].clone();
        assert!(state.get("seed").is_none());
        assert!(state["config"].get("seed").is_none());
        assert!(!state.to_string().contains(config["seed"].as_str().unwrap()));
    }
    let status = server
        .http
        .get(server.url("/api/status"))
        .header("host", format!("192.168.1.2:{}", server.addr.port()))
        .header(
            "origin",
            format!("http://192.168.1.2:{}", server.addr.port()),
        )
        .header("x-test-peer", "192.0.2.10:12345")
        .send()
        .await
        .unwrap();
    assert_eq!(status.status(), reqwest::StatusCode::OK);
    let status: Value = status.json().await.unwrap();
    assert_eq!(status["is_local"], false);
    assert!(status["staging"].get("seed").is_none());
    assert!(server.status().await["staging"].get("seed").is_none());
}

#[tokio::test]
async fn per_peer_and_watcher_quotas_leave_room_for_the_host() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut peers = Vec::new();
    for ip in ["192.0.2.1", "192.0.2.2"] {
        for _ in 0..8 {
            peers.push(server.peer_as(Some(ip)).await);
        }
        let error = connect_async(server.ws_request(Some(ip)))
            .await
            .unwrap_err();
        let tokio_tungstenite::tungstenite::Error::Http(response) = error else {
            panic!("expected quota rejection")
        };
        assert_eq!(response.status().as_u16(), 503);
    }
    // A third IP cannot bypass the separately bounded unjoined/watcher pool.
    assert!(
        connect_async(server.ws_request(Some("192.0.2.3")))
            .await
            .is_err()
    );
    let mut host = server.peer().await;
    host.send(json!({"type":"ping"})).await;
    host.until("pong").await;
    peers.pop().unwrap().close().await;
    let mut replacement = server.peer_as(Some("192.0.2.2")).await;
    replacement.send(json!({"type":"ping"})).await;
    replacement.until("pong").await;
}

#[tokio::test]
async fn remote_connections_cannot_use_the_four_reserved_loopback_slots() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut setup = setup();
    for player in &mut setup.players {
        player.is_human = true;
    }
    server.create(&setup).await;
    let mut peers = Vec::new();
    for seat in 1..8 {
        let mut peer = server.peer_as(Some(&format!("192.0.2.{seat}"))).await;
        peer.send(json!({"type":"game_take", "seat":seat})).await;
        let token = peer.until("seat_token").await["token"].clone();
        peer.send(json!({"type":"join", "seat":seat, "name":"Player", "token":token}))
            .await;
        peer.until("welcome").await;
        peer.until("view").await;
        peers.push(peer);
    }
    let seats: Vec<_> = (0..8).map(|i| json!({"kind":if i == 0 {"host"} else {"open"}, "name":"Player", "faction":"India", "team":i})).collect();
    server
        .configure_room(
            &json!({"seed":"5", "width":24, "height":20, "turn_limit":100, "seats":seats}),
        )
        .await;
    for seat in 1..8 {
        let mut peer = server
            .peer_as(Some(&format!("192.0.2.{}", seat + 10)))
            .await;
        peer.take(seat, "Player").await;
        peers.push(peer);
    }
    for i in 0..14 {
        peers.push(server.peer_as(Some(&format!("192.0.2.{}", i + 30))).await);
    }
    let error = connect_async(server.ws_request(Some("192.0.2.99")))
        .await
        .unwrap_err();
    let tokio_tungstenite::tungstenite::Error::Http(response) = error else {
        panic!("expected reserved-capacity rejection")
    };
    assert_eq!(response.status().as_u16(), 503);
    for _ in 0..4 {
        peers.push(server.peer().await);
    }
    assert!(connect_async(server.ws_request(None)).await.is_err());
}

#[tokio::test]
async fn heartbeat_traffic_cannot_extend_a_watchers_pre_join_deadline() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    server.configure_room(&room_config()).await;
    let mut watcher = server.peer_as(Some("192.0.2.10")).await;
    watcher.send(json!({"type":"staging_watch"})).await;
    watcher.until("staging").await;
    timeout(Duration::from_secs(35), async {
        let mut heartbeat = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if watcher.socket.send(Message::Ping(vec![1].into())).await.is_err() { break; }
                }
                incoming = watcher.socket.next() => match incoming {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Ping(data))) => { if watcher.socket.send(Message::Pong(data)).await.is_err() { break; } }
                    _ => {}
                }
            }
        }
    }).await.expect("watcher heartbeats kept an unjoined socket alive");
}

// Game completion can be slow in debug builds on a saturated machine. It has a
// separate budget from the latency of each LAN status/seat-list request.
const LARGE_GAME_TIMEOUT: Duration = Duration::from_secs(120);
const STATUS_LATENCY_LIMIT: Duration = Duration::from_secs(1);

async fn timed_lan_status(server: &Running, phase: &str) -> (Duration, Value) {
    let started = std::time::Instant::now();
    let snapshot: Value = server
        .http
        .get(server.url("/api/status"))
        .header("x-test-peer", "192.0.2.10:12345")
        .timeout(STATUS_LATENCY_LIMIT)
        .send()
        .await
        .unwrap_or_else(|e| panic!("status request during {phase}: {e}"))
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let elapsed = started.elapsed();
    // An async timeout alone can miss a synchronous stall on this same executor.
    // Measure through the response body with a wall clock as well.
    assert!(
        elapsed < STATUS_LATENCY_LIMIT,
        "status during {phase} took {elapsed:?}; limit {STATUS_LATENCY_LIMIT:?}"
    );
    assert_eq!(snapshot["is_local"], false);
    (elapsed, snapshot)
}

async fn assert_status_responsive_during<T: Send + 'static>(
    server: &Running,
    phase: &str,
    previous_turn: u32,
    work: impl std::future::Future<Output = T> + Send + 'static,
) -> T {
    let completed = std::cell::Cell::new(false);
    let reads = async {
        let mut samples = 0;
        let mut cached_reads = 0;
        let mut worst = Duration::ZERO;
        loop {
            let (elapsed, status) = timed_lan_status(server, phase).await;
            samples += 1;
            worst = worst.max(elapsed);
            cached_reads += usize::from(status["turn"] == previous_turn);
            // Finish measuring the in-flight request even if work just completed.
            if completed.get() {
                break;
            }
        }
        assert!(
            cached_reads > 0,
            "no pre-commit status sampled during {phase}"
        );
        eprintln!("{phase}: {samples} status reads, {cached_reads} pre-commit, worst {worst:?}");
    };
    let work = async {
        let started = std::time::Instant::now();
        // Decoding/dropping the large turn views is test-client CPU work. Running it
        // on the server's executor would manufacture a status stall in debug builds.
        let client = tokio::task::spawn_blocking(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(work)
        });
        let result = timeout(LARGE_GAME_TIMEOUT, client).await;
        completed.set(true);
        eprintln!("{phase}: work completed in {:?}", started.elapsed());
        result
            .unwrap_or_else(|_| panic!("{phase} exceeded {LARGE_GAME_TIMEOUT:?}"))
            .expect("test client failed")
    };
    // Arm a status request first and keep one in flight throughout the operation.
    // A single-thread executor exposes regressions hidden by spare runtime workers.
    let ((), result) = tokio::join!(biased; reads, work);
    result
}

#[tokio::test(flavor = "current_thread")]
async fn status_stays_responsive_during_large_game_creation_and_turn_resolution() {
    let root = TempDir::new().unwrap();
    // Include archive compression, as well as AI, resolution and autosave, in the turn.
    let server = Running::start_with_checkpoint_turns(root.path(), None, 1).await;
    let mut baseline = Duration::ZERO;
    for _ in 0..8 {
        baseline = baseline.max(timed_lan_status(&server, "baseline").await.0);
    }
    eprintln!("status baseline: worst {baseline:?}; limit {STATUS_LATENCY_LIMIT:?}");
    let mut setup = setup();
    setup.width = 128;
    setup.height = 128;
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let url = server.url("/api/game");
    assert_status_responsive_during(&server, "creation", 0, async move {
        http.post(url)
            .json(&game_json::setup_to_value(&setup))
            .timeout(LARGE_GAME_TIMEOUT)
            .send()
            .await
            .expect("large game creation failed")
            .error_for_status()
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
    })
    .await;
    let (_, status) = timed_lan_status(&server, "created").await;
    assert_eq!(status["has_game"], true);
    assert_eq!(status["turn"], 1);
    assert_eq!(status["seats"].as_array().unwrap().len(), 2);

    let (mut a, a_view) = server.join(0).await;
    let (mut b, b_view) = server.join(1).await;
    // A large game's turn can take longer than the default 20 s frame wait.
    a.recv_timeout = LARGE_GAME_TIMEOUT;
    b.recv_timeout = LARGE_GAME_TIMEOUT;
    a.orders(&human_orders(&a_view)).await;
    b.orders(&human_orders(&b_view)).await;
    a.ready(true).await;
    a.seats(|seats| seats[0]["ready"] == true).await;
    let peers = assert_status_responsive_during(&server, "turn resolution", 1, async move {
        b.ready(true).await;
        a.turn(2, 0).await;
        b.turn(2, 1).await;
        (a, b)
    })
    .await;
    let (_, status) = timed_lan_status(&server, "resolved").await;
    assert_eq!(status["turn"], 2);
    for seat in status["seats"].as_array().unwrap() {
        assert_eq!(seat["connected"], true);
        assert_eq!(seat["ready"], false);
        assert_eq!(seat["orders"], 0);
    }
    drop(peers);
}

#[tokio::test]
async fn research_queue_survives_lan_restart_advances_and_hides_rival_plans() {
    let root = TempDir::new().unwrap();
    let server = Running::start(root.path()).await;
    let mut setup = setup();
    setup.players.truncate(2);
    setup.players[1].team = legatum_core::TeamId(1);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.state.players[0].science_stored = 1000;
    server
        .http
        .post(server.url("/api/load"))
        .header("content-type", "application/json")
        .body(game_json::state_to_json(&session.state))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let (mut a, _) = server.join(0).await;
    let (mut b, _) = server.join(1).await;
    let bad = [Order::SetResearchQueue {
        techs: vec![TechId::Mathematics],
    }];
    assert_eq!(a.orders(&bad).await["errors"].as_array().unwrap().len(), 1);
    let queue = [Order::SetResearchQueue {
        techs: vec![TechId::BronzeWorking, TechId::Writing],
    }];
    assert_eq!(a.orders(&queue).await["errors"], json!([]));
    a.ready(true).await;
    b.ready(true).await;
    let report = a.until("turn").await;
    assert!(
        report["events"]
            .as_array()
            .unwrap()
            .contains(&json!({"type":"ResearchQueueAdvanced", "player":0, "next":"Writing"}))
    );
    let view = a.next().await;
    assert_eq!(view["view"]["research"]["queue"], json!(["Writing"]));
    let rival_report = b.turn(2, 1).await;
    assert!(
        !rival_report["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "ResearchQueueAdvanced")
    );
    let saved = game_json::parse_state(&server.save().await).unwrap();
    assert_eq!(saved.players[0].research_queue, vec![TechId::Writing]);
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(root.path()).await;
    let (mut a, view) = restarted.join(0).await;
    let (mut b, _) = restarted.join(1).await;
    assert_eq!(view.research.queue, vec![TechId::Writing]);
    a.ready(true).await;
    b.ready(true).await;
    let report = a.until("turn").await;
    assert!(
        report["events"]
            .as_array()
            .unwrap()
            .contains(&json!({"type":"ResearchQueueAdvanced", "player":0, "next":null}))
    );
    let view = a.next().await;
    assert_eq!(view["view"]["research"]["queue"], json!([]));
    assert!(view["view"]["research"]["current"].is_null());
    let rival_report = b.turn(3, 1).await;
    assert!(
        !rival_report["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "ResearchQueueAdvanced")
    );
}

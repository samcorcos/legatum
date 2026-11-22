//! The online host's protocol, end to end through `NetHost` as the browser host drives it: the
//! host's own connection is local, every other peer is a guest through the relay. Messages are
//! drained after every step, as `app/src/browserHost.ts` does.
use legatum_core::{
    Faction, GameSetup, Order, PlayerId, PlayerView, TechId, UnitKind, player_view,
};
use legatum_wasm::{
    host::net::{NetHost, PRE_JOIN_TIMEOUT_MS},
    json as game_json,
    session::{BrainKind, Session},
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    rc::Rc,
};

/// What the browser keeps between host sessions: the online autosave and game histories.
#[derive(Default)]
struct Disk {
    save: RefCell<Option<String>>,
    recordings: RefCell<Vec<String>>,
}

impl Disk {
    fn new() -> Rc<Self> {
        Rc::new(Self::default())
    }

    fn autosave(&self) -> String {
        self.save.borrow().clone().expect("an autosave")
    }
}

struct World {
    host: NetHost,
    inboxes: BTreeMap<u64, VecDeque<String>>,
    closed: BTreeSet<u64>,
    now: f64,
    disk: Rc<Disk>,
}

impl World {
    fn pump(&mut self) {
        for out in self.host.drain(self.now) {
            let inbox = self.inboxes.entry(out.conn).or_default();
            inbox.extend(out.texts);
            if out.close {
                self.closed.insert(out.conn);
            }
        }
        if let Some(save) = self.host.take_save() {
            *self.disk.save.borrow_mut() = Some(save);
        }
        self.disk
            .recordings
            .borrow_mut()
            .extend(self.host.take_recordings());
    }
}

struct Running {
    world: Rc<RefCell<World>>,
    tokens: RefCell<BTreeMap<usize, String>>,
}

/// An admin call's answer, shaped like the old HTTP reply.
struct Reply {
    status: u16,
    body: Value,
}

impl Reply {
    fn status(&self) -> u16 {
        self.status
    }

    fn json(&self) -> Value {
        self.body.clone()
    }

    /// The body of a successful reply.
    fn ok_json(&self) -> Value {
        assert_eq!(self.status, 200, "{}", self.body);
        self.body.clone()
    }
}

impl Running {
    fn start(disk: &Rc<Disk>) -> Self {
        Self::start_with_checkpoint_turns(disk, 5)
    }

    fn start_with_checkpoint_turns(disk: &Rc<Disk>, checkpoint_turns: u32) -> Self {
        let save = disk.save.borrow().clone();
        let host = NetHost::with_checkpoints(save.as_deref(), checkpoint_turns).unwrap();
        let world = Rc::new(RefCell::new(World {
            host,
            inboxes: BTreeMap::new(),
            closed: BTreeSet::new(),
            now: 0.0,
            disk: disk.clone(),
        }));
        world.borrow_mut().pump();
        Self {
            world,
            tokens: RefCell::new(BTreeMap::new()),
        }
    }

    /// Runs the host's clock forward.
    fn advance(&self, ms: f64) {
        let mut world = self.world.borrow_mut();
        world.now += ms;
        let now = world.now;
        world.host.tick(now);
        world.pump();
    }

    fn admin(&self, f: impl FnOnce(&mut NetHost) -> Result<(), String>) -> Reply {
        let mut world = self.world.borrow_mut();
        let result = f(&mut world.host);
        world.pump();
        match result {
            Ok(()) => Reply {
                status: 200,
                body: serde_json::to_value(world.host.status(true)).unwrap(),
            },
            Err(message) => Reply {
                status: 400,
                body: json!({"type": "error", "message": message}),
            },
        }
    }

    fn status(&self) -> Value {
        serde_json::to_value(self.world.borrow().host.status(true)).unwrap()
    }

    fn create(&self, setup: &GameSetup) -> Value {
        self.tokens.borrow_mut().clear();
        let setup = game_json::setup_to_value(setup).to_string();
        let reply = self.admin(|host| host.new_game(&setup));
        assert_eq!(reply.status, 200, "{}", reply.body);
        reply.body
    }

    fn try_create(&self, setup: &Value) -> Reply {
        self.admin(|host| host.new_game(&setup.to_string()))
    }

    fn load(&self, save: &str) -> Reply {
        self.admin(|host| host.load_game(save))
    }

    fn save(&self) -> String {
        self.world.borrow().host.save_game().unwrap()
    }

    /// The host's own connection, in its tab.
    fn peer(&self) -> Peer {
        self.connect(true)
    }

    /// A guest through the relay.
    fn guest(&self) -> Peer {
        self.connect(false)
    }

    fn connect(&self, local: bool) -> Peer {
        let mut world = self.world.borrow_mut();
        let now = world.now;
        let conn = world.host.connect(local, now).expect("admitted");
        world.pump();
        Peer {
            world: self.world.clone(),
            conn,
            open: true,
            game_id: String::new(),
            turn: 0,
            token: String::new(),
        }
    }

    fn join(&self, seat: usize) -> (Peer, PlayerView) {
        let mut peer = self.peer();
        let saved = self.tokens.borrow().get(&seat).cloned();
        peer.token = match saved {
            Some(token) => token,
            None => {
                peer.send(json!({"type":"game_take", "seat":seat}));
                let token = peer.until("seat_token")["token"]
                    .as_str()
                    .unwrap()
                    .to_owned();
                self.tokens.borrow_mut().insert(seat, token.clone());
                token
            }
        };
        peer.send(json!({"type": "join", "seat": seat, "name": format!("Human {seat}"), "token":peer.token}));
        let welcome = peer.until("welcome");
        assert_eq!(welcome["seat"], seat);
        assert!(welcome["game_id"].is_string());
        let view = serde_json::from_value(peer.until("view")["view"].clone()).unwrap();
        (peer, view)
    }
}

struct Peer {
    world: Rc<RefCell<World>>,
    conn: u64,
    open: bool,
    game_id: String,
    turn: u32,
    token: String,
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.close();
    }
}

impl Peer {
    /// The peer's socket closes (the relay reports it, or the host's own tab drops it).
    fn close(&mut self) {
        if std::mem::take(&mut self.open) {
            let mut world = self.world.borrow_mut();
            world.host.disconnect(self.conn);
            world.pump();
        }
    }

    fn send(&mut self, value: Value) {
        let mut world = self.world.borrow_mut();
        let now = world.now;
        world.host.receive(self.conn, &value.to_string(), now);
        world.pump();
    }

    fn try_next(&mut self) -> Option<Value> {
        let text = self
            .world
            .borrow_mut()
            .inboxes
            .entry(self.conn)
            .or_default()
            .pop_front()?;
        let value: Value = serde_json::from_str(&text).unwrap();
        if value["type"] == "welcome" {
            self.game_id = value["game_id"].as_str().unwrap().into();
        }
        if value["type"] == "view" {
            self.turn = value["view"]["turn"].as_u64().unwrap() as u32;
        }
        Some(value)
    }

    fn next(&mut self) -> Value {
        self.try_next().expect("no message from the host")
    }

    /// The host closed this connection, and nothing is left to read.
    fn is_closed(&mut self) -> bool {
        let closed = self.world.borrow().closed.contains(&self.conn);
        closed && self.try_next().is_none()
    }

    fn until(&mut self, kind: &str) -> Value {
        loop {
            let message = self.next();
            if message["type"] == kind {
                return message;
            }
            assert_ne!(
                message["type"], "error",
                "unexpected server error: {message}"
            );
        }
    }

    fn seats(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        loop {
            let message = self.until("seats");
            if predicate(&message["seats"]) {
                return message["seats"].clone();
            }
        }
    }

    fn ready(&mut self, ready: bool) {
        self.send(
            json!({"type": "ready", "game_id": self.game_id, "turn": self.turn, "ready": ready}),
        );
    }

    fn orders(&mut self, orders: &[Order]) -> Value {
        self.send(
            json!({"type": "orders", "game_id": self.game_id, "turn": self.turn, "orders": orders}),
        );
        self.until("validation")
    }

    fn turn(&mut self, expected: u32, me: u8) -> Value {
        let turn = self.until("turn");
        assert_eq!(turn["turn"], expected);
        assert!(turn["hash"].as_str().unwrap().parse::<u64>().is_ok());
        // The view must immediately follow this client's turn report.
        let message = self.next();
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
fn remove_capitals(server: &Running, seats: &[PlayerId]) {
    let mut state = game_json::parse_state(&server.save()).unwrap();
    state.cities.retain(|_, city| !seats.contains(&city.owner));
    for tile in &mut state.map.tiles {
        if tile.owner.is_some_and(|owner| seats.contains(&owner)) {
            tile.owner = None;
            tile.city = None;
        }
    }
    assert_eq!(server.load(&game_json::state_to_json(&state)).status(), 200);
}

#[test]
fn two_seats_resolve_native_turn_with_private_views_and_restartable_autosave() {
    let root = Disk::new();
    let server = Running::start(&root);
    let status = server.status();
    assert_eq!(status["mode"], "server");
    assert_eq!(status["is_local"], true);
    assert_eq!(status["has_game"], false);
    assert_eq!(status["seats"], json!([]));
    let status = server.create(&setup());
    assert_eq!(status["turn"], 1);
    assert_eq!(status["seats"].as_array().unwrap().len(), 2);
    let (mut a, first_a) = server.join(0);
    let (mut b, first_b) = server.join(1);
    assert_eq!((first_a.me, first_b.me), (PlayerId(0), PlayerId(1)));
    for peer in [&mut a, &mut b] {
        let seats = peer.seats(|s| s[0]["connected"] == true && s[1]["connected"] == true);
        assert_eq!(seats[0]["name"], "Human 0");
        assert_eq!(seats[1]["faction"], "Byzantium");
    }
    let orders_a = human_orders(&first_a);
    let orders_b = human_orders(&first_b);
    assert_eq!(a.orders(&orders_a)["errors"], json!([]));
    assert_eq!(b.orders(&orders_b)["errors"], json!([]));
    a.seats(|s| s[0]["orders"] == 2 && s[1]["orders"] == 2);
    a.ready(true);
    b.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false);
    assert_eq!(server.status()["turn"], 1);
    b.ready(true);
    let turn_a = a.until("turn");
    let view_a = a.next();
    let turn_b = b.until("turn");
    let view_b = b.next();
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
    });
    let save = server.save();
    assert_eq!(game_json::parse_state(&save).unwrap(), native.state);
    assert_eq!(root.autosave(), save);
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(&root);
    assert_eq!(restarted.status()["turn"], 2);
    assert_eq!(restarted.save(), save);
    let (mut a, _) = restarted.join(0);
    let (mut b, _) = restarted.join(1);
    a.ready(true);
    b.ready(true);
    let turn = a.turn(3, 0);
    b.turn(3, 1);
    let orders = native.ai_orders();
    native.step(&orders);
    assert_eq!(
        turn["hash"],
        native.state.hash().to_string(),
        "recreated AI brains continue identically"
    );
}

#[test]
fn restarted_host_reclaims_with_explicit_take_after_credential_rejection() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (host, _) = server.join(0);
    let expired = host.token.clone();
    drop(host);
    drop(server);

    let restarted = Running::start(&root);
    let mut host = restarted.peer();
    host.send(json!({ "type": "join", "seat": 0, "name": "Host", "token": expired }));
    assert_eq!(host.until("error")["code"], "invalid_seat_credential");
    let status = restarted.status();
    assert_eq!(status["seats"][0]["reserved"], true);
    assert_eq!(status["seats"][0]["connected"], false);
    let mut guest = restarted.guest();
    guest.send(json!({ "type": "game_take", "seat": 0 }));
    guest.until("error");
    host.send(json!({ "type": "game_take", "seat": 0 }));
    let fresh = host.until("seat_token")["token"].clone();
    assert_ne!(fresh, expired);
    host.send(json!({ "type": "join", "seat": 0, "name": "Host", "token": fresh }));
    assert_eq!(host.until("welcome")["seat"], 0);
    assert_eq!(host.until("view")["view"]["me"], 0);
}

#[test]
fn readiness_waits_for_first_join_orders_clear_ready_and_disconnect_unblocks() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut a, _) = server.join(0);
    a.ready(true);
    a.seats(|s| s[0]["ready"] == true);
    assert_eq!(
        server.status()["turn"],
        1,
        "a never-joined seat blocks resolution"
    );
    a.orders(&[]);
    a.seats(|s| s[0]["ready"] == false);
    let (mut b, _) = server.join(1);
    b.ready(true);
    a.seats(|s| s[1]["ready"] == true);
    assert_eq!(
        server.status()["turn"],
        1,
        "replacing orders cleared seat 0 readiness"
    );
    b.ready(false);
    a.seats(|s| s[1]["ready"] == false);
    a.ready(true);
    a.seats(|s| s[0]["ready"] == true);
    b.close();
    a.turn(2, 0);
    a.seats(|s| s[1]["connected"] == false);
    let (mut b, view) = server.join(1);
    assert_eq!(view.turn, 2);
    assert_eq!(view.me, PlayerId(1));
    a.ready(true);
    b.ready(true);
    a.turn(3, 0);
    b.turn(3, 1);
}

#[test]
fn readiness_ignores_connected_eliminated_seats_without_advancing_for_spectators() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    remove_capitals(&server, &[PlayerId(1)]);
    let (mut living, _) = server.join(0);
    let (mut eliminated, _) = server.join(1);
    living.ready(true);
    eliminated.ready(true);
    assert_eq!(living.turn(2, 0)["finished"], false);
    eliminated.turn(2, 1);
    let state = game_json::parse_state(&server.save()).unwrap();
    assert!(!state.player(PlayerId(1)).alive);
    for turn in 3..=4 {
        living.ready(true);
        living.turn(turn, 0);
        eliminated.turn(turn, 1);
        assert_eq!(server.status()["seats"][1]["ready"], false);
    }

    living.close();
    eliminated.seats(|s| s[0]["connected"] == false);
    eliminated.ready(true);
    eliminated.send(json!({"type": "ping"}));
    eliminated.until("pong");
    assert_eq!(
        server.status()["turn"],
        4,
        "an eliminated spectator cannot advance turns while all living humans are offline"
    );
}

#[test]
fn observers_advance_ai_turns_once_every_human_is_eliminated() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    remove_capitals(&server, &[PlayerId(0), PlayerId(1)]);
    let (mut a, _) = server.join(0);
    let (mut b, _) = server.join(1);
    a.ready(true);
    b.ready(true);
    assert_eq!(a.turn(2, 0)["finished"], false);
    b.turn(2, 1);
    let state = game_json::parse_state(&server.save()).unwrap();
    assert!(!state.player(PlayerId(0)).alive && !state.player(PlayerId(1)).alive);
    assert!(!state.finished, "AI empires keep playing");
    a.ready(true);
    b.ready(true);
    a.turn(3, 0);
    b.turn(3, 1);
}

#[test]
fn readiness_ignores_unjoined_eliminated_seats_on_load_and_restart() {
    let root = Disk::new();
    let server = Running::start(&root);
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
    assert_eq!(
        server
            .load(&game_json::state_to_json(&native.state))
            .status(),
        200
    );
    let (mut living, _) = server.join(0);
    living.ready(true);
    living.turn(3, 0);
    drop(living);
    drop(server);

    let restarted = Running::start(&root);
    let (mut living, _) = restarted.join(0);
    living.ready(true);
    living.turn(4, 0);
    let seats = restarted.status()["seats"].clone();
    assert_eq!(seats[1]["connected"], false);
    assert_eq!(seats[1]["reserved"], true);
}

#[test]
fn readiness_after_restart_waits_only_for_connected_living_seats() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut a, _) = server.join(0);
    let (mut b, _) = server.join(1);
    a.ready(true);
    b.ready(true);
    a.turn(2, 0);
    b.turn(2, 1);
    b.close();
    a.seats(|s| s[1]["connected"] == false);
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(&root);
    let (mut a, _) = restarted.join(0);
    for turn in 3..=4 {
        a.ready(true);
        a.turn(turn, 0);
    }
    let (mut b, view) = restarted.join(1);
    assert_eq!(view.turn, 4);
    a.ready(true);
    a.seats(|s| s[0]["ready"] == true && s[1]["connected"] == true && s[1]["ready"] == false);
    assert_eq!(restarted.status()["turn"], 4);
    b.ready(true);
    a.turn(5, 0);
    b.turn(5, 1);
    a.ready(true);
    a.seats(|s| s[0]["ready"] == true);
    b.close();
    a.turn(6, 0);
}

#[test]
fn last_joiner_wins_and_old_disconnect_cannot_detach_successor() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut old, _) = server.join(0);
    let (mut teammate, _) = server.join(1);
    old.ready(true);
    old.seats(|s| s[0]["ready"] == true);
    let (mut replacement, _) = server.join(0);
    assert_eq!(
        old.until("error"),
        json!({
            "type": "error",
            "code": "seat_taken",
            "message": "Your seat was claimed by another connection."
        })
    );
    assert!(old.is_closed());
    drop(old);
    replacement.send(json!({"type": "ping"}));
    replacement.until("pong");
    teammate.ready(true);
    replacement.seats(|s| s[1]["ready"] == true && s[0]["ready"] == false);
    assert_eq!(server.status()["turn"], 1);
    replacement.ready(true);
    replacement.turn(2, 0);
    teammate.turn(2, 1);
    assert_eq!(server.status()["seats"][0]["connected"], true);
}

#[test]
fn ordinary_errors_and_game_replacement_do_not_report_seat_takeover() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut peer, _) = server.join(0);
    peer.send(json!({"type": "nonsense"}));
    let error = peer.until("error");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("Invalid message")
    );
    assert_ne!(error["code"], "seat_taken");
    peer.send(json!({"type": "ping"}));
    peer.until("pong");

    server.create(&setup());
    assert_eq!(
        peer.until("error"),
        json!({"type": "error", "message": "The host replaced the game. Please rejoin."})
    );
    assert!(peer.is_closed());
}

#[test]
fn ranger_forecasts_use_the_requested_attack_mode() {
    use legatum_core::{AttackMode, Hex, Terrain, forecast_attack_mode, game::spawn_unit};

    let root = Disk::new();
    let server = Running::start(&root);
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
    let ranger = spawn_unit(&mut state, PlayerId(0), UnitKind::Archer, Hex::new(5, 5));
    spawn_unit(&mut state, PlayerId(2), UnitKind::Spearman, target);
    state.units.get_mut(&ranger).unwrap().hp = 20;
    legatum_core::visibility::update_explored(&mut state);
    assert_eq!(server.load(&game_json::state_to_json(&state)).status(), 200);
    let (mut peer, view) = server.join(0);
    for mode in [AttackMode::Melee, AttackMode::Ranged] {
        peer.send(json!({"type": "forecast", "req_id": 7, "unit": ranger, "target": target, "mode": mode}));
        let reply = peer.until("forecast");
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
        peer.send(request);
        assert_eq!(peer.until("error")["type"], "error");
    }
}

#[test]
fn bad_messages_validation_forecasts_and_host_load_preserve_protocol() {
    let root = Disk::new();
    let server = Running::start(&root);
    let mut unjoined = server.peer();
    unjoined.send(json!({"type": "join", "seat": 0, "name": "A", "token":"invalid"}));
    assert!(
        unjoined.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("No game")
    );
    unjoined.send(json!({"type": "ping"}));
    unjoined.until("pong");
    // Exercise the numeric seed variant too.
    let response = server.try_create(
        &serde_json::to_value(GameSetup::standard(
            42,
            [Faction::India, Faction::Byzantium],
        ))
        .unwrap(),
    );
    assert_eq!(response.status(), 200);
    let mut unjoined = server.peer();
    for message in [
        json!({"type": "ready", "ready": true}),
        json!({"type": "join", "seat": 2, "name": "AI", "token":"invalid"}),
        json!({"type": "nonsense"}),
    ] {
        unjoined.send(message);
        unjoined.until("error");
    }
    let (mut a, view) = server.join(0);
    let enemy_order = Order::SkipUnit {
        unit: view
            .units
            .iter()
            .find(|u| u.owner == PlayerId(1))
            .unwrap()
            .id,
    };
    assert_eq!(
        a.orders(&[enemy_order])["errors"].as_array().unwrap().len(),
        1
    );
    a.send(json!({"type": "forecast", "req_id": "example-7", "unit": 999999, "target": {"q": 0, "r": 0}, "mode": "Melee"}));
    let forecast = a.until("forecast");
    assert_eq!(forecast["req_id"], "example-7");
    assert_eq!(forecast["forecast"], Value::Null);
    let saved = server.save();
    assert_eq!(server.load("not a save").status(), 400);
    assert_eq!(server.save(), saved);
    for _ in 0..2 {
        assert_eq!(server.load(&saved).status(), 200);
        let restored = server.save();
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
        a.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("replaced")
    );
    assert_eq!(server.status()["seats"][0]["connected"], false);
}

#[test]
fn every_turn_is_saved_and_finished_games_do_not_advance() {
    let root = Disk::new();
    let server = Running::start(&root);
    let mut setup = setup();
    setup.turn_limit = Some(12);
    server.create(&setup);
    let (mut a, view_a) = server.join(0);
    let (mut b, view_b) = server.join(1);
    a.orders(&human_orders(&view_a));
    b.orders(&human_orders(&view_b));
    for turn in 2..=13 {
        a.ready(true);
        b.ready(true);
        let result = a.turn(turn, 0);
        b.turn(turn, 1);
        assert_eq!(result["finished"], turn == 13);
        let saved = game_json::parse_state(&root.autosave()).unwrap();
        assert_eq!(saved.turn, turn);
    }
    // The finished game's history was handed over for the archive.
    let last: Value = serde_json::from_str(root.recordings.borrow().last().unwrap()).unwrap();
    assert!(last["metadata"]["finished_at"].is_string());
    a.ready(true);
    assert!(
        a.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("finished")
    );
    assert_eq!(server.status()["turn"], 13);
}

#[test]
fn welcome_restores_retained_orders_after_disconnect_and_takeover() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut old, view) = server.join(0);
    let (mut teammate, _) = server.join(1);
    let unit = view.units.iter().find(|u| u.owner == view.me).unwrap().id;
    let orders = [Order::DisbandUnit { unit }];
    assert_eq!(old.orders(&orders)["errors"], json!([]));
    old.close();
    teammate.seats(|s| s[0]["connected"] == false);
    let mut rejoined = server.peer();
    let token = old.token.clone();
    rejoined.send(json!({"type": "join", "seat": 0, "name": "Rejoined", "token":token}));
    let welcome = rejoined.until("welcome");
    assert_eq!(welcome["orders"], json!(orders));
    assert_eq!(rejoined.next()["type"], "view");
    let mut replacement = server.peer();
    replacement.send(json!({"type": "join", "seat": 0, "name": "Replacement", "token":token}));
    let welcome = replacement.until("welcome");
    assert_eq!(welcome["orders"], json!(orders));
    assert_eq!(replacement.next()["type"], "view");
    replacement.orders(&[]);
    replacement.ready(true);
    teammate.ready(true);
    replacement.turn(2, 0);
    assert!(
        game_json::parse_state(&server.save())
            .unwrap()
            .units
            .contains_key(&unit)
    );
}

#[test]
fn unknown_move_targets_validate_and_fail_generically_at_resolution() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut a, view) = server.join(0);
    let (mut b, _) = server.join(1);
    let state = game_json::parse_state(&server.save()).unwrap();
    let target = state
        .map
        .hexes()
        .find(|&h| view.tile(h).unwrap().terrain.is_none() && state.map.move_cost(h).is_none())
        .unwrap();
    let unit = view.units.iter().find(|u| u.owner == view.me).unwrap().id;
    assert_eq!(
        a.orders(&[Order::MoveUnit { unit, to: target }])["errors"],
        json!([])
    );
    assert_eq!(server.status()["turn"], 1);
    a.ready(true);
    b.ready(true);
    let turn = a.turn(2, 0);
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

#[test]
fn replayed_turn_frames_cannot_change_pending_orders_or_readiness() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut a, view) = server.join(0);
    let (mut b, _) = server.join(1);
    let unit = view
        .units
        .iter()
        .find(|u| u.owner == PlayerId(0))
        .unwrap()
        .id;
    let stale_orders = json!({"type": "orders", "game_id": a.game_id, "turn": a.turn, "orders": [{"type": "DisbandUnit", "unit": unit}]});
    let stale_ready = json!({"type": "ready", "game_id": a.game_id, "turn": a.turn, "ready": true});
    a.ready(true);
    b.ready(true);
    a.turn(2, 0);
    b.turn(2, 1);
    a.orders(&[Order::SkipUnit { unit }]);
    let before = server.status();
    for frame in [
        stale_orders,
        stale_ready,
        json!({"type": "ready", "ready": true}),
    ] {
        a.send(frame);
        a.until("error");
        assert_eq!(server.status(), before);
    }
    a.ready(true);
    b.ready(true);
    a.turn(3, 0);
    b.turn(3, 1);
    assert!(
        game_json::parse_state(&server.save())
            .unwrap()
            .units
            .contains_key(&unit)
    );
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
    fn configure_room(&self, config: &Value) -> Value {
        let reply = self.try_configure_room(config);
        assert_eq!(reply.status, 200, "{}", reply.body);
        reply.body
    }

    fn try_configure_room(&self, config: &Value) -> Reply {
        self.admin(|host| host.configure_staging(&config.to_string()))
    }

    fn start_room(&self) -> Reply {
        let mut world = self.world.borrow_mut();
        let result = world.host.start_staging();
        world.pump();
        match result {
            Ok(()) => Reply {
                status: 200,
                body: serde_json::to_value(world.host.status(true)).unwrap(),
            },
            Err(e) => Reply {
                status: if e.code.is_some() { 409 } else { 400 },
                body: json!({"type": "error", "code": e.code, "message": e.message}),
            },
        }
    }

    fn close_room(&self) -> Reply {
        self.admin(|host| {
            host.close_staging();
            Ok(())
        })
    }
}

impl Peer {
    fn room(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        loop {
            let message = self.until("staging");
            if predicate(&message["state"]) {
                return message["state"].clone();
            }
        }
    }

    fn take(&mut self, seat: usize, name: &str) {
        self.send(json!({"type":"staging_take", "seat":seat, "name":name, "token":(!self.token.is_empty()).then_some(&self.token)}))
            ;
        self.token = self.until("staging_taken")["token"]
            .as_str()
            .unwrap()
            .to_owned();
        self.room(|s| {
            s["occupants"][seat]["name"] == name && s["occupants"][seat]["connected"] == true
        });
    }

    fn room_ready(&mut self, seat: usize) {
        self.send(json!({"type":"staging_ready", "ready":true}));
        self.room(|s| s["occupants"][seat]["ready"] == true);
    }
}

#[test]
fn staging_three_clients_start_in_human_order_and_resolve_native_first_turn() {
    let root = Disk::new();
    let server = Running::start(&root);
    assert!(server.status()["staging"].is_null());
    let mut host = server.peer();
    host.send(json!({"type":"staging_watch"}));
    let config = room_config();
    let status = server.configure_room(&config);
    assert_eq!(status["has_game"], false);
    let mut public_config = config.clone();
    public_config.as_object_mut().unwrap().remove("seed");
    // The room shows the barbarian option, on unless the host turned it off.
    public_config["barbarians"] = json!(true);
    assert_eq!(status["staging"]["config"], public_config);
    host.until("staging");
    host.take(0, "Host");
    let mut alice = server.guest();
    alice.take(3, "Alice");
    let not_ready = server.start_room();
    assert_eq!(not_ready.status(), 409);
    assert_eq!(not_ready.json()["code"], "not_ready");
    let mut bob = server.guest();
    bob.take(5, "Bob");
    alice.send(json!({"type":"staging_update", "name":"Alice II", "faction":"India"}));
    alice.room(|s| s["occupants"][3]["name"] == "Alice II");
    alice.room_ready(3);
    bob.room_ready(5);
    let status: Value = server.start_room().ok_json();
    assert!(status["staging"].is_null());
    assert_eq!(status["seats"].as_array().unwrap().len(), 3);
    assert_eq!(status["turn"], 1);
    let state = game_json::parse_state(&server.save()).unwrap();
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
        let started = peer.until("staging_started");
        assert_eq!(started["seat"], seat);
        if let Some(id) = &game_id {
            assert_eq!(&started["game_id"], id);
        }
        game_id = Some(started["game_id"].clone());
        peer.send(
            json!({"type":"join", "seat":seat, "name":players[seat].name, "token":peer.token}),
        );
        let welcome = peer.until("welcome");
        assert_eq!(welcome["game_id"], started["game_id"]);
        assert_eq!(welcome["orders"], json!([]));
        let message = peer.next();
        assert_eq!(message["type"], "view");
        let view: PlayerView = serde_json::from_value(message["view"].clone()).unwrap();
        assert_eq!(view.me, PlayerId(seat as u8));
        let submitted = human_orders(&view);
        assert_eq!(peer.orders(&submitted)["errors"], json!([]));
        orders.insert(view.me, submitted);
        peer.ready(true);
        if seat < 2 {
            peer.seats(|s| s[seat]["ready"] == true);
            assert_eq!(server.status()["turn"], 1);
        }
    }
    native.step(&orders);
    for (seat, peer) in peers.iter_mut().enumerate() {
        assert_eq!(
            peer.turn(2, seat as u8)["hash"],
            native.state.hash().to_string()
        );
    }
    assert_eq!(
        game_json::parse_state(&server.save()).unwrap(),
        native.state
    );
    drop(server);
    let restarted = Running::start(&root);
    assert_eq!(restarted.status()["seats"].as_array().unwrap().len(), 3);
}

#[test]
fn staging_settings_re_ready_reclaim_and_close_preserve_stable_seats_and_running_game() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let saved = server.save();
    let mut config = room_config();
    let original = server.configure_room(&config);
    assert_eq!(server.save(), saved);
    let mut host = server.peer();
    host.take(0, "Host");
    let mut alice = server.guest();
    let mut bob = server.guest();
    alice.take(3, "Alice");
    bob.take(5, "Bob");
    alice.room_ready(3);
    bob.room_ready(5);
    // Idempotent settings do not unready players; an actual change does.
    let unchanged = server.configure_room(&config);
    assert_eq!(unchanged["staging"]["occupants"][3]["ready"], true);
    config["seed"] = json!("6");
    let updated = server.configure_room(&config);
    assert_eq!(
        updated["staging"]["room_id"],
        original["staging"]["room_id"]
    );
    assert_eq!(updated["staging"]["occupants"][3]["ready"], false);
    assert_eq!(updated["staging"]["occupants"][5]["ready"], false);
    assert_eq!(server.start_room().status(), 409);
    bob.send(json!({"type":"staging_take", "seat":3, "name":"Thief"}));
    assert_eq!(bob.until("error")["code"], "seat_occupied");
    bob.send(json!({"type":"staging_take", "seat":1, "name":"AI thief"}));
    assert_eq!(bob.until("error")["code"], "not_open");
    assert_eq!(server.status()["staging"]["occupants"][5]["name"], "Bob");
    alice.send(json!({"type":"staging_update", "faction":"Byzantium"}));
    alice.room(|s| s["occupants"][3]["faction"] == "Byzantium");
    alice.close();
    host.room(|s| s["occupants"][3]["connected"] == false);
    let mut reloaded = server.guest();
    reloaded.token = alice.token.clone();
    reloaded.take(3, "Alice reloaded");
    let reclaimed = server.status();
    assert_eq!(reclaimed["staging"]["occupants"][3]["faction"], "Byzantium");
    assert_eq!(reclaimed["staging"]["occupants"][3]["ready"], false);
    config["seats"][3]["kind"] = json!("closed");
    server.configure_room(&config);
    reloaded.until("staging_seat_lost");
    let stable = bob.room(|s| s["config"]["seats"][3]["kind"] == "closed");
    assert!(stable["occupants"][3].is_null());
    assert_eq!(stable["occupants"][5]["name"], "Bob");
    assert_eq!(stable["occupants"][5]["connected"], true);
    bob.send(json!({"type":"staging_leave"}));
    bob.room(|s| s["occupants"][5].is_null());
    assert_eq!(server.close_room().status(), 200);
    host.until("staging_closed");
    bob.until("staging_closed");
    assert!(server.status()["staging"].is_null());
    assert_eq!(server.save(), saved);
    reloaded.send(json!({"type":"staging_take", "seat":3, "name":"Alice"}));
    assert_eq!(reloaded.until("error")["code"], "no_room");
    assert_eq!(server.start_room().json()["code"], "no_room");
}

#[test]
fn invalid_staging_preserves_the_room_live_game_and_autosave() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let saved = server.save();
    let mut config = room_config();
    config["seats"][3]["kind"] = json!("closed");
    config["seats"][5]["kind"] = json!("closed");
    let original = server.configure_room(&config);
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
        let response = server.try_configure_room(&invalid);
        assert_eq!(response.status(), 400, "{pointer}");
        assert_eq!(server.status()["staging"], original["staging"]);
    }
    let mut host = server.peer();
    host.take(0, "Host");
    assert_eq!(server.save(), saved);
    assert_eq!(root.autosave(), saved);
    host.send(json!({"type":"ping"}));
    host.until("pong");
}

#[test]
fn direct_host_path_accepts_one_through_eight_humans_and_rejects_interleaved_humans() {
    let root = Disk::new();
    let server = Running::start(&root);
    for humans in 1..=8 {
        let mut setup = setup();
        for (i, p) in setup.players.iter_mut().enumerate() {
            p.is_human = i < humans;
        }
        assert_eq!(
            server.create(&setup)["seats"].as_array().unwrap().len(),
            humans
        );
        let (_peer, view) = server.join(humans - 1);
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
                .try_create(&game_json::setup_to_value(&setup))
                .status(),
            400
        );
    }
}

#[test]
fn staging_one_two_and_eight_humans_can_start_and_finish_the_first_turn() {
    for humans in [1, 2, 8] {
        let root = Disk::new();
        let server = Running::start(&root);
        let seats: Vec<_> = (0..8).map(|i| json!({
            "kind": if i == 0 { "host" } else if i < humans { "open" } else { "ai" },
            "name": format!("Player {i}"), "faction": "Random", "team": i, "difficulty": "Normal"
        })).collect();
        server.configure_room(
            &json!({"seed":"5", "width":40, "height":26, "turn_limit":100, "seats":seats}),
        );
        let mut peers = Vec::new();
        for seat in 0..humans {
            let mut peer = server.peer();
            peer.take(seat, &format!("Player {seat}"));
            if seat != 0 {
                peer.room_ready(seat);
            }
            peers.push(peer);
        }
        assert_eq!(server.start_room().status(), 200);
        for (seat, peer) in peers.iter_mut().enumerate() {
            assert_eq!(peer.until("staging_started")["seat"], seat);
            peer.send(json!({"type":"join", "seat":seat, "name":format!("Player {seat}"), "token":peer.token}))
                ;
            peer.until("welcome");
            peer.until("view");
            peer.ready(true);
        }
        for (seat, peer) in peers.iter_mut().enumerate() {
            peer.turn(2, seat as u8);
        }
        assert_eq!(server.status()["seats"].as_array().unwrap().len(), humans);
    }
}

#[test]
fn staging_credentials_survive_disconnect_handoff_and_authenticated_takeover() {
    let root = Disk::new();
    let server = Running::start(&root);
    let mut config = room_config();
    config["seats"][5]["kind"] = json!("closed");
    server.configure_room(&config);
    let mut host = server.peer();
    host.take(0, "Host");
    let mut alice = server.guest();
    alice.take(3, "Alice");
    let token = alice.token.clone();
    assert_eq!(token.len(), 64);
    assert!(token.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_ne!(token, host.token);
    let mut attacker = server.guest();
    attacker.send(json!({"type":"staging_watch"}));
    attacker.until("staging");
    for credential in [Value::Null, json!("forged"), json!(host.token)] {
        attacker.send(json!({"type":"staging_take", "seat":3, "name":"Thief", "token":credential}));
        assert_eq!(attacker.until("error")["code"], "seat_occupied");
    }
    alice.close();
    host.room(|s| s["occupants"][3]["connected"] == false);
    attacker.send(json!({"type":"staging_take", "seat":3, "name":"Thief"}));
    assert_eq!(attacker.until("error")["code"], "seat_reserved");
    let mut resumed = server.guest();
    resumed.token = token.clone();
    resumed.take(3, "Alice");
    assert_eq!(resumed.token, token);
    // The same credential can displace a still-connected stale staging socket.
    let mut replacement = server.guest();
    replacement.token = token.clone();
    replacement.take(3, "Alice");
    assert_eq!(resumed.until("error")["code"], "seat_taken");
    replacement.room_ready(3);
    assert_eq!(server.start_room().status(), 200);
    let started = replacement.until("staging_started");
    assert_eq!(started["seat"], 1); // Stable room slot 3 compacted to game seat 1.
    assert_eq!(started["token"], token);
    // Reservations already exist before any gameplay join, including the host seat.
    for seat in [0, 1] {
        for credential in [Value::Null, json!("forged")] {
            attacker.send(json!({"type":"join", "seat":seat, "name":"Thief", "token":credential}));
            attacker.until("error");
        }
        attacker.send(json!({"type":"game_take", "seat":seat}));
        attacker.until("error");
    }
    replacement.close();
    let mut game_peer = server.guest();
    // A reload that missed staging_started recovers the compacted seat using the token.
    game_peer.send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":token}));
    assert_eq!(game_peer.until("staging_started")["seat"], 1);
    game_peer.send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}));
    game_peer.until("welcome");
    let view = game_peer.until("view");
    assert_eq!(view["view"]["me"], 1);
    let mut takeover = server.guest();
    takeover.send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}));
    takeover.until("welcome");
    takeover.until("view");
    assert_eq!(game_peer.until("error")["code"], "seat_taken");
    attacker.send(json!({"type":"join", "seat":1, "name":"Thief", "token":"forged"}));
    attacker.until("error");
    takeover.send(json!({"type":"ping"}));
    takeover.until("pong");
    // Loopback explicitly reassigns the seat and invalidates the old credential.
    let mut admin = server.peer();
    admin.send(json!({"type":"game_take", "seat":1}));
    let fresh = admin.until("seat_token")["token"].clone();
    assert_ne!(fresh, token);
    assert_eq!(takeover.until("error")["code"], "seat_taken");
    attacker.send(json!({"type":"join", "seat":1, "name":"Old owner", "token":token}));
    attacker.until("error");
    admin.send(json!({"type":"join", "seat":1, "name":"New owner", "token":fresh}));
    admin.until("welcome");
}

#[test]
fn game_release_is_loopback_only_clears_orders_and_invalidates_the_old_token() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    let (mut host, _) = server.join(0);
    let mut guest = server.guest();
    guest.send(json!({"type":"game_take", "seat":1}));
    let old_token = guest.until("seat_token")["token"].clone();
    guest.send(json!({"type":"join", "seat":1, "name":"Alice", "token":old_token}));
    guest.until("welcome");
    let view: PlayerView = serde_json::from_value(guest.until("view")["view"].clone()).unwrap();
    guest.orders(&human_orders(&view));
    guest.ready(true);
    let before = host.seats(|s| s[1]["ready"] == true);
    let saved = server.save();

    // A reconnect that beats the host's confirmation must not be kicked out.
    host.send(json!({"type":"game_release", "seat":1}));
    assert!(
        host.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("connected")
    );
    assert_eq!(server.status()["seats"], before);
    host.send(json!({"type":"game_release", "seat":8}));
    host.until("error");
    assert_eq!(server.status()["seats"], before);

    guest.close();
    let disconnected = host.seats(|s| s[1]["connected"] == false);
    assert_eq!(disconnected[1]["reserved"], true);
    assert_eq!(disconnected[1]["orders"], 2);
    let mut newcomer = server.guest();
    newcomer.send(json!({"type":"game_release", "seat":1}));
    assert!(
        newcomer.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("loopback host")
    );
    assert_eq!(server.status()["seats"], disconnected);
    newcomer.send(json!({"type":"game_take", "seat":1}));
    newcomer.until("error");

    host.send(json!({"type":"game_release", "seat":1}));
    let opened = host.seats(|s| s[1]["reserved"] == false);
    assert_eq!(opened[0], before[0]); // The host keeps their own seat.
    assert_eq!(opened[1]["connected"], false);
    assert_eq!(opened[1]["ready"], false);
    assert_eq!(opened[1]["orders"], 0);
    for key in ["seat", "player", "name", "faction"] {
        assert_eq!(opened[1][key], before[1][key]);
    }
    assert_eq!(server.status()["seats"], opened);
    assert_eq!(server.save(), saved);
    newcomer.send(json!({"type":"join", "seat":1, "name":"Old owner", "token":old_token}));
    newcomer.until("error");
    newcomer.send(json!({"type":"game_take", "seat":1}));
    let token = newcomer.until("seat_token")["token"].clone();
    assert_ne!(token, old_token);
    newcomer.send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}));
    let welcome = newcomer.until("welcome");
    assert_eq!(welcome["orders"], json!([]));
    assert_eq!(welcome["game_id"], host.game_id);
    assert_eq!(newcomer.until("view")["view"]["me"], 1);
    let mut old_owner = server.guest();
    old_owner.send(json!({"type":"join", "seat":1, "name":"Old owner", "token":old_token}));
    old_owner.until("error");
    host.ready(true);
    host.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false);
    assert_eq!(server.status()["turn"], 1);
    newcomer.ready(true);
    host.turn(2, 0);
    newcomer.turn(2, 1);
}

#[test]
fn game_release_opens_a_seat_locked_after_restart_for_a_lan_guest() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.create(&setup());
    drop(server);
    let server = Running::start(&root);
    let (mut host, _) = server.join(0);
    let mut guest = server.guest();
    guest.send(json!({"type":"game_take", "seat":1}));
    guest.until("error");
    host.send(json!({"type":"game_release", "seat":1}));
    host.seats(|s| s[1]["reserved"] == false);
    // A restored seat stays nonblocking while disconnected, even after release.
    host.ready(true);
    host.turn(2, 0);
    guest.send(json!({"type":"game_take", "seat":1}));
    let token = guest.until("seat_token")["token"].clone();
    guest.send(json!({"type":"join", "seat":1, "name":"Alice", "token":token}));
    guest.until("welcome");
    let view = guest.until("view");
    assert_eq!(view["view"]["me"], 1);
    assert_eq!(view["view"]["turn"], 2);
    host.ready(true);
    host.seats(|s| s[0]["ready"] == true && s[1]["ready"] == false);
    assert_eq!(server.status()["turn"], 2);
    guest.ready(true);
    host.turn(3, 0);
    guest.turn(3, 1);
}

#[test]
fn staging_release_opens_only_disconnected_network_seats_without_moving_the_host() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.configure_room(&room_config());
    let mut host = server.peer();
    host.take(0, "Host");
    let mut alice = server.guest();
    alice.take(3, "Alice");
    let old_token = alice.token.clone();
    let mut bob = server.guest();
    bob.take(5, "Bob");
    alice.room_ready(3);
    bob.room_ready(5);
    let before =
        host.room(|s| s["occupants"][3]["ready"] == true && s["occupants"][5]["ready"] == true);

    host.send(json!({"type":"staging_release", "seat":3}));
    assert_eq!(host.until("error")["code"], "seat_connected");
    assert_eq!(server.status()["staging"], before);
    alice.close();
    let disconnected = host.room(|s| s["occupants"][3]["connected"] == false);
    bob.send(json!({"type":"staging_release", "seat":3}));
    assert_eq!(bob.until("error")["code"], "host_only");
    for seat in [0, 1, 2, 8] {
        host.send(json!({"type":"staging_release", "seat":seat}));
        assert_eq!(host.until("error")["code"], "not_releasable");
    }
    assert_eq!(server.status()["staging"], disconnected);

    host.send(json!({"type":"staging_release", "seat":3}));
    let opened = host.room(|s| s["occupants"][3].is_null());
    assert_eq!(opened["config"], before["config"]);
    assert_eq!(opened["occupants"][0], before["occupants"][0]);
    assert_eq!(opened["occupants"][5], before["occupants"][5]);
    assert_eq!(server.status()["staging"], opened);
    assert_ne!(server.start_room().status(), 200);
    let mut old_owner = server.guest();
    old_owner.send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}));
    assert_eq!(old_owner.until("error")["code"], "seat_expired");
    assert_eq!(server.status()["staging"], opened);
    let mut newcomer = server.guest();
    newcomer.take(3, "Alice II");
    assert_ne!(newcomer.token, old_token);
    assert_eq!(server.status()["staging"]["occupants"][3]["ready"], false);
    old_owner.send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}));
    assert_eq!(old_owner.until("error")["code"], "seat_occupied");
    newcomer.close();
    host.room(|s| s["occupants"][3]["connected"] == false);
    old_owner.send(json!({"type":"staging_take", "seat":3, "name":"Alice", "token":old_token}));
    assert_eq!(old_owner.until("error")["code"], "seat_reserved");
}

#[test]
fn only_host_subscriptions_receive_the_seed_and_status_is_always_public() {
    let root = Disk::new();
    let server = Running::start(&root);
    let config = room_config();
    let status = server.configure_room(&config);
    assert!(status["staging"].get("seed").is_none());
    assert!(status["staging"]["config"].get("seed").is_none());
    let mut host = server.peer();
    host.send(json!({"type":"staging_watch", "host":true}));
    assert_eq!(host.until("staging")["state"]["seed"], config["seed"]);
    for remote in [false, true] {
        let mut guest = if remote {
            server.guest()
        } else {
            server.peer()
        };
        guest.send(json!({"type":"staging_watch", "host":remote}));
        let state = guest.until("staging")["state"].clone();
        assert!(state.get("seed").is_none());
        assert!(state["config"].get("seed").is_none());
        assert!(!state.to_string().contains(config["seed"].as_str().unwrap()));
    }
    // The relay serves the status it was pushed with `is_local` false; the seed is never in it.
    let status = serde_json::to_value(server.world.borrow().host.status(false)).unwrap();
    assert_eq!(status["is_local"], false);
    assert!(status["staging"].get("seed").is_none());
    assert!(server.status()["staging"].get("seed").is_none());
}

#[test]
fn pings_cannot_extend_a_watchers_pre_join_deadline() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.configure_room(&room_config());
    let mut watcher = server.guest();
    watcher.send(json!({"type":"staging_watch"}));
    watcher.until("staging");
    let step = 2_000.0;
    let mut waited = 0.0;
    while waited + step < PRE_JOIN_TIMEOUT_MS {
        server.advance(step);
        waited += step;
        watcher.send(json!({"type":"ping"}));
        watcher.until("pong");
    }
    server.advance(step);
    assert!(
        watcher.is_closed(),
        "pings kept an unseated guest connected"
    );
}

#[test]
fn a_seated_guest_stays_but_one_that_goes_silent_is_closed() {
    let root = Disk::new();
    let server = Running::start(&root);
    server.configure_room(&room_config());
    let mut guest = server.guest();
    guest.take(3, "Alice");
    server.advance(PRE_JOIN_TIMEOUT_MS * 2.0);
    guest.send(json!({"type":"ping"}));
    guest.until("pong");
    // Leaving the seat starts a fresh join deadline.
    guest.send(json!({"type":"staging_leave"}));
    guest.until("staging_left");
    server.advance(PRE_JOIN_TIMEOUT_MS - 1.0);
    guest.send(json!({"type":"ping"}));
    guest.until("pong");
    server.advance(2.0);
    assert!(guest.is_closed());
    let mut quiet = server.guest();
    quiet.take(5, "Bob");
    server.advance(legatum_wasm::host::net::SILENCE_TIMEOUT_MS);
    assert!(quiet.is_closed(), "a silent guest stayed connected");
    // The host's own connection is never timed out for silence.
    let mut host = server.peer();
    host.take(0, "Host");
    server.advance(legatum_wasm::host::net::SILENCE_TIMEOUT_MS * 2.0);
    host.send(json!({"type":"ping"}));
    host.until("pong");
}

#[test]
fn at_most_eight_guests_and_a_full_game_still_admits_the_host() {
    let root = Disk::new();
    let server = Running::start(&root);
    let guests: Vec<_> = (0..legatum_wasm::host::net::MAX_GUESTS)
        .map(|_| server.guest())
        .collect();
    assert!(server.world.borrow_mut().host.connect(false, 0.0).is_err());
    let _host = server.peer();
    drop(guests);
    let _guest = server.guest();
}

#[test]
fn research_queue_survives_lan_restart_advances_and_hides_rival_plans() {
    let root = Disk::new();
    let server = Running::start(&root);
    let mut setup = setup();
    setup.players.truncate(2);
    setup.players[1].team = legatum_core::TeamId(1);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.state.players[0].science_stored = 1000;
    assert_eq!(
        server
            .load(&game_json::state_to_json(&session.state))
            .status(),
        200
    );
    let (mut a, _) = server.join(0);
    let (mut b, _) = server.join(1);
    let bad = [Order::SetResearchQueue {
        techs: vec![TechId::Mathematics],
    }];
    assert_eq!(a.orders(&bad)["errors"].as_array().unwrap().len(), 1);
    let queue = [Order::SetResearchQueue {
        techs: vec![TechId::BronzeWorking, TechId::Writing],
    }];
    assert_eq!(a.orders(&queue)["errors"], json!([]));
    a.ready(true);
    b.ready(true);
    let report = a.until("turn");
    assert!(
        report["events"]
            .as_array()
            .unwrap()
            .contains(&json!({"type":"ResearchQueueAdvanced", "player":0, "next":"Writing"}))
    );
    let view = a.next();
    assert_eq!(view["view"]["research"]["queue"], json!(["Writing"]));
    let rival_report = b.turn(2, 1);
    assert!(
        !rival_report["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "ResearchQueueAdvanced")
    );
    let saved = game_json::parse_state(&server.save()).unwrap();
    assert_eq!(saved.players[0].research_queue, vec![TechId::Writing]);
    drop(a);
    drop(b);
    drop(server);

    let restarted = Running::start(&root);
    let (mut a, view) = restarted.join(0);
    let (mut b, _) = restarted.join(1);
    assert_eq!(view.research.queue, vec![TechId::Writing]);
    a.ready(true);
    b.ready(true);
    let report = a.until("turn");
    assert!(
        report["events"]
            .as_array()
            .unwrap()
            .contains(&json!({"type":"ResearchQueueAdvanced", "player":0, "next":null}))
    );
    let view = a.next();
    assert_eq!(view["view"]["research"]["queue"], json!([]));
    assert!(view["view"]["research"]["current"].is_null());
    let rival_report = b.turn(3, 1);
    assert!(
        !rival_report["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["type"] == "ResearchQueueAdvanced")
    );
}

/// "Save to file" then "Host from a save": the host's save, kept as a file, starts a new host on a
/// fresh browser with the same turn and seat names, and the same text loads as a solo game.
#[test]
fn host_save_file_resumes_on_a_fresh_browser_and_loads_solo() {
    let first = Disk::new();
    let server = Running::start(&first);
    server.create(&setup());
    let (mut a, _) = server.join(0);
    let (mut b, _) = server.join(1);
    a.ready(true);
    b.ready(true);
    a.turn(2, 0);
    b.turn(2, 1);
    let file = server.save();
    let before = server.status();
    drop((a, b));
    drop(server);

    // A different browser: its disk holds nothing but the file the host chose.
    let fresh = Disk::new();
    *fresh.save.borrow_mut() = Some(file.clone());
    let resumed = Running::start(&fresh);
    let after = resumed.status();
    assert_eq!(after["turn"], before["turn"]);
    assert_eq!(after["has_game"], true);
    let names = |status: &Value| -> Vec<Value> {
        status["seats"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].clone())
            .collect()
    };
    assert_eq!(names(&after), names(&before));
    assert_eq!(resumed.save(), file);
    // A resume alone keeps every seat reserved; hosting from a file opens the guests' seats.
    assert_eq!(after["seats"][1]["reserved"], true);
    {
        let mut world = resumed.world.borrow_mut();
        world.host.open_guest_seats();
        world.pump();
    }
    let opened = resumed.status();
    assert_eq!(
        opened["seats"][0]["reserved"], true,
        "the host's own seat stays its own"
    );
    assert_eq!(opened["seats"][1]["reserved"], false);
    assert_eq!(names(&opened), names(&before), "seat names carry over");
    // The host takes seat 0 and a guest takes seat 1, each with a new credential.
    let (_, view) = resumed.join(0);
    assert_eq!(view.turn, 2);
    let mut guest = resumed.guest();
    guest.send(json!({"type": "game_take", "seat": 0}));
    assert!(
        guest.until("error")["message"]
            .as_str()
            .unwrap()
            .contains("reserved")
    );
    guest.send(json!({"type": "game_take", "seat": 1}));
    let token = guest.until("seat_token")["token"]
        .as_str()
        .unwrap()
        .to_owned();
    guest.send(json!({"type": "join", "seat": 1, "name": "Human 1", "token": token}));
    assert_eq!(guest.until("welcome")["seat"], 1);

    let solo = legatum_wasm::Game::from_save_json(&file);
    assert!(solo.is_ok(), "an online save is a normal engine save");
}

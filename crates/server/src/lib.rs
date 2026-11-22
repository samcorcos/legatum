//! Authoritative, single-game LAN host. All state changes, including disconnects and saves,
//! are serialized under one mutex; native game work and disk I/O run on Tokio's blocking pool.

mod admission;
mod auth;
mod credentials;
pub mod embedded;
mod events;
mod host;
mod outbox;
mod protocol;
pub mod relay;
mod saves;
mod staging;
pub use relay::{ONLINE_PROTOCOL, RelayConfig, RelayHandle, RoomAnnounce};
pub use staging::{
    FactionChoice, Occupant, SeatKind, StagingConfig, StagingSeat, StagingSettings, StagingState,
};

pub use auth::require_loopback;
use axum::extract::rejection::StringRejection;
use axum::{
    Json, Router,
    extract::{
        ConnectInfo, DefaultBodyLimit, Request, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use futures_util::SinkExt;
use host::Host;
use protocol::{ClientMessage, ServerMessage};
pub use protocol::{SeatStatus, Status};
use std::{
    net::{IpAddr, SocketAddr},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::time::{Instant, timeout};
use tower_http::services::ServeDir;

const SCENARIO_RUN_BODY_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct Server {
    host: Arc<Mutex<Host>>,
    request_policy: Arc<auth::RequestPolicy>,
    admission: Arc<admission::Admission>,
    status: Arc<RwLock<Status>>,
    /// The relay's lobby row, refreshed with `status`.
    summary: Arc<RwLock<relay::Summary>>,
    /// Ticks after every refresh of `status`, so a relay tunnel can push changes.
    refreshed: Arc<tokio::sync::watch::Sender<()>>,
}

impl Server {
    /// Creates the saves directory and restores autosave.json if present. A corrupt save is
    /// reported to the host instead of silently replacing it with a new game.
    pub fn open(saves_dir: PathBuf, port: u16, lan_urls: Vec<String>) -> Result<Self, String> {
        let archive_dir = legatum_archive::archive_root(None)
            .inspect_err(|e| eprintln!("Game archive off: {e}"))
            .ok();
        Self::open_with_archive(
            saves_dir,
            port,
            lan_urls,
            archive_dir,
            legatum_wasm::history::CHECKPOINT_TURNS,
        )
    }

    /// `archive_dir` of `None`, or one that cannot be opened, runs the game without the local
    /// archive (one log line): the archive is a record, never a reason to refuse to host.
    pub fn open_with_archive(
        saves_dir: PathBuf,
        port: u16,
        lan_urls: Vec<String>,
        archive_dir: impl Into<Option<PathBuf>>,
        checkpoint_turns: u32,
    ) -> Result<Self, String> {
        let request_policy = Arc::new(auth::RequestPolicy::new(port, &lan_urls));
        let archive = archive_dir.into().and_then(|dir| {
            legatum_archive::Archive::open(dir)
                .inspect_err(|e| eprintln!("Game archive off: {e}"))
                .ok()
        });
        let host =
            guarded(|| Host::open_with_archive(saves_dir, lan_urls, archive, checkpoint_turns))?;
        Ok(Self {
            status: Arc::new(RwLock::new(host.status(false))),
            summary: Arc::new(RwLock::new(host.summary())),
            refreshed: Arc::new(tokio::sync::watch::Sender::new(())),
            host: Arc::new(Mutex::new(host)),
            request_policy,
            admission: Arc::new(admission::Admission::new()),
        })
    }

    /// Serve with `into_make_service_with_connect_info::<SocketAddr>()` so authorization is
    /// based on the TCP peer. Host/Origin are checked separately against the listener URLs;
    /// neither Host nor X-Forwarded-For can grant loopback admin access.
    pub fn router(self, static_dir: PathBuf) -> Router {
        self.router_with(|routes| routes.fallback_service(ServeDir::new(static_dir)))
    }

    /// The API and WebSocket routes, with `fallback` adding static file serving.
    fn router_with(self, fallback: impl FnOnce(Router<Server>) -> Router<Server>) -> Router {
        let host_only = Router::new()
            .route("/api/game", post(new_game))
            .route("/api/load", post(load_game))
            .route("/api/save", get(save_game))
            .route(
                "/api/staging",
                post(configure_staging).delete(close_staging),
            )
            .route("/api/staging/start", post(start_staging))
            .route_layer(middleware::from_fn(local_only));
        let protected = Router::new()
            .merge(host_only)
            .route("/ws", any(upgrade))
            .route("/api/status", get(status))
            .route("/api/archive", post(upload_archive))
            .route(
                "/api/scenario-runs",
                post(upload_scenario_run).layer(DefaultBodyLimit::max(SCENARIO_RUN_BODY_LIMIT)),
            )
            .route_layer(middleware::from_fn_with_state(
                self.clone(),
                own_origin_only,
            ));
        fallback(Router::new().merge(protected))
            .layer(middleware::from_fn(security_headers))
            .layer(DefaultBodyLimit::max(16 * 1024 * 1024))
            .with_state(self)
    }

    async fn access<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Host) -> Result<T, String> + Send + 'static,
    ) -> Result<T, String> {
        let host = self.host.clone();
        let status = self.status.clone();
        let summary = self.summary.clone();
        let refreshed = self.refreshed.clone();
        tokio::task::spawn_blocking(move || {
            let mut host = host.lock().map_err(|_| "Game lock poisoned")?;
            let result = guarded(|| f(&mut host));
            // Publish even on error: a failed delivery may have evicted a connection.
            *status.write().map_err(|_| "Status lock poisoned")? = host.status(false);
            *summary.write().map_err(|_| "Status lock poisoned")? = host.summary();
            refreshed.send_replace(());
            result
        })
        .await
        .map_err(|e| format!("Game worker failed: {e}"))?
    }
}

fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|panic| {
        Err(format!(
            "Game error: {}",
            legatum_wasm::panic_message(&*panic)
        ))
    })
}

/// Every non-loopback IPv4 interface address, sorted and deduplicated for stable display.
pub fn lan_urls(port: u16) -> std::io::Result<Vec<String>> {
    let mut urls: Vec<_> = if_addrs::get_if_addrs()?
        .into_iter()
        .filter_map(|interface| match interface.ip() {
            IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => {
                Some(format!("http://{ip}:{port}"))
            }
            _ => None,
        })
        .collect();
    urls.sort();
    urls.dedup();
    Ok(urls)
}

fn error_response(code: StatusCode, message: String) -> Response {
    (
        code,
        Json(ServerMessage::Error {
            code: None,
            message,
        }),
    )
        .into_response()
}

async fn local_only(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    if let Err(message) = require_loopback(peer) {
        return error_response(StatusCode::FORBIDDEN, message.into());
    }
    next.run(request).await
}

/// Every response, the served UI included: no MIME sniffing, no framing by another site, and
/// no Referer carrying a LAN address elsewhere.
async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("content-security-policy", "frame-ancestors 'none'"),
        ("referrer-policy", "no-referrer"),
    ] {
        headers.insert(name, axum::http::HeaderValue::from_static(value));
    }
    response
}

async fn own_origin_only(State(server): State<Server>, request: Request, next: Next) -> Response {
    if let Err(message) = server.request_policy.require_own_origin(request.headers()) {
        return error_response(StatusCode::FORBIDDEN, message.into());
    }
    if request.method() == Method::POST && !auth::is_json(request.headers()) {
        return error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Content-Type must be application/json".into(),
        );
    }
    next.run(request).await
}

async fn status(
    State(server): State<Server>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
) -> Response {
    // No blocking job and no game mutex. Readers only copy a bounded eight-seat snapshot.
    let snapshot = server.status.read().map(|s| s.clone());
    match snapshot {
        Ok(mut status) => {
            status.is_local = require_loopback(peer).is_ok();
            ([("cache-control", "no-store")], Json(status)).into_response()
        }
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Status lock poisoned".into(),
        ),
    }
}

async fn new_game(State(server): State<Server>, body: String) -> Response {
    match server
        .access(move |host| {
            host.new_game(legatum_wasm::json::parse_setup(&body)?)?;
            Ok(host.status(true))
        })
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn load_game(State(server): State<Server>, body: String) -> Response {
    match server
        .access(move |host| {
            // Accept the raw opaque save body, and a JSON string containing that body. Never
            // round-trip the state through JavaScript numbers.
            let save = serde_json::from_str::<String>(&body).unwrap_or(body);
            host.load_game(&save)?;
            Ok(host.status(true))
        })
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn save_game(State(server): State<Server>) -> Response {
    match server.access(|host| host.save_game()).await {
        Ok(save) => ([("cache-control", "no-store")], save).into_response(),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn upload_archive(
    State(server): State<Server>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let local = require_loopback(peer).is_ok();
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_owned();
    match server.access(move |host| {
        if !local && !host.archive_authorized(&token) { return Ok(Err("A valid LAN seat token is required")); }
        let recording: legatum_archive::Recording = serde_json::from_str(&body).map_err(|e| format!("Invalid archive: {e}"))?;
        if !matches!(recording.metadata.source, legatum_archive::Source::HumanLocal | legatum_archive::Source::HumanHotseat) {
            return Err("Only local human game histories can be uploaded; LAN games are recorded by the host".into());
        }
        host.upload_archive(&recording)?;
        Ok(Ok(()))
    }).await {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Err(message)) => error_response(StatusCode::FORBIDDEN, message.into()),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

/// A finished scenario run from the web or phone build: saved under `<saves>/scenarios/`.
/// No seat token: a scenario is a single-player game in that browser and has no seat. The
/// route is still limited to this server's own origin, JSON bodies of at most 2 MiB, ids that
/// are plain names, and a capped folder; the reply names the file relative to the saves folder.
async fn upload_scenario_run(
    State(server): State<Server>,
    body: Result<String, StringRejection>,
) -> Response {
    let body = match body {
        Ok(body) => body,
        Err(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "Scenario run exceeds the 2 MiB upload limit".into(),
            );
        }
        Err(error) => return error.into_response(),
    };
    match server
        .access(move |host| host.save_scenario_run(&body))
        .await
    {
        // Only the path inside the saves folder: the host's absolute path names its user.
        Ok(path) => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            Json(serde_json::json!({ "path": format!("scenarios/{name}") })).into_response()
        }
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn configure_staging(State(server): State<Server>, body: String) -> Response {
    match server
        .access(move |host| {
            let config = serde_json::from_str(&body).map_err(|e| format!("Invalid room: {e}"))?;
            host.configure_staging(config)?;
            Ok(host.status(true))
        })
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn close_staging(State(server): State<Server>) -> Response {
    match server
        .access(|host| {
            host.close_staging();
            Ok(host.status(true))
        })
        .await
    {
        Ok(status) => Json(status).into_response(),
        Err(error) => error_response(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn start_staging(State(server): State<Server>) -> Response {
    match server
        .access(|host| {
            if let Err(e) = host.staging_can_start() {
                return Ok(Err(e));
            }
            host.start_staging()?;
            Ok(Ok(host.status(true)))
        })
        .await
    {
        Ok(Ok(status)) => Json(status).into_response(),
        Ok(Err(e)) => (
            StatusCode::CONFLICT,
            Json(ServerMessage::Error {
                code: Some(e.code),
                message: e.message.into(),
            }),
        )
            .into_response(),
        Err(error) => error_response(StatusCode::BAD_REQUEST, error),
    }
}

async fn upgrade(
    State(server): State<Server>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(permit) = server
        .admission
        .acquire(peer.ip(), require_loopback(peer).is_ok())
    else {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "The server has reached its connection limit".into(),
        );
    };
    ws.max_message_size(64 * 1024)
        .max_frame_size(64 * 1024)
        .on_upgrade(move |socket| {
            connection(server, Lan(socket), permit, require_loopback(peer).is_ok())
        })
}

async fn send(socket: &mut WebSocket, message: Message) -> bool {
    matches!(
        timeout(Duration::from_secs(5), socket.send(message)).await,
        Ok(Ok(()))
    )
}

async fn finish_close(socket: &mut WebSocket) {
    // Flushing Close alone does not finish the handshake. Dropping TCP while a late
    // Pong is in flight can reset the connection and discard the terminal error.
    // SinkExt::close also flushes Tungstenite's queued reply to a peer-initiated Close;
    // sending another Close message in that state would fail with SendAfterClosing.
    let _ = timeout(Duration::from_secs(5), async {
        if socket.close().await.is_ok() {
            // Drain through the peer's Close and the final flush/EOF. Application
            // messages received during shutdown must not reach the game anymore.
            while let Some(Ok(_)) = socket.recv().await {}
        }
    })
    .await;
}

/// One client connection's frames: a LAN WebSocket, or a guest multiplexed over the relay tunnel.
/// Both run the same `connection` loop, so heartbeats, deadlines, the outbox and admission match.
pub(crate) trait Transport: Send + 'static {
    /// Sends one frame; false when the peer is gone or the write stalled.
    fn send(&mut self, message: Message) -> impl Future<Output = bool> + Send;
    /// The next frame; `None` once the connection is closed or broken.
    fn recv(&mut self) -> impl Future<Output = Option<Message>> + Send;
    /// Ends the connection after the game has let go of it.
    fn finish_close(&mut self) -> impl Future<Output = ()> + Send;
}

/// A LAN WebSocket, sent and closed exactly as before relayed guests existed.
struct Lan(WebSocket);

impl Transport for Lan {
    fn send(&mut self, message: Message) -> impl Future<Output = bool> + Send {
        send(&mut self.0, message)
    }

    async fn recv(&mut self) -> Option<Message> {
        match self.0.recv().await {
            Some(Ok(message)) => Some(message),
            _ => None,
        }
    }

    fn finish_close(&mut self) -> impl Future<Output = ()> + Send {
        finish_close(&mut self.0)
    }
}

async fn connection<T: Transport>(
    server: Server,
    mut socket: T,
    mut permit: admission::Permit,
    local: bool,
) {
    let (sender, receiver) = outbox::channel();
    let Ok(id) = server.access(move |host| host.connect(sender, local)).await else {
        return;
    };
    let outbox::Receiver {
        mut reliable,
        mut seats,
        mut staging,
        mut close,
        mut admitted,
    } = receiver;
    let mut heartbeat = tokio::time::interval(Duration::from_secs(20));
    let mut last_seen = Instant::now();
    let mut joined = false;
    let mut join_deadline = Instant::now() + admission::PRE_JOIN_TIMEOUT;
    // The close signal has a separate slot, and cancels even an in-progress socket send.
    // Neither network I/O nor waiting for a slow reader holds the game mutex.
    let shutdown = outbox::until_closed(&mut close, async {
        loop {
            tokio::select! {
                // Poll the deadline first even if broadcasts never stop arriving.
                biased;
                changed = admitted.changed() => {
                    if changed.is_err() { break; }
                    joined = *admitted.borrow_and_update();
                    if !permit.admitted(joined) { break; }
                    if !joined { join_deadline = Instant::now() + admission::PRE_JOIN_TIMEOUT; }
                }
                _ = tokio::time::sleep_until(join_deadline), if !joined => { break; }
                _ = heartbeat.tick() => {
                    if last_seen.elapsed() >= Duration::from_secs(60)
                        || !socket.send(Message::Ping(Vec::new().into())).await
                    { break; }
                }
                outgoing = reliable.recv() => {
                    let Some(outgoing) = outgoing else { break };
                    let mut delivered = true;
                    for text in &outgoing.texts {
                        if !socket.send(Message::Text(text.clone().into())).await {
                            delivered = false;
                            break;
                        }
                    }
                    if !delivered { break; }
                }
                incoming = socket.recv() => {
                    last_seen = Instant::now();
                    let result = match incoming {
                        Some(Message::Text(text)) => match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(message) => server.access(move |host| host.message(id, message)).await,
                            Err(error) => Err(format!("Invalid message: {error}")),
                        },
                        Some(Message::Ping(bytes)) => {
                            if !socket.send(Message::Pong(bytes)).await { break; }
                            Ok(())
                        }
                        Some(Message::Pong(_)) => Ok(()),
                        Some(Message::Binary(_)) => Err("Send JSON text frames".into()),
                        _ => break,
                    };
                    if let Err(message) = result {
                        let text = serde_json::to_string(&ServerMessage::Error { code: None, message }).expect("protocol serializes");
                        if !socket.send(Message::Text(text.into())).await { break; }
                    }
                }
                changed = seats.changed() => {
                    if changed.is_err() { break; }
                    let snapshot = seats.borrow_and_update().clone();
                    if let Some(text) = snapshot
                        && !socket.send(Message::Text(text.to_string().into())).await { break; }
                }
                changed = staging.changed() => {
                    if changed.is_err() { break; }
                    let snapshot = staging.borrow_and_update().clone();
                    if let Some(text) = snapshot
                        && !socket.send(Message::Text(text.to_string().into())).await { break; }
                }
            }
        }
    })
    .await;
    if let Some(outbox::Close { code, message }) = shutdown {
        let text = serde_json::to_string(&ServerMessage::Error { code, message })
            .expect("protocol serializes");
        socket.send(Message::Text(text.into())).await;
    }
    let _ = server
        .access(move |host| {
            host.disconnect(id);
            Ok(())
        })
        .await;
    // Detach from the game before waiting on the peer; retain admission until the
    // bounded closing handshake ends, then release it before dropping the TCP socket.
    socket.finish_close().await;
    drop(permit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn cached_status_does_not_wait_for_game_work_or_the_executor() {
        let root = tempfile::tempdir().unwrap();
        let server = Server::open_with_archive(
            root.path().join("saves"),
            7878,
            vec![],
            root.path().join("archive"),
            5,
        )
        .unwrap();
        let busy = server.clone();
        let (locked, wait_for_lock) = tokio::sync::oneshot::channel();
        let (release, wait_for_release) = std::sync::mpsc::channel();
        let worker = tokio::spawn(async move {
            busy.access(move |_| {
                locked.send(()).unwrap();
                // Status must run before game work can finish. A finite wait also makes
                // an accidental inline/blocking access fail instead of hanging the test.
                wait_for_release
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(|e| format!("game work prevented status from running: {e}"))
            })
            .await
        });
        wait_for_lock.await.unwrap();
        let response = timeout(
            Duration::from_secs(1),
            status(
                State(server),
                ConnectInfo("127.0.0.1:1234".parse().unwrap()),
            ),
        )
        .await;
        let released = release.send(());
        let result = worker.await.unwrap();
        result.expect("status must finish while game work is still pending");
        released.unwrap();
        assert_eq!(
            response.expect("status waited for game work").status(),
            StatusCode::OK
        );
    }

    async fn websocket_pair() -> (
        WebSocket,
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let app = Router::new().route(
            "/ws",
            get(move |ws: WebSocketUpgrade| {
                let sender = sender.clone();
                async move {
                    ws.on_upgrade(move |socket| async move {
                        sender.send(socket).await.unwrap();
                    })
                }
            }),
        );
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let (client, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .unwrap();
        let server = receiver.recv().await.unwrap();
        task.abort();
        (server, client)
    }

    #[tokio::test]
    async fn close_handshake_drains_late_heartbeat_and_waits_for_peer_reply() {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message as ClientFrame;

        let (mut server, mut client) = websocket_pair().await;
        server.send(Message::Ping(vec![1].into())).await.unwrap();
        assert!(matches!(
            client.next().await.unwrap().unwrap(),
            ClientFrame::Ping(_)
        ));
        // Reading Ping queues the client's Pong, but deliberately do not flush it yet.
        // The server must keep reading while closing, including this late heartbeat.
        {
            let closing = finish_close(&mut server);
            tokio::pin!(closing);
            let frame = timeout(Duration::from_secs(20), async {
                tokio::select! {
                    biased;
                    () = &mut closing => panic!("server closed TCP before the peer's Close reply"),
                    frame = client.next() => frame.unwrap().unwrap(),
                }
            })
            .await
            .unwrap();
            assert!(matches!(frame, ClientFrame::Close(_)));
            // Receiving Close queues the reply. No acknowledgment was possible earlier.
            client.flush().await.unwrap();
            timeout(Duration::from_secs(20), closing).await.unwrap();
        }
        drop(server);
        assert!(
            client.next().await.is_none(),
            "closing handshake must end with clean EOF"
        );
    }

    #[tokio::test]
    async fn close_handshake_has_a_deadline_for_an_unresponsive_peer() {
        let (mut server, _client) = websocket_pair().await;
        // Establish real I/O first; advance only the closing deadline with virtual time.
        tokio::time::pause();
        let closing = finish_close(&mut server);
        tokio::pin!(closing);
        assert!(futures_util::poll!(&mut closing).is_pending());
        tokio::time::advance(Duration::from_secs(4)).await;
        assert!(futures_util::poll!(&mut closing).is_pending());
        // Let the timer driver process its deadline, including its millisecond tick.
        timeout(Duration::from_secs(2), closing)
            .await
            .expect("an unresponsive peer held the closing socket past its deadline");
    }
}

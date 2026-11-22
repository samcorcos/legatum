//! The server inside the desktop app: its own Tokio runtime thread, the UI served from the app's
//! bundled assets, and optionally a relay room for online guests (`plans/online-play.md`).
pub use crate::{ONLINE_PROTOCOL, RelayConfig, RoomAnnounce};
use crate::{RelayHandle, Server};
use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, thread::JoinHandle, time::Duration};
use tokio::sync::oneshot;

/// Returns a bundled UI file and its MIME type for a raw request path without its query
/// (`/`, `/assets/app.js`, `/dir/`); `None` is a 404. The callback maps `/` to `index.html`.
pub type Assets = Arc<dyn Fn(&str) -> Option<(Vec<u8>, String)> + Send + Sync>;

pub struct EmbedConfig {
    pub saves_dir: PathBuf,
    pub archive_dir: PathBuf,
    /// Serves every non-API path. Unknown paths are 404; there is no `index.html` fallback.
    pub assets: Assets,
    pub relay: Option<RelayConfig>,
    /// false: listen on 127.0.0.1 only. true: also on the LAN (port 7878 when free).
    pub bind_lan: bool,
}

/// A running embedded server. Dropping it stops the server without waiting.
pub struct Handle {
    pub port: u16,
    /// `http://127.0.0.1:<port>/`, the host window's page.
    pub url: String,
    /// The relay room id, when a relay was configured.
    pub room: Option<String>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Handle {
    /// Closes the relay room (`bye`) and the listener, then waits for the server thread.
    /// Returns within about two seconds, even when the relay is unreachable.
    pub fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

type Started = Result<(u16, Option<String>), String>;

/// Opens the saves, binds the listener and (when configured) announces the relay room before
/// returning, so every error is reported here.
pub fn start(config: EmbedConfig) -> Result<Handle, String> {
    let (ready, started) = std::sync::mpsc::channel::<Started>();
    let (stop, stopped) = oneshot::channel();
    let thread = std::thread::Builder::new()
        .name("legatum-server".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("legatum-server")
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = ready.send(Err(format!("Cannot start the server runtime: {error}")));
                    return;
                }
            };
            runtime.block_on(serve(config, ready, stopped));
            // Game work on the blocking pool must not keep the app waiting on exit.
            runtime.shutdown_timeout(Duration::from_millis(300));
        })
        .map_err(|e| format!("Cannot start the server thread: {e}"))?;
    match started.recv() {
        Ok(Ok((port, room))) => Ok(Handle {
            port,
            url: format!("http://127.0.0.1:{port}/"),
            room,
            stop: Some(stop),
            thread: Some(thread),
        }),
        Ok(Err(error)) => {
            let _ = thread.join();
            Err(error)
        }
        Err(_) => {
            let _ = thread.join();
            Err("The server stopped while starting".into())
        }
    }
}

async fn serve(
    config: EmbedConfig,
    ready: std::sync::mpsc::Sender<Started>,
    stopped: oneshot::Receiver<()>,
) {
    let EmbedConfig {
        saves_dir,
        archive_dir,
        assets,
        relay,
        bind_lan,
    } = config;
    let opened = async {
        let listener = bind(bind_lan).await?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("Cannot read the server address: {e}"))?
            .port();
        let lan_urls = if bind_lan {
            crate::lan_urls(port).map_err(|e| format!("Cannot list network interfaces: {e}"))?
        } else {
            Vec::new()
        };
        let server = Server::open_with_archive(
            saves_dir,
            port,
            lan_urls,
            archive_dir,
            legatum_wasm::history::CHECKPOINT_TURNS,
        )?;
        let relay = match relay {
            Some(relay) => Some(server.run_relay(relay).await?),
            None => None,
        };
        Ok::<_, String>((listener, port, server, relay))
    }
    .await;
    let (listener, port, server, relay) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    let app = server
        .router_with(move |routes| {
            routes.fallback(move |request: Request| {
                let assets = assets.clone();
                async move { asset(&assets, &request) }
            })
        })
        .into_make_service_with_connect_info::<SocketAddr>();
    let _ = ready.send(Ok((port, relay.as_ref().map(RelayHandle::room))));
    tokio::select! {
        served = axum::serve(listener, app) => {
            if let Err(error) = served {
                eprintln!("The embedded server stopped: {error}");
            }
        }
        _ = stopped => {}
    }
    if let Some(relay) = relay {
        // Bounded: `bye` goes out at once when the tunnel is up; an unreachable relay is not awaited.
        let _ = tokio::time::timeout(Duration::from_millis(1500), relay.stop()).await;
    }
}

async fn bind(lan: bool) -> Result<tokio::net::TcpListener, String> {
    let bound = if lan {
        match tokio::net::TcpListener::bind(("0.0.0.0", 7878)).await {
            Ok(listener) => Ok(listener),
            Err(_) => tokio::net::TcpListener::bind(("0.0.0.0", 0)).await,
        }
    } else {
        tokio::net::TcpListener::bind(("127.0.0.1", 0)).await
    };
    bound.map_err(|e| format!("Cannot open a port for the server: {e}"))
}

fn asset(assets: &Assets, request: &Request) -> Response {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    // The raw path without the query, e.g. `/`, `/assets/app.js` or `/dir/`.
    let path = request.uri().path();
    if path
        .split('/')
        .any(|segment| segment == "." || segment == "..")
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    match assets(path) {
        Some((bytes, mime)) => ([(header::CONTENT_TYPE, mime)], bytes).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

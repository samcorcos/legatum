//! Online play, host side (plans/online-play.md, "App shell"): the app runs `legatum-server`
//! in-process on 127.0.0.1, the server dials out to the relay, and the host's window moves to
//! the server's page. Leaving the game (the `online_host_stop` command, or the window going back
//! to the bundled UI), closing the window and quitting all stop the server.
//!
//! Window-free logic (labels, relay URL, CSP, names, asset paths, origins) is plain functions
//! with unit tests; the commands and hooks at the bottom are thin wrappers around them.

use std::sync::{Arc, Mutex};
use tauri::utils::config::Csp;
use tauri::{Manager, Runtime, Url};

/// The relay base URL, fixed at build time (`LEGATUM_RELAY_URL=https://... scripts/build-mac-app.sh`).
const RELAY_URL_ENV: Option<&str> = option_env!("LEGATUM_RELAY_URL");
/// The release label (`tools/release/check.sh` exports it for the whole build).
const BUILD_LABEL_ENV: Option<&str> = option_env!("LEGATUM_BUILD_LABEL");
/// `git describe --tags --match 'playtest-*' --always`, from build.rs.
const BUILD_DESCRIBE: &str = env!("LEGATUM_BUILD_DESCRIBE");

/// RoomAnnounce limits (plans/online-play.md).
const MAX_NAME_CHARS: usize = 80;
const MAX_BUILD_CHARS: usize = 40;

// ---------------------------------------------------------------------------------------------
// Build label

/// Rust twin of `formatBuildLabel` in app/src/buildLabel.ts (same cases, same tests), so the
/// lobby compares the label the title screen shows.
pub fn format_build_label(label: &str, describe: &str) -> String {
    if !label.is_empty() {
        return label.to_string();
    }
    // /^playtest-([\w.]+?)(?:-(\d+)-g([0-9a-f]+))?$/: the tag holds no '-', so it ends at the
    // first '-', and the rest is empty or "-<commits>-g<hash>".
    let tagged = describe.strip_prefix("playtest-").and_then(|rest| {
        let (tag, tail) = rest.split_once('-').unwrap_or((rest, ""));
        let tag_ok = !tag.is_empty()
            && tag
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.');
        if !tag_ok {
            return None;
        }
        if tail.is_empty() {
            return Some((tag, None));
        }
        let (count, hash) = tail.split_once("-g")?;
        let count_ok = !count.is_empty() && count.bytes().all(|b| b.is_ascii_digit());
        let hash_ok =
            !hash.is_empty() && hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        (count_ok && hash_ok).then_some((tag, Some(hash)))
    });
    match tagged {
        Some((tag, None)) => format!("Playtest {tag}"),
        Some((tag, Some(hash))) => format!("Dev · after {tag} ({})", prefix(hash, 7)),
        None if !describe.is_empty() => format!("Dev ({})", prefix(describe, 7)),
        None => "Dev build".into(),
    }
}

fn prefix(s: &str, chars: usize) -> &str {
    s.char_indices().nth(chars).map_or(s, |(i, _)| &s[..i])
}

/// This build's label, as the title screen shows it, cut to the relay's 40 characters. The same
/// value goes in the room announce and in `online_info`, so two copies of one build always match.
pub fn build_label() -> String {
    let label = format_build_label(BUILD_LABEL_ENV.unwrap_or(""), BUILD_DESCRIBE);
    prefix(&label, MAX_BUILD_CHARS).to_string()
}

// ---------------------------------------------------------------------------------------------
// Relay URL and CSP

/// The relay base URL without a trailing slash, or `None` when unset, empty or not http(s).
pub fn normalize_relay_url(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim().trim_end_matches('/');
    if raw.is_empty() {
        return None;
    }
    match Url::parse(raw) {
        Ok(url)
            if matches!(url.scheme(), "https" | "http")
                && url.host_str().is_some()
                && url.query().is_none()
                && url.fragment().is_none() =>
        {
            Some(raw.to_string())
        }
        _ => None,
    }
}

/// The relay this build talks to.
pub fn relay_url() -> Option<String> {
    normalize_relay_url(RELAY_URL_ENV)
}

/// The two CSP `connect-src` sources the relay needs: its http(s) origin (lobby, status) and
/// the matching ws(s) origin (guest sockets).
pub fn relay_csp_sources(relay: &str) -> Vec<String> {
    let Ok(url) = Url::parse(relay) else {
        return Vec::new();
    };
    let Some(host) = url.host_str() else {
        return Vec::new();
    };
    let port = url.port().map(|p| format!(":{p}")).unwrap_or_default();
    let ws = if url.scheme() == "https" { "wss" } else { "ws" };
    vec![
        format!("{}://{host}{port}", url.scheme()),
        format!("{ws}://{host}{port}"),
    ]
}

/// Add the relay's origins to `connect-src` (created from `default-src` if absent).
pub fn add_relay_to_csp(csp: &mut Csp, relay: &str) {
    let sources = relay_csp_sources(relay);
    if sources.is_empty() {
        return;
    }
    match csp {
        Csp::DirectiveMap(map) => {
            if !map.contains_key("connect-src") {
                let base = map.get("default-src").cloned().unwrap_or_default();
                map.insert("connect-src".into(), base);
            }
            let connect = map.get_mut("connect-src").expect("inserted above");
            for s in sources {
                if !connect.contains(&s) {
                    connect.push(s);
                }
            }
        }
        Csp::Policy(policy) => {
            let mut directives: Vec<String> = policy
                .split(';')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();
            let extra = sources.join(" ");
            match directives
                .iter_mut()
                .find(|d| d.split_whitespace().next() == Some("connect-src"))
            {
                Some(d) => *d = format!("{d} {extra}"),
                None => directives.push(format!("connect-src 'self' {extra}")),
            }
            *policy = directives.join("; ");
        }
    }
}

/// The bundled UI's CSP gains the relay's exact origins at startup. The relay URL is a build
/// input, so this keeps tauri.conf.json free of wildcard hosts and of build-time rewriting.
pub fn configure_csp(config: &mut tauri::Config) {
    if let (Some(relay), Some(csp)) = (relay_url(), config.app.security.csp.as_mut()) {
        add_relay_to_csp(csp, &relay);
    }
}

// ---------------------------------------------------------------------------------------------
// Host name and command shapes

/// The host's display name for the lobby: trimmed, 1..=80 characters, no control characters.
pub fn validate_host_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Enter a name to host a game.".into());
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(format!("Names are at most {MAX_NAME_CHARS} characters."));
    }
    if name.chars().any(char::is_control) {
        return Err("Names cannot hold control characters.".into());
    }
    Ok(name.to_string())
}

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct HostStarted {
    /// `http://127.0.0.1:<port>/`: the window navigates here.
    pub url: String,
    /// The relay room id; `null` when this build has no relay (hosting is local only).
    pub room: Option<String>,
}

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct OnlineInfo {
    pub relay: Option<String>,
    pub build: String,
    pub rules_version: u32,
    pub protocol: u32,
}

pub fn online_info_value() -> OnlineInfo {
    OnlineInfo {
        relay: relay_url(),
        build: build_label(),
        rules_version: legatum_core::RULES_VERSION,
        protocol: legatum_server::ONLINE_PROTOCOL,
    }
}

pub fn relay_config(relay: Option<String>, name: String) -> Option<legatum_server::RelayConfig> {
    relay.map(|url| legatum_server::RelayConfig {
        url,
        announce: legatum_server::RoomAnnounce {
            name,
            build: build_label(),
            rules_version: legatum_core::RULES_VERSION,
            protocol: legatum_server::ONLINE_PROTOCOL,
        },
    })
}

/// `~/Library/Application Support/Legatum/saves`, the default of the `legatum-server` CLI, so
/// a game hosted from the app and one hosted from Terminal share autosaves.
pub fn default_saves_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var_os("HOME").ok_or("HOME is unset")?;
    Ok(std::path::PathBuf::from(home).join("Library/Application Support/Legatum/saves"))
}

// ---------------------------------------------------------------------------------------------
// Assets

/// The asset key for a request path: no leading slash, no query or fragment, `index.html` for a
/// folder. `None` for paths that try to climb out (`..`) or hold backslashes or NULs.
pub fn asset_key(request_path: &str) -> Option<String> {
    let path = request_path.split(['?', '#']).next().unwrap_or("");
    let path = path.trim_start_matches('/');
    if path.contains('\\') || path.contains('\0') {
        return None;
    }
    if path.split('/').any(|seg| seg == ".." || seg == ".") {
        return None;
    }
    if path.is_empty() || path.ends_with('/') {
        return Some(format!("{path}index.html"));
    }
    Some(path.to_string())
}

fn extension(key: &str) -> Option<&str> {
    let name = key.rsplit('/').next()?;
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty()).then_some(ext)
}

/// The mime type for the file types the UI ships; `None` leaves Tauri's guess.
pub fn mime_for(key: &str) -> Option<&'static str> {
    Some(match extension(key)?.to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "txt" => "text/plain; charset=utf-8",
        _ => return None,
    })
}

/// One asset for the embedded server, looked up with `lookup` (Tauri's resolver in the app).
/// Tauri answers a missing file with `index.html`; that fallback is kept for page routes, but a
/// missing script, image or other file is a 404, not the app's HTML under the wrong name.
pub fn resolve_asset(
    request_path: &str,
    lookup: impl Fn(&str) -> Option<(Vec<u8>, String)>,
) -> Option<(Vec<u8>, String)> {
    let key = asset_key(request_path)?;
    let (bytes, guessed) = lookup(&key)?;
    let html_page = matches!(extension(&key), None | Some("html" | "htm"));
    if html_page {
        return Some((bytes, mime_for("index.html")?.to_string()));
    }
    // Tauri also guesses text/html for extensions it does not know, so only the bytes tell
    // whether this is the index fallback.
    if guessed.starts_with("text/html")
        && lookup("index.html").is_some_and(|(index, _)| index == bytes)
    {
        return None;
    }
    Some((bytes, mime_for(&key).map_or(guessed, str::to_string)))
}

// ---------------------------------------------------------------------------------------------
// Origins

/// Whether `url` is a page of the embedded server on `port`.
pub fn is_server_url(url: &Url, port: u16) -> bool {
    url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
        && url.port() == Some(port)
}

/// The bundled UI's own URL: the dev server in `tauri dev`, else the app's custom protocol
/// (`tauri://localhost/` on macOS, Linux and iOS; `http://tauri.localhost/` on Windows and
/// Android, where `useHttpsScheme` is off).
pub fn app_url(dev_url: Option<&Url>, windows_or_android: bool) -> Url {
    if let Some(dev) = dev_url {
        return dev.clone();
    }
    let s = if windows_or_android {
        "http://tauri.localhost/"
    } else {
        "tauri://localhost/"
    };
    Url::parse(s).expect("static URL")
}

/// Whether `url` is the bundled UI (same origin as `app`).
pub fn is_app_url(url: &Url, app: &Url) -> bool {
    url.scheme() == app.scheme()
        && url.host_str() == app.host_str()
        && url.port_or_known_default() == app.port_or_known_default()
}

// ---------------------------------------------------------------------------------------------
// Tauri state, commands and hooks

struct Running {
    handle: legatum_server::embedded::Handle,
    port: u16,
}

/// The embedded server while one runs.
#[derive(Default)]
pub struct OnlineHost {
    running: Mutex<Option<Running>>,
}

impl OnlineHost {
    fn take(&self) -> Option<Running> {
        self.running.lock().ok()?.take()
    }

    pub(crate) fn port(&self) -> Option<u16> {
        self.running.lock().ok()?.as_ref().map(|r| r.port)
    }

    /// Stop the server now (blocks until the server has said `bye` to the relay).
    pub(crate) fn stop_blocking(&self) {
        if let Some(running) = self.take() {
            running.handle.stop();
        }
    }

    /// Stop the server on a worker thread, for hooks that must not block the window.
    fn stop_in_background(&self) {
        if let Some(running) = self.take() {
            let spawned = std::thread::Builder::new()
                .name("online-stop".into())
                .spawn(move || running.handle.stop());
            if let Err(e) = spawned {
                eprintln!("legatum: online server not stopped: {e}");
            }
        }
    }
}

pub(crate) fn bundled_ui_url<R: Runtime>(app: &tauri::AppHandle<R>) -> Url {
    let dev = if tauri::is_dev() {
        app.config().build.dev_url.as_ref()
    } else {
        None
    };
    app_url(dev, cfg!(any(windows, target_os = "android")))
}

/// Start hosting: the embedded server on 127.0.0.1, announced to the relay when this build has
/// one. A server already running is stopped first. The UI then navigates the window to `url`.
#[tauri::command]
pub async fn online_host_start<R: Runtime>(
    app: tauri::AppHandle<R>,
    name: String,
) -> Result<HostStarted, String> {
    tauri::async_runtime::spawn_blocking(move || start_host(&app, &name))
        .await
        .map_err(|e| e.to_string())?
}

/// `online_host_start` on the calling thread (blocks while the server starts and announces).
pub(crate) fn start_host<R: Runtime>(
    app: &tauri::AppHandle<R>,
    name: &str,
) -> Result<HostStarted, String> {
    let name = validate_host_name(name)?;
    let host = app.state::<OnlineHost>();
    host.stop_blocking();
    // The very UI the app bundles, from Tauri's embedded assets (the dist folder in `tauri dev`).
    let resolver = app.asset_resolver();
    let config = legatum_server::embedded::EmbedConfig {
        saves_dir: default_saves_dir()?,
        archive_dir: app.state::<legatum_archive::Archive>().root().to_path_buf(),
        assets: Arc::new(move |path: &str| {
            resolve_asset(path, |key| {
                let asset = resolver.get(key.to_string())?;
                Some((asset.bytes, asset.mime_type))
            })
        }),
        relay: relay_config(relay_url(), name),
        bind_lan: false,
    };
    let handle = legatum_server::embedded::start(config)?;
    let started = HostStarted {
        url: handle.url.clone(),
        room: handle.room.clone(),
    };
    let port = handle.port;
    let mut slot = host.running.lock().map_err(|e| e.to_string())?;
    if let Some(raced) = slot.replace(Running { handle, port }) {
        // Two starts at once: keep the newest, stop the other off this thread.
        std::thread::spawn(move || raced.handle.stop());
    }
    Ok(started)
}

/// Stop hosting. Called from the host page on 127.0.0.1 (Leave game), it also takes that window
/// back to the bundled UI; called from the bundled UI it only stops the server.
#[tauri::command]
pub async fn online_host_stop<R: Runtime>(
    app: tauri::AppHandle<R>,
    webview: tauri::Webview<R>,
) -> Result<(), String> {
    let host = app.state::<OnlineHost>();
    let port = host.port();
    let on_server_page = match (port, webview.url()) {
        (Some(port), Ok(url)) => is_server_url(&url, port),
        (None, Ok(url)) => url.scheme() == "http" && url.host_str() == Some("127.0.0.1"),
        _ => false,
    };
    if let Some(running) = host.take() {
        tauri::async_runtime::spawn_blocking(move || running.handle.stop())
            .await
            .map_err(|e| e.to_string())?;
    }
    if on_server_page {
        webview
            .navigate(bundled_ui_url(&app))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// What the lobby compares: this build's relay, label, rules version and online protocol.
#[tauri::command]
pub fn online_info() -> OnlineInfo {
    online_info_value()
}

/// The `online` plugin: holds the server's state and stops the server when the window goes back
/// to the bundled UI by itself (the host page setting `location` to the app URL), when the main
/// window is destroyed, and when the app exits (blocking, so the relay gets `bye` first).
pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("online")
        .setup(|app, _| {
            app.manage(OnlineHost::default());
            // Off unless LEGATUM_ONLINE_SELFTEST names a log file (src/selftest.rs).
            crate::selftest::setup(app);
            Ok(())
        })
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                crate::selftest::page_loaded(webview, payload.url());
            }
        })
        .on_navigation(|webview, url| {
            // Leaving means navigating to the app's own origin. Other targets do not count:
            // wry also reports subframe navigations, and a reload stays on 127.0.0.1.
            let app = webview.app_handle();
            if let Some(host) = app.try_state::<OnlineHost>() {
                if host.port().is_some() && is_app_url(url, &bundled_ui_url(app)) {
                    host.stop_in_background();
                }
            }
            true
        })
        .on_event(|app, event| {
            let stop = match event {
                tauri::RunEvent::Exit => true,
                tauri::RunEvent::WindowEvent {
                    label,
                    event: tauri::WindowEvent::Destroyed,
                    ..
                } => label == "main",
                _ => false,
            };
            if stop {
                if let Some(host) = app.try_state::<OnlineHost>() {
                    host.stop_blocking();
                }
            }
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tauri::utils::config::CspDirectiveSources;

    fn joined(sources: &CspDirectiveSources) -> String {
        match sources {
            CspDirectiveSources::Inline(s) => s.clone(),
            CspDirectiveSources::List(l) => l.join(" "),
        }
    }

    #[test]
    fn build_label_matches_the_title_screen_cases() {
        // The cases of app/src/buildLabel.test.ts.
        assert_eq!(
            format_build_label("Playtest 7a", "playtest-6.3-12-gabcdef1"),
            "Playtest 7a"
        );
        assert_eq!(format_build_label("", "playtest-6.3"), "Playtest 6.3");
        assert_eq!(format_build_label("", "playtest-7a"), "Playtest 7a");
        assert_eq!(
            format_build_label("", "playtest-6.3-12-gabcdef123"),
            "Dev · after 6.3 (abcdef1)"
        );
        assert_eq!(format_build_label("", "abcdef123"), "Dev (abcdef1)");
        assert_eq!(format_build_label("", ""), "Dev build");
        // Not the playtest shape: the whole describe string is the fallback.
        assert_eq!(format_build_label("", "playtest-6.3-x"), "Dev (playtes)");
        assert_eq!(format_build_label("", "playtest--1-gab"), "Dev (playtes)");
        assert!(build_label().chars().count() <= MAX_BUILD_CHARS);
        assert!(!build_label().is_empty());
    }

    #[test]
    fn relay_url_is_normalized_or_dropped() {
        assert_eq!(normalize_relay_url(None), None);
        assert_eq!(normalize_relay_url(Some("  ")), None);
        assert_eq!(
            normalize_relay_url(Some("https://legatum-relay.x.workers.dev/")),
            Some("https://legatum-relay.x.workers.dev".into())
        );
        assert_eq!(
            normalize_relay_url(Some("http://localhost:8787")),
            Some("http://localhost:8787".into())
        );
        assert_eq!(normalize_relay_url(Some("wss://relay.example")), None);
        assert_eq!(normalize_relay_url(Some("relay.example")), None);
        assert_eq!(
            normalize_relay_url(Some("https://relay.example/?a=1")),
            None
        );
    }

    #[test]
    fn relay_config_carries_the_announce() {
        assert!(relay_config(None, "Sam".into()).is_none());
        let cfg = relay_config(Some("https://r.example".into()), "Sam".into()).unwrap();
        assert_eq!(cfg.url, "https://r.example");
        assert_eq!(cfg.announce.name, "Sam");
        assert_eq!(cfg.announce.build, build_label());
        assert_eq!(cfg.announce.rules_version, legatum_core::RULES_VERSION);
        assert_eq!(cfg.announce.protocol, legatum_server::ONLINE_PROTOCOL);
        let info = online_info_value();
        assert_eq!(info.build, cfg.announce.build);
        let json = serde_json::to_value(&info).unwrap();
        for key in ["relay", "build", "rules_version", "protocol"] {
            assert!(json.get(key).is_some(), "{key}");
        }
    }

    #[test]
    fn csp_gains_exactly_the_relay_origins() {
        assert_eq!(
            relay_csp_sources("https://legatum-relay.x.workers.dev"),
            vec![
                "https://legatum-relay.x.workers.dev",
                "wss://legatum-relay.x.workers.dev"
            ]
        );
        assert_eq!(
            relay_csp_sources("http://localhost:8787/base"),
            vec!["http://localhost:8787", "ws://localhost:8787"]
        );

        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let mut csp: Csp = serde_json::from_value(conf["app"]["security"]["csp"].clone()).unwrap();
        add_relay_to_csp(&mut csp, "https://r.example");
        add_relay_to_csp(&mut csp, "https://r.example");
        let Csp::DirectiveMap(map) = &csp else {
            panic!("tauri.conf.json uses a directive map")
        };
        let connect = joined(&map["connect-src"]);
        assert_eq!(
            connect,
            "'self' ipc: http://ipc.localhost https://r.example wss://r.example"
        );
        assert_eq!(joined(&map["default-src"]), "'self'");

        let mut map = HashMap::new();
        map.insert(
            "default-src".to_string(),
            CspDirectiveSources::Inline("'self'".into()),
        );
        let mut csp = Csp::DirectiveMap(map);
        add_relay_to_csp(&mut csp, "https://r.example");
        let Csp::DirectiveMap(map) = &csp else {
            unreachable!()
        };
        assert_eq!(
            joined(&map["connect-src"]),
            "'self' https://r.example wss://r.example"
        );

        let mut csp = Csp::Policy("default-src 'self'; connect-src 'self'".into());
        add_relay_to_csp(&mut csp, "https://r.example");
        assert_eq!(
            csp.to_string(),
            "default-src 'self'; connect-src 'self' https://r.example wss://r.example"
        );
    }

    #[test]
    fn host_names_are_trimmed_and_bounded() {
        assert_eq!(validate_host_name("  Sam ").unwrap(), "Sam");
        assert!(validate_host_name("   ").is_err());
        assert!(validate_host_name(&"é".repeat(80)).is_ok());
        assert!(validate_host_name(&"é".repeat(81)).is_err());
        assert!(validate_host_name("Sam\u{7}").is_err());
    }

    #[test]
    fn asset_paths_map_to_bundle_keys() {
        assert_eq!(asset_key("/").as_deref(), Some("index.html"));
        assert_eq!(asset_key("").as_deref(), Some("index.html"));
        assert_eq!(asset_key("/?join=1").as_deref(), Some("index.html"));
        assert_eq!(
            asset_key("/assets/index-abc.js").as_deref(),
            Some("assets/index-abc.js")
        );
        assert_eq!(asset_key("assets/a.css#x").as_deref(), Some("assets/a.css"));
        assert_eq!(asset_key("/art/").as_deref(), Some("art/index.html"));
        assert_eq!(asset_key("/../secret"), None);
        assert_eq!(asset_key("/a/./b.js"), None);
        assert_eq!(asset_key("/a\\b.js"), None);
    }

    #[test]
    fn mime_types_follow_the_extension() {
        assert_eq!(
            mime_for("assets/i.js"),
            Some("text/javascript; charset=utf-8")
        );
        assert_eq!(mime_for("x.wasm"), Some("application/wasm"));
        assert_eq!(mime_for("f/a.WOFF2"), Some("font/woff2"));
        assert_eq!(mime_for("README"), None);
        assert_eq!(mime_for(".hidden"), None);
        assert_eq!(mime_for("x.unknown"), None);
    }

    #[test]
    fn resolve_asset_keeps_page_fallback_but_404s_missing_files() {
        // Mimics Tauri's resolver: known files, else index.html.
        let lookup = |key: &str| -> Option<(Vec<u8>, String)> {
            match key {
                "assets/a.js" => Some((b"js".to_vec(), "text/html".into())),
                "art/x.png" => Some((vec![0x89, b'P'], "image/png".into())),
                "data/x.bin" => Some((vec![1], "application/octet-stream".into())),
                _ => Some((b"<html>".to_vec(), "text/html".into())),
            }
        };
        let (b, m) = resolve_asset("/", lookup).unwrap();
        assert_eq!(
            (b.as_slice(), m.as_str()),
            (&b"<html>"[..], "text/html; charset=utf-8")
        );
        // A page route without an extension gets the app (single-page fallback).
        assert_eq!(
            resolve_asset("/game", lookup).unwrap().1,
            "text/html; charset=utf-8"
        );
        // A real file Tauri guessed as HTML gets the right type from our table.
        let (b, m) = resolve_asset("/assets/a.js", lookup).unwrap();
        assert_eq!(
            (b.as_slice(), m.as_str()),
            (&b"js"[..], "text/javascript; charset=utf-8")
        );
        assert_eq!(resolve_asset("/art/x.png", lookup).unwrap().1, "image/png");
        assert_eq!(
            resolve_asset("/data/x.bin", lookup).unwrap().1,
            "application/octet-stream"
        );
        assert!(resolve_asset("/missing.js", lookup).is_none());
        assert!(resolve_asset("/../x", lookup).is_none());
    }

    #[test]
    fn origins_tell_the_server_page_from_the_bundled_ui() {
        let server = Url::parse("http://127.0.0.1:51234/").unwrap();
        assert!(is_server_url(&server, 51234));
        assert!(is_server_url(
            &Url::parse("http://127.0.0.1:51234/x?y").unwrap(),
            51234
        ));
        assert!(!is_server_url(&server, 7878));
        assert!(!is_server_url(
            &Url::parse("https://127.0.0.1:51234/").unwrap(),
            51234
        ));
        assert!(!is_server_url(
            &Url::parse("http://10.0.0.2:51234/").unwrap(),
            51234
        ));

        let mac = app_url(None, false);
        assert_eq!(mac.as_str(), "tauri://localhost/");
        assert_eq!(app_url(None, true).as_str(), "http://tauri.localhost/");
        let dev = Url::parse("http://localhost:5173/").unwrap();
        assert_eq!(app_url(Some(&dev), false), dev);

        assert!(is_app_url(&Url::parse("tauri://localhost/").unwrap(), &mac));
        assert!(is_app_url(
            &Url::parse("tauri://localhost/index.html").unwrap(),
            &mac
        ));
        assert!(!is_app_url(&server, &mac));
        assert!(!is_app_url(
            &Url::parse("http://localhost:5174/").unwrap(),
            &dev
        ));
    }

    #[test]
    fn remote_capability_reaches_only_the_loopback_page_and_two_commands() {
        let cap: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/online-remote.json")).unwrap();
        let urls: Vec<&str> = cap["remote"]["urls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u.as_str().unwrap())
            .collect();
        assert_eq!(urls, ["http://127.0.0.1:*"]);
        let pattern: tauri::utils::acl::RemoteUrlPattern = urls[0].parse().unwrap();
        assert!(pattern.test(&Url::parse("http://127.0.0.1:51234/").unwrap()));
        assert!(pattern.test(&Url::parse("http://127.0.0.1:7/game?x=1#y").unwrap()));
        assert!(!pattern.test(&Url::parse("http://192.168.1.2:51234/").unwrap()));
        assert!(!pattern.test(&Url::parse("https://127.0.0.1:51234/").unwrap()));
        assert!(!pattern.test(&Url::parse("http://127.0.0.1.evil.example:80/").unwrap()));
        let mut perms: Vec<&str> = cap["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .collect();
        perms.sort();
        assert_eq!(perms, ["allow-online-host-stop", "allow-online-info"]);
    }

    #[test]
    fn every_registered_command_is_declared_and_allowed_locally() {
        // build.rs declares the app's commands (which turns on ACL checks for all of them), so a
        // command missing from default.json would fail in the bundled UI at runtime.
        let lib = include_str!("lib.rs");
        let handler = lib
            .split("generate_handler![")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .unwrap();
        let build = include_str!("../build.rs");
        let default: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        let allowed: Vec<&str> = default["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| p.as_str())
            .collect();
        let mut count = 0;
        for path in handler.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let name = path.rsplit("::").next().unwrap();
            count += 1;
            assert!(
                build.contains(&format!("\"{name}\"")),
                "{name} not in build.rs"
            );
            let perm = format!("allow-{}", name.replace('_', "-"));
            assert!(
                allowed.contains(&perm.as_str()),
                "{perm} not in default.json"
            );
        }
        assert!(count >= 11);
    }
}

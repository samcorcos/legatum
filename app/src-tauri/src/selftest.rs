//! Online-hosting self-test: `LEGATUM_ONLINE_SELFTEST=<log file>` makes the app check, without
//! anyone clicking, the parts of online hosting that only a real window can show (navigation to
//! 127.0.0.1, IPC from that page, the way back). It appends `step=<name> ok|fail|warn <detail>`
//! lines to the file and to stderr, then `result=pass|fail`, and exits 0 or 1 (60 s at most).
//!
//! Off by default: without the variable nothing is managed and the hooks return at once. It
//! adds no command and no server route; results come back through `eval_with_callback`, which
//! reads a value the page's script left in `window.__legatumSelftest`.

use crate::online::{self, OnlineHost};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Manager, Runtime, Url};

const ENV: &str = "LEGATUM_ONLINE_SELFTEST";
const TOTAL: Duration = Duration::from_secs(60);
const STEP_WAIT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Ok,
    Fail,
    /// A failure that does not fail the run (the fallback way out).
    Warn,
}

/// The log file, when the variable is set to a non-empty path.
pub fn log_path(var: Option<std::ffi::OsString>) -> Option<std::path::PathBuf> {
    var.filter(|v| !v.is_empty()).map(Into::into)
}

/// One log line: `step=<name> ok|fail|warn <detail>`, detail on one line.
pub fn format_line(step: &str, outcome: Outcome, detail: &str) -> String {
    let word = match outcome {
        Outcome::Ok => "ok",
        Outcome::Fail => "fail",
        Outcome::Warn => "warn",
    };
    let detail: String = detail
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(400)
        .collect();
    format!("step={step} {word} {detail}")
        .trim_end()
        .to_string()
}

/// Exit code for the run: 1 if any step failed (warnings do not count) or none ran.
pub fn exit_code(outcomes: &[Outcome]) -> i32 {
    if outcomes.is_empty() || outcomes.contains(&Outcome::Fail) {
        1
    } else {
        0
    }
}

/// Script for one step: `body` runs with `put(ok, detail)` and `t` (the Tauri internals or
/// undefined) in scope; `put` stores the result under the step's name for `read_js` to fetch.
pub fn step_js(step: &str, body: &str) -> String {
    let key = serde_json::to_string(step).expect("string");
    format!(
        "(() => {{ const S = (window.__legatumSelftest = window.__legatumSelftest || {{}}); \
         const put = (ok, detail) => {{ S[{key}] = {{ ok: !!ok, detail: String(detail).slice(0, 300) }}; }}; \
         const t = window.__TAURI_INTERNALS__; \
         try {{ {body} }} catch (e) {{ put(false, 'threw: ' + e); }} }})()"
    )
}

/// Script that returns the step's stored result as a JSON string ("null" while pending).
pub fn read_js(step: &str) -> String {
    let key = serde_json::to_string(step).expect("string");
    format!("JSON.stringify((window.__legatumSelftest || {{}})[{key}] ?? null)")
}

/// A step result from `eval_with_callback`'s output: the JSON of the string `read_js` returns.
pub fn decode_result(raw: &str) -> Option<(bool, String)> {
    let mut value: serde_json::Value = serde_json::from_str(raw).ok()?;
    if let serde_json::Value::String(inner) = &value {
        value = serde_json::from_str(inner).ok()?;
    }
    let ok = value.get("ok")?.as_bool()?;
    let detail = value
        .get("detail")
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string();
    Some((ok, detail))
}

const STATUS_JS: &str = "fetch('/api/status', { cache: 'no-store' }).then(r => r.json()).then(\
    s => put(s.is_local === true && s.mode === 'server', 'is_local=' + s.is_local + ' mode=' + s.mode), \
    e => put(false, 'fetch failed: ' + e));";
const IPC_INFO_JS: &str = "if (!t) { put(false, 'no __TAURI_INTERNALS__ on this page'); return; } \
    t.invoke('online_info').then(i => put(i && typeof i.build === 'string' && typeof i.protocol === 'number', \
    JSON.stringify(i)), e => put(false, 'rejected: ' + e));";
/// `training_poll` has no side effects and succeeds in the bundled UI, so a success here means
/// the remote capability is too wide.
const IPC_DENIED_JS: &str =
    "if (!t) { put(false, 'no __TAURI_INTERNALS__ on this page'); return; } \
    t.invoke('training_poll').then(v => put(false, 'allowed, returned ' + JSON.stringify(v)), \
    e => put(true, 'rejected: ' + e));";
const LEAVE_JS: &str = "if (!t) { put(false, 'no __TAURI_INTERNALS__ on this page'); return; } \
    t.invoke('online_host_stop').catch(e => put(false, 'rejected: ' + e));";

/// Page loads of the main window, from the plugin's `on_page_load`.
struct Events(Mutex<mpsc::Sender<Url>>);

/// Called from the plugin's setup: manages the event channel and starts the runner when the
/// variable is set.
pub fn setup<R: Runtime>(app: &tauri::AppHandle<R>) {
    let Some(path) = log_path(std::env::var_os(ENV)) else {
        return;
    };
    let (tx, rx) = mpsc::channel();
    app.manage(Events(Mutex::new(tx)));
    let log = Arc::new(Log {
        path,
        done: AtomicBool::new(false),
    });
    let watchdog = log.clone();
    let app_for_watchdog = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(TOTAL);
        watchdog.finish(&app_for_watchdog, &[Outcome::Fail], "timeout after 60 s");
    });
    let app = app.clone();
    std::thread::spawn(move || {
        let runner = Runner {
            app: &app,
            rx,
            deadline: Instant::now() + TOTAL - Duration::from_secs(1),
            log: &log,
            outcomes: Vec::new(),
        };
        let outcomes = runner.run();
        log.finish(&app, &outcomes, "");
    });
}

/// Called from the plugin's `on_page_load` (Finished). Does nothing outside self-test mode.
pub fn page_loaded<R: Runtime>(webview: &tauri::Webview<R>, url: &Url) {
    if webview.label() != "main" {
        return;
    }
    if let Some(events) = webview.app_handle().try_state::<Events>() {
        if let Ok(tx) = events.0.lock() {
            let _ = tx.send(url.clone());
        }
    }
}

struct Log {
    path: std::path::PathBuf,
    done: AtomicBool,
}

impl Log {
    fn line(&self, line: &str) {
        eprintln!("legatum selftest: {line}");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(f, "{line}");
        }
    }

    /// Write the verdict once, stop any server (so the relay gets `bye`) and exit.
    fn finish<R: Runtime>(&self, app: &tauri::AppHandle<R>, outcomes: &[Outcome], why: &str) {
        if self.done.swap(true, Ordering::SeqCst) {
            return;
        }
        if !why.is_empty() {
            self.line(&format_line("timeout", Outcome::Fail, why));
        }
        let code = exit_code(outcomes);
        self.line(&format!(
            "result={}",
            if code == 0 { "pass" } else { "fail" }
        ));
        if let Some(host) = app.try_state::<OnlineHost>() {
            host.stop_blocking();
        }
        std::process::exit(code);
    }
}

struct Runner<'a, R: Runtime> {
    app: &'a tauri::AppHandle<R>,
    rx: mpsc::Receiver<Url>,
    deadline: Instant,
    log: &'a Log,
    outcomes: Vec<Outcome>,
}

impl<R: Runtime> Runner<'_, R> {
    fn record(&mut self, step: &str, outcome: Outcome, detail: &str) -> bool {
        self.log.line(&format_line(step, outcome, detail));
        self.outcomes.push(outcome);
        outcome == Outcome::Ok
    }

    fn left(&self, cap: Duration) -> Duration {
        cap.min(self.deadline.saturating_duration_since(Instant::now()))
    }

    /// The next main-window page load matching `want`, within `cap`.
    fn wait_load(&self, cap: Duration, want: impl Fn(&Url) -> bool) -> Option<Url> {
        let end = Instant::now() + self.left(cap);
        loop {
            let wait = end.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(wait) {
                Ok(url) if want(&url) => return Some(url),
                Ok(_) => continue,
                Err(_) => return None,
            }
        }
    }

    fn drain(&self) {
        while self.rx.try_recv().is_ok() {}
    }

    fn window(&self) -> Result<tauri::WebviewWindow<R>, String> {
        self.app
            .get_webview_window("main")
            .ok_or_else(|| "no main window".to_string())
    }

    /// Run `body` on the page and wait for its `put`.
    fn page_step(&self, step: &str, body: &str) -> (bool, String) {
        let window = match self.window() {
            Ok(w) => w,
            Err(e) => return (false, e),
        };
        if let Err(e) = window.eval(step_js(step, body)) {
            return (false, format!("eval failed: {e}"));
        }
        self.poll(&window, step, STEP_WAIT)
            .unwrap_or((false, "no answer from the page".into()))
    }

    fn poll(
        &self,
        window: &tauri::WebviewWindow<R>,
        step: &str,
        cap: Duration,
    ) -> Option<(bool, String)> {
        let end = Instant::now() + self.left(cap);
        while Instant::now() < end {
            let (tx, rx) = mpsc::channel();
            let _ = window.eval_with_callback(read_js(step), move |raw| {
                let _ = tx.send(raw);
            });
            if let Ok(raw) = rx.recv_timeout(Duration::from_millis(500)) {
                if let Some(result) = decode_result(&raw) {
                    return Some(result);
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        None
    }

    /// Start the host as `online_host_start` does and load its page in the main window.
    fn host_and_navigate(&mut self, start_step: &str, nav_step: &str) -> Option<u16> {
        let started = match online::start_host(self.app, "Selftest") {
            Ok(s) => s,
            Err(e) => {
                self.record(start_step, Outcome::Fail, &e);
                return None;
            }
        };
        let Some(port) = self.app.state::<OnlineHost>().port() else {
            self.record(
                start_step,
                Outcome::Fail,
                "started, but no server is recorded",
            );
            return None;
        };
        let room = started.room.as_deref().unwrap_or("none");
        self.record(
            start_step,
            Outcome::Ok,
            &format!("port={port} room={room} url={}", started.url),
        );
        self.drain();
        let navigated = Url::parse(&started.url)
            .map_err(|e| e.to_string())
            .and_then(|url| {
                self.window()?
                    .navigate(url)
                    .map_err(|e| format!("navigate failed: {e}"))
            });
        if let Err(e) = navigated {
            self.record(nav_step, Outcome::Fail, &e);
            return None;
        }
        match self.wait_load(STEP_WAIT, |u| online::is_server_url(u, port)) {
            Some(url) => {
                self.record(nav_step, Outcome::Ok, &format!("loaded {url}"));
                Some(port)
            }
            None => {
                self.record(nav_step, Outcome::Fail, "no page load on 127.0.0.1");
                None
            }
        }
    }

    fn port_closed(&self, port: u16, cap: Duration) -> bool {
        let end = Instant::now() + self.left(cap);
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        loop {
            if std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_err() {
                return true;
            }
            if Instant::now() >= end {
                return false;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    fn run(mut self) -> Vec<Outcome> {
        let app_url = online::bundled_ui_url(self.app);
        // The bundled page must be up first (the app was launched normally).
        if self
            .wait_load(Duration::from_secs(30), |u| online::is_app_url(u, &app_url))
            .is_none()
        {
            self.record("boot", Outcome::Fail, "the bundled page never loaded");
            return self.outcomes;
        }
        let Some(port) = self.host_and_navigate("start", "navigate") else {
            return self.outcomes;
        };
        for (step, body) in [
            ("status", STATUS_JS),
            ("ipc_info", IPC_INFO_JS),
            ("ipc_denied", IPC_DENIED_JS),
        ] {
            let (ok, detail) = self.page_step(step, body);
            self.record(step, if ok { Outcome::Ok } else { Outcome::Fail }, &detail);
        }

        // leave: the Leave game path, online_host_stop from the page.
        self.drain();
        let window = self.window();
        let leave = match &window {
            Ok(w) => w
                .eval(step_js("leave", LEAVE_JS))
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.clone()),
        };
        match leave {
            Err(e) => {
                self.record("leave", Outcome::Fail, &format!("eval failed: {e}"));
            }
            Ok(()) => match self.wait_load(STEP_WAIT, |u| online::is_app_url(u, &app_url)) {
                Some(url) if self.port_closed(port, Duration::from_secs(5)) => {
                    self.record("leave", Outcome::Ok, &format!("back on {url}, port closed"));
                }
                Some(url) => {
                    self.record(
                        "leave",
                        Outcome::Fail,
                        &format!("back on {url} but port {port} still open"),
                    );
                }
                None => {
                    let why = window
                        .ok()
                        .and_then(|w| self.poll(&w, "leave", Duration::from_secs(2)))
                        .map_or("no page load of the app origin".into(), |(_, d)| d);
                    self.record("leave", Outcome::Fail, &why);
                    return self.outcomes;
                }
            },
        }

        // leave_fallback: the page navigates itself to the app origin; the hook stops the server.
        let fallback = self.leave_fallback(&app_url);
        let (outcome, detail) = match fallback {
            Ok(d) => (Outcome::Ok, d),
            Err(d) => (Outcome::Warn, d),
        };
        self.record("leave_fallback", outcome, &detail);
        self.outcomes
    }

    fn leave_fallback(&mut self, app_url: &Url) -> Result<String, String> {
        let started =
            online::start_host(self.app, "Selftest").map_err(|e| format!("start: {e}"))?;
        let port = self
            .app
            .state::<OnlineHost>()
            .port()
            .ok_or("no port after start")?;
        self.drain();
        let window = self.window()?;
        let url = Url::parse(&started.url).map_err(|e| e.to_string())?;
        window.navigate(url).map_err(|e| e.to_string())?;
        self.wait_load(STEP_WAIT, |u| online::is_server_url(u, port))
            .ok_or("no page load on 127.0.0.1")?;
        self.drain();
        let target = serde_json::to_string(app_url.as_str()).expect("string");
        window
            .eval(format!("location.href = {target};"))
            .map_err(|e| e.to_string())?;
        let back = self
            .wait_load(STEP_WAIT, |u| online::is_app_url(u, app_url))
            .ok_or_else(|| format!("no page load of {app_url} after location.href"))?;
        let stopped = self.app.state::<OnlineHost>().port().is_none();
        if stopped && self.port_closed(port, Duration::from_secs(5)) {
            Ok(format!("back on {back}, hook stopped the server"))
        } else {
            Err(format!(
                "back on {back} but the server on {port} still runs"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_unless_the_variable_names_a_file() {
        assert_eq!(log_path(None), None);
        assert_eq!(log_path(Some("".into())), None);
        assert_eq!(
            log_path(Some("/tmp/s.log".into())),
            Some(std::path::PathBuf::from("/tmp/s.log"))
        );
    }

    #[test]
    fn lines_are_one_per_step() {
        assert_eq!(
            format_line("start", Outcome::Ok, "port=1 room=none"),
            "step=start ok port=1 room=none"
        );
        assert_eq!(
            format_line("leave", Outcome::Fail, "a\nb"),
            "step=leave fail a b"
        );
        assert_eq!(
            format_line("leave_fallback", Outcome::Warn, ""),
            "step=leave_fallback warn"
        );
    }

    #[test]
    fn warnings_do_not_fail_the_run() {
        assert_eq!(exit_code(&[Outcome::Ok, Outcome::Warn]), 0);
        assert_eq!(exit_code(&[Outcome::Ok, Outcome::Fail, Outcome::Warn]), 1);
        assert_eq!(exit_code(&[]), 1);
    }

    #[test]
    fn page_results_round_trip_through_eval_output() {
        // eval_with_callback hands back the JSON of the string read_js returns.
        let inner = r#"{"ok":true,"detail":"is_local=true mode=server"}"#;
        let raw = serde_json::to_string(inner).unwrap();
        assert_eq!(
            decode_result(&raw),
            Some((true, "is_local=true mode=server".into()))
        );
        assert!(decode_result(inner).unwrap().0);
        assert_eq!(decode_result("\"null\""), None);
        assert_eq!(decode_result(""), None);
        assert_eq!(decode_result("null"), None);
    }

    #[test]
    fn step_scripts_store_under_their_own_key() {
        let js = step_js("ipc_info", IPC_INFO_JS);
        assert!(js.contains(r#"S["ipc_info"] ="#));
        assert!(js.contains("t.invoke('online_info')"));
        assert!(read_js("ipc_info").contains(r#"["ipc_info"]"#));
        // The denied probe must use a command the remote capability does not grant.
        let remote: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/online-remote.json")).unwrap();
        let granted = remote["permissions"].to_string();
        assert!(!granted.contains("training-poll"));
        assert!(IPC_DENIED_JS.contains("'training_poll'"));
    }
}

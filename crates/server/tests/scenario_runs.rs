use legatum_server::Server;
use serde_json::json;
use std::{fs, net::SocketAddr, path::Path, time::SystemTime};
use tempfile::TempDir;
use tokio::task::JoinHandle;

struct Running {
    addr: SocketAddr,
    _task: JoinHandle<()>,
    root: TempDir,
    http: reqwest::Client,
}

impl Running {
    async fn start() -> Self {
        let root = tempfile::tempdir().unwrap();
        let static_dir = root.path().join("static");
        fs::create_dir_all(&static_dir).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = Server::open_with_archive(
            root.path().join("saves"),
            addr.port(),
            vec![format!("http://192.168.1.2:{}", addr.port())],
            root.path().join("archive"),
            5,
        )
        .unwrap()
        .router(static_dir);
        let app = app.layer(axum::middleware::from_fn(
            |mut request: axum::extract::Request, next: axum::middleware::Next| async move {
                request.extensions_mut().insert(axum::extract::ConnectInfo(
                    "192.168.1.20:1234".parse::<SocketAddr>().unwrap(),
                ));
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
            _task: task,
            root,
            http: reqwest::Client::builder().no_proxy().build().unwrap(),
        }
    }

    fn scenarios(&self) -> std::path::PathBuf {
        self.root.path().join("saves/scenarios")
    }

    async fn upload(&self, body: String) -> reqwest::Response {
        self.http
            .post(format!("http://{}/api/scenario-runs", self.addr))
            .header("Host", format!("192.168.1.2:{}", self.addr.port()))
            .header("Origin", format!("http://192.168.1.2:{}", self.addr.port()))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap()
    }
}

fn run(id: &str, n: usize) -> String {
    json!({
        "format": "legatum-scenario-run",
        "scenario": { "id": id },
        "finished_at": format!("2026-10-18T00:00:{n:02}Z"),
    })
    .to_string()
}

fn file_count(dir: &Path) -> usize {
    fs::read_dir(dir).unwrap().count()
}

#[tokio::test]
async fn oversized_upload_has_a_clear_error() {
    let server = Running::start().await;
    let body = json!({
        "format": "legatum-scenario-run",
        "scenario": { "id": "large" },
        "finished_at": "2026-10-18T00:00:00Z",
        "padding": "x".repeat(2 * 1024 * 1024),
    })
    .to_string();
    let response = server.upload(body).await;
    assert_eq!(response.status(), 413);
    assert!(
        response
            .text()
            .await
            .unwrap()
            .contains("2 MiB upload limit")
    );
    assert!(!server.scenarios().exists());
}

#[tokio::test]
async fn upload_501_evicts_the_oldest_run() {
    let server = Running::start().await;
    assert_eq!(server.upload(run("oldest", 0)).await.status(), 200);
    let oldest = server.scenarios().join("oldest-2026-10-18T00-00-00Z.json");
    fs::File::options()
        .write(true)
        .open(&oldest)
        .unwrap()
        .set_modified(SystemTime::UNIX_EPOCH)
        .unwrap();
    for n in 0..500 {
        let response = server.upload(run(&format!("run-{n:03}"), 0)).await;
        assert_eq!(
            response.status(),
            200,
            "upload {n}: {}",
            response.text().await.unwrap()
        );
    }
    assert_eq!(file_count(&server.scenarios()), 500);
    assert!(!oldest.exists());
    assert!(
        server
            .scenarios()
            .join("run-000-2026-10-18T00-00-00Z.json")
            .exists()
    );
    assert!(
        server
            .scenarios()
            .join("run-499-2026-10-18T00-00-00Z.json")
            .exists()
    );
}

#[tokio::test]
async fn path_like_id_is_rejected() {
    let server = Running::start().await;
    let response = server.upload(run("../escape", 0)).await;
    assert_eq!(response.status(), 400);
    assert!(response.text().await.unwrap().contains("invalid id"));
    assert!(!server.scenarios().exists());
}

#[tokio::test]
async fn byte_budget_evicts_oldest_file_before_upload() {
    let server = Running::start().await;
    fs::create_dir_all(server.scenarios()).unwrap();
    let old = server.scenarios().join("old.json");
    let recent = server.scenarios().join("recent.json");
    let file = fs::File::create(&old).unwrap();
    file.set_len(60 * 1024 * 1024).unwrap();
    file.set_modified(SystemTime::UNIX_EPOCH).unwrap();
    fs::File::create(&recent)
        .unwrap()
        .set_len(40 * 1024 * 1024)
        .unwrap();
    assert_eq!(server.upload(run("new", 0)).await.status(), 200);
    assert!(!old.exists());
    assert!(recent.exists());
    assert_eq!(file_count(&server.scenarios()), 2);
}

#[tokio::test]
async fn reply_names_the_file_relative_to_saves_and_carries_security_headers() {
    let server = Running::start().await;
    let response = server.upload(run("relative", 0)).await;
    assert_eq!(response.status(), 200);
    for (name, value) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "no-referrer"),
    ] {
        assert_eq!(response.headers()[name], value);
    }
    let body: serde_json::Value = response.json().await.unwrap();
    // The host's absolute path would name its user; only the saves-relative name is sent.
    assert_eq!(body["path"], "scenarios/relative-2026-10-18T00-00-00Z.json");
}

#[test]
fn a_server_whose_archive_cannot_open_still_starts() {
    let root = tempfile::tempdir().unwrap();
    // A file where the archive folder should be: the archive is off, the game still hosts.
    let blocked = root.path().join("archive");
    fs::write(&blocked, "").unwrap();
    assert!(Server::open_with_archive(root.path().join("saves"), 7878, vec![], blocked, 5).is_ok());
    assert!(Server::open_with_archive(root.path().join("saves2"), 7878, vec![], None, 5).is_ok());
}

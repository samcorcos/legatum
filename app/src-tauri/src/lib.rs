// The game runs in WASM; native archive commands compress and finalize plugin-fs writes.
// Online hosting runs legatum-server inside the app (online.rs, plans/online-play.md).

mod online;
mod scenario_dropins;
mod selftest;

use tauri::Manager;
use tauri_plugin_fs::FsExt;

const MAX_ARCHIVE_JSON_BYTES: usize = 64 * 1024 * 1024;
const MAX_SCENARIO_RUN_JSON_BYTES: usize = 64 * 1024 * 1024;

fn check_json_size(json: &str, limit: usize) -> Result<(), String> {
    if json.len() > limit {
        return Err(format!("JSON input exceeds the {limit} byte limit"));
    }
    Ok(())
}

fn parse_archive_json(json: &str) -> Result<legatum_archive::Recording, String> {
    check_json_size(json, MAX_ARCHIVE_JSON_BYTES)?;
    serde_json::from_str(json).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct StagedArchive {
    ticket: String,
    path: String,
    bytes: Vec<u8>,
}

/// The local game archive, or `None` when its folder could not be opened at launch: the game
/// then runs without a record rather than refusing to start.
struct LocalArchive(Option<legatum_archive::Archive>);

impl LocalArchive {
    fn get(&self) -> Result<&legatum_archive::Archive, String> {
        self.0
            .as_ref()
            .ok_or_else(|| "the game archive is off".to_owned())
    }
}

/// `None` when the archive is off: the web side keeps the history on this device.
#[tauri::command]
fn archive_stage(
    json: String,
    archive: tauri::State<'_, LocalArchive>,
) -> Result<Option<StagedArchive>, String> {
    match &archive.0 {
        Some(archive) => archive_stage_inner(&json, archive.root()).map(Some),
        None => Ok(None),
    }
}

fn archive_stage_inner(
    json: &str,
    archive_root: &std::path::Path,
) -> Result<StagedArchive, String> {
    let recording = parse_archive_json(json)?;
    let bytes = legatum_archive::encode(&recording)?;
    let ticket = format!(
        "{}-{}",
        recording.metadata.game_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let directory = legatum_archive::outside_repo(&archive_root.join("incoming"))?;
    if !directory.starts_with(archive_root) {
        return Err("incoming directory escapes archive".into());
    }
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    Ok(StagedArchive {
        path: directory
            .join(format!("{ticket}.tape.json.zst"))
            .to_string_lossy()
            .into(),
        ticket,
        bytes,
    })
}

#[tauri::command]
fn archive_commit(ticket: String, archive: tauri::State<'_, LocalArchive>) -> Result<(), String> {
    let archive = archive.get()?;
    if ticket.is_empty()
        || ticket.len() > 200
        || !ticket
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid archive ticket".into());
    }
    let path = legatum_archive::outside_repo(
        &archive
            .root()
            .join("incoming")
            .join(format!("{ticket}.tape.json.zst")),
    )?;
    if !path.starts_with(archive.root()) {
        return Err("archive ticket escapes root".into());
    }
    let recording = legatum_archive::read(&path)?;
    archive.write(&recording)?;
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

/// `~/LegatumData`, where scenario runs, drop-ins and `app-info.json` live; where HOME is unset
/// (Windows), the platform data folder (`%APPDATA%\Legatum`).
pub(crate) fn legatum_data_dir() -> Result<std::path::PathBuf, String> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(|h| std::path::PathBuf::from(h).join("LegatumData"))
        .or_else(legatum_archive::data_dir)
        .ok_or_else(|| "no home folder found".to_owned())
}

/// Save a finished scenario run as ~/LegatumData/scenarios/<name> (name: `<id>-<time>.json`,
/// built by app/src/scenarios/scenarios.ts::runFileName). Returns the full path.
#[tauri::command]
fn scenario_save_run(name: String, json: String) -> Result<String, String> {
    check_json_size(&json, MAX_SCENARIO_RUN_JSON_BYTES)?;
    let stem = name.strip_suffix(".json").unwrap_or_default();
    if stem.is_empty()
        || stem.len() > 120
        || !stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid scenario run name".into());
    }
    let run: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    if run["format"] != "legatum-scenario-run" {
        return Err("not a scenario run".into());
    }
    let dir = legatum_data_dir()?.join("scenarios");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&name);
    let temp = dir.join(format!("{stem}.json.tmp"));
    std::fs::write(&temp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&temp, &path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().into())
}

/// Training mode: the battle search for one turn, on a native thread over every core.
#[derive(Default)]
struct Training {
    job: std::sync::Mutex<Option<TrainingJob>>,
}

struct TrainingJob {
    turn: u32,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    progress: std::sync::Arc<std::sync::Mutex<Option<legatum_ai::training::TrainingProgress>>>,
    result: std::sync::Arc<std::sync::Mutex<Option<Result<String, String>>>>,
}

/// Start this turn's search (the request comes from the WASM game's `training_request`).
/// A running search for an earlier turn is cancelled.
#[tauri::command]
fn training_start(
    request: String,
    cap_ms: u64,
    training: tauri::State<'_, Training>,
) -> Result<(), String> {
    check_json_size(&request, MAX_ARCHIVE_JSON_BYTES)?;
    let req: legatum_ai::training::TrainingRequest =
        serde_json::from_str(&request).map_err(|e| e.to_string())?;
    let mut slot = training.job.lock().map_err(|e| e.to_string())?;
    if let Some(old) = slot.as_ref() {
        if old.turn == req.state.turn {
            return Ok(());
        }
        old.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let job = TrainingJob {
        turn: req.state.turn,
        cancel: Default::default(),
        progress: Default::default(),
        result: Default::default(),
    };
    let (cancel, progress, result) = (job.cancel.clone(), job.progress.clone(), job.result.clone());
    std::thread::Builder::new()
        .name("training".into())
        .stack_size(16 << 20)
        .spawn(move || {
            let cfg = legatum_ai::training::TrainingConfig {
                cap_ms,
                ..Default::default()
            };
            let start = std::time::Instant::now();
            let clock = || {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    u64::MAX
                } else {
                    start.elapsed().as_millis() as u64
                }
            };
            let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                legatum_ai::training::search_turn(&req.state, req.hard, &cfg, clock, |p| {
                    if let Ok(mut slot) = progress.lock() {
                        *slot = Some(p.clone());
                    }
                })
            }));
            let out = match run {
                Ok(r) => serde_json::to_string(&r).map_err(|e| e.to_string()),
                Err(_) => Err("training search panicked".into()),
            };
            if let Ok(mut slot) = result.lock() {
                *slot = Some(out);
            }
        })
        .map_err(|e| e.to_string())?;
    *slot = Some(job);
    Ok(())
}

/// `{"progress":…,"result":TrainingResult JSON string or null}` for the running search.
#[tauri::command]
fn training_poll(training: tauri::State<'_, Training>) -> Result<String, String> {
    let slot = training.job.lock().map_err(|e| e.to_string())?;
    let Some(job) = slot.as_ref() else {
        return Ok("null".into());
    };
    let progress = job.progress.lock().map_err(|e| e.to_string())?.clone();
    let result = job.result.lock().map_err(|e| e.to_string())?.clone();
    if let Some(Err(e)) = result {
        return Err(e);
    }
    serde_json::to_string(&serde_json::json!({
        "turn": job.turn,
        "progress": progress,
        "result": result.and_then(Result::ok),
    }))
    .map_err(|e| e.to_string())
}

fn valid_game_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 120
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Append training log lines to `<app data>/training/<game_id>.jsonl`. Returns the path.
#[tauri::command]
fn training_log_append(
    app: tauri::AppHandle,
    game_id: String,
    lines: Vec<String>,
) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("training");
    training_log_append_inner(&dir, &game_id, &lines)
}

/// Append Training-question answers to `<app data>/training/answers/<game_id>.jsonl`.
#[tauri::command]
fn why_log_append(
    app: tauri::AppHandle,
    game_id: String,
    lines: Vec<String>,
) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("training")
        .join("answers");
    training_log_append_inner(&dir, &game_id, &lines)
}

fn training_log_append_inner(
    dir: &std::path::Path,
    game_id: &str,
    lines: &[String],
) -> Result<String, String> {
    use std::io::Write;
    if !valid_game_id(game_id) {
        return Err("invalid game id".into());
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{game_id}.jsonl"));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    for line in lines {
        if line.contains('\n') {
            return Err("a log line holds a newline".into());
        }
        serde_json::from_str::<serde_json::Value>(line).map_err(|e| e.to_string())?;
        writeln!(file, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(path.to_string_lossy().into())
}

fn open_archive(flag: Option<std::path::PathBuf>) -> Result<legatum_archive::Archive, String> {
    legatum_archive::Archive::open(legatum_archive::archive_root(flag)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut context = tauri::generate_context!();
    // The relay URL is a build input: its origins join the CSP's connect-src here.
    online::configure_csp(context.config_mut());
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(online::plugin())
        .setup(|app| {
            let args: Vec<_> = std::env::args().collect();
            let flag = args
                .windows(2)
                .find(|a| a[0] == "--archive-dir")
                .map(|a| std::path::PathBuf::from(&a[1]));
            // The archive is a record, never a reason to refuse to start: on any failure the
            // game runs without it, with one log line.
            let archive = open_archive(flag).and_then(|archive| {
                app.fs_scope()
                    .allow_directory(archive.root(), true)
                    .map_err(|e| e.to_string())?;
                Ok(archive)
            });
            if let Err(e) = &archive {
                eprintln!("legatum: game archive off: {e}");
            }
            app.manage(LocalArchive(archive.ok()));
            app.manage(Training::default());
            // The training log's folder exists from launch, so a missing folder never hides
            // whether the search ran (`training_log_append` writes into it).
            if let Ok(dir) = app.path().app_data_dir() {
                if let Err(e) = std::fs::create_dir_all(dir.join("training")) {
                    eprintln!("legatum: training log folder not created: {e}");
                }
            }
            // Tells `tools/scenarios/publish.sh` which rules version this install plays.
            if let Ok(dir) = legatum_data_dir() {
                if let Err(e) = scenario_dropins::write_app_info(&dir) {
                    eprintln!("legatum: app-info.json not written: {e}");
                }
            }
            // Leave a core for the window while the search runs.
            if std::env::var_os("LEGATUM_ROLLOUT_THREADS").is_none() {
                let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
                std::env::set_var(
                    "LEGATUM_ROLLOUT_THREADS",
                    cores.saturating_sub(1).max(1).to_string(),
                );
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            archive_stage,
            archive_commit,
            scenario_save_run,
            scenario_dropins::scenario_dropins,
            training_start,
            training_poll,
            training_log_append,
            why_log_append,
            online::online_host_start,
            online::online_host_stop,
            online::online_info
        ])
        .run(context)
        .expect("error while running Legatum");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn training_log_lines_append_as_json_lines_under_a_safe_game_id() {
        let dir =
            std::env::temp_dir().join(format!("legatum-training-test-{}", std::process::id()));
        let lines = vec![r#"{"type":"decision","turn":3}"#.to_string()];
        let path = training_log_append_inner(&dir, "game-1", &lines).unwrap();
        training_log_append_inner(&dir, "game-1", &lines).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap().lines().count(), 2);
        assert!(training_log_append_inner(&dir, "../x", &lines).is_err());
        assert!(training_log_append_inner(&dir, "g", &["not json".into()]).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn oversized_archive_and_scenario_commands_reject_before_parsing_or_writing() {
        let archive = " ".repeat(MAX_ARCHIVE_JSON_BYTES + 1);
        assert!(
            archive_stage_inner(&archive, std::path::Path::new("/unused"))
                .err()
                .unwrap()
                .contains("byte limit")
        );
        let scenario = " ".repeat(MAX_SCENARIO_RUN_JSON_BYTES + 1);
        assert!(scenario_save_run("example.json".into(), scenario)
            .unwrap_err()
            .contains("byte limit"));
    }

    #[test]
    fn an_archive_that_cannot_open_is_off_not_fatal() {
        let dir = std::env::temp_dir().join(format!("legatum-archive-off-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // A file where the archive folder should be cannot open as an archive.
        let blocked = dir.join("archive");
        std::fs::write(&blocked, "").unwrap();
        assert!(open_archive(Some(blocked)).is_err());
        let off = LocalArchive(None);
        assert!(off.get().is_err());
        // A home folder tracked with git is fine.
        std::fs::create_dir_all(dir.join("home/.git")).unwrap();
        assert!(open_archive(Some(dir.join("home/LegatumData/archive"))).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }
}

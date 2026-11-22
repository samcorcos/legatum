//! Scenario files dropped into the user's folder (`~/LegatumData/scenarios/lab/`), listed by the
//! Scenario Lab next to the bundled ones. A file there needs no build or install: the Lab reads
//! the folder each time it opens (`tools/scenarios/publish.sh` puts files there).
//!
//! Every file is checked with `legatum_wasm::scenario::check_file`, the check the engine runs
//! when a scenario starts. A file that fails is still returned, with the reason, so the Lab can
//! list it greyed out. Nothing here panics on a bad file.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// A scenario file larger than this is not read (the biggest bundled one is about 1 MB).
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
/// Files read from the folder at most, so a stray pile cannot stall the Lab.
const MAX_FILES: usize = 200;

#[derive(Debug, Serialize, PartialEq)]
pub struct Dropin {
    /// The file's name in the folder.
    pub file: String,
    /// The file's text when it is readable, so the app can parse it like a bundled scenario.
    pub text: Option<String>,
    /// Why the Lab cannot offer it, in plain words. `None` when it is good.
    pub reason: Option<String>,
}

/// The folder: `~/LegatumData/scenarios/lab/` (the same `LegatumData` the app saves runs into).
pub fn dropin_dir() -> Result<PathBuf, String> {
    Ok(crate::legatum_data_dir()?.join("scenarios/lab"))
}

/// Every `*.json` in `dir`, sorted by name; an absent folder is an empty list.
pub fn load_dropins(dir: &Path) -> Vec<Dropin> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".json") && !n.starts_with('.'))
        .collect();
    names.sort();
    names.truncate(MAX_FILES);
    names
        .into_iter()
        .map(|file| {
            let path = dir.join(&file);
            let result = match std::fs::metadata(&path) {
                Ok(m) if m.len() > MAX_FILE_BYTES => Err("larger than 16 MB".to_string()),
                Ok(_) => std::fs::read_to_string(&path).map_err(|e| format!("cannot read: {e}")),
                Err(e) => Err(format!("cannot read: {e}")),
            };
            match result {
                Err(reason) => Dropin {
                    file,
                    text: None,
                    reason: Some(reason),
                },
                Ok(text) => {
                    // A panic inside the engine's check must not take the Lab down with it.
                    let checked =
                        std::panic::catch_unwind(|| legatum_wasm::scenario::check_file(&text))
                            .unwrap_or_else(|_| Err("the engine could not read this file".into()));
                    Dropin {
                        file,
                        reason: checked.err(),
                        text: Some(text),
                    }
                }
            }
        })
        .collect()
}

/// The Lab calls this each time it opens.
#[tauri::command]
pub fn scenario_dropins() -> Result<Vec<Dropin>, String> {
    Ok(load_dropins(&dropin_dir()?))
}

/// Tell `tools/scenarios/publish.sh` which rules version this install plays, so it can check a
/// scenario against the installed app instead of the repository's newest engine.
pub fn write_app_info(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let info = serde_json::json!({ "rules_version": legatum_core::RULES_VERSION });
    std::fs::write(dir.join("app-info.json"), info.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_wasm::scenario::{decode_state, ScenarioFile};

    /// A real Lab scenario, so the test reads the same files the app ships.
    const REAL: &str = include_str!("../../../scenarios/lab/athens-break-the-hold.json");

    /// The same scenario with its state re-saved as plain JSON under another rules version.
    fn made_for(version: u32) -> String {
        let file = ScenarioFile::parse(REAL).unwrap();
        let state = decode_state(&file.state).unwrap();
        let mut json: serde_json::Value =
            serde_json::from_str(&legatum_wasm::json::state_to_json(&state)).unwrap();
        json["save_version"] = serde_json::Value::from(version);
        let mut out: serde_json::Value = serde_json::from_str(REAL).unwrap();
        out["state"] = serde_json::json!({
            "encoding": "json", "hash": file.state.hash, "data": json.to_string(),
        });
        out.to_string()
    }

    #[test]
    fn the_folder_lists_good_files_and_names_the_reason_for_bad_ones() {
        let dir = std::env::temp_dir().join(format!("legatum-dropins-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(load_dropins(&dir).is_empty(), "an absent folder is empty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("good.json"), REAL).unwrap();
        std::fs::write(dir.join("garbage.json"), "{ not json").unwrap();
        std::fs::write(dir.join("notes.txt"), "ignored").unwrap();
        // A run file the app saved is not a scenario: listed with a reason, never crashing.
        std::fs::write(dir.join("run.json"), r#"{"format":"legatum-scenario-run"}"#).unwrap();
        // Made for another rules version: the reason says which, and the text is still returned.
        std::fs::write(
            dir.join("old.json"),
            made_for(legatum_core::RULES_VERSION + 1),
        )
        .unwrap();
        // Re-saved under the current version it loads: the helper itself is sound.
        std::fs::write(dir.join("same.json"), made_for(legatum_core::RULES_VERSION)).unwrap();

        let found = load_dropins(&dir);
        let names: Vec<&str> = found.iter().map(|d| d.file.as_str()).collect();
        assert_eq!(
            names,
            [
                "garbage.json",
                "good.json",
                "old.json",
                "run.json",
                "same.json"
            ]
        );
        assert_eq!(found[1].reason, None);
        assert_eq!(found[4].reason, None);
        assert!(found[1].text.is_some());
        assert!(found[0]
            .reason
            .as_deref()
            .unwrap()
            .contains("invalid scenario"));
        assert_eq!(
            found[2].reason.as_deref().unwrap(),
            format!(
                "made for rules version {}; this app is {}",
                legatum_core::RULES_VERSION + 1,
                legatum_core::RULES_VERSION
            )
        );
        assert!(found[3].reason.is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn app_info_names_the_rules_version() {
        let dir = std::env::temp_dir().join(format!("legatum-appinfo-test-{}", std::process::id()));
        write_app_info(&dir).unwrap();
        let info: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("app-info.json")).unwrap())
                .unwrap();
        assert_eq!(info["rules_version"], legatum_core::RULES_VERSION);
        let _ = std::fs::remove_dir_all(dir);
    }
}

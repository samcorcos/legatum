//! Test the public CLI with fixed, committed tapes, including merge and failure behavior.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "legatum-replay-export-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn export_file_directory_merge_and_reject_corrupt_tape() {
    let temp = TempDir::new();
    let out = temp.0.join("replays");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let tape = fixtures.join("game-00002-seed-18446744073709551615.json");
    let export = |input: &std::path::Path, label: &str| {
        Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
            .args([
                "--replay-export",
                input.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
                "--label",
                label,
            ])
            .output()
            .unwrap()
    };
    let single = export(&tape, "CLI sample");
    assert!(
        single.status.success(),
        "{}",
        String::from_utf8_lossy(&single.stderr)
    );
    let file = out.join("CLI-sample-game-00002-seed-18446744073709551615.json");
    let bytes = fs::read(&file).unwrap();
    let replay: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(replay["header"]["seed"], "18446744073709551615");
    assert_eq!(replay["header"]["turns"], 40);
    assert_eq!(replay["frames"].as_array().unwrap().len(), 40);
    let directory = export(&fixtures, "archive");
    assert!(
        directory.status.success(),
        "{}",
        String::from_utf8_lossy(&directory.stderr)
    );
    assert!(export(&tape, "CLI sample").status.success());
    assert_eq!(fs::read(&file).unwrap(), bytes);
    let index_bytes = fs::read(out.join("index.json")).unwrap();
    let index: Vec<serde_json::Value> = serde_json::from_slice(&index_bytes).unwrap();
    assert_eq!(
        index.len(),
        10,
        "nine fixture tapes plus the single export; preserve labels, replace same filename"
    );
    let corrupt = temp.0.join("corrupt.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tape).unwrap()).unwrap();
    value["turns"][0]["hash"] = "0".into();
    fs::write(&corrupt, value.to_string()).unwrap();
    let failed = export(&corrupt, "CLI sample");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("state hash differs after turn 1"));
    assert_eq!(fs::read(&file).unwrap(), bytes);
    assert_eq!(fs::read(out.join("index.json")).unwrap(), index_bytes);
}

#[test]
fn mirrored_matchups_preserve_both_replays_and_tape_import_identities() {
    let temp = TempDir::new();
    let out = temp.0.join("replays");
    let tapes = temp.0.join("tapes");
    let generated = Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
        .args([
            "--strategy-matchups",
            "--games",
            "20",
            "--seed",
            "989001",
            "--turn-limit",
            "2",
            "--threads",
            "2",
            "--export-replays",
            "20",
            "--label",
            "mirrored",
            "--out-dir",
            temp.0.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--tape-out",
            tapes.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let index_path = out.join("index.json");
    let bytes = fs::read(&index_path).unwrap();
    let index: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(index.len(), 20);
    for i in 0..20 {
        let file = format!("mirrored-game-{i:05}-seed-{}.json", 989001 + i / 2);
        let replay: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join(&file)).unwrap()).unwrap();
        assert_eq!(replay["header"]["game_index"], i);
        assert_eq!(replay["header"]["seed"], (989001 + i / 2).to_string());
        assert!(
            index
                .iter()
                .any(|e| e["file"] == file && e["game_index"] == i)
        );
    }
    let import = |input: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_legatum-sim"))
            .args([
                "--replay-export",
                input.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
                "--label",
                "mirrored",
            ])
            .output()
            .unwrap()
    };
    let imported = import(&tapes);
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let imported_bytes = fs::read(&index_path).unwrap();
    let imported_index: Vec<serde_json::Value> = serde_json::from_slice(&imported_bytes).unwrap();
    assert_eq!(imported_index.len(), 20);
    assert!(
        import(&tapes.join("game-00001-seed-989001.json"))
            .status
            .success()
    );
    assert_eq!(fs::read(&index_path).unwrap(), imported_bytes);
    // A true identity collision is still rejected before any output is modified.
    fs::copy(
        tapes.join("game-00001-seed-989001.json"),
        tapes.join("game-1-seed-989001.json"),
    )
    .unwrap();
    let failed = import(&tapes);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("duplicate replay identity"));
    assert_eq!(fs::read(&index_path).unwrap(), imported_bytes);
}

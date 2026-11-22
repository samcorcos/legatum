use legatum_archive::{Archive, Recording};
use legatum_core::GameSetup;
use legatum_wasm::session::{BrainKind, Session};
use std::fs;

fn game(turns: u32) -> Session {
    let mut setup = GameSetup::all_ai(u64::MAX - 2);
    setup.turn_limit = Some(turns);
    Session::new(&setup, |_| BrainKind::Scripted)
}

fn finish(session: &mut Session) {
    while !session.state.finished {
        let orders = session.ai_orders();
        session.step(&orders);
    }
}

#[test]
fn roundtrip_replay_telemetry_index_and_retry() {
    let root = tempfile::tempdir().unwrap();
    let archive = Archive::open(root.path().into()).unwrap();
    let mut session = game(3);
    let checkpoint = archive.write(session.history.as_ref().unwrap()).unwrap();
    assert!(checkpoint.exists());
    finish(&mut session);
    let history = session.history.as_ref().unwrap();
    let path = archive.write(history).unwrap();
    assert!(!checkpoint.exists());
    let read = legatum_archive::read(&path).unwrap();
    assert_eq!(&read, history);
    assert_eq!(read.replay().unwrap().hash(), session.state.hash());
    assert_eq!(read.telemetry.len(), 4);
    assert_eq!(
        read.telemetry.last().unwrap().players[0].vp,
        read.metadata.seats[0].final_vp
    );
    assert_eq!(
        legatum_wasm::tape::replay_checked(&read.tape)
            .unwrap()
            .first_mismatch(&read.tape),
        None
    );
    // Fresh writer simulates restart between file rename and index retry.
    let archive = Archive::open(root.path().into()).unwrap();
    archive.write(history).unwrap();
    let index = fs::read_to_string(root.path().join("index.jsonl")).unwrap();
    assert_eq!(index.lines().count(), 1);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&index).unwrap(),
        serde_json::to_value(&read.metadata).unwrap()
    );
    assert!(!index.contains("orders"));
    let body = serde_json::to_string(&read).unwrap();
    assert!(!body.contains("token") && !body.contains("credential"));
}

#[test]
fn zstd_truncation_and_bit_corruption_are_errors() {
    let root = tempfile::tempdir().unwrap();
    let archive = Archive::open(root.path().into()).unwrap();
    let path = archive.write(game(1).history.as_ref().unwrap()).unwrap();
    let original = fs::read(&path).unwrap();
    let corrupt = root.path().join("bad.zst");
    for bytes in [
        &original[..original.len() - 1],
        &original[..original.len() / 2],
        b"not zstd",
    ] {
        fs::write(&corrupt, bytes).unwrap();
        assert!(legatum_archive::read(&corrupt).is_err());
    }
    let mut flipped = original;
    let position = flipped.len() - 1;
    flipped[position] ^= 1; // The required frame checksum catches otherwise-decodable corruption.
    fs::write(&corrupt, flipped).unwrap();
    assert!(legatum_archive::read(&corrupt).is_err());
}

#[test]
fn simultaneous_writers_repair_torn_index_and_deduplicate() {
    let root = tempfile::tempdir().unwrap();
    let mut session = game(1);
    finish(&mut session);
    let history = session.history.as_ref().unwrap();
    let archive = Archive::open(root.path().into()).unwrap();
    archive.write(history).unwrap();
    // Simulate a file committed without its index line, followed by a partial append.
    fs::write(root.path().join("index.jsonl"), b"{\"game_id\":").unwrap();
    std::thread::scope(|s| {
        for _ in 0..4 {
            let root = root.path();
            s.spawn(move || Archive::open(root.into()).unwrap().write(history).unwrap());
        }
    });
    assert_eq!(
        fs::read_to_string(root.path().join("index.jsonl"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn rejects_repo_symlink_escape_bad_ids_and_changed_rules() {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    fs::create_dir_all(repo.join(".git")).unwrap();
    // Any git repository is allowed; only a Legatum source checkout is refused.
    assert!(Archive::open(repo.join("ok")).is_ok());
    fs::create_dir_all(repo.join("crates/archive/src")).unwrap();
    fs::write(repo.join("crates/archive/src/lib.rs"), "").unwrap();
    assert!(Archive::open(repo.join("archive")).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&repo, root.path().join("alias")).unwrap();
        assert!(Archive::open(root.path().join("alias/data")).is_err());
    }
    let mut history = game(1).history.unwrap();
    history.metadata.rules_fingerprint = "old-rules".into();
    assert!(history.replay().unwrap_err().contains("incompatible"));
    history.metadata.game_id = "../../escape".into();
    assert!(Archive::relative_path(&history).is_err());
}

#[test]
fn resumes_tape_and_reports_first_divergence() {
    let mut session = game(3);
    session.step(&Default::default());
    let mut session =
        Session::from_save_json(&session.save_json(), |_| BrainKind::Scripted).unwrap();
    session.step(&Default::default());
    session.step(&Default::default());
    let mut history = session.history.unwrap();
    assert_eq!(history.replay().unwrap().hash(), session.state.hash());
    history.tape.turns[1].hash = "1".into();
    assert!(
        history
            .replay()
            .unwrap_err()
            .contains("first divergent turn: 2")
    );
}

#[test]
fn legacy_save_is_explicitly_partial_and_still_replays() {
    let mut session = game(3);
    session.step(&Default::default());
    let old_save = legatum_wasm::json::state_to_json(&session.state);
    let mut resumed = Session::from_save_json(&old_save, |_| BrainKind::Scripted).unwrap();
    finish(&mut resumed);
    let history: Recording = resumed.history.unwrap();
    assert!(!history.metadata.complete_history);
    assert_eq!(history.metadata.first_turn, 2);
    assert_eq!(history.replay().unwrap().hash(), resumed.state.hash());
}

/// Index lines from games played under since-renamed factions must not block new commits
/// (designer, 2026-11-01: winning a game showed "invalid index line: unknown variant `Tollkeepers`").
#[test]
fn old_index_lines_with_renamed_factions_do_not_block_a_commit() {
    let root = tempfile::tempdir().unwrap();
    let old = r#"{"schema_version":1,"game_id":"old-1","created_at":"2026-09-27T21:30:54Z","finished_at":null,"source":"lan","seats":[{"player":0,"name":"Player 1","faction":"Tollkeepers","team":0,"difficulty":"Normal","human":true,"final_vp":0,"titles":[]}]}"#;
    fs::write(root.path().join("index.jsonl"), format!("{old}\n")).unwrap();
    let archive = Archive::open(root.path().into()).unwrap();
    let mut session = game(3);
    finish(&mut session);
    archive.write(session.history.as_ref().unwrap()).unwrap();
    let index = fs::read_to_string(root.path().join("index.jsonl")).unwrap();
    assert_eq!(index.lines().count(), 2);
    assert!(index.starts_with(old));
}

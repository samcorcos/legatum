//! Every shipped scenario (repo `scenarios/*.json`, and the war scenarios in `scenarios/war/`) is a legal, playable position: it parses, its
//! state matches its hash, the rules accept the position and the objective, the seat is the
//! only human, and the AI plays it deterministically from the same seed.

use legatum_core::scenario::Outcome;
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session};
use std::path::PathBuf;

fn scenarios() -> Vec<(PathBuf, ScenarioFile)> {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scenarios"));
    let mut files: Vec<_> = [dir.clone(), dir.join("war")]
        .iter()
        .flat_map(|d| std::fs::read_dir(d).expect("scenarios directory"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 11,
        "the four skateboard and seven war scenarios ship"
    );
    files
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap();
            let file =
                ScenarioFile::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            (p, file)
        })
        .collect()
}

/// Play `turns` turns with every empire on the scripted AI; return the final hash and status.
fn play(file: &ScenarioFile, turns: u32) -> (u64, Outcome) {
    let start = file.prepare(false).unwrap();
    let active = ActiveScenario::new(file.clone(), start.clone());
    let mut session = Session::from_state(start, |_| BrainKind::Scripted);
    session.history = None;
    for _ in 0..turns {
        let orders = session.ai_orders();
        session.step(&orders);
    }
    (session.state.hash(), active.status(&session.state).outcome)
}

#[test]
fn every_scenario_is_legal_and_names_its_seat() {
    for (path, file) in scenarios() {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        assert_eq!(
            file.id,
            name,
            "{}: id must match the file name",
            path.display()
        );
        let state = file.prepare(true).unwrap_or_else(|e| panic!("{name}: {e}"));
        let humans: Vec<_> = state
            .players
            .iter()
            .filter(|p| p.is_human)
            .map(|p| p.id)
            .collect();
        assert_eq!(humans, vec![file.seat], "{name}");
        assert_eq!(state.player(file.seat).faction, file.faction, "{name}");
        assert!(state.player(file.seat).alive, "{name}");
        let status = ActiveScenario::new(file.clone(), state.clone()).status(&state);
        assert_eq!(
            status.outcome,
            Outcome::Playing,
            "{name}: {}",
            status.reason
        );
        assert_eq!(status.turns_left, file.objective.turns, "{name}");
        assert!(!file.objective.summary.contains("TODO"), "{name}");
        assert!(!file.briefing.contains("TODO"), "{name}");
        assert!(!file.predictions.good_play.contains("TODO"), "{name}");
    }
}

#[test]
fn scenario_play_is_deterministic() {
    for (_, file) in scenarios() {
        assert_eq!(play(&file, 3), play(&file, 3), "{}", file.id);
    }
}

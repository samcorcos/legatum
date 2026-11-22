//! Deterministic, observational AI diagnostics. Never feeds hidden information into a brain.
mod detector;
mod report;
pub mod scenario;

pub use detector::Config;
use legatum_core::{GameState, new_game, resolve_turn};
use legatum_wasm::{history::Recording, tape::Tape};
pub use report::{Aggregate, Behavior, Incident, Report, write_report};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Archive replay uses exactly the same compatibility and hash gates as --replay.
/// No report is returned (or written) for a partial/divergent replay.
pub fn analyze_recording(recording: &Recording, config: Config) -> Result<Report, String> {
    recording.validate()?;
    recording.compatible()?;
    let state = match &recording.initial_state {
        Some(json) => legatum_wasm::json::parse_state(json)?,
        None => new_game(&recording.tape.setup),
    };
    let mut report = analyze(&recording.tape, state, &recording.metadata.game_id, config)?;
    report
        .engine_commit
        .clone_from(&recording.metadata.engine_commit);
    report.complete_history = recording.metadata.complete_history;
    Ok(report)
}

/// For freshly generated batch tapes only. Historical files must use the archive entrypoint,
/// which additionally checks the rules fingerprint and supports legacy-save anchors.
pub fn analyze_tape(tape: &Tape, game_id: &str, config: Config) -> Result<Report, String> {
    analyze(tape, new_game(&tape.setup), game_id, config)
}

fn analyze(
    tape: &Tape,
    mut state: GameState,
    game_id: &str,
    config: Config,
) -> Result<Report, String> {
    if config.settler_turns == 0 {
        return Err("settler threshold must be positive".into());
    }
    if state.hash() != tape.initial_hash {
        return Err("first divergent turn: 0 (initial state)".into());
    }
    let mut detector = detector::Detector::new(game_id, &state, config);
    for t in &tape.turns {
        if state.turn != t.turn || state.finished {
            return Err(format!("invalid replay turn {}", t.turn));
        }
        let (next, events) = catch_unwind(AssertUnwindSafe(|| resolve_turn(&state, &t.orders)))
            .map_err(|p| {
                format!(
                    "replay turn {} panicked: {}",
                    t.turn,
                    legatum_wasm::panic_message(p.as_ref())
                )
            })?;
        if next.hash().to_string() != t.hash {
            return Err(format!("first divergent turn: {}", t.turn));
        }
        detector.observe(&state, &t.orders, &next, &events);
        state = next;
    }
    Ok(detector.finish(&state))
}

//! Export hash-verified recorded positions on the tape's original rules for a current-engine audit.
//! audit_states TAPE OUT.jsonl; run on an engine whose replay reproduces every recorded hash.
use legatum_core::resolve_turn;
use legatum_wasm::tape::Tape;
use std::io::Write;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let mut state = tape.initial_state();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    for turn in &tape.turns {
        assert_eq!(state.turn, turn.turn);
        writeln!(out, "{}", serde_json::to_string(&state).unwrap()).unwrap();
        state = resolve_turn(&state, &turn.orders).0;
        assert_eq!(
            state.hash().to_string(),
            turn.hash,
            "replay drift at T{}",
            turn.turn
        );
    }
    writeln!(out, "{}", serde_json::to_string(&state).unwrap()).unwrap();
    println!("{} turns, 0 hash mismatches", tape.turns.len());
}

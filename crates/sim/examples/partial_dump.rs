//! Replay a web recording (the uploaded record JSON, decompressed), including a partial one that
//! starts from `initial_state`, and write every turn's state and events.
//! partial_dump RECORD.json OUTDIR
use legatum_core::resolve_turn;
use legatum_wasm::tape::Tape;
use std::io::Write;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let rec: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    let tape = Tape::from_json(&rec["tape"].to_string()).unwrap();
    let mut state = match rec.get("initial_state") {
        Some(serde_json::Value::String(s)) => {
            legatum_wasm::json::parse_state_for_rehash(s).unwrap()
        }
        Some(v) if v.is_object() => legatum_wasm::json::state_from_value(v.clone(), false).unwrap(),
        _ => tape.initial_state(),
    };
    let dir = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(dir).unwrap();
    let mut snaps =
        std::io::BufWriter::new(std::fs::File::create(dir.join("snaps.jsonl")).unwrap());
    let mut evs = std::io::BufWriter::new(std::fs::File::create(dir.join("events.jsonl")).unwrap());
    let mut bad = 0;
    for turn in &tape.turns {
        assert_eq!(state.turn, turn.turn);
        writeln!(snaps, "{}", serde_json::to_string(&state).unwrap()).unwrap();
        let (next, events) = resolve_turn(&state, &turn.orders);
        for e in &events {
            writeln!(
                evs,
                "{{\"turn\":{},\"e\":{}}}",
                turn.turn,
                serde_json::to_string(e).unwrap()
            )
            .unwrap();
        }
        state = next;
        if state.hash().to_string() != turn.hash {
            bad += 1;
        }
    }
    writeln!(snaps, "{}", serde_json::to_string(&state).unwrap()).unwrap();
    println!("{} turns, {} hash mismatches", tape.turns.len(), bad);
}

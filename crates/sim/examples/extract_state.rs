//! Replay a tape's recorded orders to a turn and write a trimmed regression fixture.
//! extract_state <tape.json> <turn> <state-out.json> [<orders-out.json> <turns>]
//!
//! Writes the complete GameState before <turn> resolves (JSON; compress it with
//! `gzip -9n`), and optionally the
//! recorded orders for the next <turns> turns as a JSON array of {turn, orders}.
//! Prints the fnv1a hash of the written state JSON and of each later resolved state.
use legatum_core::resolve_turn;
use legatum_wasm::tape::Tape;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let turn: u32 = a[2].parse()?;
    let mut state = tape.initial_state();
    for t in tape.turns.iter().filter(|t| t.turn < turn) {
        state = resolve_turn(&state, &t.orders).0;
    }
    assert_eq!(state.turn, turn, "tape does not reach turn {turn}");
    let json = serde_json::to_string(&state)?;
    println!(
        "state before turn {turn}: fnv1a {}",
        legatum_core::state::fnv1a(json.as_bytes())
    );
    std::fs::File::create(&a[3])?.write_all(json.as_bytes())?;
    if let (Some(out), Some(n)) = (a.get(4), a.get(5)) {
        let n: u32 = n.parse()?;
        let slice: Vec<_> = tape
            .turns
            .iter()
            .filter(|t| t.turn >= turn && t.turn < turn + n)
            .map(|t| serde_json::json!({"turn": t.turn, "orders": t.orders}))
            .collect();
        for t in tape
            .turns
            .iter()
            .filter(|t| t.turn >= turn && t.turn < turn + n)
        {
            state = resolve_turn(&state, &t.orders).0;
            println!("after turn {}: state hash {}", t.turn, state.hash());
        }
        std::fs::write(out, serde_json::to_string(&slice)?)?;
    }
    Ok(())
}

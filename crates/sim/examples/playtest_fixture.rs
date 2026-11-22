//! Extract a small observed state and AI memory from a recorded playtest tape.
//! playtest_fixture <tape.json> <turn> <out.json>
use legatum_wasm::session::{BrainKind, Session};
use legatum_wasm::tape::Tape;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let archive: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let tape = Tape::from_json(&archive.get("tape").unwrap_or(&archive).to_string())?;
    let turn: u32 = args[2].parse()?;
    let mut session = Session::new(&tape.setup, |_| BrainKind::Scripted);
    for recorded in &tape.turns {
        if recorded.turn == turn {
            let memories: Vec<_> = session
                .state
                .players
                .iter()
                .filter(|p| !p.is_human)
                .map(|p| (p.id, session.brain(p.id).and_then(|b| b.opponent_memory())))
                .collect();
            std::fs::write(
                &args[3],
                serde_json::to_vec(&json!({
                    "state": session.state,
                    "memories": memories,
                    "future_orders": tape.turns.iter().filter(|t| t.turn >= turn).take(4)
                        .map(|t| &t.orders).collect::<Vec<_>>(),
                }))?,
            )?;
            return Ok(());
        }
        let _ = session.ai_orders();
        session.step(&recorded.orders);
        if session.state.hash().to_string() != recorded.hash {
            return Err(format!("tape differs at turn {}", recorded.turn).into());
        }
    }
    Err(format!("turn {turn} is not in tape").into())
}

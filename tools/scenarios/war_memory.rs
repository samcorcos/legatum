//! War-scenario AI memory (claude/war-fixtures): replay a save with the engine it was recorded on,
//! letting every AI seat's brain plan every turn from its own view (as the app did) while the
//! recorded orders advance the game, and dump each AI seat's saved memory (`OpponentMemory`: the
//! assessments, adaptation, strategy switches, threat log and campaign memory, which is all
//! `ScriptedBrain` carries between turns) at the start of the chosen turns.
//!
//! Checks: every turn hash matches the recording; how many recorded AI order lists the replanned
//! brains reproduce; and at the end whether the replayed memory equals the save's own
//! `ai_opponent_memory` (the strongest proof the replay is faithful).
//!
//! war_memory <save.legatum> <out-dir> <turn,turn,...>  ->  <out-dir>/memory-T<turn>.json
use legatum_core::PlayerId;
use legatum_wasm::session::{BrainKind, Session};
use std::collections::BTreeMap;

fn memories(session: &Session) -> BTreeMap<PlayerId, legatum_ai::OpponentMemory> {
    session
        .state
        .players
        .iter()
        .filter_map(|p| {
            session
                .brain(p.id)
                .and_then(|b| b.opponent_memory())
                .map(|m| (p.id, m))
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).expect("read save");
    let out = std::path::PathBuf::from(&a[2]);
    std::fs::create_dir_all(&out).unwrap();
    let dumps: Vec<u32> = a[3]
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();
    let outer: serde_json::Value = serde_json::from_str(&text).unwrap();
    let value: serde_json::Value = serde_json::from_str(outer["game"].as_str().unwrap()).unwrap();
    let history: legatum_wasm::history::Recording =
        serde_json::from_value(value["archive"].clone()).expect("archive");
    let state = match &history.initial_state {
        Some(json) => legatum_wasm::json::parse_state(json).unwrap(),
        None => history.tape.initial_state(),
    };
    let mut session = Session::from_state(state, |_| BrainKind::Scripted);
    let (mut hash_bad, mut orders_same, mut orders_diff) = (0, 0, 0);
    let dump = |session: &Session| {
        let m = memories(session);
        std::fs::write(
            out.join(format!("memory-T{}.json", session.state.turn)),
            serde_json::to_string(&m).unwrap(),
        )
        .unwrap();
    };
    for t in &history.tape.turns {
        if dumps.contains(&session.state.turn) {
            dump(&session);
        }
        let planned = session.ai_orders();
        for (p, orders) in &planned {
            if t.orders.get(p) == Some(orders) {
                orders_same += 1;
            } else {
                orders_diff += 1;
            }
        }
        session.step(&t.orders);
        if session.state.hash().to_string() != t.hash {
            hash_bad += 1;
        }
    }
    if dumps.contains(&session.state.turn) {
        dump(&session);
    }
    let replayed = serde_json::to_value(memories(&session)).unwrap();
    let saved = value.get("ai_opponent_memory").cloned();
    println!(
        "replayed {} turns, {} hash mismatches, AI order lists reproduced {}/{}, final memory {} the save's",
        history.tape.turns.len(),
        hash_bad,
        orders_same,
        orders_same + orders_diff,
        match saved {
            Some(s) if s == replayed => "EQUALS",
            Some(_) => "DIFFERS FROM",
            None => "has no counterpart in",
        }
    );
}

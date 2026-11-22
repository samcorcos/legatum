//! Recompute deterministic fixture hashes after an intentional rules change.
//! Usage: cargo run --release -p legatum-sim --example rehash_fixtures -- path.json ...
use legatum_core::resolve_turn;
use legatum_wasm::tape::{TAPE_VERSION, Tape};
use serde_json::Value;
use std::{env, fs};

fn is_retired_item(value: &Value) -> bool {
    value.get("kind").and_then(Value::as_str) == Some("Building")
        && value
            .get("building")
            .and_then(Value::as_str)
            .is_some_and(|name| matches!(name, "AidStation" | "Reactor"))
}

/// Fixed-order tapes can outlive a building. Remove production orders for retired kinds before
/// rehashing so the fixture keeps replaying under the current rules.
fn drop_retired_orders(raw: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(raw) else {
        return raw.to_owned();
    };
    let Some(turns) = value.get_mut("turns").and_then(Value::as_array_mut) else {
        return raw.to_owned();
    };
    for turn in turns {
        let Some(players) = turn.get_mut("orders").and_then(Value::as_object_mut) else {
            continue;
        };
        for orders in players.values_mut().filter_map(Value::as_array_mut) {
            orders.retain(|order| !order.get("item").is_some_and(is_retired_item));
            for order in orders {
                if let Some(queue) = order.get_mut("queue").and_then(Value::as_array_mut) {
                    queue.retain(|item| !is_retired_item(item));
                }
            }
        }
    }
    value.to_string()
}

fn main() {
    for path in env::args().skip(1) {
        let raw = fs::read_to_string(&path).expect("read fixture");
        let mut tape = Tape::from_json(&drop_retired_orders(&raw)).expect("parse fixture");
        tape.version = TAPE_VERSION;
        let mut state = tape.initial_state();
        let old_initial = tape.initial_hash;
        tape.initial_hash = state.hash();
        let mut first_changed = None;
        for (index, turn) in tape.turns.iter_mut().enumerate() {
            assert_eq!(state.turn, turn.turn, "turn mismatch in {path}");
            state = resolve_turn(&state, &turn.orders).0;
            let hash = state.hash().to_string();
            if hash != turn.hash && first_changed.is_none() {
                first_changed = Some(index + 1);
            }
            turn.hash = hash;
        }
        fs::write(&path, format!("{}\n", tape.to_json())).expect("write fixture");
        println!(
            "{path}: initial changed={}, first turn hash changed={first_changed:?}, turns={}",
            old_initial != tape.initial_hash,
            tape.turns.len()
        );
    }
}

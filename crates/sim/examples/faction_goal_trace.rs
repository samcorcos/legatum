//! Replay recorded orders, checking every hash, and inspect a faction's public goal budget.
//! Usage: cargo run --release -p legatum-sim --example faction_goal_trace -- <tape.json>

use legatum_core::{Faction, resolve_turn, view::player_view};
use legatum_wasm::tape::Tape;
use serde_json::json;

fn main() {
    let path = std::env::args().nth(1).expect("tape path");
    let tape =
        Tape::from_json(&std::fs::read_to_string(path).expect("read tape")).expect("decode tape");
    let mut state = tape.initial_state();
    assert_eq!(state.hash(), tape.initial_hash);
    let player = state
        .players
        .iter()
        .find(|p| p.faction == Faction::Mongols)
        .expect("Mongols seat")
        .id;
    for turn in &tape.turns {
        let view = player_view(&state, player);
        let cities: Vec<_> = view
            .cities
            .iter()
            .filter(|c| c.owner == player && c.detail.is_some())
            .collect();
        let units: Vec<_> = view.units.iter().filter(|u| u.owner == player).collect();
        let (next, events) = resolve_turn(&state, &turn.orders);
        assert_eq!(next.hash().to_string(), turn.hash);
        println!(
            "{}",
            json!({
                "turn":turn.turn, "cities":cities, "units":units, "economy":view.economy,
                "goal_done":view.players.iter().find(|p|p.id==player).unwrap().faction_goal_done,
                "orders":turn.orders.get(&player), "events":events,
            })
        );
        state = next;
    }
}

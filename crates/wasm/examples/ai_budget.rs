//! The live AI speed budget: an 8-empire Standard game with every seat AI, timed per turn.
//! `Game::end_turn_json("")` is the whole lobby turn (all seats' planning, resolution, history),
//! and with the `seat-timing` feature each seat's own planning time comes from
//! `Game::seat_timings_json`. `scripts/perf-wasm-budget.mjs` is the same loop on the WASM build.
//!   cargo build --release -p legatum-wasm --features seat-timing --example ai_budget
//!   RAYON_NUM_THREADS=1 target/release/examples/ai_budget <seed> <turns> <hard|search> > out.csv
//! `search` makes every seat Hard+Search (the lobby's option). One thread: pass
//! `RAYON_NUM_THREADS=1`, since the WASM build is single-threaded and native plans seats in parallel.
//! CSV: turn,end_turn_ms,view_ms,hash,seat0_ms..seat7_ms,searched_seats (bitmask).
use legatum_wasm::Game;
use std::time::Instant;

const FACTIONS: [&str; 8] = [
    "Switzerland",
    "India",
    "Byzantium",
    "Germany",
    "Celts",
    "Mongols",
    "Sparta",
    "Athens",
];

fn setup_json(seed: u64, search: bool) -> String {
    let seats: Vec<_> = (0..8)
        .map(|i| {
            serde_json::json!({
                "name": format!("AI {i}"),
                "faction": FACTIONS[(i + seed as usize) % 8],
                "team": i,
                "is_human": false,
                "difficulty": "Hard",
                "search": search,
            })
        })
        .collect();
    serde_json::json!({
        "seed": seed.to_string(), "width": 48, "height": 32, "turn_limit": null,
        "barbarians": true, "players": seats,
    })
    .to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(7_000_001, |a| a.parse().expect("seed"));
    let turns: u32 = args.get(2).map_or(260, |a| a.parse().expect("turns"));
    let search = args.get(3).is_some_and(|a| a == "search");
    let mut game = Game::from_setup_json(&setup_json(seed, search)).expect("setup");
    println!(
        "turn,end_turn_ms,view_ms,hash,{},searched_seats",
        (0..8)
            .map(|i| format!("seat{i}_ms"))
            .collect::<Vec<_>>()
            .join(",")
    );
    for t in 1..=turns {
        let start = Instant::now();
        if game.end_turn_json("").is_err() {
            break;
        }
        let end_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        game.view_json(0).expect("view");
        let view_ms = start.elapsed().as_secs_f64() * 1000.0;
        let rows: Vec<(u8, f64, bool)> =
            serde_json::from_str(&game.seat_timings_json()).expect("seat timings");
        let mut ms = vec![String::new(); 8];
        let mut mask = 0u32;
        for (seat, m, searched) in rows {
            ms[seat as usize] = format!("{m:.2}");
            mask |= (searched as u32) << seat;
        }
        println!(
            "{t},{end_ms:.2},{view_ms:.2},{},{},{mask}",
            game.hash(),
            ms.join(",")
        );
    }
}

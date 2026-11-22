//! Wall time of whole `Game::end_turn_json` turns, the call the app makes, natively: the number
//! to compare with the WASM build's `end_turn` (same setups as the Node timing script). Seat 0
//! is plain Hard; the others are Hard or Hard + Search.
//!   app_turn_time <players> <turns> <hard|search> [seed]
//! Native sessions plan AI seats in parallel; build with `-p legatum-sim` alongside to also
//! parallelise rollouts (cargo feature unification turns on `parallel-search`).
use legatum_wasm::Game;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let players: usize = args[1].parse().expect("players");
    let turns: u32 = args[2].parse().expect("turns");
    let search = args[3] == "search";
    let seed: u64 = args.get(4).map_or(12345, |a| a.parse().expect("seed"));
    let factions = [
        "Switzerland",
        "India",
        "Byzantium",
        "Germany",
        "Celts",
        "Mongols",
        "Sparta",
        "Athens",
    ];
    let seats: Vec<_> = (0..players)
        .map(|i| {
            serde_json::json!({
                "name": format!("AI {i}"),
                "faction": factions[(i + seed as usize) % 8],
                "team": i,
                "is_human": false,
                "difficulty": "Hard",
                "search": search && i > 0,
            })
        })
        .collect();
    let small = players <= 4;
    let setup = serde_json::json!({
        "seed": seed.to_string(),
        "width": if small { 40 } else { 48 },
        "height": if small { 26 } else { 32 },
        "turn_limit": null,
        "barbarians": true,
        "players": seats,
    });
    let mut game = Game::from_setup_json(&setup.to_string()).expect("setup");
    let mut ms = Vec::new();
    for _ in 0..turns {
        let start = Instant::now();
        if game.end_turn_json("").is_err() {
            break;
        }
        ms.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let total: f64 = ms.iter().sum();
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p) as usize];
    println!(
        "{players}p {}: {} turns, ms/turn mean {:.1} p50 {:.1} p95 {:.1} max {:.1}, total {:.1} s",
        args[3],
        ms.len(),
        total / ms.len() as f64,
        q(0.5),
        q(0.95),
        ms[ms.len() - 1],
        total / 1000.0
    );
}

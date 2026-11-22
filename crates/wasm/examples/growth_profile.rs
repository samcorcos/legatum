//! What grows as a game grows: an 8-empire Standard game with every seat AI, measuring every
//! `step` turns the costs the browser pays on the main thread around End Turn.
//!   cargo build --release -p legatum-wasm --example growth_profile
//!   RAYON_NUM_THREADS=1 target/release/examples/growth_profile <seed> <turns> [step] > out.csv
//! Seat 0 carries a typed name, as a player's seat does, so `public_archive` takes the path a real
//! upload takes. CSV columns (times in ms, sizes in bytes):
//! turn, units, cities, end_turn (whole lobby turn), view (one seat's fresh view), view_bytes,
//! save (the autosave's engine save), save_bytes, load (reading that save back), archive (the owner's
//! history), archive_bytes, public_archive (the upload's renamed copy), rss_kb (process memory).
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

fn setup_json(seed: u64) -> String {
    let seats: Vec<_> = (0..8)
        .map(|i| {
            serde_json::json!({
                "name": if i == 0 { "Dana".to_string() } else { format!("AI {}", i + 1) },
                "faction": FACTIONS[(i + seed as usize) % 8],
                "team": i,
                "is_human": false,
                "difficulty": "Hard",
                "search": false,
            })
        })
        .collect();
    serde_json::json!({
        "seed": seed.to_string(), "width": 48, "height": 32, "turn_limit": null,
        "barbarians": true, "players": seats,
    })
    .to_string()
}

fn ms<T>(run: impl FnOnce() -> T) -> (T, f64) {
    let start = Instant::now();
    let out = run();
    (out, start.elapsed().as_secs_f64() * 1000.0)
}

fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(7_000_001, |a| a.parse().expect("seed"));
    let turns: u32 = args.get(2).map_or(300, |a| a.parse().expect("turns"));
    let step: u32 = args.get(3).map_or(10, |a| a.parse().expect("step"));
    let mut game = Game::from_setup_json(&setup_json(seed)).expect("setup");
    println!(
        "turn,units,cities,end_turn,view,view_bytes,save,save_bytes,load,archive,archive_bytes,public_archive,rss_kb"
    );
    for t in 1..=turns {
        let (ended, end_ms) = ms(|| game.end_turn_json(""));
        if ended.is_err() {
            break;
        }
        if t % step != 0 {
            continue;
        }
        let (view, view_ms) = ms(|| game.view_json(0).expect("view"));
        let parsed: serde_json::Value = serde_json::from_str(&view).expect("view json");
        let count = |key: &str| parsed[key].as_array().map_or(0, Vec::len);
        let (save, save_ms) = ms(|| game.save_json());
        let (loaded, load_ms) = ms(|| Game::from_save_json(&save));
        assert!(loaded.is_ok(), "the save reads back");
        let (archive, archive_ms) = ms(|| game.archive().unwrap_or_else(|_| panic!("archive")));
        let (_, public_ms) = ms(|| {
            game.public_archive()
                .unwrap_or_else(|_| panic!("public archive"))
        });
        println!(
            "{t},{},{},{end_ms:.1},{view_ms:.1},{},{save_ms:.1},{},{load_ms:.1},{archive_ms:.1},{},{public_ms:.1},{}",
            count("units"),
            count("cities"),
            view.len(),
            save.len(),
            archive.len(),
            rss_kb()
        );
    }
}

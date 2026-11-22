//! Play a game through the app's own entry point (`Game`, the same code the WASM build runs) and
//! print the state hash after every turn, one per line. `scripts/wasm-search-check.mjs` plays the
//! same setup in the WASM build and compares the lists, so a Hard+Search game (setup
//! `search: true`) is checked to play identically native and WASM.
//!   search_hashes <setup.json> <turns>
use legatum_wasm::Game;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let setup = std::fs::read_to_string(&args[1]).expect("setup file");
    let turns: u32 = args[2].parse().expect("turns");
    let mut game = Game::from_setup_json(&setup).expect("setup");
    println!("{}", game.hash());
    for _ in 0..turns {
        if game.end_turn_json("").is_err() {
            break;
        }
        println!("{}", game.hash());
    }
}

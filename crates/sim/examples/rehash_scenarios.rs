//! Re-encode the game state embedded in scenario files after an intentional rules or state-format
//! change (a state field added or removed changes `GameState::hash`, which the scenario loader
//! checks). The state is loaded without the hash check and written back with its new hash;
//! baselines are left alone (re-record them with `legatum-sim scenario <file> --record`).
//! Usage: cargo run --release -p legatum-sim --example rehash_scenarios -- scenarios/*.json
use legatum_wasm::scenario::{ScenarioFile, decode_state_unchecked, encode_state};
use std::{env, fs};

fn main() {
    for path in env::args().skip(1) {
        let raw = fs::read_to_string(&path).expect("read scenario");
        let mut file = ScenarioFile::parse(&raw).expect("parse scenario");
        let old = file.state.hash.clone();
        let state = decode_state_unchecked(&file.state).expect("decode state");
        file.state = encode_state(&state);
        let mut out = serde_json::to_string_pretty(&file).expect("scenario JSON");
        out.push('\n');
        fs::write(&path, out).expect("write scenario");
        println!("{path}: hash {old} -> {}", file.state.hash);
    }
}

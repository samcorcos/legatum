//! Veteran levels by faction, from tapes.
//!   levels_audit <tapes-dir> [turn]
//! At `turn` (default: each game's end), per faction: seats, military units at promotion level
//! 2, 3 and 4 or higher (Sparta ladder via `actions::veteran_level`), and veterans retired.
use legatum_core::{GameState, resolve_turn, rules::actions::veteran_level};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).expect("tapes dir");
    let at: Option<u32> = std::env::args().nth(2).map(|s| s.parse().expect("turn"));
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    // faction -> [seats, lv2, lv3, lv4+, retired]
    let mut out: BTreeMap<String, [u64; 5]> = BTreeMap::new();
    for file in &files {
        let tape = Tape::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();
        let mut state = tape.initial_state();
        for t in &tape.turns {
            if at.is_some_and(|a| state.turn >= a) {
                break;
            }
            state = resolve_turn(&state, &t.orders).0;
        }
        tally(&state, &mut out);
    }
    for (f, r) in &out {
        let n = r[0].max(1) as f64;
        println!(
            "{f:14} seats {:5}  per seat: level 2 {:.2}  level 3 {:.2}  level 4+ {:.2}  retired {:.2}",
            r[0],
            r[1] as f64 / n,
            r[2] as f64 / n,
            r[3] as f64 / n,
            r[4] as f64 / n
        );
    }
}

fn tally(state: &GameState, out: &mut BTreeMap<String, [u64; 5]>) {
    for p in &state.players {
        let r = out.entry(format!("{:?}", p.faction)).or_default();
        r[0] += 1;
        r[4] += u64::from(p.veterans_retired);
        for u in state.units.values().filter(|u| u.owner == p.id) {
            if !legatum_core::data::unit_stats(u.kind).military {
                continue;
            }
            match veteran_level(state, u) {
                2 => r[1] += 1,
                3 => r[2] += 1,
                l if l >= 4 => r[3] += 1,
                _ => {}
            }
        }
    }
}

//! Trace a human's movement orders through a recorded tape (analysis only).
//!
//! tape_unit_trace <save.legatum> <first_turn> <last_turn> [player]
//!     For every turn in the range, prints the player's move orders (with each unit's tile,
//!     distance to the target and recorded route), then each ordered unit's tile, remaining
//!     route and distance to its target after resolution.
//!
//! Env: DUMP_AT=<turn> DUMP_TO=<file> writes the state before that turn's orders (run on the
//! code that recorded the tape). LOAD_FROM=<file> LOAD_AT=<turn> starts from such a dump and
//! skips earlier turns, with hashes unchecked, to replay the same orders on changed rules.
//! EVENTS=1 also prints every event of each turn in the range (Debug form).
use legatum_core::{GameState, Hex, Order, PlayerId, UnitId};
use serde_json::Value;
use std::cell::RefCell;

thread_local! { static WATCH: RefCell<Vec<(UnitId, Hex)>> = const { RefCell::new(Vec::new()) }; }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let game = outer["game"].as_str().expect("game JSON string");
    let v: Value = serde_json::from_str(game).unwrap();
    let rec: legatum_wasm::history::Recording =
        serde_json::from_value(v["archive"].clone()).expect("archive");
    let (first, last): (u32, u32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let me = PlayerId(a.get(4).map_or(0, |s| s.parse().unwrap()));
    let mut state = legatum_core::new_game(&rec.tape.setup);
    let dump_at: Option<u32> = std::env::var("DUMP_AT").ok().map(|v| v.parse().unwrap());
    let load_at: Option<u32> = std::env::var("LOAD_AT").ok().map(|v| v.parse().unwrap());
    if let Ok(path) = std::env::var("LOAD_FROM") {
        state = legatum_wasm::json::parse_state(&std::fs::read_to_string(path).unwrap()).unwrap();
    }
    for t in &rec.tape.turns {
        if load_at.is_some_and(|at| t.turn < at) {
            continue;
        }
        if dump_at == Some(t.turn) {
            let to = std::env::var("DUMP_TO").unwrap();
            std::fs::write(to, serde_json::to_string(&state).unwrap()).unwrap();
        }
        if t.turn >= first && t.turn <= last {
            report(&state, t, me);
        }
        let (next, events) = legatum_core::resolve_turn(&state, &t.orders);
        if std::env::var("EVENTS").is_ok() && t.turn >= first && t.turn <= last {
            for e in &events {
                println!("  EVENT {e:?}");
            }
        }
        state = next;
        if load_at.is_none() {
            assert_eq!(state.hash().to_string(), t.hash, "hash at turn {}", t.turn);
        }
        if t.turn >= first && t.turn <= last {
            after(&state, t.turn);
        }
    }
}

fn report(state: &GameState, t: &legatum_wasm::tape::TapeTurn, me: PlayerId) {
    println!("=== turn {}", t.turn);
    WATCH.with(|w| {
        let mut w = w.borrow_mut();
        for o in t.orders.get(&me).into_iter().flatten() {
            let (unit, to, route) = match o {
                Order::MoveUnit { unit, to } => (*unit, *to, Vec::new()),
                Order::MoveUnitPath { unit, to, path, .. } => (*unit, *to, path.clone()),
                _ => continue,
            };
            let u = &state.units[&unit];
            println!(
                "ORDER {:?} u{} at ({},{}) -> ({},{}) hex distance {} route {:?}",
                u.kind,
                unit.0,
                u.pos.q,
                u.pos.r,
                to.q,
                to.r,
                u.pos.distance(to),
                route.iter().map(|h| (h.q, h.r)).collect::<Vec<_>>()
            );
            w.push((unit, to));
        }
    });
}

fn after(state: &GameState, turn: u32) {
    WATCH.with(|w| {
        for (id, to) in w.borrow().iter() {
            match state.units.get(id) {
                Some(u) => println!(
                    "  after T{turn}: u{} at ({},{}) hex distance {} goto {:?} route left {}",
                    id.0,
                    u.pos.q,
                    u.pos.r,
                    u.pos.distance(*to),
                    u.goto.map(|g| (g.q, g.r)),
                    u.path.len()
                ),
                None => println!("  after T{turn}: u{} gone", id.0),
            }
        }
    });
}

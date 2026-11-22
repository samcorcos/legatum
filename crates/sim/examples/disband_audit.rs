//! Which military units the AIs disband, and where.
//!   disband_audit <tapes-dir> [skip-player]
//! Replays every tape in the directory and counts military disbands (by order or for unpaid
//! upkeep) by players other than `skip-player` (e.g. the benchmark proxy's seat when it is
//! fixed). A disband is "at the front" when the owner was at war and the unit stood within 6
//! hexes of one of its own cities that had an at-war enemy military unit within 3.
use legatum_core::{Event, GameState, PlayerId, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).expect("tapes dir");
    let skip: Option<u8> = std::env::args().nth(2).map(|s| s.parse().expect("player"));
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut counts = BTreeMap::<String, u64>::new();
    let mut games = 0u64;
    for file in &files {
        let tape = Tape::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();
        let mut state = tape.initial_state();
        games += 1;
        for t in &tape.turns {
            let (next, events) = resolve_turn(&state, &t.orders);
            for e in &events {
                let Event::UnitDisbanded {
                    unit,
                    owner,
                    reason,
                    ..
                } = e
                else {
                    continue;
                };
                if skip == Some(owner.0) || owner.is_barbarian() {
                    continue;
                }
                let Some(u) = state.units.get(unit) else {
                    continue;
                };
                if !military(&state, u.kind, *owner) {
                    continue;
                }
                let kind = if reason.contains("by order") {
                    "ordered"
                } else {
                    "upkeep"
                };
                let war = state
                    .players
                    .iter()
                    .any(|p| p.id != *owner && p.alive && state.at_war(*owner, p.id));
                let front = war
                    && state
                        .cities
                        .values()
                        .filter(|c| c.owner == *owner && c.pos.distance(u.pos) <= 6)
                        .any(|c| {
                            state.units.values().any(|e| {
                                e.owner != *owner
                                    && !e.owner.is_barbarian()
                                    && state.at_war(*owner, e.owner)
                                    && military(&state, e.kind, e.owner)
                                    && e.pos.distance(c.pos) <= 3
                            })
                        });
                let place = if front {
                    "at the front"
                } else if war {
                    "at war, elsewhere"
                } else {
                    "at peace"
                };
                *counts.entry(format!("{kind}, {place}")).or_default() += 1;
            }
            state = next;
        }
    }
    println!("games {games}");
    for (k, n) in &counts {
        println!(
            "  {k}: {n} ({:.2} per game)",
            *n as f64 / games.max(1) as f64
        );
    }
}

fn military(state: &GameState, kind: legatum_core::UnitKind, owner: PlayerId) -> bool {
    legatum_core::data::unit_stats_for(state.player(owner).faction, kind).military
        && kind != legatum_core::UnitKind::Scout
}

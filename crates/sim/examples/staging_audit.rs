//! Where an attacker's army stood when it declared war.
//!   staging_audit <tapes-dir>
//! For every war declaration in the tapes, counts the declarer's military units (not Scouts) on
//! the victim's land, and within 3 hexes of one of the victim's cities, on the turn the
//! declaration was ordered (the state before it resolved).
use legatum_core::{Event, PlayerId, resolve_turn};
use legatum_wasm::tape::Tape;

fn main() {
    let dir = std::env::args().nth(1).expect("tapes dir");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let (mut wars, mut on_land, mut near, mut total) = (0u32, 0u32, 0u32, 0u32);
    for file in &files {
        let tape = Tape::from_json(&std::fs::read_to_string(file).unwrap()).unwrap();
        let mut state = tape.initial_state();
        for t in &tape.turns {
            let (next, events) = resolve_turn(&state, &t.orders);
            for e in &events {
                let Event::WarDeclared { relation } = e else {
                    continue;
                };
                for declarer in &relation.declarers {
                    let victim = if relation.a == *declarer {
                        relation.b
                    } else {
                        relation.a
                    };
                    let team_of = |p: PlayerId| state.player(p).team;
                    wars += 1;
                    for u in state.units.values().filter(|u| {
                        !u.owner.is_barbarian()
                            && team_of(u.owner) == *declarer
                            && u.kind != legatum_core::UnitKind::Scout
                            && legatum_core::data::unit_stats(u.kind).military
                    }) {
                        total += 1;
                        let tile_owner = state.map.get(u.pos).and_then(|t| t.owner);
                        if tile_owner.is_some_and(|o| !o.is_barbarian() && team_of(o) == victim) {
                            on_land += 1;
                        }
                        if state.cities.values().any(|c| {
                            !c.owner.is_barbarian()
                                && team_of(c.owner) == victim
                                && c.pos.distance(u.pos) <= 3
                        }) {
                            near += 1;
                        }
                    }
                }
            }
            state = next;
        }
    }
    println!(
        "{wars} declarations; declarer military {total}: on the victim's land {on_land}, within 3 of a victim city {near}"
    );
}

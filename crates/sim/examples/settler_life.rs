//! Replay a tape and print each Settler's life (positions per turn); `stats` counts moving and idle Settler-turns, `lives` one line per Settler.
use legatum_core::{GameState, UnitKind, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let mut state: GameState = tape.initial_state();
    let mut life: BTreeMap<u32, Vec<(u32, String)>> = BTreeMap::new();
    let mut owner: BTreeMap<u32, u8> = BTreeMap::new();
    for turn in &tape.turns {
        for u in state.units.values().filter(|u| u.kind == UnitKind::Settler) {
            let near = state
                .cities_of(u.owner)
                .map(|c| c.pos.distance(u.pos))
                .min()
                .unwrap_or(99);
            life.entry(u.id.0).or_default().push((
                turn.turn,
                format!(
                    "({},{}) d{} goto{:?}",
                    u.pos.q,
                    u.pos.r,
                    near,
                    u.goto.map(|g| (g.q, g.r))
                ),
            ));
            owner.insert(u.id.0, u.owner.0);
        }
        let (next, _) = resolve_turn(&state, &turn.orders);
        state = next;
    }
    if a.get(2).is_some_and(|x| x == "lives") {
        for (id, v) in &life {
            let mut targets: Vec<&str> = Vec::new();
            for (_, s) in v {
                if let Some(g) = s.split("goto").nth(1)
                    && g != "None"
                    && targets.last() != Some(&g)
                {
                    targets.push(g);
                }
            }
            println!("{} {} {} {}", owner[id], v[0].0, v.len(), targets.len());
        }
        return Ok(());
    }
    if a.get(2).is_some_and(|x| x == "stats") {
        let (mut mv, mut idle_city, mut idle_out) = (0, 0, 0);
        for v in life.values() {
            for (t, s) in v {
                if *t < 10 || *t > 60 {
                    continue;
                }
                if s.contains("gotoSome") {
                    mv += 1
                } else if s.contains(" d0 ") {
                    idle_city += 1
                } else {
                    idle_out += 1
                }
            }
        }
        println!("{mv} {idle_city} {idle_out}");
        return Ok(());
    }
    for (id, v) in life {
        if v.len() >= 9 && v[0].0 <= 60 {
            println!(
                "settler {id} owner {} turns {}..{}",
                owner[&id],
                v[0].0,
                v.last().unwrap().0
            );
            for (t, s) in v {
                println!("   T{t} {s}");
            }
        }
    }
    Ok(())
}

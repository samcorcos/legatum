//! Hash-checked snapshots from an archive or plain tape. Optional arguments:
//! output directory for PlayerViews, and an additional turn to inspect.
use legatum_core::{player_view, resolve_turn};
use serde_json::json;
fn main() {
    let path = std::env::args().nth(1).expect("archive path");
    let selected = std::env::args().nth(3).map(|s| s.parse::<u32>().unwrap());
    let (tape, mut state) = if path.ends_with(".json") {
        let tape =
            legatum_wasm::tape::Tape::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let state = tape.initial_state();
        (tape, state)
    } else {
        let recording = legatum_archive::read(std::path::Path::new(&path)).unwrap();
        recording.compatible().unwrap();
        let state = match &recording.initial_state {
            Some(s) => legatum_wasm::json::parse_state(s).unwrap(),
            None => recording.tape.initial_state(),
        };
        (recording.tape, state)
    };
    assert_eq!(state.hash(), tape.initial_hash);
    let mut snapshots = Vec::new();
    for t in &tape.turns {
        if [200, 300, 400, 600, 1200].contains(&t.turn) || selected == Some(t.turn) {
            for p in &state.players {
                let v = player_view(&state, p.id);
                if (t.turn == 1200 || selected == Some(t.turn))
                    && let Some(directory) = std::env::args().nth(2)
                {
                    std::fs::create_dir_all(&directory).unwrap();
                    std::fs::write(
                        std::path::Path::new(&directory).join(format!("player-{}.json", p.id.0)),
                        serde_json::to_vec(&v).unwrap(),
                    )
                    .unwrap();
                }
                let cities: Vec<_> = v.cities.iter().filter(|c| c.owner == p.id).filter_map(|c| {
                    let d = c.detail.as_ref()?;
                    Some(json!({"id":c.id,"pos":c.pos,"pop":c.population,"build":d.producing,"turns":d.turns_to_complete,"yields":d.yields.total,"buildings":d.buildings}))
                }).collect();
                snapshots.push(json!({"turn":t.turn,"player":p.id,"faction":v.faction_of(p.id),"economy":v.economy,"players":v.players,"cities":cities,"own_units":v.units.iter().filter(|u|u.owner==p.id).collect::<Vec<_>>(),"orders":t.orders.get(&p.id)}));
            }
        }
        let (next, _) = resolve_turn(&state, &t.orders);
        assert_eq!(next.hash().to_string(), t.hash);
        state = next;
    }
    println!("{}", serde_json::to_string(&snapshots).unwrap());
}

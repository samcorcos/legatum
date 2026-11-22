//! War-scenario extraction (claude/war-scenarios): replay a user save with the engine it was
//! recorded on, verify every turn against the recorded hashes, dump full states at chosen turns
//! and a per-turn summary for the human benchmark.
//! war_extract <save.legatum> <out-dir> <turn,turn,...> [seat:first-last]
//! With the optional fourth argument it also writes `orders-<seat>.json`: that seat's recorded
//! orders for turns first..=last, for the scenario suite to replay (`war_script`).
use legatum_core::resolve_turn;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).expect("read save");
    let out = std::path::PathBuf::from(&a[2]);
    std::fs::create_dir_all(&out).unwrap();
    let dumps: Vec<u32> = a[3].split(',').filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect();
    let outer: serde_json::Value = serde_json::from_str(&text).unwrap();
    let game = outer["game"].as_str().unwrap();
    let value: serde_json::Value = serde_json::from_str(game).unwrap();
    let history: legatum_wasm::history::Recording =
        serde_json::from_value(value["archive"].clone()).expect("archive");
    let mut state = match &history.initial_state {
        Some(json) => legatum_wasm::json::parse_state(json).unwrap(),
        None => history.tape.initial_state(),
    };
    // seat:first-last
    let scripted: Option<(u8, u32, u32)> = a.get(4).filter(|s| !s.is_empty()).map(|s| {
        let (seat, range) = s.split_once(':').expect("seat:first-last");
        let (first, last) = range.split_once('-').expect("seat:first-last");
        (seat.parse().unwrap(), first.parse().unwrap(), last.parse().unwrap())
    });
    let mut script = std::collections::BTreeMap::<String, serde_json::Value>::new();
    let mut summary = std::fs::File::create(out.join("summary.jsonl")).unwrap();
    let mut mismatches = 0;
    for t in &history.tape.turns {
        if dumps.contains(&state.turn) {
            std::fs::write(out.join(format!("T{}.json", state.turn)), legatum_wasm::json::state_to_json(&state)).unwrap();
        }
        if let Some((seat, first, last)) = scripted {
            if (first..=last).contains(&t.turn) {
                let orders = t.orders.get(&legatum_core::PlayerId(seat)).cloned().unwrap_or_default();
                script.insert(t.turn.to_string(), serde_json::to_value(orders).unwrap());
            }
        }
        let before = state.clone();
        let (next, events) = resolve_turn(&state, &t.orders);
        state = next;
        let ok = state.hash().to_string() == t.hash;
        if !ok { mismatches += 1; }
        let cities: Vec<_> = state.cities.values().map(|c| serde_json::json!({
            "id": c.id, "name": c.name, "owner": c.owner, "pos": c.pos, "hp": c.hp,
            "pop": c.population, "capital": c.is_capital,
            "walls": c.buildings.iter().any(|b| format!("{b:?}") == "Walls"),
        })).collect();
        let mut units = std::collections::BTreeMap::<String, std::collections::BTreeMap<String, u32>>::new();
        for u in state.units.values() {
            *units.entry(u.owner.0.to_string()).or_default().entry(format!("{:?}", u.kind)).or_default() += 1;
        }
        let ev: Vec<String> = events.iter().map(|e| format!("{e:?}"))
            .filter(|s| ["Captur", "Eliminat", "Founded", "Destroyed", "Killed", "Razed", "War", "Peace", "Died", "Lost"].iter().any(|k| s.contains(k)))
            .collect();
        let orders0: Vec<String> = t.orders.iter().filter(|(p, _)| state.player(**p).is_human || before.player(**p).is_human)
            .flat_map(|(_, os)| os.iter().map(|o| format!("{o:?}"))).collect();
        writeln!(summary, "{}", serde_json::json!({
            "turn": t.turn, "hash_ok": ok, "cities": cities, "units": units,
            "alive": state.players.iter().map(|p| p.alive).collect::<Vec<_>>(),
            "events": ev, "human_orders": orders0,
        })).unwrap();
    }
    if dumps.contains(&state.turn) {
        std::fs::write(out.join(format!("T{}.json", state.turn)), legatum_wasm::json::state_to_json(&state)).unwrap();
    }
    if let Some((seat, first, last)) = scripted {
        let file = serde_json::json!({ "seat": seat, "first": first, "last": last, "turns": script });
        std::fs::write(out.join(format!("orders-{seat}.json")), serde_json::to_string(&file).unwrap()).unwrap();
    }
    println!("replayed {} turns, {} hash mismatches, final turn {}", history.tape.turns.len(), mismatches, state.turn);
}

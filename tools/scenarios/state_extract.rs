//! State extraction for Scenario Lab cuts (tools/scenarios/extract-state.sh): like war_extract.rs, plus the
//! coalition target and the wars on each turn in summary.jsonl. Needs an engine with coalitions (8.9 or later).
//! War-scenario extraction (claude/war-scenarios): replay a user save with the engine it was
//! recorded on, verify every turn against the recorded hashes, dump full states at chosen turns
//! and a per-turn summary for the human benchmark.
//! war_extract <save.legatum> <out-dir> <turn,turn,...> [audit-plans]
//! Selected turns include fogged views; events.jsonl covers every turn. With audit-plans,
//! replay original brains with persistent memory and compare their orders with the tape.
use legatum_core::resolve_turn;
use std::io::Write;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&a[1]).expect("read save");
    let out = std::path::PathBuf::from(&a[2]);
    std::fs::create_dir_all(&out).unwrap();
    let dumps: Vec<u32> = a[3]
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();
    let outer: serde_json::Value = serde_json::from_str(&text).unwrap();
    let game = outer["game"].as_str().unwrap();
    let value: serde_json::Value = serde_json::from_str(game).unwrap();
    let history: legatum_wasm::history::Recording =
        serde_json::from_value(value["archive"].clone()).expect("archive");
    let mut state = match &history.initial_state {
        Some(json) => legatum_wasm::json::parse_state(json).unwrap(),
        None => history.tape.initial_state(),
    };
    let audit_plans = a.get(4).is_some_and(|s| s == "audit-plans");
    let mut brains: Vec<_> = state
        .players
        .iter()
        .filter(|p| audit_plans && !p.is_human)
        .map(|p| {
            (
                p.id,
                legatum_wasm::session::make_brain(
                    legatum_wasm::session::BrainKind::Scripted,
                    &state,
                    p.id,
                ),
            )
        })
        .collect();
    let mut plans = audit_plans.then(|| std::fs::File::create(out.join("plans.jsonl")).unwrap());
    let mut event_log = std::fs::File::create(out.join("events.jsonl")).unwrap();
    let mut summary = std::fs::File::create(out.join("summary.jsonl")).unwrap();
    let mut mismatches = 0;
    for t in &history.tape.turns {
        if dumps.contains(&state.turn) {
            std::fs::write(
                out.join(format!("T{}.json", state.turn)),
                legatum_wasm::json::state_to_json(&state),
            )
            .unwrap();
        }
        if audit_plans {
            for (id, brain) in &mut brains {
                if !state.player(*id).alive {
                    continue;
                }
                let view = legatum_core::player_view(&state, *id);
                if dumps.contains(&state.turn) {
                    std::fs::write(
                        out.join(format!("T{}-p{}.memory.json", state.turn, id.0)),
                        serde_json::to_string(&brain.opponent_memory()).unwrap(),
                    )
                    .unwrap();
                }
                let memory_before = brain.opponent_memory();
                let orders = brain.orders(&view);
                let recorded = t.orders.get(id);
                writeln!(plans.as_mut().unwrap(), "{}", serde_json::json!({"turn": t.turn, "player": id, "matches_recorded": recorded == Some(&orders), "recorded_orders": recorded, "orders": orders, "memory_before": memory_before, "plan_events": brain.plan_events(), "campaign_events": brain.campaign_events()})).unwrap();
            }
        }
        let before = state.clone();
        if dumps.contains(&state.turn) {
            for player in &state.players {
                let view = legatum_core::player_view(&state, player.id);
                std::fs::write(
                    out.join(format!("T{}-p{}.view.json", state.turn, player.id.0)),
                    serde_json::to_string(&view).unwrap(),
                )
                .unwrap();
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        writeln!(
            event_log,
            "{}",
            serde_json::json!({"turn": t.turn, "events": events})
        )
        .unwrap();
        state = next;
        let ok = state.hash().to_string() == t.hash;
        if !ok {
            mismatches += 1;
        }
        let cities: Vec<_> = state
            .cities
            .values()
            .map(|c| {
                serde_json::json!({
                    "id": c.id, "name": c.name, "owner": c.owner, "pos": c.pos, "hp": c.hp,
                    "pop": c.population, "capital": c.is_capital,
                    "walls": c.buildings.iter().any(|b| format!("{b:?}") == "Walls"),
                })
            })
            .collect();
        let mut units =
            std::collections::BTreeMap::<String, std::collections::BTreeMap<String, u32>>::new();
        for u in state.units.values() {
            *units
                .entry(u.owner.0.to_string())
                .or_default()
                .entry(format!("{:?}", u.kind))
                .or_default() += 1;
        }
        let ev: Vec<String> = events
            .iter()
            .map(|e| format!("{e:?}"))
            .filter(|s| {
                [
                    "Captur",
                    "Eliminat",
                    "Founded",
                    "Destroyed",
                    "Killed",
                    "Razed",
                    "War",
                    "Peace",
                    "Died",
                    "Lost",
                ]
                .iter()
                .any(|k| s.contains(k))
            })
            .collect();
        let orders0: Vec<String> = t
            .orders
            .iter()
            .filter(|(p, _)| state.player(**p).is_human || before.player(**p).is_human)
            .flat_map(|(_, os)| os.iter().map(|o| format!("{o:?}")))
            .collect();
        writeln!(summary, "{}", serde_json::json!({
            "turn": t.turn, "hash_ok": ok, "cities": cities, "units": units,
            "alive": state.players.iter().map(|p| p.alive).collect::<Vec<_>>(),
            "coalition": legatum_core::diplomacy::coalition::target_state(&state).map(|t| t.0),
            "wars": state.players.iter().filter(|a| a.alive).flat_map(|a| state.players.iter().filter(|b| b.alive && b.id.0 > a.id.0).filter(|b| state.at_war(a.id, b.id)).map(|b| format!("{}-{}", a.id.0, b.id.0)).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "events": ev, "human_orders": orders0,
        })).unwrap();
    }
    if dumps.contains(&state.turn) {
        std::fs::write(
            out.join(format!("T{}.json", state.turn)),
            legatum_wasm::json::state_to_json(&state),
        )
        .unwrap();
    }
    println!(
        "replayed {} turns, {} hash mismatches, final turn {}",
        history.tape.turns.len(),
        mismatches,
        state.turn
    );
}

//! Replay a playtest tape and dump everything needed to audit the AI empires, read-only.
//! playtest_ai_trace <tape.json> <out-dir> [check-brains=1]
//! Writes, one JSON object per line and per turn (state before that turn's orders resolve):
//!   snap.jsonl   players (gold, income, techs, research, supply), cities
//!                (build, stored/cost, yields, buildings) and units
//!   events.jsonl every event from resolving that turn
//!   brains.jsonl for each AI seat, whether a freshly built brain on that seat's view
//!                reproduces the tape's orders exactly (proves the tape is this AI's play)
use legatum_core::economy::{city_production_cost, city_yields, player_income};
use legatum_core::{PlayerId, player_view, resolve_turn};
use legatum_wasm::session::{BrainKind, make_brain};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let out = std::path::Path::new(&a[2]);
    let check = a.get(3).is_none_or(|x| x != "0");
    std::fs::create_dir_all(out)?;
    let mut snap = std::io::BufWriter::new(std::fs::File::create(out.join("snap.jsonl"))?);
    let mut evs = std::io::BufWriter::new(std::fs::File::create(out.join("events.jsonl"))?);
    let mut brains = std::io::BufWriter::new(std::fs::File::create(out.join("brains.jsonl"))?);
    let mut state = tape.initial_state();
    eprintln!(
        "initial hash matches: {}",
        state.hash() == tape.initial_hash
    );
    let mut first_mismatch = None;
    // DUMP_TURNS=50,93 also writes state-<turn>.json (the state before that turn resolves).
    let dump: Vec<u32> = std::env::var("DUMP_TURNS")
        .ok()
        .map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_default();
    for turn in &tape.turns {
        if dump.contains(&turn.turn) {
            std::fs::write(
                out.join(format!("state-{}.json", turn.turn)),
                serde_json::to_string(&state)?,
            )?;
        }
        let players: Vec<_> = state
            .players
            .iter()
            .map(|p| {
                let inc = player_income(&state, p.id);
                json!({
                    "id": p.id.0, "alive": p.alive, "gold": p.gold,
                    "gold_per_turn": inc.gold.total, "upkeep": inc.upkeep,
                    "science": inc.science.total, "science_stored": p.science_stored,
                    "researching": p.researching, "queue": p.research_queue,
                    "techs": p.techs, "supply": p.supply,
                    "settlers_trained": p.settlers_trained,
                    "military_purchases": p.military_purchases,
                    "failed_sites": p.failed_city_sites.len(),
                })
            })
            .collect();
        let cities: Vec<_> = state
            .cities
            .values()
            .map(|c| {
                let y = city_yields(&state, c).total;
                json!({
                    "id": c.id.0, "owner": c.owner.0, "name": c.name, "pos": c.pos,
                    "capital": c.is_capital, "pop": c.population, "food_stored": c.food_stored,
                    "producing": c.producing,
                    "stored": c.production_stored,
                    "cost": c.producing.map(|i| city_production_cost(&state, c, i)),
                    "prod": y.production, "food": y.food, "gold": y.gold, "sci": y.science,
                    "buildings": c.buildings, "hp": c.hp, "queue": c.production_queue,
                })
            })
            .collect();
        let units: Vec<_> = state
            .units
            .values()
            .map(|u| {
                json!({"id": u.id.0, "owner": u.owner.0, "kind": u.kind, "pos": u.pos,
                       "hp": u.hp, "goto": u.goto})
            })
            .collect();
        serde_json::to_writer(
            &mut snap,
            &json!({"turn": turn.turn, "players": players, "cities": cities, "units": units}),
        )?;
        snap.write_all(b"\n")?;

        if check {
            let mut rows = Vec::new();
            for p in &state.players {
                if p.is_human || !p.alive {
                    continue;
                }
                let mut brain = make_brain(BrainKind::Scripted, &state, p.id);
                let mine = brain.orders(&player_view(&state, p.id));
                let taped = turn.orders.get(&p.id).cloned().unwrap_or_default();
                let same = serde_json::to_string(&mine)? == serde_json::to_string(&taped)?;
                rows.push(json!({"player": p.id.0, "same": same,
                                 "brain": if same { json!(null) } else { json!(mine) }}));
            }
            serde_json::to_writer(&mut brains, &json!({"turn": turn.turn, "rows": rows}))?;
            brains.write_all(b"\n")?;
        }

        let (next, events) = resolve_turn(&state, &turn.orders);
        serde_json::to_writer(&mut evs, &json!({"turn": turn.turn, "events": events}))?;
        evs.write_all(b"\n")?;
        if first_mismatch.is_none() && next.hash().to_string() != turn.hash {
            first_mismatch = Some(turn.turn);
        }
        state = next;
    }
    eprintln!("first hash mismatch: {first_mismatch:?}");
    let _ = PlayerId(0);
    Ok(())
}

//! Hash-checked, read-only Gold audit of a packaged human save.
//! gold_audit SAVE.legatum > audit.jsonl
use legatum_core::{economy, resolve_turn};
use serde_json::{Value, json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).expect("save path");
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let game: Value = serde_json::from_str(outer["game"].as_str().expect("game string"))?;
    let rec: legatum_wasm::history::Recording = serde_json::from_value(game["archive"].clone())?;
    let mut state = match &rec.initial_state {
        Some(saved) => legatum_wasm::json::parse_state(saved)?,
        None => rec.tape.initial_state(),
    };
    assert_eq!(state.hash(), rec.tape.initial_hash, "initial hash");
    println!("{}", json!({"metadata": game["archive"]["metadata"]}));
    for turn in &rec.tape.turns {
        let players: Vec<_> = state.players.iter().map(|p| {
            let income = economy::player_income(&state, p.id);
            let cities: Vec<_> = state.cities_of(p.id).map(|c| json!({
                "id": c.id.0, "name": c.name, "sources": economy::city_yields(&state, c).sources,
                "buildings": c.buildings.iter().map(|b| legatum_core::data::building_name_for(p.faction, *b)).collect::<Vec<_>>(),
            })).collect();
            let wars: Vec<_> = state.players.iter().filter(|o| o.id != p.id && state.at_war(p.id, o.id))
                .map(|o| o.id.0).collect();
            json!({"id": p.id.0, "faction": p.faction, "human": p.is_human,
                "era": state.era_of(p.id), "gold": p.gold, "income": income.gold,
                "cities": cities, "wars": wars, "alive": p.alive})
        }).collect();
        let (next, events) = resolve_turn(&state, &turn.orders);
        assert_eq!(
            next.hash().to_string(),
            turn.hash,
            "turn {} hash",
            turn.turn
        );
        let after: Vec<_> = next
            .players
            .iter()
            .map(|p| json!({"id":p.id.0,"gold":p.gold}))
            .collect();
        println!(
            "{}",
            json!({"turn": turn.turn, "players": players, "after": after,
            "events": events, "orders": turn.orders})
        );
        state = next;
    }
    Ok(())
}

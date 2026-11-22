//! Inspect one recorded, fogged PlayerView. No simulation or hidden state enters the planner.
//! war_position VIEW.json SETUP_SEED [DECISIONS.json] [MEMORY.json]
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, PlayerView};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        return Err(
            "usage: war_position VIEW[.gz] SETUP_SEED [DECISIONS.json] [MEMORY[.gz]]".into(),
        );
    }
    let view: PlayerView = serde_json::from_reader(reader(&args[1])?)?;
    let seed = legatum_core::rng::Rng::derive(
        args[2].parse()?,
        &[legatum_core::rng::purpose::AI, u64::from(view.me.0)],
    )
    .next_u64();
    let mut brain =
        ScriptedBrain::with_difficulty(view.faction_of(view.me), seed, view.me, AiDifficulty::Hard);
    if let Some(path) = args.get(3) {
        let overrides: serde_json::Map<String, serde_json::Value> =
            serde_json::from_reader(reader(path)?)?;
        let mut decisions = serde_json::to_value(brain.decisions)?;
        decisions.as_object_mut().unwrap().extend(overrides);
        brain.decisions = serde_json::from_value(decisions)?;
    }
    if let Some(path) = args.get(4) {
        let memory = serde_json::from_reader(reader(path)?)?;
        if let Some(memory) = memory {
            brain.restore_opponent_memory(memory);
        }
    }
    let orders = brain.orders(&view);
    let (_, production) = brain.production_diagnosis(&view);
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "turn": view.turn, "player": view.me, "orders": orders,
            "production": production, "plans": brain.plan_events(),
            "campaign_events": brain.campaign_events(),
            "memory": brain.opponent_memory(),
        }))?
    );
    Ok(())
}

fn reader(path: &str) -> Result<Box<dyn std::io::Read>, std::io::Error> {
    let file = std::fs::File::open(path)?;
    Ok(if path.ends_with(".gz") {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    })
}

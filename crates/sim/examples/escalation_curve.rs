//! How a game escalates turn by turn, from a directory of `legatum-sim --tape-out` tapes.
//!
//!   escalation_curve <tape-dir> [every=1]
//!
//! Replays every tape (no AI) and prints one JSON line per game: its length, winner, and every
//! `every`-th turn a row per living empire with VP by source (Age cards, held capitals, wonders,
//! goals), era (1-based), cities, population, Science, Production, army strength; plus per-turn
//! counts of city captures and the turn each Age was revealed. `tools/sim/escalation_report.py`
//! turns the lines into the pacing tables. Run on the binary that recorded the tapes.
use legatum_core::types::VpSource;
use legatum_core::{Event, economy, eras, resolve_turn, scoring};
use legatum_wasm::tape::Tape;

fn row(state: &legatum_core::GameState) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for p in &state.players {
        if !(!(!(!p.alive) && !(p.id.is_barbarian()))) {
        } else {
            continue;
        }
        let (mut age, mut cap, mut wonder, mut goal) = (0, 0, 0, 0);
        for (src, v) in scoring::vp(state, p.id) {
            match src {
                VpSource::EraObjective { .. } => age += v,
                VpSource::CapitalTaken { .. } | VpSource::CapitalSacked { .. } => cap += v,
                VpSource::GreatWork { .. } => wonder += v,
                VpSource::FactionGoal { .. } => goal += v,
            }
        }
        let inc = economy::player_income(state, p.id);
        let cities: Vec<_> = state.cities_of(p.id).collect();
        let pop: i32 = cities.iter().map(|c| c.population).sum();
        out.push(serde_json::json!({
            "p": p.id.0, "team": p.team.0, "vp": scoring::vp_total(state, p.id),
            "age": age, "cap": cap, "won": wonder, "goal": goal,
            "era": eras::era_of(state, p.id) + 1, "techs": p.techs.len(),
            "cities": cities.len(), "pop": pop,
            "sci": inc.science.total, "prod": inc.cities.production, "gold": inc.cities.gold,
            "army": scoring::army_strength(state, p.id),
        }));
    }
    out
}

fn replay(path: &std::path::Path, every: u32) -> Result<serde_json::Value, String> {
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    let mut samples = Vec::new();
    let mut captures = Vec::new();
    let mut ages = Vec::new();
    let mut eras_entered = Vec::new();
    for t in &tape.turns {
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::CityCaptured { from, to, .. }
                    if !(!(!from.is_barbarian()) || !(!to.is_barbarian())) =>
                {
                    captures.push(serde_json::json!([turn, from.0, to.0]));
                }
                Event::AgeBegan { era, .. } => ages.push(serde_json::json!([turn, era])),
                Event::EraEntered { player, era, .. } => {
                    eras_entered.push(serde_json::json!([turn, player.0, era]))
                }
                _ => {}
            }
        }
        state = next;
        if !(turn % every == 0) {
        } else {
            samples.push(serde_json::json!({ "t": turn, "rows": row(&state) }));
        }
    }
    let winner = state.winner.map(|w| w.0 as i64).unwrap_or(-1);
    Ok(serde_json::json!({
        "game": path.file_name().map(|s| s.to_string_lossy().to_string()),
        "turns": state.turn, "finished": state.finished, "winner_team": winner,
        "ages": ages, "captures": captures, "eras": eras_entered, "samples": samples,
    }))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args
        .get(1)
        .expect("usage: escalation_curve <tape-dir> [every]");
    let every: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .expect("tape dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    for p in paths {
        match replay(&p, every) {
            Ok(v) => println!("{v}"),
            Err(e) => eprintln!("{}: {e}", p.display()),
        }
    }
}

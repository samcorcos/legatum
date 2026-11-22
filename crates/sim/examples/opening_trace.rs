//! Replay a tape (e.g. a playtest save's `archive.tape`) and trace one empire's opening:
//! opening_trace <tape.json> <player> [until-turn=60]
//! Per turn: the player's capital Production, what it builds (progress/cost), population, and
//! every empire's city count. Reports hash mismatches (a tape from an older engine drifts).
use legatum_core::economy::{city_production_cost, city_yields};
use legatum_core::{PlayerId, resolve_turn};
use legatum_wasm::tape::Tape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let me = PlayerId(a[2].parse()?);
    let until: u32 = a.get(3).map_or(Ok(60), |x| x.parse())?;
    let mut state = tape.initial_state();
    println!(
        "initial hash matches: {}",
        state.hash() == tape.initial_hash
    );
    let mut first_mismatch = None;
    for turn in &tape.turns {
        if turn.turn > until {
            break;
        }
        let cities: Vec<usize> = state
            .players
            .iter()
            .map(|p| state.cities_of(p.id).count())
            .collect();
        if let Some(c) = state.capital_of(me) {
            let y = city_yields(&state, c);
            let build = c.producing.map(|item| {
                format!(
                    "{item:?} {}/{}",
                    c.production_stored,
                    city_production_cost(&state, c, item)
                )
            });
            println!(
                "t{:>3} pop {} food {} prod {:>2} food+ {:>2} build {:<40} cities {:?}",
                turn.turn,
                c.population,
                c.food_stored,
                y.total.production,
                y.total.food,
                build.unwrap_or_else(|| "-".into()),
                cities
            );
        }
        let (next, _) = resolve_turn(&state, &turn.orders);
        if first_mismatch.is_none() && next.hash().to_string() != turn.hash {
            first_mismatch = Some(turn.turn);
        }
        state = next;
    }
    println!("first hash mismatch: {first_mismatch:?}");
    Ok(())
}

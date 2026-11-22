//! A game's VP race over time, from its tape.
//!   vp_trace <tape.json> [every]
//! Every `every` turns (default 25) and on each victory-hold event: each team's VP, and any hold
//! in progress.
use legatum_core::{Event, resolve_turn, scoring::team_vp};
use legatum_wasm::tape::Tape;

fn main() {
    let path = std::env::args().nth(1).expect("tape");
    let every: u32 = std::env::args()
        .nth(2)
        .map_or(25, |s| s.parse().expect("every"));
    let tape = Tape::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut state = tape.initial_state();
    for t in &tape.turns {
        let (next, events) = resolve_turn(&state, &t.orders);
        let hold_event = events.iter().any(|e| {
            matches!(
                e,
                Event::VictoryHoldStarted { .. } | Event::VictoryHoldBroken { .. }
            )
        });
        state = next;
        if state.turn.is_multiple_of(every) || hold_event {
            let vp: Vec<String> = state
                .teams()
                .into_iter()
                .map(|team| format!("{}:{}", team.0, team_vp(&state, team)))
                .collect();
            let gold: Vec<String> = state
                .players
                .iter()
                .map(|p| format!("{}:{}", p.id.0, p.gold))
                .collect();
            println!(
                "T{} vp [{}] gold [{}] holds {:?}{}",
                state.turn,
                vp.join(" "),
                gold.join(" "),
                state.vp_holds,
                if hold_event { " *" } else { "" }
            );
        }
    }
}

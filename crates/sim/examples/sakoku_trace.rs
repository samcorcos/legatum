//! Japan's Sakoku vow over a game's tape: when it closed and reopened, how long the VP was held,
//! and whether any order failed because of the vow (`sakoku_trace <tape.json> [...]`).
use legatum_core::{Faction, events::Event, resolve_turn};
use legatum_wasm::tape::Tape;

fn main() {
    for path in std::env::args().skip(1) {
        let tape = Tape::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut state = tape.initial_state();
        let mut log: Vec<String> = Vec::new();
        let mut held_turns = 0u32;
        let mut failed = 0u32;
        let india: Vec<_> = state
            .players
            .iter()
            .filter(|p| p.faction == Faction::India)
            .map(|p| p.id)
            .collect();
        if india.is_empty() {
            continue;
        }
        for t in &tape.turns {
            let turn = state.turn;
            let (next, events) = resolve_turn(&state, &t.orders);
            state = next;
            for e in &events {
                match e {
                    Event::SakokuClosed { player } => {
                        log.push(format!("T{turn} p{} closed", player.0))
                    }
                    Event::SakokuReopened { player, turns } => {
                        log.push(format!("T{turn} p{} reopened after {turns}", player.0))
                    }
                    Event::FactionGoalLost {
                        player,
                        faction: Faction::India,
                        ..
                    } => log.push(format!("T{turn} p{} lost the VP", player.0)),
                    Event::OrderFailed { player, reason }
                        if india.contains(player) && reason.contains("closed the country") =>
                    {
                        failed += 1;
                        log.push(format!("T{turn} p{} order failed: {reason}", player.0));
                    }
                    _ => {}
                }
            }
            held_turns += india
                .iter()
                .filter(|p| state.player(**p).faction_goal_done)
                .count() as u32;
        }
        println!(
            "{path}: turns {} winner {:?} india {:?} VP-held turn-ends {held_turns} failed {failed} | {}",
            state.turn,
            state.winner,
            india
                .iter()
                .map(|p| (p.0, state.player(*p).sakoku_turns))
                .collect::<Vec<_>>(),
            log.join("; ")
        );
    }
}

//! Reproducible native automation tape for scripts/wasm-tape-check.mjs.
use legatum_core::{
    Faction, GameSetup, Order, TeamId, UnitKind, new_game, player_view, resolve_turn,
};
use legatum_wasm::tape::{TAPE_VERSION, Tape, TapeTurn};
use std::collections::BTreeMap;

fn main() {
    let mut setup = GameSetup::standard(99123, [Faction::India, Faction::Byzantium]);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players.truncate(2);
    setup.players[1].team = TeamId(1);
    let mut state = new_game(&setup);
    let mut tape = Tape {
        version: TAPE_VERSION,
        setup,
        initial_hash: state.hash(),
        turns: Vec::new(),
    };
    for _ in 0..20 {
        let mut orders = BTreeMap::new();
        for player in &state.players {
            let view = player_view(&state, player.id);
            let mut list = Vec::new();
            for unit in view.units.iter().filter(|u| u.owner == player.id) {
                if unit.kind == UnitKind::Scout && (state.turn == 1 || unit.automation.is_some()) {
                    if let Some(order) = legatum_ai::explore_step(&view, unit.id) {
                        list.push(order);
                    }
                } else if state.turn == 1 && unit.kind == UnitKind::Settler {
                    list.push(Order::FoundCity { unit: unit.id });
                }
            }
            orders.insert(player.id, list);
        }
        let next = resolve_turn(&state, &orders).0;
        tape.turns.push(TapeTurn {
            turn: state.turn,
            orders,
            hash: next.hash().to_string(),
        });
        state = next;
        if state.finished {
            break;
        }
    }
    println!("{}", tape.to_json());
}

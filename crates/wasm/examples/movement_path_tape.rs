//! Intentionally regenerate the confirmed-route save/load tape:
//! cargo run --release -p legatum-wasm --example movement_path_tape > crates/sim/tests/fixtures/movement-path.json
use legatum_core::{
    Faction, GameSetup, Order, PlayerId, UnitKind, new_game, path, player_view, resolve_turn,
    validate_orders,
};
use legatum_wasm::{
    session::OrderMap,
    tape::{Tape, TapeTurn, replay_checked},
};

fn main() {
    let mut setup = GameSetup::standard(u64::MAX - 9, [Faction::India, Faction::Byzantium]);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players.truncate(2);
    setup.players[1].team = legatum_core::TeamId(1);
    let mut state = new_game(&setup);
    let mut tape = Tape {
        version: legatum_wasm::tape::TAPE_VERSION,
        setup,
        initial_hash: state.hash(),
        turns: Vec::new(),
    };
    let view = player_view(&state, PlayerId(0));
    let u = view
        .units
        .iter()
        .find(|u| u.owner == view.me && u.kind == UnitKind::Soldier)
        .unwrap();
    let route = state
        .map
        .hexes()
        .filter_map(|to| {
            path::find_path(&state.map, u.pos, to, |h| {
                if view
                    .units
                    .iter()
                    .any(|v| v.id != u.id && v.military && v.pos == h)
                {
                    return None;
                }
                view.tile(h)
                    .and_then(|t| t.terrain)
                    .and_then(|t| t.move_cost())
            })
        })
        .max_by_key(|p| (p.len(), p.clone()))
        .unwrap();
    assert!(route.len() > 2, "fixture must exercise a persistent route");
    let id = u.id;
    let to = *route.last().unwrap();
    let order = Order::MoveUnitPath {
        unit: id,
        to,
        path: route,
        stop_on_block: false,
    };
    assert!(validate_orders(&state, PlayerId(0), std::slice::from_ref(&order)).is_empty());
    for turn in 1..=8 {
        let orders = if turn == 1 {
            OrderMap::from([(PlayerId(0), vec![order.clone()])])
        } else {
            OrderMap::new()
        };
        state = resolve_turn(&state, &orders).0;
        if turn == 1 {
            assert!(!state.units[&id].path.is_empty());
        }
        tape.turns.push(TapeTurn {
            turn,
            orders,
            hash: state.hash().to_string(),
        });
    }
    assert_eq!(state.units[&id].pos, to);
    assert!(state.units[&id].path.is_empty());
    replay_checked(&tape).unwrap();
    println!("{}", tape.to_json());
}

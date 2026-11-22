//! Intentionally regenerate the fixed queue tape:
//! cargo run --release -p legatum-wasm --example research_queue_tape > crates/sim/tests/fixtures/research-queue.json
use legatum_core::{
    Event, Faction, GameSetup, Order, PlayerId, TechId, UnitKind, new_game, resolve_turn,
};
use legatum_wasm::{
    session::OrderMap,
    tape::{TAPE_VERSION, Tape, TapeTurn, replay_checked},
};

fn main() {
    let mut setup = GameSetup::standard(u64::MAX - 8, [Faction::India, Faction::Byzantium]);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players.truncate(2);
    setup.players[1].team = legatum_core::TeamId(1);
    setup.turn_limit = Some(120);
    let mut state = new_game(&setup);
    let mut tape = Tape {
        version: TAPE_VERSION,
        setup,
        initial_hash: state.hash(),
        turns: Vec::new(),
    };
    let mut finished_queue = false;
    for turn in 1..=120 {
        let mut orders = OrderMap::new();
        if turn == 1 {
            for p in [PlayerId(0), PlayerId(1)] {
                let unit = state
                    .units_of(p)
                    .find(|u| u.kind == UnitKind::Settler)
                    .unwrap()
                    .id;
                orders.insert(p, vec![Order::FoundCity { unit }]);
            }
        }
        if [1, 2, 3].contains(&turn) {
            orders
                .entry(PlayerId(0))
                .or_default()
                .push(Order::SetResearchQueue {
                    techs: if turn == 2 {
                        vec![]
                    } else {
                        vec![
                            TechId::BronzeWorking,
                            TechId::Writing,
                            TechId::Irrigation,
                            TechId::Currency,
                        ]
                    },
                });
        }
        let (next, events) = resolve_turn(&state, &orders);
        finished_queue |= events.contains(&Event::ResearchQueueAdvanced {
            player: PlayerId(0),
            next: None,
        });
        state = next;
        tape.turns.push(TapeTurn {
            turn,
            orders,
            hash: state.hash().to_string(),
        });
    }
    assert!(
        finished_queue,
        "tape must exercise the whole research plan: {:?}",
        state.players[0]
    );
    assert!(state.players[0].research_queue.is_empty());
    replay_checked(&tape).unwrap();
    println!("{}", tape.to_json());
}

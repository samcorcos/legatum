//! Replay recorded orders to inspect development decisions without rerunning or changing AI.
use legatum_core::{
    BuildingKind, Order, ProductionItem, UnitKind, new_game, player_view, resolve_turn,
};
use legatum_wasm::tape::Tape;

fn main() {
    for path in std::env::args().skip(1) {
        let tape = Tape::from_json(&std::fs::read_to_string(&path).expect("read tape"))
            .expect("parse tape");
        let mut state = new_game(&tape.setup);
        assert_eq!(state.hash(), tape.initial_hash);
        for turn in &tape.turns {
            for (player, orders) in &turn.orders {
                for order in orders {
                    if let Order::SetProduction { city, item } = order
                        && matches!(
                            item,
                            ProductionItem::Unit {
                                unit: UnitKind::Settler
                            } | ProductionItem::Building {
                                building: BuildingKind::Hydroponics
                                    | BuildingKind::Autoworks
                                    | BuildingKind::PlanetaryFoundry
                                    | BuildingKind::RelayUplink
                                    | BuildingKind::WorldExchange
                            }
                        )
                    {
                        let view = player_view(&state, *player);
                        println!(
                            "{}",
                            serde_json::json!({"seed":tape.setup.seed.to_string(), "turn":turn.turn,
                            "player":player, "faction":state.player(*player).faction, "city":city,
                            "item":item, "development":view.development, "growth_investments":view.research.unlocks.iter()
                                .filter(|u| u.yields.food > 0).collect::<Vec<_>>() })
                        );
                    }
                }
            }
            state = resolve_turn(&state, &turn.orders).0;
            assert_eq!(
                state.hash().to_string(),
                turn.hash,
                "{path} turn {}",
                turn.turn
            );
        }
    }
}

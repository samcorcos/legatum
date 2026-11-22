//! Intentional production rules fixture. Replay checks every turn and save/load boundary.
use legatum_core::{
    BuildingKind, Event, Faction, GameSetup, Order, PlayerId, ProductionItem, TechId, UnitKind,
    economy, new_game, resolve_turn, validate_orders,
};
use legatum_wasm::{
    session::OrderMap,
    tape::{Tape, TapeTurn, replay_checked},
};

fn main() {
    let mut setup = GameSetup::standard(u64::MAX - 10, [Faction::Switzerland, Faction::Byzantium]);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players.truncate(2);
    setup.players[1].team = legatum_core::TeamId(1);
    setup.turn_limit = Some(150);
    let mut state = new_game(&setup);
    let mut tape = Tape {
        version: legatum_wasm::tape::TAPE_VERSION,
        setup,
        initial_hash: state.hash(),
        turns: Vec::new(),
    };
    let me = PlayerId(0);
    let scout = ProductionItem::Unit {
        unit: UnitKind::Scout,
    };
    let market = ProductionItem::Building {
        building: BuildingKind::Market,
    };
    let monument = ProductionItem::Building {
        building: BuildingKind::Granary,
    };
    let mut bought = false;
    let mut market_completed = false;
    let mut granary_completed = false;
    for turn in 1..=150 {
        let mut orders = OrderMap::new();
        if turn == 1 {
            for p in [me, PlayerId(1)] {
                let unit = state
                    .units_of(p)
                    .find(|u| u.kind == UnitKind::Settler)
                    .unwrap()
                    .id;
                orders.insert(p, vec![Order::FoundCity { unit }]);
            }
            orders.get_mut(&me).unwrap().push(Order::SetResearchQueue {
                techs: vec![TechId::Irrigation, TechId::Currency],
            });
        } else if !bought && state.player(me).techs.contains(&TechId::Currency) {
            let city = state.cities_of(me).next().unwrap();
            if economy::buy_blocker(&state, city, scout).is_none() {
                orders.insert(
                    me,
                    vec![
                        Order::SetProductionQueue {
                            city: city.id,
                            queue: vec![market, monument],
                        },
                        Order::BuyItem {
                            city: city.id,
                            item: scout,
                        },
                    ],
                );
            }
        }
        for (p, os) in &orders {
            assert!(validate_orders(&state, *p, os).is_empty());
        }
        let (next, events) = resolve_turn(&state, &orders);
        market_completed |= events.iter().any(|e| matches!(e, Event::ProductionCompleted { owner, item, .. } if *owner == me && *item == market));
        granary_completed |= events.iter().any(|e| matches!(e, Event::ProductionCompleted { owner, item, .. } if *owner == me && *item == monument));
        if events.iter().any(|e| matches!(e, Event::ItemBought { owner, item, .. } if *owner == me && *item == scout)) {
            bought = true;
            let city = next.cities_of(me).next().unwrap();
            assert!(city.producing == Some(market) || city.producing == Some(monument));
        }
        state = next;
        tape.turns.push(TapeTurn {
            turn,
            orders,
            hash: state.hash().to_string(),
        });
    }
    assert!(
        bought && market_completed && granary_completed,
        "exercise a separate purchase and both queued completions (bought={bought}, market={market_completed}, granary={granary_completed}, gold={})",
        state.player(me).gold,
    );
    replay_checked(&tape).unwrap();
    println!("{}", tape.to_json());
}

//! Controlled economic opening: real generated starts and paid builds, without ruin rewards,
//! diffusion, combat or expansion. This isolates income pacing; it is not a human-play benchmark.
//! cargo run --release -p legatum-core --example science_opening -- 100 700001
use legatum_core::{
    BuildingKind, Event, Faction, GameSetup, Order, PlayerId, ProductionItem, TechId, UnitKind,
    new_game, resolve_turn, rules::cities::validate_city_order, validate_orders,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The five Ancient techs (the strict era gate: nothing from the next era until all are known),
/// then the first Classical one.
const RESEARCH: [TechId; 6] = [
    TechId::BronzeWorking,
    TechId::Writing,
    TechId::Irrigation,
    TechId::Currency,
    TechId::Masonry,
    TechId::CodeOfLaws,
];
const BUILDS: [ProductionItem; 4] = [
    ProductionItem::Unit {
        unit: UnitKind::Scout,
    },
    ProductionItem::Building {
        building: BuildingKind::Workshop,
    },
    ProductionItem::Building {
        building: BuildingKind::Laboratory,
    },
    ProductionItem::Building {
        building: BuildingKind::Granary,
    },
];

fn trial(seed: u64, faction: Faction) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = None;
    setup.players[0].faction = faction;
    for player in &mut setup.players {
        player.is_human = true;
    }
    let mut state = new_game(&setup);
    // Rival capitals stay idle and submit no orders. No one researches for them.
    let player = PlayerId(0);
    let mut discoveries = Vec::new();
    let mut buildings = Vec::new();
    let mut selected = BTreeMap::new();
    let mut build_index = 0;
    let cap = 300;
    while state.turn <= cap && discoveries.len() < RESEARCH.len() && !state.finished {
        let turn = state.turn;
        let mut orders = Vec::new();
        if state.player(player).researching.is_none()
            && let Some(tech) = RESEARCH
                .iter()
                .find(|tech| !state.player(player).techs.contains(tech))
        {
            orders.push(Order::SetResearch { tech: *tech });
            selected.insert(*tech, turn);
        }
        if let Some(city) = state.capital_of(player)
            && city.producing.is_none()
            && let Some(item) = BUILDS.get(build_index)
        {
            let order = Order::SetProduction {
                city: city.id,
                item: *item,
            };
            if validate_city_order(&state, player, &order).is_ok() {
                orders.push(order);
            }
        }
        assert!(validate_orders(&state, player, &orders).is_empty());
        let (next, events) = resolve_turn(&state, &BTreeMap::from([(player, orders)]));
        for event in events {
            match event {
                Event::TechResearched { player: p, tech } if p == player => {
                    discoveries.push(json!({
                        "tech": tech, "selected_turn": selected[&tech], "completion_turn": turn,
                        "research_turns": turn - selected[&tech] + 1,
                    }))
                }
                Event::ProductionCompleted { owner, item, .. } if owner == player => {
                    assert_eq!(item, BUILDS[build_index]);
                    build_index += 1;
                    buildings.push(json!({ "item": item, "completion_turn": turn }));
                }
                Event::RuinExplored { .. } => panic!("controlled opening must not collect ruins"),
                Event::OrderFailed { .. } => {
                    panic!("controlled opening submitted a rejected order")
                }
                _ => {}
            }
        }
        state = next;
    }
    json!({ "seed": seed.to_string(), "faction": faction,
        "discoveries": discoveries, "production": buildings, "last_turn": state.turn - 1 })
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let maps: u64 = args
        .get(1)
        .map_or(100, |arg| arg.parse().expect("map count"));
    let seed: u64 = args.get(2).map_or(700001, |arg| arg.parse().expect("seed"));
    let mut trials = Vec::new();
    for offset in 0..maps {
        for faction in Faction::ALL {
            trials.push(trial(seed.wrapping_add(offset), faction));
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({
        "definition": "Begin with the capital founded; select Bronze Working on turn 1, then Writing, Irrigation, Currency, Masonry (the whole Ancient era) and Code of Laws (the first Classical tech) without research gaps. Build Scout, Workshop, Library, Granary as legal with normal paid production; no purchases or expansion. Units stay still, so there are no ruins or combat; rivals leave their capitals idle and never research, so no diffusion. Resolved turn 1 is the first turn. Research durations include the selection and completion turns. Factions share each map seed; trials are not independent human games.",
        "first_seed": seed.to_string(), "maps": maps, "trials": trials,
    })).unwrap());
}

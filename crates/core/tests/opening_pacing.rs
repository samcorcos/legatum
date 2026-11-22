//! Opening pacing on unmodified generated terrain, including food growth and claimed tiles.
//! Population is territory: a capital works its whole first ring, so start Production spans 5 (the
//! 3 floor plus the capital's +2) to 14 on forest and hills, rather than the old 5 to 6.
//! These are build durations from zero progress, not AI queue order or gold purchases.

use legatum_core::economy;
use legatum_core::rules::cities::{apply_city_orders, process_borders, process_cities};
use legatum_core::{
    CityId, Event, Faction, GameSetup, GameState, Order, ProductionItem, Terrain, UnitKind,
    new_game,
};

const SEEDS: [u64; 6] = [1, 2, 3, 7, 42, 100];

fn capitals(seed: u64) -> GameState {
    new_game(&GameSetup::all_ai(seed))
}

#[test]
fn new_capitals_have_playable_turn_one_production_and_build_estimates() {
    for seed in SEEDS {
        let mut state = capitals(seed);
        let ids: Vec<_> = state.cities.keys().copied().collect();
        assert_eq!(ids.len(), 8);
        for id in ids {
            let city = &state.cities[&id];
            let terrain = state.map.get(city.pos).unwrap().terrain;
            assert!(matches!(terrain, Terrain::Grassland | Terrain::Plains));
            assert_eq!(city.population, 1);
            assert_eq!(city.production_stored, 0);
            assert_eq!(state.player(city.owner).settlers_trained, 0);
            // Every Sparta Settler pays the +50% non-military surcharge (user, 2026-10-04).
            let settler = if state.player(city.owner).faction == Faction::Sparta {
                8..=21
            } else {
                5..=14
            };
            for (unit, target) in [(UnitKind::Soldier, 6..=16), (UnitKind::Settler, settler)] {
                let item = ProductionItem::Unit { unit };
                state.cities.get_mut(&id).unwrap().producing = Some(item);
                let city = &state.cities[&id];
                let production = economy::city_yields(&state, city).total.production;
                assert!(
                    (5..=14).contains(&production),
                    "seed {seed}, {id:?}, {terrain:?}, {unit:?}: {production} production"
                );
                let cost = economy::production_cost(&state, city.owner, item);
                let turns = (cost + production - 1) / production;
                assert!(
                    target.contains(&turns),
                    "seed {seed}, {id:?}, {unit:?}: estimate {turns}"
                );
            }
        }
    }
}

fn build_turns(mut state: GameState, id: CityId, unit: UnitKind) -> u32 {
    let item = ProductionItem::Unit { unit };
    let settler = unit == UnitKind::Settler;
    // A Settler needs population 2 to be chosen: open with a Scout and switch to the
    // Settler as soon as the city grows (the Scout's progress stays banked).
    let city = state.cities.get_mut(&id).unwrap();
    city.producing = Some(if settler {
        ProductionItem::Unit {
            unit: UnitKind::Scout,
        }
    } else {
        item
    });
    // Use the real economic phases so population, the Settler's population gate,
    // overflow and newly claimed tiles all affect the observed completion time.
    for turn in 1..=40 {
        let owner = state.cities[&id].owner;
        if settler && state.cities[&id].producing != Some(item) && state.cities[&id].population >= 2
        {
            let mut events = Vec::new();
            apply_city_orders(
                &mut state,
                owner,
                &[Order::SetProduction { city: id, item }],
                &mut events,
            );
            assert!(events.is_empty(), "{events:?}");
        }
        let mut events = Vec::new();
        process_cities(&mut state, &mut events);
        process_borders(&mut state, &mut events);
        if events.iter().any(|e| {
            matches!(e, Event::ProductionCompleted { city, item: built, .. }
                if *city == id && *built == item)
        }) {
            return turn;
        }
        state.turn += 1;
    }
    panic!("{unit:?} did not finish in 40 turns");
}

#[test]
fn first_units_finish_on_time_with_automatic_tile_priorities() {
    let mut outside_target = Vec::new();
    for seed in SEEDS {
        let state = capitals(seed);
        for id in state.cities.keys().copied() {
            // The capital bonus and Standard opening prices shorten initial builds.
            for (unit, target) in [(UnitKind::Soldier, 6..=18), (UnitKind::Settler, 10..=28)] {
                let turns = build_turns(state.clone(), id, unit);
                if !target.contains(&turns) {
                    outside_target.push(format!("seed {seed}, {id:?}: first {unit:?} completed in {turns} turns (expected {target:?})"));
                }
            }
        }
    }
    assert!(outside_target.is_empty(), "{}", outside_target.join("\n"));
}

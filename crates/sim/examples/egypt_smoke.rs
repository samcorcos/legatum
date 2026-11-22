//! Eight Hard games with Egypt rotated through every seat. Run on AWS spot with run-batch.
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, Order, ProductionItem, economy, rules,
};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::path::PathBuf;

fn arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map_or_else(|| default.into(), |w| w[1].clone())
}

fn play(seed: u64, seat: usize) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.players[seat].faction = Faction::Egypt;
    for player in &mut setup.players {
        player.difficulty = AiDifficulty::Hard;
    }
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    let mut buildings = [0u32; 8];
    let mut founded = [0u32; 8];
    let mut growths = [0u32; 8];
    let mut wars = [0u32; 8];
    let mut invalid = 0usize;
    let mut egypt_invalid = 0usize;
    let mut dry_build_orders = 0usize;
    let mut snapshots = Vec::new();
    while !session.state.finished && session.state.turn <= 1200 {
        let orders = session.ai_orders();
        invalid += orders
            .iter()
            .map(|(p, os)| rules::validate_orders(&session.state, *p, os).len())
            .sum::<usize>();
        if let Some(egypt_orders) = orders.get(&session.state.players[seat].id) {
            egypt_invalid += rules::validate_orders(
                &session.state,
                session.state.players[seat].id,
                egypt_orders,
            )
            .len();
            for order in egypt_orders {
                let city = match order {
                    Order::SetProduction {
                        city,
                        item: ProductionItem::Building { .. },
                    } => Some(*city),
                    Order::SetProductionQueue { city, queue }
                        if queue.first().is_some_and(|item| {
                            matches!(item, ProductionItem::Building { .. })
                        }) =>
                    {
                        Some(*city)
                    }
                    _ => None,
                };
                if let Some(city) = city.and_then(|id| session.state.cities.get(&id))
                    && economy::city_totals(&session.state, city).food <= 2 * city.population
                {
                    dry_build_orders += 1;
                }
            }
        }
        for event in session.step(&orders) {
            match event {
                Event::CityFounded { owner, .. } if !owner.is_barbarian() => {
                    founded[owner.0 as usize] += 1
                }
                Event::CityGrew { owner, .. } if !owner.is_barbarian() => {
                    growths[owner.0 as usize] += 1
                }
                Event::ProductionCompleted {
                    owner,
                    item: ProductionItem::Building { .. },
                    ..
                } if !owner.is_barbarian() => buildings[owner.0 as usize] += 1,
                Event::WarDeclared { relation } => {
                    wars[relation.a.0 as usize] += 1;
                    wars[relation.b.0 as usize] += 1;
                }
                _ => {}
            }
        }
        if session.state.turn == 50 || session.state.turn == 100 {
            let cities: Vec<_> = session
                .state
                .players
                .iter()
                .map(|p| session.state.cities_of(p.id).count())
                .collect();
            let population: Vec<_> = session
                .state
                .players
                .iter()
                .map(|p| {
                    session
                        .state
                        .cities_of(p.id)
                        .map(|c| c.population)
                        .sum::<i32>()
                })
                .collect();
            snapshots.push(json!({"turn": session.state.turn, "egypt_cities": cities[seat],
                "other_hard_mean_cities": (cities.iter().sum::<usize>() - cities[seat]) as f64 / 7.0,
                "egypt_population": population[seat],
                "other_hard_mean_population": (population.iter().sum::<i32>() - population[seat]) as f64 / 7.0,
                "egypt_growths": growths[seat],
                "other_hard_mean_growths": (growths.iter().sum::<u32>() - growths[seat]) as f64 / 7.0,
                "egypt_buildings": buildings[seat],
                "other_hard_mean_buildings": (buildings.iter().sum::<u32>() - buildings[seat]) as f64 / 7.0,
                "egypt_wars": wars[seat],
                "other_hard_mean_wars": (wars.iter().sum::<u32>() - wars[seat]) as f64 / 7.0}));
        }
    }
    json!({"seed": seed, "seat": seat, "finished": session.state.finished,
        "turn": session.state.turn, "invalid": invalid, "egypt_invalid": egypt_invalid, "dry_build_orders": dry_build_orders, "egypt_founded": founded[seat],
        "egypt_growths": growths[seat],
        "egypt_buildings": buildings[seat], "egypt_wars": wars[seat], "snapshots": snapshots})
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = arg(&args, "--games", "8").parse().unwrap();
    let seed: u64 = arg(&args, "--seed", "410000").parse().unwrap();
    let rows: Vec<_> = (0..games)
        .into_par_iter()
        .map(|i| {
            let seat = (seed as usize + i) % 8;
            match std::panic::catch_unwind(|| play(seed + i as u64, seat)) {
                Ok(row) => row,
                Err(_) => json!({"seed": seed + i as u64, "seat": seat, "panic": true}),
            }
        })
        .collect();
    let report = json!({"games": rows});
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    let out = PathBuf::from(arg(&args, "--out-dir", "."));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("egypt-smoke.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

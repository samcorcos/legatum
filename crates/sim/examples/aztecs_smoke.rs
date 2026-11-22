//! Functionality only: eight Hard seats, one of the nine factions omitted per game in rotation, so every faction
//! is seated in 8 of every 9 games and the seats rotate.
//! Run via tools/cloud/run-batch.sh --bin aztecs_smoke, never a long local run.
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, rules, scoring};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

fn arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map_or_else(|| default.into(), |w| w[1].clone())
}
fn play(seed: u64) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    let all: Vec<_> = Faction::ALL.into_iter().collect();
    let omitted = (seed % all.len() as u64) as usize;
    let mut roster: Vec<_> = all
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| (i != omitted).then_some(f))
        .collect();
    roster.rotate_right((seed % 8) as usize);
    for (p, faction) in setup.players.iter_mut().zip(roster) {
        p.faction = faction;
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    let mut invalid = 0;
    let mut failures = BTreeMap::<String, usize>::new();
    let mut flowers = BTreeMap::<u8, usize>::new();
    let mut rites = BTreeMap::<u8, usize>::new();
    let cap = 1200;
    while !session.state.finished && session.state.turn <= cap {
        let orders = session.ai_orders();
        for (p, list) in &orders {
            let errors = rules::validate_orders(&session.state, *p, list);
            invalid += errors.len();
            for error in errors {
                *failures
                    .entry(format!("validation: {}", error.message))
                    .or_default() += 1;
            }
        }
        for event in session.step(&orders) {
            match event {
                Event::FlowerWars { owner, .. } => *flowers.entry(owner.0).or_default() += 1,
                Event::SwarmUnleashed { player, .. } => *rites.entry(player.0).or_default() += 1,
                Event::OrderFailed { reason, .. } => {
                    *failures.entry(format!("resolution: {reason}")).or_default() += 1
                }
                _ => {}
            }
        }
    }
    let state = &session.state;
    let seats: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            json!({
                "faction": p.faction, "cities": state.cities_of(p.id).count(),
                "vp": scoring::vp(state, p.id).iter().map(|(_, v)| *v).sum::<u32>(),
                "flower_wars_population": flowers.get(&p.id.0).copied().unwrap_or(0),
                "new_fire_completed": rites.get(&p.id.0).copied().unwrap_or(0),
            })
        })
        .collect();
    json!({"seed": seed, "turn": state.turn, "stalled": !state.finished, "invalid_orders": invalid, "failures": failures, "seats": seats})
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: usize = arg(&args, "--games", "64").parse().unwrap();
    let seed: u64 = arg(&args, "--seed", "8310000").parse().unwrap();
    let threads: usize = arg(&args, "--threads", "8").parse().unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    let rows: Vec<_> = (0..games).into_par_iter().map(|i| {
        let seed = seed + i as u64;
        match std::panic::catch_unwind(|| play(seed)) {
            Ok(row) => row,
            Err(e) => json!({"seed": seed, "panic": e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("unknown panic")}),
        }
    }).collect();
    let mut by_faction = BTreeMap::<String, (u64, u64, u64, u64, u64)>::new();
    let mut failures = BTreeMap::<String, u64>::new();
    for row in &rows {
        if let Some(seats) = row["seats"].as_array() {
            for seat in seats {
                let s = by_faction
                    .entry(seat["faction"].as_str().unwrap().into())
                    .or_default();
                s.0 += 1;
                s.1 += seat["cities"].as_u64().unwrap();
                s.2 += seat["vp"].as_u64().unwrap();
                s.3 += seat["flower_wars_population"].as_u64().unwrap();
                s.4 += seat["new_fire_completed"].as_u64().unwrap();
            }
        }
        if let Some(f) = row["failures"].as_object() {
            for (k, v) in f {
                *failures.entry(k.clone()).or_default() += v.as_u64().unwrap();
            }
        }
    }
    let factions: BTreeMap<_, _> = by_faction.into_iter().map(|(f, (n,c,v,p,r))| (f, json!({"seats": n, "mean_cities": c as f64/n as f64, "mean_vp": v as f64/n as f64, "flower_wars_population": p, "new_fire_completions": r}))).collect();
    let summary = json!({"games": games, "panics": rows.iter().filter(|r| r.get("panic").is_some()).count(),
        "stalls_at_1200": rows.iter().filter(|r| r["stalled"] == true).count(),
        "invalid_orders": rows.iter().filter_map(|r| r["invalid_orders"].as_u64()).sum::<u64>(),
        "failures": failures, "factions": factions});
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    let out = PathBuf::from(arg(&args, "--out-dir", "."));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join("aztecs-smoke.json"),
        serde_json::to_vec_pretty(&json!({"summary": summary, "games": rows})).unwrap(),
    )
    .unwrap();
}

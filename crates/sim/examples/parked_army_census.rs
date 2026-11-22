//! Parked armies, upgrades and captures from a directory of `legatum-sim --tape-out` tapes
//! (gene `march_refit`, Fable review 2026-11-09 change 2).
//!
//! cargo run --release -p legatum-sim --example parked_army_census -- <tape-dir> [threads]
//!
//! Replays every tape (no AI) and counts per game, over every seat:
//! - **idle-army turns**: seat-turns in which three or more of the seat's combat units stand
//!   within three tiles of a city of an empire it is at War with, and none of its units attacks
//!   anything that turn (the 8.9.29 Celts at St. Gallen: 15 such turns);
//! - **upgrades** (`UnitUpgraded` events) and the Gold spent on them;
//! - **captures** (`CityCaptured`, plus Celtic sacks, `CityRazed` with sack Gold), and city
//!   attacks (battles on a city by a seat's unit).
//!
//! Prints one JSON line per game and a summary line with the per-game means.
use legatum_core::events::{Attacker, Defender};
use legatum_core::rules::is_military;
use legatum_core::{Event, PlayerId, UnitKind, resolve_turn};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use std::collections::BTreeSet;

#[derive(Default, Clone, Copy)]
struct Game {
    turns: u32,
    idle_turns: u32,
    upgrades: u32,
    upgrade_gold: i64,
    captures: u32,
    city_attacks: u32,
}

fn replay(path: &std::path::Path) -> Result<Game, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tape = Tape::from_json(&text)?;
    let mut state = tape.initial_state();
    let mut g = Game::default();
    for t in &tape.turns {
        let (next, events) = resolve_turn(&state, &t.orders);
        let attackers: BTreeSet<PlayerId> = events
            .iter()
            .filter_map(|e| match e {
                Event::Battle {
                    attacker: Attacker::Unit(_),
                    attacker_owner,
                    ..
                } => Some(*attacker_owner),
                _ => None,
            })
            .collect();
        for p in state.players.iter().filter(|p| p.alive) {
            if attackers.contains(&p.id) {
                continue;
            }
            let enemy_cities: Vec<_> = state
                .cities
                .values()
                .filter(|c| {
                    !c.owner.is_barbarian() && c.owner != p.id && state.at_war(p.id, c.owner)
                })
                .map(|c| c.pos)
                .collect();
            if enemy_cities.is_empty() {
                continue;
            }
            let near = state
                .units
                .values()
                .filter(|u| u.owner == p.id && is_military(u.kind) && u.kind != UnitKind::Scout)
                .filter(|u| enemy_cities.iter().any(|c| c.distance(u.pos) <= 3))
                .count();
            if near >= 3 {
                g.idle_turns += 1;
            }
        }
        for e in &events {
            match e {
                Event::UnitUpgraded { gold, .. } => {
                    g.upgrades += 1;
                    g.upgrade_gold += i64::from(*gold);
                }
                Event::CityCaptured { .. } => g.captures += 1,
                Event::CityRazed {
                    sack_gold: Some(_), ..
                } => g.captures += 1,
                Event::Battle {
                    attacker: Attacker::Unit(_),
                    defender: Defender::City { .. },
                    ..
                } => g.city_attacks += 1,
                _ => {}
            }
        }
        state = next;
    }
    g.turns = state.turn;
    Ok(g)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args
        .get(1)
        .expect("usage: parked_army_census <tape-dir> [threads]");
    let threads: usize = args.get(2).map_or(8, |s| s.parse().expect("threads"));
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .expect("tape dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("pool");
    let games: Vec<(String, Game)> = pool.install(|| {
        paths
            .par_iter()
            .map(|p| {
                let g = replay(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                (p.file_name().unwrap().to_string_lossy().into_owned(), g)
            })
            .collect()
    });
    let n = games.len().max(1) as f64;
    let mut sum = Game::default();
    for (name, g) in &games {
        println!(
            "{{\"tape\":\"{name}\",\"turns\":{},\"idle_turns\":{},\"upgrades\":{},\"upgrade_gold\":{},\"captures\":{},\"city_attacks\":{}}}",
            g.turns, g.idle_turns, g.upgrades, g.upgrade_gold, g.captures, g.city_attacks
        );
        sum.turns += g.turns;
        sum.idle_turns += g.idle_turns;
        sum.upgrades += g.upgrades;
        sum.upgrade_gold += g.upgrade_gold;
        sum.captures += g.captures;
        sum.city_attacks += g.city_attacks;
    }
    println!(
        "{{\"summary\":true,\"games\":{},\"turns\":{:.1},\"idle_turns\":{:.2},\"upgrades\":{:.2},\"upgrade_gold\":{:.1},\"captures\":{:.2},\"city_attacks\":{:.2}}}",
        games.len(),
        f64::from(sum.turns) / n,
        f64::from(sum.idle_turns) / n,
        f64::from(sum.upgrades) / n,
        sum.upgrade_gold as f64 / n,
        f64::from(sum.captures) / n,
        f64::from(sum.city_attacks) / n
    );
}

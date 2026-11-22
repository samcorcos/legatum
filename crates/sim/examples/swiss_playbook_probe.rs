//! The Swiss playbook probe: eight Hard AI seats (`GameSetup::all_ai`, eight of the nine factions
//! by seed), every seat on the same AI params, and per seat the numbers the designer's Swiss plan
//! is judged by (`docs/sim-results/ai-quality/swiss-playbook-1105/README.md`): cities, treasury and
//! standing army at fixed turns, how soon a new city raises its Walls, what it builds first, Settlers
//! trained and bought, purchases by kind, cities lost, the hoard goal, the winner.
//!
//!   cargo run --release -p legatum-sim --example swiss_playbook_probe -- \
//!       --games 64 --seed 9311003 [--ai-params FILE] [--threads N] [--out-dir DIR] [--label L]
//!
//! `--setup FILE` (a `GameSetup` JSON, every seat AI) plays that fixed map instead: game i keeps the
//! map and reseeds the dice and the brains from `--seed` + i (the designer's 8.9.24 start, every
//! seat Hard); `--setup 8924` is the designer's 8.9.24 start, every seat Hard
//! (`docs/sim-results/ai-quality/swiss-playbook-1105/setup-8924.json`, read at run time).
//!
//! Run it in the cloud (`tools/cloud/run-batch.sh --bin swiss_playbook_probe`), never a long local run.
use legatum_ai::{AiParams, ScriptedBrain};
use legatum_core::{AiDifficulty, Event, GameSetup, PlayerId, ProductionItem, UnitKind, data};
use legatum_wasm::session::{BrainKind, Session, brain_seed};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

/// The designer's 8.9.24 start. Read at run time, not built in: the public export leaves the file out (it is a
/// personal game setup), and a missing file must not break the workspace build.
const SETUP_8924: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/sim-results/ai-quality/swiss-playbook-1105/setup-8924.json"
);

fn arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map_or_else(|| default.into(), |w| w[1].clone())
}

const MARKS: [u32; 4] = [25, 50, 100, 150];

#[derive(Default, Clone)]
struct Seat {
    cities: BTreeMap<u32, usize>,
    gold: BTreeMap<u32, i32>,
    military: BTreeMap<u32, usize>,
    walled: BTreeMap<u32, (usize, usize)>,
    founded: u32,
    first_build: BTreeMap<String, u32>,
    walls_delay: Vec<u32>,
    settlers_built_t100: u32,
    settlers_bought_t100: u32,
    military_built_t100: u32,
    buys_t100: BTreeMap<String, (u32, i32)>,
    cities_lost: u32,
    cities_taken: u32,
    hoard_turn: Option<u32>,
    eliminated: Option<u32>,
    max_cities: usize,
    /// The first five technologies, in order, with their turns.
    first_techs: Vec<(String, u32)>,
}

fn kind(item: &ProductionItem) -> String {
    match item {
        ProductionItem::Unit { unit } if *unit == UnitKind::Settler => "settler".into(),
        ProductionItem::Unit { unit } if data::pays_upkeep(*unit) => "military".into(),
        ProductionItem::Unit { .. } => "civilian".into(),
        ProductionItem::Building { building } => format!("{building:?}"),
        ProductionItem::Wonder { .. } => "wonder".into(),
        ProductionItem::Improvement { .. } => "improvement".into(),
        ProductionItem::Project { .. } => "project".into(),
    }
}

fn play(seed: u64, params: Option<&AiParams>, fixed: Option<&GameSetup>) -> Value {
    let mut session = match fixed {
        None => {
            let mut setup = GameSetup::all_ai(seed);
            for p in &mut setup.players {
                p.difficulty = AiDifficulty::Hard;
            }
            setup.turn_limit = None;
            Session::new(&setup, |_| BrainKind::Scripted)
        }
        Some(setup) => {
            let mut state = legatum_core::new_game(setup);
            // The same map; this game's own dice and brains.
            state.setup.seed = seed;
            Session::from_state(state, |_| BrainKind::Scripted)
        }
    };
    let start: Vec<_> = session
        .state
        .players
        .iter()
        .map(|p| {
            session
                .state
                .units_of(p.id)
                .find(|u| u.kind == UnitKind::Settler)
                .map(|u| u.pos)
        })
        .collect();
    if let Some(params) = params {
        for p in session.state.players.clone() {
            session.replace_brain(
                p.id,
                Box::new(ScriptedBrain::with_params(
                    p.faction,
                    brain_seed(seed, p.id),
                    p.id,
                    AiDifficulty::Hard,
                    params,
                )),
            );
        }
    }
    let n = session.state.players.len();
    let mut seats = vec![Seat::default(); n];
    let capitals: Vec<_> = (0..n)
        .map(|i| {
            session
                .state
                .cities_of(PlayerId(i as u8))
                .map(|c| c.id)
                .collect::<Vec<_>>()
        })
        .collect();
    // City -> (founding turn, owner, first item recorded, walls recorded)
    let mut young: BTreeMap<legatum_core::CityId, (u32, PlayerId, bool, bool)> = BTreeMap::new();
    let cap = 400;
    while !session.state.finished && session.state.turn <= cap {
        let turn = session.state.turn;
        let orders = session.ai_orders();
        for event in session.step(&orders) {
            match event {
                Event::CityFounded { city, owner, .. } => {
                    if !capitals[owner.0 as usize].contains(&city) {
                        seats[owner.0 as usize].founded += 1;
                        young.insert(city, (turn, owner, false, false));
                    }
                }
                Event::ProductionCompleted { city, owner, item } => {
                    let s = &mut seats[owner.0 as usize];
                    let k = kind(&item);
                    if turn <= 100 {
                        if k == "settler" {
                            s.settlers_built_t100 += 1;
                        } else if k == "military" {
                            s.military_built_t100 += 1;
                        }
                    }
                    if let Some(entry) = young.get_mut(&city)
                        && entry.1 == owner
                    {
                        if !entry.2 && !matches!(item, ProductionItem::Improvement { .. }) {
                            entry.2 = true;
                            *s.first_build.entry(k.clone()).or_default() += 1;
                        }
                        if !entry.3 && k == "Walls" {
                            entry.3 = true;
                            s.walls_delay.push(turn - entry.0);
                        }
                    }
                }
                Event::ItemBought {
                    owner, item, gold, ..
                } if turn <= 100 => {
                    let s = &mut seats[owner.0 as usize];
                    let k = kind(&item);
                    if k == "settler" {
                        s.settlers_bought_t100 += 1;
                    }
                    let e = s.buys_t100.entry(k).or_default();
                    e.0 += 1;
                    e.1 += gold;
                }
                Event::CityCaptured { from, to, city, .. } => {
                    if !from.is_barbarian() && (from.0 as usize) < n {
                        seats[from.0 as usize].cities_lost += 1;
                    }
                    if (to.0 as usize) < n {
                        seats[to.0 as usize].cities_taken += 1;
                    }
                    young.remove(&city);
                }
                Event::TechResearched { player, tech } if (player.0 as usize) < n => {
                    let s = &mut seats[player.0 as usize];
                    if s.first_techs.len() < 5 {
                        s.first_techs.push((format!("{tech:?}"), turn));
                    }
                }
                Event::FactionGoalCompleted {
                    player,
                    faction: legatum_core::Faction::Switzerland,
                    ..
                } => {
                    let s = &mut seats[player.0 as usize];
                    s.hoard_turn.get_or_insert(turn);
                }
                _ => {}
            }
        }
        let state = &session.state;
        for (i, p) in state.players.iter().enumerate() {
            let s = &mut seats[i];
            let cities: Vec<_> = state.cities_of(p.id).collect();
            s.max_cities = s.max_cities.max(cities.len());
            if !p.alive && s.eliminated.is_none() {
                s.eliminated = Some(turn);
            }
            if MARKS.contains(&turn) || turn == 60 {
                s.cities.insert(turn, cities.len());
                s.gold.insert(turn, p.gold);
                s.military.insert(
                    turn,
                    state
                        .units_of(p.id)
                        .filter(|u| data::pays_upkeep(u.kind))
                        .count(),
                );
                let walled = cities
                    .iter()
                    .filter(|c| data::wall_tier(&c.buildings) > 0)
                    .count();
                s.walled.insert(turn, (walled, cities.len()));
            }
        }
    }
    let state = &session.state;
    let winners: Vec<_> = state
        .winner
        .map(|t| {
            state
                .players
                .iter()
                .filter(|p| p.team == t)
                .map(|p| p.id.0)
                .collect()
        })
        .unwrap_or_default();
    let rows: Vec<_> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let s = &seats[i];
            json!({
                "faction": p.faction, "win": winners.contains(&p.id.0), "alive": p.alive,
                "cities": s.cities, "gold": s.gold, "military": s.military,
                "walled": s.walled.iter().map(|(t, (w, c))| (t.to_string(), json!([w, c]))).collect::<BTreeMap<_, _>>(),
                "founded": s.founded, "first_build": s.first_build, "walls_delay": s.walls_delay,
                "settlers_built_t100": s.settlers_built_t100, "settlers_bought_t100": s.settlers_bought_t100,
                "military_built_t100": s.military_built_t100,
                "buys_t100": s.buys_t100.iter().map(|(k, (c, g))| (k.clone(), json!([c, g]))).collect::<BTreeMap<_, _>>(),
                "cities_lost": s.cities_lost, "cities_taken": s.cities_taken,
                "hoard_turn": s.hoard_turn, "eliminated": s.eliminated, "max_cities": s.max_cities, "first_techs": s.first_techs,
                "end_cities": state.cities_of(p.id).count(), "end_gold": p.gold,
            })
        })
        .collect();
    json!({"seed": seed, "turn": state.turn, "stalled": !state.finished, "seats": rows, "start": start})
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: usize = arg(&args, "--games", "64").parse().unwrap();
    let seed: u64 = arg(&args, "--seed", "9311003").parse().unwrap();
    let threads: usize = arg(&args, "--threads", "8").parse().unwrap();
    let label = arg(&args, "--label", "swiss-probe");
    let params_path = arg(&args, "--ai-params", "");
    let setup_path = arg(&args, "--setup", "");
    let fixed: Option<&'static GameSetup> = (!setup_path.is_empty()).then(|| {
        let text = if setup_path == "8924" {
            std::fs::read_to_string(SETUP_8924)
                .expect("read setup-8924.json (not in the public export)")
        } else {
            std::fs::read_to_string(&setup_path).expect("read --setup")
        };
        &*Box::leak(Box::new(
            serde_json::from_str::<GameSetup>(&text).expect("parse --setup"),
        ))
    });
    let params: Option<&'static AiParams> = (!params_path.is_empty()).then(|| {
        let text = std::fs::read_to_string(&params_path).expect("read --ai-params");
        &*Box::leak(Box::new(
            serde_json::from_str::<AiParams>(&text).expect("parse --params"),
        ))
    });
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
    let rows: Vec<_> = (0..games)
        .into_par_iter()
        .map(|i| {
            let seed = seed + i as u64;
            match std::panic::catch_unwind(|| play(seed, params, fixed)) {
                Ok(row) => row,
                Err(e) => json!({"seed": seed, "panic": e.downcast_ref::<String>().map(String::as_str).or_else(|| e.downcast_ref::<&str>().copied()).unwrap_or("unknown panic")}),
            }
        })
        .collect();
    let mut seats = BTreeMap::<String, (u64, u64)>::new();
    for row in &rows {
        for s in row["seats"].as_array().into_iter().flatten() {
            let e = seats
                .entry(s["faction"].as_str().unwrap().into())
                .or_default();
            e.0 += 1;
            e.1 += u64::from(s["win"] == true);
        }
    }
    let share: BTreeMap<_, _> = seats
        .iter()
        .map(|(f, (n, w))| {
            (
                f.clone(),
                json!({"seats": n, "wins": w, "share": *w as f64 / *n as f64}),
            )
        })
        .collect();
    let summary = json!({"label": label, "games": games, "seed": seed,
        "panics": rows.iter().filter(|r| r.get("panic").is_some()).count(),
        "stalled": rows.iter().filter(|r| r["stalled"] == true).count(),
        "win_share": share});
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    let out = PathBuf::from(arg(&args, "--out-dir", "."));
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        out.join(format!("{label}-probe.json")),
        serde_json::to_vec(&json!({"summary": summary, "games": rows})).unwrap(),
    )
    .unwrap();
}

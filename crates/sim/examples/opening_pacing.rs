//! Opening-pace measurement (playtest G8): how fast AI-only empires expand, at the Standard pace.
//!
//! opening_pacing <games> <first-seed> <out.jsonl> [experiments] [difficulty]
//!   experiments: comma-separated RuleExperiments field
//!   names (e.g. `early_pace`) or `-`; difficulty: easy|normal|hard (default normal).
//!
//! Setup matches legatum-sim's default run (one seat per faction, shuffled from the seed,
//! scripted brains, no turn limit, safety cap 4x expected length). Writes one JSON line per
//! game and prints a summary: first Settler completed, second city founded, cities per empire
//! at fixed turns, capital Production, turns for the capital to build a Soldier at turn 2,
//! game length and faction win rates (best/worst ratio, pillar 8).
use legatum_core::economy::{city_yields, production_cost};
use legatum_core::experiments::RuleExperiments;
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, PlayerId, ProductionItem, UnitKind};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;

const CITY_TURNS: [u32; 5] = [25, 50, 75, 100, 150];
/// Sampled at the start of these turns; the capital exists from turn 1.
const PROD_TURNS: [u32; 4] = [2, 10, 25, 50];

#[derive(Serialize, Default, Clone)]
struct Empire {
    faction: String,
    first_soldier: Option<u32>,
    first_settler: Option<u32>,
    second_city: Option<u32>,
    third_city: Option<u32>,
    /// First six completed builds, "turn:item".
    first_builds: Vec<String>,
    cities_at: Vec<Option<u32>>,
    capital_production_at: Vec<Option<i32>>,
    soldier_turns_at_2: Option<i32>,
    settler_cost_at_2: Option<i32>,
}

#[derive(Serialize)]
struct Game {
    seed: u64,
    finished: bool,
    turns: u32,
    winner: Option<u8>,
    empires: Vec<Empire>,
    panic: Option<String>,
}

fn setup(seed: u64, rules: RuleExperiments, d: AiDifficulty) -> GameSetup {
    // Same seat shuffle as legatum-sim's runner::setup_for_game.
    let mut s = GameSetup::all_ai(seed);
    let mut factions = Faction::ALL;
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (p, f) in s.players.iter_mut().zip(factions) {
        p.faction = f;
        p.difficulty = d;
    }
    s.rule_experiments = rules;
    s
}

fn play(seed: u64, rules: RuleExperiments, d: AiDifficulty) -> Game {
    let setup = setup(seed, rules, d);
    // OPENING_TURNS stops a probe early (the opening is settled by turn 60).
    let cap = std::env::var("OPENING_TURNS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(1200);
    let mut s = Session::new(&setup, |_| BrainKind::Scripted);
    s.history = None;
    let mut e: Vec<Empire> = s
        .state
        .players
        .iter()
        .map(|p| Empire {
            faction: format!("{:?}", p.faction),
            cities_at: vec![None; CITY_TURNS.len()],
            capital_production_at: vec![None; PROD_TURNS.len()],
            ..Empire::default()
        })
        .collect();
    // Starting capitals are already founded and do not emit a turn-resolution event.
    let mut founded: Vec<u32> = s
        .state
        .players
        .iter()
        .map(|p| s.state.cities_of(p.id).count() as u32)
        .collect();
    let soldier = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    let mut panic = None;
    while !s.state.finished && s.state.turn < cap {
        let turn = s.state.turn;
        for (i, emp) in e.iter_mut().enumerate() {
            let p = PlayerId(i as u8);
            if let Some(k) = CITY_TURNS.iter().position(|t| *t == turn) {
                emp.cities_at[k] = Some(s.state.cities_of(p).count() as u32);
            }
            if let Some(k) = PROD_TURNS.iter().position(|t| *t == turn)
                && let Some(c) = s.state.capital_of(p)
            {
                let prod = city_yields(&s.state, c).total.production;
                emp.capital_production_at[k] = Some(prod);
                if turn == 2 {
                    let cost = production_cost(&s.state, p, soldier);
                    emp.soldier_turns_at_2 = Some((cost + prod - 1) / prod.max(1));
                    emp.settler_cost_at_2 = Some(production_cost(&s.state, p, settler));
                }
            }
        }
        let step = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let orders = s.ai_orders();
            s.step(&orders)
        }));
        let events = match step {
            Ok(events) => events,
            Err(err) => {
                panic = Some(
                    err.downcast_ref::<String>()
                        .cloned()
                        .or_else(|| err.downcast_ref::<&str>().map(|x| x.to_string()))
                        .unwrap_or_default(),
                );
                break;
            }
        };
        for ev in &events {
            match ev {
                Event::ProductionCompleted { owner, item, .. } => {
                    let emp = &mut e[owner.0 as usize];
                    if emp.first_builds.len() < 6 {
                        let name = match item {
                            ProductionItem::Unit { unit } => format!("{unit:?}"),
                            ProductionItem::Building { building }
                            | ProductionItem::Wonder { building, .. } => format!("{building:?}"),
                            ProductionItem::Project { project } => format!("{project:?}"),
                            ProductionItem::Improvement { improvement, .. } => {
                                format!("{improvement:?}")
                            }
                        };
                        emp.first_builds.push(format!("{turn}:{name}"));
                    }
                    if *item == soldier && emp.first_soldier.is_none() {
                        emp.first_soldier = Some(turn);
                    }
                    if *item == settler && emp.first_settler.is_none() {
                        emp.first_settler = Some(turn);
                    }
                }
                Event::CityFounded { owner, .. } => {
                    let k = owner.0 as usize;
                    founded[k] += 1;
                    if founded[k] == 2 {
                        e[k].second_city = Some(turn);
                    }
                    if founded[k] == 3 {
                        e[k].third_city = Some(turn);
                    }
                }
                _ => {}
            }
        }
    }
    Game {
        seed,
        finished: s.state.finished,
        turns: s.state.turn - 1,
        winner: s.state.winner.map(|t| t.0),
        empires: e,
        panic,
    }
}

fn pct(v: &mut [u32], p: usize) -> String {
    if v.is_empty() {
        return "-".into();
    }
    v.sort_unstable();
    let i = (p * (v.len() - 1) + 50) / 100;
    v[i.min(v.len() - 1)].to_string()
}

fn dist(mut v: Vec<u32>, total: usize) -> String {
    let reached = v.len();
    format!(
        "{} / {} / {} (reached {reached}/{total})",
        pct(&mut v, 50),
        pct(&mut v, 10),
        pct(&mut v, 90)
    )
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let first: u64 = a[2].parse().unwrap();
    let out = &a[3];
    let mut flags = serde_json::Map::new();
    if let Some(list) = a.get(4).filter(|x| x.as_str() != "-") {
        for name in list.split(',') {
            flags.insert(name.into(), true.into());
        }
    }
    let rules: RuleExperiments = serde_json::from_value(flags.clone().into()).unwrap();
    assert_eq!(
        serde_json::to_value(rules)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, v)| **v == serde_json::Value::Bool(true))
            .count(),
        flags.len(),
        "unknown experiment name"
    );
    let difficulty: AiDifficulty = a.get(5).map_or(AiDifficulty::Normal, |d| {
        serde_json::from_str(&format!("\"{}\"", capitalize(d))).unwrap()
    });
    let results: Vec<Game> = (first..first + games)
        .into_par_iter()
        .map(|seed| play(seed, rules, difficulty))
        .collect();
    let mut f = std::fs::File::create(out).unwrap();
    for g in &results {
        writeln!(f, "{}", serde_json::to_string(g).unwrap()).unwrap();
    }
    let ok: Vec<&Game> = results.iter().filter(|g| g.panic.is_none()).collect();
    let emps: Vec<&Empire> = ok.iter().flat_map(|g| &g.empires).collect();
    let total = emps.len();
    println!(
        "Standard games={} panics={} experiments={:?} difficulty={:?}",
        results.len(),
        results.len() - ok.len(),
        flags.keys().collect::<Vec<_>>(),
        difficulty
    );
    println!("metric: median / p10 / p90");
    let col = |f: &dyn Fn(&Empire) -> Option<u32>| emps.iter().filter_map(|e| f(e)).collect();
    println!(
        "first Soldier completed: {}",
        dist(col(&|e| e.first_soldier), total)
    );
    println!(
        "first Settler completed: {}",
        dist(col(&|e| e.first_settler), total)
    );
    println!(
        "second city founded:     {}",
        dist(col(&|e| e.second_city), total)
    );
    println!(
        "third city founded:      {}",
        dist(col(&|e| e.third_city), total)
    );
    for (k, t) in CITY_TURNS.iter().enumerate() {
        let v: Vec<u32> = emps.iter().filter_map(|e| e.cities_at[k]).collect();
        let mean = v.iter().sum::<u32>() as f64 / v.len().max(1) as f64;
        println!("cities at turn {t}: mean {mean:.2}, {}", dist(v, total));
    }
    for (k, t) in PROD_TURNS.iter().enumerate() {
        let v: Vec<u32> = emps
            .iter()
            .filter_map(|e| e.capital_production_at[k].map(|x| x as u32))
            .collect();
        println!("capital Production at turn {t}: {}", dist(v, total));
    }
    println!(
        "Soldier turns from capital at turn 2: {}",
        dist(col(&|e| e.soldier_turns_at_2.map(|x| x as u32)), total)
    );
    println!(
        "Settler cost at turn 2: {}",
        dist(col(&|e| e.settler_cost_at_2.map(|x| x as u32)), total)
    );
    let finished: Vec<u32> = ok.iter().filter(|g| g.finished).map(|g| g.turns).collect();
    println!(
        "game length (finished): {} ; unfinished (cap) {}",
        dist(finished, ok.len()),
        ok.iter().filter(|g| !g.finished).count()
    );
    let mut seats: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
    for g in &ok {
        for (i, emp) in g.empires.iter().enumerate() {
            let r = seats.entry(&emp.faction).or_default();
            r.0 += 1;
            if g.winner == Some(i as u8) {
                r.1 += 1;
            }
        }
    }
    let rates: Vec<(&str, f64)> = seats
        .iter()
        .map(|(f, (n, w))| (*f, *w as f64 / (*n).max(1) as f64))
        .collect();
    for (f, r) in &rates {
        print!("{f} {:.1}%  ", r * 100.0);
    }
    let best = rates.iter().map(|x| x.1).fold(0.0, f64::max);
    let worst = rates.iter().map(|x| x.1).fold(1.0, f64::min);
    println!(
        "\nfaction win rate best/worst: {:.1}% / {:.1}% = {}",
        best * 100.0,
        worst * 100.0,
        if worst > 0.0 {
            format!("{:.2}x", best / worst)
        } else {
            "inf".into()
        }
    );
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

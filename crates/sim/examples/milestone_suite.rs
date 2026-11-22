//! Human milestone suite: the AI plays the user's seat from turn 1 on the setups of his own
//! games and is scored on how close it gets to what he did by the same turn (cities, rivals
//! eliminated, Gold, VP). The suite (`crates/sim/data/milestone-suite.json`,
//! built by `tools/ai-quality/milestones/extract.py`) is compiled in, so cloud runs need only
//! this binary and an arms file.
//!
//! milestone_suite [--ai-params ARMS.json] [--games SALTS] [--seed FIRST_SALT]
//!                 [--role train|holdout|all] [--maps ID,ID] [--out-dir DIR] [--suite FILE]
//!
//! ARMS.json is `{"arms": [{"label": "...", "params": <AiParams>}, ...]}` or one plain
//! AiParams file; without it the single arm is the built-in AI. The params apply to the
//! recorded human seat only; every rival plays the built-in AI at its recorded difficulty.
//! Each salt plays its own AI seeds and its own combat, movement and barbarian dice, so salts
//! are independent samples (docs/learnings.md, 2026-10-18). `--games`/`--seed` are read as the
//! salt count and first salt, so `tools/cloud/run-batch.sh --bin milestone_suite` splits salts
//! across servers. Writes DIR/milestones.json and prints one line per arm and map.
//!
//! Score of one game: the mean over the map's targets of min(AI value / human value, 1).
//! 1.0 means every milestone was met by its turn.

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{Event, GameSetup, GameState, PlayerId, scoring};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SUITE: &str = include_str!("../data/milestone-suite.json");

#[derive(Clone, Deserialize)]
struct Suite {
    maps: Vec<MapSpec>,
}

#[derive(Clone, Deserialize)]
struct MapSpec {
    id: String,
    role: String,
    horizon: u32,
    seat: u8,
    setup: GameSetup,
    targets: Vec<Target>,
}

#[derive(Clone, Deserialize)]
struct Target {
    metric: String,
    turn: u32,
    value: i64,
}

#[derive(Deserialize)]
struct Arm {
    label: String,
    params: AiParams,
}

fn salted(seed: u64, salt: u64) -> u64 {
    Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
}

/// The seat's numbers at the start of `state.turn`, matching the save telemetry's frames.
fn snapshot(
    state: &GameState,
    seat: PlayerId,
    captured: i64,
    lost: i64,
) -> BTreeMap<&'static str, i64> {
    let me = state.player(seat);
    let eliminated = state
        .players
        .iter()
        .filter(|p| p.id != seat && !p.id.is_barbarian() && !p.alive)
        .count();
    BTreeMap::from([
        ("cities", state.cities_of(seat).count() as i64),
        ("eliminated", eliminated as i64),
        ("vp", i64::from(scoring::vp_total(state, seat))),
        ("gold", i64::from(me.gold).max(0)),
        ("techs", me.techs.len() as i64),
        ("captured", captured),
        ("lost", lost),
    ])
}

fn play(map: &MapSpec, params: &AiParams, salt: u64) -> Value {
    let mut setup = map.setup.clone();
    let seat = PlayerId(map.seat);
    let seat_difficulty = legatum_core::AiDifficulty::Hard;
    setup.players[map.seat as usize].is_human = false;
    setup.players[map.seat as usize].difficulty = seat_difficulty;
    let mut state = legatum_core::new_game(&setup);
    state.setup.seed = salted(state.setup.seed, salt ^ 0xD1CE);
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let id = PlayerId(i as u8);
            let seed = salted(legatum_wasm::session::brain_seed(map.setup.seed, id), salt);
            let brain: Box<dyn Brain> = if id == seat {
                Box::new(ScriptedBrain::with_params(
                    player.faction,
                    seed,
                    id,
                    seat_difficulty,
                    params,
                ))
            } else if player.search {
                Box::new(legatum_ai::search::SearchBrain::new(
                    ScriptedBrain::with_difficulty(player.faction, seed, id, player.difficulty),
                    legatum_ai::search::SearchConfig::in_app(setup.players.len()),
                ))
            } else {
                Box::new(ScriptedBrain::with_difficulty(
                    player.faction,
                    seed,
                    id,
                    player.difficulty,
                ))
            };
            brain
        })
        .collect();
    let mut turns: Vec<u32> = map.targets.iter().map(|t| t.turn).collect();
    turns.sort_unstable();
    turns.dedup();
    let mut at: BTreeMap<u32, BTreeMap<&'static str, i64>> = BTreeMap::new();
    let (mut captured, mut lost) = (0i64, 0i64);
    let mut first_capture: Option<u32> = None;
    while !state.finished && state.turn <= map.horizon {
        if turns.contains(&state.turn) {
            at.insert(state.turn, snapshot(&state, seat, captured, lost));
        }
        if state.turn == map.horizon {
            break;
        }
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                (
                    p.id,
                    brains[p.id.0 as usize].orders(&legatum_core::player_view(&state, p.id)),
                )
            })
            .collect();
        let turn = state.turn;
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        state = next;
        for event in &events {
            if let Event::CityCaptured { from, to, .. } = event {
                if *to == seat {
                    captured += 1;
                    first_capture.get_or_insert(turn);
                } else if *from == seat {
                    lost += 1;
                }
            }
        }
    }
    // A game that ended early (an elimination win or loss) keeps its final numbers.
    let last = snapshot(&state, seat, captured, lost);
    for t in &turns {
        at.entry(*t).or_insert_with(|| last.clone());
    }
    let mut values = serde_json::Map::new();
    let mut sum = 0.0;
    let mut met = 0;
    for t in &map.targets {
        let v = at[&t.turn]
            .get(t.metric.as_str())
            .copied()
            .unwrap_or_else(|| panic!("{}: unknown milestone metric {}", map.id, t.metric));
        values.insert(format!("{}@{}", t.metric, t.turn), json!(v));
        sum += (v as f64 / t.value as f64).min(1.0);
        met += i32::from(v >= t.value);
    }
    json!({
        "salt": salt,
        "score": sum / map.targets.len() as f64,
        "met": met,
        "values": values,
        "captured": captured,
        "lost": lost,
        "first_capture": first_capture,
        "end_turn": state.turn,
        "seat_alive": state.player(seat).alive,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let suite: Suite = match flag("--suite") {
        Some(path) => serde_json::from_str(&std::fs::read_to_string(path).expect("read suite"))
            .expect("parse suite"),
        None => serde_json::from_str(SUITE).expect("compiled-in suite"),
    };
    let salts: u64 = flag("--games").map_or(16, |v| v.parse().expect("integer --games"));
    let first: u64 = flag("--seed").map_or(1, |v| v.parse().expect("integer --seed"));
    let role = flag("--role").unwrap_or_else(|| "all".into());
    let wanted: Option<Vec<String>> =
        flag("--maps").map(|m| m.split(',').map(str::to_string).collect());
    let maps: Vec<MapSpec> = suite
        .maps
        .into_iter()
        .filter(|m| role == "all" || m.role == role)
        .filter(|m| wanted.as_ref().is_none_or(|w| w.contains(&m.id)))
        .collect();
    assert!(!maps.is_empty(), "no suite maps match --role/--maps");
    let arms: Vec<Arm> = match flag("--ai-params") {
        None => vec![Arm {
            label: "builtin".into(),
            params: AiParams::builtin(),
        }],
        Some(path) => {
            let raw: Value =
                serde_json::from_str(&std::fs::read_to_string(&path).expect("read --ai-params"))
                    .expect("parse --ai-params");
            if raw.get("arms").is_some() {
                serde_json::from_value(raw["arms"].clone()).expect("arms: [{label, params}]")
            } else {
                let params: AiParams = serde_json::from_value(raw).expect("AiParams");
                vec![Arm {
                    label: params.name.clone(),
                    params,
                }]
            }
        }
    };
    for arm in &arms {
        arm.params
            .validate()
            .unwrap_or_else(|e| panic!("arm {}: {e}", arm.label));
    }
    // Longest games first, so the parallel pass ends evenly.
    let mut jobs: Vec<(usize, usize, u64)> = (0..arms.len())
        .flat_map(|a| (0..maps.len()).flat_map(move |m| (0..salts).map(move |s| (a, m, first + s))))
        .collect();
    let cost = |m: usize| u64::from(maps[m].horizon) * maps[m].setup.players.len() as u64;
    jobs.sort_by_key(|&(a, m, s)| (std::cmp::Reverse(cost(m)), a, s));
    let started = std::time::Instant::now();
    let results: Vec<((usize, usize), Value)> = jobs
        .par_iter()
        .map(|&(a, m, salt)| ((a, m), play(&maps[m], &arms[a].params, salt)))
        .collect();
    let mut grouped: BTreeMap<(usize, usize), Vec<Value>> = BTreeMap::new();
    for (key, game) in results {
        grouped.entry(key).or_default().push(game);
    }
    let mut report_arms = Vec::new();
    for (a, arm) in arms.iter().enumerate() {
        let mut report_maps = Vec::new();
        for (m, map) in maps.iter().enumerate() {
            let mut games = grouped.remove(&(a, m)).unwrap_or_default();
            games.sort_by_key(|g| g["salt"].as_u64());
            let n = games.len().max(1) as f64;
            let mean = games
                .iter()
                .map(|g| g["score"].as_f64().unwrap())
                .sum::<f64>()
                / n;
            let all_met = games
                .iter()
                .filter(|g| g["met"].as_u64() == Some(map.targets.len() as u64))
                .count();
            println!(
                "{:<24} {:<22} score {:.3}  all targets met {}/{}",
                arm.label,
                map.id,
                mean,
                all_met,
                games.len()
            );
            report_maps.push(json!({
                "id": map.id, "role": map.role, "mean_score": mean, "all_met": all_met,
                "games": games,
            }));
        }
        report_arms.push(json!({"label": arm.label, "maps": report_maps}));
    }
    let report = json!({
        "schema": 1,
        "salts": {"first": first, "count": salts},
        "wall_seconds": started.elapsed().as_secs_f64(),
        "arms": report_arms,
    });
    if let Some(dir) = flag("--out-dir") {
        std::fs::create_dir_all(&dir).expect("create --out-dir");
        let path = std::path::Path::new(&dir).join("milestones.json");
        std::fs::write(&path, serde_json::to_string(&report).unwrap()).expect("write report");
    } else {
        println!("{}", serde_json::to_string(&report).unwrap());
    }
}

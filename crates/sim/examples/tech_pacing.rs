//! Matched technology-pacing sweep for cloud runs.
//!
//!   tools/cloud/run-batch.sh --max-hours 1 --bin tech_pacing -- --games 256 --players 8 --mode hard
//!   mode is `hard` or `search`; each run covers Standard at current cost and
//!   experimental 100/80/65/50/40 percent. Search mode uses it on every seat.

use legatum_core::{AiDifficulty, Faction, GameSetup, data, scoring};
use legatum_sim::diversity::{self, Archetype, Collector};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

const ERAS: [&str; legatum_core::eras::ERA_COUNT] = legatum_core::eras::ERA_NAMES;
const PERCENTS: [Option<u8>; 6] = [None, Some(100), Some(80), Some(65), Some(50), Some(40)];

#[derive(Clone, Debug)]
struct Args {
    games: usize,
    seed: u64,
    players: usize,
    mode: String,
    label: String,
    out_dir: PathBuf,
}

fn args() -> Args {
    let mut a = Args {
        games: 256,
        seed: 9_100_001,
        players: 8,
        mode: "hard".into(),
        label: "tech-pacing".into(),
        out_dir: PathBuf::from("."),
    };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        let key = raw[i].as_str();
        if key == "--search" {
            a.mode = "search".into();
            i += 1;
            continue;
        }
        let value = raw
            .get(i + 1)
            .unwrap_or_else(|| panic!("{key} needs a value"));
        match key {
            "--games" => a.games = value.parse().expect("games"),
            "--seed" => a.seed = value.parse().expect("seed"),
            "--players" => a.players = value.parse().expect("players"),
            "--mode" => a.mode = value.clone(),
            "--label" => a.label = value.clone(),
            "--out-dir" => a.out_dir = value.into(),
            "--out" | "--status-file" => {}
            _ => {
                i += 1;
                continue;
            }
        }
        i += 2;
    }
    assert!(a.games > 0 && matches!(a.players, 4 | 8));
    assert!(matches!(a.mode.as_str(), "hard" | "search"));
    a
}

#[derive(Serialize)]
struct SeatResult {
    seat: u8,
    faction: String,
    winner: bool,
    techs_end: usize,
    era_techs_end: BTreeMap<String, usize>,
    entered_era_turn: BTreeMap<String, Option<u32>>,
    techs_by_turn: Vec<usize>,
    strategy: String,
    six_techs: bool,
}

fn play(players: usize, cost_percent: Option<u8>, search: bool, seed: u64) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    let mut factions = Faction::ALL;
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (p, faction) in setup.players.iter_mut().zip(factions) {
        p.faction = faction;
        p.difficulty = AiDifficulty::Hard;
        p.search = search;
    }
    setup.rule_experiments.tech_cost_percent = cost_percent;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut classifier = Collector::new();
    let mut techs_by_turn = vec![Vec::<usize>::new(); players];
    let mut entered = vec![BTreeMap::<String, u32>::new(); players];
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut turns = 0;
    while !session.state.finished && turns < cap {
        let resolved_turn = session.state.turn;
        let orders = session.ai_orders();
        let events = session.step(&orders);
        for event in &events {
            classifier.observe_event(resolved_turn, event);
        }
        classifier.observe_resolved_turn(resolved_turn, &session.state);
        turns += 1;
        for (seat, player) in session.state.players.iter().enumerate() {
            techs_by_turn[seat].push(player.techs.len());
            for tech in &player.techs {
                entered[seat]
                    .entry(data::tech_era(*tech).to_owned())
                    .or_insert(resolved_turn);
            }
        }
    }
    let state = &session.state;
    let winner_team = state.winner.map(|team| team.0);
    let seats: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            let winner = winner_team == Some(p.team.0);
            let mut era_counts = BTreeMap::new();
            let mut first_era = BTreeMap::new();
            for era in ERAS {
                era_counts.insert(
                    era.to_owned(),
                    p.techs
                        .iter()
                        .filter(|t| data::tech_era(**t) == era)
                        .count(),
                );
                first_era.insert(era.to_owned(), entered[p.id.0 as usize].get(era).copied());
            }
            let archetype = classifier
                .telemetry
                .empires
                .get(&p.id.0)
                .map(diversity::classify)
                .unwrap_or(Archetype::Generalist);
            SeatResult {
                seat: p.id.0,
                faction: format!("{:?}", p.faction),
                winner,
                techs_end: p.techs.len(),
                era_techs_end: era_counts,
                entered_era_turn: first_era,
                techs_by_turn: techs_by_turn[p.id.0 as usize].clone(),
                strategy: archetype.name().to_owned(),
                six_techs: p.techs.len() >= 6,
            }
        })
        .collect();
    let winner_six = seats.iter().any(|s| s.winner && s.six_techs);
    json!({
        "seed": seed.to_string(), "turns": turns, "finished": state.finished,
        "winner_team": winner_team,
        "winner_six_techs": winner_six, "seats": seats,
        "vp": state.players.iter().map(|p| (p.id.0.to_string(), scoring::vp_total(state, p.id))).collect::<BTreeMap<_,_>>(),
    })
}

fn main() {
    let a = args();
    std::fs::create_dir_all(&a.out_dir).expect("output directory");
    let mut cohorts = serde_json::Map::new();
    for percent in PERCENTS {
        let games: Vec<_> = (0..a.games)
            .into_par_iter()
            .map(|i| {
                play(
                    a.players,
                    percent,
                    a.mode == "search",
                    a.seed.wrapping_add(i as u64),
                )
            })
            .collect();
        let key = format!(
            "{}-{}",
            "standard",
            percent.map_or_else(|| "current".into(), |p| p.to_string())
        );
        eprintln!(
            "completed {} {} standard {}%",
            a.players,
            a.mode,
            percent.map_or(0, u8::from)
        );
        cohorts.insert(key, json!({ "games": games }));
    }
    let output = json!({
        "meta": {"players": a.players, "mode": a.mode, "games_per_cohort": a.games,
            "first_seed": a.seed.to_string(), "label": a.label,
            "search_config": if a.mode == "search" { "application SearchConfig::in_app(players), every seat" } else { "Scripted Hard, every seat" },
            "tech_cost_percent_values": ["current", 100, 80, 65, 50, 40],
            "strategy_classifier": diversity::CLASSIFIER_VERSION},
        "cohorts": cohorts
    });
    let path = a
        .out_dir
        .join(format!("tech-pacing-{}-{}.json", a.players, a.mode));
    std::fs::write(&path, serde_json::to_vec(&output).expect("serialize")).expect("write output");
    eprintln!("wrote {}", path.display());
}

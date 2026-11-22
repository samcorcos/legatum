//! Climb arena: the full-game objective of the AI climb program
//! (`docs/sim-results/ai-quality/climb-program/README.md`). One faction's Hard AI plays generated
//! 4-player Small games against a field of Hard AIs; every arm plays the same games.
//!
//! climb_arena --faction F [--ai-params ARMS.json] [--field FIELD.json] [--games N] [--seed S]
//!             [--players 4] [--width 40 --height 26] [--out-dir DIR] [--label L]
//!
//! ARMS.json is `{"arms": [{"label": "...", "params": <AiParams>, "mode": "seat"|"opponents"|"all", "games": N}, ...]}` or
//! one plain AiParams file (one arm, "seat"); without it the single arm is the built-in AI.
//! "seat": the arm's params play the faction's seat and every rival plays FIELD (the built-in AI
//! without `--field`). "all": every seat plays the arm's params (the league: a file holding each
//! faction's best as `faction_overrides`). Game `i` uses seed `S + i`: the faction sits in seat
//! `seed % players` and its rivals are drawn from the other factions with that seed, so arms are paired
//! game for game and seeds are independent samples (own map, AI seeds and dice). `--games`/`--seed`
//! /`--label`/`--out-dir` follow `tools/cloud/run-batch.sh --bin`, which splits seeds across servers.
//! Writes DIR/<label>-games.json per arm (compact per-game rows) and prints one line per arm.

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, PlayerId, scoring};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

const CHECKPOINTS: [u32; 4] = [50, 100, 150, 200];
const SAFETY_TURNS: u32 = 700;

#[derive(Deserialize)]
struct Arm {
    label: String,
    params: AiParams,
    #[serde(default)]
    mode: Option<String>,
    games: Option<u64>,
}

#[derive(Deserialize)]
struct ArmsFile {
    arms: Vec<Arm>,
}

struct Args {
    faction: Faction,
    arms: Vec<(String, AiParams, Mode, u64)>,
    field: AiParams,
    seed: u64,
    players: usize,
    width: i32,
    height: i32,
    out_dir: PathBuf,
}

#[derive(Clone, Copy)]
enum Mode {
    Seat,
    Opponents,
    All,
}

fn load(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn checked(p: AiParams, what: &str) -> AiParams {
    p.validate().unwrap_or_else(|e| panic!("{what}: {e}"));
    p
}

fn parse_args() -> Args {
    let mut it = std::env::args().skip(1);
    let (mut faction, mut arms_file, mut field_file) = (None, None, None);
    let (mut games, mut seed, mut players, mut width, mut height) = (64u64, 1u64, 4usize, 40, 26);
    let mut out_dir = PathBuf::from(".");
    while let Some(a) = it.next() {
        let mut v = || it.next().unwrap_or_else(|| panic!("{a} needs a value"));
        match a.as_str() {
            "--faction" => {
                let name = v();
                faction = Some(
                    Faction::ALL
                        .into_iter()
                        .find(|f| format!("{f:?}").eq_ignore_ascii_case(&name))
                        .unwrap_or_else(|| panic!("unknown faction {name}")),
                );
            }
            "--ai-params" => arms_file = Some(v()),
            "--field" => field_file = Some(v()),
            "--games" => games = v().parse().expect("--games"),
            "--seed" => seed = v().parse().expect("--seed"),
            "--players" => players = v().parse().expect("--players"),
            "--width" => width = v().parse().expect("--width"),
            "--height" => height = v().parse().expect("--height"),
            "--out-dir" => out_dir = PathBuf::from(v()),
            // Passed by run-batch.sh; this binary has no use for them.
            "--label" | "--out" | "--status-file" => {
                v();
            }
            other => panic!("unknown argument {other}"),
        }
    }
    let faction = faction.expect("--faction is required");
    assert!((2..=8).contains(&players), "--players must be 2..=8");
    let arms = match arms_file {
        None => vec![(
            "builtin".to_string(),
            AiParams::builtin(),
            Mode::Seat,
            games,
        )],
        Some(path) => {
            let text = load(&path);
            match serde_json::from_str::<ArmsFile>(&text) {
                Ok(f) => f
                    .arms
                    .into_iter()
                    .map(|a| {
                        let mode = match a.mode.as_deref() {
                            None | Some("seat") => Mode::Seat,
                            Some("opponents") => Mode::Opponents,
                            Some("all") => Mode::All,
                            Some(m) => panic!("arm {}: unknown mode {m}", a.label),
                        };
                        let p = checked(a.params, &a.label);
                        (a.label, p, mode, a.games.unwrap_or(games))
                    })
                    .collect(),
                Err(_) => {
                    let p: AiParams = serde_json::from_str(&text).unwrap_or_else(|e| {
                        panic!("{path}: neither an arms file nor AiParams: {e}")
                    });
                    let p = checked(p, &path);
                    vec![(p.name.clone(), p, Mode::Seat, games)]
                }
            }
        }
    };
    let field = match field_file {
        None => AiParams::builtin(),
        Some(path) => checked(
            serde_json::from_str(&load(&path)).unwrap_or_else(|e| panic!("{path}: {e}")),
            &path,
        ),
    };
    Args {
        faction,
        arms,
        field,
        seed,
        players,
        width,
        height,
        out_dir,
    }
}

/// The game's lobby: the faction in seat `seed % players`, rivals drawn from the others.
fn setup(args: &Args, game_seed: u64, mode: Mode) -> (GameSetup, usize) {
    let mut setup = GameSetup::all_ai(game_seed);
    setup.players.truncate(args.players);
    (setup.width, setup.height) = (args.width, args.height);
    let mut others: Vec<Faction> = Faction::ALL
        .into_iter()
        .filter(|f| *f != args.faction)
        .collect();
    let mut rng = Rng::derive(game_seed, &[purpose::SETUP, 0xC11B]);
    for i in (1..others.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        others.swap(i, j);
    }
    // By seed, not by index, so a run split across servers seats every game the same way.
    let seat = (game_seed % args.players as u64) as usize;
    let mut rivals = others.into_iter();
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = if i == seat {
            args.faction
        } else {
            rivals.next().expect("enough rival factions")
        };
        p.difficulty = AiDifficulty::Hard;
        p.is_human = false;
        p.search = false;
    }
    if matches!(mode, Mode::Opponents) {
        // Keep the same map, dice and faction field. The observed seat is a master-played
        // rival; the climbed faction moves one seat clockwise and is the only tuned rival.
        let rival = (seat + 1) % args.players;
        let (observed, climbed) = (setup.players[seat].faction, setup.players[rival].faction);
        setup.players[seat].faction = climbed;
        setup.players[rival].faction = observed;
    }
    (setup, seat)
}

fn play(args: &Args, params: &AiParams, mode: Mode, index: u64) -> Value {
    let game_seed = args.seed + index;
    let (setup, seat_index) = setup(args, game_seed, mode);
    let seat = PlayerId(seat_index as u8);
    let mut state = legatum_core::new_game(&setup);
    let master = AiParams::builtin();
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let id = PlayerId(i as u8);
            let seed = legatum_wasm::session::brain_seed(game_seed, id);
            let use_arm = match mode {
                Mode::Seat => id == seat,
                Mode::Opponents => p.faction == args.faction,
                Mode::All => true,
            };
            let brain: Box<dyn Brain> = Box::new(ScriptedBrain::with_params(
                p.faction,
                seed,
                id,
                AiDifficulty::Hard,
                if use_arm {
                    params
                } else if matches!(mode, Mode::Opponents) && id == seat {
                    &master
                } else {
                    &args.field
                },
            ));
            brain
        })
        .collect();
    let mut cities = BTreeMap::new();
    let mut vp = BTreeMap::new();
    let (mut captured, mut lost) = (0u32, 0u32);
    let mut war_declarations = 0u32;
    let (mut first_war, mut first_capture) = (None::<u32>, None::<u32>);
    let my_team = state.player(seat).team;
    while !state.finished && state.turn <= SAFETY_TURNS {
        if CHECKPOINTS.contains(&state.turn) {
            cities.insert(state.turn, state.cities_of(seat).count() as u32);
            vp.insert(state.turn, scoring::vp_total(&state, seat));
        }
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let view = legatum_core::player_view(&state, p.id);
                (p.id, brains[p.id.0 as usize].orders(&view))
            })
            .collect();
        let turn = state.turn;
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        state = next;
        for event in &events {
            match event {
                Event::WarDeclared { relation }
                    if relation.a == my_team || relation.b == my_team =>
                {
                    first_war.get_or_insert(turn);
                    if relation.declarers.contains(&my_team) {
                        war_declarations += 1;
                    }
                }
                Event::CityCaptured { to, .. } if *to == seat => {
                    captured += 1;
                    first_capture.get_or_insert(turn);
                }
                Event::CityCaptured { from, .. } if *from == seat => lost += 1,
                _ => {}
            }
        }
    }
    let won = state.finished && state.winner == Some(my_team);
    let my_vp = scoring::vp_total(&state, seat);
    let rival_vp = state
        .players
        .iter()
        .filter(|p| p.id != seat && !p.id.is_barbarian())
        .map(|p| scoring::vp_total(&state, p.id))
        .max()
        .unwrap_or(0);
    let eliminated = state
        .players
        .iter()
        .filter(|p| p.id != seat && !p.id.is_barbarian() && !p.alive)
        .count();
    let winners: Vec<String> = state
        .players
        .iter()
        .filter(|p| Some(p.team) == state.winner)
        .map(|p| format!("{:?}", p.faction))
        .collect();
    json!({
        "seed": game_seed,
        "seat": seat_index,
        "factions": setup.players.iter().map(|p| format!("{:?}", p.faction)).collect::<Vec<_>>(),
        "won": won,
        "winners": winners,
        "finished": state.finished,
        "end_turn": state.turn,
        "alive": state.player(seat).alive,
        "vp": my_vp,
        "rival_vp": rival_vp,
        "cities_end": state.cities_of(seat).count(),
        "cities": cities,
        "vp_at": vp,
        "captured": captured,
        "war_declarations": war_declarations,
        "lost": lost,
        "eliminated": eliminated,
        "first_war": first_war,
        "first_capture": first_capture,
    })
}

fn main() {
    let args = parse_args();
    std::fs::create_dir_all(&args.out_dir).expect("create --out-dir");
    let jobs: Vec<(usize, u64)> = (0..args.arms.len())
        .flat_map(|a| (0..args.arms[a].3).map(move |i| (a, i)))
        .collect();
    let rows: Vec<(usize, Value)> = jobs
        .par_iter()
        .map(|&(a, i)| {
            let (_, params, mode, _) = &args.arms[a];
            (a, play(&args, params, *mode, i))
        })
        .collect();
    for (a, (label, _, mode, _)) in args.arms.iter().enumerate() {
        let games: Vec<&Value> = rows.iter().filter(|r| r.0 == a).map(|r| &r.1).collect();
        let wins = games.iter().filter(|g| g["won"] == json!(true)).count();
        let path = args.out_dir.join(format!("{label}-games.json"));
        let out = json!({
            "faction": format!("{:?}", args.faction),
            "label": label,
            "mode": match mode { Mode::Seat => "seat", Mode::Opponents => "opponents", Mode::All => "all" },
            "players": args.players,
            "first_seed": args.seed,
            "games": games,
        });
        std::fs::write(&path, serde_json::to_string(&out).expect("serialize"))
            .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        println!(
            "{label}: {:?} won {wins}/{} ({:.1}%)",
            args.faction,
            games.len(),
            100.0 * wins as f64 / games.len().max(1) as f64
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opponents_mode_keeps_seeded_lobby_but_moves_climbed_faction_to_rival() {
        let args = Args {
            faction: Faction::Mongols,
            arms: Vec::new(),
            field: AiParams::builtin(),
            seed: 17,
            players: 4,
            width: 40,
            height: 26,
            out_dir: PathBuf::new(),
        };
        let (seat_game, seat) = setup(&args, 17, Mode::Seat);
        let (opponents_game, observed) = setup(&args, 17, Mode::Opponents);
        assert_eq!(seat, observed);
        assert_eq!(seat_game.players[seat].faction, Faction::Mongols);
        assert_ne!(opponents_game.players[seat].faction, Faction::Mongols);
        assert_eq!(
            opponents_game.players[(seat + 1) % 4].faction,
            Faction::Mongols
        );
        assert_eq!(seat_game.seed, opponents_game.seed);
    }
}

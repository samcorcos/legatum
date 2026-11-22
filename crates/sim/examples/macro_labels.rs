//! Offline full-game-relevant macro labels. Each row is one view-only decision group; options
//! share determinizations, and values are the mean win chance after 40 or 80 Standard turns.
//! Cloud: LEGATUM_CLOUD_EXAMPLES=macro_labels tools/cloud/build-linux.sh
//!   tools/cloud/run-batch.sh --max-hours 2 --bin macro_labels -- --games 24 --seed 7300001 --label macro-train
use legatum_ai::learned_macro;
use legatum_ai::search::{SearchBrain, SearchConfig};
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, GameSetup, PlayerId, new_game, player_view, resolve_turn};
use legatum_wasm::session::brain_seed;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const MAPS: &str = include_str!("../data/hard-vs-human-maps.json");

#[derive(Deserialize)]
struct Map {
    id: String,
    setup: GameSetup,
    human_seat: u8,
}

#[derive(Serialize)]
struct OptionRow {
    features: [i32; learned_macro::FEATURES],
    head: usize,
    value: i32,
    first_two: i32,
    plan: String,
}

#[derive(Serialize)]
struct Row {
    source: String,
    seed: u64,
    turn: u32,
    seat: u8,
    faction: String,
    players: usize,
    horizon: u32,
    samples: u8,
    options: Vec<OptionRow>,
    chosen: usize,
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn collect(
    setup: &GameSetup,
    source: &str,
    target_seat: Option<u8>,
    sample_seed: u64,
    samples: u8,
    writer: &mut impl Write,
) {
    let mut state = new_game(setup);
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            let seed = Rng::derive(
                brain_seed(setup.seed, p.id),
                &[purpose::AI, 0x5a17, sample_seed],
            )
            .next_u64();
            ScriptedBrain::with_difficulty(p.faction, seed, p.id, AiDifficulty::Hard)
        })
        .collect();
    let turns = [1, 20, 40, 60, 90];
    while !state.finished && state.turn <= 90 {
        if turns.contains(&state.turn) {
            let seats: Vec<_> = target_seat.map_or_else(
                || state.players.iter().map(|p| p.id).collect(),
                |id| vec![PlayerId(id)],
            );
            for seat in seats.into_iter().filter(|seat| state.player(*seat).alive) {
                let view = player_view(&state, seat);
                let opening = state.turn == 1;
                let horizon = if state.turn.is_multiple_of(40) {
                    80
                } else {
                    40
                };
                let config = SearchConfig {
                    candidates: 8,
                    horizon,
                    opening_horizon: horizon,
                    samples,
                    openings: opening,
                    fronts: true,
                    fitted_plan: true,
                    ..SearchConfig::default()
                };
                // Preserve the simulated seat's real opponent and campaign memory. Review
                // the copied plan at this sampled position without changing the real game.
                let base = brains[seat.0 as usize].offline_review_copy(state.turn);
                let mut oracle = SearchBrain::new(base, config);
                oracle.orders(&view);
                if let Some(event) = oracle.search_events().first() {
                    let options: Vec<_> = event
                        .options
                        .iter()
                        .map(|o| OptionRow {
                            features: learned_macro::features(&view, &o.kind, o.opening),
                            head: learned_macro::head(&o.kind, o.opening),
                            value: o.mean + o.beyond,
                            first_two: o.samples.iter().take(2).sum::<i32>() / 2 + o.beyond,
                            plan: format!("{:?}/{:?}", o.kind, o.opening),
                        })
                        .collect();
                    if options.len() > 1 {
                        let row = Row {
                            source: source.into(),
                            seed: sample_seed,
                            turn: state.turn,
                            seat: seat.0,
                            faction: format!("{:?}", state.player(seat).faction),
                            players: state.players.len(),
                            horizon,
                            samples,
                            options,
                            chosen: event.chosen,
                        };
                        serde_json::to_writer(&mut *writer, &row).expect("write label");
                        writer.write_all(b"\n").expect("newline");
                    }
                }
            }
        }
        let orders = brains
            .iter_mut()
            .filter(|b| state.player(b.player).alive)
            .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
            .collect();
        state = resolve_turn(&state, &orders).0;
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: u64 = arg(&args, "--games")
        .unwrap_or_else(|| "1".into())
        .parse()
        .expect("games");
    let seed: u64 = arg(&args, "--seed")
        .unwrap_or_else(|| "7300001".into())
        .parse()
        .expect("seed");
    let samples: u8 = arg(&args, "--samples")
        .unwrap_or_else(|| "64".into())
        .parse()
        .expect("samples");
    assert!(samples >= 64, "oracle needs at least 64 samples");
    let out = PathBuf::from(arg(&args, "--out-dir").unwrap_or_else(|| ".".into()))
        .join("macro-labels.jsonl");
    let mut writer = BufWriter::new(File::create(out).expect("output"));
    let maps: Vec<Map> = serde_json::from_str(MAPS).expect("benchmark maps");
    for i in 0..games {
        let run_seed = seed + i;
        if i % 4 == 0 {
            let map = &maps[(run_seed as usize / 4) % maps.len()];
            let mut setup = map.setup.clone();
            for player in &mut setup.players {
                player.is_human = false;
                player.difficulty = AiDifficulty::Hard;
            }
            collect(
                &setup,
                &format!("benchmark:{}", map.id),
                Some(map.human_seat),
                run_seed,
                samples,
                &mut writer,
            );
        } else {
            let mut setup = GameSetup::all_ai(run_seed);
            if i % 2 == 0 {
                setup.players.truncate(4);
                setup.width = 24;
                setup.height = 16;
            }
            for player in &mut setup.players {
                player.difficulty = AiDifficulty::Hard;
            }
            collect(&setup, "simulation", None, run_seed, samples, &mut writer);
        }
        writer.flush().expect("flush labels");
        eprintln!("labelled game {}/{}", i + 1, games);
    }
}

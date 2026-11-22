//! Paired fresh-map validation for the opening-only distilled macro gene.
//!
//! Cloud: LEGATUM_CLOUD_EXAMPLES=distilled_fresh_maps tools/cloud/build-linux.sh
//!   tools/cloud/run-batch.sh --max-hours 1 --bin distilled_fresh_maps \
//!     --ai-params docs/sim-results/distilled-macro/opening-only.json -- \
//!     --players 4 --games 1024 --seed 18446744072709550000
use legatum_ai::search::{SearchBrain, SearchConfig};
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, PlayerId, new_game, player_view, resolve_turn, scoring,
};
use legatum_wasm::session::brain_seed;
use rayon::prelude::*;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize)]
struct SeatResult {
    seat: u8,
    faction: Faction,
    gene_on: bool,
    won: bool,
    cities_t50: Option<usize>,
    vp_t100: Option<u32>,
}

#[derive(Serialize)]
struct GameResult {
    pair: u64,
    seed: u64,
    players: usize,
    mirrored: bool,
    finished: bool,
    turns_resolved: u32,
    seats: Vec<SeatResult>,
}

fn value(args: &[String], key: &str, fallback: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == key)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| fallback.into())
}

fn factions(pair: u64, players: usize) -> Vec<Faction> {
    let all = Faction::ALL;
    if players == 8 {
        let offset = (pair as usize) % all.len();
        return (0..players)
            .map(|seat| all[(seat + offset) % all.len()])
            .collect();
    }
    // Every faction appears in exactly half of the four-player map seeds. Across the paired
    // orientations, every appearance is evaluated once with and once without the gene.
    let offset = (pair as usize) % all.len();
    (0..players)
        .map(|seat| all[(offset + seat) % all.len()])
        .collect()
}

fn run_game(
    pair: u64,
    seed: u64,
    players: usize,
    mirrored: bool,
    gene_off: &'static AiParams,
    gene_on: &'static AiParams,
) -> GameResult {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    (setup.width, setup.height) = (48, 32);
    let faction_order = factions(pair, players);
    for (seat, player) in setup.players.iter_mut().enumerate() {
        player.difficulty = AiDifficulty::Hard;
        player.faction = faction_order[seat];
    }

    let policy_index = pair as usize;
    let mut genes_on = (0..players)
        .map(|seat| (seat + policy_index).is_multiple_of(2))
        .collect::<Vec<_>>();
    if mirrored {
        genes_on.reverse();
    }
    let state0 = new_game(&setup);
    let mut brains: Vec<Box<dyn Brain>> = state0
        .players
        .iter()
        .map(|player| {
            let seed = brain_seed(setup.seed, player.id);
            let params = if genes_on[player.id.0 as usize] {
                gene_on
            } else {
                gene_off
            };
            let hard = ScriptedBrain::with_params(
                player.faction,
                seed,
                player.id,
                AiDifficulty::Hard,
                params,
            );
            Box::new(SearchBrain::new(hard, SearchConfig::in_app(players))) as Box<dyn Brain>
        })
        .collect();

    let mut state = state0;
    let mut cities_t50 = None;
    let mut vp_t100 = None;
    while !state.finished && state.turn <= 1000 {
        let orders = brains
            .iter_mut()
            .enumerate()
            .filter(|(seat, _)| state.player(PlayerId(*seat as u8)).alive)
            .map(|(seat, brain)| {
                let player = PlayerId(seat as u8);
                (player, brain.orders(&player_view(&state, player)))
            })
            .collect();
        state = resolve_turn(&state, &orders).0;
        if state.turn == 51 {
            cities_t50 = Some(
                state
                    .players
                    .iter()
                    .map(|player| {
                        state
                            .cities
                            .values()
                            .filter(|city| city.owner == player.id)
                            .count()
                    })
                    .collect::<Vec<_>>(),
            );
        }
        if state.turn == 101 {
            vp_t100 = Some(
                state
                    .players
                    .iter()
                    .map(|player| scoring::vp_total(&state, player.id))
                    .collect::<Vec<_>>(),
            );
        }
    }

    let seats = state
        .players
        .iter()
        .map(|player| SeatResult {
            seat: player.id.0,
            faction: player.faction,
            gene_on: genes_on[player.id.0 as usize],
            won: state.winner == Some(player.team),
            cities_t50: cities_t50
                .as_ref()
                .map(|values| values[player.id.0 as usize]),
            vp_t100: vp_t100.as_ref().map(|values| values[player.id.0 as usize]),
        })
        .collect();
    GameResult {
        pair,
        seed,
        players,
        mirrored,
        finished: state.finished,
        turns_resolved: state.turn.saturating_sub(1),
        seats,
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let players = value(&args, "--players", "8")
        .parse::<usize>()
        .expect("players");
    assert!(matches!(players, 4 | 8), "--players must be 4 or 8");
    let games = value(&args, "--games", "1024")
        .parse::<usize>()
        .expect("games");
    assert!(
        games > 0 && games.is_multiple_of(2),
        "--games must be a positive even count"
    );
    let first_seed = value(&args, "--seed", "9000000000000000000")
        .parse::<u64>()
        .expect("seed");
    let pairs = args
        .windows(2)
        .find(|pair| pair[0] == "--pairs")
        .map(|pair| {
            pair[1]
                .split(',')
                .map(|value| value.parse::<u64>().expect("pair index"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| (0..(games / 2) as u64).collect());
    let params_path = value(
        &args,
        "--ai-params",
        "docs/sim-results/distilled-macro/opening-only.json",
    );
    let candidate: AiParams =
        serde_json::from_slice(&std::fs::read(params_path).expect("read --ai-params candidate"))
            .expect("parse candidate params");
    candidate.validate().expect("validate candidate params");
    let gene_on = Box::leak(Box::new(candidate));
    let mut off = (*gene_on).clone();
    off.decisions.learned_macro = 0;
    let gene_off = Box::leak(Box::new(off));
    let output = PathBuf::from(value(&args, "--out-dir", ".")).join("distilled-fresh-maps.json");
    let finished = AtomicUsize::new(0);
    let indexes = (0..pairs.len() * 2).collect::<Vec<_>>();
    let total = indexes.len();
    let results = indexes
        .into_par_iter()
        .map(|index| {
            let pair = pairs[index / 2];
            let seed = first_seed + pair;
            let result = run_game(pair, seed, players, index % 2 == 1, gene_off, gene_on);
            let count = finished.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 || count.is_multiple_of(32) || count == total {
                eprintln!(
                    "finished game {count}/{} (seed {seed}, {players} players)",
                    total
                );
            }
            result
        })
        .collect::<Vec<_>>();
    std::fs::write(
        output,
        serde_json::to_vec(&results).expect("serialize results"),
    )
    .expect("write results");
    eprintln!(
        "completed {games} mirrored games on {} fresh Standard maps ({players} players)",
        games / 2
    );
}

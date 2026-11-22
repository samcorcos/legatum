//! Eight-player Standard Hard games: how many AI seats are broke and empty-handed at T80, and
//! does the T80 VP leader win? (Contest-leader follow-up: the armies were too small because the
//! economies were.)
//!
//! economy_t80 <games> [first_seed=9600001] [params.json]
//!
//! Eight empires, Medium map, solo teams, all Hard, no turn limit (safety cap 4x the expected
//! length). Prints one JSON line: the share of seats with 0 Gold and 0 units at the end of T80,
//! mean cities at T50 and T80, the share of games whose strict T80 VP leader (at least 1 VP,
//! no tie) wins, and the share of games that reach the cap.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::scoring::team_vp;
use legatum_core::{AiDifficulty, Faction, GameSetup, PlayerId, TeamId, player_view, resolve_turn};
use rayon::prelude::*;
use serde_json::json;

struct Out {
    broke: u32,
    seats: u32,
    cities50: u32,
    cities80: u32,
    leader_games: u32,
    leader_won: u32,
    capped: u32,
}

fn play(seed: u64, params: &'static AiParams) -> Out {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(8);
    (setup.width, setup.height) = (48, 32);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (seat, (p, f)) in setup.players.iter_mut().zip(factions).enumerate() {
        p.faction = f;
        p.team = TeamId(seat as u8);
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<(PlayerId, Box<dyn Brain>)> = (0..8u8)
        .map(|i| {
            let id = PlayerId(i);
            let b: Box<dyn Brain> = Box::new(ScriptedBrain::with_params(
                state.player(id).faction,
                legatum_wasm::session::brain_seed(setup.seed, id),
                id,
                AiDifficulty::Hard,
                params,
            ));
            (id, b)
        })
        .collect();
    let mut o = Out {
        broke: 0,
        seats: 0,
        cities50: 0,
        cities80: 0,
        leader_games: 0,
        leader_won: 0,
        capped: 0,
    };
    let mut leader: Option<TeamId> = None;
    while !state.finished && state.turn < cap {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
            }
        }
        let turn = state.turn;
        let (next, _) = resolve_turn(&state, &orders);
        state = next;
        if turn == 50 {
            o.cities50 = state.cities.len() as u32;
        }
        if turn == 80 {
            o.cities80 = state.cities.len() as u32;
            for p in state.players.iter().filter(|p| !p.id.is_barbarian()) {
                o.seats += 1;
                if p.gold <= 0 && state.units_of(p.id).count() == 0 {
                    o.broke += 1;
                }
            }
            let mut vps: Vec<(u32, TeamId)> = state
                .teams()
                .into_iter()
                .filter(|t| *t != TeamId::BARBARIAN)
                .map(|t| (team_vp(&state, t), t))
                .collect();
            vps.sort_by_key(|a| std::cmp::Reverse(a.0));
            if vps[0].0 >= 1 && (vps.len() < 2 || vps[0].0 > vps[1].0) {
                leader = Some(vps[0].1);
            }
        }
    }
    if state.turn >= cap && !state.finished {
        o.capped = 1;
    }
    if let Some(l) = leader {
        o.leader_games = 1;
        o.leader_won = u32::from(state.winner == Some(l));
    }
    o
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let first: u64 = a.get(2).map_or(9_600_001, |s| s.parse().unwrap());
    let params: &'static AiParams =
        Box::leak(Box::new(match a.get(3).filter(|s| s.ends_with(".json")) {
            Some(path) => serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
            None => AiParams::builtin(),
        }));
    let outs: Vec<Out> = (0..games)
        .into_par_iter()
        .map(|i| play(first + i, params))
        .collect();
    let sum = |f: fn(&Out) -> u32| outs.iter().map(|o| f(o) as f64).sum::<f64>();
    println!(
        "{}",
        json!({"games": games,
            "broke_empty_share_t80": sum(|o| o.broke) / sum(|o| o.seats),
            "mean_cities_t50_all_seats": sum(|o| o.cities50) / games as f64 / 8.0,
            "mean_cities_t80_all_seats": sum(|o| o.cities80) / games as f64 / 8.0,
            "t80_leader_games": sum(|o| o.leader_games),
            "t80_leader_win_rate": sum(|o| o.leader_won) / sum(|o| o.leader_games),
            "capped_share": sum(|o| o.capped) / games as f64})
    );
}

//! Decision regret of the plan estimate (AI war plan step 3, gene `learned_value`): which plan
//! does each estimate pick, and how often does that pick go on to win?
//!
//! Plays Hard self-play games (plain Hard on every seat, as the app). At each `--turns` sample,
//! for one alive seat, it asks the seat's planner for its plan candidates three times: with the
//! hand formula (`learned_value` 0), the learned network (1) and the refitted linear weights (2).
//! An estimate's pick is its top-valued candidate (the planner's own hysteresis and review
//! timing are left out). Where the picks differ, each picked plan is forced for `--horizon`
//! turns from the TRUE state (an offline evaluation; every brain still decides from its own
//! view) and the game is played to the end, `--samples` times with reseeded dice, the same dice
//! for every option. Regret differences need no oracle: regret(a) - regret(b) is the win rate of
//! b's pick minus a's, and it is zero wherever they agree.
//!
//!   value_regret --games 64 --seed 8880001 [--players 8,8,8,4] [--turns 40,80,120,160]
//!                [--options 4] [--samples 16] [--horizon 20] [--out-dir D] [--label L]
//!
//! Cloud: LEGATUM_CLOUD_EXAMPLES=value_regret tools/cloud/build-linux.sh, then
//!   tools/cloud/run-batch.sh --bin value_regret --count 2 --max-hours 1 -- --games 128 --seed 8880001
use legatum_ai::plan::PlanKind;
use legatum_ai::search::OracleNode;
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, GameState, PlayerId, new_game, player_view, resolve_turn,
};
use legatum_wasm::session::brain_seed;
use rayon::prelude::*;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const MODES: [i32; 3] = [0, 1, 2];

fn arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn list<T: std::str::FromStr>(v: &str) -> Vec<T>
where
    T::Err: std::fmt::Debug,
{
    v.split(',')
        .map(|x| x.parse().expect("list value"))
        .collect()
}

struct Opts {
    turns: Vec<u32>,
    options: usize,
    samples: u64,
    horizon: u32,
}

/// The live brains, with `seat` reviewing its plan now under estimate `mode`.
fn with_mode(brains: &[ScriptedBrain], seat: PlayerId, mode: i32, turn: u32) -> Vec<ScriptedBrain> {
    brains
        .iter()
        .map(|b| {
            if b.player == seat {
                let mut c = b.offline_review_copy(turn);
                c.decisions.learned_value = mode;
                c
            } else {
                b.rollout_copy()
            }
        })
        .collect()
}

/// Wins of `seat` out of `samples` when `plan` is forced from `state`.
fn wins(
    state: &GameState,
    brains: &[ScriptedBrain],
    seat: PlayerId,
    plan: &PlanKind,
    o: &Opts,
) -> u32 {
    (0..o.samples)
        .into_par_iter()
        .map(|s| {
            let mut st = state.clone();
            st.setup.seed =
                legatum_core::rng::Rng::derive(state.setup.seed, &[0x7e61, state.turn as u64, s])
                    .next_u64();
            let mut node = OracleNode::from_recorded_state(&st, brains, seat);
            node.advance(plan, o.horizon);
            node.continue_opening(1000);
            u32::from(node.metrics().won)
        })
        .sum()
}

fn play(index: usize, seed: u64, players: usize, o: &Opts) -> Vec<serde_json::Value> {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    (setup.width, setup.height) = (48, 32);
    let mut factions = Faction::ALL;
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (player, faction) in setup.players.iter_mut().zip(factions) {
        player.faction = faction;
        player.difficulty = AiDifficulty::Hard;
    }
    let mut state = new_game(&setup);
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(
                p.faction,
                brain_seed(seed, p.id),
                p.id,
                AiDifficulty::Hard,
            )
        })
        .collect();
    let mut rows = Vec::new();
    while !state.finished && state.turn <= 400 {
        if let Some(k) = o.turns.iter().position(|t| *t == state.turn) {
            let alive: Vec<PlayerId> = state
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| p.id)
                .collect();
            let seat = alive[(index + k) % alive.len()];
            let mut values: Vec<Vec<(PlanKind, i32)>> = Vec::new();
            for m in MODES {
                let mut node = OracleNode::from_recorded_state(
                    &state,
                    &with_mode(&brains, seat, m, state.turn),
                    seat,
                );
                values.push(node.choices());
            }
            let mut pool: Vec<PlanKind> = {
                let mut c = values[0].clone();
                c.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
                c.into_iter().take(o.options).map(|(k, _)| k).collect()
            };
            let pick_of = |vals: &[(PlanKind, i32)]| {
                vals.iter().max_by_key(|(_, v)| *v).map(|(k, _)| k.clone())
            };
            for v in &values {
                if let Some(p) = pick_of(v)
                    && !pool.contains(&p)
                {
                    pool.push(p);
                }
            }
            let picks: Vec<Option<usize>> = values
                .iter()
                .map(|v| pick_of(v).and_then(|p| pool.iter().position(|k| *k == p)))
                .collect();
            let mut distinct: Vec<usize> = picks.iter().flatten().copied().collect();
            distinct.sort();
            distinct.dedup();
            let won: Vec<Option<u32>> = (0..pool.len())
                .map(|i| {
                    (distinct.len() > 1 && distinct.contains(&i))
                        .then(|| wins(&state, &brains, seat, &pool[i], o))
                })
                .collect();
            rows.push(serde_json::json!({
                "game": index, "seed": seed, "players": players, "turn": state.turn, "seat": seat.0,
                "faction": format!("{:?}", state.player(seat).faction),
                "pool": pool.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>(),
                "values": values.iter().map(|v| pool.iter().map(|k| v.iter().find(|(x, _)| x == k).map(|(_, val)| *val)).collect::<Vec<_>>()).collect::<Vec<_>>(),
                "picks": picks, "samples": o.samples, "wins": won,
            }));
        }
        let orders = brains
            .iter_mut()
            .filter(|b| state.player(b.player).alive)
            .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
            .collect();
        state = resolve_turn(&state, &orders).0;
    }
    rows
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = arg(&args, "--games").map_or(4, |v| v.parse().expect("--games"));
    let first: u64 = arg(&args, "--seed").map_or(8_880_001, |v| v.parse().expect("--seed"));
    let players: Vec<usize> = list(&arg(&args, "--players").unwrap_or_else(|| "8,8,8,4".into()));
    let o = Opts {
        turns: list(&arg(&args, "--turns").unwrap_or_else(|| "40,80,120,160".into())),
        options: arg(&args, "--options").map_or(4, |v| v.parse().expect("--options")),
        samples: arg(&args, "--samples").map_or(16, |v| v.parse().expect("--samples")),
        horizon: arg(&args, "--horizon").map_or(20, |v| v.parse().expect("--horizon")),
    };
    let label = arg(&args, "--label").unwrap_or_else(|| "regret".into());
    let dir = PathBuf::from(arg(&args, "--out-dir").unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir).expect("out dir");
    let out = Mutex::new(
        std::fs::File::create(dir.join(format!("value-regret-{label}.jsonl"))).expect("out"),
    );
    let done = AtomicUsize::new(0);
    (0..games).into_par_iter().for_each(|i| {
        let rows = play(i, first + i as u64, players[i % players.len()], &o);
        let mut f = out.lock().unwrap();
        for r in rows {
            writeln!(f, "{r}").expect("write");
        }
        let k = done.fetch_add(1, Ordering::Relaxed) + 1;
        eprintln!("{label}: {k}/{games} games");
    });
}

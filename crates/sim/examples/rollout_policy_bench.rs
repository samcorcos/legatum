//! Rollout policy speed: single-threaded rollout turns per second for each policy level.
//!   rollout_policy_bench [states] [horizon]
//! Plays Small-map Standard Hard duels (seeds 7100001..) to turns 40 and 90, determinizes
//! seat 0's view, and times rollouts of `horizon` turns from each state with every seat on
//! rollout policy 0 to 4 (`SearchConfig::rollout_policy`). Same states for every level.
use legatum_ai::search::{determinize, prior_map, rollout, rollout_seat};
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{
    AiDifficulty, Faction, GameSetup, GameState, PlayerId, player_view, resolve_turn,
};
use std::time::Instant;

fn states(n: usize) -> Vec<GameState> {
    let mut out = Vec::new();
    for i in 0..n {
        let mut setup = GameSetup::all_ai(7_100_001 + i as u64);
        setup.players.truncate(2);
        (setup.width, setup.height) = (40, 26);
        for (s, p) in setup.players.iter_mut().enumerate() {
            p.faction = Faction::ALL[(i + s * 3) % 8];
            p.difficulty = AiDifficulty::Hard;
        }
        let mut state = legatum_core::new_game(&setup);
        let mut brains: Vec<ScriptedBrain> = (0..2u8)
            .map(|s| {
                ScriptedBrain::with_difficulty(
                    setup.players[s as usize].faction,
                    9,
                    PlayerId(s),
                    AiDifficulty::Hard,
                )
            })
            .collect();
        for stop in [40, 90] {
            while state.turn < stop && !state.finished {
                let orders = brains
                    .iter_mut()
                    .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
                    .collect();
                state = resolve_turn(&state, &orders).0;
            }
            if !state.finished {
                let view = player_view(&state, PlayerId(0));
                out.push(determinize(&view, &prior_map(&view, 1), 3));
            }
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).map_or(6, |x| x.parse().unwrap());
    let horizon: u32 = args.get(2).map_or(20, |x| x.parse().unwrap());
    let starts = states(n);
    for level in 0..=4u8 {
        let t = Instant::now();
        let mut turns = 0u64;
        let mut score = 0i64;
        for (i, s) in starts.iter().enumerate() {
            let me = PlayerId(0);
            let seat =
                ScriptedBrain::with_difficulty(s.player(me).faction, 5, me, AiDifficulty::Hard)
                    .with_rollout_policy(level);
            let before = s.turn;
            score += rollout(
                s.clone(),
                me,
                rollout_seat(seat, level, s.turn),
                horizon,
                77 + i as u64,
                level << 4,
            ) as i64;
            turns += horizon as u64;
            let _ = before;
        }
        let secs = t.elapsed().as_secs_f64();
        println!(
            "level {level}: {} rollouts x {horizon} turns in {:.2}s = {:.0} rollout turns/s (1 thread); mean horizon chance {}",
            starts.len(),
            secs,
            turns as f64 / secs,
            score / starts.len().max(1) as i64
        );
    }
}

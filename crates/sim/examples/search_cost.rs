//! Wall time of Hard+Search decisions on a quiet pool (pillar 7).
//!   search_cost <search-spec> [games] [first-seed]
//! Plays 1v1 duel-matrix games one at a time (Small map, Standard, Hard): seat i mod 2 is
//! Hard+Search, the other plain Hard. Rollouts use the whole rayon global pool, as a
//! desktop app would. Prints per-turn decision time of the search seat and of both seats.
use legatum_ai::search::{SearchBrain, SearchConfig};
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Faction, GameSetup, PlayerId, player_view, resolve_turn};
use std::time::Instant;

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v.get(((v.len() as f64 - 1.0) * q) as usize)
        .copied()
        .unwrap_or(0.0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = SearchConfig::parse(args.get(1).map_or("", |s| s)).expect("search spec");
    let games: usize = args.get(2).map_or(8, |g| g.parse().expect("games"));
    let first: u64 = args.get(3).map_or(7_100_001, |s| s.parse().expect("seed"));
    let (mut seat_ms, mut turn_ms, mut search_ms) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..games {
        let all = Faction::ALL;
        let cell = (i / 2) % 64;
        let (a, b, seat_a) = (all[cell / 8], all[cell % 8], i % 2);
        let mut setup = GameSetup::all_ai(first + (i / 2) as u64);
        setup.players.truncate(2);
        (setup.width, setup.height) = (40, 26);
        for (s, p) in setup.players.iter_mut().enumerate() {
            p.faction = if s == seat_a { a } else { b };
            p.difficulty = AiDifficulty::Hard;
        }
        let mut state = legatum_core::new_game(&setup);
        let mut brains: Vec<Box<dyn Brain>> = (0..2u8)
            .map(|s| {
                let hard = ScriptedBrain::with_difficulty(
                    setup.players[s as usize].faction,
                    1000 + s as u64,
                    PlayerId(s),
                    AiDifficulty::Hard,
                );
                if s as usize == seat_a {
                    Box::new(SearchBrain::new(hard, config)) as Box<dyn Brain>
                } else {
                    Box::new(hard)
                }
            })
            .collect();
        while !state.finished && state.turn < 600 {
            let mut orders = std::collections::BTreeMap::new();
            let mut total = 0.0;
            for (s, brain) in brains.iter_mut().enumerate() {
                let id = PlayerId(s as u8);
                if !state.player(id).alive {
                    continue;
                }
                let before = brain.search_events().len();
                let t = Instant::now();
                let o = brain.orders(&player_view(&state, id));
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                total += ms;
                if s == seat_a {
                    seat_ms.push(ms);
                    if brain.search_events().len() > before {
                        search_ms.push(ms);
                    }
                }
                orders.insert(id, o);
            }
            turn_ms.push(total);
            state = resolve_turn(&state, &orders).0;
        }
        eprintln!("game {i}: {a:?} vs {b:?}, {} turns", state.turn);
    }
    let n = seat_ms.len().max(1) as f64;
    println!(
        "{}: {} games, {} turns. Search seat ms/turn mean {:.1} p95 {:.1} max {:.1}; turns with a search: {} (mean {:.0} ms, p95 {:.0}, max {:.0}); both seats ms/turn mean {:.1} p95 {:.1} max {:.1}",
        config.label(),
        games,
        turn_ms.len(),
        seat_ms.iter().sum::<f64>() / n,
        pct(&mut seat_ms.clone(), 0.95),
        seat_ms.iter().cloned().fold(0.0, f64::max),
        search_ms.len(),
        search_ms.iter().sum::<f64>() / search_ms.len().max(1) as f64,
        pct(&mut search_ms.clone(), 0.95),
        search_ms.iter().cloned().fold(0.0, f64::max),
        turn_ms.iter().sum::<f64>() / turn_ms.len().max(1) as f64,
        pct(&mut turn_ms.clone(), 0.95),
        turn_ms.iter().cloned().fold(0.0, f64::max),
    );
}

//! Determinization fidelity and rollout cost for Hard+Search.
//!   search_probe [seed] [turn...]
//! Plays an 8-seat Standard Hard game to each listed turn, then for seat 0 compares its real
//! view with the view of a state determinized from that view (own facts must match; rival
//! public facts should be close), and times 20-turn rollouts from the true and the
//! determinized state.
use legatum_ai::search::{calibrate_economy, chance_in, determinize, prior_map, rollout};
use legatum_ai::winchance::WinFacts;
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, GameSetup, PlayerId, player_view, resolve_turn};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).map_or(3_600_001, |s| s.parse().expect("seed"));
    let mut turns: Vec<u32> = args[2.min(args.len())..]
        .iter()
        .map(|t| t.parse().expect("turn"))
        .collect();
    if turns.is_empty() {
        turns = vec![40, 80, 120];
    }
    let mut setup = GameSetup::all_ai(seed);
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(p.faction, 11 + p.id.0 as u64, p.id, AiDifficulty::Hard)
        })
        .collect();
    let me = PlayerId(0);
    for target in turns {
        let t0 = Instant::now();
        while state.turn < target && !state.finished {
            let orders = brains
                .iter_mut()
                .filter(|b| state.player(b.player).alive)
                .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
                .collect();
            state = resolve_turn(&state, &orders).0;
        }
        let played = t0.elapsed();
        if state.finished || !state.player(me).alive {
            println!("game over before turn {target}");
            break;
        }
        let view = player_view(&state, me);
        let t = Instant::now();
        let prior = prior_map(&view, 99);
        let mapgen_ms = t.elapsed().as_millis();
        let t = Instant::now();
        let det = determinize(&view, &prior, 1234);
        let det_ms = t.elapsed().as_millis();
        let v2 = player_view(&det, me);
        println!(
            "== turn {} (played in {:.1}s; mapgen {mapgen_ms} ms, determinize {det_ms} ms)",
            state.turn,
            played.as_secs_f64()
        );
        let own = |v: &legatum_core::PlayerView| {
            let mut u: Vec<_> = v
                .units
                .iter()
                .filter(|u| u.owner == me)
                .map(|u| (u.id, u.pos, u.hp))
                .collect();
            u.sort();
            u
        };
        println!(
            "own units equal: {}; own cities {} vs {}; gold {} vs {}; gpt {} vs {}; spt {} vs {}; visible units {} vs {}",
            own(&view) == own(&v2),
            view.cities.iter().filter(|c| c.owner == me).count(),
            v2.cities.iter().filter(|c| c.owner == me).count(),
            view.economy.gold,
            v2.economy.gold,
            view.economy.gold_per_turn.total,
            v2.economy.gold_per_turn.total,
            view.economy.science_per_turn.total,
            v2.economy.science_per_turn.total,
            view.units.len(),
            v2.units.len()
        );
        let detail = |v: &legatum_core::PlayerView| {
            v.cities
                .iter()
                .filter(|c| c.owner == me)
                .map(|c| {
                    let d = c.detail.as_ref().unwrap();
                    (
                        c.id,
                        c.population,
                        d.yields.total.food,
                        d.yields.total.production,
                        d.yields.total.gold,
                        d.producing,
                    )
                })
                .collect::<Vec<_>>()
        };
        if detail(&view) != detail(&v2) {
            println!(
                "  city detail differs:\n   real {:?}\n   det  {:?}",
                detail(&view),
                detail(&v2)
            );
        }
        println!(
            "chance {} vs {}  (truth-state chance {})",
            WinFacts::of(&view).chance(),
            WinFacts::of(&v2).chance(),
            chance_in(&state, me)
        );
        for (name, value) in [
            (
                "economy",
                (|p: &legatum_core::view::PlayerPublic| p.economy) as fn(&_) -> i32,
            ),
            ("army", |p| p.army),
            ("advancement", |p| p.advancement),
        ] {
            let vals =
                |v: &legatum_core::PlayerView| v.players.iter().map(value).collect::<Vec<_>>();
            println!(
                "  {name}: real {:?}\n  {:>w$}  det  {:?}",
                vals(&view),
                "",
                vals(&v2),
                w = name.len()
            );
        }
        println!(
            "  cities by player real {:?} det {:?}",
            view.players.iter().map(|p| p.cities).collect::<Vec<_>>(),
            v2.players.iter().map(|p| p.cities).collect::<Vec<_>>()
        );
        let mut cal = det.clone();
        calibrate_economy(&mut cal, &view);
        let v3 = player_view(&cal, me);
        println!(
            "  calibrated economy {:?}, chance {}",
            v3.players.iter().map(|p| p.economy).collect::<Vec<_>>(),
            WinFacts::of(&v3).chance()
        );
        // Search overhead per rollout: copying the seat's brain, and determinizing.
        let b0 = &brains[0];
        let t = Instant::now();
        for _ in 0..100 {
            std::hint::black_box(b0.rollout_copy());
        }
        let copy_us = t.elapsed().as_secs_f64() * 1e6 / 100.0;
        let t = Instant::now();
        for i in 0..20 {
            std::hint::black_box(determinize(&view, &prior, i));
        }
        let det_us = t.elapsed().as_secs_f64() * 1e6 / 20.0;
        let t = Instant::now();
        for _ in 0..20 {
            std::hint::black_box(det.clone());
        }
        let clone_us = t.elapsed().as_secs_f64() * 1e6 / 20.0;
        println!(
            "  overhead: brain copy {copy_us:.1} us, determinize {det_us:.0} us, state clone {clone_us:.1} us"
        );
        // Rollout cost: plain Hard everywhere, 20 Standard turns.
        for (label, start) in [("truth", state.clone()), ("determinized", det.clone())] {
            let t = Instant::now();
            let brain =
                ScriptedBrain::with_difficulty(state.player(me).faction, 5, me, AiDifficulty::Hard);
            let c = rollout(start, me, Box::new(brain), 20, 77, 0);
            println!(
                "  rollout from {label}: {:.0} ms for 20 turns, horizon chance {c}",
                t.elapsed().as_secs_f64() * 1000.0
            );
        }
        // Cost split for the real game: one turn's views, orders and resolution.
        let t = Instant::now();
        let views: Vec<_> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| player_view(&state, p.id))
            .collect();
        let view_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let orders = brains
            .iter_mut()
            .zip(&views)
            .map(|(b, v)| (b.player, b.orders(v)))
            .collect();
        let orders_ms = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        let _ = resolve_turn(&state, &orders);
        println!(
            "  one real turn: views {view_ms:.1} ms, orders {orders_ms:.1} ms, resolve {:.1} ms",
            t.elapsed().as_secs_f64() * 1000.0
        );
    }
}

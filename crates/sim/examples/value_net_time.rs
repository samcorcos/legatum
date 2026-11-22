//! What the learned value network (gene `learned_value`) costs the live AI.
//!
//!   RAYON_NUM_THREADS=1 value_net_time [--seeds 7000001,7000002] [--turns 260] [--calls 200000]
//!
//! 1. Per estimate: one `WinFacts::chance()` on a T100 8-player view, hand formula against the
//!    network (ns per call, the facts built once, as the planner reuses them).
//! 2. Per lobby turn: 8-empire Standard games, every seat plain Hard, played with the gene off and
//!    on (both through `ScriptedBrain::with_params`), timing every turn's whole-lobby planning on
//!    one thread. Prints median and p95 per window (turn +/- 10) for each arm.
use legatum_ai::winchance::WinFacts;
use legatum_ai::{AiParams, DecisionParams, ScriptedBrain};
use legatum_core::{AiDifficulty, Faction, GameSetup, PlayerId, player_view};
use legatum_wasm::session::{BrainKind, Session, brain_seed};
use std::hint::black_box;
use std::time::Instant;

fn arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn session(seed: u64, params: &AiParams) -> Session {
    let mut setup = GameSetup::all_ai(seed);
    (setup.width, setup.height) = (48, 32);
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(i + seed as usize) % 8];
        p.difficulty = AiDifficulty::Hard;
    }
    let mut s = Session::new(&setup, |_| BrainKind::Scripted);
    for p in s.state.players.clone() {
        let brain = ScriptedBrain::with_params(
            p.faction,
            brain_seed(setup.seed, p.id),
            p.id,
            AiDifficulty::Hard,
            params,
        );
        s.replace_brain(p.id, Box::new(brain));
    }
    s
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: Vec<u64> = arg(&args, "--seeds")
        .unwrap_or_else(|| "7000001,7000002".into())
        .split(',')
        .map(|s| s.parse().expect("seed"))
        .collect();
    let turns: u32 = arg(&args, "--turns").map_or(260, |v| v.parse().expect("turns"));
    let calls: u32 = arg(&args, "--calls").map_or(200_000, |v| v.parse().expect("calls"));
    assert!(
        legatum_ai::value_net::net().is_some(),
        "crates/ai/data/value_net.json did not load"
    );
    let off = AiParams::default();
    let mut on = AiParams::default();
    on.decisions.learned_value = 1;

    // 1. Per estimate.
    let mut s = session(seeds[0], &off);
    while s.state.turn < 100 && !s.state.finished {
        let o = s.ai_orders();
        s.step(&o);
    }
    let view = player_view(&s.state, PlayerId(0));
    let base = WinFacts::of(&view);
    let hand = base.clone().with_relative(&DecisionParams::default());
    let learned = base.clone().with_relative(&on.decisions);
    assert!(learned.learned == 1 && hand.learned == 0);
    for (name, facts) in [("hand formula", &hand), ("learned network", &learned)] {
        let t = Instant::now();
        let mut sum = 0i64;
        for _ in 0..calls {
            sum += black_box(facts).chance() as i64;
        }
        let ns = t.elapsed().as_nanos() as f64 / calls as f64;
        println!(
            "per estimate, {name}: {ns:.0} ns ({} teams, T{}, chance {})",
            facts.teams.len(),
            view.turn,
            sum / calls as i64
        );
    }

    // 2. Per lobby turn.
    let windows = [50u32, 150, 250];
    for (arm, params) in [("gene off", &off), ("gene on", &on)] {
        let mut per_window: Vec<Vec<f64>> = vec![Vec::new(); windows.len()];
        let mut all = Vec::new();
        for &seed in &seeds {
            let mut s = session(seed, params);
            while s.state.turn <= turns && !s.state.finished {
                let t = Instant::now();
                let o = s.ai_orders();
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                for (i, w) in windows.iter().enumerate() {
                    if s.state.turn.abs_diff(*w) <= 10 {
                        per_window[i].push(ms);
                    }
                }
                all.push(ms);
                s.step(&o);
            }
        }
        let mut line = format!("lobby planning, {arm}:");
        for (i, w) in windows.iter().enumerate() {
            let n = per_window[i].len();
            line += &format!(
                " T{w} median {:.1} p95 {:.1} ms (n {n});",
                pct(&mut per_window[i], 0.5),
                pct(&mut per_window[i], 0.95)
            );
        }
        let total: f64 = all.iter().sum();
        line += &format!(
            " all turns mean {:.1} ms over {}",
            total / all.len().max(1) as f64,
            all.len()
        );
        println!("{line}");
    }
}

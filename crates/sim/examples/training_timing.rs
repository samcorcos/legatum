//! Training mode timing: seconds per turn of the native battle search (all cores, the 15 s cap,
//! a real clock), in an all-Training-AI game.
//!
//!   LEGATUM_ROLLOUT_THREADS=16 cargo run --release -p legatum-sim --example training_timing --
//!       [--players 6] [--turns 150] [--seed 7100003] [--cap-ms 15000] [--log FILE]
//!       [--round2] [--confirm 16] [--confirm-turns 6] [--top-k 3]
//!
//! `--round2`: Training round 2's settings (the full-game check, kept plans).
//!
//! Prints one row per turn with battles and search time, then the mean, p50, p95 and max of
//! the turns that had battles, and how many hit the cap.
use legatum_ai::training::{LogLine, TrainingConfig, search_turn_with};
use legatum_core::{AiDifficulty, GameSetup};
use legatum_wasm::session::{BrainKind, Session};
use std::io::Write;
use std::time::Instant;

fn arg(args: &[String], key: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == key).map(|w| w[1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let players: usize = arg(&args, "--players").map_or(6, |v| v.parse().expect("players"));
    let turns: u32 = arg(&args, "--turns").map_or(150, |v| v.parse().expect("turns"));
    let seed: u64 = arg(&args, "--seed").map_or(7_100_003, |v| v.parse().expect("seed"));
    let cap_ms: u64 = arg(&args, "--cap-ms").map_or(15_000, |v| v.parse().expect("cap"));
    let mut log = arg(&args, "--log")
        .map(|p| std::io::BufWriter::new(std::fs::File::create(p).expect("log")));
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    let (w, h) = if players <= 4 { (40, 26) } else { (48, 32) };
    (setup.width, setup.height) = (w, h);
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
        p.search = true;
    }
    let mut s = Session::new(&setup, |_| BrainKind::Scripted);
    s.history = None;
    let mut cfg = TrainingConfig {
        cap_ms,
        ..Default::default()
    };
    if args.iter().any(|a| a == "--round2") {
        cfg = cfg.round2();
    }
    if let Some(v) = arg(&args, "--confirm") {
        cfg.confirm = v.parse().expect("confirm");
    }
    if let Some(v) = arg(&args, "--confirm-turns") {
        cfg.confirm_turns = v.parse().expect("confirm turns");
    }
    if let Some(v) = arg(&args, "--top-k") {
        cfg.top_k = v.parse().expect("top k");
    }
    let mut plans = Vec::new();
    let mut rows = Vec::new();
    let mut verdicts = std::collections::BTreeMap::new();
    println!("turn  battles  hard_ms  search_ms  rollouts  changed  complete");
    while !s.state.finished && s.state.turn <= turns {
        let t0 = Instant::now();
        let hard = s.training_hard_orders();
        let hard_ms = t0.elapsed().as_millis() as u64;
        let t1 = Instant::now();
        let result = search_turn_with(
            &s.state,
            hard,
            &cfg,
            &plans,
            |_| true,
            || t1.elapsed().as_millis() as u64,
            |_| {},
        );
        plans.clone_from(&result.plans);
        let search_ms = t1.elapsed().as_millis() as u64;
        let battles = result.records.len();
        let rollouts: u64 = result.records.iter().map(|r| r.rollouts).sum();
        let changed = result
            .records
            .iter()
            .filter(|r| r.changed_units > 0 || r.stance != legatum_ai::training::Stance::Hard)
            .count();
        for r in &result.records {
            if let Some(c) = &r.confirm {
                *verdicts.entry(c.reason.clone()).or_insert(0u32) += 1;
            }
        }
        let complete = result.records.iter().filter(|r| r.complete).count();
        if let Some(log) = &mut log {
            for r in &result.records {
                writeln!(log, "{}", LogLine::Decision(Box::new(r.clone())).json()).expect("log");
            }
        }
        s.apply_training(&result).expect("apply");
        let orders = s.take_ai_orders();
        s.step(&orders);
        if battles > 0 {
            println!(
                "{:4}  {battles:7}  {hard_ms:7}  {search_ms:9}  {rollouts:8}  {changed:7}  {complete:8}",
                s.state.turn - 1
            );
            rows.push((search_ms, battles, changed, search_ms + 50 >= cap_ms));
        }
    }
    if rows.is_empty() {
        println!("no battles");
        return;
    }
    let mut ms: Vec<u64> = rows.iter().map(|r| r.0).collect();
    ms.sort_unstable();
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p) as usize];
    let battles: usize = rows.iter().map(|r| r.1).sum();
    let changed: usize = rows.iter().map(|r| r.2).sum();
    println!(
        "TRAINING-TIMING {}",
        serde_json::json!({
            "players": players, "turns_with_battles": rows.len(), "battles": battles,
            "changed": changed, "mean_ms": ms.iter().sum::<u64>() / ms.len() as u64,
            "p50_ms": q(0.5), "p95_ms": q(0.95), "max_ms": ms[ms.len() - 1],
            "capped_turns": rows.iter().filter(|r| r.3).count(), "cap_ms": cap_ms,
            "threads": std::env::var("LEGATUM_ROLLOUT_THREADS").unwrap_or_default(),
            "confirm": cfg.confirm, "confirm_turns": cfg.confirm_turns, "top_k": cfg.top_k,
            "persist": cfg.persist, "verdicts": verdicts,
        })
    );
}

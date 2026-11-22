//! What a player waits on after End Turn, turn by turn, on a Standard 8-empire map: the app's
//! own `Game::end_turn_json` (7 Hard AI seats plus a human in seat 0 whose orders come from a
//! Hard scripted brain, so the human's empire grows like a real player's), the follow-up
//! `Game::view_json(0)` the UI makes, and, from a shadow `Session` playing the same game, the
//! split of a turn into AI planning (`ai_orders`) and rules resolution (`step`). Single thread:
//! pass `RAYON_NUM_THREADS=1` (the WASM build is single-threaded; native plans seats in parallel).
//!   turn_profile [turns=400] [seed=7000001] [csv|-] [orders-file] [save-at-turn:path] [initial-save-path]
//! With an orders file, the human's per-turn orders JSON are written one per line, so
//! `scripts/perf-wasm-turns.mjs` replays the same game through the WASM build.
//! Prints one summary row per bucket of turns (or one CSV row per turn with `csv`).
use legatum_core::{GameSetup, PlayerId, player_view};
use legatum_wasm::Game;
use legatum_wasm::session::{BrainKind, OrderMap, Session, make_brain};
use std::time::Instant;

const FACTIONS: [&str; 8] = [
    "Switzerland",
    "India",
    "Byzantium",
    "Germany",
    "Celts",
    "Mongols",
    "Sparta",
    "Athens",
];

fn setup_json(seed: u64, players: usize) -> String {
    let seats: Vec<_> = (0..players)
        .map(|i| {
            serde_json::json!({
                "name": if i == 0 { "Me".to_string() } else { format!("AI {i}") },
                "faction": FACTIONS[(i + seed as usize) % 8],
                "team": i,
                "is_human": i == 0,
                "difficulty": "Hard",
            })
        })
        .collect();
    serde_json::json!({
        "seed": seed.to_string(), "width": 48, "height": 32, "turn_limit": null,
        "barbarians": true, "players": seats,
    })
    .to_string()
}

#[derive(Default, Clone)]
struct Row {
    turn: u32,
    hash: String,
    end_ms: f64,
    view_ms: f64,
    view_kb: f64,
    ai_ms: f64,
    step_ms: f64,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let turns: u32 = args.get(1).map_or(400, |a| a.parse().expect("turns"));
    let seed: u64 = args.get(2).map_or(7_000_001, |a| a.parse().expect("seed"));
    let csv = args.get(3).is_some_and(|a| a == "csv");
    let capture = args.get(3).is_some_and(|a| a == "capture");
    let mut orders_out = args
        .get(4)
        .map(|p| std::io::BufWriter::new(std::fs::File::create(p).expect("orders file")));
    let save_at = args.get(5).map(|spec| {
        let (turn, path) = spec.split_once(':').expect("save-at-turn:path");
        (turn.parse::<u32>().expect("save turn"), path)
    });
    let json = setup_json(seed, if capture { 4 } else { 8 });
    let mut game = Game::from_setup_json(&json).expect("setup");
    if let Some(path) = args.get(6) {
        std::fs::write(path, game.save_json()).expect("write initial save");
    }
    let setup: GameSetup = legatum_wasm::json::parse_setup(&json).expect("setup");
    let mut shadow = Session::new(&setup, |_| BrainKind::Scripted);
    shadow.history = None;
    let mut human = make_brain(BrainKind::Scripted, &shadow.state, PlayerId(0));
    let mut rows: Vec<Row> = Vec::new();
    for t in 1..=turns {
        let mine = human.orders(&player_view(&shadow.state, PlayerId(0)));
        let mut human_orders = OrderMap::new();
        human_orders.insert(PlayerId(0), mine);
        let human_json = serde_json::to_string(&human_orders).expect("orders");
        if let Some(out) = orders_out.as_mut() {
            use std::io::Write;
            writeln!(out, "{human_json}").expect("write orders");
        }
        let start = Instant::now();
        let Ok(report) = game.end_turn_json(&human_json) else {
            break;
        };
        let end_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let view = game.view_json(0).expect("view");
        let view_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let mut orders = shadow.ai_orders();
        let ai_ms = start.elapsed().as_secs_f64() * 1000.0;
        orders.extend(human_orders);
        let start = Instant::now();
        shadow.step(&orders);
        let step_ms = start.elapsed().as_secs_f64() * 1000.0;
        let hash = game.hash();
        assert_eq!(hash, shadow.state.hash().to_string(), "shadow diverged");
        if capture {
            println!(
                "{}",
                serde_json::json!({
                    "turn": t, "report": report, "view": view, "hash": hash,
                })
            );
        }
        if let Some((turn, path)) = save_at
            && t == turn
        {
            std::fs::write(path, game.save_json()).expect("write save");
        }
        rows.push(Row {
            turn: t,
            hash,
            end_ms,
            view_ms,
            view_kb: view.len() as f64 / 1024.0,
            ai_ms,
            step_ms,
        });
        if shadow.state.finished {
            break;
        }
    }
    if capture {
        return;
    }
    if csv {
        println!("turn,end_turn_ms,view_ms,view_kb,ai_ms,step_ms,hash");
        for r in &rows {
            println!(
                "{},{:.2},{:.2},{:.1},{:.2},{:.2},{}",
                r.turn, r.end_ms, r.view_ms, r.view_kb, r.ai_ms, r.step_ms, r.hash
            );
        }
        return;
    }
    println!(
        "seed {seed}, {} turns played (human seat 0 driven by a Hard brain, 7 Hard AI)",
        rows.len()
    );
    println!("turns        end_turn mean/p95/max ms | view(0) ms  KB | AI plan ms | resolve ms");
    let buckets: [(u32, u32); 8] = [
        (1, 10),
        (11, 30),
        (31, 60),
        (61, 100),
        (101, 150),
        (151, 200),
        (201, 300),
        (301, 1000),
    ];
    for (lo, hi) in buckets {
        let sel: Vec<&Row> = rows
            .iter()
            .filter(|r| r.turn >= lo && r.turn <= hi)
            .collect();
        if sel.is_empty() {
            continue;
        }
        let n = sel.len() as f64;
        let mean = |f: fn(&Row) -> f64| sel.iter().map(|r| f(r)).sum::<f64>() / n;
        let mut e: Vec<f64> = sel.iter().map(|r| r.end_ms).collect();
        e.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = e[((e.len() - 1) as f64 * 0.95) as usize];
        println!(
            "{lo:>3}-{hi:<4} {:>3} {:>9.1} / {:>6.1} / {:>6.1} | {:>8.1} {:>6.0} | {:>9.1} | {:>9.1}",
            sel.len(),
            mean(|r| r.end_ms),
            p95,
            e[e.len() - 1],
            mean(|r| r.view_ms),
            mean(|r| r.view_kb),
            mean(|r| r.ai_ms),
            mean(|r| r.step_ms),
        );
    }
}

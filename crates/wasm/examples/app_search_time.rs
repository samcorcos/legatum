//! How long Hard+Search seats take per turn on one thread, as in the app (the WASM engine
//! cannot run rollouts in parallel). Seat 0 is plain Hard and stands in for the human.
//!   app_search_time <players> <turns> <search-spec|none> [seed]
//! Prints the thread CPU time of all AI seats' planning per turn (mean, p50, p95, max) and the search
//! seats' share. Native single thread; the WASM build runs about 1.3x slower (see learnings).
use legatum_ai::search::{SearchBrain, SearchConfig};
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Faction, GameSetup, PlayerId, ViewDetail, resolve_turn};
use std::collections::BTreeMap;

/// Thread CPU time in ms: a busy machine inflates wall time, not this.
fn cpu_ms() -> f64 {
    // With rollouts on a thread pool (build with `-p legatum-sim` too) this thread only waits:
    // set LEGATUM_WALL=1 to read the wall clock instead.
    if std::env::var_os("LEGATUM_WALL").is_some() {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        return START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1000.0;
    }
    #[repr(C)]
    struct Timespec {
        sec: i64,
        nsec: i64,
    }
    unsafe extern "C" {
        fn clock_gettime(clock: i32, ts: *mut Timespec) -> i32;
    }
    #[cfg(target_os = "macos")]
    const THREAD_CPU: i32 = 16;
    #[cfg(not(target_os = "macos"))]
    const THREAD_CPU: i32 = 3;
    let mut ts = Timespec { sec: 0, nsec: 0 };
    // SAFETY: a valid pointer to a timespec-sized struct.
    unsafe { clock_gettime(THREAD_CPU, &mut ts) };
    ts.sec as f64 * 1000.0 + ts.nsec as f64 / 1e6
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let players: usize = args.get(1).map_or(2, |a| a.parse().expect("players"));
    let turns: u32 = args.get(2).map_or(60, |a| a.parse().expect("turns"));
    let spec = args.get(3).map_or("none", |s| s.as_str());
    let seed: u64 = args.get(4).map_or(12345, |a| a.parse().expect("seed"));
    let config = (spec != "none").then(|| {
        if spec == "app" {
            SearchConfig::in_app(players)
        } else {
            SearchConfig::parse(spec).expect("search spec")
        }
    });
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    if players <= 4 {
        (setup.width, setup.height) = (40, 26);
    }
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(i + seed as usize) % Faction::ALL.len()];
        p.difficulty = AiDifficulty::Hard;
        p.search = i > 0 && config.is_some();
    }
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<Box<dyn Brain>> = (0..players as u8)
        .map(|s| {
            let id = PlayerId(s);
            let seed = legatum_wasm::session::brain_seed(setup.seed, id);
            let hard = ScriptedBrain::with_difficulty(
                setup.players[s as usize].faction,
                seed,
                id,
                AiDifficulty::Hard,
            );
            match config {
                Some(c) if s > 0 => Box::new(SearchBrain::new(hard, c)) as Box<dyn Brain>,
                _ => Box::new(hard),
            }
        })
        .collect();
    let mut turn_ms = Vec::new();
    // Wall time of each seat's planning when it ran a search, and how many searches share a turn.
    let mut search_ms = Vec::new();
    let mut per_turn = Vec::new();
    while !state.finished && state.turn < turns {
        let public = legatum_core::public_view(&state);
        let start = cpu_ms();
        let mut orders = BTreeMap::new();
        let mut searched = 0;
        for (s, brain) in brains.iter_mut().enumerate() {
            let id = PlayerId(s as u8);
            if state.player(id).alive {
                let view =
                    legatum_core::player_view_detail(&state, id, &public, ViewDetail::Decisions);
                let before = brain.search_events().len();
                let t = cpu_ms();
                orders.insert(id, brain.orders(&view));
                if brain.search_events().len() > before {
                    search_ms.push(cpu_ms() - t);
                    searched += 1;
                }
            }
        }
        turn_ms.push(cpu_ms() - start);
        per_turn.push(searched);
        state = resolve_turn(&state, &orders).0;
    }
    let mut sorted = turn_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| sorted[((sorted.len() - 1) as f64 * p) as usize];
    search_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "{} searches in {} turns (max {} on one turn), per search mean {:.0} ms p95 {:.0} max {:.0}",
        search_ms.len(),
        per_turn.len(),
        per_turn.iter().max().copied().unwrap_or(0),
        search_ms.iter().sum::<f64>() / search_ms.len().max(1) as f64,
        search_ms
            .get(search_ms.len().saturating_sub(1) * 95 / 100)
            .copied()
            .unwrap_or(0.0),
        search_ms.last().copied().unwrap_or(0.0)
    );
    println!(
        "{players}p {spec}: {} turns, planning ms/turn mean {:.1} p50 {:.1} p95 {:.1} max {:.1}",
        sorted.len(),
        sorted.iter().sum::<f64>() / sorted.len().max(1) as f64,
        q(0.5),
        q(0.95),
        sorted.last().copied().unwrap_or(0.0)
    );
}

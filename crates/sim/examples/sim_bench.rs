//! Engine + AI throughput benchmark: whole games played by the scripted brains through the
//! shared `Session` loop (player_view → brain.orders → resolve_turn), with no runner telemetry,
//! tape or archive. This is the loop the search AI's rollouts multiply.
//!
//!   cargo run --release -p legatum-sim --example sim_bench -- [duel|small|eight|all|search|views|rollout] [games] [threads]
//!
//! Scenarios:
//! - duel: 1v1 Standard, Hard vs Hard, barbarians on, default map (48x32: map size does not scale
//!   with player count; the lobby default is Medium for every player count).
//! - small: 1v1 Standard, Hard vs Hard, Small map (40x26): the AI-quality ledger's duel.
//! - eight: 8 players, Standard, all Hard, barbarians on, default map.
//!
//! `rollout` measures the search AI's workload instead: from mid-game states (duel turn 30,
//! small-map duel turn 60, eight-player turn 100) it times `GameState::clone` and 20-turn forward runs with fresh
//! scripted brains (turns per second from a mid-game state).
//!
//! `search` is the search AI's game: 1v1 Standard on the Small map (40x26), seat 0 Hard+Search
//! at the default search config plus fitted plan weights (`SearchConfig::default()` with `w`:
//! k3 h20 d4 w), seat 1 plain Hard, seeds 7_000_001.. (default 4 games). It reports CPU and wall
//! seconds per game and a digest of every final state hash, turn count and search decision
//! (each search's options, sample results and choice). Rollouts run on the rollout pool:
//! `LEGATUM_ROLLOUT_THREADS=1` for instruction counts, all cores for wall time.
//!
//! `latency` times single games from a plain thread (AI seats planned in parallel) against the
//! same games planned one seat at a time, and checks they end identically.
//!
//! `views` prints a digest of every seat's PlayerView JSON on every turn of a few games, to
//! check that view-building changes leave the views byte-identical.
//!
//! Each scenario runs `games` games (seeds 7_000_001..) single-threaded, then a larger batch
//! from the same seeds across `threads` worker threads (default: all cores; 64 games per thread
//! for duel, 16 for small, 4 for eight, so the slowest game doesn't dominate the wall time). It prints games/s, turns/s and
//! ms/turn, CPU seconds per game, and a digest of every final state hash: an optimization must
//! leave the digest unchanged.
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Faction, GameSetup};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use std::time::Instant;

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    players: usize,
    /// Map size; None is the default (48x32).
    map: Option<(i32, i32)>,
}

const DUEL: Scenario = Scenario {
    name: "duel-standard-hard",
    players: 2,
    map: None,
};
/// The AI-quality ledger's 1v1 (docs/sim-results/ai-quality): Small map.
const SMALL: Scenario = Scenario {
    name: "duel-std-small-hard",
    players: 2,
    map: Some((40, 26)),
};
const EIGHT: Scenario = Scenario {
    name: "eight-standard-hard",
    players: 8,
    map: None,
};
const FIRST_SEED: u64 = 7_000_001;

fn setup(s: Scenario, seed: u64) -> GameSetup {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(s.players);
    // The runner's faction shuffle (runner::setup_for_game), so the bench plays sim-like games.
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (player, faction) in setup.players.iter_mut().zip(factions) {
        player.faction = faction;
        player.difficulty = AiDifficulty::Hard;
    }
    if let Some((w, h)) = s.map {
        (setup.width, setup.height) = (w, h);
    }
    setup
}

/// One full game; returns (turns, final state hash).
fn play(s: Scenario, seed: u64) -> (u32, u64) {
    let setup = setup(s, seed);
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut turns = 0;
    while !session.state.finished && turns < cap {
        let orders = session.ai_orders();
        session.step(&orders);
        turns += 1;
    }
    (turns, session.state.hash())
}

/// The rollout workload: play `seed` to turn `mid`, then from that state time `GameState::clone`
/// and play `rollouts` forward runs of `horizon` turns, each from a fresh clone with fresh
/// scripted brains. Returns (clone microseconds, rollout turns, rollout seconds, digest), or
/// None when the game ended before `mid`.
fn rollouts(
    s: Scenario,
    seed: u64,
    mid: u32,
    horizon: u32,
    n: u32,
) -> Option<(f64, u64, f64, u64)> {
    use legatum_core::{ViewCache, ViewDetail, player_view_cached, public_view, resolve_turn};
    let setup = setup(s, seed);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    while session.state.turn < mid {
        if session.state.finished {
            return None;
        }
        let orders = session.ai_orders();
        session.step(&orders);
    }
    let start = session.state;
    const CLONES: u32 = 200;
    let t0 = Instant::now();
    for _ in 0..CLONES {
        std::hint::black_box(start.clone());
    }
    let clone_us = t0.elapsed().as_secs_f64() * 1e6 / f64::from(CLONES);
    let (mut turns, mut digest) = (0u64, 0u64);
    let t0 = Instant::now();
    for r in 0..n {
        let mut state = start.clone();
        state.setup.seed = seed.wrapping_add(u64::from(r) << 32);
        let mut cache = ViewCache::default();
        let mut brains: Vec<_> = state
            .players
            .iter()
            .map(|p| legatum_wasm::session::make_brain(BrainKind::Scripted, &state, p.id))
            .collect();
        for _ in 0..horizon {
            if state.finished {
                break;
            }
            // As Session::ai_orders: shared public parts, decision-only views.
            let public = public_view(&state);
            let orders = state
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| {
                    let view = player_view_cached(
                        &state,
                        p.id,
                        &public,
                        ViewDetail::Decisions,
                        &mut cache,
                    );
                    (p.id, brains[p.id.0 as usize].orders(&view))
                })
                .collect();
            state = resolve_turn(&state, &orders).0;
            turns += 1;
        }
        digest = (digest ^ state.hash()).wrapping_mul(0x0100_0000_01b3);
    }
    Some((clone_us, turns, t0.elapsed().as_secs_f64(), digest))
}

fn rollout_report(s: Scenario, mid: u32) {
    let (mut clone_us, mut turns, mut secs, mut digest, mut games) = (0.0, 0u64, 0.0, 0u64, 0);
    for seed in FIRST_SEED..FIRST_SEED + 8 {
        if let Some((c, t, x, d)) = rollouts(s, seed, mid, 20, 6) {
            clone_us += c;
            turns += t;
            secs += x;
            digest = (digest ^ d).wrapping_mul(0x0100_0000_01b3);
            games += 1;
        }
    }
    println!(
        "{:<20} rollouts from turn {mid} ({games} games x 6 x 20 turns, 1 thread): \
         turns/s={:>7.0} ms/turn={:>6.3} state-clone-us={:>7.1} digest={digest:016x}",
        s.name,
        turns as f64 / secs,
        secs * 1000.0 / turns as f64,
        clone_us / f64::from(games.max(1)),
    );
}

/// Digest of every seat's PlayerView JSON on every turn of a few games: an optimization of
/// view building must leave it unchanged (views are data for the UI too, not only the AI).
fn view_digest(s: Scenario, games: u64, turns: u32) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325u64;
    for seed in FIRST_SEED..FIRST_SEED + games {
        let setup = setup(s, seed);
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        session.history = None;
        // Full views through a cache kept across the game: the digest proves cached views
        // are byte-identical to fresh ones (compare with an older build's digest).
        let mut cache = legatum_core::ViewCache::default();
        for _ in 0..turns {
            if session.state.finished {
                break;
            }
            let public = legatum_core::public_view(&session.state);
            for p in &session.state.players {
                let view = legatum_core::player_view_cached(
                    &session.state,
                    p.id,
                    &public,
                    legatum_core::ViewDetail::Full,
                    &mut cache,
                );
                let json = serde_json::to_string(&view).expect("view JSON");
                for b in json.bytes() {
                    digest = (digest ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
                }
            }
            let orders = session.ai_orders();
            session.step(&orders);
        }
    }
    digest
}

/// Wall time of one game and its slowest AI-planning turn, from the calling thread.
fn timed_game(s: Scenario, seed: u64) -> (f64, f64, u64) {
    let setup = setup(s, seed);
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let (start, mut slowest) = (Instant::now(), 0f64);
    let mut turns = 0;
    while !session.state.finished && turns < cap {
        let t = Instant::now();
        let orders = session.ai_orders();
        slowest = slowest.max(t.elapsed().as_secs_f64());
        session.step(&orders);
        turns += 1;
    }
    (start.elapsed().as_secs_f64(), slowest, session.state.hash())
}

/// One game's latency: seats planned in parallel (from a plain thread, as a server or a
/// single game runs) against one at a time (inside a one-thread pool, as batch sims run).
fn latency(s: Scenario, seed: u64) {
    let (par, par_turn, par_hash) = timed_game(s, seed);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool");
    let (seq, seq_turn, seq_hash) = pool.install(|| timed_game(s, seed));
    assert_eq!(par_hash, seq_hash, "parallel seats must play the same game");
    println!(
        "{:<20} one game seed {seed}: parallel seats {par:.2}s (slowest AI turn {:.0} ms), \
         sequential {seq:.2}s ({:.0} ms); same final state",
        s.name,
        par_turn * 1000.0,
        seq_turn * 1000.0
    );
}

/// The search AI's game: seat 0 Hard+Search (k3 h20 d4 w), seat 1 plain Hard, through the
/// same loop as `Session::ai_orders` (a view cache per seat, decision-only views). Returns
/// (turns, final state hash, digest of every search event).
fn search_game(seed: u64) -> (u32, u64, u64) {
    use legatum_core::{PlayerId, ViewCache, ViewDetail, player_view_cached, public_view};
    let setup = setup(SMALL, seed);
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut state = session.state;
    let config = legatum_ai::search::SearchConfig {
        fitted_plan: true,
        ..Default::default()
    };
    let mut brains: Vec<Box<dyn legatum_ai::Brain>> = state
        .players
        .iter()
        .map(|p| -> Box<dyn legatum_ai::Brain> {
            let seed = legatum_wasm::session::brain_seed(state.setup.seed, p.id);
            let hard = legatum_ai::ScriptedBrain::with_difficulty(
                p.faction,
                seed,
                p.id,
                AiDifficulty::Hard,
            );
            if p.id == PlayerId(0) {
                Box::new(legatum_ai::search::SearchBrain::new(hard, config))
            } else {
                Box::new(hard)
            }
        })
        .collect();
    let mut caches: Vec<ViewCache> = state.players.iter().map(|_| ViewCache::default()).collect();
    let mut turns = 0;
    while !state.finished && turns < cap {
        let public = public_view(&state);
        let mut orders = legatum_wasm::session::OrderMap::new();
        for p in state.players.iter().filter(|p| p.alive) {
            let i = p.id.0 as usize;
            let view =
                player_view_cached(&state, p.id, &public, ViewDetail::Decisions, &mut caches[i]);
            let o = brains[i].orders(&view);

            orders.insert(p.id, o);
        }
        state = legatum_core::resolve_turn(&state, &orders).0;
        turns += 1;
    }
    let events = serde_json::to_string(brains[0].search_events()).expect("search events JSON");
    let digest = events.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    (turns, state.hash(), digest)
}

fn search_report(games: u64) {
    let (wall0, cpu0) = (Instant::now(), cpu_seconds());
    let (mut turns, mut digest) = (0u64, 0xcbf2_9ce4_8422_2325u64);
    for seed in FIRST_SEED..FIRST_SEED + games {
        let (t, hash, events) = search_game(seed);
        turns += u64::from(t);
        digest = (digest ^ hash ^ events ^ u64::from(t)).wrapping_mul(0x0100_0000_01b3);
    }
    let (wall, cpu) = (wall0.elapsed().as_secs_f64(), cpu_seconds() - cpu0);
    let threads = std::env::var("LEGATUM_ROLLOUT_THREADS").unwrap_or_else(|_| "all".into());
    println!(
        "search-small-k3h20d4w rollout-threads={threads} games={games} turns={turns} \
         wall={wall:.2}s cpu={cpu:.2}s wall-s/game={:.3} cpu-s/game={:.3} digest={digest:016x}",
        wall / games as f64,
        cpu / games as f64,
    );
}

fn cpu_seconds() -> f64 {
    // SAFETY: getrusage writes into a zeroed struct we own.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    let t = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 / 1e6;
    t(usage.ru_utime) + t(usage.ru_stime)
}

struct Run {
    wall: f64,
    cpu: f64,
    turns: u64,
    digest: u64,
}

fn run(s: Scenario, games: u64, threads: usize) -> Run {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("thread pool");
    let (wall0, cpu0) = (Instant::now(), cpu_seconds());
    let results: Vec<(u32, u64)> = pool.install(|| {
        (0..games)
            .into_par_iter()
            .map(|i| play(s, FIRST_SEED + i))
            .collect()
    });
    let (wall, cpu) = (wall0.elapsed().as_secs_f64(), cpu_seconds() - cpu0);
    let digest = results.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, (t, x)| {
        (h ^ x ^ u64::from(*t)).wrapping_mul(0x0100_0000_01b3)
    });
    Run {
        wall,
        cpu,
        turns: results.iter().map(|(t, _)| u64::from(*t)).sum(),
        digest,
    }
}

fn report(s: Scenario, games: u64, threads: usize, r: &Run) {
    println!(
        "{:<20} threads={:<2} games={:<4} turns={:<6} wall={:>7.2}s cpu={:>7.2}s \
         games/s={:>7.2} turns/s={:>8.0} ms/turn(cpu)={:>6.3} cpu-s/game={:>6.3} digest={:016x}",
        s.name,
        threads,
        games,
        r.turns,
        r.wall,
        r.cpu,
        games as f64 / r.wall,
        r.turns as f64 / r.wall,
        r.cpu * 1000.0 / r.turns as f64,
        r.cpu / games as f64,
        r.digest,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let which = args.first().map_or("all", String::as_str);
    let games: u64 = args.get(1).map_or(0, |g| g.parse().expect("games"));
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let threads: usize = args.get(2).map_or(cores, |t| t.parse().expect("threads"));
    // (scenario, single-threaded games, games per worker thread in the parallel run)
    let scenarios: Vec<(Scenario, u64, u64)> = match which {
        "duel" => vec![(DUEL, 32, 64)],
        "small" => vec![(SMALL, 16, 16)],
        "eight" => vec![(EIGHT, 8, 4)],
        "all" => vec![(DUEL, 32, 64), (SMALL, 16, 16), (EIGHT, 8, 4)],
        "views" => {
            println!("view-digest duel={:016x}", view_digest(DUEL, 4, 80));
            println!("view-digest eight={:016x}", view_digest(EIGHT, 1, 150));
            return;
        }
        "latency" => {
            latency(DUEL, FIRST_SEED);
            latency(EIGHT, FIRST_SEED);
            latency(EIGHT, FIRST_SEED + 1);
            return;
        }
        "search" => {
            search_report(if games == 0 { 4 } else { games });
            return;
        }
        "rollout" => {
            rollout_report(DUEL, 30);
            rollout_report(SMALL, 60);
            rollout_report(EIGHT, 100);
            return;
        }
        other => {
            panic!(
                "unknown scenario {other}: duel, small, eight, all, search, views, latency or rollout"
            )
        }
    };
    let mut load = [0f64; 3];
    // SAFETY: getloadavg fills at most 3 doubles of the array we own.
    unsafe { libc::getloadavg(load.as_mut_ptr(), 3) };
    println!("cores={cores} load-average(1m)={:.1}", load[0]);
    for (s, default_games, per_thread) in scenarios {
        let games = if games == 0 { default_games } else { games };
        let single = run(s, games, 1);
        report(s, games, 1, &single);
        if threads > 1 {
            let many = games.max(threads as u64 * per_thread);
            let multi = run(s, many, threads);
            report(s, many, threads, &multi);
        }
    }
}

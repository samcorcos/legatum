//! Functional probe of the hold-to-win rule with AI seats: how often VP holds start, break and
//! complete, and how games end. Solo teams, rotated factions, every seat Hard (or Normal).
//! hold_ai_probe <games> <players> [seed=9500001] [threads=4] [hard|normal]
//! One line per game, then totals. Ad hoc evidence only; formal telemetry lives in the runner.
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, TeamId};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;

#[derive(Default, Clone)]
struct Tally {
    started: u32,
    broken: u32,
    /// Broken holds by turns held.
    broken_after: Vec<u32>,
    won_by_hold: u32,
    won_other: u32,
    drawn: u32,
    capped: u32,
    turns: u32,
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let games: u64 = a[1].parse().unwrap();
    let players: usize = a[2].parse().unwrap();
    let seed: u64 = a.get(3).map_or(9_500_001, |x| x.parse().unwrap());
    let threads: usize = a.get(4).map_or(4, |x| x.parse().unwrap());
    let difficulty = match a.get(5).map(String::as_str) {
        Some("normal") => AiDifficulty::Normal,
        _ => AiDifficulty::Hard,
    };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    let results: Vec<(u64, Tally, String)> = pool.install(|| {
        (0..games)
            .into_par_iter()
            .map(|g| run(seed + g, players, difficulty))
            .collect()
    });
    let mut total = Tally::default();
    for (s, t, line) in &results {
        println!("seed {s}: {line}");
        total.started += t.started;
        total.broken += t.broken;
        total.broken_after.extend(&t.broken_after);
        total.won_by_hold += t.won_by_hold;
        total.won_other += t.won_other;
        total.drawn += t.drawn;
        total.capped += t.capped;
        total.turns += t.turns;
    }
    total.broken_after.sort();
    println!(
        "TOTAL games {games} players {players} {difficulty:?}: holds started {} broken {} \
         (turns held {:?}) | won by hold {} other wins {} draws {} capped {} | mean length {}",
        total.started,
        total.broken,
        total.broken_after,
        total.won_by_hold,
        total.won_other,
        total.drawn,
        total.capped,
        total.turns / games.max(1) as u32
    );
}

fn run(seed: u64, players: usize, difficulty: AiDifficulty) -> (u64, Tally, String) {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    setup.turn_limit = None;
    let (w, h) = if players <= 2 { (40, 26) } else { (48, 32) };
    setup.width = w;
    setup.height = h;
    for (seat, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(seat + seed as usize) % Faction::ALL.len()];
        p.team = TeamId(seat as u8);
        p.difficulty = difficulty;
    }
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 2;
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut t = Tally::default();
    let mut log = Vec::new();
    while !session.state.finished && session.state.turn < cap {
        let orders = session.ai_orders();
        let turn = session.state.turn;
        for e in session.step(&orders) {
            match e {
                Event::VictoryHoldStarted {
                    team, vp, target, ..
                } => {
                    t.started += 1;
                    log.push(format!("t{turn} T{} start {vp}/{target}", team.0));
                }
                Event::VictoryHoldBroken {
                    team,
                    turns_held,
                    vp,
                    ..
                } => {
                    t.broken += 1;
                    t.broken_after.push(turns_held);
                    log.push(format!(
                        "t{turn} T{} broke after {turns_held} at {vp}",
                        team.0
                    ));
                }
                Event::GameWon { team, reason } => {
                    if reason.contains(" held ") {
                        t.won_by_hold += 1;
                    } else {
                        t.won_other += 1;
                    }
                    log.push(format!("t{turn} T{} won: {reason}", team.0));
                }
                Event::GameDrawn { reason } => {
                    t.drawn += 1;
                    log.push(format!("t{turn} draw: {reason}"));
                }
                _ => {}
            }
        }
    }
    if !session.state.finished {
        t.capped = 1;
        log.push(format!("capped at t{}", session.state.turn));
    }
    t.turns = session.state.turn;
    (seed, t, log.join("; "))
}

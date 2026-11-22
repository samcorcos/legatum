//! How often the AIs actually use aircraft, from a directory of `legatum-sim --tape-out` tapes.
//!
//!   air_usage <tape-dir>
//!
//! Replays every tape (no AI) and counts, from the orders and the engine's own events: games in
//! which any empire reached the Flight era, built a Fighter or Bomber, ordered a Bomber or Fighter
//! strike, saw a strike land, and saw a Fighter meet a strike; plus totals per game and the median
//! turn of the first Flight technology and of the first aircraft strike. Barbarians have no aircraft.
//! Run on the same binary that recorded the tapes (a replay under other rules diverges).
use legatum_core::Order;
use legatum_core::{Event, ProductionItem, TechId, UnitKind, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

#[derive(Default)]
struct Game {
    turns: u32,
    flight_turn: Option<u32>,
    first_strike_turn: Option<u32>,
    built: BTreeMap<&'static str, u32>,
    strike_orders: BTreeMap<&'static str, u32>,
    strikes_landed: BTreeMap<&'static str, u32>,
    interceptions: u32,
    rebases: u32,
}

fn name(kind: UnitKind) -> &'static str {
    if kind == UnitKind::Fighter {
        "Fighter"
    } else {
        "Bomber"
    }
}

fn replay(path: &std::path::Path) -> Result<Game, String> {
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    let mut g = Game::default();
    for t in &tape.turns {
        let turn = state.turn;
        // Aircraft ids and kinds at the start of the turn: orders and ranged battles name them.
        let planes: BTreeMap<_, _> = state.aircraft.iter().map(|a| (a.id, a.kind)).collect();
        for list in t.orders.values() {
            for o in list {
                if let Order::AirStrike { plane, .. } = o
                    && let Some(kind) = planes.get(plane)
                {
                    *g.strike_orders.entry(name(*kind)).or_default() += 1;
                    g.first_strike_turn.get_or_insert(turn);
                }
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::TechResearched {
                    tech: TechId::Flight,
                    ..
                } => {
                    g.flight_turn.get_or_insert(turn);
                }
                Event::ProductionCompleted {
                    item: ProductionItem::Unit { unit },
                    ..
                } if legatum_core::data::is_aircraft(*unit) => {
                    *g.built.entry(name(*unit)).or_default() += 1;
                }
                Event::Battle {
                    attacker,
                    defender,
                    ranged,
                    ..
                } => {
                    let Some(kind) = attacker.unit().and_then(|id| planes.get(&id)) else {
                        continue;
                    };
                    let met = matches!(
                        defender,
                        legatum_core::events::Defender::Unit {
                            unit_kind: UnitKind::Fighter,
                            ..
                        }
                    );
                    if met && !ranged {
                        g.interceptions += 1;
                    } else if *ranged {
                        *g.strikes_landed.entry(name(*kind)).or_default() += 1;
                    }
                }
                Event::AircraftRebased { .. } => g.rebases += 1,
                _ => {}
            }
        }
        state = next;
    }
    g.turns = state.turn;
    Ok(g)
}

fn median(mut v: Vec<u32>) -> String {
    if v.is_empty() {
        return "-".into();
    }
    v.sort_unstable();
    v[v.len() / 2].to_string()
}

fn main() {
    let dir = std::env::args().nth(1).expect("tapes dir");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let games: Vec<Game> = files
        .iter()
        .map(|f| replay(f).unwrap_or_else(|e| panic!("{}: {e}", f.display())))
        .collect();
    let n = games.len().max(1) as f64;
    let share = |pred: &dyn Fn(&Game) -> bool| games.iter().filter(|g| pred(g)).count();
    let pct = |c: usize| format!("{c} of {} ({:.1}%)", games.len(), 100.0 * c as f64 / n);
    let total = |f: &dyn Fn(&Game) -> u32| games.iter().map(f).sum::<u32>();
    println!(
        "games {}   median length {} turns",
        games.len(),
        median(games.iter().map(|g| g.turns).collect())
    );
    println!(
        "any empire knew Flight: {}   median first turn {}",
        pct(share(&|g| g.flight_turn.is_some())),
        median(games.iter().filter_map(|g| g.flight_turn).collect())
    );
    for kind in ["Bomber", "Fighter"] {
        println!(
            "{kind}: built in {}; strike ordered in {}; strike landed in {}   (totals: {} built, {} strike orders, {} strikes landed)",
            pct(share(&|g| g.built.contains_key(kind))),
            pct(share(&|g| g.strike_orders.contains_key(kind))),
            pct(share(&|g| g.strikes_landed.contains_key(kind))),
            total(&|g| g.built.get(kind).copied().unwrap_or(0)),
            total(&|g| g.strike_orders.get(kind).copied().unwrap_or(0)),
            total(&|g| g.strikes_landed.get(kind).copied().unwrap_or(0)),
        );
    }
    println!(
        "a Fighter met a strike in {}   (total {})",
        pct(share(&|g| g.interceptions > 0)),
        total(&|g| g.interceptions)
    );
    println!(
        "rebases (resolved): {} in total, {:.2} per game",
        total(&|g| g.rebases),
        f64::from(total(&|g| g.rebases)) / n
    );
    println!(
        "any aircraft strike ordered in {}   median first turn {}",
        pct(share(&|g| g.first_strike_turn.is_some())),
        median(games.iter().filter_map(|g| g.first_strike_turn).collect())
    );
}

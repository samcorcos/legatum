//! Dead wars in a directory of `legatum-sim --tape-out` tapes: wars nobody fights, how long
//! they last and how their Peace offers fare.
//!
//! cargo run --release -p legatum-sim --example war_peace_metrics -- <tape-dir> [threads] [dead=10]
//!
//! Replays every tape (no AI). A war is one stretch of War between two teams. A war turn is
//! "dead" when no battle between the two sides (unit or city, either direction) happened in
//! the last `dead` turns (counting from the war's start). Per war: turns, battles, captures
//! between the two sides, its longest stretch without a battle, how it ended (Peace, or still
//! running when one side died or the game ended), and the Peace offers made between the two
//! during the war and refused. Prints one JSON summary.
use legatum_core::diplomacy::{RelationState, TreatyKind};
use legatum_core::events::Defender;
use legatum_core::{Event, GameState, Order, PlayerId, TeamId, resolve_turn};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone, Default)]
struct War {
    turns: u32,
    dead_turns: u32,
    battles: u32,
    captures: u32,
    last_battle: u32,
    longest_gap: u32,
    offers: u32,
    refused: u32,
    /// Offers made while the war was dead (no battle in `dead` turns), and of those refused.
    dead_offers: u32,
    dead_refused: u32,
    peace: bool,
}

#[derive(Default)]
struct Game {
    turns: u32,
    wars: Vec<War>,
    captures: u32,
    peaces: u32,
}

fn team(state: &GameState, p: PlayerId) -> Option<TeamId> {
    (!p.is_barbarian()).then(|| state.team_of(p))
}

fn key(a: TeamId, b: TeamId) -> (TeamId, TeamId) {
    if a < b { (a, b) } else { (b, a) }
}

fn replay(path: &std::path::Path, dead: u32) -> Result<Game, String> {
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    let mut g = Game::default();
    let mut open: BTreeMap<(TeamId, TeamId), War> = BTreeMap::new();
    for t in &tape.turns {
        let turn = state.turn;
        // Offers this turn between two teams at war.
        for (p, orders) in &t.orders {
            for o in orders {
                if let Order::ProposeTreaty {
                    to,
                    kind: TreatyKind::Peace,
                } = o
                    && let Some(me) = team(&state, *p)
                    && let Some(w) = open.get_mut(&key(me, *to))
                {
                    w.offers += 1;
                    if turn.saturating_sub(w.last_battle) >= dead {
                        w.dead_offers += 1;
                    }
                }
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::Battle {
                    attacker_owner,
                    defender,
                    ..
                } => {
                    let d = match defender {
                        Defender::Unit { owner, .. } | Defender::City { owner, .. } => *owner,
                    };
                    if let (Some(a), Some(b)) = (team(&state, *attacker_owner), team(&state, d))
                        && let Some(w) = open.get_mut(&key(a, b))
                    {
                        w.battles += 1;
                        w.last_battle = turn;
                    }
                }
                Event::CityCaptured { from, to, .. } => {
                    g.captures += 1;
                    if let (Some(a), Some(b)) = (team(&state, *from), team(&state, *to))
                        && let Some(w) = open.get_mut(&key(a, b))
                    {
                        w.captures += 1;
                        w.last_battle = turn;
                    }
                }
                Event::ProposalRejected { proposal, .. } if proposal.kind == TreatyKind::Peace => {
                    if let Some(w) = open.get_mut(&key(proposal.from, proposal.to)) {
                        w.refused += 1;
                        if turn.saturating_sub(w.last_battle) >= dead {
                            w.dead_refused += 1;
                        }
                    }
                }
                Event::PeaceEstablished { .. } => g.peaces += 1,
                _ => {}
            }
        }
        state = next;
        // Wars as they stand after the turn.
        let relations = state
            .diplomacy
            .as_ref()
            .map_or(&[][..], |d| &d.relations[..]);
        let now: Vec<(TeamId, TeamId)> = relations
            .iter()
            .filter(|r| r.state == RelationState::War)
            .filter(|r| state.team_alive(r.a) && state.team_alive(r.b))
            .map(|r| key(r.a, r.b))
            .collect();
        let ended: Vec<_> = open.keys().filter(|k| !now.contains(k)).copied().collect();
        for k in ended {
            let mut w = open.remove(&k).expect("open war");
            w.peace = relations
                .iter()
                .any(|r| key(r.a, r.b) == k && r.state != RelationState::War)
                && state.team_alive(k.0)
                && state.team_alive(k.1);
            g.wars.push(w);
        }
        for k in now {
            let w = open.entry(k).or_insert_with(|| War {
                last_battle: turn,
                ..War::default()
            });
            w.turns += 1;
            let gap = turn.saturating_sub(w.last_battle);
            w.longest_gap = w.longest_gap.max(gap);
            if gap >= dead {
                w.dead_turns += 1;
            }
        }
    }
    g.wars.extend(open.into_values());
    g.turns = state.turn;
    Ok(g)
}

fn pct(n: u32, d: u32) -> f64 {
    if d == 0 {
        0.0
    } else {
        (1000.0 * n as f64 / d as f64).round() / 10.0
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let threads: usize = args.get(2).map_or(8, |s| s.parse().unwrap());
    let dead: u32 = args.get(3).map_or(10, |s| s.parse().unwrap());
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let chunks: Vec<Vec<std::path::PathBuf>> = (0..threads)
        .map(|i| paths.iter().skip(i).step_by(threads).cloned().collect())
        .collect();
    let games: Vec<Game> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|c| {
                s.spawn(move || {
                    c.iter()
                        .map(|p| replay(p, dead).unwrap())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let n = games.len().max(1) as f64;
    let wars: Vec<&War> = games.iter().flat_map(|g| &g.wars).collect();
    let sum = |f: &dyn Fn(&War) -> u32| wars.iter().map(|w| f(w)).sum::<u32>();
    let war_turns = sum(&|w| w.turns);
    let dead_turns = sum(&|w| w.dead_turns);
    let long_dead = |gap: u32| wars.iter().filter(|w| w.longest_gap >= gap).count() as u32;
    let dead20: Vec<&&War> = wars.iter().filter(|w| w.longest_gap >= 20).collect();
    let mut durations: Vec<u32> = wars.iter().map(|w| w.turns).collect();
    durations.sort_unstable();
    let mut dead_durations: Vec<u32> = dead20.iter().map(|w| w.turns).collect();
    dead_durations.sort_unstable();
    let median = |v: &[u32]| v.get(v.len() / 2).copied().unwrap_or(0);
    let offers = sum(&|w| w.offers);
    let refused = sum(&|w| w.refused);
    let dead_offers = sum(&|w| w.dead_offers);
    let dead_refused = sum(&|w| w.dead_refused);
    let no_battle = wars.iter().filter(|w| w.battles == 0).count() as u32;
    let out = json!({
        "games": games.len(),
        "dead_after_turns": dead,
        "mean_game_turns": (games.iter().map(|g| g.turns as f64).sum::<f64>() / n * 10.0).round() / 10.0,
        "wars_per_game": (wars.len() as f64 / n * 100.0).round() / 100.0,
        "war_turns_per_game": (war_turns as f64 / n * 10.0).round() / 10.0,
        "dead_war_turns_per_game": (dead_turns as f64 / n * 10.0).round() / 10.0,
        "dead_share_percent": pct(dead_turns, war_turns),
        "median_war_turns": median(&durations),
        "wars_never_fought_percent": pct(no_battle, wars.len() as u32),
        "wars_with_20_dead_turns_percent": pct(long_dead(20), wars.len() as u32),
        "wars_with_40_dead_turns_percent": pct(long_dead(40), wars.len() as u32),
        "dead20_median_war_turns": median(&dead_durations),
        "dead20_ended_by_peace_percent": pct(dead20.iter().filter(|w| w.peace).count() as u32, dead20.len() as u32),
        "all_wars_ended_by_peace_percent": pct(wars.iter().filter(|w| w.peace).count() as u32, wars.len() as u32),
        "peace_offers_per_game": (offers as f64 / n * 10.0).round() / 10.0,
        "peace_refused_percent": pct(refused, offers),
        "dead_peace_offers_per_game": (dead_offers as f64 / n * 10.0).round() / 10.0,
        "dead_peace_refused_percent": pct(dead_refused, dead_offers),
        "peace_deals_per_game": (games.iter().map(|g| g.peaces).sum::<u32>() as f64 / n * 100.0).round() / 100.0,
        "captures_per_game": (games.iter().map(|g| g.captures).sum::<u32>() as f64 / n * 100.0).round() / 100.0,
        "captures_in_wars_per_game": (sum(&|w| w.captures) as f64 / n * 100.0).round() / 100.0,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

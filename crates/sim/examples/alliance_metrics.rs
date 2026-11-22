//! Alliance and war metrics from `legatum-sim --tape-out` tapes: alliances formed, the largest
//! alliance bloc at T50 to T250, wars, captures, and the share of wars with a capture.
//!
//!     cargo run --release -p legatum-sim --example alliance_metrics -- <tape-dir> [threads]
//!
//! Tapes come from `legatum-sim --players 8 --difficulty hard --barbarians true --tape-out DIR`
//! (see `tools/sim/flip-metrics.sh`).
use legatum_core::diplomacy::{RelationState, alliance_sides};
use legatum_core::{Event, PlayerId, TeamId, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Game {
    turns: u32,
    alliances: u32,
    wars: u32,
    captures: u32,
    /// (war count, wars with a capture between the pair, peace-ended wars, peace-ended with capture)
    war_total: u32,
    war_capt: u32,
    war_ended: u32,
    war_ended_capt: u32,
    /// max bloc size (teams) sampled at turns 50,100,150,200,250
    bloc: [u32; 5],
    alliance_pairs_turns: u64,
}

fn replay(path: &std::path::Path) -> Result<Game, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tape = Tape::from_json(&text)?;
    let mut state = tape.initial_state();
    let mut g = Game::default();
    // open wars: pair -> captured between them so far
    let mut open: BTreeMap<(TeamId, TeamId), bool> = BTreeMap::new();
    let team_of: BTreeMap<PlayerId, TeamId> =
        state.players.iter().map(|p| (p.id, p.team)).collect();
    for t in &tape.turns {
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::TreatyActivated { relation, .. }
                    if relation.state == RelationState::Alliance =>
                {
                    g.alliances += 1;
                }
                Event::WarDeclared { relation } => {
                    g.wars += relation.declarers.len() as u32;
                    open.entry((relation.a.min(relation.b), relation.a.max(relation.b)))
                        .or_insert(false);
                    g.war_total += 1;
                }
                Event::PeaceEstablished { relation } => {
                    let k = (relation.a.min(relation.b), relation.a.max(relation.b));
                    if let Some(c) = open.remove(&k) {
                        g.war_ended += 1;
                        g.war_ended_capt += u32::from(c);
                        g.war_capt += u32::from(c);
                    }
                }
                Event::CityCaptured { from, to, .. } => {
                    g.captures += 1;
                    if let (Some(a), Some(b)) = (team_of.get(from), team_of.get(to)) {
                        let k = (*a.min(b), *a.max(b));
                        if let Some(c) = open.get_mut(&k) {
                            *c = true;
                        }
                    }
                }
                _ => {}
            }
        }
        state = next;
        let alive: BTreeSet<TeamId> = state
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
            .map(|p| p.team)
            .collect();
        let rels: &[legatum_core::diplomacy::Relation] = state
            .diplomacy
            .as_ref()
            .map_or(&[], |d| d.relations.as_slice());
        let sides = alliance_sides(rels);
        let biggest = sides
            .iter()
            .map(|s| s.iter().filter(|t| alive.contains(t)).count() as u32)
            .max()
            .unwrap_or(0)
            .max(1);
        for (i, at) in [50u32, 100, 150, 200, 250].iter().enumerate() {
            if turn == *at {
                g.bloc[i] = biggest;
            }
        }
        g.alliance_pairs_turns += rels
            .iter()
            .filter(|r| r.state == RelationState::Alliance)
            .count() as u64;
    }
    for (_, c) in open {
        g.war_capt += u32::from(c);
    }
    g.turns = state.turn;
    Ok(g)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut paths: Vec<_> = std::fs::read_dir(&args[1])
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let threads: usize = args.get(2).map_or(6, |s| s.parse().unwrap());
    let chunks: Vec<Vec<std::path::PathBuf>> = (0..threads)
        .map(|i| paths.iter().skip(i).step_by(threads).cloned().collect())
        .collect();
    let games: Vec<Game> = std::thread::scope(|s| {
        let hs: Vec<_> = chunks
            .into_iter()
            .map(|c| s.spawn(move || c.iter().map(|p| replay(p).unwrap()).collect::<Vec<_>>()))
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let n = games.len().max(1) as f64;
    let sum = |f: &dyn Fn(&Game) -> u32| games.iter().map(|g| f(g) as f64).sum::<f64>();
    println!("games {}", games.len());
    println!("mean resolved turns {:.1}", sum(&|g| g.turns) / n);
    println!("alliances formed per game {:.2}", sum(&|g| g.alliances) / n);
    println!(
        "alliance-turns per game {:.1}",
        games
            .iter()
            .map(|g| g.alliance_pairs_turns as f64)
            .sum::<f64>()
            / n
    );
    println!(
        "wars (war relations opened) per game {:.2}; declarations {:.2}",
        sum(&|g| g.war_total) / n,
        sum(&|g| g.wars) / n
    );
    println!("captures per game {:.2}", sum(&|g| g.captures) / n);
    println!(
        "wars with at least one capture between the pair: {:.1}% of {:.0} wars",
        100.0 * sum(&|g| g.war_capt) / sum(&|g| g.war_total).max(1.0),
        sum(&|g| g.war_total)
    );
    println!(
        "wars ended by Peace: {:.2} per game; with a capture before the peace {:.1}%",
        sum(&|g| g.war_ended) / n,
        100.0 * sum(&|g| g.war_ended_capt) / sum(&|g| g.war_ended).max(1.0)
    );
    for (i, at) in [50, 100, 150, 200, 250].iter().enumerate() {
        let mut v: Vec<u32> = games
            .iter()
            .filter(|g| g.turns > *at)
            .map(|g| g.bloc[i])
            .collect();
        let mean = v.iter().sum::<u32>() as f64 / v.len().max(1) as f64;
        let mx = v.iter().max().copied().unwrap_or(0);
        v.sort_unstable();
        let median = if v.is_empty() {
            0.0
        } else {
            (f64::from(v[(v.len() - 1) / 2]) + f64::from(v[v.len() / 2])) / 2.0
        };
        println!(
            "largest alliance bloc (teams) at T{at}: median {median:.1}, mean {mean:.2}, max {mx}, games reaching it {}",
            v.len()
        );
    }
}

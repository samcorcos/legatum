//! The Celts' wars and sacks in a directory of `legatum-sim --tape-out` tapes.
//!
//! cargo run --release -p legatum-sim --example celts_wars -- <tape-dir> [threads]
//!
//! Replays every tape (no AI) and, for each Celtic seat, records from the engine's own events:
//! wars declared (target faction, the target's VP rank, the army ratio at the declaration),
//! cities sacked (victim faction, whether it led the public VP race, the victim's army against
//! the Celts' that turn), Raise the Warbands lost, and whether the Celts won. Prints the win
//! rate by number of sacks and the sacks by victim and by army ratio.
use legatum_core::rules::is_military;
use legatum_core::scoring::team_vp;
use legatum_core::{Event, Faction, GameState, PlayerId, TeamId, data, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

#[derive(Default, Clone)]
struct Seat {
    won: bool,
    /// (turn, target faction, target led VP, target army per 100 of ours)
    wars: Vec<(u32, Faction, bool, i32)>,
    /// (turn, victim faction, victim led VP, victim army per 100 of ours, victim VP)
    sacks: Vec<(u32, Faction, bool, i32, u32)>,
    goal_lost: u32,
    gold: i32,
    /// Wars the Celts declared: (target team, declared turn, sacks in it, ended turn).
    ledger: Vec<(TeamId, u32, u32, Option<u32>)>,
    /// Wars others declared on the Celts, and Celtic cities lost to capture.
    wars_on: u32,
    lost: u32,
    /// Camps cleared by the Celts' own team with no Gold (their own warbands), and by others.
    own_camps_cleared: u32,
    /// Warband camps raised by this seat's sacks.
    camps_raised: u32,
    camps_cleared_by_others: u32,
}

fn army(state: &GameState, team: TeamId) -> i32 {
    state
        .units
        .values()
        .filter(|u| is_military(u.kind) && state.team_of(u.owner) == team)
        .map(|u| {
            let s = data::unit_stats_for(state.player(u.owner).faction, u.kind);
            s.strength.max(s.ranged) * u.hp.clamp(0, 100) / 100
        })
        .sum()
}

fn leads(state: &GameState, team: TeamId) -> bool {
    let mine = team_vp(state, team);
    state
        .teams()
        .into_iter()
        .all(|t| t == team || team_vp(state, t) < mine)
}

fn replay(path: &std::path::Path) -> Result<Vec<Seat>, String> {
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    let celts: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.faction == Faction::Celts)
        .map(|p| p.id)
        .collect();
    let mut seats: BTreeMap<PlayerId, Seat> = celts.iter().map(|p| (*p, Seat::default())).collect();
    for t in &tape.turns {
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::PeaceEstablished { relation } => {
                    for seat in seats.values_mut() {
                        for w in seat.ledger.iter_mut().filter(|w| w.3.is_none()) {
                            if (relation.a == w.0 || relation.b == w.0)
                                && relation.effective_turn >= w.1
                            {
                                w.3 = Some(relation.effective_turn);
                            }
                        }
                    }
                }
                Event::CityCaptured { from, .. } => {
                    if let Some(seat) = seats.get_mut(from) {
                        seat.lost += 1;
                    }
                }
                Event::WarDeclared { relation } => {
                    for (pid, seat) in seats.iter_mut() {
                        let mine = state.team_of(*pid);
                        if (relation.a == mine || relation.b == mine)
                            && !relation.declarers.contains(&mine)
                        {
                            seat.wars_on += 1;
                        }
                    }
                    for d in &relation.declarers {
                        let target = if *d == relation.a {
                            relation.b
                        } else {
                            relation.a
                        };
                        for (pid, seat) in seats.iter_mut() {
                            if state.team_of(*pid) == *d {
                                let theirs = army(&state, target);
                                let ours = army(&state, *d).max(1);
                                let faction = state
                                    .players
                                    .iter()
                                    .find(|p| p.team == target)
                                    .map_or(Faction::Celts, |p| p.faction);
                                seat.wars.push((
                                    turn,
                                    faction,
                                    leads(&state, target),
                                    theirs * 100 / ours,
                                ));
                                seat.ledger.push((target, turn, 0, None));
                            }
                        }
                    }
                }
                Event::CityRazed {
                    from,
                    by,
                    sack_gold,
                    camp,
                    ..
                } => {
                    if let Some(seat) = seats.get_mut(by) {
                        let vteam = state.team_of(*from);
                        let ours = army(&state, state.team_of(*by)).max(1);
                        seat.sacks.push((
                            turn,
                            state.player(*from).faction,
                            leads(&state, vteam),
                            army(&state, vteam) * 100 / ours,
                            team_vp(&state, vteam),
                        ));
                        seat.gold += sack_gold.unwrap_or(0);
                        seat.camps_raised += u32::from(camp.is_some());
                        if let Some(w) = seat
                            .ledger
                            .iter_mut()
                            .rev()
                            .find(|w| w.0 == vteam && w.3.is_none())
                        {
                            w.2 += 1;
                        }
                    }
                }
                Event::BarbarianCampCleared { by, gold, .. } => {
                    for (pid, seat) in seats.iter_mut() {
                        if state.team_of(*pid) == state.team_of(*by) && *gold == 0 {
                            seat.own_camps_cleared += 1;
                        } else if state.team_of(*pid) != state.team_of(*by)
                            && !by.is_barbarian()
                            && *gold == 0
                        {
                            seat.camps_cleared_by_others += 1;
                        }
                    }
                }
                Event::FactionGoalLost {
                    player,
                    faction: Faction::Celts,
                    ..
                } => {
                    if let Some(seat) = seats.get_mut(player) {
                        seat.goal_lost += 1;
                    }
                }
                _ => {}
            }
        }
        state = next;
    }
    if let Some(w) = state.winner {
        for (pid, seat) in seats.iter_mut() {
            seat.won = state.team_of(*pid) == w;
        }
    }
    Ok(seats.into_values().collect())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    let threads: usize = args.get(2).map_or(6, |s| s.parse().unwrap());
    let chunks: Vec<Vec<std::path::PathBuf>> = (0..threads)
        .map(|i| paths.iter().skip(i).step_by(threads).cloned().collect())
        .collect();
    let seats: Vec<Seat> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|c| {
                s.spawn(move || {
                    c.iter()
                        .flat_map(|p| replay(p).unwrap())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let n = seats.len().max(1) as f64;
    let wins = seats.iter().filter(|s| s.won).count();
    println!(
        "celtic seats {} wins {} ({:.1}%)",
        seats.len(),
        wins,
        100.0 * wins as f64 / n
    );
    let wars: usize = seats.iter().map(|s| s.wars.len()).sum();
    let sacks: usize = seats.iter().map(|s| s.sacks.len()).sum();
    println!(
        "wars declared per seat {:.2}, sacks per seat {:.2}, Gold taken per seat {:.0}, Warbands lost per seat {:.2}",
        wars as f64 / n,
        sacks as f64 / n,
        seats.iter().map(|s| s.gold as f64).sum::<f64>() / n,
        seats.iter().map(|s| s.goal_lost as f64).sum::<f64>() / n
    );
    println!("win rate by number of sacks");
    for k in 0..=6u32 {
        let rows: Vec<_> = seats
            .iter()
            .filter(|s| s.sacks.len().min(6) as u32 == k)
            .collect();
        if rows.is_empty() {
            continue;
        }
        println!(
            "  {}{}: {} seats, {:.1}% won",
            k,
            if k == 6 { "+" } else { "" },
            rows.len(),
            100.0 * rows.iter().filter(|s| s.won).count() as f64 / rows.len() as f64
        );
    }
    println!("win rate by number of wars declared");
    for k in 0..=5u32 {
        let rows: Vec<_> = seats
            .iter()
            .filter(|s| s.wars.len().min(5) as u32 == k)
            .collect();
        if rows.is_empty() {
            continue;
        }
        println!(
            "  {}{}: {} seats, {:.1}% won, {:.2} sacks, {:.2} Warbands lost",
            k,
            if k == 5 { "+" } else { "" },
            rows.len(),
            100.0 * rows.iter().filter(|s| s.won).count() as f64 / rows.len() as f64,
            rows.iter().map(|s| s.sacks.len() as f64).sum::<f64>() / rows.len() as f64,
            rows.iter().map(|s| s.goal_lost as f64).sum::<f64>() / rows.len() as f64,
        );
    }
    println!(
        "warband camps cleared per seat: by the Celts' own units {:.2}",
        seats
            .iter()
            .map(|s| s.own_camps_cleared as f64)
            .sum::<f64>()
            / n,
    );
    println!(
        "sack camps raised per seat {:.2}",
        seats.iter().map(|s| s.camps_raised as f64).sum::<f64>() / n,
    );
    let ledger: Vec<_> = seats.iter().flat_map(|s| s.ledger.iter()).collect();
    let paid = ledger.iter().filter(|w| w.2 > 0).count();
    let mut dur: Vec<u32> = ledger
        .iter()
        .map(|w| w.3.unwrap_or(400).saturating_sub(w.1))
        .collect();
    dur.sort_unstable();
    println!(
        "Celtic wars {}: {} with a sack ({:.0}%), {:.2} sacks per paid war; median length {} turns",
        ledger.len(),
        paid,
        100.0 * paid as f64 / ledger.len().max(1) as f64,
        ledger.iter().map(|w| w.2 as f64).sum::<f64>() / paid.max(1) as f64,
        dur.get(dur.len() / 2).copied().unwrap_or(0)
    );
    println!(
        "wars declared on the Celts per seat {:.2}, Celtic cities lost per seat {:.2}",
        seats.iter().map(|s| s.wars_on as f64).sum::<f64>() / n,
        seats.iter().map(|s| s.lost as f64).sum::<f64>() / n
    );
    for (label, pick) in [("seats with 0 sacks", 0usize), ("seats with 1+ sacks", 1)] {
        let rows: Vec<_> = seats
            .iter()
            .filter(|s| s.sacks.is_empty() == (pick == 0))
            .collect();
        println!(
            "  {label}: {} seats, {:.2} wars declared, {:.2} wars on them, {:.2} cities lost, {:.2} Warbands lost",
            rows.len(),
            rows.iter().map(|s| s.wars.len() as f64).sum::<f64>() / rows.len().max(1) as f64,
            rows.iter().map(|s| s.wars_on as f64).sum::<f64>() / rows.len().max(1) as f64,
            rows.iter().map(|s| s.lost as f64).sum::<f64>() / rows.len().max(1) as f64,
            rows.iter().map(|s| s.goal_lost as f64).sum::<f64>() / rows.len().max(1) as f64,
        );
    }
    let mut by_victim: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for s in &seats {
        for w in &s.wars {
            by_victim.entry(format!("{:?}", w.1)).or_default().0 += 1;
        }
        for k in &s.sacks {
            by_victim.entry(format!("{:?}", k.1)).or_default().1 += 1;
        }
    }
    println!("wars declared / sacks by target faction");
    for (f, (w, k)) in by_victim {
        println!("  {f}: {w} wars, {k} sacks");
    }
    let led = seats
        .iter()
        .flat_map(|s| s.sacks.iter())
        .filter(|k| k.2)
        .count();
    println!(
        "sacks of the strict VP leader: {} of {} ({:.0}%); wars on the leader: {} of {}",
        led,
        sacks,
        100.0 * led as f64 / sacks.max(1) as f64,
        seats
            .iter()
            .flat_map(|s| s.wars.iter())
            .filter(|w| w.2)
            .count(),
        wars
    );
    println!("victim army per 100 of the Celts' at the sack: quartiles");
    let mut r: Vec<i32> = seats
        .iter()
        .flat_map(|s| s.sacks.iter().map(|k| k.3))
        .collect();
    r.sort_unstable();
    if !r.is_empty() {
        println!(
            "  p25 {} median {} p75 {}",
            r[r.len() / 4],
            r[r.len() / 2],
            r[r.len() * 3 / 4]
        );
    }
    let mut r: Vec<i32> = seats
        .iter()
        .flat_map(|s| s.wars.iter().map(|w| w.3))
        .collect();
    r.sort_unstable();
    if !r.is_empty() {
        println!(
            "army per 100 at the declaration: p25 {} median {} p75 {}",
            r[r.len() / 4],
            r[r.len() / 2],
            r[r.len() * 3 / 4]
        );
    }
    let mut turns: Vec<u32> = seats
        .iter()
        .flat_map(|s| s.sacks.iter().map(|k| k.0))
        .collect();
    turns.sort_unstable();
    if !turns.is_empty() {
        println!(
            "sack turns: p25 {} median {} p75 {}",
            turns[turns.len() / 4],
            turns[turns.len() / 2],
            turns[turns.len() * 3 / 4]
        );
    }
}

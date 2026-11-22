//! War and conquest metrics from a directory of `legatum-sim --tape-out` tapes, per faction.
//!
//! cargo run --release -p legatum-sim --example war_metrics -- <tape-dir> [threads]
//!
//! Replays every tape (no AI) and counts, from the engine's own events: wars declared
//! (declaring team against target team, AI vs AI in all-AI runs), city captures, captures
//! lost back within 10 turns (retaken by the previous owner, or taken by anyone), and each
//! faction's first declaration and first capture turns. Barbarians never capture cities.
use legatum_core::{Event, Faction, PlayerId, TeamId, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::{BTreeMap, BTreeSet};

/// Seats, seats that declared, first declaration turns, wars declared, captures made,
/// first capture turns, cities lost.
type FactionStats = (u32, u32, Vec<u32>, u32, u32, Vec<u32>, u32);

#[derive(Default, Clone)]
struct Game {
    turns: u32,
    /// (turn, declaring team, target team)
    wars: Vec<(u32, TeamId, TeamId)>,
    /// (turn, city, from, to)
    captures: Vec<(u32, u32, PlayerId, PlayerId)>,
    /// (effective turn, team a, team b) of every Peace established between empires.
    peaces: Vec<(u32, TeamId, TeamId)>,
    /// (effective turn, declaring team, target team) of every declaration, with the declaring
    /// team's latest Peace with that target (its effective turn) when one was signed before.
    declarations: Vec<(u32, TeamId, TeamId, Option<u32>)>,
    factions: Vec<Faction>,
    teams: Vec<TeamId>,
    /// Faction of each member of the winning team (empty without a winner).
    winners: Vec<Faction>,
    winner_attacked_by_two_before_100: bool,
    /// India's vow: closes, reopens, vow VP scored (first time or regained), vow VP lost, and
    /// cities its owner lost while closed.
    sakoku: [u32; 5],
    /// Per seat: its treasury at the start of turns 100, 150 and 200 while alive.
    gold_at: Vec<[Option<i32>; 3]>,
    /// Per seat: summed treasury and turns over the turns it was at War with another empire.
    war_gold: Vec<(i64, u32)>,
}

/// Turns the census reads each seat's treasury at.
const GOLD_TURNS: [u32; 3] = [100, 150, 200];

fn replay(path: &std::path::Path) -> Result<Game, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tape = Tape::from_json(&text)?;
    let mut state = tape.initial_state();
    let mut g = Game {
        factions: state.players.iter().map(|p| p.faction).collect(),
        teams: state.players.iter().map(|p| p.team).collect(),
        gold_at: vec![[None; 3]; state.players.len()],
        war_gold: vec![(0, 0); state.players.len()],
        ..Game::default()
    };
    let mut closed: Vec<PlayerId> = Vec::new();
    let mut pressured: BTreeSet<TeamId> = BTreeSet::new();
    for t in &tape.turns {
        let turn = state.turn;
        if turn < 100 {
            let teams: BTreeSet<_> = state
                .players
                .iter()
                .filter(|p| p.alive)
                .map(|p| p.team)
                .collect();
            for &team in &teams {
                if teams
                    .iter()
                    .filter(|&&other| other != team && state.teams_at_war(team, other))
                    .count()
                    >= 2
                {
                    pressured.insert(team);
                }
            }
        }
        for (seat, p) in state.players.iter().enumerate() {
            if !p.alive {
                continue;
            }
            if let Some(i) = GOLD_TURNS.iter().position(|t| *t == turn) {
                g.gold_at[seat][i] = Some(p.gold);
            }
            if state
                .players
                .iter()
                .any(|o| o.alive && o.team != p.team && state.at_war(p.id, o.id))
            {
                g.war_gold[seat].0 += i64::from(p.gold);
                g.war_gold[seat].1 += 1;
            }
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        for e in &events {
            match e {
                Event::PeaceEstablished { relation } => {
                    g.peaces
                        .push((relation.effective_turn, relation.a, relation.b));
                }
                Event::WarDeclared { relation } => {
                    for d in &relation.declarers {
                        let target = if *d == relation.a {
                            relation.b
                        } else {
                            relation.a
                        };
                        g.wars.push((turn, *d, target));
                        let last_peace = g
                            .peaces
                            .iter()
                            .rev()
                            .find(|(_, a, b)| {
                                (*a == *d && *b == target) || (*a == target && *b == *d)
                            })
                            .map(|p| p.0);
                        g.declarations
                            .push((relation.effective_turn, *d, target, last_peace));
                    }
                }
                Event::CityCaptured { city, from, to, .. } => {
                    g.captures.push((turn, city.0, *from, *to));
                    if closed.contains(from) {
                        g.sakoku[4] += 1;
                    }
                }
                Event::SakokuClosed { player } => {
                    g.sakoku[0] += 1;
                    closed.push(*player);
                }
                Event::SakokuReopened { player, .. } => {
                    g.sakoku[1] += 1;
                    closed.retain(|p| p != player);
                }
                Event::FactionGoalCompleted {
                    faction: Faction::India,
                    ..
                } => g.sakoku[2] += 1,
                Event::FactionGoalLost {
                    faction: Faction::India,
                    ..
                } => g.sakoku[3] += 1,
                _ => {}
            }
        }
        state = next;
    }
    g.turns = state.turn;
    if let Some(w) = state.winner {
        g.winner_attacked_by_two_before_100 = pressured.contains(&w);
        g.winners = state
            .players
            .iter()
            .filter(|p| p.team == w)
            .map(|p| p.faction)
            .collect();
    }
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
    let games: Vec<Game> = std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|c| s.spawn(move || c.iter().map(|p| replay(p).unwrap()).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let n = games.len().max(1) as f64;
    let two_front_winners = games
        .iter()
        .filter(|g| g.winner_attacked_by_two_before_100)
        .count();
    println!(
        "eventual winner at war with two empires at once before T100: {two_front_winners}/{} ({:.1}%)",
        games.len(),
        100.0 * two_front_winners as f64 / n
    );
    let wars: usize = games.iter().map(|g| g.wars.len()).sum();
    let captures: usize = games.iter().map(|g| g.captures.len()).sum();
    let mut repeated_pairs = 0;
    let mut repeated_within_30 = 0;
    for g in &games {
        let mut last = BTreeMap::new();
        for &(turn, a, b) in &g.wars {
            let pair = (a.min(b), a.max(b));
            if let Some(previous) = last.insert(pair, turn) {
                repeated_pairs += 1;
                repeated_within_30 += usize::from(turn.saturating_sub(previous) <= 30);
            }
        }
    }
    let mut lost_back_owner = 0;
    let mut lost_back_any = 0;
    for g in &games {
        for (i, &(turn, city, from, _)) in g.captures.iter().enumerate() {
            let later = g.captures[i + 1..]
                .iter()
                .filter(|(t, c, _, _)| *c == city && *t <= turn + 10);
            let mut any = false;
            let mut owner = false;
            for &(_, _, _, to2) in later {
                any = true;
                owner |= to2 == from;
            }
            lost_back_any += usize::from(any);
            lost_back_owner += usize::from(owner);
        }
    }
    println!("games {}", games.len());
    println!(
        "mean resolved turns {:.1}",
        games.iter().map(|g| g.turns as f64).sum::<f64>() / n
    );
    println!("wars declared per game {:.2}", wars as f64 / n);
    println!(
        "repeat declarations against the same team: {} ({:.1}% of wars), within 30 turns: {} ({:.1}% of wars)",
        repeated_pairs,
        100.0 * repeated_pairs as f64 / wars.max(1) as f64,
        repeated_within_30,
        100.0 * repeated_within_30 as f64 / wars.max(1) as f64
    );
    println!("captures per game {:.2}", captures as f64 / n);
    println!(
        "captures per war {:.2}",
        captures as f64 / (wars.max(1) as f64)
    );
    println!(
        "captures lost back within 10 turns: to the previous owner {} ({:.1}%), to anyone {} ({:.1}%)",
        lost_back_owner,
        100.0 * lost_back_owner as f64 / captures.max(1) as f64,
        lost_back_any,
        100.0 * lost_back_any as f64 / captures.max(1) as f64
    );
    // Peace-then-war flip-flops: a declaration against a team the declarer made Peace with
    // `limit` turns before (either side could have signed it), by the declarer's faction.
    let mut flips: BTreeMap<String, (u32, u32, u32, u32)> = BTreeMap::new();
    let mut peaces_total = 0usize;
    let mut flips5 = 0usize;
    let mut flips10 = 0usize;
    for g in &games {
        peaces_total += g.peaces.len();
        for &(turn, declarer, _target, last_peace) in &g.declarations {
            let age = last_peace.map(|p| turn.saturating_sub(p));
            let seat = g.teams.iter().position(|t| *t == declarer);
            let faction = seat.map_or("?".to_string(), |i| format!("{:?}", g.factions[i]));
            let e = flips.entry(faction).or_default();
            e.0 += 1;
            if age.is_some_and(|a| a <= 5) {
                e.1 += 1;
                flips5 += 1;
            }
            if age.is_some_and(|a| a <= 10) {
                e.2 += 1;
                flips10 += 1;
            }
            if age.is_some_and(|a| a <= 30) {
                e.3 += 1;
            }
        }
    }
    println!(
        "peaces signed per game {:.2}; declarations within 5 turns of a Peace with the target: {} ({:.1}% of wars), within 10: {} ({:.1}%) ({:.2} per game)",
        peaces_total as f64 / n,
        flips5,
        100.0 * flips5 as f64 / wars.max(1) as f64,
        flips10,
        100.0 * flips10 as f64 / wars.max(1) as f64,
        flips10 as f64 / n
    );
    // City churn: a capture of a city that was captured within 20 turns before, and cities that
    // changed hands three or more times in a game.
    let mut churn20 = 0usize;
    let mut cities_3x = 0usize;
    let mut max_changes = 0usize;
    for g in &games {
        let mut by_city: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for &(turn, city, _, _) in &g.captures {
            by_city.entry(city).or_default().push(turn);
        }
        for turns in by_city.values() {
            churn20 += turns.windows(2).filter(|w| w[1] - w[0] <= 20).count();
            cities_3x += usize::from(turns.len() >= 3);
            max_changes = max_changes.max(turns.len());
        }
    }
    println!(
        "city churn: captures of a city captured within the previous 20 turns {} ({:.1}% of captures, {:.2} per game); cities captured 3+ times {} ({:.2} per game); most captures of one city {}",
        churn20,
        100.0 * churn20 as f64 / captures.max(1) as f64,
        churn20 as f64 / n,
        cities_3x,
        cities_3x as f64 / n,
        max_changes
    );
    println!("\n| Declarer | Declarations | Within 5 turns of a Peace | Within 10 | Within 30 |");
    println!("|---|---|---|---|---|");
    for (f, (decl, f5, f10, f30)) in &flips {
        println!(
            "| {f} | {decl} | {f5} ({:.1}%) | {f10} ({:.1}%) | {f30} ({:.1}%) |",
            100.0 * *f5 as f64 / (*decl).max(1) as f64,
            100.0 * *f10 as f64 / (*decl).max(1) as f64,
            100.0 * *f30 as f64 / (*decl).max(1) as f64
        );
    }
    // Wins by faction, and India's vow (India seats only).
    let mut wins: BTreeMap<String, u32> = BTreeMap::new();
    for g in &games {
        for f in &g.winners {
            *wins.entry(format!("{f:?}")).or_default() += 1;
        }
    }
    println!(
        "wins by faction (of {} games): {}",
        games.len(),
        wins.iter()
            .map(|(f, w)| format!("{f} {w} ({:.1}%)", 100.0 * *w as f64 / n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let india_seats: usize = games
        .iter()
        .map(|g| g.factions.iter().filter(|f| **f == Faction::India).count())
        .sum();
    let s = games.iter().fold([0u32; 5], |mut acc, g| {
        for (a, b) in acc.iter_mut().zip(g.sakoku) {
            *a += b;
        }
        acc
    });
    println!(
        "India's vow over {india_seats} India seats: closed {} ({:.2}/seat), reopened {}, vow VP scored {}, vow VP lost {}, cities lost while closed {}",
        s[0],
        s[0] as f64 / india_seats.max(1) as f64,
        s[1],
        s[2],
        s[3],
        s[4]
    );
    // Per faction: seats, games that declared, median first declaration, wars declared,
    // captures made, median first capture, cities lost.
    let mut per: BTreeMap<String, FactionStats> = BTreeMap::new();
    for g in &games {
        for (seat, f) in g.factions.iter().enumerate() {
            let team = g.teams[seat];
            let e = per.entry(format!("{f:?}")).or_default();
            e.0 += 1;
            let mine: Vec<_> = g.wars.iter().filter(|w| w.1 == team).collect();
            if let Some(first) = mine.first() {
                e.1 += 1;
                e.2.push(first.0);
            }
            e.3 += mine.len() as u32;
            let caps: Vec<_> = g
                .captures
                .iter()
                .filter(|c| c.3 == PlayerId(seat as u8))
                .collect();
            e.4 += caps.len() as u32;
            if let Some(first) = caps.first() {
                e.5.push(first.0);
            }
            e.6 += g
                .captures
                .iter()
                .filter(|c| c.2 == PlayerId(seat as u8))
                .count() as u32;
        }
    }
    println!(
        "\n| Faction | Seats | Declared a war | First war turn (median) | Wars declared / seat | Captures / seat | First capture turn (median) | Cities lost / seat |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for (f, (seats, declared, first, wars, caps, first_cap, lost)) in per {
        let s = seats.max(1) as f64;
        println!(
            "| {f} | {seats} | {:.0}% | {} | {:.2} | {:.2} | {} | {:.2} |",
            100.0 * declared as f64 / s,
            median(first),
            wars as f64 / s,
            caps as f64 / s,
            median(first_cap),
            lost as f64 / s
        );
    }
    // Gold held by faction: the treasury at turns 100, 150 and 200 (seats alive then), and
    // each seat's mean treasury over its turns at War.
    let mut gold: BTreeMap<String, ([Vec<u32>; 3], Vec<u32>)> = BTreeMap::new();
    for g in &games {
        for (seat, f) in g.factions.iter().enumerate() {
            let e = gold.entry(format!("{f:?}")).or_default();
            for (i, at) in g.gold_at[seat].iter().enumerate() {
                if let Some(v) = at {
                    e.0[i].push((*v).max(0) as u32);
                }
            }
            let (sum, turns) = g.war_gold[seat];
            if turns > 0 {
                e.1.push((sum / i64::from(turns)).max(0) as u32);
            }
        }
    }
    println!(
        "\n| Faction | Gold T100 (median) | Gold T150 (median) | Gold T200 (median) | Mean Gold at War (median seat) | Seats at War |"
    );
    println!("|---|---|---|---|---|---|");
    for (f, (at, war)) in gold {
        let seats_at_war = war.len();
        let [a, b, c] = at;
        println!(
            "| {f} | {} | {} | {} | {} | {seats_at_war} |",
            median(a),
            median(b),
            median(c),
            median(war)
        );
    }
}

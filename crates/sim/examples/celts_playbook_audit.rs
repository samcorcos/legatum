//! How the AI Celts play their kit, from a directory of `legatum-sim --tape-out` tapes.
//!
//! cargo run --release -p legatum-sim --example celts_playbook_audit -- <tape-dir> [threads]
//!
//! Replays every tape (no AI) and, for each Celtic seat, measures the designer's 8.9.25
//! playbook from the engine's own state and events: how many Celtic cities stand garrisoned each
//! turn (and by how many military units), cities at T50/T100/T150, the first and every later Call
//! the Wild, whether each rite camp stands on the side of its city that faces the nearest rival
//! city, sacks, Celtic cities lost, rival attacks on Celtic cities and units, Celtic units killed
//! by rivals, and the win. Prints a summary and one JSON line (`json {...}`) for receipts.
use legatum_core::rules::is_military;
use legatum_core::scoring::team_vp;
use legatum_core::{Event, Faction, GameState, Hex, PlayerId, events::Defender, resolve_turn};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Default, Clone)]
struct Seat {
    won: bool,
    alive: bool,
    vp: u32,
    cities_at: [u32; 3],
    city_turns: u64,
    garrisoned_city_turns: u64,
    garrison_units: u64,
    army_units: u64,
    seat_turns: u64,
    wild_calls: Vec<(u32, u32)>,
    rite_camps: u32,
    rite_camps_facing: u32,
    goal_turns: u32,
    goal_lost: u32,
    sacks: u32,
    lost: u32,
    rival_city_attacks: u32,
    rival_unit_attacks: u32,
    units_killed_by_rivals: u32,
    wars_on: u32,
    wars_declared: u32,
}

fn rival_city_distance(state: &GameState, me: PlayerId, at: Hex) -> Option<i32> {
    let team = state.team_of(me);
    state
        .cities
        .values()
        .filter(|c| !c.owner.is_barbarian() && state.team_of(c.owner) != team)
        .map(|c| c.pos.distance(at))
        .min()
}

fn replay(path: &std::path::Path) -> Result<Vec<Seat>, String> {
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    let trace = std::env::var_os("CELTS_AUDIT_TRACE").is_some();
    let celts: Vec<PlayerId> = state
        .players
        .iter()
        .filter(|p| p.faction == Faction::Celts)
        .map(|p| p.id)
        .collect();
    let mut seats: BTreeMap<PlayerId, Seat> = celts.iter().map(|p| (*p, Seat::default())).collect();
    if seats.is_empty() {
        return Ok(Vec::new());
    }
    for t in &tape.turns {
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &t.orders);
        // A unit killed in battle this turn: who struck it last.
        let mut struck_by: BTreeMap<legatum_core::UnitId, PlayerId> = BTreeMap::new();
        for e in &events {
            if trace {
                let celtic = |p: &PlayerId| seats.contains_key(p);
                let line = match e {
                    Event::WildCalled { player, camps } if celtic(player) => {
                        Some(format!("called, {camps} camps"))
                    }
                    Event::FactionGoalCompleted { player, .. } if celtic(player) => {
                        Some("goal scores".into())
                    }
                    Event::FactionGoalLost { player, .. } if celtic(player) => {
                        Some("goal lost".into())
                    }
                    Event::CityFounded { .. }
                    | Event::CityCaptured { .. }
                    | Event::CityRazed { .. } => Some(format!("{e:?}").chars().take(120).collect()),
                    Event::BarbarianCampCleared { at, by, gold, .. }
                        if state
                            .map
                            .get(*at)
                            .and_then(|t| t.owner)
                            .is_some_and(|o| celtic(&o)) =>
                    {
                        Some(format!(
                            "camp at {at:?} in Celtic land cleared by {by:?} (gold {gold})"
                        ))
                    }
                    _ => None,
                };
                if let Some(l) = line {
                    println!("{} T{turn} {l}", path.display());
                }
            }
            match e {
                Event::Battle {
                    attacker_owner,
                    defender,
                    ..
                } => {
                    let (owner, unit) = match defender {
                        Defender::Unit { owner, unit, .. } => (*owner, Some(*unit)),
                        Defender::City { owner, .. } => (*owner, None),
                    };
                    if let Some(seat) = seats.get_mut(&owner) {
                        let rival = !attacker_owner.is_barbarian()
                            && state.team_of(*attacker_owner) != state.team_of(owner);
                        if rival {
                            match unit {
                                Some(u) => {
                                    seat.rival_unit_attacks += 1;
                                    struck_by.insert(u, *attacker_owner);
                                }
                                None => seat.rival_city_attacks += 1,
                            }
                        }
                    }
                }
                Event::UnitDestroyed { unit, owner, .. } => {
                    if let Some(seat) = seats.get_mut(owner)
                        && struck_by.contains_key(unit)
                    {
                        seat.units_killed_by_rivals += 1;
                    }
                }
                Event::CityCaptured { from, .. } => {
                    if let Some(seat) = seats.get_mut(from) {
                        seat.lost += 1;
                    }
                }
                Event::CityRazed { by, sack_gold, .. } if sack_gold.is_some() => {
                    if let Some(seat) = seats.get_mut(by) {
                        seat.sacks += 1;
                    }
                }
                Event::WarDeclared { relation } => {
                    for (pid, seat) in seats.iter_mut() {
                        let mine = state.team_of(*pid);
                        if relation.a == mine || relation.b == mine {
                            if relation.declarers.contains(&mine) {
                                seat.wars_declared += 1;
                            } else {
                                seat.wars_on += 1;
                            }
                        }
                    }
                }
                Event::WildCalled { player, camps } => {
                    if let Some(seat) = seats.get_mut(player) {
                        seat.wild_calls.push((turn, *camps));
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
        // Rite camps raised this turn: does each stand nearer the closest rival city than its city?
        let called: Vec<PlayerId> = events
            .iter()
            .filter_map(|e| match e {
                Event::WildCalled { player, .. } => Some(*player),
                _ => None,
            })
            .collect();
        if let Some(b) = next.barbarians.as_ref() {
            for p in called {
                for camp in b
                    .camps
                    .iter()
                    .filter(|c| c.rite == Some(p) && c.founded_turn == turn)
                {
                    let Some(city) = next
                        .map
                        .get(camp.pos)
                        .and_then(|t| t.city)
                        .and_then(|id| next.cities.get(&id))
                    else {
                        continue;
                    };
                    let seat = seats.get_mut(&p).expect("Celtic seat");
                    seat.rite_camps += 1;
                    if let (Some(dc), Some(dk)) = (
                        rival_city_distance(&next, p, city.pos),
                        rival_city_distance(&next, p, camp.pos),
                    ) && dk < dc
                    {
                        seat.rite_camps_facing += 1;
                    }
                }
            }
        }
        state = next;
        for (pid, seat) in seats.iter_mut() {
            if !state.player(*pid).alive {
                continue;
            }
            seat.seat_turns += 1;
            if state.player(*pid).faction_goal_done {
                seat.goal_turns += 1;
            }
            let cities: Vec<Hex> = state.cities_of(*pid).map(|c| c.pos).collect();
            for (i, at) in [50u32, 100, 150].iter().enumerate() {
                if state.turn == *at {
                    seat.cities_at[i] = cities.len() as u32;
                }
            }
            let mut on_city: BTreeMap<Hex, u64> = BTreeMap::new();
            let mut army = 0u64;
            for u in state
                .units
                .values()
                .filter(|u| u.owner == *pid && is_military(u.kind))
            {
                if u.kind == legatum_core::UnitKind::Scout {
                    continue;
                }
                army += 1;
                if cities.contains(&u.pos) {
                    *on_city.entry(u.pos).or_default() += 1;
                }
            }
            seat.city_turns += cities.len() as u64;
            seat.garrisoned_city_turns += on_city.len() as u64;
            seat.garrison_units += on_city.values().sum::<u64>();
            seat.army_units += army;
        }
    }
    for (pid, seat) in seats.iter_mut() {
        seat.won = state.winner == Some(state.team_of(*pid));
        seat.alive = state.player(*pid).alive;
        seat.vp = team_vp(&state, state.team_of(*pid));
    }
    Ok(seats.into_values().collect())
}

fn quartiles(mut v: Vec<u32>) -> Option<(u32, u32, u32)> {
    if v.is_empty() {
        return None;
    }
    v.sort_unstable();
    Some((v[v.len() / 4], v[v.len() / 2], v[v.len() * 3 / 4]))
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
    let sum = |f: &dyn Fn(&Seat) -> f64| seats.iter().map(f).sum::<f64>();
    let wins = seats.iter().filter(|s| s.won).count();
    let city_turns = sum(&|s| s.city_turns as f64).max(1.0);
    let garrisoned = sum(&|s| s.garrisoned_city_turns as f64) / city_turns;
    let per_city = sum(&|s| s.garrison_units as f64) / city_turns;
    let army_per_city = sum(&|s| s.army_units as f64) / city_turns;
    let called: Vec<&Seat> = seats.iter().filter(|s| !s.wild_calls.is_empty()).collect();
    let first_call = quartiles(called.iter().map(|s| s.wild_calls[0].0).collect());
    let camps = sum(&|s| s.rite_camps as f64);
    let facing = sum(&|s| s.rite_camps_facing as f64);
    let cities_at: Vec<f64> = (0..3)
        .map(|i| sum(&|s| s.cities_at[i] as f64) / n)
        .collect();
    println!(
        "celtic seats {} wins {} ({:.1}%), alive at end {:.1}%, mean final VP {:.2}",
        seats.len(),
        wins,
        100.0 * wins as f64 / n,
        100.0 * seats.iter().filter(|s| s.alive).count() as f64 / n,
        sum(&|s| s.vp as f64) / n
    );
    println!(
        "cities per seat at T50 {:.2}, T100 {:.2}, T150 {:.2}",
        cities_at[0], cities_at[1], cities_at[2]
    );
    println!(
        "garrison: {:.1}% of Celtic city-turns hold a military unit; {:.2} units on the city per city-turn; army {:.2} per city",
        100.0 * garrisoned,
        per_city,
        army_per_city
    );
    println!(
        "Call the Wild: {} of {} seats called ({:.1}%), {:.2} calls per calling seat, first call p25/median/p75 {:?}; goal held {:.1} turns per seat, lost {:.2} times",
        called.len(),
        seats.len(),
        100.0 * called.len() as f64 / n,
        called
            .iter()
            .map(|s| s.wild_calls.len() as f64)
            .sum::<f64>()
            / called.len().max(1) as f64,
        first_call,
        sum(&|s| s.goal_turns as f64) / n,
        sum(&|s| s.goal_lost as f64) / n
    );
    println!(
        "rite camps {} ({:.0}% on the rival-facing side of their city)",
        camps,
        100.0 * facing / camps.max(1.0)
    );
    println!(
        "per seat: sacks {:.2}, cities lost {:.2}, wars declared {:.2}, wars on them {:.2}, rival attacks on cities {:.2} and units {:.2}, units killed by rivals {:.2}",
        sum(&|s| s.sacks as f64) / n,
        sum(&|s| s.lost as f64) / n,
        sum(&|s| s.wars_declared as f64) / n,
        sum(&|s| s.wars_on as f64) / n,
        sum(&|s| s.rival_city_attacks as f64) / n,
        sum(&|s| s.rival_unit_attacks as f64) / n,
        sum(&|s| s.units_killed_by_rivals as f64) / n
    );
    let out = json!({
        "seats": seats.len(),
        "wins": wins,
        "win_rate": wins as f64 / n,
        "alive_rate": seats.iter().filter(|s| s.alive).count() as f64 / n,
        "final_vp": sum(&|s| s.vp as f64) / n,
        "cities_t50": cities_at[0],
        "cities_t100": cities_at[1],
        "cities_t150": cities_at[2],
        "garrisoned_city_share": garrisoned,
        "garrison_units_per_city": per_city,
        "army_per_city": army_per_city,
        "wild_called_share": called.len() as f64 / n,
        "wild_calls_per_calling_seat": called.iter().map(|s| s.wild_calls.len() as f64).sum::<f64>() / called.len().max(1) as f64,
        "first_call_quartiles": first_call,
        "goal_turns_per_seat": sum(&|s| s.goal_turns as f64) / n,
        "goal_lost_per_seat": sum(&|s| s.goal_lost as f64) / n,
        "rite_camps": camps,
        "rite_camps_facing_share": facing / camps.max(1.0),
        "sacks_per_seat": sum(&|s| s.sacks as f64) / n,
        "cities_lost_per_seat": sum(&|s| s.lost as f64) / n,
        "wars_declared_per_seat": sum(&|s| s.wars_declared as f64) / n,
        "wars_on_per_seat": sum(&|s| s.wars_on as f64) / n,
        "rival_city_attacks_per_seat": sum(&|s| s.rival_city_attacks as f64) / n,
        "rival_unit_attacks_per_seat": sum(&|s| s.rival_unit_attacks as f64) / n,
        "units_killed_by_rivals_per_seat": sum(&|s| s.units_killed_by_rivals as f64) / n,
    });
    println!("json {out}");
}

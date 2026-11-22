//! Replay order tapes to measure opening wars from the state immediately before declaration.
//! Usage: opening_wars <tape-directory> [trace-seat]
//! Observer only: never supplies information to an AI or alters a tape.
use legatum_core::{Event, UnitKind, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let trace = args.get(2).map(|s| s.parse::<u8>()).transpose()?;
    let mut paths = std::fs::read_dir(&args[1])?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|p| p.extension().is_some_and(|ext| ext == "json"));
    paths.sort();
    for path in paths {
        let tape = Tape::from_json(&std::fs::read_to_string(&path)?)?;
        let mut state = tape.initial_state();
        let mut wars = Vec::new();
        let mut captures = Vec::new();
        let mut first_army = BTreeMap::new();
        for turn in &tape.turns {
            for player in &state.players {
                let army = state
                    .units
                    .values()
                    .filter(|u| {
                        u.owner == player.id
                            && u.kind != UnitKind::Scout
                            && legatum_core::data::unit_stats(u.kind).military
                    })
                    .count();
                if army >= 5 {
                    first_army.entry(player.id).or_insert(turn.turn);
                }
            }
            for (seat, orders) in &turn.orders {
                if trace != Some(seat.0) || turn.turn > 180 {
                    continue;
                }
                let v = player_view(&state, *seat);
                let force: Vec<_> = v
                    .units
                    .iter()
                    .filter(|u| u.owner == *seat && u.military && u.kind != UnitKind::Scout)
                    .collect();
                if trace == Some(seat.0) && turn.turn <= 180 {
                    let units: Vec<_> = force.iter().map(|u| json!({"id":u.id,"kind":u.kind,"pos":u.pos,"hp":u.hp,
                        "nearest":v.cities.iter().filter(|c| c.owner != *seat).map(|c|u.pos.distance(c.pos)).min()})).collect();
                    eprintln!(
                        "{}",
                        json!({"file":path.file_name().unwrap().to_string_lossy(),"turn":turn.turn,"seat":seat,"gold":v.economy.gold,"income":v.economy.gold_per_turn.total,"units":units,"enemies":v.units.iter().filter(|u| u.owner != *seat).collect::<Vec<_>>(),"cities":v.cities.iter().map(|c|json!({"id":c.id,"owner":c.owner,"pos":c.pos,"hp":c.hp,"visible":c.visible})).collect::<Vec<_>>(),"orders":orders})
                    );
                }
            }
            let (next, events) = resolve_turn(&state, &turn.orders);
            if trace.is_some() && turn.turn <= 180 {
                eprintln!(
                    "{}",
                    json!({"file":path.file_name().unwrap().to_string_lossy(), "turn":turn.turn, "events": events})
                );
            }
            assert_eq!(
                next.hash().to_string(),
                turn.hash,
                "{} turn {}",
                path.display(),
                turn.turn
            );
            for event in events {
                match event {
                    Event::CityCaptured { from, to, .. } => {
                        captures.push(json!({"turn":turn.turn,"from":from,"to":to}));
                    }
                    Event::WarDeclared { relation } => {
                        for seat in state
                            .players
                            .iter()
                            .filter(|p| relation.declarers.contains(&p.team))
                        {
                            let v = player_view(&state, seat.id);
                            let against = if relation.a == v.team {
                                relation.b
                            } else {
                                relation.a
                            };
                            let force: Vec<_> = v
                                .units
                                .iter()
                                .filter(|u| {
                                    u.owner == seat.id && u.military && u.kind != UnitKind::Scout
                                })
                                .collect();
                            let local: Vec<_> = v.cities.iter().filter(|c| v.team_of(c.owner) == against)
                                .map(|c| {
                                    let units: Vec<_> = force.iter().filter(|u| u.pos.distance(c.pos) <= 4).collect();
                                    json!({"city": c.id, "visible": c.visible,
                                        "units": units.len(), "healthy": units.iter().filter(|u| u.hp >= 60).count(),
                                        "melee": units.iter().filter(|u| u.ranged == 0).count(),
                                        "ranged": units.iter().filter(|u| u.ranged > 0).count()})
                                }).collect();
                            wars.push(
                                json!({"seat": seat.id, "turn": turn.turn, "against": against,
                                "army": force.len(),
                                "melee": force.iter().filter(|u| u.ranged == 0).count(),
                                "ranged": force.iter().filter(|u| u.ranged > 0).count(),
                                "cities": local}),
                            );
                        }
                    }
                    _ => {}
                }
            }
            state = next;
        }
        println!(
            "{}",
            json!({"file":path.file_name().unwrap().to_string_lossy(),"wars":wars,"captures":captures,
                "first_five_units": first_army})
        );
    }
    Ok(())
}

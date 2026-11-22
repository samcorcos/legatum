//! Observer-only, hash-checked siege evidence from a native order tape.
//! cargo run --release -p legatum-sim --example conquest_trace -- tape.json [TURN SEAT]
//! Optional TURN/SEAT emits just that fogged planning view for a focused regression.
use legatum_core::{Event, Order, UnitKind, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
use serde_json::json;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).ok_or("expected tape.json")?;
    let selected = args
        .get(2)
        .map(|turn| {
            Ok::<_, String>((
                turn.parse::<u32>().map_err(|e| e.to_string())?,
                args.get(3)
                    .ok_or("expected SEAT after TURN")?
                    .parse::<u8>()
                    .map_err(|e| e.to_string())?,
            ))
        })
        .transpose()?;
    let tape = Tape::from_json(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)?;
    let mut state = tape.initial_state();
    if state.hash() != tape.initial_hash {
        return Err("initial hash mismatch".into());
    }
    for turn in &tape.turns {
        for (&seat, orders) in &turn.orders {
            let v = player_view(&state, seat);
            if let Some(wanted) = selected {
                if wanted == (turn.turn, seat.0) {
                    println!("{}", serde_json::to_string(&v).map_err(|e| e.to_string())?);
                    return Ok(());
                }
                continue;
            }
            let army: Vec<_> = v
                .units
                .iter()
                .filter(|u| u.owner == seat && u.military && u.kind != UnitKind::Scout)
                .collect();
            let declarations: Vec<_> = orders
                .iter()
                .filter(|o| matches!(o, Order::DeclareWar { .. }))
                .collect();
            let sieges: Vec<_> = v.cities.iter().filter(|c| v.at_war(seat, c.owner))
                .filter(|c| army.iter().any(|u| u.pos.distance(c.pos) <= 5))
                .map(|c| {
                    let near: Vec<_> = army.iter().filter(|u| u.pos.distance(c.pos) <= 5).map(|u| {
                        let order = orders.iter().find(|o| match o {
                            Order::MoveUnit { unit, .. } | Order::MoveUnitPath { unit, .. }
                            | Order::RangedAttack { unit, .. } | Order::SkipUnit { unit }
                            | Order::SleepUnit { unit, .. } | Order::UpgradeUnit { unit } => *unit == u.id,
                            _ => false,
                        });
                        json!({"id":u.id,"kind":u.kind,"hp":u.hp,"distance":u.pos.distance(c.pos),"pos":u.pos,"order":order})
                    }).collect();
                    json!({"id":c.id,"owner":c.owner,"pos":c.pos,"visible":c.visible,"hp":c.hp,"walls":c.walls,"strength":c.strength,"units":near})
                }).collect();
            if !declarations.is_empty() || !sieges.is_empty() {
                println!(
                    "{}",
                    json!({"seed":tape.setup.seed.to_string(),"turn":turn.turn,"seat":seat,"army":army.len(),"shooters":army.iter().filter(|u| u.ranged>0).count(),"declarations":declarations,"sieges":sieges})
                );
            }
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        if next.hash().to_string() != turn.hash {
            return Err(format!("hash mismatch at {}", turn.turn));
        }
        let events: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::CityCaptured { .. }
                        | Event::WarDeclared { .. }
                        | Event::PeaceEstablished { .. }
                        | Event::Battle { .. }
                )
            })
            .collect();
        if selected.is_none() {
            println!(
                "{}",
                json!({"seed":tape.setup.seed.to_string(),"turn":turn.turn,"events":events})
            );
        }
        state = next;
    }
    Ok(())
}

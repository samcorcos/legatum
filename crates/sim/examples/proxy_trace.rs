//! Hash-checked, observer-only turn telemetry for a Proxy loss archive.
//! cargo run --release -p legatum-sim --example proxy_trace -- archive.tape.json.zst SEAT
use legatum_core::{Event, PlayerId, new_game, player_view, resolve_turn};
use serde_json::json;
use std::{collections::BTreeMap, path::Path};

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let path = args.get(1).ok_or("expected archive path and Proxy seat")?;
    let seat = PlayerId(
        args.get(2)
            .ok_or("expected seat")?
            .parse::<u8>()
            .map_err(|e| e.to_string())?,
    );
    let recording = legatum_archive::read(Path::new(path))?;
    recording.compatible()?;
    let mut state = match &recording.initial_state {
        Some(s) => legatum_wasm::json::parse_state(s)?,
        None => new_game(&recording.tape.setup),
    };
    if state.hash() != recording.tape.initial_hash {
        return Err("initial state hash mismatch".into());
    }
    if !state.players.iter().any(|p| p.id == seat) {
        return Err("seat is not in the archived setup".into());
    }
    for t in &recording.tape.turns {
        let view = player_view(&state, seat);
        let orders = t.orders.get(&seat).cloned().unwrap_or_default();
        let (next, events) = resolve_turn(&state, &t.orders);
        if next.hash().to_string() != t.hash {
            return Err(format!("hash mismatch at {}", t.turn));
        }
        let own_events: Vec<_> = events
            .iter()
            .filter(|e| match e {
                Event::CityFounded { owner, .. }
                | Event::ProductionCompleted { owner, .. }
                | Event::MoveBlocked { owner, .. }
                | Event::UnitDestroyed { owner, .. }
                | Event::UnitDisbanded { owner, .. }
                | Event::ItemBought { owner, .. } => *owner == seat,
                Event::TechResearched { player, .. } | Event::OrderFailed { player, .. } => {
                    *player == seat
                }
                Event::WarDeclared { relation }
                | Event::PeaceEstablished { relation }
                | Event::TreatyActivated { relation, .. } => {
                    relation.a == view.team || relation.b == view.team
                }
                Event::ProposalCreated { proposal }
                | Event::ProposalAccepted { proposal, .. }
                | Event::ProposalRejected { proposal, .. } => {
                    proposal.from == view.team || proposal.to == view.team
                }
                Event::CityCaptured { from, to, .. } => *from == seat || *to == seat,
                Event::Battle {
                    attacker_owner,
                    defender,
                    ..
                } => {
                    *attacker_owner == seat
                        || match defender {
                            legatum_core::events::Defender::Unit { owner, .. }
                            | legatum_core::events::Defender::City { owner, .. } => *owner == seat,
                        }
                }
                _ => false,
            })
            .collect();
        let mut units = BTreeMap::new();
        for u in next.units_of(seat) {
            *units.entry(u.kind).or_insert(0) += 1;
        }
        println!(
            "{}",
            json!({
                "turn": t.turn, "seat": seat, "orders": orders, "events": own_events,
                "before": {
                    "gold": view.economy.gold,
                    "income": view.economy.gold_per_turn.total,
                    "science": view.economy.science_per_turn.total,
                    "diplomacy": view.diplomacy,
                    "standings": view.standings,
                    "city_detail": view.cities.iter().filter(|c| c.owner == seat).collect::<Vec<_>>(),
                    "own_units": view.units.iter().filter(|u| u.owner == seat).collect::<Vec<_>>(),
                    "known_enemy_cities": view.cities.iter().filter(|c| c.owner != seat).collect::<Vec<_>>(),
                    "visible_enemy_units": view.units.iter().filter(|u| u.owner != seat).collect::<Vec<_>>(),
                },
                "after": {
                    "cities": next.cities_of(seat).count(),
                    "techs": next.player(seat).techs,
                    "units": units,
                },
            })
        );
        state = next;
    }
    Ok(())
}

//! p892_assault <tape.json> <aggressor-player> <victim-player> <from-turn> <to-turn> [city-id]
//! Diagnostic for the 8.9.2 Mongol assault: replays the tape and, for each turn in the range,
//! prints the aggressor's military units (kind, tile, hp), the victim's military units,
//! the targeted city (hp, walls, garrison) and every battle, capture and war event between them.
use legatum_core::events::{Attacker, Defender};
use legatum_core::rules::is_military;
use legatum_core::{CityId, Event, PlayerId, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let aggressor = PlayerId(a[2].parse()?);
    let victim = PlayerId(a[3].parse()?);
    let (from, to): (u32, u32) = (a[4].parse()?, a[5].parse()?);
    let city_id = a.get(6).map(|s| s.parse::<u32>()).transpose()?.map(CityId);
    let mut state = tape.initial_state();
    for turn in &tape.turns {
        let t = turn.turn;
        if (from..=to).contains(&t) {
            println!("=== T{t} (start of turn)");
            if t == from {
                for c in state
                    .cities
                    .values()
                    .filter(|c| c.owner == aggressor || c.owner == victim)
                {
                    println!(
                        "  CITYLIST id {} {} owner P{} pos ({},{}) hp {} pop {}",
                        c.id.0, c.name, c.owner.0, c.pos.q, c.pos.r, c.hp, c.population
                    );
                }
            }
            for (label, who) in [("ATT", aggressor), ("DEF", victim)] {
                let mut by_kind: BTreeMap<String, Vec<String>> = BTreeMap::new();
                for u in state
                    .units
                    .values()
                    .filter(|u| u.owner == who && is_military(u.kind))
                {
                    by_kind
                        .entry(format!("{:?}", u.kind))
                        .or_default()
                        .push(format!("({},{})hp{}", u.pos.q, u.pos.r, u.hp));
                }
                println!("  {label} P{} units: {by_kind:?}", who.0);
            }
            if let Some(c) = city_id.and_then(|id| state.cities.get(&id)) {
                let garrison: Vec<String> = state
                    .units
                    .values()
                    .filter(|u| u.pos == c.pos && u.owner == c.owner)
                    .map(|u| format!("{:?}hp{}", u.kind, u.hp))
                    .collect();
                println!(
                    "  CITY {} {} owner P{} pos ({},{}) hp {} pop {} buildings {:?} garrison {garrison:?}",
                    c.id.0, c.name, c.owner.0, c.pos.q, c.pos.r, c.hp, c.population, c.buildings
                );
            }
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        if (from..=to).contains(&t) {
            for e in &events {
                match e {
                    Event::Battle {
                        attacker,
                        attacker_owner,
                        defender,
                        at,
                        damage_to_attacker,
                        damage_to_defender,
                        attacker_hp_after,
                        defender_hp_after,
                        ranged,
                        ..
                    } => {
                        let (dow, dkind) = match defender {
                            Defender::Unit {
                                owner, unit_kind, ..
                            } => (*owner, format!("{unit_kind:?}")),
                            Defender::City { owner, city } => (*owner, format!("City{}", city.0)),
                        };
                        if *attacker_owner != aggressor
                            && *attacker_owner != victim
                            && dow != aggressor
                            && dow != victim
                        {
                            continue;
                        }
                        if !((*attacker_owner == aggressor && dow == victim)
                            || (*attacker_owner == victim && dow == aggressor))
                        {
                            continue;
                        }
                        let akind = match attacker {
                            Attacker::Unit(id) => state
                                .units
                                .get(id)
                                .map_or("?".to_string(), |u| format!("{:?}", u.kind)),
                            Attacker::City { city } => format!("City{}", city.0),
                        };
                        println!(
                            "  BATTLE P{}:{akind} -> P{}:{dkind} at ({},{}) {} dmg to att {damage_to_attacker} (hp {attacker_hp_after}), to def {damage_to_defender} (hp {defender_hp_after})",
                            attacker_owner.0,
                            dow.0,
                            at.q,
                            at.r,
                            if *ranged { "ranged" } else { "melee" }
                        );
                    }
                    Event::CityCaptured { .. }
                    | Event::WarDeclared { .. }
                    | Event::PeaceEstablished { .. } => {
                        println!("  {e:?}");
                    }
                    Event::UnitDestroyed { owner, .. }
                        if *owner == aggressor || *owner == victim =>
                    {
                        println!("  {e:?}");
                    }
                    _ => {}
                }
            }
        }
        state = next;
    }
    Ok(())
}

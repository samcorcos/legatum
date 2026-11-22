//! Trace a Roman seat through a scripted-AI tape (analysis only):
//! rome_trace <tape.json> [first_turn] [last_turn]
//! Per turn, before resolution: Rome's cities and each melee unit (kind, level, XP, HP, tile,
//! distance to the nearest own city) with its order this turn, the nearest barbarian unit and
//! camp Rome's own view shows, and the nearest visible rival military unit. After resolution,
//! battles a Roman unit fought and Roman foundings.
use legatum_core::{Event, Faction, Order, UnitClass, data, player_view, resolve_turn};
use legatum_wasm::tape::Tape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let first: u32 = a.get(2).map_or(1, |s| s.parse().unwrap());
    let last: u32 = a.get(3).map_or(100, |s| s.parse().unwrap());
    let mut state = tape.initial_state();
    let me = state
        .players
        .iter()
        .find(|p| p.faction == Faction::Rome)
        .ok_or("no Roman seat")?
        .id;
    for turn in &tape.turns {
        let show = (first..=last).contains(&turn.turn);
        let orders = turn.orders.get(&me).cloned().unwrap_or_default();
        if show {
            let v = player_view(&state, me);
            let cities: Vec<_> = state.cities_of(me).map(|c| c.pos).collect();
            let near_city =
                |h: legatum_core::Hex| cities.iter().map(|c| c.distance(h)).min().unwrap_or(99);
            let melee: Vec<_> = state
                .units_of(me)
                .filter(|u| data::unit_class(u.kind) == Some(UnitClass::Melee))
                .collect();
            let army = state
                .units_of(me)
                .filter(|u| data::unit_stats(u.kind).military)
                .count();
            let barb = v
                .units
                .iter()
                .filter(|u| u.owner.is_barbarian())
                .map(|u| (near_city(u.pos), u.pos, u.kind, u.hp))
                .min();
            let camp = v
                .barbarian_camps
                .iter()
                .map(|c| (near_city(c.pos), c.pos, c.visible))
                .min();
            let rival = v
                .units
                .iter()
                .filter(|u| u.owner != me && !u.owner.is_barbarian() && u.military)
                .map(|u| (near_city(u.pos), u.owner.0))
                .min();
            let prod: Vec<String> = state
                .cities_of(me)
                .map(|c| format!("{:?}", c.producing).chars().take(40).collect())
                .collect();
            println!(
                "T{} cities={} army={} gold={} barb={:?} camp={:?} rival={:?} wars={:?} prod={:?}",
                turn.turn,
                cities.len(),
                army,
                state.player(me).gold,
                barb,
                camp,
                rival,
                state
                    .players
                    .iter()
                    .filter(|p| p.id != me && v.at_war(me, p.id))
                    .map(|p| p.id.0)
                    .collect::<Vec<_>>(),
                prod
            );
            if std::env::var("CAMPS").is_ok() {
                let camps: Vec<_> = state
                    .barbarians
                    .as_ref()
                    .map(|b| b.camps.iter().map(|c| near_city(c.pos)).collect())
                    .unwrap_or_default();
                let rivals: Vec<_> = state
                    .cities
                    .values()
                    .filter(|c| c.owner != me)
                    .map(|c| (near_city(c.pos), c.owner.0, c.population))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .take(4)
                    .collect();
                println!("   TRUE camps={camps:?} rival_cities={rivals:?}");
            }
            for u in &melee {
                let order = orders.iter().find(|o| match o {
                    Order::MoveUnit { unit, .. }
                    | Order::SkipUnit { unit }
                    | Order::FoundCity { unit } => *unit == u.id,
                    o => format!("{o:?}").contains(&format!("UnitId({})", u.id.0)),
                });
                println!(
                    "   {:?} {:?} lvl{} xp{} hp{} at{:?} d{} -> {}",
                    u.id,
                    u.kind,
                    legatum_core::rules::promotion::level_for(Faction::Rome, u.xp),
                    u.xp,
                    u.hp,
                    u.pos,
                    near_city(u.pos),
                    order.map_or("-".into(), |o| format!("{o:?}"))
                );
            }
        }
        let (next, events) = resolve_turn(&state, &turn.orders);
        if next.hash().to_string() != turn.hash {
            return Err(format!("tape hash differs after turn {}", turn.turn).into());
        }
        if show {
            for e in &events {
                let s = format!("{e:?}");
                let ours = match e {
                    Event::CityFounded { owner, .. } => *owner == me,
                    _ => false,
                };
                if ours
                    || (s.starts_with("Battle")
                        && state
                            .units_of(me)
                            .any(|u| s.contains(&format!("UnitId({})", u.id.0))))
                    || (s.starts_with("UnitPromoted")
                        && state
                            .units_of(me)
                            .any(|u| s.contains(&format!("UnitId({})", u.id.0))))
                {
                    println!("   EV {}", s.chars().take(220).collect::<String>());
                }
            }
        }
        state = next;
    }
    Ok(())
}

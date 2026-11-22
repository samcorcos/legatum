//! Sparta and Celts goal state over a game's tape: War Shrines, veteran levels, retirements,
//! wild camps (`faction_goals_trace <tape.json> [every]`).
use legatum_core::{BuildingKind, Faction, resolve_turn, rules::promotion};
use legatum_wasm::tape::Tape;

fn main() {
    let path = std::env::args().nth(1).expect("tape");
    let every: u32 = std::env::args()
        .nth(2)
        .map_or(10, |s| s.parse().expect("every"));
    let tape = Tape::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut state = tape.initial_state();
    for t in &tape.turns {
        let (next, _events) = resolve_turn(&state, &t.orders);
        state = next;
        if !state.turn.is_multiple_of(every) {
            continue;
        }
        for p in &state.players {
            match p.faction {
                Faction::Sparta => {
                    let shrines = state
                        .cities_of(p.id)
                        .filter(|c| c.buildings.contains(&BuildingKind::WarShrine))
                        .count();
                    let mut levels = [0u32; 8];
                    for u in state
                        .units_of(p.id)
                        .filter(|u| legatum_core::data::unit_stats(u.kind).military)
                    {
                        levels[promotion::level_for(Faction::Sparta, u.xp) as usize] += 1;
                    }
                    println!(
                        "T{} Sparta p{} cities {} shrines {} levels {:?} retired {} done {}",
                        state.turn,
                        p.id.0,
                        state.cities_of(p.id).count(),
                        shrines,
                        levels,
                        p.veterans_retired,
                        p.faction_goal_done
                    );
                }
                Faction::Celts => {
                    let (with, owned) = legatum_core::goals::wild_camp_cities(&state, p.id);
                    let rite = state.barbarians.as_ref().map_or(0, |b| {
                        b.camps.iter().filter(|c| c.rite == Some(p.id)).count()
                    });
                    println!(
                        "T{} Celts p{} cities {} camps {with}/{owned} rite camps {rite} done {}",
                        state.turn, p.id.0, owned, p.faction_goal_done
                    );
                }
                _ => {}
            }
        }
    }
}

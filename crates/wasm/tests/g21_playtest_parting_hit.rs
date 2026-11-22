//! Playtest G21, from the user's own game (playtest 3, engine 29a5b2a): on turn 106 their
//! wounded Archer (#62, 58 HP) was ordered north to (25,16) while AI 5's adjacent Spearman (#82)
//! attacked it. The contact pin kept the Archer on its tile ("pinned by an enemy attack") and
//! it ended the turn at 14 HP beside the Spearman. The fixture is the recorded state at the
//! start of turn 106 (replayed from the save's tape with the playtest-3 engine) and the
//! recorded orders of turn 106.
use legatum_core::events::Defender;
use legatum_core::{Event, GameState, Hex, Order, PlayerId, UnitId, resolve_turn};
use legatum_wasm::tape::TapeTurn;

const ARCHER: UnitId = UnitId(62);
const SPEARMAN: UnitId = UnitId(82);

fn fixture() -> (GameState, Vec<TapeTurn>) {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/playtest3-g21-turn106.json")).unwrap();
    let state = legatum_wasm::json::parse_state_for_rehash(&v["state"].to_string()).unwrap();
    // Replay the recorded orders under the current rules.
    let turns: Vec<TapeTurn> = serde_json::from_value(v["turns"].clone()).unwrap();
    (state, turns)
}

#[test]
fn the_retreating_archer_is_hit_once_then_gets_away() {
    let (state, turns) = fixture();
    assert_eq!(state.turn, 106);
    let start = Hex { q: 24, r: 18 };
    assert_eq!(state.units[&ARCHER].pos, start);
    assert_eq!(state.units[&ARCHER].hp, 58);
    assert_eq!(state.units[&SPEARMAN].pos.distance(start), 1, "adjacent");
    assert!(turns[0].orders[&PlayerId(0)].iter().any(|o| matches!(o,
        Order::MoveUnitPath { unit, to, .. } if *unit == ARCHER && *to == Hex { q: 25, r: 16 })));
    let (next, events) = resolve_turn(&state, &turns[0].orders);
    let hits: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Battle {
                attacker,
                defender: Defender::Unit { unit, .. },
                at,
                defender_leaving,
                defender_hp_after,
                ..
            } if *attacker == SPEARMAN && *unit == ARCHER => {
                Some((*at, *defender_leaving, *defender_hp_after))
            }
            _ => None,
        })
        .collect();
    assert_eq!(hits.len(), 1, "one parting hit: {events:?}");
    let (at, leaving, hp) = hits[0];
    assert_eq!(at, start, "hit where it stood");
    assert!(leaving);
    assert!(hp > 0, "it survived the hit");
    assert!(
        !events.iter().any(|e| matches!(e,
            Event::MoveBlocked { unit, reason, .. } if *unit == ARCHER && reason.contains("pinned"))),
        "no longer pinned"
    );
    let r = &next.units[&ARCHER];
    assert_ne!(r.pos, start, "it pulled back");
    assert!(r.pos.distance(Hex { q: 25, r: 16 }) < start.distance(Hex { q: 25, r: 16 }));
    assert_eq!(
        next.units[&SPEARMAN].pos, state.units[&SPEARMAN].pos,
        "the Spearman stays"
    );
}

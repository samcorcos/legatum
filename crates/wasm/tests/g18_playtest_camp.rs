//! Playtest G18, from the user's own game (playtest 3, engine 29a5b2a): on turn 17 their
//! Soldier (#3) attacked the barbarian camp guard (#27) on the adjacent camp tile. The attack
//! order kept its goto on the guard's tile, so with no new orders the Soldier attacked again on
//! turns 18 and 19 and died. The fixture is the recorded state at the start of turn 17
//! (replayed from the save's tape with the playtest-3 engine) and the recorded orders of
//! turns 17 to 19.
use legatum_core::events::Defender;
use legatum_core::{Event, GameState, Hex, PlayerId, UnitId, player_view, resolve_turn};
use legatum_wasm::tape::TapeTurn;

const SOLDIER: UnitId = UnitId(3);
const GUARD: UnitId = UnitId(27);

fn fixture() -> (GameState, Vec<TapeTurn>) {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/playtest3-g18-turn17.json")).unwrap();
    let state = legatum_wasm::json::parse_state_for_rehash(&v["state"].to_string()).unwrap();
    let turns: Vec<TapeTurn> = serde_json::from_value(v["turns"].clone()).unwrap();
    (state, turns)
}

fn soldier_attacks(events: &[Event]) -> Vec<(i32, i32)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Battle {
                attacker,
                defender: Defender::Unit { unit, .. },
                attacker_hp_after,
                defender_hp_after,
                ..
            } if *attacker == SOLDIER && *unit == GUARD => {
                Some((*attacker_hp_after, *defender_hp_after))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_camp_attack_is_one_attack_then_the_soldier_needs_orders() {
    let (mut state, turns) = fixture();
    assert_eq!(state.turn, 17);
    assert_eq!(state.units[&SOLDIER].pos, Hex { q: 18, r: 17 });
    assert_eq!(state.units[&GUARD].pos, Hex { q: 17, r: 17 });
    assert!(
        state
            .barbarians
            .as_ref()
            .unwrap()
            .camp_at(Hex { q: 17, r: 17 })
            .is_some()
    );
    let mut attacks = Vec::new();
    for (i, turn) in turns.iter().enumerate() {
        assert_eq!(turn.turn, state.turn);
        let (next, events) = resolve_turn(&state, &turn.orders);
        attacks.push(soldier_attacks(&events));
        state = next;
        if i == 0 {
            // The recorded battle, unchanged: 100 -> 68 HP against the guard's 100 -> 80.
            assert_eq!(attacks[0], vec![(68, 80)]);
            let u = &state.units[&SOLDIER];
            assert_eq!(
                (u.pos, u.goto, u.attack_target),
                (Hex { q: 18, r: 17 }, None, None),
                "stays where it fought, with no standing attack"
            );
            let view = player_view(&state, PlayerId(0));
            let v = view.units.iter().find(|u| u.id == SOLDIER).unwrap();
            assert!(
                v.goto.is_none() && v.standing_order.is_none(),
                "the client lists it as needing orders"
            );
        }
    }
    // The recorded game had two more attacks (68 -> 36 -> dead). Now: none without orders.
    assert_eq!(attacks[1..], [vec![], vec![]]);
    assert!(state.units.contains_key(&SOLDIER), "the Soldier survives");
}

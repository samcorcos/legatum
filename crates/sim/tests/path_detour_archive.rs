//! Read-only replay of the private turn-39 save behind the Tessitura attack detour. Run with
//! LEGATUM_PATH_DETOUR_SAVE=<save.legatum> cargo test -p legatum-sim --test path_detour_archive -- --ignored
use legatum_core::events::PursuitOutcome;
use legatum_core::{Event, Hex, Order, PlayerId, UnitId, resolve_turn};
use legatum_wasm::tape::Tape;

#[test]
#[ignore = "requires LEGATUM_PATH_DETOUR_SAVE, a private playtest save"]
fn tessitura_turn_38_direct_attack_is_legal() {
    let path = std::env::var("LEGATUM_PATH_DETOUR_SAVE").expect("set archive save path");
    let save: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read save"))
            .expect("save JSON");
    let game: serde_json::Value =
        serde_json::from_str(save["game"].as_str().expect("game string")).expect("game JSON");
    let tape = Tape::from_json(&game["archive"]["tape"].to_string()).expect("archive tape");
    let mut state = tape.initial_state();
    assert_eq!(state.hash(), tape.initial_hash);
    for turn in tape.turns.iter().take_while(|turn| turn.turn < 38) {
        let (next, _) = resolve_turn(&state, &turn.orders);
        assert_eq!(next.hash().to_string(), turn.hash, "turn {}", turn.turn);
        state = next;
    }
    assert_eq!(state.turn, 38);
    let (attacker, enemy) = (UnitId(3), UnitId(24));
    let from = state.units[&attacker].pos;
    let gap = Hex { q: 21, r: 14 };
    let target = state.units[&enemy].pos;
    assert_eq!((from, target), (Hex { q: 20, r: 14 }, Hex { q: 22, r: 14 }));
    assert_eq!((from.distance(gap), gap.distance(target)), (1, 1));
    assert_eq!(state.map.move_cost(gap), Some(1));
    assert!(state.units.values().all(|unit| unit.pos != gap));
    assert!(state.cities.values().all(|city| city.pos != gap));

    // What the player submitted: a five-step route around Tessitura.
    let turn = tape.turns.iter().find(|t| t.turn == 38).expect("turn 38");
    let submitted = &turn.orders[&PlayerId(0)];
    let detour = submitted.iter().find_map(|order| match order {
        Order::MoveUnitPath { unit, to, path, .. } if *unit == attacker && *to == target => {
            Some(path.clone())
        }
        _ => None,
    });
    assert_eq!(detour.map(|p| p.len()), Some(5));
    // The India Soldier steps back this turn, so the attack becomes a pursuit.
    let first_step = |events: &[Event]| {
        events.iter().find_map(|e| match e {
            Event::UnitMoved { unit, to, .. } if *unit == attacker => Some(*to),
            _ => None,
        })
    };
    let pursuit = |events: &[Event]| {
        events.iter().find_map(|e| match e {
            Event::AttackPursuit {
                unit,
                target,
                outcome,
                ..
            } if *unit == attacker && *target == enemy => Some(*outcome),
            _ => None,
        })
    };
    let (_, events) = resolve_turn(&state, &turn.orders);
    assert_eq!(first_step(&events), Some(Hex { q: 20, r: 13 }));
    assert_eq!(pursuit(&events), Some(PursuitOutcome::LostSight));

    // The corrected preview's route, with the other two Oathbearers' orders unchanged.
    let mut orders = turn.orders.clone();
    for order in orders.get_mut(&PlayerId(0)).expect("seat 0 orders") {
        if matches!(order, Order::MoveUnitPath { unit, .. } if *unit == attacker) {
            *order = Order::MoveUnitPath {
                unit: attacker,
                to: target,
                path: vec![gap, target],
                stop_on_block: false,
            };
        }
    }
    let (_, events) = resolve_turn(&state, &orders);
    assert_eq!(first_step(&events), Some(gap), "steps to the shared hex");
    assert_eq!(pursuit(&events), Some(PursuitOutcome::Followed));
}

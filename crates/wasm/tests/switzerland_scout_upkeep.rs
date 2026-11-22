//! Playtest 2026-10-06, from the user's own game: on turn 7 the Switzerland had 0 Gold, a Ranger,
//! a Soldier and two Scouts (one free from a ruin), and the top bar warned that the Ranger would
//! be disbanded. Scouts now cost no upkeep and use no free slot, so the Ranger and Soldier are
//! the two free units and nothing is lost. The test uses a current-rules fixture derived
//! from the user's archived save; the original fixture remains untouched.
use legatum_core::{Event, GameState, PlayerId, UnitKind, economy, resolve_turn};
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);

fn fixture() -> GameState {
    let save: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/playtest-switzerland-bankrupt-turn7.current.legatum"
    ))
    .unwrap();
    legatum_wasm::json::parse_state(save["game"].as_str().unwrap()).unwrap()
}

fn count(state: &GameState, kind: UnitKind) -> usize {
    state.units_of(ME).filter(|u| u.kind == kind).count()
}

#[test]
fn the_users_switzerland_are_not_bankrupt_with_two_scouts() {
    let state = fixture();
    assert_eq!(state.turn, 7);
    assert_eq!(state.player(ME).gold, 0);
    assert_eq!(count(&state, UnitKind::Scout), 2);
    assert_eq!(count(&state, UnitKind::Soldier), 1);
    assert_eq!(count(&state, UnitKind::Ranger), 1);

    assert_eq!(
        economy::upkeep_breakdown(&state, ME).total,
        0,
        "Ranger and Soldier are the first two; Scouts are exempt"
    );
    let (next, events) = resolve_turn(&state, &BTreeMap::new());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::UnitDisbanded { owner, .. } if *owner == ME)),
        "nothing is disbanded"
    );
    assert_eq!(next.units_of(ME).count(), state.units_of(ME).count());
    assert!(next.player(ME).gold >= 0);
}

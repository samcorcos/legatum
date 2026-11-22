use legatum_core::{GameSetup, GameState, PlayerId, UnitKind, new_game, player_view};

#[test]
fn creation_turn_is_saved_and_visible_only_to_the_owner() {
    let mut state = new_game(&GameSetup::all_ai(72));
    let home = state
        .units
        .values()
        .find(|u| u.owner == PlayerId(0))
        .unwrap()
        .pos;
    state.turn = 37;
    let settler = legatum_core::game::spawn_unit(&mut state, PlayerId(0), UnitKind::Settler, home);
    let foreign = legatum_core::game::spawn_unit(&mut state, PlayerId(1), UnitKind::Soldier, home);
    let view = player_view(&state, PlayerId(0));
    assert_eq!(
        view.units
            .iter()
            .find(|u| u.id == settler)
            .unwrap()
            .created_turn,
        Some(37)
    );
    assert_eq!(
        view.units
            .iter()
            .find(|u| u.id == foreign)
            .unwrap()
            .created_turn,
        None
    );
    let loaded: GameState = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(loaded.units[&settler].created_turn, 37);
    assert_eq!(loaded.hash(), state.hash());

    // Legacy units have no age field. Preserve their original initial serialization,
    // and conservatively treat already-existing settlers as eligible for commitment.
    let starting = new_game(&GameSetup::all_ai(72));
    let json = serde_json::to_string(&starting).unwrap();
    assert!(!json.contains("created_turn"));
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert!(loaded.units.values().all(|u| u.created_turn == 1));
}

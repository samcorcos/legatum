use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::{Map, ThreatObservation};
use legatum_core::visibility::{MAX_THREAT_OBSERVATIONS, THREAT_MEMORY_TURNS, update_explored};
use legatum_core::*;

fn sighting() -> (GameState, UnitId, UnitId) {
    let mut setup = GameSetup::all_ai(7);
    setup.players.truncate(3);
    setup.players[1].team = setup.players[0].team;
    setup.players[0].is_human = true;
    let mut state = blank_game(&setup, Map::new(48, 32, Terrain::Grassland));
    let scout = spawn_unit(&mut state, PlayerId(1), UnitKind::Scout, Hex::new(8, 8));
    let spearman = spawn_unit(&mut state, PlayerId(2), UnitKind::Spearman, Hex::new(10, 8));
    state.turn = 20;
    update_explored(&mut state);
    (state, scout, spearman)
}

#[test]
fn observations_are_shared_save_stable_and_never_follow_hidden_units() {
    let (mut state, scout, spearman) = sighting();
    let observed = state.players[0].observed_threats.clone();
    assert_eq!(observed.len(), 1);
    assert_eq!(observed, state.players[1].observed_threats);
    assert_eq!(observed[0].seen_turn, 20);
    state.units.get_mut(&scout).unwrap().pos = Hex::new(1, 1);
    update_explored(&mut state);
    let before = player_view(&state, PlayerId(0));
    assert!(before.units.iter().all(|u| u.id != spearman));
    assert_eq!(before.observed_threats, observed);
    // Neither relocation, changed health/kind, nor death outside vision is evidence.
    state.units.get_mut(&spearman).unwrap().pos = Hex::new(20, 8);
    state.units.get_mut(&spearman).unwrap().kind = UnitKind::Pikeman;
    update_explored(&mut state);
    assert_eq!(player_view(&state, PlayerId(0)).observed_threats, observed);
    state.units.remove(&spearman);
    update_explored(&mut state);
    state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(player_view(&state, PlayerId(0)).observed_threats, observed);
    // Seeing the former tile empty clears it, even before a mutable observation pass.
    state.units.get_mut(&scout).unwrap().pos = Hex::new(8, 8);
    assert!(player_view(&state, PlayerId(0)).observed_threats.is_empty());
    update_explored(&mut state);
    assert!(state.players[0].observed_threats.is_empty());
}

#[test]
fn new_sighting_replaces_the_old_location_and_expiry_does_not_refresh_in_fog() {
    let (mut state, scout, spearman) = sighting();
    state.units.get_mut(&spearman).unwrap().pos = Hex::new(20, 8);
    state.units.get_mut(&scout).unwrap().pos = Hex::new(18, 8);
    state.turn += 1;
    update_explored(&mut state);
    let memory = &state.players[0].observed_threats;
    assert_eq!(memory.len(), 1);
    assert_eq!(memory[0].pos, Hex::new(20, 8));
    assert_eq!(memory[0].seen_turn, 21);
    state.units.get_mut(&scout).unwrap().pos = Hex::new(1, 1);
    state.turn = 21 + THREAT_MEMORY_TURNS - 1;
    update_explored(&mut state);
    assert_eq!(state.players[0].observed_threats.len(), 1);
    state.turn += 1;
    assert!(player_view(&state, PlayerId(0)).observed_threats.is_empty());
    update_explored(&mut state);
    assert!(state.players[0].observed_threats.is_empty());
}

#[test]
fn bounded_memory_keeps_the_newest_observations_deterministically() {
    let (mut state, scout, _) = sighting();
    state.units.get_mut(&scout).unwrap().pos = Hex::new(1, 1);
    state.players[0].observed_threats = (0..MAX_THREAT_OBSERVATIONS + 5)
        .map(|i| ThreatObservation {
            unit: UnitId(1000 + i as u32),
            owner: PlayerId(2),
            kind: UnitKind::Spearman,
            pos: Hex::new(20, 8),
            seen_turn: if i < 5 { 19 } else { 20 },
        })
        .collect();
    update_explored(&mut state);
    assert_eq!(
        state.players[0].observed_threats.len(),
        MAX_THREAT_OBSERVATIONS
    );
    assert!(
        state.players[0]
            .observed_threats
            .iter()
            .all(|o| o.seen_turn == 20)
    );
    let before = state.clone();
    update_explored(&mut state);
    assert_eq!(state, before);
}

#[test]
fn legacy_save_does_not_populate_fog_with_live_threats() {
    let (mut state, scout, spearman) = sighting();
    state.units.get_mut(&scout).unwrap().pos = Hex::new(1, 1);
    let mut json = serde_json::to_value(&state).unwrap();
    for player in json["players"].as_array_mut().unwrap() {
        player.as_object_mut().unwrap().remove("observed_threats");
    }
    let mut loaded: GameState = serde_json::from_value(json).unwrap();
    update_explored(&mut loaded);
    let view = player_view(&loaded, PlayerId(0));
    assert!(view.observed_threats.is_empty());
    assert!(view.units.iter().all(|u| u.id != spearman));
}

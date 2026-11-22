//! Regression (gene `storm_fallen`): a fallen (0 HP) coastal enemy city whose only land approach
//! is held by our own Archer, with our Spearman behind it. Every Hard staging search wants a free
//! tile beside the city and finds none, so without the gene the Spearman holds forever. The pattern
//! of the rules-50 all-Hard base game 21999, where Egypt held the Mongols' Karakorum at 0 HP for
//! 888 turns with its own CombatDrone on the city's only land neighbour on its side.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);
const TARGET: Hex = Hex::new(12, 8);
const APPROACH: Hex = Hex::new(11, 8);

/// A one-hex-wide land spit from our city at (3, 8) east to the enemy city at the tip, the rest
/// Coast: the enemy city's only land neighbour is `APPROACH`.
fn spit() -> (GameState, UnitId) {
    let mut setup = GameSetup::all_ai(21999);
    setup.players.truncate(2);
    (setup.width, setup.height) = (20, 16);
    setup.players[ME.0 as usize].faction = Faction::Egypt;
    setup.players[ENEMY.0 as usize].faction = Faction::Mongols;
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    let mut map = Map::new(20, 16, Terrain::Coast);
    for q in 1..=12 {
        map.get_mut(Hex::new(q, 8)).unwrap().terrain = Terrain::Grassland;
    }
    let mut state = blank_game(&setup, map);
    // Past the opening, as in the traced game (Hard's openings own the first turns).
    state.turn = 300;
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, Hex::new(2, 8)), (ENEMY, TARGET)] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    for c in state.cities.values_mut().filter(|c| c.pos == TARGET) {
        c.hp = 0;
    }
    let archer = spawn_unit(&mut state, ME, UnitKind::Archer, APPROACH);
    let soldier = spawn_unit(&mut state, ME, UnitKind::Spearman, Hex::new(10, 8));
    legatum_core::visibility::update_explored(&mut state);
    assert!(
        TARGET
            .ring(1)
            .into_iter()
            .all(|h| h == APPROACH || state.map.get(h).is_none_or(|t| t.terrain == Terrain::Coast))
    );
    assert_eq!(state.units[&archer].pos, APPROACH);
    (state, soldier)
}

/// Plays our Hard seat (the enemy skips) up to `turns` turns; the turn the city fell, if it did.
fn play(storm: i32, turns: u32) -> Option<u32> {
    let (mut state, _) = spit();
    let mut brain = ScriptedBrain::with_difficulty(Faction::Egypt, 7, ME, AiDifficulty::Hard);
    brain.decisions.storm_fallen = storm;
    for _ in 0..turns {
        let ours = brain.orders(&player_view(&state, ME));
        assert!(validate_orders(&state, ME, &ours).is_empty());
        let turn = state.turn;
        state = resolve_turn(&state, &BTreeMap::from([(ME, ours), (ENEMY, Vec::new())])).0;
        if state
            .cities
            .values()
            .any(|c| c.pos == TARGET && c.owner == ME)
        {
            return Some(turn);
        }
    }
    None
}

#[test]
fn a_fallen_city_behind_our_own_archer_is_stormed_only_with_the_gene() {
    assert_eq!(
        play(0, 12),
        None,
        "gene off: the Spearman holds behind the Archer"
    );
    let taken = play(1, 12).expect("gene on: the city falls");
    // T300 the Spearman and the Archer swap places; T301 the Spearman storms the city.
    assert!(taken <= 302, "taken on turn {taken}");
}

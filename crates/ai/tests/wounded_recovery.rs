//! Wounded recovery through the full Default planner and real turn resolution.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::data;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const OTHER: PlayerId = PlayerId(1);

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn scenario(faction: Faction) -> (GameState, UnitId) {
    let mut setup = GameSetup::standard(7, [faction, Faction::India]);
    setup.width = 24;
    setup.height = 16;
    setup.turn_limit = None;
    setup.barbarians = false;
    setup.players.truncate(2);
    for (seat, player) in setup.players.iter_mut().enumerate() {
        player.team = TeamId(seat as u8);
    }
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    for (owner, pos) in [(ME, at(2, 8)), (OTHER, at(22, 8))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    // Ordinary recovery must work after the one-time faction objective is complete.
    state.player_mut(ME).faction_goal_done = true;
    for i in 0..state.map.len() {
        state.player_mut(ME).explored.set(i);
    }
    state.turn = 30;
    let wounded = spawn_unit(&mut state, ME, UnitKind::Soldier, at(10, 8));
    state.units.get_mut(&wounded).unwrap().hp = 40;
    (state, wounded)
}

fn step(state: &mut GameState, difficulty: AiDifficulty) -> Vec<Order> {
    update_explored(state);
    let view = player_view(state, ME);
    let orders =
        ScriptedBrain::with_difficulty(view.faction_of(ME), 7, ME, difficulty).orders(&view);
    assert!(validate_orders(state, ME, &orders).is_empty(), "{orders:?}");
    let (next, events) = resolve_turn(state, &BTreeMap::from([(ME, orders.clone())]));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { .. })),
        "{events:?}"
    );
    *state = next;
    orders
}

fn at_war(state: &mut GameState) {
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
}

#[test]
fn wounded_sparta_at_war_can_recover_normally() {
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        let (mut state, wounded) = scenario(Faction::Sparta);
        at_war(&mut state);
        for _ in 0..3 {
            let orders = step(&mut state, difficulty);
            // The AI never puts a wounded unit to sleep; an idle one still gains the flat 2.
            assert!(!orders.contains(&Order::SleepUnit {
                unit: wounded,
                until_healed: false
            }));
        }
    }
}

#[test]
fn wounded_units_that_can_heal_in_the_field_still_rest_there() {
    let (mut state, wounded) = scenario(Faction::Byzantium);
    let before = state.units[&wounded].clone();
    let orders = step(&mut state, AiDifficulty::Hard);
    assert!(orders.contains(&Order::SkipUnit { unit: wounded }));
    assert_eq!(state.units[&wounded].pos, before.pos);
    assert_eq!(state.units[&wounded].hp - before.hp, data::HEAL_FIELD);
}

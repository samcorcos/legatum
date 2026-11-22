//! A rules-49 saved state upgrades to rules 50 for simulation (`legatum_wasm::json::upgrade_from_49`)
//! and plays on with Hard in every seat. The fixture is the T148 state of the "India: strike the
//! leader" Lab scenario as it stood at rules 49 (the owner's recorded game): the Celts had sacked
//! Dyrrachium, an original capital, and the Mongols held Lugdunum.
use std::io::Read;

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, CityId, Faction, player_view, resolve_turn};
use legatum_wasm::json::{parse_state, upgrade_from_49};
use legatum_wasm::session::OrderMap;

fn rules_49_json() -> String {
    let mut out = String::new();
    flate2::read::GzDecoder::new(
        &include_bytes!("fixtures-rules49/india-strike-the-leader-T148.state.json.gz")[..],
    )
    .read_to_string(&mut out)
    .unwrap();
    out
}

#[test]
fn a_rules_49_state_upgrades_and_plays_ten_turns() {
    let json = rules_49_json();
    // The app path still refuses it.
    assert!(
        parse_state(&json)
            .unwrap_err()
            .contains("incompatible rules version")
    );
    let mut state = upgrade_from_49(&json).expect("upgrade");
    assert_eq!(state.turn, 148);
    let celts = state
        .players
        .iter()
        .find(|p| p.faction == Faction::Celts)
        .unwrap()
        .id;
    // The sacked capital keeps its point for good; nobody else's 49 list carries over.
    assert_eq!(
        state.player(celts).capitals_sacked,
        vec![(CityId(3), "Dyrrachium".to_string())]
    );
    assert!(
        state
            .players
            .iter()
            .filter(|p| p.id != celts)
            .all(|p| p.capitals_sacked.is_empty())
    );
    let mut brains: Vec<(legatum_core::PlayerId, ScriptedBrain)> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            (
                p.id,
                ScriptedBrain::with_difficulty(
                    p.faction,
                    legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                    p.id,
                    AiDifficulty::Hard,
                ),
            )
        })
        .collect();
    for _ in 0..10 {
        if state.finished {
            break;
        }
        let mut orders = OrderMap::new();
        for (id, brain) in &mut brains {
            if state.player(*id).alive {
                orders.insert(*id, brain.orders(&player_view(&state, *id)));
            }
        }
        state = resolve_turn(&state, &orders).0;
    }
    assert!(
        state.finished || state.turn == 158,
        "stopped at T{}",
        state.turn
    );
    legatum_core::validation::check_state(&state).unwrap();
}

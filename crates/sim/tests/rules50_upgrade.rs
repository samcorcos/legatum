//! A rules-50 saved state upgrades to the current rules (53) for simulation
//! (`legatum_wasm::json::upgrade_from_49`, which also reads 50, 51 and 52) and plays on with Hard in
//! every seat. Rules 51 to 53 change no field (52 renamed identifiers, which still load through serde
//! aliases), so the upgrade is the identity plus the version stamp; a rules-51 or rules-52 save
//! takes the same path. The fixture is the T148 state of the "India:
//! strike the leader" Lab scenario as it stood at rules 50.
use std::io::Read;

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, player_view, resolve_turn};
use legatum_wasm::json::{parse_state, parse_state_for_rehash, state_to_json, upgrade_from_49};
use legatum_wasm::session::OrderMap;

fn rules_50_json() -> String {
    let mut out = String::new();
    flate2::read::GzDecoder::new(
        &include_bytes!("fixtures-rules50/india-strike-the-leader-T148.state.json.gz")[..],
    )
    .read_to_string(&mut out)
    .unwrap();
    out
}

#[test]
fn a_rules_50_state_upgrades_unchanged_and_plays_ten_turns() {
    let json = rules_50_json();
    assert!(json.contains("\"save_version\":50"));
    // The app path still refuses it.
    assert!(
        parse_state(&json)
            .unwrap_err()
            .contains("incompatible rules version: saved Some(50)")
    );
    let mut state = upgrade_from_49(&json).expect("upgrade");
    assert_eq!(state.turn, 148);
    // The identity: the same state as a plain read that skips the version check.
    assert_eq!(state, parse_state_for_rehash(&json).unwrap());
    // Saved again, it is a current save the app loads.
    let save = state_to_json(&state);
    assert!(save.contains(&format!("\"save_version\":{}", legatum_core::RULES_VERSION)));
    assert_eq!(parse_state(&save).unwrap(), state);
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

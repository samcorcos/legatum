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

/// The fixture predates the 2026-11-22 identifier rename (rules 52): its units and buildings
/// carry the old names (`Soldier`, `Laboratory`, ...). They load through the serde aliases
/// on `UnitKind` and `BuildingKind`, and the engine writes only the new names back.
#[test]
fn a_rules_49_state_names_the_old_identifiers_and_loads_as_the_renamed_kinds() {
    use legatum_core::{BuildingKind, UnitKind};
    let json = rules_49_json();
    let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
    let state = upgrade_from_49(&json).expect("upgrade");
    let units = raw["units"].as_object().unwrap();
    let cities = raw["cities"].as_object().unwrap();
    for (old, kind, new) in [
        ("Soldier", UnitKind::Spearman, "Spearman"),
        ("Ranger", UnitKind::Archer, "Archer"),
        ("Marksman", UnitKind::Crossbowman, "Crossbowman"),
        ("AssaultTrooper", UnitKind::Musketman, "Musketman"),
        ("Heir", UnitKind::HeirToTheThrone, "HeirToTheThrone"),
    ] {
        let recorded = units.values().filter(|u| u["kind"] == old).count();
        assert!(recorded > 0, "the fixture has no {old}");
        assert_eq!(units.values().filter(|u| u["kind"] == new).count(), 0);
        let loaded = state.units.values().filter(|u| u.kind == kind).count();
        assert_eq!(loaded, recorded, "{old} loads as {kind:?}");
    }
    for (old, kind) in [
        ("Laboratory", BuildingKind::Library),
        ("College", BuildingKind::University),
        ("Clearinghouse", BuildingKind::Bank),
        ("Hydroponics", BuildingKind::Mill),
        ("MachineShop", BuildingKind::Forge),
        ("RelayUplink", BuildingKind::GreatLibrary),
        ("WarShrine", BuildingKind::Agoge),
    ] {
        let holds =
            |c: &serde_json::Value| c["buildings"].as_array().unwrap().iter().any(|b| b == old);
        let recorded = cities.values().filter(|c| holds(c)).count();
        assert!(recorded > 0, "the fixture has no {old}");
        let loaded = state
            .cities
            .values()
            .filter(|c| c.buildings.contains(&kind))
            .count();
        assert_eq!(loaded, recorded, "{old} loads as {kind:?}");
    }
    // Written back, no unit or building keeps an old name.
    let saved = serde_json::to_value(&state).unwrap();
    let old_units = ["Soldier", "Ranger", "Marksman", "AssaultTrooper", "Heir"];
    assert!(
        saved["units"]
            .as_object()
            .unwrap()
            .values()
            .all(|u| !old_units.contains(&u["kind"].as_str().unwrap()))
    );
    let old_buildings = ["Laboratory", "College", "Clearinghouse", "Hydroponics"];
    assert!(saved["cities"].as_object().unwrap().values().all(|c| {
        c["buildings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| !old_buildings.contains(&b.as_str().unwrap()))
    }));
}

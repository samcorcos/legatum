//! Records written before the 2026-11-22 identifier rename (rules 52) name units and buildings
//! by their old identifiers (`Soldier`, `Ranger`, `Laboratory`, ...). They still parse, through
//! the serde aliases in `legatum_core::types`. These two fixtures are such records: a recorded
//! tape of orders and a saved state (only its `save_version` is refreshed at a rules bump).
use legatum_core::{Order, ProductionItem, UnitKind};
use legatum_wasm::tape::Tape;

const TAPE: &str = include_str!("fixtures/playtest3-g2.tape.json");
const STATE: &str = include_str!("fixtures/playtest3-g2-turn6.current.json");

#[test]
fn a_tape_recorded_with_the_old_names_loads_as_the_renamed_kinds() {
    assert!(TAPE.contains(r#"{"kind":"Unit","unit":"Soldier"}"#));
    assert!(TAPE.contains(r#"{"kind":"Unit","unit":"Ranger"}"#));
    let tape = Tape::from_json(TAPE).unwrap();
    let queued: Vec<UnitKind> = tape
        .turns
        .iter()
        .flat_map(|turn| turn.orders.values().flatten())
        .flat_map(|order| match order {
            Order::SetProductionQueue { queue, .. } => queue.clone(),
            Order::SetProduction { item, .. } => vec![*item],
            _ => Vec::new(),
        })
        .filter_map(|item| match item {
            ProductionItem::Unit { unit } => Some(unit),
            _ => None,
        })
        .collect();
    assert!(queued.contains(&UnitKind::Spearman), "{queued:?}");
    assert!(queued.contains(&UnitKind::Archer), "{queued:?}");
    // Written again, the tape names only the new identifiers.
    let written = tape.to_json();
    assert!(!written.contains("\"Soldier\"") && !written.contains("\"Ranger\""));
    assert!(written.contains(r#""unit":"Spearman""#));
    assert_eq!(
        Tape::from_json(&written).unwrap().turns.len(),
        tape.turns.len()
    );
}

#[test]
fn a_saved_state_with_the_old_names_loads_as_the_renamed_kinds() {
    let raw: serde_json::Value = serde_json::from_str(STATE).unwrap();
    let recorded = |kind: &str| {
        raw["units"]
            .as_object()
            .unwrap()
            .values()
            .filter(|u| u["kind"] == kind)
            .count()
    };
    assert!(recorded("Soldier") > 0, "the fixture was re-serialized");
    assert_eq!(recorded("Spearman"), 0);
    let state = legatum_wasm::json::parse_state(STATE).unwrap();
    let loaded = |kind: UnitKind| state.units.values().filter(|u| u.kind == kind).count();
    assert_eq!(loaded(UnitKind::Spearman), recorded("Soldier"));
    assert_eq!(loaded(UnitKind::Archer), recorded("Ranger"));
    let saved = legatum_wasm::json::state_to_json(&state);
    assert!(!saved.contains("\"kind\":\"Soldier\""));
    assert!(saved.contains("\"kind\":\"Spearman\""));
}

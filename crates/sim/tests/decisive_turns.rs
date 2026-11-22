//! Recorded, hash-verified decision states. Run the ignored gaps with
//! `tools/decision-audit/run-decisive.sh` after an AI change.
use legatum_ai::{Brain, OpponentMemory, ScriptedBrain};
use legatum_core::{AiDifficulty, PlayerId, player_view};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

fn contains(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => a
            .iter()
            .all(|(k, v)| b.get(k).is_some_and(|x| contains(v, x))),
        (Value::Array(a), Value::Array(b)) => {
            a.len() <= b.len() && a.iter().zip(b).all(|(x, y)| contains(x, y))
        }
        _ => expected == actual,
    }
}

fn current_faction_names(raw: &str) -> String {
    // The 8.6 recording predates the faction rename. Keep the recorded bytes and hash intact;
    // translate only the in-memory copy used by this current-engine choice check.
    let mut text = raw.to_owned();
    for (old, new) in [
        ("Tollkeepers", "Switzerland"),
        ("Tollkeeper", "Switzerland"),
        ("Choir", "Japan"),
        ("Remnants", "Byzantium"),
        ("Remnant", "Byzantium"),
        ("Crucible", "Germany"),
        ("Veyr", "Celts"),
        ("Glassroad", "Mongols"),
        ("Oathbound", "Sparta"),
        ("Signalwrights", "Athens"),
        ("Signalwright", "Athens"),
    ] {
        text = text.replace(old, new);
    }
    text
}

fn check(id: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/decisive");
    let cases: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    let case = cases.iter().find(|c| c["id"] == id).unwrap();
    let game = case["engine"].as_str().unwrap();
    let turn = case["turn"].as_u64().unwrap();
    let stem = format!("{game}-T{turn}");
    let recorded = std::fs::read_to_string(root.join(format!("{stem}-state.json"))).unwrap();
    assert_eq!(
        legatum_core::state::fnv1a(recorded.as_bytes()).to_string(),
        case["recorded_hash"].as_str().unwrap(),
        "fixture changed since the verified replay"
    );
    let state: legatum_core::GameState =
        serde_json::from_str(&current_faction_names(&recorded)).unwrap();
    assert_eq!(state.turn, turn as u32);
    let seat = PlayerId(case["seat"].as_u64().unwrap() as u8);
    let mut brain = ScriptedBrain::with_difficulty(
        state.player(seat).faction,
        legatum_wasm::session::brain_seed(state.setup.seed, seat),
        seat,
        AiDifficulty::Hard,
    );
    let memory: BTreeMap<PlayerId, OpponentMemory> = serde_json::from_str(&current_faction_names(
        &std::fs::read_to_string(root.join(format!("{stem}-memory.json"))).unwrap(),
    ))
    .unwrap();
    if let Some(m) = memory.get(&seat) {
        brain.restore_opponent_memory(m.clone());
    }
    let orders = brain.orders(&player_view(&state, seat));
    let actual: Vec<Value> = orders
        .iter()
        .map(|o| serde_json::to_value(o).unwrap())
        .collect();
    assert!(
        actual
            .iter()
            .any(|order| contains(&case["expected"], order)),
        "{id}: expected {}, got {}",
        case["expected"],
        serde_json::to_string(&actual).unwrap()
    );
}

macro_rules! gap {
    ($name:ident) => {
        #[test]
        #[ignore = "known AI choice gap; run explicitly after a candidate fix"]
        fn $name() {
            check(stringify!($name));
        }
    };
}
gap!(germany_t33_army);
gap!(germany_t17_settler);
gap!(swiss_t153_garrison);
gap!(swiss_t140_bomber);
gap!(swiss_t59_composite_archer);
gap!(swiss_t45_settler);
gap!(swiss_t147_refuse_peace);
gap!(oathbound_t29_war);

//! Damaged saves never crash the engine: a valid save with a few numbers changed either fails to
//! load with an error, or loads into a state every player can view and the turn can resolve.
//! Seeded and deterministic; `LEGATUM_SAVE_FUZZ_CASES` raises the case count locally.

use legatum_core::rng::Rng;
use legatum_core::{GameSetup, PlayerId, player_view, resolve_turn};
use legatum_wasm::json::{parse_state, state_to_json};
use legatum_wasm::session::{BrainKind, OrderMap, Session};
use serde_json::Value;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// A small all-AI game some turns in: cities, units, diplomacy and barbarians all exist.
fn played(seed: u64, turns: u32) -> (Session, OrderMap) {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(4);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    let mut orders = OrderMap::new();
    for _ in 0..turns {
        orders = session.ai_orders();
        session.step(&orders);
    }
    let next = session.ai_orders();
    (session, if next.is_empty() { orders } else { next })
}

/// Every number in the save, grouped by its top-level section so the small sections (units,
/// cities, diplomacy) are picked as often as the large ones (map tiles, explored bitsets).
fn numeric_leaves(v: &Value) -> BTreeMap<String, Vec<Vec<String>>> {
    fn walk(v: &Value, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        match v {
            Value::Number(_) => out.push(path.clone()),
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    path.push(i.to_string());
                    walk(item, path, out);
                    path.pop();
                }
            }
            Value::Object(fields) => {
                for (k, item) in fields {
                    path.push(k.clone());
                    walk(item, path, out);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    let mut groups = BTreeMap::new();
    for (section, item) in v.as_object().unwrap() {
        if section == "save_version" {
            continue;
        }
        let mut leaves = Vec::new();
        walk(item, &mut vec![section.clone()], &mut leaves);
        if !leaves.is_empty() {
            groups.insert(section.clone(), leaves);
        }
    }
    groups
}

fn slot<'a>(v: &'a mut Value, path: &[String]) -> &'a mut Value {
    path.iter().fold(v, |v, key| match v {
        Value::Array(items) => &mut items[key.parse::<usize>().unwrap()],
        _ => &mut v[key.as_str()],
    })
}

fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> &'a T {
    &items[rng.range(0, items.len() as i64 - 1) as usize]
}

/// Every state real play reaches must load: three full all-Hard games, saved and loaded through
/// `parse_state` (and so `check_state`) every few turns and at the end, coming back identical.
#[test]
fn every_state_of_full_all_hard_games_round_trips_through_save_validation() {
    const EVERY: u32 = 5;
    const CAP: u32 = 450;
    for seed in [1, 2, 3] {
        let mut setup = GameSetup::all_ai(seed);
        for p in &mut setup.players {
            p.difficulty = legatum_core::AiDifficulty::Hard;
        }
        let mut session = Session::new(&setup, |_| BrainKind::Scripted);
        let check = |state: &legatum_core::GameState| {
            let loaded = parse_state(&state_to_json(state)).unwrap_or_else(|e| {
                panic!(
                    "seed {seed} turn {}: a reached state was refused: {e}",
                    state.turn
                )
            });
            assert!(
                loaded == *state,
                "seed {seed} turn {}: round trip",
                state.turn
            );
        };
        while !session.state.finished && session.state.turn < CAP {
            if session.state.turn.is_multiple_of(EVERY) {
                check(&session.state);
            }
            let orders = session.ai_orders();
            session.step(&orders);
        }
        assert!(session.state.finished, "seed {seed}: the game must finish");
        check(&session.state);
    }
}

#[test]
fn a_mutated_save_loads_views_and_resolves_without_panicking() {
    let cases: u64 = std::env::var("LEGATUM_SAVE_FUZZ_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(150);
    let (session, orders) = played(17, 40);
    let base: Value = serde_json::from_str(&state_to_json(&session.state)).unwrap();
    parse_state(&base.to_string()).expect("the unmutated save loads");
    let groups = numeric_leaves(&base);
    let sections: Vec<&String> = groups.keys().collect();
    let no_orders = BTreeMap::new();
    let mut loaded = 0;
    for case in 0..cases {
        let mut rng = Rng::derive(0x5a5e_f022, &[case]);
        let mut save = base.clone();
        let mut changes = Vec::new();
        for _ in 0..rng.range(1, 3) {
            let section = *pick(&mut rng, &sections);
            let path = pick(&mut rng, &groups[section]);
            let leaf = slot(&mut save, path);
            let old = leaf.as_i64().unwrap_or(0);
            let new = match rng.range(0, 11) {
                0 => -1,
                1 => 0,
                2 => 1,
                3 => old + 1,
                4 => old - 1,
                5 => 7,
                6 => 200,
                7 => 255,
                8 => 65_536,
                9 => 9_999_999,
                10 => i64::from(i32::MAX),
                _ => rng.range(0, 100),
            };
            *leaf = Value::from(new);
            changes.push(format!("{}={new}", path.join(".")));
        }
        let Ok(state) = parse_state(&save.to_string()) else {
            continue;
        };
        loaded += 1;
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            for p in 0..state.players.len() {
                player_view(&state, PlayerId(p as u8));
            }
            resolve_turn(&state, &no_orders);
            resolve_turn(&state, &orders);
        }));
        assert!(
            outcome.is_ok(),
            "case {case}: a save that loads must not panic; changes: {changes:?}"
        );
    }
    eprintln!("save fuzz: {loaded} of {cases} mutated saves loaded");
    assert!(loaded > 0, "some mutations must leave a loadable save");
}

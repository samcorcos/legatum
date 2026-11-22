//! Replay a saved game's tape on the core alone and write what one player saw before chosen turns.
//!
//! tape_views <save.legatum> <out-dir> <player> <turn,turn,...> [<unit> <q> <r>]
//!
//! For each turn writes `view-t<N>.json` (the player's `PlayerView` before that turn resolves, as
//! the player saw it) and `orders-t<N>.json` (the orders every player gave that turn); with a unit
//! and a target, also `forecast-t<N>.json` (the melee forecast the player had). Every turn's state
//! hash is checked against the tape. Source of the p881 horseman fixture
//! (`app/src/test/p881/situation.json`).
use legatum_core::{AttackMode, GameSetup, Hex, Order, PlayerId, UnitId};
use serde_json::Value;
use std::collections::BTreeMap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    assert!(
        a.len() >= 5,
        "usage: tape_views <save> <out-dir> <player> <turn,turn,...> [<unit> <q> <r>]"
    );
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let game: Value = serde_json::from_str(outer["game"].as_str().expect("game JSON")).unwrap();
    let tape = &game["archive"]["tape"];
    let me = PlayerId(a[3].parse().unwrap());
    let turns: Vec<u32> = a[4].split(',').map(|t| t.parse().unwrap()).collect();
    let watch = a.get(5).map(|u| {
        (
            UnitId(u.parse().unwrap()),
            Hex {
                q: a[6].parse().unwrap(),
                r: a[7].parse().unwrap(),
            },
        )
    });
    let mut setup = tape["setup"].clone();
    if let Some(seed) = setup["seed"].as_str() {
        setup["seed"] = Value::from(seed.parse::<u64>().unwrap());
    }
    let setup: GameSetup = serde_json::from_value(setup).expect("setup");
    let mut state = legatum_core::new_game(&setup);
    for t in tape["turns"].as_array().unwrap() {
        let turn = t["turn"].as_u64().unwrap() as u32;
        let orders: BTreeMap<PlayerId, Vec<Order>> =
            serde_json::from_value(t["orders"].clone()).expect("orders");
        if turns.contains(&turn) {
            let mut seen = state.clone();
            legatum_core::visibility::update_explored(&mut seen);
            let view = legatum_core::player_view(&seen, me);
            let write = |name: String, json: String| {
                std::fs::write(format!("{}/{name}-t{turn}.json", a[2]), json).unwrap()
            };
            write("view".into(), serde_json::to_string(&view).unwrap());
            write("orders".into(), serde_json::to_string(&orders).unwrap());
            if let Some((unit, target)) = watch {
                let forecast =
                    legatum_core::forecast_attack_mode(&view, unit, target, AttackMode::Melee);
                write("forecast".into(), serde_json::to_string(&forecast).unwrap());
            }
        }
        state = legatum_core::resolve_turn(&state, &orders).0;
        assert_eq!(
            state.hash().to_string(),
            t["hash"].as_str().unwrap(),
            "hash at turn {turn}"
        );
        if turns.iter().all(|&want| want <= turn) {
            break;
        }
    }
}

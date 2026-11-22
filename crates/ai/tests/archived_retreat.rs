//! The real T243 regression; see fixtures/retreat-t243.md for replay provenance.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(1);

fn decide(state: &GameState) -> Vec<Order> {
    let view = player_view(state, ME);
    ScriptedBrain::new(view.faction_of(ME), state.setup.seed, ME).orders(&view)
}

#[test]
fn t243_response_survives_traffic_id_permutations_and_save_load() {
    let saved: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/retreat-t243-state.json")).unwrap();
    let mut original: GameState = serde_json::from_value(saved).unwrap();
    // The archive predates strategic resources (G11): give every seat Iron and Coal deposits
    // inside its borders so the recorded Trooper orders of the other seats stay supplied.
    let ids: Vec<_> = original.players.iter().map(|p| p.id).collect();
    for p in ids {
        let spots: Vec<usize> = (0..original.map.tiles.len())
            .filter(|&i| {
                let t = &original.map.tiles[i];
                t.owner == Some(p) && t.resource.is_none() && t.terrain.is_land()
            })
            .take(12)
            .collect();
        for (n, i) in spots.into_iter().enumerate() {
            original.map.tiles[i].resource = Some(if n % 2 == 0 {
                Resource::Iron
            } else {
                Resource::Coal
            });
        }
    }
    // This pre-LOS archive saw Ranger 68 through (0, 16). Keep that threat visible
    // under terrain sight so the traffic regression still includes its volley.
    original.map.get_mut(Hex::new(0, 16)).unwrap().terrain = Terrain::Plains;
    for ids in [
        [6, 148, 162],
        [6, 162, 148],
        [148, 6, 162],
        [148, 162, 6],
        [162, 6, 148],
        [162, 148, 6],
    ] {
        let mut state = original.clone();
        for id in [6, 148, 162] {
            state.units.remove(&UnitId(id));
        }
        for (old, new) in [6, 148, 162].into_iter().zip(ids) {
            let mut unit = original.units[&UnitId(old)].clone();
            unit.id = UnitId(new);
            state.units.insert(unit.id, unit);
        }
        let retreating = UnitId(ids[2]);
        let before = state.units[&retreating].clone();
        assert_eq!(before.hp, 54);
        let visible = player_view(&state, ME);
        let threats: Vec<_> = visible
            .units
            .iter()
            .filter(|u| {
                u.military
                    && visible
                        .players
                        .iter()
                        .any(|p| p.id == u.owner && p.team != visible.team)
            })
            .collect();
        assert!(threats.iter().any(|u| u.id == UnitId(68)));
        let orders = decide(&state);
        for loaded in [false, true] {
            let state = if loaded {
                serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap()
            } else {
                state.clone()
            };
            let actual = decide(&state);
            assert_eq!(actual, orders, "save/load changed the decision");
            assert!(validate_orders(&state, ME, &actual).is_empty());
            // Preserve the archived fixture. This seat is replanned below; its historical
            // item-less purchase predates independent buying and is not replayed.
            let mut recorded: serde_json::Value =
                serde_json::from_str(include_str!("fixtures/retreat-t243-orders.json")).unwrap();
            recorded["1"]
                .as_array_mut()
                .unwrap()
                .retain(|o| o["type"] != "BuyItem");
            // Another empire's archived shot now lacks visibility under terrain LOS;
            // it is unrelated to the retreating unit and cannot be replayed legally.
            recorded["0"]
                .as_array_mut()
                .unwrap()
                .retain(|o| o["type"] != "RangedAttack" || o["unit"] != 42);
            let mut simultaneous: BTreeMap<PlayerId, Vec<Order>> =
                serde_json::from_value(recorded).unwrap();
            // Include Soldier 6's movement and Soldier 148's competing first step.
            // Either routine mover may yield to an escape during coordination.
            for (unit, to) in [(6, Hex::new(-2, 17)), (148, Hex::new(-1, 16))] {
                assert!(simultaneous[&ME].contains(&Order::MoveUnit {
                    unit: UnitId(unit),
                    to,
                }));
            }
            simultaneous.insert(ME, actual.clone());
            let (next, events) = resolve_turn(&state, &simultaneous);
            let failed: Vec<_> = events
                .iter()
                // Other seats' archived builds may predate today's production rules;
                // the retreating seat's orders must still hold under terrain sight.
                .filter(|e| matches!(e, Event::OrderFailed { player, .. } if *player == ME))
                .collect();
            assert!(failed.is_empty(), "{failed:?}");
            let escaped = &next.units[&retreating];
            assert!(
                escaped.hp > 0 && escaped.hp < before.hp,
                "the pre-move volley still hits"
            );
            if escaped.pos == before.pos {
                // Terrain sight now lets the city counterfire, making a hold a valid
                // response. The traffic permutation must still preserve that order.
                assert!(actual.contains(&Order::SkipUnit { unit: retreating }));
            } else {
                assert!(
                    threats
                        .iter()
                        .all(|e| escaped.pos.distance(e.pos) > e.max_moves.max(e.range)),
                    "retreat must leave nearby visible threats: {escaped:?}"
                );
            }
            assert!(!events.iter().any(|e| matches!(e,
                Event::Bounce { units, .. } if units.iter().all(|id| state.units[id].owner == ME)
            )), "friendly bounce with ids={ids:?}: {events:?}");
        }
    }
}

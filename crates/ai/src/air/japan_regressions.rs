use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, CityId, PlayerId, ProductionItem, UnitId};

fn load<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    serde_json::from_reader(flate2::read::GzDecoder::new(bytes)).unwrap()
}
fn germany() -> (PlayerView, ScriptedBrain) {
    let mut view: PlayerView = load(include_bytes!(
        "../fixtures/japan-1026/T185-p2.view.json.gz"
    ));
    // Recorded before the Machine Age, when Flight gave Bombers and Fighters; they now need Air
    // Power, which stands in for the old Flight here.
    view.research.known.push(TechId::AirPower);
    let seed =
        legatum_core::rng::Rng::derive(3_811_224_394_289_556, &[legatum_core::rng::purpose::AI, 2])
            .next_u64();
    let mut brain =
        ScriptedBrain::with_difficulty(view.faction_of(view.me), seed, view.me, AiDifficulty::Hard);
    // This archived air decision predates combat-valued production. Isolate the air gene.
    brain.decisions.modernization_value = 0;
    // Likewise the recorded decision predates `air_defense` defaulting on.
    brain.decisions.air_defense = 0;
    // And `march_past_reach` (a unit stages toward a campaign city instead of waiting).
    brain.decisions.march_past_reach = 0;
    // The archived decision also predates threat-aware production valuation.
    brain.decisions.production_threat_agreement = 0;
    let memory: Option<crate::OpponentMemory> = load(include_bytes!(
        "../fixtures/japan-1026/T185-p2.memory.json.gz"
    ));
    brain.restore_opponent_memory(memory.unwrap());
    (view, brain)
}

#[test]
fn germany_185_builds_a_fighter_before_owning_a_bomber() {
    let (view, mut brain) = germany();
    let old = brain.orders(&view);
    let recorded: Vec<Order> = load(include_bytes!(
        "../fixtures/japan-1026/T185-p2.recorded.json.gz"
    ));
    assert_eq!(
        old, recorded,
        "zero genes preserve the recorded Germany decision"
    );
    assert!(old.contains(&Order::SetProduction {
        city: CityId(3),
        item: ProductionItem::Unit {
            unit: UnitKind::Bomber
        }
    }));
    let (_, mut brain) = germany();
    brain.decisions.air_defense = 100;
    let new = brain.orders(&view);
    assert!(
        new.contains(&Order::SetProduction {
            city: CityId(3),
            item: ProductionItem::Unit {
                unit: UnitKind::Fighter
            }
        }),
        "{new:?}"
    );
}

#[test]
fn germany_214_a_defensive_fighter_intercepts_the_recorded_bombing() {
    let mut json: serde_json::Value =
        load(include_bytes!("../fixtures/japan-1026/T214.state.json.gz"));
    json["setup"]["seed"] = serde_json::json!(3_811_224_394_289_556u64);
    let mut state: legatum_core::GameState = serde_json::from_value(json).unwrap();
    // Explicit intervention: the one German aircraft was built as a Fighter, not a Bomber.
    state
        .aircraft
        .iter_mut()
        .find(|a| a.id == UnitId(221))
        .unwrap()
        .kind = UnitKind::Fighter;
    let view = legatum_core::player_view(&state, PlayerId(2));
    let orders = air_orders_with_policy(&view, true, true, &[], 100, true);
    assert!(
        !orders.iter().any(|o| matches!(
            o,
            Order::AirStrike {
                plane: UnitId(221),
                ..
            }
        )),
        "guard Cologne: {orders:?}"
    );
    let mut all = std::collections::BTreeMap::new();
    all.insert(PlayerId(2), orders);
    all.insert(
        PlayerId(0),
        [162, 175, 194, 210]
            .into_iter()
            .map(|id| Order::AirStrike {
                plane: UnitId(id),
                target: legatum_core::Hex::new(10, 21),
            })
            .collect(),
    );
    for (p, orders) in &all {
        assert!(legatum_core::validate_orders(&state, *p, orders).is_empty());
    }
    let (_, events) = legatum_core::resolve_turn(&state, &all);
    let city_hits = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                legatum_core::Event::Battle {
                    attacker_owner: PlayerId(0),
                    defender: legatum_core::events::Defender::City { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        city_hits, 3,
        "one Fighter cancels one of four strikes, not the whole wave: {events:?}"
    );
}

#[test]
fn bomber_rebases_from_an_idle_rear_hangar_then_strikes() {
    let mut v = ViewBuilder::new(30, 16)
        .city(1, ME, off(2, 8), 3)
        .city(2, ME, off(11, 8), 3)
        .city(3, ENEMY, off(17, 8), 3)
        .unit(2, ENEMY, UnitKind::Spearman, off(16, 8))
        .aircraft(30, UnitKind::Bomber, 1)
        .build();
    assert!(air_orders_with_policy(&v, true, true, &[], 0, false).is_empty());
    let orders = air_orders_with_policy(&v, true, true, &[], 0, true);
    assert_eq!(
        orders,
        vec![Order::Rebase {
            plane: UnitId(30),
            to: CityId(2)
        }]
    );
    v.aircraft[0].city = CityId(2);
    assert!(
        air_orders_with_policy(&v, true, true, &[], 0, true)
            .iter()
            .any(|o| matches!(
                o,
                Order::AirStrike {
                    plane: UnitId(30),
                    ..
                }
            ))
    );
}

#[test]
fn rebase_reserves_the_last_hangar_slot_in_aircraft_id_order() {
    let v = ViewBuilder::new(30, 16)
        .city(1, ME, off(2, 8), 3)
        .city(2, ME, off(11, 8), 3)
        .city(3, ENEMY, off(17, 8), 3)
        .aircraft(30, UnitKind::Bomber, 1)
        .aircraft(31, UnitKind::Bomber, 1)
        .aircraft(32, UnitKind::Bomber, 2)
        .build();
    let orders = air_orders_with_policy(&v, true, true, &[], 0, true);
    assert_eq!(
        orders
            .iter()
            .filter(|o| matches!(o, Order::Rebase { to: CityId(2), .. }))
            .count(),
        1
    );
    assert!(orders.contains(&Order::Rebase {
        plane: UnitId(30),
        to: CityId(2)
    }));
}

#[test]
fn fighter_production_counts_coverage_and_a_pending_fighter() {
    let mut v = ViewBuilder::new(24, 16)
        .city(1, ME, off(5, 8), 5)
        .city(2, ENEMY, off(12, 8), 3)
        .era_of(2, data::era_index(TechId::Flight))
        .build();
    // The fixture's enemy is PlayerId(2).
    let value = |v: &PlayerView, orders: &[Order]| {
        production_value_with_policy(v, &v.cities[0], UnitKind::Fighter, 100, orders)
    };
    assert!(value(&v, &[]) > 1800);
    let pending = [Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Fighter,
        },
    }];
    assert_eq!(value(&v, &pending), 0);
    v.aircraft.push(legatum_core::view::AircraftView {
        id: UnitId(30),
        owner: ME,
        kind: UnitKind::Fighter,
        city: CityId(1),
        hp: 100,
        level: 0,
        xp: 0,
        xp_next: None,
        upgrade_cost: None,
        upgrade_options: Vec::new(),
        unsupplied: None,
    });
    assert_eq!(value(&v, &[]), 0);
}

#[test]
fn germany_186_finishes_its_useful_bomber_instead_of_restarting_infantry() {
    let view: PlayerView = load(include_bytes!(
        "../fixtures/japan-1026/T186-p2.view.json.gz"
    ));
    let memory: Option<crate::OpponentMemory> = load(include_bytes!(
        "../fixtures/japan-1026/T186-p2.memory.json.gz"
    ));
    let mut brain = germany().1;
    brain.restore_opponent_memory(memory.clone().unwrap());
    let old = brain.orders(&view);
    assert!(old.contains(&Order::SetProduction {
        city: CityId(3),
        item: ProductionItem::Unit {
            unit: UnitKind::Infantry
        }
    }));
    brain.restore_opponent_memory(memory.unwrap());
    brain.decisions.air_commitment = 1;
    let new = brain.orders(&view);
    assert!(
        !new.iter().any(|o| matches!(
            o,
            Order::SetProduction {
                city: CityId(3),
                ..
            }
        )),
        "finish the Bomber: {new:?}"
    );
}

#[test]
fn germany_214_evacuates_its_actual_bomber_before_cologne_falls() {
    let mut json: serde_json::Value =
        load(include_bytes!("../fixtures/japan-1026/T214.state.json.gz"));
    json["setup"]["seed"] = serde_json::json!(3_811_224_394_289_556u64);
    let state: legatum_core::GameState = serde_json::from_value(json).unwrap();
    let view = legatum_core::player_view(&state, PlayerId(2));
    for enabled in [false, true] {
        let air = air_orders_with_policy(&view, true, true, &[], 0, enabled);
        assert!(
            legatum_core::validate_orders(&state, PlayerId(2), &air).is_empty(),
            "{air:?}"
        );
        let mut all: std::collections::BTreeMap<PlayerId, Vec<Order>> =
            load(include_bytes!("../fixtures/japan-1026/T214.orders.json.gz"));
        all.get_mut(&PlayerId(2))
            .unwrap()
            .retain(|o| !matches!(o, Order::AirStrike { .. } | Order::Rebase { .. }));
        all.get_mut(&PlayerId(2)).unwrap().extend(air);
        let (next, _) = legatum_core::resolve_turn(&state, &all);
        assert_eq!(next.cities[&CityId(18)].owner, PlayerId(0));
        assert_eq!(
            next.aircraft.iter().any(|a| a.id == UnitId(221)),
            enabled,
            "only the rebased Bomber survives"
        );
    }
}

#[test]
fn athens_197_uses_its_actual_bomber_against_germany() {
    let mut json: serde_json::Value =
        load(include_bytes!("../fixtures/japan-1026/T197.state.json.gz"));
    json["setup"]["seed"] = serde_json::json!(3_811_224_394_289_556u64);
    let state: legatum_core::GameState = serde_json::from_value(json).unwrap();
    let view = legatum_core::player_view(&state, PlayerId(4));
    assert!(view.research.known.contains(&TechId::Flight));
    let orders = air_orders_with_policy(&view, true, true, &[], 100, true);
    // Every aircraft sees 4 tiles around its base (RULES_VERSION 36), so it picks among more targets than
    // the (6, 22) it struck when only a ground unit's sight lit the map.
    assert!(orders.iter().any(|o| matches!(
        o,
        Order::AirStrike {
            plane: UnitId(195),
            ..
        }
    )));
    assert!(legatum_core::validate_orders(&state, PlayerId(4), &orders).is_empty());
    let (_, events) = legatum_core::resolve_turn(
        &state,
        &std::collections::BTreeMap::from([(PlayerId(4), orders)]),
    );
    assert!(events.iter().any(|e| matches!(e, legatum_core::Event::Battle { attacker, damage_to_defender, .. } if attacker.unit() == Some(UnitId(195)) && *damage_to_defender > 100)));
}

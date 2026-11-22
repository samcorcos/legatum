use crate::game::spawn_unit;
use crate::rules::cities::tests::{P0, P1, at, found, test_game};
use crate::rules::cities::{check_found_city, found_city};
use crate::{BuildingKind, Faction, GameState, Order, ProductionItem, TechId, UnitKind, data};

fn rome() -> GameState {
    let mut state = test_game(&[(Faction::Rome, 0), (Faction::Athens, 1)]);
    state.player_mut(P0).gold = 1_000;
    state
}

fn promoted(state: &mut GameState, kind: UnitKind, pos: crate::Hex) -> crate::UnitId {
    let id = spawn_unit(state, P0, kind, pos);
    state.units.get_mut(&id).unwrap().xp = crate::rules::promotion::threshold(1).unwrap();
    id
}

#[test]
fn roman_veterans_found_without_gold_or_price_escalation() {
    let mut state = rome();
    assert!(!data::available_to_faction(
        Faction::Rome,
        ProductionItem::Unit {
            unit: UnitKind::Settler
        }
    ));
    let capital = found(&mut state, P0, at(5, 5));
    state.cities.get_mut(&capital).unwrap().production_stored = 49;
    assert_eq!(
        state.player(P0).settlers_trained,
        0,
        "founding a capital does not count as training a Settler"
    );
    state.player_mut(P0).gold = 0;
    let id = promoted(&mut state, UnitKind::Spearman, at(11, 5));
    let city = found_city(&mut state, id, &mut Vec::new()).unwrap();
    assert_eq!(state.cities[&city].population, 1);
    assert!(!state.units.contains_key(&id));
    assert_eq!(state.player(P0).gold, 0);
    assert_eq!(
        state.cities[&capital].production_stored, 49,
        "veteran founding does not spend a city's banked Production"
    );
    assert_eq!(state.player(P0).settlers_trained, 0);
    let legionary = promoted(&mut state, UnitKind::Phalangite, at(17, 5));
    let second_city = found_city(&mut state, legionary, &mut Vec::new()).unwrap();
    assert_eq!(state.cities[&second_city].population, 2);
    assert_eq!(state.player(P0).gold, 0);
    assert_eq!(state.player(P0).settlers_trained, 0);
    let restored: GameState =
        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert_eq!(restored.player(P0).faction, Faction::Rome);
    assert_eq!(restored.player(P0).settlers_trained, 0);
    assert_eq!(restored.cities[&second_city].population, 2);
}

#[test]
fn failed_foundation_spends_nothing_and_captured_settlers_still_work() {
    let mut state = rome();
    found(&mut state, P0, at(5, 5));
    let veteran = promoted(&mut state, UnitKind::Spearman, at(11, 5));
    state.player_mut(P0).gold = 0;
    state.units.get_mut(&veteran).unwrap().hp = 99;
    assert!(
        check_found_city(&state, veteran)
            .unwrap_err()
            .contains("full health")
    );
    state.units.get_mut(&veteran).unwrap().hp = 100;
    state.units.get_mut(&veteran).unwrap().xp = 0;
    assert!(
        check_found_city(&state, veteran)
            .unwrap_err()
            .contains("promotion")
    );
    state.units.get_mut(&veteran).unwrap().xp = crate::rules::promotion::threshold(1).unwrap();
    state.units.get_mut(&veteran).unwrap().pos = at(6, 5);
    assert!(
        check_found_city(&state, veteran)
            .unwrap_err()
            .contains("Too close")
    );
    assert!(state.units.contains_key(&veteran));
    assert_eq!(state.player(P0).gold, 0);
    assert_eq!(state.player(P0).settlers_trained, 0);
    let captured = spawn_unit(&mut state, P0, UnitKind::Settler, at(12, 5));
    found_city(&mut state, captured, &mut Vec::new()).unwrap();
    assert_eq!(state.player(P0).gold, 0);
    assert_eq!(state.player(P0).settlers_trained, 0);
}

#[test]
fn castrum_restores_fifty_hp_on_one_melee_upgrade_order() {
    let mut state = rome();
    let city = found(&mut state, P0, at(5, 5));
    state
        .cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::Walls);
    state.player_mut(P0).techs.extend([
        TechId::Mathematics,
        TechId::BronzeWorking,
        TechId::IronWorking,
        TechId::Feudalism,
    ]);
    state.map.get_mut(at(5, 5).neighbors()[0]).unwrap().resource = Some(crate::Resource::Iron);
    let id = spawn_unit(&mut state, P0, UnitKind::Spearman, at(5, 5));
    state.units.get_mut(&id).unwrap().hp = 40;
    assert!(crate::rules::units::check_upgrade(&state, P0, id, state.player(P0).gold).is_ok());
    let mut no_castrum = state.clone();
    no_castrum
        .cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .remove(&BuildingKind::Walls);
    let order = [Order::UpgradeUnit { unit: id }];
    let mut events = Vec::new();
    crate::rules::units::apply_unit_orders(
        &mut state,
        &std::collections::BTreeMap::from([(P0, order.as_slice())]),
        &mut events,
    );
    assert_eq!(state.units[&id].kind, UnitKind::Pikeman, "{events:?}");
    assert_eq!(state.units[&id].hp, 90);
    let mut capped = state.clone();
    capped.units.get_mut(&id).unwrap().kind = UnitKind::Spearman;
    capped.units.get_mut(&id).unwrap().hp = 80;
    crate::rules::units::apply_unit_orders(
        &mut capped,
        &std::collections::BTreeMap::from([(P0, order.as_slice())]),
        &mut Vec::new(),
    );
    assert_eq!(
        capped.units[&id].hp, 100,
        "a single upgrade heals at most to full health"
    );
    crate::rules::units::apply_unit_orders(
        &mut no_castrum,
        &std::collections::BTreeMap::from([(P0, order.as_slice())]),
        &mut Vec::new(),
    );
    assert_eq!(
        no_castrum.units[&id].hp, 40,
        "ordinary Walls do not heal an upgrade"
    );
}

#[test]
fn simultaneous_roman_founders_in_conflicting_sites_both_keep_their_units_and_gold() {
    let mut state = test_game(&[(Faction::Rome, 0), (Faction::Rome, 1)]);
    state.player_mut(P0).gold = 200;
    state.player_mut(P1).gold = 200;
    let a = promoted(&mut state, UnitKind::Spearman, at(9, 9));
    let b = spawn_unit(&mut state, P1, UnitKind::Spearman, at(10, 9));
    state.units.get_mut(&b).unwrap().xp = crate::rules::promotion::threshold(1).unwrap();
    let mut events = Vec::new();
    let candidates =
        crate::rules::units::found_city_candidates(&state, &[(P0, a), (P1, b)], &mut events);
    assert!(candidates.is_empty());
    assert!(state.units.contains_key(&a) && state.units.contains_key(&b));
    assert_eq!(state.player(P0).gold, 200);
    assert_eq!(state.player(P1).gold, 200);
}

#[test]
fn nonconflicting_same_turn_foundations_do_not_spend_gold() {
    let mut state = rome();
    found(&mut state, P0, at(5, 5));
    let a = promoted(&mut state, UnitKind::Spearman, at(11, 5));
    let b = promoted(&mut state, UnitKind::Spearman, at(17, 5));
    let gold = state.player(P0).gold;
    let candidates =
        crate::rules::units::found_city_candidates(&state, &[(P0, a), (P0, b)], &mut Vec::new());
    assert_eq!(candidates.len(), 2);
    found_city(&mut state, a, &mut Vec::new()).unwrap();
    found_city(&mut state, b, &mut Vec::new()).unwrap();
    assert_eq!(state.player(P0).gold, gold);
    assert_eq!(state.player(P0).settlers_trained, 0);
}

#[test]
fn founding_rechecks_health_after_the_order_and_before_healing() {
    let mut state = rome();
    found(&mut state, P0, at(5, 5));
    let veteran = promoted(&mut state, UnitKind::Spearman, at(11, 5));
    let gold = state.player(P0).gold;
    let order = [Order::FoundCity { unit: veteran }];
    let plan = crate::rules::units::apply_unit_orders(
        &mut state,
        &std::collections::BTreeMap::from([(P0, order.as_slice())]),
        &mut Vec::new(),
    );
    assert_eq!(plan.found, [(P0, veteran)]);
    state.units.get_mut(&veteran).unwrap().hp = 99; // Phase 2-4 combat left a wound.
    let mut events = Vec::new();
    assert!(
        crate::rules::units::found_city_candidates(&state, &plan.found, &mut events).is_empty()
    );
    assert!(events.iter().any(
        |e| matches!(e, crate::Event::OrderFailed { reason, .. } if reason.contains("full health"))
    ));
    assert!(state.units.contains_key(&veteran));
    assert_eq!(state.player(P0).gold, gold);
    assert_eq!(state.player(P0).settlers_trained, 0);
}

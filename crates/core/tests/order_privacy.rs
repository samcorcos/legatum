//! Rejected orders must not distinguish hidden foreign entities from nonexistent IDs.
use legatum_core::*;
use std::collections::BTreeMap;

fn game() -> GameState {
    let mut setup = GameSetup::standard(7, [Faction::India, Faction::Byzantium]);
    setup.width = 32;
    setup.height = 24;
    setup.players.truncate(2);
    for (i, player) in setup.players.iter_mut().enumerate() {
        player.team = TeamId(i as u8);
        player.is_human = true;
    }
    new_game(&setup)
}

fn assert_private_errors(state: &GameState, absent: Order, foreign: Order, message: &str) {
    let errors = |order| validate_orders(state, PlayerId(0), &[order]);
    let absent_errors = errors(absent.clone());
    assert_eq!(absent_errors.len(), 1);
    assert_eq!(absent_errors[0].message, message);
    assert_eq!(
        serde_json::to_vec(&absent_errors).unwrap(),
        serde_json::to_vec(&errors(foreign.clone())).unwrap(),
        "validation leaked entity existence for {foreign:?}"
    );
    let failures = |order| {
        let (_, events) = resolve_turn(state, &BTreeMap::from([(PlayerId(0), vec![order])]));
        events
            .into_iter()
            .filter(|event| matches!(event, Event::OrderFailed { .. }))
            .collect::<Vec<_>>()
    };
    let absent_failures = failures(absent);
    assert_eq!(
        absent_failures,
        vec![Event::OrderFailed {
            player: PlayerId(0),
            reason: message.into()
        }]
    );
    assert_eq!(
        serde_json::to_vec(&absent_failures).unwrap(),
        serde_json::to_vec(&failures(foreign)).unwrap(),
        "resolution leaked entity existence"
    );
}

#[test]
fn absent_and_hidden_foreign_units_have_byte_identical_errors() {
    let state = game();
    let view = player_view(&state, PlayerId(0));
    let enemy = state
        .units
        .values()
        .find(|u| u.owner == PlayerId(1))
        .unwrap()
        .id;
    assert!(view.units.iter().all(|u| u.id != enemy));
    let absent = UnitId(u32::MAX);
    assert!(!state.units.contains_key(&absent));
    let orders = |unit| {
        [
            Order::MoveUnit {
                unit,
                to: Hex::new(0, 0),
            },
            Order::MoveUnitPath {
                unit,
                to: Hex::new(0, 0),
                path: vec![Hex::new(0, 0)],
                stop_on_block: false,
            },
            Order::SkipUnit { unit },
            Order::FoundCity { unit },
            Order::DisbandUnit { unit },
            Order::RangedAttack {
                unit,
                target: Hex::new(0, 0),
            },
            Order::UpgradeUnit { unit },
        ]
    };
    for (absent, foreign) in orders(absent).into_iter().zip(orders(enemy)) {
        assert_private_errors(&state, absent, foreign, "No such unit of yours");
    }
}

#[test]
fn absent_and_hidden_foreign_cities_have_byte_identical_errors() {
    let state = game();
    let orders = state
        .units
        .values()
        .filter(|u| u.kind == UnitKind::Settler)
        .map(|u| (u.owner, vec![Order::FoundCity { unit: u.id }]))
        .collect();
    let (state, _) = resolve_turn(&state, &orders);
    assert_eq!(state.cities.len(), 2);
    let view = player_view(&state, PlayerId(0));
    let enemy = state
        .cities
        .values()
        .find(|c| c.owner == PlayerId(1))
        .unwrap()
        .id;
    assert!(view.cities.iter().all(|c| c.id != enemy));
    let absent = CityId(u32::MAX);
    assert!(!state.cities.contains_key(&absent));
    let orders = |city| {
        [
            Order::SetProduction {
                city,
                item: ProductionItem::Unit {
                    unit: UnitKind::Scout,
                },
            },
            Order::BuyItem {
                city,
                item: ProductionItem::Unit {
                    unit: UnitKind::Soldier,
                },
            },
        ]
    };
    for (absent, foreign) in orders(absent).into_iter().zip(orders(enemy)) {
        assert_private_errors(&state, absent, foreign, "No such city of yours");
    }
}

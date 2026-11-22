//! Public territory and fogged city identity prevent founding too close to a rival city.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::settlement::settlement_scores;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn scenario() -> (GameState, UnitId, Hex) {
    let mut setup = GameSetup::all_ai(11);
    setup.width = 24;
    setup.height = 16;
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Switzerland;
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    for (owner, pos) in [(ME, Hex::new(2, 6)), (ENEMY, Hex::new(11, 6))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    let site = Hex::new(8, 6);
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, site);
    state.units.get_mut(&settler).unwrap().goto = Some(site);
    for i in 0..state.map.len() {
        state.players[0].explored.set(i);
    }
    state.turn = 20;
    update_explored(&mut state);
    let view = player_view(&state, ME);
    assert!(view.cities.iter().any(|c| c.owner == ENEMY && !c.visible));
    assert!(
        view.tiles
            .iter()
            .any(|t| t.visible && t.owner == Some(ENEMY))
    );
    (state, settler, site)
}

fn relocation(state: &GameState, settler: UnitId, rejected: Hex) -> Vec<Order> {
    let view = player_view(state, ME);
    let unit = view.units.iter().find(|u| u.id == settler).unwrap();
    assert!(
        settlement_scores(&view, unit, &[])
            .iter()
            .all(|s| s.pos != rejected)
    );
    let orders: Vec<_> = ScriptedBrain::new(view.faction_of(ME), state.setup.seed, ME)
        .orders(&view)
        .into_iter()
        .filter(|o| legatum_core::rules::units::order_unit(o) == Some(settler))
        .collect();
    assert!(
        matches!(orders.as_slice(), [Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }] if *to != rejected),
        "{orders:?}"
    );
    assert!(validate_orders(state, ME, &orders).is_empty());
    orders
}

#[test]
fn arriving_settler_and_player_advice_reject_border_of_fogged_city() {
    let (state, settler, rejected) = scenario();
    let orders = relocation(&state, settler, rejected);
    let (next, events) = resolve_turn(&state, &BTreeMap::from([(ME, orders)]));
    assert_ne!(next.units[&settler].pos, rejected);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { player: ME, .. }))
    );
}

#[test]
fn failed_founding_relocates_next_turn_even_after_save_load() {
    let (state, settler, rejected) = scenario();
    let (next, events) = resolve_turn(
        &state,
        &BTreeMap::from([(ME, vec![Order::FoundCity { unit: settler }])]),
    );
    assert!(events.iter().any(
        |e| matches!(e, Event::OrderFailed { player: ME, reason } if reason.contains("Too close"))
    ));
    assert!(next.players[0].failed_city_sites.contains(&rejected));
    let reloaded: GameState = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
    // The failed site remains excluded even if border and city evidence later vanish
    // from a synthetic view (for example, after its owner razes the rival city).
    let mut without_borders = player_view(&reloaded, ME);
    for tile in &mut without_borders.tiles {
        tile.owner = None;
    }
    without_borders.cities.retain(|c| c.owner == ME);
    assert!(
        !legatum_core::settlement::border_city_risks(&without_borders)
            [without_borders.index(rejected).unwrap()]
    );
    assert!(!legatum_core::settlement::known_legal_site(
        &without_borders,
        rejected
    ));
    let orders = relocation(&reloaded, settler, rejected);
    let (moved, events) = resolve_turn(&reloaded, &BTreeMap::from([(ME, orders)]));
    assert_ne!(moved.units[&settler].pos, rejected);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { player: ME, .. }))
    );
}

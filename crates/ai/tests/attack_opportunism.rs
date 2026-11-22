//! The user's rush scenarios (playtest-6.2): an AI attacker that waited, and one that took a
//! city and walked away from it. Provenance in fixtures/attack-opportunism.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::BTreeMap;

fn load(gz: &[u8]) -> GameState {
    serde_json::from_reader(flate2::read::GzDecoder::new(gz)).unwrap()
}

fn brain_orders(state: &GameState, p: PlayerId) -> Vec<Order> {
    brain_orders_with(state, p, |_| {})
}

fn brain_orders_with(
    state: &GameState,
    p: PlayerId,
    tune: impl FnOnce(&mut ScriptedBrain),
) -> Vec<Order> {
    let view = player_view(state, p);
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, p.0 as u64]).next_u64();
    let difficulty = state.setup.players[p.0 as usize].difficulty;
    let mut brain = ScriptedBrain::with_difficulty(view.faction_of(p), seed, p, difficulty);
    tune(&mut brain);
    brain.orders(&view)
}

/// Every living seat plays the scripted AI at its own difficulty, as `Session` does.
fn all_ai_orders(state: &GameState) -> BTreeMap<PlayerId, Vec<Order>> {
    state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| (p.id, brain_orders(state, p.id)))
        .collect()
}

fn unit_order(orders: &[Order], unit: UnitId) -> Option<&Order> {
    orders
        .iter()
        .find(|o| legatum_core::rules::units::order_unit(o) == Some(unit))
}

// ---- The Last Bridge (rush-offense) ----

const SPARTA: PlayerId = PlayerId(5);
const MONGOLS_TEAM: TeamId = TeamId(4);
const LAST_BRIDGE: Hex = Hex::new(21, 16);
const STRIKE_FORCE: [UnitId; 4] = [UnitId(39), UnitId(40), UnitId(41), UnitId(42)];

fn last_bridge_start() -> GameState {
    let mut state = load(include_bytes!("fixtures/rush-offense-t22.json.gz"));
    assert_eq!(state.turn, 22);
    // The run was the human's: as in the scenario baselines, the seat is the Hard AI here.
    state.players[SPARTA.0 as usize].is_human = false;
    state.setup.players[SPARTA.0 as usize].difficulty = AiDifficulty::Hard;
    assert!(!state.at_war(SPARTA, PlayerId(4)));
    let city = state
        .cities
        .values()
        .find(|c| c.pos == LAST_BRIDGE)
        .unwrap();
    assert_eq!(city.name, "Last Bridge");
    assert!(!legatum_core::data::has_building(
        &city.buildings,
        BuildingKind::Walls
    ));
    state
}

#[test]
fn sparta_next_to_an_unwalled_last_city_declares_and_marches_on_turn_22() {
    let state = last_bridge_start();
    let orders = brain_orders(&state, SPARTA);
    assert!(
        orders.contains(&Order::DeclareWar {
            against: MONGOLS_TEAM,
            breach: false
        }),
        "the user declared on turn 22; the AI fortified for six turns: {orders:?}"
    );
    let marching = STRIKE_FORCE
        .iter()
        .filter(|u| match unit_order(&orders, **u) {
            Some(Order::MoveUnit { to, .. }) => {
                to.distance(LAST_BRIDGE) < state.units[u].pos.distance(LAST_BRIDGE)
            }
            Some(Order::RangedAttack { target, .. }) => *target == LAST_BRIDGE,
            _ => false,
        })
        .count();
    assert!(
        marching >= 3,
        "the strike force moves in the same turn: {orders:?}"
    );
    assert!(
        validate_orders(&state, SPARTA, &orders).is_empty(),
        "{:?}",
        validate_orders(&state, SPARTA, &orders)
    );
}

#[test]
fn sparta_captures_last_bridge_as_fast_as_the_user() {
    let mut state = last_bridge_start();
    let mut captured = None;
    while state.turn < 34 && captured.is_none() {
        let orders = all_ai_orders(&state);
        let (next, events) = resolve_turn(&state, &orders);
        if events
            .iter()
            .any(|e| matches!(e, Event::CityCaptured { to, .. } if *to == SPARTA))
        {
            captured = Some(state.turn);
        }
        state = next;
    }
    // The user captured on turn 26; 50 Hard baselines on the old AI captured around turn 33.
    let turn = captured.expect("Last Bridge falls inside the window");
    assert!(turn <= 27, "captured on turn {turn}");
}

// ---- Glass at the Gate (rush-defense), seen from the Mongols attacker ----

const MONGOLS: PlayerId = PlayerId(2);
const INDIA: PlayerId = PlayerId(6);
const HYMNAL: Hex = Hex::new(23, 15);
const KEYSTONE_WRECK: Hex = Hex::new(16, 24);
const CAPTOR: UnitId = UnitId(25);

#[derive(serde::Deserialize)]
struct ArchivedTurn {
    turn: u32,
    orders: BTreeMap<PlayerId, Vec<Order>>,
}

fn archived_turns() -> Vec<ArchivedTurn> {
    serde_json::from_str(include_str!("fixtures/rush-defense-t21-t29-orders.json")).unwrap()
}

/// The archived pre-T24 state: Mongols took Hymnal on T23. The fixture is the archive's
/// T21 state with T21-T23 replayed on the archive's engine (every hash matched); routes
/// around friends (2026-10-03) changed how those orders resolve, so they aren't replayed here.
fn after_the_capture() -> GameState {
    let state = load(include_bytes!("fixtures/rush-defense-t24.json.gz"));
    assert_eq!(state.turn, 24);
    let hymnal = state.cities.values().find(|c| c.pos == HYMNAL).unwrap();
    assert_eq!(hymnal.owner, MONGOLS);
    assert_eq!(state.units[&CAPTOR].pos, HYMNAL);
    assert!(player_view(&state, MONGOLS).economy.bankruptcy.is_some());
    state
}

#[test]
fn the_captor_holds_the_city_instead_of_being_disbanded() {
    let state = after_the_capture();
    let orders = brain_orders(&state, MONGOLS);
    // Archived: DisbandUnit for u25 (the captor in Hymnal) and both Archers, to cover a
    // 3-Gold deficit. The deficit is structural (Gold 0, -3 a turn), so leaving it to the
    // engine would cost the costliest unit this turn and again every turn after
    // (campaign funding, 2026-10-04). The AI covers it itself with its least useful units,
    // and never with the captor holding the new city.
    let mut orders_map = BTreeMap::new();
    orders_map.insert(MONGOLS, orders.clone());
    let (_, events) = resolve_turn(&state, &orders_map);
    assert!(
        !events.iter().any(|e| matches!(e,
            Event::UnitDisbanded { owner, reason, .. }
                if *owner == MONGOLS && reason.contains("Treasury ran dry"))),
        "{orders:?}"
    );
    assert!(
        !matches!(
            unit_order(&orders, CAPTOR),
            Some(Order::DisbandUnit { .. } | Order::MoveUnit { .. })
        ),
        "{orders:?}"
    );
    // Archived: u9 and u43 marched to stage against Keystone Wreck, a neutral city.
    for o in &orders {
        if let Order::MoveUnit { to, .. } = o {
            assert!(
                to.distance(KEYSTONE_WRECK) > 4,
                "staging against a neutral while the front is open: {o:?}"
            );
        }
    }
}

#[test]
fn mongols_stops_its_settler_build_while_hymnal_is_under_siege() {
    let state = after_the_capture();
    let waystation = state
        .cities
        .values()
        .find(|c| c.name == "Waystation")
        .unwrap();
    assert_eq!(
        waystation.producing,
        Some(ProductionItem::Unit {
            unit: UnitKind::Settler
        })
    );
    let orders = brain_orders(&state, MONGOLS);
    assert!(orders.iter().any(|o| matches!(o,
        Order::SetProduction { city, item }
            if *city == waystation.id && *item != ProductionItem::Unit { unit: UnitKind::Settler }
    )), "Waystation keeps its Settler while the fresh capture is exposed: {orders:?}");
}

#[test]
fn mongols_keep_the_population_one_capture_before_the_india_counterattacks_with_raze_value() {
    // Before the full Power window, decadence cannot reduce this city's future output. The
    // value decision keeps the T24 capture even though India may counterattack later.
    let state = after_the_capture();
    let orders = brain_orders(&state, MONGOLS);
    let hymnal = state.cities.values().find(|c| c.pos == HYMNAL).unwrap().id;
    assert!(
        !orders.contains(&Order::Raze { city: hymnal }),
        "{orders:?}"
    );
}

#[test]
fn mongols_still_holds_hymnal_when_the_india_counterattacks() {
    // Counterfactual: the user's own orders for T24-T29 against the current Mongols AI.
    // Everyone else replays the archive. In the archive the empty city fell on T29.
    // About keeping and defending the city, so gene `raze_value` (default 1 since 2026-10-31,
    // which now keeps this fresh population-1 capture on T24; see the previous test) is pinned to 0.
    let mut state = after_the_capture();
    for t in archived_turns().iter().skip(3) {
        assert_eq!(t.turn, state.turn);
        let mut orders = t.orders.clone();
        orders.insert(
            MONGOLS,
            brain_orders_with(&state, MONGOLS, |b| b.decisions.raze_value = 0),
        );
        state = resolve_turn(&state, &orders).0;
    }
    assert_eq!(state.turn, 30);
    let hymnal = state.cities.values().find(|c| c.pos == HYMNAL).unwrap();
    assert_eq!(
        hymnal.owner, MONGOLS,
        "Hymnal held through the counterattack"
    );
    assert!(state.at_war(MONGOLS, INDIA));
}

#[test]
fn mongols_still_holds_hymnal_with_capture_hold_and_strike_kill() {
    // The same counterfactual with genes `capture_hold` and `strike_kill` on: a fresh capture
    // waits for the attack passes only while no visible attack can retake it this turn, so the
    // India counterattack still finds it held.
    let mut state = after_the_capture();
    for t in archived_turns().iter().skip(3) {
        let mut orders = t.orders.clone();
        orders.insert(
            MONGOLS,
            brain_orders_with(&state, MONGOLS, |b| {
                b.decisions.raze_value = 0;
                b.decisions.capture_hold = 1;
                b.decisions.strike_kill = 1;
            }),
        );
        state = resolve_turn(&state, &orders).0;
    }
    let hymnal = state.cities.values().find(|c| c.pos == HYMNAL).unwrap();
    assert_eq!(
        hymnal.owner, MONGOLS,
        "Hymnal held through the counterattack"
    );
}

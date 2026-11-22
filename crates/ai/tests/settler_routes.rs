//! Real planning and resolution; archive provenance is in fixtures/settler-cleared-goto.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rng::{Rng, purpose};
use legatum_core::rules::cities::found_city;
use legatum_core::rules::units::order_unit;
use legatum_core::state::Map;
use legatum_core::visibility::{reveal_map, update_explored};
use legatum_core::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

type Orders = BTreeMap<PlayerId, Vec<Order>>;

#[derive(Deserialize)]
struct Frame {
    turn: u32,
    orders: Orders,
}

#[derive(Deserialize)]
struct Case {
    player: PlayerId,
    state: GameState,
    turns: Vec<Frame>,
}

fn cases() -> Vec<Case> {
    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }
    let fixture: Fixture = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/settler-cleared-goto.json.gz")[..],
    ))
    .unwrap();
    // Pin the decoded current-state hashes for these three recorded starts.
    let repinned = [
        "9944627897978336795",
        "10636368559488355015",
        "5302166457566693392",
    ];
    assert_eq!(fixture.cases.len(), repinned.len());
    for (case, pinned) in fixture.cases.iter().zip(repinned) {
        assert_eq!(case.state.hash().to_string(), pinned);
    }
    fixture.cases
}

fn brain(state: &GameState, player: PlayerId) -> ScriptedBrain {
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, player.0 as u64]).next_u64();
    ScriptedBrain::with_difficulty(
        state.player(player).faction,
        seed,
        player,
        AiDifficulty::Normal,
    )
}

/// `brain` with the Settler-destination genes (`border_explained`, `fallback_risk`, default 1 since
/// 2026-10-31) off: the archived state is about the repeated route circuit, and with them on the
/// Settler founds its city on T150 instead of walking.
fn route_brain(state: &GameState, player: PlayerId) -> ScriptedBrain {
    let mut brain = brain(state, player);
    brain.decisions.border_explained = 0;
    brain.decisions.fallback_risk = 0;
    brain
}

fn blocked(events: &[Event], settler: UnitId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(e, Event::MoveBlocked { unit, reason, .. }
        if *unit == settler && reason == "no route to the destination")
        })
        .count()
}

#[test]
fn recorded_destination_only_orders_still_reproduce_the_cleared_goto_stall() {
    let case = cases()
        .into_iter()
        .find(|c| c.player == PlayerId(4))
        .unwrap();
    let mut state = case.state;
    // The archived Mongols route is now lawful at Peace. Retain this regression
    // for ordinary closed-border settlers without rewriting historical orders.
    state.player_mut(case.player).faction = Faction::Switzerland;
    let settler = UnitId(47);
    for frame in case.turns {
        assert_eq!(frame.turn, state.turn);
        assert!(frame.orders[&case.player].contains(&Order::MoveUnit {
            unit: settler,
            to: Hex::new(8, 12),
        }));
        let (next, events) = resolve_turn(&state, &frame.orders);
        assert_eq!(blocked(&events, settler), 1);
        let unit = player_view(&next, case.player)
            .units
            .into_iter()
            .find(|u| u.id == settler)
            .unwrap();
        assert_eq!(unit.pos, Hex::new(16, 9));
        assert_eq!(unit.goto, None);
        assert_eq!(unit.moves_left, unit.max_moves);
        state = next;
    }
    assert_eq!(state.turn, 129);
}

#[test]
fn archived_settler_makes_legal_progress_through_t128_even_with_a_fresh_brain_every_turn() {
    let case = cases()
        .into_iter()
        .find(|c| c.player == PlayerId(4))
        .unwrap();
    let mut state = case.state;
    let settler = UnitId(47);
    let start = state.units[&settler].pos;
    let mut persistent = brain(&state, case.player);
    let mut positions = BTreeSet::from([start]);
    let mut failures = 0;
    for frame in case.turns {
        let loaded: GameState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        let generated = persistent.orders(&player_view(&state, case.player));
        assert_eq!(
            generated,
            brain(&loaded, case.player).orders(&player_view(&loaded, case.player))
        );
        assert!(validate_orders(&state, case.player, &generated).is_empty());
        let mut orders = frame.orders;
        orders.insert(case.player, generated);
        let (next, events) = resolve_turn(&state, &orders);
        failures += blocked(&events, settler);
        if let Some(unit) = next.units.get(&settler) {
            assert_eq!(unit.owner, case.player);
            positions.insert(unit.pos);
        }
        if frame.turn == 118 {
            assert!(
                positions.len() > 1,
                "must leave the origin within three attempts"
            );
        }
        state = next;
    }
    assert!(failures <= 2, "repeated {failures} route failures");
    assert!(
        positions.len() >= 3,
        "no useful local progress: {positions:?}"
    );
    assert_eq!(state.turn, 129);
}

#[test]
fn archived_detour_avoids_repeating_a_three_tile_loop() {
    // Historical seed 990146, master 4827ca7, pre-order T150. Five recorded turns
    // reproduce U72's 20,11 -> 19,11 -> 18,11 -> 20,11 circuit.
    let saved: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/settler-detour-loop.json.gz")[..],
    ))
    .unwrap();
    let case: Case = serde_json::from_value(saved).unwrap();
    // The fixture was recut under the current state shape.
    let settler = UnitId(72);
    // The archived goal was (2, 12). Current site scoring and public fogged borders can
    // change the destination; the regression is the repeated three-tile circuit.
    // Preserve the historical submitted circuit. Re-resolving its other players'
    // old orders under V9 prices/yields no longer reproduces every old position.
    let historical_targets: Vec<_> = case
        .turns
        .iter()
        .take(3)
        .flat_map(|frame| {
            frame.orders[&case.player]
                .iter()
                .filter_map(|order| match order {
                    Order::MoveUnit { unit, to } if *unit == settler => Some(*to),
                    _ => None,
                })
        })
        .collect();
    assert_eq!(
        historical_targets,
        [
            Hex::new(19, 11),
            Hex::new(18, 11),
            case.state.units[&settler].pos
        ]
    );

    let mut state = case.state;
    let mut persistent = route_brain(&state, case.player);
    let mut visited = BTreeSet::from([state.units[&settler].pos]);
    for frame in case.turns {
        let loaded: GameState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        // A save round trip changes nothing: a fresh brain orders the same from the state and
        // from its reloaded copy. The persistent brain's memory may legitimately differ from a
        // fresh one (under World Ages, rules 50, it picks the Pyramids on T153 where a fresh brain picks Walls), so
        // the route is driven by the persistent brain, as in a real game.
        assert_eq!(
            route_brain(&state, case.player).orders(&player_view(&state, case.player)),
            route_brain(&loaded, case.player).orders(&player_view(&loaded, case.player))
        );
        let generated = persistent.orders(&player_view(&state, case.player));
        assert!(validate_orders(&state, case.player, &generated).is_empty());
        assert!(
            generated.iter().any(|o| matches!(o,
                Order::MoveUnitPath { unit, .. } | Order::MoveUnit { unit, .. } | Order::SkipUnit { unit }
                    if *unit == settler
            )),
            "T{}: {generated:?}",
            frame.turn
        );
        let previous = state.units[&settler].pos;
        let mut orders = frame.orders;
        orders.insert(case.player, generated);
        state = resolve_turn(&loaded, &orders).0;
        // V9 can win this historical late-game state earlier. Keep the five-turn
        // route-only scenario running to check for a repeated movement tile.
        state.finished = false;
        state.winner = None;
        let current = state.units[&settler].pos;
        assert!(
            current == previous || visited.insert(current),
            "detour repeated a tile"
        );
    }
}

const ME: PlayerId = PlayerId(0);
const OTHER: PlayerId = PlayerId(1);
const GOAL: Hex = Hex::new(11, 6);
const FALLBACK: Hex = Hex::new(5, 11);
const CHOKE: Hex = Hex::new(8, 6);

fn corridor() -> (GameState, UnitId) {
    let mut setup = GameSetup::all_ai(7);
    // Scripted route scenarios: no barbarian camps wandering in.
    setup.barbarians = false;
    setup.width = 24;
    setup.height = 16;
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Switzerland;
    let mut map = Map::new(setup.width, setup.height, Terrain::Mountains);
    for h in (1..=11)
        .map(|q| Hex::new(q, 6))
        .chain((6..=11).map(|r| Hex::new(5, r)))
        .chain(GOAL.within(2))
        .chain(Hex::new(17, 6).within(1))
    {
        map.get_mut(h).unwrap().terrain = Terrain::Grassland;
    }
    let mut state = blank_game(&setup, map);
    for (owner, pos) in [(ME, Hex::new(1, 6)), (OTHER, Hex::new(17, 6))] {
        let unit = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, unit, &mut Vec::new()).unwrap();
    }
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, Hex::new(5, 6));
    // Production completes with no movement remaining, as in the archived T115.
    state.units.get_mut(&settler).unwrap().moves_left = 0;
    state.turn = 20;
    // Most of this map was explored before a peaceful neighbour claimed the choke.
    reveal_map(&mut state, ME, Hex::new(5, 6), 40);
    let choke_index = state.map.index(CHOKE).unwrap();
    let swapped_index = state.map.index(Hex::new(16, 12)).unwrap();
    let mut explored = legatum_core::bitset::BitSet::new(state.map.len());
    for i in 0..state.map.len() {
        if i != choke_index && i != swapped_index {
            explored.set(i);
        }
    }
    state.players[0].explored = explored;
    state.players[0].tile_memory[choke_index] = None;
    state.players[0].tile_memory[swapped_index] = None;
    for i in 0..state.map.len() {
        let h = state.map.hex_at(i);
        if h != GOAL && h != FALLBACK {
            state.players[0].failed_city_sites.insert(h);
        }
    }
    state.map.get_mut(CHOKE).unwrap().owner = Some(OTHER);
    update_explored(&mut state);
    let view = player_view(&state, ME);
    assert!(!view.tile(CHOKE).unwrap().visible);
    assert_eq!(view.tile(CHOKE).unwrap().owner, None);
    (state, settler)
}

fn settler_orders(state: &GameState, planner: &mut ScriptedBrain, settler: UnitId) -> Vec<Order> {
    let orders: Vec<_> = planner
        .orders(&player_view(state, ME))
        .into_iter()
        .filter(|o| order_unit(o) == Some(settler))
        .collect();
    assert!(validate_orders(state, ME, &orders).is_empty(), "{orders:?}");
    orders
}

#[test]
fn blocked_safe_detour_stops_without_losing_its_destination() {
    let ally = PlayerId(2);
    let mut setup = GameSetup::all_ai(7);
    // Scripted route scenarios: no barbarian camps wandering in.
    setup.barbarians = false;
    setup.width = 24;
    setup.height = 16;
    setup.players.truncate(3);
    setup.players[0].faction = Faction::Switzerland;
    setup.players[2].team = setup.players[0].team;
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, Hex::new(1, 6)), (OTHER, Hex::new(17, 6))] {
        let unit = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, unit, &mut Vec::new()).unwrap();
    }
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, Hex::new(5, 6));
    let defender = spawn_unit(&mut state, OTHER, UnitKind::Soldier, Hex::new(8, 6));
    let blocker = spawn_unit(&mut state, ally, UnitKind::Settler, Hex::new(4, 8));
    state.turn = 20;
    state.units.get_mut(&settler).unwrap().goto = Some(GOAL);
    state.units.get_mut(&settler).unwrap().moves_left = 1;
    reveal_map(&mut state, ME, Hex::new(5, 6), 40);
    for i in 0..state.map.len() {
        let h = state.map.hex_at(i);
        if h != GOAL {
            state.players[0].failed_city_sites.insert(h);
        }
    }
    // The soldier was observed before leaving sight; its known reach still matters.
    state.players[0]
        .observed_threats
        .push(legatum_core::state::ThreatObservation {
            unit: defender,
            owner: OTHER,
            kind: UnitKind::Soldier,
            pos: Hex::new(8, 6),
            seen_turn: state.turn,
        });
    update_explored(&mut state);
    let generated = settler_orders(&state, &mut brain(&state, ME), settler);
    assert!(
        matches!(generated.as_slice(), [Order::MoveUnitPath { to, path, .. }]
        if *to == GOAL && path.starts_with(&[Hex::new(5, 7), Hex::new(5, 8)])),
        "{generated:?}"
    );
    let orders = Orders::from([
        (ME, generated),
        (OTHER, vec![Order::SkipUnit { unit: defender }]),
        (
            ally,
            vec![Order::MoveUnit {
                unit: blocker,
                to: Hex::new(5, 8),
            }],
        ),
    ]);
    let (next, events) = resolve_turn(&state, &orders);
    assert_eq!(next.units[&blocker].pos, Hex::new(5, 8));
    assert_eq!(next.units[&settler].pos, Hex::new(5, 7), "{events:?}");
    assert_eq!(next.units[&settler].goto, Some(GOAL));
    assert!(next.units[&settler].pos.distance(next.units[&defender].pos) > 2);
    // The ally now stands on the route's next tile and stays. The settler would pass through
    // it, but crossing it and landing needs both moves and only one is left, so it waits on
    // the last free tile for next turn (a hop that does not fit is no block, so no new orders).
    assert!(!next.units[&settler].path.is_empty());

    // Save/load and a fresh planner must keep the policy: hold or choose a safe route.
    let loaded: GameState = serde_json::from_slice(&serde_json::to_vec(&next).unwrap()).unwrap();
    let replanned = settler_orders(&loaded, &mut brain(&loaded, ME), settler);
    assert_eq!(
        replanned,
        settler_orders(&next, &mut brain(&next, ME), settler)
    );
    assert!(
        matches!(replanned.as_slice(), [Order::SkipUnit { .. }])
            || matches!(replanned.as_slice(), [Order::MoveUnitPath { to, path, stop_on_block: true, .. }]
            if *to == GOAL && path.iter().all(|h| h.distance(Hex::new(8, 6)) > 2)),
        "{replanned:?}"
    );
    let (next, events) = resolve_turn(&loaded, &Orders::from([(ME, replanned)]));
    assert!(
        events
            .iter()
            .all(|e| !matches!(e, Event::UnitMoved { unit, to, .. }
        if *unit == settler && to.distance(Hex::new(8, 6)) <= 2))
    );
    assert!(next.units[&settler].pos.distance(next.units[&defender].pos) > 2);
}

#[test]
fn public_border_advice_founds_the_reachable_fallback_after_restarts() {
    for restart in [false, true] {
        let (mut state, settler) = corridor();
        let mut planner = brain(&state, ME);
        let first = settler_orders(&state, &mut planner, settler);
        assert_eq!(
            first,
            vec![Order::MoveUnit {
                unit: settler,
                to: FALLBACK
            }]
        );
        state = resolve_turn(&state, &Orders::from([(ME, first)])).0;
        for _ in 0..16 {
            if !state.units.contains_key(&settler) {
                break;
            }
            if restart {
                state = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
                planner = brain(&state, ME);
            }
            let orders = settler_orders(&state, &mut planner, settler);
            state = resolve_turn(&state, &Orders::from([(ME, orders)])).0;
        }
        assert!(
            state.cities_of(ME).any(|c| c.pos == FALLBACK),
            "must found the reachable fallback, not hold indefinitely; restart={restart}, unit={:?}",
            state.units.get(&settler)
        );
        assert!(!state.units.contains_key(&settler));
    }
}

#[test]
fn unexplored_choke_ownership_cannot_change_initial_or_reconstructed_decisions() {
    let (blocked_world, settler) = corridor();
    let mut open_world = blocked_world.clone();
    open_world.map.get_mut(CHOKE).unwrap().owner = None;
    // Territory totals are public: keep them equal by moving this hidden claim to
    // another unseen tile, outside either city's workable radius and our routes.
    open_world.map.get_mut(Hex::new(16, 12)).unwrap().owner = Some(OTHER);
    for moves in [0, 2] {
        let mut blocked_world = blocked_world.clone();
        let mut open_world = open_world.clone();
        for state in [&mut blocked_world, &mut open_world] {
            state.units.get_mut(&settler).unwrap().moves_left = moves;
        }
        let a = serde_json::to_value(player_view(&blocked_world, ME)).unwrap();
        let b = serde_json::to_value(player_view(&open_world, ME)).unwrap();
        for (key, value) in a.as_object().unwrap() {
            assert!(value == &b[key], "views differ in {key}");
        }
        let blocked_orders =
            settler_orders(&blocked_world, &mut brain(&blocked_world, ME), settler);
        let open_orders = settler_orders(&open_world, &mut brain(&open_world, ME), settler);
        assert_eq!(blocked_orders, open_orders);
    }
}

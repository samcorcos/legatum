//! Scout reconnaissance must use the same observed hazards before and after losing sight.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rng::{Rng, purpose};
use legatum_core::rules::cities::found_city;
use legatum_core::rules::units::order_unit;
use legatum_core::state::{Map, ThreatObservation};
use legatum_core::visibility::{THREAT_MEMORY_TURNS, update_explored};
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn reload(state: &GameState) -> GameState {
    serde_json::from_slice(&serde_json::to_vec(state).unwrap()).unwrap()
}

fn decide(state: &GameState, scout: UnitId) -> Vec<Order> {
    let me = state.units[&scout].owner;
    let view = player_view(state, me);
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, me.0 as u64]).next_u64();
    // Recreate the brain on every decision, as after loading a save.
    ScriptedBrain::with_difficulty(view.faction_of(me), seed, me, AiDifficulty::Normal)
        .orders(&view)
        .into_iter()
        .filter(|o| order_unit(o) == Some(scout))
        .collect()
}

fn step(state: &GameState, scout: UnitId) -> (GameState, Vec<Event>) {
    let loaded = reload(state);
    let orders = decide(state, scout);
    assert_eq!(orders, decide(&loaded, scout), "save changed the decision");
    let me = state.units[&scout].owner;
    assert!(validate_orders(state, me, &orders).is_empty());
    let all = BTreeMap::from([(me, orders)]);
    let next = resolve_turn(state, &all);
    assert_eq!(next, resolve_turn(&loaded, &all));
    (reload(&next.0), next.1)
}

fn assert_safe_steps(events: &[Event], scout: UnitId, hazard: Hex, reach: i32) {
    for event in events {
        if let Event::UnitMoved { unit, to, .. } = event
            && *unit == scout
        {
            assert!(to.distance(hazard) > reach, "unsafe step: {event:?}");
        }
    }
}

#[test]
fn archived_t58_scouts_do_not_reverse_retreat_after_save_load() {
    let fixture: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/scout-t58.json.gz")[..],
    ))
    .unwrap();
    let original: GameState = serde_json::from_value(fixture["state"].clone()).unwrap();
    let mut original = original;
    // This pre-LOS tape first spots Spearman 21 through a Forest tile.
    original.map.get_mut(Hex::new(23, 18)).unwrap().terrain = Terrain::Grassland;
    // Scout 33's archived retreat then leaves its sight radius; Hills would now
    // extend that radius and keep the Spearman live instead of remembered.
    original.map.get_mut(Hex::new(6, 25)).unwrap().terrain = Terrain::Plains;
    for (scout, soldier, retreat) in [
        // The German Spearman's ordinary movement changes the first retreat; terrain sight changes the second.
        (UnitId(23), UnitId(21), Hex::new(21, 18)),
        (UnitId(33), UnitId(12), Hex::new(7, 24)),
    ] {
        let mut state = original.clone();
        let me = state.units[&scout].owner;
        let hazard = state.units[&soldier].pos;
        for frame in fixture["turns"].as_array().unwrap() {
            let loaded = reload(&state);
            let view = player_view(&loaded, me);
            let orders = decide(&state, scout);
            assert_eq!(orders, decide(&loaded, scout));
            assert!(validate_orders(&state, me, &orders).is_empty());
            if state.turn == 58 {
                assert!(view.units.iter().any(|u| u.id == soldier));
                assert_eq!(
                    orders,
                    vec![Order::MoveUnit {
                        unit: scout,
                        to: retreat
                    }]
                );
            } else {
                assert!(!view.units.iter().any(|u| u.id == soldier));
                assert!(
                    view.observed_threats
                        .iter()
                        .any(|o| { o.unit == soldier && o.pos == hazard && o.seen_turn == 58 })
                );
            }
            // Replan just this scout; retain every other archived simultaneous order.
            // These seven turns contain no obsolete item-less BuyItem orders.
            let mut all: BTreeMap<PlayerId, Vec<Order>> =
                serde_json::from_value(frame["orders"].clone()).unwrap();
            let ours = all.get_mut(&me).unwrap();
            ours.retain(|o| order_unit(o) != Some(scout));
            ours.extend(orders);
            let (next, events) = resolve_turn(&state, &all);
            assert_eq!((next.clone(), events.clone()), resolve_turn(&loaded, &all));
            assert_safe_steps(&events, scout, hazard, 2);
            if !next.units.contains_key(&scout) {
                // Replaying the archived orders on current rules (growth claims land, the
                // economy shifted) can leave this empire short of Gold on the last turn, and
                // the scout is then disbanded for upkeep: that ends the counterfactual, but only
                // if it is the stated reason.
                assert!(
                    events
                        .iter()
                        .any(|e| matches!(e, Event::UnitDisbanded { unit, .. } if *unit == scout)),
                    "scout vanished without a disband"
                );
                break;
            }
            assert!(next.units[&scout].pos.distance(hazard) > 2);
            assert_eq!(next.units[&soldier].pos, hazard);
            state = reload(&next);
        }
    }
}

/// A corridor with a guarded eastern ruin/city and an optional safe northern frontier.
fn corridor(alternative: bool) -> (GameState, UnitId, UnitId) {
    let mut setup = GameSetup::all_ai(7);
    // Scripted threat scenarios: no barbarian camps wandering in.
    setup.barbarians = false;
    setup.width = 30;
    setup.height = 18;
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Switzerland;
    setup.players[1].faction = Faction::Byzantium;
    let mut map = Map::new(setup.width, setup.height, Terrain::Mountains);
    for q in 1..=13 {
        map.get_mut(Hex::new(q, 8)).unwrap().terrain = Terrain::Grassland;
    }
    if alternative {
        for r in 3..=7 {
            map.get_mut(Hex::new(4, r)).unwrap().terrain = Terrain::Grassland;
        }
    }
    map.get_mut(Hex::new(8, 8)).unwrap().ruin = true;
    let mut state = blank_game(&setup, map);
    // A reconnaissance fixture: the guarding army is hostile, so start at War.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, Hex::new(1, 8)), (ENEMY, Hex::new(9, 8))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, Hex::new(7, 8));
    let soldier = spawn_unit(&mut state, ENEMY, UnitKind::Spearman, Hex::new(9, 8));
    for i in 0..state.map.len() {
        if !alternative || state.map.hex_at(i) != Hex::new(4, 3) {
            state.player_mut(ME).explored.set(i);
        }
    }
    state.turn = 20;
    update_explored(&mut state);
    (state, scout, soldier)
}

#[test]
fn retreat_is_followed_by_progress_on_a_safe_alternative_frontier() {
    let (state, scout, soldier) = corridor(true);
    let hazard = state.units[&soldier].pos;
    let (mut state, _) = step(&state, scout);
    let unexplored = state.map.index(Hex::new(4, 3)).unwrap();
    assert!(!state.player(ME).explored.get(unexplored));
    for _ in 0..5 {
        let (next, events) = step(&state, scout);
        assert_safe_steps(&events, scout, hazard, 2);
        state = next;
    }
    assert!(
        state.player(ME).explored.get(unexplored),
        "safe exploration stalled"
    );
}

#[test]
fn trapped_scout_holds_until_normal_observation_expiry() {
    let (state, scout, soldier) = corridor(false);
    let (mut state, _) = step(&state, scout);
    let pos = state.units[&scout].pos;
    let sighting = player_view(&state, ME)
        .observed_threats
        .into_iter()
        .find(|o| o.unit == soldier)
        .unwrap();
    while state.turn < sighting.seen_turn + THREAT_MEMORY_TURNS {
        assert_eq!(decide(&state, scout), vec![Order::SkipUnit { unit: scout }]);
        state = step(&state, scout).0;
        assert_eq!(state.units[&scout].pos, pos);
    }
    assert!(
        !player_view(&state, ME)
            .observed_threats
            .iter()
            .any(|o| o.unit == soldier)
    );
    assert!(matches!(
        decide(&state, scout).as_slice(),
        [Order::MoveUnit { .. }]
    ));
}

#[test]
fn unsafe_exploration_is_not_mistaken_for_completed_exploration() {
    let (state, scout, _) = corridor(false);
    let (state, _) = step(&state, scout);
    let mut view = player_view(&state, ME);
    // The same guarded ruin, before learning of any enemy city, with upkeep to pay.
    view.cities.retain(|c| c.owner == ME);
    view.economy.upkeep = 1;
    let scout_orders = |view: &PlayerView| {
        ScriptedBrain::new(view.faction_of(ME), 7, ME)
            .orders(view)
            .into_iter()
            .filter(|o| order_unit(o) == Some(scout))
            .collect::<Vec<_>>()
    };
    assert_eq!(scout_orders(&view), vec![Order::SkipUnit { unit: scout }]);
    // Scouts are free: even with no connected frontier left, one is held, never disbanded
    // for upkeep it does not cost.
    let barrier = view.index(Hex::new(6, 8)).unwrap();
    view.tiles[barrier].terrain = Some(Terrain::Mountains);
    assert_eq!(scout_orders(&view), vec![Order::SkipUnit { unit: scout }]);
}

#[test]
fn unseen_movement_or_death_cannot_clear_avoidance_but_visible_clearance_can() {
    let (mut state, scout, soldier) = corridor(false);
    // Only walled cities fire: wall the guarded city so its ruin stays unsafe.
    state
        .cities
        .values_mut()
        .find(|c| c.owner == ENEMY)
        .unwrap()
        .buildings
        .insert(BuildingKind::Walls);
    let (state, _) = step(&state, scout);
    let orders = decide(&state, scout);
    assert_eq!(orders, vec![Order::SkipUnit { unit: scout }]);
    for died in [false, true] {
        let mut changed = state.clone();
        if died {
            changed.units.remove(&soldier);
        } else {
            changed.units.get_mut(&soldier).unwrap().pos = Hex::new(13, 8);
        }
        update_explored(&mut changed);
        let before = player_view(&state, ME);
        let after = player_view(&changed, ME);
        // Public army totals may change after a death; they do not locate or clear a sighting.
        assert_eq!(before.observed_threats, after.observed_threats);
        assert_eq!(before.units, after.units);
        assert_eq!(before.tiles, after.tiles);
        assert_eq!(orders, decide(&reload(&changed), scout));
        // A friendly observer sees the old tile empty. No private death/move signal is used.
        changed.map.get_mut(Hex::new(9, 10)).unwrap().terrain = Terrain::Grassland;
        changed.map.get_mut(Hex::new(9, 9)).unwrap().terrain = Terrain::Grassland;
        spawn_unit(&mut changed, ME, UnitKind::Settler, Hex::new(9, 10));
        update_explored(&mut changed);
        assert!(
            !player_view(&changed, ME)
                .observed_threats
                .iter()
                .any(|o| o.unit == soldier)
        );
        // The observer also reveals the city. Clearing the unit sighting does not
        // make its adjacent ruin safe: the city still fires at a visiting scout.
        assert_eq!(
            decide(&reload(&changed), scout),
            vec![Order::SkipUnit { unit: scout }]
        );
        changed
            .cities
            .values_mut()
            .find(|c| c.owner == ENEMY)
            .unwrap()
            .hp = 0;
        update_explored(&mut changed);
        assert!(matches!(
            decide(&reload(&changed), scout).as_slice(),
            [Order::MoveUnit { .. }]
        ));
    }
}

#[test]
fn distant_ruin_uses_safe_waypoints_around_live_and_remembered_threats() {
    for (kind, faction) in [
        (UnitKind::Spearman, Faction::Byzantium),
        (UnitKind::Scout, Faction::Mongols),
    ] {
        for visible in [false, true] {
            let (mut state, scout, soldier) = corridor(false);
            state.player_mut(ENEMY).faction = faction;
            for t in &mut state.map.tiles {
                t.terrain = Terrain::Grassland;
                t.ruin = false;
            }
            let hazard = Hex::new(9, 8);
            let stats = legatum_core::data::unit_stats_for(faction, kind);
            let reach = stats.moves.max(stats.range);
            let (start, target) = if reach > 3 {
                (Hex::new(4, 8), Hex::new(8, 13))
            } else {
                (Hex::new(5, 8), Hex::new(12, 9))
            };
            state.map.get_mut(target).unwrap().ruin = true;
            // The cheap ordinary route cuts the corner through the threat's reach.
            state.map.get_mut(Hex::new(4, 9)).unwrap().terrain = Terrain::Hills;
            state.units.get_mut(&scout).unwrap().pos = start;
            state.units.get_mut(&soldier).unwrap().kind = kind;
            state.player_mut(ME).observed_threats = vec![ThreatObservation {
                unit: soldier,
                owner: ENEMY,
                kind,
                pos: hazard,
                seen_turn: state.turn - 1,
            }];
            if visible {
                spawn_unit(&mut state, ME, UnitKind::Settler, Hex::new(9, 10));
            }
            // Record the distant ruin as earlier exploration, without granting live sight.
            let index = state.map.index(target).unwrap();
            state.player_mut(ME).tile_memory[index] = Some(legatum_core::state::TileMemory {
                owner: None,
                ruin: true,
                improved: false,
                city: None,
            });
            update_explored(&mut state);
            assert!(
                target.distance(hazard) > reach,
                "the destination itself is safe"
            );
            assert!(
                legatum_core::path::land_path(&state.map, state.units[&scout].pos, target)
                    .unwrap()
                    .iter()
                    .any(|h| h.distance(hazard) <= reach)
            );
            assert_eq!(
                player_view(&state, ME)
                    .units
                    .iter()
                    .any(|u| u.id == soldier),
                visible
            );
            let mut reached = false;
            let mut trace = Vec::new();
            for _ in 0..11 {
                trace.push((state.units[&scout].pos, decide(&state, scout)));
                let (next, events) = step(&state, scout);
                assert_safe_steps(&events, scout, hazard, reach);
                reached |= next.units[&scout].pos == target;
                state = next;
                if reached {
                    break;
                }
            }
            assert!(
                reached,
                "safe detour stalled: {kind:?}, visible={visible}, trace={trace:?}"
            );
        }
    }
}

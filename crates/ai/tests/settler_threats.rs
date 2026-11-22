//! A settler must remember a defender after retreat takes it out of sight.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::visibility::update_explored;
use legatum_core::*;
use std::collections::{BTreeMap, BTreeSet};

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn scenario() -> (GameState, UnitId, UnitId) {
    let mut setup = GameSetup::all_ai(7);
    // Scripted threat scenarios: no barbarian camps wandering in.
    setup.barbarians = false;
    setup.width = 24;
    setup.height = 16;
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Switzerland;
    setup.players[1].faction = Faction::Byzantium;
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    // This hazard fixture assumes War explicitly; ordinary new games begin at Peace.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, Hex::new(3, 8)), (ENEMY, Hex::new(16, 8))] {
        let unit = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        let city = found_city(&mut state, unit, &mut Vec::new()).unwrap();
        if owner == ENEMY {
            // Population is territory: a growing rival would claim ring two toward the settler
            // and put a foreign border in its sight. A city in resistance neither grows nor
            // claims, which keeps this hazard fixture's geometry fixed.
            state.cities.get_mut(&city).unwrap().resistance = 1_000;
        }
    }
    let settler = spawn_unit(&mut state, ME, UnitKind::Settler, Hex::new(7, 8));
    let defender = spawn_unit(&mut state, ENEMY, UnitKind::Soldier, Hex::new(9, 8));
    state.units.get_mut(&settler).unwrap().goto = Some(Hex::new(12, 8));
    // Known terrain is legitimate earlier exploration; current enemy units remain fogged.
    for i in 0..state.map.len() {
        state.players[0].explored.set(i);
        // Isolate the travel decision: all but two candidate sites were previously
        // rejected. The nearby site requires re-entering the defender's reach;
        // the more distant northern site has a known safe route.
        let h = state.map.hex_at(i);
        if h != Hex::new(7, 8) && h != Hex::new(3, 3) {
            state.players[0].failed_city_sites.insert(h);
        }
    }
    state.turn = 20;
    update_explored(&mut state);
    (state, settler, defender)
}

fn orders(state: &GameState, settler: UnitId) -> Vec<Order> {
    let view = player_view(state, ME);
    let generated: Vec<_> = ScriptedBrain::new(view.faction_of(ME), state.setup.seed, ME)
        .orders(&view)
        .into_iter()
        .filter(|o| legatum_core::rules::units::order_unit(o) == Some(settler))
        .collect();
    assert!(validate_orders(state, ME, &generated).is_empty());
    generated
}

#[test]
fn settler_makes_progress_after_losing_sight_of_a_stationary_defender() {
    for reload in [false, true] {
        let (mut state, settler, defender) = scenario();
        let hazard = state.units[&defender].pos;
        let mut visited = BTreeSet::new();
        let mut lost_sight = false;
        for step in 0..20 {
            if reload {
                state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            }
            let Some(unit) = state.units.get(&settler) else {
                break;
            };
            let pos = unit.pos;
            assert!(visited.insert(pos), "revisited {pos:?} on step {step}");
            let visible = player_view(&state, ME)
                .units
                .iter()
                .any(|u| u.id == defender);
            lost_sight |= !visible;
            if step > 0 {
                assert!(
                    pos.distance(hazard) > 2,
                    "re-entered remembered danger at {pos:?}"
                );
            }
            let generated = orders(&state, settler);
            state = resolve_turn(
                &state,
                &BTreeMap::from([
                    (ME, generated),
                    (ENEMY, vec![Order::SkipUnit { unit: defender }]),
                ]),
            )
            .0;
        }
        assert!(lost_sight, "fixture must lose sight after retreat");
        assert_eq!(
            state.cities_of(ME).count(),
            2,
            "must found a safe alternate site"
        );
        assert_eq!(state.units[&defender].pos, hazard);
    }
}

#[test]
fn settler_uses_safe_waypoints_when_the_only_site_needs_a_detour() {
    for reload in [false, true] {
        let (mut state, settler, defender) = scenario();
        let hazard = state.units[&defender].pos;
        let destination = Hex::new(12, 8);
        state.players[0].failed_city_sites.insert(Hex::new(7, 8));
        state.players[0].failed_city_sites.insert(Hex::new(3, 3));
        state.players[0].failed_city_sites.remove(&destination);
        let mut left_sight = false;
        for step in 0..24 {
            if reload {
                state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            }
            let Some(unit) = state.units.get(&settler) else {
                break;
            };
            if step > 0 {
                assert!(
                    unit.pos.distance(hazard) > 2,
                    "detour entered danger: {:?}",
                    unit.pos
                );
            }
            left_sight |= !player_view(&state, ME)
                .units
                .iter()
                .any(|u| u.id == defender);
            state = resolve_turn(
                &state,
                &BTreeMap::from([
                    (ME, orders(&state, settler)),
                    (ENEMY, vec![Order::SkipUnit { unit: defender }]),
                ]),
            )
            .0;
        }
        assert!(left_sight);
        assert!(
            state.cities_of(ME).any(|c| c.pos == destination),
            "did not finish safe detour"
        );
        assert_eq!(state.units[&defender].pos, hazard);
    }
}

#[test]
fn only_observed_removal_allows_the_rejected_site_again() {
    let (mut state, settler, defender) = scenario();
    let original = state.units[&settler].pos;
    // Decision-time visibility must be recorded before retreat hides it.
    state.players[0].observed_threats.clear();
    state = resolve_turn(&state, &BTreeMap::from([(ME, orders(&state, settler))])).0;
    assert!(
        !player_view(&state, ME)
            .units
            .iter()
            .any(|u| u.id == defender)
    );
    assert!(
        state.players[0]
            .observed_threats
            .iter()
            .any(|o| o.unit == defender)
    );
    let with_defender = orders(&state, settler);
    state.units.remove(&defender); // Not witnessed; this must not change the decision.
    update_explored(&mut state);
    assert_eq!(orders(&state, settler), with_defender);
    assert!(!with_defender.iter().any(|order| matches!(order,
        Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
        if *unit == settler && *to == original
    )));
    spawn_unit(&mut state, ME, UnitKind::Scout, Hex::new(10, 8));
    update_explored(&mut state); // Reconnaissance sees the former defender's empty tile.
    state = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
    assert!(
        state.players[0]
            .observed_threats
            .iter()
            .all(|o| o.unit != defender)
    );
    let returning = orders(&state, settler);
    assert!(matches!(returning.as_slice(),
        [Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }]
        if *unit == settler && *to == original
    ));
    let (returned, events) = resolve_turn(&state, &BTreeMap::from([(ME, returning)]));
    assert_eq!(returned.units[&settler].pos, original);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::OrderFailed { player: ME, .. }))
    );
}

#[test]
fn archived_windows_break_the_loop_with_fresh_brains_and_save_load() {
    use legatum_core::rng::{Rng, purpose};
    let fixture: serde_json::Value = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/settler-hazards.json.gz")[..],
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut state: GameState = serde_json::from_value(case["state"].clone()).unwrap();
        let settler: UnitId = serde_json::from_value(case["settler"].clone()).unwrap();
        let defender: UnitId = serde_json::from_value(case["defender"].clone()).unwrap();
        let player = state.units[&settler].owner;
        let original_pos = state.units[&settler].pos;
        let hazard = state.units[&defender].pos;
        let seed = Rng::derive(state.setup.seed, &[purpose::AI, player.0 as u64]).next_u64();
        let mut positions = Vec::new();
        let mut founded = false;
        let cities_before: Vec<Hex> = state.cities_of(player).map(|c| c.pos).collect();
        for frame in case["turns"].as_array().unwrap() {
            let Some(unit) = state.units.get(&settler) else {
                break;
            };
            positions.push(unit.pos);
            let decision = |state: &GameState| {
                ScriptedBrain::new(state.player(player).faction, seed, player)
                    .orders(&player_view(state, player))
            };
            let generated = decision(&state);
            let loaded = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            assert_eq!(generated, decision(&loaded));
            let generated: Vec<_> = generated
                .into_iter()
                .filter(|o| legatum_core::rules::units::order_unit(o) == Some(settler))
                .collect();
            assert!(validate_orders(&state, player, &generated).is_empty());
            founded |= generated.contains(&Order::FoundCity { unit: settler });
            // Historical BuyItem funded the active build. Translate that explicit intent
            // for this current-rules counterfactual.
            let mut raw = frame["orders"].clone();
            let mut producing: BTreeMap<_, _> = state
                .cities
                .values()
                .filter_map(|c| c.producing.map(|item| (c.id.0, item)))
                .collect();
            for list in raw.as_object_mut().unwrap().values_mut() {
                // Under current rules (growth claims land, the first Settler is full price) a city
                // may have no active build where the archive bought one: drop that purchase.
                list.as_array_mut().unwrap().retain_mut(|order| {
                    if order["type"] == "SetProduction" {
                        producing.insert(
                            order["city"].as_u64().unwrap() as u32,
                            serde_json::from_value(order["item"].clone()).unwrap(),
                        );
                    } else if order["type"] == "BuyItem" && order.get("item").is_none() {
                        let city = order["city"].as_u64().unwrap() as u32;
                        let Some(item) = producing.get(&city) else {
                            return false;
                        };
                        order["item"] = serde_json::to_value(item).unwrap();
                    }
                    true
                });
            }
            let mut all: BTreeMap<PlayerId, Vec<Order>> = serde_json::from_value(raw).unwrap();
            let ours = all.get_mut(&player).unwrap();
            ours.retain(|o| legatum_core::rules::units::order_unit(o) != Some(settler));
            ours.extend(generated);
            let next = resolve_turn(&state, &all).0;
            assert_eq!(next, resolve_turn(&loaded, &all).0);
            state = serde_json::from_str(&serde_json::to_string(&next).unwrap()).unwrap();
            assert_eq!(state.units[&defender].pos, hazard, "fixture defender moved");
        }
        // The only return permitted is the initial escape from the first encounter.
        assert!(
            positions
                .windows(4)
                .all(|p| !(p[0] == p[2] && p[1] == p[3])),
            "{positions:?}"
        );
        if settler == UnitId(48) {
            assert!(founded && !state.units.contains_key(&settler));
            // The exact site belongs to the (shared, UI-visible) settlement scoring; what this
            // test pins is a new city founded well clear of the remembered defender.
            assert!(
                state
                    .cities_of(player)
                    .any(|c| !cities_before.contains(&c.pos) && c.pos.distance(hazard) >= 4)
            );
            assert!(state.turn <= 121);
        } else {
            assert!(
                state.units[&settler].pos.distance(original_pos) >= 2,
                "{positions:?}"
            );
        }
    }
}

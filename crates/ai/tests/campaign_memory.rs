//! Real terrain, fog and observations from the campaign loop; see fixtures/campaign-t86.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(1);
const SPEARMEN: [UnitId; 2] = [UnitId(40), UnitId(44)];

fn original() -> GameState {
    let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/campaign-t86.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(state.turn, 86);
    let mut state = state;
    state.map.get_mut(Hex::new(23, 19)).unwrap().terrain = Terrain::Plains;
    let view = player_view(&state, ME);
    assert!(view.units.iter().all(|u| u.owner == ME));
    for (id, pos) in [(9, Hex::new(26, 18)), (30, Hex::new(26, 17))] {
        assert!(
            view.observed_threats
                .iter()
                .any(|o| { o.unit == UnitId(id) && o.pos == pos && o.seen_turn == 85 })
        );
    }
    // The archive was played before cities fired back; continue it under its own rules.
    state
}

fn decide(state: &GameState) -> Vec<Order> {
    let view = player_view(state, ME);
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, ME.0 as u64]).next_u64();
    ScriptedBrain::new(view.faction_of(ME), seed, ME).orders(&view)
}

fn advance(state: &GameState) -> (GameState, Vec<Event>) {
    let generated = decide(state);
    assert!(validate_orders(state, ME, &generated).is_empty());
    let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
    for u in state.units.values() {
        orders
            .entry(u.owner)
            .or_default()
            .push(Order::SkipUnit { unit: u.id });
    }
    orders.insert(ME, generated);
    resolve_turn(state, &orders)
}

fn roundtrip(state: &GameState) -> GameState {
    serde_json::from_slice(&serde_json::to_vec(state).unwrap()).unwrap()
}

#[test]
fn archived_campaign_does_not_repeat_approach_retreat_with_holding_defenders() {
    let initial = original();
    let defenders = [UnitId(9), UnitId(30)].map(|id| (id, initial.units[&id].pos));
    let mut uninterrupted = initial.clone();
    let mut loaded = initial;
    let mut positions = [Vec::new(), Vec::new()];
    let mut attacks = [Vec::new(), Vec::new()];
    for _ in 0..8 {
        loaded = roundtrip(&loaded);
        assert_eq!(decide(&uninterrupted), decide(&loaded));
        for (i, id) in SPEARMEN.iter().enumerate() {
            positions[i].push(uninterrupted.units[id].pos);
        }
        let (next, events) = advance(&uninterrupted);
        for (i, id) in SPEARMEN.iter().enumerate() {
            attacks[i].push(events.iter().any(|e| {
                matches!(e,
                    Event::Battle { attacker, damage_to_defender, .. }
                    if attacker == id && *damage_to_defender > 0
                )
            }));
        }
        uninterrupted = next;
        loaded = advance(&loaded).0;
        assert_eq!(uninterrupted, loaded);
        for (id, pos) in defenders {
            assert_eq!(uninterrupted.units[&id].pos, pos, "defender must hold");
        }
    }
    for ((id, mut path), attacks) in SPEARMEN.into_iter().zip(positions).zip(attacks) {
        path.push(uninterrupted.units[&id].pos);
        // Successful attacks can end adjacent to a surviving city and legitimately
        // require recovery. Reject the reported empty movement loop, not real combat.
        for (turn, w) in path.windows(4).enumerate() {
            let repeats = w[0] != w[1] && w[0] == w[2] && w[1] == w[3];
            assert!(
                !repeats || attacks[turn..turn + 3].iter().any(|hit| *hit),
                "{id:?} repeats approach/retreat without attacking: {path:?}"
            );
        }
    }
}

#[test]
fn unseen_defender_positions_cannot_change_campaign_orders() {
    let state = original();
    let view = player_view(&state, ME);
    let mut changed = state.clone();
    // The archived units already moved away from their observations without being seen.
    // Move them again, exclusively through fog, without changing any information P1 has.
    let destinations: Vec<_> = (0..state.map.len())
        .map(|i| state.map.hex_at(i))
        .filter(|h| {
            view.tile(*h).is_some_and(|t| !t.visible)
                && state.map.move_cost(*h).is_some()
                && state.units.values().all(|u| u.pos != *h)
        })
        .take(2)
        .collect();
    for (id, pos) in [UnitId(9), UnitId(30)].into_iter().zip(destinations) {
        changed.units.get_mut(&id).unwrap().pos = pos;
    }
    assert_eq!(
        serde_json::to_value(&view).unwrap(),
        serde_json::to_value(player_view(&changed, ME)).unwrap()
    );
    assert_eq!(decide(&state), decide(&changed));
}

#[test]
fn reconnaissance_clears_defender_memories_without_forcing_a_city_assault() {
    use legatum_core::game::spawn_unit;
    use legatum_core::visibility::update_explored;

    let mut state = original();
    let target = Hex::new(26, 18);
    // Give the player actual reconnaissance of both last-seen tiles after the defenders
    // leave. Merely deleting them behind fog must not clear the observations.
    for id in [UnitId(9), UnitId(30)] {
        state.units.remove(&id);
    }
    assert!(
        player_view(&state, ME)
            .observed_threats
            .iter()
            .any(|o| o.unit == UnitId(9))
    );
    let scout = spawn_unit(&mut state, ME, UnitKind::Scout, Hex::new(26, 19));
    update_explored(&mut state);
    let view = player_view(&state, ME);
    assert!(
        view.observed_threats
            .iter()
            .all(|o| ![UnitId(9), UnitId(30)].contains(&o.unit))
    );
    assert!(view.cities.iter().any(|c| c.pos == target && c.visible));
    // Keep reconnaissance in place while checking that current city-assault decisions remain
    // legal and stable across save/load. Clearing memory alone need not justify an attack.
    for _ in 0..6 {
        let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for u in state.units.values() {
            orders
                .entry(u.owner)
                .or_default()
                .push(Order::SkipUnit { unit: u.id });
        }
        let mut ours = decide(&state);
        assert_eq!(ours, decide(&roundtrip(&state)));
        ours.retain(|o| legatum_core::rules::units::order_unit(o) != Some(scout));
        ours.push(Order::SkipUnit { unit: scout });
        assert!(validate_orders(&state, ME, &ours).is_empty());
        orders.insert(ME, ours);
        state = resolve_turn(&roundtrip(&state), &orders).0;
    }
    assert!(SPEARMEN.iter().all(|id| state.units.contains_key(id)));
}

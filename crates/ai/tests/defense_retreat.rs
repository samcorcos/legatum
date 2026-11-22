//! Real defense/retreat loop; replay provenance is in fixtures/defense-t121.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(4);
const RANGER: UnitId = UnitId(49);
const THREAT: UnitId = UnitId(42);
const BLOCKER: UnitId = UnitId(69);
const CITY: Hex = Hex::new(13, 16);
const SUPPORT: Hex = Hex::new(12, 16);
const ENEMY_POS: Hex = Hex::new(16, 13);

fn original() -> GameState {
    let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/defense-t121.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(state.turn, 121);
    let mut state = state;
    // The archive used radius sight through this impassable detour. Keep the
    // threat visible without opening a land approach in the original case.
    for h in [Hex::new(14, 15), Hex::new(15, 14)] {
        state.map.get_mut(h).unwrap().terrain = Terrain::Ocean;
    }
    assert_eq!(state.units[&RANGER].pos, SUPPORT);
    assert_eq!(state.units[&RANGER].hp, 100);
    assert_eq!(state.units[&THREAT].pos, ENEMY_POS);
    assert!(state.cities_of(ME).any(|c| c.pos == CITY));
    state
}

fn open_approach() -> GameState {
    let mut state = original();
    // Unlike the adapted archive's impassable detour, this permits u42 to attack the
    // city within its three moves. A weak Archer must use safe support instead.
    for h in [Hex::new(14, 15), Hex::new(15, 14)] {
        state.map.get_mut(h).unwrap().terrain = Terrain::Plains;
    }
    legatum_core::visibility::update_explored(&mut state);
    state
}

fn decide(state: &GameState) -> Vec<Order> {
    let view = player_view(state, ME);
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, ME.0 as u64]).next_u64();
    ScriptedBrain::new(view.faction_of(ME), seed, ME)
        .orders(&view)
        .into_iter()
        .filter(|o| legatum_core::rules::units::order_unit(o).is_some())
        .collect()
}

fn advance(state: &GameState, blocked: bool) -> GameState {
    let mut ours = decide(state);
    if blocked {
        ours.retain(|o| legatum_core::rules::units::order_unit(o) != Some(BLOCKER));
        ours.push(Order::SkipUnit { unit: BLOCKER });
    }
    assert!(validate_orders(state, ME, &ours).is_empty());
    let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
    for u in state.units.values() {
        orders
            .entry(u.owner)
            .or_default()
            .push(Order::SkipUnit { unit: u.id });
    }
    orders.insert(ME, ours);
    let (next, events) = resolve_turn(state, &orders);
    // Other archived seats can hit old production orders: Byzantium's thin-levy cap and
    // India's retired Rice Storehouse do not concern this seat's defender movement.
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { reason, .. }
            if !reason.contains("Levies full")
                && !reason.contains("No people can build the Rice Storehouse yet"))),
        "{events:?}"
    );
    next
}

fn check_six_turns(mut state: GameState, blocked: bool) -> GameState {
    let mut path = vec![state.units[&RANGER].pos];
    for _ in 0..6 {
        let view = player_view(&state, ME);
        assert!(
            view.units
                .iter()
                .any(|u| u.id == THREAT && u.pos == ENEMY_POS)
        );
        let loaded: GameState =
            serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        assert_eq!(decide(&state), decide(&loaded));
        let next = advance(&state, blocked);
        assert_eq!(next, advance(&loaded, blocked));
        state = next;
        assert_eq!(state.units[&THREAT].pos, ENEMY_POS);
        if blocked {
            assert_eq!(state.units[&BLOCKER].pos, SUPPORT);
        }
        path.push(state.units[&RANGER].pos);
    }
    assert!(
        path.windows(4)
            .all(|w| w[0] == w[1] || w[0] != w[2] || w[1] != w[3]),
        "defense must not undo retreat repeatedly: {path:?}"
    );
    state
}

#[test]
fn archived_defender_does_not_alternate_city_and_support() {
    check_six_turns(original(), false);
}

#[test]
fn reachable_melee_threat_sends_a_ranged_defender_into_its_shielding_city() {
    // Civ 6: the healthy city absorbs every attack on its tile, so the Archer garrisons it
    // (and can shoot from safety) instead of standing exposed in support.
    let state = check_six_turns(open_approach(), false);
    assert!(state.cities_of(ME).any(|c| c.pos == CITY && c.hp > 0));
    assert_eq!(state.units[&RANGER].pos, CITY);
}

#[test]
fn blocking_the_original_escape_does_not_shift_the_same_loop_to_another_tile() {
    let mut state = original();
    state.units.get_mut(&RANGER).unwrap().pos = CITY;
    // u69 is a later reinforcement in the archive; add it early in this counterfactual.
    let mut blocker = state.units[&RANGER].clone();
    blocker.id = BLOCKER;
    blocker.kind = UnitKind::Spearman;
    blocker.pos = SUPPORT;
    state.units.insert(BLOCKER, blocker);
    state.next_unit_id = state.next_unit_id.max(BLOCKER.0);
    check_six_turns(state, true);
}

#[test]
fn a_suitable_city_defender_stays_and_fortifies() {
    let mut state = open_approach();
    let defender = state.units.get_mut(&RANGER).unwrap();
    defender.kind = UnitKind::Pikeman;
    defender.pos = CITY;
    state = check_six_turns(state, false);
    assert_eq!(state.units[&RANGER].pos, CITY);
    assert_eq!(
        state.units[&RANGER].fortify,
        legatum_core::data::FORTIFY_MAX
    );
}

#[test]
fn visibly_clearing_the_threat_allows_a_healthy_defender_to_reoccupy() {
    // A fallen city exposes its garrison: the weak Archer waits in support while a
    // healthy defender moves in once the visible threat is gone.
    let mut state = open_approach();
    state
        .cities
        .values_mut()
        .filter(|c| c.pos == CITY)
        .for_each(|c| c.hp = 0);
    state.units.get_mut(&RANGER).unwrap().hp = 60;
    let mut state = advance(&state, false);
    assert_eq!(state.units[&RANGER].pos, SUPPORT);
    assert!(player_view(&state, ME).tile(ENEMY_POS).unwrap().visible);
    state.units.remove(&THREAT);
    legatum_core::visibility::update_explored(&mut state);
    assert!(player_view(&state, ME).tile(ENEMY_POS).unwrap().visible);
    for _ in 0..3 {
        state = advance(&state, false);
        if state
            .units
            .values()
            .any(|u| u.owner == ME && legatum_core::rules::is_military(u.kind) && u.pos == CITY)
        {
            assert_eq!(state.units[&RANGER].pos, SUPPORT);
            return;
        }
    }
    panic!("cleared visible threat must allow a defender back into the city");
}

#[test]
fn hidden_enemy_changes_cannot_change_defense_orders() {
    let state = open_approach();
    let view = player_view(&state, ME);
    let mut changed = state.clone();
    let destination = (0..state.map.len())
        .map(|i| state.map.hex_at(i))
        .find(|h| {
            view.tile(*h).is_some_and(|t| !t.visible)
                && state.map.move_cost(*h).is_some()
                && state.units.values().all(|u| u.pos != *h)
        })
        .unwrap();
    let hidden = changed
        .units
        .values_mut()
        .find(|u| {
            u.owner != ME
                && legatum_core::data::unit_stats(u.kind).military
                && view.tile(u.pos).is_some_and(|t| !t.visible)
        })
        .unwrap();
    // Position is hidden; changing its kind/HP would change public army standings.
    hidden.pos = destination;
    assert!(
        serde_json::to_value(&view).unwrap()
            == serde_json::to_value(player_view(&changed, ME)).unwrap(),
        "hidden relocation must leave the player's information unchanged"
    );
    assert_eq!(decide(&state), decide(&changed));
}

#[test]
fn fortified_soldier_retreats_before_wounds_make_later_attack_rounds_lethal() {
    use legatum_core::events::Defender;
    use legatum_core::game::{blank_game, spawn_unit};
    use legatum_core::rules::cities::found_city;
    use legatum_core::state::Map;

    let enemy = PlayerId(0);
    let city = Hex::new(6, 8);
    // All three attacks roll maximum damage in their respective movement rounds.
    let mut setup = GameSetup::all_ai(113833);
    setup.width = 24;
    setup.height = 16;
    // Authored before cities fired back; city strike has its own tests in core.
    setup.players[ME.0 as usize].faction = Faction::Switzerland;
    setup.players[enemy.0 as usize].faction = Faction::Mongols;
    let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
    // This synthetic regression exercises active attacks, not the Peace start.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (owner, pos) in [(ME, city), (enemy, Hex::new(16, 8))] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut Vec::new()).unwrap();
    }
    // A fallen city: its garrison defends (core `combat::city_tile_defender`).
    state
        .cities
        .values_mut()
        .filter(|c| c.pos == city)
        .for_each(|c| c.hp = 0);
    let defender = spawn_unit(&mut state, ME, UnitKind::Spearman, city);
    state.units.get_mut(&defender).unwrap().fortify = 3;
    // Independent approaches: one attack in movement round two, two in round three.
    let attackers = [Hex::new(8, 8), Hex::new(9, 5), Hex::new(6, 11)]
        .map(|pos| spawn_unit(&mut state, enemy, UnitKind::Spearman, pos));
    legatum_core::visibility::update_explored(&mut state);
    let view = player_view(&state, ME);
    assert!(attackers.iter().all(|id| {
        view.units
            .iter()
            .any(|u| u.id == *id && u.hp == 100 && u.max_moves == 3)
    }));
    assert_eq!(state.units[&defender].hp, 100);
    let attack =
        combat::unit_attack_strength_for(Faction::Mongols, UnitKind::Spearman, 100, 0).total;
    let defense = combat::unit_defense_strength_for(
        Faction::Switzerland,
        UnitKind::Spearman,
        100,
        Terrain::Plains,
        3,
        0,
    )
    .total;
    assert_eq!(3 * combat::damage(attack, defense, combat::ROLL_MAX), 96);

    let attack_orders: Vec<_> = attackers
        .into_iter()
        .map(|unit| Order::MoveUnit { unit, to: city })
        .collect();
    assert!(validate_orders(&state, enemy, &attack_orders).is_empty());
    let (held, events) = resolve_turn(
        &state,
        &BTreeMap::from([
            (ME, vec![Order::SkipUnit { unit: defender }]),
            (enemy, attack_orders.clone()),
        ]),
    );
    let hits: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Battle {
                defender: Defender::Unit { unit: id, .. },
                defender_strength,
                damage_to_defender,
                defender_hp_after,
                ..
            } if *id == defender => Some((
                defender_strength.total,
                *damage_to_defender,
                *defender_hp_after,
            )),
            _ => None,
        })
        .collect();
    assert_eq!(hits, [(23, 32, 68), (20, 36, 0), (20, 36, 0)]);
    assert!(!held.units.contains_key(&defender));

    let orders = decide(&state);
    assert!(validate_orders(&state, ME, &orders).is_empty());
    // A garrison holds its city: only maximum rolls kill it, so the city is not
    // certain to fall and the Spearman keeps absorbing the attacks.
    assert!(
        !orders.iter().any(|order| matches!(order,
            Order::MoveUnit { unit, to } if *unit == defender && *to != city
        )),
        "the garrison must hold its city: {orders:?}"
    );

    // The same fortified Spearman in the field, beside the city, retreats before the
    // later rounds can become lethal (the retreat-before-worst-case forecast).
    let field = Hex::new(5, 8);
    state.units.get_mut(&defender).unwrap().pos = field;
    legatum_core::visibility::update_explored(&mut state);
    let orders = decide(&state);
    assert!(validate_orders(&state, ME, &orders).is_empty());
    assert!(
        orders.iter().any(|order| matches!(order,
            Order::MoveUnit { unit, to } if *unit == defender && *to != field && *to != city
        )),
        "a field unit must retreat before the later attacks become lethal: {orders:?}"
    );
}

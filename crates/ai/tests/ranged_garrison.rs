//! Playtest-6: a ranged garrison held instead of shooting; provenance in fixtures/garrison-t32.md.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(4);
const HUMAN: PlayerId = PlayerId(0);
const GARRISON: UnitId = UnitId(44);
const HUMAN_RANGER: UnitId = UnitId(37);
const HUMAN_SOLDIER: UnitId = UnitId(45);
const RESONATOR: Hex = Hex::new(20, 15);

#[derive(serde::Deserialize)]
struct ArchivedTurn {
    turn: u32,
    orders: BTreeMap<PlayerId, Vec<Order>>,
}

/// Replay the pinned T32 state and its orders under current rules. The mountain at (21, 14)
/// blocks the Ranger-to-city line, so neither city nor garrison takes these shots.
fn archived_states() -> Vec<GameState> {
    replay_archived_states(true)
}

fn replay_archived_states(check_pins: bool) -> Vec<GameState> {
    let mut state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/garrison-t32.json.gz")[..],
    ))
    .unwrap();
    assert_eq!(state.turn, 32);
    let turns: Vec<ArchivedTurn> =
        serde_json::from_str(include_str!("fixtures/garrison-t32-t33-orders.json")).unwrap();
    let mut states = vec![state.clone()];
    // Pin the current replay states while retaining the recorded orders.
    let current_hashes: Vec<String> =
        serde_json::from_str(include_str!("fixtures/garrison-current-hashes.json")).unwrap();
    let mut replay_hashes = Vec::new();
    for t in &turns {
        assert_eq!(t.turn, state.turn);
        state = resolve_turn(&state, &t.orders).0;
        replay_hashes.push(state.hash().to_string());
        states.push(state.clone());
    }
    if check_pins {
        assert_eq!(replay_hashes, current_hashes);
    }
    states
}

#[test]
#[ignore = "regenerates current-rule replay pins"]
fn repin_current_garrison_hashes() {
    let hashes: Vec<_> = replay_archived_states(false)
        .iter()
        .skip(1)
        .map(|s| s.hash().to_string())
        .collect();
    std::fs::write(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/garrison-current-hashes.json"
        ),
        serde_json::to_string_pretty(&hashes).unwrap() + "\n",
    )
    .unwrap();
}

fn decide(state: &GameState, difficulty: AiDifficulty) -> Vec<Order> {
    let view = player_view(state, ME);
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, ME.0 as u64]).next_u64();
    let mut brain = ScriptedBrain::with_difficulty(view.faction_of(ME), seed, ME, difficulty);
    // These positions are about the garrison's ranged shot, not the army-commitment genes
    // (default 1 since 2026-10-31), which move other units.
    brain.decisions.capture_hold = 0;
    brain.decisions.strike_kill = 0;
    brain.orders(&view)
}

fn garrison_order(orders: &[Order]) -> Option<&Order> {
    orders
        .iter()
        .find(|o| legatum_core::rules::units::order_unit(o) == Some(GARRISON))
}

#[test]
fn archived_positions_have_no_legal_garrison_shot_under_terrain_sight() {
    for state in archived_states() {
        assert!(state.at_war(HUMAN, ME), "T{}", state.turn);
        let garrison = &state.units[&GARRISON];
        assert_eq!(garrison.pos, RESONATOR);
        assert_eq!(garrison.kind, UnitKind::Ranger);
        assert_eq!(garrison.hp, 100, "the archived Ranger's shots now lack LOS");
        let city = state.cities.values().find(|c| c.pos == RESONATOR).unwrap();
        assert_eq!(
            city.hp,
            117 + legatum_core::data::CITY_HEAL * (state.turn - 32) as i32,
            "blocked shots leave the city free to regenerate"
        );
        let ranger = &state.units[&HUMAN_RANGER];
        assert_eq!(ranger.pos.distance(RESONATOR), 2);
        assert_eq!(
            legatum_core::los::blocker(RESONATOR, ranger.pos, |h| {
                state.map.get(h).map(|t| t.terrain)
            }),
            Some(Hex::new(21, 14)),
            "T{}",
            state.turn
        );
        let view = player_view(&state, ME);
        assert!(
            forecast_attack_mode(&view, GARRISON, ranger.pos, AttackMode::Ranged).is_none(),
            "T{}",
            state.turn
        );
        for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
            let orders = decide(&state, difficulty);
            assert_eq!(
                garrison_order(&orders),
                Some(&Order::SkipUnit { unit: GARRISON }),
                "T{} {difficulty:?}",
                state.turn
            );
            assert!(validate_orders(&state, ME, &orders).is_empty());
        }
    }
}

#[test]
fn ranged_garrison_fires_when_the_archived_corridor_is_opened() {
    for mut state in archived_states() {
        // Restore a legal two-hex shot without altering either side's position or orders.
        state.map.get_mut(Hex::new(21, 14)).unwrap().terrain = Terrain::Plains;
        let ranger = &state.units[&HUMAN_RANGER];
        assert!(legatum_core::los::visible(RESONATOR, ranger.pos, |h| {
            state.map.get(h).map(|t| t.terrain)
        }));
        for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
            let orders = decide(&state, difficulty);
            let Some(&Order::RangedAttack { target, .. }) = garrison_order(&orders) else {
                panic!(
                    "T{} {difficulty:?}: garrison did not shoot through the opened corridor",
                    state.turn
                );
            };
            assert!(
                state
                    .units
                    .values()
                    .any(|u| u.owner == HUMAN && u.pos == target && u.pos.distance(RESONATOR) <= 2),
                "T{} {difficulty:?}: shot {target:?}",
                state.turn
            );
            assert!(validate_orders(&state, ME, &orders).is_empty());
        }
    }
}

#[test]
fn opening_the_archived_corridor_exposes_the_city_but_not_its_garrison() {
    let mut state = archived_states().remove(0);
    state.map.get_mut(Hex::new(21, 14)).unwrap().terrain = Terrain::Plains;
    let orders = vec![Order::RangedAttack {
        unit: HUMAN_RANGER,
        target: RESONATOR,
    }];
    assert!(validate_orders(&state, HUMAN, &orders).is_empty());
    let (next, _) = resolve_turn(&state, &BTreeMap::from([(HUMAN, orders)]));
    assert_eq!(next.units[&GARRISON].hp, 100);
    assert!(
        next.cities
            .values()
            .find(|c| c.pos == RESONATOR)
            .unwrap()
            .hp
            < 117
    );
}

#[test]
fn a_soldier_that_can_take_the_city_next_turn_is_the_more_valuable_target() {
    // Counterfactual on T34: open the mountain corridor, then the Soldier stands where it attacked from on T35, next to a
    // nearly broken city, and the garrison has the 27 HP it had in the archive (before cities
    // shielded garrisons). The Ranger's volley breaks the 10-HP city in the ranged phase, which
    // exposes the garrison; the Soldier then fights it in the first movement round and, if it
    // wins, takes the city the same turn. That capture stake, not a fixed melee-first rule,
    // should outweigh the Ranger, which can only break the city.
    let mut state = archived_states().pop().unwrap();
    assert_eq!(state.turn, 34);
    state.map.get_mut(Hex::new(21, 14)).unwrap().terrain = Terrain::Plains;
    let adjacent = Hex::new(20, 14);
    assert!(!state.units.values().any(|u| u.pos == adjacent));
    state.units.get_mut(&HUMAN_SOLDIER).unwrap().pos = adjacent;
    state.units.get_mut(&GARRISON).unwrap().hp = 27;
    let city = state
        .cities
        .values_mut()
        .find(|c| c.pos == RESONATOR)
        .unwrap();
    city.hp = 10;
    let view = player_view(&state, ME);
    for target in [state.units[&HUMAN_RANGER].pos, adjacent] {
        assert!(forecast_attack_mode(&view, GARRISON, target, AttackMode::Ranged).is_some());
    }
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        let orders = decide(&state, difficulty);
        assert_eq!(
            garrison_order(&orders),
            Some(&Order::RangedAttack {
                unit: GARRISON,
                target: adjacent
            }),
            "{difficulty:?}"
        );
    }
}

#[test]
fn a_soldier_against_a_city_that_cannot_break_is_no_capture_threat() {
    // The same Soldier beside the full-HP garrison of a city that this turn's fire cannot
    // break: its attack only hits the city, so it carries no capture stake and the garrison
    // shoots the Ranger, whose volley would otherwise wear the city down.
    let mut state = archived_states().pop().unwrap();
    state.map.get_mut(Hex::new(21, 14)).unwrap().terrain = Terrain::Plains;
    let adjacent = Hex::new(20, 14);
    state.units.get_mut(&HUMAN_SOLDIER).unwrap().pos = adjacent;
    let city = state
        .cities
        .values_mut()
        .find(|c| c.pos == RESONATOR)
        .unwrap();
    city.hp = 200;
    // Replaying the archived orders on current rules (population is territory) now completes a
    // unit the old replay never built (u53, beside the Soldier), which the Soldier then
    // threatens more than the Ranger's volley. Drop it: this counterfactual is about the garrison
    // choosing between the Ranger and a Soldier that cannot capture.
    state.units.remove(&UnitId(53));
    for difficulty in [AiDifficulty::Normal, AiDifficulty::Hard] {
        let orders = decide(&state, difficulty);
        assert_eq!(
            garrison_order(&orders),
            Some(&Order::RangedAttack {
                unit: GARRISON,
                target: state.units[&HUMAN_RANGER].pos
            }),
            "{difficulty:?}"
        );
    }
}

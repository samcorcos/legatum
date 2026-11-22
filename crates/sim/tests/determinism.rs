//! Fixed order tapes pin engine behavior independently of today's AI or tape generator.
//! See fixtures/README.md for intentional regeneration and cross-target verification.

use legatum_core::{Faction, GameSetup};
use legatum_wasm::replay_tape_json;
use legatum_wasm::tape::{Tape, replay_checked};

const TAPES: [&str; 3] = [
    include_str!("fixtures/game-00000-seed-18446744073709551613.json"),
    include_str!("fixtures/game-00001-seed-18446744073709551614.json"),
    include_str!("fixtures/game-00002-seed-18446744073709551615.json"),
];

fn check_fixture(index: usize, seed: u64) {
    let tape = Tape::from_json(TAPES[index]).expect("committed tape parses");
    let mut setup = GameSetup::all_ai(seed);
    setup.turn_limit = Some(40);
    // Check the established seeded shuffle across the original nine-faction pool.
    // The tape's initial and per-turn hashes pin the concrete roster; repin regenerates it.
    let mut factions: [Faction; 9] = Faction::ALL[..9].try_into().unwrap();
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (p, f) in setup.players.iter_mut().zip(factions) {
        p.faction = f;
    }
    assert_eq!(tape.setup, setup, "seed {seed}; run tools/sim/repin.sh");
    assert_eq!(
        tape.turns.len(),
        40,
        "fixture must retain all 40 turns; run tools/sim/repin.sh"
    );

    let replay = replay_checked(&tape)
        .unwrap_or_else(|e| panic!("seed {seed}: {e}; run tools/sim/repin.sh"));
    assert_eq!(replay.initial_hash, tape.initial_hash, "seed {seed}");
    assert_eq!(replay.hashes.len(), tape.turns.len(), "seed {seed}");
    for (i, (hash, turn)) in replay.hashes.iter().zip(&tape.turns).enumerate() {
        assert_eq!(turn.turn, i as u32 + 1, "seed {seed}");
        assert_eq!(
            hash.to_string(),
            turn.hash,
            "seed {seed}, turn {}",
            turn.turn
        );
    }
}

#[test]
fn recorded_hashes_seed_max_minus_two() {
    check_fixture(0, u64::MAX - 2);
}

#[test]
fn recorded_hashes_seed_max_minus_one() {
    check_fixture(1, u64::MAX - 1);
}

#[test]
fn recorded_hashes_seed_max() {
    check_fixture(2, u64::MAX);
}

fn short_tape() -> Tape {
    let mut tape = Tape::from_json(TAPES[0]).unwrap();
    tape.turns.truncate(2);
    tape
}

#[test]
fn wasm_api_rejects_changed_initial_hash() {
    let mut tape = short_tape();
    tape.initial_hash ^= 1;
    let error = replay_tape_json(&tape.to_json()).unwrap_err();
    assert!(error.starts_with("initial state hash differs:"), "{error}");
}

#[test]
fn wasm_api_rejects_changed_turn_hash() {
    let mut tape = short_tape();
    let hash = tape.turns[1].hash.parse::<u64>().unwrap();
    tape.turns[1].hash = (hash ^ 1).to_string();
    let error = replay_tape_json(&tape.to_json()).unwrap_err();
    assert!(error.starts_with("hash differs after turn 2:"), "{error}");
}

#[test]
fn wasm_api_rejects_wrong_turn_number() {
    let mut tape = short_tape();
    tape.turns[0].turn = 2;
    let error = replay_tape_json(&tape.to_json()).unwrap_err();
    assert_eq!(error, "tape turn 1 has turn number 2, expected 1");
}

#[test]
fn research_queue_tape_survives_every_save_boundary() {
    let tape = Tape::from_json(include_str!("fixtures/research-queue.json")).unwrap();
    assert_eq!(tape.turns.len(), 120);
    replay_checked(&tape).unwrap();
}

#[test]
fn scripted_hard_city_fire_tape_survives_every_save_boundary() {
    let tape = Tape::from_json(include_str!("fixtures/city-strike-awareness.json")).unwrap();
    assert_eq!(tape.turns.len(), 160);
    assert!(
        tape.setup
            .players
            .iter()
            .all(|p| p.difficulty == legatum_core::AiDifficulty::Hard)
    );
    replay_checked(&tape).unwrap();
}

#[test]
fn diplomacy_boundary_tape_preserves_pending_offers_and_guarantees() {
    let tape = Tape::from_json(include_str!("fixtures/diplomacy-slice-1.json")).unwrap();
    assert_eq!(tape.version, legatum_wasm::tape::TAPE_VERSION);
    replay_checked(&tape).unwrap();
}

#[test]
fn production_queue_and_separate_purchase_tape_survives_every_save_boundary() {
    let tape = Tape::from_json(include_str!("fixtures/production-queue-buy.json")).unwrap();
    assert_eq!(tape.turns.len(), 150);
    replay_checked(&tape).unwrap();
}

#[test]
fn confirmed_movement_path_survives_every_save_boundary() {
    let tape = Tape::from_json(include_str!("fixtures/movement-path.json")).unwrap();
    assert_eq!(tape.turns.len(), 8);
    replay_checked(&tape).unwrap();
}

#[test]
fn alliance_tape_survives_every_save_boundary_with_gold_and_betrayal() {
    let tape = Tape::from_json(include_str!("fixtures/diplomacy-alliance.json")).unwrap();
    assert_eq!(tape.version, legatum_wasm::tape::TAPE_VERSION);
    replay_checked(&tape).unwrap();
    let mut state = tape.initial_state();
    let breach_gold = legatum_core::diplomacy::TreatyKind::Alliance.breach_cost();
    let (mut joined, mut betrayed, mut breach, mut free_exit, mut paid) =
        (false, false, false, false, false);
    for turn in &tape.turns {
        let (next, events) = legatum_core::resolve_turn(&state, &turn.orders);
        state = next;
        let d = state.diplomacy.as_ref().unwrap();
        betrayed |= !d.betrayals.is_empty();
        for e in &events {
            match e {
                legatum_core::Event::WarJoined { .. } => joined = true,
                legatum_core::Event::TreatyBreached { cost, .. } => breach |= *cost == breach_gold,
                legatum_core::Event::TreatyEnded { reason, .. } => {
                    free_exit |= reason.ends_with("left the Alliance")
                        && !events
                            .iter()
                            .any(|e| matches!(e, legatum_core::Event::TreatyBreached { .. }));
                }
                legatum_core::Event::TreatyPayment { .. } => paid = true,
                _ => {}
            }
        }
    }
    assert!(joined && betrayed && breach && free_exit && paid);
}

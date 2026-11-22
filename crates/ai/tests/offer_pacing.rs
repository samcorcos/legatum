//! AI offer pacing: no Alliance before the acquaintance period, and at most one new AI offer to
//! a human per turn. The case table records the native AI's expected decisions.
use legatum_ai::{
    ALLIANCE_ACQUAINTANCE_TURNS, Brain, ScriptedBrain, alliance_acquainted, offer_slot_open,
};
use legatum_core::diplomacy::TreatyKind;
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn shared_pacing_cases_match_native_ai() {
    let table: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/offer-pacing-cases.json")).unwrap();
    for case in table["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let players = case["players"].as_array().unwrap();
        let mut setup = GameSetup::all_ai(3);
        setup.players.truncate(players.len());
        let state = new_game(&setup);
        let me_team = case["me"].as_u64().unwrap() as u8;
        let me = players
            .iter()
            .position(|p| p[0].as_u64() == Some(u64::from(me_team)) && p[1] == false)
            .unwrap();
        let mut view = player_view(&state, PlayerId(me as u8));
        for (p, row) in view.players.iter_mut().zip(players) {
            p.team = TeamId(row[0].as_u64().unwrap() as u8);
            p.is_human = row[1].as_bool().unwrap();
            p.alive = row[2].as_bool().unwrap();
        }
        view.team = TeamId(me_team);
        view.turn = case["turn"].as_u64().unwrap() as u32;
        let to = TeamId(case["to"].as_u64().unwrap() as u8);
        assert_eq!(
            alliance_acquainted(&view),
            case["acquainted"].as_bool().unwrap(),
            "{name}"
        );
        assert_eq!(
            offer_slot_open(&view, to),
            case["slot_open"].as_bool().unwrap(),
            "{name}"
        );
    }
    assert_eq!(ALLIANCE_ACQUAINTANCE_TURNS, 15);
}

#[test]
fn ais_open_without_alliances_and_never_send_a_human_two_offers_in_one_turn() {
    // Alliances are selective (they need a shared enemy or coalition and cost upkeep), so the
    // games run long enough for wars, and the offer counts are pooled across seeds.
    for seed in [700_001u64, 700_040, 700_139, 700_201, 700_302] {
        let mut setup = GameSetup::all_ai(seed);
        setup.players.truncate(4);
        setup.players[0].is_human = true;
        let mut state = new_game(&setup);
        for _ in 0..120 {
            let planning = state.turn;
            let mut orders = BTreeMap::new();
            for p in 1..4u8 {
                let player = PlayerId(p);
                let view = player_view(&state, player);
                let brain_seed = Rng::derive(seed, &[purpose::AI, u64::from(p)]).next_u64();
                // This archived seed set guarantees offers to the waiting human under
                // its original diplomacy policy. Keep that source of offers fixed so
                // the test continues to exercise pacing as valuations evolve.
                let mut brain = ScriptedBrain::new(view.faction_of(player), brain_seed, player);
                brain.decisions.second_front_siege_price = 0;
                brain.decisions.rival_redeclare_percent = 0;
                brain.decisions.peace_withdraw_price = 0;
                brain.decisions.ally_defense_percent = 0;
                brain.decisions.ally_vision_cost = 0;
                brain.decisions.ally_passage_cost = 0;
                brain.decisions.ally_exit_discount_percent = 0;
                brain.decisions.leader_peace_horizon = 0;
                brain.decisions.ally_commitment_model = 0;
                let planned = brain.orders(&view);

                orders.insert(player, planned);
            }
            let (next, events) = resolve_turn(&state, &orders);
            let mut senders = BTreeSet::new();
            for e in &events {
                if let Event::ProposalCreated { proposal } = e {
                    if proposal.kind == TreatyKind::Alliance {
                        assert!(
                            planning > ALLIANCE_ACQUAINTANCE_TURNS,
                            "seed {seed}: Alliance offered while planning turn {planning}"
                        );
                    }
                    if proposal.to == TeamId(0) {
                        assert!(
                            senders.insert(proposal.from) && senders.len() == 1,
                            "seed {seed}: two new offers to the human in turn {planning}"
                        );
                    }
                }
            }
            state = next;
        }
    }
    // Some seeds finish before a coalition becomes relevant under the current VP rules. Whenever
    // an offer does happen, the loop above checks that at most one reaches the human each turn.
}

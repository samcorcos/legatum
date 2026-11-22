//! Alliances paid in Gold: upkeep, an ally answering a call to arms, an early exit (breach Gold and a
//! betrayal record), a free exit after the protected window, and war between former allies.
//! Standard pace; every empire founds its capital on turn 1.
use legatum_core::{
    diplomacy::{ALLIANCE_PROTECTED_TURNS, Agreement, ProposalId, TreatyKind},
    *,
};
use legatum_wasm::tape::{Tape, TapeTurn};
use std::collections::BTreeMap;

fn main() {
    let output = std::env::args().nth(1).expect("output path");
    let mut setup = GameSetup::all_ai(9943);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players[1].team = TeamId(0);
    // These capitals never meet (nobody explores), so every team starts having met the others;
    // contact has core tests.
    setup.rule_experiments.start_met = true;
    let mut tape = Tape {
        version: legatum_wasm::tape::TAPE_VERSION,
        setup: setup.clone(),
        initial_hash: 0,
        turns: vec![],
    };
    let mut state = tape.initial_state();
    tape.initial_hash = state.hash();
    let breach = TreatyKind::Alliance.breach_cost();
    let offer = |to| Order::ProposeTreaty {
        to: TeamId(to),
        kind: TreatyKind::Alliance,
    };
    let accept = |id| Order::RespondToProposal {
        proposal: ProposalId(id),
        accept: true,
        explanation: None,
    };
    let leave = |with| Order::BreakTreaty {
        with: TeamId(with),
        agreement: Agreement::Security,
    };
    let war = |against| Order::DeclareWar {
        against: TeamId(against),
        breach: false,
    };
    // Phase 0: found capitals. Phase 1: wait until player 4 can pay the breach.
    // Then one scripted step per turn, waiting where a step needs it.
    let mut phase = 0;
    let mut signed = 0;
    for turn in 1..=400 {
        let rich = |p: u8, gold: i32| state.player(PlayerId(p)).gold >= gold;
        let os: Vec<(u8, Vec<Order>)> = match phase {
            0 => state
                .units
                .values()
                .filter(|u| u.kind == UnitKind::Settler)
                .map(|u| (u.owner.0, vec![Order::FoundCity { unit: u.id }]))
                .collect(),
            1 if rich(4, breach) => vec![
                (0, vec![offer(2)]),
                (2, vec![offer(3)]),
                (4, vec![offer(6)]),
            ],
            2 => {
                signed = turn;
                vec![
                    (2, vec![accept(1)]),
                    (3, vec![accept(2)]),
                    (6, vec![accept(3)]),
                ]
            }
            // Player 0 declares: its ally (team 2) is called to arms against team 5.
            3 => vec![(0, vec![war(5)])],
            // Team 2 joins the war by answering the call.
            4 => {
                let call = state.diplomacy.as_ref().unwrap().calls[0].id;
                vec![(
                    2,
                    vec![Order::RespondToProposal {
                        proposal: call,
                        accept: true,
                        explanation: None,
                    }],
                )]
            }
            // Player 4 leaves its Alliance inside the window.
            5 => vec![(4, vec![leave(6)])],
            6 if turn > signed + ALLIANCE_PROTECTED_TURNS => vec![(0, vec![leave(2)])],
            7 => vec![(0, vec![war(2)])],
            8 => break,
            _ => {
                let (next, _) = resolve_turn(&state, &BTreeMap::new());
                state = next;
                tape.turns.push(TapeTurn {
                    turn,
                    orders: BTreeMap::new(),
                    hash: state.hash().to_string(),
                });
                continue;
            }
        };
        phase += 1;
        let orders: BTreeMap<_, _> = os.into_iter().map(|(p, os)| (PlayerId(p), os)).collect();
        for (p, os) in &orders {
            assert!(
                validate_orders(&state, *p, os).is_empty(),
                "turn {turn}: {:?}",
                validate_orders(&state, *p, os)
            );
        }
        let (next, events) = resolve_turn(&state, &orders);
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::OrderFailed { .. })),
            "{events:?}"
        );
        state = next;
        tape.turns.push(TapeTurn {
            turn,
            orders,
            hash: state.hash().to_string(),
        });
    }
    assert_eq!(phase, 8, "the tape reached every step");
    assert!(state.teams_at_war(TeamId(2), TeamId(5)));
    assert!(state.teams_at_war(TeamId(0), TeamId(2)));
    std::fs::write(output, tape.to_json()).unwrap();
}

//! Regenerate the small hand-authored boundary tape; the three mixed-AI fixtures use the sim.
use legatum_core::{
    diplomacy::{ProposalId, TreatyKind},
    *,
};
use legatum_wasm::tape::{TAPE_VERSION, Tape, TapeTurn};
use std::collections::BTreeMap;
fn main() {
    let output = std::env::args().nth(1).expect("output tape path");
    let mut setup = GameSetup::all_ai(9941);
    // A scripted scenario tape: barbarians (G10) stay off so it keeps testing its feature.
    setup.barbarians = false;
    setup.players[1].team = TeamId(0);
    let mut state = new_game(&setup);
    let mut tape = Tape {
        version: TAPE_VERSION,
        setup,
        initial_hash: state.hash(),
        turns: vec![],
    };
    let war = || Order::DeclareWar {
        against: TeamId(2),
        breach: false,
    };
    let offer = || Order::ProposeTreaty {
        to: TeamId(2),
        kind: TreatyKind::Peace,
    };
    for turn in 1..=12 {
        let orders = match turn {
            1 => BTreeMap::from([
                (PlayerId(0), vec![war()]),
                (PlayerId(1), vec![war()]),
                (
                    PlayerId(2),
                    vec![Order::DeclareWar {
                        against: TeamId(0),
                        breach: false,
                    }],
                ),
            ]),
            2 | 7 | 9 => BTreeMap::from([(PlayerId(0), vec![offer()])]),
            3 => BTreeMap::from([(
                PlayerId(2),
                vec![Order::RespondToProposal {
                    proposal: ProposalId(1),
                    accept: true,
                    explanation: None,
                }],
            )]),
            5 | 6 => BTreeMap::from([(PlayerId(0), vec![war()])]),
            8 => BTreeMap::from([(
                PlayerId(2),
                vec![Order::RespondToProposal {
                    proposal: ProposalId(2),
                    accept: false,
                    explanation: None,
                }],
            )]),
            _ => BTreeMap::new(),
        };
        state = resolve_turn(&state, &orders).0;
        tape.turns.push(TapeTurn {
            turn,
            orders,
            hash: state.hash().to_string(),
        });
    }
    std::fs::write(output, tape.to_json()).unwrap();
}

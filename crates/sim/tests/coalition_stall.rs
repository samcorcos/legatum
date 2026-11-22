//! Regression: held-out coalition game 910030, rotation 4 (solo teams, Standard, Normal
//! scripted AIs) ran to the 1,200-turn cap. Every team had banked its seven permanent VP,
//! ties at 80% made the lowest TeamId the coalition target, so
//! whoever touched 8 of 10 VP was dogpiled until the tie moved again. With
//! a tie at the top no longer a coalition target, the same game ends (turn 408 when fixed).
use legatum_core::{Faction, GameSetup, TeamId};
use legatum_wasm::session::{BrainKind, Session};

#[test]
fn held_out_coalition_stall_seed_finishes() {
    let mut setup = GameSetup::all_ai(910030);
    setup.turn_limit = None;
    for (seat, p) in setup.players.iter_mut().enumerate() {
        p.faction = Faction::ALL[(seat + 4) % 8];
        p.team = TeamId(seat as u8);
    }
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    // Baseline games on these maps end by turn 550. With promotions (G9) this seed follows a
    // different path through the same title deadlock (0 captures, every team at 5-9 VP) and
    // ends at turn 726; 900 still fails long before the old 1,200-turn stall.
    while !session.state.finished && session.state.turn < 900 {
        let orders = session.ai_orders();
        session.step(&orders);
    }
    assert!(
        session.state.finished && session.state.winner.is_some(),
        "stalled at turn {}",
        session.state.turn
    );
}

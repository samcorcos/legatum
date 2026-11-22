//! The scripted AI under the strict era gate, in real games: it never leaves its research
//! empty (apart from the turn an era is entered, when the gate allows no queue), every order
//! it gives validates, and an empire that enters an era has learned every technology of the
//! eras before it.
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::eras;
use legatum_core::{AiDifficulty, GameSetup, Order, PlayerId, new_game, resolve_turn};
use std::collections::BTreeMap;

#[test]
fn ais_keep_researching_and_finish_each_era_they_leave() {
    let mut setup = GameSetup::all_ai(31);
    for p in &mut setup.players {
        p.difficulty = AiDifficulty::Hard;
    }
    let mut state = new_game(&setup);
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(
                p.faction,
                31 ^ u64::from(p.id.0),
                p.id,
                AiDifficulty::Hard,
            )
        })
        .collect();
    let mut idle_streak = vec![0u32; state.players.len()];
    let mut best_era = 0;
    while !state.finished && state.turn <= 140 {
        let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for p in state.players.iter().filter(|p| p.alive) {
            let view = legatum_core::player_view(&state, p.id);
            let os = brains[p.id.0 as usize].orders(&view);
            let errors = legatum_core::validate_orders(&state, p.id, &os);
            assert!(
                errors.is_empty(),
                "turn {} {:?}: {errors:?}\n{os:#?}",
                state.turn,
                p.id
            );
            orders.insert(p.id, os);
        }
        let (next, events) = resolve_turn(&state, &orders);
        state = next;
        for p in state.players.iter().filter(|p| p.alive) {
            let entered = events.iter().any(
                |e| matches!(e, legatum_core::Event::EraEntered { player, .. } if *player == p.id),
            );
            let idle = p.researching.is_none() && !eras::all_researched(&p.techs) && !entered;
            let streak = &mut idle_streak[p.id.0 as usize];
            *streak = if idle { *streak + 1 } else { 0 };
            assert!(*streak < 2, "{:?} sat idle on turn {}", p.id, state.turn);
            // Everything before the current era is known: the gate leaves no gaps.
            let era = eras::current_era(&p.techs);
            best_era = best_era.max(era);
            for before in 0..era {
                assert!(
                    eras::era_techs(before).all(|t| p.techs.contains(&t)),
                    "{:?} is in era {era} without finishing era {before}",
                    p.id
                );
            }
        }
    }
    assert!(
        best_era >= 3,
        "the fastest empire only reached era {best_era}"
    );
}

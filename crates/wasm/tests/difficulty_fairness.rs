//! Pillar 6: identical submitted orders must have identical consequences at every difficulty.
use legatum_core::{AiDifficulty, GameSetup, GameState, new_game, player_view, resolve_turn};
use legatum_wasm::session::{BrainKind, Session};

fn assert_same_rules_and_views(reference: &GameState, alternative: &GameState) {
    // Difficulty is saved setup metadata, so raw state hashes intentionally differ.
    // Normalize only that metadata; the public view now displays it too. All resources, prices,
    // visibility and rules stay compared.
    let mut normalized = alternative.clone();
    for p in &mut normalized.setup.players {
        p.difficulty = AiDifficulty::Normal;
    }
    assert_eq!(&normalized, reference);
    for p in &reference.players {
        assert_eq!(player_view(reference, p.id), player_view(&normalized, p.id));
    }
}

#[test]
fn same_orders_have_identical_rules_and_views_at_every_difficulty() {
    let setup = GameSetup::all_ai(930001);
    let mut normal = Session::new(&setup, |_| BrainKind::Scripted);
    let mut alternatives: Vec<_> = [AiDifficulty::Easy, AiDifficulty::Hard]
        .into_iter()
        .map(|difficulty| {
            let mut setup = setup.clone();
            for p in &mut setup.players {
                p.difficulty = difficulty;
            }
            new_game(&setup)
        })
        .collect();
    for state in &alternatives {
        assert_same_rules_and_views(&normal.state, state);
    }
    for _ in 0..600 {
        if normal.state.finished {
            break;
        }
        // After ordinary settlement, expose every late-game option in this synthetic fixture.
        // Apply the identical fixture change to all three states; the production game does
        // not grant technologies. This avoids relying on a particular AI research route.
        if normal.state.turn == 31 {
            for state in std::iter::once(&mut normal.state).chain(alternatives.iter_mut()) {
                for player in &mut state.players {
                    player.techs = legatum_core::TechId::ALL.into_iter().collect();
                    player.researching = None;
                }
            }
            for state in &alternatives {
                assert_same_rules_and_views(&normal.state, state);
            }
        }
        let orders = normal.ai_orders();
        let events = normal.step(&orders);
        for state in &mut alternatives {
            let (next, alternative_events) = resolve_turn(state, &orders);
            assert_eq!(events, alternative_events);
            *state = next;
            assert_same_rules_and_views(&normal.state, state);
        }
    }
    assert!(normal.state.finished, "check through natural victory");
    assert!(
        normal.state.players.iter().any(|p| p
            .techs
            .iter()
            .any(|t| legatum_core::eras::era_of_tech(*t) >= 5)),
        "exercise the late eras"
    );
}

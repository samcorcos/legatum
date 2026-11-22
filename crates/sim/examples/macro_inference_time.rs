//! Native single-thread timing for the model's feature extraction and integer dot product.
use legatum_ai::learned_macro;
use legatum_ai::plan::PlanKind;
use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, GameSetup, PlayerId, new_game, player_view, resolve_turn};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let mut setup = GameSetup::all_ai(7_300_001);
    for player in &mut setup.players {
        player.difficulty = AiDifficulty::Hard;
    }
    let mut state = new_game(&setup);
    let mut brains: Vec<_> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(p.faction, 100 + p.id.0 as u64, p.id, AiDifficulty::Hard)
        })
        .collect();
    while state.turn < 60 && !state.finished {
        let orders = brains
            .iter_mut()
            .filter(|b| state.player(b.player).alive)
            .map(|b| (b.player, b.orders(&player_view(&state, b.player))))
            .collect();
        state = resolve_turn(&state, &orders).0;
    }
    let me = PlayerId(0);
    let view = player_view(&state, me);
    let plan = view
        .cities
        .iter()
        .find(|c| c.owner != me)
        .map_or(PlanKind::Economy, |c| PlanKind::Campaign {
            against: view.team_of(c.owner),
            target: c.id,
        });
    let trials = 100_000;
    let start = Instant::now();
    let mut checksum = 0i64;
    for _ in 0..trials {
        let features = learned_macro::features(black_box(&view), black_box(&plan), None);
        checksum += i64::from(black_box(learned_macro::value(&features, &plan, None)));
    }
    println!(
        "turn={} players={} units={} cities={} plan={plan:?} ns_per_decision={} checksum={checksum}",
        view.turn,
        view.players.len(),
        view.units.len(),
        view.cities.len(),
        start.elapsed().as_nanos() / trials
    );
}

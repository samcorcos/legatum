//! Decision parity against the archived playtest 8.5 Mongols game (seed/salt 874654782136188).
use legatum_ai::{
    Brain, ScriptedBrain,
    search::{SearchBrain, SearchConfig},
};
use legatum_core::{Order, PlayerId, UnitId, UnitKind, player_view, resolve_turn};
use legatum_wasm::session::brain_seed;
use std::{collections::BTreeMap, path::Path};

fn local_unit_order(orders: &[Order], local: &std::collections::BTreeSet<UnitId>) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| {
            let unit = match o {
                Order::SetAutomation { unit, .. }
                | Order::MoveUnit { unit, .. }
                | Order::MoveUnitPath { unit, .. }
                | Order::SleepUnit { unit, .. }
                | Order::SkipUnit { unit }
                | Order::FoundCity { unit }
                | Order::DisbandUnit { unit }
                | Order::RangedAttack { unit, .. }
                | Order::UpgradeUnit { unit } => Some(*unit),
                _ => None,
            };
            unit.is_some_and(|id| local.contains(&id))
        })
        .cloned()
        // The archive predates the idle-order fold (2026-10-20): its AI Holds read as Sleep now,
        // and the AI's one stay-put order is Skip.
        .map(|o| match o {
            Order::SleepUnit { unit, .. } => Order::SkipUnit { unit },
            o => o,
        })
        .collect()
}

#[test]
fn archived_mongols_settler_and_escort_decisions_match_for_three_real_episodes() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/playtest-8.5-mongols-win.current.tape.json.zst");
    let recording = legatum_archive::read(&fixture).expect("read archived full game tape");
    assert_eq!(recording.tape.setup.seed, 874_654_782_136_188);
    // The archived game's map predates the current start-position rules.
    let mut state = legatum_wasm::json::parse_state_for_rehash(include_str!(
        "fixtures/archived/playtest-8.5-mongols-initial.current.json"
    ))
    .expect("read archived initial state");
    let mut brains: BTreeMap<PlayerId, Box<dyn Brain>> = BTreeMap::new();
    for p in &state.players {
        if !p.is_human {
            let base = ScriptedBrain::with_difficulty(
                p.faction,
                brain_seed(state.setup.seed, p.id),
                p.id,
                state.setup.players[p.id.0 as usize].difficulty,
            );
            let brain: Box<dyn Brain> = if state.setup.players[p.id.0 as usize].search {
                Box::new(SearchBrain::new(
                    base,
                    SearchConfig::in_app(state.players.len()),
                ))
            } else {
                Box::new(base)
            };
            brains.insert(p.id, brain);
        }
    }
    let mut checked_episodes = std::collections::BTreeSet::new();
    for recorded in &recording.tape.turns {
        let mut replay_orders = recorded.orders.clone();
        for (&seat, brain) in &mut brains {
            if !state.player(seat).alive {
                continue;
            }
            let view = player_view(&state, seat);
            let generated = brain.orders(&view);
            let settlers: Vec<_> = state
                .units_of(seat)
                .filter(|u| u.kind == UnitKind::Settler)
                .map(|u| u.id)
                .collect();
            for settler in settlers {
                let pos = state.units[&settler].pos;
                let local: std::collections::BTreeSet<_> = state
                    .units_of(seat)
                    .filter(|u| {
                        u.id == settler
                            || (legatum_core::rules::is_military(u.kind)
                                && u.pos.distance(pos) <= 2)
                    })
                    .map(|u| u.id)
                    .collect();
                let expected = local_unit_order(
                    recorded.orders.get(&seat).map(Vec::as_slice).unwrap_or(&[]),
                    &local,
                );
                let actual = local_unit_order(&generated, &local);
                if expected != actual {
                    panic!(
                        "turn {} seat {:?} settler {:?}: recorded {expected:?}, generated {actual:?}",
                        recorded.turn, seat, settler
                    );
                }
                if !expected.is_empty() {
                    checked_episodes.insert((seat, settler));
                    // Replace the seat's local unit orders with the independently recomputed ones;
                    // all other game orders remain as recorded for this replayed turn.
                    let retained: Vec<_> = replay_orders
                        .get(&seat)
                        .into_iter()
                        .flatten()
                        .filter(|o| local_unit_order(std::slice::from_ref(*o), &local).is_empty())
                        .cloned()
                        .collect();
                    replay_orders
                        .insert(seat, retained.into_iter().chain(actual.clone()).collect());
                    break;
                }
            }
        }
        let recorded_state = resolve_turn(&state, &recorded.orders).0;
        let recomputed_state = resolve_turn(&state, &replay_orders).0;
        let settler_state = |s: &legatum_core::GameState| {
            (
                s.units
                    .values()
                    .filter(|u| u.kind == UnitKind::Settler)
                    .map(|u| (u.id, u.owner, u.pos))
                    .collect::<Vec<_>>(),
                s.cities
                    .values()
                    .map(|c| (c.owner, c.pos))
                    .collect::<Vec<_>>(),
            )
        };
        assert_eq!(
            settler_state(&recomputed_state),
            settler_state(&recorded_state),
            "turn {} Settler outcome",
            recorded.turn
        );
        state = recomputed_state;
        if checked_episodes.len() >= 3 {
            break;
        }
    }
    assert!(
        checked_episodes.len() >= 3,
        "only {} distinct recorded Settler episodes matched",
        checked_episodes.len()
    );
}

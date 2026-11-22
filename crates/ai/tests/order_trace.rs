//! The order trace is diagnostics only: a traced run must emit exactly the live orders and leave
//! exactly the live brain memory, and its stages must account for every order that was dropped.
use legatum_ai::{Brain, OrderTrace, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::*;
use std::collections::BTreeMap;

fn load(archive: &[u8]) -> GameState {
    let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(archive)).unwrap();
    state
}

fn brain(state: &GameState, player: PlayerId, difficulty: AiDifficulty) -> ScriptedBrain {
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, player.0 as u64]).next_u64();
    ScriptedBrain::with_difficulty(state.player(player).faction, seed, player, difficulty)
}

fn canonical(orders: &[Order]) -> Vec<String> {
    let mut out: Vec<String> = orders
        .iter()
        .map(|o| serde_json::to_string(o).unwrap())
        .collect();
    out.sort();
    out
}

/// Replaying every stage's additions and rejections over the proposed orders gives the executed ones.
fn assert_trace_accounts_for_every_order(trace: &OrderTrace) {
    let mut orders = trace.proposed.clone();
    for stage in &trace.stages {
        for rejected in &stage.rejected {
            assert!(
                !rejected.reason.is_empty(),
                "{} rejected without a reason",
                stage.name
            );
            let i = orders
                .iter()
                .position(|o| *o == rejected.order)
                .unwrap_or_else(|| panic!("{} rejects an order it never saw", stage.name));
            orders.remove(i);
        }
        orders.extend(stage.added.iter().cloned());
    }
    assert_eq!(canonical(&orders), canonical(&trace.executed));
}

/// Returns how many orders the stages added or rejected over the run.
fn run(archive: &[u8], difficulty: AiDifficulty) -> usize {
    let mut state = load(archive);
    let players: Vec<PlayerId> = state.players.iter().map(|p| p.id).collect();
    let mut untraced: BTreeMap<PlayerId, ScriptedBrain> = players
        .iter()
        .map(|&p| (p, brain(&state, p, difficulty)))
        .collect();
    let mut traced: BTreeMap<PlayerId, ScriptedBrain> = players
        .iter()
        .map(|&p| (p, brain(&state, p, difficulty)))
        .collect();
    let (mut compared, mut stage_changes) = (0, 0);
    for _ in 0..8 {
        let mut turn_orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for &p in &players {
            let view = player_view(&state, p);
            let live = untraced.get_mut(&p).unwrap().orders(&view);
            let (orders, trace) = traced.get_mut(&p).unwrap().order_diagnosis(&view);
            assert_eq!(
                live, orders,
                "player {p:?} turn {}: traced orders differ",
                state.turn
            );
            assert_eq!(trace.executed, live);
            assert_eq!(
                untraced[&p].opponent_memory(),
                traced[&p].opponent_memory(),
                "player {p:?} turn {}: traced run changed brain memory",
                state.turn
            );
            assert_eq!(
                untraced[&p].search_events().len(),
                traced[&p].search_events().len()
            );
            assert_trace_accounts_for_every_order(&trace);
            stage_changes += trace
                .stages
                .iter()
                .map(|s| s.added.len() + s.rejected.len())
                .sum::<usize>();
            // The &self diagnostic runs the live path on a copy: the same orders, no memory change.
            let before = traced[&p].opponent_memory();
            let (replanned, _) = traced[&p].production_diagnosis(&view);
            assert_eq!(
                replanned, live,
                "player {p:?}: production_diagnosis differs from live"
            );
            assert_eq!(before, traced[&p].opponent_memory());
            compared += 1;
            turn_orders.insert(p, live);
        }
        state = resolve_turn(&state, &turn_orders).0;
    }
    assert!(compared >= 8 * players.len());
    stage_changes
}

#[test]
fn traced_run_emits_the_same_orders_and_leaves_the_same_memory_as_an_untraced_run() {
    // Easy and Normal lapse orders, so the finish stage has rejections to account for.
    let mut changes = 0;
    for difficulty in [AiDifficulty::Hard, AiDifficulty::Normal, AiDifficulty::Easy] {
        changes += run(include_bytes!("fixtures/campaign-t86.json.gz"), difficulty);
        changes += run(include_bytes!("fixtures/defense-t121.json.gz"), difficulty);
    }
    assert!(
        changes > 0,
        "no stage ever changed an order; the fixtures exercise nothing"
    );
}

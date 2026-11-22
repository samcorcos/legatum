//! A committed Strike on a 0-HP city whose owner keeps re-garrisoning it; see
//! fixtures/siege-ring.md. Hard India plays from its own view and its restored memory; every
//! other seat replays its recorded orders where they are still legal.

use legatum_ai::{Brain, OpponentMemory, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::rules::units::order_unit;
use legatum_core::*;
use std::collections::BTreeMap;

const INDIA: PlayerId = PlayerId(2);
const CITY: Hex = Hex { q: 37, r: 7 };
/// The two healthy melee units that never joined the ring (Cuirassier, Assault Trooper).
const HEALTHY: [UnitId; 2] = [UnitId(234), UnitId(9)];
/// Every melee unit on the Strike's roster near the city on T143–T145.
const MELEE: [UnitId; 5] = [
    UnitId(127),
    UnitId(152),
    UnitId(268),
    UnitId(234),
    UnitId(9),
];

#[derive(serde::Deserialize)]
struct Start {
    state: GameState,
    memory: OpponentMemory,
}

#[derive(serde::Deserialize)]
struct Fixture {
    t139: Start,
    t143: Start,
    /// Every other seat's recorded orders, by turn.
    orders: BTreeMap<u32, BTreeMap<PlayerId, Vec<Order>>>,
}

fn fixture() -> Fixture {
    serde_json::from_reader(flate2::read::GzDecoder::new(
        &include_bytes!("fixtures/siege-ring-t139-t145.json.gz")[..],
    ))
    .unwrap()
}

/// Recorded orders minus any that the changed position has made illegal.
fn legal(state: &GameState, player: PlayerId, mut orders: Vec<Order>) -> Vec<Order> {
    loop {
        let errors = validate_orders(state, player, &orders);
        let Some(e) = errors.first() else {
            return orders;
        };
        orders.remove(e.index);
    }
}

struct Turn {
    turn: u32,
    before: GameState,
    orders: Vec<Order>,
    events: Vec<Event>,
}

fn play(
    start: Start,
    others: &BTreeMap<u32, BTreeMap<PlayerId, Vec<Order>>>,
    turns: u32,
) -> Vec<Turn> {
    let mut state = start.state;
    let p = state.setup.players[INDIA.0 as usize].clone();
    let seed = Rng::derive(state.setup.seed, &[purpose::AI, INDIA.0 as u64]).next_u64();
    let mut brain =
        ScriptedBrain::with_difficulty(state.player(INDIA).faction, seed, INDIA, p.difficulty);
    assert_eq!(p.difficulty, AiDifficulty::Hard);
    brain.restore_opponent_memory(start.memory);
    let mut out = Vec::new();
    for _ in 0..turns {
        let turn = state.turn;
        let orders = brain.orders(&player_view(&state, INDIA));
        assert!(
            validate_orders(&state, INDIA, &orders).is_empty(),
            "{orders:?}"
        );
        let mut all: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for (player, recorded) in others.get(&turn).cloned().unwrap_or_default() {
            all.insert(player, legal(&state, player, recorded));
        }
        all.insert(INDIA, orders.clone());
        let (next, events) = resolve_turn(&state, &all);
        out.push(Turn {
            turn,
            before: std::mem::replace(&mut state, next),
            orders,
            events,
        });
    }
    out
}

/// No order, or only a skip or sleep.
fn idle(orders: &[Order], unit: UnitId) -> bool {
    !orders.iter().any(|o| {
        !matches!(o, Order::SkipUnit { .. } | Order::SleepUnit { .. })
            && order_unit(o) == Some(unit)
    })
}

fn attacked(events: &[Event], unit: UnitId) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::Battle { attacker, at, .. } if attacker.unit() == Some(unit) && *at == CITY)
    })
}

fn summary(t: &Turn, after: &GameState) {
    let city = t.before.cities.values().find(|c| c.pos == CITY).unwrap();
    eprintln!("T{} city hp {} owner {:?}", t.turn, city.hp, city.owner);
    for id in MELEE {
        let Some(u) = t.before.units.get(&id) else {
            continue;
        };
        let order = t.orders.iter().find(|o| order_unit(o) == Some(id));
        let end = after.units.get(&id).map(|u| (u.pos, u.hp));
        eprintln!(
            "  u{} {:?} {:?} hp{} -> {:?} attacked {} end {:?}",
            id.0,
            u.kind,
            u.pos,
            u.hp,
            order,
            attacked(&t.events, id),
            end
        );
    }
}

fn log(turns: &[Turn]) {
    for (i, t) in turns.iter().enumerate() {
        let after = turns.get(i + 1).map_or(&t.before, |n| &n.before);
        summary(t, after);
    }
}

#[test]
fn healthy_units_join_the_ring_around_a_fallen_city() {
    let f = fixture();
    assert_eq!(f.t143.state.turn, 143);
    let turns = play(f.t143, &f.orders, 3);
    log(&turns);
    let (t143, after) = (&turns[0], &turns[1].before);
    assert!(
        HEALTHY.iter().any(|id| {
            attacked(&t143.events, *id)
                || after
                    .units
                    .get(id)
                    .is_some_and(|u| u.pos.distance(CITY) <= 1)
        }),
        "a healthy unit must attack the city or close the ring on T143"
    );
    for t in &turns[..2] {
        assert!(
            MELEE
                .iter()
                .filter(|id| t.before.units.get(id).is_some_and(|u| u.owner == INDIA))
                .any(|id| !idle(&t.orders, *id)),
            "T{}: every roster melee unit skipped",
            t.turn
        );
    }
}

#[test]
fn the_cuirassier_joins_the_assault_before_the_reinforcement() {
    let f = fixture();
    assert_eq!(f.t139.state.turn, 139);
    let turns = play(f.t139, &f.orders, 4);
    log(&turns);
    let joined = turns.iter().enumerate().any(|(i, t)| {
        let after = turns.get(i + 1).map(|n| &n.before);
        attacked(&t.events, UnitId(234))
            || after.is_some_and(|s| {
                s.units
                    .get(&UnitId(234))
                    .is_some_and(|u| u.pos.distance(CITY) <= 1)
            })
    });
    assert!(
        joined,
        "Cuirassier 234 never attacked or stood next to the city on T139–T142"
    );
}

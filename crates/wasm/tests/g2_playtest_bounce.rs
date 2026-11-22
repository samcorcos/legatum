//! Playtest G2, from the user's own game (playtest 3, engine 0db3a5f): on turn 6 their Spearman
//! (#3) was ordered onto the adjacent Scout (#14) of AI 5, a team still at Peace. The Scout
//! walked away and the Spearman stood still with a route to the empty tile and no explanation.
use legatum_core::events::PursuitOutcome;
use legatum_core::rules::movement::PEACE_BLOCK_PREFIX;
use legatum_core::{Event, GameState, Hex, Order, PlayerId, TeamId, UnitId, resolve_turn};
use legatum_wasm::tape::Tape;

const SOLDIER: UnitId = UnitId(3);
const SCOUT: UnitId = UnitId(14);

/// The recorded game at the start of turn 6, and that turn's recorded orders.
fn turn_six() -> (GameState, std::collections::BTreeMap<PlayerId, Vec<Order>>) {
    let tape = Tape::from_json(include_str!("fixtures/playtest3-g2.tape.json")).unwrap();
    // Keep the recorded turn's map and units: map generation has since changed.
    let state = legatum_wasm::json::parse_state_for_rehash(include_str!(
        "fixtures/playtest3-g2-turn6.current.json"
    ))
    .unwrap();
    assert_eq!(state.turn, 6);
    assert_eq!(state.units[&SOLDIER].pos, Hex { q: 0, r: 14 });
    assert_eq!(state.units[&SCOUT].pos, Hex { q: 1, r: 13 });
    assert!(
        !state.at_war(PlayerId(0), PlayerId(4)),
        "the Scout's team was at Peace"
    );
    (state, tape.turns[5].orders.clone())
}

#[test]
fn the_recorded_order_is_reported_as_blocked_by_a_team_at_peace() {
    let (state, orders) = turn_six();
    assert!(orders[&PlayerId(0)].contains(&Order::MoveUnit {
        unit: SOLDIER,
        to: Hex { q: 1, r: 13 }
    }));
    let (next, events) = resolve_turn(&state, &orders);
    let reason = format!("{PEACE_BLOCK_PREFIX}AI 5's Scout");
    assert!(
        events.iter().any(|e| matches!(e,
            Event::MoveBlocked { unit, reason: r, .. } if *unit == SOLDIER && *r == reason)),
        "the report names who is there: {events:?}"
    );
    assert!(!events.iter().any(|e| matches!(e, Event::Battle { .. })));
    assert_eq!(next.units[&SOLDIER].attack_target, None);
}

#[test]
fn declaring_war_with_the_attack_never_bounces_silently() {
    let (state, mut orders) = turn_six();
    orders.get_mut(&PlayerId(0)).unwrap().insert(
        0,
        Order::DeclareWar {
            against: TeamId(4),
            breach: false,
        },
    );
    let (next, events) = resolve_turn(&state, &orders);
    assert!(next.at_war(PlayerId(0), PlayerId(4)));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, .. } if *unit == SOLDIER)),
        "{events:?}"
    );
    // G20: plain Peace, so War starts this turn and the Spearman attacks the Scout now.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::WarDeclared { relation } if relation.effective_turn == 6))
    );
    assert!(!events.iter().any(|e| matches!(e,
        Event::AttackPursuit { unit, outcome: PursuitOutcome::AwaitingWar, .. } if *unit == SOLDIER)));
    assert!(
        events.iter().any(|e| matches!(e,
            Event::Battle { attacker, defender: legatum_core::events::Defender::Unit { unit, .. }, .. }
                if *attacker == SOLDIER && *unit == SCOUT)),
        "the Soldier attacked the Scout on turn 6: {events:?}"
    );
}

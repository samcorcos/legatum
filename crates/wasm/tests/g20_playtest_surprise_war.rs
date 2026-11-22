//! Playtest G20, from the user's own game (playtest 3, engine 29a5b2a): on turn 102 they
//! declared War on AI 5 (team 4, plain Peace, no treaty) and ordered their Soldier (#54) onto
//! AI 5's Settler (#83). The declaration took effect on turn 103, so the Soldier held while the
//! Settler walked away, which read as a bounce; the Soldier caught it on turn 103. The fixture
//! is the recorded state at the start of turn 102 (replayed from the save's tape with the
//! playtest-3 engine) and the recorded orders of turns 102 and 103.
use legatum_core::events::PursuitOutcome;
use legatum_core::{Event, GameState, Hex, Order, PlayerId, TeamId, UnitId, resolve_turn};
use legatum_wasm::tape::TapeTurn;

const SOLDIER: UnitId = UnitId(54);
const SETTLER: UnitId = UnitId(83);

fn fixture() -> (GameState, Vec<TapeTurn>) {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/playtest3-g20-turn102.json")).unwrap();
    let state = legatum_wasm::json::parse_state_for_rehash(&v["state"].to_string()).unwrap();
    let turns: Vec<TapeTurn> = serde_json::from_value(v["turns"].clone()).unwrap();
    (state, turns)
}

#[test]
fn declaring_war_at_plain_peace_captures_the_settler_on_the_same_turn() {
    let (state, turns) = fixture();
    assert_eq!(state.turn, 102);
    assert_eq!(state.units[&SOLDIER].pos, Hex { q: 27, r: 18 });
    assert_eq!(state.units[&SETTLER].pos, Hex { q: 27, r: 17 });
    assert!(!state.at_war(PlayerId(0), PlayerId(4)), "plain Peace");
    let orders = &turns[0].orders[&PlayerId(0)];
    assert!(orders.contains(&Order::DeclareWar {
        against: TeamId(4),
        breach: false
    }));
    assert!(orders.contains(&Order::MoveUnit {
        unit: SOLDIER,
        to: Hex { q: 27, r: 17 }
    }));
    let (next, events) = resolve_turn(&state, &turns[0].orders);
    let declared = events
        .iter()
        .position(
            |e| matches!(e, Event::WarDeclared { relation } if relation.effective_turn == 102),
        )
        .expect("War starts on turn 102");
    let captured = events
        .iter()
        .position(|e| {
            matches!(e, Event::SettlerCaptured { unit, to, .. }
                if *unit == SETTLER && *to == PlayerId(0))
        })
        .unwrap_or_else(|| panic!("captured on turn 102: {events:?}"));
    assert!(
        declared < captured,
        "the report reads: declared, then captured"
    );
    assert!(!events.iter().any(|e| matches!(e,
        Event::AttackPursuit { unit, outcome: PursuitOutcome::AwaitingWar, .. } if *unit == SOLDIER)));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::MoveBlocked { unit, .. } if *unit == SOLDIER))
    );
    assert!(next.at_war(PlayerId(0), PlayerId(4)));
    assert_eq!(next.units[&SETTLER].owner, PlayerId(0));
}

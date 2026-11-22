//! Co-op Gold gifts (`Order::GiveGold`, rules 50): an empire gives Gold to a living teammate.
//!
//! A gift is checked against the giver's Gold at the start of the turn (all its gifts together).
//! It moves after every empire's city orders and actions, so the giver's own purchases this turn
//! are paid first and the gift is capped at the Gold left then; the teammate can spend it from
//! next turn. Gifts resolve in player-id order, then in each giver's order list. The AI never
//! gives Gold (for now).
use crate::events::Event;
use crate::orders::{Order, OrderError};
use crate::state::GameState;
use crate::types::PlayerId;
use std::collections::BTreeMap;

/// Why `player` cannot give `amount` Gold to `to`, or `None` when it can (on its own; the total of
/// a turn's gifts is checked in `validate_gifts`).
pub fn gift_blocker(
    state: &GameState,
    player: PlayerId,
    to: PlayerId,
    amount: i32,
) -> Option<String> {
    if amount < 1 {
        return Some("a gift must be at least 1 Gold".into());
    }
    if to == player {
        return Some("you cannot give Gold to yourself".into());
    }
    let Some(target) = state.players.get(to.0 as usize) else {
        return Some("no such empire".into());
    };
    if !target.alive {
        return Some(format!("{} has been eliminated", target.name));
    }
    if to.is_barbarian() || state.team_of(to) != state.team_of(player) {
        return Some(format!("{} is not on your team", target.name));
    }
    None
}

pub fn validate_gifts(state: &GameState, player: PlayerId, orders: &[Order]) -> Vec<OrderError> {
    let mut errors = Vec::new();
    let mut left = state.player(player).gold;
    for (index, order) in orders.iter().enumerate() {
        let Order::GiveGold { to, amount } = order else {
            continue;
        };
        if let Some(message) = gift_blocker(state, player, *to, *amount) {
            errors.push(OrderError { index, message });
            continue;
        }
        if *amount > left {
            errors.push(OrderError {
                index,
                message: format!("you have {} Gold to give", left.max(0)),
            });
            continue;
        }
        left -= amount;
    }
    errors
}

/// Move every valid gift, after the turn's purchases. An invalid gift is skipped with an
/// `OrderFailed` event; a gift larger than the Gold left gives what is left.
pub fn apply_gifts(
    state: &mut GameState,
    orders: &BTreeMap<PlayerId, &[Order]>,
    events: &mut Vec<Event>,
) {
    for (player, os) in orders {
        for order in os.iter() {
            let Order::GiveGold { to, amount } = order else {
                continue;
            };
            if let Some(reason) = gift_blocker(state, *player, *to, *amount) {
                events.push(Event::OrderFailed {
                    player: *player,
                    reason: format!("Gold gift not given: {reason}"),
                });
                continue;
            }
            let given = (*amount).min(state.player(*player).gold.max(0));
            if given == 0 {
                events.push(Event::OrderFailed {
                    player: *player,
                    reason: "Gold gift not given: no Gold left after this turn's purchases".into(),
                });
                continue;
            }
            state.player_mut(*player).gold -= given;
            state.player_mut(*to).gold += given;
            events.push(Event::GoldGiven {
                from: *player,
                to: *to,
                amount: given,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameSetup, new_game, resolve_turn};

    /// Four empires: 0 and 1 on one team, 2 and 3 on others.
    fn game() -> GameState {
        let mut setup = GameSetup::all_ai(4);
        setup.players[1].team = setup.players[0].team;
        let mut state = new_game(&setup);
        for p in &mut state.players {
            p.gold = 100;
        }
        assert_eq!(state.team_of(PlayerId(0)), state.team_of(PlayerId(1)));
        assert_ne!(state.team_of(PlayerId(0)), state.team_of(PlayerId(2)));
        state
    }

    fn give(to: u8, amount: i32) -> Order {
        Order::GiveGold {
            to: PlayerId(to),
            amount,
        }
    }

    #[test]
    fn a_gift_moves_gold_to_a_teammate_at_the_turn_boundary() {
        let state = game();
        assert!(crate::validate_orders(&state, PlayerId(0), &[give(1, 40)]).is_empty());
        let (after, events) =
            resolve_turn(&state, &BTreeMap::from([(PlayerId(0), vec![give(1, 40)])]));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::GoldGiven {
                from: PlayerId(0),
                to: PlayerId(1),
                amount: 40
            }
        )));
        // Both treasuries also move by their ordinary income; the gift is the difference.
        let (control, _) = resolve_turn(&state, &BTreeMap::new());
        assert_eq!(
            after.player(PlayerId(0)).gold,
            control.player(PlayerId(0)).gold - 40
        );
        assert_eq!(
            after.player(PlayerId(1)).gold,
            control.player(PlayerId(1)).gold + 40
        );
    }

    #[test]
    fn only_a_living_teammate_can_receive_and_never_more_than_you_have() {
        let state = game();
        let errors = |orders: &[Order]| crate::validate_orders(&state, PlayerId(0), orders);
        assert!(
            errors(&[give(2, 10)])[0]
                .message
                .contains("not on your team")
        );
        assert!(errors(&[give(0, 10)])[0].message.contains("yourself"));
        assert!(errors(&[give(1, 0)])[0].message.contains("at least 1"));
        assert!(
            errors(&[give(1, 101)])[0]
                .message
                .contains("100 Gold to give")
        );
        // Two gifts together may not exceed the treasury either.
        let two = errors(&[give(1, 60), give(1, 60)]);
        assert_eq!(two.len(), 1);
        assert_eq!(two[0].index, 1);
        // At resolution a gift to a rival is skipped, never applied.
        let (after, events) =
            resolve_turn(&state, &BTreeMap::from([(PlayerId(0), vec![give(2, 10)])]));
        assert!(!events.iter().any(|e| matches!(e, Event::GoldGiven { .. })));
        let (control, _) = resolve_turn(&state, &BTreeMap::new());
        assert_eq!(
            after.player(PlayerId(2)).gold,
            control.player(PlayerId(2)).gold
        );
    }

    #[test]
    fn a_gift_is_capped_at_the_gold_left_after_purchases() {
        let mut state = game();
        state.player_mut(PlayerId(0)).gold = 30;
        // Spend the treasury first by hand, as a purchase would, then resolve the gift alone.
        let orders = vec![give(1, 30)];
        let live: BTreeMap<PlayerId, &[Order]> = BTreeMap::from([(PlayerId(0), orders.as_slice())]);
        state.player_mut(PlayerId(0)).gold = 12;
        let mut events = Vec::new();
        apply_gifts(&mut state, &live, &mut events);
        assert_eq!(state.player(PlayerId(0)).gold, 0);
        assert_eq!(state.player(PlayerId(1)).gold, 112);
        assert!(matches!(events[..], [Event::GoldGiven { amount: 12, .. }]));
    }
}

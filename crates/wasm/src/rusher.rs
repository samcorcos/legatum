//! A scripted rusher for scenarios: one AI empire plays the way a skilled human rushes, because the
//! Hard AI does not (it waits for a siege it can forecast). From the scenario's data alone:
//! - it declares war on the listed empires from the listed turns (an ordinary `DeclareWar` order,
//!   retried each turn until the war is on or the rules refuse it), and never makes peace with them;
//! - its capital trains Spearmen (Hoplites for Sparta) without a break;
//! - every military unit but Scouts marches on the nearest city of an empire it is at war with
//!   (or of `target`), waits within two tiles of it, and attacks together once `storm` units
//!   stand that close.
//!
//! Everything else the empire does is the normal AI's. The rusher keeps no memory, so a restored
//! game plays on exactly as before. Used by the app and by the Hard AI baseline.

use crate::session::OrderMap;
use legatum_core::diplomacy::TreatyKind;
use legatum_core::{
    GameState, Order, PlayerId, ProductionItem, UnitKind, player_view, rules::is_military,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RusherWar {
    pub against: PlayerId,
    /// The first turn the declaration is ordered.
    pub turn: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rusher {
    /// The empire that rushes (an AI; never the human's seat).
    pub player: PlayerId,
    pub wars: Vec<RusherWar>,
    /// Units that must stand within two tiles of the target before the army attacks.
    pub storm: u32,
    /// March on this empire's nearest city; by default on the nearest city of any empire at war.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<PlayerId>,
}

/// Overwrite the rusher's orders for this planning turn.
pub fn apply(orders: &mut OrderMap, state: &GameState, rusher: &Rusher) {
    let me = rusher.player;
    if !state.player(me).alive || state.player(me).is_human {
        return;
    }
    let mut mine = orders.remove(&me).unwrap_or_default();

    // Never make peace with the empires it fights.
    let enemies: BTreeSet<_> = rusher
        .wars
        .iter()
        .map(|w| state.team_of(w.against))
        .collect();
    mine.retain(|o| !matches!(o, Order::ProposeTreaty { to, kind: TreatyKind::Peace } if enemies.contains(to)));
    if let Some(d) = state.diplomacy.as_ref() {
        for o in mine.iter_mut() {
            if let Order::RespondToProposal {
                proposal, accept, ..
            } = o
                && *accept
                && d.proposals.iter().any(|p| {
                    p.id == *proposal && enemies.contains(&p.from) && p.kind == TreatyKind::Peace
                })
            {
                *accept = false;
            }
        }
    }

    // Declare the scripted wars.
    let view = player_view(state, me);
    for w in &rusher.wars {
        if state.turn < w.turn || !state.player(w.against).alive || state.at_war(me, w.against) {
            continue;
        }
        let target = state.team_of(w.against);
        if let Some(a) = view.diplomacy.actions.iter().find(|a| {
            matches!(a.order, Order::DeclareWar { against, breach: false } if against == target)
                && a.blocked_reason.is_none()
        }) {
            mine.push(a.order.clone());
        }
    }

    // Train Spearmen in the capital, and march.
    let home = state
        .cities_of(me)
        .find(|c| c.is_capital)
        .or_else(|| state.cities_of(me).next())
        .map(|c| (c.id, c.pos, c.producing));
    if let Some((city, _, producing)) = &home {
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        mine.retain(|o| {
            !matches!(o, Order::SetProduction { city: c, .. } | Order::SetProductionQueue { city: c, .. } | Order::BuyItem { city: c, .. } if c == city)
        });
        if producing.as_ref() != Some(&soldier) {
            mine.push(Order::SetProduction {
                city: *city,
                item: soldier,
            });
        }
    }
    let target = home.as_ref().and_then(|(_, home_pos, _)| {
        state
            .cities
            .values()
            .filter(|c| c.owner != me && !c.owner.is_barbarian() && state.at_war(me, c.owner))
            .filter(|c| rusher.target.is_none_or(|t| c.owner == t))
            .min_by_key(|c| (c.pos.distance(*home_pos), c.population, c.id.0))
            .map(|c| c.pos)
    });
    if let Some(target) = target {
        let army: Vec<_> = state
            .units
            .values()
            .filter(|u| u.owner == me && is_military(u.kind) && u.kind != UnitKind::Scout)
            .map(|u| (u.id, u.pos))
            .collect();
        let ids: BTreeSet<_> = army.iter().map(|(id, _)| *id).collect();
        mine.retain(|o| !touches(o, &ids));
        let near = army.iter().filter(|(_, p)| p.distance(target) <= 2).count() as u32;
        for (id, pos) in army {
            if near >= rusher.storm || pos.distance(target) > 2 {
                mine.push(Order::MoveUnit {
                    unit: id,
                    to: target,
                });
            } else {
                mine.push(Order::SkipUnit { unit: id });
            }
        }
    }
    orders.insert(me, mine);
}

/// Whether an order commands one of these units.
fn touches(order: &Order, units: &BTreeSet<legatum_core::UnitId>) -> bool {
    let unit = match order {
        Order::SetAutomation { unit, .. }
        | Order::SetAutoFire { unit, .. }
        | Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SleepUnit { unit, .. }
        | Order::SkipUnit { unit }
        | Order::FoundCity { unit }
        | Order::DisbandUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit } => unit,
        _ => return false,
    };
    units.contains(unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{BrainKind, Session};
    use legatum_core::{GameSetup, new_game};

    #[test]
    fn the_rusher_trains_soldiers_and_leaves_other_empires_alone() {
        let mut session =
            Session::from_state(new_game(&GameSetup::all_ai(3)), |_| BrainKind::Scripted);
        for _ in 0..6 {
            let orders = session.ai_orders();
            session.step(&orders);
        }
        let mut orders = session.ai_orders();
        let before = orders.clone();
        let rusher = Rusher {
            player: PlayerId(1),
            wars: vec![],
            storm: 3,
            target: None,
        };
        apply(&mut orders, &session.state, &rusher);
        let capital = session
            .state
            .cities_of(PlayerId(1))
            .find(|c| c.is_capital)
            .expect("a capital by turn 7");
        let soldier = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        let sets_soldiers = orders[&PlayerId(1)]
            .iter()
            .any(|o| matches!(o, Order::SetProduction { city, item } if *city == capital.id && *item == soldier));
        assert!(sets_soldiers || capital.producing.as_ref() == Some(&soldier));
        assert_eq!(orders[&PlayerId(0)], before[&PlayerId(0)]);
        assert_eq!(orders[&PlayerId(2)], before[&PlayerId(2)]);
    }

    #[test]
    fn a_rusher_is_never_a_human_seat_or_a_dead_empire() {
        let mut state = new_game(&GameSetup::all_ai(3));
        state.players[1].is_human = true;
        let mut orders = OrderMap::new();
        orders.insert(PlayerId(1), vec![]);
        let rusher = Rusher {
            player: PlayerId(1),
            wars: vec![],
            storm: 3,
            target: None,
        };
        apply(&mut orders, &state, &rusher);
        assert!(orders[&PlayerId(1)].is_empty());
    }
}

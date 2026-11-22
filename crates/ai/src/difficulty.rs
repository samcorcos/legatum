//! AI difficulty: the same scripted AI deciding worse or better. No resource or rule changes
//! (pillar 6): Easy plays timid and inattentive, Hard reinforces and plans scoring investments earlier.

use crate::knobs::Knobs;
use legatum_core::rng::{Rng, purpose};
use legatum_core::{AiDifficulty, Order, PlayerView, UnitId};

/// Chance (percent) that an Easy AI lets any one planned order lapse this turn.
pub const EASY_LAPSE_PERCENT: u32 = 20;

/// Jitter stream for lapses, distinct from the planner's streams.
const LAPSE_STREAM: u64 = 0x1a95e;

/// Scale a faction's knobs for a difficulty. Normal leaves them unchanged.
pub fn scale_knobs(k: Knobs, d: AiDifficulty) -> Knobs {
    match d {
        AiDifficulty::Normal => k,
        // Smaller armies, attacks only with a big edge (higher threshold), fewer cities.
        AiDifficulty::Easy => Knobs::new(
            k.expansion - 15,
            k.military - 25,
            k.science,
            k.economy,
            k.aggression - 35,
        ),
        // Reinforce the opening sooner without distorting the faction's investment route
        // or accepting worse attacks. Hard's later budget/scoring decisions live in Planner.
        AiDifficulty::Hard => Knobs::new(
            k.expansion,
            k.military + 20,
            k.science,
            k.economy,
            k.aggression,
        ),
    }
    .clamped()
}

/// Easy AIs miss some of their best moves: each planned order lapses with a seeded chance.
/// Deterministic given (seed, turn, player, order index).
pub fn lapse(
    orders: Vec<Order>,
    d: AiDifficulty,
    seed: u64,
    view: &PlayerView,
    assault_groups: &[Vec<UnitId>],
) -> Vec<Order> {
    if d != AiDifficulty::Easy {
        return orders;
    }
    // Purchases name their item and never depend on a same-turn SetProduction.
    let mut kept: Vec<_> = orders
        .into_iter()
        .enumerate()
        .filter(|(i, o)| {
            // Inattention never leaves the research plan empty: an idle laboratory wastes
            // the turn after an era-up, however distracted the empire is.
            if matches!(
                o,
                Order::SetResearch { .. } | Order::SetResearchQueue { .. }
            ) && view.research.current.is_none()
            {
                return true;
            }
            let mut r = Rng::derive(
                seed,
                &[
                    view.turn as u64,
                    purpose::AI,
                    view.me.0 as u64,
                    LAPSE_STREAM,
                    *i as u64,
                ],
            );
            !r.chance(EASY_LAPSE_PERCENT)
        })
        .map(|(_, o)| o)
        .collect();
    // An exposed staging move depends on its partners arriving. Explicit holds also
    // clear any old persistent destination when Easy forgets one of those moves.
    for group in assault_groups {
        if group.iter().all(|id| {
            kept.iter()
                .any(|o| matches!(o, Order::MoveUnit { unit, .. } if unit == id))
        }) {
            continue;
        }
        for &id in group {
            let hold = Order::SkipUnit { unit: id };
            if let Some(order) = kept
                .iter_mut()
                .find(|o| legatum_core::rules::units::order_unit(o) == Some(id))
            {
                *order = hold;
            } else {
                kept.push(hold);
            }
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::Faction;

    #[test]
    fn easy_never_lapses_the_research_plan_while_research_is_empty() {
        use crate::fixture::ViewBuilder;
        let view = ViewBuilder::new(16, 16).build();
        assert!(view.research.current.is_none());
        let queue = Order::SetResearchQueue {
            techs: vec![legatum_core::TechId::Irrigation],
        };
        for seed in 0..200 {
            let kept = lapse(vec![queue.clone()], AiDifficulty::Easy, seed, &view, &[]);
            assert_eq!(kept, vec![queue.clone()], "seed {seed}");
        }
    }

    #[test]
    fn easy_lapses_purchases_independently_of_construction() {
        use crate::fixture::{ME, ViewBuilder, off};
        use legatum_core::{CityId, ProductionItem, UnitKind};
        let view = ViewBuilder::new(16, 16).city(1, ME, off(4, 4), 3).build();
        let select = Order::SetProduction {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        };
        let buy = Order::BuyItem {
            city: CityId(1),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        };
        let mut purchases = 0;
        let mut lapses = 0;
        for seed in 0..100 {
            let kept = lapse(
                vec![select.clone(), buy.clone()],
                AiDifficulty::Easy,
                seed,
                &view,
                &[],
            );
            if kept.contains(&buy) {
                purchases += 1;
                assert!(kept.iter().any(|o| o == &buy));
            } else {
                lapses += 1;
            }
        }
        assert!(purchases > 0 && lapses > 0);
    }

    #[test]
    fn easy_cancels_exposed_staging_together_when_a_partner_forgets() {
        use crate::fixture::{ME, ViewBuilder};
        use legatum_core::{Hex, UnitKind};
        let view = ViewBuilder::new(24, 16)
            .unit(1, ME, UnitKind::Spearman, Hex::new(5, 5))
            .unit(2, ME, UnitKind::Spearman, Hex::new(5, 6))
            .goto(1, Hex::new(8, 5))
            .goto(2, Hex::new(8, 6))
            .build();
        let moves = vec![
            Order::MoveUnit {
                unit: UnitId(1),
                to: Hex::new(7, 5),
            },
            Order::MoveUnit {
                unit: UnitId(2),
                to: Hex::new(7, 6),
            },
        ];
        let holds = [
            Order::SkipUnit { unit: UnitId(1) },
            Order::SkipUnit { unit: UnitId(2) },
        ];
        let group = vec![UnitId(1), UnitId(2)];
        let mut arrivals = 0;
        let mut cancellations = 0;
        for seed in 0..100 {
            let orders = lapse(
                moves.clone(),
                AiDifficulty::Easy,
                seed,
                &view,
                std::slice::from_ref(&group),
            );
            if orders == moves {
                arrivals += 1;
            } else {
                cancellations += 1;
                assert_eq!(orders.len(), holds.len());
                assert!(holds.iter().all(|hold| orders.contains(hold)), "{orders:?}");
            }
        }
        assert!(arrivals > 0 && cancellations > 0);
    }

    #[test]
    fn normal_is_unchanged_and_order_is_easy_normal_hard() {
        for f in Faction::ALL {
            let k = Knobs::for_faction(f);
            assert_eq!(scale_knobs(k, AiDifficulty::Normal), k);
            let easy = scale_knobs(k, AiDifficulty::Easy);
            let hard = scale_knobs(k, AiDifficulty::Hard);
            assert!(easy.military < k.military);
            assert_eq!(hard.military, (k.military + 20).min(100));
            if k.military < 100 {
                assert!(k.military < hard.military);
            }
            assert!(easy.attack_threshold() >= k.attack_threshold());
            if k.attack_threshold() < 130 {
                assert!(easy.attack_threshold() > k.attack_threshold());
            }
            assert_eq!(hard.attack_threshold(), k.attack_threshold());
            assert!(easy.target_cities() <= k.target_cities());
            assert!(hard.target_cities() >= k.target_cities());
            assert_eq!(easy, easy.clamped());
            assert_eq!(hard, hard.clamped());
        }
    }
}

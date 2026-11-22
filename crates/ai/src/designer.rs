//! The designer imitation seat: an opponent that plays the designer's recurring plays, for
//! measurement only (the scenario gate's opponent column, `crates/sim/examples/scenario_gate.rs`,
//! and `designer_predict`). No app game, difficulty or default uses it.
//!
//! It is Hard with a fixed gene set (`designer.json`) taken from his recorded games, as written up
//! in `docs/playtests/human-reasoning.md` and the coach playbook (an early strike on an open
//! neighbour, follow-up captures city by city, paying through Walls for a prize city, Settler
//! snipes, focus fire, commitment to a war he started, the decisive purchase, frontier Walls, hard
//! expansion while unthreatened), plus two order-level habits his tapes show and no gene expresses:
//!
//! - **He hardly buys, and spends the reserve when attacked.** Hard in his seat makes 80 to 396
//!   purchases a game, he makes 0 to 7 (decision audit 2026-10-31): the decisive aircraft, and
//!   "cash saved up to buy walls and a ranger if I get surprise attacked" (human-reasoning,
//!   2026-10-07). Hard's purchases are kept only for aircraft and for a city with an enemy army at
//!   war within three hexes, so the Gold builds up as his does. (v1 dropped the emergency buys too,
//!   and Hard beat it more easily than Hard in the gate's rival cards: 2026-11-09 README.)
//! - **Walls first.** Past the opening (turn 25) a city of his without Walls builds them next
//!   (Aztecs 8.9.29: 10 of 11 cities walled; most of his production orders are Walls).
//!
//! Validation and its limits: `docs/sim-results/scenario-gate/README.md`.

use crate::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::{
    AiDifficulty, BuildingKind, CityId, Faction, Order, PlayerId, PlayerView, ProductionItem,
};
use std::collections::BTreeSet;
use std::sync::OnceLock;

/// The first turn of the Walls-first habit (his opening builds units and Settlers).
const WALLS_FROM_TURN: u32 = 25;

/// The imitation seat's genes.
pub fn params() -> &'static AiParams {
    static PARAMS: OnceLock<AiParams> = OnceLock::new();
    PARAMS.get_or_init(|| {
        let params: AiParams =
            serde_json::from_str(include_str!("designer.json")).expect("designer.json parses");
        params.validate().expect("designer.json genes in bounds");
        params
    })
}

/// A brain that plays the designer's seat.
pub fn brain(faction: Faction, seed: u64, player: PlayerId) -> DesignerBrain {
    DesignerBrain {
        inner: ScriptedBrain::with_params(faction, seed, player, AiDifficulty::Hard, params()),
    }
}

pub struct DesignerBrain {
    inner: ScriptedBrain,
}

impl DesignerBrain {
    /// His habits applied to Hard's orders for this turn.
    fn habits(view: &PlayerView, orders: &mut Vec<Order>) {
        drop_purchases(orders, &endangered(view));
        if view.turn < WALLS_FROM_TURN {
            return;
        }
        let walls = ProductionItem::Building {
            building: BuildingKind::Walls,
        };
        for city in view
            .cities
            .iter()
            .filter(|c| c.owner == view.me && !c.walls)
        {
            let Some(d) = city.detail.as_ref() else {
                continue;
            };
            if d.producing == Some(walls) || !d.options.iter().any(|o| o.item == walls) {
                continue;
            }
            // Keep what Hard wanted after the Walls.
            let mut queue = vec![walls];
            orders.retain(|o| match o {
                Order::SetProduction { city: c, item } if *c == city.id => {
                    queue.push(*item);
                    false
                }
                Order::SetProductionQueue { city: c, queue: q } if *c == city.id => {
                    queue.extend(q.iter().copied().filter(|i| *i != walls));
                    false
                }
                _ => true,
            });
            if queue.len() == 1 {
                queue.extend(d.producing.filter(|i| *i != walls));
            }
            orders.push(Order::SetProductionQueue {
                city: city.id,
                queue,
            });
        }
    }
}

/// Own cities with a visible military unit of an empire at war with us within this many hexes.
const DANGER_RADIUS: i32 = 3;

/// His cities in danger: where he spends his reserve.
fn endangered(view: &PlayerView) -> BTreeSet<CityId> {
    view.cities
        .iter()
        .filter(|c| c.owner == view.me)
        .filter(|c| {
            view.units.iter().any(|u| {
                u.owner != view.me
                    && view.at_war(view.me, u.owner)
                    && legatum_core::rules::is_military(u.kind)
                    && u.pos.distance(c.pos) <= DANGER_RADIUS
            })
        })
        .map(|c| c.id)
        .collect()
}

/// Drop every purchase but an aircraft's and those for a city in danger.
fn drop_purchases(orders: &mut Vec<Order>, endangered: &BTreeSet<CityId>) {
    orders.retain(|o| match o {
        Order::BuyItem { city, .. } if endangered.contains(city) => true,
        Order::BuyItem {
            item: ProductionItem::Unit { unit },
            ..
        } => legatum_core::data::is_aircraft(*unit),
        Order::BuyItem { .. } | Order::HireCompany { .. } => false,
        _ => true,
    });
}

impl Brain for DesignerBrain {
    fn orders(&mut self, view: &PlayerView) -> Vec<Order> {
        let mut orders = self.inner.orders(view);
        Self::habits(view, &mut orders);
        orders
    }
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        let mut orders = self.inner.plan(view);
        Self::habits(view, &mut orders);
        orders
    }
    fn add_air_orders(&self, view: &PlayerView, orders: &mut Vec<Order>) {
        self.inner.add_air_orders(view, orders);
    }
    fn opponent_memory(&self) -> Option<OpponentMemory> {
        self.inner.opponent_memory()
    }
    fn restore_opponent_memory(&mut self, memory: OpponentMemory) {
        self.inner.restore_opponent_memory(memory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genes_parse_and_differ_from_hard() {
        assert_ne!(params().decisions, crate::DecisionParams::default());
    }

    #[test]
    fn drops_purchases_but_aircraft() {
        let city = legatum_core::CityId(1);
        let mut orders = vec![
            Order::BuyItem {
                city,
                item: ProductionItem::Unit {
                    unit: legatum_core::UnitKind::Spearman,
                },
            },
            Order::BuyItem {
                city,
                item: ProductionItem::Unit {
                    unit: legatum_core::UnitKind::Bomber,
                },
            },
            Order::BuyItem {
                city,
                item: ProductionItem::Building {
                    building: BuildingKind::Walls,
                },
            },
        ];
        drop_purchases(&mut orders, &BTreeSet::new());
        assert_eq!(orders.len(), 1);
        assert!(matches!(
            orders[0],
            Order::BuyItem {
                item: ProductionItem::Unit {
                    unit: legatum_core::UnitKind::Bomber
                },
                ..
            }
        ));
    }
}

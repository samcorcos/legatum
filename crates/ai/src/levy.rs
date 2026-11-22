//! Thin levies (Byzantium): the cap on armed units is public (`PlayerPublic::levy`), and the
//! core blocks new armed units at it (`economy::production_check`: "Levies full: N of M").
//!
//! **The Byzantium AI** never plans an army larger than the cap (`Planner::desired_army`), so
//! Production goes to buildings, Settlers and the Heir instead of units the core would refuse.
//! A city whose current item became an armed unit at the cap is moved to something else
//! (banked progress is kept, so nothing is wasted). Gold upgrades fill the slots with better
//! units (`hold_or_upgrade`). In peace, with nothing threatening, it disbands one unit that has
//! no upgrade offer to free a slot for a unit it can already train and that is clearly stronger
//! (`plan_levy_turnover`); it never does so at war or under threat.
//!
//! **Rivals** treat a Byzantium's cap as the public number it is: a campaign against a Byzantium
//! whose armed units our own army outnumbers beyond its cap gains capture odds
//! (`levy_edge_odds`), because it cannot answer a bigger force by training more.

use crate::scripted::Planner;
use legatum_core::data;
use legatum_core::{Faction, Order, ProductionItem, TeamId, UnitKind};

/// A trainable unit must be at least this much stronger (percent of the old one's best strength)
/// before the AI gives up a slot to train it.
const CLEARLY_BETTER_PERCENT: i32 = 150;
/// Capture odds (percentage points) a rival gains for each armed unit it fields beyond a
/// Byzantium's cap, at most `EDGE_UNITS_MAX` units.
const EDGE_ODDS_PER_UNIT: i32 = 4;
const EDGE_UNITS_MAX: i32 = 4;

fn best_strength(faction: Faction, kind: UnitKind) -> i32 {
    let s = data::unit_stats_for(faction, kind);
    s.strength.max(s.ranged)
}

impl Planner<'_> {
    /// This seat's armed units and their cap, if its faction has one.
    pub(crate) fn levy_now(&self) -> Option<(i32, i32)> {
        let v = self.view();
        v.players
            .iter()
            .find(|p| p.id == v.me)
            .and_then(|p| p.levy)
            .map(|l| (l.units as i32, l.cap as i32))
    }

    /// The core would refuse to continue or finish `item` for want of a slot.
    pub(crate) fn levy_blocks(&self, item: Option<ProductionItem>) -> bool {
        matches!(item, Some(ProductionItem::Unit { unit }) if data::pays_upkeep(unit))
            && self.levy_now().is_some_and(|(units, cap)| units >= cap)
    }

    /// Capture-odds bonus (percentage points) against a Byzantium `owner` whose public cap our
    /// army outnumbers.
    pub(crate) fn levy_edge_odds(&self, owner: TeamId) -> i32 {
        let v = self.view();
        let Some(cap) = v
            .players
            .iter()
            .filter(|p| p.alive && p.team == owner)
            .filter_map(|p| p.levy)
            .map(|l| l.cap as i32)
            .reduce(|a, b| a + b)
        else {
            return 0;
        };
        let ours = self
            .ctx
            .own_units
            .iter()
            .filter(|u| data::pays_upkeep(u.kind))
            .count() as i32;
        (ours - cap).clamp(0, EDGE_UNITS_MAX) * EDGE_ODDS_PER_UNIT
    }

    /// Final guard on this turn's orders: armed units finishing this turn and those bought
    /// together never exceed the cap (the core would refuse the later ones).
    pub(crate) fn enforce_levy(&mut self) {
        let Some((units, cap)) = self.levy_now() else {
            return;
        };
        let completing = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| {
                !self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == c.id))
            })
            .filter_map(|c| c.detail.as_ref())
            .filter(|d| {
                d.resistance <= 0
                    && d.turns_to_complete == Some(1)
                    && matches!(d.producing, Some(ProductionItem::Unit { unit })
                        if data::pays_upkeep(unit))
            })
            .count() as i32;
        let mut room = cap - units - completing;
        self.orders.retain(|o| match o {
            Order::BuyItem {
                item: ProductionItem::Unit { unit },
                ..
            } if data::pays_upkeep(*unit) => {
                room -= 1;
                room >= 0
            }
            _ => true,
        });
        // A unit bought this turn fills its slot at once: no armed unit may be started after
        // the last slot is gone (the core refuses "Levies full" when the order is validated).
        if room <= 0 {
            self.orders.retain(|o| {
                !matches!(o, Order::SetProduction {
                    item: ProductionItem::Unit { unit },
                    ..
                } if data::pays_upkeep(*unit))
            });
        }
    }

    /// At the cap, in peace and with nothing threatening: disband the weakest unit that has no
    /// upgrade offer, when a clearly stronger unit it can train is waiting behind the cap.
    pub(crate) fn plan_levy_turnover(&mut self) {
        let v = self.view();
        if self.levy_now().is_none_or(|(units, cap)| units < cap)
            || v.players
                .iter()
                .any(|p| p.alive && p.team != v.team && v.teams_at_war(v.team, p.team))
            || self.ctx.threat.iter().any(|t| *t > 0)
            || !self.ctx.threat_units.is_empty() && self.endangered()
        {
            return;
        }
        let waiting = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref())
            .flat_map(|d| &d.unavailable)
            .filter(|o| o.reason.starts_with("Levies full"))
            .filter_map(|o| match o.item {
                ProductionItem::Unit { unit } => Some(unit),
                _ => None,
            })
            .map(|unit| best_strength(self.faction, unit))
            .max();
        let Some(waiting) = waiting else { return };
        let Some(weak) = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| {
                data::pays_upkeep(u.kind)
                    && u.contract_until.is_none()
                    && u.upgrade_cost.is_none()
                    && !self.has_acted(u.id)
            })
            .filter(|u| {
                waiting * 100 >= best_strength(self.faction, u.kind) * CLEARLY_BETTER_PERCENT
                    && self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.faction, u.kind)
                        .upgrades_to
                        .is_some()
            })
            .min_by_key(|u| (best_strength(self.faction, u.kind), u.hp, u.id))
        else {
            return;
        };
        self.unit_order(weak.id, Order::DisbandUnit { unit: weak.id });
    }
}

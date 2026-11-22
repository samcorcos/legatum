//! The reserve ledger: one Gold runway forecast from the seat's own `PlayerView` economy,
//! read before any strategic commitment and again after this turn's own orders.
//!
//! When the runway is short the AI funds before it cuts: it stops discretionary Gold
//! spending, moves non-front cities off new troops (or onto Gold), carries a campaign only
//! if its capture comes first, and otherwise cuts its least useful units far from the front
//! on its own terms. The engine's bankruptcy disband (the costliest unit, wherever it
//! stands) should then almost never fire.

use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{Order, ProductionItem, UnitId};

/// Turns this treasury still pays at `net` Gold per turn: `None` while income covers upkeep,
/// `Some(0)` when this turn already ends negative.
pub(crate) fn runway_of(gold: i32, net: i32) -> Option<u32> {
    if gold + net < 0 {
        return Some(0);
    }
    (net < 0).then(|| (gold / -net) as u32)
}

impl Planner<'_> {
    /// The shortest runway the AI plans to keep, in Standard turns.
    pub(crate) fn runway_floor(&self) -> u32 {
        4
    }

    /// The runway at the start of the turn from the seat's own economy forecast (current
    /// income minus upkeep), before any of this turn's orders. Troops about to complete are
    /// left out: funding pauses them before it commits anything else, so a campaign is judged
    /// on the runway it can actually keep.
    pub(crate) fn gold_runway(&self) -> Option<u32> {
        let e = &self.view().economy;
        runway_of(e.gold, e.gold_per_turn.total)
    }

    /// Troops completing this turn that this turn's orders leave in production.
    fn completing_troops(&self) -> i32 {
        self.ctx
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
                        if legatum_core::data::pays_upkeep(unit))
            })
            .count() as i32
    }

    /// Net Gold per turn after this turn's orders so far: purchases' new upkeep, disbands,
    /// troops still completing, production switches that change city Gold, and
    /// `extra_units` more troops.
    pub(crate) fn planned_net_gold(&self, extra_units: i32) -> i32 {
        let original = self
            .view()
            .units
            .iter()
            .filter(|u| {
                u.owner == self.ctx.me
                    && legatum_core::data::pays_upkeep(u.kind)
                    && u.contract_until.is_none()
            })
            .count() as i32;
        let current = self.upkeep_unit_count()
            + self.planned_military_purchases()
            + self.completing_troops()
            + extra_units;
        self.view().economy.gold_per_turn.total + self.campaign_income_delta()
            - (self.engine_military_upkeep(current) - self.engine_military_upkeep(original))
    }

    /// The runway after this turn's orders so far, from the Gold not yet spent.
    pub(crate) fn planned_runway(&self, extra_units: i32) -> Option<u32> {
        runway_of(self.gold_left, self.planned_net_gold(extra_units))
    }

    /// Whether spending `cost` more Gold this turn keeps the runway at the floor.
    pub(crate) fn affordable_beyond_runway(&self, cost: i32) -> bool {
        runway_of(self.gold_left - cost, self.planned_net_gold(0))
            .is_none_or(|r| r >= self.runway_floor())
    }

    /// A city that is defending itself keeps its own builds: threatened, in resistance, or
    /// with a hostile army or city close by.
    pub(crate) fn funding_front(&self, ci: usize, c: &CityView) -> bool {
        let v = self.view();
        self.ctx.threat[ci] > 0
            || (self.faction == legatum_core::Faction::India
                && self.difficulty == legatum_core::AiDifficulty::Hard
                && self.decisions.india_prewar_guard > 0
                && self.assessed_border_threat(c.pos))
            || c.detail.as_ref().is_some_and(|d| d.resistance > 0)
            || self.ctx.enemy_units.iter().any(|e| {
                e.military
                    && !e.owner.is_barbarian()
                    && v.at_war(v.me, e.owner)
                    && e.pos.distance(c.pos) <= 5
            })
            || self
                .ctx
                .enemy_cities
                .iter()
                .any(|e| v.at_war(v.me, e.owner) && e.pos.distance(c.pos) <= 5)
    }

    /// A unit within 4 hexes of a city we are at war with is at the front.
    pub(crate) fn at_front(&self, pos: legatum_core::Hex) -> bool {
        let v = self.view();
        self.ctx
            .enemy_cities
            .iter()
            .any(|c| v.at_war(v.me, c.owner) && pos.distance(c.pos) <= 4)
    }

    /// Within reach of one of our cities that an enemy at war stands near: a defender, the
    /// last unit to disband for upkeep (playtest 8.2: AIs disbanded healthy units beside the
    /// city the human was besieging, two turns before losing it).
    pub(crate) fn defending(&self, pos: legatum_core::Hex) -> bool {
        self.hard_defense()
            && self.decisions.protect_defenders > 0
            && self.ctx.own_cities.iter().any(|c| {
                c.pos.distance(pos) <= crate::military::DEFEND_RADIUS
                    && self.ctx.enemy_units.iter().any(|e| {
                        e.military
                            && !e.owner.is_barbarian()
                            && e.kind != legatum_core::UnitKind::Scout
                            && e.pos.distance(c.pos) <= crate::analysis::THREAT_RADIUS
                    })
            })
    }

    /// The active campaign's forecast capture lands inside this runway.
    pub(crate) fn campaign_capture_within(&self, runway: u32) -> bool {
        let Some(active) = &self.campaigns.active else {
            return false;
        };
        self.view()
            .cities
            .iter()
            .find(|c| c.id == active.target)
            .and_then(|city| self.campaign_siege_estimate(city, &active.committed))
            .is_some_and(|(turns, _)| turns.max(0) as u32 <= runway)
    }

    /// Funding before cutting, step (b) and the build rule: a non-front city doesn't start or
    /// continue a troop whose upkeep leaves less than the runway floor, unless the campaign's
    /// capture comes first. It switches to the non-unit option paying the most Gold instead;
    /// while the runway is short it also takes any option that raises Gold (a Germany
    /// Gold project, a Market) in place of a building that doesn't.
    pub(crate) fn fund_production_runway(&mut self) {
        let floor = self.runway_floor();
        let cities: Vec<(usize, &CityView)> =
            self.ctx.own_cities.iter().copied().enumerate().collect();
        for (ci, c) in cities {
            let Some(d) = &c.detail else { continue };
            if d.resistance > 0 || self.funding_front(ci, c) {
                continue;
            }
            let ordered = self.orders.iter().rev().find_map(|o| match o {
                Order::SetProduction { city, item } if *city == c.id => Some(*item),
                _ => None,
            });
            let item = ordered.or(d.producing);
            let troop = matches!(item, Some(ProductionItem::Unit { unit })
                if self.view().rule_experiments.unit_stats_for(self.faction, unit).military);
            let civilian = matches!(item, Some(ProductionItem::Unit { .. })) && !troop;
            // A troop completing this turn is already in the planned upkeep.
            let counted = ordered.is_none() && d.turns_to_complete == Some(1);
            let pause = troop
                && self
                    .planned_runway(i32::from(!counted))
                    .is_some_and(|r| r < floor && !self.campaign_capture_within(r));
            // Gene `troop_runway`: a troop not yet under way must also leave the longer runway
            // (`Planner::troop_unfunded`).
            let pause = pause
                || !counted
                    && d.production_stored == 0
                    && matches!(item, Some(ProductionItem::Unit { unit })
                        if self.troop_unfunded(ci, c, unit) && !self.campaign_capture_within(0));
            let short = self.planned_runway(0).is_some_and(|r| r < floor);
            if !pause && (!short || civilian) {
                continue;
            }
            let current_gold = d
                .options
                .iter()
                .find(|o| Some(o.item) == item)
                .map_or(d.yields.total.gold, |o| o.yields.total.gold);
            let current_gold = self.remitted_city_gold(c, current_gold);
            let best = d
                .options
                .iter()
                .filter(|o| !matches!(o.item, ProductionItem::Unit { .. }))
                .filter(|o| self.gold_swap_allows(o.item))
                .filter(|o| pause || self.remitted_city_gold(c, o.yields.total.gold) > current_gold)
                .max_by_key(|o| {
                    (
                        self.remitted_city_gold(c, o.yields.total.gold)
                            - self.remitted_city_gold(c, d.yields.total.gold),
                        -o.turns,
                        std::cmp::Reverse(o.item),
                    )
                })
                .map(|o| o.item);
            let Some(best) = best.filter(|b| Some(*b) != item) else {
                continue;
            };
            // An item bought this turn is done; it can't also be set as the city's production
            // (a tile improvement is refused once built).
            if self.orders.iter().any(
                |o| matches!(o, Order::BuyItem { city, item: bought } if *city == c.id && *bought == best),
            ) {
                continue;
            }
            self.orders
                .retain(|o| !matches!(o, Order::SetProduction { city, .. } if *city == c.id));
            if d.producing != Some(best) {
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: best,
                });
            }
        }
    }

    /// Step (a): this turn's discretionary purchases may not leave the runway under the
    /// floor. `protected` purchases (emergency and threat defense) stand; the others are
    /// withdrawn, newest first, until the runway is restored.
    pub(crate) fn veto_unfunded_purchases(&mut self, protected: &[Order]) {
        let floor = self.runway_floor();
        while self.planned_runway(0).is_some_and(|r| r < floor) {
            let Some(i) = self
                .orders
                .iter()
                .rposition(|o| matches!(o, Order::BuyItem { .. }) && !protected.contains(o))
            else {
                return;
            };
            let Order::BuyItem { city, item } = self.orders.remove(i) else {
                unreachable!("found a purchase")
            };
            let refund = self.purchase_cost(city, item);
            self.gold_left += refund;
            self.record_blocked_expense(city, item, refund, crate::plan::BlockReason::RunwayFloor);
        }
    }

    /// Step (d): when funding still leaves the runway under the floor, cut the least useful
    /// field units far from the front now, on our terms, rather than let the treasury run
    /// dry and the engine take the costliest unit wherever it stands. Garrisons, committed
    /// campaign units and units at the front are never cut here.
    pub(crate) fn trim_unfunded_army(&mut self) {
        let floor = self.runway_floor();
        while self.planned_runway(0).is_some_and(|r| r < floor) {
            let units = self.upkeep_unit_count() + self.planned_military_purchases();
            if self.engine_military_upkeep(units) <= self.engine_military_upkeep(units - 1) {
                return;
            }
            let victim: Option<UnitId> = self
                .ctx
                .own_units
                .iter()
                .filter(|u| {
                    legatum_core::data::pays_upkeep(u.kind)
                        && !self.has_acted(u.id)
                        && !self.campaign_protects(u.id)
                        && !self.at_front(u.pos)
                        && !self.defending(u.pos)
                        && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                })
                .min_by_key(|u| (self.unit_keep_value(u), u.strength.max(u.ranged), u.id))
                .map(|u| u.id);
            let Some(id) = victim else { return };
            if !self.unit_order(id, Order::DisbandUnit { unit: id }) {
                return;
            }
            self.ctx.own_units.retain(|u| u.id != id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::runway_of;

    #[test]
    fn runway_counts_the_turns_the_treasury_still_pays() {
        // 9 Gold at -2: 7, 5, 3, 1 are paid; the fifth turn ends at -1.
        assert_eq!(runway_of(9, -2), Some(4));
        assert_eq!(runway_of(1, -2), Some(0));
        assert_eq!(runway_of(0, 0), None);
        assert_eq!(runway_of(5, 3), None);
        assert_eq!(runway_of(2, -2), Some(1));
    }
}

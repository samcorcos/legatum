//! Paid, view-derived faction objectives shared by construction, movement and diplomacy.
//! Plans are recomputed each turn; a captured city or blocked route invalidates the plan.

use crate::scripted::Planner;
use legatum_core::view::CityView;
use legatum_core::{BuildingKind, Order, ProductionItem, UnitKind, data};

fn building(building: BuildingKind) -> ProductionItem {
    ProductionItem::Building { building }
}

/// What a faction-goal item needs before this city can start it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GoalNeed {
    /// Start this item now: the goal item itself, or the first startable item on its
    /// requirement chain (a lower building tier, the goal's economic foundation).
    Build(ProductionItem),
    /// A technology that no research has delivered yet gates the chain.
    Research(legatum_core::TechId),
    /// Nothing to start now: population, resources or an already finished chain.
    Wait,
}

/// Research value per goal VP of a technology the pending goal's buildings need. A Market
/// is 3 VP for Mongols, so Exchange outranks the ordinary economic picks.
const GOAL_TECH_VALUE_PER_VP: i32 = 150;

impl Planner<'_> {
    /// The first thing to start toward `item` in `city`, walking its requirement chain:
    /// the item itself, then lower tiers, then the unlocking technology.
    pub(crate) fn goal_need(&self, city: &CityView, item: ProductionItem) -> GoalNeed {
        self.goal_chain(city, item)
    }

    fn goal_chain(&self, city: &CityView, item: ProductionItem) -> GoalNeed {
        let d = city.detail.as_ref().expect("own city");
        if let ProductionItem::Building { building: b } = item
            && data::has_building(&d.buildings, b)
        {
            return GoalNeed::Wait;
        }
        if d.options.iter().any(|o| o.item == item) {
            return GoalNeed::Build(item);
        }
        if let Some(tech) = data::item_requires_for_rules(self.view().rule_experiments, item)
            && !self.view().research.known.contains(&tech)
        {
            return GoalNeed::Research(tech);
        }
        if let ProductionItem::Building { building: b } = item
            && let Some(previous) = data::building_predecessor(b)
            && !d.buildings.contains(&previous)
        {
            return self.goal_chain(city, building(previous));
        }
        GoalNeed::Wait
    }

    /// Every item the pending goal still needs somewhere in the empire, whichever city
    /// ends up building it, and the technologies the goal names outright. Research reads
    /// this to unlock them early.
    ///
    /// Audit of the other goals (playtest 8.3 follow-up, 8-player Standard Hard, 1,000
    /// games): Switzerland (Gold), Athens (the broadcast), Byzantium (Science) and Celts (a Granary,
    /// buildable from turn 1) have no gated item. Germany's four technologies are known by
    /// the end in over 98% of seats and Sparta's Fortification and Ballistics in over 99%:
    /// their misses come from population and war, not a missing unlock, so they keep their
    /// ordinary research and production scoring.
    fn goal_items(&self) -> (Vec<ProductionItem>, Vec<legatum_core::TechId>) {
        if !self.goal_pending() {
            return (Vec::new(), Vec::new());
        }
        if self.faction == legatum_core::Faction::India {
            return (
                vec![
                    ProductionItem::Building {
                        building: BuildingKind::Market,
                    },
                    ProductionItem::Building {
                        building: BuildingKind::ImperialDiwan,
                    },
                ],
                Vec::new(),
            );
        }
        (Vec::new(), Vec::new())
    }

    /// Technologies the pending goal's items wait on, with their unresearched prerequisites.
    pub(crate) fn goal_techs(&self) -> &std::collections::BTreeSet<legatum_core::TechId> {
        self.goal_techs.get_or_init(|| {
            let known = &self.view().research.known;
            let (items, named) = self.goal_items();
            let mut techs = std::collections::BTreeSet::new();
            let rules = self.view().rule_experiments;
            for tech in items
                .into_iter()
                .filter_map(|item| data::item_requires_for_rules(rules, item))
                .chain(named)
            {
                crate::investment::prerequisites(tech, known, &mut techs);
            }
            techs
        })
    }

    /// Research value of a technology the pending faction goal needs; 0 for any other.
    pub(crate) fn goal_tech_priority(&self, tech: legatum_core::TechId) -> i32 {
        if self.goal_techs().contains(&tech) {
            legatum_core::goals::spec(self.faction).vp as i32 * GOAL_TECH_VALUE_PER_VP
        } else {
            0
        }
    }

    /// A goal is paid through normal quotes, reserve, upkeep and purchase limits. Begin
    /// before the endgame; no free building, upgrade, resource, population or army slot.
    pub(crate) fn plan_faction_goal(&mut self) {
        let mut steps = Vec::new();
        self.hall_steps(&mut steps);
        self.diwan_steps(&mut steps);
        // Earlier construction stays committed. One goal purchase per turn leaves the
        // ordinary planner its economy; separate cities may build the plan in parallel.
        let mut bought = false;
        for (id, mut item) in steps {
            let Some((ci, city)) = self
                .ctx
                .own_cities
                .iter()
                .enumerate()
                .find(|(_, c)| c.id == id)
            else {
                continue;
            };
            let d = city.detail.as_ref().unwrap();
            if d.resistance > 0 || self.ctx.threat[ci] > self.local_defense(city)
                || self.orders.iter().any(|o| matches!(o, Order::SetProduction { city, .. } | Order::BuyItem { city, .. } if *city == id)) { continue; }
            // A goal item that is not yet startable is pursued through whatever it waits
            // for, at the goal's priority: the Workshop beneath a Market, a lower building
            // tier. A missing technology is the research planner's job (`goal_techs`).
            match self.goal_need(city, item) {
                GoalNeed::Build(first) => item = first,
                GoalNeed::Research(_) | GoalNeed::Wait => continue,
            }
            let Some(opt) = d.options.iter().find(|o| o.item == item) else {
                continue;
            };
            let military = matches!(item, ProductionItem::Unit { unit } if self.view().rule_experiments.unit_stats_for(self.faction, unit).military);
            // The objective must pay for its economic/defensive foundation too.
            // Do not consume a young colony's Workshop or garrison budget for a
            // Market/settler that cannot finish the entire network on its own.
            let minimum = self.ctx.own_cities.len()
                + usize::from(
                    self.difficulty == legatum_core::AiDifficulty::Hard
                        && self.ctx.own_cities.len() >= 3,
                );
            if !military
                && self.goal_unit_pipeline(|u| {
                    self.view()
                        .rule_experiments
                        .unit_stats_for(self.faction, u)
                        .military
                        && u != UnitKind::Scout
                }) < minimum
            {
                continue;
            }
            let upkeep = if matches!(item, ProductionItem::Unit { unit } if data::pays_upkeep(unit))
            {
                self.marginal_military_upkeep()
            } else {
                0
            };
            let reserve = self.goal_reserve(upkeep);
            let india_diwan = self.faction == legatum_core::Faction::India
                && item == building(BuildingKind::ImperialDiwan);
            if (self.gold_left < reserve && !india_diwan)
                || opt.turns < 1
                || opt.turns > self.turns_remaining()
            {
                continue;
            }
            if !bought
                && opt.turns > 1
                && opt.buy_block.is_none()
                && (!military || self.ctx.threat[ci] == 0 && self.military_purchase_available())
                && let Some(cost) = opt
                    .buy_cost
                    .filter(|c| *c > 0 && self.gold_left - c >= reserve)
            {
                self.gold_left -= cost;
                self.orders.push(Order::BuyItem { city: id, item });
                bought = true;
            } else if d.producing.is_none()
                || self.campaigns.plan.posture == Some(crate::plan::Posture::Goal)
                    && d.producing != Some(item)
                || d.producing != Some(item) && self.hall_takes_over(id, item)
                || india_diwan
                    && d.producing != Some(item)
                    && d.turns_to_complete.is_none_or(|turns| turns > 3)
            {
                self.orders.push(Order::SetProduction { city: id, item });
            }
        }
    }

    fn diwan_steps(&self, steps: &mut Vec<(legatum_core::CityId, ProductionItem)>) {
        if self.faction != legatum_core::Faction::India
            || self.difficulty != legatum_core::AiDifficulty::Hard
            || self.decisions.india_diwan_commit == 0
            || !self.goal_pending()
            || self.view().turn < 70
            || !self.ctx.own_cities.iter().all(|city| {
                city.detail
                    .as_ref()
                    .is_some_and(|d| data::has_building(&d.buildings, BuildingKind::Market))
            })
        {
            return;
        }
        let diwan = building(BuildingKind::ImperialDiwan);
        if self.ctx.own_cities.iter().any(|city| {
            city.detail
                .as_ref()
                .is_some_and(|d| d.producing == Some(diwan))
        }) {
            return;
        }
        if let Some((id, _)) = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter_map(|(ci, city)| {
                let option = city
                    .detail
                    .as_ref()?
                    .options
                    .iter()
                    .find(|o| o.item == diwan)?;
                (self.ctx.threat[ci] <= self.local_defense(city))
                    .then_some((city.id, (self.threat_exposure(city), option.turns, city.id)))
            })
            .min_by_key(|(_, rank)| *rank)
        {
            steps.push((id, diwan));
        }
    }

    fn goal_reserve(&self, upkeep: i32) -> i32 {
        self.gold_reserve()
            .max((-self.view().economy.gold_per_turn.total + upkeep).max(0) * 40)
    }

    fn goal_unit_pipeline(&self, qualifies: impl Fn(UnitKind) -> bool) -> usize {
        let item = |i| matches!(i, ProductionItem::Unit { unit } if qualifies(unit));
        self.ctx
            .own_units
            .iter()
            .filter(|u| qualifies(u.kind))
            .count()
            + self
                .ctx
                .own_cities
                .iter()
                .map(|c| {
                    let pending = self.orders.iter().rev().find_map(|o| match o {
                        Order::SetProduction { city, item } if *city == c.id => Some(*item),
                        _ => None,
                    });
                    if let Some(next) = pending {
                        usize::from(item(next))
                    } else {
                        let d = c.detail.as_ref().unwrap();
                        d.producing
                            .into_iter()
                            .chain(d.production_queue.iter().copied())
                            .filter(|i| item(*i))
                            .count()
                    }
                })
                .sum::<usize>()
            + self
                .orders
                .iter()
                .filter(|o| matches!(o, Order::BuyItem { item: i, .. } if item(*i)))
                .count()
    }
}

#[cfg(test)]
#[path = "goals_tests.rs"]
mod tests;

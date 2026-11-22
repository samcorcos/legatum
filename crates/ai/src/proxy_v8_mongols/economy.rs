//! Reinvest a successful campaign's treasury in the cities that now have to support it.
use super::*;

impl Plan<'_> {
    pub(super) fn buy_infrastructure(&mut self) {
        if self.ctx.own_cities.len() < 5 {
            return;
        }
        let mut reserve = 50 + (-self.ctx.view.economy.gold_per_turn.total).max(0) * 12;
        if self.target.is_some() {
            reserve += 200;
        }
        // The hoard is a live goal: keep it while held, bank it while pending.
        if self
            .ctx
            .view
            .players
            .iter()
            .any(|p| p.id == self.ctx.me && p.faction == Faction::Switzerland)
        {
            reserve = reserve.max(legatum_core::goals::SWISS_HOARD_GOLD);
        }
        let mut offers = Vec::new();
        for c in &self.ctx.own_cities {
            let d = c.detail.as_ref().expect("own detail");
            if d.resistance > 0 {
                continue;
            }
            for o in &d.options {
                if o.turns <= 5 || o.buy_block.is_some() {
                    continue;
                }
                let Some(cost) = o.buy_cost else {
                    continue;
                };
                let value = match o.item {
                    ProductionItem::Building {
                        building: BuildingKind::Library,
                    } => 650,
                    ProductionItem::Building {
                        building: BuildingKind::Market,
                    } => {
                        if self.ctx.view.economy.gold_per_turn.total < 15 {
                            700
                        } else {
                            500
                        }
                    }
                    ProductionItem::Building {
                        building: BuildingKind::University,
                    } => 600,
                    ProductionItem::Building {
                        building: BuildingKind::Workshop,
                    } => 450,
                    _ => continue,
                };
                offers.push((Reverse(value * 1000 / cost.max(1)), c.id, o.item, cost));
            }
        }
        offers.sort();
        for (_, city, item, cost) in offers {
            if self
                .orders
                .iter()
                .any(|o| matches!(o, Order::BuyItem { city: id, .. } if *id == city))
                || self.gold < cost + reserve
            {
                continue;
            }
            self.orders.push(Order::BuyItem { city, item });
            self.gold -= cost;
        }
    }
}

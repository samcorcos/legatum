//! Mercenary Companies (Byzantium, `legatum_core::company`): when the Byzantium AI hires, and from
//! whom. Everything is priced from the public view (`PlayerView::company_offers`); the AI never
//! sees more than a human sees in the shop.
//!
//! **Why it hires.** Thin levies cap its own army at cities + 1, so its field army is rented. It
//! hires only with a purpose that pays within the 30-turn contract: it is at War and either a
//! campaign, a conquest or a containment march is under way, or one of its cities is threatened.
//! The want is the army the planner would field without the cap (`uncapped_army`) less the armed
//! units it already has, Companies included; at most `HIRES_PER_TURN` a turn, so contracts are
//! staggered rather than expiring together.
//!
//! **What it hires and from whom.** Each seller offers only its own special unit, so the choice
//! is the seller; a row the seller cannot field yet is skipped. Each offer scores the unit's
//! strength (in the seller's version) per Gold plus the
//! class-triangle value against the armies it faces (`composition_bonus`), less a penalty for
//! enriching a seller that is ahead of it in the race (the Gold goes to the seller, so paying
//! the leader is dearer than paying a distant minnow). It hires in the city with the most
//! exposure when threatened, otherwise the one nearest the target, never in a city that already
//! bought or hired this turn.
//!
//! **What it can afford.** The price comes out of Gold above the reserve, and the new upkeep
//! (2 Gold a turn a Company) must leave the treasury at least `runway_floor` turns of runway,
//! so an unpaid Company (which turns barbarian) is a mistake it avoids by construction.
use crate::scripted::Planner;
use legatum_core::company;
use legatum_core::data;
use legatum_core::view::CityView;
use legatum_core::{Faction, Order};

/// Companies hired in one turn at most.
const HIRES_PER_TURN: i32 = 2;
/// Score points lost per VP the seller leads this seat by.
const SELLER_LEAD_PENALTY: i32 = 40;
/// Extra penalty when the seller holds the highest VP in the game.
const SELLER_LEADER_PENALTY: i32 = 120;

impl Planner<'_> {
    /// Whether renting an army pays now: at War, with an offensive under way or a city threatened.
    fn hire_purpose(&self) -> bool {
        let v = self.view();
        let at_war = v.players.iter().any(|p| {
            p.alive && p.team != v.team && !p.id.is_barbarian() && v.teams_at_war(v.team, p.team)
        });
        let attacking = self.conquest_city().is_some()
            || self.contain_war().is_some()
            || self.campaigns.active.is_some();
        at_war && (attacking || self.ctx.threat.iter().any(|t| *t > 0))
    }

    /// The city to hire in: the most threatened, else the nearest the target, else any, among
    /// cities that have not bought or hired this turn.
    fn hire_city(&self) -> Option<&CityView> {
        let target = self.conquest_city().map(|c| c.pos).or_else(|| {
            self.campaigns.active.as_ref().and_then(|c| {
                self.view()
                    .cities
                    .iter()
                    .find(|city| city.id == c.target)
                    .map(|city| city.pos)
            })
        });
        self.ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.detail.as_ref().is_some_and(|d| d.resistance <= 0)
                    && !self.orders.iter().any(|o| match o {
                        Order::BuyItem { city, .. }
                        | Order::HireCompany { city, .. }
                        | Order::Raze { city } => *city == c.id,
                        _ => false,
                    })
            })
            .max_by_key(|(i, c)| {
                (
                    self.ctx.threat[*i],
                    target.map_or(0, |t| -c.pos.distance(t)),
                    std::cmp::Reverse(c.id),
                )
            })
            .map(|(_, c)| *c)
    }

    /// The offer worth most per Gold now, among those the treasury can carry.
    fn best_offer(&self, price_room: i32, gold: i32) -> Option<company::Offer> {
        let v = self.view();
        let best_vp = v
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.vp)
            .max()
            .unwrap_or(0);
        let my_vp = v
            .players
            .iter()
            .find(|p| p.id == v.me)
            .map_or(0, |p| p.vp as i32);
        v.company_offers
            .iter()
            .filter(|o| o.unavailable.is_none() && o.price <= price_room && o.price <= gold)
            .filter(|o| {
                let stats = self
                    .view()
                    .rule_experiments
                    .unit_stats_for(v.faction_of(o.seller), o.unit);
                stats.strength.max(stats.ranged) > 0
            })
            .max_by_key(|o| {
                // The seller's own version of its special unit (strength, shot, moves).
                let stats = self
                    .view()
                    .rule_experiments
                    .unit_stats_for(v.faction_of(o.seller), o.unit);
                // What a Company costs over its contract: the price plus its upkeep.
                let cost = o.price + company::UPKEEP * company::COMPANY_TURNS as i32;
                let power = stats.strength.max(stats.ranged) * 1000 / cost.max(1);
                let (seller_vp, seller_best) = v
                    .players
                    .iter()
                    .find(|p| p.id == o.seller)
                    .map_or((0, false), |p| (p.vp as i32, p.vp == best_vp));
                let penalty = (seller_vp - my_vp).max(0) * SELLER_LEAD_PENALTY
                    + if seller_best {
                        SELLER_LEADER_PENALTY
                    } else {
                        0
                    };
                (
                    power + self.composition_bonus(o.unit) - penalty,
                    std::cmp::Reverse(o.price),
                    std::cmp::Reverse(o.seller),
                    std::cmp::Reverse(o.unit as u8),
                )
            })
            .cloned()
    }

    /// Hire Companies for the army the planner wants beyond the cap.
    pub(crate) fn plan_company(&mut self) {
        if self.faction != Faction::Byzantium
            || self.view().company_offers.is_empty()
            || !self.hire_purpose()
        {
            return;
        }
        let hired = |p: &Self| {
            p.orders
                .iter()
                .filter(|o| matches!(o, Order::HireCompany { .. }))
                .count() as i32
        };
        let army = self
            .ctx
            .own_units
            .iter()
            .filter(|u| data::pays_upkeep(u.kind))
            .count() as i32;
        let want = self.uncapped_army() - army;
        for _ in 0..want.min(HIRES_PER_TURN) {
            let new = hired(self) + 1;
            let Some(city) = self.hire_city().map(|c| c.id) else {
                return;
            };
            let reserve = self.gold_reserve();
            let Some(offer) = self.best_offer(self.gold_left - reserve, self.gold_left) else {
                return;
            };
            // The runway after paying and carrying the new upkeep, 2 Gold a Company a turn.
            let net = self.planned_net_gold(0) - company::UPKEEP * new;
            if crate::funding::runway_of(self.gold_left - offer.price, net)
                .is_some_and(|r| r < self.runway_floor())
            {
                return;
            }
            self.gold_left -= offer.price;
            self.orders.push(Order::HireCompany {
                city,
                unit: offer.unit,
                seller: offer.seller,
            });
        }
    }
}

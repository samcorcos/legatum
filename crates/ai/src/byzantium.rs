//! Byzantium's **Buy a city** and **the fall of Constantinople** (core `buy_city.rs`, rules 63),
//! played from both sides. AI only: every read is the planner's own `PlayerView` (the public
//! prices in `city_buys`, the public offers in `city_offers`, the public capitals in
//! `byzantine_capitals`, public treasuries and armies), never the game state (pillar 6).
//!
//! **Buyer** (`plan_city_purchase`, gene `buy_city_value`). Each city Byzantium may buy is worth
//! what founding an equal city would cost: a Settler's Gold price (twice that when no good site is
//! left to found), `POP_GOLD` per population beyond the first for the growth turns, the full Gold
//! price of its buildings (the price charges half), `LAND_GOLD` for its land and output from the
//! first turn, and `GOAL_GOLD` for a goal's held VP in it. The gene weights that worth against the
//! bid and the chance the owner breaks the offer before the sale. **The bid** (core lets it run
//! from the price to `MAX_BID_PERCENT`% of it; all of it goes to the seller): the owner's threat
//! (`offer_threat`) is the armed units it can add in or next to the city before the sale, from
//! what we see: its visible units that can get there within the offer's turns, one bought unit a
//! turn its public treasury pays for, and Walls under way (a level counts as
//! `WALLS_PERCENT / GUARD_PERCENT` units). Each candidate bid covers the price with 0, 1, 2, ...
//! more units (`buy_city::defended_price`); the risk (`offer_risk`) grows with the threat the bid
//! leaves uncovered, plus gene `buy_city_overbid` percent of a unit for what we cannot see. The
//! candidate with the best expected gain is the bid; an offer likely lost (`LIKELY_FAIL`) is
//! never made. Of the bids within reach (affordable now, or net income pays the rest within gene
//! `buy_city_bank_turns`), the best positive gain is bid now if affordable, else the treasury is
//! held for it (`city_buy_savings`, part of `gold_reserve`). No offer while an enemy at War
//! stands near our cities.
//!
//! **Capital** (gene `byz_capital_guard`). While its capture would take every other city (no
//! Heir refuge), the capital's wanted garrison and its garrison and Walls value grow with the
//! cities that ride on it (`byz_capital_garrison_value`, `byz_capital_walls_value`), and a
//! threatened capital sends the Heir to shelter in another city (`goal_forge_heir.rs`).
//!
//! **Sellers** (gene `offer_defence`, `defend_city_offers`). When a Byzantium is buying one of our
//! cities, the cheapest way to lift the price above the escrow (the bid) before the sale, counted
//! by the view's `guards_to_stop`: units that can reach the city or a tile next to it in time,
//! then a bought unit or Walls in the city, unless even those and a purchase every turn left
//! cannot reach the count (then no Gold is spent on it). War only
//! for a city clearly worth more than the war (a goal's VP or a large city, with twice the
//! buyer's public army); otherwise the Gold is taken. In peace a city whose land touches a
//! Byzantium's wants a defender (gene `byz_border_guard`).
//!
//! **Rivals' targets** (gene `byz_capital_target`). A Byzantine capital with no refuge is worth
//! every other known Byzantine city as a conquest, campaign or strike target
//! (`byz_collapse_value`, `byz_collapse_cities`).
use crate::scripted::Planner;
use legatum_core::buy_city::{
    self, CityBuyView, FAIL_FEE_PERCENT, GUARD_PERCENT, OFFER_TURNS, WALLS_PERCENT,
};
use legatum_core::experiments::FactionRule;
use legatum_core::view::CityView;
use legatum_core::{BuildingKind, Faction, Hex, Order, ProductionItem, UnitKind, data};

/// Gold per population beyond the first: the growth turns a founded city would still need.
const POP_GOLD: i32 = 120;
/// Gold for a city's land and its output from the first turn.
const LAND_GOLD: i32 = 100;
/// Gold for a goal's held VP in the city (it follows the holder).
const GOAL_GOLD: i32 = 600;
/// Expected-gain bonus per hex between the city and its owner's capital (capped at 10 hexes):
/// a far city is harder for its owner to reinforce.
const FAR_GOLD_PER_HEX: i32 = 5;
/// An offer the owner breaks at least this often (percent) is likely lost: never made.
const LIKELY_FAIL: i32 = 60;
/// Risk (percent) of an offer that covers every threat we see: a war, an unseen army.
const BASE_RISK: i32 = 5;
/// Risk (percent) per armed unit of the owner's threat a bid leaves uncovered.
const UNCOVERED_RISK: i32 = 45;
/// Most extra units a bid tries to cover.
const MAX_COVER: i32 = 8;
/// Seller: a city at least this big is worth a war (with the army to win it).
const WAR_CITY_POPULATION: i32 = 8;

/// One appraised buyable city at one bid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Appraisal {
    pub city: legatum_core::CityId,
    pub price: i32,
    /// The Gold offered (the price or more).
    pub bid: i32,
    /// Gold-equivalent worth, weighted by the gene.
    pub value: i32,
    /// Percent chance the owner breaks the offer.
    pub risk: i32,
    /// Risk-weighted gain in Gold.
    pub gain: i32,
    /// The order is legal now.
    pub ready: bool,
    /// Only the treasury blocks it.
    pub short_of_gold: bool,
}

impl Planner<'_> {
    fn byzantium_buys(&self) -> bool {
        self.faction == Faction::Byzantium
            && self.decisions.buy_city_value > 0
            && self.rule_on(FactionRule::ByzantiumBuyCity)
    }

    /// A Settler's Gold price for us: what founding a city costs.
    fn settler_gold(&self) -> i32 {
        let v = self.view();
        let cost = v.rule_experiments.base_production_cost(
            self.faction,
            ProductionItem::Unit {
                unit: UnitKind::Settler,
            },
        );
        cost * data::BUY_COST_PERCENT / 100
    }

    /// Gold-equivalent worth of a buyable city (see the module doc).
    pub(crate) fn city_buy_worth(&self, buy: &CityBuyView, city: &CityView) -> i32 {
        let founding = self.settler_gold() * if self.count_good_sites(1) == 0 { 2 } else { 1 };
        let goal = if buy.holds_goal { GOAL_GOLD } else { 0 };
        founding + POP_GOLD * (city.population - 1).max(0) + buy.buildings_gold + LAND_GOLD + goal
    }

    /// Armed units the owner can add in or next to `city` before a sale, from what we see: its
    /// visible units that can get there within the offer's turns, one bought unit a turn its
    /// public treasury pays for, and Walls under way (a level counts as
    /// `WALLS_PERCENT / GUARD_PERCENT` units).
    pub(crate) fn offer_threat(&self, city: &CityView) -> i32 {
        let v = self.view();
        let reach = v
            .units
            .iter()
            .filter(|u| {
                legatum_core::rules::is_military(u.kind)
                    && !u.owner.is_barbarian()
                    && v.same_team(u.owner, city.owner)
                    && u.pos.distance(city.pos) > 1
                    && u.pos.distance(city.pos) - 1 <= u.max_moves.max(1) * OFFER_TURNS as i32
            })
            .count() as i32;
        let defender_gold = self.cheapest_defender_gold().max(1);
        let buys = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .map_or(0, |p| {
                (p.gold.max(0) / defender_gold).min(OFFER_TURNS as i32)
            });
        let walls = if city.walls_building {
            WALLS_PERCENT / GUARD_PERCENT
        } else {
            0
        };
        reach + buys + walls
    }

    /// Percent chance the owner lifts the price above `bid` before the sale: `BASE_RISK`, plus
    /// `UNCOVERED_RISK` per unit of `threat` (and gene `buy_city_overbid` percent of a unit for
    /// what we cannot see) beyond the units the bid absorbs.
    pub(crate) fn offer_risk(
        &self,
        buy: &CityBuyView,
        city: &CityView,
        threat: i32,
        bid: i32,
    ) -> i32 {
        let covered = buy_city::guards_to_stop(
            buy.base_price,
            i32::from(city.walls_level),
            buy.guards,
            buy.holds_goal,
            bid,
        ) - 1;
        let uncovered = (100 * threat + self.decisions.buy_city_overbid - 100 * covered).max(0);
        (BASE_RISK + UNCOVERED_RISK * uncovered / 100).clamp(BASE_RISK, 95)
    }

    /// Our cheapest Gold price of an armed unit: the stand-in for what a seller pays for one more
    /// guard (rival prices are private; ours are the same rule).
    fn cheapest_defender_gold(&self) -> i32 {
        self.ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref())
            .flat_map(|d| d.options.iter())
            .filter(|o| {
                matches!(o.item, ProductionItem::Unit { unit } if legatum_core::rules::is_military(unit))
            })
            .map(|o| o.buy_cost.unwrap_or(o.cost * data::BUY_COST_PERCENT / 100))
            .min()
            .unwrap_or(80)
    }

    /// Every city we could buy, appraised at each candidate bid: the price with 0, 1, 2, ... more
    /// of the owner's units covered (up to the threat, `MAX_COVER` and core's `max_bid`).
    pub(crate) fn city_buy_appraisals(&self) -> Vec<Appraisal> {
        let v = self.view();
        let gene = self.decisions.buy_city_value;
        let mut out = Vec::new();
        for b in v.city_buys.iter().filter(|b| b.buyer == v.me) {
            let ready = b.blocker.is_none();
            let short_of_gold = b.short_of_gold > 0;
            if !ready && !short_of_gold {
                continue;
            }
            let Some(city) = v.cities.iter().find(|c| c.id == b.city) else {
                continue;
            };
            let value = self.city_buy_worth(b, city) * gene / 100;
            let threat = self.offer_threat(city);
            let price = b.price.max(1);
            let far = v
                .cities
                .iter()
                .find(|c| c.owner == city.owner && c.is_capital)
                .map_or(0, |cap| {
                    cap.pos.distance(city.pos).min(10) * FAR_GOLD_PER_HEX
                });
            let walls = i32::from(city.walls_level);
            let mut last = 0;
            for k in 0..=(threat + 1).min(MAX_COVER) {
                let bid = buy_city::defended_price(b.base_price, walls, b.guards + k, b.holds_goal)
                    .max(price);
                if bid > b.max_bid.max(price) || bid <= last {
                    break;
                }
                last = bid;
                let risk = self.offer_risk(b, city, threat, bid);
                let gain = ((100 - risk) * (value - bid + far)
                    - risk * bid * FAIL_FEE_PERCENT / 100)
                    / 100;
                out.push(Appraisal {
                    city: b.city,
                    price,
                    bid,
                    value,
                    risk,
                    gain,
                    ready,
                    short_of_gold,
                });
            }
        }
        out
    }

    /// An enemy at War stands within three hexes of one of our cities: the Gold defends first.
    fn war_at_home(&self) -> bool {
        let v = self.view();
        self.ctx.threat_units.iter().any(|u| {
            u.military
                && !u.owner.is_barbarian()
                && v.at_war(v.me, u.owner)
                && self
                    .ctx
                    .own_cities
                    .iter()
                    .any(|c| c.pos.distance(u.pos) <= 3)
        })
    }

    /// Byzantium: make the best offer, or bank toward it (see the module doc).
    pub(crate) fn plan_city_purchase(&mut self) {
        if !self.byzantium_buys() {
            return;
        }
        let v = self.view();
        if v.city_offers.iter().any(|o| o.buyer == v.me) || self.war_at_home() {
            return;
        }
        let good: Vec<Appraisal> = self
            .city_buy_appraisals()
            .into_iter()
            .filter(|a| a.gain > 0 && a.risk < LIKELY_FAIL)
            .collect();
        let key = |a: &&Appraisal| (a.gain, std::cmp::Reverse(a.city), std::cmp::Reverse(a.bid));
        let gold = self.gold_left;
        let income = v.economy.gold_per_turn.total;
        let affordable = |a: &Appraisal| a.ready && a.bid <= gold;
        // Within reach: affordable now, or income pays the rest within `buy_city_bank_turns`.
        let reach = |a: &&Appraisal| {
            let need = a.bid - gold.max(0);
            affordable(a)
                || (income > 0
                    && (need + income - 1) / income <= self.decisions.buy_city_bank_turns)
        };
        let Some(best) = good.iter().filter(reach).max_by_key(key).copied() else {
            return;
        };
        if affordable(&best) {
            self.orders.push(Order::BuyCity {
                city: best.city,
                gold: best.bid,
            });
            self.gold_left -= best.bid;
        } else {
            self.city_buy_savings = best.bid;
        }
    }

    // ---------- The capital (the fall of Constantinople) ----------

    /// Cities that would fall with `city`, ours: our current capital while the collapse is on,
    /// the gene is on and no Heir shelters in another of our cities. 0 otherwise.
    pub(crate) fn byz_capital_riders(&self, city: &CityView) -> i32 {
        let v = self.view();
        if self.faction != Faction::Byzantium
            || self.decisions.byz_capital_guard == 0
            || !self.rule_on(FactionRule::ByzantiumCollapse)
            || city.owner != v.me
            || !city.is_capital
            || v.byzantine_capitals
                .iter()
                .any(|b| b.player == v.me && b.refuge.is_some())
        {
            return 0;
        }
        self.ctx.own_cities.len() as i32 - 1
    }

    /// Our capital carries other cities and a rival unit can reach it.
    pub(crate) fn byz_capital_pressed(&self, city: &CityView) -> bool {
        self.byz_capital_riders(city) > 0 && self.rival_unit_in_reach(city.pos)
    }

    /// The capital's wanted guards (in it or next to it): one, plus one per three other cities
    /// at gene 100.
    pub(crate) fn byz_capital_wanted_guards(&self, city: &CityView) -> i32 {
        1 + self.byz_capital_riders(city) * self.decisions.byz_capital_guard / 300
    }

    /// Production value of one more guard trained in the pressed capital while it has fewer than
    /// it wants; 0 otherwise.
    pub(crate) fn byz_capital_garrison_value(&self, city: &CityView) -> i32 {
        if !self.byz_capital_pressed(city) {
            return 0;
        }
        let guards = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                legatum_core::rules::is_military(u.kind)
                    && u.kind != UnitKind::Scout
                    && u.pos.distance(city.pos) <= 1
            })
            .count() as i32;
        if guards >= self.byz_capital_wanted_guards(city) {
            return 0;
        }
        self.byz_capital_stake_value(city)
    }

    /// Production value of Walls (or their upgrade) in the pressed capital; 0 otherwise.
    pub(crate) fn byz_capital_walls_value(&self, city: &CityView) -> i32 {
        if !self.byz_capital_pressed(city) {
            return 0;
        }
        self.byz_capital_stake_value(city)
    }

    fn byz_capital_stake_value(&self, city: &CityView) -> i32 {
        let riders = self.byz_capital_riders(city);
        (1200 + 400 * riders * self.decisions.byz_capital_guard / 100).min(8000)
    }

    /// Our capital carries other cities and an enemy is near: the Heir shelters elsewhere.
    pub(crate) fn byz_capital_threatened(&self) -> Option<legatum_core::CityId> {
        let capital = self.ctx.own_cities.iter().find(|c| c.is_capital)?;
        let riders = if self.faction == Faction::Byzantium
            && self.decisions.byz_capital_guard > 0
            && self.rule_on(FactionRule::ByzantiumCollapse)
        {
            self.ctx.own_cities.len() as i32 - 1
        } else {
            0
        };
        (riders > 0 && self.rival_unit_in_reach(capital.pos)).then_some(capital.id)
    }

    // ---------- Sellers ----------

    /// Raise the price of every city of ours under offer above its escrow, or answer with War
    /// when the city is clearly worth it (see the module doc).
    pub(crate) fn defend_city_offers(&mut self) {
        if self.decisions.offer_defence == 0 {
            return;
        }
        let v = self.view();
        let offers: Vec<_> = v
            .city_offers
            .iter()
            .filter(|o| o.seller == v.me && o.guards_to_stop > 0)
            .cloned()
            .collect();
        for offer in offers {
            let Some(city) = self
                .ctx
                .own_cities
                .iter()
                .copied()
                .find(|c| c.id == offer.city)
            else {
                continue;
            };
            let turns = offer.turns_left.max(1) as i32;
            let mut need = offer.guards_to_stop;
            need -= self.send_guards(city, need, turns);
            // Gold only when a purchase now and one each later turn (Walls worth two) can
            // still close the gap; otherwise keep it.
            if need > 0 && need <= turns * WALLS_PERCENT / GUARD_PERCENT {
                need -= self.buy_guard(city, need);
            }
            if need > 0 {
                self.offer_war(city, offer.buyer);
            }
        }
    }

    /// Our armed units in the city at `pos` or next to it.
    fn own_guards(&self, pos: Hex) -> i32 {
        let v = self.view();
        v.units
            .iter()
            .filter(|u| {
                legatum_core::rules::is_military(u.kind)
                    && v.same_team(u.owner, v.me)
                    && u.pos.distance(pos) <= 1
            })
            .count() as i32
    }

    /// Send up to `need` free units that can stand in or next to `city` within `turns` turns.
    /// Returns how many were sent.
    fn send_guards(&mut self, city: &CityView, need: i32, turns: i32) -> i32 {
        let v = self.view();
        let mut free: Vec<_> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| {
                legatum_core::rules::is_military(u.kind)
                    && !self.has_acted(u.id)
                    && u.pos.distance(city.pos) > 1
                    && u.pos.distance(city.pos) - 1 <= u.max_moves.max(1) * turns
                    && self.ctx.reachable(u.pos, city.pos)
                    // Never strip another city's only garrison.
                    && !self.ctx.own_cities.iter().any(|c| {
                        c.pos == u.pos
                            && !self
                                .ctx
                                .own_units
                                .iter()
                                .any(|o| o.id != u.id && o.military && o.pos == c.pos)
                    })
            })
            .collect();
        free.sort_by_key(|u| (u.pos.distance(city.pos), u.id));
        let mut sent = 0;
        for u in free {
            if sent >= need {
                break;
            }
            let spots = std::iter::once(city.pos).chain(city.pos.ring(1));
            let Some(to) = spots
                .filter(|h| {
                    v.index(*h).is_some_and(|i| {
                        !self.ctx.friendly_military_at[i]
                            && !self.ctx.hostile_at[i]
                            && !self.ctx.neutral_at[i]
                    }) && self.ctx.move_cost(*h).is_some()
                        && !self.mil_dest.contains(h)
                        && self.ctx.reachable(u.pos, *h)
                })
                .min_by_key(|h| (h.distance(u.pos), *h != city.pos))
            else {
                continue;
            };
            if self.unit_order(u.id, Order::MoveUnit { unit: u.id, to }) {
                self.mil_dest.insert(to);
                sent += 1;
            }
        }
        sent
    }

    /// Buy Walls (worth two guards) or the cheapest armed unit in `city`, whichever covers
    /// `need` for less. Returns the guards it adds.
    fn buy_guard(&mut self, city: &CityView, need: i32) -> i32 {
        if self
            .orders
            .iter()
            .any(|o| matches!(o, Order::BuyItem { city: id, .. } | Order::BuyTile { city: id, .. } if *id == city.id))
        {
            return 0;
        }
        let Some(d) = city.detail.as_ref() else {
            return 0;
        };
        let gold = self.gold_left;
        let affordable = |o: &&legatum_core::view::ProductionOption| {
            o.buy_blocker.is_none() && o.buy_cost.is_some_and(|c| c <= gold)
        };
        let walls = d
            .options
            .iter()
            .filter(affordable)
            .filter(|o| {
                matches!(o.item, ProductionItem::Building { building }
                    if data::building_family(building) == BuildingKind::Walls
                        || building == BuildingKind::UpgradeWalls)
            })
            .map(|o| (o.buy_cost.unwrap_or(i32::MAX), o.item, 2))
            .min();
        let unit = d
            .options
            .iter()
            .filter(affordable)
            .filter(|o| {
                matches!(o.item, ProductionItem::Unit { unit }
                    if legatum_core::rules::is_military(unit) && unit != UnitKind::Scout)
            })
            .map(|o| (o.buy_cost.unwrap_or(i32::MAX), o.item, 1))
            .min();
        let pick = match (walls, unit) {
            (Some(w), Some(u)) if need >= 2 || w.0 <= u.0 => Some(w),
            (Some(w), None) => Some(w),
            (_, Some(u)) => Some(u),
            (None, None) => None,
        };
        let Some((cost, item, covers)) = pick else {
            return 0;
        };
        self.orders.push(Order::BuyItem {
            city: city.id,
            item,
        });
        self.gold_left -= cost;
        covers
    }

    /// Declare War on the buyer when the city is clearly worth more than the war: it holds a
    /// goal's VP or is large, our public army is at least twice the buyer's, the declaration is
    /// legal and we fight no other war.
    fn offer_war(&mut self, city: &CityView, buyer: legatum_core::PlayerId) {
        let v = self.view();
        let team = v.team_of(buyer);
        let holds_goal = v
            .city_buys
            .iter()
            .any(|b| b.city == city.id && b.holds_goal);
        let army =
            |p: legatum_core::PlayerId| v.players.iter().find(|q| q.id == p).map_or(0, |q| q.army);
        let other_war = v
            .players
            .iter()
            .any(|p| p.alive && p.id != buyer && !p.id.is_barbarian() && v.at_war(v.me, p.id));
        if (holds_goal || city.population >= WAR_CITY_POPULATION)
            && army(v.me) >= 2 * army(buyer).max(1)
            && !other_war
            && crate::opportunity::legal_declaration(v, team)
            && !self
                .orders
                .iter()
                .any(|o| matches!(o, Order::DeclareWar { against, .. } if *against == team))
        {
            self.orders.push(Order::DeclareWar {
                against: team,
                breach: false,
            });
        }
    }

    /// Gene `byz_border_guard`: Production value of a defender in an own city whose land touches
    /// a Byzantium's, while no armed unit of ours stands in or next to it; 0 otherwise.
    pub(crate) fn byz_border_guard_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        if self.decisions.byz_border_guard == 0 || self.faction == Faction::Byzantium {
            return 0;
        }
        let touched = v.city_buys.iter().any(|b| {
            b.city == city.id && b.buyer != v.me && !v.at_war(v.me, b.buyer) && city.owner == v.me
        });
        if !touched || self.own_guards(city.pos) > 0 {
            return 0;
        }
        2200 * self.decisions.byz_border_guard / 100
    }

    // ---------- Rivals' targets ----------

    /// The other Byzantine cities that fall with `city`: it is a Byzantium's current capital
    /// with no Heir refuge, the collapse rule is on and the gene is on. Known cities count as
    /// themselves; the owner's public city count fills in the rest.
    pub(crate) fn byz_collapse_cities(&self, city: &CityView) -> Vec<Option<&CityView>> {
        let v = self.view();
        if self.decisions.byz_capital_target == 0
            || !self.rule_on(FactionRule::ByzantiumCollapse)
            || city.owner.is_barbarian()
            || v.same_team(v.me, city.owner)
            || !v
                .byzantine_capitals
                .iter()
                .any(|b| b.player == city.owner && b.capital == Some(city.id) && b.refuge.is_none())
        {
            return Vec::new();
        }
        let known: Vec<_> = v
            .cities
            .iter()
            .filter(|c| c.owner == city.owner && c.id != city.id)
            .map(Some)
            .collect();
        let total = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .map_or(0, |p| p.cities as usize)
            .saturating_sub(1);
        let unknown = total.saturating_sub(known.len());
        known
            .into_iter()
            .chain(std::iter::repeat_n(None, unknown))
            .collect()
    }

    /// Gene `byz_capital_target`: the keep value of every other city that falls with this
    /// Byzantine capital (an unknown one at a new city's), weighted by the gene.
    pub(crate) fn byz_collapse_value(&self, city: &CityView) -> i32 {
        let total: i32 = self
            .byz_collapse_cities(city)
            .into_iter()
            .map(|c| {
                c.map_or_else(
                    || crate::production::new_city_keep_value(1),
                    crate::production::city_keep_value,
                )
            })
            .sum();
        total * self.decisions.byz_capital_target / 100
    }
}

#[cfg(test)]
#[path = "byzantium_tests.rs"]
mod tests;

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
//! price and the chance the owner breaks the offer before the sale (`offer_risk`: its visible
//! units that can step in or next to the city within the offer's turns, a treasury that can buy a
//! defender, Walls under way). An offer likely lost (`LIKELY_FAIL`) is never made. The best
//! positive expected gain is bought now, or the treasury is
//! held for it (`city_buy_savings`, part of `gold_reserve`) when income pays the rest within
//! `BANK_TURNS`. No offer while an enemy at War stands near our cities.
//!
//! **Capital** (gene `byz_capital_guard`). While its capture would take every other city (no
//! Heir refuge), the capital's wanted garrison and its garrison and Walls value grow with the
//! cities that ride on it (`byz_capital_garrison_value`, `byz_capital_walls_value`), and a
//! threatened capital sends the Heir to shelter in another city (`goal_forge_heir.rs`).
//!
//! **Sellers** (gene `offer_defence`, `defend_city_offers`). When a Byzantium is buying one of our
//! cities, the cheapest way to lift the price above the escrow before the sale: units that can
//! reach the city or a tile next to it in time, then a bought unit or Walls in the city. War only
//! for a city clearly worth more than the war (a goal's VP or a large city, with twice the
//! buyer's public army); otherwise the Gold is taken. In peace a city whose land touches a
//! Byzantium's wants a defender (gene `byz_border_guard`).
//!
//! **Rivals' targets** (gene `byz_capital_target`). A Byzantine capital with no refuge is worth
//! every other known Byzantine city as a conquest, campaign or strike target
//! (`byz_collapse_value`, `byz_collapse_cities`).
use crate::scripted::Planner;
use legatum_core::buy_city::{CityBuyView, FAIL_FEE_PERCENT, OFFER_TURNS};
use legatum_core::experiments::FactionRule;
use legatum_core::view::CityView;
use legatum_core::{BuildingKind, Faction, Hex, Order, ProductionItem, UnitKind, data};

/// Gold per population beyond the first: the growth turns a founded city would still need.
const POP_GOLD: i32 = 120;
/// Gold for a city's land and its output from the first turn.
const LAND_GOLD: i32 = 100;
/// Gold for a goal's held VP in the city (it follows the holder).
const GOAL_GOLD: i32 = 600;
/// Turns of income the buyer banks toward its best offer.
const BANK_TURNS: i32 = 10;
/// Expected-gain bonus per hex between the city and its owner's capital (capped at 10 hexes):
/// a far city is harder for its owner to reinforce.
const FAR_GOLD_PER_HEX: i32 = 5;
/// An offer the owner breaks at least this often (percent) is likely lost: never made.
const LIKELY_FAIL: i32 = 60;
/// The breakdown labels (core `buy_city::base_breakdown`, `with_defences`).
const BUILDINGS_LABEL: &str = "Buildings";
const GOAL_LABEL: &str = "Holds a goal";
/// Seller: a city at least this big is worth a war (with the army to win it).
const WAR_CITY_POPULATION: i32 = 8;

/// One appraised buyable city.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Appraisal {
    pub city: legatum_core::CityId,
    pub price: i32,
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
        let line = |prefix: &str| {
            buy.breakdown
                .parts
                .iter()
                .filter(|(label, _)| label.starts_with(prefix))
                .map(|(_, gold)| *gold)
                .sum::<i32>()
        };
        let buildings = line(BUILDINGS_LABEL) * 100 / legatum_core::buy_city::BUILDING_PERCENT;
        let goal = if line(GOAL_LABEL) > 0 { GOAL_GOLD } else { 0 };
        founding + POP_GOLD * (city.population - 1).max(0) + buildings + LAND_GOLD + goal
    }

    /// Percent chance the owner lifts the price above the escrow before the sale, from what we
    /// see: its units that can reach the city or a tile next to it within the offer's turns, a
    /// treasury that can buy a defender there, and Walls under way.
    pub(crate) fn offer_risk(&self, city: &CityView) -> i32 {
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
        let defender_gold = self.cheapest_defender_gold();
        let rich = v
            .players
            .iter()
            .find(|p| p.id == city.owner)
            .is_some_and(|p| p.gold >= defender_gold);
        let risk = 10
            + 30 * reach.min(3)
            + if rich { 55 } else { 0 }
            + if city.walls_building { 25 } else { 0 };
        risk.clamp(5, 95)
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

    /// Every city we could buy, appraised.
    pub(crate) fn city_buy_appraisals(&self) -> Vec<Appraisal> {
        let v = self.view();
        let gene = self.decisions.buy_city_value;
        v.city_buys
            .iter()
            .filter(|b| b.buyer == v.me)
            .filter_map(|b| {
                let ready = b.blocker.is_none();
                let short_of_gold = b
                    .blocker
                    .as_deref()
                    .is_some_and(|r| r.starts_with("Needs ") && b.price > v.economy.gold);
                if !ready && !short_of_gold {
                    return None;
                }
                let city = v.cities.iter().find(|c| c.id == b.city)?;
                let value = self.city_buy_worth(b, city) * gene / 100;
                let risk = self.offer_risk(city);
                let price = b.price.max(1);
                let far = v
                    .cities
                    .iter()
                    .find(|c| c.owner == city.owner && c.is_capital)
                    .map_or(0, |cap| {
                        cap.pos.distance(city.pos).min(10) * FAR_GOLD_PER_HEX
                    });
                let gain = ((100 - risk) * (value - price + far)
                    - risk * price * FAIL_FEE_PERCENT / 100)
                    / 100;
                Some(Appraisal {
                    city: b.city,
                    price,
                    value,
                    risk,
                    gain,
                    ready,
                    short_of_gold,
                })
            })
            .collect()
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
        let Some(best) = self
            .city_buy_appraisals()
            .into_iter()
            .filter(|a| a.gain > 0 && a.risk < LIKELY_FAIL)
            .max_by_key(|a| (a.gain, std::cmp::Reverse(a.city)))
        else {
            return;
        };
        if best.ready && best.price <= self.gold_left {
            self.orders.push(Order::BuyCity { city: best.city });
            self.gold_left -= best.price;
            return;
        }
        let income = v.economy.gold_per_turn.total;
        let need = best.price - self.gold_left.max(0);
        if income > 0 && need > 0 && (need + income - 1) / income <= BANK_TURNS {
            self.city_buy_savings = best.price;
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
            .filter(|o| o.seller == v.me && o.price_now <= o.escrow)
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
            let guards = self.own_guards(city.pos);
            let percent = 100
                + i32::from(city.walls_level) * legatum_core::buy_city::WALLS_PERCENT
                + guards * legatum_core::buy_city::GUARD_PERCENT;
            let step = (offer.price_now * legatum_core::buy_city::GUARD_PERCENT / percent).max(1);
            let mut need = (offer.escrow - offer.price_now) / step + 1;
            need -= self.send_guards(city, need, offer.turns_left.max(1) as i32);
            if need > 0 {
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
        let holds_goal = v.city_buys.iter().any(|b| {
            b.city == city.id
                && b.breakdown
                    .parts
                    .iter()
                    .any(|(label, _)| label.starts_with(GOAL_LABEL))
        });
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

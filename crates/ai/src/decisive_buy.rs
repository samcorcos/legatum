//! Gene `decisive_buy`: spend the reserve on the decisive unit at the decisive moment
//! (designer-decision patterns 2026-11, fix 2; pattern rows 4 and 5).
//!
//! The designer banks Gold and spends the hoard on one unit when it decides a war: at the 8.9.24
//! Swiss T140, one turn after Flight, he bought a Bomber for 805 Gold (+2.38 VP in the paired
//! decision diff). Hard kept the Gold: its plan was Peace, so `hoard_spent_at_war` did not fire,
//! and the 5,000 Gold hoard savings stayed the purchase reserve.
//!
//! This pass prices a purchase by its forecast effect on a live war, not by the plan's label.
//! For every enemy city at war in reach of a city of ours, the siege forecast runs with the
//! field force that can reach it (and our aircraft), then again with the bought unit added
//! (a plane strikes from the next turn; a land unit, at gene level 2, marches from its city).
//! The gain is the capture odds the unit adds times the VP the capture brings (a capital, a
//! live goal, a Fight objective, a title). The price is the VP the spent Gold puts at risk: a Swiss saving
//! toward the hoard delays its VP by the turns of income the purchase costs, out of the turns
//! left; a held hoard is never spent. The purchase must leave the deficit reserve, keep its
//! upkeep payable, and add at least `decisive_buy_min` VP percent net. Peacetime and routine
//! buys see none of this, so they keep the reserve.
use crate::composition;
use crate::scripted::Planner;
use legatum_core::view::{AircraftView, CityView, PlayerView, UnitView};
use legatum_core::{CityId, Hex, Order, ProductionItem, TechId, UnitId, UnitKind, data};

/// The id the forecasts give the land unit we would buy.
const BOUGHT_UNIT: UnitId = UnitId(u32::MAX - 1);
/// The id the forecasts give the plane we would buy.
const BOUGHT_PLANE: UnitId = UnitId(u32::MAX - 2);

/// One priced candidate (diagnostics and the test read it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Decisive {
    pub city: CityId,
    pub item: ProductionItem,
    pub cost: i32,
    /// The capture this unit adds the most to.
    pub target: CityId,
    /// Capture odds of that target without and with the unit, percent.
    pub odds: (i32, i32),
    /// VP percent that target gains: its added capture odds times the capture's VP.
    pub best: i32,
    /// VP percent over every target: the sum of their gains.
    pub gain: i32,
    /// Hexes from the buying city to that target (a nearer base reaches more of the front).
    pub distance: i32,
    /// VP percent the spent Gold puts at risk.
    pub price: i32,
}

impl Planner<'_> {
    fn decisive_gate(
        &self,
        city: CityId,
        item: ProductionItem,
        target: CityId,
        gate: impl Into<String>,
    ) {
        if self.decision_trace.is_some() {
            let mut row = crate::DecisionOption::new(
                "decisive_buy",
                format!("city={city:?} item={item:?} target={target:?}"),
                "gated_out",
            );
            row.gate = Some(gate.into());
            self.note_option(row);
        }
    }
    /// Gold a decisive purchase must leave: the deficit runway and the defense floor, never
    /// the goal savings. A held Swiss hoard scores a live VP, so it stays whole.
    fn decisive_floor(&self) -> i32 {
        let deficit = self.decisions.reserve_deficit_turns
            * (-self.view().economy.gold_per_turn.total).max(0);
        let floor = 100.max(deficit);
        if self.hoard_held() {
            floor.max(self.ordinary_gold_reserve())
        } else {
            floor
        }
    }

    /// VP percent that spending `cost` Gold puts at risk: a Swiss saving toward the hoard
    /// reaches its VP later by the turns of income the purchase takes, a share of the turns
    /// left in the game (0 for everyone else: their reserve holds no VP).
    fn decisive_price(&self, cost: i32) -> i32 {
        if self.faction != legatum_core::Faction::Switzerland || !self.saving_for_goal() {
            return 0;
        }
        let income = self.view().economy.gold_per_turn.total;
        let vp = legatum_core::goals::spec(self.faction).vp as i32;
        if income <= 0 {
            return vp * 100;
        }
        let hoard = legatum_core::goals::SWISS_HOARD_GOLD;
        let turns = |gold: i32| ((hoard - gold).max(0) + income - 1) / income;
        let delay = turns(self.gold_left - cost) - turns(self.gold_left);
        // An uncapped game still ends around the expected length.
        let expected = data::EXPECTED_GAME_TURNS as i32 - self.view().turn as i32;
        let left = self
            .turns_remaining()
            .min(expected)
            .max(self.plan_horizon());
        (vp * 100 * delay / left).min(vp * 100)
    }

    /// VP a capture of `city` brings: an original capital, a live goal taken or denied, a
    /// Fight objective or a title (`conquest_victory_value`).
    fn decisive_capture_vp(&self, city: &CityView) -> i32 {
        let (taken, denied) = self.capture_goal_swing(city);
        self.conquest_victory_value(city)
            + self.capital_capture_vp(city)
            + self.capital_race_vp(city)
            + taken
            + denied
    }

    /// Our aircraft that can strike `city` this turn, with the turn they first strike (0).
    pub(crate) fn planes_in_reach(&self, city: &CityView) -> Vec<(AircraftView, i32)> {
        let v = self.view();
        v.aircraft
            .iter()
            .filter(|a| a.owner == v.me && data::is_bomber(a.kind))
            .filter(|a| {
                v.cities.iter().any(|base| {
                    base.id == a.city
                        && base.pos.distance(city.pos) <= data::unit_stats(a.kind).range
                })
            })
            .map(|a| (a.clone(), 0))
            .collect()
    }

    /// Estimated chance that a new Bomber's decisive strike is intercepted. A Fighter seen in
    /// range is direct evidence; entering Modern only makes Flight and a hidden Fighter possible.
    /// The view does not expose rival research or aircraft under fog, so unknown stays uncertain.
    fn interception_risk(v: &PlayerView, city: &CityView, bomber: UnitKind, gene: i32) -> i32 {
        let modern = v.era_of(city.owner) >= data::era_index(TechId::Flight) - 1;
        if gene == 0 {
            return if modern { 100 } else { 0 };
        }
        if v.air_sightings.iter().any(|a| {
            a.owner == city.owner
                && data::is_fighter(a.kind)
                && (bomber != UnitKind::StealthBomber || a.kind == UnitKind::StealthFighter)
                && v.cities.iter().any(|base| {
                    base.id == a.city
                        && base.pos.distance(city.pos) <= data::fighter_guard_radius(a.kind)
                })
        }) {
            // It may strike or rebase instead of guarding; sight is not certainty.
            75
        } else if let Some(contact) = v.air_contacts.iter().find(|a| {
            a.owner == city.owner
                && a.kind.is_some_and(data::is_fighter)
                && (bomber != UnitKind::StealthBomber || a.kind == Some(UnitKind::StealthFighter))
                && v.turn.saturating_sub(a.turn) <= 10
        }) {
            // A stale base or a battle report proves a Fighter was fielded, but says nothing
            // certain about whether it is still guarding this target.
            if contact.city.is_some_and(|id| {
                v.cities.iter().any(|base| {
                    base.id == id
                        && contact.kind.is_some_and(|kind| {
                            base.pos.distance(city.pos) <= data::fighter_guard_radius(kind)
                        })
                })
            }) {
                60
            } else {
                50
            }
        } else if modern {
            25
        } else {
            0
        }
    }

    /// Capture odds (percent) of the forecast siege of `city`, 0 when it fails.
    fn decisive_odds(
        &self,
        view: &PlayerView,
        city: &CityView,
        force: &[(&UnitView, Hex, i32)],
        planes: &[(AircraftView, i32)],
    ) -> i32 {
        if !force.iter().any(|(u, _, _)| composition::charges(u.kind)) {
            return 0;
        }
        let march = force.iter().map(|(_, _, t)| *t).max().unwrap_or(0);
        self.forecast_siege_air(view, city, force, planes)
            .map_or(0, |s| crate::plan::capture_odds(s.turns, march))
    }

    /// Where a land unit bought in `home` would attack `city` from, and the turn it arrives
    /// (bought this turn, it marches from the next).
    fn decisive_tile(
        &self,
        view: &PlayerView,
        home: &CityView,
        city: &CityView,
        unit: &UnitView,
    ) -> Option<(Hex, i32)> {
        let reach = if composition::shooter_only(unit.kind, unit.ranged) {
            unit.range.max(1)
        } else {
            1
        };
        city.pos
            .within(reach)
            .into_iter()
            .filter(|h| *h != city.pos)
            .filter(|h| view.tile(*h).and_then(|t| t.move_cost()).is_some())
            .filter(|h| !view.units.iter().any(|u| u.pos == *h))
            .filter(|h| !view.cities.iter().any(|c| c.pos == *h))
            .map(|h| {
                let steps = home.pos.distance(h);
                (
                    1 + (steps + unit.max_moves.max(1) - 1) / unit.max_moves.max(1),
                    h,
                )
            })
            .filter(|(turns, _)| *turns <= self.decisions.march_turns + 1)
            .min()
            .map(|(turns, h)| (h, turns))
    }

    /// The best decisive purchase this turn, if any: every buyable military unit in every
    /// city in reach of an enemy city at war, priced by the capture odds it adds.
    pub(crate) fn decisive_candidates(&self) -> Vec<Decisive> {
        let v = self.view();
        let gene = self.decisions.decisive_buy;
        if gene == 0 || !self.hard_defense() {
            if self.decision_trace.is_some() {
                let mut row = crate::DecisionOption::new("decisive_buy", "all", "gated_out");
                row.gate = Some(
                    if gene == 0 {
                        "decisive_buy disabled"
                    } else {
                        "Hard defense unavailable"
                    }
                    .into(),
                );
                self.note_option(row);
            }
            return Vec::new();
        }
        let floor = self.decisive_floor();
        let upkeep = self.marginal_military_upkeep();
        let targets: Vec<(&CityView, i32)> = v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && v.at_war(v.me, c.owner))
            .map(|c| (c, self.decisive_capture_vp(c)))
            .filter(|(_, vp)| *vp > 0)
            .collect();
        if targets.is_empty() {
            if self.decision_trace.is_some() {
                let mut row = crate::DecisionOption::new("decisive_buy", "all", "gated_out");
                row.gate = Some("no at-war enemy city with positive capture VP".into());
                self.note_option(row);
            }
            return Vec::new();
        }
        let mut out = Vec::new();
        for (city, vp) in targets {
            let force = self.strike_force_within(v, city, self.decisions.march_turns);
            let planes = self.planes_in_reach(city);
            let base = self.decisive_odds(v, city, &force, &planes);
            // A land unit marches at most `march_turns` from the turn after it is bought.
            let land_reach = (self.decisions.march_turns + 1) * 4 + 2;
            // The view with the land unit we would buy, cloned once per target.
            let mut sim: Option<PlayerView> = None;
            for home in &self.ctx.own_cities {
                let d = home.detail.as_ref().expect("own city detail");
                let distance = home.pos.distance(city.pos);
                if distance > land_reach.max(data::BOMBER_RANGE)
                    || d.resistance > 0
                    || self
                        .orders
                        .iter()
                        .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == home.id))
                {
                    if self.decision_trace.is_some() {
                        let mut row = crate::DecisionOption::new(
                            "decisive_buy",
                            format!("city={:?} target={:?}", home.id, city.id),
                            "gated_out",
                        );
                        row.gate = Some(
                            if distance > land_reach.max(data::BOMBER_RANGE) {
                                "city beyond decisive reach"
                            } else if d.resistance > 0 {
                                "city in resistance"
                            } else {
                                "earlier purchase at city"
                            }
                            .into(),
                        );
                        self.note_option(row);
                    }
                    continue;
                }
                for o in &d.options {
                    let ProductionItem::Unit { unit: kind } = o.item else {
                        continue;
                    };
                    let Some(cost) = o.buy_cost.filter(|c| *c > 0) else {
                        self.decisive_gate(home.id, o.item, city.id, "no positive buy quote");
                        continue;
                    };
                    if o.buy_block.is_some_and(|block| {
                        self.decisions.named_fund == 0
                            || !matches!(block, legatum_core::economy::BuyBlock::NotEnoughGold)
                    }) || !self
                        .view()
                        .rule_experiments
                        .unit_stats_for(self.faction, kind)
                        .military
                        || kind == UnitKind::Scout
                        || data::is_fighter(kind)
                        || self.decisions.named_fund == 0 && self.gold_left - cost < floor
                        || data::pays_upkeep(kind) && v.economy.gold_per_turn.total < upkeep
                    {
                        if self.decision_trace.is_some() {
                            let gate = if let Some(block) = o.buy_block {
                                format!("buy blocked: {block:?}")
                            } else if !self
                                .view()
                                .rule_experiments
                                .unit_stats_for(self.faction, kind)
                                .military
                            {
                                "not military".to_owned()
                            } else if kind == UnitKind::Scout {
                                "Scout excluded".to_owned()
                            } else if data::is_fighter(kind) {
                                "fighter excluded".to_owned()
                            } else if self.decisions.named_fund == 0
                                && self.gold_left - cost < floor
                            {
                                "Gold floor".to_owned()
                            } else {
                                "upkeep runway".to_owned()
                            };
                            self.decisive_gate(home.id, o.item, city.id, gate);
                        }
                        continue;
                    }
                    let risk = if data::is_bomber(kind) {
                        Self::interception_risk(v, city, kind, self.decisions.air_stage_read)
                    } else {
                        0
                    };
                    let odds = if data::is_bomber(kind) {
                        let based = v.aircraft.iter().filter(|a| a.city == home.id).count();
                        if home.pos.distance(city.pos) > data::unit_stats(kind).range
                            || risk == 100
                            || based >= data::AIR_BASE_CAP
                        {
                            self.decisive_gate(
                                home.id,
                                o.item,
                                city.id,
                                if home.pos.distance(city.pos) > data::unit_stats(kind).range {
                                    "bomber range"
                                } else if risk == 100 {
                                    "interception risk 100%"
                                } else {
                                    "air base full"
                                },
                            );
                            continue;
                        }
                        let mut with = planes.clone();
                        with.push((
                            AircraftView {
                                id: BOUGHT_PLANE,
                                owner: v.me,
                                kind,
                                city: home.id,
                                hp: data::UNIT_HP,
                                level: 0,
                                xp: 0,
                                xp_next: None,
                                upgrade_cost: None,
                                upgrade_options: Vec::new(),
                                unsupplied: None,
                            },
                            1,
                        ));
                        self.decisive_odds(v, city, &force, &with)
                    } else if (gene >= 2 || self.decisions.named_fund > 0)
                        && !data::is_aircraft(kind)
                    {
                        let mut unit = crate::purchase_defense::purchased_defender(v, home, kind);
                        unit.id = BOUGHT_UNIT;
                        let Some((tile, turns)) = self.decisive_tile(v, home, city, &unit) else {
                            self.decisive_gate(
                                home.id,
                                o.item,
                                city.id,
                                "land unit has no timely staging tile",
                            );
                            continue;
                        };
                        let sim = sim.get_or_insert_with(|| {
                            let mut sim = v.clone();
                            sim.units.push(unit.clone());
                            sim
                        });
                        *sim.units.last_mut().expect("the bought unit") = unit.clone();
                        let mut with = force.clone();
                        with.push((&unit, tile, turns));
                        self.decisive_odds(sim, city, &with, &planes)
                    } else {
                        self.decisive_gate(
                            home.id,
                            o.item,
                            city.id,
                            "decisive_buy level excludes this unit class",
                        );
                        continue;
                    };
                    let gain = (odds - base).max(0) * vp * (100 - risk) / 100;
                    if gain == 0 {
                        self.decisive_gate(home.id, o.item, city.id, "zero marginal capture gain");
                        continue;
                    }
                    // One purchase can add to several captures: its gain is their sum, shown
                    // with the capture it adds the most to.
                    if let Some(c) = out
                        .iter_mut()
                        .find(|c: &&mut Decisive| c.city == home.id && c.item == o.item)
                    {
                        if gain > c.best {
                            (c.target, c.odds, c.best, c.distance) =
                                (city.id, (base, odds), gain, distance);
                        }
                        c.gain += gain;
                    } else {
                        out.push(Decisive {
                            city: home.id,
                            item: o.item,
                            cost,
                            target: city.id,
                            odds: (base, odds),
                            best: gain,
                            gain,
                            distance,
                            price: self.decisive_price(cost),
                        });
                    }
                }
            }
        }
        out
    }

    /// What a Bomber based at `home` could strike this turn, in `air.rs`'s strike-value units
    /// (damage times the target's worth, per hundred): every enemy military unit and city at war
    /// in its range, each at its best forecast.
    fn strike_reach(&self, home: &CityView, kind: UnitKind) -> i32 {
        let v = self.view();
        let mut sim = v.clone();
        sim.aircraft.push(AircraftView {
            id: BOUGHT_PLANE,
            owner: v.me,
            kind,
            city: home.id,
            hp: data::UNIT_HP,
            level: 0,
            xp: 0,
            xp_next: None,
            upgrade_cost: None,
            upgrade_options: Vec::new(),
            unsupplied: None,
        });
        let range = data::unit_stats(kind).range;
        let units = v
            .units
            .iter()
            .filter(|u| u.military && !u.owner.is_barbarian() && v.at_war(v.me, u.owner))
            .map(|u| (u.pos, crate::tuned(data::unit_stats(u.kind).cost)));
        let cities = v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && v.at_war(v.me, c.owner))
            .map(|c| (c.pos, 120));
        units
            .chain(cities)
            .filter(|(pos, _)| pos.distance(home.pos) <= range)
            .filter_map(|(pos, worth)| {
                legatum_core::forecast_attack_mode(
                    &sim,
                    BOUGHT_PLANE,
                    pos,
                    legatum_core::AttackMode::Ranged,
                )
                .map(|f| f.expected_damage_to_defender * worth / 100)
            })
            .sum()
    }

    /// Buy the decisive unit: the candidate with the largest net gain at least
    /// `decisive_buy_min` VP percent, then the cheaper, then the nearer to its target. A Bomber
    /// goes to the qualifying city whose range holds the most of the war (units and cities it
    /// could strike), not only the captures that priced it: a base on the far side of the front
    /// buys the same plane and strikes little (8.9.24 T140: +0.12 VP from Biel, +1.71 from
    /// Shangdu). One a turn, and none on top of this turn's other military purchase.
    pub(crate) fn buy_decisive(&mut self) -> Option<ProductionItem> {
        let fund = if self.decisions.named_fund > 0 && self.hard_defense() {
            match self.named_fund() {
                Some(fund)
                    if fund.kind == crate::gold_hold::FundKind::Campaign
                        && fund.ready
                        && self.gold_left >= fund.cost =>
                {
                    Some(fund)
                }
                Some(_) => return None,
                None => None,
            }
        } else {
            None
        };
        if self.orders.iter().any(
            |o| matches!(o, Order::BuyItem { item, .. } if crate::production::is_military_unit(*item)),
        ) {
            if self.decision_trace.is_some() {
                let mut row = crate::DecisionOption::new("decisive_buy", "all", "gated_out");
                row.gate = Some("earlier military purchase spent this turn's buy slot".into());
                self.note_option(row);
            }
            return None;
        }
        let candidates = self.decisive_candidates();
        let passing: Vec<Decisive> = candidates
            .iter()
            .filter(|c| c.gain - c.price >= self.decisions.decisive_buy_min)
            .filter(|c| {
                self.ctx
                    .own_cities
                    .iter()
                    .find(|city| city.id == c.city)
                    .is_none_or(|city| !self.military_upgrade_hold(city))
            })
            .filter(|c| fund.is_none_or(|_| self.gold_left >= c.cost))
            .filter(|c| {
                fund.is_none_or(|fund| fund.city == Some(c.city) && fund.item == Some(c.item))
            })
            .cloned()
            .collect();
        let highest = passing
            .iter()
            .filter_map(|c| match c.item {
                ProductionItem::Unit { unit } => Some(data::unit_era_tier(unit)),
                _ => None,
            })
            .max();
        let passing: Vec<Decisive> = if self.decisions.named_fund > 0
            && self.hard_defense()
            && self.decisions.hard_purchase_scorer == 0
        {
            passing
                .into_iter()
                .filter(|c| match c.item {
                    ProductionItem::Unit { unit } => Some(data::unit_era_tier(unit)) == highest,
                    _ => false,
                })
                .collect()
        } else {
            passing
        };
        let mut best = passing
            .iter()
            .max_by_key(|c| {
                let combat = if self.decisions.hard_purchase_scorer > 0
                    && let ProductionItem::Unit { unit } = c.item
                {
                    self.ctx
                        .own_cities
                        .iter()
                        .find(|city| city.id == c.city)
                        .map_or(0, |city| {
                            self.military_purchase_score(unit, c.cost, city.pos)
                        })
                } else {
                    0
                };
                (
                    c.gain - c.price,
                    combat,
                    std::cmp::Reverse(c.cost),
                    std::cmp::Reverse(c.distance),
                    std::cmp::Reverse(c.city),
                )
            })
            .cloned();
        if best.is_none() && self.decision_trace.is_some() {
            for c in &candidates {
                let mut row = crate::DecisionOption::new(
                    "decisive_buy",
                    format!("city={:?} item={:?} target={:?}", c.city, c.item, c.target),
                    "gated_out",
                );
                row.gate = Some(format!(
                    "net gain {} below decisive_buy_min {}",
                    c.gain - c.price,
                    self.decisions.decisive_buy_min
                ));
                row.score = Some(c.gain - c.price);
                self.note_option(row);
            }
        }
        let mut best = best.take()?;
        if let ProductionItem::Unit { unit } = best.item
            && data::is_bomber(unit)
        {
            let reach = |id: CityId| {
                self.ctx
                    .own_cities
                    .iter()
                    .find(|c| c.id == id)
                    .map_or(0, |c| self.strike_reach(c, unit))
            };
            best = passing
                .iter()
                .filter(|c| c.item == best.item)
                .max_by_key(|c| {
                    (
                        reach(c.city),
                        std::cmp::Reverse(c.cost),
                        std::cmp::Reverse(c.city),
                    )
                })
                .expect("the best candidate qualifies")
                .clone();
        }
        if self.decision_trace.is_some() {
            let mut ranking: Vec<i32> = passing.iter().map(|c| c.gain - c.price).collect();
            ranking.sort_unstable_by(|a, b| b.cmp(a));
            for c in &candidates {
                let qualifies = c.gain - c.price >= self.decisions.decisive_buy_min;
                let selected =
                    c.city == best.city && c.item == best.item && c.target == best.target;
                let mut row = crate::DecisionOption::new(
                    "decisive_buy",
                    format!("city={:?} item={:?} target={:?}", c.city, c.item, c.target),
                    if !qualifies {
                        "gated_out"
                    } else if selected {
                        "selected"
                    } else {
                        "considered_rejected"
                    },
                );
                row.score = Some(c.gain - c.price);
                row.winner_score = ranking.first().copied();
                row.runner_up_score = ranking.get(1).copied();
                if !qualifies {
                    row.gate = Some(format!(
                        "net gain below decisive_buy_min {}",
                        self.decisions.decisive_buy_min
                    ));
                }
                if selected {
                    row.selected_order = Some(Order::BuyItem {
                        city: c.city,
                        item: c.item,
                    });
                }
                self.note_option(row);
            }
        }
        self.gold_left -= best.cost;
        self.orders.push(Order::BuyItem {
            city: best.city,
            item: best.item,
        });
        Some(best.item)
    }
}

#[cfg(test)]
mod air_read_tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::PlayerId;
    use legatum_core::view::AirSighting;

    #[test]
    fn modern_is_uncertain_and_a_seen_fighter_changes_the_estimate() {
        let rival = PlayerId(2);
        let mut view = ViewBuilder::new(24, 16)
            .city(1, ME, off(3, 8), 3)
            .city(2, rival, off(12, 8), 3)
            .era_of(2, data::era_index(TechId::Flight))
            .build();
        let target = view
            .cities
            .iter()
            .find(|c| c.owner == rival)
            .unwrap()
            .clone();
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::Bomber, 0),
            100
        );
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::Bomber, 1),
            25
        );
        view.air_contacts.push(legatum_core::state::AirContact {
            owner: rival,
            kind: Some(UnitKind::Fighter),
            turn: view.turn,
            city: None,
        });
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::Bomber, 1),
            50
        );
        view.air_contacts[0].city = Some(target.id);
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::Bomber, 1),
            60
        );
        view.air_sightings.push(AirSighting {
            owner: rival,
            kind: UnitKind::Fighter,
            city: target.id,
        });
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::Bomber, 1),
            75
        );
        assert_eq!(
            Planner::interception_risk(&view, &target, UnitKind::StealthBomber, 1),
            25
        );
    }
}

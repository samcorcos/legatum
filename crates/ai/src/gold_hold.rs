//! Gene `buy_hold`: a purchase competes with keeping the Gold (decision audit 2026-10-31: in
//! the designer's seat Hard made 80 to 396 purchases a game, he made 0 to 7 and banked, and his
//! production and buys beat Hard's by +0.13 cities and +0.15 VP per 10-turn window).
//!
//! A purchase is a full-price copy at 200% of Production (175% Swiss), so Gold spent today and
//! Gold spent later convert at the same rate; what differs is the moment. Gold kept pays most at
//! the next era's unit spike (upgrades) and in an emergency (a defender for a city about to
//! fall). So the routine passes spend only the Gold above those two uses: the upgrade bill for
//! units whose newest form is researched or due within `SPIKE_TURNS`, and, while at war or under
//! threat, the cheapest defender for each threatened city (one at war with nothing in sight,
//! two at most). The upgrade, rescue, door, threat, victory, faction goal and hoard passes don't
//! see the hold, so the kept Gold is spent where it pays; the hold is bounded by `INCOME_TURNS`
//! of income, so a treasury never piles up past what those uses take. At 2 the readiness and
//! frontline defender purchases also leave the hold unless an attack can reach the city this
//! turn (`defense_risk`). At 3 the hold also banks the army spike: the dearest unit we can buy
//! now for each of up to four cities, which the war's purchases or the cap release.
use crate::composition;
use crate::plan::PlanKind;
use crate::scripted::Planner;
use legatum_core::{AiDifficulty, CityId, Order, ProductionItem, TechId, UnitKind, data};
use std::collections::BTreeMap;

/// Research turns ahead an upgrade's technology may be and still count toward the spike.
const SPIKE_TURNS: i32 = 10;
/// The hold never exceeds this many turns of income.
const INCOME_TURNS: i32 = 25;
/// Defenders the emergency part keeps Gold for.
const MAX_DEFENDERS: i32 = 2;
/// Cities whose era unit the level-3 army spike banks for.
const SPIKE_CITIES: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FundKind {
    Campaign,
    Upgrade(TechId),
    Emergency,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NamedFund {
    pub kind: FundKind,
    pub cost: i32,
    pub city: Option<CityId>,
    pub item: Option<ProductionItem>,
    pub ready: bool,
}

impl Planner<'_> {
    /// Purchases are ordered before units exist. Keep the fund's destination in the
    /// saved observation memory and bind each recruit by city, kind and creation turn.
    pub(crate) fn record_fund_deliveries(&mut self) {
        let Some(fund) = self.named_fund() else {
            return;
        };
        let campaign_target = self.routed_plan().and_then(|plan| match &plan.kind {
            PlanKind::Campaign { target, .. } => Some(*target),
            PlanKind::Contain { against } => self
                .ctx
                .enemy_cities
                .iter()
                .filter(|c| self.view().team_of(c.owner) == *against)
                .min_by_key(|c| {
                    (
                        self.ctx
                            .own_cities
                            .iter()
                            .map(|home| home.pos.distance(c.pos))
                            .min()
                            .unwrap_or(i32::MAX),
                        c.id,
                    )
                })
                .map(|c| c.id),
            _ => None,
        });
        let mut deliveries = Vec::new();
        for order in &self.orders {
            match order {
                Order::BuyItem {
                    city,
                    item: ProductionItem::Unit { unit },
                } if composition::is_combat(*unit) => {
                    let (destination, offensive) = match fund.kind {
                        FundKind::Emergency => (fund.city, false),
                        FundKind::Campaign
                            if fund.city == Some(*city)
                                && fund.item == Some(ProductionItem::Unit { unit: *unit }) =>
                        {
                            (campaign_target, true)
                        }
                        FundKind::Campaign | FundKind::Upgrade(_) => continue,
                    };
                    let Some(destination) = destination else {
                        continue;
                    };
                    if self
                        .ctx
                        .own_cities
                        .iter()
                        .copied()
                        .find(|c| c.id == *city)
                        .is_some_and(|c| self.defense_risk(c, None).falls)
                    {
                        continue;
                    }
                    deliveries.push(crate::campaign::FundDelivery {
                        ordered_turn: self.view().turn,
                        bought_in: Some(*city),
                        kind: Some(*unit),
                        unit: None,
                        destination,
                        offensive,
                    });
                }
                Order::UpgradeUnit { unit } if matches!(fund.kind, FundKind::Upgrade(_)) => {
                    let destination = campaign_target.or_else(|| {
                        let at = self.ctx.own_units.iter().find(|u| u.id == *unit)?.pos;
                        self.ctx
                            .enemy_cities
                            .iter()
                            .filter(|c| self.view().at_war(self.view().me, c.owner))
                            .min_by_key(|c| (c.pos.distance(at), c.id))
                            .map(|c| c.id)
                    });
                    let Some(destination) = destination else {
                        continue;
                    };
                    deliveries.push(crate::campaign::FundDelivery {
                        ordered_turn: self.view().turn,
                        bought_in: None,
                        kind: None,
                        unit: Some(*unit),
                        destination,
                        offensive: true,
                    });
                }
                _ => {}
            }
        }
        self.campaigns.fund_deliveries.extend(deliveries);
    }

    /// One target from our routed war, imminent research, or the army visible at our door.
    /// Quotes come from our own city options; no hidden state enters the fund.
    pub(crate) fn named_fund(&self) -> Option<NamedFund> {
        *self.named_fund.get_or_init(|| self.compute_named_fund())
    }

    fn compute_named_fund(&self) -> Option<NamedFund> {
        if self.decisions.named_fund == 0 || self.difficulty != AiDifficulty::Hard {
            return None;
        }
        let v = self.view();
        let emergency = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| {
                let nearest = v
                    .units
                    .iter()
                    .filter(|u| {
                        u.military
                            && u.kind != UnitKind::Scout
                            && !u.owner.is_barbarian()
                            && v.at_war(v.me, u.owner)
                    })
                    .map(|u| u.pos.distance(c.pos))
                    .min()?;
                if nearest > 8 || c.detail.as_ref()?.resistance > 0 {
                    return None;
                }
                let held = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military && u.kind != UnitKind::Scout && u.pos.distance(c.pos) <= 2
                    })
                    .count() as i32;
                let need = (2 - held).max(0);
                if need == 0 {
                    return None;
                }
                let best = c
                    .detail
                    .as_ref()?
                    .options
                    .iter()
                    .filter_map(|o| {
                        let ProductionItem::Unit { unit } = o.item else {
                            return None;
                        };
                        if !crate::production::is_military_unit(o.item)
                            || unit == UnitKind::Scout
                            || o.buy_block.is_some_and(|b| {
                                !matches!(b, legatum_core::economy::BuyBlock::NotEnoughGold)
                            })
                        {
                            return None;
                        }
                        let cost = o.buy_cost.filter(|cost| *cost > 0)?;
                        let strength = v
                            .rule_experiments
                            .unit_stats_for(self.faction, unit)
                            .strength;
                        let survives = v
                            .units
                            .iter()
                            .filter(|u| {
                                u.military
                                    && !u.owner.is_barbarian()
                                    && v.at_war(v.me, u.owner)
                                    && u.pos.distance(c.pos) <= 8
                            })
                            .all(|u| {
                                legatum_core::combat::base_damage(u.strength, strength)
                                    < data::UNIT_HP
                            });
                        let rank = if self.decisions.hard_purchase_scorer > 0 {
                            self.military_purchase_score(unit, cost, c.pos)
                        } else {
                            i32::from(data::unit_era_tier(unit))
                        };
                        (survives && rank >= 0).then_some((rank, strength, -cost, o.item, cost))
                    })
                    .max_by_key(|(rank, strength, neg_cost, _, _)| (*rank, *strength, *neg_cost))?;
                Some((nearest, c.id, need, best.3, best.4))
            })
            .min_by_key(|(nearest, city, _, _, _)| (*nearest, *city));
        if let Some((distance, city, need, item, quote)) = emergency {
            return Some(NamedFund {
                kind: FundKind::Emergency,
                cost: need * quote,
                city: Some(city),
                item: Some(item),
                ready: distance <= 3,
            });
        }

        let mut waves: BTreeMap<TechId, i32> = BTreeMap::new();
        for u in &self.ctx.own_units {
            if !composition::is_combat(u.kind)
                || u.contract_until.is_some()
                || u.upgrade_supply_blocker.is_some()
            {
                continue;
            }
            let Some(to) = data::unit_stats(u.kind).upgrades_to else {
                continue;
            };
            let Some(tech) = data::unit_stats(to).requires else {
                continue;
            };
            let due = v.research.known.contains(&tech)
                || v.research.current == Some(tech)
                    && v.research.turns_left.is_some_and(|n| n <= SPIKE_TURNS);
            if !due {
                continue;
            }
            let Some(base_cost) =
                legatum_core::rules::units::upgrade_cost_for(self.faction, u.kind)
            else {
                continue;
            };
            let forecast = if v.research.known.contains(&TechId::MilitaryEngineering) {
                base_cost / 2
            } else {
                base_cost
            };
            let cost = if v.research.known.contains(&tech) {
                u.upgrade_breakdown.as_ref().map_or(forecast, |b| b.total)
            } else {
                forecast
            };
            *waves.entry(tech).or_default() += cost;
        }
        let wave = waves
            .into_iter()
            .max_by_key(|(_, cost)| *cost)
            .map(|(tech, cost)| NamedFund {
                kind: FundKind::Upgrade(tech),
                cost,
                city: None,
                item: None,
                ready: v.research.known.contains(&tech),
            });
        if wave.is_some() {
            return wave;
        }

        if let Some(plan) = self.routed_plan() {
            let target_team = match &plan.kind {
                PlanKind::Campaign { against, .. } | PlanKind::Contain { against } => {
                    Some(*against)
                }
                _ => None,
            };
            if let Some(team) = target_team {
                let best = self
                    .decisive_candidates()
                    .into_iter()
                    .filter(|candidate| {
                        candidate.gain - candidate.price >= self.decisions.decisive_buy_min
                    })
                    .filter(|candidate| {
                        v.cities
                            .iter()
                            .any(|c| c.id == candidate.target && v.team_of(c.owner) == team)
                            && match &plan.kind {
                                PlanKind::Campaign { target, .. } => candidate.target == *target,
                                _ => true,
                            }
                    })
                    .filter(|candidate| {
                        if self.decisions.purchase_generation_floor == 0 {
                            return true;
                        }
                        let ProductionItem::Unit { unit } = candidate.item else {
                            return true;
                        };
                        self.ctx
                            .own_cities
                            .iter()
                            .find(|c| c.id == candidate.city)
                            .is_some_and(|c| self.purchase_generation(c, unit) == Some(unit))
                    })
                    .max_by_key(|candidate| {
                        let unit = match candidate.item {
                            ProductionItem::Unit { unit } => unit,
                            _ => UnitKind::Scout,
                        };
                        let rank = if self.decisions.hard_purchase_scorer > 0 {
                            self.ctx
                                .own_cities
                                .iter()
                                .find(|c| c.id == candidate.city)
                                .map_or(0, |c| {
                                    self.military_purchase_score(unit, candidate.cost, c.pos)
                                })
                        } else {
                            i32::from(data::unit_era_tier(unit))
                        };
                        (
                            if self.decisions.hard_purchase_scorer > 0 {
                                candidate.gain - candidate.price
                            } else {
                                rank
                            },
                            if self.decisions.hard_purchase_scorer > 0 {
                                rank
                            } else {
                                candidate.gain - candidate.price
                            },
                            -candidate.cost,
                        )
                    });
                if let Some(candidate) = best {
                    let staged = v
                        .cities
                        .iter()
                        .find(|c| c.id == candidate.target)
                        .is_some_and(|target| {
                            self.ctx.own_units.iter().any(|u| {
                                u.military
                                    && u.pos.distance(target.pos) <= self.decisions.march_turns * 3
                            })
                        });
                    return Some(NamedFund {
                        kind: FundKind::Campaign,
                        cost: candidate.cost,
                        city: Some(candidate.city),
                        item: Some(candidate.item),
                        ready: staged,
                    });
                }
            }
        }

        None
    }

    /// The Gold routine purchases leave for the spike and emergencies (0 with the gene off).
    pub(crate) fn held_gold(&self) -> i32 {
        if let Some(fund) = self.named_fund() {
            return fund.cost;
        }
        if self.decisions.buy_hold == 0 {
            return 0;
        }
        if let Some(held) = self.held_gold.get() {
            return held;
        }
        let v = self.view();
        let known = |t: TechId| v.research.known.contains(&t);
        let due = |t: TechId| {
            known(t)
                || v.research.current == Some(t)
                    && v.research.turns_left.is_some_and(|n| n <= SPIKE_TURNS)
        };
        let halve = known(TechId::MilitaryEngineering);
        let spike: i32 = self
            .ctx
            .own_units
            .iter()
            .filter(|u| composition::is_combat(u.kind) && u.contract_until.is_none())
            .filter_map(|u| {
                let mut kind = u.kind;
                let mut total = 0;
                while let Some(to) = data::unit_stats(kind).upgrades_to {
                    if !data::unit_stats(to).requires.is_none_or(due) {
                        break;
                    }
                    let cost = legatum_core::rules::units::upgrade_cost_for(self.faction, kind)?;
                    total += if halve { cost / 2 } else { cost };
                    kind = to;
                }
                (total > 0).then_some(total)
            })
            .sum();
        // Emergencies: threatened cities, or one defender while at war with nothing in sight.
        let threatened = (0..self.ctx.own_cities.len())
            .filter(|i| self.ctx.threat[*i] - self.ctx.barbarian_threat[*i] > 0)
            .count() as i32;
        let at_war = v
            .players
            .iter()
            .any(|p| p.alive && p.id != v.me && v.at_war(v.me, p.id));
        let defenders = if threatened > 0 {
            threatened.min(MAX_DEFENDERS)
        } else {
            i32::from(at_war)
        };
        let quotes: Vec<(u8, i32, i32)> = self
            .ctx
            .own_cities
            .iter()
            .filter_map(|c| c.detail.as_ref())
            .flat_map(|d| d.options.iter())
            .filter(|o| {
                matches!(o.item, ProductionItem::Unit { .. })
                    && crate::production::is_military_unit(o.item)
            })
            .filter_map(|o| {
                let ProductionItem::Unit { unit } = o.item else {
                    return None;
                };
                let cost = o.buy_cost.filter(|c| *c > 0)?;
                let strength = v
                    .rule_experiments
                    .unit_stats_for(self.faction, unit)
                    .strength;
                Some((data::unit_era_tier(unit), strength, cost))
            })
            .collect();
        let quote = if self.decisions.named_fund > 0 && self.difficulty == AiDifficulty::Hard {
            quotes
                .iter()
                .max_by_key(|q| (q.0, q.1, -q.2))
                .map_or(0, |q| q.2)
        } else {
            quotes.iter().map(|q| q.2).min().unwrap_or(0)
        };
        // Level 3 banks for the army spike too: the dearest unit we can buy now (the era's
        // best) for each city, up to `SPIKE_CITIES`, bought when the war or the era comes.
        let army = if self.decisions.buy_hold >= 3 {
            let cities = (self.ctx.own_cities.len() as i32).min(SPIKE_CITIES);
            cities * quotes.iter().map(|q| q.2).max().unwrap_or(0)
        } else {
            0
        };
        let cap = v.economy.gold_per_turn.total.max(0) * INCOME_TURNS;
        let held = (spike + defenders * quote + army).min(cap);
        self.held_gold.set(Some(held));
        held
    }
}

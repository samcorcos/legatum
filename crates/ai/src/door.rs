//! Defend where the enemy army is (gene `door_defense`; idea #23, AI war plan W2).
//!
//! The 8.9.13 Sparta review: every city fell in one to four turns to two to six units, the
//! defenders bought three units in eighty turns of war, and no lost city was ever retaken. A
//! bigger standing army backfired against an adaptive Sparta, because the defenders paid for
//! it with their economy. So this answers only the army we can see at the door, sized to it:
//!
//! - **The front.** A visible enemy field unit that can reach one of our cities within
//!   `DOOR_TURNS` turns of march threatens the nearest of them (plus `threat_aim_slack`). Its
//!   threat is its Strongest Army value (strength squared, HP-weighted, so an era's lead
//!   counts as it does in a fight), times `door_defense` percent; the defence is the same
//!   value of our units within `DOOR_GUARD_RADIUS` plus the city's own. The gap is what to
//!   close, and nothing is spent once it is closed.
//! - **Gold.** The era's best unit bought in the city, or a Gold upgrade of a unit already
//!   there that has no target this turn, whichever closes more of the gap per Gold. Only
//!   solvency and a held Swiss hoard are kept back: a goal still being saved for doesn't
//!   outweigh the city.
//! - **Production.** A front still short builds the strongest unit it can finish soon.
//! - **Troops.** The nearest free unit that can reach an empty threatened city this turn
//!   goes in (while the city has HP it shields its garrison). Then the attackers' few or
//!   wounded units near our cities are struck with everything in reach when the forecast
//!   kills them and no striker is left to die.
//! - **Retaking.** A city of the enemy's beside ours at half HP or less, whose visible
//!   garrison our nearby force outweighs, is assaulted (`coordinate_city_assaults_at`).
//!
//! Everything reads only this turn's view and keeps no memory: when the army leaves, the
//! front is gone and so is the response.

use crate::composition;
use crate::scripted::Planner;
use crate::tactics::CityShield;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    AttackMode, BuildingKind, CityId, Hex, Order, ProductionItem, UnitKind, army, data,
    forecast_attack_mode,
};

/// Turns of march within which a visible enemy unit threatens a city.
pub(crate) const DOOR_TURNS: i32 = 3;
/// Own units this close to a city count as its defence (and may be upgraded for it).
pub(crate) const DOOR_GUARD_RADIUS: i32 = 2;
/// Enemy units this close to one of our cities are focused-strike targets.
const DOOR_STRIKE_RADIUS: i32 = 3;
/// A focused strike needs this percent of the target's HP in expected damage.
const DOOR_KILL_MARGIN: i32 = 115;
/// A striker with a higher forecast chance of dying stays out.
const DOOR_STRIKER_LOSS: u32 = 30;

/// Parts of the door (gene `door_parts`, a bit mask), so each can be measured alone.
pub(crate) const DOOR_BUY: i32 = 1;
pub(crate) const DOOR_UPGRADE: i32 = 2;
pub(crate) const DOOR_PRODUCTION: i32 = 4;
pub(crate) const DOOR_STRIKE: i32 = 8;
pub(crate) const DOOR_REINFORCE: i32 = 16;
pub(crate) const DOOR_RETAKE: i32 = 32;

/// One threatened city.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DoorFront {
    pub city: CityId,
    pub pos: Hex,
    /// Squared-strength value of the enemy units that can arrive, times `door_defense` / 100.
    pub threat: i32,
    /// The same value of our units within `DOOR_GUARD_RADIUS` and the city itself.
    pub defense: i32,
    /// Fewest turns any of those enemy units needs to strike the city.
    pub arrival: i32,
}

impl DoorFront {
    pub fn gap(&self) -> i32 {
        self.threat - self.defense
    }
}

/// Turns `e` needs to strike `target` (a shooter from its range, a melee unit from beside it).
fn strike_turns(e: &UnitView, target: Hex) -> i32 {
    let reach = if e.ranged > 0 && e.range > 0 {
        e.range
    } else {
        1
    };
    let steps = (e.pos.distance(target) - reach).max(0);
    let moves = e.max_moves.max(1);
    (steps + moves - 1) / moves
}

pub(crate) fn unit_value(u: &UnitView) -> i32 {
    army::combat_value(u.strength, u.ranged, u.hp, u.level, u.unsupplied.is_some())
}

fn city_value(c: &CityView) -> i32 {
    let full = army::full_health_value(c.strength);
    (i64::from(full) * i64::from(c.hp.max(0)) / i64::from(c.max_hp.max(1))) as i32
}

impl Planner<'_> {
    pub(crate) fn door_active(&self) -> bool {
        self.decisions.door_defense > 0 && self.threat_aware()
    }

    fn door_part(&self, part: i32) -> bool {
        self.decisions.door_parts & part != 0
    }

    /// An enemy field unit (not a garrison, a Scout, a civilian or a barbarian).
    fn door_attacker(&self, e: &UnitView) -> bool {
        e.military
            && composition::is_combat(e.kind)
            && !e.owner.is_barbarian()
            && !self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos)
    }

    /// Every own city a visible enemy army can strike within `DOOR_TURNS`, most urgent first.
    pub(crate) fn door_fronts(&self) -> Vec<DoorFront> {
        if !self.door_active() {
            return Vec::new();
        }
        let weight = self.decisions.door_defense;
        let mut threat = vec![0i64; self.ctx.own_cities.len()];
        let mut arrival = vec![i32::MAX; self.ctx.own_cities.len()];
        for e in self
            .ctx
            .enemy_units
            .iter()
            .filter(|e| self.door_attacker(e))
        {
            let reach: Vec<(usize, i32, i32)> = self
                .ctx
                .own_cities
                .iter()
                .enumerate()
                .filter(|(_, c)| self.ctx.reachable(e.pos, c.pos))
                .map(|(i, c)| (i, e.pos.distance(c.pos), strike_turns(e, c.pos)))
                .filter(|(_, _, turns)| *turns <= DOOR_TURNS)
                .collect();
            let Some(nearest) = reach.iter().map(|(_, d, _)| *d).min() else {
                continue;
            };
            let value = i64::from(unit_value(e));
            for (i, d, turns) in reach {
                if d <= nearest + self.decisions.threat_aim_slack {
                    threat[i] += value;
                    arrival[i] = arrival[i].min(turns);
                }
            }
        }
        let mut fronts: Vec<DoorFront> = self
            .ctx
            .own_cities
            .iter()
            .enumerate()
            .filter(|(i, _)| threat[*i] > 0)
            .map(|(i, c)| {
                let guards: i64 = self
                    .ctx
                    .own_units
                    .iter()
                    .filter(|u| {
                        u.military
                            && composition::is_combat(u.kind)
                            && u.pos.distance(c.pos) <= DOOR_GUARD_RADIUS
                    })
                    .map(|u| i64::from(unit_value(u)))
                    .sum();
                DoorFront {
                    city: c.id,
                    pos: c.pos,
                    threat: (threat[i] * i64::from(weight) / 100).min(i64::from(i32::MAX)) as i32,
                    defense: (guards + i64::from(city_value(c))).min(i64::from(i32::MAX)) as i32,
                    arrival: arrival[i],
                }
            })
            .collect();
        fronts.sort_by_key(|f| (f.arrival, std::cmp::Reverse(f.gap()), f.city));
        fronts
    }

    /// Gold the door may not spend: solvency, a held Swiss hoard and a live goal's floor.
    fn door_gold_floor(&self) -> i32 {
        let deficit = (-self.view().economy.gold_per_turn.total).max(0);
        let solvency = self.decisions.reserve_deficit_turns * deficit;
        let hoard = if self.hoard_held() {
            legatum_core::goals::SWISS_HOARD_GOLD + deficit
        } else {
            0
        };
        solvency.max(hoard).max(self.live_goal_floor())
    }

    /// The strength a unit of `kind` brings to `pos` against the foes seen there.
    fn door_power(&self, kind: UnitKind, level: u8, hp: i32, pos: Hex) -> i32 {
        self.modernization_power(kind, level, hp, false, pos)
    }

    /// Whether `u` has a visible enemy it could strike this turn (then it fights, not refits).
    fn door_engaged(&self, u: &UnitView) -> bool {
        let reach = if u.ranged > 0 && u.range > 0 {
            u.range
        } else {
            u.max_moves.max(1)
        };
        self.ctx
            .enemy_units
            .iter()
            .any(|e| e.military && e.pos.distance(u.pos) <= reach)
    }

    /// Gold and Production for the threatened cities, sized to each front's gap.
    /// Runs in `plan_cities` after the existing rescue purchases, before optional spending.
    pub(crate) fn plan_door_defense(&mut self) {
        let mut fronts = self.door_fronts();
        fronts.retain(|f| f.gap() > 0);
        if fronts.is_empty() {
            return;
        }
        let floor = self.door_gold_floor();
        let sparta_level = u8::from(self.faction == legatum_core::Faction::Sparta);
        // Candidates the strategic-supply budget refused this turn.
        let mut refused: Vec<Order> = Vec::new();
        loop {
            // (score, gap closed, cost, order)
            let mut best: Option<(i64, usize, i32, i32, Order)> = None;
            for (fi, f) in fronts.iter().enumerate() {
                let gap = f.gap();
                if gap <= 0 {
                    continue;
                }
                let Some(c) = self.ctx.own_cities.iter().copied().find(|c| c.id == f.city) else {
                    continue;
                };
                let Some(d) = c.detail.as_ref() else {
                    continue;
                };
                let mut consider =
                    |gain: i32, cost: i32, score_override: Option<i64>, order: Order| {
                        if gain <= 0 || cost <= 0 || refused.contains(&order) {
                            return;
                        }
                        let score = score_override
                            .unwrap_or_else(|| i64::from(gain.min(gap)) * 1000 / i64::from(cost));
                        if best.as_ref().is_none_or(|b| score > b.0) {
                            best = Some((score, fi, gain, cost, order));
                        }
                    };
                let bought_here = self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == c.id));
                if self.door_part(DOOR_BUY)
                    && !bought_here
                    && d.resistance <= 0
                    && !self.military_upgrade_hold(c)
                {
                    let upkeep_floor = ((self.marginal_military_upkeep()
                        - self.view().economy.gold_per_turn.total)
                        .max(0)
                        * 10)
                        .max(floor);
                    for o in &d.options {
                        let ProductionItem::Unit { unit } = o.item else {
                            continue;
                        };
                        if !composition::is_combat(unit) || o.buy_block.is_some() {
                            continue;
                        }
                        let Some(cost) = o.buy_cost.filter(|c| *c > 0) else {
                            continue;
                        };
                        if self.gold_left - cost < upkeep_floor {
                            continue;
                        }
                        if !self.military_purchase_viable(c, unit, cost) {
                            continue;
                        }
                        let gain = self.door_power(unit, sparta_level, data::UNIT_HP, c.pos);
                        consider(
                            gain,
                            cost,
                            (self.decisions.hard_purchase_scorer > 0
                                && self.difficulty == legatum_core::AiDifficulty::Hard)
                                .then(|| {
                                    i64::from(self.military_purchase_score(unit, cost, c.pos))
                                        * i64::from(gain.min(gap))
                                        / i64::from(gain.max(1))
                                }),
                            Order::BuyItem {
                                city: c.id,
                                item: o.item,
                            },
                        );
                    }
                }
                for u in self.ctx.own_units.iter().copied() {
                    if !self.door_part(DOOR_UPGRADE)
                        || u.pos.distance(c.pos) > DOOR_GUARD_RADIUS
                        || !composition::is_combat(u.kind)
                        || self.has_acted(u.id)
                        || u.contract_until.is_some()
                        || u.upgrade_supply_blocker.is_some()
                        || self.door_engaged(u)
                    {
                        continue;
                    }
                    let Some(cost) = u.upgrade_cost.filter(|c| *c > 0) else {
                        continue;
                    };
                    let Some(to) = data::unit_stats(u.kind).upgrades_to else {
                        continue;
                    };
                    if self.gold_left - cost < floor {
                        continue;
                    }
                    let gain = self.door_power(to, u.level, u.hp, u.pos)
                        - self.door_power(u.kind, u.level, u.hp, u.pos);
                    consider(gain, cost, None, Order::UpgradeUnit { unit: u.id });
                }
            }
            let Some((_, fi, gain, cost, order)) = best else {
                break;
            };
            let mut orders = self.orders.clone();
            orders.push(order.clone());
            if !crate::supply::within_budget(self.view(), orders).contains(&order) {
                refused.push(order);
                continue;
            }
            self.gold_left -= cost;
            fronts[fi].defense += gain;
            match order {
                Order::UpgradeUnit { unit } => {
                    if let Some(u) = self.ctx.own_units.iter().copied().find(|u| u.id == unit) {
                        self.mil_dest.insert(u.pos);
                    }
                    self.unit_order(unit, order);
                }
                _ => self.orders.push(order),
            }
        }
        if self.door_part(DOOR_PRODUCTION) {
            self.door_production(&fronts);
        }
    }

    /// A front still short of its gap builds the strongest unit it can finish before long.
    fn door_production(&mut self, fronts: &[DoorFront]) {
        for f in fronts.iter().filter(|f| f.gap() > 0) {
            let Some(c) = self.ctx.own_cities.iter().copied().find(|c| c.id == f.city) else {
                continue;
            };
            let Some(d) = c.detail.as_ref() else {
                continue;
            };
            if d.resistance > 0
                || self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == c.id))
            {
                continue;
            }
            let fall_turn = (self.decisions.siege_survival_clock > 0 && self.hard_defense())
                .then(|| self.defense_risk(c, None).production_deadline())
                .flatten();
            let producing_unit = matches!(d.producing, Some(ProductionItem::Unit { unit })
                if composition::is_combat(unit));
            let walls_under_way = self.decisions.keep_walls > 0
                && matches!(d.producing, Some(ProductionItem::Building { building })
                    if data::building_family(building) == BuildingKind::Walls);
            let current_finishes =
                fall_turn.is_none_or(|fall| d.turns_to_complete.is_some_and(|turns| turns < fall));
            if (producing_unit || walls_under_way) && current_finishes
                || self.keep_committed_settler(c) && fall_turn.is_none()
            {
                self.door_cities.push(c.id);
                continue;
            }
            let horizon = f.arrival + DOOR_TURNS;
            let level = u8::from(self.faction == legatum_core::Faction::Sparta);
            let best = d
                .options
                .iter()
                .filter(|o| {
                    o.turns > 0
                        && o.turns <= horizon
                        && fall_turn.is_none_or(|fall| o.turns < fall)
                        && o.resource_blocker.is_none()
                        && matches!(o.item, ProductionItem::Unit { unit } if composition::is_combat(unit))
                })
                .max_by_key(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        unreachable!()
                    };
                    (
                        i64::from(self.door_power(unit, level, data::UNIT_HP, c.pos))
                            * i64::from(fall_turn.map_or(1, |fall| (fall - o.turns).max(1)))
                            * 1000 / i64::from(o.turns + 1),
                        std::cmp::Reverse(o.cost),
                        std::cmp::Reverse(o.item),
                    )
                });
            if let Some(o) = best {
                self.door_cities.push(c.id);
                self.orders.push(Order::SetProduction {
                    city: c.id,
                    item: o.item,
                });
            }
        }
    }

    /// Military passes of the door, run early in `plan_military`: focused strikes on the
    /// attackers near our cities, then a garrison for each empty threatened city.
    pub(crate) fn door_military(&mut self, army: &[&UnitView]) {
        if !self.door_active() {
            return;
        }
        // A lost city is retaken whether or not an army stands at our other cities.
        if self.door_part(DOOR_RETAKE) {
            self.door_retake(army);
        }
        let fronts = self.door_fronts();
        if fronts.is_empty() {
            return;
        }
        self.rotate_exposed_garrisons(army, &fronts);
        if self.door_part(DOOR_STRIKE) {
            self.door_strikes(army);
        }
        if self.door_part(DOOR_REINFORCE) {
            self.door_reinforce(army, &fronts);
        }
    }

    /// A city whose present garrison is forecast to fall can exchange places with a
    /// stronger defender on the next tile. Friendly two-unit swaps resolve together;
    /// this lets a line unit take the city before its remaining shield is exhausted.
    /// The adjacent free melee unit that should swap places with `guard`, the unit in `city`:
    /// only while the city would fall, lose its garrison or have its shield broken with the
    /// guard in it, and only a replacement forecast to hold (and, when the guard alone would
    /// hold, one with greater defense value against the adjacent attackers). Shared by
    /// `rotate_exposed_garrisons` and `plan.rs::execute_hold`.
    pub(crate) fn garrison_swap<'u>(
        &self,
        city: &CityView,
        guard: &UnitView,
        free: &[&'u UnitView],
    ) -> Option<&'u UnitView> {
        let current = self.defense_risk(city, None);
        let shield_breaks = self.city_shield(city.pos) == CityShield::MeleeOnly;
        if !current.falls && !current.garrison_lost && !shield_breaks {
            return None;
        }
        let defense_value = |u: &UnitView| {
            crate::analysis::defense_power(self.view(), u)
                + self
                    .ctx
                    .threat_units
                    .iter()
                    .filter(|e| e.military && e.pos.distance(city.pos) <= 1)
                    .map(|e| {
                        legatum_core::combat::class_bonus_for_rules(
                            self.view().rule_experiments.faction_rules_off,
                            self.faction,
                            u.kind,
                            e.kind,
                        )
                        .1
                    })
                    .sum::<i32>()
        };
        let guard_value = defense_value(guard);
        free.iter()
            .copied()
            .filter(|u| {
                u.id != guard.id
                    && u.pos.distance(city.pos) == 1
                    && composition::charges(u.kind)
                    && self.ctx.move_cost(city.pos).is_some_and(|cost| {
                        legatum_core::rules::movement::can_step(u.max_moves, u.max_moves, cost)
                    })
                    && self.ctx.move_cost(u.pos).is_some_and(|cost| {
                        legatum_core::rules::movement::can_step(
                            guard.max_moves,
                            guard.max_moves,
                            cost,
                        )
                    })
                    && !self.mil_dest.contains(&u.pos)
            })
            .filter_map(|u| {
                let risk = self.defense_risk(city, Some(u.clone()));
                ((risk.holds_longer_than(current) || !risk.falls)
                    && !risk.garrison_lost
                    && (current.falls || current.garrison_lost || defense_value(u) > guard_value))
                    .then_some((defense_value(u), u))
            })
            .max_by_key(|(value, u)| (*value, u.hp, std::cmp::Reverse(u.id)))
            .map(|(_, u)| u)
    }

    fn rotate_exposed_garrisons(&mut self, army: &[&UnitView], fronts: &[DoorFront]) {
        for front in fronts {
            let Some(city) = self
                .ctx
                .own_cities
                .iter()
                .copied()
                .find(|c| c.id == front.city)
            else {
                continue;
            };
            if self.mil_dest.contains(&city.pos) {
                continue;
            }
            let Some(guard) = self.free(army).into_iter().find(|u| u.pos == city.pos) else {
                continue;
            };
            let replacement = self.garrison_swap(city, guard, &self.free(army));
            if let Some(u) = replacement {
                self.unit_order(
                    guard.id,
                    Order::MoveUnit {
                        unit: guard.id,
                        to: u.pos,
                    },
                );
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: city.pos,
                    },
                );
                self.mil_dest.insert(u.pos);
                self.mil_dest.insert(city.pos);
            }
        }
    }

    /// The attackers' units near our cities that everything in reach can kill this turn,
    /// highest threat first, when no striker is forecast to die and the kill is worth the damage.
    fn door_strikes(&mut self, army: &[&UnitView]) {
        let mut targets: Vec<&UnitView> = self
            .ctx
            .enemy_units
            .iter()
            .copied()
            .filter(|e| {
                self.door_attacker(e)
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| c.pos.distance(e.pos) <= DOOR_STRIKE_RADIUS)
                    && self.view().tile(e.pos).is_some_and(|t| t.visible)
            })
            .collect();
        targets.sort_by_key(|e| {
            (
                std::cmp::Reverse(self.enemy_threat(e, e.hp)),
                e.hp,
                std::cmp::Reverse(unit_value(e)),
                e.id,
            )
        });
        for e in targets {
            self.door_strike(army, e);
        }
    }

    fn door_strike(&mut self, army: &[&UnitView], e: &UnitView) {
        let committed = self.committed_fire();
        let remaining = e.hp - committed.get(&e.pos).copied().unwrap_or(0);
        if remaining <= 0 {
            return;
        }
        let need = remaining * DOOR_KILL_MARGIN / 100;
        // Shots first: they resolve before any move, and their low rolls weaken the target
        // for the melee forecasts below (`melee_forecast` reads the queued volleys).
        let mut shots: Vec<(i32, &UnitView)> = self
            .free(army)
            .into_iter()
            .filter(|u| {
                u.ranged > 0
                    && u.range > 0
                    && (1..=u.range).contains(&u.pos.distance(e.pos))
                    && self.ranged_unit_clear(u, u.pos, e.pos)
            })
            .filter_map(|u| {
                let f = forecast_attack_mode(&self.attack_view(), u.id, e.pos, AttackMode::Ranged)?;
                (f.expected_damage_to_defender > 0).then_some((f.expected_damage_to_defender, u))
            })
            .collect();
        shots.sort_by_key(|(dmg, u)| (std::cmp::Reverse(*dmg), u.id));
        let mut planned_shots = Vec::new();
        let mut dealt = 0;
        for (dmg, u) in shots {
            if dealt >= need {
                break;
            }
            dealt += dmg;
            planned_shots.push(u);
        }
        let mut chargers: Vec<&UnitView> = Vec::new();
        if dealt < need {
            // Queue the shots so the melee forecasts see them; withdrawn if no kill.
            let mark = self.orders.len();
            for u in &planned_shots {
                self.orders.push(Order::RangedAttack {
                    unit: u.id,
                    target: e.pos,
                });
            }
            let mut melee: Vec<(i32, &UnitView, legatum_core::BattleForecast)> = self
                .free(army)
                .into_iter()
                .filter(|u| {
                    composition::charges(u.kind)
                        && u.kind != UnitKind::Scout
                        && u.hp >= self.decisions.recover_hp
                        && u.pos.distance(e.pos) <= u.max_moves.max(1)
                })
                .filter_map(|u| {
                    let f = self.melee_forecast(u, e.pos)?;
                    let hp_after = u.hp - f.expected_damage_to_attacker;
                    (f.loss_chance <= DOOR_STRIKER_LOSS
                        && hp_after > 0
                        && self.incoming_at(u, e.pos, hp_after, Some(e.id)) < hp_after
                        && self.reaches_this_turn(u, e.pos))
                    .then_some((f.expected_damage_to_defender, u, f))
                })
                .collect();
            self.orders.truncate(mark);
            melee.sort_by_key(|(dmg, u, _)| (std::cmp::Reverse(*dmg), u.id));
            for (dmg, u, _) in melee {
                if dealt >= need {
                    break;
                }
                dealt += dmg;
                chargers.push(u);
            }
        }
        if dealt < need {
            return;
        }
        // The prize is the unit and the threat it carries; the price is the HP we lose.
        let their_cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.view().faction_of(e.owner), e.kind)
            .cost;
        let price: i32 = chargers
            .iter()
            .filter_map(|u| {
                let f = self.melee_forecast(u, e.pos)?;
                let cost = self
                    .view()
                    .rule_experiments
                    .unit_stats_for(self.faction, u.kind)
                    .cost;
                Some(f.expected_damage_to_attacker * cost / u.hp.max(1))
            })
            .sum();
        if their_cost * self.decisions.door_defense < price * 100 {
            return;
        }
        for u in planned_shots {
            self.mil_dest.insert(u.pos);
            self.unit_order(
                u.id,
                Order::RangedAttack {
                    unit: u.id,
                    target: e.pos,
                },
            );
        }
        for u in chargers {
            self.unit_order(
                u.id,
                Order::MoveUnit {
                    unit: u.id,
                    to: e.pos,
                },
            );
        }
    }

    /// An empty threatened city that still has HP shields a garrison: the nearest free unit
    /// that can get in this turn goes in (a shooter while the city is healthy).
    fn door_reinforce(&mut self, army: &[&UnitView], fronts: &[DoorFront]) {
        for f in fronts.iter().filter(|f| f.gap() > 0) {
            let Some(c) = self.ctx.own_cities.iter().copied().find(|c| c.id == f.city) else {
                continue;
            };
            let held =
                self.view().units.iter().any(|u| {
                    u.military && u.pos == c.pos && self.view().same_team(u.owner, c.owner)
                });
            if held || c.hp <= 0 || self.mil_dest.contains(&c.pos) {
                continue;
            }
            let shooter_first = c.hp * 2 > data::CITY_HP;
            // Only a unit that gets in this turn, and never another city's garrison. Measured
            // on the war suite: walking units in over two or three turns, or pulling garrisons
            // from cities with no front, took Japan T142 from 84% to 67%, because the walkers
            // were picked off and the emptied cities were the next targets.
            let unit = self
                .free(army)
                .into_iter()
                .filter(|u| {
                    composition::is_combat(u.kind)
                        && !self.ctx.own_cities.iter().any(|o| o.pos == u.pos)
                        && self.reaches_this_turn(u, c.pos)
                })
                .min_by_key(|u| ((u.ranged > 0) != shooter_first, u.pos.distance(c.pos), u.id));
            if let Some(u) = unit {
                self.go_to(u, c.pos);
            }
        }
    }

    /// A weakened enemy city beside ours whose visible garrison our nearby force outweighs.
    fn door_retake(&mut self, army: &[&UnitView]) {
        let target = self
            .ctx
            .enemy_cities
            .iter()
            .filter(|city| {
                city.visible
                    && city.hp * 2 <= city.max_hp
                    && self.attack_view().at_war(self.view().me, city.owner)
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(city.pos) <= 6)
            })
            .filter(|city| {
                let held: i64 = self
                    .ctx
                    .enemy_units
                    .iter()
                    .filter(|e| e.military && e.pos.distance(city.pos) <= 1)
                    .map(|e| i64::from(unit_value(e)))
                    .sum::<i64>()
                    + i64::from(city_value(city));
                let ours: i64 = self
                    .free(army)
                    .into_iter()
                    .filter(|u| {
                        composition::is_combat(u.kind)
                            && u.hp >= 50
                            && strike_turns(u, city.pos) <= 1
                    })
                    .map(|u| i64::from(unit_value(u)))
                    .sum();
                ours > held
                    && army.iter().any(|u| {
                        composition::charges(u.kind)
                            && u.kind != UnitKind::Scout
                            && u.hp >= 50
                            && u.pos.distance(city.pos) <= 2 * u.max_moves.max(1)
                    })
            })
            .min_by_key(|city| (city.hp, city.pos))
            .map(|city| city.pos);
        if let Some(pos) = target {
            self.assault_city_at(army, pos);
        }
    }
}

#[cfg(test)]
mod siege_deadline_tests {
    use super::*;
    use crate::Knobs;
    use legatum_core::{AiDifficulty, GameState, PlayerId, player_view};

    #[test]
    fn door_credits_a_fast_defender_and_rejects_a_unit_due_on_the_fall_turn() {
        let state: GameState = serde_json::from_reader(flate2::read::GzDecoder::new(
            &include_bytes!("../tests/fixtures/gergovia-t30.json.gz")[..],
        ))
        .unwrap();
        let mut view = player_view(&state, PlayerId(5));
        let city = view
            .cities
            .iter_mut()
            .find(|c| c.name == "Gergovia")
            .unwrap();
        let detail = city.detail.as_mut().unwrap();
        detail.producing = None;
        let front = DoorFront {
            city: city.id,
            pos: city.pos,
            threat: 1000,
            defense: 0,
            arrival: 1,
        };
        let mut probe = Planner::new(
            &view,
            Knobs::for_faction(view.faction_of(view.me)),
            view.faction_of(view.me),
            7,
        );
        probe.difficulty = AiDifficulty::Hard;
        probe.decisions.siege_survival_clock = 1;
        probe.decisions.production_threat_agreement = 1;
        probe.decisions.walled_defense_percent = 100;
        let fall = probe
            .defense_risk(
                view.cities.iter().find(|c| c.id == front.city).unwrap(),
                None,
            )
            .turns_to_fall
            .unwrap();
        assert!(fall > 1 && fall <= 4);
        drop(probe);
        let detail = view
            .cities
            .iter_mut()
            .find(|c| c.id == front.city)
            .unwrap()
            .detail
            .as_mut()
            .unwrap();
        detail.options.retain(|o| {
            matches!(
                o.item,
                ProductionItem::Unit {
                    unit: UnitKind::Soldier | UnitKind::Ranger
                }
            )
        });
        for o in &mut detail.options {
            o.turns = if o.item
                == (ProductionItem::Unit {
                    unit: UnitKind::Soldier,
                }) {
                1
            } else {
                fall
            };
        }
        for gene in [0, 1] {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(view.faction_of(view.me)),
                view.faction_of(view.me),
                7,
            );
            p.difficulty = AiDifficulty::Hard;
            p.decisions.siege_survival_clock = gene;
            p.decisions.production_threat_agreement = 1;
            p.decisions.walled_defense_percent = 100;
            p.decisions.keep_walls = 1;
            p.door_production(std::slice::from_ref(&front));
            let started: Vec<_> = p
                .orders
                .iter()
                .filter_map(|o| match o {
                    Order::SetProduction { city, item } if *city == front.city => Some(*item),
                    _ => None,
                })
                .collect();
            if gene == 1 {
                assert_eq!(
                    started,
                    vec![ProductionItem::Unit {
                        unit: UnitKind::Soldier
                    }]
                );
            } else {
                assert_eq!(started.len(), 1);
            }
        }
    }
}

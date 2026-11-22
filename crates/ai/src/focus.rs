//! Gene `focus_fire`: battle tactics that concentrate on kills (AI war plan step 2, "battle
//! search": focus fire and kill the exposed shooter).
//!
//! The rules reward concentration (`game-rules.md` §4): a defender fights back once per turn and
//! splits that counterattack among every melee attacker that hits it in the same stage, and
//! damage a unit survives heals while a dead unit acts no more. Two units that each wound a
//! different enemy pay two full counterattacks and kill nothing; the same two on one enemy pay
//! one, split, and often kill it. The ordinary passes choose each unit's attack alone, so their
//! shots and charges spread.
//!
//! This pass runs before them and allocates jointly. For every visible enemy field unit, the
//! free shooters that can hit it now and the healthy chargers that reach it this turn are
//! ordered (shots first, since they resolve before any move and weaken the target for the
//! charges), and each prefix of that list is priced as one plan:
//!
//! - **Kill chance:** the exact distribution of the summed damage over every attacker's
//!   80-120% roll (integer counts, so it is order-independent and deterministic), against the
//!   target's HP after shots already ordered this turn.
//! - **Prize:** the kill chance times the target's Production cost plus the threat it carries
//!   (`enemy_threat`: its best expected hit on us next turn, or a city it could take, in HP,
//!   priced at our army's mean cost), so the exposed shooter or would-be captor goes first; and,
//!   without the kill, the damage share at half value (it heals).
//! - **Price:** each charger's forecast damage (the counterattack split among the plan's
//!   chargers, `queued_split_forecast`) and death chance at its cost, plus, after a kill, the
//!   extra fire it stands in on the target's tile; and each attacker's value as a lone hit (what
//!   the ordinary passes would get from it anyway), so a plan pays only when killing is worth
//!   more than spreading.
//!
//! The best plan over all targets is ordered, its units leave the pool, and the search repeats
//! until no plan has a positive value. Units it doesn't use go on to the ordinary passes.
//! City tiles are left to the city-assault coordinator (a city with HP shields its garrison).
//! Everything reads only this turn's view.
//!
//! Gene `gather_hold`, the last military pass: stragglers wait for the army. A unit whose move
//! this turn would end where the visible enemy (its units' charges and shots, and walled
//! cities' strikes) can hit it next turn, and where its expected share of that fire (split
//! among the friendly units ending beside it) is at least `gather_hold` percent of its HP, stops
//! at the last tile of its route where the share is below that, or stays. Attacks, retreats,
//! coordinated assault groups and moves into or beside our own cities (defence) are left
//! alone, and nothing is held that is no worse off moving than staying.

use crate::composition;
use crate::scripted::Planner;
use legatum_core::path::find_path;
use legatum_core::rules::movement::can_step;
use legatum_core::view::UnitView;
use legatum_core::{AttackMode, BattleForecast, Hex, Order, combat, forecast_attack_mode};
use std::collections::BTreeMap;

/// Most attackers priced in one plan (the distribution has 41^k outcomes; six fit in a u64).
const MAX_ATTACKERS: usize = 6;
const ROLLS: u64 = (combat::ROLL_MAX - combat::ROLL_MIN + 1) as u64;

/// One attacker in a plan: its order and the damage it deals at each roll.
struct Attack<'u> {
    unit: &'u UnitView,
    /// Damage at each roll, ROLL_MIN..=ROLL_MAX.
    rolls: Vec<i32>,
    expected: i32,
}

struct Plan<'u> {
    value: i64,
    target: Hex,
    attackers: Vec<(&'u UnitView, bool)>,
}

/// Chance, out of 10,000, that the summed damage of `attacks` reaches `hp`.
pub(crate) fn kill_chance(hp: i32, attacks: &[Vec<i32>]) -> i64 {
    let hp = hp.max(1) as usize;
    // counts[d]: roll combinations dealing exactly d damage (capped at hp).
    let mut counts = vec![0u64; hp + 1];
    counts[0] = 1;
    for rolls in attacks {
        let mut next = vec![0u64; hp + 1];
        for (d, &c) in counts.iter().enumerate() {
            if c == 0 {
                continue;
            }
            for &dmg in rolls {
                next[(d + dmg.max(0) as usize).min(hp)] += c;
            }
        }
        counts = next;
    }
    let total = ROLLS.pow(attacks.len() as u32);
    (u128::from(counts[hp]) * 10_000 / u128::from(total)) as i64
}

/// Chance, out of 10,000, that one shot with forecast `f` deals at least `hp`.
pub(crate) fn shot_kill_chance(hp: i32, f: &BattleForecast) -> i64 {
    kill_chance(hp, &[rolls(f)])
}

fn rolls(f: &BattleForecast) -> Vec<i32> {
    (combat::ROLL_MIN..=combat::ROLL_MAX)
        .map(|roll| combat::damage(f.attacker_strength.total, f.defender_strength.total, roll))
        .collect()
}

impl Planner<'_> {
    /// Gene `focus_fire` (percent weight on a kill's prize; 0: off). See the module doc.
    pub(crate) fn focus_fire(&mut self, army: &[&UnitView]) {
        if self.decisions.focus_fire <= 0 || !self.hard_defense() {
            return;
        }
        let costs: Vec<i32> = army
            .iter()
            .map(|u| {
                self.view()
                    .rule_experiments
                    .unit_stats_for(self.faction, u.kind)
                    .cost
            })
            .collect();
        let ref_cost = (costs.iter().sum::<i32>() / (costs.len().max(1) as i32)).max(1);
        // One plan per round; at most one round per enemy in reach.
        for _ in 0..self.ctx.enemy_units.len() {
            let Some(plan) = self.best_focus_plan(army, ref_cost) else {
                break;
            };
            for (u, shot) in plan.attackers {
                if shot {
                    self.mil_dest.insert(u.pos);
                    self.unit_order(
                        u.id,
                        Order::RangedAttack {
                            unit: u.id,
                            target: plan.target,
                        },
                    );
                } else {
                    self.unit_order(
                        u.id,
                        Order::MoveUnit {
                            unit: u.id,
                            to: plan.target,
                        },
                    );
                }
            }
        }
    }

    fn best_focus_plan<'u>(&mut self, army: &[&'u UnitView], ref_cost: i32) -> Option<Plan<'u>> {
        let free: Vec<&'u UnitView> = self
            .free(army)
            .into_iter()
            .filter(|u| u.kind != legatum_core::UnitKind::Scout && u.hp > 0)
            .collect();
        if free.is_empty() {
            return None;
        }
        let committed = self.committed_fire();
        let targets: Vec<&UnitView> = self
            .ctx
            .enemy_units
            .iter()
            .copied()
            .filter(|e| {
                e.military
                    && self.view().tile(e.pos).is_some_and(|t| t.visible)
                    && !self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos)
                    && !self.view().cities.iter().any(|c| c.pos == e.pos)
            })
            .collect();
        let mut best: Option<Plan<'u>> = None;
        for e in targets {
            let remaining = e.hp - committed.get(&e.pos).copied().unwrap_or(0);
            if remaining <= 0 {
                continue;
            }
            let Some(plan) = self.focus_plan_on(&free, e, remaining, ref_cost) else {
                continue;
            };
            if best.as_ref().is_none_or(|b| {
                plan.value > b.value || (plan.value == b.value && plan.target < b.target)
            }) {
                best = Some(plan);
            }
        }
        best.filter(|p| p.value > 0)
    }

    /// The best prefix of the attackers on `e`, by plan value (module doc).
    fn focus_plan_on<'u>(
        &mut self,
        free: &[&'u UnitView],
        e: &UnitView,
        remaining: i32,
        ref_cost: i32,
    ) -> Option<Plan<'u>> {
        let view = self.attack_view().into_owned();
        let mut shots: Vec<Attack<'u>> = free
            .iter()
            .copied()
            .filter(|u| {
                u.ranged > 0
                    && u.range > 0
                    && (1..=u.range).contains(&u.pos.distance(e.pos))
                    && self.ranged_unit_clear(u, u.pos, e.pos)
            })
            .filter_map(|u| {
                let f = forecast_attack_mode(&view, u.id, e.pos, AttackMode::Ranged)?;
                (f.expected_damage_to_defender > 0).then(|| Attack {
                    unit: u,
                    rolls: rolls(&f),
                    expected: f.expected_damage_to_defender,
                })
            })
            .collect();
        shots.sort_by_key(|a| (std::cmp::Reverse(a.expected), a.unit.id));
        shots.truncate(MAX_ATTACKERS);
        // Chargers: healthy units that reach the target this turn, not a city's garrison.
        let chargers: Vec<&'u UnitView> = free
            .iter()
            .copied()
            .filter(|u| {
                composition::charges(u.kind)
                    && !composition::shooter_only(u.kind, u.ranged)
                    && u.hp >= self.decisions.recover_hp
                    && u.pos.distance(e.pos) <= u.max_moves.max(1)
                    && !self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                    && !shots.iter().any(|s| s.unit.id == u.id)
                    && self.reaches_this_turn(u, e.pos)
            })
            .collect();
        if shots.is_empty() && chargers.is_empty() {
            return None;
        }
        let their_cost = self
            .view()
            .rule_experiments
            .unit_stats_for(self.view().faction_of(e.owner), e.kind)
            .cost;
        let threat = i64::from(self.enemy_threat(e, remaining));
        let kill_value = (i64::from(their_cost) + threat * i64::from(ref_cost) / 100)
            * i64::from(self.decisions.focus_fire)
            / 100;
        let lone = |expected: i32| {
            i64::from(expected.min(remaining)) * i64::from(their_cost) / i64::from(remaining) / 2
        };
        let mut best: Option<Plan<'u>> = None;
        let mut consider = |value: i64, attackers: Vec<(&'u UnitView, bool)>| {
            if best.as_ref().is_none_or(|b| value > b.value) {
                best = Some(Plan {
                    value,
                    target: e.pos,
                    attackers,
                });
            }
        };
        // Shot prefixes alone.
        let mut alone = 0i64;
        let mut dealt = 0;
        for k in 1..=shots.len() {
            alone += lone(shots[k - 1].expected);
            dealt += shots[k - 1].expected;
            let dist: Vec<Vec<i32>> = shots[..k].iter().map(|a| a.rolls.clone()).collect();
            let p = kill_chance(remaining, &dist);
            let partial = lone(dealt);
            let prize = (p * kill_value + (10_000 - p) * partial) / 10_000;
            consider(
                prize - alone,
                shots[..k].iter().map(|a| (a.unit, true)).collect(),
            );
        }
        if chargers.is_empty() {
            return best;
        }
        // Every shot prefix (none included) followed by charger prefixes. The shots are queued so
        // the charges' forecasts see the volley (`melee_forecast`), then withdrawn.
        for k in 0..=shots.len() {
            let mark = self.orders.len();
            for a in &shots[..k] {
                self.orders.push(Order::RangedAttack {
                    unit: a.unit.id,
                    target: e.pos,
                });
            }
            let mut melee: Vec<(&'u UnitView, BattleForecast)> = chargers
                .iter()
                .copied()
                .filter_map(|u| Some((u, self.melee_forecast(u, e.pos)?)))
                .filter(|(u, f)| {
                    self.attack_city_damage(u, f) + f.expected_damage_to_attacker < u.hp
                })
                .collect();
            melee.sort_by_key(|(u, f)| (std::cmp::Reverse(f.expected_damage_to_defender), u.id));
            melee.truncate(MAX_ATTACKERS - k);
            let shot_alone: i64 = shots[..k].iter().map(|a| lone(a.expected)).sum();
            let shot_dealt: i32 = shots[..k].iter().map(|a| a.expected).sum();
            for m in 1..=melee.len() {
                // Queue the earlier chargers so this prefix's forecasts split the counterattack.
                let charge_mark = self.orders.len();
                for (u, _) in &melee[..m] {
                    self.orders.push(Order::MoveUnit {
                        unit: u.id,
                        to: e.pos,
                    });
                }
                let mut dist: Vec<Vec<i32>> = shots[..k].iter().map(|a| a.rolls.clone()).collect();
                let mut price = 0i64;
                let mut alone = shot_alone;
                let mut dealt = shot_dealt;
                let mut forecasts = Vec::new();
                for (u, _) in &melee[..m] {
                    let Some(f) = self.melee_forecast(u, e.pos) else {
                        continue;
                    };
                    forecasts.push((*u, f));
                }
                for (_, f) in &forecasts {
                    dist.push(rolls(f));
                    dealt += f.expected_damage_to_defender;
                    alone += lone(f.expected_damage_to_defender);
                }
                let p = kill_chance(remaining, &dist);
                for (u, f) in &forecasts {
                    let cost = i64::from(
                        self.view()
                            .rule_experiments
                            .unit_stats_for(self.faction, u.kind)
                            .cost,
                    );
                    let hp = i64::from(u.hp.max(1));
                    let taken =
                        i64::from(f.expected_damage_to_attacker + self.attack_city_damage(u, f));
                    price += taken.min(hp) * cost / hp + i64::from(f.loss_chance) * cost / 200;
                    // After a kill it stands on the target's tile instead of its own.
                    let hp_after = (u.hp - f.expected_damage_to_attacker).max(1);
                    let there = self.incoming_at(u, e.pos, hp_after, Some(e.id));
                    let here = self.incoming_at(u, u.pos, hp_after, Some(e.id));
                    price +=
                        p * i64::from((there - here).max(0)) * cost / i64::from(hp_after) / 10_000;
                }
                self.orders.truncate(charge_mark);
                if forecasts.len() < m {
                    continue;
                }
                let partial = lone(dealt);
                let prize = (p * kill_value + (10_000 - p) * partial) / 10_000;
                let attackers = shots[..k]
                    .iter()
                    .map(|a| (a.unit, true))
                    .chain(melee[..m].iter().map(|(u, _)| (*u, false)))
                    .collect();
                consider(prize - price - alone, attackers);
            }
            self.orders.truncate(mark);
        }
        best
    }
}

impl Planner<'_> {
    /// The tiles of `u`'s route toward `to` that it reaches this turn, in order (fogged costs).
    fn route_this_turn(&self, u: &UnitView, to: Hex) -> Vec<Hex> {
        let Some(path) = find_path(self.movement_map(), u.pos, to, |h| {
            let i = self.view().index(h)?;
            self.ctx
                .move_cost(h)
                .filter(|_| h == to || !self.ctx.hostile_at[i])
        }) else {
            return vec![];
        };
        let mut moves = u.max_moves;
        let mut reached = vec![];
        for h in path {
            let Some(cost) = self.ctx.move_cost(h) else {
                break;
            };
            if !can_step(moves, u.max_moves, cost) {
                break;
            }
            moves = (moves - cost).max(0);
            reached.push(h);
        }
        reached
    }

    /// Gene `gather_hold` (module doc): hold stragglers short of the enemy's reach.
    pub(crate) fn hold_stragglers(&mut self, army: &[&UnitView]) {
        if self.decisions.gather_hold <= 0 || !self.hard_defense() || !self.at_war() {
            return;
        }
        let mine: BTreeMap<legatum_core::UnitId, &UnitView> = army
            .iter()
            .filter(|u| u.kind != legatum_core::UnitKind::Scout)
            .map(|u| (u.id, *u))
            .collect();
        // Where each of our units ends this turn: its move's last reachable tile, or where it is.
        let mut ends: BTreeMap<legatum_core::UnitId, Hex> =
            mine.values().map(|u| (u.id, u.pos)).collect();
        let mut moves: Vec<(usize, legatum_core::UnitId, Hex, Vec<Hex>)> = vec![];
        for (i, o) in self.orders.iter().enumerate() {
            let (unit, to) = match o {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => (*unit, *to),
                _ => continue,
            };
            let Some(u) = mine.get(&unit) else {
                continue;
            };
            let route = self.route_this_turn(u, to);
            if let Some(&end) = route.last() {
                ends.insert(unit, end);
            }
            moves.push((i, unit, to, route));
        }
        let assault: std::collections::BTreeSet<_> = self
            .assault_groups
            .iter()
            .flatten()
            .copied()
            .chain(self.assault_attacks.iter().copied())
            .chain(self.retreats.iter().copied())
            .chain(self.contest_units.iter().copied())
            .collect();
        let own_city_near = |h: Hex| self.ctx.own_cities.iter().any(|c| c.pos.distance(h) <= 1);
        let hostile = |h: Hex| self.view().index(h).is_some_and(|i| self.ctx.hostile_at[i]);
        let mut holds: Vec<(usize, legatum_core::UnitId, Option<Hex>)> = vec![];
        for (i, unit, to, route) in moves {
            let u = mine[&unit];
            let Some(&end) = route.last() else {
                continue;
            };
            if assault.contains(&unit) || hostile(to) || hostile(end) || own_city_near(to) {
                continue;
            }
            let hp = (u.hp - self.city_damage_now(u)).max(1);
            // Expected fire on a tile, split among our units ending beside it.
            let share = |h: Hex, ends: &BTreeMap<legatum_core::UnitId, Hex>| {
                let beside = ends
                    .iter()
                    .filter(|(id, e)| **id != unit && e.distance(h) <= 1)
                    .count() as i32;
                self.incoming_at(u, h, hp, None) / (1 + beside)
            };
            let threshold = hp * self.decisions.gather_hold / 100;
            let there = share(end, &ends);
            if there < threshold || there <= share(u.pos, &ends) {
                continue;
            }
            // The last tile short of the threshold along the route, or stay.
            let stop = route[..route.len() - 1]
                .iter()
                .rev()
                .copied()
                .find(|h| share(*h, &ends) < threshold && self.free_tile_for(u, *h));
            ends.insert(unit, stop.unwrap_or(u.pos));
            holds.push((i, unit, stop));
        }
        for (i, unit, stop) in holds {
            self.orders[i] = match stop {
                Some(to) => {
                    self.mil_dest.insert(to);
                    Order::MoveUnit { unit, to }
                }
                None => Order::SkipUnit { unit },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::kill_chance;

    #[test]
    fn kill_chance_counts_every_roll_combination() {
        let sure = vec![vec![50; 41]];
        assert_eq!(kill_chance(50, &sure), 10_000);
        assert_eq!(kill_chance(51, &sure), 0);
        // Two attacks of 30 each always kill 60 HP; one never does.
        let thirty = vec![30; 41];
        assert_eq!(kill_chance(60, &[thirty.clone(), thirty.clone()]), 10_000);
        assert_eq!(kill_chance(60, &[thirty]), 0);
        // Damage 1..=41 by roll: exactly half the single rolls reach 21 or more.
        let ramp: Vec<i32> = (1..=41).collect();
        assert_eq!(
            kill_chance(21, std::slice::from_ref(&ramp)),
            21 * 10_000 / 41
        );
        // Order of the attacks does not matter.
        let other: Vec<i32> = (10..=50).collect();
        assert_eq!(
            kill_chance(55, &[ramp.clone(), other.clone()]),
            kill_chance(55, &[other, ramp])
        );
    }
}

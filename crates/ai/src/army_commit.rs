//! Army commitment from the decision audit (2026-10-31): in the designer's seat Hard held back
//! units he kept attacking with. Two genes, both from the fogged view only.
//!
//! **`capture_hold`.** Hard pulled the nearest unit back into every city in resistance (or
//! damaged with enemies about) before any attack pass ran, while he kept his three to six units
//! attacking; `hold_captures` was the pass behind about a third of the kills Hard skipped in his
//! seat. With the gene a front city takes a holder first only when holding it now is worth more
//! than what the holder would do instead. Holding is worth the city when the visible attack would
//! take it empty this turn and a holder would stop that (`purchase_defense::city_risk_with`, the
//! forecast door defense and raze use), at a 90% fall chance. The holder's alternative is its
//! best capture this turn: an ungarrisoned enemy city at war it reaches and takes (population
//! times the capture chance). Both sides count cities by population. Front cities that don't
//! need a holder now get one from the units the attack passes leave free (`hold_captures_late`),
//! and the early anti-Mongol garrison pass leaves them alone too.
//!
//! **`strike_kill`.** The opportunity strike (`press_strike`) sent its force to staging tiles or
//! held it there while he killed the wounded or exposed enemy beside it. With the gene a force
//! unit that can kill a visible field unit alone this turn for more than it risks is left to the
//! attack passes (focus fire, counterattack, routine attacks) instead of staging.
//!
//! **`shot_sweep`** (decision audit round 4). A shooter fires only on an explicit order, and the
//! passes that run before the fire passes (the routine garrison, leaving a city's strike range,
//! the wounded's retreat, staging) gave twelve of the shooters he killed with a move instead: the
//! fire passes see only units nothing has ordered yet. The sweep runs after every pass: a shooter
//! ordered to move (not onto an enemy) that can kill a visible enemy field unit from where it
//! stands, given the shots already ordered (kill chance at least 50%, the target's cost times
//! the kill chance above its own cost times the chance the visible enemy kills it if it stays),
//! fires instead, unless it is the holder that keeps an own city from falling this turn.
use crate::purchase_defense::{city_attacks, city_risk_with};
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};

/// Chance, in percent, that a front city the visible attack is forecast to take empty is lost.
const FALL_PERCENT: i32 = 90;

impl Planner<'_> {
    /// Whether `holder` should hold front city `city` before the attack passes run.
    pub(crate) fn capture_hold_now(&self, city: &CityView, holder: &UnitView) -> bool {
        let v = self.view();
        // The city as the enemy would find it with no unit of ours inside.
        let mut empty = v.clone();
        empty
            .units
            .retain(|u| !(u.pos == city.pos && u.military && v.team_of(u.owner) == v.team));
        let attacks = city_attacks(&empty, city, &|u, h| self.arrives(u, h));
        if !city_risk_with(&empty, city, None, &attacks).falls {
            return false;
        }
        // A holder that would fall with the city saves nothing.
        if city_risk_with(&empty, city, Some(holder.clone()), &attacks).falls {
            return false;
        }
        let tech_loss = if self.faction == legatum_core::Faction::Athens
            && city.greece_captive_at_risk
            && self.decisions.greece_captive_defend > 0
        {
            let era = v
                .players
                .iter()
                .filter(|p| p.alive && p.team != v.team)
                .map(|p| p.era)
                .min()
                .unwrap_or(v.research.era);
            legatum_core::TechId::ALL
                .iter()
                .copied()
                .filter(|t| legatum_core::eras::era_of_tech(*t) == era)
                .map(|t| self.tech_value(t))
                .max()
                .unwrap_or(0)
                * self.decisions.greece_captive_defend
                / 100
        } else {
            0
        };
        self.capture_now_value(holder) <= city.population.max(1) * FALL_PERCENT + tech_loss
    }

    /// The holder's best capture this turn: an ungarrisoned enemy city at war it reaches, worth
    /// its population times the capture chance (percent).
    pub(crate) fn capture_now_value(&self, u: &UnitView) -> i32 {
        if !crate::composition::charges(u.kind) {
            return 0;
        }
        let v = self.view();
        self.ctx
            .enemy_cities
            .iter()
            .filter(|c| c.visible && v.at_war(v.me, c.owner))
            .filter(|c| {
                !self
                    .ctx
                    .enemy_units
                    .iter()
                    .any(|e| e.military && e.pos == c.pos)
            })
            .filter(|c| self.reaches_this_turn(u, c.pos))
            .map(|c| {
                let chance = if c.hp <= 0 {
                    100
                } else {
                    self.melee_forecast(u, c.pos)
                        .map_or(0, |f| f.win_chance.min(100) as i32)
                };
                (c.population.max(1) * 100 + self.greece_captive_tech_value(c)) * chance / 100
            })
            .max()
            .unwrap_or(0)
    }
}

impl Planner<'_> {
    /// Gene `strike_kill`: whether `u` can kill a visible enemy field unit at war this turn by
    /// itself, for more than it risks: a shot in range and sight, or a charge it reaches, with a
    /// forecast kill chance of at least 50% and the target's cost times the kill chance above
    /// its own cost times its loss chance. City tiles are the assault coordinator's.
    pub(crate) fn kill_beside(&self, u: &UnitView) -> bool {
        let v = self.view();
        let own_cost = i64::from(
            self.view()
                .rule_experiments
                .unit_stats_for(self.faction, u.kind)
                .cost,
        );
        let shooter = crate::composition::shooter_only(u.kind, u.ranged);
        let charger = crate::composition::charges(u.kind) && !shooter;
        if !shooter && !charger {
            return false;
        }
        let view = self.attack_view();
        self.ctx
            .enemy_units
            .iter()
            .filter(|e| e.military && v.at_war(v.me, e.owner))
            .filter(|e| !self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos))
            .filter(|e| u.pos.distance(e.pos) <= u.max_moves.max(u.range).max(1))
            .any(|e| {
                let forecast = if shooter {
                    if u.range <= 0
                        || u.pos.distance(e.pos) > u.range
                        || !self.ranged_unit_clear(u, u.pos, e.pos)
                    {
                        return false;
                    }
                    legatum_core::forecast_attack_mode(
                        &view,
                        u.id,
                        e.pos,
                        legatum_core::AttackMode::Ranged,
                    )
                } else {
                    if !self.reaches_this_turn(u, e.pos) {
                        return false;
                    }
                    self.melee_forecast(u, e.pos)
                };
                let Some(f) = forecast else {
                    return false;
                };
                let their_cost = i64::from(
                    v.rule_experiments
                        .unit_stats_for(v.faction_of(e.owner), e.kind)
                        .cost,
                );
                f.win_chance >= 50
                    && their_cost * i64::from(f.win_chance) > own_cost * i64::from(f.loss_chance)
            })
    }
}

impl Planner<'_> {
    /// Gene `shot_sweep` (see the module doc): turn a shooter's move into a lethal shot from
    /// where it stands when that is worth more than the move.
    pub(crate) fn shot_sweep(&mut self) {
        use legatum_core::{AttackMode, Order, forecast_attack_mode};
        if self.decisions.shot_sweep == 0 || !self.hard_defense() {
            return;
        }
        let mut committed = self.committed_fire();
        for i in 0..self.orders.len() {
            let (unit, to) = match self.orders[i] {
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => (unit, to),
                _ => continue,
            };
            let Some(u) = self.ctx.own_units.iter().copied().find(|u| u.id == unit) else {
                continue;
            };
            if !u.military
                || !crate::composition::shooter_only(u.kind, u.ranged)
                || u.range <= 0
                || self.ctx.enemy_units.iter().any(|e| e.pos == to)
                || self.ctx.enemy_cities.iter().any(|c| c.pos == to)
            {
                continue;
            }
            let v = self.view();
            let own_cost = i64::from(
                self.view()
                    .rule_experiments
                    .unit_stats_for(self.faction, u.kind)
                    .cost,
            );
            let view = self.attack_view();
            let mut best: Option<(i64, legatum_core::UnitId, legatum_core::Hex, i32)> = None;
            for e in self.ctx.enemy_units.iter().filter(|e| e.military) {
                let remaining = e.hp - committed.get(&e.pos).copied().unwrap_or(0);
                if remaining <= 0
                    || u.pos.distance(e.pos) > u.range
                    || self.ctx.enemy_cities.iter().any(|c| c.pos == e.pos)
                    || !self.ranged_unit_clear(u, u.pos, e.pos)
                {
                    continue;
                }
                let Some(f) = forecast_attack_mode(&view, u.id, e.pos, AttackMode::Ranged) else {
                    continue;
                };
                if f.garrison.as_ref().is_some_and(|g| g.shielded) {
                    continue;
                }
                let kill = crate::focus::shot_kill_chance(remaining, &f);
                if kill < 5_000 {
                    continue;
                }
                // If it stays, the rest of the visible enemy (the target gone) may finish it.
                let incoming = self.incoming_at(u, u.pos, u.hp, Some(e.id));
                let loss = if incoming >= u.hp {
                    10_000
                } else {
                    10_000 * i64::from(incoming) / i64::from(u.hp.max(1)) / 2
                };
                let their_cost = i64::from(
                    v.rule_experiments
                        .unit_stats_for(v.faction_of(e.owner), e.kind)
                        .cost,
                );
                let value = their_cost * kill - own_cost * loss;
                if value > 0 && best.is_none_or(|(b, ..)| value > b) {
                    best = Some((value, e.id, e.pos, f.expected_damage_to_defender));
                }
            }
            let Some((_, _, target, damage)) = best else {
                continue;
            };
            // The holder that keeps an own city from falling this turn keeps its move.
            if let Some(city) = self.ctx.own_cities.iter().copied().find(|c| c.pos == to) {
                let attacks = city_attacks(v, city, &|a, h| self.arrives(a, h));
                if city_risk_with(v, city, None, &attacks).falls
                    && !city_risk_with(v, city, Some(u.clone()), &attacks).falls
                {
                    continue;
                }
            }
            *committed.entry(target).or_default() += damage;
            self.orders[i] = Order::RangedAttack { unit, target };
        }
    }
}

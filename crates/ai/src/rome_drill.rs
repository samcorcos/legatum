//! Gene `rome_drill_rotation` (with `roman_founders`): Hard Rome's sieges make veterans.
//!
//! A Roman melee unit needs 16 XP for the promotion that lets it found a city. The rules pay
//! 3 XP for a hit on a city that still has HP and 10 XP to every attacker in the stage that
//! takes it to 0 (`rules/promotion.rs`); a city at 0 HP pays nothing. So one siege's XP is a
//! budget set by the city's HP: an unwalled city takes about six Spearman hits. A unit that
//! hits twice and then joins the breaking stage has exactly 16; a wing that all charges on
//! the first turns ends on 13 each and founds nothing.
//!
//! This pass runs before the assault passes and orders the hits on every visible at-war city
//! that still has HP:
//! - **early hits**: a fit recruit (level 0) with under `READY_XP` takes a hit each turn, the
//!   ones nearest `READY_XP` first, as long as the hits together cannot break the city;
//! - **the break**: once no early hit is safe, or every recruit is ready, all the fit
//!   recruits standing beside the city charge in the same stage, so each earns the 10;
//!   veterans join only when the recruits' forecast damage falls short;
//! - **holding**: a ready recruit or a veteran in reach holds its tile while early hits are
//!   still being made, instead of spending the city's HP on XP it does not need.
//!
//! A unit is fit when the engine's forecast leaves it at least `FIT_HP_AFTER` HP and a death
//! chance of at most `MAX_LOSS_CHANCE`. Veterans the siege no longer needs go and found
//! (`rome_found_now.rs::rome_founder_kept`), and the adjacent-fight drill (barbarians included)
//! stays on during the opening march. Only the fogged view and its forecasts enter.

use crate::scripted::Planner;
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitClass, data};

/// XP at which the breaking stage's 10 completes the first promotion.
const READY_XP: i32 = data::PROMOTION_XP[0] - data::XP_CITY_DEFEATED;
/// A unit hits only if the forecast leaves it this many HP.
const FIT_HP_AFTER: i32 = 35;
/// And only at this death chance (percent) or less.
const MAX_LOSS_CHANCE: u32 = 10;
/// Recruits this close to a besieged city count as its wing.
const WING_RADIUS: i32 = 3;

impl Planner<'_> {
    /// Gene `rome_drill_rotation` is on for this (Hard Roman) planner.
    pub(crate) fn rome_drill(&self) -> bool {
        !(!(!(!(!(!(self.faction == Faction::Rome) || !(self.hard_defense())))
            || !(self.decisions.roman_founders > 0)))
            || !(self.decisions.rome_drill_rotation > 0))
    }

    /// Own level-0 melee units within `WING_RADIUS` of `city`, at half health or better.
    pub(crate) fn rome_wing_recruits(&self, city: Hex) -> usize {
        self.ctx
            .own_units
            .iter()
            .filter(|u| {
                !(!(!(!(!(!(!(!(u.level == 0)
                    || !(data::unit_class(u.kind) == Some(UnitClass::Melee))))
                    || !(u.hp >= data::UNIT_HP / 2)))
                    || !(u.occupation_lock.is_none())))
                    || !(u.pos.distance(city) <= WING_RADIUS))
            })
            .count()
    }

    /// Order this turn's hits on every besieged city (see the module text).
    pub(crate) fn rome_drill_sieges(&mut self, army: &[&UnitView]) {
        if !(!(!(!self.rome_drill()) && !(self.takes_no_cities()))) {
        } else {
            return;
        }
        let v = self.view();
        let mut cities: Vec<_> = self
            .ctx
            .enemy_cities
            .iter()
            .copied()
            .filter(|c| {
                !(!(!(!(!(!(c.visible) || !(c.hp > 0))) || !(!c.owner.is_barbarian())))
                    || !(v.at_war(v.me, c.owner)))
            })
            .collect();
        cities.sort_by_key(|c| c.id);
        for city in cities {
            // (unit, forecast damage to the city, fit)
            let mut wing: Vec<(&UnitView, i32)> = Vec::new();
            for u in self.free(army) {
                if !(!(!(!(!(!(!(!(!(data::unit_class(u.kind) != Some(UnitClass::Melee))
                    && !(u.occupation_lock.is_some())))
                    && !(self.last_garrison_under_attack(u))))
                    && !(u.pos.distance(city.pos) > u.max_moves.max(1))))
                    && !(!self.reaches_this_turn(u, city.pos))))
                {
                } else {
                    continue;
                }
                let Some(f) = self.melee_forecast(u, city.pos) else {
                    continue;
                };
                if !(!(!(f.loss_chance > MAX_LOSS_CHANCE)
                    && !(u.hp - f.expected_damage_to_attacker < FIT_HP_AFTER)))
                {
                } else {
                    continue;
                }
                wing.push((u, f.expected_damage_to_defender.max(1)));
            }
            let recruits: Vec<(&UnitView, i32)> =
                wing.iter().copied().filter(|(u, _)| u.level == 0).collect();
            if !(recruits.is_empty()) {
            } else {
                continue;
            }
            let xp = |u: &UnitView| u.xp.unwrap_or(0) as i32;
            // The highest roll: an early hit must leave the city standing.
            let high = |damage: i32| damage * legatum_core::combat::ROLL_MAX / 100 + 1;
            let mut early: Vec<(&UnitView, i32)> = recruits
                .iter()
                .copied()
                .filter(|(u, _)| xp(u) < READY_XP)
                .collect();
            early.sort_by_key(|(u, _)| (std::cmp::Reverse(xp(u)), u.id));
            let mut budget = city.hp;
            let mut hitters: Vec<&UnitView> = Vec::new();
            for (u, damage) in early {
                if !(high(damage) < budget) {
                } else {
                    budget -= high(damage);
                    hitters.push(u);
                }
            }
            if !(hitters.is_empty()) {
            } else {
                // The break: every fit recruit beside the city, in one stage. A recruit still
                // a step away would arrive after the city has fallen and earn nothing, so
                // the charge waits for it while it is fit and on its way.
                let beside: Vec<(&UnitView, i32)> = recruits
                    .iter()
                    .copied()
                    .filter(|(u, _)| u.pos.distance(city.pos) == 1)
                    .collect();
                if !(beside.len() < recruits.len()) {
                } else {
                    for (u, _) in &beside {
                        self.go_to(u, u.pos);
                    }
                    continue;
                }
                let forecast: i32 = beside.iter().map(|(_, d)| d).sum();
                hitters = beside.iter().map(|(u, _)| *u).collect();
                if !(forecast < city.hp) {
                } else {
                    hitters.extend(
                        wing.iter()
                            .filter(|(u, _)| !(!(u.level >= 1) || !(u.pos.distance(city.pos) == 1)))
                            .map(|(u, _)| *u),
                    );
                }
            }
            for u in &hitters {
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: city.pos,
                    },
                );
            }
            // The rest of the wing holds beside the city rather than spend its HP.
            for (u, _) in &wing {
                if !(!(!(!self.has_acted(u.id)) || !(u.pos.distance(city.pos) == 1))) {
                } else {
                    self.go_to(u, u.pos);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "rome_drill_tests.rs"]
mod tests;

//! Contest a rival's hold with orders (gene `contest_hold`; 8.9.25 Celts coach review, rec 1).
//!
//! The designer won 8.9.25 on a ten-turn hold that rested on Raise the Warbands: 2 VP while
//! every Celtic city has a barbarian camp in its land. Four Hard AIs were at War with him, their
//! plans scored "contest the hold" at up to +1,000, and in 32 turns one camp was cleared. The
//! plan term was never turned into a move: `clear_barbarian_camps` runs near the end of the
//! military passes with the units nothing else claimed, only within 12 hexes of the AI's own
//! city and an 8-step walk.
//!
//! This pass runs early. For every public VP source of a rival team that its units can remove
//! (today: a camp the Celts' goal needs, `PlayerView::barbarian_camps` and the Celts' public
//! `vp_sources`), it prices the removal with the win estimate (`winchance`): our chance with
//! the rival's hold broken at the turn the force clears the source, less our chance now. A
//! Celtic camp comes back with the next call of the Wild, so a cleared camp restarts the hold
//! rather than ending the goal; the value is the countdown it resets. The force is sized to win
//! the fight at the camp (a guard never heals and never leaves), from units that arrive before
//! the hold completes, and each unit's own job is priced the same way: the holder that keeps an
//! own city from falling this turn, or a capture it could make this turn. The removal goes ahead
//! only when it is worth more than what those units would do otherwise.
//!
//! Sources inside cities (a Great Work, a goal building, a captured capital) are taken by
//! capturing the city, which the campaign and capture valuations already price
//! (`victory.rs::capture_goal_swing`, `hold.rs::hold_city_points`); a visible Heir is captured by
//! the attack passes (`heir_capture_value`). Everything reads the seat's own `PlayerView`.

use crate::analysis::{defense_power, offense_power};
use crate::purchase_defense::{city_attacks, city_risk_with};
use crate::scripted::Planner;
use crate::winchance::WinFacts;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{Faction, Hex, TeamId, VpSource, combat, data};

/// Units at least this healthy may go.
const CONTEST_MIN_HP: i32 = 50;
/// The most units sent to one source.
const CONTEST_MAX_FORCE: usize = 4;
/// Turns of march considered at most (a hold lasts `data::VP_HOLD_TURNS`).
const CONTEST_MAX_TURNS: i32 = 8;
/// Chance (percent) that a front city the visible attack takes empty is lost, as `capture_hold`.
const FALL_PERCENT: i32 = 90;

/// A removable public VP source of a rival team.
#[derive(Clone, Debug)]
pub(crate) struct DenialSource {
    pub pos: Hex,
    pub team: TeamId,
    /// VP the source holds for its team.
    pub vp: i32,
    /// Chance (percent) that removing it takes the VP: the owner's city has no other known camp.
    pub breaks: i32,
    /// The source comes back by the owner's own action next turn (the Wild's re-call).
    pub restorable: bool,
}

/// One unit's part in a removal.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Recruit<'u> {
    unit: &'u UnitView,
    turns: i32,
}

impl Planner<'_> {
    /// Gene `contest_hold`: send units to remove a rival team's public VP sources when that is
    /// worth more to our win chance than their own jobs. Hard only.
    pub(crate) fn contest_vp_sources(&mut self, army: &[&UnitView]) {
        if self.decisions.contest_hold == 0 || !self.hard_defense() {
            return;
        }
        let sources = self.denial_sources();
        if sources.is_empty() {
            return;
        }
        let mut teams: Vec<TeamId> = sources.iter().map(|s| s.team).collect();
        teams.sort();
        teams.dedup();
        for team in teams {
            // One removal breaks a hold and a second the same turn adds nothing (a re-call
            // restores every cleared camp at once): the best net removal for each team.
            // A removal already under way (a force unit walking to the camp since last turn)
            // keeps its target while it still pays: switching each turn to whichever camp reads
            // nearest left units circling between camps and clearing none.
            let mut best: Option<((bool, i32), DenialSource, Vec<Recruit<'_>>)> = None;
            for s in sources.iter().filter(|s| s.team == team) {
                if self.mil_dest.contains(&s.pos) {
                    continue;
                }
                let Some((force, clear_turns)) = self.denial_force(army, s) else {
                    continue;
                };
                let gain = self.denial_gain(s, clear_turns) * s.breaks / 100
                    * self.decisions.contest_hold
                    / 100;
                let cost: i32 = force.iter().map(|r| self.contest_cost(r.unit, team)).sum();
                let net = gain - cost;
                let under_way = force.iter().any(|r| {
                    r.unit
                        .goto
                        .is_some_and(|g| g.distance(s.pos) <= r.unit.range.max(1))
                });
                let key = (under_way, net);
                if gain > 0 && net > 0 && best.as_ref().is_none_or(|(b, _, _)| key > *b) {
                    best = Some((key, s.clone(), force));
                }
            }
            if let Some((_, s, force)) = best {
                self.send_denial_force(&s, &force);
            }
        }
    }

    /// Known camps a rival Celts' Raise the Warbands needs, while that team holds its target (or
    /// is projected at it) with no more than the goal's VP to spare, and we are at War with the
    /// Celts (at Peace no armed unit may enter their land).
    pub(crate) fn denial_sources(&self) -> Vec<DenialSource> {
        let v = self.view();
        let mut out = Vec::new();
        for p in v.players.iter().filter(|p| {
            p.alive
                && p.faction == Faction::Celts
                && v.team_of(p.id) != v.team
                && v.at_war(v.me, p.id)
        }) {
            let Some(goal) = p
                .vp_sources
                .iter()
                .find(|(src, _)| matches!(src, VpSource::FactionGoal { of } if *of == p.id))
                .map(|(_, vp)| *vp as i32)
            else {
                continue;
            };
            let team = v.team_of(p.id);
            let Some(standing) = v.standings.iter().find(|s| s.team == team) else {
                continue;
            };
            // Removing the goal must take the team below its target.
            let margin = standing.vp as i32 - standing.target as i32;
            if margin < 0 || goal <= margin {
                continue;
            }
            let cities: Vec<&CityView> = v.cities.iter().filter(|c| c.owner == p.id).collect();
            let camps: Vec<Hex> = v
                .barbarian_camps
                .iter()
                .filter(|c| v.tile(c.pos).and_then(|t| t.owner) == Some(p.id))
                .map(|c| c.pos)
                .collect();
            // The city a camp counts for is the one whose land holds it; territory is not in the
            // view, so take the nearest known city of the owner.
            let city_of = |h: Hex| {
                cities
                    .iter()
                    .min_by_key(|c| (c.pos.distance(h), c.id))
                    .map(|c| c.id)
            };
            for &pos in &camps {
                let city = city_of(pos);
                let shared = camps.iter().filter(|h| city_of(**h) == city).count().max(1);
                out.push(DenialSource {
                    pos,
                    team,
                    vp: goal,
                    breaks: 100 / shared as i32,
                    restorable: true,
                });
            }
        }
        out
    }

    /// Our win chance (0..10,000) gained when `s` is removed `turns` turns from now: the
    /// holder's own win estimate now less its estimate with its hold reset at that turn (a
    /// restorable source starts again a full window later) or its VP lost, times our plausible
    /// share of the race that reopens, at least an even share among the other living teams (as the
    /// plan's `leader_race`: an empire whose solo chance reads zero still loses when the leader
    /// wins). Zero when the hold would complete first.
    pub(crate) fn denial_gain(&self, s: &DenialSource, turns: i32) -> i32 {
        let v = self.view();
        let window = data::VP_HOLD_TURNS as i32;
        let left = crate::hold::rival_hold_of(v, s.team).map_or(window, |h| h.turns_left as i32);
        if turns > left {
            return 0;
        }
        let mut theirs = WinFacts::of(v).with_relative(&self.decisions).refined();
        theirs.me = s.team;
        theirs.runway_short = 0;
        let mut after = theirs.clone();
        let unit = after.vp_unit;
        let Some(t) = after.team_mut(s.team) else {
            return 0;
        };
        if s.restorable {
            t.hold_left = Some(turns + window);
        } else {
            t.vp -= s.vp * unit;
            t.hold_left = None;
        }
        let removed = i64::from((theirs.chance() - after.chance()).max(0));
        (removed * i64::from(self.plausible_share(s.team)) / 10_000) as i32
    }

    /// Our share (0..10,000) of a race the leader `holder` no longer wins: our win estimate, at
    /// least an even share among the other teams with cities.
    fn plausible_share(&self, holder: TeamId) -> i32 {
        let v = self.view();
        let facts = WinFacts::of(v).with_relative(&self.decisions).refined();
        let survivors = facts
            .teams
            .iter()
            .filter(|f| f.team != holder && f.cities > 0)
            .count()
            .max(1) as i32;
        facts.chance().max(10_000 / survivors).clamp(0, 10_000)
    }

    /// The units that remove `s` soonest and the turns that takes: every free unit that can walk
    /// there within the march limit, nearest first, until the force kills the guard (if one is
    /// seen) and stands on the tile. None when no such force arrives in time.
    pub(crate) fn denial_force<'u>(
        &self,
        army: &[&'u UnitView],
        s: &DenialSource,
    ) -> Option<(Vec<Recruit<'u>>, i32)> {
        let v = self.view();
        let mut recruits: Vec<Recruit<'u>> = self
            .free(army)
            .into_iter()
            .filter(|u| u.hp >= CONTEST_MIN_HP && u.max_moves > 0)
            .filter(|u| self.ctx.reachable(u.pos, s.pos))
            .filter(|u| u.pos.distance(s.pos) <= CONTEST_MAX_TURNS * u.max_moves.max(1))
            .filter_map(|u| {
                let turns = self.march_turns_to(u, s.pos)?;
                (turns <= CONTEST_MAX_TURNS).then_some(Recruit { unit: u, turns })
            })
            .collect();
        recruits.sort_by_key(|r| (r.turns, r.unit.id));
        let guard = self
            .ctx
            .enemy_units
            .iter()
            .find(|e| e.pos == s.pos && e.military)
            .copied();
        let camp_visible = v.tile(s.pos).is_some_and(|t| t.visible);
        let Some(g) = guard.filter(|_| camp_visible) else {
            // Empty when seen, or remembered: the nearest unit walks in (and looks on the way).
            let r = recruits.first().copied()?;
            return Some((vec![r], r.turns.max(1)));
        };
        // A guard: add units until their summed damage a turn kills it in time. It never heals,
        // so the turns after the last arrival are its HP over that damage; a melee winner steps
        // in on the kill, a shooter-only force needs one more turn to walk in.
        let defence = defense_power(v, g).max(1);
        let mut force: Vec<Recruit<'u>> = Vec::new();
        let mut best: Option<(Vec<Recruit<'u>>, i32)> = None;
        for r in recruits.into_iter().take(CONTEST_MAX_FORCE * 2) {
            if force.len() >= CONTEST_MAX_FORCE {
                break;
            }
            force.push(r);
            let damage: i32 = force
                .iter()
                .map(|f| combat::base_damage(offense_power(v, f.unit), defence))
                .sum();
            let melee = force
                .iter()
                .any(|f| crate::composition::charges(f.unit.kind));
            // A lone melee attacker must also survive the guard's reply long enough.
            let reply: i32 = if melee {
                force
                    .iter()
                    .filter(|f| crate::composition::charges(f.unit.kind))
                    .map(|f| combat::base_damage(defence, offense_power(v, f.unit)))
                    .min()
                    .unwrap_or(0)
            } else {
                0
            };
            let arrive = force.iter().map(|f| f.turns).max().unwrap_or(0).max(1);
            let rounds = (g.hp + damage.max(1) - 1) / damage.max(1);
            let melee_survives = !melee
                || force
                    .iter()
                    .filter(|f| crate::composition::charges(f.unit.kind))
                    .any(|f| f.unit.hp > reply * rounds.min(3));
            if !melee_survives {
                continue;
            }
            let turns = arrive - 1 + rounds + i32::from(!melee);
            if best.as_ref().is_none_or(|(_, t)| turns < *t) {
                best = Some((force.clone(), turns));
            }
            // More units only help while they shorten the fight.
            if rounds <= 1 && melee {
                break;
            }
        }
        best
    }

    /// Turns for `u` to walk to `to` on known land it may enter, routing around friendly units.
    fn march_turns_to(&self, u: &UnitView, to: Hex) -> Option<i32> {
        if u.pos == to {
            return Some(0);
        }
        let path = legatum_core::path::find_path(self.movement_map(), u.pos, to, |h| {
            self.ctx
                .move_cost(h)
                .filter(|_| h == to || self.free_tile_for(u, h))
        })?;
        let cost: i32 = path
            .iter()
            .map(|h| self.ctx.move_cost(*h).unwrap_or(1).max(1))
            .sum();
        let moves = u.max_moves.max(1);
        Some((cost + moves - 1) / moves)
    }

    /// What `u` would do otherwise, on the same footing as `denial_gain`: the own city it alone
    /// keeps from falling this turn, or the enemy city it can take this turn, as that city's share
    /// of our population times our plausible share of the race. Routine garrisons, staging and
    /// rest cost nothing here.
    pub(crate) fn contest_cost(&self, u: &UnitView, holder: TeamId) -> i32 {
        let v = self.view();
        let share = self.plausible_share(holder) as i64;
        let ours: i64 = self
            .ctx
            .own_cities
            .iter()
            .map(|c| i64::from(c.population.max(1)))
            .sum::<i64>()
            .max(1);
        let mut cost: i64 = 0;
        if let Some(city) = self.ctx.own_cities.iter().copied().find(|c| c.pos == u.pos) {
            let mut empty = v.clone();
            empty
                .units
                .retain(|o| !(o.pos == city.pos && o.military && v.team_of(o.owner) == v.team));
            let attacks = city_attacks(&empty, city, &|e, h| self.arrives(e, h));
            if city_risk_with(&empty, city, None, &attacks).falls
                && !city_risk_with(&empty, city, Some(u.clone()), &attacks).falls
            {
                cost += share * i64::from(city.population.max(1)) * i64::from(FALL_PERCENT)
                    / (ours * 100);
            }
        }
        // `capture_now_value`: the city's population times the capture chance (percent).
        cost += share * i64::from(self.capture_now_value(u)) / (ours * 100);
        cost.max(0) as i32
    }

    /// Orders for the removal. An empty camp is walked into. At a guarded camp shooters fire
    /// from range (or walk to a tile in range), and melee units attack once beside it when the
    /// forecast wins or risks little (the guard never heals, so every hit counts), otherwise they
    /// close in beside it: a guard never leaves its camp to strike them.
    fn send_denial_force(&mut self, s: &DenialSource, force: &[Recruit<'_>]) {
        let guarded = self
            .ctx
            .enemy_units
            .iter()
            .any(|e| e.pos == s.pos && e.military)
            && self.view().tile(s.pos).is_some_and(|t| t.visible);
        let mut order: Vec<&UnitView> = force.iter().map(|r| r.unit).collect();
        // Shots resolve before the melee steps in.
        order.sort_by_key(|u| (!crate::composition::shooter_only(u.kind, u.ranged), u.id));
        for u in order {
            if self.has_acted(u.id) {
                continue;
            }
            // The late straggler hold (`gather_hold`) would stop the walk short of the Celtic
            // cities' fire: the removal is worth that fire.
            self.contest_units.insert(u.id);
            if !guarded {
                self.go_to(u, s.pos);
            } else if crate::composition::shooter_only(u.kind, u.ranged) {
                self.contest_shot(u, s.pos);
            } else {
                self.contest_charge(u, s.pos);
            }
        }
        self.mil_dest.insert(s.pos);
    }

    fn contest_shot(&mut self, u: &UnitView, target: Hex) {
        let d = u.pos.distance(target);
        if d >= 1 && d <= u.range && self.ranged_unit_clear(u, u.pos, target) {
            self.mil_dest.insert(u.pos);
            self.unit_order(
                u.id,
                legatum_core::Order::RangedAttack { unit: u.id, target },
            );
            return;
        }
        let spot = target
            .ring(u.range.max(1))
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h) && self.ranged_unit_clear(u, *h, target))
            .min_by_key(|h| (h.distance(u.pos), *h));
        match spot {
            Some(h) => self.go_to(u, h),
            None => self.go_to(u, target),
        }
    }

    fn contest_charge(&mut self, u: &UnitView, target: Hex) {
        if u.pos.distance(target) <= 1 || self.reaches_this_turn(u, target) {
            let go = self
                .melee_forecast(u, target)
                .is_some_and(|f| f.win_chance >= 50 || f.loss_chance <= 30);
            if go {
                // Approved by this forecast: movement reconciliation keeps the attack.
                self.assault_attacks.insert(u.id);
                self.unit_order(
                    u.id,
                    legatum_core::Order::MoveUnit {
                        unit: u.id,
                        to: target,
                    },
                );
                return;
            }
        }
        if u.pos.distance(target) <= 1 {
            self.hold_or_upgrade(u);
            return;
        }
        let spot = target
            .ring(1)
            .into_iter()
            .filter(|h| self.free_tile_for(u, *h))
            .min_by_key(|h| (h.distance(u.pos), *h));
        match spot {
            Some(h) => self.go_to(u, h),
            None => self.hold_or_upgrade(u),
        }
    }
}

#[cfg(test)]
#[path = "contest_tests.rs"]
mod tests;

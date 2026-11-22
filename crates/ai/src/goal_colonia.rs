//! Rome's goal, the Colonia Victrix (`legatum_core::goals::GoalRule::Colonia`), in the Hard AI
//! (gene `rome_colonia`, with `roman_founders`).
//!
//! A city founded by a Roman melee veteran of level `goals::COLONIA_LEVEL` (3) on a site within
//! `goals::COLONIA_BORDER_REACH` tiles of a rival's land is the Colonia: 2 VP to whoever holds it.
//! Rome's veterans found its cities, so the goal is a question of which veteran founds where:
//! - **reserve**: while the Colonia is open a level-3 veteran never founds an ordinary city, and
//!   while no unit is level 3 the most experienced veteran below it is kept from founding too
//!   (at level 2 always, at level 1 once Rome has three cities), so the war that earns the third
//!   level has a unit to earn it (`colonia_reserved`). The other veterans found cities as before.
//! - **found**: a level-3 veteran walks to the most defensible reachable site beside a rival's
//!   border and founds there (`plan_colonia`). The city must be held: a Colonia that falls
//!   hands its captor the 2 VP, so a site is priced by its hold risk (the rival army that can
//!   reach it against ours near it, the border's war state, the distance from Rome's cities).
//!   With no such site the veteran waits as a soldier: a rival's land grows, a war moves.
//! - **hold**: the founded Colonia is garrisoned, walled and defended first, and retaken first
//!   if it falls (`goal_cities.rs`).
//!
//! Only the fogged view enters: tile owners are public on explored land
//! (`goals::known_colonia_site`), and a founder's level is its own.

use crate::scripted::Planner;
use legatum_core::settlement::settlement_candidates_with_risky;
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitClass, UnitKind, data, goals};

/// Value of the Colonia's 2 VP in the site search's currency (a held goal city, per VP).
const COLONIA_VALUE_PER_VP: i32 = 400;
/// A Colonia site further than this many turns' walk is not "reachable in reasonable time".
pub(crate) const COLONIA_MAX_TRAVEL: i32 = 12;
/// Each turn of travel keeps a level-3 veteran out of the war that made it.
const COLONIA_TRAVEL_COST: i32 = 25;
/// A site beyond this many hexes from Rome's nearest city is harder to reinforce: each hex
/// beyond adds this much hold risk (percent).
const COLONIA_HOME_REACH: i32 = 5;
const COLONIA_FAR_RISK_PER_HEX: i32 = 6;
/// Cities Rome founds before it keeps its best level-1 veteran from founding, to grow it into
/// the Colonia's founder (level 3 needs 90 XP: in a 160-game smoke without this no Roman unit
/// ever reached it, each level-1 veteran being spent on a city).
const COLONIA_RESERVE_CITIES: usize = 3;
/// Hold-risk percent above which a site is not worth founding: the expected swing is negative.
const COLONIA_MAX_RISK: i32 = 45;

impl Planner<'_> {
    /// Gene `rome_colonia`: Hard Rome pursues its goal now (open, not switched off).
    pub(crate) fn colonia_on(&self) -> bool {
        self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && self.decisions.rome_colonia > 0
            && goals::known_colonia_open(self.view())
    }

    /// Melee units of ours, with their levels.
    fn own_melee(&self) -> impl Iterator<Item = &UnitView> {
        self.ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| data::unit_class(u.kind) == Some(UnitClass::Melee))
    }

    /// Gene `rome_colonia`: `u` is kept from ordinary founding. Every level-3 melee unit while
    /// the Colonia is open; while none exists, the single most experienced veteran below level 3
    /// (highest level, then XP, then lowest id), the one closest to earning the founder: always
    /// at level 2, and at level 1 once Rome has `COLONIA_RESERVE_CITIES` cities (before that
    /// every veteran founds, since expansion comes first).
    pub(crate) fn colonia_reserved(&self, u: &UnitView) -> bool {
        if !self.colonia_on() || data::unit_class(u.kind) != Some(UnitClass::Melee) {
            return false;
        }
        if goals::known_colonia_founder(self.view(), u) {
            return true;
        }
        if self.own_melee().any(|m| m.level >= goals::COLONIA_LEVEL) {
            return false;
        }
        (u.level + 1 == goals::COLONIA_LEVEL
            || (u.level >= 1 && self.ctx.own_cities.len() >= COLONIA_RESERVE_CITIES))
            && self
                .own_melee()
                .filter(|m| m.level >= 1)
                .max_by_key(|m| (m.level, m.xp.unwrap_or(0), std::cmp::Reverse(m.id)))
                .is_some_and(|best| best.id == u.id)
    }

    /// Percent chance a Colonia founded at `site` is lost: the visible rival army that can reach
    /// it against our own near it, more when we are at war with the border's owner, plus its
    /// distance from Rome's nearest city. `founder` is consumed by founding, so it is not counted.
    pub(crate) fn colonia_hold_risk(&self, site: Hex, founder: legatum_core::UnitId) -> i32 {
        let v = self.view();
        let weight = |d: i32| match d {
            0..=2 => 100,
            3 => 75,
            4 => 50,
            5 => 35,
            6 => 25,
            _ => 0,
        };
        let ours: i32 = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.id != founder && u.military && u.kind != UnitKind::Scout)
            .map(|u| crate::analysis::threat_power(v, u) * weight(u.pos.distance(site)) / 100)
            .sum();
        let theirs: i32 = v
            .units
            .iter()
            .filter(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && !u.owner.is_barbarian()
                    && !v.same_team(v.me, u.owner)
                    && !v.allied(v.me, u.owner)
            })
            .map(|u| {
                let at_war = if v.at_war(v.me, u.owner) { 2 } else { 1 };
                at_war * crate::analysis::threat_power(v, u) * weight(u.pos.distance(site)) / 100
            })
            .sum();
        // The rival whose border it is: at war, it comes for the city; much stronger, it can.
        let border_owners: Vec<_> = site
            .within(goals::COLONIA_BORDER_REACH)
            .into_iter()
            .filter_map(|h| v.tile(h).filter(|t| t.explored).and_then(|t| t.owner))
            .filter(|o| !o.is_barbarian() && !v.same_team(v.me, *o))
            .collect();
        let our_army = crate::diplomacy::strength(v, v.team).max(1);
        let chance = if border_owners.iter().any(|o| v.at_war(v.me, *o)) {
            60
        } else if border_owners
            .iter()
            .any(|o| crate::diplomacy::strength(v, v.team_of(*o)) > our_army * 3 / 2)
        {
            40
        } else {
            25
        };
        // A new city's own defence counts for a little.
        let superiority = theirs.saturating_mul(100) / (theirs + ours + 15).max(1);
        let home = self
            .ctx
            .own_cities
            .iter()
            .map(|c| c.pos.distance(site))
            .min()
            .unwrap_or(COLONIA_MAX_TRAVEL);
        let far = (home - COLONIA_HOME_REACH).max(0) * COLONIA_FAR_RISK_PER_HEX;
        (chance * superiority / 100 + far).clamp(0, 95)
    }

    /// The best Colonia site for founder `s`: a known legal site beside a rival's border, safe
    /// to found on (the Settler search's own hazard gates), within `COLONIA_MAX_TRAVEL` turns,
    /// whose hold risk leaves the 2 VP worth more than the swing they risk. Returns the value,
    /// the site and its travel turns.
    pub(crate) fn colonia_site(&self, s: &UnitView) -> Option<(i32, Hex, i32)> {
        self.colonia_site_tally(s, None)
    }

    /// `colonia_site`, also counting why each candidate was dropped when `tally` is given
    /// (decision trace only; never changes the choice).
    fn colonia_site_tally(
        &self,
        s: &UnitView,
        mut tally: Option<&mut std::collections::BTreeMap<&'static str, i32>>,
    ) -> Option<(i32, Hex, i32)> {
        let mut note = |gate: &'static str| {
            if let Some(t) = tally.as_deref_mut() {
                *t.entry(gate).or_default() += 1;
            }
        };
        let v = self.view();
        let gap = v.rule_experiments.min_city_gap(v.faction_of(self.ctx.me));
        let points = goals::spec_for(goals::GoalRule::Colonia).vp as i32;
        let mut best: Option<(i32, Hex, i32)> = None;
        // A Colonia site is beside a rival's border, which is exactly where the settler search
        // flags border risk (an unseen rival city could stand within the spacing gap): the
        // ordinary search drops those sites, which left Rome's veterans with no Colonia site at
        // all. The founder walks there anyway; arriving reveals the land, and an illegal site
        // then drops out of the list on its own.
        for site in settlement_candidates_with_risky(v, self.ctx.site_score.basics(), s) {
            let h = site.pos;
            let Some(turns) = site.travel_turns else {
                note("no route");
                continue;
            };
            let gate = if turns > COLONIA_MAX_TRAVEL {
                Some("too far")
            } else if !goals::known_colonia_site(v, h) {
                Some("not a Colonia site")
            } else if self.enemy_war_tile(h) {
                Some("enemy war tile")
            } else if self.enemy_army_near(h) {
                Some("enemy army near")
            } else if self.ctx.in_danger(h) {
                Some("in danger")
            } else if site.threats > 0 {
                Some("threats")
            } else if self.settle_targets.iter().any(|t| t.distance(h) <= gap) {
                Some("claimed by another settler")
            } else if v.units.iter().any(|u| {
                u.owner != self.ctx.me
                    && u.kind == UnitKind::Settler
                    && u.pos.distance(h) <= v.rule_experiments.min_city_gap(v.faction_of(u.owner))
            }) {
                Some("foreign settler")
            } else if turns + 8 >= self.turns_remaining() {
                Some("too late")
            } else if self
                .campaigns
                .found_tried
                .iter()
                .any(|&(unit, at, turn)| unit == s.id && at == h && turn + 1 == v.turn)
            {
                Some("found tried")
            } else {
                None
            };
            if let Some(gate) = gate {
                note(gate);
                continue;
            }
            let risk = self.colonia_hold_risk(h, s.id);
            if risk > COLONIA_MAX_RISK {
                note("hold risk");
                continue;
            }
            note("passed");
            let value = points * COLONIA_VALUE_PER_VP * (100 - 2 * risk) / 100
                + site.score.max(0) / 4
                - turns * COLONIA_TRAVEL_COST;
            if value > 0 && best.is_none_or(|(b, _, _)| value > b) {
                best = Some((value, h, turns));
            }
        }
        best
    }

    /// Decision trace only: one row per level-3 melee unit saying which founder gate it
    /// fails, or how its candidate sites fared.
    fn trace_colonia_founders(
        &self,
        siege: Option<&(legatum_core::view::CityView, Vec<legatum_core::UnitId>)>,
    ) {
        for u in self
            .own_melee()
            .filter(|u| goals::known_colonia_founder(self.view(), u))
        {
            let gate = if u.hp != data::UNIT_HP {
                Some(format!("hp {}", u.hp))
            } else if self.has_acted(u.id) {
                Some("already ordered".into())
            } else if u.occupation_lock.is_some() {
                Some("locked in a captured city".into())
            } else if !self.roman_unit_can_leave(u) {
                Some("cannot leave".into())
            } else if siege.is_some_and(|(city, ids)| {
                ids.contains(&u.id) && {
                    let rest: Vec<_> = ids.iter().copied().filter(|id| *id != u.id).collect();
                    !self.roman_capture_holds(city, &rest)
                }
            }) {
                Some("needed for siege".into())
            } else {
                None
            };
            let mut tally = std::collections::BTreeMap::new();
            let best = self.colonia_site_tally(&healed(u), Some(&mut tally));
            let sites: Vec<String> = tally.iter().map(|(k, n)| format!("{k}={n}")).collect();
            let mut row = crate::DecisionOption::new(
                "colonia",
                format!(
                    "unit={:?} pos={:?} level={} sites[{}]",
                    u.id,
                    u.pos,
                    u.level,
                    sites.join(",")
                ),
                if gate.is_some() {
                    "gated_out"
                } else if best.is_some() {
                    "candidate"
                } else {
                    "no_site"
                },
            );
            row.gate = gate;
            row.score = best.map(|(value, _, _)| value);
            self.note_option(row);
        }
    }

    /// Gene `rome_colonia`: send this turn's Colonia founder. Of the level-3 melee units free
    /// to leave (not the last defender of a threatened city, not a live siege's unit), the
    /// full-health one with the best site founds there, or walks toward it. Founding needs full
    /// health, so when only wounded veterans have a site, the best of them that is out of
    /// enemy reach rests this turn to heal for it; without this claim the war passes kept
    /// spending a wounded veteran, which never healed. Returns whether a founder was ordered.
    pub(crate) fn plan_colonia(&mut self) -> bool {
        if !self.colonia_on() {
            return false;
        }
        let siege = self.roman_live_siege();
        if self.decision_trace.is_some() {
            self.trace_colonia_founders(siege.as_ref());
        }
        let free: Vec<UnitView> = self
            .own_melee()
            .filter(|u| {
                goals::known_colonia_founder(self.view(), u)
                    && !self.has_acted(u.id)
                    && u.occupation_lock.is_none()
                    && self.roman_unit_can_leave(u)
                    && siege.as_ref().is_none_or(|(city, ids)| {
                        !ids.contains(&u.id) || {
                            let rest: Vec<_> =
                                ids.iter().copied().filter(|id| *id != u.id).collect();
                            self.roman_capture_holds(city, &rest)
                        }
                    })
            })
            .cloned()
            .collect();
        let best = |wounded: bool| {
            free.iter()
                .filter(|u| (u.hp < data::UNIT_HP) == wounded)
                .filter(|u| !wounded || self.colonia_can_rest(u))
                .filter_map(|u| self.colonia_site(&healed(u)).map(|best| (u, best)))
                .max_by_key(|(u, (value, _, _))| (*value, std::cmp::Reverse(u.id)))
        };
        if let Some((s, (_, site, _))) = best(false) {
            let s = s.clone();
            self.colonia_founder = Some(s.id);
            self.settle_targets.push(site);
            if site == s.pos {
                let turn = self.view().turn;
                self.campaigns
                    .found_tried
                    .retain(|&(unit, _, t)| unit != s.id && t + 1 >= turn);
                self.campaigns.found_tried.push((s.id, s.pos, turn));
                self.unit_order(s.id, Order::FoundCity { unit: s.id });
            } else {
                self.escort_sites.push(site);
                self.move_settler(&s, site);
                if !self.has_acted(s.id) {
                    // Its walk would have ended in a city that locks it: it waits a turn.
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            }
            if self.decision_trace.is_some() {
                let tag = format!("unit: {:?}", s.id);
                let order = self
                    .orders
                    .iter()
                    .map(|o| format!("{o:?}"))
                    .rfind(|o| o.contains(&tag));
                self.note_option(crate::DecisionOption::new(
                    "colonia",
                    format!("unit={:?} site={site:?} order={order:?}", s.id),
                    "executed",
                ));
            }
            return true;
        }
        let Some((s, _)) = best(true) else {
            return false;
        };
        let id = s.id;
        self.colonia_founder = Some(id);
        self.mil_dest.insert(s.pos);
        self.unit_order(id, Order::SkipUnit { unit: id });
        true
    }

    /// Whether a wounded veteran can rest where it stands: out of known enemy reach, on land
    /// that heals it (Rome's own, or nobody's).
    fn colonia_can_rest(&self, u: &UnitView) -> bool {
        self.exposure_at(u.pos) == 0
            && self
                .view()
                .tile(u.pos)
                .is_some_and(|t| t.owner.is_none_or(|o| o == self.ctx.me))
    }

    /// Gene `rome_colonia`, called by `unit_order` for every move: keeps the would-be founder
    /// (`colonia_reserved`) out of a city whose occupation lock would hold it there until
    /// Peace. The lock is the city's, so a locked veteran can never be swapped out: in a
    /// 320-game run, 35 of the 55 turns Rome had a level-3 veteran, every one was locked in a
    /// city it had taken. The move is any `MoveUnit`/`MoveUnitPath` ending on a city that
    /// carries `CityView::occupation_lock`: a capture, or entering Rome's own occupied city.
    ///
    /// `None` lets the order stand. Otherwise the founder stands aside: another free Roman
    /// melee unit that reaches the city this turn takes the same move (`Some` of whether its
    /// order went in), or, for an enemy city, the capture waits a turn when such a unit is
    /// within two turns (`Some(false)`), once: a capture that waited last turn goes ahead.
    /// Otherwise the founder captures: Rome does not give up a city it would have taken. It
    /// keeps fighting in the field either way.
    pub(crate) fn colonia_lock_guard(
        &mut self,
        unit: legatum_core::UnitId,
        order: &Order,
    ) -> Option<bool> {
        let to = match order {
            Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => *to,
            _ => return None,
        };
        let v = self.view();
        let city = v
            .cities
            .iter()
            .find(|c| c.pos == to && c.occupation_lock.is_some())?;
        let enemy = !v.same_team(v.me, city.owner);
        let u = self.ctx.own_units.iter().find(|u| u.id == unit).copied()?;
        if u.pos == to || u.occupation_lock.is_some() || !self.colonia_reserved(u) {
            return None;
        }
        let u = u.clone();
        let note = |p: &Self, outcome: &'static str, why: String| {
            if p.decision_trace.is_some() {
                let mut row = crate::DecisionOption::new(
                    "colonia",
                    format!("unit={:?} lock city at {to:?}", u.id),
                    outcome,
                );
                row.gate = Some(why);
                p.note_option(row);
            }
        };
        // The founder's own Colonia walk (a retreat home included) never ends in such a city,
        // and a city it stood aside from this turn is not entered behind its substitute.
        if self.colonia_founder == Some(unit) || self.colonia_aside.contains(&to) {
            note(
                self,
                "stood_aside",
                if self.colonia_founder == Some(unit) {
                    "the founder's walk does not end in a city that would lock it".into()
                } else {
                    "the founder already stood aside from this city this turn".into()
                },
            );
            return Some(false);
        }
        let others: Vec<UnitView> = self
            .own_melee()
            .filter(|m| m.id != unit && !self.has_acted(m.id) && !self.colonia_reserved(m))
            .filter(|m| m.occupation_lock.is_none() && self.ctx.reachable(m.pos, to))
            .cloned()
            .collect();
        let substitute = others
            .iter()
            .filter(|m| m.pos.distance(to) <= m.max_moves.max(1))
            .max_by_key(|m| {
                (
                    crate::analysis::attack_power(self.view(), m),
                    std::cmp::Reverse(m.id),
                )
            })
            .cloned();
        let near = others
            .iter()
            .any(|m| m.pos.distance(to) <= 2 * m.max_moves.max(1));
        if let Some(m) = substitute {
            note(
                self,
                "stood_aside",
                format!(
                    "founder kept out of the occupation lock; {:?} moves instead",
                    m.id
                ),
            );
            self.colonia_aside.push(to);
            return Some(self.unit_order(m.id, Order::MoveUnit { unit: m.id, to }));
        }
        let turn = self.view().turn;
        let waited = self
            .campaigns
            .colonia_waits
            .iter()
            .any(|&(at, t)| at == to && t + 1 == turn);
        if enemy && near && !waited {
            self.campaigns.colonia_waits.retain(|&(_, t)| t + 1 >= turn);
            if !self.campaigns.colonia_waits.contains(&(to, turn)) {
                self.campaigns.colonia_waits.push((to, turn));
            }
            self.colonia_aside.push(to);
            note(
                self,
                "stood_aside",
                "founder kept out of the occupation lock; the capture waits for another unit"
                    .into(),
            );
            return Some(false);
        }
        note(
            self,
            "allowed",
            if waited {
                "the capture already waited a turn".into()
            } else {
                "no other unit can take the city".into()
            },
        );
        None
    }
}

/// The veteran as it will be once healed: founding needs full health, and the site search
/// lists sites only for a founder that has it.
fn healed(u: &UnitView) -> UnitView {
    UnitView {
        hp: data::UNIT_HP,
        ..u.clone()
    }
}

#[cfg(test)]
#[path = "goal_colonia_tests.rs"]
mod tests;

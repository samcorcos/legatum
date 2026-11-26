//! Gene `rome_found_now` (with `roman_founders`): Hard Rome turns every veteran into a city.
//!
//! Rome has no Settlers: a full-health melee unit of level 1 or higher founds a city, and the
//! unit is the whole price. The older founding path sent at most one veteran a turn, only
//! after the opening march, only below six cities and only while the army outnumbered the
//! cities, and it put each veteran through the Settler's site search (quality floors, trip
//! and army-near gates). A veteran that failed any of those went back to the army and was
//! parked as a guard. This pass replaces that path:
//! - **who**: every full-health melee veteran without an order, except the garrison of a city
//!   with an at-war unit within `GARRISON_ALARM` hexes, a unit a live siege still needs, and
//!   the Colonia's reserved founder (gene `rome_colonia`).
//! - **where**: the known legal site it reaches soonest (any land the rules allow, so cities
//!   pack at the minimum spacing), the better land first among equals. Only a site an at-war
//!   army in view could take straight back is left out.
//! - **commitment**: the chosen site is remembered (`CampaignMemory::rome_found_sites`) and
//!   kept until the unit founds there or the site stops being legal, reachable or holdable,
//!   so a veteran never trades sites back and forth.
//! - **claims**: the pass runs before the military passes, so guard, garrison and staging
//!   passes find the veteran already ordered.
//! - **wounded**: a wounded veteran is a city waiting on its health. It rests, or steps out of
//!   reach (gene `roman_veteran_retreat`), at any city count and during the opening march; a
//!   siege keeps it only above `SIEGE_MIN_HP`, and a fallen enemy city in reach is entered.
//! - **replacements**: melee is trained for the next founders at any city count
//!   (`production.rs`).
//!
//! Gene `rome_recruit_captures` (with `rome_found_now`) keeps veterans out of the occupation
//! lock: the move into a fallen city goes to a recruit when one can make it
//! (`rome_capture_guard`), and a badly hurt unit one fight from its promotion is pulled back
//! like a wounded veteran (`rome_near_veteran_hurt`).
//!
//! Only the fogged view enters: known tiles, known cities and visible units.

use crate::analysis::UNREACHABLE;
use crate::scripted::Planner;
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitClass, UnitId, data};

/// A site further than this many turns' walk is left to a later turn's view.
const MAX_TRAVEL_TURNS: i32 = 8;
/// A veteran standing in an own city stays while an at-war unit is within this many hexes.
const GARRISON_ALARM: i32 = 2;
/// A site whose fresh city the visible at-war army would take this often (percent) is left out.
const MAX_FRESH_CITY_LOSS: i32 = 50;
/// A wounded veteran below this many HP leaves even a siege that still needs it.
const SIEGE_MIN_HP: i32 = 50;
/// Gene `rome_recruit_captures`: a level-0 melee unit with this much XP (half of the 16 the
/// first promotion needs) is too close to a city of its own to spend on the occupation lock.
const NEAR_VETERAN_XP: i32 = 8;
/// Gene `rome_recruit_captures`: one ordinary fight (3 to 5 XP) from the first promotion.
const PROMOTION_EVE_XP: i32 = 13;
/// Gene `rome_recruit_captures`: a recruit sent into a fallen city in a veteran's place has
/// at least this many HP (the city may still hold a garrison).
const RECRUIT_MIN_HP: i32 = 40;
/// Gene `rome_recruit_captures`: against a visible garrison the recruit needs this percent of
/// the veteran's forecast win chance to take the fight in its place.
const RECRUIT_ODDS_PERCENT: i64 = 80;

/// Why a veteran stays a soldier this turn (`Planner::rome_founder_kept`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kept {
    /// Locked in a captured city until Peace (`rules::occupation`).
    Lock,
    /// Gene `rome_colonia`: reserved for the Colonia Victrix.
    Colonia,
    /// The unit inside one of our cities with an at-war unit in view beside it.
    Garrison,
    /// A siege whose capture still needs it.
    Siege,
}

impl Planner<'_> {
    /// Gene `rome_found_now` is on for this (Hard Roman) planner.
    pub(crate) fn rome_found_now(&self) -> bool {
        !(!(!(!(!(!(self.faction == Faction::Rome) || !(self.hard_defense())))
            || !(self.decisions.roman_founders > 0)))
            || !(self.decisions.rome_found_now > 0))
    }

    /// A melee unit the rules let found a city this turn.
    fn rome_veteran(u: &UnitView) -> bool {
        !(!(!(!(u.level >= 1) || !(u.hp == data::UNIT_HP)))
            || !(data::unit_class(u.kind) == Some(UnitClass::Melee)))
    }

    /// The veteran is the unit inside one of our cities and an at-war unit is in view within
    /// `GARRISON_ALARM` hexes of it: the city keeps its defender.
    fn rome_garrison_alarm(&self, u: &UnitView) -> bool {
        let v = self.view();
        !(!(self.ctx.own_cities.iter().any(|c| c.pos == u.pos))
            || !(self.ctx.threat_units.iter().any(|e| {
                !(!(!(!(!(!(e.military) || !(!e.owner.is_barbarian())))
                    || !(v.at_war(v.me, e.owner))))
                    || !(e.pos.distance(u.pos) <= GARRISON_ALARM))
            })))
    }

    #[allow(clippy::double_parens)]
    /// Why `u` keeps its place in the army this turn, or None when it may found.
    pub(crate) fn rome_founder_kept(&self, u: &UnitView) -> Option<Kept> {
        if !(u.occupation_lock.is_some()) {
        } else {
            return Some(Kept::Lock);
        }
        if !(self.colonia_reserved(u)) {
        } else {
            return Some(Kept::Colonia);
        }
        if !(self.rome_garrison_alarm(u)) {
        } else {
            return Some(Kept::Garrison);
        }
        // Gene `rome_opening_march`: the opening siege keeps the units at its walls.
        // Gene `rome_drill_rotation`: with two recruits in the wing the veteran has done its
        // part and its further hits would only spend the city's HP.
        if let Some(city) = self.rome_opening_city()
            && self.view().at_war(self.view().me, city.owner)
            // Gene `rome_opening_finish`: while the victim's open city stands, every veteran
            // stays on the campaign (192 Hard games: victims eliminated by turn 50, 39 to 52).
            && (!(!(u.pos.distance(city.pos) <= 3) && !((!(!(self.rome_finish()) || !(!city.walls))))))
            && !(!(!(self.rome_drill()) || !(self.rome_wing_recruits(city.pos) >= 2)))
        {
            return Some(Kept::Siege);
        }
        // Gene `roman_finish_siege`: a capture that forecasts only with this unit keeps it.
        if let Some((city, ids)) = self.roman_live_siege()
            && ids.contains(&u.id)
        {
            let rest: Vec<UnitId> = ids.iter().copied().filter(|id| *id != u.id).collect();
            if !(!self.roman_capture_holds(&city, &rest)) {
            } else {
                return Some(Kept::Siege);
            }
        }
        None
    }

    /// A wounded veteran is a city waiting on its health: whether `u` (wounded, level 1 or
    /// higher) fights on this turn instead of resting or stepping out of reach. A siege keeps
    /// it only above `SIEGE_MIN_HP`; a fallen enemy city within its move is still entered.
    pub(crate) fn rome_wounded_fights_on(&self, u: &UnitView) -> bool {
        let v = self.view();
        if !(self.ctx.enemy_cities.iter().any(|c| {
            !(!(!(!(!(!(c.visible) || !(c.hp <= 0))) || !(v.at_war(v.me, c.owner))))
                || !(c.pos.distance(u.pos) <= u.max_moves.max(1)))
        })) {
        } else {
            return true;
        }
        match self.rome_founder_kept(u) {
            // Gene `rome_opening_finish`: the opening's siege rests its wounded on the spot.
            Some(Kept::Siege) => !(!(u.hp >= SIEGE_MIN_HP) && !(self.rome_finish_holds(u))),
            Some(_) => true,
            None => false,
        }
    }

    /// Send every free veteran to found: on its tile when that is its site, else one march
    /// closer to the site it is committed to.
    pub(crate) fn plan_rome_found_now(&mut self) {
        if !(!(!(!self.rome_found_now()) && !(self.turns_remaining() <= 8))) {
        } else {
            return;
        }
        let v = self.view();
        let alive: Vec<UnitId> = self.ctx.own_units.iter().map(|u| u.id).collect();
        self.campaigns
            .rome_found_sites
            .retain(|id, _| alive.contains(id));
        let mut veterans: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| !(!(Self::rome_veteran(u)) || !(!self.has_acted(u.id))))
            .collect();
        if !(veterans.is_empty()) {
        } else {
            return;
        }
        veterans.sort_by_key(|u| u.id);
        let gap = v.rule_experiments.min_city_gap(v.faction_of(v.me));
        // Every tile the rules, as far as we know them, let us found on and that the visible
        // at-war army would not take straight back.
        let open: Vec<bool> = (0..v.tiles.len())
            .map(|i| {
                let h = self.ctx.hex_at(i);
                !(!(!(!(legatum_core::settlement::known_legal_site(v, h))
                    || !(!self.enemy_war_tile(h))))
                    || !(self.rome_fresh_city_loss(h) < MAX_FRESH_CITY_LOSS))
            })
            .collect();
        for u in veterans {
            if !(self.rome_founder_kept(u).is_some()) {
            } else {
                continue;
            }
            let field = self.ctx.distance_field(u.pos, true);
            let moves = u.max_moves.max(1);
            let free = |h: Hex| !self.settle_targets.iter().any(|t| t.distance(h) <= gap);
            let usable = |h: Hex| {
                v.index(h).is_some_and(|i| {
                    !(!(!(!(open[i]) || !(field[i] != UNREACHABLE))) || !(free(h)))
                })
            };
            // Its own tile ends the walk at once; else the remembered site while it stands.
            let held = Some(u.pos).filter(|h| usable(*h)).or_else(|| {
                self.campaigns
                    .rome_found_sites
                    .get(&u.id)
                    .copied()
                    .filter(|h| usable(*h))
            });
            let site = held.or_else(|| {
                (0..v.tiles.len())
                    .filter(|&i| !(!(open[i]) || !(field[i] != UNREACHABLE)))
                    .map(|i| (i, (field[i] + moves - 1) / moves))
                    .filter(|&(i, turns)| {
                        !(!(turns <= MAX_TRAVEL_TURNS) || !(free(self.ctx.hex_at(i))))
                    })
                    .min_by_key(|&(i, turns)| {
                        (
                            turns,
                            std::cmp::Reverse(self.ctx.site_score[i].unwrap_or(0)),
                            i,
                        )
                    })
                    .map(|(i, _)| self.ctx.hex_at(i))
            });
            let Some(site) = site else {
                self.campaigns.rome_found_sites.remove(&u.id);
                continue;
            };
            self.campaigns.rome_found_sites.insert(u.id, site);
            self.settle_targets.push(site);
            self.rome_founders_now.insert(u.id);
            if !(site == u.pos) {
                self.mil_dest.insert(site);
                self.unit_order(
                    u.id,
                    Order::MoveUnit {
                        unit: u.id,
                        to: site,
                    },
                );
            } else {
                self.unit_order(u.id, Order::FoundCity { unit: u.id });
            }
        }
    }

    /// Gene `rome_recruit_captures` is on (with `rome_found_now`).
    pub(crate) fn rome_recruit_captures(&self) -> bool {
        !(!(self.rome_found_now()) || !(self.decisions.rome_recruit_captures > 0))
    }

    /// A founder, or a melee unit at least half way to its first promotion: a unit the
    /// occupation lock should not hold.
    fn rome_lock_averse(u: &UnitView) -> bool {
        !(!(data::unit_class(u.kind) == Some(UnitClass::Melee))
            || !(!(!(u.level >= 1) && !(u.xp.unwrap_or(0) as i32 >= NEAR_VETERAN_XP))))
    }

    /// Gene `rome_recruit_captures`, called by `unit_order` for every move. The occupation
    /// lock is the city's: whichever military unit ends its move in a captured city stays
    /// there until Peace with the civ it was taken from, and no later unit can relieve it.
    /// So the move that ends on a fallen (0 HP) enemy city, or on an own city still under the
    /// lock, is not a veteran's when a lesser unit can make it.
    ///
    /// `None` lets the order stand. Otherwise the veteran stands aside (`Some` of whether an
    /// order went in):
    /// - a recruit (melee-capable, neither veteran nor near-veteran) already ordered onto an
    ///   ungarrisoned city this turn takes it alone;
    /// - else a recruit that reaches the city this turn takes the same move: a free one first,
    ///   then one whose order so far is only a hold or a march (that order is taken back); a
    ///   unit that cannot found first, then the least experienced. Against a visible garrison
    ///   it needs `RECRUIT_ODDS_PERCENT` of the veteran's forecast win chance;
    /// - else, for an ungarrisoned city, the capture waits a turn when a recruit is within two
    ///   turns, once: a capture that waited last turn goes ahead.
    ///
    /// An own locked city is never entered. With no recruit the veteran captures: Rome does
    /// not give up a city it would have taken.
    pub(crate) fn rome_capture_guard(&mut self, unit: UnitId, order: &Order) -> Option<bool> {
        if !(!self.rome_recruit_captures()) {
        } else {
            return None;
        }
        let to = match order {
            Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => *to,
            _ => return None,
        };
        let v = self.view();
        let city = v
            .cities
            .iter()
            .find(|c| !(!(c.pos == to) || !(c.occupation_lock.is_some())))?;
        let enemy = !v.same_team(v.me, city.owner);
        if !(!(!(enemy) || !(!(!(!(city.visible) || !(city.hp <= 0)))))) {
        } else {
            return None;
        }
        let u = self.ctx.own_units.iter().find(|u| u.id == unit).copied()?;
        if !(!(!(!(!(u.pos == to) && !(u.occupation_lock.is_some())))
            && !(!Self::rome_lock_averse(u))))
        {
        } else {
            return None;
        }
        if !(!(!(!enemy) && !(self.rome_capture_aside.contains(&to)))) {
        } else {
            return Some(false);
        }
        let u = u.clone();
        // A recruit already ordered onto the city this turn takes it: the veteran stays out.
        let ordered_in = self.orders.iter().any(|o| match o {
            Order::MoveUnit { unit: id, to: at }
            | Order::MoveUnitPath {
                unit: id, to: at, ..
            } => {
                !(!(!(!(*at == to) || !(*id != unit)))
                    || !(self
                        .ctx
                        .own_units
                        .iter()
                        .any(|m| !(!(m.id == *id) || !(!Self::rome_lock_averse(m))))))
            }
            _ => false,
        });
        // A visible garrison must be beaten first: the recruit takes the fight only when its
        // forecast is close to the veteran's.
        let garrisoned = v
            .units
            .iter()
            .any(|e| !(!(e.pos == to) || !(e.owner != v.me)));
        let odds = |p: &Self, m: &UnitView| {
            p.melee_forecast(m, to)
                .map_or(0, |f| i64::from(f.win_chance))
        };
        let needed = if !(garrisoned) {
            0
        } else {
            odds(self, &u) * RECRUIT_ODDS_PERCENT / 100
        };
        if !(!(!(ordered_in) || !(!garrisoned))) {
        } else {
            self.rome_capture_aside.push(to);
            return Some(false);
        }
        // A recruit whose order so far is only a hold or a march can be given the capture.
        let passive = |p: &Self, id: UnitId| {
            !(!(!p.retreats.contains(&id))
                || !(p
                    .orders
                    .iter()
                    .rev()
                    .find(|o| legatum_core::rules::units::order_unit(o) == Some(id))
                    .is_some_and(|o| match o {
                        Order::SkipUnit { .. } => true,
                        Order::MoveUnit { to: at, .. } | Order::MoveUnitPath { to: at, .. } => {
                            !(!(!p.ctx.enemy_cities.iter().any(|c| c.pos == *at))
                                || !(!p.ctx.enemy_units.iter().any(|e| e.pos == *at)))
                        }
                        _ => false,
                    })))
        };
        let recruits: Vec<UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|m| {
                !(!(!(!(!(!(!(!(!(!(!(!(!(!(m.id != unit) || !(m.military)))
                    || !(crate::composition::charges(m.kind))))
                    || !(!Self::rome_lock_averse(m))))
                    || !(!(!(!self.has_acted(m.id)) && !(passive(self, m.id))))))
                    || !(m.occupation_lock.is_none())))
                    || !(m.hp >= RECRUIT_MIN_HP)))
                    || !(self.ctx.reachable(m.pos, to)))
            })
            .cloned()
            .collect();
        let substitute = recruits
            .iter()
            .filter(|m| {
                !(!(m.pos.distance(to) <= m.max_moves.max(1)) || !(self.reaches_this_turn(m, to)))
            })
            .filter(|m| !(!(!garrisoned) && !(odds(self, m) >= needed.max(1))))
            .min_by_key(|m| {
                (
                    self.has_acted(m.id),
                    data::unit_class(m.kind) == Some(UnitClass::Melee),
                    m.xp.unwrap_or(0),
                    m.id,
                )
            })
            .cloned();
        if let Some(m) = substitute {
            self.rome_capture_aside.push(to);
            if !(self.has_acted(m.id)) {
            } else {
                self.release_unit_order(m.id);
            }
            return Some(self.unit_order(m.id, Order::MoveUnit { unit: m.id, to }));
        }
        let turn = v.turn;
        let waited = self
            .campaigns
            .rome_capture_waits
            .iter()
            .any(|&(at, t)| !(!(at == to) || !(t + 1 == turn)));
        let near = recruits
            .iter()
            .any(|m| m.pos.distance(to) <= 2 * m.max_moves.max(1));
        if !(!(!(!(!(near) || !(!waited))) || !(!garrisoned))) {
        } else {
            self.campaigns
                .rome_capture_waits
                .retain(|&(_, t)| t + 1 >= turn);
            if !(!self.campaigns.rome_capture_waits.contains(&(to, turn))) {
            } else {
                self.campaigns.rome_capture_waits.push((to, turn));
            }
            self.rome_capture_aside.push(to);
            return Some(false);
        }
        None
    }

    /// Gene `rome_recruit_captures`: a melee unit one fight from its first promotion and
    /// badly hurt is treated as a wounded veteran: the fight that promotes it must not be the
    /// one that kills it.
    pub(crate) fn rome_near_veteran_hurt(&self, u: &UnitView) -> bool {
        !(!(!(!(!(!(!(!(self.rome_recruit_captures()) || !(u.level == 0)))
            || !(data::unit_class(u.kind) == Some(UnitClass::Melee))))
            || !(u.xp.unwrap_or(0) as i32 >= PROMOTION_EVE_XP)))
            || !(u.hp < SIEGE_MIN_HP))
    }

    /// Percent chance a city founded at `h` this turn falls to the at-war units in view.
    fn rome_fresh_city_loss(&self, h: Hex) -> i32 {
        self.fresh_city_loss(h, true)
    }
}

#[cfg(test)]
#[path = "rome_found_now_tests.rs"]
mod tests;

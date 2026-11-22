//! The held-goal cities of Rome (the Colonia Victrix) and Egypt (the royal tomb), as every empire
//! sees them. Both are public (`PlayerView::colonias`, `royal_tombs`), worth their goal's 2 VP to
//! whoever owns the city, and change hands with it, like the Courier Hall's city. So they are
//! valued and defended exactly as the Hall city is (`goal_hall.rs`):
//! - **defend**: whoever holds one garrisons, walls and defends it first, with the Hall's wider
//!   margin (`holds_goal_city`, `goal_city_guard`); Egypt also guards the city its tomb would
//!   stand in while the work is wanted or on (`goal_tomb::tomb_guard_site`);
//! - **capture**: a rival prices taking one by the 2 VP swing, the founder retaking its own
//!   higher (`goal_city_in_city_value`, folded into `hoard_denial_value`; `capture_goal_swing`
//!   for the win model; `goal_city_goal_denial` in `held_goal_denial` for war declarations);
//! - **a working Egypt**: its public work leaves every city without builds for the turns still
//!   owed (`tomb_weakness_value`, beside the Aztecs' New Fire in `swarm_weakness_value`), and the
//!   city its tomb will stand in is priced like a Great Work under construction
//!   (`tomb_site_denial`, in `rival_objective_denial`).
//!
//! Gene `goal_city_capture` (percent) scales every rival's valuation; 0 turns it off. The
//! founders' own guard follows genes `rome_colonia` and `egypt_tomb`.

use crate::scripted::Planner;
use legatum_core::view::{CityView, GoalCityView};
use legatum_core::{Faction, PlayerId, goals};

/// A held goal city is worth this much per VP when capturing it (the swing is double: the
/// holder loses the points as the captor gains them). The Courier Hall's value.
const GOAL_CITY_SWING_VALUE_PER_VP: i32 = 100;
/// The empire whose goal it is, retaking its own city: its points come back.
const GOAL_CITY_RETAKE_VALUE_PER_VP: i32 = 250;
/// Per turn of tomb work still owed, the value of a working Egypt's city that can build no
/// defender until the tomb stands (at most 15 turns: 60).
const TOMB_WEAKNESS_PER_TURN: i32 = 4;

impl Planner<'_> {
    /// Every standing Colonia Victrix and royal tomb.
    fn goal_cities(&self) -> impl Iterator<Item = &GoalCityView> {
        self.view()
            .colonias
            .iter()
            .chain(self.view().royal_tombs.iter())
    }

    fn goal_city_at(&self, city: &CityView) -> Option<&GoalCityView> {
        self.goal_cities().find(|g| g.city == city.id)
    }

    /// The goal's VP (`data::FACTION_GOAL_VP` through the goal spec).
    fn goal_city_vp(&self) -> i32 {
        goals::spec_for(goals::GoalRule::Colonia).vp as i32
    }

    /// We hold `c` and it carries a Colonia or a tomb: our own (genes `rome_colonia`,
    /// `egypt_tomb`), or one we captured (gene `goal_city_capture`).
    pub(crate) fn holds_goal_city(&self, c: &CityView) -> bool {
        self.goal_city_at(c).is_some_and(|g| {
            g.holder == self.ctx.me
                && if g.of == self.ctx.me {
                    match self.faction {
                        Faction::Rome => self.decisions.rome_colonia > 0,
                        Faction::Egypt => self.decisions.egypt_tomb > 0,
                        _ => false,
                    }
                } else {
                    self.decisions.goal_city_capture > 0
                }
        })
    }

    /// `c` holds a goal city, or is the city Egypt's tomb is about to stand in: the first place
    /// to garrison and wall.
    pub(crate) fn goal_city_guard(&self, c: &CityView) -> bool {
        self.holds_goal_city(c)
            || (self.faction == Faction::Egypt
                && self.decisions.egypt_tomb > 0
                && crate::goal_tomb::tomb_guard_site(self.view(), self.decisions.goal_open_end > 0)
                    == Some(c.id))
    }

    /// A goal city without its garrison: units are the first thing it builds.
    pub(crate) fn goal_city_unguarded(&self, c: &CityView) -> bool {
        self.goal_city_guard(c) && !self.hall_garrisoned(c)
    }

    /// Value of capturing `city` if it carries a Colonia or a tomb its holder (not on our team)
    /// scores: the captor scores its 2 VP and the holder loses them. Its founder values retaking
    /// it higher; a Celtic capture razes it, so it only denies.
    pub(crate) fn goal_city_in_city_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        let Some(g) = self.goal_city_at(city) else {
            return 0;
        };
        if v.same_team(v.me, g.holder) || self.decisions.goal_city_capture == 0 {
            return 0;
        }
        let per_vp = if g.of == v.me {
            GOAL_CITY_RETAKE_VALUE_PER_VP
        } else if self.sacks_city(city) {
            GOAL_CITY_SWING_VALUE_PER_VP / 2
        } else {
            GOAL_CITY_SWING_VALUE_PER_VP
        };
        self.goal_city_vp() * per_vp * self.decisions.goal_city_capture / 100
    }

    /// The live goal points a capture of `city` transfers (to us, unless we raze it) and
    /// removes (from its holder), for `capture_goal_swing`.
    pub(crate) fn goal_city_swing(&self, city: &CityView) -> (i32, i32) {
        let v = self.view();
        if self.decisions.goal_city_capture == 0 {
            return (0, 0);
        }
        let Some(g) = self.goal_city_at(city) else {
            return (0, 0);
        };
        if v.team_of(g.holder) == v.team {
            return (0, 0);
        }
        let vp = self.goal_city_vp();
        (if self.sacks_city(city) { 0 } else { vp }, vp)
    }

    /// Denial value of a holder scoring `of`'s goal when `of` is a Rome or an Egypt (its
    /// Colonia or tomb city, which capture takes): the swing, more when it is ours to retake.
    pub(crate) fn goal_city_goal_denial(&self, of: PlayerId) -> i32 {
        let v = self.view();
        if self.decisions.goal_city_capture == 0
            || !v.players.iter().any(|q| {
                q.id == of && matches!(q.rule(), goals::GoalRule::Colonia | goals::GoalRule::Tomb)
            })
        {
            return 0;
        }
        let per_vp = if of == v.me {
            GOAL_CITY_RETAKE_VALUE_PER_VP
        } else {
            GOAL_CITY_SWING_VALUE_PER_VP
        };
        self.goal_city_vp() * per_vp * self.decisions.goal_city_capture / 100
    }

    /// A rival Egypt at work on its tomb (public): none of its cities can build a defender for
    /// the turns still owed, so each is softer for that long.
    pub(crate) fn tomb_weakness_value(&self, city: &CityView) -> i32 {
        let v = self.view();
        if self.decisions.goal_city_capture == 0 || v.same_team(v.me, city.owner) {
            return 0;
        }
        v.players
            .iter()
            .find(|p| p.id == city.owner && p.tomb_work)
            .map_or(0, |p| {
                let left = goals::TOMB_TURNS.saturating_sub(p.tomb_turns) as i32;
                TOMB_WEAKNESS_PER_TURN * left * self.decisions.goal_city_capture / 100
            })
    }

    /// A rival Egypt at work on its tomb will score 2 VP in the city the tomb stands in (its
    /// capital, else its largest city): priced as a Great Work under construction
    /// (`rival_objective_denial`), more as the count grows.
    pub(crate) fn tomb_site_denial(&self, city: &CityView) -> i32 {
        let v = self.view();
        if self.decisions.goal_city_capture == 0 || v.same_team(v.me, city.owner) {
            return 0;
        }
        let Some(p) = v.players.iter().find(|p| p.id == city.owner && p.tomb_work) else {
            return 0;
        };
        if crate::goal_tomb::rival_tomb_site(v, p.id) != Some(city.id) {
            return 0;
        }
        let progress = p.tomb_turns.min(goals::TOMB_TURNS) as i32;
        (100 + 200 * progress / goals::TOMB_TURNS as i32) * self.decisions.goal_city_capture / 100
    }
}

#[cfg(test)]
#[path = "goal_cities_tests.rs"]
mod tests;

//! Egypt's goal, the royal tomb (`legatum_core::goals::GoalRule::Tomb`), in the AI: when Egypt
//! starts the work, when it pauses and resumes it, and which city it guards first.
//!
//! The work is an on/off action with a public count, like the reserved Sakoku vow
//! (`goal_sakoku.rs`). While it is on, no Egyptian city builds or grows for up to
//! `goals::TOMB_TURNS` turn ends (Gold and Science go on, and so do Gold purchases); then the
//! tomb stands in the capital (else the largest city), worth 2 VP to whoever holds that city.
//! So the work is the moment Egypt is weakest and rivals can see it. Decisions, all from the
//! fogged view (gene `egypt_tomb`):
//! - **start** when the 2 VP decide something (the rite test of `goal_actions::rite_pays`, with
//!   the lead the team earns while the work runs, as the Courier Hall's build), the work can
//!   finish with time to hold its points, no enemy army stands near a city, and the would-be
//!   tomb city (`TombView::site`) is garrisoned. A count already started is finished whenever it
//!   is safe: the turns already given are sunk and the rest is cheaper every turn.
//! - **pause** when an enemy army comes near a city, so the cities can build defenders again; a
//!   count further along tolerates a larger army (its turns are kept, so finishing is worth more
//!   as it grows), and a tomb a turn or two from standing is finished. Also when the game would
//!   end first.
//! - **guard** the would-be tomb city while the work is wanted, and the tomb city once it stands
//!   (`goal_cities.rs`), like the Courier Hall's city.
//!
//! Production just before the work does not need its own rule: an item's progress is kept while
//! the work is on (the city's Production and surplus Food are taken as yields, not its stored
//! progress, `economy.rs::yields_for_item`), so a half-built Walls resumes where it stopped. The
//! one thing that cannot wait 15 turns is the garrison, which the start test requires and the
//! production planner builds first in the site city (`goal_city_unguarded`).

use crate::goal_actions::{hold_turns, is_late, last_turn, standing};
use legatum_core::orders::Action;
use legatum_core::rules::actions::TombView;
use legatum_core::{CityId, PlayerView, UnitKind, goals};

/// Share of the game after which the tomb's 2 VP are banked whatever the race looks like (the
/// Courier Hall's window: both cost a stretch of the empire's output for a held city).
pub(crate) const TOMB_LATE_PERCENT: u32 = 60;
/// VP the team is expected to earn while the work runs: the tomb pays when it would reach the
/// target within this lead (as `goal_hall::HALL_LEAD_VP`).
const TOMB_LEAD_VP: u32 = 1;
/// Military units at or beside the would-be tomb city before the work starts (one on the city
/// tile): no city can build a defender for the next 15 turns.
pub(crate) const TOMB_GARRISON: usize = 2;
/// A tomb this many turn ends from standing is finished through a threat: pausing would give
/// the cities one or two turns of building and cost the points' arrival as long.
const TOMB_FINISH_THROUGH: u32 = 2;
/// Every this many turn ends already given to the tomb, one more enemy unit at the gates is
/// tolerated before the work pauses (turns 0-4: any unit pauses it; 5-9: two; 10-12: three).
const TOMB_PAUSE_TOLERANCE_TURNS: u32 = 5;
/// An enemy army this close to one of our cities stops (or pauses) the work.
const THREAT_HEXES: i32 = 3;
/// Before the work starts the approaches must be clear this far out: an army two turns away
/// arrives while no city can answer it.
const START_CLEAR_HEXES: i32 = 5;

/// Egypt's own tomb view, while its goal is open (not switched off, not standing).
fn open_tomb(view: &PlayerView) -> Option<&TombView> {
    view.actions.tomb.as_ref().filter(|t| t.tomb.is_none())
}

/// Turn ends of work still owed.
fn turns_left(t: &TombView) -> u32 {
    t.target.saturating_sub(t.turns)
}

/// At-war (or raiding barbarian) military units within `hexes` of one of our cities, counted.
fn enemies_near_cities(view: &PlayerView, hexes: i32, only: Option<CityId>) -> usize {
    let mine: Vec<_> = view
        .cities
        .iter()
        .filter(|c| c.owner == view.me && only.is_none_or(|id| c.id == id))
        .collect();
    view.units
        .iter()
        .filter(|u| {
            u.military
                && u.kind != UnitKind::Scout
                && u.owner != view.me
                && if u.owner.is_barbarian() {
                    !view.barbarians_spare_me()
                } else {
                    view.teams_at_war(view.team, view.team_of(u.owner))
                }
        })
        .filter(|u| {
            let reach = if u.owner.is_barbarian() {
                hexes.min(2)
            } else {
                hexes
            };
            mine.iter().any(|c| c.pos.distance(u.pos) <= reach)
        })
        .count()
}

/// Our military units (not Scouts) at the would-be tomb city or beside it, and whether one
/// stands on the city tile.
pub(crate) fn tomb_garrison(view: &PlayerView, site: CityId) -> (usize, bool) {
    let Some(city) = view.cities.iter().find(|c| c.id == site) else {
        return (0, false);
    };
    let guards = view.units.iter().filter(|u| {
        u.owner == view.me
            && u.military
            && u.kind != UnitKind::Scout
            && u.pos.distance(city.pos) <= 1
    });
    let on_tile = view.units.iter().any(|u| {
        u.owner == view.me && u.military && u.kind != UnitKind::Scout && u.pos == city.pos
    });
    (guards.count(), on_tile)
}

/// The 2 VP take the team to its target when the tomb stands (with the lead it earns meanwhile).
fn tomb_decides_the_game(view: &PlayerView) -> bool {
    let points = goals::spec_for(goals::GoalRule::Tomb).vp;
    standing(view).is_some_and(|(vp, target)| vp < target && vp + points + TOMB_LEAD_VP >= target)
}

/// The tomb's 2 VP are worth 15 turns of the empire's output now: they take the team to its
/// target with time to hold it, or the game is late enough to bank them; and the work can still
/// finish before the game ends, with the hold after it when it is not late. A count already
/// started is worth finishing whenever it can still finish: the turns already given are sunk.
pub(crate) fn tomb_wanted(view: &PlayerView, open_end: bool) -> bool {
    let Some(t) = open_tomb(view) else {
        return false;
    };
    let Some((vp, target)) = standing(view) else {
        return false;
    };
    let left = i64::from(turns_left(t));
    let end = i64::from(last_turn(view, open_end));
    let finishes = i64::from(view.turn) + left <= end;
    if !finishes {
        return false;
    }
    if t.turns > 0 {
        return true;
    }
    let points = goals::spec_for(goals::GoalRule::Tomb).vp;
    let late = is_late(view, TOMB_LATE_PERCENT, open_end);
    let crosses = vp < target
        && vp + points + TOMB_LEAD_VP >= target
        && i64::from(view.turn) + left + i64::from(hold_turns(view)) <= end;
    crosses || late
}

/// The work may start now: wanted, no enemy army on the approaches, and the tomb city held by
/// its garrison (one unit is enough when the 2 VP decide the game, or very late).
fn tomb_ready(view: &PlayerView, open_end: bool) -> bool {
    let Some(t) = open_tomb(view) else {
        return false;
    };
    if t.working || t.blocker.is_some() || !tomb_wanted(view, open_end) {
        return false;
    }
    if enemies_near_cities(view, START_CLEAR_HEXES, None) > 0 {
        return false;
    }
    let Some(site) = t.site else {
        return false;
    };
    let (guards, on_tile) = tomb_garrison(view, site);
    let need = if tomb_decides_the_game(view) || is_late(view, 90, open_end) {
        1
    } else {
        TOMB_GARRISON
    };
    on_tile && guards >= need
}

/// The work should pause now: an enemy army has come near one of our cities (larger the more
/// of the count is done) and the tomb is more than `TOMB_FINISH_THROUGH` turn ends from
/// standing, or the game ends before it can.
fn tomb_pause(view: &PlayerView, open_end: bool) -> bool {
    let Some(t) = open_tomb(view) else {
        return false;
    };
    if !t.working || t.blocker.is_some() {
        return false;
    }
    let left = turns_left(t);
    if i64::from(view.turn) + i64::from(left) > i64::from(last_turn(view, open_end)) {
        return true;
    }
    // The count is kept across a pause, so the more of it is done the more it is worth
    // finishing on time: a larger army is needed to stop it.
    let tolerated = (t.turns / TOMB_PAUSE_TOLERANCE_TURNS) as usize;
    left > TOMB_FINISH_THROUGH && enemies_near_cities(view, THREAT_HEXES, None) > tolerated
}

/// This turn's tomb order, if any (gene `egypt_tomb`).
pub(crate) fn tomb_action(
    view: &PlayerView,
    open_end: bool,
    decisions: &crate::DecisionParams,
) -> Option<Action> {
    if decisions.egypt_tomb == 0 {
        return None;
    }
    if tomb_ready(view, open_end) {
        Some(Action::Tomb { build: true })
    } else if tomb_pause(view, open_end) {
        Some(Action::Tomb { build: false })
    } else {
        None
    }
}

/// The city the tomb would stand in while the work is wanted or on (gene `egypt_tomb`): the
/// first city to garrison.
pub(crate) fn tomb_guard_site(view: &PlayerView, open_end: bool) -> Option<CityId> {
    let t = open_tomb(view)?;
    (t.working || tomb_wanted(view, open_end))
        .then_some(t.site)
        .flatten()
}

/// Public: the city a rival Egypt's tomb would stand in if its work finished now (its capital,
/// else its largest known city), while the work is on.
pub(crate) fn rival_tomb_site(view: &PlayerView, of: legatum_core::PlayerId) -> Option<CityId> {
    let own = || view.cities.iter().filter(|c| c.owner == of);
    own()
        .find(|c| c.is_capital)
        .or_else(|| own().max_by_key(|c| (c.population, std::cmp::Reverse(c.id))))
        .map(|c| c.id)
}

#[cfg(test)]
#[path = "goal_tomb_tests.rs"]
mod tests;

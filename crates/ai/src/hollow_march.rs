//! Gene `hollow_march` (AI war plan W2.4): a war on a hollow leader is fought with the
//! garrisons it doesn't need. Playtest 8.9.17: the Swiss were at War with the designer's
//! Mongols (ten cities, four Horsemen) for 40 turns with five of their six units standing in
//! Swiss cities. At war with the top threat every city counts as threatened
//! (`threat_exposure`), so every garrison stayed, and the campaign forecast never counts a
//! unit standing in its own city (`could_strike`), so every Mongol city read "army too weak".
//! Public information only: the enemy's hollowness is the Strongest Army title's values, and
//! the threat to a city is the military units the seat can see near it.
use crate::scripted::Planner;
use legatum_core::{TeamId, view::CityView};

/// A spared city has no visible foreign military unit this close (hexes, reachable).
pub(crate) const HOLLOW_GUARD_RADIUS: i32 = 6;

/// Armed units wanted beyond one per city during such a war, at least (`army_war_extra`'s
/// shared default until 2026-11-06, now 1): two capture units and three shooters.
pub(crate) const HOLLOW_WAR_EXTRA: i32 = 5;

/// Gold a war purchase (`BUYS`) leaves in the treasury.
pub(crate) const WAR_BUY_RESERVE: i32 = 150;

/// Parts of gene `hollow_march` (a bit mask).
/// 1: a city with no foreign army in sight keeps no standing garrison, and its units count
/// toward the march force; 2: the capital too; 4: the war army quota is at least
/// `HOLLOW_WAR_EXTRA` beyond one unit per city; 8: Gold buys the march force.
pub(crate) const SPARE: i32 = 1;
pub(crate) const SPARE_CAPITAL: i32 = 2;
pub(crate) const QUOTA: i32 = 4;
pub(crate) const BUYS: i32 = 8;

impl Planner<'_> {
    /// The enemy our plan's war fights, when it reads as a hollow leader.
    pub(crate) fn hollow_war_target(&self) -> Option<TeamId> {
        if self.decisions.hollow_march == 0 || !self.hard_defense() {
            return None;
        }
        let v = self.view();
        let team = self.routed_plan()?.kind.war_target()?;
        (v.teams_at_war(v.team, team)
            && crate::threat::hollow_bonus_from(v, team, self.hollow_floor(team)) > 0)
            .then_some(team)
    }

    /// The city needs no standing garrison while we fight a hollow leader: no foreign military
    /// unit is visible within `HOLLOW_GUARD_RADIUS`, it is not the capital (unless part
    /// `SPARE_CAPITAL`) and it guards no Hall or Wonder. Door defense still answers any army that comes into view.
    pub(crate) fn garrison_spared(&self, city: &CityView) -> bool {
        if self.decisions.hollow_march & SPARE == 0
            || self.hollow_war_target().is_none()
            || (city.is_capital && self.decisions.hollow_march & SPARE_CAPITAL == 0)
            || self.wonder_guard_city(city)
            || self.hall_guard_city(city)
        {
            return false;
        }
        let v = self.view();
        !v.units.iter().any(|u| {
            u.military
                && !v.same_team(v.me, u.owner)
                // Barbarians never attack the Celts: no reason to keep a garrison home for one.
                && !(u.owner.is_barbarian() && v.barbarians_spare_me())
                && u.pos.distance(city.pos) <= HOLLOW_GUARD_RADIUS
                && self.ctx.reachable(u.pos, city.pos)
        })
    }
}

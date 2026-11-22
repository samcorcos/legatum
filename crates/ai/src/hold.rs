//! Victory holds, as every AI reads them. A team that reaches its VP target wins only after
//! holding it for `data::VP_HOLD_TURNS` consecutive turn ends; the countdown is public
//! (`TeamStanding::victory_hold`). Live or held faction goals can be taken away, so breaking a
//! hold means removing its "VP margin" (VP above the target) plus one. Everything
//! here reads only the seat's own `PlayerView` (pillar 6).

use legatum_core::view::{CityView, PlayerView, UnitView};
use legatum_core::{PlayerId, TeamId};

/// A team counting down to victory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hold {
    pub team: TeamId,
    /// Turn ends still needed: it wins at the end of the turn this reaches 1.
    pub turns_left: u32,
    pub turns_needed: u32,
    pub vp: u32,
    pub target: u32,
}

impl Hold {
    /// VP it can lose and still hold: it breaks on losing `margin() + 1`.
    pub fn margin(&self) -> u32 {
        self.vp.saturating_sub(self.target)
    }

    /// How far the countdown has run, 1..=1000 (permille of the window).
    pub fn elapsed_permille(&self) -> i32 {
        let needed = self.turns_needed.max(1);
        ((needed - self.turns_left.min(needed) + 1) * 1000 / needed).min(1000) as i32
    }
}

/// Every alive team holding its target, most urgent first: fewest turns left, then the
/// thinnest margin (cheapest to break), then team.
pub fn holds(view: &PlayerView) -> Vec<Hold> {
    if !view.standings.iter().any(|s| s.victory_hold.is_some()) {
        return Vec::new();
    }
    let mut out: Vec<Hold> = view
        .standings
        .iter()
        .filter(|s| view.players.iter().any(|p| p.team == s.team && p.alive))
        .filter_map(|s| {
            let h = s.victory_hold?;
            Some(Hold {
                team: s.team,
                turns_left: h.turns_needed.saturating_sub(h.turns_held),
                turns_needed: h.turns_needed,
                vp: s.vp,
                target: s.target,
            })
        })
        .collect();
    out.sort_by_key(|h| (h.turns_left, h.margin(), h.team));
    out
}

/// The most urgent rival hold.
pub fn rival_hold(view: &PlayerView) -> Option<Hold> {
    holds(view).into_iter().find(|h| h.team != view.team)
}

/// `team`'s hold, if it is a rival counting down.
pub fn rival_hold_of(view: &PlayerView, team: TeamId) -> Option<Hold> {
    (team != view.team)
        .then(|| holds(view).into_iter().find(|h| h.team == team))
        .flatten()
}

/// Our own team's hold.
pub fn own_hold(view: &PlayerView) -> Option<Hold> {
    holds(view).into_iter().find(|h| h.team == view.team)
}

/// A unit's army value (core `army::combat_value`: quality, promotions, supply and health).
pub fn army_value(u: &UnitView) -> i32 {
    if !u.military {
        return 0;
    }
    legatum_core::army::combat_value(u.strength, u.ranged, u.hp, u.level, u.unsupplied.is_some())
}

/// Priority for hitting `owner`'s unit or city while its team counts down to victory, 0 when
/// it isn't: more as the clock runs, more when the hit takes VP, and more when the holder's
/// margin is thin (losing `margin + 1` VP breaks it).
/// `takes_points` (VP the hit takes) is evaluated only for a holder.
pub fn denial_priority(
    view: &PlayerView,
    owner: PlayerId,
    takes_points: impl FnOnce() -> u32,
) -> i32 {
    let Some(h) = (!owner.is_barbarian())
        .then(|| rival_hold_of(view, view.team_of(owner)))
        .flatten()
    else {
        return 0;
    };
    let takes_points = takes_points();
    let breaks = takes_points > h.margin();
    let base = 100 + h.elapsed_permille() / 10;
    base * (1 + 2 * takes_points.min(3) as i32 + 4 * i32::from(breaks)) / (1 + h.margin() as i32)
}

impl crate::scripted::Planner<'_> {
    pub(crate) fn hold_city_points(&self, city: &CityView) -> u32 {
        if self.decisions.vp_hold_break == 0 {
            return 0;
        }
        self.capture_goal_swing(city).1 as u32
    }

    pub(crate) fn hold_denial_city(&self, city: &CityView) -> i32 {
        let v = self.view();
        denial_priority(v, city.owner, || self.hold_city_points(city))
    }

    /// `denial_priority` of attacking `pos`: the enemy city there, else its military units.
    pub(crate) fn hold_denial_at(&self, pos: legatum_core::Hex) -> i32 {
        let v = self.view();
        if let Some(c) = self.ctx.enemy_cities.iter().find(|c| c.pos == pos) {
            return self.hold_denial_city(c);
        }
        self.ctx
            .enemy_units
            .iter()
            .filter(|e| e.pos == pos && e.military)
            .map(|e| denial_priority(v, e.owner, || 0))
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "hold_tests.rs"]
mod tests;

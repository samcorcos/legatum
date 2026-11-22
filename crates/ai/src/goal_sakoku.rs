//! Reserved Japan Sakoku vow (`legatum_core::goals::GoalRule::Sakoku`) in the AI: when a scenario empire closes the
//! country and when it reopens it.
//!
//! Everything is scored in one currency, hundredths of a VP (2 VP = 200), from the fogged view:
//! what the vow earns against what it costs. The costs are the diffusion it stops getting, the
//! wars its closed state gives up and the danger it can't ally against. The numbers are
//! assumptions, not yet measured (no balance sims until the AI is competent).

use crate::goal_actions::{hold_turns, is_late, last_turn, standing};
use crate::scripted::Planner;
use legatum_core::{Faction, PlayerView, data};

/// Value of one VP, in the vow's currency.
const POINT: i32 = 100;
/// Extra value of a vow that takes the team to its VP target with time to hold it.
const CROSSES_TARGET: i32 = 300;
/// The vow's points are worth less as the game runs out: this many turns of holding earn all
/// of them.
const FULL_VALUE_TURNS: i32 = 40;
/// Reopening needs the costs to beat the vow's remaining value by this much, twice over while the
/// VP is held: a vow taken is kept through ordinary danger (the first smoke closed and reopened
/// within 6 to 9 turns at a margin of 150, so every close wasted its diffusion cost).
const REOPEN_MARGIN: i32 = 500;
/// A rival city of weaker army within this distance is a war the vow would give up.
const WAR_REACH: i32 = 10;
/// Value of one war a closed country can't start (about a city's worth of conquest).
const WAR_VALUE: i32 = 120;
/// Danger priced per visible enemy military unit near one of our cities, and per much-stronger
/// neighbour that could come: a closed country can't ally against either.
const NEAR_ENEMY: i32 = 20;
const STRONG_NEIGHBOUR: i32 = 60;
/// Science is priced one to one into the vow's currency.
const SCIENCE_VALUE: i32 = 1;
/// A vow not yet taken is worth its points only when they decide the race or the game is this far
/// along (percent of its length): the same window as the other rites (`goal_actions::rite_pays`).
const SAKOKU_LATE_PERCENT: u32 = 55;

/// What the vow costs India, per source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SakokuCosts {
    /// Science lost to missing tech diffusion over the rest of the countdown.
    pub diffusion: i32,
    /// Wars the vow gives up.
    pub war: i32,
    /// Danger at the gates that no Alliance can answer.
    pub danger: i32,
}

impl SakokuCosts {
    pub(crate) fn total(self) -> i32 {
        self.diffusion + self.war + self.danger
    }
}

/// Turn ends left until the vow's VP holds, counting from now (the whole countdown when open).
fn countdown_left(view: &PlayerView) -> i32 {
    view.actions.sakoku.as_ref().map_or(0, |s| {
        if s.closed {
            s.target.saturating_sub(s.turns) as i32
        } else {
            s.target as i32
        }
    })
}

/// What holding the vow earns from here: 2 VP for the turns left after the countdown, more when
/// it takes the team to the VP target with time to hold it, nothing when the game ends first.
pub(crate) fn sakoku_gain(view: &PlayerView, open_end: bool) -> i32 {
    let Some(s) = &view.actions.sakoku else {
        return 0;
    };
    let points = legatum_core::goals::spec(Faction::India).vp as i32;
    // Wide arithmetic: a huge turn limit must not overflow.
    let start = i64::from(view.turn) + i64::from(countdown_left(view));
    let held_for = i64::from(last_turn(view, open_end)) - start;
    if held_for <= 0 {
        return 0;
    }
    let held_for = held_for.min(i64::from(FULL_VALUE_TURNS)) as i32;
    let mut gain = points * POINT * held_for / FULL_VALUE_TURNS;
    let mut crosses = false;
    if let Some((vp, target)) = standing(view) {
        // A held vow is already in the standing: only a vow still to earn can take the team over.
        let base = if s.holding {
            vp.saturating_sub(points as u32)
        } else {
            vp
        };
        crosses = base < target
            && base + points as u32 >= target
            && start + i64::from(hold_turns(view)) <= i64::from(last_turn(view, open_end));
        if crosses {
            gain += CROSSES_TARGET;
        }
    }
    // Taken, the vow is kept to its end; not yet taken, it waits for the race or the late game.
    if s.closed || crosses || is_late(view, SAKOKU_LATE_PERCENT, open_end) {
        gain
    } else {
        0
    }
}

/// Enemy military units (at war, or barbarians) near one of our cities.
fn enemies_near_home(view: &PlayerView) -> i32 {
    let mine: Vec<_> = view.cities.iter().filter(|c| c.owner == view.me).collect();
    view.units
        .iter()
        .filter(|u| {
            u.military
                && u.owner != view.me
                && (u.owner.is_barbarian() || view.at_war(view.me, u.owner))
                && mine.iter().any(|c| c.pos.distance(u.pos) <= 5)
        })
        .count() as i32
}

/// The costs of staying closed over the rest of the countdown.
pub(crate) fn sakoku_costs(view: &PlayerView) -> SakokuCosts {
    let left = countdown_left(view).max(1);
    // Diffusion: the discount on the technologies we could research, as a share of our science.
    let available = &view.research.available;
    let discount_percent = if available.is_empty() {
        0
    } else {
        available
            .iter()
            .map(|t| (t.known_by * data::TECH_DIFFUSION_PER_EMPIRE).min(data::TECH_DIFFUSION_MAX))
            .sum::<i32>()
            / available.len() as i32
    };
    let diffusion =
        view.economy.science_per_turn.total.max(0) * left * discount_percent / 100 * SCIENCE_VALUE;
    // War: rivals within reach whose army is clearly weaker than ours, and wars already running
    // that we are winning (they go on, but nothing new can start).
    let ours = crate::diplomacy::strength(view, view.team).max(1);
    let mut war = 0;
    for p in view
        .players
        .iter()
        .filter(|p| p.alive && p.team != view.team)
    {
        let team = p.team;
        // Count each team once (its first living member).
        if view
            .players
            .iter()
            .find(|q| q.alive && q.team == team)
            .map(|q| q.id)
            != Some(p.id)
        {
            continue;
        }
        let theirs = crate::diplomacy::strength(view, team);
        let near = crate::diplomacy::nearest_known(view, team).is_some_and(|d| d <= WAR_REACH);
        if near && theirs * 100 < ours * 70 {
            war += WAR_VALUE;
        }
    }
    // Danger: enemy armies at the gates, and neighbours much stronger than we are.
    let mut danger = enemies_near_home(view) * NEAR_ENEMY;
    for p in view
        .players
        .iter()
        .filter(|p| p.alive && p.team != view.team)
    {
        if view
            .players
            .iter()
            .find(|q| q.alive && q.team == p.team)
            .map(|q| q.id)
            != Some(p.id)
        {
            continue;
        }
        let near = crate::diplomacy::nearest_known(view, p.team).is_some_and(|d| d <= WAR_REACH);
        if near && crate::diplomacy::strength(view, p.team) > ours * 3 / 2 {
            danger += STRONG_NEIGHBOUR;
        }
    }
    SakokuCosts {
        diffusion,
        war,
        danger,
    }
}

/// Whether India closes the country now: the vow earns more than it costs, nothing blocks it,
/// and the game isn't nearly over.
pub(crate) fn sakoku_close_now(view: &PlayerView, open_end: bool) -> bool {
    let Some(s) = &view.actions.sakoku else {
        return false;
    };
    if s.closed || s.blocker.is_some() || is_late(view, 92, open_end) {
        return false;
    }
    let gain = sakoku_gain(view, open_end);
    gain > 0 && gain > sakoku_costs(view).total()
}

/// Turns of countdown left within which a closed country's vow is worth a Peace with anyone.
const VOW_STAKE_TURNS: i32 = 15;

/// Whether a closed country has a vow worth protecting from conquest: it holds, or scores within
/// `VOW_STAKE_TURNS`, and the game has time left for its points. A closed country starts no war
/// of its own, so a Peace costs it nothing and can end a costly war
/// (8.9.6 Swiss: India refused Athens's Peace six turns before the vow scored).
pub(crate) fn vow_at_stake(view: &PlayerView) -> bool {
    let Some(s) = &view.actions.sakoku else {
        return false;
    };
    s.closed
        && (s.holding || countdown_left(view) <= VOW_STAKE_TURNS)
        && !is_late(view, 97, crate::DecisionParams::default().goal_open_end > 0)
}

/// Whether the vow is what takes this team to (or within one point of) its VP target: closed,
/// and the VP without the vow's points is short while with them it is there or one short (the
/// closing point can come from elsewhere: a capture that completes a count). The countdown then
/// decides the race, and every rival's denial war aims at it (8.9.6 Swiss: 7 VP, 9 with the
/// vow, target 10).
pub(crate) fn vow_closes_the_race(view: &PlayerView) -> bool {
    let Some(s) = &view.actions.sakoku else {
        return false;
    };
    if !s.closed {
        return false;
    }
    let points = legatum_core::goals::spec(Faction::India).vp;
    standing(view).is_some_and(|(vp, target)| {
        let base = if s.holding {
            vp.saturating_sub(points)
        } else {
            vp
        };
        base < target && base + points + 1 >= target
    })
}

/// Whether India reopens the country now: the costs of staying closed beat what the vow still
/// earns by a clear margin (a winning war it blocks, an army it can't ally against).
pub(crate) fn sakoku_reopen_now(view: &PlayerView, open_end: bool) -> bool {
    let Some(s) = &view.actions.sakoku else {
        return false;
    };
    if !s.closed {
        return false;
    }
    let keep = sakoku_gain(view, open_end);
    let margin = if s.holding {
        2 * REOPEN_MARGIN
    } else {
        REOPEN_MARGIN
    };
    sakoku_costs(view).total() > keep + margin
}

impl Planner<'_> {
    /// Gene `vow_hold`: our closed country's vow is about to score or holds, so every war should
    /// end.
    pub(crate) fn vow_at_stake(&self) -> bool {
        self.decisions.vow_hold > 0
            && self.faction == Faction::India
            && self.difficulty == legatum_core::AiDifficulty::Hard
            && vow_at_stake(self.view())
    }

    /// Gene `vow_hold`: our closed country's vow is the VP that reaches (or nearly reaches) the target.
    pub(crate) fn vow_race_closed(&self) -> bool {
        self.decisions.vow_hold > 0
            && self.faction == Faction::India
            && self.difficulty == legatum_core::AiDifficulty::Hard
            && vow_closes_the_race(self.view())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::orders::{Action, Order};
    use legatum_core::rules::actions::SakokuView;
    use legatum_core::view::TeamStanding;
    use legatum_core::{PlayerId, TeamId, UnitKind};

    const TARGET: u32 = 30;

    fn sakoku(closed: bool, turns: u32) -> SakokuView {
        SakokuView {
            closed,
            turns,
            target: TARGET,
            holding: closed && turns >= TARGET,
            blocker: None,
        }
    }

    /// An open India at `turn` with `vp` of `target`, one city, and a distant quiet rival.
    fn india(turn: u32, vp: u32, target: u32) -> PlayerView {
        let mut v = ViewBuilder::new(40, 24)
            .faction(Faction::India)
            .city(1, ME, off(3, 3), 5)
            .city(3, ENEMY, off(35, 18), 5)
            .build();
        v.turn = turn;
        v.turn_limit = None;
        v.standings = vec![TeamStanding {
            team: TeamId(0),
            members: vec![ME],
            vp,
            target,
            victory_hold: None,
        }]
        .into();
        v.actions.sakoku = Some(sakoku(false, 0));
        v
    }

    fn vows(v: &PlayerView) -> (bool, bool) {
        let o = crate::goal_actions::goal_actions(v);
        (
            o.contains(&Order::Action {
                action: Action::Sakoku { close: true },
            }),
            o.contains(&Order::Action {
                action: Action::Sakoku { close: false },
            }),
        )
    }

    #[test]
    fn india_vows_when_the_two_vp_take_it_to_the_target_or_the_game_is_late() {
        // Crossing the target with time to hold it: the vow is worth its quiet costs.
        assert_eq!(vows(&india(140, 8, 10)), (true, false));
        // Far from the target and early: the vow waits.
        assert_eq!(vows(&india(40, 3, 10)), (false, false));
        assert_eq!(vows(&india(120, 5, 10)), (false, false));
        // Late (55% of the game) the points are banked even short of the target.
        assert_eq!(vows(&india(170, 5, 10)), (true, false));
        // So close to the end that the countdown leaves nothing to hold: never.
        assert_eq!(vows(&india(280, 8, 10)), (false, false));
    }

    #[test]
    fn a_closed_country_protects_a_vow_that_scores_soon_or_holds() {
        let mut v = india(200, 5, 10);
        v.actions.sakoku = Some(sakoku(true, 25));
        assert!(vow_at_stake(&v), "5 turns from scoring");
        v.actions.sakoku = Some(sakoku(true, 5));
        assert!(!vow_at_stake(&v), "25 turns from scoring");
        v.actions.sakoku = Some(sakoku(true, TARGET));
        assert!(vow_at_stake(&v), "holding");
        assert!(!vow_at_stake(&india(200, 5, 10)), "open");
    }

    #[test]
    fn the_vow_closes_the_race_only_when_its_points_reach_the_target() {
        let mut v = india(200, 9, 10);
        v.actions.sakoku = Some(sakoku(true, 10));
        assert!(vow_closes_the_race(&v), "9 of 10: the vow's 2 reach it");
        // Held: the standing already counts the vow; without it the team is still short.
        let mut held = india(240, 11, 10);
        held.actions.sakoku = Some(sakoku(true, TARGET));
        assert!(vow_closes_the_race(&held));
        // 7 of 10: the vow's 2 leave it one short, still the point the rivals attack.
        let mut one_short = india(200, 7, 10);
        one_short.actions.sakoku = Some(sakoku(true, 10));
        assert!(vow_closes_the_race(&one_short));
        // Far short of the target even with the vow, or not closed: not the closing VP.
        let mut far = india(200, 6, 10);
        far.actions.sakoku = Some(sakoku(true, 10));
        assert!(!vow_closes_the_race(&far));
        assert!(!vow_closes_the_race(&india(200, 9, 10)));
        // Already there without the vow: it is not what decides the race.
        let mut won = india(200, 12, 10);
        won.actions.sakoku = Some(sakoku(true, 10));
        assert!(!vow_closes_the_race(&won));
    }

    #[test]
    fn a_vow_that_costs_more_than_it_earns_is_not_taken() {
        // Late and short of the target, the vow earns 2 VP and nothing more: two much weaker
        // neighbours in reach (wars it would give up, 120 each) outweigh it.
        let quiet = india(170, 5, 10);
        assert_eq!(vows(&quiet), (true, false));
        let mut busy = quiet.clone();
        for (id, owner, pos) in [(4u32, ENEMY, off(10, 3)), (5, PlayerId(3), off(3, 10))] {
            busy = ViewBuilder { view: busy }.city(id, owner, pos, 4).build();
        }
        assert_eq!(sakoku_costs(&busy).war, 2 * WAR_VALUE);
        assert_eq!(vows(&busy), (false, false));
        // An enemy army at the gates, and the vow is refused.
        let mut v = india(140, 8, 10);
        for i in 0..40u32 {
            v = ViewBuilder { view: v }
                .unit(100 + i, ENEMY, UnitKind::Soldier, off(5, 5))
                .build();
        }
        assert!(sakoku_costs(&v).danger >= 40 * NEAR_ENEMY);
        assert_eq!(vows(&v), (false, false));
    }

    #[test]
    fn the_vow_waits_a_turn_when_it_would_ride_with_an_alliance_or_a_declaration() {
        let v = india(140, 8, 10);
        let close = Order::Action {
            action: Action::Sakoku { close: true },
        };
        let mut quiet = vec![Order::SkipUnit {
            unit: legatum_core::UnitId(1),
        }];
        crate::goal_actions::with_goal_actions(
            &v,
            &mut quiet,
            true,
            &crate::DecisionParams::default(),
        );
        assert!(quiet.contains(&close), "{quiet:?}");
        for rider in [
            Order::ProposeTreaty {
                to: TeamId(2),
                kind: legatum_core::diplomacy::TreatyKind::Alliance,
            },
            Order::DeclareWar {
                against: TeamId(2),
                breach: false,
            },
        ] {
            let mut orders = vec![rider.clone()];
            crate::goal_actions::with_goal_actions(
                &v,
                &mut orders,
                true,
                &crate::DecisionParams::default(),
            );
            assert_eq!(orders, vec![rider], "the vow waits");
        }
        // A Peace proposal does not hold it back.
        let mut peace = vec![Order::ProposeTreaty {
            to: TeamId(2),
            kind: legatum_core::diplomacy::TreatyKind::Peace,
        }];
        crate::goal_actions::with_goal_actions(
            &v,
            &mut peace,
            true,
            &crate::DecisionParams::default(),
        );
        assert!(peace.contains(&close));
    }

    #[test]
    fn a_closed_india_keeps_its_vow_until_the_costs_clearly_win() {
        let mut v = india(140, 8, 10);
        v.actions.sakoku = Some(sakoku(true, 5));
        assert_eq!(
            vows(&v),
            (false, false),
            "a quiet closed India stays closed"
        );
        // Held and counting down to a win: stays closed.
        v.actions.sakoku = Some(sakoku(true, 20));
        assert_eq!(vows(&v), (false, false));
        // A besieged closed India with 80 enemy units at its cities reopens.
        let mut v = india(140, 8, 10);
        v.actions.sakoku = Some(sakoku(true, 5));
        for i in 0..80u32 {
            v = ViewBuilder { view: v }
                .unit(100 + i, ENEMY, UnitKind::Soldier, off(5, 5))
                .build();
        }
        assert_eq!(vows(&v), (false, true));
    }

    #[test]
    fn a_vow_never_dithers_open_and_closed_in_one_view() {
        for turn in [60, 120, 140, 170, 220] {
            for vp in [3, 7, 9] {
                let (closes, reopens) = vows(&india(turn, vp, 10));
                assert!(!(closes && reopens));
                let mut v = india(turn, vp, 10);
                v.actions.sakoku = Some(sakoku(true, 3));
                let (closes, _) = vows(&v);
                assert!(!closes, "already closed");
            }
        }
    }
}

//! Public derived warnings. Membership is evidence from ordinary treaties, never a new polity.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoalitionView {
    pub target: TeamId,
    pub vp: u32,
    pub victory_target: u32,
    pub progress: u32,
    pub runner_up_progress: u32,
    pub reason: String,
    pub members: Vec<TeamId>,
    pub at_war: Vec<TeamId>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplorationView {
    pub native_land: u32,
    pub borrowed_current: u32,
}
pub fn exploration(state: &GameState, me: PlayerId) -> ExplorationView {
    let native = crate::visibility::native_visible_tiles(state, state.team_of(me));
    let visible =
        crate::visibility::visible_tiles_from_native(state, state.team_of(me), native.clone());
    ExplorationView {
        native_land: state
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                t.terrain.is_land() && (state.player(me).explored.get(*i) || native.get(*i))
            })
            .count() as u32,
        borrowed_current: visible.iter_ones().filter(|i| !native.get(*i)).count() as u32,
    }
}
pub fn target(
    standings: impl Iterator<Item = (TeamId, u32, u32)>,
) -> Option<(TeamId, u32, u32, u32, u32)> {
    let (lead, next) = select(standings.map(|(t, v, g)| (t, v, g, 0, None)))?;
    Some((
        lead.team,
        lead.vp,
        lead.goal,
        lead.progress(),
        next.progress(),
    ))
}

#[derive(Clone, Copy)]
struct Candidate {
    team: TeamId,
    vp: u32,
    goal: u32,
    city_pressure: u32,
    /// Turns left in the team's VP hold (`GameState::vp_holds`), if it is holding its target.
    hold_left: Option<u32>,
}

/// Added to a holding team's public coalition pressure (`pressure_view`), so a team counting
/// down to victory outranks every team that isn't, whatever their progress or cities.
pub const HOLD_PRESSURE: u32 = 1000;

/// Turns left before a hold wins: `turns_needed - turns_held`.
fn hold_left(hold: Option<crate::view::VictoryHold>) -> Option<u32> {
    hold.map(|h| h.turns_needed.saturating_sub(h.turns_held))
}
impl Candidate {
    fn progress(self) -> u32 {
        100 * self.vp / self.goal
    }
    fn pressure(self) -> u32 {
        self.progress() + self.city_pressure
    }
}

/// The coalition target and the runner-up. A team holding its VP target (counting down to
/// victory) outranks every team that isn't, even on equal progress; of several holders the one
/// with the fewest turns left, then the higher pressure, is the target, and holders tied on
/// both have none. Without a hold: the strictly highest pressure at 80, or at 60 with a
/// 15-point lead; a tie has no target.
fn select(
    standings: impl Iterator<Item = (TeamId, u32, u32, u32, Option<u32>)>,
) -> Option<(Candidate, Candidate)> {
    let mut ranks: Vec<_> = standings
        .filter(|(_, _, goal, _, _)| *goal > 0)
        .map(|(team, vp, goal, city_pressure, hold_left)| Candidate {
            team,
            vp,
            goal,
            city_pressure,
            hold_left,
        })
        .collect();
    ranks.sort_by_key(|c| {
        (
            c.hold_left.is_none(),
            c.hold_left.unwrap_or(0),
            std::cmp::Reverse(c.pressure()),
            c.team,
        )
    });
    let lead = *ranks.first()?;
    // A tie has no target, and nobody is a threat with no surviving opponent.
    let next = *ranks.get(1)?;
    if let Some(left) = lead.hold_left {
        let tied = next.hold_left == Some(left) && next.pressure() == lead.pressure();
        return (!tied).then_some((lead, next));
    }
    let (a, b) = (lead.pressure(), next.pressure());
    (a > b && (a >= 80 || a >= 60 && a - b >= 15)).then_some((lead, next))
}

fn city_pressure(enabled: bool, cities: u32, members: u32) -> u32 {
    if enabled {
        // Normalize by permanent-team size, including eliminated members. Otherwise a
        // teammate's elimination could itself increase the team's expansion pressure.
        10 * cities.saturating_sub(2 * members) / members.max(1)
    } else {
        0
    }
}

fn state_target(state: &GameState) -> Option<(Candidate, Candidate)> {
    select(
        state
            .teams()
            .into_iter()
            .filter(|t| state.team_alive(*t))
            .map(|t| {
                (
                    t,
                    crate::scoring::team_vp(state, t),
                    crate::scoring::team_target(state, t),
                    city_pressure(
                        state.setup.rule_experiments.expansion_coalitions,
                        state
                            .cities
                            .values()
                            .filter(|c| state.team_of(c.owner) == t)
                            .count() as u32,
                        state.players.iter().filter(|p| p.team == t).count() as u32,
                    ),
                    state
                        .vp_holds
                        .get(&t)
                        .map(|held| crate::data::VP_HOLD_TURNS.saturating_sub(*held)),
                )
            }),
    )
}

pub fn target_state(state: &GameState) -> Option<TeamId> {
    state_target(state).map(|(lead, _)| lead.team)
}

fn view_city_pressure(view: &crate::PlayerView, team: TeamId) -> u32 {
    city_pressure(
        view.rule_experiments.expansion_coalitions,
        view.players
            .iter()
            .filter(|p| p.team == team)
            .map(|p| p.cities)
            .sum(),
        view.players.iter().filter(|p| p.team == team).count() as u32,
    )
}

/// Public coalition score, also used for coalition exit conditions. Ordinary peace and
/// victory decisions still use actual VP. No fogged city positions or territory are read.
/// A team holding its VP target scores `HOLD_PRESSURE` more, less one per turn it has left.
pub fn pressure_view(view: &crate::PlayerView, team: TeamId) -> u32 {
    view.standings
        .iter()
        .find(|s| s.team == team)
        .map_or(0, |s| {
            100 * s.vp / s.target.max(1)
                + hold_left(s.victory_hold).map_or(0, |left| HOLD_PRESSURE.saturating_sub(left))
        })
        + view_city_pressure(view, team)
}

pub fn target_view(view: &crate::PlayerView) -> Option<TeamId> {
    select(
        view.standings
            .iter()
            .filter(|s| view.players.iter().any(|p| p.team == s.team && p.alive))
            .map(|s| {
                (
                    s.team,
                    s.vp,
                    s.target,
                    view_city_pressure(view, s.team),
                    hold_left(s.victory_hold),
                )
            }),
    )
    .map(|(lead, _)| lead.team)
}
pub fn from_state(state: &GameState) -> Option<CoalitionView> {
    let d = state.diplomacy.as_ref()?;
    let (lead, runner_up) = state_target(state)?;
    let (team, vp, goal, progress, next) = (
        lead.team,
        lead.vp,
        lead.goal,
        lead.progress(),
        runner_up.progress(),
    );
    let mut members = BTreeSet::new();
    for r in &d.relations {
        if shares_sight(r)
            && r.a != team
            && r.b != team
            && state.team_alive(r.a)
            && state.team_alive(r.b)
            && r.security
                .as_ref()
                .is_some_and(|t| t.coalition_target == Some(team))
        {
            members.extend([r.a, r.b]);
        }
    }
    let at_war = members
        .iter()
        .copied()
        .filter(|m| {
            d.relation(*m, team)
                .is_some_and(|r| r.state == RelationState::War)
        })
        .collect();
    Some(CoalitionView {
        target: team,
        vp,
        victory_target: goal,
        progress,
        runner_up_progress: next,
        reason: if let Some(left) = lead.hold_left {
            format!(
                "{vp} of {goal} VP ({progress}%): holding the victory target, {left} turns from winning"
            )
        } else if state.setup.rule_experiments.expansion_coalitions {
            format!(
                "{vp} of {goal} VP ({progress}%); coalition pressure {} = {progress} VP progress + {} city pressure; next team {}. City pressure: 10 per city per team member beyond two",
                lead.pressure(),
                lead.city_pressure,
                runner_up.pressure()
            )
        } else if progress >= 80 {
            format!(
                "{vp} of {goal} VP ({progress}%): at least 80% of victory and {} points ahead of the next team",
                progress - next
            )
        } else {
            format!(
                "{vp} of {goal} VP ({progress}%): {} points ahead of the next team",
                progress - next
            )
        },
        members: members.into_iter().collect(),
        at_war,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expansion_pressure_can_target_a_wide_empire_below_the_vp_threshold() {
        let bonus = city_pressure(true, 5, 1);
        assert_eq!(bonus, 30);
        let ranks = [(TeamId(0), 4, 10), (TeamId(1), 5, 10)];
        assert!(target(ranks.into_iter()).is_none());
        let (lead, next) = select(
            ranks
                .into_iter()
                .map(|(t, v, g)| (t, v, g, if t == TeamId(0) { bonus } else { 0 }, None)),
        )
        .unwrap();
        assert_eq!(lead.team, TeamId(0));
        assert_eq!(
            (lead.progress(), lead.pressure(), next.pressure()),
            (40, 70, 50)
        );
        assert_eq!(city_pressure(true, 10, 2), bonus);
        assert_eq!(city_pressure(true, 4, 2), 0);
        assert_eq!(city_pressure(false, 20, 1), 0);
        // Equal pressure must not single out a seat, even when both are near victory.
        assert!(
            select([(TeamId(0), 5, 10, 30, None), (TeamId(1), 8, 10, 0, None)].into_iter())
                .is_none()
        );
        assert!(select([(TeamId(0), 5, 10, 30, None)].into_iter()).is_none());
    }

    #[test]
    fn expansion_target_matches_every_fogged_view_and_save_load() {
        use crate::{
            Faction, Hex, PlayerId, Terrain, UnitKind,
            game::{blank_game, spawn_unit},
            rules::cities::found_city,
            state::{GameSetup, Map},
        };
        let mut setup = GameSetup::all_ai(12);
        setup.players.truncate(2);
        for p in &mut setup.players {
            p.faction = Faction::Byzantium;
        }
        setup.rule_experiments.expansion_coalitions = true;
        let mut state = blank_game(&setup, Map::new(40, 30, Terrain::Grassland));
        for (owner, positions) in [
            (
                PlayerId(0),
                vec![
                    (3, 3),
                    (10, 3),
                    (17, 3),
                    (24, 3),
                    (31, 3),
                    (3, 10),
                    (10, 10),
                    (17, 10),
                    (24, 10),
                    (31, 10),
                ],
            ),
            (PlayerId(1), vec![(3, 24)]),
        ] {
            for (x, y) in positions {
                let unit = spawn_unit(&mut state, owner, UnitKind::Settler, Hex::from_offset(x, y));
                found_city(&mut state, unit, &mut Vec::new()).unwrap();
            }
        }
        assert_eq!(target_state(&state), Some(TeamId(0)));
        for p in [PlayerId(0), PlayerId(1)] {
            let view = crate::view::player_view(&state, p);
            assert_eq!(target_view(&view), target_state(&state));
            assert_eq!(view.diplomacy.coalition.clone().unwrap().target, TeamId(0));
        }
        let encoded = serde_json::to_string(&state).unwrap();
        let decoded: GameState = serde_json::from_str(&encoded).unwrap();
        assert_eq!(target_state(&decoded), target_state(&state));
        state.setup.rule_experiments.expansion_coalitions = false;
        assert_eq!(target_state(&state), None);
    }
    #[test]
    fn thresholds_normalize_permanent_team_targets_and_ties_have_no_target() {
        assert_eq!(
            target([(TeamId(0), 12, 20), (TeamId(1), 6, 10)].into_iter()),
            None
        );
        assert_eq!(
            target([(TeamId(0), 12, 20), (TeamId(1), 4, 10)].into_iter())
                .unwrap()
                .0,
            TeamId(0)
        );
        // A tie at the top is a race, not a leader: no coalition target, whatever the ids.
        assert_eq!(
            target([(TeamId(2), 8, 10), (TeamId(1), 8, 10)].into_iter()),
            None
        );
        assert_eq!(
            target([(TeamId(0), 9, 10), (TeamId(1), 9, 10), (TeamId(2), 5, 10)].into_iter()),
            None
        );
        // Strictly ahead near victory is enough, even by a single VP.
        assert_eq!(
            target([(TeamId(2), 9, 10), (TeamId(1), 8, 10)].into_iter())
                .unwrap()
                .0,
            TeamId(2)
        );
        assert_eq!(
            target([(TeamId(0), 5, 10), (TeamId(1), 0, 10)].into_iter()),
            None
        );
        assert!(target([(TeamId(0), 6, 10), (TeamId(1), 4, 10)].into_iter()).is_some());
        assert_eq!(target([(TeamId(0), 8, 10)].into_iter()), None);
    }

    #[test]
    fn a_holding_team_is_the_target_even_on_a_progress_tie() {
        let pick = |rows: &[(u8, u32, Option<u32>)]| {
            select(
                rows.iter()
                    .map(|(t, vp, left)| (TeamId(*t), *vp, 10, 0, *left)),
            )
            .map(|(lead, _)| lead.team)
        };
        // Without holds a tie at the top has no target.
        assert_eq!(pick(&[(0, 10, None), (1, 10, None)]), None);
        // The holder outranks an equal team that isn't holding (it fell below and came back,
        // or reached the target a turn later), and any non-holder with more pressure.
        assert_eq!(pick(&[(0, 10, None), (1, 10, Some(7))]), Some(TeamId(1)));
        assert_eq!(pick(&[(0, 12, None), (1, 10, Some(9))]), Some(TeamId(1)));
        // Two holders: the one closest to winning; equal clocks fall to higher progress.
        assert_eq!(pick(&[(0, 11, Some(6)), (1, 10, Some(3))]), Some(TeamId(1)));
        assert_eq!(pick(&[(0, 11, Some(3)), (1, 10, Some(3))]), Some(TeamId(0)));
        // Holders tied on both have no single target.
        assert_eq!(pick(&[(0, 10, Some(3)), (1, 10, Some(3))]), None);
        // A holder is the target even when every other team is far behind.
        assert_eq!(pick(&[(0, 10, Some(9)), (1, 0, None)]), Some(TeamId(0)));
    }

    #[test]
    fn a_holding_team_is_the_target_in_state_and_every_view() {
        use crate::{
            Faction, PlayerId, Terrain,
            game::blank_game,
            state::{GameSetup, Map},
        };
        let mut setup = GameSetup::all_ai(13);
        setup.rule_experiments.vp_target = Some(2);
        setup.players.truncate(3);
        for p in &mut setup.players {
            p.faction = Faction::Byzantium;
        }
        let mut state = blank_game(&setup, Map::new(30, 20, Terrain::Grassland));
        // Teams 0 and 1 tie at the 2 VP test target: only team 1 is holding.
        for p in [PlayerId(0), PlayerId(1)] {
            let player = state.player_mut(p);
            player.faction_goal_done = true;
        }
        let vp = crate::scoring::team_vp(&state, TeamId(0));
        assert_eq!(vp, crate::scoring::team_vp(&state, TeamId(1)));
        assert_eq!(
            target_state(&state),
            None,
            "a tie without a hold has no target"
        );
        state.vp_holds.insert(TeamId(1), 2);
        assert_eq!(target_state(&state), Some(TeamId(1)));
        for p in [PlayerId(0), PlayerId(1), PlayerId(2)] {
            let view = crate::view::player_view(&state, p);
            assert_eq!(target_view(&view), Some(TeamId(1)));
            assert!(pressure_view(&view, TeamId(1)) > pressure_view(&view, TeamId(0)) + 900);
        }
    }
}

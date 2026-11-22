//! Victory points: World Age cards, held rival capitals, Great Works and faction goals.
//! Metric and VP math live here; `update` (run at the end of each turn) applies changes.

use crate::economy;
use crate::events::Event;
use crate::goals::{self, GoalKind};
use crate::state::GameState;
use crate::types::*;
use std::collections::BTreeMap;

/// An empire's Power: the average of its total yields (Food + Production + Gold + Science) over
/// the trailing window. Public. Zero before the first turn is recorded.
pub fn power(state: &GameState, p: PlayerId) -> i32 {
    let h = &state.player(p).power_history;
    let window = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    let recent = &h[h.len().saturating_sub(window)..];
    if recent.is_empty() {
        return 0;
    }
    (recent.iter().map(|v| i64::from(*v)).sum::<i64>() / recent.len() as i64) as i32
}

/// The midpoint of living empires' Power, for descriptive simulation telemetry.
pub fn median_power(state: &GameState) -> i32 {
    let mut powers: Vec<i32> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| power(state, p.id))
        .collect();
    powers.sort_unstable();
    match powers.len() {
        0 => 0,
        n if n % 2 == 1 => powers[n / 2],
        n => (powers[n / 2 - 1] + powers[n / 2]) / 2,
    }
}

/// The mean Power of the living empires. The leader is in it, so it rises with a runaway's own
/// Power and that runaway's lead over it stays bounded.
pub fn mean_power(state: &GameState) -> i32 {
    let powers: Vec<i64> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| i64::from(power(state, p.id)))
        .collect();
    if powers.is_empty() {
        return 0;
    }
    (powers.iter().sum::<i64>() / powers.len() as i64) as i32
}

/// Power over the mean of the living empires, in percent, negative below it.
pub fn lead_percent(state: &GameState, p: PlayerId) -> i32 {
    lead_over_mean(state, p)
}

/// Lead over the mean Power of the living empires in percent: (Power - mean) / mean.
pub fn lead_over_mean(state: &GameState, p: PlayerId) -> i32 {
    if !state.player(p).alive {
        return 0;
    }
    let mean = i64::from(mean_power(state));
    if mean <= 0 {
        return 0;
    }
    ((i64::from(power(state, p)) - mean) * 100 / mean).clamp(-100, 10_000) as i32
}

/// Decadence: the percent cut from every income of `p`, one rate for Food (as slower growth),
/// Production, Gold and Science alike. Zero at or below the mean empire; above it in proportion to
/// the lead (`RuleExperiments::decadence_percent`). Read from the value refreshed at the end of each turn.
pub fn decadence_percent(state: &GameState, p: PlayerId) -> i32 {
    if state.setup.rule_experiments.decadence_slope() == 0 {
        0
    } else {
        state.player(p).decadence_percent
    }
}

/// What decadence would be today, from the empire's current lead (what `refresh_decadence` stores).
/// Nothing until the empire has a full window of Power behind it (50 turns at Standard): a lead
/// measured over fewer turns is an opening, not a verdict, and the opening's pace is tuned.
pub fn decadence_percent_now(state: &GameState, p: PlayerId) -> i32 {
    if state.setup.rule_experiments.decadence_slope() == 0 {
        return 0;
    }
    let window = usize::from(crate::experiments::POWER_WINDOW_TURNS);
    if state.player(p).power_history.len() < window {
        return 0;
    }
    state
        .setup
        .rule_experiments
        .decadence_percent(lead_percent(state, p))
}

/// Stores each living empire's decadence for the coming turn. Called once, right after the turn's
/// Power is recorded, so the rate depends only on the trailing window and no empire's yields
/// depend on turn order.
pub fn refresh_decadence(state: &mut GameState) {
    let rates: Vec<(PlayerId, i32)> = state
        .players
        .iter()
        .map(|p| {
            let rate = if p.alive {
                decadence_percent_now(state, p.id)
            } else {
                0
            };
            (p.id, rate)
        })
        .collect();
    for (p, rate) in rates {
        state.player_mut(p).decadence_percent = rate;
    }
}

/// An empire's public economy value: Gold plus Production across its cities.
pub fn economy_value(state: &GameState, p: PlayerId) -> i32 {
    state
        .cities_of(p)
        .map(|c| {
            let y = economy::city_totals(state, c);
            y.gold.saturating_add(y.production)
        })
        .fold(0_i32, i32::saturating_add)
}

/// An empire's public technological progress: furthest era entered (times 1000), ties broken by
/// technologies known.
pub fn advancement(state: &GameState, p: PlayerId) -> i32 {
    i32::from(crate::objectives::era_of(state, p)) * 1000 + state.player(p).techs.len() as i32
}

/// An empire's army strength: combat value of its units and aircraft in cities, including era
/// quality, promotions, supply and remaining health. Public.
pub fn army_strength(state: &GameState, p: PlayerId) -> i32 {
    state
        .units_of(p)
        .map(|u| crate::army::unit_value(state, u))
        .chain(state.aircraft.iter().filter_map(|a| {
            state
                .cities
                .get(&a.city)
                .filter(|city| city.owner == p)
                .map(|_| crate::army::aircraft_value(state, a))
        }))
        .fold(0_i32, i32::saturating_add)
}

/// A player's victory points, by source.
pub fn vp(state: &GameState, p: PlayerId) -> Vec<(VpSource, u32)> {
    let mut out = Vec::new();
    if let Some(building) = state.player(p).great_work_completed {
        for tile in &state.map.tiles {
            if tile
                .landmark
                .is_some_and(|s| s.of == p && s.building == building)
                && tile.owner == Some(p)
            {
                out.push((VpSource::GreatWork { of: p }, 2));
            }
        }
    }
    for (era, record) in &state.player(p).eras {
        if record.scored.is_some() {
            out.push((VpSource::EraObjective { era: *era }, 1));
        }
    }
    for (capital, _) in capitals_held(state, p) {
        out.push((VpSource::CapitalTaken { capital }, CAPITAL_VP));
    }
    for (capital, _) in &state.player(p).capitals_sacked {
        out.push((VpSource::CapitalSacked { capital: *capital }, CAPITAL_VP));
    }
    for (of, holder) in goals::holders(state) {
        if holder == p {
            out.push((
                VpSource::FactionGoal { of },
                goals::spec(state.player(of).faction).vp,
            ));
        }
    }
    for origin in &state.players {
        if let Some(building) = origin.great_work_completed
            && let Some((_, holder)) = crate::landmarks::find(state, origin.id, building)
            && holder == p
            && origin.id != p
        {
            out.push((VpSource::GreatWork { of: origin.id }, 2));
        }
    }
    out
}

/// VP for each rival original capital held (Sam, 2026-11-22, via the Fable review): held like a
/// world wonder, not kept for good.
pub const CAPITAL_VP: u32 = 1;

/// The rival original capitals `p` holds now, by city id, with their names. Each is worth
/// `CAPITAL_VP` while `p` holds it: an original capital founded by an empire of another team.
/// Retaken by anyone or razed, the point is gone the same turn.
pub fn capitals_held(state: &GameState, p: PlayerId) -> Vec<(CityId, String)> {
    let team = state.team_of(p);
    state
        .cities
        .values()
        .filter(|c| c.owner == p && c.original_capital && state.team_of(c.founder) != team)
        .map(|c| (c.id, c.name.clone()))
        .collect()
}

/// Who holds each rival original capital that scores now: city id to (holder, city name).
/// Barbarians never score.
pub(crate) fn capital_holders(state: &GameState) -> BTreeMap<CityId, (PlayerId, String)> {
    state
        .cities
        .values()
        .filter(|c| {
            c.original_capital
                && !c.owner.is_barbarian()
                && state.team_of(c.founder) != state.team_of(c.owner)
        })
        .map(|c| (c.id, (c.owner, c.name.clone())))
        .collect()
}

/// Report every capital point that changed hands since `before` (the start of the turn), the
/// way a world wonder's change of hands is reported: the old holder's loss, then the new
/// holder's gain.
fn check_capitals(
    state: &GameState,
    before: &BTreeMap<CityId, (PlayerId, String)>,
    events: &mut Vec<Event>,
) {
    let now = capital_holders(state);
    for (capital, (old, name)) in before {
        if now.get(capital).map(|h| h.0) != Some(*old) {
            events.push(Event::CapitalLost {
                player: *old,
                capital: *capital,
                name: name.clone(),
            });
        }
    }
    for (capital, (new, name)) in &now {
        if before.get(capital).map(|h| h.0) != Some(*new) {
            events.push(Event::CapitalTaken {
                captor: *new,
                capital: *capital,
                name: name.clone(),
            });
        }
    }
}

pub fn vp_total(state: &GameState, p: PlayerId) -> u32 {
    vp(state, p).iter().map(|(_, v)| v).sum()
}

pub fn team_vp(state: &GameState, team: TeamId) -> u32 {
    state
        .team_members(team)
        .into_iter()
        .map(|p| vp_total(state, p))
        .sum()
}

/// How close to its target a team must be for others to treat it as about to win: within this
/// many VP (the race panel's public near-victory warning uses the same line).
pub const NEAR_VICTORY_VP: u32 = 2;

/// Whether `team` is close to winning by VP: alive and on a victory hold, or within
/// `NEAR_VICTORY_VP` of its target. Public: every input is on the race panel.
pub fn near_victory(state: &GameState, team: TeamId) -> bool {
    let members = state.team_members(team);
    if !members.iter().any(|p| state.player(*p).alive) {
        return false;
    }
    state.vp_holds.contains_key(&team)
        || team_vp(state, team).saturating_add(NEAR_VICTORY_VP) >= team_target(state, team)
}

/// The solo VP target times the number of team members.
pub fn team_target(state: &GameState, team: TeamId) -> u32 {
    state.setup.rule_experiments.solo_vp_target() * (state.team_members(team).len() as u32).max(1)
}

/// End-of-turn scoring: score objectives and goals, check victory.
/// Runs after every other phase and before the turn counter advances.
pub fn update(state: &mut GameState, events: &mut Vec<Event>) {
    let vp = crate::objectives::CaptureVpSnapshot::new(state);
    update_with_capture_snapshot(state, events, &vp);
}

/// End-of-turn scoring with the VP relation frozen before this turn's captures.
pub fn update_with_capture_snapshot(
    state: &mut GameState,
    events: &mut Vec<Event>,
    vp: &crate::objectives::CaptureVpSnapshot,
) {
    if state.finished {
        return;
    }
    crate::objectives::update(state, events, vp);
    for id in state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.id)
        .collect::<Vec<_>>()
    {
        // Sakoku's countdown: one more turn end closed.
        if let Some(n) = state.player_mut(id).sakoku_turns {
            state.player_mut(id).sakoku_turns = Some(n + 1);
        }
        check_faction_goal(state, id, events);
    }
    for id in (0..state.players.len()).map(|i| PlayerId(i as u8)) {
        check_great_work(state, id, events);
    }
    check_capitals(state, &vp.capitals, events);
    check_victory(state, events);
}

fn check_great_work(state: &mut GameState, of: PlayerId, events: &mut Vec<Event>) {
    let Some(building) = state.player(of).great_work_completed else {
        return;
    };
    let before = state.player(of).great_work_holder;
    let holder = crate::landmarks::find(state, of, building).map(|(_, h)| h);
    if before == holder {
        return;
    }
    if let Some(old) = before {
        events.push(Event::GreatWorkLost {
            player: old,
            of,
            building,
            vp: 2,
        });
    }
    if let Some(new) = holder {
        events.push(Event::GreatWorkCompleted {
            player: new,
            of,
            building,
            vp: 2,
            again: before.is_some(),
        });
    }
    state.player_mut(of).great_work_holder = holder;
}

/// Explored land tiles, a native-exploration measure used by visibility/alliance tests.
pub fn explored_land_tiles(state: &GameState, p: PlayerId) -> i32 {
    let explored = &state.player(p).explored;
    state
        .map
        .tiles
        .iter()
        .enumerate()
        .filter(|(i, t)| explored.get(*i) && t.terrain.is_land())
        .count() as i32
}

/// Claim a Permanent goal once; set or clear a Live goal every turn end (it scores only while it
/// holds). Held goals score through `goals::holders` and need no claim.
fn check_faction_goal(state: &mut GameState, p: PlayerId, events: &mut Vec<Event>) {
    let faction = state.player(p).faction;
    let spec = goals::spec_of(state, p);
    let done = state.player(p).faction_goal_done;
    let reached = match spec.kind {
        GoalKind::Permanent => done || goals::reached(state, p),
        GoalKind::Live => goals::reached(state, p),
        GoalKind::Held => return check_held_goal(state, p, events),
    };
    if reached == done {
        return;
    }
    let player = state.player_mut(p);
    player.faction_goal_done = reached;
    if reached {
        let again = player.faction_goal_reached_before;
        player.faction_goal_reached_before = true;
        events.push(Event::FactionGoalCompleted {
            player: p,
            faction,
            vp: spec.vp,
            again,
        });
    } else {
        events.push(Event::FactionGoalLost {
            player: p,
            faction,
            vp: spec.vp,
        });
    }
}

/// A Held goal scores for whoever holds its item (`goals::held_by`); report every change of
/// hands, the owner's own flag following whether it holds the item itself.
fn check_held_goal(state: &mut GameState, of: PlayerId, events: &mut Vec<Event>) {
    let faction = state.player(of).faction;
    let vp = goals::spec_of(state, of).vp;
    let holder = goals::held_by(state, of);
    state.player_mut(of).faction_goal_done = holder == Some(of);
    let before = state.player(of).goal_holder;
    if holder == before {
        return;
    }
    let player = state.player_mut(of);
    player.goal_holder = holder;
    let again = player.faction_goal_reached_before;
    if let Some(old) = before {
        events.push(Event::FactionGoalLost {
            player: old,
            faction,
            vp,
        });
    }
    if let Some(new) = holder {
        player.faction_goal_reached_before = true;
        events.push(Event::FactionGoalCompleted {
            player: new,
            faction,
            vp,
            again,
        });
    }
}

fn team_population(state: &GameState, team: TeamId) -> i32 {
    state
        .cities
        .values()
        .filter(|c| state.team_of(c.owner) == team)
        .map(|c| c.population)
        .sum()
}

pub(crate) fn team_label(state: &GameState, team: TeamId) -> String {
    state
        .team_members(team)
        .into_iter()
        .map(|p| state.player(p).name.clone())
        .collect::<Vec<_>>()
        .join(" & ")
}

fn finish(state: &mut GameState, winner: Option<TeamId>, reason: String, events: &mut Vec<Event>) {
    state.finished = true;
    state.winner = winner;
    events.push(match winner {
        Some(team) => Event::GameWon { team, reason },
        None => Event::GameDrawn { reason },
    });
}

/// Advance every team's VP hold at the end of a turn: a team at its target starts a hold (count
/// 1, event: the turn it reaches the target is the first turn held) and gains one per later turn
/// end; a team below it, or eliminated, loses its hold
/// (event if it had one). Returns the teams whose hold is complete.
fn update_vp_holds(
    state: &mut GameState,
    alive: &[TeamId],
    events: &mut Vec<Event>,
) -> Vec<TeamId> {
    let needed = crate::data::VP_HOLD_TURNS;
    let mut complete = Vec::new();
    for team in state.teams() {
        let vp = team_vp(state, team);
        let target = team_target(state, team);
        let at_target = alive.contains(&team) && vp >= target;
        match (state.vp_holds.get(&team).copied(), at_target) {
            (None, true) => {
                state.vp_holds.insert(team, 1);
                events.push(Event::VictoryHoldStarted {
                    team,
                    vp,
                    target,
                    turns_needed: needed,
                });
            }
            (Some(held), true) => {
                state.vp_holds.insert(team, held + 1);
            }
            (Some(held), false) => {
                state.vp_holds.remove(&team);
                events.push(Event::VictoryHoldBroken {
                    team,
                    turns_held: held,
                    vp,
                    target,
                });
            }
            (None, false) => {}
        }
        if state.vp_holds.get(&team).is_some_and(|h| *h >= needed) {
            complete.push(team);
        }
    }
    complete
}

/// Whether `member`'s original capital still counts for the capitals victory: it stands and a
/// member of the same team owns it (a capital owned by another team, an occupation lock included,
/// or no longer on the map has fallen; retaking it restores it). Hand-built scenarios and
/// `blank_game` may leave an empire without any founding record: they do not hand another team a
/// capitals victory before that empire has had a capital.
pub fn capital_stands(state: &GameState, member: PlayerId) -> bool {
    let team = state.team_of(member);
    let standing = state
        .cities
        .values()
        .any(|c| c.original_capital && c.founder == member && state.team_of(c.owner) == team);
    let founded = state.cities.values().any(|c| c.founder == member)
        || state.history.chronicle.iter().any(|entry| {
            matches!(entry, crate::history::Chronicle::Founded { player, .. } if *player == member)
        });
    standing || (state.player(member).alive && !founded)
}

/// Whether `team` still holds an original capital of its own: any member's capital stands.
pub fn holds_own_capital(state: &GameState, team: TeamId) -> bool {
    state
        .team_members(team)
        .into_iter()
        .any(|m| capital_stands(state, m))
}

/// Last team standing; else the only team that still holds an original capital of its own; else a
/// team that has held its VP target for `data::VP_HOLD_TURNS`
/// turn ends (several complete the same turn: highest VP, then population, exact tie = draw);
/// else at the turn limit the highest VP (exact tie = draw).
fn check_victory(state: &mut GameState, events: &mut Vec<Event>) {
    let all_teams = state.teams();
    let alive: Vec<TeamId> = all_teams
        .iter()
        .copied()
        .filter(|t| {
            state
                .team_members(*t)
                .iter()
                .any(|p| state.player(*p).alive)
        })
        .collect();

    if all_teams.len() > 1 && alive.len() <= 1 {
        match alive.first() {
            Some(t) => {
                let reason = format!("{} is the last team standing", team_label(state, *t));
                finish(state, Some(*t), reason, events);
            }
            None => finish(state, None, "No empire survived".into(), events),
        }
        return;
    }

    if all_teams.len() > 1 {
        let holders: Vec<TeamId> = alive
            .iter()
            .copied()
            .filter(|t| holds_own_capital(state, *t))
            .collect();
        if let [team] = holders[..] {
            let reason = format!(
                "{} holds the last capital: every rival's capital has fallen",
                team_label(state, team)
            );
            finish(state, Some(team), reason, events);
            return;
        }
    }

    let complete = update_vp_holds(state, &alive, events);

    // (team, vp, population), best first; ties end in the lowest team id.
    let mut ranked: Vec<(TeamId, u32, i32)> = alive
        .iter()
        .map(|t| (*t, team_vp(state, *t), team_population(state, *t)))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0)));

    let reached: Vec<(TeamId, u32, i32)> = ranked
        .iter()
        .copied()
        .filter(|(t, _, _)| complete.contains(t))
        .collect();
    if let Some(&(team, vp, pop)) = reached.first() {
        if reached.len() > 1 && reached[1].1 == vp && reached[1].2 == pop {
            let reason = format!(
                "Draw: {} teams completed their VP holds with {vp} VP and {pop} population",
                reached.iter().filter(|r| r.1 == vp && r.2 == pop).count()
            );
            finish(state, None, reason, events);
        } else {
            let reason = format!(
                "{} held {vp} of {} VP for {} turns",
                team_label(state, team),
                team_target(state, team),
                crate::data::VP_HOLD_TURNS
            );
            finish(state, Some(team), reason, events);
        }
        return;
    }

    if state
        .setup
        .turn_limit
        .is_some_and(|limit| state.turn >= limit)
        && let Some(&(team, vp, _)) = ranked.first()
    {
        if ranked.get(1).is_some_and(|r| r.1 == vp) {
            let reason = format!("Draw at the turn limit: tied on {vp} VP");
            finish(state, None, reason, events);
        } else {
            let reason = format!(
                "{} has the most VP ({vp}) at the turn limit",
                team_label(state, team)
            );
            finish(state, Some(team), reason, events);
        }
    }
}

#[cfg(test)]
mod capitals_victory_tests {
    use super::*;
    use crate::rules::test_support::{city, game, hex, unit};

    fn won_by(s: &mut GameState) -> (Option<TeamId>, String) {
        let mut events = Vec::new();
        check_victory(s, &mut events);
        let reason = events
            .iter()
            .find_map(|e| match e {
                Event::GameWon { reason, .. } | Event::GameDrawn { reason } => Some(reason.clone()),
                _ => None,
            })
            .unwrap_or_default();
        (s.winner.filter(|_| s.finished), reason)
    }

    fn found_in_chronicle(s: &mut GameState, p: u8) {
        s.history
            .chronicle
            .push(crate::history::Chronicle::Founded {
                turn: 1,
                player: PlayerId(p),
                city: "Lost".into(),
            });
    }

    fn capital(s: &mut GameState, owner: u8, col: i32) -> CityId {
        city(s, owner, hex(col, 5), 3)
    }

    #[test]
    fn the_last_rival_capital_captured_wins() {
        let mut s = game(&[0, 1]);
        let a = capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        assert_eq!(won_by(&mut s).0, None);
        s.cities.get_mut(&b).unwrap().owner = PlayerId(0);
        let (winner, reason) = won_by(&mut s);
        assert_eq!(winner, Some(TeamId(0)));
        assert!(reason.contains("holds the last capital"), "{reason}");
        assert!(s.cities.contains_key(&a));
    }

    #[test]
    fn a_third_team_holding_a_rival_capital_does_not_end_it_while_another_rival_stands() {
        let mut s = game(&[0, 1, 2]);
        capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        capital(&mut s, 2, 5);
        s.cities.get_mut(&b).unwrap().owner = PlayerId(2);
        assert_eq!(won_by(&mut s).0, None);
        assert!(!s.finished);
    }

    #[test]
    fn a_razed_capital_counts_as_fallen_and_keeps_its_team_from_winning_this_way() {
        let mut s = game(&[0, 1]);
        let a = capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        found_in_chronicle(&mut s, 0);
        found_in_chronicle(&mut s, 1);
        s.cities.remove(&b);
        assert_eq!(won_by(&mut s).0, Some(TeamId(0)));
        let mut s = game(&[0, 1]);
        capital(&mut s, 0, 2);
        capital(&mut s, 1, 8);
        found_in_chronicle(&mut s, 0);
        found_in_chronicle(&mut s, 1);
        s.cities.remove(&a);
        assert!(!holds_own_capital(&s, TeamId(0)));
        assert!(holds_own_capital(&s, TeamId(1)));
    }

    #[test]
    fn a_celtic_sack_of_the_last_rival_capital_wins() {
        let mut s = game(&[0, 1]);
        capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        found_in_chronicle(&mut s, 1);
        s.player_mut(PlayerId(0)).faction = Faction::Celts;
        let mut events = Vec::new();
        crate::rules::battle::capture_city(&mut s, b, PlayerId(0), &mut events);
        assert!(!s.cities.contains_key(&b));
        assert_eq!(won_by(&mut s).0, Some(TeamId(0)));
    }

    #[test]
    fn retaking_a_fallen_capital_restores_its_team() {
        let mut s = game(&[0, 1, 2]);
        capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        capital(&mut s, 2, 5);
        s.cities.get_mut(&b).unwrap().owner = PlayerId(0);
        assert!(!holds_own_capital(&s, TeamId(1)));
        s.cities.get_mut(&b).unwrap().owner = PlayerId(1);
        assert!(holds_own_capital(&s, TeamId(1)));
        assert_eq!(won_by(&mut s).0, None);
    }

    #[test]
    fn a_two_human_team_with_one_capital_lost_still_stands() {
        let mut s = game(&[0, 0, 1]);
        let a1 = capital(&mut s, 0, 2);
        capital(&mut s, 1, 5);
        let c = capital(&mut s, 2, 8);
        s.cities.get_mut(&a1).unwrap().owner = PlayerId(2);
        assert!(holds_own_capital(&s, TeamId(0)));
        assert_eq!(won_by(&mut s).0, None);
        s.cities.get_mut(&c).unwrap().owner = PlayerId(0);
        assert_eq!(won_by(&mut s).0, Some(TeamId(0)));
    }

    #[test]
    fn a_capital_owned_by_a_teammate_still_stands() {
        let mut s = game(&[0, 0, 1]);
        let a1 = capital(&mut s, 0, 2);
        capital(&mut s, 2, 8);
        s.cities.get_mut(&a1).unwrap().owner = PlayerId(1);
        assert!(holds_own_capital(&s, TeamId(0)));
    }

    #[test]
    fn nobody_wins_before_the_capitals_are_founded() {
        let mut s = game(&[0, 1]);
        unit(&mut s, 0, UnitKind::Settler, hex(2, 5));
        unit(&mut s, 1, UnitKind::Settler, hex(8, 5));
        assert_eq!(won_by(&mut s).0, None);
        capital(&mut s, 0, 2);
        assert_eq!(won_by(&mut s).0, None, "the rival has yet to found");
        capital(&mut s, 1, 8);
        assert_eq!(won_by(&mut s).0, None);
    }

    #[test]
    fn nobody_wins_when_no_team_holds_its_own_capital() {
        let mut s = game(&[0, 1]);
        let a = capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        s.cities.get_mut(&a).unwrap().owner = PlayerId(1);
        s.cities.get_mut(&b).unwrap().owner = PlayerId(0);
        assert_eq!(won_by(&mut s).0, None);
        assert!(!s.finished);
    }

    #[test]
    fn an_occupation_locked_capital_counts_as_fallen() {
        let mut s = game(&[0, 1]);
        capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        let c = s.cities.get_mut(&b).unwrap();
        c.owner = PlayerId(0);
        c.occupied_from = Some(PlayerId(1));
        assert_eq!(won_by(&mut s).0, Some(TeamId(0)));
    }

    #[test]
    fn a_replacement_capital_is_not_an_original_one() {
        let mut s = game(&[0, 1]);
        capital(&mut s, 0, 2);
        let b = capital(&mut s, 1, 8);
        found_in_chronicle(&mut s, 1);
        s.cities.remove(&b);
        let r = capital(&mut s, 1, 9);
        s.cities.get_mut(&r).unwrap().original_capital = false;
        assert!(!holds_own_capital(&s, TeamId(1)));
        assert_eq!(won_by(&mut s).0, Some(TeamId(0)));
    }

    #[test]
    fn a_single_team_game_is_not_decided_by_capitals() {
        let mut s = game(&[0, 0]);
        capital(&mut s, 0, 2);
        assert_eq!(won_by(&mut s).0, None);
    }
}

#[cfg(test)]
mod held_capital_tests {
    use super::*;
    use crate::objectives::CaptureVpSnapshot;
    use crate::rules::test_support::{city, game, hex};

    /// One turn end: `change` happens during the turn, then scoring runs against the
    /// start-of-turn snapshot, as `resolve_turn` does.
    fn turn(s: &mut GameState, change: impl FnOnce(&mut GameState)) -> Vec<Event> {
        let snapshot = CaptureVpSnapshot::new(s);
        change(s);
        let mut events = Vec::new();
        update_with_capture_snapshot(s, &mut events, &snapshot);
        events
    }

    fn holds(events: &[Event], started: bool, team: u8) -> bool {
        events.iter().any(|e| match e {
            Event::VictoryHoldStarted { team: t, .. } => started && *t == TeamId(team),
            Event::VictoryHoldBroken { team: t, .. } => !started && *t == TeamId(team),
            _ => false,
        })
    }

    #[test]
    fn a_held_capital_point_moves_with_the_city_and_breaks_the_hold_the_same_turn() {
        let mut s = game(&[0, 1, 2]);
        s.setup.rule_experiments.vp_target = Some(1);
        city(&mut s, 0, hex(2, 5), 3);
        let rome = city(&mut s, 1, hex(8, 5), 3);
        city(&mut s, 2, hex(14, 5), 3);
        let owner = |s: &mut GameState, p: u8| s.cities.get_mut(&rome).unwrap().owner = PlayerId(p);

        // Player 0 takes it: its point arrives and its hold starts.
        let events = turn(&mut s, |s| owner(s, 0));
        assert!(events.iter().any(|e| matches!(e,
            Event::CapitalTaken { captor: PlayerId(0), capital, .. } if *capital == rome)));
        assert_eq!(vp_total(&s, PlayerId(0)), CAPITAL_VP);
        assert!(holds(&events, true, 0));
        assert_eq!(s.vp_holds.get(&TeamId(0)), Some(&1));

        // Holding it on, the hold counts up and nothing is reported again.
        let events = turn(&mut s, |_| {});
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CapitalTaken { .. }))
        );
        assert_eq!(s.vp_holds.get(&TeamId(0)), Some(&2));

        // A third empire takes it: the point leaves player 0 the same turn, its hold breaks and
        // the new holder's hold starts.
        let events = turn(&mut s, |s| owner(s, 2));
        let lost = events
            .iter()
            .position(|e| matches!(e, Event::CapitalLost { player: PlayerId(0), capital, .. } if *capital == rome))
            .expect("player 0 loses the point");
        let taken = events
            .iter()
            .position(|e| {
                matches!(
                    e,
                    Event::CapitalTaken {
                        captor: PlayerId(2),
                        ..
                    }
                )
            })
            .expect("player 2 gains it");
        assert!(
            lost < taken,
            "the loss is reported first, like a world wonder"
        );
        assert_eq!(vp_total(&s, PlayerId(0)), 0);
        assert!(holds(&events, false, 0));
        assert!(!s.vp_holds.contains_key(&TeamId(0)));
        assert!(holds(&events, true, 2));

        // The founder takes it back: nobody holds the point.
        let events = turn(&mut s, |s| owner(s, 1));
        assert!(events.iter().any(|e| matches!(
            e,
            Event::CapitalLost {
                player: PlayerId(2),
                ..
            }
        )));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::CapitalTaken { .. }))
        );
        assert!(holds(&events, false, 2));
        assert!((0..3).all(|p| vp_total(&s, PlayerId(p)) == 0));
    }

    #[test]
    fn razing_a_held_capital_loses_its_point() {
        let mut s = game(&[0, 1, 2]);
        city(&mut s, 0, hex(2, 5), 3);
        let rome = city(&mut s, 1, hex(8, 5), 3);
        city(&mut s, 2, hex(14, 5), 3);
        turn(&mut s, |s| {
            s.cities.get_mut(&rome).unwrap().owner = PlayerId(0)
        });
        assert_eq!(
            capitals_held(&s, PlayerId(0)),
            vec![(rome, s.cities[&rome].name.clone())]
        );
        let events = turn(&mut s, |s| {
            s.cities.remove(&rome);
        });
        assert!(events.iter().any(|e| matches!(e,
            Event::CapitalLost { player: PlayerId(0), capital, .. } if *capital == rome)));
        assert!(capitals_held(&s, PlayerId(0)).is_empty());
    }
}

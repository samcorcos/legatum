//! The one-time goal actions as real decisions (`Order::Action`): Japan's Sakoku vow
//! (`goal_sakoku.rs`), the
//! Athens' broadcast and a broadcast's free-technology picks, Sparta's retirement of
//! veterans and the Celts' Call the Wild. Every brain gets them through `Brain::orders`, so they
//! are decided from the fogged view alone.
//!
//! Every rite is worth 2 VP and costs something that stays: the broadcast hands every rival
//! three technologies, the
//! retirement gives up three level-3 veterans, the Wild brings raiders to every city and camps
//! rivals can clear. So they are never fired by reflex. They fire when the 2 VP decides something:
//! it takes the team to the VP target with time left to hold it, or the game is late enough that
//! a lasting cost no longer matters.

use legatum_core::orders::Action;
use legatum_core::{Faction, Order, PlayerView, TechId, UnitId, data};

/// Share of the game (percent of its length) after which a rite's lasting cost no longer
/// matters. Three technologies for every rival keep paying the broadcast off.
const BROADCAST_LATE_PERCENT: u32 = 85;
/// The retirement gives up three level-3 veterans for good, and the Wild leaves camps to defend
/// and raiders to meet: both wait for the 2 VP to decide the game, or for the game to be late.
const RETIRE_LATE_PERCENT: u32 = 55;
const WILD_LATE_PERCENT: u32 = 70;
/// A Celts with expansion room left still calls the Wild by this share of the game.
const WILD_EXPANSION_PERCENT: u32 = 55;
/// Gene `celts_playbook` part 2: the Wild's window opens when its 2 VP leave this many VP to the
/// target (the designer called at 7 of 10 in 8.9.25).
const WILD_PLAYBOOK_SLACK: u32 = 1;
/// Turn ends a team must stay at its VP target to win (`data::VP_HOLD_TURNS`). A rite resolved this
/// turn counts as the first, so the last needed turn end is
/// `turn + needed - 1`; a game with fewer turns left can only be decided by the final count.
pub(crate) fn hold_turns(_view: &PlayerView) -> u32 {
    legatum_core::data::VP_HOLD_TURNS
}

/// The game's last turn: the turn limit, and with no limit (`open_end`, gene `goal_open_end`) there
/// is none to plan against: such a game ends when somebody wins, however long that takes, so a
/// goal whose build needs 50 turns is not "too late" at turn 800. The earlier fiction (the end a
/// tenth of the expected length ahead, calibration 2026-10-07) let a 9-VP Mongols that was 50
/// turns from its Yam Relay Station never start it in 900 turns (`docs/learnings.md` 2026-10-23).
pub(crate) fn last_turn(view: &PlayerView, open_end: bool) -> u32 {
    view.turn_limit.unwrap_or_else(|| {
        if open_end {
            u32::MAX
        } else {
            let expected = legatum_core::data::EXPECTED_GAME_TURNS;
            expected.max(view.turn + expected / 10)
        }
    })
}

/// Whether `percent` of the game is gone: measured against the turn limit, else the expected
/// game length (past it the game is always late).
pub(crate) fn is_late(view: &PlayerView, percent: u32, open_end: bool) -> bool {
    let length = if open_end {
        view.turn_limit
            .unwrap_or(legatum_core::data::EXPECTED_GAME_TURNS)
    } else {
        last_turn(view, false)
    };
    u64::from(view.turn) * 100 >= u64::from(length) * u64::from(percent)
}

/// (vp, target) of the viewer's team.
pub(crate) fn standing(view: &PlayerView) -> Option<(u32, u32)> {
    view.standings
        .iter()
        .find(|s| s.team == view.team)
        .map(|s| (s.vp, s.target))
}

/// Whether taking `vp_after` crosses the target with enough turns left to hold it, or the
/// game is late enough to bank the points.
pub(crate) fn rite_pays(
    view: &PlayerView,
    vp_after: u32,
    late_percent: u32,
    open_end: bool,
) -> bool {
    let Some((vp, target)) = standing(view) else {
        return false;
    };
    let gain = vp_after.saturating_sub(vp);
    if gain == 0 {
        return false;
    }
    let crosses = vp < target && vp_after >= target;
    let time_to_hold = view.turn + hold_turns(view) <= last_turn(view, open_end);
    (crosses && time_to_hold) || is_late(view, late_percent, open_end)
}

/// An enemy army next to one of our cities makes a lasting sacrifice unsafe.
fn city_under_threat(view: &PlayerView) -> bool {
    cities_under_threat(view, |_| true)
}

/// An enemy army next to one of our cities that `shrinks` picks.
fn cities_under_threat(view: &PlayerView, shrinks: impl Fn(legatum_core::CityId) -> bool) -> bool {
    let mine: Vec<_> = view
        .cities
        .iter()
        .filter(|c| c.owner == view.me && shrinks(c.id))
        .collect();
    view.units.iter().any(|u| {
        u.owner != view.me
            && !u.owner.is_barbarian()
            && view.teams_at_war(view.team, view.team_of(u.owner))
            && data::unit_stats(u.kind).military
            && mine.iter().any(|c| c.pos.distance(u.pos) <= 3)
    }) || view.units.iter().any(|u| {
        u.owner.is_barbarian()
            && !view.barbarians_spare_me()
            && data::unit_stats(u.kind).military
            && mine.iter().any(|c| c.pos.distance(u.pos) <= 2)
    })
}

/// Whether the Aztecs kindle the New Fire now. `closer` (gene `aztecs_playbook` bit 2, the
/// designer's 8.9.22 game 2: he kindled it at War with four empires on T144, when it took him
/// from 8 to 10 VP and started the hold he won on): when the rite starts the hold, only the
/// cities it shrinks need to be clear of an enemy army; the others keep their population.
fn swarm_now(view: &PlayerView, open_end: bool, closer: bool) -> bool {
    let Some(swarm) = &view.actions.swarm else {
        return false;
    };
    if swarm.blocker.is_some() {
        return false;
    }
    let Some((vp, _)) = standing(view) else {
        return false;
    };
    let points = legatum_core::goals::spec_for(legatum_core::goals::GoalRule::Swarm).vp;
    let net = vp + points;
    // Never when it costs as many points as it earns; and not into an enemy army, except in the
    // last turns when nothing after it matters.
    let starts_hold = standing(view).is_some_and(|(vp, target)| {
        vp < target && net >= target && view.turn + hold_turns(view) <= last_turn(view, open_end)
    });
    let threatened = if closer && starts_hold {
        cities_under_threat(view, |id| {
            swarm.cities.iter().any(|c| c.city == id && c.lost > 0)
        })
    } else {
        city_under_threat(view)
    };
    if net <= vp || (threatened && !is_late(view, 95, open_end)) {
        return false;
    }
    rite_pays(view, net, 75, open_end)
}

/// Whether the Athens broadcast now.
fn broadcast_now(view: &PlayerView, open_end: bool) -> bool {
    let Some(b) = &view.actions.broadcast else {
        return false;
    };
    if view.faction_of(view.me) != Faction::Athens || b.blocker.is_some() {
        return false;
    }
    let Some((vp, _)) = standing(view) else {
        return false;
    };
    rite_pays(
        view,
        vp + legatum_core::goals::spec(Faction::Athens).vp,
        BROADCAST_LATE_PERCENT,
        open_end,
    )
}

/// The Sparta's retirement window: the 2 VP take the team to the target with time to hold it,
/// or the game is late. Veterans gather at an Agoge city in this window (`gather_veterans`).
pub(crate) fn retire_window(view: &PlayerView, open_end: bool) -> bool {
    view.actions.retire.is_some()
        && rite_pays(
            view,
            standing(view).map_or(0, |(vp, _)| vp) + legatum_core::goals::spec(Faction::Sparta).vp,
            RETIRE_LATE_PERCENT,
            open_end,
        )
}

/// The veterans to retire this turn: the ones in an Agoge city, up to those still owed, once
/// enough veterans exist to finish (a part-retired
/// veteran guard scores nothing), the weakest first (lowest level, then HP). Never when it leaves
/// a threatened city with no garrison, except in the last turns.
fn retire_units(view: &PlayerView, open_end: bool) -> Vec<UnitId> {
    let Some(r) = &view.actions.retire else {
        return Vec::new();
    };
    let need = usize::from(r.target.saturating_sub(r.retired));
    let mut ready: Vec<(u8, i32, UnitId)> = r
        .veterans
        .iter()
        .filter(|v| v.blocker.is_none())
        .filter_map(|v| {
            let u = view.units.iter().find(|u| u.id == v.unit)?;
            Some((v.level, u.hp, v.unit))
        })
        .collect();
    let Some((vp, _)) = standing(view) else {
        return Vec::new();
    };
    // Retirements count as they happen (`retired`), and only one veteran can stand in each War
    // Shrine city, so the owed veterans go as they arrive once enough exist to finish (calibration
    // 2026-10-07: waiting for all three in one turn retired almost nobody).
    if need == 0 || ready.is_empty() || r.veterans.len() < need {
        return Vec::new();
    }
    ready.sort();
    let chosen: Vec<UnitId> = ready.iter().take(need).map(|(_, _, id)| *id).collect();
    let net = vp + legatum_core::goals::spec(Faction::Sparta).vp;
    if net <= vp || !rite_pays(view, net, RETIRE_LATE_PERCENT, open_end) {
        return Vec::new();
    }
    // A retired veteran leaves his city; one it can't spare stays while an army is at the gate.
    let empties_a_city = chosen.iter().any(|id| {
        let at = view.units.iter().find(|u| u.id == *id).map(|u| u.pos);
        at.is_some_and(|at| {
            !view.units.iter().any(|u| {
                u.owner == view.me
                    && u.military
                    && u.kind != legatum_core::UnitKind::Scout
                    && u.pos == at
                    && !chosen.contains(&u.id)
            })
        })
    });
    if empties_a_city && city_under_threat(view) && !is_late(view, 95, open_end) {
        return Vec::new();
    }
    chosen
}

/// The Celts's window: the Wild's 2 VP take the team to the target with time to hold it, or the
/// game is late. A Celts already past the target without the goal has no use for it. While the
/// goal is held, its 2 VP are already in the standing, so they are taken out first.
///
/// Gene `celts_playbook` part 2 (`playbook`) opens it as the designer did in 8.9.25: when the
/// 2 VP leave at most `WILD_PLAYBOOK_SLACK` VP to the target with time to hold it (he called at
/// 7 of 10 and held the trigger before, "otherwise id get too many points and people would gang
/// up on me"). Calling whenever the points drew no lead was measured and rejected: the AI Celts
/// called at a median T12 and lost 4.3 points of win share, their camps slowing expansion.
pub(crate) fn wild_window(
    view: &PlayerView,
    open_end: bool,
    decisions: &crate::DecisionParams,
) -> bool {
    if view.actions.wild.is_none() {
        return false;
    }
    let Some((vp, target)) = standing(view) else {
        return false;
    };
    let gain = legatum_core::goals::spec(Faction::Celts).vp;
    let held = view.faction_goal.as_ref().is_some_and(|g| g.done);
    let base = vp.saturating_sub(if held { gain } else { 0 });
    if base >= target {
        return false;
    }
    let time_to_hold = view.turn + hold_turns(view) <= last_turn(view, open_end);
    if crate::celts_playbook::playbook(view, decisions.celts_playbook, crate::celts_playbook::WILD)
    {
        return (base + gain + WILD_PLAYBOOK_SLACK >= target && time_to_hold)
            || is_late(view, decisions.celts_wild_late as u32, open_end);
    }
    (base + gain >= target && time_to_hold) || is_late(view, WILD_LATE_PERCENT, open_end)
}

/// Whether the Celts call the Wild now: the window is open, a city lacks a camp, and expansion is
/// done (no Settler under way and no room left, or the game is far enough along): a city founded
/// afterwards breaks the goal until another call.
fn wild_now(view: &PlayerView, open_end: bool, decisions: &crate::DecisionParams) -> bool {
    let Some(w) = &view.actions.wild else {
        return false;
    };
    if view.faction_of(view.me) != Faction::Celts || w.blocker.is_some() {
        return false;
    }
    // Gene `celts_playbook` part 2: a call adds a camp only to the cities without one, so it
    // pays whenever every city holds a camp afterwards; a city founded later is covered by the
    // next call, as the designer re-called after each new city and each cleared camp.
    if crate::celts_playbook::playbook(view, decisions.celts_playbook, crate::celts_playbook::WILD)
    {
        return w.with_camp < w.cities
            && w.with_camp + w.sites.len() as i32 >= w.cities
            && wild_window(view, open_end, decisions);
    }
    // A call that can't give every city a camp would only add raiders: wait until each has room.
    if w.with_camp >= w.cities
        || (w.sites.len() as i32) < w.cities
        || !wild_window(view, open_end, decisions)
    {
        return false;
    }
    is_late(view, WILD_EXPANSION_PERCENT, open_end) || expansion_done(view)
}

fn expansion_done(view: &PlayerView) -> bool {
    let settler = legatum_core::ProductionItem::Unit {
        unit: legatum_core::UnitKind::Settler,
    };
    let under_way = view
        .units
        .iter()
        .any(|u| u.owner == view.me && u.kind == legatum_core::UnitKind::Settler)
        || view.cities.iter().filter(|c| c.owner == view.me).any(|c| {
            c.detail.as_ref().is_some_and(|d| {
                d.producing == Some(settler) || d.production_queue.contains(&settler)
            })
        });
    if under_way {
        return false;
    }
    let faction = view.faction_of(view.me);
    let planner = crate::scripted::Planner::new(
        view,
        crate::Knobs::for_faction(faction),
        faction,
        u64::from(view.turn),
    );
    !planner.expansion_room()
}

/// The free-technology picks of a received broadcast: the best-valued technologies the empire
/// can take now, following up with the ones its picks unlock. `value` is the AI's research
/// valuation of a technology.
pub(crate) fn pick_free_techs(
    view: &PlayerView,
    pending: u8,
    value: impl Fn(TechId) -> i32,
) -> Vec<TechId> {
    let mut known: Vec<TechId> = view.research.known.clone();
    let mut picks: Vec<TechId> = Vec::new();
    // Free techs follow the gate as it stands when they arrive: only the receiver's current
    // era, even when the picks finish it.
    let era = legatum_core::eras::current_era(&known.iter().copied().collect());
    while picks.len() < usize::from(pending) {
        let best = TechId::ALL
            .iter()
            .copied()
            .filter(|t| !known.contains(t))
            .filter(|t| legatum_core::eras::era_of_tech(*t) == era)
            .max_by_key(|t| (value(*t), data::tech_stats(*t).cost, std::cmp::Reverse(*t)));
        let Some(tech) = best else { break };
        known.push(tech);
        picks.push(tech);
    }
    picks
}

/// Whether closing the country would ride with a declaration of war or an Alliance signature
/// among `orders` (core `diplomacy::vow_conflict`): the vow then waits a turn, since the treaty
/// and war decisions were scored on their own value and a signed Alliance blocks the vow anyway.
fn vow_rides_with_diplomacy(view: &PlayerView, orders: &[Order]) -> bool {
    orders.iter().any(|o| match o {
        Order::DeclareWar { .. } => true,
        Order::ProposeTreaty { kind, .. } => *kind == legatum_core::diplomacy::TreatyKind::Alliance,
        Order::RespondToProposal {
            proposal,
            accept: true,
            ..
        } => {
            view.diplomacy.proposals.iter().any(|p| {
                p.id == *proposal && p.kind == legatum_core::diplomacy::TreatyKind::Alliance
            })
        }
        _ => false,
    })
}

/// The planned `orders` with this turn's goal actions added (a Sakoku close drops out when it
/// would ride with a declaration or an Alliance signature).
pub fn with_goal_actions(
    view: &PlayerView,
    orders: &mut Vec<Order>,
    open_end: bool,
    decisions: &crate::DecisionParams,
) {
    let actions = goal_actions_inner(view, open_end, decisions);
    let conflict = vow_rides_with_diplomacy(view, orders);
    orders.extend(actions.into_iter().filter(|o| {
        !(conflict
            && matches!(
                o,
                Order::Action {
                    action: Action::Sakoku { close: true }
                }
            ))
    }));
}

/// The goal actions this brain orders this turn, from the view alone.
#[cfg(test)]
pub fn goal_actions(view: &PlayerView) -> Vec<Order> {
    let d = crate::DecisionParams::default();
    goal_actions_inner(view, d.goal_open_end > 0, &d)
}

fn goal_actions_inner(
    view: &PlayerView,
    open_end: bool,
    decisions: &crate::DecisionParams,
) -> Vec<Order> {
    let mut out = Vec::new();
    if let Some(free) = &view.actions.free_techs {
        let faction = view.faction_of(view.me);
        let planner = crate::scripted::Planner::new(
            view,
            crate::Knobs::for_faction(faction),
            faction,
            u64::from(view.turn),
        );
        let techs = pick_free_techs(view, free.pending, |t| planner.tech_value(t));
        if !techs.is_empty() {
            out.push(Order::Action {
                action: Action::ChooseFreeTechs { techs },
            });
        }
    }
    // Gene `aztecs_playbook` bit 2: the New Fire as the closer.
    let closer = view.faction_of(view.me) == Faction::Aztecs
        && decisions.aztecs_playbook & crate::production::AZTEC_CLOSER != 0;
    if swarm_now(view, open_end, closer) {
        out.push(Order::Action {
            action: Action::Swarm,
        });
    }
    if crate::goal_sakoku::sakoku_close_now(view, open_end) {
        out.push(Order::Action {
            action: Action::Sakoku { close: true },
        });
    } else if crate::goal_sakoku::sakoku_reopen_now(view, open_end) {
        out.push(Order::Action {
            action: Action::Sakoku { close: false },
        });
    }
    if broadcast_now(view, open_end) {
        out.push(Order::Action {
            action: Action::Broadcast,
        });
    }
    for unit in retire_units(view, open_end) {
        out.push(Order::Action {
            action: Action::RetireVeteran { unit },
        });
    }
    if wild_now(view, open_end, decisions) {
        out.push(Order::Action {
            action: Action::CallTheWild,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
    use crate::{Brain, ScriptedBrain};
    use legatum_core::orders::Action;
    use legatum_core::rules::actions::{BroadcastView, FreeTechsView, SwarmCityView, SwarmView};
    use legatum_core::view::TeamStanding;
    use legatum_core::{CityId, Hex, PlayerId, TeamId, UnitKind};

    /// An Aztec view where the New Fire is available: two big cities and the preview.
    fn aztecs(turn: u32, vp: u32, target: u32) -> PlayerView {
        let mut v = ViewBuilder::new(30, 20)
            .faction(Faction::Aztecs)
            .city(1, ME, off(3, 3), 30)
            .city(2, ME, off(3, 10), 30)
            .city(3, ENEMY, off(20, 10), 12)
            .build();
        v.turn = turn;
        v.turn_limit = None;
        v.standings = standing(vp, target).into();
        v.actions.swarm = Some(SwarmView {
            population: legatum_core::goals::AZTEC_NEW_FIRE_POPULATION,
            spare: 58,
            blocker: None,
            cities: vec![
                SwarmCityView {
                    city: CityId(1),
                    lost: 15,
                    population: 15,
                    tiles: vec![Hex::new(1, 1); 15],
                },
                SwarmCityView {
                    city: CityId(2),
                    lost: 15,
                    population: 15,
                    tiles: vec![Hex::new(2, 2); 15],
                },
            ],
        });
        v
    }

    fn swarms(v: &PlayerView) -> bool {
        goal_actions(v).contains(&Order::Action {
            action: Action::Swarm,
        })
    }

    #[test]
    fn the_swarm_is_never_fired_early_by_reflex() {
        // Turn 20 of 150, far from the target: the lasting cost isn't worth 2 VP yet.
        assert!(!swarms(&aztecs(20, 3, 20)));
        assert!(!swarms(&aztecs(20, 5, 20)));
        assert!(!swarms(&aztecs(40, 8, 20)));
    }

    #[test]
    fn the_swarm_fires_when_it_takes_the_team_to_the_target_with_time_to_hold() {
        assert!(swarms(&aztecs(70, 18, 20)));
        assert!(swarms(&aztecs(70, 19, 20)));
        // With no turns left to hold the target, late-game banking still fires it.
        assert!(swarms(&aztecs(148, 18, 20)));
    }

    #[test]
    fn the_hold_length_is_ten_standard_turns() {
        let mut v = aztecs(140, 18, 20);
        assert_eq!(hold_turns(&v), 10, "Standard");
        // Mid-game, taking the team to the target with 160 turns to hold it fires the rite.
        assert!(swarms(&v));
        // The same points short of the target, far from late, do not.
        v.standings = vec![TeamStanding {
            team: v.team,
            members: vec![ME],
            vp: 11,
            target: 20,
            victory_hold: None,
        }]
        .into();
        assert!(!swarms(&v));
    }

    #[test]
    fn the_swarm_fires_late_to_bank_the_points() {
        assert!(swarms(&aztecs(240, 5, 20)));
        assert!(!swarms(&aztecs(200, 5, 20)));
    }

    #[test]
    fn the_swarm_waits_while_an_enemy_army_stands_at_a_city() {
        let mut v = aztecs(70, 18, 20);
        assert!(swarms(&v));
        v = ViewBuilder { view: v }
            .unit(1, ENEMY, UnitKind::Spearman, off(5, 3))
            .build();
        assert!(!swarms(&v));
        // In the last turns nothing after it matters.
        v.turn = 145;
        v.turn_limit = Some(150);
        assert!(swarms(&v));
    }

    #[test]
    fn the_closer_kindles_the_new_fire_beside_an_army_at_a_city_it_does_not_shrink() {
        let decisions = crate::DecisionParams {
            aztecs_playbook: crate::production::AZTEC_CLOSER,
            ..Default::default()
        };
        let closer = |v: &PlayerView| {
            goal_actions_inner(v, false, &decisions).contains(&Order::Action {
                action: Action::Swarm,
            })
        };
        // Only Tenochtitlan (city 1) gives population; an enemy army stands at city 2.
        let mut v = aztecs(70, 18, 20);
        let swarm = v.actions.swarm.as_mut().unwrap();
        swarm.cities[0].lost = 30;
        swarm.cities[1].lost = 0;
        let v = ViewBuilder { view: v }
            .unit(1, ENEMY, UnitKind::Spearman, off(5, 10))
            .build();
        assert!(!swarms(&v), "the stock rite waits for every city");
        assert!(closer(&v), "the closer starts the hold");
        // An army at a city that shrinks still stops it.
        let v = ViewBuilder { view: v }
            .unit(2, ENEMY, UnitKind::Spearman, off(5, 3))
            .build();
        assert!(!closer(&v));
        // And it only overrides the wait when the rite starts the hold.
        let mut far = aztecs(70, 11, 20);
        far.actions.swarm.as_mut().unwrap().cities[1].lost = 0;
        let far = ViewBuilder { view: far }
            .unit(1, ENEMY, UnitKind::Spearman, off(5, 10))
            .build();
        assert!(!closer(&far));
    }

    #[test]
    fn the_swarm_is_not_ordered_when_unavailable() {
        let mut v = aztecs(120, 5, 20);
        v.actions.swarm.as_mut().unwrap().blocker = Some("short".into());
        assert!(!swarms(&v));
        v.actions.swarm = None;
        assert!(goal_actions(&v).is_empty());
    }

    fn standing(vp: u32, target: u32) -> Vec<TeamStanding> {
        vec![TeamStanding {
            team: TeamId(0),
            members: vec![ME, PlayerId(1)],
            vp,
            target,
            victory_hold: None,
        }]
    }

    fn athens(turn: u32, vp: u32, target: u32) -> PlayerView {
        let mut v = ViewBuilder::new(30, 20)
            .faction(Faction::Athens)
            .city(1, ME, off(3, 3), 5)
            .city(3, ENEMY, off(20, 10), 5)
            .build();
        v.turn = turn;
        v.turn_limit = None;
        v.standings = standing(vp, target).into();
        v.actions.broadcast = Some(BroadcastView {
            techs_each: 3,
            blocker: None,
            receivers: vec![ENEMY],
        });
        v
    }

    fn broadcasts(v: &PlayerView) -> bool {
        goal_actions(v).contains(&Order::Action {
            action: Action::Broadcast,
        })
    }

    #[test]
    fn the_broadcast_waits_for_the_target_or_the_late_game() {
        assert!(!broadcasts(&athens(20, 3, 20)), "never early by reflex");
        assert!(!broadcasts(&athens(90, 5, 20)), "not mid-game for 2 VP");
        assert!(
            broadcasts(&athens(80, 18, 20)),
            "it takes the team to the target"
        );
        assert!(
            broadcasts(&athens(260, 5, 20)),
            "late: rivals' techs matter little"
        );
        let mut v = athens(260, 5, 20);
        v.actions.broadcast.as_mut().unwrap().blocker = Some("no rival".into());
        assert!(!broadcasts(&v));
    }

    #[test]
    fn a_received_broadcast_is_spent_on_valuable_technologies_within_its_current_era() {
        let v = ViewBuilder::new(24, 18)
            .city(1, ME, off(3, 3), 4)
            .city(3, ENEMY, off(20, 10), 5)
            .build();
        let known = v.research.known.clone();
        let open: Vec<TechId> = v.research.available.iter().map(|o| o.tech).collect();
        assert!(open.len() >= 2);
        // The valuation decides: the last open tech is worth most, then whatever it unlocks.
        let favourite = *open.last().unwrap();
        let picks = pick_free_techs(&v, 3, |t| if t == favourite { 1000 } else { 0 });
        assert_eq!(picks.len(), 3);
        assert_eq!(picks[0], favourite);
        let mut seen: std::collections::BTreeSet<TechId> = known.iter().copied().collect();
        for t in &picks {
            assert_eq!(
                legatum_core::eras::era_of_tech(*t),
                legatum_core::eras::current_era(&known.iter().copied().collect()),
                "{t:?} picked outside the receiver's current era"
            );
            seen.insert(*t);
        }
        // A tree with nothing left to pick returns fewer.
        assert!(pick_free_techs(&v, 0, |_| 1).is_empty());
    }

    #[test]
    fn a_receiver_orders_a_valid_choice_through_its_brain() {
        let mut v = ViewBuilder::new(24, 18)
            .city(1, ME, off(3, 3), 4)
            .city(3, ENEMY, off(20, 10), 5)
            .build();
        v.actions.free_techs = Some(FreeTechsView {
            pending: 3,
            from: Some(ENEMY),
            captured: None,
            choices: v.research.available.iter().map(|t| t.tech).collect(),
        });
        let mut brain = ScriptedBrain::new(Faction::Switzerland, 7, ME);
        let orders = brain.orders(&v);
        assert_valid(&v, &orders);
        let chosen: Vec<_> = orders
            .iter()
            .filter_map(|o| match o {
                Order::Action {
                    action: Action::ChooseFreeTechs { techs },
                } => Some(techs.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0].len(), 3);
        // Nothing pending: no choice is ordered, and none is made on a rival's behalf.
        v.actions.free_techs = None;
        assert!(
            !ScriptedBrain::new(Faction::Switzerland, 7, ME)
                .orders(&v)
                .iter()
                .any(|o| matches!(o, Order::Action { .. }))
        );
    }
}

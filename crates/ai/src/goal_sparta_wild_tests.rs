use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
use crate::goal_actions::goal_actions;
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::orders::Action;
use legatum_core::rules::actions::{RetireVeteranView, RetireView, WildSiteView, WildView};
use legatum_core::view::{PlayerView, TeamStanding};
use legatum_core::{
    PlayerId, ProductionItem, TeamId, Terrain, UnitId, UnitKind, goals::FactionGoalView,
};

const RETIRE_LEVEL: u8 = legatum_core::goals::SPARTA_RETIRE_LEVEL;

fn planner(v: &PlayerView) -> Planner<'_> {
    Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7)
}

/// The plain Celts path, with gene `celts_playbook` off (Hard Celts play it by default).
fn planner_off(v: &PlayerView) -> Planner<'_> {
    let mut p = planner(v);
    p.decisions.celts_playbook = 0;
    p
}

fn standing(v: &mut PlayerView, vp: u32, target: u32) {
    v.standings = vec![TeamStanding {
        team: TeamId(0),
        members: vec![ME, PlayerId(1)],
        vp,
        target,
        victory_hold: None,
    }]
    .into();
}

fn orders_of(v: &PlayerView) -> Vec<Action> {
    goal_actions(v)
        .into_iter()
        .filter_map(|o| match o {
            Order::Action { action } => Some(action),
            _ => None,
        })
        .collect()
}

// ---------- Sparta ----------

/// An Sparta with an Agoge in city 1, a plain city 2, an enemy city, a garrison soldier in
/// each home city, and one soldier per entry of `levels` in city 1 (ids 10, 11, ...).
fn sparta(turn: u32, vp: u32, target: u32, retired: u8, levels: &[u8]) -> PlayerView {
    let mut b = ViewBuilder::new(30, 20)
        .faction(Faction::Sparta)
        .city(1, ME, off(3, 3), 6)
        .city(2, ME, off(3, 10), 6)
        .city(3, ENEMY, off(24, 10), 6)
        .unit(90, ME, UnitKind::Spearman, off(3, 3))
        .unit(91, ME, UnitKind::Spearman, off(3, 10));
    for (i, _) in levels.iter().enumerate() {
        b = b.unit(10 + i as u32, ME, UnitKind::Spearman, off(3, 3));
    }
    let mut v = b.build();
    v.turn = turn * 2;
    v.turn_limit = None;
    v.players[0].faction_goal_done = false;
    standing(&mut v, vp, target);
    v.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .buildings
        .push(BuildingKind::Agoge);
    let mut veterans = Vec::new();
    for (i, level) in levels.iter().enumerate() {
        let u = v
            .units
            .iter_mut()
            .find(|u| u.id == UnitId(10 + i as u32))
            .unwrap();
        u.level = *level;
        veterans.push(RetireVeteranView {
            unit: u.id,
            level: *level,
            blocker: None,
        });
    }
    v.actions.retire = Some(RetireView {
        retired,
        target: 3,
        level: legatum_core::goals::SPARTA_RETIRE_LEVEL,
        shrines: vec![legatum_core::CityId(1)],
        veterans,
    });
    v
}

fn retired(v: &PlayerView) -> Vec<UnitId> {
    orders_of(v)
        .into_iter()
        .filter_map(|a| match a {
            Action::RetireVeteran { unit } => Some(unit),
            _ => None,
        })
        .collect()
}

#[test]
fn veterans_retire_only_when_the_two_vp_decide_something() {
    let levels = [RETIRE_LEVEL, RETIRE_LEVEL, RETIRE_LEVEL + 1];
    // Far from the target and early: three veterans are worth more than 2 VP.
    assert!(retired(&sparta(20, 5, 17, 0, &levels)).is_empty());
    assert!(retired(&sparta(40, 14, 17, 0, &levels)).is_empty());
    // The 2 VP take the team to the target with time to hold it.
    assert_eq!(retired(&sparta(70, 15, 17, 0, &levels)).len(), 3);
    // Late in the game they are banked whatever the standing: the window opens at 55% of the
    // Former-pacing save's turn 83; this path now runs at Standard.
    assert_eq!(retired(&sparta(85, 5, 17, 0, &levels)).len(), 3);
    assert!(retired(&sparta(80, 5, 17, 0, &levels)).is_empty());
}

#[test]
fn retirement_retires_the_ready_veterans_as_they_arrive_and_takes_the_weakest() {
    let lv = RETIRE_LEVEL;
    // Fewer veterans in total than owed: nothing is retired (a part-retired guard scores nothing).
    assert!(retired(&sparta(70, 15, 17, 0, &[lv, lv + 1])).is_empty());
    // Three veterans in total, three owed, only one standing in the shrine city: that one goes
    // now, the rest follow as they arrive.
    let mut v = sparta(70, 15, 17, 0, &[lv, lv, lv]);
    for veteran in &mut v.actions.retire.as_mut().unwrap().veterans[1..] {
        veteran.blocker = Some("elsewhere".into());
    }
    assert_eq!(retired(&v), vec![UnitId(10)]);
    // Two of the three in place: both go.
    let mut v = sparta(70, 15, 17, 0, &[lv, lv, lv]);
    v.actions.retire.as_mut().unwrap().veterans[2].blocker = Some("elsewhere".into());
    let mut gone = retired(&v);
    gone.sort();
    assert_eq!(gone, vec![UnitId(10), UnitId(11)]);
    // None in place: nothing to retire yet.
    let mut v = sparta(70, 15, 17, 0, &[lv, lv, lv]);
    for veteran in &mut v.actions.retire.as_mut().unwrap().veterans {
        veteran.blocker = Some("elsewhere".into());
    }
    assert!(retired(&v).is_empty());
    // Four ready, three owed: the three weakest go, the strongest stays.
    let v = sparta(70, 15, 17, 0, &[lv + 2, lv, lv + 1, lv]);
    let mut gone = retired(&v);
    gone.sort();
    assert_eq!(gone, vec![UnitId(11), UnitId(12), UnitId(13)]);
    // Two already retired: one more completes it, even with fewer veterans than the target.
    let v = sparta(70, 15, 17, 2, &[lv + 1, lv]);
    assert_eq!(retired(&v), vec![UnitId(11)]);
}

#[test]
fn retirement_waits_while_an_enemy_army_stands_at_a_city_it_would_empty() {
    let mut v = sparta(70, 15, 17, 0, &[RETIRE_LEVEL; 3]);
    // The garrison soldier (90) stays in city 1, so it is not emptied.
    v = ViewBuilder { view: v }
        .unit(50, ENEMY, UnitKind::Spearman, off(5, 3))
        .build();
    assert_eq!(retired(&v).len(), 3);
    // Take the garrison away: the veterans are the city's only defenders.
    v.units.retain(|u| u.id != UnitId(90));
    assert!(retired(&v).is_empty());
    v.turn = 290;
    v.turn_limit = Some(300);
    assert_eq!(
        retired(&v).len(),
        3,
        "in the last turns nothing after it matters"
    );
}

#[test]
fn the_sparta_ai_orders_are_valid_with_retirement() {
    let v = sparta(70, 15, 17, 0, &[4, 4, 4]);
    let out = ScriptedBrain::new(Faction::Sparta, 7, ME).orders(&v);
    assert_valid(&v, &out);
    assert_eq!(
        out.iter()
            .filter(|o| matches!(
                o,
                Order::Action {
                    action: Action::RetireVeteran { .. }
                }
            ))
            .count(),
        3
    );
}

#[test]
fn veterans_gather_at_an_agoge_city_in_the_window() {
    // Three veterans stand in city 2 (no shrine); the window is open.
    let mut v = sparta(70, 15, 17, 0, &[4, 4, 4]);
    for u in v.units.iter_mut().filter(|u| u.level >= 4) {
        u.pos = off(3, 10);
    }
    let mut p = planner(&v);
    p.gather_veterans();
    let agoge = off(3, 3);
    for id in [10, 11, 12] {
        assert!(
            p.orders.iter().any(|o| matches!(o,
                Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. }
                    if *unit == UnitId(id) && to.distance(agoge) <= 1)),
            "veteran {id} heads for the Agoge city: {:?}",
            p.orders
        );
    }
    // Outside the window (early, far from the target) nobody is called in.
    let mut early = sparta(20, 5, 17, 0, &[4, 4, 4]);
    for u in early.units.iter_mut().filter(|u| u.level >= 4) {
        u.pos = off(3, 10);
    }
    let mut p = planner(&early);
    p.gather_veterans();
    assert!(p.orders.is_empty());
    // Too few veterans to finish the goal: nothing to gather for.
    let mut few = sparta(70, 15, 17, 0, &[4, 4]);
    for u in few.units.iter_mut().filter(|u| u.level >= 4) {
        u.pos = off(3, 10);
    }
    let mut p = planner(&few);
    p.gather_veterans();
    assert!(p.orders.is_empty());
    // Veterans already in the shrine city hold there.
    let at_home = sparta(70, 15, 17, 0, &[4, 4, 4]);
    let mut p = planner(&at_home);
    p.gather_veterans();
    assert!(
        p.orders
            .iter()
            .any(|o| matches!(o, Order::SkipUnit { unit } if *unit == UnitId(10)))
    );
}

#[test]
fn the_agoge_goes_in_one_city_once_a_third_of_the_game_is_gone_or_a_veteran_exists() {
    let agoge = ProductionItem::Building {
        building: BuildingKind::Agoge,
    };
    // No shrine anywhere yet: strip the fixture's.
    let bare = |turn, levels: &[u8]| {
        let mut v = sparta(turn, 5, 17, 0, levels);
        v.cities[0].detail.as_mut().unwrap().buildings.clear();
        v.cities[1].detail.as_mut().unwrap().yields.total.production = 9;
        v
    };
    // Early and no veteran: not yet.
    let v = bare(10, &[]);
    assert!(planner(&v).agoge_cities().is_empty());
    assert_eq!(planner(&v).agoge_value(&v.cities[1]), 0);
    // A veteran exists: the highest-Production city is the site, and only it.
    let v = bare(10, &[4]);
    let p = planner(&v);
    assert_eq!(p.agoge_site(), Some(legatum_core::CityId(2)));
    assert!(p.agoge_value(&v.cities[1]) >= 300);
    assert_eq!(p.agoge_value(&v.cities[0]), 0);
    // Later in the game it is wanted regardless.
    let v = bare(60, &[]);
    assert!(planner(&v).agoge_value(&v.cities[1]) >= 300);
    // Being built, or standing, anywhere ends the demand.
    let mut building = bare(60, &[]);
    building.cities[0].detail.as_mut().unwrap().producing = Some(agoge);
    assert_eq!(planner(&building).agoge_site(), None);
    let standing_one = sparta(60, 5, 17, 0, &[]);
    assert_eq!(planner(&standing_one).agoge_site(), None);
    // A done goal needs none.
    let mut done = bare(60, &[]);
    done.actions.retire = None;
    assert_eq!(planner(&done).agoge_site(), None);
}

#[test]
fn rivals_see_a_retired_sparta_army_as_a_thin_target() {
    let mut v = ViewBuilder::new(30, 20)
        .faction(Faction::Switzerland)
        .city(1, ME, off(3, 3), 6)
        .city(2, ENEMY, off(20, 10), 6)
        .unit(50, ENEMY, UnitKind::Spearman, off(20, 10))
        .build();
    v.players[2].faction = Faction::Sparta;
    v.players[2].faction_goal_done = false;
    assert_eq!(planner(&v).retired_army_weakness_value(&v.cities[1]), 0);
    // Its veterans retired (the goal is public): two defenders short of a full garrison.
    v.players[2].faction_goal_done = true;
    let thin = planner(&v).retired_army_weakness_value(&v.cities[1]);
    assert_eq!(thin, 2 * RETIRED_VALUE_PER_MISSING_UNIT);
    // A full garrison is no weakness; a teammate's city is never a target.
    let full = ViewBuilder { view: v.clone() }
        .unit(51, ENEMY, UnitKind::Spearman, off(20, 10))
        .unit(52, ENEMY, UnitKind::Spearman, off(20, 10))
        .build();
    assert_eq!(
        planner(&full).retired_army_weakness_value(&full.cities[1]),
        0
    );
    let mut ally = v.clone();
    ally.players[2].team = TeamId(0);
    assert_eq!(
        planner(&ally).retired_army_weakness_value(&ally.cities[1]),
        0
    );
}

// ---------- Celts ----------

/// A Celts with two cities, the call available (`with_camp` cities already have a camp), and the
/// goal `held` or not. The map is all water but around the cities unless `room` is set, so the
/// expansion planner finds no site.
fn celts(turn: u32, vp: u32, target: u32, with_camp: i32, held: bool, room: bool) -> PlayerView {
    let mut v = ViewBuilder::new(30, 20)
        .faction(Faction::Celts)
        .city(1, ME, off(3, 3), 6)
        .city(2, ME, off(3, 10), 6)
        .city(3, ENEMY, off(24, 10), 6)
        .unit(90, ME, UnitKind::Spearman, off(3, 3))
        .unit(91, ME, UnitKind::Spearman, off(3, 10))
        .build();
    if !room {
        for (i, t) in v.tiles.iter_mut().enumerate() {
            let h = off((i % 30) as i32, (i / 30) as i32);
            let near = [off(3, 3), off(3, 10), off(24, 10)]
                .iter()
                .any(|c| c.distance(h) <= 1);
            t.terrain = Some(if near {
                Terrain::Plains
            } else {
                Terrain::Ocean
            });
        }
    }
    v.turn = turn * 2;
    v.turn_limit = None;
    v.players[0].faction_goal_done = held;
    standing(&mut v, vp, target);
    v.faction_goal = Some(FactionGoalView {
        value: with_camp,
        target: 2,
        label: String::new(),
        live: true,
        done: held,
    });
    v.actions.wild = Some(WildView {
        blocker: None,
        sites: vec![
            WildSiteView {
                city: legatum_core::CityId(1),
                at: off(3, 6),
            },
            WildSiteView {
                city: legatum_core::CityId(2),
                at: off(3, 13),
            },
        ],
        with_camp,
        cities: 2,
    });
    v
}

fn calls(v: &PlayerView) -> bool {
    orders_of(v).contains(&Action::CallTheWild)
}

#[test]
fn the_wild_is_called_late_when_the_two_vp_decide_something() {
    // Early and far from the target: never.
    assert!(!calls(&celts(20, 5, 17, 0, false, false)));
    assert!(!calls(&celts(40, 14, 17, 0, false, false)));
    // The 2 VP take the team to the target with time to hold it, and expansion is done.
    assert!(calls(&celts(70, 15, 17, 0, false, false)));
    // Late in the game it is banked whatever the standing.
    assert!(calls(&celts(110, 5, 17, 0, false, false)));
    assert!(!calls(&celts(100, 5, 17, 0, false, false)));
    // Already past the target without it: no use for the camps.
    assert!(!calls(&celts(70, 17, 17, 0, false, false)));
}

#[test]
fn the_wild_waits_for_expansion_to_finish() {
    // Room to settle is left: the call waits until the game is far enough along.
    assert!(!calls(&celts(70, 15, 17, 0, false, true)));
    assert!(calls(&celts(90, 15, 17, 0, false, true)));
    // A Settler on the road or in production also waits (even when no site is left).
    let mut v = celts(70, 15, 17, 0, false, false);
    v = ViewBuilder { view: v }
        .unit(60, ME, UnitKind::Settler, off(5, 5))
        .build();
    assert!(!calls(&v));
    v.turn = 180;
    assert!(calls(&v), "past 55% of the game it calls whatever is left");
}

#[test]
fn the_wild_is_recalled_when_a_camp_is_cleared_and_never_while_every_city_has_one() {
    // Held with a camp in both cities: nothing to do.
    assert!(!calls(&celts(70, 17, 17, 2, true, false)));
    // A rival cleared one: the goal is lost (vp falls by 2) and the window is open again.
    assert!(calls(&celts(75, 15, 17, 1, false, false)));
    // Held but one camp short is also a re-call (a city the Celts just took).
    assert!(calls(&celts(75, 17, 17, 1, true, false)));
    // Not callable (no room for a camp anywhere): nothing is ordered.
    let mut v = celts(75, 15, 17, 0, false, false);
    v.actions.wild.as_mut().unwrap().blocker = Some("no room".into());
    assert!(!calls(&v));
}

#[test]
fn the_wild_waits_until_every_city_has_room_for_its_camp() {
    let mut v = celts(75, 15, 17, 0, false, false);
    assert!(calls(&v));
    // One of the two cities has nowhere for a camp: a call would only add raiders.
    v.actions.wild.as_mut().unwrap().sites.pop();
    assert!(!calls(&v));
}

#[test]
fn sparta_units_train_in_the_agoge_city_while_the_goal_is_open() {
    let v = sparta(60, 5, 17, 0, &[]);
    let p = planner(&v);
    assert_eq!(
        p.agoge_training_bonus(&v.cities[0], UnitKind::Spearman),
        AGOGE_TRAINING_BONUS
    );
    assert_eq!(
        p.agoge_training_bonus(&v.cities[1], UnitKind::Spearman),
        0,
        "no shrine there"
    );
    assert_eq!(p.agoge_training_bonus(&v.cities[0], UnitKind::Settler), 0);
    let mut done = v.clone();
    done.actions.retire = None;
    assert_eq!(
        planner(&done).agoge_training_bonus(&done.cities[0], UnitKind::Spearman),
        0
    );
}

#[test]
fn the_celts_ai_orders_are_valid_with_the_call() {
    let v = celts(70, 15, 17, 0, false, false);
    let out = ScriptedBrain::new(Faction::Celts, 7, ME).orders(&v);
    assert_valid(&v, &out);
    assert!(out.contains(&Order::Action {
        action: Action::CallTheWild
    }));
}

#[test]
fn a_celts_that_wants_its_wild_founds_no_new_city_once_camps_stand() {
    // Window open, camps standing: no settling (a new city breaks the goal).
    let v = celts(70, 15, 17, 2, false, false);
    assert!(planner_off(&v).wild_wanted());
    assert!(!planner_off(&v).wild_allows_settling());
    // Before the call (no camp yet) it may keep expanding; after the window closes too.
    let before = celts(70, 15, 17, 0, false, false);
    assert!(planner_off(&before).wild_allows_settling());
    let early = celts(20, 5, 17, 2, false, false);
    assert!(!planner_off(&early).wild_wanted());
    assert!(planner_off(&early).wild_allows_settling());
    // Other factions are never held back.
    let mut other = celts(70, 15, 17, 2, false, false);
    other.players[0].faction = Faction::Germany;
    assert!(planner_off(&other).wild_allows_settling());
}

/// The Celts's own view with a free army unit near each camp and camps in its territory.
fn celts_with_camps(camps: &[(i32, i32)], wanted: bool) -> PlayerView {
    let mut v = if wanted {
        celts(70, 15, 17, 2, false, true)
    } else {
        celts(20, 5, 17, 2, false, true)
    };
    for h in off(3, 3).within(3).into_iter().chain(off(3, 10).within(3)) {
        if let Some(i) = v.index(h) {
            v.tiles[i].owner = Some(ME);
        }
    }
    for (city, centre) in [(0, off(3, 3)), (1, off(3, 10))] {
        v.cities[city].detail.as_mut().unwrap().territory = centre.within(3);
    }
    v = ViewBuilder { view: v }
        .unit(70, ME, UnitKind::Spearman, off(4, 4))
        .unit(71, ME, UnitKind::Archer, off(4, 9))
        .build();
    v.barbarian_camps = camps
        .iter()
        .map(|(c, r)| legatum_core::view::BarbarianCampView {
            pos: off(*c, *r),
            visible: true,
        })
        .collect();
    v
}

fn heads_for(p: &Planner<'_>, camp: Hex) -> bool {
    p.orders.iter().any(|o| {
        matches!(o, Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }
            if to.distance(camp) <= 2)
    })
}

#[test]
fn a_celts_keeps_one_camp_per_city_while_it_wants_the_wild_and_clears_the_doubles() {
    // Wanted, one camp in city 1's land: it stays alone.
    let v = celts_with_camps(&[(3, 6)], true);
    let mut p = planner_off(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    assert!(!heads_for(&p, off(3, 6)), "{:?}", p.orders);
    // Two camps in the same city: one is cleared, the other kept.
    let v = celts_with_camps(&[(3, 6), (4, 6)], true);
    let mut p = planner_off(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    let sent = p
        .orders
        .iter()
        .filter(|o| matches!(o, Order::MoveUnit { .. } | Order::MoveUnitPath { .. }))
        .count();
    assert_eq!(
        sent, 1,
        "one camp is cleared, the other kept: {:?}",
        p.orders
    );
    // A camp counts for the city whose territory holds it, not for the nearest city: both camps
    // are nearest city 1, but one stands in city 2's territory, so each city has exactly one and
    // neither is cleared (the nearest-city count cleared one and left city 2 without a camp).
    let mut v = celts_with_camps(&[(3, 6), (4, 6)], true);
    v.cities[1]
        .detail
        .as_mut()
        .unwrap()
        .territory
        .push(off(3, 6));
    v.cities[0]
        .detail
        .as_mut()
        .unwrap()
        .territory
        .retain(|h| *h != off(3, 6));
    let mut p = planner_off(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    let sent = p
        .orders
        .iter()
        .filter(|o| matches!(o, Order::MoveUnit { .. } | Order::MoveUnitPath { .. }))
        .count();
    assert_eq!(sent, 0, "{:?}", p.orders);
    // Not wanted (early): its own camp is just a raid base, cleared like any other.
    let v = celts_with_camps(&[(3, 6)], false);
    let mut p = planner_off(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    assert!(heads_for(&p, off(3, 6)), "{:?}", p.orders);
}

#[test]
fn a_rival_goes_farther_for_the_camp_of_a_celts_that_scores_it() {
    // A camp 10 hexes from our only city (beyond the ordinary range of 8) in a Celts's land.
    let mut v = ViewBuilder::new(40, 20)
        .faction(Faction::Switzerland)
        .city(1, ME, off(3, 3), 6)
        .city(2, ENEMY, off(12, 3), 6)
        .unit(70, ME, UnitKind::Spearman, off(7, 3))
        .build();
    v.players[2].faction = Faction::Celts;
    for h in off(12, 3).within(3) {
        if let Some(i) = v.index(h) {
            v.tiles[i].owner = Some(ENEMY);
        }
    }
    v.barbarian_camps = vec![legatum_core::view::BarbarianCampView {
        pos: off(12, 6),
        visible: true,
    }];
    // Not scoring the goal: out of range, ignored.
    let mut p = planner(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    assert!(p.orders.is_empty(), "{:?}", p.orders);
    // Scoring it (public vp_sources): the camp is worth the trip.
    v.players[2].vp_sources = vec![(legatum_core::VpSource::FactionGoal { of: ENEMY }, 2)];
    assert!(planner(&v).wild_camp_denial(off(12, 6)));
    let mut p = planner(&v);
    let army: Vec<_> = p
        .ctx
        .own_units
        .iter()
        .copied()
        .filter(|u| u.military)
        .collect();
    p.clear_barbarian_camps(&army);
    assert!(!p.orders.is_empty(), "denial widens the range");
    // A teammate's camp is never denied.
    let mut ally = v.clone();
    ally.players[2].team = TeamId(0);
    assert!(!planner(&ally).wild_camp_denial(off(12, 6)));
}

// ---------- Celts playbook (gene `celts_playbook`) ----------

fn calls_with(v: &PlayerView, playbook: i32) -> bool {
    let mut orders = Vec::new();
    let d = crate::DecisionParams {
        celts_playbook: playbook,
        ..Default::default()
    };
    crate::goal_actions::with_goal_actions(v, &mut orders, true, &d);
    orders.contains(&Order::Action {
        action: Action::CallTheWild,
    })
}

/// `celts` with the core's real preview: a call lists sites only for the cities without a camp.
fn celts_real(turn: u32, vp: u32, target: u32, with_camp: i32, held: bool) -> PlayerView {
    let mut v = celts(turn, vp, target, with_camp, held, false);
    let w = v.actions.wild.as_mut().unwrap();
    w.sites.truncate((2 - with_camp).max(0) as usize);
    v
}

fn with_rival(mut v: PlayerView, rival_vp: u32) -> PlayerView {
    let mut s = v.standings.to_vec();
    s.push(TeamStanding {
        team: TeamId(1),
        members: vec![ENEMY],
        vp: rival_vp,
        target: s[0].target,
        victory_hold: None,
    });
    v.standings = s.into();
    v
}

#[test]
fn the_playbook_recalls_the_wild_after_a_cleared_camp_with_the_real_preview() {
    let wild = crate::celts_playbook::WILD;
    // One camp cleared: the core previews a camp for that city alone. The old test asked for a
    // site in every city and never re-called; the playbook re-calls.
    let v = celts_real(75, 15, 17, 1, false);
    assert!(!calls_with(&v, 0), "the old rule never re-calls");
    assert!(calls_with(&v, wild));
    // Every city holds a camp: nothing to call.
    assert!(!calls_with(&celts_real(75, 17, 17, 2, true), wild));
    // A city with no room for its camp still waits.
    let mut short = celts_real(75, 15, 17, 1, false);
    short.actions.wild.as_mut().unwrap().sites.clear();
    assert!(!calls_with(&short, wild));
}

#[test]
fn the_playbook_calls_one_vp_short_and_holds_the_trigger_before() {
    let wild = crate::celts_playbook::WILD;
    // 14 + 2 leaves one VP to 17: the designer's 7-of-10 call. The old rule waits.
    let short = with_rival(celts(40, 14, 17, 0, false, false), 15);
    assert!(calls_with(&short, wild));
    assert!(!calls_with(&short, 0));
    // Far from the target, behind or ahead: hold the trigger (an early call cost the AI Celts
    // 4.3 points of win share in 256 paired games: the camps slowed their expansion).
    let behind = with_rival(celts(20, 3, 17, 0, false, false), 6);
    assert!(!calls_with(&behind, wild));
    let ahead = with_rival(celts(20, 5, 17, 0, false, false), 4);
    assert!(!calls_with(&ahead, wild));
    // Other factions never call.
    let mut other = short.clone();
    other.players[0].faction = Faction::Germany;
    assert!(!calls_with(&other, wild));
}

#[test]
fn the_playbook_keeps_settling_after_the_call_unless_the_goal_decides_the_hold() {
    let wild = crate::celts_playbook::WILD;
    let v = celts(70, 15, 17, 2, false, false);
    let mut p = planner(&v);
    p.decisions.celts_playbook = wild;
    assert!(p.wild_allows_settling(), "the next call covers a new city");
    // Held at the target: a new city would lapse the goal and break the hold.
    let at_target = celts(70, 17, 17, 2, true, false);
    let mut p = planner(&at_target);
    p.decisions.celts_playbook = wild;
    assert!(!p.wild_allows_settling());
}

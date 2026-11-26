use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, planner};
use legatum_core::diplomacy::DiplomaticAction;
use legatum_core::view::PlayerView;
use legatum_core::{CityId, Hex, UnitId, UnitKind};

const HOME: Hex = Hex::new(10, 8);
const RIVAL: Hex = Hex::new(18, 8);
const OTHER: PlayerId = PlayerId(3);
const FAR: Hex = Hex::new(10, 17);

fn hard(v: &PlayerView, gene: i32) -> Planner<'_> {
    let mut p = planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.rome_opening_march = 1;
    p.decisions.rome_opening_finish = gene;
    p
}

/// Hard Rome at war beside an unwalled rival city: one Spearman per entry of `hp` on the
/// city's ring, and a rival Spearman inside when `garrison`.
fn siege(city_hp: i32, hp: &[i32], garrison: bool) -> PlayerView {
    let mut b = ViewBuilder::new(40, 24)
        .faction(Faction::Rome)
        .city(1, ME, HOME, 3)
        .city(5, ENEMY, RIVAL, 3);
    let ring = RIVAL.ring(1);
    for (i, hp) in hp.iter().enumerate() {
        b = b.unit_hp(i as u32 + 1, ME, UnitKind::Spearman, ring[i], *hp);
    }
    if !(garrison) {
    } else {
        b = b.unit(90, ENEMY, UnitKind::Spearman, RIVAL);
    }
    let mut v = b.build();
    v.turn = 22;
    let city = v.cities.iter_mut().find(|c| c.pos == RIVAL).unwrap();
    city.hp = city_hp;
    city.visible = true;
    assert!(v.at_war(ME, ENEMY));
    v
}

fn assault<'a>(v: &'a PlayerView, gene: i32) -> Planner<'a> {
    let mut p = hard(v, gene);
    p.choose_rome_opening();
    let army: Vec<_> = v.units.iter().filter(|u| u.owner == ME).collect();
    p.rome_finish_assault(&army);
    p
}

fn hits(p: &Planner<'_>, unit: u32) -> bool {
    p.orders.contains(&Order::MoveUnit {
        unit: UnitId(unit),
        to: RIVAL,
    })
}

/// Two rivals at peace that Rome may declare on: `ENEMY` (of `people`) six hexes from Rome,
/// `OTHER` nine hexes away.
fn two_rivals(people: Faction, turn: u32) -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Rome)
        .city(1, ME, HOME, 3)
        .city(5, ENEMY, Hex::new(16, 8), 3)
        .city(6, OTHER, FAR, 3)
        .unit(1, ME, UnitKind::Spearman, Hex::new(11, 8))
        .unit(2, ME, UnitKind::Spearman, Hex::new(11, 9))
        .unit(3, ME, UnitKind::Spearman, Hex::new(10, 9))
        .build();
    v.turn = turn;
    v.diplomacy.legacy = false;
    for team in [v.team_of(ENEMY), v.team_of(OTHER)] {
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: vec![],
            calls: vec![],
            also_agree: vec![],
            order: Order::DeclareWar {
                against: team,
                breach: false,
            },
            effective_turn: turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
    }
    let mut players = (*v.players).clone();
    players.iter_mut().find(|p| p.id == ENEMY).unwrap().faction = people;
    players.iter_mut().find(|p| p.id == OTHER).unwrap().faction = Faction::Celts;
    v.players = players.into();
    assert!(!v.at_war(ME, ENEMY) && !v.at_war(ME, OTHER));
    v
}

fn chosen(v: &PlayerView, gene: i32) -> Option<CityId> {
    let mut p = hard(v, gene);
    p.choose_rome_opening();
    p.campaigns.rome_opening_target
}

#[test]
fn the_first_victim_is_the_softest_rival_not_the_nearest() {
    // Egypt next door, the Celts farther off: the gene marches past Egypt.
    let v = two_rivals(Faction::Egypt, 19);
    assert_eq!(chosen(&v, 0), Some(CityId(5)));
    assert_eq!(chosen(&v, 1), Some(CityId(6)));
    // An ordinary neighbour that close is the victim.
    let v = two_rivals(Faction::Athens, 19);
    assert_eq!(chosen(&v, 1), Some(CityId(5)));
    // Walls under way and a second known city each cost the near rival its place.
    let mut building = two_rivals(Faction::Athens, 19);
    building
        .cities
        .iter_mut()
        .find(|c| c.id == CityId(5))
        .unwrap()
        .walls_building = true;
    assert_eq!(chosen(&building, 1), Some(CityId(6)));
}

#[test]
fn a_stubborn_people_alone_is_no_victim_before_turn_25() {
    let mut v = two_rivals(Faction::Sparta, 19);
    v.cities.retain(|c| c.id != CityId(6));
    assert_eq!(chosen(&v, 0), Some(CityId(5)));
    assert_eq!(chosen(&v, 1), None);
    v.turn = 25;
    assert_eq!(chosen(&v, 1), Some(CityId(5)));
}

#[test]
fn every_fit_unit_beside_the_open_city_hits_it_and_the_weak_one_holds() {
    let v = siege(200, &[100, 70, 15], true);
    let p = assault(&v, 1);
    assert!(hits(&p, 1) && hits(&p, 2), "{:?}", p.orders);
    assert!(!hits(&p, 3) && p.has_acted(UnitId(3)), "{:?}", p.orders);
    let off = assault(&v, 0);
    assert!(
        off.orders
            .iter()
            .all(|o| !matches!(o, Order::MoveUnit { .. })),
        "{:?}",
        off.orders
    );
}

#[test]
fn a_lone_hit_on_a_fallen_citys_garrison_waits_and_two_go_in() {
    let lone = siege(0, &[100], true);
    let p = assault(&lone, 1);
    assert!(!hits(&p, 1) && p.has_acted(UnitId(1)), "{:?}", p.orders);
    let pair = siege(0, &[100, 100], true);
    let p = assault(&pair, 1);
    assert!(hits(&p, 1) && hits(&p, 2), "{:?}", p.orders);
    // An empty fallen city is entered at once.
    let empty = siege(0, &[100], false);
    let p = assault(&empty, 1);
    assert!(hits(&p, 1), "{:?}", p.orders);
}

#[test]
fn the_war_names_the_victim_and_the_opening_runs_until_it_is_gone() {
    let mut v = siege(200, &[100, 100, 100], true);
    let mut p = hard(&v, 1);
    p.choose_rome_opening();
    assert_eq!(p.campaigns.rome_victim, None, "named on the next review");
    let memory = p.campaigns.clone();
    v.turn = 45;
    let mut p = hard(&v, 1);
    p.campaigns = memory.clone();
    p.choose_rome_opening();
    assert_eq!(p.campaigns.rome_victim, Some(ENEMY));
    assert!(
        p.rome_opening_active(),
        "past turn 40 while the victim lives"
    );
    assert!(p.rome_finish_refuses_peace(v.team_of(ENEMY)));
    assert!(p.rome_finish_keeps(Some(ENEMY)) && !p.rome_finish_keeps(Some(OTHER)));
    assert_eq!(p.rome_opening_melee_goal(), 5);
    let memory = p.campaigns.clone();
    // The gene off keeps the turn-40 end and makes no such promises.
    let mut off = hard(&v, 0);
    off.campaigns = memory.clone();
    assert!(!off.rome_opening_active() && !off.rome_finish_refuses_peace(v.team_of(ENEMY)));
    // The victim eliminated: the opening is over and Peace logic is the ordinary one.
    let mut players = (*v.players).clone();
    players.iter_mut().find(|p| p.id == ENEMY).unwrap().alive = false;
    v.players = players.into();
    v.turn = 38;
    let mut p = hard(&v, 1);
    p.campaigns = memory;
    p.choose_rome_opening();
    assert_eq!(p.campaigns.rome_opening_target, None);
    assert!(!p.rome_opening_active() && !p.rome_finish_refuses_peace(v.team_of(ENEMY)));
}

fn walled_victim_after_capture() -> PlayerView {
    let mut v = siege(200, &[100, 100, 100, 100], false);
    v.turn = 40;
    let mut captured = v
        .cities
        .iter()
        .find(|city| city.owner == ME)
        .unwrap()
        .clone();
    captured.id = CityId(9);
    captured.pos = Hex::new(14, 8);
    captured.occupation_lock = Some(ENEMY);
    v.cities.push(captured);
    v.cities
        .iter_mut()
        .find(|city| city.owner == ENEMY)
        .unwrap()
        .walls = true;
    v
}

#[test]
fn walled_peace_requires_a_capture_and_observed_walls() {
    let mut v = walled_victim_after_capture();
    let mut p = hard(&v, 1);
    p.campaigns.rome_victim = Some(ENEMY);
    p.decisions.rome_walled_peace = 0;
    let enemy_team = v.team_of(ENEMY);
    assert!(p.rome_finish_refuses_peace(enemy_team));
    p.decisions.rome_walled_peace = 1;
    assert!(p.rome_walled_peace(enemy_team));
    assert!(!p.rome_finish_refuses_peace(enemy_team));
    assert_eq!(p.wants_peace_why(enemy_team), (true, "Roman victim walled"));
    assert_eq!(p.peace_diagnosis(enemy_team).score, Some(0));
    assert_eq!(p.peace_diagnosis(enemy_team).veto, None);

    v.cities
        .iter_mut()
        .find(|city| city.owner == ENEMY)
        .unwrap()
        .walls = false;
    let mut open = hard(&v, 1);
    open.decisions.rome_walled_peace = 1;
    open.campaigns.rome_victim = Some(ENEMY);
    assert!(!open.rome_walled_peace(enemy_team));
    v.cities
        .iter_mut()
        .find(|city| city.id == CityId(9))
        .unwrap()
        .occupation_lock = None;
    v.cities
        .iter_mut()
        .find(|city| city.owner == ENEMY)
        .unwrap()
        .walls = true;
    let mut no_capture = hard(&v, 1);
    no_capture.decisions.rome_walled_peace = 1;
    no_capture.campaigns.rome_victim = Some(ENEMY);
    assert!(!no_capture.rome_walled_peace(enemy_team));
}

#[test]
fn only_walled_peace_opens_the_second_war_early() {
    let v = walled_victim_after_capture();
    let mut p = hard(&v, 1);
    p.decisions.rome_second_war = 1;
    p.campaigns.rome_victim = Some(ENEMY);
    assert!(!p.rome_second_on());
    p.decisions.rome_walled_peace = 1;
    p.choose_rome_second_war();
    assert!(p.campaigns.rome_walled_peace_window);
    assert!(p.rome_second_on());
}

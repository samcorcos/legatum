use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, planner};
use legatum_core::view::{GoalCityView, PlayerView};
use legatum_core::{AiDifficulty, CityId, Order, UnitId};

const HOME: Hex = Hex::new(10, 8);
const RIVAL: Hex = Hex::new(20, 8);

/// A Hard Rome with its capital at `HOME`, two home guards, and a rival city at `RIVAL` whose
/// land has grown to ring 2, so the legal sites four hexes from it are Colonia sites.
fn rome() -> PlayerView {
    let mut b = ViewBuilder::new(32, 16)
        .faction(Faction::Rome)
        .city(1, ME, HOME, 4)
        .city(5, ENEMY, RIVAL, 6)
        .unit(2, ME, UnitKind::Spearman, HOME)
        .unit(3, ME, UnitKind::Spearman, HOME.neighbors()[0]);
    for h in RIVAL.within(2) {
        b = b.owner(h, ENEMY);
    }
    let mut v = b.build();
    v.players[0].faction_goal_done = false;
    v
}

fn veteran(v: PlayerView, id: u32, pos: Hex, level: u8, xp: u32) -> PlayerView {
    let mut v = ViewBuilder { view: v }
        .unit(id, ME, UnitKind::Spearman, pos)
        .build();
    let u = v.units.iter_mut().find(|u| u.id == UnitId(id)).unwrap();
    u.level = level;
    u.xp = Some(xp);
    v
}

fn hard(v: &PlayerView) -> Planner<'_> {
    let mut p = planner(v, 7);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.roman_founders = 1;
    p.decisions.roman_soldier_route = 1;
    p.decisions.rome_colonia = 1;
    p
}

fn founds(p: &Planner<'_>, unit: u32) -> bool {
    p.orders.contains(&Order::FoundCity { unit: UnitId(unit) })
}

/// The first Colonia site the fixture offers: four hexes from the rival, so two from its land.
fn colonia_site(v: &PlayerView) -> Hex {
    let site = Hex::new(16, 8);
    assert_eq!(site.distance(RIVAL), 4);
    assert!(goals::known_colonia_site(v, site));
    assert!(legatum_core::settlement::known_legal_site(v, site));
    site
}

#[test]
fn a_level_three_veteran_founds_the_colonia_where_it_stands() {
    let v = rome();
    let site = colonia_site(&v);
    let v = veteran(v, 1, site, 3, 60);
    let mut p = hard(&v);
    assert!(p.colonia_on());
    p.plan_settlers();
    assert!(founds(&p, 1), "{:?}", p.orders);
    assert_eq!(p.colonia_founder, Some(UnitId(1)));
}

#[test]
fn a_level_three_veteran_walks_past_an_ordinary_site_to_the_colonia() {
    // It stands on a legal site far from the rival's land: an ordinary city there would spend
    // the Colonia's founder.
    let v = rome();
    let ordinary = Hex::new(10, 13);
    assert!(legatum_core::settlement::known_legal_site(&v, ordinary));
    assert!(!goals::known_colonia_site(&v, ordinary));
    let v = veteran(v, 1, ordinary, 3, 60);
    let mut p = hard(&v);
    p.plan_settlers();
    assert!(!founds(&p, 1), "{:?}", p.orders);
    let target = p.settle_targets.first().copied().expect("a Colonia site");
    assert!(goals::known_colonia_site(&v, target), "{target:?}");
    assert!(
        p.orders
            .iter()
            .any(|o| legatum_core::rules::units::order_unit(o) == Some(UnitId(1))),
        "{:?}",
        p.orders
    );
    // Gene off: the old founder pass founds where it stands.
    let mut off_gene = hard(&v);
    off_gene.decisions.rome_colonia = 0;
    off_gene.plan_settlers();
    assert!(founds(&off_gene, 1), "{:?}", off_gene.orders);
}

#[test]
fn with_no_colonia_site_the_founder_waits_instead_of_founding_an_ordinary_city() {
    // The rival's land reaches only ring 1: no legal site lies within two tiles of it.
    let mut v = ViewBuilder::new(32, 16)
        .faction(Faction::Rome)
        .city(1, ME, HOME, 4)
        .city(5, ENEMY, RIVAL, 6)
        .unit(2, ME, UnitKind::Spearman, HOME)
        .unit(3, ME, UnitKind::Spearman, HOME.neighbors()[0])
        .build();
    v.players[0].faction_goal_done = false;
    let ordinary = Hex::new(10, 13);
    let v = veteran(v, 1, ordinary, 3, 60);
    let mut p = hard(&v);
    assert!(p.colonia_site(&v.units[2]).is_none());
    assert!(p.colonia_reserved(&v.units[2]));
    p.plan_settlers();
    assert!(!founds(&p, 1), "{:?}", p.orders);
}

#[test]
fn the_most_experienced_level_two_is_kept_until_a_level_three_exists() {
    let v = rome();
    let v = veteran(v, 1, Hex::new(10, 12), 2, 40);
    let v = veteran(v, 4, Hex::new(11, 12), 2, 50);
    let v = veteran(v, 6, Hex::new(12, 12), 1, 20);
    let p = hard(&v);
    let unit = |id: u32| v.units.iter().find(|u| u.id == UnitId(id)).unwrap();
    assert!(!p.colonia_reserved(unit(1)));
    assert!(p.colonia_reserved(unit(4)), "the closest to level 3");
    assert!(
        !p.colonia_reserved(unit(6)),
        "lower veterans still found cities"
    );
    // With only level-1 veterans, the best is kept once Rome has three cities.
    let ones = veteran(
        veteran(rome(), 1, Hex::new(10, 12), 1, 30),
        4,
        Hex::new(11, 12),
        1,
        20,
    );
    let unit = |id: u32| ones.units.iter().find(|u| u.id == UnitId(id)).unwrap();
    assert!(
        !hard(&ones).colonia_reserved(unit(1)),
        "one city: expansion first"
    );
    let three = ViewBuilder { view: ones.clone() }
        .city(2, ME, Hex::new(4, 3), 2)
        .city(3, ME, Hex::new(4, 13), 2)
        .build();
    let p = hard(&three);
    assert!(p.colonia_reserved(unit(1)));
    assert!(!p.colonia_reserved(unit(4)));
    // Once a level 3 exists, it is the founder and the level 2s found as before.
    let v = veteran(v, 7, Hex::new(9, 12), 3, 60);
    let p = hard(&v);
    let unit = |id: u32| v.units.iter().find(|u| u.id == UnitId(id)).unwrap();
    assert!(p.colonia_reserved(unit(7)));
    assert!(!p.colonia_reserved(unit(4)));
}

#[test]
fn a_reserved_level_two_does_not_found_but_a_lesser_veteran_does() {
    let v = rome();
    let site = Hex::new(10, 13);
    let v = veteran(v, 4, site, 2, 50);
    let mut p = hard(&v);
    p.plan_settlers();
    assert!(!founds(&p, 4), "{:?}", p.orders);
    let v = rome();
    let v = veteran(v, 4, site, 1, 20);
    let mut p = hard(&v);
    p.plan_settlers();
    assert!(founds(&p, 4), "{:?}", p.orders);
}

#[test]
fn a_site_under_a_rival_army_is_riskier_than_one_beside_our_own() {
    let v = rome();
    let site = colonia_site(&v);
    let v = veteran(v, 1, Hex::new(15, 8), 3, 60);
    let quiet = hard(&v).colonia_hold_risk(site, UnitId(1));
    // A rival army four or five hexes out raises the risk; our own army beside the site lowers it.
    let mut threatened = v.clone();
    for (i, h) in [Hex::new(19, 9), Hex::new(19, 10), Hex::new(20, 9)]
        .into_iter()
        .enumerate()
    {
        threatened = ViewBuilder { view: threatened }
            .unit(50 + i as u32, ENEMY, UnitKind::Spearman, h)
            .build();
    }
    let risky = hard(&threatened).colonia_hold_risk(site, UnitId(1));
    assert!(risky > quiet, "{risky} > {quiet}");
    let mut supported = threatened.clone();
    for (i, h) in [Hex::new(15, 7), Hex::new(16, 7), Hex::new(17, 9)]
        .into_iter()
        .enumerate()
    {
        supported = ViewBuilder { view: supported }
            .unit(60 + i as u32, ME, UnitKind::Spearman, h)
            .build();
    }
    let held = hard(&supported).colonia_hold_risk(site, UnitId(1));
    assert!(held < risky, "{held} < {risky}");
    // A big enough rival army rules every site out: the veteran waits.
    let mut massed = v.clone();
    for (i, h) in RIVAL
        .within(3)
        .into_iter()
        .filter(|h| h.distance(RIVAL) == 3)
        .enumerate()
    {
        massed = ViewBuilder { view: massed }
            .unit(70 + i as u32, ENEMY, UnitKind::Spearman, h)
            .build();
    }
    assert!(hard(&massed).colonia_site(&massed.units[2]).is_none());
}

#[test]
fn a_standing_colonia_closes_the_goal() {
    let mut v = veteran(rome(), 1, Hex::new(16, 8), 3, 60);
    assert!(hard(&v).colonia_on());
    v.colonias = vec![GoalCityView {
        of: ME,
        holder: ENEMY,
        city: CityId(9),
        at: Hex::new(26, 2),
        name: "Colonia".into(),
    }];
    let p = hard(&v);
    assert!(!p.colonia_on());
    assert!(!p.colonia_reserved(&v.units[2]));
}

#[test]
fn other_factions_and_lower_difficulties_never_reserve() {
    let v = veteran(rome(), 1, Hex::new(16, 8), 3, 60);
    let mut p = hard(&v);
    p.difficulty = AiDifficulty::Normal;
    assert!(!p.colonia_on());
    let mut v = v.clone();
    v.players[0].faction = Faction::Sparta;
    assert!(!hard(&v).colonia_on());
}

#[test]
fn a_colonia_site_with_border_risk_is_still_a_site() {
    // Beyond the rival's land the map is out of sight, so an unseen rival city could stand
    // there: the settler search flags the sites beside that border (`border_risk`) and drops
    // them. Those are most of the Colonia sites; in seed 9000 game 118 the level-3 veteran
    // saw none.
    let mut v = rome();
    for (i, t) in v.tiles.iter_mut().enumerate() {
        let h = Hex::from_offset(i as i32 % 32, i as i32 / 32);
        t.visible = h.distance(RIVAL) <= 2 || h.distance(HOME) <= 3;
    }
    let site = colonia_site(&v);
    let risks = legatum_core::settlement::border_city_risks(&v);
    assert!(risks[v.index(site).unwrap()], "the fixture flags the site");
    let v = veteran(v, 1, site, 3, 60);
    let mut p = hard(&v);
    assert!(p.colonia_site(&v.units[2]).is_some());
    p.plan_settlers();
    assert!(founds(&p, 1), "{:?}", p.orders);
}

#[test]
fn a_wounded_level_three_veteran_rests_for_the_colonia() {
    // Founding needs full health. The war passes kept spending a wounded veteran (seed 9000
    // game 118: 86 to 96 HP for eight turns, moving every turn and so never healing); the
    // Colonia claims it first and it rests.
    let v = rome();
    let mut v = veteran(v, 1, Hex::new(10, 12), 3, 60);
    v.units[2].hp = 60;
    let mut p = hard(&v);
    p.plan_settlers();
    assert_eq!(p.colonia_founder, Some(UnitId(1)));
    assert!(
        p.orders.contains(&Order::SkipUnit { unit: UnitId(1) }),
        "{:?}",
        p.orders
    );
    // In enemy reach it is left to the war passes, which know how to retreat it.
    let exposed = ViewBuilder { view: v.clone() }
        .unit(50, ENEMY, UnitKind::Spearman, Hex::new(10, 14))
        .build();
    let mut p = hard(&exposed);
    assert!(p.exposure_at(Hex::new(10, 12)) > 0);
    p.plan_settlers();
    assert_ne!(p.colonia_founder, Some(UnitId(1)));
}

#[test]
fn a_veteran_locked_in_a_captured_city_is_not_the_founder() {
    // The occupation lock refuses its move (seed 9000 game 95: the level-3 veteran took
    // Cologne and was held there; the founder's walk was dropped every turn).
    let v = rome();
    let mut v = veteran(v, 1, Hex::new(10, 12), 3, 60);
    v.units[2].occupation_lock = Some(ENEMY);
    let mut p = hard(&v);
    p.plan_settlers();
    assert_eq!(p.colonia_founder, None);
    assert!(
        !p.orders
            .iter()
            .any(|o| legatum_core::rules::units::order_unit(o) == Some(UnitId(1))),
        "{:?}",
        p.orders
    );
}

/// `rome()` with the rival city capturable into an occupation lock and a level-3 veteran
/// beside it.
fn lock_city() -> PlayerView {
    let mut v = veteran(rome(), 1, RIVAL.neighbors()[0], 3, 60);
    v.cities
        .iter_mut()
        .find(|c| c.pos == RIVAL)
        .unwrap()
        .occupation_lock = Some(ENEMY);
    v
}

fn moves_to(p: &Planner<'_>, unit: u32, to: Hex) -> bool {
    p.orders.iter().any(|o| {
        matches!(o, Order::MoveUnit { unit: u, to: t } | Order::MoveUnitPath { unit: u, to: t, .. }
            if *u == UnitId(unit) && *t == to)
    })
}

#[test]
fn another_unit_takes_the_capture_that_would_lock_the_founder() {
    // Seed 9000 game 95: the level-3 veteran took Cologne and was locked there until Peace.
    let v = ViewBuilder { view: lock_city() }
        .unit(10, ME, UnitKind::Spearman, RIVAL.neighbors()[1])
        .build();
    let mut p = hard(&v);
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    assert!(moves_to(&p, 10, RIVAL), "{:?}", p.orders);
    assert!(!moves_to(&p, 1, RIVAL));
    assert!(
        !p.has_acted(UnitId(1)),
        "the founder stays free to fight in the field"
    );
    // A later pass sending it into the same city again does not walk it in behind unit 10.
    assert!(!p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    assert!(!moves_to(&p, 1, RIVAL));
    // The Colonia's own walk (here a retreat home) never ends there, and hands no one else
    // the founder's move (seed 9000 game 301: the walk home became another unit's march).
    let mut p = hard(&v);
    p.colonia_founder = Some(UnitId(1));
    assert!(!p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    assert!(p.orders.is_empty(), "{:?}", p.orders);
    // A move that locks nobody is untouched.
    let mut p = hard(&v);
    let field = RIVAL.neighbors()[0].neighbors()[0];
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: field
        }
    ));
    assert!(moves_to(&p, 1, field));
}

#[test]
fn the_capture_waits_a_turn_for_a_unit_two_turns_out_but_not_for_none() {
    let v = lock_city();
    // A soldier two turns away: the capture waits rather than lock the founder.
    let far = Hex::new(17, 8);
    assert_eq!(far.distance(RIVAL), 3);
    let waiting = ViewBuilder { view: v.clone() }
        .unit(10, ME, UnitKind::Spearman, far)
        .build();
    let mut p = hard(&waiting);
    assert!(
        far.distance(RIVAL)
            > p.ctx
                .own_units
                .iter()
                .find(|u| u.id == UnitId(10))
                .unwrap()
                .max_moves
    );
    assert!(!p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    assert!(p.orders.is_empty(), "{:?}", p.orders);
    assert_eq!(p.campaigns.colonia_waits, vec![(RIVAL, waiting.turn)]);
    // It waits one turn only: a capture that waited last turn goes ahead.
    let mut later = waiting.clone();
    later.turn += 1;
    let mut p = hard(&later);
    p.campaigns.colonia_waits = vec![(RIVAL, waiting.turn)];
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    // Nobody else near: Rome takes the city with the founder rather than not at all.
    let mut p = hard(&v);
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
    assert!(moves_to(&p, 1, RIVAL));
    // Gene off: the old behaviour.
    let mut p = hard(&waiting);
    p.decisions.rome_colonia = 0;
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: RIVAL
        }
    ));
}

#[test]
fn the_founder_does_not_garrison_an_occupied_city_of_ours_when_another_can() {
    // An own city still held down against its former owner locks whoever enters it.
    let held = Hex::new(14, 4);
    let mut v = ViewBuilder {
        view: veteran(rome(), 1, Hex::new(14, 5), 3, 60),
    }
    .city(6, ME, held, 2)
    .unit(10, ME, UnitKind::Spearman, Hex::new(13, 4))
    .build();
    v.cities
        .iter_mut()
        .find(|c| c.pos == held)
        .unwrap()
        .occupation_lock = Some(ENEMY);
    let mut p = hard(&v);
    assert!(p.unit_order(
        UnitId(1),
        Order::MoveUnit {
            unit: UnitId(1),
            to: held
        }
    ));
    assert!(moves_to(&p, 10, held), "{:?}", p.orders);
    assert!(!moves_to(&p, 1, held));
}

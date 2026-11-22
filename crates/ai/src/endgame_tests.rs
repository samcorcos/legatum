use crate::fixture::*;
use crate::scripted::Planner;
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::view::TeamStanding;
use legatum_core::*;

const PIKEMAN: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Pikeman,
};
const RELAY: ProductionItem = ProductionItem::Building {
    building: BuildingKind::GreatLibrary,
};
const LAB: ProductionItem = ProductionItem::Building {
    building: BuildingKind::Library,
};

fn set_work_completed(v: &mut PlayerView, completed: bool) {
    v.great_works.retain(|work| work.player != ME);
    if completed {
        v.great_works.push(legatum_core::projects::GreatWorkView {
            player: ME,
            completed: Some(BuildingKind::GreatLibrary),
            building: None,
            city: None,
            progress: 0,
            cost: 0,
            production_per_turn: 0,
            turns: None,
            blocker: None,
        });
    }
}

fn endgame() -> PlayerView {
    let mut b = ViewBuilder::new(24, 16)
        .known_techs(&TechId::ALL)
        .gold(4000)
        .city(1, ME, off(4, 8), 8);
    for id in 1..=16 {
        b = b.unit(
            id,
            ME,
            UnitKind::Pikeman,
            off(3 + id as i32 % 5, 6 + id as i32 / 5),
        );
    }
    let mut v = b.build();
    v.turn = 200;
    v.turn_limit = None;
    v.standings.push(TeamStanding {
        team: v.team,
        members: vec![ME],
        vp: 9,
        target: 10,
        victory_hold: None,
    });
    set_work_completed(&mut v, true);
    let d = v.cities[0].detail.as_mut().unwrap();
    d.options
        .retain(|o| o.item == PIKEMAN || o.item == RELAY || o.item == LAB);
    for opt in &mut d.options {
        opt.turns = 20;
        if opt.item != RELAY {
            opt.buy_cost = Some(320);
        }
    }
    v
}

fn hoard_view(done: bool) -> legatum_core::goals::FactionGoalView {
    legatum_core::goals::FactionGoalView {
        value: 0,
        target: legatum_core::goals::SWISS_HOARD_GOLD,
        label: String::new(),
        live: true,
        done,
    }
}

#[test]
fn a_held_hoard_is_the_ordinary_reserve_and_a_deficit_raises_it() {
    let hoard = legatum_core::goals::SWISS_HOARD_GOLD;
    let mut v = endgame();
    v.players[0].faction_goal_done = true;
    v.faction_goal = Some(hoard_view(true));
    v.economy.gold = hoard + 30;
    v.economy.gold_per_turn.total = 5;
    let reserve = |v: &PlayerView| {
        Planner::new(
            v,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        )
        .ordinary_gold_reserve()
    };
    assert_eq!(reserve(&v), hoard);
    v.economy.gold_per_turn.total = -7;
    assert_eq!(reserve(&v), hoard + 7);
    // Lost, with no plausible finish, the ordinary reserve is no longer the hoard.
    v.players[0].faction_goal_done = false;
    v.faction_goal = Some(hoard_view(false));
    v.standings[0].vp = 0;
    v.economy.gold = 0;
    v.economy.gold_per_turn.total = 0;
    assert!(reserve(&v) < hoard);
}

#[test]
fn hard_limits_routine_upkeep_but_reinforces_visible_threats() {
    let mut builder = ViewBuilder::new(32, 20)
        .gold(0)
        .city(1, ME, off(5, 5), 4)
        .city(2, ME, off(14, 5), 4)
        .city(3, ME, off(23, 5), 4);
    for id in 1..=5 {
        builder = builder.unit(id, ME, UnitKind::Spearman, off(4 + id as i32, 5));
    }
    for threatened in [false, true] {
        let mut view = builder.view.clone();
        if threatened {
            view.units.push(
                ViewBuilder::new(32, 20)
                    .unit(99, ENEMY, UnitKind::Spearman, off(5, 7))
                    .build()
                    .units
                    .remove(0),
            );
        }
        view.turn = 70;
        let spearman = ProductionItem::Unit {
            unit: UnitKind::Spearman,
        };
        for city in &mut view.cities {
            city.detail
                .as_mut()
                .unwrap()
                .options
                .retain(|o| o.item == spearman);
        }
        let orders = brain(&view, 7, AiDifficulty::Hard).orders(&view);
        assert_valid(&view, &orders);
        assert_eq!(
            orders
                .iter()
                .any(|o| matches!(o, Order::SetProduction { item, .. } if *item == spearman)),
            threatened
        );
    }
}

#[test]
fn celts_does_not_buy_a_settler_when_its_city_goal_is_already_met() {
    let mut b = ViewBuilder::new(40, 30)
        .faction(Faction::Celts)
        .known_techs(&TechId::ALL)
        .gold(2000);
    for (id, col, row) in [(1, 4, 4), (2, 14, 4), (3, 4, 14), (4, 14, 14), (5, 30, 14)] {
        b = b.city(id, ME, off(col, row), 8);
    }
    let mut v = b.build();
    v.turn = 400;
    v.turn_limit = None;
    // Routine expansion values a colony on any open land, so close the land: the only
    // remaining reason to buy a Settler would be the (already met) goal.
    let homes: Vec<_> = v.cities.iter().map(|c| c.pos).collect();
    for (col, row) in (0..v.width).flat_map(|c| (0..v.height).map(move |r| (c, r))) {
        let h = off(col, row);
        if homes.iter().all(|c| c.distance(h) > 1) {
            let i = v.index(h).expect("on the map");
            v.tiles[i].owner = Some(ENEMY);
        }
    }
    v.players[0].faction_goal_done = false;
    v.standings = vec![TeamStanding {
        team: v.team,
        members: vec![ME],
        vp: 9,
        target: 10,
        victory_hold: None,
    }]
    .into();
    set_work_completed(&mut v, true);
    let settler = ProductionItem::Unit {
        unit: UnitKind::Settler,
    };
    for c in &mut v.cities {
        let detail = c.detail.as_mut().unwrap();
        detail.options.retain(|o| o.item == settler);
        detail.options[0].turns = 10;
        detail.options[0].buy_cost = Some(160);
    }
    let orders = ScriptedBrain::new(Faction::Celts, 7, ME).orders(&v);
    assert_valid(&v, &orders);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::BuyItem { item, .. } if *item == settler))
    );
}

#[test]
fn winning_cash_goal_survives_harassment_but_does_not_sacrifice_a_falling_city() {
    let mut v = endgame();
    v.standings[0].vp = 8;
    v.players[0].faction_goal_done = false;
    v.economy.gold = 390;
    let d = v.cities[0].detail.as_mut().unwrap();
    d.producing = Some(PIKEMAN);
    d.buy_cost = Some(160);
    // A hostile unit cannot defeat the fortified city's healthy standing army.
    v.units.extend(
        ViewBuilder::new(24, 16)
            .unit(80, ENEMY, UnitKind::Pikeman, off(2, 8))
            .build()
            .units,
    );
    let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. } | Order::UpgradeUnit { .. }))
    );
    v.units.retain(|u| u.owner != ME);
    v.cities[0].hp = 1;
    assert!(
        ScriptedBrain::new(Faction::Switzerland, 7, ME)
            .orders(&v)
            .contains(&Order::BuyItem {
                city: CityId(1),
                item: PIKEMAN
            })
    );
    v.economy.gold = 560;
    let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
    assert!(orders.contains(&Order::BuyItem {
        city: CityId(1),
        item: PIKEMAN
    }));
}

#[test]
fn endgame_cash_goal_is_not_spent_again_on_each_bombardment_turn() {
    let mut v = endgame();
    v.players[0].faction_goal_done = false;
    v.economy.gold_per_turn.total = 40;
    v.units = ViewBuilder::new(24, 16)
        .unit(90, ENEMY, UnitKind::Archer, off(2, 8))
        .build()
        .units;
    // The city can absorb harassment while income banks the goal. An endless stream of
    // emergency purchases previously reset the treasury even at 8 or 9 VP.
    for vp in [7, 8, 9] {
        v.standings[0].vp = vp;
        for gold in [4_960, 4_980, 4_990] {
            v.economy.gold = gold;
            let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
            assert_valid(&v, &orders);
            assert!(
                !orders.iter().any(|o| matches!(o, Order::BuyItem { .. })),
                "must bank the goal at {vp} VP and {gold} gold: {orders:?}"
            );
        }
    }
    // Ranged harassment cannot capture even this city; the reserve still belongs to the goal.
    v.cities[0].hp = 1;
    assert!(
        !ScriptedBrain::new(Faction::Switzerland, 7, ME)
            .orders(&v)
            .iter()
            .any(|o| matches!(o, Order::BuyItem { .. }))
    );
    v.units = ViewBuilder::new(24, 16)
        .unit(90, ENEMY, UnitKind::Pikeman, off(2, 8))
        .build()
        .units;
    v.economy.gold = 560;
    let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
    assert!(orders.contains(&Order::BuyItem {
        city: CityId(1),
        item: PIKEMAN
    }));
}

#[test]
fn relay_uses_safe_city_even_if_the_fastest_city_is_threatened() {
    let mut v = ViewBuilder::new(24, 16)
        .known_techs(&TechId::ALL)
        .city(1, ME, off(3, 8), 8)
        .city(2, ME, off(14, 8), 8)
        .unit(90, ENEMY, UnitKind::Pikeman, off(2, 8))
        .build();
    for c in &mut v.cities {
        let d = c.detail.as_mut().unwrap();
        d.buildings.push(BuildingKind::Library);
        d.options.retain(|o| o.item == RELAY);
        d.yields.total.production = if c.id == CityId(1) { 20 } else { 10 };
    }
    let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(2),
        item: RELAY
    }));
    assert!(!orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: RELAY
    }));
    set_work_completed(&mut v, true);
    assert!(
        !ScriptedBrain::new(Faction::Switzerland, 7, ME)
            .orders(&v)
            .iter()
            .any(|o| matches!(o, Order::SetProduction { item, .. } if *item == RELAY))
    );
}

#[test]
fn late_scout_revisits_memory_and_is_replaced_even_on_an_explored_map() {
    let target = off(17, 8);
    let mut v = ViewBuilder::new(24, 16)
        .city(1, ME, off(3, 8), 3)
        .city(9, ENEMY, target, 2)
        .unit(1, ME, UnitKind::Scout, off(7, 8))
        .build();
    v.turn = 500;
    v.turn_limit = None;
    v.cities[1].visible = false;
    for h in target.within(3) {
        if let Some(i) = v.index(h) {
            v.tiles[i].visible = false;
        }
    }
    let p = Planner::new(
        &v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        7,
    );
    let destination = p
        .choose_frontier(&v.units[0], &[])
        .expect("re-scout destination");
    assert_eq!(destination.distance(target), legatum_core::data::UNIT_SIGHT);
    v.units.clear();
    v.cities[0].detail.as_mut().unwrap().options.retain(|o| {
        o.item
            == (ProductionItem::Unit {
                unit: UnitKind::Scout,
            })
    });
    let orders = ScriptedBrain::new(Faction::Switzerland, 7, ME).orders(&v);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: ProductionItem::Unit {
            unit: UnitKind::Scout
        }
    }));
}

#[test]
fn border_settler_avoids_unseen_city_spacing_even_on_arrival() {
    let site = off(10, 8);
    let hidden = off(13, 8);
    let mut v = ViewBuilder::new(24, 16)
        .city(1, ME, off(4, 8), 3)
        .unit(1, ME, UnitKind::Settler, site)
        .owner(off(12, 8), ENEMY)
        .build();
    let i = v.index(hidden).unwrap();
    v.tiles[i].visible = false;
    for moves_left in [0, v.units[0].max_moves] {
        v.units[0].moves_left = moves_left;
        let mut p = Planner::new(
            &v,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let alternate = p.choose_site(&v.units[0]).expect("safe alternate site");
        assert!(alternate.distance(hidden) > legatum_core::data::MIN_CITY_GAP);
        assert!(p.ctx.site_score[v.index(site).unwrap()].is_none());
        p.plan_settlers();
        assert!(!p.orders.contains(&Order::FoundCity { unit: UnitId(1) }));
        assert!(p.orders.iter().any(|order| matches!(
            order,
            Order::MoveUnit { unit: UnitId(1), to }
                | Order::MoveUnitPath { unit: UnitId(1), to, .. }
                if *to == alternate
        )));
        assert_valid(&v, &p.orders);
    }
}

#[test]
fn athens_finish_the_third_lab_before_more_optional_infrastructure() {
    let mut v = ViewBuilder::new(32, 16)
        .faction(Faction::Athens)
        .known_techs(&TechId::ALL)
        .city(1, ME, off(3, 8), 3)
        .city(2, ME, off(13, 8), 3)
        .city(3, ME, off(23, 8), 3)
        .unit(1, ME, UnitKind::Spearman, off(3, 8))
        .unit(2, ME, UnitKind::Spearman, off(13, 8))
        .unit(3, ME, UnitKind::Spearman, off(23, 8))
        .build();
    v.players[0].faction_goal_done = false;
    for c in &mut v.cities[..2] {
        c.detail
            .as_mut()
            .unwrap()
            .buildings
            .push(BuildingKind::Library);
    }
    let choose = |v: &PlayerView| {
        ScriptedBrain::new(Faction::Athens, 7, ME)
            .orders(v)
            .into_iter()
            .find_map(|o| match o {
                Order::SetProduction {
                    city: CityId(3),
                    item,
                } => Some(item),
                _ => None,
            })
    };
    assert_eq!(
        choose(&v),
        Some(ProductionItem::Building {
            building: BuildingKind::Library
        }),
        "finish an already researched faction goal before optional production"
    );
    v.turn = 200;
    v.standings.push(TeamStanding {
        team: v.team,
        members: vec![ME],
        vp: 8,
        target: 10,
        victory_hold: None,
    });
    assert_eq!(choose(&v), Some(LAB));
    v.research.known.retain(|t| *t != TechId::Philosophy);
    v.research.current = Some(TechId::Philosophy);
    assert_eq!(choose(&v), Some(LAB), "finish the lab alongside research");
    v.players[0].faction_goal_done = true;
    assert_eq!(
        choose(&v),
        Some(ProductionItem::Building {
            building: BuildingKind::Workshop
        })
    );
}

#[test]
fn reserved_home_defenders_allow_a_separate_campaign_but_free_troops_do_not() {
    let target = off(18, 8);
    let mut v = ViewBuilder::new(26, 16)
        .city(1, ME, off(3, 8), 3)
        .city(2, ME, off(12, 8), 3)
        .city(9, ENEMY, target, 2)
        .unit(91, ME, UnitKind::Pikeman, off(3, 8))
        .unit(92, ME, UnitKind::Pikeman, off(4, 8))
        .unit(90, ME, UnitKind::Pikeman, off(12, 8))
        .unit(1, ME, UnitKind::Pikeman, off(11, 7))
        .unit(2, ME, UnitKind::Pikeman, off(12, 7))
        .unit(3, ME, UnitKind::Pikeman, off(13, 7))
        .unit(80, ENEMY, UnitKind::Pikeman, off(1, 8))
        .unit(81, ENEMY, UnitKind::Pikeman, off(2, 7))
        .build();
    let marches = |v: &PlayerView| {
        let mut p = planner(v, 7);
        p.plan_military();
        assert_valid(v, &p.orders);
        p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { to, .. } if to.distance(target) <= 3))
    };
    assert!(marches(&v));
    v.units.retain(|u| u.id != UnitId(91) && u.id != UnitId(92));
    assert!(!marches(&v));
}

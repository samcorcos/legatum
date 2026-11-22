use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder, assert_valid, off};
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::view::{PlayerView, TeamStanding};
use legatum_core::{AiDifficulty, CityId, Hex, Order, PlayerId, TeamId, UnitId, VpSource};

fn is_hall(item: ProductionItem) -> bool {
    matches!(
        item,
        ProductionItem::Wonder {
            building: BuildingKind::CourierHall,
            ..
        }
    )
}

fn hall_item(v: &PlayerView, city: CityId) -> ProductionItem {
    v.cities
        .iter()
        .find(|c| c.id == city)
        .unwrap()
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| is_hall(o.item))
        .unwrap()
        .item
}

fn planner(v: &PlayerView) -> Planner<'_> {
    let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7);
    // These Hall tests predate the capitals victory and compare ordinary cities.
    p.decisions.capitals = 0;
    p
}

fn hard(v: &PlayerView) -> Planner<'_> {
    let mut p = planner(v);
    p.difficulty = AiDifficulty::Hard;
    p.decisions.peace_exposure_percent = 100;
    p.decisions.threat_horizon_turns = 2;
    p
}

fn brain(v: &PlayerView) -> Vec<Order> {
    let out = ScriptedBrain::with_difficulty(v.faction_of(ME), 7, ME, AiDifficulty::Hard).orders(v);
    assert_valid(v, &out);
    out
}

fn standings(vp: u32, target: u32) -> Vec<TeamStanding> {
    vec![TeamStanding {
        team: TeamId(0),
        members: vec![ME, PlayerId(1)],
        vp,
        target,
        victory_hold: None,
    }]
}

/// A Mongols with three cities on open plains, each garrisoned twice, a far-off rival, and a
/// Hall that takes `hall_turns` to build wherever it can start. Turn `turn` of the game.
fn mongols(turn: u32, vp: u32, target: u32) -> PlayerView {
    let mut b = ViewBuilder::new(48, 28)
        .faction(Faction::Mongols)
        .city(1, ME, off(4, 4), 4)
        .city(2, ME, off(12, 4), 4)
        .city(3, ME, off(4, 12), 4)
        .city(9, ENEMY, off(40, 22), 4);
    for (i, (c, r)) in [(4, 4), (12, 4), (4, 12)].into_iter().enumerate() {
        b = b
            .unit(10 + i as u32, ME, UnitKind::Soldier, off(c, r))
            .unit(20 + i as u32, ME, UnitKind::Soldier, off(c + 1, r));
    }
    let mut v = b.build();
    v.turn = turn * 2;
    v.players[0].faction_goal_done = false;
    v.standings = standings(vp, target).into();
    for c in &mut v.cities {
        if let Some(d) = c.detail.as_mut() {
            for o in &mut d.options {
                if o.item
                    == (ProductionItem::Building {
                        building: BuildingKind::CourierHall,
                    })
                {
                    o.item = ProductionItem::Wonder {
                        building: BuildingKind::CourierHall,
                        tile: c.pos.neighbors()[0],
                    };
                    o.wonder = Some(legatum_core::view::WonderPreview {
                        tile: c.pos.neighbors()[0],
                        lost: legatum_core::Yields::new(0, 2, 0, 0),
                    });
                }
                if is_hall(o.item) {
                    o.turns = 16;
                }
            }
        }
    }
    v
}

fn set_hall(v: &mut PlayerView, of: PlayerId, holder: PlayerId, city: u32) {
    v.courier_halls = vec![legatum_core::view::CourierHallView {
        tile: None,
        of,
        holder,
        city: CityId(city),
    }];
}

// ---------- build ----------

#[test]
fn the_hall_waits_for_expansion_to_finish() {
    // Early, with land to settle: expansion is the core loop, so no Hall.
    let early = mongols(30, 14, 16);
    assert!(planner(&early).expansion_room());
    assert!(!planner(&early).hall_wanted());
    // Nearly half the game gone: expansion is over whatever land is left.
    let mid = mongols(70, 14, 16);
    assert!(planner(&mid).hall_wanted());
    // Fewer than three cities: still expanding.
    let mut small = mongols(70, 14, 16);
    small.cities.retain(|c| c.id != CityId(3));
    small
        .units
        .retain(|u| u.pos != off(4, 12) && u.pos != off(5, 12));
    assert!(!planner(&small).hall_wanted());
}

#[test]
fn the_hall_waits_until_its_two_vp_decide_something() {
    // Far from the target and not late: the exposure is not worth 2 VP yet.
    let far = mongols(70, 4, 16);
    assert!(!planner(&far).hall_wanted());
    // Within reach of the target (with the lead the Hall earns while built): it pays.
    let near = mongols(70, 12, 16);
    assert!(planner(&near).hall_wanted());
    // Late in the game the points are banked whatever the race looks like.
    let late = mongols(100, 4, 16);
    assert!(planner(&late).hall_wanted());
    // Too late to build and still hold: never.
    let mut over = mongols(147, 12, 16);
    over.turn_limit = Some(150);
    assert!(!planner(&over).hall_wanted());
    // Only the Mongols builds it.
    let mut other = mongols(70, 12, 16);
    other.players[0].faction = Faction::Switzerland;
    assert!(!planner(&other).hall_wanted());
}

#[test]
fn the_hall_is_started_only_where_the_land_can_be_defended() {
    let mut v = mongols(70, 12, 16);
    assert!(planner(&v).hall_ready());
    // The city that will hold it is not garrisoned: units first.
    v.units
        .retain(|u| u.owner != ME || u.pos.distance(off(4, 4)) > 1);
    v.units
        .retain(|u| u.owner != ME || u.pos.distance(off(12, 4)) > 1);
    v.units
        .retain(|u| u.owner != ME || u.pos.distance(off(4, 12)) > 1);
    let p = planner(&v);
    assert!(p.hall_wanted());
    assert!(!p.hall_ready());
    assert_eq!(p.hall_value(&v.cities[0], 16), 0);
    let site = p.hall_site().unwrap();
    assert!(p.hall_unguarded(site));
    // A neighbour with a public army far larger than ours outmatches us.
    let mut weak = mongols(70, 12, 16);
    weak.cities[3].pos = off(24, 4);
    crate::fixture::set_armies(&mut weak, &[(ME, 100), (ENEMY, 900)]);
    assert!(planner(&weak).hall_wanted());
    assert!(!planner(&weak).hall_ready());
    // A city already under more fire than it answers: not now.
    let mut siege = mongols(70, 12, 16);
    siege.units.extend(
        ViewBuilder::new(48, 28)
            .unit(70, ENEMY, UnitKind::Trooper, off(6, 4))
            .unit(71, ENEMY, UnitKind::Trooper, off(6, 5))
            .unit(72, ENEMY, UnitKind::Trooper, off(5, 3))
            .build()
            .units,
    );
    let p = planner(&siege);
    assert!(p.hall_wanted());
    assert!(!p.hall_ready());
}

#[test]
fn the_hall_goes_to_the_safest_city_not_the_frontier() {
    let mut v = mongols(70, 12, 16);
    // A rival city beside city 1, and city 3 walled and far from every rival.
    v.cities.push(
        ViewBuilder::new(48, 28)
            .city(8, ENEMY, off(8, 4), 4)
            .build()
            .cities
            .remove(0),
    );
    v.cities[2].walls = true;
    let p = planner(&v);
    assert_eq!(p.hall_site().map(|c| c.id), Some(CityId(3)));
    assert_eq!(p.hall_value(&v.cities[2], 8), 2 * 1500);
    assert_eq!(p.hall_value(&v.cities[0], 8), 0);
    // A city in resistance, or one that cannot start it, is never the site.
    let mut blocked = v.clone();
    blocked.cities[2]
        .detail
        .as_mut()
        .unwrap()
        .options
        .retain(|o| !is_hall(o.item));
    assert_ne!(planner(&blocked).hall_site().map(|c| c.id), Some(CityId(3)));
}

#[test]
fn one_hall_only_and_never_a_second_while_it_stands_in_a_rivals_hands() {
    let mut v = mongols(70, 12, 16);
    assert!(planner(&v).hall_wanted());
    // Ours stands in our city.
    set_hall(&mut v, ME, ME, 1);
    assert!(!planner(&v).hall_wanted());
    assert_eq!(planner(&v).hall_value(&v.cities[2], 8), 0);
    // Captured: retake the city, never rebuild.
    set_hall(&mut v, ME, ENEMY, 9);
    let p = planner(&v);
    assert!(!p.hall_wanted());
    assert!(!p.hall_ready());
    assert_eq!(p.hall_value(&v.cities[2], 8), 0);
}

#[test]
fn the_brain_starts_the_hall_in_the_chosen_city_and_not_before_it_pays() {
    let v = mongols(70, 12, 16);
    let site = planner(&v).hall_site().unwrap().id;
    let item = hall_item(&v, site);
    let orders = brain(&v);
    assert!(
        orders.contains(&Order::SetProduction { city: site, item })
            || orders.contains(&Order::BuyItem { city: site, item }),
        "{orders:?}"
    );
    // Far from the target and not late: no Hall.
    let far = mongols(70, 4, 16);
    let orders = brain(&far);
    assert!(
        !orders.iter().any(|o| matches!(
            o,
            Order::SetProduction { item, .. } | Order::BuyItem { item, .. } if is_hall(*item)
        )),
        "{orders:?}"
    );
}

#[test]
fn the_goal_pass_buys_the_hall_when_the_gold_is_there() {
    let mut v = mongols(70, 12, 16);
    v.economy.gold = 2000;
    let site = planner(&v).hall_site().unwrap().id;
    let item = hall_item(&v, site);
    for c in &mut v.cities {
        if c.id == site {
            for o in &mut c.detail.as_mut().unwrap().options {
                if o.item == item {
                    o.buy_cost = Some(400);
                }
            }
        }
    }
    let orders = brain(&v);
    assert!(
        orders.contains(&Order::BuyItem { city: site, item }),
        "{orders:?}"
    );
    // Too poor to buy it: it is built in the ordinary way instead.
    v.economy.gold = 20;
    let orders = brain(&v);
    assert!(
        !orders.contains(&Order::BuyItem { city: site, item }),
        "{orders:?}"
    );
}

#[test]
fn a_hall_under_construction_stays_the_plan_when_the_race_moves_on() {
    let mut v = mongols(70, 12, 16);
    let site = planner(&v).hall_site().unwrap().id;
    // Far from the target and not late, but the city is already building it: finish it.
    let mut racing = mongols(70, 4, 16);
    assert!(!planner(&racing).hall_wanted());
    let racing_item = hall_item(&racing, site);
    for c in &mut racing.cities {
        if c.id == site {
            c.detail.as_mut().unwrap().producing = Some(racing_item);
        }
    }
    let p = planner(&racing);
    assert!(p.hall_wanted());
    assert_eq!(p.hall_site().map(|c| c.id), Some(site));
    assert!(p.hall_value(racing.cities.iter().find(|c| c.id == site).unwrap(), 16) > 0);
    // Not when it can no longer finish.
    v.turn = 298;
    let item = hall_item(&v, site);
    for c in &mut v.cities {
        if c.id == site {
            c.detail.as_mut().unwrap().producing = Some(item);
        }
    }
    assert!(!planner(&v).hall_wanted());
}

// ---------- defend ----------

#[test]
fn a_held_hall_city_garrisons_and_walls_before_anything_else() {
    // The Hall stands in city 1, whose garrison is gone.
    let mut v = mongols(70, 12, 16);
    v.units
        .retain(|u| u.owner != ME || u.pos.distance(off(4, 4)) > 1);
    set_hall(&mut v, ME, ME, 1);
    let city = v.cities[0].clone();
    let p = planner(&v);
    assert!(p.holds_hall_city(&city));
    assert!(p.hall_unguarded(&city));
    assert!(p.hall_guard_city(&city));
    assert!(!p.hall_unguarded(&v.cities[1]));
    assert_eq!(p.hall_defense_percent(&city), HALL_DEFENSE_MARGIN_PERCENT);
    assert_eq!(p.hall_defense_percent(&v.cities[1]), 100);
    // The city builds a defender first.
    let orders = brain(&v);
    assert!(
        orders.iter().any(|o| matches!(
            o,
            Order::SetProduction { city, item: ProductionItem::Unit { .. } }
                | Order::BuyItem { city, item: ProductionItem::Unit { .. } }
                if *city == CityId(1)
        )),
        "{orders:?}"
    );
    // A captured Hall is held, and defended, by its captor.
    let mut captor = mongols(70, 12, 16);
    captor.players[0].faction = Faction::Sparta;
    set_hall(&mut captor, ENEMY, ME, 2);
    assert!(planner(&captor).holds_hall_city(&captor.cities[1]));
    assert!(!planner(&captor).holds_hall_city(&captor.cities[0]));
    // A Hall city that is not ours is not ours to guard.
    let mut theirs = mongols(70, 12, 16);
    set_hall(&mut theirs, ME, ENEMY, 9);
    assert!(!planner(&theirs).hall_guard_city(&theirs.cities[0]));
}

#[test]
fn a_threatened_hall_city_is_answered_before_a_busier_one() {
    // Two cities, one free defender between them. City 1 (lower id) is threatened harder, but
    // the Hall stands in city 2.
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Mongols)
        .city(1, ME, off(4, 4), 4)
        .city(2, ME, off(4, 14), 4)
        .unit(1, ME, UnitKind::Soldier, off(4, 9))
        .unit(2, ENEMY, UnitKind::Trooper, off(6, 4))
        .unit(3, ENEMY, UnitKind::Trooper, off(7, 4))
        .unit(4, ENEMY, UnitKind::Soldier, off(6, 14))
        .build();
    v.turn = 40;
    v.players[0].faction_goal_done = false;
    let mut plain = hard(&v);
    plain.plan_military();
    let first = |p: &Planner| {
        p.orders.iter().find_map(|o| match o {
            Order::MoveUnit {
                unit: UnitId(1),
                to,
            } => Some(*to),
            _ => None,
        })
    };
    let plain_to = first(&plain);
    assert!(
        plain_to.is_some_and(|to| to.distance(off(4, 4)) < to.distance(off(4, 14))),
        "control: the harder-threatened city is answered first: {plain_to:?}"
    );
    set_hall(&mut v, ME, ME, 2);
    let mut guard = hard(&v);
    guard.plan_military();
    let guard_to = first(&guard);
    assert!(guard_to.is_some(), "{:?}", guard.orders);
    assert!(
        guard_to.unwrap().distance(off(4, 14)) <= 2,
        "the Hall city is answered first: {guard_to:?} (without the Hall {plain_to:?})"
    );
}

// ---------- rivals: capturing the Hall city ----------

/// A rival view: me (an Sparta) beside two equal enemy cities, the second holding a Hall.
fn rival_view() -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Sparta)
        .city(1, ME, off(4, 6), 4)
        .city(2, ME, off(4, 14), 4)
        .city(3, ENEMY, off(14, 6), 4)
        .city(4, ENEMY, off(14, 14), 4)
        .build();
    v.players[2].faction = Faction::Mongols;
    v.players[0].faction_goal_done = false;
    v
}

#[test]
fn capturing_a_hall_city_is_worth_its_two_vp_swing() {
    let mut v = rival_view();
    assert_eq!(planner(&v).hall_in_city_value(&v.cities[3]), 0);
    set_hall(&mut v, ENEMY, ENEMY, 4);
    let p = planner(&v);
    let swing = p.hall_in_city_value(&v.cities[3]);
    assert!(swing > 0);
    assert_eq!(p.capture_goal_swing(&v.cities[3]), (2, 2));
    assert_eq!(p.hold_city_points(&v.cities[3]), 2);
    let mut old = planner(&v);
    old.decisions.vp_hold_break = 0;
    assert_eq!(old.hold_city_points(&v.cities[3]), 0);
    assert_eq!(p.capture_goal_swing(&v.cities[2]), (0, 0));
    assert_eq!(p.hall_in_city_value(&v.cities[2]), 0, "another city");
    assert!(p.hoard_denial_value(&v.cities[3]) >= swing);
    // A team-mate's Hall is not ours to take.
    let mut mate = v.clone();
    mate.courier_halls[0].holder = PlayerId(1);
    assert_eq!(planner(&mate).hall_in_city_value(&mate.cities[3]), 0);
    // A Celts razes the city and the Hall with it: it only denies.
    let mut celts = v.clone();
    celts.players[0].faction = Faction::Celts;
    assert_eq!(planner(&celts).capture_goal_swing(&celts.cities[3]), (0, 2));
    assert!(planner(&celts).hall_in_city_value(&celts.cities[3]) < swing);
    assert!(planner(&celts).hall_in_city_value(&celts.cities[3]) > 0);
    // Retaking our own Hall is worth more than taking a stranger's.
    let mut retake = v.clone();
    retake.players[0].faction = Faction::Mongols;
    retake.courier_halls[0].of = ME;
    assert!(planner(&retake).hall_in_city_value(&retake.cities[3]) > swing);
}

#[test]
fn the_hard_conquest_nomination_prefers_the_hall_city() {
    let mut v = rival_view();
    // City 3 is a hair easier, so without a Hall it is the objective.
    v.cities[2].strength = 10;
    v.cities[3].strength = 14;
    assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(3)));
    set_hall(&mut v, ENEMY, ENEMY, 4);
    assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(4)));
}

#[test]
fn a_captor_that_holds_the_hall_is_a_war_target_for_its_points() {
    let mut v = rival_view();
    // The Hall is the Mongols PlayerId(3)'s, held by the enemy captor ENEMY.
    v.players[3].faction = Faction::Mongols;
    v.players[2].faction = Faction::Switzerland;
    set_hall(&mut v, PlayerId(3), ENEMY, 4);
    let enemy_team = v.team_of(ENEMY);
    assert_eq!(planner(&v).held_goal_denial(enemy_team), 0);
    v.players[2].vp_sources = vec![(VpSource::FactionGoal { of: PlayerId(3) }, 2)];
    let denial = planner(&v).held_goal_denial(enemy_team);
    assert!(denial > 0);
    // The Mongols whose Hall it was prices the war that retakes it higher.
    let mut owner = v.clone();
    owner.players[0].faction = Faction::Mongols;
    owner.players[3].faction = Faction::Switzerland;
    owner.players[0].faction = Faction::Mongols;
    owner.courier_halls[0].of = ME;
    owner.players[2].vp_sources = vec![(VpSource::FactionGoal { of: ME }, 2)];
    owner.players[0].faction = Faction::Mongols;
    assert!(planner(&owner).held_goal_denial(enemy_team) > 0);
}

#[test]
fn the_mongols_retakes_its_hall_city_rather_than_rebuilding() {
    let mut v = rival_view();
    v.players[0].faction = Faction::Mongols;
    v.players[2].faction = Faction::Sparta;
    v.cities[2].strength = 10;
    v.cities[3].strength = 14;
    v.turn = 70;
    v.standings = standings(12, 16).into();
    assert_eq!(hard(&v).conquest_city().map(|c| c.id), Some(CityId(3)));
    set_hall(&mut v, ME, ENEMY, 4);
    let p = hard(&v);
    assert_eq!(p.conquest_city().map(|c| c.id), Some(CityId(4)));
    assert!(!p.hall_wanted());
}

// ---------- rivals: staging against the holder's open land ----------

fn peace(v: &mut PlayerView) {
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    let against = v.team_of(ENEMY);
    v.diplomacy.legacy = false;
    v.diplomacy.relations = vec![Relation {
        a: v.team,
        b: against,
        state: RelationState::Peace,
        effective_turn: 1,
        guarantee_until: 0,
        version: 0,
        security: None,
        war_id: None,
        declarers: vec![],
    }];
    v.diplomacy.actions = vec![DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against,
            breach: false,
        },
        effective_turn: v.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    }];
}

/// At Peace, a Hard Sparta army beside the Mongols holder's land. The holder's territory is
/// every tile within 3 of its cities; the Hall (if any) is in `hall_city`.
fn staging_view(with_hall: bool) -> PlayerView {
    let hall_city = off(18, 8);
    let mut v = ViewBuilder::new(40, 20)
        .faction(Faction::Sparta)
        .city(1, ME, off(4, 8), 4)
        .city(2, ME, off(4, 14), 4)
        .city(4, ENEMY, hall_city, 3)
        .unit(1, ME, UnitKind::Soldier, off(9, 8))
        .unit(2, ME, UnitKind::Soldier, off(9, 9))
        .unit(3, ME, UnitKind::Trooper, off(8, 8))
        .unit(4, ME, UnitKind::Ranger, off(8, 9))
        .build();
    v.players[2].faction = Faction::Mongols;
    v.players[0].faction_goal_done = false;
    v.turn = 60;
    for h in hall_city.within(4) {
        if let Some(i) = v.index(h) {
            v.tiles[i].owner = Some(ENEMY);
        }
    }
    peace(&mut v);
    if with_hall {
        set_hall(&mut v, ENEMY, ENEMY, 4);
    }
    v
}

fn moves(p: &Planner) -> Vec<Hex> {
    p.orders
        .iter()
        .filter_map(|o| match o {
            Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => Some(*to),
            _ => None,
        })
        .collect()
}

#[test]
fn at_peace_rivals_stage_inside_the_holders_land_beside_the_hall_city() {
    let hall_city = off(18, 8);
    // Without a Hall the holder's border is closed: the army can only stage outside it.
    let closed = staging_view(false);
    let p = hard(&closed);
    assert!(!p.ctx.reachable(closed.units[0].pos, hall_city));
    // With the Hall every empire, military included, may cross all of the holder's land.
    let open = staging_view(true);
    let mut p = hard(&open);
    assert!(p.ctx.reachable(open.units[0].pos, hall_city));
    assert_eq!(p.conquest_city().map(|c| c.id), Some(CityId(4)));
    p.plan_military();
    let dests = moves(&p);
    assert!(!dests.is_empty(), "{:?}", p.orders);
    assert!(
        dests
            .iter()
            .any(|to| to.distance(hall_city) <= 3 && open.tile(*to).unwrap().owner == Some(ENEMY)),
        "the army stages inside the holder's land: {dests:?}"
    );
    assert!(
        !p.orders
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { .. }))
    );
    // The control: with the border closed, every staging tile is outside the holder's land.
    let mut c = hard(&closed);
    c.plan_military();
    assert!(
        moves(&c)
            .iter()
            .all(|to| closed.tile(*to).unwrap().owner != Some(ENEMY)),
        "{:?}",
        c.orders
    );
}

#[test]
fn an_army_staged_beside_the_hall_city_declares_war_and_strikes_the_same_turn() {
    let hall_city = off(18, 8);
    let mut v = staging_view(true);
    // Assembled one tile from the Hall city, inside the holder's land.
    for (i, pos) in [off(17, 8), off(17, 9), off(16, 8), off(16, 9)]
        .into_iter()
        .enumerate()
    {
        v.units[i].pos = pos;
        assert_eq!(v.tile(pos).unwrap().owner, Some(ENEMY));
        assert!(v.units[i].pos.distance(hall_city) <= 2);
    }
    let orders = brain(&v);
    let declare = Order::DeclareWar {
        against: v.team_of(ENEMY),
        breach: false,
    };
    assert!(orders.contains(&declare), "{orders:?}");
    // And the control: the same army without the Hall cannot even be standing there.
    let mut closed = staging_view(false);
    for (i, pos) in [off(17, 8), off(17, 9), off(16, 8), off(16, 9)]
        .into_iter()
        .enumerate()
    {
        closed.units[i].pos = pos;
    }
    assert!(!closed.passage_allowed(ME, UnitKind::Soldier, Some(ENEMY)));
}

#[test]
fn the_holder_counts_a_rival_army_at_its_border_as_a_threat() {
    // The Mongols's side of the same staging: a rival army at its border at Peace could
    // declare and walk into its Hall city, so the city is threatened and defended. (The reach a
    // declaration opens already covers the declarer's own land, Hall or not.)
    let mut v = ViewBuilder::new(40, 20)
        .faction(Faction::Mongols)
        .city(1, ME, off(18, 8), 4)
        .city(2, ME, off(24, 8), 4)
        .city(3, ENEMY, off(4, 8), 4)
        .unit(1, ME, UnitKind::Soldier, off(18, 8))
        .unit(2, ME, UnitKind::Soldier, off(24, 8))
        .unit(3, ENEMY, UnitKind::Trooper, off(14, 8))
        .unit(4, ENEMY, UnitKind::Trooper, off(14, 9))
        .build();
    v.turn = 60;
    v.players[0].faction_goal_done = false;
    for h in off(18, 8).within(3).into_iter().chain(off(24, 8).within(3)) {
        if let Some(i) = v.index(h) {
            v.tiles[i].owner = Some(ME);
        }
    }
    peace(&mut v);
    set_hall(&mut v, ME, ME, 1);
    let mut p = hard(&v);
    p.prepare_hard_defense();
    assert!(p.ctx.threat[0] > 0, "{:?}", p.ctx.threat);
    assert!(
        planner(&v).hall_defense_percent(&v.cities[0])
            > planner(&v).hall_defense_percent(&v.cities[1])
    );
}

// ---------- the end of a game with no limit ----------

#[test]
fn a_game_with_no_limit_has_no_last_turn_to_miss() {
    // Turn 800 of a game with no turn limit, 9 of 10 VP, a Hall that takes 50 turns: the old
    // fiction put the end 22 turns ahead, so it never paid and a 9-VP Mongols sat idle for
    // 900 turns (stalled seed 9983, 2026-10-23).
    let mut v = mongols(400, 9, 10);
    for c in &mut v.cities {
        if let Some(d) = c.detail.as_mut() {
            for o in &mut d.options {
                if is_hall(o.item) {
                    o.turns = 50;
                }
            }
        }
    }
    v.turn_limit = None;
    assert!(planner(&v).hall_wanted());
    let mut fiction = planner(&v);
    fiction.decisions.goal_open_end = 0;
    assert!(!fiction.hall_wanted(), "the gene at 0 keeps the old end");
    // A real turn limit still ends the game: 50 turns do not fit 40 turns from the end.
    let mut limited = v.clone();
    limited.turn_limit = Some(840);
    assert!(!planner(&limited).hall_wanted());
}

#[test]
fn a_hall_that_wins_the_game_is_not_held_back_by_exposure() {
    // 8 of 10 VP: the Hall's 2 VP reach the target. The Hall city is not garrisoned and an army
    // far larger than ours stands near: the old rule waited for both to clear, which at war with
    // half the map never happens.
    let mut v = mongols(100, 8, 10);
    v.units.retain(|u| u.owner != ME);
    crate::fixture::set_armies(&mut v, &[(ME, 100), (ENEMY, 900)]);
    v.cities[3].pos = off(14, 4);
    let p = planner(&v);
    assert!(p.hall_wanted());
    assert!(p.hall_outmatched() && !p.hall_garrisoned(p.hall_site().unwrap()));
    assert!(p.hall_ready(), "the points win the game");
    assert_eq!(
        p.hall_value(p.hall_site().unwrap(), 16),
        8000,
        "worth the game"
    );
    let mut careful = planner(&v);
    careful.decisions.hall_decisive = 0;
    assert!(!careful.hall_ready(), "the gene at 0 keeps the old check");
    // Two VP short of the target after the Hall: the check still holds.
    let mut far = v.clone();
    far.standings = standings(5, 10).into();
    assert!(!planner(&far).hall_ready());
}

#[test]
fn a_hall_that_wins_the_game_replaces_the_unit_its_city_keeps_building() {
    // 8 of 10 VP; every city repeats a Drone (cities keep a unit in production for good), so none
    // is ever idle for the goal pass to give the Hall to.
    let drone = ProductionItem::Unit {
        unit: UnitKind::Soldier,
    };
    let mut v = mongols(100, 8, 10);
    for c in &mut v.cities {
        if let Some(d) = c.detail.as_mut() {
            d.producing = Some(drone);
        }
    }
    let site = planner(&v).hall_site().unwrap().id;
    let set_hall = Order::SetProduction {
        city: site,
        item: hall_item(&v, site),
    };
    let orders = |gene: i32| {
        let mut b = ScriptedBrain::with_difficulty(Faction::Mongols, 7, ME, AiDifficulty::Hard);
        // Isolate the Hall genes: modernization independently reconsiders busy queues.
        b.decisions.modernization_value = 0;
        b.decisions.hall_decisive = gene;
        b.decisions.goal_open_end = gene;
        b.orders(&v)
    };
    assert!(orders(1).contains(&set_hall), "the Hall is built");
    assert!(
        !orders(0).contains(&set_hall),
        "the gene at 0 leaves the Drone"
    );
    // Two VP short of the target after the Hall: busy cities stay busy.
    let mut far = v.clone();
    far.standings = standings(5, 10).into();
    let mut b = ScriptedBrain::with_difficulty(Faction::Mongols, 7, ME, AiDifficulty::Hard);
    b.decisions.modernization_value = 0;
    assert!(!b.orders(&far).contains(&set_hall));
}

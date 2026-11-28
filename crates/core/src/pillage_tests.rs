//! Unit tests for enemy-occupied tile denial and pillaging (`Order::Pillage`).

use crate::data::CITY_WORK_RADIUS;
use crate::economy;
use crate::event_visibility::{event_visibility_view, visible_events};
use crate::events::Event;
use crate::game::{blank_game, spawn_unit};
use crate::hex::Hex;
use crate::improvements;
use crate::orders::Order;
use crate::resources::{self, Resource};
use crate::rules::{resolve_turn, validate_orders};
use crate::state::{GameSetup, GameState, Map, PlayerSetup};
use crate::types::*;
use crate::view::player_view;
use std::collections::BTreeMap;

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn war_game(factions: &[Faction]) -> GameState {
    let players = factions
        .iter()
        .enumerate()
        .map(|(i, &faction)| PlayerSetup {
            name: format!("P{i}"),
            faction,
            team: TeamId(i as u8),
            is_human: false,
            difficulty: Default::default(),
            search: false,
        })
        .collect();
    let setup = GameSetup {
        seed: 42,
        width: 14,
        height: 12,
        players,
        turn_limit: Some(150),
        legacy_all_war: false,
        ai_alliances: false,
        deals: false,
        barbarians: false,
        rule_experiments: Default::default(),
    };
    let mut state = blank_game(&setup, Map::new(14, 12, Terrain::Grassland));
    if let Some(d) = &mut state.diplomacy {
        for r in &mut d.relations {
            r.state = crate::diplomacy::RelationState::War;
        }
    }
    state
}

fn add_city(state: &mut GameState, _id: u32, owner: PlayerId, pos: Hex, _capital: bool) -> CityId {
    let cid = crate::rules::test_support::city(state, owner.0, pos, 3);
    for h in pos.within_iter(CITY_WORK_RADIUS) {
        if let Some(t) = state.map.get_mut(h)
            && t.owner.is_none()
        {
            t.owner = Some(owner);
            t.city = Some(cid);
        }
    }
    cid
}

fn orders_map(entries: &[(PlayerId, Vec<Order>)]) -> BTreeMap<PlayerId, Vec<Order>> {
    entries.iter().cloned().collect()
}

#[test]
fn enemy_military_unit_denies_tile_yields_and_strategic_supply() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    s.player_mut(PlayerId(0)).techs.insert(TechId::IronWorking);
    let city_pos = at(4, 4);
    let cid = add_city(&mut s, 1, PlayerId(0), city_pos, true);
    add_city(&mut s, 2, PlayerId(1), at(12, 10), true);
    let iron = at(5, 4);
    {
        let t = s.map.get_mut(iron).unwrap();
        t.terrain = Terrain::Hills;
        t.resource = Some(Resource::Iron);
        t.improved = true;
    }
    // Give P0 two Phalangites (needs 2 Iron; deposit gives 3 with Mine).
    spawn_unit(&mut s, PlayerId(0), UnitKind::Phalangite, at(3, 4));
    spawn_unit(&mut s, PlayerId(0), UnitKind::Phalangite, at(3, 5));

    let yields_before = economy::city_totals(&s, &s.cities[&cid]);
    assert_eq!(resources::supplied(&s, PlayerId(0), Resource::Iron), 3);

    // Enemy military unit sits on the Iron deposit.
    let raider = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, iron);
    assert!(!economy::worked_tiles(&s, &s.cities[&cid]).contains(&iron));
    let yields_after = economy::city_totals(&s, &s.cities[&cid]);
    assert!(yields_after.production < yields_before.production);
    assert_eq!(resources::supplied(&s, PlayerId(0), Resource::Iron), 0);

    // PlayerView reflects blocked state on both TileView and DepositView.
    let v = player_view(&s, PlayerId(0));
    assert!(v.tile(iron).unwrap().blocked);
    let iron_supply = v
        .supply
        .iter()
        .find(|sv| sv.resource == Resource::Iron)
        .unwrap();
    assert_eq!(iron_supply.supplied.total, 0);
    assert!(iron_supply.deposits[0].blocked);
    assert_eq!(iron_supply.deposits[0].supply, 0);

    // Resolving a turn while enemy sits on the sole Iron deposit puts Swordsmen into shortage.
    let (next, _) = resolve_turn(&s, &BTreeMap::new());
    let unsupplied_count = next
        .units
        .values()
        .filter(|u| u.owner == PlayerId(0) && resources::unit_unsupplied(&next, u).is_some())
        .count();
    assert_eq!(unsupplied_count, 2);

    // Removing the enemy restores yields and full Iron supply.
    let mut restored = next;
    restored.units.remove(&raider);
    assert_eq!(
        resources::supplied(&restored, PlayerId(0), Resource::Iron),
        3
    );
    let (after_clear, _) = resolve_turn(&restored, &BTreeMap::new());
    assert_eq!(
        after_clear
            .units
            .values()
            .filter(
                |u| u.owner == PlayerId(0) && resources::unit_unsupplied(&after_clear, u).is_some()
            )
            .count(),
        0
    );
}

#[test]
fn peaceful_or_civilian_unit_does_not_deny_yields_or_supply() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    s.player_mut(PlayerId(0)).techs.insert(TechId::IronWorking);
    let cid = add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    let iron = at(5, 4);
    {
        let t = s.map.get_mut(iron).unwrap();
        t.terrain = Terrain::Hills;
        t.resource = Some(Resource::Iron);
        t.improved = true;
    }

    // Enemy Settler (non-military) does not deny yields or supply.
    let settler = spawn_unit(&mut s, PlayerId(1), UnitKind::Settler, iron);
    assert!(economy::worked_tiles(&s, &s.cities[&cid]).contains(&iron));
    assert_eq!(resources::supplied(&s, PlayerId(0), Resource::Iron), 3);
    s.units.remove(&settler);

    // Peaceful military unit does not deny yields or supply.
    if let Some(d) = &mut s.diplomacy {
        for r in &mut d.relations {
            r.state = crate::diplomacy::RelationState::Peace;
        }
    }
    spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, iron);
    assert!(economy::worked_tiles(&s, &s.cities[&cid]).contains(&iron));
    assert_eq!(resources::supplied(&s, PlayerId(0), Resource::Iron), 3);
    assert!(!player_view(&s, PlayerId(0)).tile(iron).unwrap().blocked);
}

#[test]
fn pillage_eligibility_checks() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    let _cid = add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    let farm = at(5, 4);
    {
        let t = s.map.get_mut(farm).unwrap();
        t.resource = Some(Resource::Wheat);
        t.improved = true;
    }

    // Scout cannot pillage.
    let scout = spawn_unit(&mut s, PlayerId(1), UnitKind::Scout, farm);
    assert!(!validate_orders(&s, PlayerId(1), &[Order::Pillage { unit: scout }]).is_empty());
    assert!(
        player_view(&s, PlayerId(1))
            .units
            .iter()
            .find(|u| u.id == scout)
            .unwrap()
            .pillage
            .is_none()
    );
    s.units.remove(&scout);

    // Settler cannot pillage.
    let settler = spawn_unit(&mut s, PlayerId(1), UnitKind::Settler, farm);
    assert!(!validate_orders(&s, PlayerId(1), &[Order::Pillage { unit: settler }]).is_empty());
    s.units.remove(&settler);

    // Own unit cannot pillage own tile.
    let own = spawn_unit(&mut s, PlayerId(0), UnitKind::Horseman, farm);
    assert!(!validate_orders(&s, PlayerId(0), &[Order::Pillage { unit: own }]).is_empty());
    s.units.remove(&own);

    // Land military unit at War on improved enemy tile CAN pillage.
    let horse = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, farm);
    assert!(validate_orders(&s, PlayerId(1), &[Order::Pillage { unit: horse }]).is_empty());
    let preview = player_view(&s, PlayerId(1))
        .units
        .iter()
        .find(|u| u.id == horse)
        .unwrap()
        .pillage
        .expect("pillage preview");
    assert_eq!(preview.improvement, ImprovementKind::Farm);
    assert_eq!(preview.heal, improvements::PILLAGE_HEAL);
    assert_eq!(preview.gold, 0);

    // Already pillaged tile cannot be pillaged again.
    s.map.get_mut(farm).unwrap().pillaged = true;
    assert!(!validate_orders(&s, PlayerId(1), &[Order::Pillage { unit: horse }]).is_empty());
    s.map.get_mut(farm).unwrap().pillaged = false;

    // Peaceful empire's tile cannot be pillaged.
    if let Some(d) = &mut s.diplomacy {
        for r in &mut d.relations {
            r.state = crate::diplomacy::RelationState::Peace;
        }
    }
    assert!(!validate_orders(&s, PlayerId(1), &[Order::Pillage { unit: horse }]).is_empty());
}

#[test]
fn pillage_farm_heals_50_capped_at_100_and_clears_fortify() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    add_city(&mut s, 2, PlayerId(1), at(12, 10), true);
    let farm = at(5, 4);
    {
        let t = s.map.get_mut(farm).unwrap();
        t.resource = Some(Resource::Wheat);
        t.improved = true;
    }
    let raider = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, farm);
    {
        let u = s.units.get_mut(&raider).unwrap();
        u.hp = 40;
        u.fortify = 6;
    }

    let (next, events) = resolve_turn(
        &s,
        &orders_map(&[(PlayerId(1), vec![Order::Pillage { unit: raider }])]),
    );
    let u = &next.units[&raider];
    assert_eq!(u.hp, 90);
    assert_eq!(u.fortify, 0);
    assert!(next.map.get(farm).unwrap().pillaged);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::TilePillaged {
            raider: PlayerId(1),
            owner: PlayerId(0),
            at: p,
            improvement: ImprovementKind::Farm,
            gold: 0,
            healed: 50,
            ..
        } if *p == farm
    )));

    // Also verify cap at 100 on Pasture.
    let mut s2 = war_game(&[Faction::India, Faction::Rome]);
    add_city(&mut s2, 1, PlayerId(0), at(4, 4), true);
    add_city(&mut s2, 2, PlayerId(1), at(12, 10), true);
    let pasture = at(5, 4);
    {
        let t = s2.map.get_mut(pasture).unwrap();
        t.resource = Some(Resource::Cattle);
        t.improved = true;
    }
    let r2 = spawn_unit(&mut s2, PlayerId(1), UnitKind::Horseman, pasture);
    s2.units.get_mut(&r2).unwrap().hp = 75;
    let (next2, events2) = resolve_turn(
        &s2,
        &orders_map(&[(PlayerId(1), vec![Order::Pillage { unit: r2 }])]),
    );
    assert_eq!(next2.units[&r2].hp, 100);
    assert!(events2.iter().any(|e| matches!(
        e,
        Event::TilePillaged {
            improvement: ImprovementKind::Pasture,
            healed: 25,
            gold: 0,
            ..
        }
    )));
}

#[test]
fn pillage_mine_gives_era_scaled_gold_and_cuts_supply_bonus() {
    let mut s = war_game(&[Faction::Germany, Faction::Rome]);
    s.player_mut(PlayerId(0)).techs.insert(TechId::IronWorking);
    // Put raider (P1) in Classical era (era = 1 -> 25 * 2 = 50 Gold).
    for tech in TechId::ALL {
        if crate::eras::era_of_tech(tech) == 0 {
            s.player_mut(PlayerId(1)).techs.insert(tech);
        }
    }
    assert_eq!(crate::eras::current_era(&s.player(PlayerId(1)).techs), 1);
    add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    add_city(&mut s, 2, PlayerId(1), at(12, 10), true);
    let iron = at(5, 4);
    {
        let t = s.map.get_mut(iron).unwrap();
        t.terrain = Terrain::Hills;
        t.resource = Some(Resource::Iron);
        t.improved = true;
    }
    // Germany gets 2 * (2 + 1) = 6 Iron from a mined deposit, 4 unmined/pillaged.
    assert_eq!(resources::supplied(&s, PlayerId(0), Resource::Iron), 6);

    let raider = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, iron);
    let gold_before = s.player(PlayerId(1)).gold;
    let (mut next, events) = resolve_turn(
        &s,
        &orders_map(&[(PlayerId(1), vec![Order::Pillage { unit: raider }])]),
    );
    assert!(next.map.get(iron).unwrap().pillaged);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::TilePillaged {
            improvement: ImprovementKind::Mine,
            gold: 50,
            healed: 0,
            ..
        }
    )));
    // Raider received the 50 Gold pillage payout (plus any turn income).
    assert!(next.player(PlayerId(1)).gold >= gold_before + 50);

    // Once raider leaves the tile, Germany gets only base deposit supply (4), not mined (6).
    next.units.remove(&raider);
    assert_eq!(resources::supplied(&next, PlayerId(0), Resource::Iron), 4);
    let iron_view = player_view(&next, PlayerId(0))
        .supply
        .into_iter()
        .find(|sv| sv.resource == Resource::Iron)
        .unwrap();
    assert!(iron_view.deposits[0].pillaged);
    assert!(!iron_view.deposits[0].mined);
    assert_eq!(iron_view.deposits[0].supply, 4);
}

#[test]
fn raider_killed_before_pillage_phase_does_not_pillage() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    let farm = at(5, 4);
    {
        let t = s.map.get_mut(farm).unwrap();
        t.resource = Some(Resource::Wheat);
        t.improved = true;
    }
    let raider = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, farm);
    s.units.get_mut(&raider).unwrap().hp = 5;
    let archer = spawn_unit(&mut s, PlayerId(0), UnitKind::Archer, at(4, 4));

    let (next, events) = resolve_turn(
        &s,
        &orders_map(&[
            (
                PlayerId(0),
                vec![Order::RangedAttack {
                    unit: archer,
                    target: farm,
                }],
            ),
            (PlayerId(1), vec![Order::Pillage { unit: raider }]),
        ]),
    );
    assert!(!next.units.contains_key(&raider));
    assert!(!next.map.get(farm).unwrap().pillaged);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::TilePillaged { .. }))
    );
}

#[test]
fn repair_pillaged_improvement_costs_half_and_restores_yields() {
    let mut s = war_game(&[Faction::India, Faction::Rome]);
    s.player_mut(PlayerId(0)).techs.insert(TechId::IronWorking);
    s.player_mut(PlayerId(0)).gold = 200;
    let cid = add_city(&mut s, 1, PlayerId(0), at(4, 4), true);
    let iron = at(5, 4);
    {
        let t = s.map.get_mut(iron).unwrap();
        t.terrain = Terrain::Hills;
        t.resource = Some(Resource::Iron);
    }
    let mine_item = ProductionItem::Improvement {
        improvement: ImprovementKind::Mine,
        tile: iron,
    };
    let normal_prod = economy::city_production_cost(&s, &s.cities[&cid], mine_item);
    let normal_buy = economy::buy_cost_for(&s, &s.cities[&cid], mine_item).unwrap();
    assert_eq!(normal_prod, 40);
    assert_eq!(normal_buy, 80);

    // Mark improved and pillaged.
    {
        let t = s.map.get_mut(iron).unwrap();
        t.improved = true;
        t.pillaged = true;
    }
    let repair_prod = economy::city_production_cost(&s, &s.cities[&cid], mine_item);
    let repair_buy = economy::buy_cost_for(&s, &s.cities[&cid], mine_item).unwrap();
    assert_eq!(repair_prod, 20);
    assert_eq!(repair_buy, 40);

    let v = player_view(&s, PlayerId(0));
    let opt = v.cities[0]
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == mine_item)
        .expect("repair option listed");
    assert_eq!(opt.cost, 20);
    assert_eq!(opt.buy_cost, Some(40));
    assert!(opt.improvement.as_ref().unwrap().repair);

    // Buy the repair and verify tile is restored.
    let (next, _) = resolve_turn(
        &s,
        &orders_map(&[(
            PlayerId(0),
            vec![Order::BuyItem {
                city: cid,
                item: mine_item,
            }],
        )]),
    );
    let t = next.map.get(iron).unwrap();
    assert!(t.improved);
    assert!(!t.pillaged);
    assert_eq!(resources::supplied(&next, PlayerId(0), Resource::Iron), 3);
}

#[test]
fn tile_pillaged_event_respects_fog_of_war() {
    let mut s = war_game(&[Faction::India, Faction::Rome, Faction::Athens]);
    add_city(&mut s, 1, PlayerId(0), at(2, 2), true);
    add_city(&mut s, 2, PlayerId(2), at(12, 10), true);
    let farm = at(3, 2);
    {
        let t = s.map.get_mut(farm).unwrap();
        t.resource = Some(Resource::Wheat);
        t.improved = true;
    }
    let raider = spawn_unit(&mut s, PlayerId(1), UnitKind::Horseman, farm);
    crate::visibility::update_explored(&mut s);

    let before_p0 = event_visibility_view(&s, PlayerId(0));
    let before_p1 = event_visibility_view(&s, PlayerId(1));
    let before_p2 = event_visibility_view(&s, PlayerId(2));

    let (next, events) = resolve_turn(
        &s,
        &orders_map(&[(PlayerId(1), vec![Order::Pillage { unit: raider }])]),
    );
    let after_p0 = event_visibility_view(&next, PlayerId(0));
    let after_p1 = event_visibility_view(&next, PlayerId(1));
    let after_p2 = event_visibility_view(&next, PlayerId(2));

    let has_pillage = |evs: &[crate::event_visibility::VisibleEvent]| {
        evs.iter().any(|e| {
            matches!(
                e,
                crate::event_visibility::VisibleEvent::Complete(Event::TilePillaged { .. })
            )
        })
    };
    assert!(has_pillage(&visible_events(&events, &before_p0, &after_p0)));
    assert!(has_pillage(&visible_events(&events, &before_p1, &after_p1)));
    assert!(!has_pillage(&visible_events(
        &events, &before_p2, &after_p2
    )));
}

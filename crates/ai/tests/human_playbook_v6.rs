//! Check the human playbook across real fog, diplomacy and simultaneous combat boundaries.
use legatum_ai::{Brain, HumanPlaybookV6Brain};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::*;
use std::collections::BTreeMap;

const ME: PlayerId = PlayerId(0);
const ENEMY: PlayerId = PlayerId(1);

fn at(col: i32, row: i32) -> Hex {
    Hex::from_offset(col, row)
}

fn scenario() -> GameState {
    let mut setup = GameSetup::all_ai(77);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::India; // Ordinary closed-border movement.
    setup.width = 30;
    setup.height = 24;
    let mut s = blank_game(&setup, Map::new(30, 24, Terrain::Grassland));
    for (owner, h) in [
        (ME, at(5, 8)),
        (ME, at(5, 14)),
        (ME, at(11, 15)),
        (ENEMY, at(12, 8)),
    ] {
        let settler = spawn_unit(&mut s, owner, UnitKind::Settler, h);
        rules::cities::found_city(&mut s, settler, &mut Vec::new()).unwrap();
    }
    for (kind, h) in [
        (UnitKind::Spearman, at(9, 8)),
        (UnitKind::Pikeman, at(9, 10)),
        (UnitKind::Archer, at(9, 7)),
        (UnitKind::Archer, at(9, 9)),
        (UnitKind::Archer, at(8, 8)),
        (UnitKind::Scout, at(10, 8)),
    ] {
        spawn_unit(&mut s, ME, kind, h);
    }
    s.turn = 90;
    visibility::update_explored(&mut s);
    s
}

#[test]
fn stages_then_declares_across_closed_borders_without_premature_attacks() {
    let s = scenario();
    assert!(!s.at_war(ME, ENEMY));
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(
        validate_orders(&s, ME, &orders).is_empty(),
        "{:?}",
        validate_orders(&s, ME, &orders)
    );
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }),
        "{orders:?}"
    );
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { .. }))
    );
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
    assert!(next.at_war(ME, ENEMY), "{events:?}");
    // Automatic city fire may activate with the war; our units still plan from Peace.
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::Battle {
            attacker: events::Attacker::Unit(_),
            attacker_owner: ME,
            ..
        }
    )));
}

#[test]
fn a_visible_unguarded_city_does_not_wait_for_the_full_siege_force() {
    let mut s = scenario();
    let spare = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Archer)
        .unwrap()
        .id;
    s.units.remove(&spare);
    visibility::update_explored(&mut s);
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(validate_orders(&s, ME, &orders).is_empty());
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }),
        "{orders:?}"
    );
}

#[test]
fn veteran_infantry_take_an_unguarded_city_without_waiting_for_archers() {
    let mut s = scenario();
    s.units.retain(|_, u| u.kind != UnitKind::Archer);
    for u in s
        .units
        .values_mut()
        .filter(|u| u.owner == ME && u.kind == UnitKind::Spearman)
    {
        u.kind = UnitKind::Pikeman;
    }
    visibility::update_explored(&mut s);
    let city = s.city_at(at(12, 8)).unwrap().id;
    // A mature, unwalled city is still a viable target for two healthy veterans.
    s.cities.get_mut(&city).unwrap().population = 5;
    // The same small force must wait at a walled target.
    let mut walled = s.clone();
    walled
        .cities
        .get_mut(&city)
        .unwrap()
        .buildings
        .insert(BuildingKind::Walls);
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&walled, ME));
    assert!(!orders.iter().any(|o| matches!(o, Order::DeclareWar { .. })));
    let mut brain = HumanPlaybookV6Brain::default();
    for _ in 0..5 {
        let orders = brain.orders(&player_view(&s, ME));
        assert!(
            validate_orders(&s, ME, &orders).is_empty(),
            "{:?}",
            validate_orders(&s, ME, &orders)
        );
        s = resolve_turn(&s, &BTreeMap::from([(ME, orders)])).0;
        if s.cities[&city].owner == ME {
            return;
        }
    }
    panic!("infantry did not finish the unguarded city");
}

#[test]
fn staging_outside_sight_can_probe_a_remembered_city_without_reading_hidden_guards() {
    let mut a = scenario();
    a.units
        .retain(|_, u| !matches!(u.kind, UnitKind::Scout | UnitKind::Archer));
    for u in a.units.values_mut().filter(|u| u.owner == ME) {
        u.kind = UnitKind::Pikeman;
    }
    let hidden = spawn_unit(&mut a, ENEMY, UnitKind::Musketman, at(27, 20));
    visibility::update_explored(&mut a);
    let mut b = a.clone();
    b.units.get_mut(&hidden).unwrap().pos = at(12, 8);
    visibility::update_explored(&mut b);
    let av = player_view(&a, ME);
    let bv = player_view(&b, ME);
    assert_eq!(av, bv);
    assert!(!av.cities.iter().find(|c| c.owner == ENEMY).unwrap().visible);
    let orders = HumanPlaybookV6Brain::default().orders(&av);
    assert_eq!(orders, HumanPlaybookV6Brain::default().orders(&bv));
    assert!(orders.contains(&Order::DeclareWar {
        against: TeamId(1),
        breach: false
    }));
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { .. }))
    );
    assert!(validate_orders(&a, ME, &orders).is_empty());
}

#[test]
fn a_settler_reveals_a_site_when_every_known_tile_is_too_close_to_home() {
    let mut setup = GameSetup::all_ai(77);
    setup.players.truncate(2);
    setup.width = 30;
    setup.height = 24;
    let mut s = blank_game(&setup, Map::new(30, 24, Terrain::Grassland));
    let founder = spawn_unit(&mut s, ME, UnitKind::Settler, at(5, 8));
    rules::cities::found_city(&mut s, founder, &mut Vec::new()).unwrap();
    let other = spawn_unit(&mut s, ENEMY, UnitKind::Settler, at(25, 18));
    rules::cities::found_city(&mut s, other, &mut Vec::new()).unwrap();
    let settler = spawn_unit(&mut s, ME, UnitKind::Settler, at(6, 8));
    visibility::update_explored(&mut s);
    let view = player_view(&s, ME);
    let home = view.cities[0].pos;
    assert!(view.tiles.iter().enumerate().all(|(i, t)| {
        !t.explored
            || home.distance(Hex::from_offset(i as i32 % 30, i as i32 / 30))
                <= legatum_core::data::MIN_CITY_GAP
    }));
    let mut brain = HumanPlaybookV6Brain::default();
    for _ in 0..6 {
        let orders = brain.orders(&player_view(&s, ME));
        assert!(validate_orders(&s, ME, &orders).is_empty());
        s = resolve_turn(&s, &BTreeMap::from([(ME, orders)])).0;
        if !s.units.contains_key(&settler)
            && s.cities.values().filter(|c| c.owner == ME).count() == 2
        {
            return;
        }
    }
    panic!("settler did not reveal and found a reachable site");
}

#[test]
fn wounded_city_falls_after_a_volley_with_city_fire_enabled() {
    let mut s = scenario();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    let id = s.city_at(at(12, 8)).unwrap().id;
    s.cities.get_mut(&id).unwrap().hp = 10;
    // Only walled cities fire.
    s.cities
        .get_mut(&id)
        .unwrap()
        .buildings
        .insert(BuildingKind::Walls);
    let spearman = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Spearman)
        .unwrap()
        .id;
    s.units.get_mut(&spearman).unwrap().pos = at(11, 8);
    let archer = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Archer)
        .unwrap()
        .id;
    s.units.get_mut(&archer).unwrap().pos = at(10, 7);
    visibility::update_explored(&mut s);
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(validate_orders(&s, ME, &orders).is_empty());
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
    assert_eq!(next.cities[&id].owner, ME, "{events:?}");
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Battle {
            attacker: events::Attacker::City { .. },
            ..
        }
    )));
}

#[test]
fn unseen_enemy_positions_cannot_change_the_plan() {
    let mut a = scenario();
    let hidden = spawn_unit(&mut a, ENEMY, UnitKind::Spearman, at(27, 20));
    visibility::update_explored(&mut a);
    let mut b = a.clone();
    b.units.get_mut(&hidden).unwrap().pos = at(26, 21);
    visibility::update_explored(&mut b);
    let av = player_view(&a, ME);
    let bv = player_view(&b, ME);
    assert_eq!(av, bv);
    assert_eq!(
        HumanPlaybookV6Brain::default().orders(&av),
        HumanPlaybookV6Brain::default().orders(&bv)
    );
}

#[test]
fn a_settler_raid_keeps_the_capturer_in_position_for_war_activation() {
    let mut s = scenario();
    spawn_unit(&mut s, ENEMY, UnitKind::Settler, at(10, 10));
    visibility::update_explored(&mut s);
    let raider = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Pikeman)
        .unwrap()
        .id;
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(orders.contains(&Order::DeclareWar {
        against: TeamId(1),
        breach: false
    }));
    assert!(orders.contains(&Order::SkipUnit { unit: raider }));
    assert!(!orders.iter().any(|o| matches!(o,
        Order::MoveUnitPath { unit, .. } | Order::RangedAttack { unit, .. } if *unit == raider)));
    assert!(validate_orders(&s, ME, &orders).is_empty());
}

#[test]
fn withdraws_before_next_turns_unavoidable_city_shot_becomes_lethal() {
    let mut s = scenario();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    s.units.retain(|_, u| u.kind != UnitKind::Scout);
    for c in s.cities.values_mut().filter(|c| c.owner == ENEMY) {
        c.buildings.insert(BuildingKind::Walls);
    }
    let archer = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Archer)
        .unwrap()
        .id;
    s.units.get_mut(&archer).unwrap().pos = at(10, 8);
    visibility::update_explored(&mut s);
    let view = player_view(&s, ME);
    let city = view.cities.iter().find(|c| c.owner == ENEMY).unwrap();
    let original = view.units.iter().find(|u| u.id == archer).unwrap();
    let hp = (40..=100)
        .find(|hp| {
            let mut u = original.clone();
            u.hp = *hp;
            let shot = forecast_city_strike(&view, city, &u, u.pos)
                .unwrap()
                .damage_at_roll(combat::ROLL_MAX);
            if shot >= u.hp {
                return false;
            }
            u.hp -= shot;
            forecast_city_strike(&view, city, &u, u.pos)
                .unwrap()
                .damage_at_roll(combat::ROLL_MAX)
                >= u.hp
        })
        .expect("a shot survivable now but lethal next turn");
    s.units.get_mut(&archer).unwrap().hp = hp;
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(
        !orders
            .iter()
            .any(|o| matches!(o, Order::RangedAttack { unit, .. } if *unit == archer))
    );
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnitPath { unit, to, .. }
        if *unit == archer && to.distance(city.pos) > 2)),
        "{orders:?}"
    );
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
    let escaped = next
        .units
        .get(&archer)
        .expect("survives the shot before moving");
    assert!(escaped.pos.distance(city.pos) > 2, "{events:?}");
}

#[test]
fn funds_four_reinforcements_over_two_turns_without_replacing_city_builds() {
    let mut s = scenario();
    s.player_mut(ME).gold = 2500;
    let mut brain = HumanPlaybookV6Brain::default();
    let mut purchases = 0;
    for _ in 0..2 {
        let orders = brain.orders(&player_view(&s, ME));
        assert!(
            validate_orders(&s, ME, &orders).is_empty(),
            "{:?}",
            validate_orders(&s, ME, &orders)
        );
        let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
        purchases += events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Event::ItemBought {
                        owner: ME,
                        item: ProductionItem::Unit { .. },
                        ..
                    }
                )
            })
            .count();
        s = next;
    }
    assert!(purchases >= 4, "only {purchases} purchases");
    assert!(s.player(ME).gold >= 30);
}

#[test]
fn capture_infantry_can_arrive_from_two_tiles_after_the_finishing_volley() {
    let mut s = scenario();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    let city = s.city_at(at(12, 8)).unwrap().id;
    s.cities.get_mut(&city).unwrap().hp = 5;
    let spearman = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Spearman)
        .unwrap()
        .id;
    let archer = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Archer)
        .unwrap()
        .id;
    s.units.get_mut(&spearman).unwrap().pos = at(10, 8);
    s.units.get_mut(&archer).unwrap().pos = at(11, 9);
    visibility::update_explored(&mut s);
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(validate_orders(&s, ME, &orders).is_empty());
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
    assert_eq!(next.cities[&city].owner, ME, "{events:?}");
}

#[test]
fn doomed_infantry_is_not_counted_as_a_surviving_capture_unit() {
    let mut s = scenario();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    let city = s.city_at(at(12, 8)).unwrap().id;
    s.cities.get_mut(&city).unwrap().hp = 5;
    let spearman = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Spearman)
        .unwrap()
        .id;
    s.units.get_mut(&spearman).unwrap().pos = at(11, 8);
    s.units.get_mut(&spearman).unwrap().hp = 1;
    visibility::update_explored(&mut s);
    let view = player_view(&s, ME);
    let orders = HumanPlaybookV6Brain::default().orders(&view);
    assert!(!orders.iter().any(|o| matches!(o, Order::MoveUnitPath { unit, to, .. } if *unit == spearman && *to == at(12, 8))));
    assert!(validate_orders(&s, ME, &orders).is_empty());
}

#[test]
fn adjacent_infantry_finish_a_city_in_the_same_movement_round() {
    let mut s = scenario();
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = diplomacy::RelationState::War;
    }
    let city = s.city_at(at(12, 8)).unwrap().id;
    s.cities.get_mut(&city).unwrap().hp = 90;
    let spearman = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Spearman)
        .unwrap()
        .id;
    let pikeman = s
        .units
        .values()
        .find(|u| u.owner == ME && u.kind == UnitKind::Pikeman)
        .unwrap()
        .id;
    s.units.get_mut(&spearman).unwrap().pos = at(11, 8);
    s.units.get_mut(&pikeman).unwrap().pos = at(12, 9);
    visibility::update_explored(&mut s);
    let view = player_view(&s, ME);
    for id in [spearman, pikeman] {
        assert!(
            forecast_attack_mode(&view, id, at(12, 8), AttackMode::Melee)
                .unwrap()
                .win_chance
                < 100
        );
    }
    let orders = HumanPlaybookV6Brain::default().orders(&view);
    assert!(validate_orders(&s, ME, &orders).is_empty());
    for id in [spearman, pikeman] {
        assert!(orders.iter().any(|o| matches!(o, Order::MoveUnitPath { unit,to,.. } if *unit == id && *to == at(12,8))), "{orders:?}");
    }
    let (next, events) = resolve_turn(&s, &BTreeMap::from([(ME, orders)]));
    assert_eq!(next.cities[&city].owner, ME, "{events:?}");
}

#[test]
fn sparta_can_declare_for_a_veteran_expedition_while_two_guards_stay_home() {
    let mut s = scenario();
    s.player_mut(ME).faction = Faction::Sparta;
    s.units.retain(|_, u| u.owner != ME);
    let mut guards = Vec::new();
    for h in [at(5, 8), at(5, 14)] {
        let city = s.city_at(h).unwrap().id;
        s.cities
            .get_mut(&city)
            .unwrap()
            .buildings
            .insert(BuildingKind::Walls);
        guards.push(spawn_unit(&mut s, ME, UnitKind::Spearman, h));
    }
    spawn_unit(&mut s, ME, UnitKind::Pikeman, at(9, 8));
    visibility::update_explored(&mut s);
    let orders = HumanPlaybookV6Brain::default().orders(&player_view(&s, ME));
    assert!(
        validate_orders(&s, ME, &orders).is_empty(),
        "{:?}",
        validate_orders(&s, ME, &orders)
    );
    assert!(
        orders.contains(&Order::DeclareWar {
            against: TeamId(1),
            breach: false
        }),
        "{orders:?}"
    );
    for unit in guards {
        assert!(orders.contains(&Order::SkipUnit { unit }), "{orders:?}");
    }
}

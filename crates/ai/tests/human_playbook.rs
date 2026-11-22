//! Check the human playbook across real fog, diplomacy and simultaneous combat boundaries.
use legatum_ai::{Brain, HumanPlaybookBrain};
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
    let orders = HumanPlaybookBrain::default().orders(&player_view(&s, ME));
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
    let orders = HumanPlaybookBrain::default().orders(&player_view(&s, ME));
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
        HumanPlaybookBrain::default().orders(&av),
        HumanPlaybookBrain::default().orders(&bv)
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
    let orders = HumanPlaybookBrain::default().orders(&player_view(&s, ME));
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
    let orders = HumanPlaybookBrain::default().orders(&player_view(&s, ME));
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

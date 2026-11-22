use super::*;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::rules::cities::found_city;
use legatum_core::state::Map;
use legatum_core::{GameSetup, Hex, Order, ProductionItem, Terrain, resolve_turn};

fn scenario() -> (GameState, CityId, UnitId) {
    let mut setup = GameSetup::all_ai(93);
    setup.players.truncate(2);
    setup.legacy_all_war = true;
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Plains));
    let home = spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, Hex::new(4, 5));
    found_city(&mut s, home, &mut Vec::new()).unwrap();
    let enemy = spawn_unit(&mut s, PlayerId(1), UnitKind::Settler, Hex::new(12, 5));
    let city = found_city(&mut s, enemy, &mut Vec::new()).unwrap();
    let archer = spawn_unit(&mut s, PlayerId(0), UnitKind::Archer, Hex::new(10, 5));
    s.player_mut(PlayerId(0)).gold = 1000;
    s.cities.get_mut(&city).unwrap().hp = 70;
    (s, city, archer)
}

#[test]
fn counters_count_capture_direction_real_spending_and_turn_samples() {
    let (mut s, city, _) = scenario();
    let me = PlayerId(0);
    let mut c = Counters::default();
    let mut observer = Observer::default();
    let settler = spawn_unit(&mut s, me, UnitKind::Settler, Hex::new(8, 8));
    spawn_unit(&mut s, me, UnitKind::Scout, Hex::new(7, 7));
    observer.before(&mut c, &s, me);
    assert_eq!(
        (c.escorted_settler_turns, c.unescorted_settler_turns),
        (0, 1)
    );
    spawn_unit(&mut s, me, UnitKind::Spearman, Hex::new(8, 8));
    observer.before(&mut c, &s, me);
    assert_eq!(
        (c.escorted_settler_turns, c.unescorted_settler_turns),
        (1, 1)
    );
    observer.after(
        &mut c,
        &s,
        me,
        1,
        &[
            Event::SettlerCaptured {
                unit: settler,
                from: me,
                to: PlayerId::BARBARIAN,
                at: Hex::new(8, 8),
            },
            Event::SettlerCaptured {
                unit: settler,
                from: PlayerId::BARBARIAN,
                to: me,
                at: Hex::new(8, 8),
            },
            Event::ItemBought {
                city,
                owner: me,
                item: ProductionItem::Unit {
                    unit: UnitKind::Archer,
                },
                gold: 123,
            },
            Event::UnitUpgraded {
                unit: settler,
                owner: me,
                to: UnitKind::Pikeman,
                gold: 45,
            },
        ],
        None,
    );
    assert_eq!((c.settlers_captured_by, c.settlers_lost), (1, 1));
    assert_eq!(c.gold_spent, 168);
    assert_eq!(c.gold_hoarded_peak, 1000);
    assert_eq!(c.scouts_alive_by_turn[&1], 1);
    s.units.retain(|_, u| u.kind != UnitKind::Scout);
    observer.after(&mut c, &s, me, 2, &[], None);
    assert_eq!(c.scouts_alive_by_turn[&2], 0);
    let json = serde_json::to_value(&c).unwrap();
    assert_eq!(json["settlers_lost"], 1);
    assert_eq!(serde_json::from_value::<Counters>(json).unwrap(), c);
}

#[test]
fn same_turn_declaration_counts_a_surprise_war_settler_loss() {
    let (s, _, _) = scenario();
    let me = PlayerId(0);
    let enemy = PlayerId(1);
    let mut counters = Counters::default();
    let mut observer = Observer::default();
    let relation = legatum_core::diplomacy::Relation {
        a: s.team_of(me),
        b: s.team_of(enemy),
        state: legatum_core::diplomacy::RelationState::War,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: Some(1),
        declarers: vec![s.team_of(enemy)],
    };
    observer.after(
        &mut counters,
        &s,
        me,
        1,
        &[
            Event::WarDeclared { relation },
            Event::SettlerCaptured {
                unit: UnitId(99),
                from: me,
                to: enemy,
                at: Hex::new(8, 8),
            },
        ],
        None,
    );
    assert_eq!(counters.settlers_lost_to_surprise_war, 1);
}

#[test]
fn war_battles_are_counted_once_per_declared_war_not_per_shot_or_barbarian_fight() {
    let (s, _, archer) = scenario();
    let me = PlayerId(0);
    let mut relation = legatum_core::diplomacy::Relation {
        a: TeamId(0),
        b: TeamId(1),
        state: legatum_core::diplomacy::RelationState::War,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: Some(1),
        declarers: vec![TeamId(0)],
    };
    let mut c = Counters::default();
    let mut observer = Observer::default();
    observer.after(
        &mut c,
        &s,
        me,
        1,
        &[Event::WarDeclared {
            relation: relation.clone(),
        }],
        None,
    );
    assert_eq!(c.wars_with_battles, 0);
    let orders = BTreeMap::from([(
        me,
        vec![Order::RangedAttack {
            unit: archer,
            target: Hex::new(12, 5),
        }],
    )]);
    let (next, events) = resolve_turn(&s, &orders);
    assert!(events.iter().any(|e| matches!(e, Event::Battle { .. })));
    observer.after(&mut c, &next, me, 2, &events, None);
    observer.after(&mut c, &next, me, 3, &events, None);
    assert_eq!(c.wars_with_battles, 1);
    observer.after(
        &mut c,
        &next,
        me,
        4,
        &[Event::PeaceEstablished {
            relation: relation.clone(),
        }],
        None,
    );
    relation.war_id = Some(2);
    observer.after(
        &mut c,
        &next,
        me,
        5,
        &[Event::WarDeclared { relation }],
        None,
    );
    observer.after(&mut c, &next, me, 6, &events, None);
    assert_eq!(c.wars_with_battles, 2);
}

#[test]
fn marches_require_real_quota_progress_and_sieges_exclude_captures_and_casualties() {
    let (mut s, city, archer) = scenario();
    let me = PlayerId(0);
    let mut observer = Observer::default();
    let mut c = Counters::default();
    let march = CoalitionMarch {
        city,
        target: Hex::new(12, 5),
        minimum: 1,
        units: vec![(archer, Hex::new(10, 5))],
    };
    observer.after(&mut c, &s, me, 1, &[], Some(&march));
    assert_eq!(c.coalition_marches, 0, "orders alone never count");
    s.units.get_mut(&archer).unwrap().pos = Hex::new(11, 5);
    observer.after(&mut c, &s, me, 2, &[], Some(&march));
    assert_eq!(c.coalition_marches, 1);
    observer.before(&mut c, &s, me);
    s.units.get_mut(&archer).unwrap().pos = Hex::new(5, 5);
    observer.after(&mut c, &s, me, 3, &[], None);
    assert_eq!(c.sieges_abandoned, 1);
    s.units.get_mut(&archer).unwrap().pos = Hex::new(10, 5);
    observer.before(&mut c, &s, me);
    let mut captured = s.clone();
    captured.cities.get_mut(&city).unwrap().owner = me;
    observer.after(
        &mut c,
        &captured,
        me,
        4,
        &[Event::CityCaptured {
            city,
            from: PlayerId(1),
            to: me,
            bounty: None,
        }],
        None,
    );
    assert_eq!(c.sieges_abandoned, 1);
    s.units.remove(&archer);
    observer.after(&mut c, &s, me, 5, &[], None);
    assert_eq!(c.sieges_abandoned, 1);
}

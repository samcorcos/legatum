//! The Egyptian Chariot Archer (Sam, 2026-11-25): Egypt's Archer with 3 moves, a shot 40%
//! stronger against the melee line, and Forest or Hills without a road take its whole turn.
use crate::experiments::{FactionRule, FactionRuleSet};
use crate::game::spawn_unit;
use crate::hex::Hex;
use crate::orders::Order;
use crate::rules::cities::tests::{P0, P1, at, test_game};
use crate::rules::resolve_turn;
use crate::state::GameState;
use crate::types::*;
use crate::{Faction, combat, data, roads};
use std::collections::BTreeMap;

#[test]
fn the_chariot_archer_is_a_faster_egyptian_archer() {
    let own = data::unit_stats_for(Faction::Egypt, UnitKind::Archer);
    let plain = data::unit_stats(UnitKind::Archer);
    assert_eq!(own.moves, data::CHARIOT_ARCHER_MOVES);
    assert_eq!(own.moves, 3);
    assert_eq!(
        (own.cost, own.ranged, own.range, own.strength),
        (plain.cost, plain.ranged, plain.range, plain.strength)
    );
    assert_eq!(data::special_unit(Faction::Egypt), UnitKind::Archer);
    assert_eq!(
        data::unit_name_for(Faction::Egypt, UnitKind::Archer),
        "Chariot Archer"
    );
    // The Spearman is plain again (the Medjay is retired).
    assert_eq!(
        data::unit_name_for(Faction::Egypt, UnitKind::Spearman),
        "Spearman"
    );
    // Only Egypt's, only the Archer, and switched off by its rule.
    assert_eq!(
        data::unit_stats_for(Faction::Rome, UnitKind::Archer).moves,
        2
    );
    assert_eq!(
        data::unit_stats_for(Faction::Egypt, UnitKind::CompositeArcher).moves,
        2
    );
    let mut off = FactionRuleSet::EMPTY;
    off.insert(FactionRule::EgyptChariotArcher);
    assert_eq!(
        data::unit_stats_for_rules(off, Faction::Egypt, UnitKind::Archer).moves,
        2
    );
}

fn volley(faction: Faction, target: UnitKind, attacker_side: bool, shot: bool) -> i32 {
    let mut b = Breakdown::default();
    combat::add_fight_class_bonus_rules(
        &mut b,
        FactionRuleSet::EMPTY,
        faction,
        UnitKind::Archer,
        target,
        attacker_side,
        shot,
    );
    b.parts
        .iter()
        .filter(|(l, _)| l == combat::CHARIOT_VOLLEY_LABEL)
        .map(|(_, v)| *v)
        .sum()
}

#[test]
fn its_shot_is_forty_percent_stronger_against_foot_soldiers_only() {
    let ranged = data::unit_stats_for(Faction::Egypt, UnitKind::Archer).ranged;
    let expected = ranged * 40 / 100;
    assert!(expected > 0);
    for foot in [UnitKind::Spearman, UnitKind::Phalangite] {
        assert_eq!(
            volley(Faction::Egypt, foot, true, true),
            expected,
            "{foot:?}"
        );
    }
    // Not against riders or archers, not defending, not for anyone else's Archer.
    assert_eq!(volley(Faction::Egypt, UnitKind::Horseman, true, true), 0);
    assert_eq!(volley(Faction::Egypt, UnitKind::Archer, true, true), 0);
    assert_eq!(volley(Faction::Egypt, UnitKind::Spearman, false, true), 0);
    assert_eq!(volley(Faction::Rome, UnitKind::Spearman, true, true), 0);
    // Switched off with its rule.
    let mut off = FactionRuleSet::EMPTY;
    off.insert(FactionRule::EgyptChariotArcher);
    let (_, v) = combat::fight_chariot_volley_for_rules(
        off,
        Faction::Egypt,
        UnitKind::Archer,
        UnitKind::Spearman,
        true,
        true,
    );
    assert_eq!(v, 0);
}

/// An Archer at the west end of a corridor: Grassland, then `rough` from column 4 on.
fn corridor(faction: Faction, rough: Terrain, road: bool) -> (GameState, UnitId) {
    let mut s = test_game(&[(faction, 0), (Faction::India, 1)]);
    for c in 0..10 {
        for r in [4, 6] {
            s.map.get_mut(at(c, r)).unwrap().terrain = Terrain::Mountains;
        }
        let t = s.map.get_mut(at(c, 5)).unwrap();
        t.terrain = if !(c >= 4) { Terrain::Grassland } else { rough };
        if !(!(!(road) || !(c >= 4))) {
        } else {
            t.road = roads::NODE;
        }
    }
    let u = spawn_unit(&mut s, P0, UnitKind::Archer, at(1, 5));
    spawn_unit(&mut s, P1, UnitKind::Settler, at(18, 18));
    spawn_unit(&mut s, P0, UnitKind::Settler, at(1, 18));
    (s, u)
}

fn walk(s: &GameState, u: UnitId, to: Hex) -> GameState {
    let orders = BTreeMap::from([(P0, vec![Order::MoveUnit { unit: u, to }])]);
    resolve_turn(s, &orders).0
}

#[test]
fn it_runs_three_tiles_on_open_ground() {
    let (s, u) = corridor(Faction::Egypt, Terrain::Grassland, false);
    assert_eq!(walk(&s, u, at(8, 5)).units[&u].pos, at(4, 5));
    // Anyone else's Archer walks two.
    let (s, u) = corridor(Faction::Rome, Terrain::Grassland, false);
    assert_eq!(walk(&s, u, at(8, 5)).units[&u].pos, at(3, 5));
}

#[test]
fn forest_and_hills_take_its_whole_turn_unless_a_road_runs_there() {
    for rough in [Terrain::Forest, Terrain::Hills] {
        let (s, u) = corridor(Faction::Egypt, rough, false);
        // Two open tiles, then it stops short of the rough ground with a move left.
        let s = walk(&s, u, at(6, 5));
        assert_eq!(s.units[&u].pos, at(3, 5), "{rough:?}");
        // Next turn it enters one rough tile and stops there.
        let s = walk(&s, u, at(6, 5));
        assert_eq!(s.units[&u].pos, at(4, 5), "{rough:?}");
        assert_eq!(s.units[&u].moves_left, 0);
        let s = walk(&s, u, at(6, 5));
        assert_eq!(s.units[&u].pos, at(5, 5), "{rough:?}");
        // A plain Archer pays the ordinary 2 per rough tile.
        let (s, u) = corridor(Faction::Rome, rough, false);
        let s = walk(&walk(&s, u, at(6, 5)), u, at(6, 5));
        assert_eq!(s.units[&u].pos, at(4, 5), "{rough:?}");
    }
    // On a road a rough tile costs 1 as for everyone.
    let (s, u) = corridor(Faction::Egypt, Terrain::Forest, true);
    assert_eq!(walk(&s, u, at(8, 5)).units[&u].pos, at(4, 5));
    // With its rule off it is a plain Archer.
    let (mut s, u) = corridor(Faction::Egypt, Terrain::Hills, false);
    s.setup
        .rule_experiments
        .faction_rules_off
        .insert(FactionRule::EgyptChariotArcher);
    assert_eq!(walk(&s, u, at(6, 5)).units[&u].pos, at(3, 5));
}

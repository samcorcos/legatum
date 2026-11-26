//! Breakable Walls (2026-11-25; rules 62): the Walls bar takes hits before the city's HP,
//! never heals, breaks for good, and is rebuilt only with Production while not under attack.
use crate::rules::battle::{self, FightKind, Target};
use crate::rules::test_support::{city, game as fixture, hex, unit};
use crate::{combat, economy, events::Event, rules::cities, *};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn walls(level: u8) -> ProductionItem {
    ProductionItem::Building {
        building: if !(level == 0) {
            BuildingKind::UpgradeWalls
        } else {
            BuildingKind::Walls
        },
    }
}

fn walled(s: &mut GameState, level: u8) -> CityId {
    let id = city(s, 1, hex(5, 3), 3);
    let c = s.cities.get_mut(&id).unwrap();
    c.buildings.insert(BuildingKind::Walls);
    c.walls_level = level;
    id
}

#[test]
fn the_walls_bar_grows_fifty_a_level() {
    assert_eq!(
        (1..=9).map(combat::walls_max_hp).collect::<Vec<_>>(),
        [100, 150, 200, 250, 300, 350, 400, 450, 500]
    );
}

#[test]
fn a_hit_lands_on_the_walls_bar_first_and_the_rest_carries_on_at_full_rate() {
    // Half damage to the bar: 40 takes 20 off a full bar.
    assert_eq!(
        combat::city_hit(100, 200, 40, 50),
        combat::CityHit {
            walls: 20,
            hp: 0,
            past: 0
        }
    );
    // 5 left on the bar uses 10 of the hit; the other 30 hits the city's HP in full.
    assert_eq!(
        combat::city_hit(5, 200, 40, 50),
        combat::CityHit {
            walls: 5,
            hp: 30,
            past: 0
        }
    );
    // No Walls: the HP takes it, and what is past the HP carries on.
    assert_eq!(
        combat::city_hit(0, 10, 40, 50),
        combat::CityHit {
            walls: 0,
            hp: 10,
            past: 30
        }
    );
}

#[test]
fn breaking_the_bar_destroys_the_walls_and_the_city_stops_firing() {
    let mut s = fixture(&[0, 1]);
    let c = walled(&mut s, 3);
    s.cities.get_mut(&c).unwrap().walls_damage = combat::walls_max_hp(3) - 5;
    let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
    let mut fight = battle::plan_fight(&s, 1, a, Target::City(c), FightKind::Melee);
    fight.damage_to_defender = 40;
    fight.damage_to_attacker = 0;
    assert_eq!(fight.wall_percent, 50);
    let mut events = Vec::new();
    battle::apply_fights(&mut s, &[fight], &mut events);
    let city = &s.cities[&c];
    assert_eq!(city.wall_tier(), 0, "the Walls are gone");
    assert_eq!(city.walls_damage, 0);
    assert_eq!(city.hp, data::CITY_HP - 30);
    assert!(!combat::city_can_fire(city.wall_tier() > 0, city.hp));
    assert!(events.iter().any(|e| matches!(
        e,
        Event::WallsBreached { city, owner, by: Some(by) } if *city == c && *owner == P1 && *by == P0
    )));
}

#[test]
fn a_hit_the_bar_absorbs_leaves_the_city_hp_and_the_bar_never_heals() {
    let mut s = fixture(&[0, 1]);
    let c = walled(&mut s, 1);
    let a = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
    let mut fight = battle::plan_fight(&s, 1, a, Target::City(c), FightKind::Melee);
    fight.damage_to_defender = 40;
    fight.damage_to_attacker = 0;
    battle::apply_fights(&mut s, &[fight], &mut Vec::new());
    assert_eq!(s.cities[&c].hp, data::CITY_HP);
    assert_eq!(s.cities[&c].walls_hp(), 80);
    // Turns pass without attacks: the city's HP would heal, the bar does not.
    s.turn += 5;
    s.units.clear();
    cities::process_cities(&mut s, &mut Vec::new());
    assert_eq!(s.cities[&c].walls_hp(), 80);
}

#[test]
fn walls_are_not_rebuilt_or_repaired_while_the_city_is_under_attack() {
    let mut s = fixture(&[0, 1]);
    s.player_mut(P1).techs.insert(TechId::Masonry);
    let c = city(&mut s, 1, hex(5, 3), 3);
    let cost = economy::city_production_cost(&s, &s.cities[&c], walls(0));
    s.cities.get_mut(&c).unwrap().producing = Some(walls(0));
    s.cities.get_mut(&c).unwrap().last_attacked_turn = Some(s.turn);
    cities::produce(&mut s, c, cost, &mut Vec::new());
    assert_eq!(s.cities[&c].wall_tier(), 0, "no Walls while under attack");
    assert!(
        s.cities[&c].production_stored >= cost,
        "the Production waits"
    );
    // An at-war unit next to it blocks it too.
    s.cities.get_mut(&c).unwrap().last_attacked_turn = None;
    let enemy = unit(&mut s, 0, UnitKind::Spearman, hex(4, 3));
    cities::produce(&mut s, c, 0, &mut Vec::new());
    assert_eq!(s.cities[&c].wall_tier(), 0, "an enemy is next to the city");
    s.units.remove(&enemy);
    cities::produce(&mut s, c, 0, &mut Vec::new());
    assert_eq!(s.cities[&c].wall_tier(), 1, "built once the attack is over");
    assert_eq!(s.cities[&c].walls_hp(), 100);
}

#[test]
fn upgrade_walls_repairs_the_bar_and_repairs_at_the_era_cap() {
    let mut s = fixture(&[0, 1]);
    s.player_mut(P1).techs.insert(TechId::Masonry);
    let c = walled(&mut s, 1);
    s.cities.get_mut(&c).unwrap().walls_damage = 60;
    // Ancient era: level I is the cap, so Upgrade Walls repairs instead.
    assert!(economy::can_produce(&s, &s.cities[&c], walls(1)));
    let cost = economy::city_production_cost(&s, &s.cities[&c], walls(1));
    s.cities.get_mut(&c).unwrap().producing = Some(walls(1));
    cities::produce(&mut s, c, cost, &mut Vec::new());
    assert_eq!(s.cities[&c].wall_tier(), 1);
    assert_eq!(s.cities[&c].walls_hp(), 100);
    // Undamaged at the cap: nothing to do.
    assert!(!economy::can_produce(&s, &s.cities[&c], walls(1)));
}

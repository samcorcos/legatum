//! The six faction special units (user, 2026-11-14).
use legatum_core::experiments::{FactionRule, FactionRuleSet, RuleExperiments};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::{combat, data, resources, *};

const P: PlayerId = PlayerId(0);

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

/// Stats against the same unit in the hands of a faction with no rule on it.
fn base(kind: UnitKind) -> data::UnitStats {
    data::unit_stats(kind)
}

#[test]
fn eagle_warrior_is_a_stronger_dearer_spearman() {
    let (own, spear) = (
        data::unit_stats_for(Faction::Aztecs, UnitKind::Spearman),
        base(UnitKind::Spearman),
    );
    assert_eq!(own.strength, spear.strength + 4);
    assert_eq!(own.cost, spear.cost + 10);
    assert_eq!(own.moves, spear.moves);
    assert_eq!(
        data::unit_name_for(Faction::Aztecs, UnitKind::Spearman),
        "Eagle Warrior"
    );
    // Only for the Aztecs, and switched off by its rule.
    assert_eq!(
        data::unit_stats_for(Faction::Athens, UnitKind::Spearman).strength,
        spear.strength
    );
    let mut off = RuleExperiments::default();
    off.faction_rules_off
        .insert(FactionRule::AztecsEagleWarrior);
    assert_eq!(
        off.unit_stats_for(Faction::Aztecs, UnitKind::Spearman)
            .strength,
        spear.strength
    );
}

#[test]
fn gaesatae_charge_and_missing_shield_wall_show_in_the_breakdowns() {
    let kind = UnitKind::Phalangite;
    let stats = data::unit_stats_for(Faction::Celts, kind);
    assert_eq!(
        stats.strength,
        base(kind).strength,
        "the base line is the Phalangite's"
    );
    let attack = combat::unit_attack_strength_for(Faction::Celts, kind, 100, 0);
    let ordinary = combat::unit_attack_strength_for(Faction::Athens, kind, 100, 0);
    let charge = data::era_scaled(8, data::unit_era_tier(kind));
    assert_eq!(attack.total, ordinary.total + charge);
    assert!(
        attack
            .parts
            .iter()
            .any(|(l, v)| l == combat::GAESATAE_ATTACK_LABEL && *v == charge)
    );
    let defend = |f| combat::unit_defense_strength_for(f, kind, 100, Terrain::Grassland, 0, 0);
    let (celt, other) = (defend(Faction::Celts), defend(Faction::Athens));
    let loss = data::era_scaled(5, data::unit_era_tier(kind));
    assert_eq!(celt.total, other.total - loss);
    assert!(
        celt.parts
            .iter()
            .any(|(l, v)| l == combat::GAESATAE_DEFENSE_LABEL && *v == -loss)
    );
    // A meeting engagement is a charge on both sides.
    assert_eq!(
        combat::unit_meeting_strength_for(Faction::Celts, kind, 100, 0).total,
        attack.total
    );
    assert_eq!(data::unit_name_for(Faction::Celts, kind), "Gaesatae");
}

#[test]
fn peltast_moves_three_with_the_same_shot() {
    let kind = UnitKind::CompositeArcher;
    let (own, archer) = (data::unit_stats_for(Faction::Athens, kind), base(kind));
    assert_eq!(own.moves, 3);
    assert_eq!(archer.moves, 2);
    assert_eq!(
        (own.strength, own.ranged, own.range, own.cost),
        (archer.strength, archer.ranged, archer.range, archer.cost)
    );
    assert_eq!(data::unit_name_for(Faction::Athens, kind), "Peltast");
}

#[test]
fn war_elephant_is_stronger_slower_and_charges_cities_unpenalised() {
    let kind = UnitKind::Knight;
    let (own, knight) = (data::unit_stats_for(Faction::India, kind), base(kind));
    assert_eq!(own.strength, knight.strength + 8);
    assert_eq!((own.moves, knight.moves), (3, 4));
    assert_eq!(own.cost, knight.cost);
    assert_eq!(combat::mounted_city_penalty(Faction::India, kind, false), 0);
    assert!(combat::mounted_city_penalty(Faction::Athens, kind, false) > 0);
    // The breakdown against a city carries no mounted line for the Elephant, one for a Knight.
    let line = |f| {
        combat::attack_strength_rules(
            FactionRuleSet::EMPTY,
            f,
            f,
            kind,
            100,
            0,
            false,
            false,
            false,
            None,
            true,
        )
    };
    assert!(
        !line(Faction::India)
            .parts
            .iter()
            .any(|(l, _)| l == combat::MOUNTED_CITY_LABEL)
    );
    assert!(
        line(Faction::Athens)
            .parts
            .iter()
            .any(|(l, _)| l == combat::MOUNTED_CITY_LABEL)
    );
    assert_eq!(data::unit_name_for(Faction::India, kind), "War Elephant");
}

#[test]
fn reislaufer_adds_half_its_strength_against_mounted_on_top_of_the_melee_bonus() {
    let kind = UnitKind::Pikeman;
    let strength = data::unit_stats_for(Faction::Switzerland, kind).strength;
    assert_eq!(
        strength,
        base(kind).strength,
        "the base line is the Pikeman's"
    );
    let mut b = Breakdown::default();
    combat::add_fight_class_bonus_rules(
        &mut b,
        FactionRuleSet::EMPTY,
        Faction::Switzerland,
        kind,
        UnitKind::Knight,
        true,
        false,
    );
    let usual = combat::class_bonus_strength(strength);
    assert!(
        b.parts
            .iter()
            .any(|(l, v)| l == combat::MELEE_VS_MOUNTED_LABEL && *v == usual)
    );
    assert!(
        b.parts
            .iter()
            .any(|(l, v)| l == combat::REISLAUFER_LABEL && *v == strength / 2)
    );
    // Not against a spear line, and not for another faction's Musketman.
    let mut none = Breakdown::default();
    combat::add_fight_class_bonus_rules(
        &mut none,
        FactionRuleSet::EMPTY,
        Faction::Switzerland,
        kind,
        UnitKind::Spearman,
        true,
        false,
    );
    assert!(none.parts.is_empty());
    let mut other = Breakdown::default();
    combat::add_fight_class_bonus_rules(
        &mut other,
        FactionRuleSet::EMPTY,
        Faction::Athens,
        kind,
        UnitKind::Knight,
        true,
        false,
    );
    assert!(
        !other
            .parts
            .iter()
            .any(|(l, _)| l == combat::REISLAUFER_LABEL)
    );
    assert_eq!(
        data::unit_name_for(Faction::Switzerland, kind),
        "Reisläufer"
    );
}

#[test]
fn panzer_moves_one_more_and_uses_one_oil() {
    let kind = UnitKind::Tank;
    let (own, tank) = (data::unit_stats_for(Faction::Germany, kind), base(kind));
    assert_eq!(own.moves, tank.moves + 1);
    assert_eq!((own.strength, own.cost), (tank.strength, tank.cost));
    assert_eq!(data::unit_name_for(Faction::Germany, kind), "Panzer");
    let mut setup = GameSetup::all_ai(5);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Germany;
    setup.players[1].faction = Faction::Athens;
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    let oil = resources::unit_need(kind).expect("a Tank uses its era's resource");
    let panzer = spawn_unit(&mut s, P, kind, h(3, 3));
    let tank_user = spawn_unit(&mut s, PlayerId(1), kind, h(10, 3));
    let slots = |s: &GameState, p, id| {
        resources::current_users(s, p)
            .iter()
            .filter(|(u, r)| {
                *r == oil && matches!(u, resources::SupplyUser::Unit { unit } if *unit == id)
            })
            .count()
    };
    // A Panzer needs exactly one Oil, like a Tank, and one Oil supplies it.
    assert_eq!(slots(&s, P, panzer), 1);
    assert_eq!(slots(&s, PlayerId(1), tank_user), 1);
    assert_eq!(resources::in_use(&s, P, oil), 1);
    assert_eq!(resources::in_use(&s, PlayerId(1), oil), 1);
}

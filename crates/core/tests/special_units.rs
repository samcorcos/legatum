//! The six faction special units (user, 2026-11-14) and hired Companies as the seller's version.
use legatum_core::company;
use legatum_core::diplomacy::RelationState;
use legatum_core::experiments::{FactionRule, FactionRuleSet, RuleExperiments};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::{combat, data, resources, rules, *};

const P: PlayerId = PlayerId(0);
const SELLER: PlayerId = PlayerId(1);

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

/// Byzantium (P) at Peace with a seller of the given faction that knows every technology.
fn shop(seller: Faction) -> (GameState, CityId) {
    let mut setup = GameSetup::all_ai(5);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Byzantium;
    setup.players[1].faction = seller;
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    s.player_mut(P).gold = 5000;
    s.player_mut(SELLER).techs.extend(TechId::ALL);
    for i in 0..2 {
        spawn_unit(&mut s, PlayerId(i), UnitKind::Settler, h(i as i32 * 12, 14));
    }
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::Peace;
    }
    let settler = spawn_unit(&mut s, P, UnitKind::Settler, h(4, 4));
    let city = rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    (s, city)
}

fn hired(s: &GameState, city: CityId, seller: PlayerId) -> GameState {
    let unit = data::special_unit(s.player(seller).faction);
    let order = Order::HireCompany { city, unit, seller };
    let (next, _) = resolve_turn(s, &[(P, vec![order])].into_iter().collect());
    next
}

#[test]
fn a_seller_offers_only_its_own_special_unit_priced_as_its_version() {
    for (faction, kind) in [
        (Faction::Aztecs, UnitKind::Spearman),
        (Faction::Sparta, UnitKind::Spearman),
        (Faction::Celts, UnitKind::Phalangite),
        (Faction::Athens, UnitKind::CompositeArcher),
        (Faction::India, UnitKind::Knight),
        (Faction::Switzerland, UnitKind::Pikeman),
        (Faction::Mongols, data::KESHIG_KIND),
    ] {
        let (s, _) = shop(faction);
        let offers = company::offers(&s, P);
        assert_eq!(offers.len(), 1, "{faction:?}");
        assert_eq!(offers[0].unit, kind);
        assert!(offers[0].unavailable.is_none() || kind != UnitKind::Spearman);
        assert_eq!(
            offers[0].price,
            data::unit_stats_for(faction, kind).cost * company::HIRE_PERCENT / 100,
            "{faction:?} is priced as its own version"
        );
    }
}

#[test]
fn a_seller_that_cannot_field_it_yet_says_why_and_is_not_hireable() {
    let (mut s, city) = shop(Faction::Germany);
    s.player_mut(SELLER).techs.clear();
    let offers = company::offers(&s, P);
    assert_eq!(offers.len(), 1);
    let why = offers[0]
        .unavailable
        .clone()
        .expect("no Armoured Warfare yet");
    assert!(
        why.contains("Panzer") && why.contains("Armoured Warfare"),
        "{why}"
    );
    let order = Order::HireCompany {
        city,
        unit: UnitKind::Tank,
        seller: SELLER,
    };
    assert!(!validate_orders(&s, P, &[order]).is_empty());
}

#[test]
fn a_plain_unit_cannot_be_hired_from_a_seller() {
    let (s, city) = shop(Faction::Sparta);
    let order = Order::HireCompany {
        city,
        unit: UnitKind::Archer,
        seller: SELLER,
    };
    let errors = validate_orders(&s, P, &[order]);
    assert!(
        errors[0].message.contains("sells only its own"),
        "{:?}",
        errors[0]
    );
}

#[test]
fn hiring_gives_the_seller_its_own_version_with_its_name_and_stats() {
    let (s, city) = shop(Faction::Aztecs);
    let next = hired(&s, city, SELLER);
    let u = next
        .units_of(P)
        .find(|u| company::is_company(u))
        .expect("a Company");
    assert_eq!(u.kind, UnitKind::Spearman);
    assert_eq!(u.origin, Some(Faction::Aztecs));
    assert_eq!(next.made_by(u), Faction::Aztecs);
    let stats = next.unit_stats_of(u);
    assert_eq!(
        stats.strength,
        data::unit_stats_for(Faction::Aztecs, UnitKind::Spearman).strength
    );
    assert!(stats.strength > data::unit_stats_for(Faction::Byzantium, UnitKind::Spearman).strength);
    let attack = combat::unit_attack_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::Byzantium,
        next.made_by(u),
        u.kind,
        u.hp,
        0,
    );
    assert!(attack.parts.iter().any(|(l, _)| l == "Eagle Warrior"));
}

#[test]
fn a_hired_hoplite_keeps_its_phalanx_but_not_spartas_faction_wide_rules() {
    let (s, city) = shop(Faction::Sparta);
    let mut next = hired(&s, city, SELLER);
    let id = next
        .units_of(P)
        .find(|u| company::is_company(u))
        .unwrap()
        .id;
    let at = next.units[&id].pos;
    // Another Hoplite of the same empire next to it: the phalanx follows the unit.
    let neighbour = at
        .neighbors()
        .into_iter()
        .find(|n| next.map.move_cost(*n).is_some())
        .unwrap();
    let twin = spawn_unit(&mut next, P, UnitKind::Spearman, neighbour);
    next.units.get_mut(&twin).unwrap().origin = Some(Faction::Sparta);
    assert!(rules::battle::has_phalanx_neighbour(
        &next,
        &next.units[&id]
    ));
    // Beside Byzantium's own Spearman (not a Hoplite) there is none.
    next.units.get_mut(&twin).unwrap().origin = None;
    assert!(!rules::battle::has_phalanx_neighbour(
        &next,
        &next.units[&id]
    ));
    // Wounds still count: ignoring them is Sparta's own rule, not the unit's.
    let u = &next.units[&id];
    let wounded = combat::unit_attack_strength_for_rules(
        FactionRuleSet::EMPTY,
        Faction::Byzantium,
        Faction::Sparta,
        u.kind,
        50,
        0,
    );
    assert!(wounded.parts.iter().any(|(l, v)| l == "Wounded" && *v < 0));
    assert_eq!(
        next.unit_stats_of(&next.units[&id]).moves,
        data::unit_stats_for(Faction::Sparta, UnitKind::Spearman).moves
    );
}

#[test]
fn a_hired_keshig_keeps_its_bow_without_the_mongol_move() {
    let (mut s, city) = shop(Faction::Mongols);
    let tile = s.map.get_mut(h(14, 4)).unwrap();
    tile.owner = Some(SELLER);
    tile.resource = Some(Resource::Iron);
    tile.improved = true;
    let next = hired(&s, city, SELLER);
    let u = next.units_of(P).find(|u| company::is_company(u)).unwrap();
    let stats = next.unit_stats_of(u);
    assert!(stats.ranged > 0 && stats.range == data::KESHIG_RANGE);
    assert_eq!(
        stats.moves,
        data::unit_stats(data::KESHIG_KIND).moves,
        "no +1 Mongol move"
    );
    assert_eq!(data::unit_name_for(next.made_by(u), u.kind), "Keshig");
}

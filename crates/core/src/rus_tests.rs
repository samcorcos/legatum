//! Kyivan Rus' (`rus`): marriage alliances, the Druzhina, the Lavra, the goal and the steppe.
use super::*;
use crate::diplomacy::RelationState;
use crate::economy;
use crate::experiments::{FactionRule, FactionRuleSet};
use crate::goals;
use crate::orders::Order;
use crate::rules::cities::{
    self,
    tests::{P0, P1, at, found, test_game},
};
use crate::scoring;

const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

fn ally(s: &mut GameState, a: u8, b: u8) {
    s.diplomacy
        .as_mut()
        .unwrap()
        .start_alliance(TeamId(a), TeamId(b));
}

fn part(s: &mut GameState, a: u8, b: u8) {
    let r = s
        .diplomacy
        .as_mut()
        .unwrap()
        .relations
        .iter_mut()
        .find(|r| r.between(TeamId(a), TeamId(b)))
        .unwrap();
    r.state = RelationState::Peace;
    r.security = None;
}

fn rule_off(s: &mut GameState, rule: FactionRule) {
    let mut off = s.setup.rule_experiments.faction_rules_off;
    off.insert(rule);
    s.setup.rule_experiments.faction_rules_off = off;
}

/// Train one `kind` in `city` and return the new unit.
fn train(s: &mut GameState, city: CityId, kind: UnitKind) -> crate::state::Unit {
    let before: Vec<UnitId> = s.units.keys().copied().collect();
    cities::apply_city_orders(
        s,
        P0,
        &[Order::SetProduction {
            city,
            item: ProductionItem::Unit { unit: kind },
        }],
        &mut Vec::new(),
    );
    cities::produce(s, city, 10_000, &mut Vec::new());
    s.units
        .values()
        .find(|u| !before.contains(&u.id) && u.kind == kind)
        .expect("trained")
        .clone()
}

fn spearman() -> ProductionItem {
    ProductionItem::Unit {
        unit: UnitKind::Spearman,
    }
}

#[test]
fn the_druzhina_is_a_stronger_dearer_rus_knight() {
    let own = data::unit_stats_for(Faction::Rus, UnitKind::Knight);
    let plain = data::unit_stats(UnitKind::Knight);
    assert!(own.strength > plain.strength && own.cost > plain.cost);
    assert_eq!(own.moves, plain.moves);
    assert_eq!(data::special_unit(Faction::Rus), UnitKind::Knight);
    assert_eq!(
        data::unit_name_for(Faction::Rus, UnitKind::Knight),
        "Druzhina"
    );
    assert_eq!(
        data::building_name_for(Faction::Rus, BuildingKind::Library),
        "Lavra"
    );
    let line = |s: &data::UnitStats| (s.strength, s.cost, s.moves);
    let mut off = FactionRuleSet::EMPTY;
    off.insert(FactionRule::RusDruzhina);
    assert_eq!(
        line(&data::unit_stats_for_rules(
            off,
            Faction::Rus,
            UnitKind::Knight
        )),
        line(&plain)
    );
    // Nobody else's Knight is a Druzhina.
    assert_eq!(
        line(&data::unit_stats_for(Faction::Athens, UnitKind::Knight)),
        line(&plain)
    );
}

#[test]
fn rus_cities_train_an_allys_special_unit_at_its_price_and_it_keeps_it() {
    let mut s = test_game(&[
        (Faction::Rus, 0),
        (Faction::Aztecs, 1),
        (Faction::Athens, 2),
    ]);
    let kyiv = found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    found(&mut s, P2, at(4, 14));
    let plain = economy::production_cost(&s, P0, spearman());
    assert_eq!(version_for(&s, P0, UnitKind::Spearman), None);
    let own = train(&mut s, kyiv, UnitKind::Spearman);
    assert_eq!(own.origin, None);

    ally(&mut s, 0, 1);
    assert_eq!(
        version_for(&s, P0, UnitKind::Spearman),
        Some(Faction::Aztecs)
    );
    assert_eq!(
        economy::production_cost(&s, P0, spearman()),
        economy::production_cost(&s, P1, spearman()),
        "the Eagle Warrior's price"
    );
    assert!(economy::production_cost(&s, P0, spearman()) > plain);
    let eagle = train(&mut s, kyiv, UnitKind::Spearman);
    assert_eq!(eagle.origin, Some(Faction::Aztecs));
    assert_eq!(s.made_by(&eagle), Faction::Aztecs);
    assert_eq!(
        data::unit_name_for(s.made_by(&eagle), eagle.kind),
        "Eagle Warrior"
    );
    assert_eq!(
        s.unit_stats_of(&eagle).strength,
        data::unit_stats_for(Faction::Aztecs, UnitKind::Spearman).strength
    );
    // An ally with no version of this kind changes nothing.
    assert_eq!(version_for(&s, P0, UnitKind::Archer), None);

    // The Alliance ends: the Eagle Warrior stays one, new Spearmen are plain again.
    part(&mut s, 0, 1);
    assert_eq!(s.units[&eagle.id].origin, Some(Faction::Aztecs));
    assert_eq!(version_for(&s, P0, UnitKind::Spearman), None);
    assert_eq!(economy::production_cost(&s, P0, spearman()), plain);
    assert_eq!(train(&mut s, kyiv, UnitKind::Spearman).origin, None);
}

#[test]
fn the_buy_quote_matches_the_allied_price() {
    let mut s = test_game(&[(Faction::Rus, 0), (Faction::Aztecs, 1)]);
    let kyiv = found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    ally(&mut s, 0, 1);
    s.player_mut(P0).gold = 5_000;
    cities::apply_city_orders(
        &mut s,
        P0,
        &[Order::SetProduction {
            city: kyiv,
            item: spearman(),
        }],
        &mut Vec::new(),
    );
    let quote = economy::buy_quote(&s, &s.cities[&kyiv], spearman()).unwrap();
    assert_eq!(
        quote.production.total,
        economy::production_cost(&s, P0, spearman())
    );
    // `buy_cost` checks itself against the quote in debug builds.
    assert!(economy::buy_cost(&s, &s.cities[&kyiv]).is_some());
}

#[test]
fn the_knight_stays_the_druzhina_whatever_the_allies() {
    let mut s = test_game(&[(Faction::Rus, 0), (Faction::India, 1)]);
    found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    ally(&mut s, 0, 1);
    assert_eq!(data::special_unit(Faction::India), UnitKind::Knight);
    assert_eq!(version_for(&s, P0, UnitKind::Knight), None);
    let knight = ProductionItem::Unit {
        unit: UnitKind::Knight,
    };
    assert_eq!(cost_faction(&s, P0, knight), Faction::Rus);
    assert_eq!(
        economy::production_cost(&s, P0, knight),
        data::unit_stats_for(Faction::Rus, UnitKind::Knight).cost
    );
}

#[test]
fn the_older_alliance_gives_a_shared_slot_and_switches_turn_it_off() {
    let mut s = test_game(&[
        (Faction::Rus, 0),
        (Faction::Aztecs, 1),
        (Faction::Sparta, 2),
    ]);
    ally(&mut s, 0, 1);
    ally(&mut s, 0, 2);
    // Sparta's Alliance is the older one.
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        if r.between(TeamId(0), TeamId(1)) {
            r.security.as_mut().unwrap().started_turn = 9;
        }
    }
    assert_eq!(
        version_for(&s, P0, UnitKind::Spearman),
        Some(Faction::Sparta)
    );
    assert_eq!(allies(&s, P0), vec![TeamId(2), TeamId(1)]);
    // Only the Rus marry: the Aztecs' own Spearman is their own.
    assert_eq!(version_for(&s, P1, UnitKind::Spearman), None);
    rule_off(&mut s, FactionRule::RusMarriage);
    assert_eq!(version_for(&s, P0, UnitKind::Spearman), None);
    assert_eq!(
        economy::production_cost(&s, P0, spearman()),
        data::unit_stats_for(Faction::Rus, UnitKind::Spearman).cost
    );
}

#[test]
fn the_view_reads_the_same_versions_and_offers() {
    let mut s = test_game(&[
        (Faction::Rus, 0),
        (Faction::Aztecs, 1),
        (Faction::Athens, 2),
    ]);
    found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    found(&mut s, P2, at(4, 14));
    let v = crate::view::player_view(&s, P0);
    assert_eq!(known_new_versions(&v, TeamId(1)), vec![UnitKind::Spearman]);
    assert_eq!(
        known_new_versions(&v, TeamId(2)),
        vec![UnitKind::CompositeArcher]
    );
    ally(&mut s, 0, 1);
    let v = crate::view::player_view(&s, P0);
    assert_eq!(
        known_version_for(&v, UnitKind::Spearman),
        Some(Faction::Aztecs)
    );
    assert_eq!(known_alliances(&v), 1);
    assert!(known_new_versions(&v, TeamId(1)).is_empty());
    // A rival's view never marries.
    let rival = crate::view::player_view(&s, P1);
    assert!(known_new_versions(&rival, TeamId(0)).is_empty());
}

#[test]
fn the_lavra_adds_science_for_each_alliance_and_the_cached_view_follows() {
    let mut s = test_game(&[
        (Faction::Rus, 0),
        (Faction::Aztecs, 1),
        (Faction::Athens, 2),
    ]);
    let kyiv = found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    found(&mut s, P2, at(4, 14));
    s.cities
        .get_mut(&kyiv)
        .unwrap()
        .buildings
        .insert(BuildingKind::Library);
    let science = |s: &GameState| economy::city_yields(s, &s.cities[&kyiv]).total.science;
    let mut cache = crate::yields_cache::ViewCache::default();
    let viewed = |s: &GameState, cache: &mut crate::yields_cache::ViewCache| {
        let public = crate::view::public_view(s);
        crate::view::player_view_cached(s, P0, &public, crate::view::ViewDetail::Full, cache)
            .cities
            .iter()
            .find(|c| c.id == kyiv)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .yields
            .total
            .science
    };
    let none = science(&s);
    assert_eq!(viewed(&s, &mut cache), none);
    ally(&mut s, 0, 1);
    ally(&mut s, 0, 2);
    assert_eq!(science(&s), none + 2 * LAVRA_SCIENCE_PER_ALLIANCE);
    assert_eq!(
        viewed(&s, &mut cache),
        science(&s),
        "the cache saw the Alliances"
    );
    rule_off(&mut s, FactionRule::RusLavra);
    assert_eq!(science(&s), none);
    // Another people's Library gains nothing from Alliances.
    let mut t = test_game(&[(Faction::Athens, 0), (Faction::Aztecs, 1)]);
    let athens = found(&mut t, P0, at(4, 4));
    found(&mut t, P1, at(14, 4));
    t.cities
        .get_mut(&athens)
        .unwrap()
        .buildings
        .insert(BuildingKind::Library);
    let before = economy::city_yields(&t, &t.cities[&athens]).total.science;
    t.diplomacy
        .as_mut()
        .unwrap()
        .start_alliance(TeamId(0), TeamId(1));
    assert_eq!(
        economy::city_yields(&t, &t.cities[&athens]).total.science,
        before
    );
}

#[test]
fn the_goal_holds_only_while_three_living_allies_stand() {
    let mut s = test_game(&[
        (Faction::Rus, 0),
        (Faction::Aztecs, 1),
        (Faction::Athens, 2),
        (Faction::Germany, 3),
    ]);
    found(&mut s, P0, at(4, 4));
    found(&mut s, P1, at(14, 4));
    found(&mut s, P2, at(4, 14));
    found(&mut s, P3, at(14, 14));
    assert_eq!(
        goals::default_rule(Faction::Rus),
        goals::GoalRule::Marriages
    );
    let spec = goals::spec(Faction::Rus);
    assert_eq!(spec.kind, goals::GoalKind::Live);
    assert_eq!(
        (spec.vp, spec.target),
        (data::FACTION_GOAL_VP, GOAL_ALLIANCES)
    );
    ally(&mut s, 0, 1);
    ally(&mut s, 0, 2);
    assert!(!goals::reached(&s, P0));
    assert_eq!(goals::view(&s, P0).unwrap().value, 2);
    ally(&mut s, 0, 3);
    assert!(goals::reached(&s, P0));
    scoring::update(&mut s, &mut Vec::new());
    assert!(s.player(P0).faction_goal_done);
    // An ally that falls takes the points with it.
    s.player_mut(P3).alive = false;
    assert!(!goals::reached(&s, P0));
    scoring::update(&mut s, &mut Vec::new());
    assert!(!s.player(P0).faction_goal_done);
    s.player_mut(P3).alive = true;
    part(&mut s, 0, 2);
    assert!(!goals::reached(&s, P0));
    ally(&mut s, 0, 2);
    assert!(goals::reached(&s, P0));
    rule_off(&mut s, FactionRule::RusGoal);
    assert!(!goals::reached(&s, P0));
}

#[test]
fn camps_raid_rus_cities_again_sooner() {
    let mut s = test_game(&[(Faction::Rus, 0), (Faction::Athens, 1)]);
    let kyiv = found(&mut s, P0, at(4, 4));
    let athens = found(&mut s, P1, at(14, 14));
    let ordinary = crate::rules::barbarians::RAID_COOLDOWN;
    assert_eq!(raid_cooldown(&s, kyiv, ordinary), STEPPE_RAID_COOLDOWN);
    assert!(STEPPE_RAID_COOLDOWN < ordinary);
    assert_eq!(raid_cooldown(&s, athens, ordinary), ordinary);
    assert_eq!(raid_cooldown(&s, CityId(999), ordinary), ordinary);
    rule_off(&mut s, FactionRule::RusSteppe);
    assert_eq!(raid_cooldown(&s, kyiv, ordinary), ordinary);
}

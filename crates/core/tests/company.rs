//! Mercenary Companies: Byzantium hires armed units from empires it is not at War with
//! (`company`).
use legatum_core::company;
use legatum_core::diplomacy::RelationState;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::Map;
use legatum_core::*;

const P: PlayerId = PlayerId(0);
const SELLER: PlayerId = PlayerId(1);

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

/// Player 0 (Byzantium) with one city and 1,000 Gold; player 1 (Sparta) knows every tech. At Peace.
fn state() -> (GameState, CityId) {
    let mut setup = GameSetup::all_ai(5);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Byzantium;
    setup.players[1].faction = Faction::Sparta;
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    s.player_mut(P).gold = 1000;
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

fn hire(city: CityId, unit: UnitKind) -> Order {
    Order::HireCompany {
        city,
        unit,
        seller: SELLER,
    }
}

fn errors(s: &GameState, orders: &[Order]) -> Vec<String> {
    validate_orders(s, P, orders)
        .into_iter()
        .map(|e| e.message)
        .collect()
}

fn companies(s: &GameState) -> Vec<&state::Unit> {
    s.units_of(P).filter(|u| company::is_company(u)).collect()
}

#[test]
fn the_shop_lists_what_a_met_peaceful_seller_can_field_and_only_for_byzantium() {
    let (mut s, _) = state();
    let offers = company::offers(&s, P);
    assert!(
        offers
            .iter()
            .any(|o| o.seller == SELLER && o.unit == UnitKind::Soldier)
    );
    assert!(
        offers
            .iter()
            .all(|o| company::sold_by(s.player(o.seller).faction, o.unit)),
        "no Scouts, Settlers, the Heir, aircraft or hunters"
    );
    let soldier = offers
        .iter()
        .find(|o| o.unit == UnitKind::Soldier)
        .unwrap()
        .price;
    assert_eq!(
        soldier,
        data::unit_stats_for(Faction::Byzantium, UnitKind::Soldier).cost,
        "100% of the Production cost, half the ordinary purchase"
    );
    // A seller without the tech sells nothing it lacks.
    s.player_mut(SELLER).techs.clear();
    let bare = company::offers(&s, P);
    assert!(
        bare.iter()
            .all(|o| { data::item_requires(ProductionItem::Unit { unit: o.unit }).is_none() })
    );
    // At War, no shop.
    s.diplomacy.as_mut().unwrap().relations[0].state = RelationState::War;
    assert!(company::offers(&s, P).is_empty());
    // Other factions have none.
    s.player_mut(P).faction = Faction::Switzerland;
    s.diplomacy.as_mut().unwrap().relations[0].state = RelationState::Peace;
    assert!(company::offers(&s, P).is_empty());
}

#[test]
fn a_resource_company_needs_an_improved_seller_deposit_but_no_spare_supply() {
    let (mut s, city) = state();
    // Byzantium's own special unit is the Pikeman line's Trooper, which needs Iron.
    s.player_mut(SELLER).faction = Faction::Byzantium;
    let deposit = h(14, 4);
    let tile = s.map.get_mut(deposit).unwrap();
    tile.owner = Some(SELLER);
    tile.resource = Some(Resource::Iron);
    assert!(!company::seller_can_field(&s, SELLER, UnitKind::Trooper));
    s.map.get_mut(deposit).unwrap().improved = true;
    assert!(company::seller_can_field(&s, SELLER, UnitKind::Trooper));
    for i in 0..3 {
        spawn_unit(&mut s, SELLER, UnitKind::Trooper, h(16 + i, 4));
    }
    resources::update_supply(&mut s, &mut Vec::new());
    assert_eq!(resources::spare(&s, SELLER, Resource::Iron, None), 0);
    assert!(company::seller_can_field(&s, SELLER, UnitKind::Trooper));
    let order = hire(city, UnitKind::Trooper);
    assert!(errors(&s, std::slice::from_ref(&order)).is_empty());
    let (next, _) = resolve_turn(&s, &[(P, vec![order])].into_iter().collect());
    assert!(
        companies(&next)
            .iter()
            .any(|unit| unit.kind == UnitKind::Trooper)
    );
    assert_eq!(resources::spare(&next, SELLER, Resource::Iron, None), 0);
}

#[test]
fn hiring_pays_the_seller_in_full_and_the_company_appears_with_a_contract() {
    let (s, city) = state();
    let price = company::price(&s, P, SELLER, UnitKind::Soldier);
    let order = hire(city, UnitKind::Soldier);
    assert!(errors(&s, std::slice::from_ref(&order)).is_empty());
    let seller_gold = s.player(SELLER).gold;
    let (next, events) = resolve_turn(&s, &[(P, vec![order])].into_iter().collect());
    assert!(events.iter().any(
        |e| matches!(e, Event::CompanyHired { gold, until, .. } if *gold == price && *until == s.turn + company::COMPANY_TURNS)
    ));
    let hired = companies(&next);
    assert_eq!(hired.len(), 1);
    let u = hired[0];
    assert_eq!(u.kind, UnitKind::Soldier);
    assert_eq!(u.origin, Some(Faction::Sparta));
    assert_eq!(u.contract_until, Some(s.turn + company::COMPANY_TURNS));
    assert!(
        u.xp >= rules::promotion::threshold(1).unwrap(),
        "level 1 at the start"
    );
    // The seller got every Gold of the price (its own turn's income aside).
    assert!(next.player(SELLER).gold >= seller_gold + price);
    // Outside the levy, with its own upkeep line.
    assert_eq!(economy::levy_count(&next, P), 0);
    let upkeep = economy::upkeep_breakdown(&next, P);
    assert!(
        upkeep
            .parts
            .iter()
            .any(|(label, v)| label == "Mercenary Companies" && *v == company::UPKEEP)
    );
}

#[test]
fn companies_ignore_the_levy_cap_and_a_city_hires_once_a_turn() {
    let (mut s, city) = state();
    for i in 0..2 {
        spawn_unit(&mut s, P, UnitKind::Soldier, h(5 + i, 4));
    }
    assert_eq!(economy::levy(&s, P), Some((2, 2)));
    let own = errors(
        &s,
        &[Order::BuyItem {
            city,
            item: ProductionItem::Unit {
                unit: UnitKind::Soldier,
            },
        }],
    );
    assert_eq!(own.len(), 1, "levies are full: {own:?}");
    assert!(errors(&s, &[hire(city, UnitKind::Soldier)]).is_empty());
    let twice = errors(
        &s,
        &[hire(city, UnitKind::Soldier), hire(city, UnitKind::Soldier)],
    );
    assert_eq!(twice.len(), 1, "one purchase per city per turn: {twice:?}");
}

#[test]
fn only_byzantium_hires_and_a_seller_at_war_sells_nothing() {
    let (mut s, city) = state();
    s.diplomacy.as_mut().unwrap().relations[0].state = RelationState::War;
    let at_war = errors(&s, &[hire(city, UnitKind::Soldier)]);
    assert_eq!(at_war.len(), 1);
    assert!(at_war[0].contains("not at War"), "{at_war:?}");
    s.diplomacy.as_mut().unwrap().relations[0].state = RelationState::Peace;
    s.player_mut(P).faction = Faction::Germany;
    let other = errors(&s, &[hire(city, UnitKind::Soldier)]);
    assert!(other[0].contains("Only Byzantium"), "{other:?}");
}

#[test]
fn a_company_goes_home_after_its_last_turn_and_cannot_be_upgraded() {
    let (mut s, _) = state();
    let id = spawn_unit(&mut s, P, UnitKind::Soldier, h(5, 4));
    s.units.get_mut(&id).unwrap().contract_until = Some(s.turn + 1);
    s.units.get_mut(&id).unwrap().origin = Some(Faction::Sparta);
    s.player_mut(P).techs.extend(TechId::ALL);
    let blocked = errors(&s, &[Order::UpgradeUnit { unit: id }]);
    assert!(
        blocked.iter().any(|m| m.contains("hired Company")),
        "{blocked:?}"
    );
    let (s1, _) = resolve_turn(&s, &Default::default());
    assert!(s1.units.contains_key(&id), "serves its last turn");
    let (s2, events) = resolve_turn(&s1, &Default::default());
    assert!(!s2.units.contains_key(&id));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::CompanyDeparted { owner, .. } if *owner == P))
    );
}

#[test]
fn an_unpaid_company_turns_barbarian_before_any_levy_is_lost() {
    let (mut s, _) = state();
    s.setup.barbarians = false;
    s.barbarians = None;
    let levy = spawn_unit(&mut s, P, UnitKind::Soldier, h(5, 4));
    for i in 0..3 {
        let id = spawn_unit(&mut s, P, UnitKind::Soldier, h(6 + i, 4));
        s.units.get_mut(&id).unwrap().contract_until = Some(s.turn + 20);
        s.units.get_mut(&id).unwrap().origin = Some(Faction::Sparta);
    }
    s.player_mut(P).gold = 0;
    let mut turned = false;
    let mut cur = s;
    for _ in 0..12 {
        let (next, events) = resolve_turn(&cur, &Default::default());
        turned |= events
            .iter()
            .any(|e| matches!(e, Event::CompanyTurned { .. }));
        cur = next;
        if turned {
            break;
        }
    }
    assert!(
        turned,
        "no income, so the treasury cannot pay the Companies"
    );
    assert!(cur.units.contains_key(&levy), "the levy unit is kept");
    let barbarian = cur
        .units
        .values()
        .filter(|u| u.owner.is_barbarian() && u.kind == UnitKind::Soldier)
        .count();
    assert!(barbarian >= 1, "the unpaid Company stands as a barbarian");
    assert!(cur.barbarians.as_ref().unwrap().camps.is_empty());
    let mutineer = cur
        .units
        .values()
        .find(|u| u.owner.is_barbarian() && u.kind == UnitKind::Soldier)
        .unwrap();
    let (mutineer_id, mutineer_pos) = (mutineer.id, mutineer.pos);
    let target = mutineer_pos
        .neighbors()
        .into_iter()
        .find(|h| cur.map.get(*h).is_some() && cur.units_at(*h).next().is_none())
        .unwrap();
    let captive = spawn_unit(&mut cur, P, UnitKind::Settler, target);
    rules::barbarians::plan_turn(&mut cur, &mut vec![]);
    assert_eq!(
        cur.units[&mutineer_id].goto,
        Some(target),
        "the mutineer plans a capture"
    );
    let (next, _) = resolve_turn(&cur, &Default::default());
    assert!(
        next.units[&captive].owner.is_barbarian(),
        "the mutineer acts on its plan"
    );
    assert!(
        next.barbarians.as_ref().unwrap().camps.is_empty(),
        "natural camps stay off"
    );
}

#[test]
fn companies_use_none_of_the_employers_supply() {
    let (mut s, _) = state();
    let id = spawn_unit(&mut s, P, UnitKind::Trooper, h(5, 4));
    let before = resources::current_users(&s, P).len();
    s.units.get_mut(&id).unwrap().contract_until = Some(s.turn + 20);
    assert_eq!(resources::current_users(&s, P).len(), before - 1);
}

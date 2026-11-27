//! Byzantium's Buy a city and the fall of Constantinople (`buy_city`, rules 63).
use legatum_core::buy_city::{self, OfferOutcome};
use legatum_core::diplomacy::RelationState;
use legatum_core::experiments::FactionRule;
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::state::{City, Map};
use legatum_core::*;
use std::collections::BTreeMap;

const BYZ: PlayerId = PlayerId(0);
const SELLER: PlayerId = PlayerId(1);
const THIRD: PlayerId = PlayerId(2);

fn h(c: i32, r: i32) -> Hex {
    Hex::from_offset(c, r)
}

/// Insert a city owning its radius-1 tiles (not a capital).
fn city(s: &mut GameState, owner: PlayerId, at: Hex, pop: i32) -> CityId {
    let id = s.alloc_city_id();
    s.cities.insert(
        id,
        City {
            last_attacked_turn: None,
            id,
            owner,
            founder: owner,
            name: format!("City {}", id.0),
            pos: at,
            population: pop,
            food_stored: 0,
            production_stored: 7,
            producing: None,
            bought: None,
            production_queue: Vec::new(),
            build_automation: None,
            last_auto_pick: None,
            auto_queued: None,
            production_banked: Vec::new(),
            production_queue_notified: Default::default(),
            wonder_lost: None,
            last_purchase_turn: None,
            buildings: Default::default(),
            walls_level: 0,
            walls_damage: 0,
            hp: data::CITY_HP,
            grown_tiles: Vec::new(),
            tiles_claimed: 0,
            is_capital: false,
            original_capital: false,
            greece_captive_paid: Default::default(),
            occupied_from: None,
            founded_turn: 1,
            resistance: 0,
        },
    );
    for t in at.within(1) {
        if let Some(tile) = s.map.get_mut(t) {
            tile.owner = Some(owner);
            tile.city = Some(id);
        }
    }
    id
}

fn capital(s: &mut GameState, owner: PlayerId, at: Hex, pop: i32) -> CityId {
    let id = city(s, owner, at, pop);
    let c = s.cities.get_mut(&id).unwrap();
    c.is_capital = true;
    c.original_capital = true;
    id
}

struct World {
    s: GameState,
    byz_capital: CityId,
    seller_capital: CityId,
    /// The seller's city next to Byzantium's land.
    target: CityId,
    /// A seller city far from Byzantium.
    far: CityId,
}

/// Byzantium (0), Switzerland (1, the seller) and the Celts (2), one per team, all met, at Peace.
/// Byzantium's capital at (3,5); the seller's target city at (6,5), touching it.
fn world_with(rules_off: &[FactionRule]) -> World {
    let mut setup = GameSetup::all_ai(11);
    setup.width = 24;
    setup.height = 14;
    setup.players.truncate(3);
    setup.players[0].faction = Faction::Byzantium;
    setup.players[1].faction = Faction::Switzerland;
    setup.players[2].faction = Faction::Celts;
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.team = TeamId(i as u8);
    }
    for rule in rules_off {
        setup.rule_experiments.faction_rules_off.insert(*rule);
    }
    let mut s = blank_game(&setup, Map::new(24, 14, Terrain::Grassland));
    let teams = s.teams();
    s.diplomacy.as_mut().unwrap().meet_all(&teams);
    for r in &mut s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::Peace;
    }
    let byz_capital = capital(&mut s, BYZ, h(3, 5), 5);
    let target = city(&mut s, SELLER, h(6, 5), 4);
    let seller_capital = capital(&mut s, SELLER, h(12, 5), 5);
    let far = city(&mut s, SELLER, h(16, 10), 3);
    capital(&mut s, THIRD, h(20, 2), 3);
    s.player_mut(BYZ).gold = 3000;
    s.player_mut(SELLER).gold = 100;
    World {
        s,
        byz_capital,
        seller_capital,
        target,
        far,
    }
}

fn world() -> World {
    world_with(&[])
}

fn buy(city: CityId) -> Order {
    Order::BuyCity { city, gold: 0 }
}

fn bid(city: CityId, gold: i32) -> Order {
    Order::BuyCity { city, gold }
}

fn errors(s: &GameState, p: PlayerId, orders: &[Order]) -> Vec<String> {
    validate_orders(s, p, orders)
        .into_iter()
        .map(|e| e.message)
        .collect()
}

fn step(s: &GameState, orders: &[(PlayerId, Vec<Order>)]) -> (GameState, Vec<Event>) {
    resolve_turn(s, &orders.iter().cloned().collect::<BTreeMap<_, _>>())
}

fn ended(events: &[Event]) -> Option<(OfferOutcome, i32, i32, i32, String)> {
    events.iter().find_map(|e| match e {
        Event::CityOfferEnded {
            outcome,
            paid,
            refund,
            fee,
            reason,
            ..
        } => Some((*outcome, *paid, *refund, *fee, reason.clone())),
        _ => None,
    })
}

/// Run turns with no orders until the offer ends; returns the state and that turn's events.
fn run_out(mut s: GameState) -> (GameState, Vec<Event>) {
    for _ in 0..6 {
        let (next, events) = step(&s, &[]);
        s = next;
        if ended(&events).is_some() {
            return (s, events);
        }
    }
    panic!("the offer never ended");
}

#[test]
fn an_offer_on_a_touching_rival_city_is_legal_and_moves_the_price_into_escrow() {
    let w = world();
    let price = buy_city::price(&w.s, w.target);
    assert!(buy_city::blocker(&w.s, BYZ, w.target).is_none());
    assert!(errors(&w.s, BYZ, &[buy(w.target)]).is_empty());
    let (control, _) = step(&w.s, &[]);
    let (s, events) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    assert_eq!(s.player(BYZ).gold, control.player(BYZ).gold - price);
    let offer = s.player(BYZ).city_offer.clone().expect("offer running");
    assert_eq!(offer.city, w.target);
    assert_eq!(offer.seller, SELLER);
    assert_eq!(offer.escrow, price);
    assert_eq!(offer.ends_turn, w.s.turn + buy_city::OFFER_TURNS);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::CityOfferMade { city, buyer: BYZ, seller: SELLER, escrow, .. }
            if *city == w.target && *escrow == price
    )));
    // The seller's view shows the running offer and the price it must beat.
    let seller_view = player_view(&s, SELLER);
    let o = seller_view
        .city_offers
        .iter()
        .find(|o| o.city == w.target)
        .expect("the seller sees the offer");
    assert_eq!((o.buyer, o.escrow, o.turns_left), (BYZ, price, 3));
    // The offer survives a save.
    let back: GameState = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back.player(BYZ).city_offer, s.player(BYZ).city_offer);
}

#[test]
fn eligibility_refusals() {
    let w = world();
    let refuse = |s: &GameState, city: CityId, needle: &str| {
        let reason = buy_city::blocker(s, BYZ, city).expect("refused");
        assert!(
            reason.contains(needle),
            "{reason:?} should mention {needle:?}"
        );
        let errs = errors(s, BYZ, &[buy(city)]);
        assert_eq!(errs, vec![reason.clone()], "validation shares the refusal");
    };
    // War.
    let mut s = w.s.clone();
    s.diplomacy.as_mut().unwrap().relations[0].state = RelationState::War;
    refuse(&s, w.target, "War");
    // A teammate's city.
    let mut s = w.s.clone();
    s.players[1].team = TeamId(0);
    refuse(&s, w.target, "your team");
    // A capital, and an original capital whoever holds it.
    refuse(&w.s, w.seller_capital, "capital");
    let mut s = w.s.clone();
    s.cities.get_mut(&w.target).unwrap().original_capital = true;
    refuse(&s, w.target, "capital");
    // Not touching Byzantine land.
    refuse(&w.s, w.far, "does not touch");
    // In capture resistance.
    let mut s = w.s.clone();
    s.cities.get_mut(&w.target).unwrap().resistance = 2;
    refuse(&s, w.target, "resisting");
    // Not enough Gold.
    let mut s = w.s.clone();
    s.player_mut(BYZ).gold = 10;
    refuse(&s, w.target, "Gold");
    // Not Byzantium.
    let mut s = w.s.clone();
    s.player_mut(BYZ).faction = Faction::Sparta;
    refuse(&s, w.target, "Only Byzantium");
    // A second offer while one runs, and two in one order list.
    let (mut s2, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    let second_city = city(&mut s2, SELLER, h(4, 8), 2);
    refuse(&s2, second_city, "one offer at a time");
    let mut s3 = w.s.clone();
    let other = city(&mut s3, SELLER, h(4, 8), 2);
    assert!(buy_city::blocker(&s3, BYZ, other).is_none());
    let errs = errors(&s3, BYZ, &[buy(w.target), buy(other)]);
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("one offer at a time"), "{errs:?}");
}

#[test]
fn the_price_adds_population_buildings_walls_guards_and_doubles_for_a_goal() {
    let mut w = world();
    let rules = w.s.setup.rule_experiments;
    // Population 4: 75 + 4 x 25.
    assert_eq!(buy_city::price(&w.s, w.target), 75 + 4 * 25);
    // A Granary adds a fifth of its Gold price.
    let granary = rules.base_production_cost(
        Faction::Switzerland,
        ProductionItem::Building {
            building: BuildingKind::Granary,
        },
    ) * data::BUY_COST_PERCENT
        / 100;
    w.s.cities
        .get_mut(&w.target)
        .unwrap()
        .buildings
        .insert(BuildingKind::Granary);
    let base = 75 + 4 * 25 + granary * 20 / 100;
    assert_eq!(buy_city::price(&w.s, w.target), base);
    // Walls level 2 (+50%) and one armed unit next to it (+20%), added together.
    w.s.cities.get_mut(&w.target).unwrap().walls_level = 2;
    spawn_unit(&mut w.s, SELLER, UnitKind::Spearman, h(7, 5));
    let defended = base * (100 + 2 * 25 + 20) / 100;
    assert_eq!(buy_city::price(&w.s, w.target), defended);
    // Byzantium's own units and civilians do not count.
    spawn_unit(&mut w.s, BYZ, UnitKind::Spearman, h(5, 4));
    spawn_unit(&mut w.s, SELLER, UnitKind::Settler, h(6, 4));
    assert_eq!(buy_city::price(&w.s, w.target), defended);
    // A goal's held VP in it doubles the price: here the Heir standing in the city.
    spawn_unit(&mut w.s, SELLER, UnitKind::HeirToTheThrone, h(6, 5));
    let doubled = buy_city::price(&w.s, w.target);
    assert_eq!(doubled, defended * 2);
    // The breakdown's lines sum to the price.
    let b = buy_city::price_breakdown(&w.s, &w.s.cities[&w.target]);
    assert_eq!(b.total, doubled);
    assert_eq!(b.parts.iter().map(|(_, g)| g).sum::<i32>(), doubled);
}

#[test]
fn war_declared_by_either_side_cancels_the_offer_with_a_full_refund() {
    for declarer in [BYZ, SELLER] {
        let w = world();
        let (s, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
        let escrow = s.player(BYZ).city_offer.as_ref().unwrap().escrow;
        let against = if declarer == BYZ {
            TeamId(1)
        } else {
            TeamId(0)
        };
        let war = Order::DeclareWar {
            against,
            breach: false,
        };
        let (control, _) = step(&s, &[]);
        let (after, events) = step(&s, &[(declarer, vec![war])]);
        assert!(after.at_war(BYZ, SELLER), "the declaration took effect");
        let (outcome, paid, refund, fee, reason) = ended(&events).expect("offer ended");
        assert_eq!(
            (outcome, paid, refund, fee),
            (OfferOutcome::Cancelled, 0, escrow, 0)
        );
        assert!(reason.contains("War"), "{reason}");
        assert!(after.player(BYZ).city_offer.is_none());
        assert_eq!(
            after.player(BYZ).gold,
            control.player(BYZ).gold + escrow,
            "the escrow comes back in full"
        );
        assert_eq!(after.cities[&w.target].owner, SELLER);
    }
}

#[test]
fn a_price_rise_above_the_escrow_fails_the_offer_for_a_ten_percent_fee() {
    let w = world();
    let (s, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    let escrow = s.player(BYZ).city_offer.as_ref().unwrap().escrow;
    // The seller moves a Spearman next to the city: +20%.
    let mut s = s;
    spawn_unit(&mut s, SELLER, UnitKind::Spearman, h(7, 5));
    let (control, _) = {
        let mut c = s.clone();
        c.player_mut(BYZ).city_offer = None;
        step(&c, &[])
    };
    let (after, events) = step(&s, &[]);
    let fee = escrow * buy_city::FAIL_FEE_PERCENT / 100;
    let (outcome, _, refund, lost, reason) = ended(&events).expect("offer ended");
    assert_eq!(
        (outcome, refund, lost),
        (OfferOutcome::Failed, escrow - fee, fee)
    );
    assert!(reason.contains("price rose"), "{reason}");
    assert_eq!(
        after.player(BYZ).gold,
        control.player(BYZ).gold + escrow - fee
    );
    assert_eq!(
        after.player(SELLER).gold,
        control.player(SELLER).gold,
        "the fee goes to nobody"
    );
    assert_eq!(after.cities[&w.target].owner, SELLER);
}

#[test]
fn growth_alone_never_breaks_an_offer() {
    let w = world();
    let (mut s, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    s.cities.get_mut(&w.target).unwrap().population += 2;
    let (after, events) = run_out(s);
    assert_eq!(ended(&events).unwrap().0, OfferOutcome::Sold);
    assert_eq!(after.cities[&w.target].owner, BYZ);
}

#[test]
fn a_sale_transfers_everything_without_capture_effects_and_pays_the_seller() {
    let mut w = world();
    {
        let c = w.s.cities.get_mut(&w.target).unwrap();
        c.buildings.insert(BuildingKind::Granary);
        c.walls_level = 1;
        c.production_stored = 33;
        c.producing = Some(ProductionItem::Unit {
            unit: UnitKind::Spearman,
        });
    }
    // A seller Settler and a third party's Scout stand in the city.
    let settler = spawn_unit(&mut w.s, SELLER, UnitKind::Settler, h(6, 5));
    let (s, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    let escrow = s.player(BYZ).city_offer.as_ref().unwrap().escrow;
    let before = s.cities[&w.target].clone();
    let seller_gold = s.player(SELLER).gold;
    let (after, events) = run_out(s);
    let (outcome, paid, ..) = ended(&events).unwrap();
    assert_eq!((outcome, paid), (OfferOutcome::Sold, escrow));
    assert_eq!(
        after.turn,
        w.s.turn + buy_city::OFFER_TURNS + 1,
        "sold at the end of turn order turn + 3"
    );
    let c = &after.cities[&w.target];
    assert_eq!(c.owner, BYZ);
    assert!(c.population >= before.population, "no half-population loss");
    assert_eq!(c.resistance, 0);
    assert_eq!(c.occupied_from, None);
    assert_eq!(c.walls_level, 1, "Walls stay");
    assert!(c.buildings.contains(&BuildingKind::Granary));
    assert!(!c.is_capital);
    for t in h(6, 5).within(1) {
        assert_eq!(
            after.map.get(t).unwrap().owner,
            Some(BYZ),
            "its land goes with it"
        );
    }
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::CityCaptured { .. } | Event::CityRazed { .. })),
        "a sale is not a capture"
    );
    assert!(
        after.player(SELLER).gold >= seller_gold + escrow - 50,
        "the seller is paid"
    );
    // The seller's Settler left for the seller's nearest city.
    let u = &after.units[&settler];
    assert_eq!(u.owner, SELLER);
    assert_ne!(u.pos, h(6, 5));
    assert_eq!(after.city_at(u.pos).map(|c| c.owner), Some(SELLER));
    assert!(after.player(BYZ).city_offer.is_none());
}

#[test]
fn a_sale_moves_foreign_units_out_to_their_owners_nearest_free_tile() {
    let mut w = world();
    let spear = spawn_unit(&mut w.s, SELLER, UnitKind::Spearman, h(6, 5));
    // The seller's capital is occupied by a Spearman already: the moved one needs another tile.
    spawn_unit(&mut w.s, SELLER, UnitKind::Spearman, h(12, 5));
    w.s.cities.get_mut(&w.far).unwrap().population = 1;
    // Price includes the guard, so pay for it.
    let (s, _) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    let mut events = Vec::new();
    let mut s = s;
    buy_city::transfer_city(&mut s, w.target, BYZ, &mut events);
    let u = &s.units[&spear];
    assert_ne!(u.pos, h(6, 5));
    assert!(
        s.units_at(u.pos)
            .filter(|o| o.id != spear)
            .all(|o| o.owner == SELLER)
    );
    let tile = s.map.get(u.pos).unwrap();
    assert_eq!(tile.owner, Some(SELLER), "a free city or tile of its owner");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::UnitMoved { unit, .. } if *unit == spear))
    );
}

#[test]
fn switched_off_the_order_is_refused_and_no_prices_are_shown() {
    let w = world_with(&[FactionRule::ByzantiumBuyCity]);
    assert!(buy_city::blocker(&w.s, BYZ, w.target).is_some());
    assert_eq!(errors(&w.s, BYZ, &[buy(w.target)]).len(), 1);
    assert!(player_view(&w.s, BYZ).city_buys.is_empty());
    assert!(player_view(&w.s, SELLER).city_buys.is_empty());
    let (s, events) = step(&w.s, &[(BYZ, vec![buy(w.target)])]);
    assert!(s.player(BYZ).city_offer.is_none());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { player: BYZ, .. }))
    );
}

#[test]
fn views_show_prices_to_byzantium_and_to_the_owner_of_a_touching_city() {
    let w = world();
    let v = player_view(&w.s, BYZ);
    let row = v
        .city_buys
        .iter()
        .find(|b| b.city == w.target)
        .expect("Byzantium sees the price");
    assert_eq!(row.price, buy_city::price(&w.s, w.target));
    assert!(row.blocker.is_none());
    let seller = player_view(&w.s, SELLER);
    let own = seller
        .city_buys
        .iter()
        .find(|b| b.city == w.target)
        .expect("the owner sees what Byzantium would pay");
    assert_eq!((own.buyer, own.price), (BYZ, row.price));
    assert!(
        !seller.city_buys.iter().any(|b| b.city == w.far),
        "a city Byzantium cannot buy is not listed for others"
    );
    let caps = &v.byzantine_capitals;
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].capital, Some(w.byz_capital));
    assert_eq!(caps[0].refuge, None);
}

#[test]
fn an_overbid_is_held_in_full_takes_more_guards_to_break_and_all_goes_to_the_seller() {
    let w = world();
    let price = buy_city::price(&w.s, w.target);
    assert_eq!(price, 175);
    let gold = price * 150 / 100;
    assert!(errors(&w.s, BYZ, &[bid(w.target, gold)]).is_empty());
    let (control, _) = step(&w.s, &[]);
    let (s, events) = step(&w.s, &[(BYZ, vec![bid(w.target, gold)])]);
    assert_eq!(s.player(BYZ).gold, control.player(BYZ).gold - gold);
    assert_eq!(s.player(BYZ).city_offer.as_ref().unwrap().escrow, gold);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::CityOfferMade { escrow, price: p, guards_to_stop: 3, .. } if *escrow == gold && *p == price
    )));
    // +50%: two armed units (+20% each) stay under it (245 of 262); a third breaks it.
    let o = player_view(&s, SELLER)
        .city_offers
        .into_iter()
        .find(|o| o.city == w.target)
        .unwrap();
    assert_eq!(
        (o.escrow, o.guards_to_stop, o.walls_to_stop),
        (gold, 3, Some(3))
    );
    let mut s = s;
    spawn_unit(&mut s, SELLER, UnitKind::Spearman, h(7, 5));
    spawn_unit(&mut s, SELLER, UnitKind::Spearman, h(5, 5));
    let o = player_view(&s, SELLER)
        .city_offers
        .into_iter()
        .find(|o| o.city == w.target)
        .unwrap();
    assert_eq!((o.price_now, o.guards_to_stop), (price * 140 / 100, 1));
    let seller_gold = s.player(SELLER).gold;
    let (after, events) = run_out(s.clone());
    let (outcome, paid, ..) = ended(&events).unwrap();
    assert_eq!(
        (outcome, paid),
        (OfferOutcome::Sold, gold),
        "two guards do not stop it"
    );
    assert_eq!(after.cities[&w.target].owner, BYZ);
    assert!(
        after.player(SELLER).gold >= seller_gold + gold - 50,
        "all of the bid is paid"
    );
    // A third guard does.
    spawn_unit(&mut s, SELLER, UnitKind::Spearman, h(6, 4));
    let (_, events) = run_out(s);
    let (outcome, _, refund, fee, _) = ended(&events).unwrap();
    assert_eq!(outcome, OfferOutcome::Failed);
    assert_eq!(
        (refund, fee),
        (gold - gold / 10, gold / 10),
        "the fee is 10% of the bid"
    );
}

#[test]
fn a_bid_below_the_price_above_the_cap_or_the_treasury_is_refused() {
    let w = world();
    let price = buy_city::price(&w.s, w.target);
    let below = errors(&w.s, BYZ, &[bid(w.target, price - 1)]);
    assert_eq!(below.len(), 1);
    assert!(below[0].contains("below the price"), "{below:?}");
    let most = price * buy_city::MAX_BID_PERCENT / 100;
    assert!(errors(&w.s, BYZ, &[bid(w.target, most)]).is_empty());
    let above = errors(&w.s, BYZ, &[bid(w.target, most + 1)]);
    assert!(above[0].contains("The most you can offer"), "{above:?}");
    let mut s = w.s.clone();
    s.player_mut(BYZ).gold = price + 10;
    let poor = errors(&s, BYZ, &[bid(w.target, price + 50)]);
    assert!(poor[0].starts_with("Needs "), "{poor:?}");
    let (after, events) = step(&s, &[(BYZ, vec![bid(w.target, price + 50)])]);
    assert!(after.player(BYZ).city_offer.is_none());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::OrderFailed { player: BYZ, .. }))
    );
}

#[test]
fn an_order_without_gold_offers_the_price_and_old_json_still_reads() {
    let w = world();
    let old: Order =
        serde_json::from_str(&format!(r#"{{"type":"BuyCity","city":{}}}"#, w.target.0)).unwrap();
    assert_eq!(old, buy(w.target));
    assert_eq!(
        serde_json::to_string(&buy(w.target)).unwrap(),
        format!(r#"{{"type":"BuyCity","city":{}}}"#, w.target.0),
        "gold 0 is left out, so old tapes hash the same"
    );
    let (s, _) = step(&w.s, &[(BYZ, vec![old])]);
    assert_eq!(
        s.player(BYZ).city_offer.as_ref().unwrap().escrow,
        buy_city::price(&w.s, w.target)
    );
}

#[test]
fn the_view_carries_machine_fields_and_the_bid_choices_with_what_stops_each() {
    let mut w = world();
    let granary = w.s.setup.rule_experiments.base_production_cost(
        Faction::Switzerland,
        ProductionItem::Building {
            building: BuildingKind::Granary,
        },
    ) * data::BUY_COST_PERCENT
        / 100;
    w.s.cities
        .get_mut(&w.target)
        .unwrap()
        .buildings
        .insert(BuildingKind::Granary);
    spawn_unit(&mut w.s, SELLER, UnitKind::Spearman, h(7, 5));
    let v = player_view(&w.s, BYZ);
    let row = v.city_buys.iter().find(|b| b.city == w.target).unwrap();
    assert_eq!(row.buildings_gold, granary);
    assert_eq!(
        (row.holds_goal, row.guards, row.short_of_gold),
        (false, 1, 0)
    );
    assert_eq!(row.max_bid, row.price * 3);
    let percents: Vec<i32> = row.bids.iter().map(|b| b.percent).collect();
    assert_eq!(percents, vec![100, 150, 200, 300]);
    for b in &row.bids {
        assert_eq!(b.gold, row.price * b.percent / 100);
        let base = buy_city::base_breakdown(&w.s, &w.s.cities[&w.target]).total;
        assert_eq!(
            b.guards_to_stop,
            buy_city::guards_to_stop(base, 0, 1, false, b.gold)
        );
        assert!(
            buy_city::defended_price(base, 0, 1 + b.guards_to_stop, false) > b.gold
                && buy_city::defended_price(base, 0, b.guards_to_stop, false) <= b.gold,
            "the smallest number of more units that breaks it"
        );
    }
    assert_eq!(
        row.bids[0].guards_to_stop, 1,
        "at the price, one more unit breaks it"
    );
    // The defended price matches the breakdown.
    let c = &w.s.cities[&w.target];
    assert_eq!(
        buy_city::defended_price(buy_city::base_breakdown(&w.s, c).total, 0, 1, false),
        row.price
    );
    // Only the buyer sees the bid choices; the owner sees the machine fields.
    let own = player_view(&w.s, SELLER);
    let seen = own.city_buys.iter().find(|b| b.city == w.target).unwrap();
    assert!(seen.bids.is_empty());
    assert_eq!(seen.guards, 1);
    // Short of Gold: the machine field says by how much.
    let mut s = w.s.clone();
    s.player_mut(BYZ).gold = 100;
    let v = player_view(&s, BYZ);
    let row = v.city_buys.iter().find(|b| b.city == w.target).unwrap();
    assert_eq!(row.short_of_gold, row.price - 100);
    // A goal in it.
    spawn_unit(&mut s, SELLER, UnitKind::HeirToTheThrone, h(6, 5));
    let v = player_view(&s, BYZ);
    assert!(
        v.city_buys
            .iter()
            .find(|b| b.city == w.target)
            .unwrap()
            .holds_goal
    );
}

// The fall of Constantinople.

/// Byzantium with two more cities; the seller (now at War with it) captures the capital.
fn collapse_world(rules_off: &[FactionRule]) -> (GameState, Vec<CityId>) {
    let mut w = world_with(rules_off);
    let a = city(&mut w.s, BYZ, h(3, 9), 4);
    let b = city(&mut w.s, BYZ, h(3, 1), 6);
    for r in &mut w.s.diplomacy.as_mut().unwrap().relations {
        r.state = RelationState::War;
    }
    (w.s, vec![w.byz_capital, a, b])
}

#[test]
fn taking_byzantiums_capital_takes_every_byzantine_city() {
    let (mut s, cities) = collapse_world(&[]);
    let mut events = Vec::new();
    rules::battle::capture_city(&mut s, cities[0], SELLER, &mut events);
    for c in &cities {
        assert_eq!(s.cities[c].owner, SELLER);
        assert_eq!(
            s.cities[c].resistance,
            data::CAPTURE_RESISTANCE_TURNS,
            "an ordinary capture"
        );
    }
    assert_eq!(
        s.cities[&cities[2]].population, 3,
        "half population, as any capture"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::CapitalFell { player: BYZ, captor: SELLER, cities: c, .. } if c == &cities[1..]
    )));
    let captures = events
        .iter()
        .filter(|e| matches!(e, Event::CityCaptured { .. }))
        .count();
    assert_eq!(captures, 3);
}

#[test]
fn the_heir_in_another_city_gives_refuge_and_that_city_becomes_the_capital() {
    let (mut s, cities) = collapse_world(&[]);
    let heir = spawn_unit(&mut s, BYZ, UnitKind::HeirToTheThrone, h(3, 1));
    s.units.get_mut(&heir).unwrap().heir_of = Some(BYZ);
    assert_eq!(
        player_view(&s, SELLER).byzantine_capitals[0].refuge,
        Some(cities[2])
    );
    let mut events = Vec::new();
    rules::battle::capture_city(&mut s, cities[0], SELLER, &mut events);
    assert_eq!(s.cities[&cities[0]].owner, SELLER);
    assert_eq!(s.cities[&cities[1]].owner, BYZ);
    assert_eq!(s.cities[&cities[2]].owner, BYZ);
    assert!(
        s.cities[&cities[2]].is_capital,
        "the refuge is the new capital"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::HeirRefuge { player: BYZ, refuge, .. } if *refuge == cities[2]
    )));
    // An Heir in the field gives no refuge.
    let (mut s, cities) = collapse_world(&[]);
    let heir = spawn_unit(&mut s, BYZ, UnitKind::HeirToTheThrone, h(8, 11));
    s.units.get_mut(&heir).unwrap().heir_of = Some(BYZ);
    rules::battle::capture_city(&mut s, cities[0], SELLER, &mut Vec::new());
    assert!(cities.iter().all(|c| s.cities[c].owner == SELLER));
}

#[test]
fn a_celtic_captor_sacks_every_byzantine_city() {
    let (mut s, cities) = collapse_world(&[]);
    let mut events = Vec::new();
    rules::battle::capture_city(&mut s, cities[0], THIRD, &mut events);
    assert!(
        cities.iter().all(|c| !s.cities.contains_key(c)),
        "all burned"
    );
    let razed = events
        .iter()
        .filter(|e| matches!(e, Event::CityRazed { by: THIRD, .. }))
        .count();
    assert_eq!(razed, 3);
}

#[test]
fn switched_off_only_the_capital_falls() {
    let (mut s, cities) = collapse_world(&[FactionRule::ByzantiumCollapse]);
    rules::battle::capture_city(&mut s, cities[0], SELLER, &mut Vec::new());
    assert_eq!(s.cities[&cities[0]].owner, SELLER);
    assert_eq!(s.cities[&cities[1]].owner, BYZ);
    assert_eq!(s.cities[&cities[2]].owner, BYZ);
    assert!(player_view(&s, SELLER).byzantine_capitals.is_empty());
}

#[test]
fn a_collapse_in_battle_is_deterministic_and_eliminates_an_empire_left_with_nothing() {
    let (mut s, cities) = collapse_world(&[]);
    s.cities.get_mut(&cities[0]).unwrap().hp = 0;
    let spear = spawn_unit(&mut s, SELLER, UnitKind::Spearman, h(4, 5));
    s.units.get_mut(&spear).unwrap().goto = Some(h(3, 5));
    let (a, ea) = step(&s, &[]);
    let (b, eb) = step(&s, &[]);
    assert_eq!(a.hash(), b.hash());
    assert_eq!(ea, eb);
    assert!(cities.iter().all(|c| a.cities[c].owner == SELLER), "{ea:?}");
    assert!(!a.player(BYZ).alive, "no city and no Settler left");
}

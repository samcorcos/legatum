//! Byzantium's Buy a city and the fall of Constantinople, both sides (`byzantium.rs`).
use crate::fixture::{ENEMY, ME, ViewBuilder, off};
use crate::scripted::Planner;
use legatum_core::buy_city::{CapitalFallView, CityBuyView, CityOfferView};
use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
use legatum_core::view::PlayerView;
use legatum_core::{Breakdown, CityId, Faction, Order, PlayerId, ProductionItem, UnitId, UnitKind};

fn peace(v: &mut PlayerView, a: PlayerId, b: PlayerId) {
    v.diplomacy.legacy = false;
    let (a, b) = (v.team_of(a), v.team_of(b));
    v.diplomacy.relations.push(Relation {
        a,
        b,
        state: RelationState::Peace,
        effective_turn: 1,
        guarantee_until: 0,
        version: 1,
        security: None,
        war_id: None,
        declarers: Vec::new(),
    });
}

fn breakdown(price: i32) -> Breakdown {
    let mut b = Breakdown::default();
    b.add("Base price", price);
    b
}

fn buy(city: u32, buyer: PlayerId, price: i32, blocker: Option<&str>) -> CityBuyView {
    CityBuyView {
        city: CityId(city),
        buyer,
        price,
        breakdown: breakdown(price),
        blocker: blocker.map(Into::into),
    }
}

/// Byzantium (me) at turn 60 with its capital and a second city, a rival city of population 4
/// next to its land for `price`, and `gold` in the treasury.
fn buyer_view(price: i32, gold: i32) -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Byzantium)
        .city(1, ME, off(4, 4), 4)
        .city(2, ME, off(10, 4), 3)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(2, ME, UnitKind::Spearman, off(10, 4))
        .city(10, ENEMY, off(30, 18), 5)
        .city(11, ENEMY, off(14, 4), 4)
        .build();
    v.turn = 60;
    peace(&mut v, ME, ENEMY);
    v.economy.gold = gold;
    v.economy.gold_per_turn = breakdown(20);
    v.players[ENEMY.0 as usize].gold = 0;
    let blocker = (gold < price).then(|| format!("Needs {price} Gold: you have {gold}"));
    v.city_buys = vec![buy(11, ME, price, blocker.as_deref())];
    v
}

fn planner(v: &PlayerView) -> Planner<'_> {
    let mut p = crate::fixture::planner(v, 7);
    p.difficulty = legatum_core::AiDifficulty::Hard;
    p
}

fn buys(p: &Planner<'_>) -> Vec<CityId> {
    p.orders
        .iter()
        .filter_map(|o| match o {
            Order::BuyCity { city } => Some(*city),
            _ => None,
        })
        .collect()
}

#[test]
fn byzantium_buys_a_cheap_undefended_city_and_gene_zero_never_buys() {
    let v = buyer_view(390, 600);
    let mut p = planner(&v);
    p.plan_city_purchase();
    assert_eq!(buys(&p), vec![CityId(11)]);
    assert_eq!(
        p.gold_left,
        600 - 390,
        "the escrow is taken from this turn's Gold"
    );

    let mut p = planner(&v);
    p.decisions.buy_city_value = 0;
    p.plan_city_purchase();
    assert!(buys(&p).is_empty());
}

#[test]
fn byzantium_banks_toward_an_offer_income_pays_within_ten_turns() {
    // 390 Gold at 20 a turn: 100 now leaves 15 turns, too far; 250 leaves 7.
    let far = buyer_view(390, 100);
    let mut p = planner(&far);
    p.plan_city_purchase();
    assert_eq!(p.city_buy_savings, 0);

    let near = buyer_view(390, 250);
    let mut p = planner(&near);
    p.plan_city_purchase();
    assert!(buys(&p).is_empty());
    assert_eq!(p.city_buy_savings, 390);
    assert!(
        p.gold_reserve() >= 390,
        "routine purchases leave the banked Gold"
    );
}

#[test]
fn byzantium_skips_an_offer_the_owner_can_break_or_one_that_costs_more_than_it_is_worth() {
    // Three rival units within the offer's reach and a treasury to buy a guard.
    let mut v = buyer_view(390, 600);
    for (i, col) in [17, 18, 19].into_iter().enumerate() {
        v.units.push(
            ViewBuilder::new(40, 24)
                .unit(90 + i as u32, ENEMY, UnitKind::Spearman, off(col, 4))
                .build()
                .units[0]
                .clone(),
        );
    }
    v.players[ENEMY.0 as usize].gold = 500;
    let p = planner(&v);
    let a = p.city_buy_appraisals();
    assert_eq!(a[0].risk, 95, "{a:?}");
    let mut p = planner(&v);
    p.plan_city_purchase();
    assert!(buys(&p).is_empty(), "the offer would likely fail");

    // A price above the city's worth.
    let dear = buyer_view(5000, 6000);
    let mut p = planner(&dear);
    p.plan_city_purchase();
    assert!(buys(&p).is_empty());
}

#[test]
fn byzantium_makes_no_offer_while_one_runs_or_war_is_at_home() {
    let mut v = buyer_view(390, 600);
    v.city_offers.push(CityOfferView {
        city: CityId(10),
        buyer: ME,
        seller: ENEMY,
        escrow: 400,
        ends_turn: 62,
        turns_left: 2,
        price_now: 400,
    });
    let mut p = planner(&v);
    p.plan_city_purchase();
    assert!(buys(&p).is_empty());
}

/// Byzantium (me) with `n` cities, the first its capital, and a rival Spearman at Peace five
/// hexes from the capital.
fn empire(n: u32) -> PlayerView {
    let mut b = ViewBuilder::new(48, 24).faction(Faction::Byzantium);
    for i in 0..n {
        b = b.city(i + 1, ME, off(4 + 6 * i as i32, 4), 3);
    }
    let mut v = b
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .unit(90, ENEMY, UnitKind::Spearman, off(4, 9))
        .city(20, ENEMY, off(30, 20), 3)
        .build();
    peace(&mut v, ME, ENEMY);
    let enemy_team = v.team_of(ENEMY);
    v.diplomacy.actions.push(DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order: Order::DeclareWar {
            against: enemy_team,
            breach: false,
        },
        effective_turn: v.turn,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    });
    v.byzantine_capitals = vec![CapitalFallView {
        player: ME,
        capital: Some(CityId(1)),
        refuge: None,
    }];
    v
}

#[test]
fn the_capital_guard_grows_with_the_empire_and_stops_with_a_refuge() {
    let small = empire(2);
    let big = empire(7);
    let (ps, pb) = (planner(&small), planner(&big));
    let (cs, cb) = (&small.cities[0], &big.cities[0]);
    assert!(pb.capital_pressed(cb));
    assert_eq!(ps.byz_capital_wanted_guards(cs), 1);
    assert_eq!(pb.byz_capital_wanted_guards(cb), 3);
    // One guard stands in each: the small empire is content, the big one wants two more.
    assert_eq!(ps.byz_capital_garrison_value(cs), 0);
    assert_eq!(pb.byz_capital_garrison_value(cb), 1200 + 400 * 6);
    assert!(pb.byz_capital_walls_value(cb) > ps.byz_capital_walls_value(cs));
    assert!(ps.byz_capital_walls_value(cs) > 0);
    // Another city is not the capital.
    assert_eq!(pb.byz_capital_walls_value(&big.cities[1]), 0);

    let mut sheltered = empire(7);
    sheltered.byzantine_capitals[0].refuge = Some(CityId(2));
    let p = planner(&sheltered);
    assert_eq!(p.byz_capital_riders(&sheltered.cities[0]), 0);
    assert_eq!(p.byz_capital_walls_value(&sheltered.cities[0]), 0);

    let mut off_gene = planner(&big);
    off_gene.decisions.byz_capital_guard = 0;
    assert_eq!(off_gene.byz_capital_walls_value(cb), 0);
}

#[test]
fn a_threatened_capital_sends_the_heir_to_another_city() {
    let mut v = empire(3);
    v.units.push(
        ViewBuilder::new(40, 24)
            .unit(2, ME, UnitKind::Spearman, off(10, 4))
            .build()
            .units[0]
            .clone(),
    );
    let mut heir = ViewBuilder::new(40, 24)
        .unit(77, ME, UnitKind::HeirToTheThrone, off(4, 4))
        .build()
        .units[0]
        .clone();
    heir.military = false;
    v.units.push(heir);
    let mut p = planner(&v);
    p.protect_heirs();
    assert!(
        p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit, to } if *unit == UnitId(77) && *to != off(4, 4))),
        "{:?}",
        p.orders
    );
}

/// A rival (me, Athens) whose city 5 a Byzantium (ENEMY) is buying for `escrow`.
fn seller_view(escrow: i32, price_now: i32) -> PlayerView {
    let mut v = ViewBuilder::new(40, 24)
        .faction(Faction::Athens)
        .city(1, ME, off(4, 4), 3)
        .unit(1, ME, UnitKind::Spearman, off(4, 4))
        .city_with_detail(5, ME, off(12, 4), 3)
        .city(20, ENEMY, off(18, 4), 3)
        .build();
    peace(&mut v, ME, ENEMY);
    v.economy.gold = 0;
    v.city_offers.push(CityOfferView {
        city: CityId(5),
        buyer: ENEMY,
        seller: ME,
        escrow,
        ends_turn: v.turn + 2,
        turns_left: 2,
        price_now,
    });
    v.city_buys = vec![buy(5, ENEMY, price_now, None)];
    v
}

#[test]
fn a_seller_sends_a_unit_next_to_its_city_to_break_the_offer() {
    let mut v = seller_view(330, 330);
    v.units.push(
        ViewBuilder::new(40, 24)
            .unit(3, ME, UnitKind::Spearman, off(9, 5))
            .build()
            .units[0]
            .clone(),
    );
    let mut p = planner(&v);
    p.defend_city_offers();
    assert!(
        p.orders.iter().any(|o| matches!(o,
            Order::MoveUnit { unit, to } if *unit == UnitId(3) && to.distance(off(12, 4)) <= 1)),
        "{:?}",
        p.orders
    );
    // The capital's only garrison stays home.
    assert!(
        !p.orders
            .iter()
            .any(|o| matches!(o, Order::MoveUnit { unit, .. } if *unit == UnitId(1)))
    );
    // Gene off: the Gold is taken.
    let mut p = planner(&v);
    p.decisions.offer_defence = 0;
    p.defend_city_offers();
    assert!(p.orders.is_empty());
}

#[test]
fn a_seller_with_no_unit_in_reach_buys_a_guard_in_the_city() {
    let mut v = seller_view(330, 330);
    v.economy.gold = 300;
    let d = v.cities[1].detail.as_mut().unwrap();
    for o in &mut d.options {
        if o.item
            == (ProductionItem::Unit {
                unit: UnitKind::Spearman,
            })
        {
            o.buy_cost = Some(80);
        }
    }
    let mut p = planner(&v);
    p.defend_city_offers();
    assert!(
        p.orders.contains(&Order::BuyItem {
            city: CityId(5),
            item: ProductionItem::Unit {
                unit: UnitKind::Spearman
            }
        }),
        "{:?}",
        p.orders
    );
    // An offer already failing needs nothing.
    let mut v = seller_view(330, 400);
    v.economy.gold = 300;
    let mut p = planner(&v);
    p.defend_city_offers();
    assert!(p.orders.is_empty());
}

#[test]
fn a_seller_declares_war_only_for_a_city_worth_it() {
    let war = |pop: i32, army: i32| {
        let mut v = seller_view(330, 330);
        v.cities[1].population = pop;
        let enemy_team = v.team_of(ENEMY);
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against: enemy_team,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        crate::fixture::set_armies(&mut v, &[(ME, army), (ENEMY, 100)]);
        let mut p = planner(&v);
        p.defend_city_offers();
        p.orders
            .iter()
            .any(|o| matches!(o, Order::DeclareWar { .. }))
    };
    assert!(war(9, 300));
    assert!(!war(3, 300), "a small city: take the Gold");
    assert!(!war(9, 150), "no army to win the war");
}

#[test]
fn a_city_touching_byzantium_wants_a_defender_in_peace() {
    let v = seller_view(330, 330);
    let mut v = v;
    v.city_offers.clear();
    let p = planner(&v);
    assert_eq!(p.byz_border_guard_value(&v.cities[1]), 2200 * 30 / 100);
    assert_eq!(p.byz_border_guard_value(&v.cities[0]), 0, "not touching");
}

#[test]
fn rivals_value_a_byzantine_capital_as_every_byzantine_city() {
    let byz = PlayerId(3);
    let mut v = ViewBuilder::new(40, 24)
        .city(1, ME, off(4, 4), 3)
        .city(30, byz, off(20, 4), 3)
        .city(31, byz, off(26, 4), 3)
        .city(32, byz, off(26, 10), 3)
        .build();
    v.players[3].faction = Faction::Byzantium;
    v.players[3].cities = 4; // one unseen
    v.byzantine_capitals = vec![CapitalFallView {
        player: byz,
        capital: Some(CityId(30)),
        refuge: None,
    }];
    let p = planner(&v);
    assert_eq!(p.byz_collapse_cities(&v.cities[1]).len(), 3);
    let value = p.byz_collapse_value(&v.cities[1]);
    assert_eq!(
        value,
        2 * crate::production::city_keep_value(&v.cities[2])
            + crate::production::new_city_keep_value(1)
    );
    assert_eq!(p.byz_collapse_value(&v.cities[2]), 0, "not the capital");

    let mut sheltered = v.clone();
    sheltered.byzantine_capitals[0].refuge = Some(CityId(31));
    assert_eq!(
        planner(&sheltered).byz_collapse_value(&sheltered.cities[1]),
        0
    );

    let mut p = planner(&v);
    p.decisions.byz_capital_target = 0;
    assert_eq!(p.byz_collapse_value(&v.cities[1]), 0);
}

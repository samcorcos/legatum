//! The frontline defender check (`production.rs::buy_frontline_defender`): a defender it can't
//! buy holds its price back but doesn't end every other purchase (playtest 8.9.17).
use super::*;
use crate::Knobs;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use legatum_core::{CityId, TechId};

const GRANARY: ProductionItem = ProductionItem::Building {
    building: BuildingKind::Granary,
};
const SPEARMAN: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Spearman,
};

fn tally() -> Tally {
    Tally {
        army: 0,
        rangers: 0,
        scouts: 0,
        heir_pipeline: 0,
        settler_pipeline: 0,
        slow_settlers: 0,
        relay_started: false,
        sites: 0,
    }
}

/// Athens: city 1 has an enemy Spearman two tiles away and no garrison, so the frontline check
/// wants a defender there; city 2, far from it, can buy a Granary for 80. City 1's Spearman is
/// quoted at `soldier` (None: no unit can be bought there).
fn view(gold: i32, spearman: Option<i32>) -> legatum_core::PlayerView {
    let mut view = ViewBuilder::new(32, 20)
        .faction(Faction::Athens)
        .known_techs(&TechId::ALL)
        .gold(gold)
        .city(1, ME, Hex::from_offset(5, 5), 5)
        .city(2, ME, Hex::from_offset(19, 5), 5)
        .unit(50, ENEMY, UnitKind::Spearman, Hex::from_offset(7, 5))
        .build();
    view.economy.gold_per_turn.total = 50;
    let d = view.cities[0].detail.as_mut().unwrap();
    d.options
        .retain(|o| o.item == SPEARMAN && spearman.is_some());
    for o in &mut d.options {
        o.buy_cost = spearman;
        o.buy_block = spearman
            .is_some_and(|cost| cost > gold)
            .then_some(legatum_core::economy::BuyBlock::NotEnoughGold);
    }
    let d = view.cities[1].detail.as_mut().unwrap();
    d.options.retain(|o| o.item == GRANARY);
    for o in &mut d.options {
        o.turns = 10;
        o.buy_cost = Some(80);
        o.buy_block = None;
    }
    view
}

/// Gene `buy_hold` is pinned to 0: these tests are about the frontline defender and what a
/// threatened city does not hold back; buy_hold 3 (default since 2026-10-31) holds Gold from the
/// routine Granary and has its own tests.
fn planner(view: &legatum_core::PlayerView) -> Planner<'_> {
    let mut p = Planner::new(
        view,
        Knobs::for_faction(Faction::Athens),
        Faction::Athens,
        7,
    );
    p.decisions.buy_hold = 0;
    p
}

fn granary_bought(p: &Planner<'_>) -> bool {
    p.orders.contains(&Order::BuyItem {
        city: CityId(2),
        item: GRANARY,
    })
}

#[test]
fn a_threatened_city_that_cannot_buy_a_defender_does_not_stop_other_purchases() {
    // No unit can be bought in the threatened city: the Granary elsewhere is still bought.
    let v = view(200, None);
    let mut p = planner(&v);
    p.plan_buy(&mut tally());
    assert!(granary_bought(&p), "{:?}", p.orders);

    // The threatened city already has this turn's purchase (door defense or a rescue).
    let v = view(200, Some(60));
    let mut p = planner(&v);
    p.orders.push(Order::BuyItem {
        city: CityId(1),
        item: SPEARMAN,
    });
    p.plan_buy(&mut tally());
    assert!(granary_bought(&p), "{:?}", p.orders);
}

#[test]
fn an_unaffordable_defender_keeps_its_price_and_an_affordable_one_is_bought_first() {
    // A 300-Gold Spearman the 200-Gold treasury can't pay: the Granary would spend the Gold
    // the defender needs, so nothing is bought.
    let v = view(200, Some(300));
    let mut p = planner(&v);
    p.plan_buy(&mut tally());
    assert!(p.purchase_orders().is_empty(), "{:?}", p.orders);
    assert_eq!(p.gold_left, 200);

    // Enough for the defender and the Granary: the defender first, and the turn's routine
    // purchases stop there, as before.
    let v = view(400, Some(150));
    let mut p = planner(&v);
    p.plan_buy(&mut tally());
    assert_eq!(
        p.orders,
        vec![Order::BuyItem {
            city: CityId(1),
            item: SPEARMAN,
        }]
    );
}

/// Gene `hoard_at_war` (`scripted.rs::hoard_spent_at_war`): a Swiss saving toward the hoard
/// keeps the whole 5,000 as its purchase reserve, unless its plan fights a war it is in.
#[test]
fn the_swiss_hoard_savings_are_spendable_while_the_plan_fights_a_war() {
    use crate::plan::{Plan, PlanKind};
    let mut v = ViewBuilder::new(32, 20)
        .faction(Faction::Switzerland)
        .known_techs(&TechId::ALL)
        .gold(1_000)
        .city(1, ME, Hex::from_offset(5, 5), 5)
        .city(2, ME, Hex::from_offset(12, 5), 5)
        .city(3, ENEMY, Hex::from_offset(24, 5), 5)
        .build();
    v.turn = 80;
    v.economy.gold_per_turn.total = 50;
    v.players[ME.0 as usize].faction_goal_done = false;
    let enemy = v.team_of(ENEMY);
    let reserve = |gene: i32, plan: Option<PlanKind>| {
        let mut p = Planner::new(
            &v,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.difficulty = legatum_core::AiDifficulty::Hard;
        p.plan_routing = true;
        p.decisions.hoard_at_war = gene;
        p.decisions.swiss_hoard_any_war = 0;
        p.campaigns.plan.active = plan.map(|kind| Plan {
            kind,
            start_turn: 70,
            review_turn: 90,
            reserve: 0,
            value: 300,
            terms: Vec::new(),
            breach: false,
            misses: 0,
        });
        assert!(p.saving_for_goal());
        p.ordinary_gold_reserve()
    };
    let hoard = legatum_core::goals::SWISS_HOARD_GOLD;
    let contain = || Some(PlanKind::Contain { against: enemy });
    assert_eq!(reserve(0, contain()), hoard, "gene off");
    assert_eq!(reserve(1, None), hoard, "no war plan");
    assert_eq!(
        reserve(1, Some(PlanKind::Economy)),
        hoard,
        "an economy plan"
    );
    assert!(reserve(1, contain()) < hoard, "{}", reserve(1, contain()));
    assert_eq!(reserve(1, contain()), 100);
    assert!(v.at_war(ME, ENEMY));
    let mut p = Planner::new(
        &v,
        Knobs::for_faction(Faction::Switzerland),
        Faction::Switzerland,
        7,
    );
    p.difficulty = legatum_core::AiDifficulty::Hard;
    p.plan_routing = true;
    p.decisions.swiss_hoard_any_war = 1;
    assert_eq!(
        p.ordinary_gold_reserve(),
        100,
        "real war releases a remote hoard"
    );
}

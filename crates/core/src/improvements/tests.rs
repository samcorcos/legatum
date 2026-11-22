use super::*;
use crate::orders::Order;
use crate::rules::cities::tests::{P0, P1, at, test_game};
use crate::rules::test_support::city;
use crate::view::player_view;
use crate::{Faction, resolve_turn};
use std::collections::BTreeMap;

const FARM: ImprovementKind = ImprovementKind::Farm;

fn orders(p: PlayerId, os: Vec<Order>) -> BTreeMap<PlayerId, Vec<Order>> {
    BTreeMap::from([(p, os)])
}

fn farm(tile: Hex) -> ProductionItem {
    ProductionItem::Improvement {
        improvement: FARM,
        tile,
    }
}

/// Byzantium city (no terrain bonuses) at (5,5) owning ring 1 and the radius-3 tile (8,5); Wheat
/// at (6,5) in ring 1, a hidden Iron at (7,5) in ring 2 and Cattle beyond the radius.
fn game() -> (GameState, CityId) {
    let mut s = test_game(&[(Faction::Byzantium, 0), (Faction::Byzantium, 1)]);
    let id = city(&mut s, 0, at(5, 5), 3);
    city(&mut s, 1, at(16, 2), 1);
    for h in at(5, 5).within(3) {
        let t = s.map.get_mut(h).unwrap();
        t.owner = Some(P0);
        t.city = Some(id);
    }
    s.map.get_mut(at(6, 5)).unwrap().resource = Some(Resource::Wheat);
    s.map.get_mut(at(7, 5)).unwrap().resource = Some(Resource::Iron);
    s.player_mut(P0).gold = 1000;
    (s, id)
}

fn food(s: &GameState, id: CityId) -> i32 {
    economy::city_yields(s, &s.cities[&id]).total.food
}

#[test]
fn a_resource_yields_nothing_until_improved() {
    let (mut s, id) = game();
    let bare = food(&s, id);
    s.map.get_mut(at(6, 5)).unwrap().resource = None;
    assert_eq!(food(&s, id), bare, "an unimproved Wheat tile is plain land");
    s.map.get_mut(at(6, 5)).unwrap().resource = Some(Resource::Wheat);
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    assert_eq!(food(&s, id), bare + 1, "a Farm adds the Wheat bonus");
    s.player_mut(P0).techs.insert(TechId::CropRotation);
    let upgraded = food(&s, id);
    // Intensive Cultivation upgrades the Farm by itself (+1 more Food). Mill isn't built.
    assert_eq!(upgraded, bare + 2);
}

#[test]
fn only_owned_worked_known_matching_tiles_can_be_improved() {
    let (mut s, id) = game();
    let c = s.cities[&id].clone();
    assert_eq!(blocker(&s, &c, FARM, at(6, 5)), None);
    // Plain land, the wrong kind, a hidden strategic deposit and the centre are refused.
    assert!(
        blocker(&s, &c, FARM, at(4, 5))
            .unwrap()
            .contains("no known resource")
    );
    assert!(
        blocker(&s, &c, ImprovementKind::Mine, at(6, 5))
            .unwrap()
            .contains("needs a Farm")
    );
    assert!(
        blocker(&s, &c, ImprovementKind::Mine, at(7, 5))
            .unwrap()
            .contains("no known resource")
    );
    s.player_mut(P0)
        .techs
        .extend([TechId::BronzeWorking, TechId::IronWorking]);
    let c = s.cities[&id].clone();
    assert_eq!(blocker(&s, &c, ImprovementKind::Mine, at(7, 5)), None);
    // Owned but beyond the work radius: claimed to block rivals only.
    let far = at(9, 5);
    assert_eq!(far.distance(c.pos), 4);
    s.map.get_mut(far).unwrap().resource = Some(Resource::Wheat);
    {
        let t = s.map.get_mut(far).unwrap();
        t.owner = Some(P0);
        t.city = Some(id);
    }
    assert!(
        blocker(&s, &c, FARM, far)
            .unwrap()
            .contains("Only worked tiles")
    );
    // Not this city's tile.
    s.map.get_mut(at(6, 5)).unwrap().city = None;
    assert!(
        blocker(&s, &c, FARM, at(6, 5))
            .unwrap()
            .contains("isn't owned")
    );
    s.map.get_mut(at(6, 5)).unwrap().city = Some(id);
    // Already improved.
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    assert!(
        blocker(&s, &c, FARM, at(6, 5))
            .unwrap()
            .contains("already improved")
    );
    assert!(candidates(&s, &c).iter().all(|i| *i != farm(at(6, 5))));
}

#[test]
fn a_city_builds_the_improvement_and_the_tile_changes() {
    let (mut s, id) = game();
    let tile = at(6, 5);
    let cost = economy::production_cost(&s, P0, farm(tile));
    assert_eq!(cost, base_cost(FARM));
    assert!(cost >= 30, "Listed cost, never cheaper");
    let bad = Order::SetProduction {
        city: id,
        item: ProductionItem::Improvement {
            improvement: ImprovementKind::Mine,
            tile,
        },
    };
    assert!(!crate::validate_orders(&s, P0, &[bad]).is_empty());
    let order = Order::SetProduction {
        city: id,
        item: farm(tile),
    };
    assert!(crate::validate_orders(&s, P0, std::slice::from_ref(&order)).is_empty());
    let mut next = s.clone();
    let mut orders = orders(P0, vec![order]);
    let mut completed = false;
    for _ in 0..40 {
        let (n, events) = resolve_turn(&next, &orders);
        next = n;
        orders.clear();
        if events.iter().any(|e| {
            matches!(e, crate::events::Event::ProductionCompleted { item, .. } if *item == farm(tile))
        }) {
            completed = true;
            break;
        }
    }
    assert!(completed, "the Farm completes");
    assert!(next.map.get(tile).unwrap().improved);
    assert_eq!(next.cities[&id].producing, None);
    s.map.get_mut(tile).unwrap().improved = true;
    assert!(
        economy::production_blocker(&s, &s.cities[&id], farm(tile), true).is_some(),
        "an improved tile can't be improved again"
    );
}

#[test]
fn population_loss_keeps_the_improvement() {
    let (mut s, id) = game();
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    // Capture keeps the improved land: the conqueror inherits it.
    let mut captured = s.clone();
    let mut events = Vec::new();
    crate::rules::battle::capture_city(&mut captured, id, P1, &mut events);
    assert_eq!(captured.map.get(at(6, 5)).unwrap().owner, Some(P1));
    assert!(captured.map.get(at(6, 5)).unwrap().improved);

    // Growth-claimed tiles are released when population falls.
    let (mut s2, id2) = game();
    s2.map.get_mut(at(6, 5)).unwrap().improved = true;
    s2.cities.get_mut(&id2).unwrap().grown_tiles = vec![at(6, 5)];
    s2.cities.get_mut(&id2).unwrap().population = 1;
    crate::rules::cities::settle_territory(&mut s2, id2, &mut Vec::new());
    assert_eq!(s2.map.get(at(6, 5)).unwrap().owner, None);
    assert!(s2.map.get(at(6, 5)).unwrap().improved);
}

#[test]
fn queued_improvement_is_dropped_when_the_tile_is_lost() {
    let (mut s, id) = game();
    let tile = at(6, 5);
    s.cities.get_mut(&id).unwrap().producing = Some(farm(tile));
    s.cities.get_mut(&id).unwrap().production_stored = 10;
    {
        let t = s.map.get_mut(tile).unwrap();
        t.owner = Some(P1);
        t.city = None;
    }
    let (next, events) = resolve_turn(&s, &BTreeMap::new());
    assert_eq!(next.cities[&id].producing, None);
    assert!(events.iter().any(|e| matches!(
        e,
        crate::events::Event::OrderFailed { reason, .. } if reason.contains("can no longer produce Farm")
    )));
}

#[test]
fn the_view_lists_improvement_options_and_shows_the_built_tile() {
    let (mut s, id) = game();
    let v = player_view(&s, P0);
    let detail = v
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    let opt = detail
        .options
        .iter()
        .find(|o| o.item == farm(at(6, 5)))
        .expect("the Wheat tile is an option");
    let preview = opt.improvement.as_ref().unwrap();
    assert_eq!(preview.gain, Yields::new(1, 0, 0, 0));
    assert_eq!(preview.upgrade.as_ref().unwrap().name, "Irrigated Farm");
    assert!(opt.turns > 0);
    assert!(
        !detail.options.iter().any(
            |o| matches!(o.item, ProductionItem::Improvement { tile, .. } if tile == at(7, 5))
        ),
        "hidden Iron isn't offered"
    );
    s.map.get_mut(at(6, 5)).unwrap().improved = true;
    let v = player_view(&s, P0);
    let tile = v.tiles[s.map.index(at(6, 5)).unwrap()].improvement.unwrap();
    assert_eq!(tile.kind, FARM);
    assert!(!tile.upgraded);
    s.player_mut(P0).techs.insert(TechId::CropRotation);
    let v = player_view(&s, P0);
    assert!(
        v.tiles[s.map.index(at(6, 5)).unwrap()]
            .improvement
            .unwrap()
            .upgraded
    );
    // A rival never sees the upgraded form.
    let v = player_view(&s, P1);
    if let Some(i) = v.tiles[s.map.index(at(6, 5)).unwrap()].improvement {
        assert!(!i.upgraded);
    }
}

#[test]
fn an_unimproved_tile_keeps_old_hashes() {
    let (s, _) = game();
    let json = serde_json::to_string(&s.map.tiles).unwrap();
    assert!(!json.contains("improved"));
}

#[test]
fn an_improvement_banks_its_progress_like_any_build() {
    let (mut s, id) = game();
    let tile = at(6, 5);
    let spearman = ProductionItem::Unit {
        unit: UnitKind::Spearman,
    };
    let c = s.cities.get_mut(&id).unwrap();
    c.production_stored = 0;
    c.switch_production(Some(farm(tile)));
    c.production_stored = 12;
    c.switch_production(Some(spearman));
    assert_eq!(c.banked(farm(tile)), 12);
    assert_eq!(c.production_stored, 0);
    c.switch_production(Some(farm(tile)));
    assert_eq!(c.production_stored, 12, "switching back resumes the bank");
}

#[test]
fn an_improvement_can_be_bought_and_the_tile_changes_at_once() {
    let (mut s, id) = game();
    let tile = at(6, 5);
    let item = farm(tile);
    let cost = economy::buy_cost_for(&s, &s.cities[&id], item).expect("buyable");
    assert_eq!(cost, 2 * economy::production_cost(&s, P0, item));
    let order = Order::BuyItem { city: id, item };
    assert!(crate::validate_orders(&s, P0, std::slice::from_ref(&order)).is_empty());
    let (next, events) = resolve_turn(&s, &orders(P0, vec![order]));
    assert!(next.map.get(tile).unwrap().improved);
    assert!(events.iter().any(|e| matches!(
        e,
        crate::events::Event::ItemBought { gold, item: bought, .. } if *gold == cost && *bought == item
    )));
    s.map.get_mut(tile).unwrap().improved = true;
    assert!(economy::buy_cost_for(&s, &s.cities[&id], item).is_none());
}

#[test]
fn a_city_founded_on_an_improved_tile_replaces_the_improvement() {
    let (mut s, _) = game();
    let tile = at(9, 9);
    s.map.get_mut(tile).unwrap().resource = Some(Resource::Wheat);
    s.map.get_mut(tile).unwrap().improved = true;
    s.map.get_mut(tile).unwrap().owner = Some(P0);
    let settler = crate::game::spawn_unit(&mut s, P0, UnitKind::Settler, tile);
    crate::rules::cities::found_city(&mut s, settler, &mut Vec::new()).expect("legal site");
    assert!(!s.map.get(tile).unwrap().improved);
    // The centre still yields its resource (a city centre counts as improved).
    let new = s.cities.values().find(|c| c.pos == tile).unwrap();
    let y = economy::city_yields(&s, new);
    assert!(y.sources.iter().any(|x| x.label == "Resources: Wheat"));
}

#[test]
fn egypt_is_not_offered_a_quarry_but_may_still_order_one() {
    let (mut s, id) = game();
    s.player_mut(P0).faction = Faction::Egypt;
    s.map.get_mut(at(4, 5)).unwrap().resource = Some(Resource::Stone);
    let quarry = ProductionItem::Improvement {
        improvement: ImprovementKind::Quarry,
        tile: at(4, 5),
    };
    let offered = |s: &GameState| {
        player_view(s, P0)
            .cities
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .detail
            .as_ref()
            .unwrap()
            .options
            .iter()
            .any(|o| o.item == quarry)
    };
    // The Nile: Egypt has no Production, so a Quarry gives it nothing and the view leaves it out.
    assert!(!offered(&s));
    assert!(offered_farm(&s, id), "the Farm is still offered");
    assert!(blocker(&s, &s.cities[&id], ImprovementKind::Quarry, at(4, 5)).is_none());
    s.player_mut(P0).faction = Faction::Byzantium;
    assert!(offered(&s), "other factions see the Quarry");
}

fn offered_farm(s: &GameState, id: CityId) -> bool {
    player_view(s, P0)
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|o| o.item == farm(at(6, 5)))
}

use super::*;
use crate::Knobs;
use crate::fixture::{ME, ViewBuilder};
use legatum_core::{Faction, Hex, PlayerView};

fn planner(v: &PlayerView) -> Planner<'_> {
    Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7)
}

#[test]
fn factions_without_a_gated_goal_item_ask_for_no_extra_research() {
    // The audit: these goals wait on nothing the research planner could still deliver.
    for faction in [
        Faction::Switzerland,
        Faction::Athens,
        Faction::Byzantium,
        Faction::Celts,
        Faction::Germany,
        Faction::Sparta,
        Faction::Mongols,
    ] {
        let mut v = ViewBuilder::new(24, 18)
            .faction(faction)
            .city(1, ME, Hex::new(4, 4), 4)
            .build();
        v.players[0].faction_goal_done = false;
        assert!(planner(&v).goal_techs().is_empty(), "{faction:?}");
    }
}
#[test]
fn india_commits_a_safe_city_to_the_diwan_after_all_offices() {
    use crate::Knobs;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::{AiDifficulty, BuildingKind, Faction, Order, ProductionItem, UnitKind};

    let mut view = ViewBuilder::new(24, 16)
        .faction(Faction::India)
        .city(1, ME, off(4, 4), 3)
        .city(2, ME, off(9, 4), 3)
        .unit(1, ME, UnitKind::Soldier, off(4, 4))
        .unit(2, ME, UnitKind::Soldier, off(9, 4))
        .build();
    view.turn = 100;
    view.players[0].faction_goal_done = false;
    let diwan = ProductionItem::Building {
        building: BuildingKind::ImperialDiwan,
    };
    for city in &mut view.cities {
        let detail = city.detail.as_mut().unwrap();
        detail.buildings.push(BuildingKind::Market);
        let mut option = detail.options[0].clone();
        option.item = diwan;
        option.cost = 300;
        option.turns = 15;
        option.buy_cost = None;
        detail.options.push(option);
    }
    let mut planner =
        crate::scripted::Planner::new(&view, Knobs::for_faction(Faction::India), Faction::India, 7);
    planner.difficulty = AiDifficulty::Hard;
    planner.decisions.india_diwan_commit = 1;
    planner.plan_faction_goal();
    assert!(planner.orders.iter().any(|order| matches!(order,
        Order::SetProduction { item, .. } if *item == diwan)));
}

//! Reproducible, peaceful city investment comparisons using the real turn resolver.
//! cargo run -p legatum-ai --release --example city_economy_lab > results.csv
use legatum_ai::city_automation_choice;
use legatum_core::{resources::Resource, *};
use std::collections::BTreeMap;

fn main() {
    println!("case,policy,gold,science,production,population,improvements,first_pick");
    for (name, terrain, population, late) in [
        ("young-fertile", Terrain::Grassland, 1, false),
        ("young-hills", Terrain::Hills, 1, false),
        ("young-plains", Terrain::Plains, 1, false),
        ("established-fertile", Terrain::Grassland, 6, false),
        ("established-hills", Terrain::Hills, 6, false),
        ("late-fertile", Terrain::Grassland, 10, true),
        ("late-hills", Terrain::Hills, 10, true),
        ("late-plains", Terrain::Plains, 10, true),
    ] {
        for faction in [
            Faction::Byzantium,
            Faction::Egypt,
            Faction::Germany,
            Faction::India,
        ] {
            for seed in [11, 29] {
                let mut setup = GameSetup::standard(seed, [faction, Faction::Athens]);
                setup.players.truncate(2);
                setup.players[1].team = TeamId(1);
                setup.width = 32;
                setup.height = 24;
                setup.barbarians = false;
                let mut initial = new_game(&setup);
                let city = initial.cities_of(PlayerId(0)).next().unwrap().id;
                let center = initial.cities[&city].pos;
                let land: Vec<_> = initial
                    .map
                    .hexes()
                    .filter(|h| h.distance(center) <= 3)
                    .collect();
                for h in land {
                    let t = initial.map.get_mut(h).unwrap();
                    t.terrain = terrain;
                    t.resource = None;
                    t.ruin = false;
                    t.wonder = None;
                }
                for (h, resource) in center.neighbors().into_iter().zip([
                    Resource::Wheat,
                    Resource::Cattle,
                    Resource::Copper,
                    Resource::Iron,
                    Resource::Stone,
                    Resource::Coal,
                ]) {
                    initial.map.get_mut(h).unwrap().resource = Some(resource);
                }
                initial.cities.get_mut(&city).unwrap().population = population;
                let mut ring: Vec<_> = initial
                    .map
                    .hexes()
                    .filter(|h| h.distance(center) == 2)
                    .collect();
                ring.sort();
                let grown: Vec<_> = ring.into_iter().take((population - 1) as usize).collect();
                for h in &grown {
                    let tile = initial.map.get_mut(*h).unwrap();
                    tile.owner = Some(PlayerId(0));
                    tile.city = Some(city);
                }
                initial.cities.get_mut(&city).unwrap().tiles_claimed = grown.len() as i32;
                initial.cities.get_mut(&city).unwrap().grown_tiles = grown;
                let techs: Vec<_> = if late {
                    TechId::ALL.to_vec()
                } else {
                    vec![
                        TechId::Irrigation,
                        TechId::BronzeWorking,
                        TechId::Masonry,
                        TechId::Currency,
                        TechId::Writing,
                    ]
                };
                initial.player_mut(PlayerId(0)).techs.extend(techs);
                if late {
                    initial.turn = 250;
                }
                for policy in [
                    "automation",
                    "gold-first",
                    "growth-first",
                    "production-first",
                ] {
                    let mut state = initial.clone();
                    let mut totals = Yields::ZERO;
                    let mut first = String::new();
                    for _ in 0..60 {
                        let view = player_view(&state, PlayerId(0));
                        let c = view.cities.iter().find(|c| c.id == city).unwrap();
                        let d = c.detail.as_ref().unwrap();
                        totals += d.yields.total;
                        let idle = d.producing.is_none()
                            || matches!(d.producing, Some(ProductionItem::Project { .. }));
                        let mut orders = Vec::new();
                        if idle {
                            let pick = if policy == "automation" {
                                city_automation_choice(&view, city, CityBuildFocus::Economy)
                            } else {
                                d.options
                                    .iter()
                                    .filter(|o| o.resource_blocker.is_none() && o.turns >= 0)
                                    .filter_map(|o| {
                                        let y = match o.item {
                                            ProductionItem::Building { building } => {
                                                state
                                                    .setup
                                                    .rule_experiments
                                                    .building_stats_for(
                                                        view.faction_of(view.me),
                                                        building,
                                                    )
                                                    .yields
                                            }
                                            ProductionItem::Improvement { .. } => {
                                                o.improvement.as_ref()?.gain
                                            }
                                            _ => return None,
                                        };
                                        let primary = match policy {
                                            "gold-first" => {
                                                y.gold * 8
                                                    + y.science * 5
                                                    + y.production * 3
                                                    + y.food
                                            }
                                            "growth-first" => {
                                                y.food * 8
                                                    + y.production * 5
                                                    + y.gold * 3
                                                    + y.science
                                            }
                                            _ => {
                                                y.production * 8
                                                    + y.food * 5
                                                    + y.gold * 3
                                                    + y.science
                                            }
                                        };
                                        (primary > 0)
                                            .then_some((primary * 100 / (o.turns + 5), o.item))
                                    })
                                    .max()
                                    .map(|(_, item)| item)
                                    .or_else(|| {
                                        d.options
                                            .iter()
                                            .find(|o| {
                                                o.item
                                                    == ProductionItem::Project {
                                                        project: IndustryProject::Gold,
                                                    }
                                            })
                                            .map(|o| o.item)
                                    })
                            };
                            if let Some(item) = pick {
                                if first.is_empty() {
                                    first = format!("{item:?}").replace(',', ";");
                                }
                                orders.push(Order::SetProduction { city, item });
                            }
                        }
                        let (next, _) =
                            resolve_turn(&state, &BTreeMap::from([(PlayerId(0), orders)]));
                        assert!(next.turn > state.turn, "fixture ended early");
                        state = next;
                    }
                    let improvements = state
                        .map
                        .hexes()
                        .filter(|h| h.distance(center) <= 3 && state.map.get(*h).unwrap().improved)
                        .count();
                    println!(
                        "{faction:?}-{name}-{seed},{policy},{},{},{},{},{},{}",
                        totals.gold,
                        totals.science,
                        totals.production,
                        state.cities[&city].population,
                        improvements,
                        first
                    );
                }
            }
        }
    }
}

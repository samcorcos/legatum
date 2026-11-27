//! Delegated city development, separate from Hard's war and victory priorities.
//! All candidates and forecasts come from the same fogged view shown to the player.
use legatum_core::{
    CityId, Faction, IndustryProject, PlayerView, ProductionItem, ScienceConversion, TechId,
    Yields, data,
};

pub(crate) fn choose(view: &PlayerView, city: CityId) -> Option<ProductionItem> {
    let city = view
        .cities
        .iter()
        .find(|c| c.id == city && c.owner == view.me)?;
    let d = city.detail.as_ref()?;
    if d.resistance > 0 || city.occupation_lock.is_some() {
        return None;
    }
    let faction = view.faction_of(view.me);
    let base = d.idle_yields.as_ref().unwrap_or(&d.yields).total;
    let surplus = base.food - data::FOOD_PER_POP * city.population;
    // Food protects existing citizens first. Growth earns value when it claims a worked tile
    // or opens another economic building; beyond the work radius, more population is no goal.
    let nile = faction == Faction::Egypt && view.rule_experiments.egypt_nile();
    let growth = d.next_claim.as_ref().is_some_and(|c| c.worked)
        || d.population_unlocks.iter().any(|u| {
            u.population > city.population && matches!(u.item, ProductionItem::Building { .. })
        });
    let food_weight = if surplus <= 0 {
        14
    } else if growth && surplus < 4 {
        7
    } else if growth {
        3
    } else {
        0
    };
    let production_weight = if base.production < 8 { 6 } else { 3 };
    let food_weight = if nile {
        food_weight.max(production_weight)
    } else {
        food_weight
    };
    let science_weight = if TechId::ALL.iter().all(|t| view.research.known.contains(t)) {
        if view.research.science_conversion == ScienceConversion::Gold {
            4
        } else {
            0
        }
    } else {
        5
    };
    let raw_gain = |mut y: Yields| {
        if nile {
            y.food = y.food * view.rule_experiments.egypt_nile_food_percent() / 100;
            y.production = 0;
        }
        y
    };
    let points = |y: Yields| {
        8 * y.gold
            + science_weight * y.science
            + production_weight * y.production
            + food_weight * y.food
    };
    let horizon = view
        .turn_limit
        .unwrap_or(data::DEFAULT_TURN_LIMIT)
        .saturating_sub(view.turn)
        .min(60) as i32;
    let gold = d.options.iter().find(|o| {
        o.item
            == ProductionItem::Project {
                project: IndustryProject::Gold,
            }
    });
    let forgone_gold = gold.map_or(0, |o| (o.yields.total.gold - base.gold).max(0));
    let investment = d
        .options
        .iter()
        .filter_map(|o| {
            if o.turns < 0
                || o.resource_blocker.is_some()
                || d.producing == Some(o.item)
                || !view
                    .rule_experiments
                    .available_to_player(faction, view.era_of(view.me), o.item)
            {
                return None;
            }
            let mut gain = match o.item {
                ProductionItem::Building { building } => {
                    let quoted = view
                        .research
                        .unlocks
                        .iter()
                        .find(|u| u.item == o.item)
                        .and_then(|u| u.cities.iter().find(|c| c.city == city.id))
                        .filter(|u| !u.yields_after.sources.is_empty());
                    // Full player views include exact effects (distance, supply and faction rules).
                    // Sparse decision/test views use the canonical incremental building yields.
                    quoted.map_or_else(
                        || {
                            raw_gain(
                                view.rule_experiments
                                    .building_stats_for(faction, building)
                                    .yields,
                            )
                        },
                        |u| {
                            let y = u.yields_after.total;
                            Yields::new(
                                y.food - base.food,
                                y.production - base.production,
                                y.gold - base.gold,
                                y.science - base.science,
                            )
                        },
                    )
                }
                ProductionItem::Improvement { tile, .. } => {
                    let preview = o.improvement.as_ref()?;
                    let mut gain = if tile.distance(city.pos) <= data::CITY_WORK_RADIUS {
                        raw_gain(preview.gain)
                    } else {
                        Yields::ZERO
                    };
                    // A strategic improvement can also bring an existing idle industry back online.
                    // Use its real incremental yields rather than a military supply score.
                    if view.supply.iter().any(|s| {
                        s.resource == preview.resource && s.idle.len() == 1 && s.spare == -1
                    }) && let Some(idle) = d
                        .idle_buildings
                        .iter()
                        .find(|b| b.resource == preview.resource)
                    {
                        gain += raw_gain(
                            view.rule_experiments
                                .building_stats_for(faction, idle.building)
                                .yields,
                        );
                    }
                    gain
                }
                _ => return None,
            };
            if o.item
                == (ProductionItem::Building {
                    building: legatum_core::BuildingKind::Market,
                })
            {
                gain.gold += city.revenue_office_upkeep_savings.unwrap_or(0)
                    + city.revenue_office_remittance_gain.unwrap_or(0);
            }
            let benefit = points(gain);
            let turns = o.turns.max(1);
            // Production invested now forgoes the Gold project. Reject investments that cannot
            // repay that income inside the bounded planning window; banked work is in o.turns.
            let net = benefit * (horizon - turns).max(0) - 8 * forgone_gold * turns;
            (benefit > 0 && net > 0).then_some((net * 100 / (turns + 5), o.item))
        })
        .max();
    // A growth project is an investment too. Compare how much sooner it claims the next
    // worked tile than the Gold project. Egypt must choose Grow to grow at all.
    let growth_project = d
        .options
        .iter()
        .filter(|o| {
            matches!(
                o.item,
                ProductionItem::Project {
                    project: IndustryProject::Food | IndustryProject::Grow
                }
            )
        })
        .filter_map(|o| {
            let claim = d.next_claim.as_ref().filter(|c| c.worked)?;
            let food_left = (d.growth_threshold - d.food_stored).max(1);
            let rate = o.yields.total.food - data::FOOD_PER_POP * city.population;
            if rate <= 0 {
                return None;
            }
            let turns = (food_left + rate - 1) / rate;
            if turns >= horizon {
                return None;
            }
            let base_rate = gold.map_or(surplus, |g| {
                g.yields.total.food - data::FOOD_PER_POP * city.population
            });
            let without = if base_rate > 0 {
                (food_left + base_rate - 1) / base_rate
            } else {
                horizon
            };
            let saved = (without.min(horizon) - turns).max(0);
            let mut gain = raw_gain(claim.yields);
            gain.food -= data::FOOD_PER_POP;
            gain.science += (city.population + 1) / 2 - city.population / 2;
            // Growth has to repay income, not an abstract stock of unused build capacity.
            // In Nile cities the added surplus matters at the Gold project's conversion rate.
            let growth_points = if nile {
                let food = gain.food;
                gain.food = 0;
                points(gain)
                    + 8 * food
                        * view.rule_experiments.egypt_nile_build_percent()
                        * view
                            .rule_experiments
                            .project_percent(faction, IndustryProject::Gold)
                        / 10_000
            } else {
                points(gain)
            };
            let benefit = growth_points * saved - 8 * forgone_gold * turns;
            (benefit > 0).then_some((benefit * 100 / (turns + 5), o.item))
        })
        .max();
    investment
        .into_iter()
        .chain(growth_project)
        .max()
        .map(|(_, item)| item)
        .or_else(|| gold.map(|o| o.item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{ME, ViewBuilder, off};
    use legatum_core::{BuildingKind, ImprovementKind, Resource, TechId, view::ImprovementPreview};

    fn view() -> PlayerView {
        let mut v = ViewBuilder::new(20, 20)
            .known_techs(&[TechId::Currency, TechId::Writing, TechId::BronzeWorking])
            .city(1, ME, off(8, 8), 4)
            .build();
        v.research.unlocks.clear();
        v.cities[0].detail.as_mut().unwrap().yields.total = Yields::new(14, 12, 3, 3);
        v
    }
    fn only(v: &mut PlayerView, buildings: &[BuildingKind]) {
        let d = v.cities[0].detail.as_mut().unwrap();
        d.options.retain(|o| matches!(o.item, ProductionItem::Building { building } if buildings.contains(&building)));
        for o in &mut d.options {
            o.turns = 5;
        }
    }
    #[test]
    fn wonder_placement_is_never_delegated_even_when_it_would_pay_back() {
        let mut v = view();
        only(&mut v, &[BuildingKind::Market]);
        let d = v.cities[0].detail.as_mut().unwrap();
        assert!(!d.options.is_empty());
        for option in &mut d.options {
            option.item = ProductionItem::Wonder {
                building: BuildingKind::GreatLibrary,
                tile: off(8, 8),
            };
            option.turns = 1;
        }
        assert_eq!(choose(&v, CityId(1)), None);
    }

    #[test]
    fn comparable_investments_favor_gold_then_science() {
        let mut v = view();
        only(
            &mut v,
            &[
                BuildingKind::Market,
                BuildingKind::Library,
                BuildingKind::Workshop,
            ],
        );
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Market
            })
        );
        v.cities[0].detail.as_mut().unwrap().options.retain(|o| {
            o.item
                != ProductionItem::Building {
                    building: BuildingKind::Market,
                }
        });
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Library
            })
        );
    }
    #[test]
    fn starving_city_feeds_itself_before_income() {
        let mut v = view();
        only(&mut v, &[BuildingKind::Granary, BuildingKind::Market]);
        v.cities[0].detail.as_mut().unwrap().yields.total.food = 7;
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Granary
            })
        );
    }
    #[test]
    fn resource_improvements_compete_on_payback_and_cannot_train_units() {
        let mut v = view();
        only(&mut v, &[BuildingKind::Market]);
        v.cities[0].detail.as_mut().unwrap().options[0].turns = 10;
        let mut improvement = v.cities[0].detail.as_ref().unwrap().options[0].clone();
        improvement.item = ProductionItem::Improvement {
            improvement: ImprovementKind::Mine,
            tile: off(9, 8),
        };
        improvement.improvement = Some(ImprovementPreview {
            tile: off(9, 8),
            resource: Resource::Copper,
            gain: Yields::new(0, 2, 0, 0),
            upgrade: None,
        });
        improvement.turns = 1;
        let mine = improvement.item;
        v.cities[0].detail.as_mut().unwrap().yields.total.production = 4;
        v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .push(improvement);
        assert_eq!(choose(&v, CityId(1)), Some(mine));
        v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .last_mut()
            .unwrap()
            .resource_blocker = Some("unavailable".into());
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Market
            })
        );
    }
    #[test]
    fn end_of_game_uses_gold_project_instead_of_an_unpayable_investment() {
        let mut v = view();
        v.turn = 499;
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Project {
                project: IndustryProject::Gold
            })
        );
    }
    #[test]
    fn egypt_can_grow_to_a_profitable_claim_instead_of_running_gold_forever() {
        let mut v = view();
        v.players.iter_mut().find(|p| p.id == v.me).unwrap().faction = Faction::Egypt;
        let d = v.cities[0].detail.as_mut().unwrap();
        d.options.retain(|o| {
            o.item
                == ProductionItem::Project {
                    project: IndustryProject::Gold,
                }
        });
        d.options[0].yields.total = Yields::new(8, 0, 4, 3);
        let mut grow = d.options[0].clone();
        grow.item = ProductionItem::Project {
            project: IndustryProject::Grow,
        };
        grow.yields.total = Yields::new(12, 0, 3, 3);
        d.options.push(grow);
        d.growth_threshold = 8;
        d.food_stored = 0;
        d.next_claim = Some(legatum_core::view::ClaimPreview {
            tile: off(9, 8),
            yields: Yields::new(2, 0, 0, 0),
            worked: true,
        });
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Project {
                project: IndustryProject::Grow
            })
        );
        v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .next_claim
            .as_mut()
            .unwrap()
            .worked = false;
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Project {
                project: IndustryProject::Gold
            })
        );
    }
    #[test]
    fn revenue_office_uses_the_visible_treasury_savings() {
        let mut v = view();
        only(&mut v, &[BuildingKind::Market, BuildingKind::Library]);
        v.cities[0]
            .detail
            .as_mut()
            .unwrap()
            .options
            .iter_mut()
            .find(|o| {
                o.item
                    == ProductionItem::Building {
                        building: BuildingKind::Market,
                    }
            })
            .unwrap()
            .turns = 20;
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Library
            })
        );
        v.cities[0].revenue_office_remittance_gain = Some(20);
        assert_eq!(
            choose(&v, CityId(1)),
            Some(ProductionItem::Building {
                building: BuildingKind::Market
            })
        );
    }
}

//! Public victory commitments and private, conditional growth forecasts. No hidden state is
//! needed by a planner: the same quotes are serialized for humans and the fogged-view AI.
use crate::{data, economy, rules::cities, state::City, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrowthForecast {
    pub city: CityId,
    pub population: i32,
    #[serde(default)]
    pub turns_to_six: Option<i32>,
    pub turns_to_eight: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevelopmentForecast {
    pub cities: usize,
    pub tall_cities: usize,
    pub growth: Vec<GrowthForecast>,
    /// First simultaneous pair at population 8+, assuming current yield conditions hold.
    /// None if no pair occurs within the next expected game length.
    pub tall_turns: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GreatWorkView {
    pub player: PlayerId,
    pub completed: Option<BuildingKind>,
    /// Public construction commitment, even outside sight. Never includes coordinates.
    pub building: Option<BuildingKind>,
    pub city: Option<CityId>,
    pub progress: i32,
    pub cost: i32,
    /// Public city Production while this wonder is being built, for the race forecast.
    #[serde(default)]
    pub production_per_turn: i32,
    /// Only the owner's view quotes private production rates and blockers.
    pub turns: Option<i32>,
    pub blocker: Option<String>,
}

/// Reassign citizens after each growth using today's territory, build priorities and buildings.
/// Assumes no builds complete, no settlers consume population, and no territory changes.
/// Resistance delays growth; a non-positive future surplus makes the forecast unavailable.
pub fn turns_to_population(state: &GameState, city: &City, target: i32) -> Option<i32> {
    if city.population >= target {
        return turns_to_population_squatted(state, city, target, &[]);
    }
    turns_to_population_squatted(state, city, target, &economy::denied_near(state, city))
}

/// `turns_to_population` with `economy::denied_near(state, city)` already
/// computed (growth keeps the city's position and owner, so its squatted tiles are fixed).
pub(crate) fn turns_to_population_squatted(
    state: &GameState,
    city: &City,
    target: i32,
    squatted: &[crate::hex::Hex],
) -> Option<i32> {
    grow_to(state, city, target, |c| {
        economy::city_totals_squatted(state, c, c.producing, None, squatted).food
    })
}

/// The growth walk behind `turns_to_population`, with `food(variant)` giving a variant's Food
/// (its total at that population, with resistance cleared).
fn grow_to(
    state: &GameState,
    city: &City,
    target: i32,
    mut food: impl FnMut(&City) -> i32,
) -> Option<i32> {
    if target > crate::validation::MAX_POPULATION {
        return None;
    }
    let mut city = city.clone();
    let mut turns = if city.population >= target {
        0
    } else {
        city.resistance
    };
    city.resistance = 0;
    while city.population < target {
        let surplus = food(&city) - data::FOOD_PER_POP * city.population;
        let threshold = economy::growth_threshold_total(state, &city);
        let remaining = threshold - city.food_stored;
        let step = if remaining <= surplus {
            1
        } else if surplus > 0 {
            (remaining + surplus - 1) / surplus
        } else {
            return None;
        };
        city.food_stored = crate::validation::add_stored(
            city.food_stored,
            step.saturating_mul(surplus).saturating_sub(threshold),
        );
        city.population += 1;
        turns += step;
    }
    Some(turns)
}

/// Keep territory, buildings, research and build priorities fixed; reassign citizens as
/// population changes. Growth uses pre-production yields, so future builds and settler
/// population costs are excluded. Resistance pauses both growth and starvation.
fn turns_to_tall_empire(state: &GameState, player: PlayerId) -> Option<i32> {
    tall_empire_with(state, player, |_, c| economy::city_totals(state, c).food)
}

/// `turns_to_tall_empire` with `food(index, variant)` giving the Food of the `index`th of the
/// player's cities in a forecast variant (population and resistance changed).
fn tall_empire_with(
    state: &GameState,
    player: PlayerId,
    mut food_of: impl FnMut(usize, &City) -> i32,
) -> Option<i32> {
    let mut cities: Vec<_> = state
        .cities_of(player)
        .cloned()
        .map(|c| (c, None))
        .collect();
    if cities.len() < 2 {
        return None;
    }
    if cities.iter().filter(|(c, _)| c.population >= 8).count() >= 2 {
        return Some(0);
    }
    for turn in 1..=crate::data::EXPECTED_GAME_TURNS {
        for (index, (city, food)) in cities.iter_mut().enumerate() {
            if city.resistance > 0 {
                city.resistance -= 1;
                continue;
            }
            // With current yield conditions held fixed, only a population change needs
            // another citizen assignment. Avoid recalculating yields every forecast turn.
            let rate = *food.get_or_insert_with(|| food_of(index, city));
            let population = city.population;
            let threshold = economy::growth_threshold_total(state, city);
            let _ = cities::grow_city(city, rate, threshold);
            if city.population != population {
                *food = None;
            }
        }
        if cities.iter().filter(|(c, _)| c.population >= 8).count() >= 2 {
            return Some(turn as i32);
        }
    }
    None
}

/// `development_forecast` with city yields from `cache` (`yields_cache`).
pub(crate) fn development_forecast_cached(
    state: &GameState,
    player: PlayerId,
    cache: &mut crate::yields_cache::ViewCache,
) -> DevelopmentForecast {
    let mut entries: Vec<_> = state
        .cities_of(player)
        .map(|city| (city, cache.take(state, city)))
        .collect();
    // A variant's Food from the cache, checked against the direct computation in debug builds.
    let food = |entry: &mut crate::yields_cache::CityCache, city: &City, variant: &City| {
        let food = entry
            .totals(
                state,
                city,
                variant.population,
                variant.resistance,
                variant.producing,
            )
            .food;
        debug_assert_eq!(food, economy::city_totals(state, variant).food);
        food
    };
    let growth: Vec<_> = entries
        .iter_mut()
        .map(|(city, entry)| {
            let walk = |target, entry: &mut crate::yields_cache::CityCache| {
                grow_to(state, city, target, |v| food(entry, city, v))
            };
            GrowthForecast {
                city: city.id,
                population: city.population,
                turns_to_six: walk(6, entry),
                turns_to_eight: walk(8, entry),
            }
        })
        .collect();
    let tall_turns = tall_empire_with(state, player, |i, v| {
        let (city, entry) = &mut entries[i];
        food(entry, city, v)
    });
    for (city, entry) in entries {
        cache.put(city.id, entry);
    }
    let forecast = DevelopmentForecast {
        cities: growth.len(),
        tall_cities: growth.iter().filter(|g| g.population >= 8).count(),
        tall_turns,
        growth,
    };
    debug_assert_eq!(forecast, development_forecast(state, player));
    forecast
}

pub fn development_forecast(state: &GameState, player: PlayerId) -> DevelopmentForecast {
    let growth: Vec<_> = state
        .cities_of(player)
        .map(|city| GrowthForecast {
            city: city.id,
            population: city.population,
            turns_to_six: turns_to_population(state, city, 6),
            turns_to_eight: turns_to_population(state, city, 8),
        })
        .collect();
    DevelopmentForecast {
        cities: growth.len(),
        tall_cities: growth.iter().filter(|g| g.population >= 8).count(),
        tall_turns: turns_to_tall_empire(state, player),
        growth,
    }
}

pub fn great_work_views(state: &GameState, viewer: PlayerId) -> Vec<GreatWorkView> {
    state
        .players
        .iter()
        .map(|p| {
            let city = state
                .cities_of(p.id)
                .find(|c| c.producing.is_some_and(data::is_great_work_item));
            let item = city.and_then(|c| c.producing);
            let building = item.and_then(|i| {
                if let ProductionItem::Building { building }
                | ProductionItem::Wonder { building, .. } = i
                {
                    Some(building)
                } else {
                    None
                }
            });
            let cost = item.map_or(0, |i| economy::production_cost(state, p.id, i));
            let progress = city.map_or(0, |c| c.production_stored);
            let met = p.team == state.player(viewer).team
                || state
                    .diplomacy
                    .as_ref()
                    .is_some_and(|d| d.met(p.team, state.player(viewer).team));
            let production_per_turn = city
                .filter(|_| met)
                .map_or(0, |c| economy::city_totals(state, c).production);
            let blocker = city.zip(building).and_then(|(c, b)| {
                if p.id != viewer {
                    None
                } else if c.resistance > 0 {
                    Some("Resisting occupation".into())
                } else {
                    economy::building_blocker(state, c, b, false)
                }
            });
            let turns = city
                .filter(|_| p.id == viewer && blocker.is_none())
                .and_then(|c| {
                    let rate = economy::city_totals(state, c).production;
                    if progress >= cost {
                        Some(1)
                    } else if rate > 0 {
                        Some((cost - progress + rate - 1) / rate)
                    } else {
                        None
                    }
                });
            GreatWorkView {
                player: p.id,
                completed: p.great_work_completed,
                building,
                city: city.map(|c| c.id),
                progress,
                cost,
                production_per_turn,
                turns,
                blocker,
            }
        })
        .collect()
}

//! Research previews use the same prices, city gates and yield rules as resolution.

use crate::{data, economy, rules::units::upgrade_price, state::GameState, types::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlockCity {
    pub city: CityId,
    /// Ignores the unresearched technology, but checks all city requirements today.
    pub blocker: Option<String>,
    pub build_turns: Option<i32>,
    pub yields_after: economy::CityYields,
    pub food_surplus_after: i32,
    /// Hypothetical population-8 ETA with the food building installed at current population
    /// and stored food, using balanced tiles. Excludes research and construction time.
    pub turns_to_eight_after: Option<i32>,
    #[serde(default)]
    pub turns_to_six_after: Option<i32>,
    /// Installed family at max(current population, its gate); zero stored food if raised.
    /// Excludes research, construction, and the time needed to reach the population gate.
    #[serde(default)]
    pub turns_to_eight_at_gate: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechUnlock {
    pub tech: TechId,
    pub item: ProductionItem,
    pub name: String,
    pub cost: i32,
    pub cost_breakdown: Breakdown,
    /// Full purchase price with no stored production; no empire buys Great Works.
    pub buy_cost: Option<i32>,
    /// Additional yields from this paid step; earlier family tiers remain in place.
    pub yields: Yields,
    pub city_strength: i32,
    pub unit_strength: i32,
    pub unit_ranged: i32,
    pub unit_range: i32,
    pub unit_moves: i32,
    pub siege_bonus: i32,
    pub upgrade_cost: Option<i32>,
    pub upgrade_from: Option<UnitKind>,
    pub predecessor: Option<BuildingKind>,
    pub min_population: i32,
    pub cities: Vec<UnlockCity>,
    /// Why this faction cannot build the item yet ("Walls for the Mongols from the Medieval era"):
    /// the card lists it, but says this in place of the build line, price and forecasts.
    #[serde(default)]
    pub locked_note: Option<String>,
}

pub fn cost_breakdown(state: &GameState, player: PlayerId, item: ProductionItem) -> Breakdown {
    let base = data::item_cost(item);
    let faction = state.player(player).faction;
    let modified = match item {
        ProductionItem::Unit { unit } => {
            state
                .setup
                .rule_experiments
                .unit_stats_for(faction, unit)
                .cost
        }
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            state
                .setup
                .rule_experiments
                .building_stats_for(faction, building)
                .cost
        }
        ProductionItem::Project { .. } => 0,
        ProductionItem::Improvement { improvement, .. } => {
            crate::improvements::base_cost(improvement)
        }
    };
    let mut result = Breakdown::default();
    result.add("Listed cost", base);
    result.add("Faction", modified - base);
    let experimental = state
        .setup
        .rule_experiments
        .base_production_cost(faction, item);
    result.add("Balance override", experimental - modified);
    if item
        == (ProductionItem::Unit {
            unit: UnitKind::Settler,
        })
    {
        result.add(
            "Previously trained Settlers",
            state.setup.rule_experiments.settler_cost_step()
                * state.player(player).settlers_trained,
        );
    }
    if economy::sparta_nonmilitary_premium(state, player, item) {
        result.add("Sparta: non-military +50%", (result.total + 1) / 2);
    }
    result
}

/// The card line that replaces the build line for an item the faction lists but cannot build yet.
fn locked_note(
    state: &GameState,
    player: PlayerId,
    item: ProductionItem,
    name: &str,
) -> Option<String> {
    let p = state.player(player);
    if state
        .setup
        .rule_experiments
        .available_to_player(p.faction, state.era_of(player), item)
    {
        return None;
    }
    Some(format!(
        "{name} for the {} from the Medieval era",
        data::faction_name(p.faction)
    ))
}

pub fn tech_unlocks(state: &GameState, player: PlayerId) -> Vec<TechUnlock> {
    let mut cache = crate::yields_cache::ViewCache::default();
    cache.begin(state);
    tech_unlocks_for(state, player, true, &mut cache)
}

/// `tech_unlocks`; without `full`, the after-yields carry totals only and the growth
/// forecasts are None (`view::ViewDetail::Decisions`).
pub(crate) fn tech_unlocks_for(
    state: &GameState,
    player: PlayerId,
    full: bool,
    cache: &mut crate::yields_cache::ViewCache,
) -> Vec<TechUnlock> {
    let p = state.player(player);
    let buy_percent = economy::buy_percent(state, player);
    let items = data::production_items();
    // Per-city previews shared across items: a build's Production depends on the item only
    // through its auto focus, and a unit leaves the city's buildings (so its after-yields)
    // unchanged. Computed lazily; results are identical to recomputing per item.
    let mut bases: Vec<CityBase> = state
        .cities_of(player)
        .map(|city| CityBase {
            city,
            cached: cache.take(state, city),
            production: Vec::new(),
        })
        .collect();
    // Only what this faction can build and that does something for it, under its own name.
    let unlocks = items
        // A card lists what the technology will unlock for the faction, whatever its era now: the
        // Mongols' Walls are on the Masonry card although they raise them from the Medieval era.
        .filter(|item| {
            let era = state.era_of(player).max(data::MONGOL_WALLS_ERA);
            state
                .setup
                .rule_experiments
                .available_to_player(p.faction, era, *item)
        })
        .filter_map(|item| {
            let tech = data::item_requires_for_rules(state.setup.rule_experiments, item)?;
            let cost = economy::production_cost(state, player, item);
            let mut unlock = TechUnlock {
                tech,
                item,
                name: String::new(),
                cost,
                cost_breakdown: cost_breakdown(state, player, item),
                buy_cost: (!economy::is_unbuyable(item)).then_some(cost * buy_percent / 100),
                yields: Yields::ZERO,
                city_strength: 0,
                unit_strength: 0,
                unit_ranged: 0,
                unit_range: 0,
                unit_moves: 0,
                siege_bonus: 0,
                upgrade_cost: None,
                upgrade_from: None,
                predecessor: None,
                min_population: 0,
                cities: Vec::new(),
                locked_note: None,
            };
            match item {
                ProductionItem::Project { .. } | ProductionItem::Improvement { .. } => {
                    unreachable!("projects and improvements are not tech unlocks")
                }
                ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
                    let stats = state
                        .setup
                        .rule_experiments
                        .building_stats_for(p.faction, building);
                    unlock.name = data::building_name_for(p.faction, building).into();
                    unlock.locked_note = locked_note(state, player, item, &unlock.name);
                    unlock.yields = stats.yields;
                    unlock.city_strength = stats.city_strength;
                    unlock.predecessor = data::building_predecessor(building);
                    unlock.min_population = data::building_min_population(building);
                }
                ProductionItem::Unit { unit } => {
                    let stats = state.setup.rule_experiments.unit_stats_for(p.faction, unit);
                    unlock.name = data::unit_name_for(p.faction, unit).into();
                    unlock.locked_note = locked_note(state, player, item, &unlock.name);
                    unlock.unit_strength = stats.strength;
                    unlock.unit_ranged = stats.ranged;
                    unlock.unit_range = stats.range;
                    unlock.unit_moves =
                        state.setup.rule_experiments.unit_moves_for(p.faction, unit);
                    unlock.siege_bonus = data::siege_city_bonus(unit);
                    unlock.upgrade_from = UnitKind::ALL
                        .into_iter()
                        .find(|base| data::unit_stats(*base).upgrades_to == Some(unit));
                    unlock.upgrade_cost = unlock
                        .upgrade_from
                        .and_then(|base| upgrade_price(state, player, base, false))
                        .map(|c| c.total);
                }
            }
            unlock.cities = bases
                .iter_mut()
                .map(|base| {
                    let city = base.city;
                    // Decision views keep only whether a block exists (empty words).
                    let mut blocker = match item {
                        ProductionItem::Building { building }
                        | ProductionItem::Wonder { building, .. }
                            if full =>
                        {
                            economy::building_blocker(state, city, building, true)
                        }
                        ProductionItem::Building { building }
                        | ProductionItem::Wonder { building, .. } => {
                            economy::building_check::<economy::Blocked>(state, city, building, true)
                                .map(|_| String::new())
                        }
                        ProductionItem::Unit { .. }
                        | ProductionItem::Project { .. }
                        | ProductionItem::Improvement { .. } => None,
                    };
                    if city.resistance > 0 {
                        blocker = Some(if full {
                            "Resisting occupation".into()
                        } else {
                            String::new()
                        });
                    }
                    let focus = economy::auto_focus(city, Some(item));
                    let production = match base.production.iter().find(|(f, _)| *f == focus) {
                        Some((_, p)) => *p,
                        None => {
                            let p = base
                                .cached
                                .totals(state, city, city.population, city.resistance, Some(item))
                                .production;
                            base.production.push((focus, p));
                            p
                        }
                    };
                    let build_turns = (blocker.is_none() && production > 0)
                        .then(|| (cost + production - 1) / production.max(1));
                    let installed = match item {
                        ProductionItem::Building { building } => Some(building),
                        _ => None,
                    };
                    // Decision views work out after-yields only where a brain reads them:
                    // the industry buildings' Production gain (`legatum_ai::production`).
                    let read = full
                        || matches!(installed, Some(BuildingKind::Forge | BuildingKind::Factory));
                    let (yields_after, food_surplus_after) = if read {
                        let yields = base.cached.yields(state, city, installed, full);
                        let food = yields.total.food - data::FOOD_PER_POP * city.population;
                        (yields, food)
                    } else {
                        (economy::CityYields::empty(), 0)
                    };
                    // Growth forecasts (human views only) walk a copy with the item built.
                    let (turns_to_eight_after, turns_to_six_after, turns_to_eight_at_gate) =
                        if full && unlock.yields.food > 0 {
                            let mut completed = city.clone();
                            if let Some(building) = installed {
                                economy::install_building(&mut completed, building);
                            }
                            completed.producing = None;
                            let squatted = base.cached.squatted();
                            let after = crate::projects::turns_to_population_squatted(
                                state, &completed, 8, squatted,
                            );
                            let six = crate::projects::turns_to_population_squatted(
                                state, &completed, 6, squatted,
                            );
                            if completed.population < unlock.min_population {
                                completed.population = unlock.min_population;
                                completed.food_stored = 0;
                            }
                            let gate = crate::projects::turns_to_population_squatted(
                                state, &completed, 8, squatted,
                            );
                            (after, six, gate)
                        } else {
                            (None, None, None)
                        };
                    UnlockCity {
                        turns_to_six_after,
                        turns_to_eight_at_gate,
                        turns_to_eight_after,
                        city: city.id,
                        blocker,
                        build_turns,
                        yields_after,
                        food_surplus_after,
                    }
                })
                .collect();
            Some(unlock)
        })
        .collect();
    for base in bases {
        cache.put(base.city.id, base.cached);
    }
    unlocks
}

struct CityBase<'a> {
    city: &'a crate::state::City,
    /// This city's cached yields (across items, and across turns while its inputs hold).
    cached: crate::yields_cache::CityCache,
    /// Current Production by auto focus.
    production: Vec<(CityFocus, i32)>,
}

/// Actual city-by-city change from an immediate yield breakthrough, after faction penalties.
pub fn income_change(
    state: &GameState,
    player: PlayerId,
    tech: TechId,
) -> Vec<economy::YieldSource> {
    if !matches!(
        tech,
        TechId::Irrigation | TechId::PrintingPress | TechId::FusionPower | TechId::RecursiveAi
    ) || state.player(player).techs.contains(&tech)
    {
        return Vec::new();
    }
    state
        .cities_of(player)
        .filter_map(|city| {
            let squatted = economy::denied_near(state, city);
            let before =
                economy::city_totals_squatted(state, city, city.producing, None, &squatted);
            let after =
                economy::city_totals_squatted(state, city, city.producing, Some(tech), &squatted);
            (before != after).then(|| economy::YieldSource {
                label: city.name.clone(),
                yields: after - before,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechEffects {
    pub upgrades: Vec<UpgradeChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeChange {
    pub unit: UnitId,
    pub from: UnitKind,
    pub to: UnitKind,
    pub before: i32,
    pub after: Breakdown,
    /// Own territory and upgrade technology today. Treasury is quoted separately.
    pub eligible: bool,
}

/// What researching `tech` would change that the item list cannot show: Military Engineering
/// halves upgrade Gold, so each of the player's upgradable units shows its new price.
pub(crate) fn tech_effects_for(state: &GameState, player: PlayerId, tech: TechId) -> TechEffects {
    let mut effects = TechEffects::default();
    if state.player(player).techs.contains(&tech) {
        return effects;
    }
    if tech == TechId::MilitaryEngineering {
        effects.upgrades = state
            .units_of(player)
            .filter_map(|unit| {
                let to = data::unit_stats(unit.kind).upgrades_to?;
                Some(UpgradeChange {
                    unit: unit.id,
                    from: unit.kind,
                    to,
                    before: upgrade_price(state, player, unit.kind, false)?.total,
                    after: upgrade_price(state, player, unit.kind, true)?,
                    eligible: data::unit_stats(to)
                        .requires
                        .is_none_or(|t| state.player(player).techs.contains(&t))
                        && state
                            .map
                            .get(unit.pos)
                            .is_some_and(|t| t.owner == Some(player)),
                })
            })
            .collect();
    }
    effects
}

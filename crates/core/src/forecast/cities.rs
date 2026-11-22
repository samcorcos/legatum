//! City/economy forecasts with current queues and positions held fixed. Pending orders,
//! movement, combat and other empires' production are deliberately not predicted.

use crate::data;
use crate::economy::{self, PlayerIncome};
use crate::hex::Hex;
use crate::rules::cities;
use crate::state::GameState;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProductionStall {
    Population { required: i32, available: i32 },
    SpawnBlocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisbandForecast {
    /// None for a unit completing next turn: its future id is not yet known.
    pub unit: Option<UnitId>,
    pub kind: UnitKind,
    pub pos: Hex,
    /// The producing city, only for a unit completing next turn.
    pub city: Option<CityId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankruptcyForecast {
    /// Treasury plus city income minus upkeep, before the bankruptcy reset to zero.
    pub gold: Breakdown,
    pub disband: Option<DisbandForecast>,
}

pub(crate) struct CityForecasts {
    pub bankruptcy: Option<BankruptcyForecast>,
    pub stalls: BTreeMap<CityId, ProductionStall>,
    /// Current builds priced after earlier cities' projected completions, in resolution order.
    pub production_costs: BTreeMap<CityId, i32>,
    /// Upkeep-paying land units after this turn's completions (used to preview a new build).
    pub military_units: i32,
}

/// Forecast only the viewer's city completions. Neighbors of owned cities are visible, so
/// spawn checks do not reveal hidden units. Do not simulate other players' private queues.
pub(crate) fn forecast_cities(
    state: &GameState,
    me: PlayerId,
    income: &PlayerIncome,
) -> CityForecasts {
    // Most views need no copy. Reuse the actual growth/production rules on a private copy
    // only when a unit has enough production to complete during the next city phase.
    let mut projected = Cow::Borrowed(state);
    let mut stalls = BTreeMap::new();
    let mut production_costs = BTreeMap::new();
    let mut completions = BTreeMap::new();
    for city in state.cities_of(me) {
        let Some(item) = city.producing else {
            continue;
        };
        let cost = economy::city_production_cost(&projected, city, item);
        production_costs.insert(city.id, cost);
        let ProductionItem::Unit { unit } = item else {
            continue;
        };
        if city.resistance > 0 || !economy::can_produce(&projected, city, item) {
            continue;
        }
        let yields = economy::city_totals(state, city);
        if city.production_stored + yields.production < cost {
            continue;
        }
        let sim = projected.to_mut();
        let mut events = Vec::new();
        cities::grow(sim, city.id, yields.food, &mut events);
        let grown = &sim.cities[&city.id];
        if unit == UnitKind::Settler && grown.population < data::SETTLER_MIN_POP {
            stalls.insert(
                city.id,
                ProductionStall::Population {
                    required: data::SETTLER_MIN_POP,
                    available: grown.population,
                },
            );
        } else if cities::spawn_tile(sim, me, city.pos, unit).is_none() {
            stalls.insert(city.id, ProductionStall::SpawnBlocked);
        } else {
            cities::produce(sim, city.id, yields.production, &mut events);
            completions.insert(UnitId(sim.next_unit_id), city.id);
        }
    }

    let military_units = economy::upkeep_unit_count(&projected, me) as i32;

    let mut gold = Breakdown::default();
    gold.add("Treasury", state.player(me).gold);
    for (label, value) in &income.gold.parts {
        gold.add(label.clone(), *value);
    }
    gold.add(
        "Upkeep of completing units",
        income.upkeep - economy::upkeep(&projected, me),
    );
    let bankruptcy = (state.player(me).alive && gold.total < 0).then(|| BankruptcyForecast {
        gold,
        disband: cities::bankruptcy_victim(&projected, me).map(|id| {
            let u = &projected.units[&id];
            let city = completions.get(&id).copied();
            DisbandForecast {
                unit: city.is_none().then_some(id),
                kind: u.kind,
                pos: u.pos,
                city,
            }
        }),
    });
    CityForecasts {
        bankruptcy,
        stalls,
        production_costs,
        military_units,
    }
}

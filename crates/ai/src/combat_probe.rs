//! Read-only probe of the AI's city-assault forecasts for the combat lab
//! (`crates/sim/examples/combat_lab.rs`). It changes no decision: it asks the same planner
//! the same questions Hard asks, and reports the answers. Offline studies only.

use super::*;
use crate::ScriptedBrain;
use legatum_core::{CityId, PlayerView};

/// What Hard thinks of taking one visible enemy city from the current view.
#[derive(Clone, Debug, Default)]
pub struct AssaultProbe {
    /// The city is currently in sight (otherwise the forecast assumes one fortified Soldier).
    pub city_visible: bool,
    /// Units the strike forecast counts (healthy, able to reach an assault tile in time).
    pub force: usize,
    /// `forecast_siege`: (turns to capture counting the approach, expected Production lost),
    /// or None when the force cannot take the city within the window.
    pub siege: Option<(i32, i32)>,
    /// `opportunity_strike` names this city: the forecast pays for the war and the losses.
    pub strike: bool,
    /// Orders `coordinate_city_assaults_at` issues for this city now (0 = the assault
    /// check declined or nobody was in range).
    pub assault_orders: usize,
}

impl ScriptedBrain {
    /// The AI's forecast and assault-commit answers for `city` from `view`.
    #[doc(hidden)]
    pub fn probe_city_assault(&self, view: &PlayerView, city: CityId) -> Option<AssaultProbe> {
        let target = view
            .cities
            .iter()
            .find(|c| c.id == city && view.at_war(view.me, c.owner))?;
        let mut p = Planner::new(view, self.knobs, self.faction, self.seed);
        p.difficulty = self.difficulty;
        p.decisions = self.decisions.for_faction(self.faction);
        let _ = p.conquest_city.set(Some(city));
        let (force_len, siege) = {
            let force = p.strike_force(view, target);
            let siege = p
                .forecast_siege(view, target, &force)
                .map(|s| (s.turns, s.losses));
            (force.len(), siege)
        };
        let strike = p.opportunity_strike().is_some_and(|s| s.city == city);
        let army = p.ctx.own_units.clone();
        p.coordinate_city_assaults_at(&army, Some(target.pos));
        Some(AssaultProbe {
            city_visible: target.visible,
            force: force_len,
            siege,
            strike,
            assault_orders: p.orders.len(),
        })
    }
}

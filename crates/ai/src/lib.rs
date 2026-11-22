//! AI brains. A brain reads only a PlayerView and returns orders, so it can never cheat.
//!
//! Brains are deterministic given (view, seed): integer math only, ordered iteration only, and
//! randomness only through `Rng::derive(seed, [turn, purpose::AI, player, ...])`. Scripted and
//! opening brains retain only view-derived rival observations between turns.

/// The AI's decision numbers (keep values, loss ratios, military budgets) were tuned with unit
/// and building costs in half Production points (a Spearman was 40, a Settler 75). The tables now
/// hold the real prices, so a listed cost is halved where it feeds one of those numbers.
pub(crate) const fn tuned(listed_cost: i32) -> i32 {
    listed_cost / 2
}

mod adaptation;
mod air;
mod analysis;
mod army_commit;
mod barbarians;
#[doc(hidden)]
pub mod buy_trace;
mod campaign;
mod capitals;
mod celts_playbook;
mod civilians;
mod company;
mod composition;
mod decisive_buy;
mod exploration;
mod funding;
mod goal_actions;
mod goal_forge_heir;
mod goal_hall;
mod goal_sakoku;
mod goal_sparta_wild;
mod gold_hold;
mod intel;
mod objective_pursuit;
pub mod opponents;
pub use diplomacy::{
    ALLIANCE_ACQUAINTANCE_TURNS, PeaceDiagnosis, TREATY_HELD_PENALTY, TreatyBudget, TreatyCost,
    alliance_acquainted, alliance_opportunity, alliance_security, nearest_known, offer_slot_open,
    treaty_cost, treaty_reach,
};
pub use exploration::{EXPLORE_MIN_HP, explore_step};
mod conquest;
mod contest;
mod decision_trace;
pub mod designer;
mod development;
mod difficulty;
mod diplomacy;
mod door;
mod dual;
mod focus;
mod hard_defense;
pub mod hard_v2;
pub mod hold;
mod hollow_march;
mod investment;
mod knobs;
pub mod learned_macro;
mod levy;
mod military;
mod modernization;
mod movement;
pub mod openings;
pub mod operation;
mod opportunity;
mod params;
pub mod plan;
mod production;
pub use production::{city_automation_choice, city_automation_choice_with_pipeline};
mod proxy;
mod proxy_v4;
mod proxy_v5;
mod proxy_v6;
mod proxy_v7;
mod proxy_v8_mongols;
mod proxy_v8_sparta;
mod purchase_defense;
pub use purchase_defense::city_siege_times;
mod random;
mod raze;
mod research;
mod runaway;
mod scripted;
pub mod search;
mod short_objective;
mod siege_buy;
mod site_strategy;
mod strategy;
mod supply;
mod supply_value;
mod tactics;
pub mod threat;
pub mod training;
pub mod tuned_openings;
pub mod value_net;
mod victory;
mod war_diagnosis;
pub mod why;
pub mod winchance;

/// The same target choice used by the scripted army's ranged-fire pass, for a human
/// garrison that has opted into automatic fire. Reads only that player's visible view.
pub fn automatic_garrison_target(
    view: &legatum_core::PlayerView,
    unit: legatum_core::UnitId,
) -> Option<legatum_core::Hex> {
    let u = view
        .units
        .iter()
        .find(|u| u.id == unit && u.owner == view.me)?;
    if u.ranged <= 0
        || !view
            .cities
            .iter()
            .any(|c| c.owner == view.me && c.pos == u.pos)
    {
        return None;
    }
    let faction = view.faction_of(view.me);
    let planner = scripted::Planner::new(view, knobs::Knobs::for_faction(faction), faction, 0);
    planner.best_shot_automatic(u, &std::collections::BTreeMap::new())
}

#[cfg(test)]
mod army_commit_tests;
#[cfg(test)]
mod dead_war_tests;
#[cfg(test)]
mod door_tests;
#[cfg(test)]
mod dual_tests;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod hard_defense_tests;
#[cfg(test)]
mod hollow_tests;
#[cfg(test)]
mod intel_tests;
#[cfg(test)]
mod levy_tests;
#[cfg(test)]
mod plan_tests;
#[cfg(test)]
mod reaim_tests;
#[cfg(test)]
mod tests;

use legatum_core::{Order, PlayerView};

pub use campaign::{
    Campaign, CampaignEnd, CampaignEvent, CampaignMemory, favorable_visible_capture,
};
pub use decision_trace::DecisionOption;
pub use knobs::Knobs;
pub use military::AssaultProbe;
pub use params::{AiParams, DecisionOverrides, DecisionParams, KnobValues, PARAMS_SCHEMA};
pub use production::{ProductionTrace, SettlerGates};
pub use proxy::{PROXY_V2_VERSION, ProxyV2Brain};
pub use proxy_v4::{PROXY_VERSION, ProxyBrain};
pub use proxy_v5::{HumanPlaybookBrain, PROXY_V5_VERSION};
pub use proxy_v6::{HumanPlaybookV6Brain, PROXY_V6_VERSION};
pub use proxy_v7::{HumanPlaybookV7Brain, PROXY_V7_VERSION};
pub use proxy_v8_mongols::{HumanMongolsBrain, PROXY_V8_MONGOLS_VERSION};
pub use proxy_v8_sparta::{HumanSpartaBrain, PROXY_V8_SPARTA_VERSION};
pub use random::RandomBrain;
pub use scripted::{OrderStage, OrderTrace, RejectedOrder, ScriptedBrain};
pub use strategy::Strategy;

/// View-derived opponent observations and strategic adjustments retained in session saves.
/// This contains no hidden game state and does not alter core state or replay hashes.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OpponentMemory {
    pub assessments: opponents::OpponentAssessments,
    pub(crate) adaptation: adaptation::Adaptation,
    pub switches: Vec<opponents::StrategySwitch>,
    /// Threat-driven decisions (absent in older saves).
    #[serde(default)]
    pub threat_log: Vec<threat::ThreatEvent>,
    #[serde(default)]
    pub campaigns: campaign::CampaignMemory,
}

pub trait Brain: Send {
    /// None for brains that do not maintain the opponent-assessment layer.
    fn opponent_memory(&self) -> Option<OpponentMemory> {
        None
    }
    fn restore_opponent_memory(&mut self, _memory: OpponentMemory) {}

    fn strategy_switches(&self) -> &[opponents::StrategySwitch] {
        &[]
    }
    /// Threat-driven decisions (top-threat marks, walls, treaties, second fronts).
    fn threat_log(&self) -> &[threat::ThreatEvent] {
        &[]
    }
    fn campaign_events(&self) -> &[campaign::CampaignEvent] {
        &[]
    }
    /// Goal-driven plan switches and blocked purchases (Hard scripted seats).
    fn plan_events(&self) -> &[plan::PlanEvent] {
        &[]
    }
    /// Every purchase the plan layer blocked, including those not kept as events.
    /// Diagnostics (`war_trace`): why the seat does or doesn't march on an enemy city.
    #[doc(hidden)]
    fn war_diagnosis(&self, _view: &PlayerView) -> Vec<String> {
        Vec::new()
    }
    fn blocked_expenses(&self) -> u32 {
        0
    }
    /// Hard+Search rollouts, one per searched plan review point.
    fn search_events(&self) -> &[search::SearchEvent] {
        &[]
    }
    /// Diagnostic only: whether this brain asks for the true state (omniscient search, a
    /// flagged upper bound that reads hidden state). Never true for a shipped brain.
    fn wants_true_state(&self) -> bool {
        false
    }
    fn observe_true_state(&mut self, _state: std::sync::Arc<legatum_core::GameState>) {}
    fn opening_handoff(&self) -> Option<openings::OpeningHandoff> {
        None
    }
    fn primitive_coalition_march(&self) -> Option<&openings::CoalitionMarch> {
        None
    }
    fn opening_phase_switches(&self) -> &[openings::PhaseSwitch] {
        &[]
    }
    /// The opening this brain plays, if it runs an opening policy (Hard v2, league seats).
    fn opening(&self) -> Option<openings::Opening> {
        None
    }
    /// This brain's decisions for one turn.
    fn plan(&mut self, view: &PlayerView) -> Vec<Order>;

    /// Rollout policies only: this seat submits no orders on `turn`, so the caller may skip
    /// building its view (`search::Lazy`). Every real brain plays every turn.
    fn rests(&self, _turn: u32) -> bool {
        false
    }

    /// Add aircraft actions after ground and goal orders are known. Policy wrappers forward
    /// this hook to their underlying scripted brain; calling `plan` alone omits aircraft.
    fn add_air_orders(&self, _view: &PlayerView, _orders: &mut Vec<Order>) {}

    /// Every brain sees only production it has strategic supply for and never starts more
    /// supply users than are spare, so its orders stay valid.
    fn orders(&mut self, view: &PlayerView) -> Vec<Order> {
        let view = supply::affordable(view);
        let mut orders = self.plan(&view);
        goal_actions::with_goal_actions(
            &view,
            &mut orders,
            DecisionParams::default().goal_open_end > 0,
            &DecisionParams::default(),
        );
        self.add_air_orders(&view, &mut orders);
        let mut orders = supply::within_budget(&view, orders);
        drop_stays_in_allied_cities(&view, &mut orders);
        drop_leaving_orders_of_locked_units(&view, &mut orders);
        drop_purchases_in_frozen_cities(&view, &mut orders);
        orders
    }

    /// The same live call with optional observer data. Only scripted Hard overrides this.
    fn orders_with_decision_trace(
        &mut self,
        view: &PlayerView,
    ) -> (Vec<Order>, Option<OrderTrace>) {
        (self.orders(view), None)
    }
}

/// A unit standing in an ally's city must move out: a Skip or Sleep there is refused, so it is
/// left without an order (free Alliances put units in allied cities more often). A move that
/// ends in an ally's city is refused too (it may be crossed, never stopped in), so it is dropped.
pub(crate) fn drop_stays_in_allied_cities(view: &PlayerView, orders: &mut Vec<Order>) {
    let ally_city_at = |at: legatum_core::Hex| {
        view.cities.iter().any(|c| {
            c.pos == at && !view.same_team(view.me, c.owner) && view.allied(view.me, c.owner)
        })
    };
    let in_ally_city = |id: legatum_core::UnitId| {
        view.units.iter().any(|u| {
            u.id == id
                && view.cities.iter().any(|c| {
                    c.pos == u.pos
                        && !view.same_team(view.me, c.owner)
                        && view.allied(view.me, c.owner)
                })
        })
    };
    orders.retain(|o| match o {
        Order::SkipUnit { unit } | Order::SleepUnit { unit, .. } => !in_ally_city(*unit),
        Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } => !ally_city_at(*to),
        _ => true,
    });
}

/// Occupation lock: a locked unit cannot leave its city, and validation refuses
/// the order that would take it out (or disband it), so the AI never issues one.
pub(crate) fn drop_leaving_orders_of_locked_units(view: &PlayerView, orders: &mut Vec<Order>) {
    let locked_at = |id: legatum_core::UnitId| {
        view.units
            .iter()
            .find(|u| u.id == id && u.owner == view.me && u.occupation_lock.is_some())
            .map(|u| u.pos)
    };
    orders.retain(|o| match o {
        Order::MoveUnit { unit, to, .. } | Order::MoveUnitPath { unit, to, .. } => {
            locked_at(*unit).is_none_or(|at| at == *to)
        }
        Order::SetAutomation {
            unit,
            automation: Some(_),
            ..
        }
        | Order::DisbandUnit { unit } => locked_at(*unit).is_none(),
        _ => true,
    });
}

/// Occupation lock: a captured city at war with its victim buys and hires nothing,
/// and validation refuses it, so the AI never issues one. Its own cities carry the civ in
/// `CityView::occupation_lock` exactly while they are frozen.
pub(crate) fn drop_purchases_in_frozen_cities(view: &PlayerView, orders: &mut Vec<Order>) {
    orders.retain(|o| match o {
        Order::BuyItem { city, .. } | Order::HireCompany { city, .. } => !view
            .cities
            .iter()
            .any(|c| c.id == *city && c.owner == view.me && c.occupation_lock.is_some()),
        _ => true,
    });
}

#[cfg(test)]
mod endgame_tests;
#[cfg(test)]
mod expansion_tests;
#[cfg(test)]
mod faction_tests;
#[cfg(test)]
mod india_settler_tests;
#[cfg(test)]
mod spend_hoard_tests;

#[cfg(test)]
mod occupation_lock_tests {
    use super::*;
    use legatum_core::{Hex, UnitId, UnitKind};

    #[test]
    fn locked_units_get_no_leaving_orders() {
        let home = crate::fixture::off(5, 5);
        let mut view = crate::fixture::ViewBuilder::new(12, 12)
            .city(1, crate::fixture::ME, home, 3)
            .unit(10, crate::fixture::ME, UnitKind::Spearman, home)
            .unit(
                11,
                crate::fixture::ME,
                UnitKind::Spearman,
                crate::fixture::off(4, 5),
            )
            .build();
        let out = crate::fixture::off(6, 5);
        let orders = |to: Hex| {
            vec![
                Order::MoveUnit {
                    unit: UnitId(10),
                    to,
                },
                Order::MoveUnit {
                    unit: UnitId(10),
                    to: home,
                },
                Order::DisbandUnit { unit: UnitId(10) },
                Order::MoveUnit {
                    unit: UnitId(11),
                    to,
                },
                Order::SkipUnit { unit: UnitId(10) },
            ]
        };
        let mut all = orders(out);
        drop_leaving_orders_of_locked_units(&view, &mut all);
        assert_eq!(all, orders(out), "no locked unit: nothing is dropped");
        view.units[0].occupation_lock = Some(legatum_core::PlayerId(1));
        let mut kept = orders(out);
        drop_leaving_orders_of_locked_units(&view, &mut kept);
        let buy = || Order::BuyItem {
            city: legatum_core::CityId(1),
            item: legatum_core::ProductionItem::Unit {
                unit: UnitKind::Spearman,
            },
        };
        let mut bought = vec![buy()];
        drop_purchases_in_frozen_cities(&view, &mut bought);
        assert_eq!(
            bought,
            vec![buy()],
            "no record on the city: nothing dropped"
        );
        view.cities[0].occupation_lock = Some(legatum_core::PlayerId(1));
        drop_purchases_in_frozen_cities(&view, &mut bought);
        assert!(bought.is_empty());
        assert_eq!(
            kept,
            vec![
                Order::MoveUnit {
                    unit: UnitId(10),
                    to: home
                },
                Order::MoveUnit {
                    unit: UnitId(11),
                    to: out
                },
                Order::SkipUnit { unit: UnitId(10) },
            ]
        );
    }
}

#[cfg(test)]
mod automatic_garrison_tests {
    use super::*;
    use legatum_core::{PlayerId, UnitId, UnitKind};

    #[test]
    fn garrison_uses_the_armys_visible_ranged_target_choice() {
        let home = crate::fixture::off(5, 5);
        let enemy = crate::fixture::off(6, 5);
        let view = crate::fixture::ViewBuilder::new(12, 12)
            .city(1, crate::fixture::ME, home, 3)
            .unit(10, crate::fixture::ME, UnitKind::Archer, home)
            .unit(20, crate::fixture::ENEMY, UnitKind::Spearman, enemy)
            .build();
        assert_eq!(automatic_garrison_target(&view, UnitId(10)), Some(enemy));
        let mut peaceful = view.clone();
        peaceful.diplomacy.legacy = false;
        assert_eq!(automatic_garrison_target(&peaceful, UnitId(10)), None);
    }

    /// Barbarians never attack the Celts, so a Celtic garrison's standing auto-fire passes them
    /// by (`PlayerView::is_threat`); anyone else's garrison still shoots them, and the Celts can
    /// still shoot one by an explicit order (`Planner::best_shot`).
    #[test]
    fn auto_fire_skips_barbarians_for_the_celts_only() {
        let home = crate::fixture::off(5, 5);
        let camp_guard = crate::fixture::off(6, 5);
        for (faction, fires) in [
            (legatum_core::Faction::Celts, false),
            (legatum_core::Faction::Switzerland, true),
        ] {
            let view = crate::fixture::ViewBuilder::new(12, 12)
                .faction(faction)
                .city(1, crate::fixture::ME, home, 3)
                .unit(10, crate::fixture::ME, UnitKind::Archer, home)
                .unit(20, PlayerId::BARBARIAN, UnitKind::Spearman, camp_guard)
                .build();
            assert_eq!(
                automatic_garrison_target(&view, UnitId(10)),
                fires.then_some(camp_guard),
                "{faction:?}"
            );
            let planner =
                scripted::Planner::new(&view, knobs::Knobs::for_faction(faction), faction, 0);
            assert_eq!(
                planner.best_shot(&view.units[0], &std::collections::BTreeMap::new()),
                Some(camp_guard),
                "{faction:?}: an explicit shot at a barbarian is still chosen"
            );
        }
    }

    /// With a barbarian and a rival both in range, a Celtic auto-fire takes the rival.
    #[test]
    fn celtic_auto_fire_still_takes_a_rival_beside_a_barbarian() {
        let home = crate::fixture::off(5, 5);
        let barbarian = crate::fixture::off(6, 5);
        let rival = crate::fixture::off(4, 5);
        let view = crate::fixture::ViewBuilder::new(12, 12)
            .faction(legatum_core::Faction::Celts)
            .city(1, crate::fixture::ME, home, 3)
            .unit(10, crate::fixture::ME, UnitKind::Archer, home)
            .unit(20, PlayerId::BARBARIAN, UnitKind::Spearman, barbarian)
            .unit(21, crate::fixture::ENEMY, UnitKind::Spearman, rival)
            .build();
        assert_eq!(automatic_garrison_target(&view, UnitId(10)), Some(rival));
    }

    #[test]
    fn walled_garrison_auto_fires_past_forest() {
        let home = crate::fixture::off(5, 5);
        let middle = crate::fixture::off(6, 5);
        let enemy = crate::fixture::off(7, 5);
        let view = crate::fixture::ViewBuilder::new(12, 12)
            .city(1, crate::fixture::ME, home, 3)
            .terrain(middle, legatum_core::Terrain::Forest)
            .unit(10, crate::fixture::ME, UnitKind::Archer, home)
            .unit(20, crate::fixture::ENEMY, UnitKind::Spearman, enemy)
            .build();
        assert_eq!(automatic_garrison_target(&view, UnitId(10)), None);
        let mut walled = view;
        walled.cities[0].walls = true;
        assert_eq!(automatic_garrison_target(&walled, UnitId(10)), Some(enemy));
    }
}

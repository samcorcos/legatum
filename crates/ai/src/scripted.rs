//! The scripted AI: knob-weighted heuristics over a PlayerView.
//!
//! One `Planner` is built per `orders()` call. It plans research, then cities (production, focus,
//! buying), then civilians (settlers, scouts), then the army. Each step records which units have
//! an order so no unit ever gets two, and which tiles units are headed for so they don't bounce.

use crate::Brain;
use crate::analysis::Ctx;
use crate::difficulty;
use crate::knobs::Knobs;
use legatum_core::rng::{Rng, purpose};
use legatum_core::state::Map;
use legatum_core::view::UnitView;
use legatum_core::{AiDifficulty, Faction, Hex, Order, PlayerId, PlayerView, UnitId};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The scripted AI. Knobs are chosen per faction; decisions are deterministic given the view
/// and the seed plus prior observations. All evidence comes from PlayerView; session saves
/// preserve the observation memory.
pub struct ScriptedBrain {
    pub player: PlayerId,
    pub faction: Faction,
    pub seed: u64,
    pub knobs: Knobs,
    pub decisions: crate::DecisionParams,
    /// The faction's personality before difficulty scaling (built-in or tuned params).
    pub base: Knobs,
    pub difficulty: AiDifficulty,
    pub strategy: Option<crate::Strategy>,
    pub opponents: crate::opponents::OpponentAssessments,
    pub(crate) adaptation: crate::adaptation::Adaptation,
    pub(crate) strategy_switches: Vec<crate::opponents::StrategySwitch>,
    pub(crate) threat_log: Vec<crate::threat::ThreatEvent>,
    pub(crate) campaigns: crate::campaign::CampaignMemory,
    /// Replanning the same observation starts from the same campaign state. An
    /// actual turn advance commits the preceding decision to persistent memory.
    planning_campaigns: Option<(u32, crate::campaign::CampaignMemory)>,
    /// Hard routes strategy through its goal-driven plan (`plan.rs`). On by default; off only
    /// to measure the plan layer against the scripted rules it replaces.
    pub plan_layer: bool,
    /// Hard+Search: plan review points are decided by rollouts (`search`). None, the default,
    /// is plain Hard.
    pub search: Option<crate::search::SearchConfig>,
    /// Search rollouts have no search hook but must play the same Sparta war policy as the
    /// live searched seat; plain Hard keeps its proven plan and tactical thresholds.
    sparta_search_policy: bool,
    pub(crate) search_log: Vec<crate::search::SearchEvent>,
    /// Rollout-only policy level (0 = full scripted Hard; see `SearchConfig::rollout_policy`).
    /// Never set on a brain that plays the real game.
    pub(crate) fast: u8,
    /// Diagnostic only (`SearchConfig::omniscient`): the true state, set by the simulator.
    pub(crate) oracle: Option<std::sync::Arc<legatum_core::GameState>>,
    /// Map-generation samples for unexplored terrain, one per determinization sample.
    prior_maps:
        std::sync::Mutex<std::collections::BTreeMap<u64, std::sync::Arc<crate::search::Prior>>>,
}

#[cfg(test)]
mod sparta_search_policy_tests {
    use super::*;

    #[test]
    fn plain_hard_keeps_prior_war_policy_but_search_rollouts_keep_the_new_one() {
        let mut brain =
            ScriptedBrain::with_difficulty(Faction::Sparta, 1, PlayerId(0), AiDifficulty::Hard);
        brain.decisions.sparta_conquest_bundle = 1;
        brain.decisions.sparta_force_campaign = 1;
        brain.decisions.sparta_march_min_hp = 20;
        brain.decisions.sparta_capture_heal_forecast = 1;
        brain.decisions.sparta_search_only = 1;
        let hard = brain.planner_decisions();
        assert_eq!(hard.sparta_force_campaign, 0);
        assert_eq!(hard.sparta_march_min_hp, 30);
        assert_eq!(hard.sparta_capture_heal_forecast, 0);

        let searched = brain.with_search(crate::search::SearchConfig::default());
        for decisions in [
            searched.planner_decisions(),
            searched.rollout_copy().planner_decisions(),
        ] {
            assert_eq!(decisions.sparta_force_campaign, 1);
            assert_eq!(decisions.sparta_march_min_hp, 20);
            assert_eq!(decisions.sparta_capture_heal_forecast, 1);
        }
    }

    #[test]
    fn plain_hard_plays_the_sparta_war_policy_by_default() {
        // Playtest 8.8.1 and the war scenarios: plain Hard Sparta never declared with five
        // Spearmen against a one-city Germany (Sparta's First Blood 0/64; 64/64 with the
        // policy). The default now gives plain Hard the same war genes as Hard + Search.
        let brain =
            ScriptedBrain::with_difficulty(Faction::Sparta, 1, PlayerId(0), AiDifficulty::Hard);
        let hard = brain.planner_decisions();
        assert_eq!(hard.sparta_force_campaign, 1);
        assert_eq!(hard.sparta_conquest_bundle, 1);
        assert_eq!(hard.sparta_march_min_hp, 20);
        assert_eq!(hard.sparta_capture_heal_forecast, 1);
    }
}

#[cfg(test)]
mod swiss_playbook_difficulty_tests {
    use super::*;

    #[test]
    fn expansion_level_three_changes_hard_swiss_only() {
        let hard = ScriptedBrain::with_difficulty(
            Faction::Switzerland,
            1,
            PlayerId(0),
            AiDifficulty::Hard,
        )
        .planner_decisions();
        let normal = ScriptedBrain::with_difficulty(
            Faction::Switzerland,
            1,
            PlayerId(0),
            AiDifficulty::Normal,
        )
        .planner_decisions();
        assert_eq!(hard.swiss_playbook, 3);
        assert_eq!(hard.settler_chain_priority, 1500);
        assert_eq!(hard.settler_min_site_score, 40);
        assert_eq!(normal.swiss_playbook, 2);
        assert_eq!(normal.settler_chain_priority, 500);
        assert_eq!(normal.settler_min_site_score, 60);
    }
}

impl ScriptedBrain {
    /// Run Hard's existing tactical planner on a small arena view, without asking its
    /// strategic layer to fund a campaign or choose production. Offline studies only.
    /// The view remains the sole source of information.
    pub fn tactical_orders(
        &self,
        view: &PlayerView,
        target_city: Option<legatum_core::CityId>,
    ) -> Vec<Order> {
        let mut p = Planner::new(view, self.knobs, self.faction, self.seed);
        p.difficulty = self.difficulty;
        p.decisions = self.planner_decisions();
        if let Some(city) = target_city.filter(|id| {
            view.cities
                .iter()
                .any(|c| c.id == *id && view.at_war(view.me, c.owner))
        }) {
            let _ = p.conquest_city.set(Some(city));
        }
        p.plan_siege_buys();
        p.plan_military();
        p.dedupe_first_steps();
        p.supported_defence_orders();
        self.finish(view, p)
    }

    /// Faction knobs scaled for `difficulty` (see `difficulty.rs`).
    pub fn with_difficulty(
        faction: Faction,
        seed: u64,
        player: PlayerId,
        difficulty: AiDifficulty,
    ) -> Self {
        let knobs = difficulty::scale_knobs(Knobs::for_faction(faction), difficulty);
        Self {
            difficulty,
            ..Self::with_knobs(faction, seed, player, knobs)
        }
    }

    /// Tuned decision parameters (`--ai-params`) for this faction and difficulty.
    pub fn with_params(
        faction: Faction,
        seed: u64,
        player: PlayerId,
        difficulty: AiDifficulty,
        params: &crate::AiParams,
    ) -> Self {
        Self {
            difficulty,
            base: params.base(faction),
            decisions: params.decisions(faction),
            ..Self::with_knobs(faction, seed, player, params.knobs(faction, difficulty))
        }
    }

    pub fn new(faction: Faction, seed: u64, player: PlayerId) -> Self {
        Self::with_knobs(faction, seed, player, Knobs::for_faction(faction))
    }

    pub fn with_knobs(faction: Faction, seed: u64, player: PlayerId, knobs: Knobs) -> Self {
        Self {
            player,
            faction,
            seed,
            knobs: knobs.clamped(),
            decisions: crate::DecisionParams::builtin_for(faction),
            base: Knobs::for_faction(faction),
            difficulty: AiDifficulty::Normal,
            strategy: None,
            opponents: Default::default(),
            adaptation: Default::default(),
            strategy_switches: Vec::new(),
            threat_log: Vec::new(),
            campaigns: Default::default(),
            planning_campaigns: None,
            plan_layer: true,
            search: None,
            sparta_search_policy: false,
            search_log: Vec::new(),
            fast: 0,
            oracle: None,
            prior_maps: Default::default(),
        }
    }

    /// Hard+Search: this brain decides its plan review points by rollouts.
    pub fn with_search(mut self, config: crate::search::SearchConfig) -> Self {
        self.search = Some(config);
        self.sparta_search_policy = true;
        self
    }

    /// A copy for one rollout: the same knobs and memory, no search, no logs kept.
    /// Benchmarks only: this brain plays rollout policy level `level` (never a real seat).
    #[doc(hidden)]
    pub fn with_rollout_policy(mut self, level: u8) -> Self {
        self.fast = level;
        self
    }

    /// A rollout copy playing rollout policy level `fast` (`SearchConfig::rollout_policy`).
    pub(crate) fn rollout_copy_with(&self, fast: u8) -> Self {
        let mut b = self.rollout_copy();
        b.fast = fast;
        b
    }

    #[doc(hidden)]
    pub fn rollout_copy(&self) -> Self {
        Self {
            player: self.player,
            faction: self.faction,
            seed: self.seed,
            knobs: self.knobs,
            decisions: self.decisions,
            base: self.base,
            difficulty: self.difficulty,
            strategy: self.strategy,
            opponents: self.opponents.clone(),
            adaptation: self.adaptation,
            strategy_switches: Vec::new(),
            threat_log: Vec::new(),
            campaigns: self.campaigns.clone(),
            planning_campaigns: None,
            plan_layer: self.plan_layer,
            // A rollout copy never searches, but keeps pricing its plans the same way.
            search: self
                .search
                .filter(|c| c.fitted_plan)
                .map(|c| crate::search::SearchConfig {
                    candidates: 0,
                    openings: false,
                    postures: false,
                    triggers: false,
                    stances: false,
                    tactics: false,
                    ..c
                }),
            sparta_search_policy: self.sparta_search_policy,
            search_log: Vec::new(),
            fast: 0,
            oracle: None,
            prior_maps: Default::default(),
        }
    }

    /// Commit an operation (gene `operations`, `operation.rs`; offline only). The planning
    /// memo of this turn is dropped so the next pass sees it.
    #[doc(hidden)]
    pub fn set_operation(&mut self, operation: crate::operation::Operation) {
        self.campaigns.operation = Some(operation);
        self.planning_campaigns = None;
    }

    /// Offline labels: keep this seat's observations and active plan, but review it now.
    /// No live seat calls this; the copied memory is changed only inside a diagnostic search.
    #[doc(hidden)]
    pub fn offline_review_copy(&self, turn: u32) -> Self {
        let mut copy = self.rollout_copy();
        if let Some(plan) = copy.campaigns.plan.active.as_mut() {
            plan.review_turn = turn;
        }
        copy.campaigns.plan.searched.clear();
        copy
    }

    /// The map-generation sample `index` for unexplored terrain (fixed for the whole game).
    pub(crate) fn prior_map(
        &self,
        view: &PlayerView,
        index: u64,
    ) -> std::sync::Arc<crate::search::Prior> {
        let mut maps = self.prior_maps.lock().expect("prior map cache");
        maps.entry(index)
            .or_insert_with(|| {
                let seed = Rng::derive(self.seed, &[0x5EA4_C403, index]).next_u64();
                std::sync::Arc::new(crate::search::prior_map(view, seed))
            })
            .clone()
    }

    pub fn with_strategy(
        faction: Faction,
        seed: u64,
        player: PlayerId,
        strategy: crate::Strategy,
    ) -> Self {
        Self {
            strategy: Some(strategy),
            ..Self::with_knobs(faction, seed, player, strategy.knobs())
        }
    }
}

impl ScriptedBrain {
    /// One planning pass. Call `assess_opponents` for the view first: the planner reads the
    /// opponent and threat model.
    pub(crate) fn planner<'a>(&'a self, view: &'a PlayerView) -> Planner<'a> {
        self.planner_with(view, false)
    }

    pub(crate) fn planner_decisions(&self) -> crate::DecisionParams {
        let mut decisions = self.decisions;
        // The third Swiss playbook level was measured for Hard alone. Easy and Normal keep
        // the level-two Settler valuation even when Hard Switzerland uses level three.
        if self.faction == Faction::Switzerland
            && self.difficulty != AiDifficulty::Hard
            && decisions.swiss_playbook >= 3
        {
            decisions.swiss_playbook = 2;
        }
        if self.faction == Faction::Sparta
            && decisions.sparta_search_only > 0
            && !self.sparta_search_policy
        {
            decisions.sparta_conquest_bundle = 0;
            decisions.sparta_force_campaign = 0;
            decisions.sparta_strike_min_hp = 50;
            decisions.sparta_expedition_min_hp = 60;
            decisions.sparta_march_min_hp = 30;
            decisions.sparta_capture_heal_forecast = 0;
        }
        decisions.for_faction(self.faction)
    }

    pub(crate) fn planner_with<'a>(
        &'a self,
        view: &'a PlayerView,
        production_trace: bool,
    ) -> Planner<'a> {
        let mut p = self.planner_through_plan(view, production_trace);
        p.pass("plan");
        p.plan_diplomacy();
        p.plan_levy_turnover();
        p.pass("execute_snipe");
        p.execute_snipe();
        p.pass("execute_field_strike");
        p.execute_field_strike();
        p.pass("plan_egress");
        p.plan_egress();
        p.plan_research();
        p.plan_yields();
        p.pass("execute_hold");
        p.execute_hold();
        p.pass("cities");
        p.operation_buys();
        p.plan_siege_buys();
        p.plan_cities();
        p.plan_raze();
        p.plan_company();
        p.pass("plan_settlers");
        p.plan_settlers();
        p.pass("stalk_settlers");
        p.stalk_settlers();
        p.pass("refresh_stale_nominee");
        p.refresh_stale_nominee();
        p.pass("plan_scouts");
        p.plan_scouts();
        p.choose_stance();
        p.plan_military();
        p.tag_buys("after_cities");
        p.pass("dedupe_first_steps");
        p.dedupe_first_steps();
        p.supported_defence_orders();
        p.pass("enforce_levy");
        p.enforce_levy();
        p.enforce_purchase_generation();
        p.pass("siege_deadline");
        p.enforce_siege_deadlines();
        p.record_fund_deliveries();
        p.pass("end");
        p
    }

    /// The planner as diplomacy sees it: set up, the campaign reconciled and the plan chosen.
    fn planner_through_plan<'a>(
        &'a self,
        view: &'a PlayerView,
        production_trace: bool,
    ) -> Planner<'a> {
        let knobs = crate::plan::Posture::apply(self.campaigns.plan.posture, self.adapted_knobs());
        let mut p = Planner::new(view, knobs, self.faction, self.seed);
        p.personality_aggression = self.knobs.aggression;
        p.campaigns = self.campaigns.clone();
        p.opponents = Some(&self.opponents);
        p.difficulty = self.difficulty;
        p.decisions = self.planner_decisions();
        p.strategy = if self.adaptation == crate::adaptation::Adaptation::Expand
            && self.strategy == Some(crate::Strategy::Rush)
        {
            Some(crate::Strategy::Wide)
        } else {
            self.strategy
        };
        if self.search.is_some_and(|c| c.holds) {
            p.campaigns.plan.holds = true;
        }
        if self.search.is_some_and(|c| c.fronts) {
            p.campaigns.plan.fronts = true;
        }
        // A searched stance is this turn's alone: chosen again before the army moves.
        if self.search.is_some_and(|c| c.stances) {
            p.campaigns.plan.stance = None;
        }
        p.fast = self.fast;
        p.search = self
            .search
            .filter(|_| p.difficulty == AiDifficulty::Hard)
            .map(|config| crate::search::SearchHook {
                config,
                brain: self,
                oracle: self.oracle.as_deref().filter(|_| config.omniscient),
            });
        p.plan_routing = self.plan_layer && crate::plan::routes_plans(self.difficulty);
        if production_trace {
            p.production_trace = Some(Default::default());
            p.pass_trace = Some(PassTrace::default());
            p.decision_trace = Some(Default::default());
        }
        p.spare_own_camp_guards();
        p.prepare_hard_defense();
        p.note_city_sightings();
        p.start_second_front();
        p.choose_short_objective();
        p.review_operation();
        p.update_roster();
        p.reconcile_campaign();
        p.choose_plan();
        p.note_operation_took();
        p
    }

    /// Final orders from a planning pass, recording threat-driven decisions in `log`.
    pub(crate) fn finish_logged(
        &self,
        view: &PlayerView,
        p: &mut Planner<'_>,
        log: &mut Vec<crate::threat::ThreatEvent>,
    ) -> Vec<Order> {
        let mut orders = std::mem::take(&mut p.orders);
        if p.decisions.declare_and_strike == 0 {
            // Control arm: preserve the old first-volley delay.
            for order in &mut orders {
                if let Order::RangedAttack { unit, target } = *order
                    && !view
                        .units
                        .iter()
                        .any(|u| u.military && u.pos == target && view.at_war(view.me, u.owner))
                    && !view
                        .cities
                        .iter()
                        .any(|c| c.pos == target && view.at_war(view.me, c.owner))
                {
                    *order = Order::SkipUnit { unit };
                }
            }
        }
        let orders = difficulty::lapse(orders, self.difficulty, self.seed, view, &p.assault_groups);
        p.adopt_started_siege(view, &orders);
        p.note_war_fights(&orders);
        crate::threat::log_decisions(p, &orders, log);
        orders
    }

    pub(crate) fn finish(&self, view: &PlayerView, p: Planner<'_>) -> Vec<Order> {
        difficulty::lapse(
            p.orders,
            self.difficulty,
            self.seed,
            view,
            &p.assault_groups,
        )
    }
}

/// Diagnostics: one turn's orders as each stage of the live path saw them. `proposed` is what the
/// planner emitted; every later stage lists what it added and what it rejected, with the reason;
/// `executed` is exactly what `Brain::orders` returns. Recording never changes a decision.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct OrderTrace {
    #[serde(skip)]
    active: bool,
    pub proposed: Vec<Order>,
    pub stages: Vec<OrderStage>,
    pub executed: Vec<Order>,
    /// What every idle city weighed and chose (a city given production by an earlier planning
    /// step, such as an emergency, a victory or a faction goal, has no entry).
    pub production: Vec<crate::production::ProductionTrace>,
    /// Per unit, the planning passes that gave or changed its order, in order (the last one
    /// set the proposed order); `-pass` marks a pass that dropped the unit's order.
    pub passes: BTreeMap<u32, Vec<&'static str>>,
    /// Production orders by city, including passes before routine scoring.
    pub city_passes: BTreeMap<u32, Vec<&'static str>>,
    #[serde(skip)]
    pub(crate) city_choices: Vec<(Order, &'static str)>,
    pub production_defense: Vec<crate::production::ProductionDefenseTrace>,
    /// Options seen by the live Hard planner, including gates before valuation.
    pub decisions: Vec<crate::DecisionOption>,
}

/// Diagnostics: which planning pass produced each unit order (`Planner::pass`).
#[derive(Clone, Debug, Default)]
pub(crate) struct PassTrace {
    current: &'static str,
    snapshot: Vec<Order>,
    by_unit: BTreeMap<u32, Vec<&'static str>>,
    by_city: BTreeMap<u32, Vec<&'static str>>,
    choices: Vec<(Order, &'static str)>,
}

impl PassTrace {
    /// Credit every unit order new since the last mark to the pass that just ran.
    fn mark(&mut self, orders: &[Order], next: &'static str) {
        let unit = legatum_core::rules::units::order_unit;
        for o in orders {
            if let Order::SetProduction { city, .. } | Order::BuyItem { city, .. } = o
                && !self.snapshot.contains(o)
            {
                self.by_city.entry(city.0).or_default().push(self.current);
                self.choices.push((o.clone(), self.current));
            }
            if let Some(u) = unit(o)
                && !self.snapshot.contains(o)
            {
                let chain = self.by_unit.entry(u.0).or_default();
                if chain.last() != Some(&self.current) {
                    chain.push(self.current);
                }
            }
        }
        for o in &self.snapshot {
            if let Some(u) = unit(o)
                && !orders.contains(o)
                && !orders.iter().any(|x| unit(x) == Some(u))
            {
                self.by_unit.entry(u.0).or_default().push("-");
            }
        }
        self.snapshot = orders.to_vec();
        self.current = next;
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct OrderStage {
    pub name: &'static str,
    pub added: Vec<Order>,
    pub rejected: Vec<RejectedOrder>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RejectedOrder {
    pub order: Order,
    pub reason: &'static str,
}

impl OrderTrace {
    fn recording() -> Self {
        Self {
            active: true,
            ..Self::default()
        }
    }

    /// The orders as they stand before a stage, kept only when recording.
    fn before(&self, orders: &[Order]) -> Vec<Order> {
        if self.active {
            orders.to_vec()
        } else {
            Vec::new()
        }
    }

    /// Record what a stage changed: orders gone from `after` are rejected with `reason`, orders
    /// new in `after` are added (in their order).
    fn stage(
        &mut self,
        name: &'static str,
        reason: &'static str,
        before: Vec<Order>,
        after: &[Order],
    ) {
        if !self.active {
            return;
        }
        let mut unmatched: Vec<Option<&Order>> = after.iter().map(Some).collect();
        let mut rejected = Vec::new();
        for order in before {
            match unmatched
                .iter_mut()
                .find(|slot| slot.is_some_and(|o| *o == order))
            {
                Some(slot) => *slot = None,
                None => rejected.push(RejectedOrder { order, reason }),
            }
        }
        let added = unmatched.into_iter().flatten().cloned().collect();
        self.stages.push(OrderStage {
            name,
            added,
            rejected,
        });
    }
}

#[cfg(test)]
mod order_trace_tests {
    use super::*;

    #[test]
    fn production_passes_record_early_overrides_and_replacements() {
        let mut trace = PassTrace::default();
        let city = legatum_core::CityId(7);
        let unit = Order::SetProduction {
            city,
            item: legatum_core::ProductionItem::Unit {
                unit: legatum_core::UnitKind::Spearman,
            },
        };
        let building = Order::SetProduction {
            city,
            item: legatum_core::ProductionItem::Building {
                building: legatum_core::BuildingKind::Granary,
            },
        };
        trace.mark(&[], "emergency_production");
        trace.mark(std::slice::from_ref(&unit), "routine_production");
        trace.mark(std::slice::from_ref(&unit), "fund_production_runway");
        trace.mark(&[building], "end");
        assert_eq!(
            trace.by_city[&7],
            vec!["emergency_production", "fund_production_runway"]
        );
        assert!(trace.by_unit.is_empty());
    }

    #[test]
    fn a_stage_lists_rejections_with_reasons_and_additions_in_order() {
        let skip = |n| Order::SkipUnit { unit: UnitId(n) };
        let sleep = |n| Order::SleepUnit {
            unit: UnitId(n),
            until_healed: false,
        };
        let mut trace = OrderTrace::recording();
        let before = vec![skip(1), skip(2), skip(2), skip(3)];
        let after = vec![skip(1), sleep(9), skip(2), sleep(8)];
        trace.stage("test", "why", before, &after);
        let stage = &trace.stages[0];
        let rejected: Vec<_> = stage
            .rejected
            .iter()
            .map(|r| (&r.order, r.reason))
            .collect();
        assert_eq!(rejected, [(&skip(2), "why"), (&skip(3), "why")]);
        assert_eq!(stage.added, [sleep(9), sleep(8)]);
        // An untraced run records nothing and copies nothing.
        let mut off = OrderTrace::default();
        assert!(off.before(&after).is_empty());
        off.stage("test", "why", after.clone(), &[]);
        assert!(off.stages.is_empty());
    }
}

impl ScriptedBrain {
    /// A copy that keeps every observation and log, so it plans exactly as `self` would this
    /// turn. Diagnostics only: it lets a `&self` diagnostic run the live path.
    fn diagnostic_copy(&self) -> Self {
        Self {
            player: self.player,
            faction: self.faction,
            seed: self.seed,
            knobs: self.knobs,
            decisions: self.decisions,
            base: self.base,
            difficulty: self.difficulty,
            strategy: self.strategy,
            opponents: self.opponents.clone(),
            adaptation: self.adaptation,
            strategy_switches: self.strategy_switches.clone(),
            threat_log: self.threat_log.clone(),
            campaigns: self.campaigns.clone(),
            planning_campaigns: self.planning_campaigns.clone(),
            plan_layer: self.plan_layer,
            search: self.search,
            sparta_search_policy: self.sparta_search_policy,
            search_log: self.search_log.clone(),
            fast: self.fast,
            oracle: self.oracle.clone(),
            prior_maps: Default::default(),
        }
    }

    /// Diagnostics: this turn's orders and the trace of how they were reached. Runs the live
    /// path (`Brain::orders`) on this brain, so the orders and the memory left behind are exactly
    /// those of an untraced call.
    #[doc(hidden)]
    pub fn order_diagnosis(&mut self, view: &PlayerView) -> (Vec<Order>, OrderTrace) {
        let mut trace = OrderTrace::recording();
        let orders = self.orders_staged(view, &mut trace);
        trace.executed = orders.clone();
        crate::decision_trace::complete_production(&mut trace, view, &orders);
        crate::decision_trace::reconcile(
            &mut trace.decisions,
            &orders,
            &trace.stages,
            &trace.passes,
            &trace.city_passes,
        );
        (orders, trace)
    }

    /// Diagnostics: this turn's orders (exactly those `orders` returns) plus what every idle city
    /// weighed and chose. Works on a copy: this brain's memory is untouched.
    #[doc(hidden)]
    pub fn production_diagnosis(
        &self,
        view: &PlayerView,
    ) -> (Vec<Order>, Vec<crate::production::ProductionTrace>) {
        let (orders, trace) = self.diagnostic_copy().order_diagnosis(view);
        (orders, trace.production)
    }

    /// Diagnostics: why Settler `unit` got this turn's order (its plan branch, the verdict on
    /// its own tile and the best site of each kind) and, given `probe`, the site search's
    /// verdict on that tile too. Works on a copy: memory is untouched.
    #[doc(hidden)]
    pub fn settler_diagnosis(
        &self,
        view: &PlayerView,
        unit: legatum_core::UnitId,
        probe: Option<legatum_core::Hex>,
    ) -> Vec<String> {
        let mut copy = self.diagnostic_copy();
        crate::civilians::trace_settler(unit, probe, || copy.order_diagnosis(view)).1
    }
}

impl ScriptedBrain {
    /// Diagnostics: how this AI would answer a Peace with `against` this turn, as its
    /// diplomacy sees it (after the plan is chosen), and why.
    #[doc(hidden)]
    pub fn peace_diagnosis(
        &self,
        view: &PlayerView,
        against: legatum_core::TeamId,
    ) -> crate::PeaceDiagnosis {
        let affordable = crate::supply::affordable(view);
        self.planner_through_plan(&affordable, false)
            .peace_diagnosis(against)
    }

    /// Diagnostics: per own city this turn, the conqueror alarm, the neighbour-fall warning,
    /// the local defence
    /// it is weighed against, the exposure to the top threat and whether a rival's border
    /// threat is assessed there. Works on a copy: memory is untouched.
    #[doc(hidden)]
    pub fn alarm_diagnosis(&self, view: &PlayerView) -> Vec<(legatum_core::CityId, [i32; 5])> {
        let affordable = crate::supply::affordable(view);
        let p = self.planner_through_plan(&affordable, false);
        p.ctx
            .own_cities
            .iter()
            .map(|c| {
                (
                    c.id,
                    [
                        p.conqueror_alarm(c),
                        p.neighbour_fall_power(c),
                        p.local_defense(c),
                        i32::from(p.threat_exposure(c)),
                        i32::from(p.assessed_border_threat(c.pos)),
                    ],
                )
            })
            .collect()
    }
}

/// Add the air planner's orders, except for a plane the ground planner already ordered (gene
/// `breach`), which keeps that order.
fn extend_air(orders: &mut Vec<Order>, air: Vec<Order>) {
    let ordered: BTreeSet<UnitId> = orders
        .iter()
        .filter_map(|o| match o {
            Order::AirStrike { plane, .. } => Some(*plane),
            _ => None,
        })
        .collect();
    orders.extend(air.into_iter().filter(|o| match o {
        Order::AirStrike { plane, .. }
        | Order::Rebase { plane, .. }
        | Order::UpgradeAircraft { plane, .. } => !ordered.contains(plane),
        _ => true,
    }));
}

impl ScriptedBrain {
    /// The one live order path, optionally traced: plan, goal actions, air, budget, allied-city
    /// filters. `Brain::orders` and the diagnostics both run it.
    fn orders_staged(&mut self, view: &PlayerView, trace: &mut OrderTrace) -> Vec<Order> {
        let affordable = crate::supply::affordable(view);
        let mut orders = self.plan_staged(&affordable, trace);
        let before = trace.before(&orders);
        crate::goal_actions::with_goal_actions(
            &affordable,
            &mut orders,
            self.decisions.goal_open_end > 0,
            &self.decisions,
        );
        trace.stage(
            "goal-actions",
            "replaced or dropped by a faction goal action",
            before,
            &orders,
        );
        if self.difficulty != AiDifficulty::Easy {
            let attack_view =
                projected_attack_view(&affordable, &orders, self.decisions.declare_and_strike > 0);
            let air = crate::air::air_orders_with_prizes(
                &attack_view,
                self.decisions.fighter_strike != 0,
                self.decisions.air_marginal_strikes != 0,
                &orders,
                self.decisions.air_defense,
                self.decisions.air_rebase != 0,
                self.decisions.air_stage_read != 0 && self.difficulty == AiDifficulty::Hard,
            );
            let before = trace.before(&orders);
            extend_air(&mut orders, air);
            trace.stage("air", "", before, &orders);
        }
        let before = trace.before(&orders);
        let mut orders = crate::supply::within_budget(&affordable, orders);
        trace.stage(
            "budget",
            "over budget (supply::within_budget)",
            before,
            &orders,
        );
        // An ally's units may pass through its cities but not stop there: a move that ends in one
        // is refused by the engine, so drop it and let the unit be planned again next turn.
        let before = trace.before(&orders);
        orders.retain(|o| match o {
            Order::MoveUnit { to, .. } => !view.cities.iter().any(|c| {
                c.pos == *to && !view.same_team(view.me, c.owner) && view.allied(view.me, c.owner)
            }),
            _ => true,
        });
        trace.stage(
            "allied-move",
            "move would end in an allied city",
            before,
            &orders,
        );
        let before = trace.before(&orders);
        crate::drop_stays_in_allied_cities(view, &mut orders);
        trace.stage(
            "allied-stay",
            "stay or path move ends in an allied city",
            before,
            &orders,
        );
        let before = trace.before(&orders);
        crate::drop_leaving_orders_of_locked_units(view, &mut orders);
        trace.stage(
            "occupation-lock",
            "order would take a locked unit out of its captured city",
            before,
            &orders,
        );
        let before = trace.before(&orders);
        crate::drop_purchases_in_frozen_cities(view, &mut orders);
        trace.stage(
            "occupation-freeze",
            "purchase or hire in a city frozen by the occupation lock",
            before,
            &orders,
        );
        orders
    }

    /// `Brain::plan`, optionally traced: the planner's orders, the final transform
    /// (`finish_logged`) and searched tactics.
    fn plan_staged(&mut self, view: &PlayerView, trace: &mut OrderTrace) -> Vec<Order> {
        if view.finished {
            return Vec::new();
        }
        self.assess_opponents(view);
        let starting_campaigns = self
            .planning_campaigns
            .as_ref()
            .filter(|(turn, _)| *turn == view.turn)
            .map_or_else(|| self.campaigns.clone(), |(_, memory)| memory.clone());
        self.planning_campaigns = Some((view.turn, starting_campaigns.clone()));
        self.campaigns = starting_campaigns;
        let mut log = std::mem::take(&mut self.threat_log);
        let mut p = self.planner_with(view, trace.active);
        if trace.active {
            trace.production_defense = p.production_defense_trace();
        }
        trace.proposed = trace.before(&p.orders);
        let orders = self.finish_logged(view, &mut p, &mut log);
        let proposed = trace.proposed.clone();
        trace.stage(
            "finish",
            "difficulty lapse or first-volley delay",
            proposed,
            &orders,
        );
        if let Some(production) = p.production_trace.take() {
            trace.production = production.into_inner();
        }
        if let Some(passes) = p.pass_trace.take() {
            trace.passes = passes.by_unit;
            trace.city_passes = passes.by_city;
            trace.city_choices = passes.choices;
        }
        if let Some(decisions) = p.decision_trace.take() {
            trace.decisions = decisions.into_inner();
        }
        let searched = p
            .search_event
            .take()
            .into_iter()
            .chain(p.posture_event.take())
            .chain(p.stance_event.take());
        self.campaigns = p.campaigns.clone();
        self.search_log.extend(searched);
        self.threat_log = log;
        // Hard+Search tactics: the engaged units' orders this turn are searched.
        match self
            .search
            .filter(|c| c.tactics && self.difficulty == AiDifficulty::Hard)
        {
            Some(config) => {
                let before = trace.before(&orders);
                let (orders, event) = crate::search::tactics::choose(self, &config, view, orders);
                self.search_log.extend(event);
                trace.stage(
                    "search-tactics",
                    "replaced by searched tactics",
                    before,
                    &orders,
                );
                orders
            }
            None => orders,
        }
    }
}

impl Brain for ScriptedBrain {
    fn orders_with_decision_trace(
        &mut self,
        view: &PlayerView,
    ) -> (Vec<Order>, Option<OrderTrace>) {
        if self.difficulty == AiDifficulty::Hard {
            let (orders, trace) = self.order_diagnosis(view);
            (orders, Some(trace))
        } else {
            (self.orders(view), None)
        }
    }
    fn add_air_orders(&self, view: &PlayerView, orders: &mut Vec<Order>) {
        if self.difficulty == AiDifficulty::Easy {
            return;
        }
        let attack_view =
            projected_attack_view(view, orders, self.decisions.declare_and_strike > 0);
        let air = crate::air::air_orders_with_prizes(
            &attack_view,
            self.decisions.fighter_strike != 0,
            self.decisions.air_marginal_strikes != 0,
            orders,
            self.decisions.air_defense,
            self.decisions.air_rebase != 0,
            self.decisions.air_stage_read != 0 && self.difficulty == AiDifficulty::Hard,
        );
        extend_air(orders, air);
    }

    /// Plan and return orders using the scripted policy.
    fn orders(&mut self, view: &PlayerView) -> Vec<Order> {
        self.orders_staged(view, &mut OrderTrace::default())
    }

    fn opponent_memory(&self) -> Option<crate::OpponentMemory> {
        Some(crate::OpponentMemory {
            assessments: self.opponents.clone(),
            adaptation: self.adaptation,
            switches: self.strategy_switches.clone(),
            threat_log: self.threat_log.clone(),
            campaigns: self.campaigns.clone(),
        })
    }
    fn restore_opponent_memory(&mut self, memory: crate::OpponentMemory) {
        self.opponents = memory.assessments;
        self.adaptation = memory.adaptation;
        self.strategy_switches = memory.switches;
        self.threat_log = memory.threat_log;
        self.campaigns = memory.campaigns;
        self.planning_campaigns = None;
    }

    fn strategy_switches(&self) -> &[crate::opponents::StrategySwitch] {
        &self.strategy_switches
    }
    fn threat_log(&self) -> &[crate::threat::ThreatEvent] {
        &self.threat_log
    }
    fn campaign_events(&self) -> &[crate::campaign::CampaignEvent] {
        &self.campaigns.events
    }
    fn plan_events(&self) -> &[crate::plan::PlanEvent] {
        &self.campaigns.plan.events
    }
    fn war_diagnosis(&self, view: &PlayerView) -> Vec<String> {
        ScriptedBrain::war_diagnosis(self, view)
    }
    fn blocked_expenses(&self) -> u32 {
        self.campaigns.plan.blocked_expenses
    }
    fn search_events(&self) -> &[crate::search::SearchEvent] {
        &self.search_log
    }
    fn wants_true_state(&self) -> bool {
        self.search.is_some_and(|c| c.omniscient)
    }
    fn observe_true_state(&mut self, state: std::sync::Arc<legatum_core::GameState>) {
        if self.wants_true_state() {
            self.oracle = Some(state);
        }
    }
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        self.plan_staged(view, &mut OrderTrace::default())
    }
}

/// Jitter streams, so different decisions draw independent noise.
pub(crate) mod tag {
    pub const RESEARCH: u64 = 1;
    pub const PRODUCTION: u64 = 2;
    pub const SETTLE: u64 = 3;
}

/// Per-pass memo of hard-defence declaration reach, by rival unit.
pub(crate) type RouteMemo = RefCell<BTreeMap<UnitId, Rc<Vec<bool>>>>;
/// Per-pass memo of each city's attacks (`purchase_defense::city_attacks`).
pub(crate) type AttackMemo =
    RefCell<BTreeMap<legatum_core::CityId, Rc<Vec<(i32, UnitId, legatum_core::AttackMode)>>>>;
/// Per-pass memo of strike forces by (city, window, at war with its owner).
pub(crate) type StrikeMemo = RefCell<BTreeMap<StrikeKey, Rc<Vec<(UnitId, Hex, i32)>>>>;
/// (city, approach window, at war with its owner).
pub(crate) type StrikeKey = (legatum_core::CityId, i32, bool);

pub(crate) struct Planner<'a> {
    pub conquest_city: std::cell::OnceCell<Option<legatum_core::CityId>>,
    /// Per unit, tiles it can't reach this pass once a campaign approach search failed.
    pub(crate) approach_reach: std::cell::RefCell<
        std::collections::BTreeMap<UnitId, std::rc::Rc<crate::military::ApproachReach>>,
    >,
    /// Hard-defence declaration reach per rival unit this pass (`hard_defense.rs`).
    pub(crate) declaration_routes: RouteMemo,
    /// A prospective planner's link to its parent's `declaration_routes`.
    pub(crate) shared_routes: Option<&'a RouteMemo>,
    /// `purchase_defense::can_arrive` per (unit, target) this pass (the view is fixed).
    pub(crate) arrivals: std::cell::RefCell<std::collections::BTreeMap<(UnitId, Hex), bool>>,
    /// Gene `border_explained`: `civilians::unexplained_border_risks` of this view.
    pub(crate) unexplained_border_risks: std::cell::OnceCell<Vec<bool>>,
    /// `purchase_defense::city_attacks` per city this pass.
    pub(crate) city_attacks: AttackMemo,
    /// One survival forecast per own city on this fixed player view. Counterfactual defenders
    /// are deliberately uncached; the baseline is shared by all production and hold passes.
    pub(crate) defense_clocks: std::cell::RefCell<
        std::collections::BTreeMap<legatum_core::CityId, crate::purchase_defense::DefenseRisk>,
    >,
    /// `strike_force_within` per (city, window, at war with its owner) this pass.
    pub(crate) strike_forces: StrikeMemo,
    /// `diplomatic_campaign` per rival team this pass (it builds a prospective planner).
    pub(crate) diplomatic_campaigns:
        std::cell::RefCell<std::collections::BTreeMap<legatum_core::TeamId, Option<Hex>>>,
    /// This turn's Yam Relay Station decision (`goal_hall.rs`).
    pub(crate) hall_plan: std::cell::OnceCell<crate::goal_hall::HallPlan>,
    /// Technologies the pending faction goal's buildings need (`goal_techs`).
    pub goal_techs: std::cell::OnceCell<std::collections::BTreeSet<legatum_core::TechId>>,
    pub strike_plan: std::cell::OnceCell<Option<crate::opportunity::Strike>>,
    pub opening: Option<crate::openings::OpeningPlan<'a>>,
    pub primitives: Option<crate::openings::PrimitivePlan<'a>>,
    pub strike_target: Option<legatum_core::TeamId>,
    pub strike_city: Option<legatum_core::CityId>,
    pub primitive_march: Option<crate::openings::CoalitionMarch>,
    pub ctx: Ctx<'a>,
    pub opponents: Option<&'a crate::opponents::OpponentAssessments>,
    pub knobs: Knobs,
    /// Stable diplomatic disposition; strategic adaptation may lower military aggression
    /// when a local target is unsuitable, without making a personality trust strangers.
    pub personality_aggression: i32,
    pub decisions: crate::DecisionParams,
    pub faction: Faction,
    pub seed: u64,
    pub orders: Vec<Order>,
    /// Units that already have their one unit-action order this turn.
    acted: BTreeSet<UnitId>,
    pub(crate) modernization_savings: i32,
    pub(crate) modernization_cache: RefCell<BTreeMap<(Hex, legatum_core::UnitKind), i32>>,
    /// Tiles a military unit is ordered into (or holding), to avoid friendly bounces.
    pub mil_dest: crate::analysis::HexSet,
    /// Exposed staging moves stand or fall together during movement reconciliation.
    pub assault_groups: Vec<Vec<UnitId>>,
    /// Melee orders approved by a whole-city capture forecast, including attrition.
    pub assault_attacks: BTreeSet<UnitId>,
    /// Tactical escapes keep their priority through the final movement reservations.
    pub retreats: BTreeSet<UnitId>,
    /// Units sent to remove a rival's VP source this turn (gene `contest_hold`).
    pub contest_units: BTreeSet<UnitId>,
    /// Sites settlers are headed for this turn (found orders within 3 of each other all fail).
    pub settle_targets: Vec<Hex>,
    /// Sites settlers are walking to (not founding at this turn): candidates for an escort.
    pub escort_sites: Vec<Hex>,
    /// Gene `roman_found_first`: the Roman veteran sent to found a city this turn.
    pub(crate) roman_founder: Option<UnitId>,
    /// Gene `rome_colonia`: the Roman veteran sent to found the Colonia Victrix this turn.
    pub(crate) colonia_founder: Option<UnitId>,
    /// Gene `rome_colonia`: cities the would-be founder stood aside from this turn
    /// (`colonia_lock_guard`): it does not follow another unit in.
    pub(crate) colonia_aside: Vec<Hex>,
    /// Gold not yet committed this turn.
    pub gold_left: i32,
    /// A funded objective may save toward its next purchase after emergency defense.
    pub faction_goal_savings: i32,
    /// Gold set aside for the Siege the war on a walled city still needs (`siege_buy.rs`).
    pub siege_savings: i32,
    /// `exposure_key` per tile index: it reads only the view and the fixed enemy lists.
    pub(crate) exposure_memo: std::cell::RefCell<Vec<Option<(i32, i32)>>>,
    /// `safe_campaign_tile` per tile index (it reads the same fixed facts as `exposure_key`).
    pub(crate) safe_memo: std::cell::RefCell<Vec<Option<bool>>>,
    /// Visible military tiles from which a rival could intercept a Settler next turn.
    /// The player view is fixed for this planning pass.
    pub(crate) settler_intercept: std::cell::OnceCell<Vec<bool>>,
    /// Gene `buy_hold`: this pass's held Gold (`gold_hold.rs`), computed once.
    pub(crate) held_gold: std::cell::Cell<Option<i32>>,
    /// The one fund chosen from this turn's fogged view and routed plan.
    pub(crate) named_fund: std::cell::OnceCell<Option<crate::gold_hold::NamedFund>>,
    pub work_plan: Option<crate::investment::WorkPlan>,
    /// Decision policy only; the planner still receives exactly the same fogged view.
    pub difficulty: AiDifficulty,
    pub strategy: Option<crate::Strategy>,
    /// Production choices recorded for diagnostics; never read by planning.
    pub production_trace: Option<std::cell::RefCell<Vec<crate::production::ProductionTrace>>>,
    pub(crate) decision_trace: Option<std::cell::RefCell<Vec<crate::DecisionOption>>>,
    /// Which pass gave each unit its order, recorded for diagnostics; never read by planning.
    pub(crate) pass_trace: Option<PassTrace>,
    /// Gene `capture_hold`: front cities whose holder waits for the attack passes this turn.
    pub(crate) capture_deferred: Vec<legatum_core::Hex>,
    /// Gene `forecast_garrison_holds` 1: the siege forecast is running for the stance pass.
    pub(crate) stance_forecast: std::cell::Cell<bool>,
    pub campaigns: crate::campaign::CampaignMemory,
    pub campaign_withdraw: Vec<UnitId>,
    /// Hard's goal-driven plan routes war, peace and purchases (`plan.rs`).
    pub plan_routing: bool,
    /// Rollout-only policy level (`ScriptedBrain::fast`).
    pub(crate) fast: u8,
    /// Own cities a rival army is attacking (Hard). Defenders near them may stand in the
    /// attackers' reach instead of keeping clear of it.
    pub(crate) relief: Vec<legatum_core::Hex>,
    /// Cities short of their door-defence gap this turn (`door.rs`): a combat unit they are
    /// building is kept, not reconsidered.
    pub(crate) door_cities: Vec<legatum_core::CityId>,
    /// Hard+Search: rollouts decide plan review points (`search`).
    pub(crate) search: Option<crate::search::SearchHook<'a>>,
    /// This pass's search, for the brain's log.
    pub(crate) search_event: Option<crate::search::SearchEvent>,
    /// This pass's posture search (Hard+Search with postures).
    pub(crate) posture_event: Option<crate::search::SearchEvent>,
    /// This pass's stance search (Hard+Search tactics).
    pub(crate) stance_event: Option<crate::search::SearchEvent>,
}

/// The combat relationship after this submission's valid surprise declarations resolve.
pub(crate) fn projected_attack_view<'a>(
    view: &'a PlayerView,
    orders: &[Order],
    enabled: bool,
) -> std::borrow::Cow<'a, PlayerView> {
    if !enabled {
        return std::borrow::Cow::Borrowed(view);
    }
    let declarations: Vec<_> = orders
        .iter()
        .filter_map(|o| match o {
            Order::DeclareWar {
                against,
                breach: false,
            } => Some(*against),
            _ => None,
        })
        .collect();
    if declarations.is_empty() {
        return std::borrow::Cow::Borrowed(view);
    }
    let mut projected = view.clone();
    for relation in &mut projected.diplomacy.relations {
        if declarations
            .iter()
            .any(|team| relation.between(projected.team, *team))
            && legatum_core::diplomacy::surprise_war(relation, projected.turn)
        {
            relation.state = legatum_core::diplomacy::RelationState::War;
        }
    }
    std::borrow::Cow::Owned(projected)
}

impl<'a> Planner<'a> {
    pub(crate) fn note_option(&self, option: crate::DecisionOption) {
        if let Some(trace) = &self.decision_trace {
            trace.borrow_mut().push(option);
        }
    }
    /// Diagnostics: the pass named `next` is about to run (see `PassTrace`). Free when not
    /// tracing; never changes a decision.
    pub(crate) fn pass(&mut self, next: &'static str) {
        if let Some(t) = &mut self.pass_trace {
            t.mark(&self.orders, next);
        }
    }

    pub(crate) fn attack_view(&self) -> std::borrow::Cow<'_, PlayerView> {
        projected_attack_view(
            self.view(),
            &self.orders,
            self.decisions.declare_and_strike > 0,
        )
    }
    pub fn new(view: &'a PlayerView, knobs: Knobs, faction: Faction, seed: u64) -> Self {
        Self::with_ctx(view, Ctx::new(view), knobs, faction, seed)
    }

    /// A planner on a prospective view of `parent`'s (see `Ctx::prospective`).
    pub(crate) fn prospective(
        view: &'a PlayerView,
        parent: &Ctx<'_>,
        knobs: Knobs,
        faction: Faction,
        seed: u64,
    ) -> Self {
        Self::with_ctx(view, Ctx::prospective(view, parent), knobs, faction, seed)
    }

    fn with_ctx(
        view: &'a PlayerView,
        ctx: Ctx<'a>,
        knobs: Knobs,
        faction: Faction,
        seed: u64,
    ) -> Self {
        let work_plan = crate::investment::work_plan(&ctx);
        Self {
            conquest_city: Default::default(),
            diplomatic_campaigns: Default::default(),
            strike_forces: Default::default(),
            arrivals: Default::default(),
            unexplained_border_risks: Default::default(),
            city_attacks: Default::default(),
            defense_clocks: Default::default(),
            held_gold: Default::default(),
            named_fund: Default::default(),
            declaration_routes: Default::default(),
            shared_routes: None,
            approach_reach: Default::default(),
            hall_plan: Default::default(),
            goal_techs: Default::default(),
            strike_plan: Default::default(),
            ctx,
            opponents: None,
            opening: None,
            primitives: None,
            strike_target: None,
            strike_city: None,
            primitive_march: None,
            work_plan,
            difficulty: AiDifficulty::Normal,
            strategy: None,
            production_trace: None,
            decision_trace: None,
            pass_trace: None,
            capture_deferred: Vec::new(),
            stance_forecast: std::cell::Cell::new(false),
            campaigns: Default::default(),
            campaign_withdraw: Vec::new(),
            plan_routing: false,
            search: None,
            search_event: None,
            fast: 0,
            relief: Vec::new(),
            door_cities: Vec::new(),
            posture_event: None,
            stance_event: None,
            knobs,
            personality_aggression: knobs.aggression,
            decisions: crate::DecisionParams::builtin_for(faction),
            faction,
            seed,
            orders: Vec::new(),
            acted: BTreeSet::new(),
            modernization_savings: 0,
            modernization_cache: Default::default(),
            mil_dest: crate::analysis::HexSet::new(view.width, view.height),
            assault_groups: Vec::new(),
            assault_attacks: BTreeSet::new(),
            retreats: BTreeSet::new(),
            contest_units: BTreeSet::new(),
            settle_targets: Vec::new(),
            escort_sites: Vec::new(),
            roman_founder: None,
            colonia_founder: None,
            colonia_aside: Vec::new(),
            gold_left: view.economy.gold,
            faction_goal_savings: 0,
            siege_savings: 0,
            exposure_memo: Default::default(),
            safe_memo: Default::default(),
            settler_intercept: Default::default(),
        }
    }

    pub fn view(&self) -> &'a PlayerView {
        self.ctx.view
    }

    /// Whether a faction rule applies in this game: an ablation can switch it off, and the AI
    /// then plans without it.
    pub(crate) fn rule_on(&self, rule: legatum_core::experiments::FactionRule) -> bool {
        !self.view().rule_experiments.rule_off(rule)
    }

    pub(crate) fn movement_map(&self) -> &'a Map {
        let view = self.view();
        view.tiles.terrain_map(view.width, view.height)
    }

    /// Deterministic noise in `0..=max` for one decision.
    pub fn jitter(&self, stream: u64, id: u64, max: i32) -> i32 {
        if max <= 0 {
            return 0;
        }
        let mut r = Rng::derive(
            self.seed,
            &[
                self.view().turn as u64,
                purpose::AI,
                self.ctx.me.0 as u64,
                stream,
                id,
            ],
        );
        r.range(0, max as i64) as i32
    }

    pub fn has_acted(&self, u: UnitId) -> bool {
        self.acted.contains(&u)
    }

    pub(crate) fn release_modernization_hold(&mut self, unit: UnitId) {
        self.orders
            .retain(|o| !matches!(o, Order::SkipUnit { unit: id } if *id == unit));
        self.acted.remove(&unit);
    }

    /// Issue a unit-action order, unless the unit already has one. Returns whether it was issued.
    pub fn unit_order(&mut self, unit: UnitId, order: Order) -> bool {
        if self.has_acted(unit) {
            return false;
        }
        // Rome's Colonia founder never walks into an occupation lock (`goal_colonia.rs`).
        if let Some(placed) = self.colonia_lock_guard(unit, &order) {
            return placed;
        }
        if !self.acted.insert(unit) {
            return false;
        }
        // A longer route can cross a newly sighted enemy even when its destination remains
        // useful. Stop at the last clear tile; the core rejects known enemies inside a
        // MoveUnitPath, including when stop_on_block is set.
        let order =
            match order {
                Order::MoveUnitPath {
                    unit,
                    to,
                    path,
                    stop_on_block,
                } => {
                    let blocked =
                        path.iter()
                            .take(path.len().saturating_sub(1))
                            .position(|h| {
                                self.view().units.iter().any(|u| {
                                    u.pos == *h && self.view().at_war(self.view().me, u.owner)
                                }) || self.view().cities.iter().any(|c| {
                                    c.pos == *h && self.view().at_war(self.view().me, c.owner)
                                })
                            });
                    if let Some(i) = blocked {
                        if let Some(&safe_to) = path[..i].last() {
                            Order::MoveUnitPath {
                                unit,
                                to: safe_to,
                                path: path[..i].to_vec(),
                                stop_on_block,
                            }
                        } else {
                            Order::SkipUnit { unit }
                        }
                    } else {
                        Order::MoveUnitPath {
                            unit,
                            to,
                            path,
                            stop_on_block,
                        }
                    }
                }
                other => other,
            };
        self.orders.push(order);
        true
    }

    /// Move `u` to `to`, or hold if `to` is where it stands or not a legal target.
    pub fn move_or_hold(&mut self, u: &UnitView, to: Hex) -> bool {
        if to == u.pos || !self.ctx.reachable(u.pos, to) {
            self.unit_order(u.id, Order::SkipUnit { unit: u.id })
        } else {
            self.unit_order(u.id, Order::MoveUnit { unit: u.id, to })
        }
    }

    /// Units that had a destination but did not move at all last turn (full moves left, goto
    /// kept): they bounced off a friendly unit or were blocked.
    pub fn stuck(u: &UnitView) -> bool {
        u.goto.is_some_and(|g| g != u.pos) && u.moves_left >= u.max_moves && u.max_moves > 0
    }

    /// Friendly units of the same class stepping into the same tile all bounce, and would do
    /// so again every turn with the same orders. Break the tie: of two stuck same-class units
    /// within 2 of each other, the higher id holds this turn and the lower id goes first.
    pub fn plan_yields(&mut self) {
        let stuck: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| Self::stuck(u))
            .collect();
        for u in &stuck {
            // Tactical response must not be preempted by a routine traffic hold.
            if self.exposure_at(u.pos) > 0 {
                continue;
            }
            let yields = stuck
                .iter()
                .any(|o| o.id < u.id && o.military == u.military && o.pos.distance(u.pos) <= 2)
                || self.view().units.iter().any(|o| {
                    // Other teams' destinations are private. Full movement remaining and
                    // proximity are visible; a stable ID breaks recurring neutral contests.
                    o.owner != self.ctx.me
                        && !self.view().at_war(self.ctx.me, o.owner)
                        && o.id < u.id
                        && o.moves_left >= o.max_moves
                        && o.pos.distance(u.pos) <= 2
                });
            if yields {
                if u.military {
                    self.mil_dest.insert(u.pos);
                }
                self.unit_order(u.id, Order::SkipUnit { unit: u.id });
            }
        }
    }

    pub(crate) fn attack_threshold(&self) -> i32 {
        crate::search::Stance::threshold(
            self.campaigns.plan.stance,
            self.knobs.attack_threshold() * self.decisions.attack_ratio_percent / 100,
        )
    }

    /// Gold the AI keeps in hand before buying or upgrading.
    pub fn gold_reserve(&self) -> i32 {
        self.ordinary_gold_reserve()
            .max(self.faction_goal_savings)
            .max(self.siege_savings)
            + self.modernization_savings
            + self.operation_reserve()
    }

    /// Gene `hoard_at_war`: the plan fights a war we are in, and the not-yet-held hoard is not
    /// what wins the game, so the Gold saved toward it is spendable.
    /// `swiss_hoard_any_war` also releases it when an actual war lacks that plan.
    pub(crate) fn hoard_spent_at_war(&self) -> bool {
        if self.decisions.hoard_at_war == 0
            || !self.hard_defense()
            || self.hoard_held()
            || self.saving_wins()
        {
            return false;
        }
        let v = self.view();
        self.routed_plan()
            .and_then(|p| p.kind.war_target())
            .is_some_and(|team| v.teams_at_war(v.team, team))
            || (self.faction == Faction::Switzerland
                && self.decisions.swiss_hoard_any_war > 0
                && v.players
                    .iter()
                    .any(|player| player.id != v.me && v.at_war(v.me, player.id)))
    }

    pub(crate) fn ordinary_gold_reserve(&self) -> i32 {
        let defender = if self.hard_defense() && self.decisions.neighbour_fall > 0 {
            self.ctx.own_cities.iter().filter(|c| self.neighbour_fall_power(c) > 0)
                .filter_map(|c| c.detail.as_ref())
                .flat_map(|d| d.options.iter())
                .filter(|o| matches!(o.item, legatum_core::ProductionItem::Unit { unit }
                    if crate::composition::is_melee_line(unit) || crate::composition::is_ranged_line(unit)))
                .filter_map(|o| o.buy_cost)
                .min().unwrap_or(0)
        } else {
            0
        };
        if self.faction == Faction::Switzerland
            && self.saving_for_goal()
            && !self.hoard_spent_at_war()
        {
            // Bank the treasury only when its VP is the best concrete finish.
            // Otherwise land, buildings and defenders can earn more than idle Gold.
            // Emergency defenders still bypass this ordinary purchase reserve.
            // Held, the hoard scores a live VP: only Gold above it (plus any deficit the
            // turn's income will draw down) is spendable on routine buys.
            let hoard = legatum_core::goals::SWISS_HOARD_GOLD;
            return defender.max(if self.hoard_held() {
                hoard + (-self.view().economy.gold_per_turn.total).max(0)
            } else {
                hoard
            });
        }
        if self.faction == Faction::Switzerland {
            let defense = 100;
            let deficit = self.decisions.reserve_deficit_turns
                * (-self.view().economy.gold_per_turn.total).max(0);
            return defense.max(deficit).max(defender);
        }
        if let Some(o) = self.opening.filter(|_| !self.rushing()) {
            return o.policy.numbers.gold_save.max(defender);
        }
        if let Some(reserve) = self.primitive_gold_reserve() {
            return reserve.max(defender);
        }
        if let Some(o) = self.opening {
            return o.policy.numbers.gold_save.max(defender);
        }
        // Solvent empires invest their income. A reserve pays an actual deficit, not a
        // growing arbitrary tax on every city; it must not strand a dying empire's gold.
        let deficit = self.decisions.reserve_deficit_turns
            * (-self.view().economy.gold_per_turn.total).max(0);
        deficit.max(defender)
    }

    /// Pressure rises in the second half, or when a hostile team approaches its public VP
    /// target. It changes objectives and army investment, never the safe-attack threshold.
    pub(crate) fn conquest_pressure(&self) -> i32 {
        let v = self.view();
        let horizon = v
            .turn_limit
            .unwrap_or(legatum_core::data::EXPECTED_GAME_TURNS)
            .max(1);
        let time = (u64::from(v.turn)
            .saturating_mul(2)
            .saturating_sub(u64::from(horizon))
            * 100
            / u64::from(horizon))
        .min(100) as i32;
        let race = v
            .standings
            .iter()
            .filter(|s| s.team != v.team && s.target > 0)
            .map(|s| {
                // Hard reads the public VP track earlier and reacts to a runaway leader
                // from 40% of its target instead of 60%.
                if self.difficulty == AiDifficulty::Hard {
                    (s.vp * 100 / s.target)
                        .saturating_sub(self.decisions.leader_pressure_hard_percent as u32)
                        .min(50) as i32
                        * 2
                } else {
                    (s.vp * 100 / s.target)
                        .saturating_sub(self.decisions.leader_pressure_percent as u32)
                        .min(40) as i32
                        * 2
                }
            })
            .max()
            .unwrap_or(0);
        // A conqueror on the war path is a race too, even with few VP (threat.rs).
        time.max(race).max(self.threat_pressure())
    }

    pub(crate) fn turns_remaining(&self) -> i32 {
        // Uncapped games must keep researching and building past the expected length.
        self.view().turn_limit.map_or(i32::MAX, |limit| {
            limit
                .saturating_sub(self.view().turn)
                .saturating_add(1)
                .min(i32::MAX as u32) as i32
        })
    }
}

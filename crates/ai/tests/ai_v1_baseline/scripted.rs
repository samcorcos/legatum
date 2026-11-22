//! The scripted v1 AI: knob-weighted heuristics over a PlayerView.
//!
//! One `Planner` is built per `orders()` call. It plans research, then cities (production, focus,
//! buying), then civilians (settlers, scouts), then the army. Each step records which units have
//! an order so no unit ever gets two, and which tiles units are headed for so they don't bounce.

use crate::ai_v1_baseline::Brain;
use crate::ai_v1_baseline::analysis::Ctx;
use crate::ai_v1_baseline::knobs::Knobs;
use legatum_core::rng::{Rng, purpose};
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, PlayerId, PlayerView, UnitId};
use std::collections::BTreeSet;

/// The scripted v1 AI. Knobs are chosen per faction; decisions are deterministic given the view
/// and the seed. It keeps no state between turns (anything it needs to remember, such as a
/// settler's chosen site, it reads back from its units' gotos in the view).
pub struct ScriptedBrain {
    pub player: PlayerId,
    pub faction: Faction,
    pub seed: u64,
    pub knobs: Knobs,
}

impl ScriptedBrain {
    pub fn new(faction: Faction, seed: u64, player: PlayerId) -> Self {
        Self::with_knobs(faction, seed, player, Knobs::for_faction(faction))
    }

    pub fn with_knobs(faction: Faction, seed: u64, player: PlayerId, knobs: Knobs) -> Self {
        Self {
            player,
            faction,
            seed,
            knobs: knobs.clamped(),
        }
    }
}

impl Brain for ScriptedBrain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        if view.finished {
            return Vec::new();
        }
        let mut p = Planner::new(view, self.knobs, self.faction, self.seed);
        p.plan_research();
        p.plan_yields();
        p.plan_cities();
        p.plan_settlers();
        p.plan_scouts();
        p.plan_military();
        p.dedupe_first_steps();
        p.orders
    }
}

/// Jitter streams, so different decisions draw independent noise.
pub(crate) mod tag {
    pub const RESEARCH: u64 = 1;
    pub const PRODUCTION: u64 = 2;
    pub const SETTLE: u64 = 3;
}

pub(crate) struct Planner<'a> {
    pub ctx: Ctx<'a>,
    pub knobs: Knobs,
    pub faction: Faction,
    pub seed: u64,
    pub orders: Vec<Order>,
    /// Units that already have their one unit-action order this turn.
    acted: BTreeSet<UnitId>,
    /// Tiles a military unit is ordered into (or holding), to avoid friendly bounces.
    pub mil_dest: BTreeSet<Hex>,
    /// Sites settlers are headed for this turn (found orders within 3 of each other all fail).
    pub settle_targets: Vec<Hex>,
    /// Sites settlers are walking to (not founding at this turn): candidates for an escort.
    pub escort_sites: Vec<Hex>,
    /// Gold not yet committed this turn.
    pub gold_left: i32,
}

impl<'a> Planner<'a> {
    pub fn new(view: &'a PlayerView, knobs: Knobs, faction: Faction, seed: u64) -> Self {
        Self {
            ctx: Ctx::new(view),
            knobs,
            faction,
            seed,
            orders: Vec::new(),
            acted: BTreeSet::new(),
            mil_dest: BTreeSet::new(),
            settle_targets: Vec::new(),
            escort_sites: Vec::new(),
            gold_left: view.economy.gold,
        }
    }

    pub fn view(&self) -> &'a PlayerView {
        self.ctx.view
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

    /// Issue a unit-action order, unless the unit already has one. Returns whether it was issued.
    pub fn unit_order(&mut self, unit: UnitId, order: Order) -> bool {
        if !self.acted.insert(unit) {
            return false;
        }
        self.orders.push(order);
        true
    }

    /// Move `u` to `to`, or hold if `to` is where it stands or not a legal target.
    pub fn move_or_hold(&mut self, u: &UnitView, to: Hex) -> bool {
        if to == u.pos || !self.ctx.passable_known(to) {
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
            let yields = stuck
                .iter()
                .any(|o| o.id < u.id && o.military == u.military && o.pos.distance(u.pos) <= 2);
            if yields {
                if u.military {
                    self.mil_dest.insert(u.pos);
                }
                self.unit_order(u.id, Order::SkipUnit { unit: u.id });
            }
        }
    }

    /// Gold the AI keeps in hand before buying or upgrading.
    pub fn gold_reserve(&self) -> i32 {
        let goal_done = self
            .view()
            .players
            .iter()
            .find(|p| p.id == self.ctx.me)
            .is_some_and(|p| p.faction_goal_done);
        if self.faction == Faction::Switzerland && !goal_done {
            // Faction goal: hold 400 gold at once.
            return 400;
        }
        30 + 10 * self.ctx.own_cities.len() as i32 + (100 - self.knobs.economy) / 2
    }
}

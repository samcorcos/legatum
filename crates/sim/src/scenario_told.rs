//! `legatum-sim scenario --tell-objective` (measurement only): the scenario seat's Hard brain is
//! told the card's win condition as its goal, to split "does not know the goal" from "cannot
//! execute it". Nothing in the app or a normal game builds this brain.
//!
//! The goal is handed over the way the offline operation search commits a plan: a pinned
//! operation (`legatum_ai::operation`, `OperationKind::Pin`) that the plan layer must route
//! until the card's last turn. Every executor is Hard's own; only the plan choice is told.
//! - **Take a city** (`OwnCity`): a Campaign on the first card target the seat does not own yet,
//!   against its owner's team.
//! - **Hold the starting cities** (no city to take): the quiet plan (`PlanKind::Economy`), so no
//!   offensive campaign, Contain war or siege stance on another war's city pulls the army away.
//!   With `--tell-hold-city`, `PlanKind::Hold` on the own city with the most visible enemy
//!   military within 3 tiles instead (the Hard+Search hold plan; the quiet plan while none is
//!   threatened). Before `plan.rs::execute_hold` could swap a stronger defender in, this took
//!   Hold Biel from 34/50 to 6/50.
//!
//! The brain reads only its own `PlayerView`, like every brain; the card's condition is the one
//! thing it is given that a normal seat does not have.

use legatum_ai::operation::{Operation, OperationKind};
use legatum_ai::plan::{PlanEvent, PlanKind};
use legatum_ai::{Brain, OrderTrace, ScriptedBrain};
use legatum_core::scenario::Condition;
use legatum_core::{CityId, Order, PlayerView};
use std::sync::{Arc, Mutex};

/// What the card asks, as the seat's brain is told it.
#[derive(Clone, Debug)]
pub struct Told {
    /// Cities to take, in card order.
    pub take: Vec<CityId>,
    /// The card asks to hold the starting cities and names no city to take.
    pub hold: bool,
    /// Tell a hold as `PlanKind::Hold` on the most threatened city, not the quiet plan.
    pub hold_city: bool,
    /// First turn after the window (the operation's budget ends here).
    pub until: u32,
}

impl Told {
    pub fn from_card(win: &[Condition], keep: &[Condition], until: u32, hold_city: bool) -> Self {
        let take: Vec<CityId> = win
            .iter()
            .filter_map(|c| match c {
                Condition::OwnCity(id) => Some(*id),
                _ => None,
            })
            .collect();
        let hold = take.is_empty()
            && win
                .iter()
                .chain(keep)
                .any(|c| matches!(c, Condition::HoldStartingCities));
        Self {
            take,
            hold,
            hold_city,
            until,
        }
    }

    /// The plan this turn's view should pin, if any.
    pub fn plan(&self, view: &PlayerView) -> Option<PlanKind> {
        if let Some(city) = self.take.iter().find_map(|id| {
            view.cities
                .iter()
                .find(|c| c.id == *id && c.owner != view.me && !view.same_team(view.me, c.owner))
        }) {
            return Some(PlanKind::Campaign {
                against: view.team_of(city.owner),
                target: city.id,
            });
        }
        if !self.hold {
            return None;
        }
        let threatened = view
            .cities
            .iter()
            .filter(|c| c.owner == view.me)
            .map(|c| {
                let near = view
                    .units
                    .iter()
                    .filter(|u| {
                        u.military && view.at_war(view.me, u.owner) && u.pos.distance(c.pos) <= 3
                    })
                    .count();
                (near, std::cmp::Reverse(c.id), c.id)
            })
            .filter(|(near, _, _)| *near > 0)
            .max();
        match threatened {
            Some((_, _, city)) if self.hold_city => Some(PlanKind::Hold { city }),
            _ => Some(PlanKind::Economy),
        }
    }
}

/// One turn of the traced seat: its orders and the trace of how they were reached.
pub type TraceSlot = Arc<Mutex<Vec<(u32, Vec<Order>, OrderTrace)>>>;

/// The scenario seat's brain with an optional told objective and an optional order trace.
pub struct SeatBrain {
    pub inner: ScriptedBrain,
    pub told: Option<Told>,
    /// Plans told, one per turn, and the plan layer's switches (for the run report).
    pub log: Arc<Mutex<Vec<(u32, String)>>>,
    /// Plan events already logged.
    pub seen: usize,
    pub trace: Option<TraceSlot>,
}

impl SeatBrain {
    fn tell(&mut self, view: &PlayerView) {
        let Some(told) = &self.told else {
            return;
        };
        let Some(plan) = told.plan(view) else {
            return;
        };
        self.log
            .lock()
            .expect("told log")
            .push((view.turn, format!("told {plan:?}")));
        // A pinned operation is honoured only with gene `operations` on (or a second-front
        // bit of `runaway_coalition`); telling turns it on for this seat.
        self.inner.decisions.operations = self.inner.decisions.operations.max(1);
        self.inner.set_operation(Operation::new(
            OperationKind::Pin { plan },
            view.turn,
            told.until.saturating_sub(view.turn).max(1),
            0,
        ));
    }
}

impl Brain for SeatBrain {
    fn plan(&mut self, view: &PlayerView) -> Vec<Order> {
        self.inner.plan(view)
    }

    fn orders(&mut self, view: &PlayerView) -> Vec<Order> {
        self.tell(view);
        let orders = self.decide(view);
        let events = self.inner.plan_events();
        let mut log = self.log.lock().expect("told log");
        for e in events.iter().skip(self.seen) {
            if let PlanEvent::Switch { to, reason, .. } = e {
                log.push((view.turn, format!("plan {to:?} ({reason:?})")));
            }
        }
        self.seen = events.len();
        orders
    }

    fn add_air_orders(&self, view: &PlayerView, orders: &mut Vec<Order>) {
        self.inner.add_air_orders(view, orders);
    }
}

impl SeatBrain {
    fn decide(&mut self, view: &PlayerView) -> Vec<Order> {
        match &self.trace {
            Some(slot) => {
                let (orders, trace) = self.inner.order_diagnosis(view);
                slot.lock()
                    .expect("trace")
                    .push((view.turn, orders.clone(), trace));
                orders
            }
            None => self.inner.orders(view),
        }
    }
}

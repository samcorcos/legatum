//! A committed operation (AI strategy reset increment 1b, `docs/design/ai-strategy/fable-2026-11-07.md`
//! root cause 3, "intent never becomes action").
//!
//! The plan layer re-chooses its plan every turn, and a plan only becomes action through
//! executor passes that each re-judge it (the campaign's funded forecast, the reachability
//! gates, Sparta's campaign rule). In the operation search (increment 1) a forced strike took
//! its target in only 20-28% of samples. An operation is the commitment that survives those
//! re-judgements:
//!
//! - **kind and target**: a strike on one enemy city (optionally bought), or a pinned plan;
//! - **roster**: every own land military unit within `radius` of the target when it starts,
//!   plus every unit produced or bought for it while it runs (a new unit in a city within the
//!   radius);
//! - **phases**: assemble at a staging ring 3 tiles from the target, strike when two thirds of
//!   the roster is there (or the assembly turn passes; War is declared then), storm when the
//!   city's HP is 0;
//! - **abort conditions** and a **turn budget**.
//!
//! Rostered units are reserved: the operation pass orders them before every other military
//! pass, so garrison, defence quotas, upgrades and escorts never see them. The one exception
//! is a home city actually under attack (an enemy unit next to it, or a damaged city with one
//! within 2): the roster units within 4 tiles of it are released to its defence that turn, and
//! the override is logged. A running operation's plan outranks the plan layer's re-choice and
//! faction rules (`plan.rs::choose_plan`), its campaign is not ended by the campaign reconciler,
//! and its strike refuses Peace with the target's team. Only its own abort conditions end it.
//!
//! Gene `operations` (default 0) turns the executor on. Nothing in live play creates an
//! operation yet: `search::OracleNode::advance_committed` (the offline `operation_search`) does.

use crate::campaign::Campaign;
use crate::opportunity::legal_declaration;
use crate::plan::PlanKind;
use crate::scripted::Planner;
use legatum_core::view::UnitView;
use legatum_core::{CityId, Hex, Order, ProductionItem, TeamId, UnitId, UnitKind, data};
use serde::{Deserialize, Serialize};

/// What the operation does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    /// March the roster on `target`, strike when assembled, storm when its HP is 0. `buy`:
    /// spend the treasury on units where the strike assembles.
    Strike {
        against: TeamId,
        target: CityId,
        buy: bool,
    },
    /// Hold this plan over the plan layer and faction rules (Economy, Peace, ...). No roster.
    Pin { plan: PlanKind },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Assemble,
    Strike,
    Storm,
    /// A pinned plan holds.
    Hold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationOutcome {
    Running,
    /// The target city is ours.
    Captured,
    /// The turn budget ran out.
    Expired,
    /// The target city no longer exists.
    TargetGone,
    /// A third team took the target.
    TargetTaken,
    /// Peace with the target's team after the strike began.
    Peace,
    /// No unit left on the roster and nothing to buy one with.
    RosterLost,
    /// War on the target's team stayed illegal for 5 turns after assembly.
    DeclarationBlocked,
    /// A pinned plan's objective is no longer possible.
    PlanEnded,
}

/// What happened, for the search's referee logs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationLog {
    /// The seat's plan was the operation's on its first turn and, for a strike, the roster
    /// was not empty.
    pub took: bool,
    pub roster_start: u32,
    pub roster_max: u32,
    /// Most roster units within 3 tiles of the target on one turn.
    pub arrived: u32,
    pub strike_turn: Option<u32>,
    pub storm_turn: Option<u32>,
    /// Roster attacks on the target city or units on it (orders issued).
    pub attacks: u32,
    pub bought: u32,
    pub gold_spent: i32,
    /// Roster units released to a home city under attack (unit-turns).
    pub overrides: u32,
    /// (turn, city) of each home-defence release.
    pub override_events: Vec<(u32, CityId)>,
    pub end_turn: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    pub kind: OperationKind,
    pub start_turn: u32,
    /// Last turn of the budget (exclusive).
    pub until: u32,
    /// Roster radius around the target.
    pub radius: i32,
    pub roster: Vec<UnitId>,
    /// Roster filled (the first planning pass after the start).
    pub started: bool,
    pub phase: Phase,
    /// Strike no later than this turn, assembled or not.
    pub assemble_by: u32,
    /// Turns in the Strike phase with War still undeclared and illegal.
    pub blocked_turns: u32,
    /// War with the target's team was seen after the strike began.
    pub at_war_seen: bool,
    pub outcome: OperationOutcome,
    pub log: OperationLog,
}

/// Roster units this close to the staging ring's target count as assembled.
const ASSEMBLED: i32 = 4;
/// Roster units this close to the target count as arrived.
const ARRIVED: i32 = 3;
/// Roster units this close to a home city under attack are released to it.
const RELEASE: i32 = 4;

impl Operation {
    pub fn new(kind: OperationKind, start_turn: u32, turns: u32, radius: i32) -> Self {
        Self {
            phase: match kind {
                OperationKind::Strike { .. } => Phase::Assemble,
                OperationKind::Pin { .. } => Phase::Hold,
            },
            kind,
            start_turn,
            until: start_turn.saturating_add(turns),
            radius,
            roster: Vec::new(),
            started: false,
            assemble_by: start_turn.saturating_add(10),
            blocked_turns: 0,
            at_war_seen: false,
            outcome: OperationOutcome::Running,
            log: OperationLog::default(),
        }
    }

    /// The plan the operation holds.
    pub fn plan(&self) -> PlanKind {
        match &self.kind {
            OperationKind::Strike {
                against, target, ..
            } => PlanKind::Campaign {
                against: *against,
                target: *target,
            },
            OperationKind::Pin { plan } => plan.clone(),
        }
    }

    pub fn running(&self) -> bool {
        self.outcome == OperationOutcome::Running
    }
}

impl Planner<'_> {
    /// The running operation, when gene `operations` honours it.
    pub(crate) fn operation(&self) -> Option<&Operation> {
        self.campaigns.operation.as_ref().filter(|op| {
            // Gene `runaway_coalition` bits 2 and 8 commit second-front strikes (`runaway.rs`).
            // Gene `short_objective` holds its objective as a pinned operation.
            (self.decisions.operations > 0
                || self.decisions.runaway_coalition & 26 != 0
                || self.decisions.short_objective > 0)
                && op.running()
        })
    }

    /// The running strike's (team, city), if any.
    pub(crate) fn operation_strike(&self) -> Option<(TeamId, CityId)> {
        match self.operation().map(|op| &op.kind) {
            Some(OperationKind::Strike {
                against, target, ..
            }) => Some((*against, *target)),
            _ => None,
        }
    }

    /// A running strike on `team` refuses Peace (`diplomacy::wants_peace_why`).
    pub(crate) fn operation_refuses_peace(&self, team: TeamId) -> bool {
        self.operation_strike()
            .is_some_and(|(against, _)| against == team)
    }

    /// A running bought strike keeps the treasury for its own purchases (`gold_reserve`).
    pub(crate) fn operation_reserve(&self) -> i32 {
        match self.operation().map(|op| &op.kind) {
            Some(OperationKind::Strike { buy: true, .. }) => self.view().economy.gold.max(0),
            _ => 0,
        }
    }

    pub(crate) fn end_operation(&mut self, outcome: OperationOutcome) {
        let turn = self.view().turn;
        if let Some(op) = self.campaigns.operation.as_mut() {
            op.outcome = outcome;
            op.log.end_turn = Some(turn);
        }
    }

    /// Before the plan is chosen: check the budget and the abort conditions. Only these end
    /// an operation; the plan layer cannot.
    pub(crate) fn review_operation(&mut self) {
        let Some(op) = self.operation().cloned() else {
            return;
        };
        let v = self.view();
        if v.turn >= op.until {
            self.end_operation(OperationOutcome::Expired);
            return;
        }
        if let OperationKind::Strike {
            against, target, ..
        } = op.kind
        {
            let Some(city) = v.cities.iter().find(|c| c.id == target) else {
                self.end_operation(OperationOutcome::TargetGone);
                return;
            };
            if city.owner == v.me || v.same_team(v.me, city.owner) {
                self.end_operation(OperationOutcome::Captured);
                return;
            }
            if v.team_of(city.owner) != against {
                self.end_operation(OperationOutcome::TargetTaken);
                return;
            }
            if op.at_war_seen && !v.teams_at_war(v.team, against) {
                self.end_operation(OperationOutcome::Peace);
                return;
            }
            if op.blocked_turns >= 5 {
                self.end_operation(OperationOutcome::DeclarationBlocked);
                return;
            }
            // The campaign the strike runs is its own: the reconciler leaves it alone, and a
            // capture or a lost city is judged above.
            if v.teams_at_war(v.team, against) && op.phase != Phase::Assemble {
                self.campaign_withdraw.retain(|id| !op.roster.contains(id));
            }
        }
    }

    /// The plan decision a running operation imposes (`plan.rs::choose_plan`): its own plan,
    /// injected when the plan layer no longer lists it. A pinned plan whose objective is gone
    /// ends here (Peace with a team we are no longer at war with, or a Snipe, Field strike,
    /// Contain or Hold that is off the list).
    pub(crate) fn operation_candidate(
        &mut self,
        candidates: &[crate::plan::Candidate],
    ) -> Option<crate::plan::Candidate> {
        let op = self.operation()?.clone();
        let kind = op.plan();
        if let Some(c) = candidates.iter().find(|c| c.kind.same_objective(&kind)) {
            return Some(c.clone());
        }
        let v = self.view();
        let injectable = match &kind {
            PlanKind::Campaign { .. } | PlanKind::Economy => true,
            PlanKind::Peace { with } => v.teams_at_war(v.team, *with),
            // A pinned Hold (a told scenario hold or the short-horizon objective) stands while
            // the city is ours; the plan layer lists one only for Hard+Search.
            PlanKind::Hold { city } => self.ctx.own_cities.iter().any(|c| c.id == *city),
            _ => false,
        };
        if !injectable {
            self.end_operation(OperationOutcome::PlanEnded);
            return None;
        }
        Some(crate::plan::Candidate {
            kind,
            value: 0,
            terms: vec![("committed operation".into(), 0)],
            reserve: 0,
            breach: false,
        })
    }

    /// After the plan is chosen: whether the operation took (its first turn).
    pub(crate) fn note_operation_took(&mut self) {
        let Some(op) = self.operation() else {
            return;
        };
        if op.start_turn != self.view().turn {
            return;
        }
        let routed = self
            .routed_plan()
            .is_some_and(|p| p.kind.same_objective(&op.plan()));
        let took = routed
            && match op.kind {
                OperationKind::Strike { .. } => !op.roster.is_empty(),
                OperationKind::Pin { .. } => true,
            };
        if let Some(op) = self.campaigns.operation.as_mut() {
            op.log.took = took;
        }
    }

    fn roster_unit(&self, u: &UnitView) -> bool {
        u.military && u.kind != UnitKind::Scout && !data::is_aircraft(u.kind)
    }

    /// Fill the roster on the first pass and add units produced or bought for it since.
    pub(crate) fn update_roster(&mut self) {
        let Some(op) = self.operation().cloned() else {
            return;
        };
        let OperationKind::Strike { target, .. } = op.kind else {
            return;
        };
        let v = self.view();
        let Some(city) = v.cities.iter().find(|c| c.id == target) else {
            return;
        };
        let pos = city.pos;
        let radius = op.radius;
        let mut roster: Vec<UnitId> = op
            .roster
            .iter()
            .copied()
            .filter(|id| self.ctx.own_units.iter().any(|u| u.id == *id))
            .collect();
        if !op.started {
            // Every unit in reach, except the last defender of a city with an enemy army
            // within 3 tiles.
            // Gene `runaway_coalition` bit 16 keeps a home guard where the enemy's army is within
            // 8 tiles (a member must not empty a city the threat takes next turn).
            let guard = if self.decisions.runaway_coalition & 16 != 0 {
                8
            } else {
                3
            };
            let threatened: Vec<Hex> = self
                .ctx
                .own_cities
                .iter()
                .filter(|c| {
                    self.ctx
                        .threat_units
                        .iter()
                        .any(|e| e.military && e.pos.distance(c.pos) <= guard)
                })
                .map(|c| c.pos)
                .collect();
            let mut kept: Vec<Hex> = Vec::new();
            for u in self.ctx.own_units.iter().filter(|u| self.roster_unit(u)) {
                if u.pos.distance(pos) > radius {
                    continue;
                }
                if threatened.contains(&u.pos) && !kept.contains(&u.pos) {
                    kept.push(u.pos);
                    continue;
                }
                roster.push(u.id);
            }
        } else {
            // New units: produced or bought since the start, in a city within the radius.
            for u in self.ctx.own_units.iter().filter(|u| self.roster_unit(u)) {
                if !roster.contains(&u.id)
                    && u.created_turn.is_some_and(|t| t >= op.start_turn)
                    && u.pos.distance(pos) <= radius
                    && self.ctx.own_cities.iter().any(|c| c.pos == u.pos)
                {
                    roster.push(u.id);
                }
            }
        }
        roster.sort_unstable();
        roster.dedup();
        let farthest = roster
            .iter()
            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id))
            .map(|u| u.pos.distance(pos))
            .max()
            .unwrap_or(0);
        let n = roster.len() as u32;
        let op = self.campaigns.operation.as_mut().expect("operation");
        if !op.started {
            op.started = true;
            op.log.roster_start = n;
            // About two tiles a turn to the staging ring, at least 3 turns and at most 10.
            op.assemble_by = op.start_turn
                + ((farthest - ASSEMBLED).max(0) as u32)
                    .div_ceil(2)
                    .clamp(3, 10);
        }
        op.log.roster_max = op.log.roster_max.max(n);
        op.roster = roster;
    }

    /// Buy for a bought strike, before any other purchase: the strongest land unit in each of
    /// the two own cities nearest the target (one each a turn).
    pub(crate) fn operation_buys(&mut self) {
        let Some(op) = self.operation().cloned() else {
            return;
        };
        let OperationKind::Strike {
            target, buy: true, ..
        } = op.kind
        else {
            return;
        };
        let v = self.view();
        let Some(city) = v.cities.iter().find(|c| c.id == target) else {
            return;
        };
        let mut homes: Vec<_> = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| c.pos.distance(city.pos) <= op.radius)
            .filter(|c| c.detail.as_ref().is_some_and(|d| d.resistance == 0))
            .copied()
            .collect();
        homes.sort_by_key(|c| (c.pos.distance(city.pos), c.id));
        let mut bought = 0;
        let mut spent = 0;
        for home in homes.into_iter().take(2) {
            if self
                .orders
                .iter()
                .any(|o| matches!(o, Order::BuyItem { city, .. } if *city == home.id))
            {
                continue;
            }
            let d = home.detail.as_ref().expect("own city detail");
            let best = d
                .options
                .iter()
                .filter_map(|o| {
                    let ProductionItem::Unit { unit } = o.item else {
                        return None;
                    };
                    let cost = o.buy_cost.filter(|c| *c > 0 && *c <= self.gold_left)?;
                    let stats = v.rule_experiments.unit_stats_for(self.faction, unit);
                    (o.buy_block.is_none()
                        && stats.military
                        && unit != UnitKind::Scout
                        && !data::is_aircraft(unit))
                    .then_some((stats.strength.max(stats.ranged), cost, o.item))
                })
                .max_by_key(|(power, cost, _)| (*power, std::cmp::Reverse(*cost)));
            let Some((_, cost, item)) = best else {
                continue;
            };
            self.gold_left -= cost;
            self.orders.push(Order::BuyItem {
                city: home.id,
                item,
            });
            bought += 1;
            spent += cost;
        }
        if let Some(op) = self.campaigns.operation.as_mut() {
            op.log.bought += bought;
            op.log.gold_spent += spent;
        }
    }

    /// The operation's military pass, before every other one: order the roster.
    pub(crate) fn execute_operation(&mut self) {
        let Some(op) = self.operation().cloned() else {
            return;
        };
        let OperationKind::Strike {
            against, target, ..
        } = op.kind
        else {
            return;
        };
        let v = self.view();
        let Some(city) = v.cities.iter().find(|c| c.id == target) else {
            return;
        };
        let pos = city.pos;
        let units: Vec<&UnitView> = op
            .roster
            .iter()
            .filter_map(|id| self.ctx.own_units.iter().find(|u| u.id == *id).copied())
            .collect();
        if units.is_empty() && op.started {
            let can_buy =
                matches!(op.kind, OperationKind::Strike { buy: true, .. }) && self.gold_left > 0;
            if !can_buy {
                self.end_operation(OperationOutcome::RosterLost);
                return;
            }
        }
        // A home city under attack takes the roster units near it this turn (logged).
        let attacked: Vec<(CityId, Hex)> = self
            .ctx
            .own_cities
            .iter()
            .filter(|c| {
                self.ctx.threat_units.iter().any(|e| {
                    e.military
                        && (e.pos.distance(c.pos) <= 1
                            || c.hp < c.max_hp && e.pos.distance(c.pos) <= 2)
                })
            })
            .map(|c| (c.id, c.pos))
            .collect();
        let mut released = Vec::new();
        let mut events = Vec::new();
        for u in &units {
            if let Some((id, _)) = attacked
                .iter()
                .find(|(_, at)| at.distance(u.pos) <= RELEASE)
            {
                released.push(u.id);
                if !events.contains(id) {
                    events.push(*id);
                }
            }
        }
        let active: Vec<&UnitView> = units
            .iter()
            .copied()
            .filter(|u| !released.contains(&u.id))
            .collect();
        let at_war = v.teams_at_war(v.team, against);
        let assembled = active
            .iter()
            .filter(|u| u.pos.distance(pos) <= ASSEMBLED)
            .count();
        let arrived = units
            .iter()
            .filter(|u| u.pos.distance(pos) <= ARRIVED)
            .count() as u32;
        let mut phase = op.phase;
        if phase == Phase::Assemble
            && (v.turn >= op.assemble_by || assembled * 3 >= active.len().max(1) * 2)
        {
            phase = Phase::Strike;
        }
        if phase != Phase::Assemble && city.hp == 0 {
            phase = Phase::Storm;
        }
        let mut blocked = op.blocked_turns;
        let mut declared = at_war;
        if phase != Phase::Assemble && !at_war {
            if legal_declaration(v, against) {
                if !self
                    .orders
                    .iter()
                    .any(|o| matches!(o, Order::DeclareWar { against: a, .. } if *a == against))
                {
                    self.orders.push(Order::DeclareWar {
                        against,
                        breach: false,
                    });
                }
                declared = true;
                blocked = 0;
            } else {
                blocked += 1;
            }
        }
        if phase != Phase::Assemble && declared {
            // The strike runs as this seat's campaign, its committed force the roster.
            let keep = self
                .campaigns
                .active
                .as_ref()
                .is_some_and(|c| c.target == target);
            if keep {
                let c = self.campaigns.active.as_mut().expect("campaign");
                c.committed = op.roster.clone();
                c.expected_value = c.expected_value.max(1);
            } else {
                self.campaigns.active = Some(Campaign {
                    target,
                    start_turn: v.turn,
                    committed: op.roster.clone(),
                    deadline: None,
                    expected_value: 1,
                    target_was_unwalled: !city.walls,
                });
            }
            self.campaign_withdraw.retain(|id| !op.roster.contains(id));
        }
        if phase != Phase::Assemble && declared {
            // A coordinated assault first (it storms a city at 0 HP), then each unit strikes.
            self.operation_assault(&active, pos);
            for u in self.free(&active) {
                if u.pos.distance(pos) <= ARRIVED {
                    self.strike(u, pos, false);
                } else {
                    self.operation_approach(u, pos);
                }
            }
        } else {
            for u in self.free(&active) {
                if u.pos.distance(pos) <= ASSEMBLED {
                    self.hold_or_upgrade(u);
                } else {
                    self.operation_approach(u, pos);
                }
            }
        }
        let ids: Vec<UnitId> = active.iter().map(|u| u.id).collect();
        let attacks = self
            .orders
            .iter()
            .filter(|o| match o {
                Order::RangedAttack { unit, target: t } => ids.contains(unit) && *t == pos,
                Order::MoveUnit { unit, to } => ids.contains(unit) && *to == pos,
                _ => false,
            })
            .count() as u32;
        let turn = v.turn;
        let op = self.campaigns.operation.as_mut().expect("operation");
        op.phase = phase;
        op.blocked_turns = blocked;
        op.at_war_seen |= at_war && phase != Phase::Assemble;
        op.log.arrived = op.log.arrived.max(arrived);
        op.log.attacks += attacks;
        op.log.overrides += released.len() as u32;
        op.log
            .override_events
            .extend(events.into_iter().map(|c| (turn, c)));
        if phase != Phase::Assemble && declared && op.log.strike_turn.is_none() {
            op.log.strike_turn = Some(turn);
        }
        if phase == Phase::Storm && op.log.storm_turn.is_none() {
            op.log.storm_turn = Some(turn);
        }
    }

    /// Close on the target: a safe staging tile on the ring of 3, else the ring of 4, else
    /// straight at it.
    fn operation_approach(&mut self, u: &UnitView, pos: Hex) {
        let step = self
            .campaign_staging_spot(u, pos, 3)
            .or_else(|| self.campaign_staging_spot(u, pos, 4))
            .or_else(|| self.campaign_approach(u, pos));
        // No safe way in: the nearest open tile on the ring of 3, safe or not.
        let step = step.or_else(|| {
            pos.ring(3)
                .into_iter()
                .filter(|h| self.ctx.move_cost(*h).is_some() && self.free_tile_for(u, *h))
                .min_by_key(|h| (h.distance(u.pos), *h))
        });
        match step {
            Some(to) => self.go_to(u, to),
            None => self.hold_or_upgrade(u),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::diplomacy::{DiplomaticAction, Relation, RelationState};
    use legatum_core::{AiDifficulty, Faction, PlayerView};

    /// Sparta at Peace with a legal declaration on the enemy, whose city is 15 tiles off, and
    /// three Spearmen near home.
    fn sparta_view() -> PlayerView {
        let mut v = ViewBuilder::new(40, 20)
            .faction(Faction::Sparta)
            .city(1, ME, off(3, 5), 3)
            .city(2, ENEMY, off(18, 5), 3)
            .unit(1, ME, UnitKind::Spearman, off(5, 5))
            .unit(2, ME, UnitKind::Spearman, off(5, 6))
            .unit(3, ME, UnitKind::Spearman, off(5, 7))
            .build();
        v.diplomacy.legacy = false;
        let against = v.team_of(ENEMY);
        v.diplomacy.relations.push(Relation {
            a: v.team,
            b: against,
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        v.diplomacy.actions.push(DiplomaticAction {
            new_wars: Vec::new(),
            calls: Vec::new(),
            also_agree: Vec::new(),
            order: Order::DeclareWar {
                against,
                breach: false,
            },
            effective_turn: v.turn,
            cost: 0,
            blocked_reason: None,
            upkeep: 0,
            betrayal: false,
        });
        v
    }

    fn planner(v: &PlayerView, operations: i32, op: Operation) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
        p.difficulty = AiDifficulty::Hard;
        p.plan_routing = true;
        p.decisions.operations = operations;
        // Bit 2 of `runaway_coalition` also honours operations; test the gene alone.
        p.decisions.runaway_coalition = 0;
        p.campaigns.operation = Some(op);
        p.review_operation();
        p.update_roster();
        p.reconcile_campaign();
        p.choose_plan();
        p.note_operation_took();
        p
    }

    fn strike(v: &PlayerView) -> Operation {
        Operation::new(
            OperationKind::Strike {
                against: v.team_of(ENEMY),
                target: CityId(2),
                buy: false,
            },
            v.turn,
            30,
            20,
        )
    }

    #[test]
    fn a_strike_the_plan_layer_does_not_offer_takes_and_marches_its_roster() {
        let v = sparta_view();
        let mut p = planner(&v, 1, strike(&v));
        assert!(matches!(
            p.routed_plan().map(|plan| &plan.kind),
            Some(PlanKind::Campaign { target, .. }) if *target == CityId(2)
        ));
        let op = p.campaigns.operation.clone().unwrap();
        assert!(op.log.took);
        assert_eq!(op.roster, vec![UnitId(1), UnitId(2), UnitId(3)]);
        p.plan_military();
        let target = off(18, 5);
        for u in p.ctx.own_units.clone() {
            let to = p.orders.iter().find_map(|o| match o {
                Order::MoveUnit { unit, to } if *unit == u.id => Some(*to),
                _ => None,
            });
            let to = to.unwrap_or_else(|| panic!("unit {:?} not marched", u.id));
            assert!(to.distance(target) < u.pos.distance(target));
        }
        // Still assembling: no declaration yet.
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::DeclareWar { .. }))
        );
        assert_eq!(
            p.campaigns.operation.as_ref().unwrap().phase,
            Phase::Assemble
        );
    }

    #[test]
    fn without_the_gene_an_operation_is_ignored() {
        let v = sparta_view();
        let p = planner(&v, 0, strike(&v));
        assert!(!matches!(
            p.routed_plan().map(|plan| &plan.kind),
            Some(PlanKind::Campaign { .. })
        ));
        assert!(!p.campaigns.operation.unwrap().log.took);
    }

    #[test]
    fn a_pinned_economy_outranks_spartas_campaign_rule() {
        let v = sparta_view();
        let pin = |v: &PlayerView, operations| {
            let mut p = Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1);
            p.difficulty = AiDifficulty::Hard;
            p.plan_routing = true;
            p.decisions.sparta_force_campaign = 1;
            p.decisions.sparta_valued_expedition = 0;
            p.decisions.operations = operations;
            p.decisions.runaway_coalition = 0;
            p.campaigns.operation = Some(Operation::new(
                OperationKind::Pin {
                    plan: PlanKind::Economy,
                },
                v.turn,
                30,
                12,
            ));
            p.choose_plan();
            p.note_operation_took();
            p.routed_plan().map(|plan| plan.kind.clone())
        };
        assert!(matches!(pin(&v, 0), Some(PlanKind::Campaign { .. })));
        assert_eq!(pin(&v, 1), Some(PlanKind::Economy));
    }

    #[test]
    fn a_home_city_under_attack_takes_the_roster_units_near_it() {
        let mut v = sparta_view();
        let raider = ViewBuilder::new(40, 20)
            .unit(9, ENEMY, UnitKind::Spearman, off(4, 5))
            .build()
            .units
            .into_iter()
            .find(|u| u.id == UnitId(9))
            .unwrap();
        v.units.push(raider);
        // At War: a foreign unit at Peace is no attack.
        let against = v.team_of(ENEMY);
        for r in &mut v.diplomacy.relations {
            if r.between(v.team, against) {
                r.state = RelationState::War;
            }
        }
        let mut p = planner(&v, 1, strike(&v));
        p.plan_military();
        let op = p.campaigns.operation.unwrap();
        assert_eq!(op.log.overrides, op.roster.len() as u32);
        assert_eq!(op.log.override_events, vec![(v.turn, CityId(1))]);
    }
}

//! Gene `short_objective`: a short-horizon objective step (2026-11-10, Lab cards Hold Biel and
//! Ride to Dyrrachium, `docs/sim-results/ai-quality/ai-objective-1110/`).
//!
//! Told a card's goal, Hard won Hold Biel 49/50 and Dyrrachium 50/50 against 34 and 2: its
//! executors were fine, the plan layer pointed them at the wrong thing (an offensive Campaign
//! while Biel was besieged; walled Nicomedia for its deposit while open Dyrrachium was in
//! reach). This step finds such an objective from the seat's own view, by valuation:
//!
//! - **Nominate** every enemy city the forces in reach take within `WINDOW` turns by the
//!   campaign forecast (`plan.rs::campaign_forecast`), and every own city a visible enemy army
//!   is forecast to take within `WINDOW` turns (the same siege forecast, played from the
//!   attacker's side).
//! - **Price** in Production, the currency of the tactical layer: a capture is worth the city
//!   (400 + 80 per population) at its capture odds (forecast turns under gene `capture_horizon`,
//!   200 unless set) less the forecast's expected losses, and less the expected loss of every
//!   home city forecast to fall (its odds times its worth): an offence pays for the home it
//!   leaves. Not the plan layer's win chance: a leader's chance saturates (Hold Biel's Swiss sit
//!   at 99.3% to 99.99%, so losing Biel priced at 13 points and every capture at 4 to 35), and
//!   its supply term priced Bikaner on Two Cities at twice a certain win.
//!   Only cities of teams already at war, or of the team the routed plan is about to declare
//!   on, are nominated: the step picks targets but starts no war of its own.
//! - **Commit** to the best through a pinned operation (`OperationKind::Pin`), the mechanism
//!   the told arm uses: a Campaign on the city, or, when home cities are forecast to fall and
//!   no offence is worth that risk, a Hold on the most endangered one (`execute_hold`).
//! - **Change course** when the pinned capture's odds stay under `ODDS_FLOOR` for two turns in
//!   a row (that city is left alone for `COOLDOWN` turns), when another objective beats it by
//!   `switch_margin`, or when it is met.
//!
//! Only the seat's `PlayerView` is read; nothing here knows a scenario's win condition.

use crate::operation::{Operation, OperationKind, OperationOutcome};
use crate::plan::{PlanKind, capture_odds};
use crate::scripted::Planner;
use legatum_core::view::{CityView, UnitView};
use legatum_core::{AiDifficulty, CityId, Hex, TeamId, UnitKind};
use serde::{Deserialize, Serialize};

/// Turns within which a capture or a loss counts as a short-horizon objective.
pub(crate) const WINDOW: i32 = 5;
/// Capture odds (percent) under which a pinned capture counts as failing.
pub(crate) const ODDS_FLOOR: i32 = 15;
/// Turns a failed capture's city is left alone.
const COOLDOWN: u32 = 5;
/// Enemy units this close to an own city count as its attackers.
const ATTACK_REACH: i32 = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortMemory {
    /// The objective held: its plan, its value when chosen, and turns in a row its odds were
    /// under the floor.
    pub current: Option<(PlanKind, i32, u32)>,
    /// Captures given up as failing, with the turn.
    pub dropped: Vec<(CityId, u32)>,
}

impl ShortMemory {
    pub fn is_empty(&self) -> bool {
        self.current.is_none() && self.dropped.is_empty()
    }
}

/// A city's worth in Production, the stake the tactical layer uses for a capture or a loss
/// (`tactics.rs::enemy_threat`, `breach.rs`).
fn city_worth(c: &CityView) -> i32 {
    400 + 80 * c.population
}

struct Nomination {
    plan: PlanKind,
    value: i32,
}

impl Planner<'_> {
    /// The pinned operation this step owns, if the running one is it.
    fn short_pin_running(&self) -> bool {
        let Some((plan, _, _)) = &self.campaigns.short.current else {
            return false;
        };
        self.campaigns.operation.as_ref().is_some_and(|op| {
            op.running() && matches!(&op.kind, OperationKind::Pin { plan: p } if p == plan)
        })
    }

    /// Own city `city`'s fall to the visible army at its door: (odds percent, attacking team),
    /// when the siege forecast played from the attacker's side takes it within `WINDOW` turns.
    fn home_fall(&self, city: &CityView) -> Option<(i32, TeamId)> {
        let v = self.view();
        let attackers: Vec<&UnitView> = self
            .ctx
            .enemy_units
            .iter()
            .copied()
            .filter(|e| {
                e.military
                    && e.kind != UnitKind::Scout
                    && !e.owner.is_barbarian()
                    && v.at_war(v.me, e.owner)
                    && e.pos.distance(city.pos) <= ATTACK_REACH
            })
            .collect();
        // The team with the most units at the door.
        let mut teams: Vec<TeamId> = attackers.iter().map(|e| v.team_of(e.owner)).collect();
        teams.sort_unstable();
        let team = teams.iter().copied().max_by_key(|t| {
            (
                teams.iter().filter(|x| *x == t).count(),
                std::cmp::Reverse(*t),
            )
        })?;
        let owner = attackers
            .iter()
            .find(|e| v.team_of(e.owner) == team)
            .map(|e| e.owner)?;
        let ring: Vec<Hex> = city
            .pos
            .within(1)
            .into_iter()
            .filter(|h| *h != city.pos && self.ctx.move_cost(*h).is_some())
            .collect();
        let force: Vec<(&UnitView, Hex, i32)> = attackers
            .iter()
            .copied()
            .filter(|e| v.team_of(e.owner) == team)
            .filter_map(|e| {
                let to = ring
                    .iter()
                    .copied()
                    .min_by_key(|h| (h.distance(e.pos), *h))?;
                let moves = e.max_moves.max(1);
                let t = (e.pos.distance(to) + moves - 1) / moves;
                Some((
                    e,
                    if e.pos.distance(city.pos) <= 1 {
                        e.pos
                    } else {
                        to
                    },
                    t,
                ))
            })
            .collect();
        if force.is_empty() {
            return None;
        }
        let mut sim = v.clone();
        sim.me = owner;
        sim.team = team;
        let siege = self.forecast_siege(&sim, city, &force)?;
        (siege.turns <= WINDOW).then(|| (capture_odds(siege.turns, 0), team))
    }

    /// The capture odds and value of a Campaign on `city` now, priced as a nomination: the
    /// city's worth at the capture odds, less the forecast's expected Production lost. Odds come
    /// from the forecast turns under gene `capture_horizon` (200 unless the gene has its own
    /// value).
    fn short_capture(&mut self, city: &CityView) -> Option<(i32, i32)> {
        let committed = self
            .campaigns
            .active
            .as_ref()
            .filter(|c| c.target == city.id)
            .map(|c| c.committed.clone());
        let f = self
            .campaign_forecast(city, committed.as_deref())
            .or_else(|| self.campaign_forecast(city, None))?;
        let gene = self.decisions.capture_horizon;
        if gene == 0 {
            self.decisions.capture_horizon = 200;
        }
        let turns = self.capture_turns(city, &f);
        self.decisions.capture_horizon = gene;
        if turns > WINDOW {
            return None;
        }
        let odds = capture_odds(turns, f.march);
        Some((odds, odds * city_worth(city) / 100 - f.losses))
    }

    /// Gene `short_objective`, run before the plan is chosen (see the module notes).
    pub(crate) fn choose_short_objective(&mut self) {
        if self.decisions.short_objective == 0
            || self.difficulty != AiDifficulty::Hard
            || self.opening.is_some()
        {
            return;
        }
        let v = self.view();
        let turn = v.turn;
        self.campaigns
            .short
            .dropped
            .retain(|(_, t)| turn < t + COOLDOWN);
        // Home cities forecast to fall, with their expected loss.
        let mut endangered: Vec<(i32, CityId)> = Vec::new();
        let mut attackers: Vec<TeamId> = Vec::new();
        let cities: Vec<&CityView> = self.ctx.own_cities.clone();
        for c in cities {
            if let Some((odds, team)) = self.home_fall(c) {
                let loss = odds * city_worth(c) / 100;
                endangered.push((loss, c.id));
                attackers.push(team);
            }
        }
        let home_loss: i32 = endangered.iter().map(|(l, _)| *l).sum();
        // Captures in reach within the window.
        let army: Vec<Hex> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .map(|u| u.pos)
            .collect();
        // The war the plan layer chose to start (its routed plan's enemy, still at Peace): the
        // step picks the city in it, but starts no war of its own.
        let planned_war = self
            .routed_plan()
            .and_then(|p| p.kind.war_target())
            .filter(|t| {
                !v.teams_at_war(v.team, *t) && crate::opportunity::legal_declaration(v, *t)
            });
        let targets: Vec<CityView> = v
            .cities
            .iter()
            .filter(|c| {
                !c.owner.is_barbarian()
                    && !v.same_team(v.me, c.owner)
                    && (v.at_war(v.me, c.owner)
                        || (planned_war == Some(v.team_of(c.owner)) && !self.stale_city(c)))
                    && !self
                        .campaigns
                        .short
                        .dropped
                        .iter()
                        .any(|(id, _)| *id == c.id)
                    && army.iter().any(|p| p.distance(c.pos) <= 3 * WINDOW)
            })
            .cloned()
            .collect();
        let mut nominations: Vec<Nomination> = Vec::new();
        let mut odds_of: Vec<(CityId, i32)> = Vec::new();
        for city in &targets {
            if let Some((odds, value)) = self.short_capture(city) {
                odds_of.push((city.id, odds));
                if odds >= ODDS_FLOOR {
                    nominations.push(Nomination {
                        plan: PlanKind::Campaign {
                            against: v.team_of(city.owner),
                            target: city.id,
                        },
                        value: value - home_loss,
                    });
                }
            }
        }
        // A Hold is worth its home loss only against a plan that takes the army away: a
        // Campaign, or a war on a team that isn't at our door. The quiet plan, Peace or a
        // Contain war on the attacker already fight at home (Athens two pikemen T112: a Hold
        // pinned over Contain against Sparta left the Pikemen alive, -7 pp in the war suite).
        let leaves_home = match self.routed_plan().map(|p| &p.kind) {
            None | Some(PlanKind::Campaign { .. }) | Some(PlanKind::Hold { .. }) => true,
            Some(PlanKind::Contain { against }) => !attackers.contains(against),
            Some(_) => false,
        };
        if leaves_home
            && let Some((_, city)) = endangered
                .iter()
                .max_by_key(|(l, id)| (*l, std::cmp::Reverse(*id)))
        {
            nominations.push(Nomination {
                plan: PlanKind::Hold { city: *city },
                value: 0,
            });
        }
        nominations.retain(|n| n.value > 0 || matches!(n.plan, PlanKind::Hold { .. }));
        nominations.sort_by_key(|n| std::cmp::Reverse(n.value));
        let best = nominations.first();
        // The held objective: still standing, and its capture not failing two turns in a row.
        let mut current = self.campaigns.short.current.clone();
        if let Some((plan, value, low)) = current.clone() {
            let keep = match &plan {
                PlanKind::Campaign { target, .. } => {
                    match v
                        .cities
                        .iter()
                        .find(|c| c.id == *target && !v.same_team(v.me, c.owner))
                    {
                        None => None,
                        Some(_) => {
                            let odds = odds_of
                                .iter()
                                .find(|(id, _)| id == target)
                                .map_or(0, |(_, o)| *o);
                            let low = if odds < ODDS_FLOOR { low + 1 } else { 0 };
                            if low >= 2 {
                                self.campaigns.short.dropped.push((*target, turn));
                                None
                            } else {
                                let now = nominations
                                    .iter()
                                    .find(|n| n.plan == plan)
                                    .map_or(value, |n| n.value);
                                Some((plan.clone(), now, low))
                            }
                        }
                    }
                }
                PlanKind::Hold { city } => endangered
                    .iter()
                    .any(|(_, id)| id == city)
                    .then(|| (plan.clone(), value, 0)),
                _ => None,
            };
            current = keep;
        }
        let chosen = match (current, best) {
            (Some(cur), Some(b))
                if b.plan != cur.0 && b.value > cur.1 + self.decisions.switch_margin =>
            {
                Some((b.plan.clone(), b.value, 0))
            }
            // An offence beats a held Hold only by its value; a held Hold gives way when an
            // offence worth its home risk appears.
            (Some(cur), Some(b))
                if matches!(cur.0, PlanKind::Hold { .. })
                    && matches!(b.plan, PlanKind::Campaign { .. })
                    && b.value > 0 =>
            {
                Some((b.plan.clone(), b.value, 0))
            }
            (Some(cur), _) => Some(cur),
            (None, Some(b)) => Some((b.plan.clone(), b.value, 0)),
            (None, None) => None,
        };
        let ours = self.short_pin_running();
        match chosen {
            Some((plan, value, low)) => {
                // A running operation that is not ours (a second front) keeps its pin.
                if !ours && self.operation().is_some() {
                    self.campaigns.short.current = None;
                    return;
                }
                self.campaigns.operation = Some(Operation::new(
                    OperationKind::Pin { plan: plan.clone() },
                    turn,
                    WINDOW as u32,
                    0,
                ));
                self.campaigns.short.current = Some((plan, value, low));
            }
            None => {
                if ours {
                    self.end_operation(OperationOutcome::PlanEnded);
                }
                self.campaigns.short.current = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fixture::{ENEMY, ME, ViewBuilder};
    use crate::operation::OperationKind;
    use crate::plan::PlanKind;
    use legatum_core::{AiDifficulty, CityId, Hex, UnitKind};

    #[test]
    fn an_open_city_the_army_takes_within_the_window_is_pinned_only_with_the_gene() {
        let (home, theirs) = (Hex::new(4, 6), Hex::new(10, 6));
        let v = ViewBuilder::new(24, 16)
            .city(1, ME, home, 3)
            .city(2, ENEMY, theirs, 1)
            .unit(1, ME, UnitKind::Soldier, home)
            .unit(2, ME, UnitKind::Soldier, Hex::new(9, 6))
            .unit(3, ME, UnitKind::Soldier, Hex::new(9, 7))
            .unit(4, ME, UnitKind::Soldier, Hex::new(11, 6))
            .build();
        let pinned = |gene: i32| {
            let mut p = crate::fixture::planner(&v, 1);
            p.difficulty = AiDifficulty::Hard;
            p.decisions.short_objective = gene;
            p.prepare_hard_defense();
            p.choose_short_objective();
            p.campaigns.operation.map(|op| op.kind)
        };
        assert_eq!(pinned(0), None, "gene off: nothing is pinned");
        assert!(
            matches!(
                pinned(1),
                Some(OperationKind::Pin {
                    plan: PlanKind::Campaign {
                        target: CityId(2),
                        ..
                    }
                })
            ),
            "{:?}",
            pinned(1)
        );
    }
}

//! Read-only observations of VP holds: a team must stay at its VP target for
//! `data::VP_HOLD_TURNS` consecutive turn ends to win by VP. Per game, every hold is
//! recorded (who, when it started, how it ended); per run, `HoldSummary` aggregates holds
//! started, broken and completed, how long winners took from first reaching their target to
//! winning, and comebacks. Never participates in orders, RNG, saves or hashes.

use crate::stats::{GameRecord, Outcome};
use legatum_core::{Event, GameState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldResult {
    /// The team dropped below its target (or was eliminated) and lost the count.
    Broken,
    /// The team held for the full count at the end of the game's final turn (a win, or a draw
    /// when several teams completed with identical standings).
    Completed,
    /// Still running when the game ended some other way (conquest, turn limit, safety cap).
    Unfinished,
}

/// One uninterrupted stay at the VP target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldSpan {
    pub team: u8,
    /// Resolved turn whose end the team first stood at its target (that turn counts as held).
    pub start_turn: u32,
    /// Turn the hold ended: broken on this turn's end, or the final observed turn.
    pub end_turn: u32,
    /// Consecutive turn ends held (the reaching turn included).
    pub turns_held: u32,
    pub result: HoldResult,
}

/// The winning team's path to a VP win by hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldWin {
    pub team: u8,
    /// First turn the winner ever stood at its target (maybe in a hold it later broke).
    pub first_reached_turn: u32,
    /// Turn the final, uninterrupted hold started. `win_turn - final_hold_start_turn` is always
    /// `turns_needed - 1`.
    pub final_hold_start_turn: u32,
    pub win_turn: u32,
    /// The winner's own holds that were broken before the winning one.
    pub earlier_broken_holds: u32,
    /// True unless another team first stood at its target strictly before the winner did.
    pub first_to_reach: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VictoryHoldTelemetry {
    pub turns_needed: u32,
    /// Every hold of the game in start order.
    pub holds: Vec<HoldSpan>,
    pub started: u32,
    pub broken: u32,
    pub completed: u32,
    pub unfinished: u32,
    /// Set only when the game ended with a VP hold win.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_win: Option<HoldWin>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observer {
    /// Team -> start turn of its running hold.
    open: BTreeMap<u8, u32>,
    /// Team -> first turn it ever stood at its target.
    first_reached: BTreeMap<u8, u32>,
    broken_holds: BTreeMap<u8, u32>,
    spans: Vec<HoldSpan>,
    won: Option<(u8, u32)>,
    last_turn: u32,
}

impl Observer {
    pub fn observe_event(&mut self, resolved_turn: u32, event: &Event) {
        match event {
            Event::VictoryHoldStarted { team, .. } => {
                self.open.insert(team.0, resolved_turn);
                self.first_reached.entry(team.0).or_insert(resolved_turn);
            }
            Event::VictoryHoldBroken {
                team, turns_held, ..
            } => {
                let start_turn = self.open.remove(&team.0).unwrap_or(resolved_turn);
                *self.broken_holds.entry(team.0).or_default() += 1;
                self.spans.push(HoldSpan {
                    team: team.0,
                    start_turn,
                    end_turn: resolved_turn,
                    turns_held: *turns_held,
                    result: HoldResult::Broken,
                });
            }
            Event::GameWon { team, .. } => self.won = Some((team.0, resolved_turn)),
            _ => {}
        }
    }

    pub fn observe_resolved_turn(&mut self, resolved_turn: u32) {
        self.last_turn = resolved_turn;
    }

    pub fn finish(&self, state: &GameState) -> VictoryHoldTelemetry {
        let needed = legatum_core::data::VP_HOLD_TURNS;
        let mut holds = self.spans.clone();
        for (&team, &start_turn) in &self.open {
            let held = state
                .vp_holds
                .get(&legatum_core::TeamId(team))
                .copied()
                .unwrap_or(0);
            holds.push(HoldSpan {
                team,
                start_turn,
                end_turn: self.last_turn,
                turns_held: held,
                result: if held >= needed {
                    HoldResult::Completed
                } else {
                    HoldResult::Unfinished
                },
            });
        }
        holds.sort_by_key(|h| (h.start_turn, h.team, h.end_turn));
        let count = |r: HoldResult| holds.iter().filter(|h| h.result == r).count() as u32;
        let hold_win = self.won.and_then(|(team, win_turn)| {
            let final_hold = holds
                .iter()
                .find(|h| h.team == team && h.result == HoldResult::Completed)?;
            let first_reached_turn = *self.first_reached.get(&team)?;
            Some(HoldWin {
                team,
                first_reached_turn,
                final_hold_start_turn: final_hold.start_turn,
                win_turn,
                earlier_broken_holds: self.broken_holds.get(&team).copied().unwrap_or(0),
                first_to_reach: !self
                    .first_reached
                    .iter()
                    .any(|(t, turn)| *t != team && *turn < first_reached_turn),
            })
        });
        VictoryHoldTelemetry {
            turns_needed: needed,
            started: holds.len() as u32,
            broken: count(HoldResult::Broken),
            completed: count(HoldResult::Completed),
            unfinished: count(HoldResult::Unfinished),
            holds,
            hold_win,
        }
    }
}

/// Distribution of a per-game count of turns.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TurnStats {
    pub n: usize,
    pub mean: f64,
    pub median: u32,
    pub min: u32,
    pub max: u32,
}

impl TurnStats {
    pub fn of(mut v: Vec<u32>) -> Option<Self> {
        v.sort_unstable();
        Some(Self {
            n: v.len(),
            mean: v.iter().map(|x| f64::from(*x)).sum::<f64>() / v.len() as f64,
            median: crate::stats::percentile(&v, 50)?,
            min: *v.first()?,
            max: *v.last()?,
        })
    }
}

/// Run-level hold aggregates over first plays with hold telemetry.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct HoldSummary {
    /// Records carrying hold telemetry (panics and older reports do not).
    pub observed_games: usize,
    pub excluded_games_without_holds: usize,
    pub turns_needed: Option<u32>,
    pub games_with_hold: usize,
    pub started: u64,
    pub broken: u64,
    /// Holds that reached the full count: VP wins, plus the rare simultaneous-completion draw.
    pub completed: u64,
    /// Still running when the game ended otherwise (conquest, turn limit, safety cap).
    pub unfinished: u64,
    /// broken / started, counting unfinished holds in the denominator.
    pub break_rate: f64,
    /// broken / (broken + completed): the break rate over holds that resolved.
    pub break_rate_resolved: f64,
    pub hold_wins: usize,
    /// Turns from the winner first standing at its target to winning (includes broken holds).
    pub first_reach_to_win: Option<TurnStats>,
    /// Turns from the final uninterrupted hold's start to winning (always turns_needed - 1).
    pub final_hold_to_win: Option<TurnStats>,
    /// Turn winners first stood at their target.
    pub first_reach_turn: Option<TurnStats>,
    pub win_turn: Option<TurnStats>,
    /// Hold wins by a team that had broken at least one earlier hold of its own.
    pub comebacks: usize,
    /// Hold wins by a team that another team beat to its target.
    pub wins_not_first_to_reach: usize,
}

fn ratio(n: u64, d: u64) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 }
}

impl HoldSummary {
    pub fn of(records: &[GameRecord]) -> Self {
        let mut s = Self::default();
        let (mut reach_to_win, mut final_to_win, mut reach, mut win) =
            (vec![], vec![], vec![], vec![]);
        for r in records {
            let Some(h) = r.telemetry.as_ref().and_then(|t| t.victory_holds.as_ref()) else {
                s.excluded_games_without_holds += 1;
                continue;
            };
            s.observed_games += 1;
            s.turns_needed.get_or_insert(h.turns_needed);
            s.games_with_hold += usize::from(h.started > 0);
            s.started += u64::from(h.started);
            s.broken += u64::from(h.broken);
            s.completed += u64::from(h.completed);
            s.unfinished += u64::from(h.unfinished);
            if r.outcome != Outcome::Vp {
                continue;
            }
            let Some(w) = &h.hold_win else { continue };
            s.hold_wins += 1;
            s.comebacks += usize::from(w.earlier_broken_holds > 0);
            s.wins_not_first_to_reach += usize::from(!w.first_to_reach);
            reach_to_win.push(w.win_turn.saturating_sub(w.first_reached_turn));
            final_to_win.push(w.win_turn.saturating_sub(w.final_hold_start_turn));
            reach.push(w.first_reached_turn);
            win.push(w.win_turn);
        }
        s.break_rate = ratio(s.broken, s.started);
        s.break_rate_resolved = ratio(s.broken, s.broken + s.completed);
        s.first_reach_to_win = TurnStats::of(reach_to_win);
        s.final_hold_to_win = TurnStats::of(final_to_win);
        s.first_reach_turn = TurnStats::of(reach);
        s.win_turn = TurnStats::of(win);
        s
    }
}

#[cfg(test)]
#[path = "hold_telemetry_tests.rs"]
mod tests;

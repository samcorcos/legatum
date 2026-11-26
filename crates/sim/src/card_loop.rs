//! `legatum-sim card-loop <card>`: score an AI measurement position with Hard, search a
//! line of order overrides on top of Hard, and report where the line first departs from Hard.
//!
//! A line is closed-loop: each turn the seat starts from Hard's own orders and the line replaces
//! the orders of the units, cities and empire actions it names. An override that the engine
//! refuses in this position (`validate_orders`) falls back to Hard's orders, so one line can be
//! played on any dice. Everything runs on the same `Session` and `ActiveScenario` as
//! `legatum-sim scenario`; nothing here models a rule.
use clap::Parser;
use legatum_ai::Brain as _;
use legatum_core::scenario::{Outcome, Target, TargetKind};
use legatum_core::{AiDifficulty, GameState, Hex, Order, PlayerId, UnitId};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "legatum-sim card-loop",
    about = "Score a position with Hard and search an order line that beats it"
)]
struct Cli {
    card: PathBuf,
    #[arg(long)]
    out: PathBuf,
    /// Only measure Hard on the fresh seeds (no search).
    #[arg(long)]
    score_only: bool,
    /// Line evaluations the search may spend (each plays every training seed).
    #[arg(long, default_value_t = 300)]
    max_evals: usize,
    /// Training dice seeds (indices 1..=N) the search selects on.
    #[arg(long, default_value_t = 6, value_parser = clap::value_parser!(u32).range(1..))]
    train_seeds: u32,
    /// Held-out dice seeds (indices 10000..) for the final check; never used to select.
    #[arg(long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..))]
    fresh: u32,
    /// Passes over the card's turns.
    #[arg(long, default_value_t = 2)]
    passes: u32,
    /// Battle-search rollouts per battle for the battle-search candidates (0: none).
    #[arg(long, default_value_t = 48)]
    battle_rollouts: u64,
    #[arg(long, default_value_t = 14, value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
    threads: usize,
    /// The one train / held-out card split (the scenario gate's). A held-out or unlisted card is
    /// only scored, never searched.
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/sim-results/scenario-gate/split.json"))]
    split: PathBuf,
}

pub const FRESH_START: u32 = 10_000;

#[derive(Deserialize)]
struct Split {
    train: Vec<String>,
    held_out: Vec<String>,
}

/// Why this card may not be searched under the split, or None for a training card.
pub fn search_refusal(split_json: &str, card: &str) -> Result<Option<String>, String> {
    let split: Split = serde_json::from_str(split_json).map_err(|e| format!("split: {e}"))?;
    Ok(if split.held_out.iter().any(|c| c == card) {
        Some(format!(
            "{card} is held out in the scenario-gate split: it is only scored (--score-only), never searched or tuned on"
        ))
    } else if !split.train.iter().any(|c| c == card) {
        Some(format!(
            "{card} is not in the scenario-gate split: run `python3 tools/sim/scenario_gate_split.py --add` first"
        ))
    } else {
        None
    })
}

/// What the line does with one actor (a unit, a city, research or diplomacy).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Choice {
    /// These orders on this turn only (empty: no new order).
    Exact { orders: Vec<Order>, source: String },
    /// From this turn on: shoot `to` when in range, otherwise move onto it.
    March { to: Hex, name: String },
    /// From this turn on: go to the own city at `to` and stay there (Hard still shoots from it).
    Garrison { to: Hex, name: String },
}

impl Choice {
    fn sticky(&self) -> bool {
        !matches!(self, Choice::Exact { .. })
    }
    fn label(&self) -> String {
        match self {
            Choice::Exact { orders, .. } if orders.is_empty() => "no new order".into(),
            Choice::Exact { source, .. } => source.clone(),
            Choice::March { name, .. } => format!("march on {name} from here on"),
            Choice::Garrison { name, .. } => format!("garrison {name} from here on"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Override {
    pub turn: u32,
    pub actor: String,
    pub choice: Choice,
}

/// Order overrides on top of Hard.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Line {
    pub overrides: Vec<Override>,
}

/// A stable name for who an order moves: the same on every dice seed (ids, not positions).
pub fn actor_key(order: &Order) -> String {
    use legatum_core::orders::Action;
    if let Some(unit) = legatum_core::rules::units::order_unit(order) {
        return format!("unit {}", unit.0);
    }
    match order {
        Order::AirStrike { plane, .. }
        | Order::Rebase { plane, .. }
        | Order::AirRecon { plane, .. }
        | Order::UpgradeAircraft { plane, .. } => format!("unit {}", plane.0),
        Order::Action {
            action: Action::RetireVeteran { unit },
        } => format!("unit {}", unit.0),
        Order::SetProduction { city, .. }
        | Order::SetCityAutomation { city, .. }
        | Order::SetProductionQueue { city, .. }
        | Order::BuyItem { city, .. }
        | Order::HireCompany { city, .. }
        | Order::BuyTile { city, .. }
        | Order::Raze { city } => format!("city {}", city.0),
        Order::SetResearch { .. }
        | Order::SetResearchQueue { .. }
        | Order::SetResearchGoal { .. }
        | Order::SetScienceConversion { .. } => "research".into(),
        o if legatum_core::diplomacy::is_diplomatic(o) => "diplomacy".into(),
        _ => "empire".into(),
    }
}

fn unit_of(actor: &str) -> Option<UnitId> {
    actor
        .strip_prefix("unit ")
        .and_then(|n| n.parse().ok())
        .map(UnitId)
}

/// Orders grouped by actor, in the order each actor first appears.
fn grouped(orders: &[Order]) -> Vec<(String, Vec<Order>)> {
    let mut out: Vec<(String, Vec<Order>)> = Vec::new();
    for order in orders {
        let key = actor_key(order);
        match out.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(order.clone()),
            None => out.push((key, vec![order.clone()])),
        }
    }
    out
}

fn set(groups: &mut Vec<(String, Vec<Order>)>, actor: &str, orders: Vec<Order>) {
    match groups.iter_mut().find(|(k, _)| k == actor) {
        Some((_, list)) => *list = orders,
        None => groups.push((actor.to_string(), orders)),
    }
}

fn get<'a>(groups: &'a [(String, Vec<Order>)], actor: &str) -> &'a [Order] {
    groups
        .iter()
        .find(|(k, _)| k == actor)
        .map_or(&[], |(_, o)| o.as_slice())
}

fn valid(state: &GameState, seat: PlayerId, orders: &[Order]) -> bool {
    legatum_core::validate_orders(state, seat, orders).is_empty()
}

/// The orders a choice gives in this position. None: leave the actor to Hard (a sticky choice
/// with nothing to do, or a unit that no longer exists).
fn choice_orders(
    state: &GameState,
    seat: PlayerId,
    actor: &str,
    choice: &Choice,
    current: &[Order],
) -> Option<Vec<Order>> {
    match choice {
        Choice::Exact { orders, .. } => Some(orders.clone()),
        Choice::March { to, .. } => {
            let unit = unit_of(actor)?;
            if state.units.get(&unit)?.owner != seat {
                return None;
            }
            [
                Order::RangedAttack { unit, target: *to },
                Order::AirStrike {
                    plane: unit,
                    target: *to,
                },
                Order::MoveUnit { unit, to: *to },
            ]
            .into_iter()
            .map(|o| vec![o])
            .find(|o| valid(state, seat, o))
        }
        Choice::Garrison { to, .. } => {
            let unit = unit_of(actor)?;
            let u = state.units.get(&unit)?;
            if u.owner != seat {
                return None;
            }
            if u.pos != *to {
                return Some(vec![Order::MoveUnit { unit, to: *to }]);
            }
            let leaves = current.iter().any(|o| {
                matches!(
                    o,
                    Order::MoveUnit { .. }
                        | Order::MoveUnitPath { .. }
                        | Order::SetAutomation { .. }
                )
            });
            leaves.then(|| vec![Order::SkipUnit { unit }])
        }
    }
}

/// The seat's orders this turn: Hard's with the line's overrides. Returns the orders and how many line orders the engine refused here.
pub fn seat_orders(
    state: &GameState,
    seat: PlayerId,
    hard: &[Order],
    line: &Line,
) -> (Vec<Order>, usize) {
    let turn = state.turn;
    let hard_groups = grouped(hard);
    let mut current = hard_groups.clone();
    let mut refused = 0;
    let mut active: BTreeMap<&str, &Override> = BTreeMap::new();
    for o in &line.overrides {
        let applies = if o.choice.sticky() {
            o.turn <= turn
        } else {
            o.turn == turn
        };
        if applies
            && active
                .get(o.actor.as_str())
                .is_none_or(|p| p.turn <= o.turn)
        {
            active.insert(o.actor.as_str(), o);
        }
    }
    for (actor, o) in active {
        let mine = get(&current, actor).to_vec();
        match choice_orders(state, seat, actor, &o.choice, &mine) {
            Some(orders) if valid(state, seat, &orders) => set(&mut current, actor, orders),
            Some(_) => refused += 1,
            None => {}
        }
    }
    let mut orders: Vec<Order> = current.into_iter().flat_map(|(_, o)| o).collect();
    let errors = legatum_core::validate_orders(state, seat, &orders);
    if !errors.is_empty() {
        refused += errors.len();
        let bad: std::collections::BTreeSet<usize> = errors.iter().map(|e| e.index).collect();
        orders = orders
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !bad.contains(i))
            .map(|(_, o)| o)
            .collect();
    }
    (orders, refused)
}

pub struct TraceTurn {
    pub turn: u32,
    pub state: GameState,
    pub hard: Vec<Order>,
    pub chosen: Vec<Order>,
    /// Hard's planning passes for each unit that the line overrode (diagnosis runs only).
    pub passes: BTreeMap<u32, Vec<&'static str>>,
    /// Hard's traced options on this turn (diagnosis runs only).
    pub decisions: Vec<legatum_ai::DecisionOption>,
}

pub struct Rollout {
    pub outcome: Outcome,
    pub score: i32,
    pub refused: usize,
    pub trace: Vec<TraceTurn>,
    /// Diagnosis runs: whether a shadow Hard brain gave the same orders as the session's.
    pub shadow_in_sync: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Keep {
    Nothing,
    Trace,
    /// Trace, and diagnose Hard's orders on this turn with a shadow brain.
    Diagnose(u32),
}

fn start_state(file: &ScenarioFile, index: u32) -> Result<GameState, String> {
    let mut start = file.prepare()?;
    start.setup.seed = crate::scenario::run_seed(start.setup.seed, index);
    start.setup.players[file.seat.0 as usize].difficulty = AiDifficulty::Hard;
    Ok(start)
}

/// Play the card on dice seed `index` (the baseline's numbering: 0 is the card's own seed) with
/// the same turn limit as `legatum-sim scenario`.
fn play(file: &ScenarioFile, line: &Line, index: u32, keep: Keep) -> Result<Rollout, String> {
    let start = start_state(file, index)?;
    let active = ActiveScenario::new(file.clone(), start.clone());
    let seat = file.seat;
    let mut shadow = match keep {
        Keep::Diagnose(_) => Some((
            legatum_ai::ScriptedBrain::with_difficulty(
                start.player(seat).faction,
                legatum_wasm::session::brain_seed(start.setup.seed, seat),
                seat,
                AiDifficulty::Hard,
            ),
            legatum_core::ViewCache::default(),
        )),
        _ => None,
    };
    let mut session = Session::from_state(start, |_| BrainKind::Scripted);
    session.history = None;
    let mut status = active.status(&session.state);
    let (mut trace, mut refused, mut in_sync) = (Vec::new(), 0, true);
    let mut played = 0;
    while status.outcome == Outcome::Playing && played <= file.objective.turns {
        let turn = session.state.turn;
        let mut orders = session.ai_orders();
        if let Some(r) = &file.rusher {
            legatum_wasm::rusher::apply(&mut orders, &session.state, r);
        }
        let hard = orders.get(&seat).cloned().unwrap_or_default();
        let (chosen, n) = seat_orders(&session.state, seat, &hard, line);
        refused += n;
        let (mut passes, mut decisions) = (BTreeMap::new(), Vec::new());
        if let Some((brain, cache)) = &mut shadow {
            let public = legatum_core::public_view(&session.state);
            let view = legatum_core::player_view_cached(
                &session.state,
                seat,
                &public,
                legatum_core::ViewDetail::Decisions,
                cache,
            );
            let mine = if keep == Keep::Diagnose(turn) {
                let (o, t) = brain.order_diagnosis(&view);
                passes = t.passes;
                decisions = t.decisions;
                o
            } else {
                brain.orders(&view)
            };
            in_sync &= mine == hard;
        }
        if keep != Keep::Nothing {
            trace.push(TraceTurn {
                turn,
                state: session.state.clone(),
                hard,
                chosen: chosen.clone(),
                passes,
                decisions,
            });
        }
        orders.insert(seat, chosen);
        session.step(&orders);
        played += 1;
        status = active.status(&session.state);
    }
    Ok(Rollout {
        outcome: status.outcome,
        score: status.score,
        refused,
        trace,
        shadow_in_sync: in_sync,
    })
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Measure {
    pub n: u32,
    pub wins: u32,
    pub score_sum: i64,
    /// Runs where the engine refused at least one line order (it fell back to Hard's).
    pub refused_runs: u32,
}

impl Measure {
    pub fn mean(&self) -> f64 {
        self.score_sum as f64 / f64::from(self.n.max(1))
    }
    fn key(&self) -> (i64, u32) {
        (self.score_sum, self.wins)
    }
    fn add(&mut self, r: &Rollout) {
        self.n += 1;
        self.wins += u32::from(r.outcome == Outcome::Won);
        self.score_sum += i64::from(r.score);
        self.refused_runs += u32::from(r.refused > 0);
    }
}

fn measure(file: &ScenarioFile, line: &Line, seeds: &[u32]) -> Result<Measure, String> {
    let runs: Result<Vec<Rollout>, String> = seeds
        .par_iter()
        .map(|&s| play(file, line, s, Keep::Nothing))
        .collect();
    let mut m = Measure::default();
    for r in runs? {
        m.add(&r);
    }
    Ok(m)
}

/// Measure many lines at once, every (line, seed) pair in parallel.
fn measure_many(
    file: &ScenarioFile,
    lines: &[Line],
    seeds: &[u32],
) -> Result<Vec<Measure>, String> {
    let pairs: Vec<(usize, u32)> = (0..lines.len())
        .flat_map(|i| seeds.iter().map(move |&s| (i, s)))
        .collect();
    let runs: Result<Vec<(usize, Rollout)>, String> = pairs
        .par_iter()
        .map(|&(i, s)| play(file, &lines[i], s, Keep::Nothing).map(|r| (i, r)))
        .collect();
    let mut out = vec![Measure::default(); lines.len()];
    for (i, r) in runs? {
        out[i].add(&r);
    }
    Ok(out)
}

/// A candidate change to the line: one or more overrides tried together.
struct Candidate {
    label: String,
    overrides: Vec<Override>,
}

fn with(line: &Line, overrides: &[Override]) -> Line {
    let mut next = line.clone();
    for o in overrides {
        next.overrides
            .retain(|x| !(x.turn == o.turn && x.actor == o.actor));
        next.overrides.push(o.clone());
    }
    next
}

fn is_fighter(state: &GameState, seat: PlayerId, unit: UnitId) -> Option<Hex> {
    let u = state.units.get(&unit)?;
    (u.owner == seat
        && legatum_core::rules::is_military(u.kind)
        && u.kind != legatum_core::UnitKind::Scout)
        .then_some(u.pos)
}

/// Every alternative the search tries on one turn: no new order per actor, the battle search's
/// orders, the breach on each target city, and marching on or garrisoning the
/// card's target cities.
fn candidates(
    file: &ScenarioFile,
    t: &TraceTurn,
    targets: &[Target],
    battle_rollouts: u64,
) -> Vec<Candidate> {
    let seat = file.seat;
    let state = &t.state;
    let chosen = grouped(&t.chosen);
    let mut out = Vec::new();
    let exact = |actor: &str, orders: Vec<Order>, source: &str| Override {
        turn: t.turn,
        actor: actor.to_string(),
        choice: Choice::Exact {
            orders,
            source: source.to_string(),
        },
    };
    let diff = |other: &[Order], source: &str| -> Vec<Override> {
        let theirs = grouped(other);
        let mut actors: Vec<&String> = theirs.iter().map(|(a, _)| a).collect();
        actors.extend(chosen.iter().map(|(a, _)| a));
        actors.sort();
        actors.dedup();
        actors
            .into_iter()
            .filter(|a| get(&theirs, a) != get(&chosen, a))
            .map(|a| exact(a, get(&theirs, a).to_vec(), source))
            .collect()
    };
    let push_plan = |label: String, plan: Vec<Override>, out: &mut Vec<Candidate>| {
        if plan.is_empty() {
            return;
        }
        if plan.len() > 1 {
            out.push(Candidate {
                label: label.clone(),
                overrides: plan.clone(),
            });
        }
        for o in plan {
            out.push(Candidate {
                label: format!("{label} ({})", o.actor),
                overrides: vec![o],
            });
        }
    };
    if battle_rollouts > 0 {
        let cfg = legatum_ai::training::TrainingConfig {
            max_rollouts_per_battle: battle_rollouts,
            cap_ms: 0,
            source: "scenario".into(),
            ..Default::default()
        };
        let hard = BTreeMap::from([(seat, t.chosen.clone())]);
        let result = legatum_ai::training::search_turn(state, hard, &cfg, || 0, |_| {});
        if let Some(searched) = result.orders.get(&seat) {
            push_plan(
                "battle search".into(),
                diff(searched, "battle search"),
                &mut out,
            );
        }
    }
    let view = legatum_core::player_view(state, seat);
    for target in targets.iter().filter(|g| g.kind == TargetKind::Take) {
        if let Some(plan) = legatum_ai::training::breach_orders(&view, &t.chosen, target.city) {
            let source = format!("breach {}", target.name);
            push_plan(source.clone(), diff(&plan, &source), &mut out);
        }
    }
    for (actor, orders) in &chosen {
        if !orders.is_empty() {
            out.push(Candidate {
                label: format!("no new order ({actor})"),
                overrides: vec![exact(actor, vec![], "no new order")],
            });
        }
    }
    let mut fighters: Vec<(UnitId, Hex)> = state
        .units
        .values()
        .filter_map(|u| is_fighter(state, seat, u.id).map(|p| (u.id, p)))
        .collect();
    fighters.sort_by_key(|(id, _)| id.0);
    for target in targets {
        let (take, choice) = match target.kind {
            TargetKind::Take | TargetKind::Contain => (
                true,
                Choice::March {
                    to: target.pos,
                    name: target.name.clone(),
                },
            ),
            TargetKind::Hold => (
                false,
                Choice::Garrison {
                    to: target.pos,
                    name: target.name.clone(),
                },
            ),
        };
        // Hold: only cities with a foreign military unit within 4 tiles are worth a garrison.
        if !take
            && !state.units.values().any(|u| {
                u.owner != seat
                    && legatum_core::rules::is_military(u.kind)
                    && u.pos.distance(target.pos) <= 4
            })
        {
            continue;
        }
        let near: Vec<&(UnitId, Hex)> = fighters
            .iter()
            .filter(|(_, p)| p.distance(target.pos) <= if take { 12 } else { 6 })
            .collect();
        let plan: Vec<Override> = near
            .iter()
            .map(|(id, _)| Override {
                turn: t.turn,
                actor: format!("unit {}", id.0),
                choice: choice.clone(),
            })
            .collect();
        let label = match &choice {
            Choice::March { name, .. } => format!("march on {name}"),
            _ => format!("garrison {}", target.name),
        };
        push_plan(label, plan, &mut out);
    }
    out
}

pub struct Searched {
    pub line: Line,
    pub train: Measure,
    pub evaluated: usize,
    pub accepted: Vec<String>,
}

fn search(
    file: &ScenarioFile,
    seed: Line,
    train: &[u32],
    max_evals: usize,
    passes: u32,
    battle_rollouts: u64,
) -> Result<Searched, String> {
    let start = start_state(file, train[0])?;
    let targets = ActiveScenario::new(file.clone(), start.clone())
        .status(&start)
        .targets;
    let mut line = seed;
    let mut best = measure(file, &line, train)?;
    let mut evaluated = 0;
    let mut accepted = Vec::new();
    for _ in 0..passes {
        let mut improved = false;
        let mut index = 0;
        while evaluated < max_evals {
            let trace = play(file, &line, train[0], Keep::Trace)?.trace;
            let Some(t) = trace.get(index) else { break };
            let mut cands = candidates(file, t, &targets, battle_rollouts);
            cands.retain(|c| c.overrides.iter().any(|o| !line.overrides.contains(o)));
            cands.truncate(max_evals - evaluated);
            evaluated += cands.len();
            let lines: Vec<Line> = cands.iter().map(|c| with(&line, &c.overrides)).collect();
            let scores = measure_many(file, &lines, train)?;
            if let Some((i, m)) = scores
                .iter()
                .enumerate()
                .filter(|(_, m)| m.key() > best.key())
                .max_by_key(|(i, m)| (m.key(), std::cmp::Reverse(*i)))
            {
                eprintln!(
                    "  T{} accept {} (train mean {:.1} -> {:.1}, wins {} -> {})",
                    t.turn,
                    cands[i].label,
                    best.mean(),
                    m.mean(),
                    best.wins,
                    m.wins
                );
                accepted.push(format!("T{}: {}", t.turn, cands[i].label));
                line = lines[i].clone();
                best = *m;
                improved = true;
            }
            index += 1;
        }
        if !improved || evaluated >= max_evals {
            break;
        }
    }
    Ok(Searched {
        line,
        train: best,
        evaluated,
        accepted,
    })
}

fn order_text(state: &GameState, orders: &[Order]) -> String {
    if orders.is_empty() {
        return "no new order".into();
    }
    orders
        .iter()
        .map(|order| crate::scenario::describe(state, order).1)
        .collect::<Vec<_>>()
        .join("; ")
}

fn actor_text(state: &GameState, actor: &str) -> String {
    if let Some(id) = unit_of(actor) {
        return state.units.get(&id).map_or_else(
            || format!("unit #{}", id.0),
            |u| format!("{:?} #{} at ({},{})", u.kind, id.0, u.pos.q, u.pos.r),
        );
    }
    if let Some(id) = actor
        .strip_prefix("city ")
        .and_then(|n| n.parse().ok())
        .map(legatum_core::CityId)
    {
        return state
            .cities
            .get(&id)
            .map_or_else(|| actor.to_string(), |c| c.name.clone());
    }
    actor.to_string()
}

/// One actor whose orders the line changed at the first divergence.
#[derive(Debug, Serialize)]
pub struct Divergence {
    pub turn: u32,
    pub actor: String,
    pub actor_text: String,
    pub hard_order: String,
    pub line_order: String,
    pub choice: String,
    /// Training-seed mean card score of the line, and of the line without this override.
    pub line_train_mean: f64,
    pub without_train_mean: f64,
    /// Hard's planning passes that set this unit's order (empty for cities and the empire).
    pub hard_passes: Vec<String>,
    /// Hard's traced options naming this actor's order, as `domain: option (outcome, score)`.
    pub hard_options: Vec<String>,
}

fn divergences(
    file: &ScenarioFile,
    line: &Line,
    train: &[u32],
    best: &Measure,
) -> Result<(Vec<Divergence>, Option<f64>, bool), String> {
    let probe = play(file, line, train[0], Keep::Trace)?;
    let Some(t) = probe
        .trace
        .iter()
        .find(|t| grouped(&t.hard) != grouped(&t.chosen))
    else {
        return Ok((Vec::new(), None, true));
    };
    let diag = play(file, line, train[0], Keep::Diagnose(t.turn))?;
    let dt = diag
        .trace
        .iter()
        .find(|x| x.turn == t.turn)
        .ok_or("diagnosis run ended early")?;
    let hard = grouped(&t.hard);
    let chosen = grouped(&t.chosen);
    let mut actors: Vec<&String> = hard.iter().chain(chosen.iter()).map(|(a, _)| a).collect();
    actors.sort();
    actors.dedup();
    let mut out = Vec::new();
    for actor in actors {
        let (h, c) = (get(&hard, actor), get(&chosen, actor));
        if h == c {
            continue;
        }
        // The override responsible: the latest one that applies to this actor on this turn.
        let responsible = line
            .overrides
            .iter()
            .filter(|o| {
                o.actor == *actor
                    && if o.choice.sticky() {
                        o.turn <= t.turn
                    } else {
                        o.turn == t.turn
                    }
            })
            .max_by_key(|o| o.turn);
        let (choice, without) = match responsible {
            Some(o) => {
                let mut rest = line.clone();
                rest.overrides.retain(|x| x != o);
                (o.choice.label(), measure(file, &rest, train)?.mean())
            }
            None => ("Hard order".to_string(), best.mean()),
        };
        let unit = unit_of(actor);
        let hard_passes = unit
            .and_then(|u| dt.passes.get(&u.0))
            .map(|p| p.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let hard_options = dt
            .decisions
            .iter()
            .filter(|d| {
                d.selected_order
                    .as_ref()
                    .is_some_and(|o| actor_key(o) == *actor)
            })
            .take(4)
            .map(|d| {
                format!(
                    "{}: {} ({}{})",
                    d.domain,
                    d.option,
                    d.outcome,
                    d.score.map(|s| format!(", score {s}")).unwrap_or_default()
                )
            })
            .collect();
        out.push(Divergence {
            turn: t.turn,
            actor: actor.clone(),
            actor_text: actor_text(&t.state, actor),
            hard_order: order_text(&t.state, h),
            line_order: order_text(&t.state, c),
            choice,
            line_train_mean: best.mean(),
            without_train_mean: without,
            hard_passes,
            hard_options,
        });
    }
    // The whole turn's departure: the line with every override made on this turn removed.
    let mut rest = line.clone();
    rest.overrides.retain(|o| o.turn != t.turn);
    let turn_without = measure(file, &rest, train)?.mean();
    Ok((out, Some(turn_without), diag.shadow_in_sync))
}

fn md_cell(s: &str) -> String {
    s.replace('|', "/")
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| run(&cli))
}

fn run(cli: &Cli) -> Result<(), String> {
    let file =
        ScenarioFile::parse(&std::fs::read_to_string(&cli.card).map_err(|e| e.to_string())?)?;
    if !cli.score_only {
        let split = std::fs::read_to_string(&cli.split)
            .map_err(|e| format!("{}: {e}", cli.split.display()))?;
        if let Some(why) = search_refusal(&split, &file.id)? {
            return Err(format!("Refusing to search: {why}."));
        }
    }
    let fresh: Vec<u32> = (FRESH_START..FRESH_START + cli.fresh).collect();
    let train: Vec<u32> = (1..=cli.train_seeds).collect();
    std::fs::create_dir_all(&cli.out).map_err(|e| e.to_string())?;
    let hard = measure(&file, &Line::default(), &fresh)?;
    let mut result = serde_json::json!({
        "format": "legatum-card-loop",
        "card": {"id": file.id, "revision": file.revision, "state_hash": file.state.hash, "title": file.title},
        "fresh_seed_indices": [FRESH_START, FRESH_START + cli.fresh - 1],
        "hard": hard,
        "hard_mean": hard.mean(),
    });
    if cli.score_only {
        write_json(&cli.out.join("result.json"), &result)?;
        println!(
            "{}: Hard {}/{} wins, mean {:.1}",
            file.id,
            hard.wins,
            hard.n,
            hard.mean()
        );
        return Ok(());
    }
    let searched = search(
        &file,
        Line::default(),
        &train,
        cli.max_evals,
        cli.passes,
        cli.battle_rollouts,
    )?;
    let line_fresh = measure(&file, &searched.line, &fresh)?;
    let (divs, turn_without, in_sync) =
        divergences(&file, &searched.line, &train, &searched.train)?;
    let md = report(
        &file,
        cli,
        &hard,
        &searched,
        &line_fresh,
        &divs,
        turn_without,
        in_sync,
    );
    std::fs::write(cli.out.join("report.md"), md).map_err(|e| e.to_string())?;
    let obj = result.as_object_mut().expect("object");
    obj.insert("train_seed_indices".into(), serde_json::json!(train));
    obj.insert("evaluated".into(), serde_json::json!(searched.evaluated));
    obj.insert("accepted".into(), serde_json::json!(searched.accepted));
    obj.insert("line_train".into(), serde_json::json!(searched.train));
    obj.insert("line_fresh".into(), serde_json::json!(line_fresh));
    obj.insert(
        "line_fresh_mean".into(),
        serde_json::json!(line_fresh.mean()),
    );
    obj.insert("divergences".into(), serde_json::json!(divs));
    obj.insert("shadow_in_sync".into(), serde_json::json!(in_sync));
    obj.insert(
        "first_turn_without_train_mean".into(),
        serde_json::json!(turn_without),
    );
    obj.insert("line".into(), serde_json::json!(searched.line));
    write_json(&cli.out.join("result.json"), &result)?;
    println!(
        "{}: Hard {}/{} ({:.1}); line {}/{} ({:.1}); {} evaluations; report {}",
        file.id,
        hard.wins,
        hard.n,
        hard.mean(),
        line_fresh.wins,
        line_fresh.n,
        line_fresh.mean(),
        searched.evaluated,
        cli.out.join("report.md").display()
    );
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())? + "\n";
    std::fs::write(path, text).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
fn report(
    file: &ScenarioFile,
    cli: &Cli,
    hard: &Measure,
    searched: &Searched,
    line: &Measure,
    divs: &[Divergence],
    turn_without: Option<f64>,
    in_sync: bool,
) -> String {
    let mut out = format!(
        "# {}: card loop\n\nCard `{}` revision {}, state {}. Hard and the line are measured on {} held-out dice seeds (indices {}–{}); the search selected on training seeds 1–{} only.\n\n",
        file.title,
        file.id,
        file.revision,
        file.state.hash,
        cli.fresh,
        FRESH_START,
        FRESH_START + cli.fresh - 1,
        cli.train_seeds
    );
    out.push_str(
        "| Line | Wins | Mean card score | Runs with refused orders |\n|---|---:|---:|---:|\n",
    );
    for (name, m) in [("Hard", hard), ("Searched line", line)] {
        writeln!(
            out,
            "| {name} | {}/{} | {:.1} | {} |",
            m.wins,
            m.n,
            m.mean(),
            m.refused_runs
        )
        .unwrap();
    }
    out.push('\n');
    writeln!(
        out,
        "Search: {} line evaluations ({} dice each), training mean {:.1} with {}/{} wins.\n",
        searched.evaluated,
        cli.train_seeds,
        searched.train.mean(),
        searched.train.wins,
        searched.train.n
    )
    .unwrap();
    if line.wins == line.n {
        out.push_str("**Result: the line wins on every held-out seed.**\n\n");
    } else {
        writeln!(
            out,
            "**Result: no line that wins every seed.** Best line {}/{} against Hard {}/{} (mean {:+.1}).\n",
            line.wins,
            line.n,
            hard.wins,
            hard.n,
            line.mean() - hard.mean()
        )
        .unwrap();
    }
    if !searched.accepted.is_empty() {
        out.push_str("Accepted changes, in order:\n\n");
        for a in &searched.accepted {
            writeln!(out, "- {}", md_cell(a)).unwrap();
        }
        out.push('\n');
    }
    match divs.first() {
        None => out.push_str(
            "## First divergence\n\nThe line gives Hard's orders on every turn of training seed 1: there is no divergence.\n\n",
        ),
        Some(first) => {
            writeln!(out, "## First divergence: turn {}\n", first.turn).unwrap();
            out.push_str("Both orders are from the same position on training seed 1. Value is the training-seed mean card score of the line, against the line with only this override removed. Hard's passes are the planning steps that set the unit's order (a shadow Hard brain re-ran the turn with tracing).\n\n");
            out.push_str("| Actor | Hard's order | Line's order | Line choice | Value: line vs without | Hard's passes |\n|---|---|---|---|---:|---|\n");
            for d in divs {
                writeln!(
                    out,
                    "| {} | {} | {} | {} | {:.1} vs {:.1} ({:+.1}) | {} |",
                    md_cell(&d.actor_text),
                    md_cell(&d.hard_order),
                    md_cell(&d.line_order),
                    md_cell(&d.choice),
                    d.line_train_mean,
                    d.without_train_mean,
                    d.line_train_mean - d.without_train_mean,
                    md_cell(&d.hard_passes.join(" → "))
                )
                .unwrap();
            }
            out.push('\n');
            if let Some(w) = turn_without {
                writeln!(
                    out,
                    "With every override made on turn {} removed (Hard's orders that turn, the rest of the line kept), the training mean is {:.1}, so this turn's departure from Hard is worth {:+.1}.\n",
                    first.turn,
                    w,
                    first.line_train_mean - w
                )
                .unwrap();
            }
            if !in_sync {
                out.push_str("Warning: the shadow Hard brain did not reproduce the session's orders on every turn, so the passes may be off.\n\n");
            }
        }
    }
    out.push_str("## Limits\n\nThe line overrides Hard per unit, city or empire action; an override the engine refuses in a position falls back to Hard's order. Rivals are Hard with fresh AI memory, as in the card's baseline. Values are card scores, not Hard's internal valuations.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card() -> ScenarioFile {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scenarios/lab/swiss-hold-biel.json"
        );
        ScenarioFile::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn seed_and_thread_counts_must_be_at_least_one() {
        use clap::Parser;
        let base = ["card-loop", "c.json", "--out", "o"];
        assert!(Cli::try_parse_from(base).is_ok());
        for flag in ["--train-seeds", "--fresh", "--threads"] {
            let args = base.iter().copied().chain([flag, "0"]);
            assert!(Cli::try_parse_from(args).is_err(), "{flag} 0");
        }
    }

    #[test]
    fn held_out_and_unlisted_cards_are_not_searched() {
        let split = r#"{"train": ["a"], "held_out": ["b"]}"#;
        assert_eq!(search_refusal(split, "a").unwrap(), None);
        assert!(
            search_refusal(split, "b")
                .unwrap()
                .unwrap()
                .contains("held out")
        );
        assert!(
            search_refusal(split, "c")
                .unwrap()
                .unwrap()
                .contains("--add")
        );
        let gate = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/sim-results/scenario-gate/split.json"
        ))
        .unwrap();
        // The gate holds Defend Essen out: the loop must refuse to search it.
        assert!(
            search_refusal(&gate, "germany-defend-essen")
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn empty_line_plays_hard_and_refused_orders_fall_back() {
        let actions = |orders: &[Order]| {
            let mut actions: Vec<_> = orders.iter().map(|o| format!("{o:?}")).collect();
            actions.sort();
            actions
        };
        let file = card();
        let start = start_state(&file, 1).unwrap();
        let mut session = Session::from_state(start.clone(), |_| BrainKind::Scripted);
        let hard = session.ai_orders().remove(&file.seat).unwrap_or_default();
        let (same, refused) = seat_orders(&start, file.seat, &hard, &Line::default());
        assert_eq!(refused, 0);
        assert_eq!(actions(&same), actions(&hard));
        // An attack on an empty tile is refused and leaves Hard's orders in place.
        let unit = start.units_of(file.seat).next().unwrap().id;
        let line = Line {
            overrides: vec![Override {
                turn: start.turn,
                actor: format!("unit {}", unit.0),
                choice: Choice::Exact {
                    orders: vec![Order::RangedAttack {
                        unit,
                        target: Hex { q: 0, r: 0 },
                    }],
                    source: "test".into(),
                },
            }],
        };
        let (orders, refused) = seat_orders(&start, file.seat, &hard, &line);
        assert_eq!(refused, 1);
        assert_eq!(actions(&orders), actions(&hard));
    }

    #[test]
    fn line_play_is_deterministic() {
        let file = card();
        let a = play(&file, &Line::default(), 2, Keep::Nothing).unwrap();
        let b = play(&file, &Line::default(), 2, Keep::Nothing).unwrap();
        assert_eq!((a.score, a.outcome), (b.score, b.outcome));
    }
}

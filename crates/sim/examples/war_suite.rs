//! `war_suite` (an example binary, so cloud batches can run it with `run-batch.sh --bin`): score the AI on the war scenarios cut from the user's playtests
//! (`scenarios/war/*.json`, user idea 2026-10-11: "the goal is to take these two cities in the
//! amount of time given that you know that it is achievable because I did it as a human").
//!
//! Every scenario starts from the moment a real campaign began, with a goal, a turn limit and the
//! user's own result (`ScenarioFile::human`). The AI plays the user's seat against the same
//! opponents across seeded runs; run `r` uses the same seed for every brain and parameter file,
//! so two reports pair run by run (`tools/sim/war-scenarios-report.py --compare`).
//!
//! A scenario whose attacker the AI cannot play carries a script (`war_script`): the designer's
//! recorded orders for his seat, replayed for the turns of his window.
//!
//! Each run also hunts for bugs: the position is checked with `scenario::validate_state` after
//! every turn, panics are caught, and both are listed in the report.

mod war_script;

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::Rng;
use legatum_core::scenario::Outcome;
use legatum_core::{AiDifficulty, Event, GameState, PlayerId, UnitKind, rules::is_military};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::brain_seed;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The war scenarios, built into the binary so a cloud server needs nothing else. The wrapper
/// (`tools/sim/war-scenarios.sh`) checks that every file in `scenarios/war/` is listed here.
const EMBEDDED: [&str; 19] = [
    include_str!("../../../scenarios/war/war-athens-two-pikemen-t112.json"),
    include_str!("../../../scenarios/war/war-break-the-hold-t168.json"),
    include_str!("../../../scenarios/war/war-german-air-early.json"),
    include_str!("../../../scenarios/war/war-german-air-t219.json"),
    include_str!("../../../scenarios/war/war-golden-horde-thin.json"),
    include_str!("../../../scenarios/war/war-hollow-swiss-t76.json"),
    include_str!("../../../scenarios/war/war-india-chur-ring.json"),
    include_str!("../../../scenarios/war/war-india-sparta-t142.json"),
    include_str!("../../../scenarios/war/war-mongol-bombers-t150.json"),
    include_str!("../../../scenarios/war/war-mongol-rush.json"),
    include_str!("../../../scenarios/war/war-mongols-chur-t137.json"),
    include_str!("../../../scenarios/war/war-sparta-all-out.json"),
    include_str!("../../../scenarios/war/war-sparta-next-door-t101.json"),
    include_str!("../../../scenarios/war/war-sparta-nicaea.json"),
    include_str!("../../../scenarios/war/war-sparta-walls.json"),
    include_str!("../../../scenarios/war/war-swiss-poachers.json"),
    include_str!("../../../scenarios/war/war-swiss-hold-biel.json"),
    include_str!("../../../scenarios/war/war-swiss-tepid-t180.json"),
    include_str!("../../../scenarios/war/war-swiss-tepid-t220.json"),
];

/// The seed for run `run`: the same derivation as `legatum-sim scenario` (run 0 keeps the
/// scenario's own seed, the dice the user's game had from this turn on).
fn run_seed(seed: u64, run: u32) -> u64 {
    if run == 0 {
        seed
    } else {
        Rng::derive(seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    }
}

#[derive(clap::Parser, Debug)]
#[command(
    name = "war_suite",
    about = "Score the AI on the war scenarios cut from the user's playtests"
)]
pub struct Cli {
    /// Read scenarios from this directory instead of the built-in copies.
    #[arg(long)]
    pub dir: Option<PathBuf>,
    /// Only these scenario ids (comma-separated; default: all).
    #[arg(long, value_delimiter = ',')]
    pub ids: Vec<String>,
    /// Seeded runs per scenario (`--games` is the same, for run-batch.sh).
    #[arg(long, alias = "games", default_value_t = 32)]
    pub runs: u32,
    /// First run index (`--seed` from run-batch.sh); runs are first..first+runs.
    #[arg(long, alias = "seed", default_value_t = 0)]
    pub first_run: u32,
    /// The brain in the user's seat: hard, hard+search, or training (Hard with its battles
    /// searched, `legatum_ai::training`; `WAR_TRAINING_BUDGET` rollouts per battle).
    #[arg(long, default_value = "hard")]
    pub brain: String,
    /// Candidate AI parameters (`AiParams` JSON). Without it every seat plays the built-in genes.
    #[arg(long)]
    pub ai_params: Option<PathBuf>,
    /// Which seats play the candidate parameters: seat (the north-star measure), opponents, all.
    #[arg(long, default_value = "seat")]
    pub params_seats: String,
    /// Parameters for every seat that doesn't play the candidate's (the attack check's
    /// defenders with door defense off). Without it they play the built-in genes.
    #[arg(long)]
    pub opponent_params: Option<PathBuf>,
    /// Worker threads (default: every core).
    #[arg(long, default_value_t = 0)]
    pub threads: usize,
    /// Write the JSON report here.
    #[arg(long)]
    pub report: Option<PathBuf>,
    /// run-batch.sh: the report is also written to <out-dir>/war-suite.json.
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    /// run-batch.sh passes these; they are recorded or ignored.
    #[arg(long, default_value = "")]
    pub label: String,
    #[arg(long, hide = true)]
    pub out: Option<PathBuf>,
    #[arg(long, hide = true)]
    pub status_file: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParamSeats {
    Seat,
    Opponents,
    All,
}

impl ParamSeats {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "seat" => Ok(Self::Seat),
            "opponents" => Ok(Self::Opponents),
            "all" => Ok(Self::All),
            other => Err(format!(
                "--params-seats must be seat, opponents or all, not {other}"
            )),
        }
    }
    fn candidate(self, is_seat: bool) -> bool {
        match self {
            Self::Seat => is_seat,
            Self::Opponents => !is_seat,
            Self::All => true,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Run {
    pub run: u32,
    pub seed: String,
    pub outcome: Outcome,
    pub met: bool,
    /// Turns played: until the objective was met, or the whole window.
    pub turns: u32,
    /// The seat's combat units (Scouts excluded) destroyed.
    pub units_lost: u32,
    /// Combat units (Scouts excluded) destroyed that belonged to a player at war with the seat
    /// when they died: the other side of the exchange ratio.
    pub enemy_units_lost: u32,
    /// The same, by owner (player id): who paid for the exchange.
    pub enemy_units_lost_by: BTreeMap<u8, u32>,
    /// The seat's starting cities held by someone else at the end.
    pub cities_lost: u32,
    pub cities_captured: u32,
    pub settlers_captured: u32,
    pub settlers_lost: u32,
    /// Turns after the start of the seat's first declaration of war (None: never declared).
    pub first_war: Option<u32>,
    /// Turns after the start of the seat's first city capture.
    pub first_capture: Option<u32>,
    /// Turns the seat was at war but fought no battle.
    pub idle_war_turns: u32,
    /// Engagement shape (focus fire and stragglers, 2026-10-30). One group is one enemy unit
    /// or city the seat's units attacked in one turn; a lone group had a single attacker.
    pub attack_groups: u32,
    pub lone_attack_groups: u32,
    /// Enemy units the seat's units hit, summed over turns, and how many of them died that turn
    /// (the kill conversion of the seat's fire: spread damage heals, focused damage kills).
    pub units_hit: u32,
    pub units_hit_killed: u32,
    /// The seat's combat units destroyed with no other seat combat unit within two tiles of
    /// them at the start of the turn: a straggler that arrived or stood alone.
    pub isolated_losses: u32,
    /// Surprise wars opened by the AI seat during this run.
    pub declarations: u32,
    /// HP dealt to the declared team's units and cities on that declaration turn.
    pub declaration_turn_damage: u32,
    /// Enemy cities captured from a declared team within five resolution turns.
    pub captures_within_five: u32,
    pub score: i32,
    pub reason: String,
    /// Positions the rules should never reach (turn: problems).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub invalid_states: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panic: Option<String>,
}

/// `--brain training`: the seat's battles are searched with these settings.
static TRAINING_SEAT: std::sync::OnceLock<legatum_ai::training::TrainingConfig> =
    std::sync::OnceLock::new();

fn brains(
    start: &GameState,
    seat: PlayerId,
    search_seat: bool,
    params: Option<&AiParams>,
    seats: ParamSeats,
    others: Option<&AiParams>,
    memories: &legatum_wasm::scenario::AiMemories,
) -> BTreeMap<PlayerId, Box<dyn Brain>> {
    let builtin = AiParams::builtin();
    let baseline = others.unwrap_or(&builtin);
    start
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let setup = &start.setup.players[p.id.0 as usize];
            let seed = brain_seed(start.setup.seed, p.id);
            let is_seat = p.id == seat;
            let difficulty = if is_seat {
                AiDifficulty::Hard
            } else {
                setup.difficulty
            };
            // Without a candidate the seat plays the built-in AI and the others `others`.
            let chosen = match params {
                Some(candidate) if seats.candidate(is_seat) => Some(candidate),
                Some(_) => Some(baseline),
                None if !is_seat => others,
                None => None,
            };
            let mut scripted = match chosen {
                Some(params) => {
                    ScriptedBrain::with_params(p.faction, seed, p.id, difficulty, params)
                }
                None => ScriptedBrain::with_difficulty(p.faction, seed, p.id, difficulty),
            };
            if let Some(memory) = memories.get(&p.id) {
                scripted.restore_opponent_memory(memory.clone());
            }
            // Opponents keep the brains they had in the user's game (Hard, or Hard + Search).
            let search = if is_seat {
                search_seat
            } else {
                setup.search && difficulty == AiDifficulty::Hard
            };
            let brain: Box<dyn Brain> = if search {
                Box::new(legatum_ai::search::SearchBrain::new(
                    scripted,
                    legatum_ai::search::SearchConfig::in_app(start.players.len()),
                ))
            } else {
                Box::new(scripted)
            };
            (p.id, brain)
        })
        .collect()
}

fn order_unit(o: &legatum_core::Order) -> u32 {
    match o {
        legatum_core::Order::MoveUnit { unit, .. }
        | legatum_core::Order::MoveUnitPath { unit, .. } => unit.0,
        _ => u32::MAX,
    }
}

/// Play one seeded run with the AI in the seat.
pub fn play(
    file: &ScenarioFile,
    search_seat: bool,
    params: Option<&AiParams>,
    seats: ParamSeats,
    others: Option<&AiParams>,
    run: u32,
) -> Result<Run, String> {
    let mut start = file.prepare(false)?;
    start.setup.seed = run_seed(start.setup.seed, run);
    let script = war_script::load(&file.id)?;
    let seat = file.seat;
    start.setup.players[seat.0 as usize].difficulty = AiDifficulty::Hard;
    let active = ActiveScenario::new(file.clone(), start.clone());
    let seed = start.setup.seed.to_string();
    let starting: Vec<_> = start.cities_of(seat).map(|c| c.id).collect();
    let mut state = start.clone();
    let memories = file.ai_memories()?;
    let mut brains = brains(&start, seat, search_seat, params, seats, others, &memories);
    let mut status = active.status(&state);
    let (mut units_lost, mut cities_captured, mut settlers_captured, mut settlers_lost) =
        (0, 0, 0, 0);
    let mut enemy_units_lost_by: BTreeMap<u8, u32> = BTreeMap::new();
    let (mut first_war, mut first_capture, mut idle_war_turns) = (None, None, 0);
    let (mut declarations, mut declaration_turn_damage, mut captures_within_five) = (0, 0, 0);
    let mut declared_at = BTreeMap::new();
    let (mut attack_groups, mut lone_attack_groups, mut units_hit, mut units_hit_killed) =
        (0, 0, 0, 0);
    let mut isolated_losses = 0;
    let mut invalid_states = vec![];
    let mut played = 0;
    let mut supply_seen = 0usize;
    // Training round 2 (c): the seat's searched plans kept from turn to turn.
    let mut training_plans: Vec<legatum_ai::training::StoredPlan> = Vec::new();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        while status.outcome == Outcome::Playing && played <= file.objective.turns {
            let turn = state.turn;
            let elapsed = turn - start.turn;
            let mut orders: BTreeMap<_, _> = state
                .players
                .iter()
                .filter(|p| p.alive)
                .filter_map(|p| {
                    brains
                        .get_mut(&p.id)
                        .map(|b| (p.id, b.orders(&legatum_core::player_view(&state, p.id))))
                })
                .collect();
            if let Some(cfg) = TRAINING_SEAT.get()
                && let Some(hard) = orders.get(&seat).cloned()
            {
                let mut r = legatum_ai::training::search_turn_with(
                    &state,
                    BTreeMap::from([(seat, hard)]),
                    cfg,
                    &training_plans,
                    |_| true,
                    || 0,
                    |_| {},
                );
                training_plans = std::mem::take(&mut r.plans);
                if let Some(o) = r.orders.remove(&seat) {
                    orders.insert(seat, o);
                }
            }
            if let Some(script) = &script {
                script.apply(turn, &mut orders);
            }
            // WAR_PLAN_TRACE=1 traces runs 0-2; WAR_PLAN_TRACE=1,7,29 traces those runs.
            let traced = std::env::var("WAR_PLAN_TRACE").ok().is_some_and(|v| {
                let listed: Vec<u32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if listed.len() > 1 || v.contains(',') {
                    listed.contains(&run)
                } else {
                    run < 3
                }
            });
            if traced {
                let m = brains.get(&seat).and_then(|b| b.opponent_memory());
                let plan = m
                    .as_ref()
                    .and_then(|m| m.campaigns.plan.active.as_ref())
                    .map(|p| (p.kind.clone(), p.value));
                let camp = m
                    .as_ref()
                    .and_then(|m| m.campaigns.active.as_ref())
                    .map(|c| (c.target, c.committed.len()));
                let last = m.as_ref().and_then(|m| m.campaigns.events.last().cloned());
                let army: Vec<_> = state
                    .units
                    .values()
                    .filter(|u| u.owner == seat && legatum_core::data::pays_upkeep(u.kind))
                    .collect();
                let near = state
                    .cities
                    .values()
                    .filter(|c| !c.owner.is_barbarian() && state.at_war(seat, c.owner))
                    .map(|c| {
                        let n = army.iter().filter(|u| u.pos.distance(c.pos) <= 3).count();
                        let g = state
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == c.owner
                                    && u.pos == c.pos
                                    && legatum_core::data::pays_upkeep(u.kind)
                            })
                            .count();
                        let d = army
                            .iter()
                            .map(|u| u.pos.distance(c.pos))
                            .min()
                            .unwrap_or(99);
                        (d, c.name.clone(), c.hp, g, n)
                    })
                    .min();
                let decl: Vec<_> = orders
                    .get(&seat)
                    .map(|os| {
                        os.iter()
                            .filter(|o| {
                                matches!(
                                    o,
                                    legatum_core::Order::DeclareWar { .. }
                                        | legatum_core::Order::ProposeTreaty { .. }
                                        | legatum_core::Order::RespondToProposal { .. }
                                )
                            })
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default();
                let shots = orders
                    .get(&seat)
                    .map(|os| {
                        os.iter()
                            .filter(|o| matches!(o, legatum_core::Order::RangedAttack { .. }))
                            .count()
                    })
                    .unwrap_or(0);
                let short = m.as_ref().map(|m| m.campaigns.short.current.clone());
                let op = m
                    .as_ref()
                    .and_then(|m| m.campaigns.operation.as_ref())
                    .filter(|op| op.running())
                    .map(|op| op.kind.clone());
                let seat_orders: Vec<String> = orders
                    .get(&seat)
                    .map(|os| os.iter().map(|o| format!("{o:?}")).collect())
                    .unwrap_or_default();
                eprintln!(
                    "PT {} run {run} T{} plan {:?} camp {:?} last {:?} army {} nearest {:?} shots {} dipl {:?} short {:?} op {:?}",
                    file.id,
                    state.turn,
                    plan,
                    camp,
                    last,
                    army.len(),
                    near,
                    shots,
                    decl,
                    short,
                    op
                );
                if std::env::var_os("WAR_PLAN_TRACE_ORDERS").is_some() {
                    eprintln!("PO {} run {run} T{} {:?}", file.id, state.turn, seat_orders);
                }
            }
            if first_war.is_none()
                && orders.get(&seat).is_some_and(|os| {
                    os.iter()
                        .any(|o| matches!(o, legatum_core::Order::DeclareWar { .. }))
                })
            {
                first_war = Some(elapsed);
            }
            let at_war = state
                .players
                .iter()
                .any(|p| p.alive && p.id != seat && state.at_war(seat, p.id));
            let new_declarations: Vec<_> = orders
                .get(&seat)
                .into_iter()
                .flatten()
                .filter_map(|o| match o {
                    legatum_core::Order::DeclareWar {
                        against,
                        breach: false,
                    } => Some(*against),
                    _ => None,
                })
                .collect();
            // WAR_SUPPLY_LOG: per turn, the seat's supply in use and idle, its cities, and each
            // plan switch to a Campaign priced with a supply term (does the priced supply arrive?).
            if std::env::var_os("WAR_SUPPLY_LOG").is_some() {
                let view = legatum_core::player_view(&state, seat);
                let used: i32 = view.supply.iter().map(|s| s.used.total).sum();
                let idle: usize = view.supply.iter().map(|s| s.idle.len()).sum();
                let cities: Vec<u32> = state.cities_of(seat).map(|c| c.id.0).collect();
                let events = brains.get(&seat).map(|b| b.plan_events()).unwrap_or(&[]);
                let priced: Vec<(String, i32)> = events
                    .iter()
                    .skip(supply_seen)
                    .filter_map(|e| match e {
                        legatum_ai::plan::PlanEvent::Switch { to, terms, .. } => {
                            let supply: i32 = terms
                                .iter()
                                .filter(|(n, _)| n == "supply")
                                .map(|(_, x)| *x)
                                .sum();
                            let value: i32 = terms.iter().map(|(_, x)| *x).sum();
                            match to {
                                legatum_ai::plan::PlanKind::Campaign { target, .. } => {
                                    Some((format!("{} {value}", target.0), supply))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();
                supply_seen = events.len();
                eprintln!(
                    "SUP {}",
                    serde_json::json!({"id": file.id, "run": run, "turn": state.turn,
                        "used": used, "idle": idle, "cities": cities, "switches": priced})
                );
            }
            let (next, events) = legatum_core::resolve_turn(&state, &orders);
            let prev = std::mem::replace(&mut state, next);
            for team in &new_declarations {
                if state
                    .diplomacy
                    .as_ref()
                    .is_some_and(|d| d.at_war(state.team_of(seat), *team))
                {
                    declarations += 1;
                    declared_at.insert(*team, elapsed);
                }
            }
            let mut fought = false;
            // Engagement shape: attackers per defender this turn, and which hit units died.
            let mut groups: BTreeMap<(u8, u32), std::collections::BTreeSet<u32>> = BTreeMap::new();
            let destroyed: std::collections::BTreeSet<u32> = events
                .iter()
                .filter_map(|e| match e {
                    Event::UnitDestroyed { unit, .. } => Some(unit.0),
                    _ => None,
                })
                .collect();
            for e in &events {
                if let Event::Battle {
                    attacker: legatum_core::events::Attacker::Unit(by),
                    attacker_owner,
                    defender,
                    ..
                } = e
                    && *attacker_owner == seat
                {
                    let key = match defender {
                        legatum_core::events::Defender::Unit { unit, .. } => (0, unit.0),
                        legatum_core::events::Defender::City { city, .. } => (1, city.0),
                    };
                    groups.entry(key).or_default().insert(by.0);
                }
            }
            attack_groups += groups.len() as u32;
            lone_attack_groups += groups.values().filter(|a| a.len() == 1).count() as u32;
            for (kind, id) in groups.keys() {
                if *kind == 0 {
                    units_hit += 1;
                    if destroyed.contains(id) {
                        units_hit_killed += 1;
                    }
                }
            }
            let combat = |u: &&legatum_core::state::Unit| {
                u.owner == seat && is_military(u.kind) && u.kind != UnitKind::Scout
            };
            for e in &events {
                if let Event::UnitDestroyed {
                    unit,
                    owner,
                    unit_kind,
                    ..
                } = e
                    && *owner == seat
                    && is_military(*unit_kind)
                    && *unit_kind != UnitKind::Scout
                    && let Some(was) = prev.units.get(unit)
                    && !prev
                        .units
                        .values()
                        .filter(combat)
                        .any(|o| o.id != *unit && o.pos.distance(was.pos) <= 2)
                {
                    isolated_losses += 1;
                }
            }
            for e in &events {
                match e {
                    Event::Battle {
                        attacker_owner,
                        defender:
                            legatum_core::events::Defender::Unit { owner, .. }
                            | legatum_core::events::Defender::City { owner, .. },
                        damage_to_defender,
                        ..
                    } if *attacker_owner == seat
                        && new_declarations.iter().any(|team| {
                            state
                                .diplomacy
                                .as_ref()
                                .is_some_and(|d| d.at_war(state.team_of(seat), *team))
                                && prev.team_of(*owner) == *team
                        }) =>
                    {
                        declaration_turn_damage += (*damage_to_defender).max(0) as u32;
                    }
                    Event::UnitDestroyed {
                        owner, unit_kind, ..
                    } if *owner == seat
                        && is_military(*unit_kind)
                        && *unit_kind != UnitKind::Scout =>
                    {
                        units_lost += 1
                    }
                    Event::UnitDestroyed {
                        owner, unit_kind, ..
                    } if *owner != seat
                        && !owner.is_barbarian()
                        && is_military(*unit_kind)
                        && *unit_kind != UnitKind::Scout
                        && prev.at_war(seat, *owner) =>
                    {
                        *enemy_units_lost_by.entry(owner.0).or_default() += 1;
                    }
                    Event::CityCaptured { to, from, .. } if *to == seat => {
                        cities_captured += 1;
                        first_capture.get_or_insert(elapsed);
                        if declared_at
                            .get(&prev.team_of(*from))
                            .is_some_and(|start| elapsed - start < 5)
                        {
                            captures_within_five += 1;
                        }
                    }
                    Event::SettlerCaptured { to, .. } if *to == seat => settlers_captured += 1,
                    Event::SettlerCaptured { from, .. } if *from == seat => settlers_lost += 1,
                    Event::Battle {
                        attacker_owner,
                        defender:
                            legatum_core::events::Defender::Unit { owner, .. }
                            | legatum_core::events::Defender::City { owner, .. },
                        ..
                    } if *attacker_owner == seat || *owner == seat => fought = true,
                    _ => {}
                }
            }
            if at_war && !fought {
                idle_war_turns += 1;
            }
            // A unit left standing in land that closed under it (borders grew, Peace was
            // made) is legal and stranded, and may walk toward an exit (game-rules "Mongols
            // courier default"). One that stepped this turn from open land onto a tile closed to
            // it both before and after the turn (no war at either end) entered closed land: a bug.
            let mut problems: Vec<String> = legatum_core::scenario::validate_state(&state)
                .into_iter()
                .filter(|p| !p.contains("territory without being at war"))
                .collect();
            for u in state.units.values() {
                let closed_before = || {
                    // A stranded unit may walk through closed land toward an exit.
                    prev.units.get(&u.id).is_some_and(|p| {
                        p.pos != u.pos && p.owner == u.owner && prev.unit_in_open_land(p)
                    }) && !prev.passage_allowed(
                        u.owner,
                        u.kind,
                        prev.map.get(u.pos).and_then(|t| t.owner),
                    )
                };
                if !u.owner.is_barbarian() && !state.unit_in_open_land(u) && closed_before() {
                    let land = prev.map.get(u.pos).and_then(|t| t.owner);
                    let from = prev.units[&u.id].pos;
                    problems.push(format!(
                        "unit {} ({:?}, player {}) moved from ({},{}) into closed land at ({},{}) owned by {:?} (then {:?}; at war before {}, after {})",
                        u.id.0,
                        u.kind,
                        u.owner.0,
                        from.q,
                        from.r,
                        u.pos.q,
                        u.pos.r,
                        land.map(|o| o.0),
                        state.map.get(u.pos).and_then(|t| t.owner).map(|o| o.0),
                        land.is_some_and(|o| prev.at_war(u.owner, o)),
                        land.is_some_and(|o| state.at_war(u.owner, o)),
                    ));
                }
            }
            if !problems.is_empty() && std::env::var_os("WAR_TRACE").is_some() {
                // The orders and diplomacy behind a bug report, for reproducing it.
                eprintln!(
                    "== {} run {run} T{}: {}",
                    file.id,
                    prev.turn,
                    problems.join("; ")
                );
                for (p, os) in &orders {
                    for o in os {
                        if matches!(
                            o,
                            legatum_core::Order::MoveUnit { .. }
                                | legatum_core::Order::MoveUnitPath { .. }
                        ) && !problems
                            .iter()
                            .any(|x| x.contains(&format!("unit {} ", order_unit(o))))
                        {
                            continue;
                        }
                        if matches!(
                            o,
                            legatum_core::Order::SetProduction { .. }
                                | legatum_core::Order::SetProductionQueue { .. }
                                | legatum_core::Order::SetResearch { .. }
                                | legatum_core::Order::SetResearchQueue { .. }
                                | legatum_core::Order::SkipUnit { .. }
                                | legatum_core::Order::SleepUnit { .. }
                        ) {
                            continue;
                        }
                        eprintln!("   order p{}: {o:?}", p.0);
                    }
                }
                for e in &events {
                    let t = format!("{e:?}");
                    if t.contains("War")
                        || t.contains("Peace")
                        || t.contains("Treaty")
                        || t.contains("Proposal")
                    {
                        eprintln!("   event: {}", &t[..t.len().min(220)]);
                    }
                }
            }
            if !problems.is_empty() && invalid_states.len() < 5 {
                invalid_states.push(format!("T{}: {}", state.turn, problems.join("; ")));
            }
            played += 1;
            status = active.status(&state);
        }
    }));
    let panic = caught.err().map(|e| {
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "panic".into())
    });
    let cities_lost = starting
        .iter()
        .filter(|id| state.cities.get(id).is_none_or(|c| c.owner != seat))
        .count() as u32;
    let met = status.outcome == Outcome::Won;
    Ok(Run {
        run,
        seed,
        outcome: if panic.is_some() {
            Outcome::Lost
        } else {
            status.outcome
        },
        met: met && panic.is_none(),
        turns: state.turn - start.turn,
        units_lost,
        enemy_units_lost: enemy_units_lost_by.values().sum(),
        enemy_units_lost_by,
        cities_lost,
        cities_captured,
        settlers_captured,
        settlers_lost,
        first_war,
        first_capture,
        idle_war_turns,
        attack_groups,
        lone_attack_groups,
        units_hit,
        units_hit_killed,
        isolated_losses,
        declarations,
        declaration_turn_damage,
        captures_within_five,
        score: status.score,
        reason: status.reason,
        invalid_states,
        panic,
    })
}

#[derive(Debug, Serialize)]
pub struct ScenarioReport {
    pub id: String,
    pub revision: u32,
    pub title: String,
    pub faction: String,
    pub turns: u32,
    pub objective: String,
    pub human: Option<legatum_wasm::scenario::HumanResult>,
    pub runs: u32,
    /// Share of runs that met the objective, in percent.
    pub success_percent: u32,
    /// Median turns to meet the objective, over the runs that met it.
    pub median_turns: Option<u32>,
    /// Mean combat units lost per run, times 100.
    pub units_lost_x100: u32,
    /// Mean combat units destroyed on the side at war with the seat per run, times 100.
    pub enemy_units_lost_x100: u32,
    /// Mean starting cities lost per run, times 100.
    pub cities_lost_x100: u32,
    /// Runs in which the seat never declared war (offensive scenarios start at Peace).
    pub never_declared: u32,
    pub bugs: Vec<String>,
    pub results: Vec<Run>,
}

fn report(file: &ScenarioFile, runs: Vec<Run>) -> ScenarioReport {
    let n = runs.len().max(1) as u32;
    let mut won: Vec<u32> = runs.iter().filter(|r| r.met).map(|r| r.turns).collect();
    won.sort_unstable();
    let mut bugs: Vec<String> = vec![];
    for r in &runs {
        if let Some(p) = &r.panic {
            bugs.push(format!("run {} (seed {}): panic: {p}", r.run, r.seed));
        }
        for s in &r.invalid_states {
            bugs.push(format!("run {} (seed {}): {s}", r.run, r.seed));
        }
    }
    ScenarioReport {
        id: file.id.clone(),
        revision: file.revision,
        title: file.title.clone(),
        faction: format!("{:?}", file.faction),
        turns: file.objective.turns,
        objective: file.objective.summary.clone(),
        human: file.human.clone(),
        runs: runs.len() as u32,
        success_percent: won.len() as u32 * 100 / n,
        median_turns: (!won.is_empty()).then(|| won[won.len() / 2]),
        units_lost_x100: runs.iter().map(|r| r.units_lost).sum::<u32>() * 100 / n,
        enemy_units_lost_x100: runs.iter().map(|r| r.enemy_units_lost).sum::<u32>() * 100 / n,
        cities_lost_x100: runs.iter().map(|r| r.cities_lost).sum::<u32>() * 100 / n,
        never_declared: runs.iter().filter(|r| r.first_war.is_none()).count() as u32,
        bugs,
        results: runs,
    }
}

pub fn load(dir: Option<&PathBuf>, ids: &[String]) -> Result<Vec<ScenarioFile>, String> {
    let texts: Vec<String> = match dir {
        Some(dir) => {
            let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            paths.sort();
            paths
                .iter()
                .map(|p| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display())))
                .collect::<Result<_, _>>()?
        }
        None => EMBEDDED.iter().map(|s| s.to_string()).collect(),
    };
    let mut files = vec![];
    for text in texts {
        let file = ScenarioFile::parse(&text)?;
        if ids.is_empty() || ids.contains(&file.id) {
            files.push(file);
        }
    }
    if files.is_empty() {
        return Err("no war scenarios selected".into());
    }
    Ok(files)
}

fn pct(x100: u32) -> String {
    format!("{}.{:02}", x100 / 100, x100 % 100)
}

pub fn run(cli: &Cli) -> Result<(), String> {
    let search_seat = match cli.brain.as_str() {
        "hard" => false,
        "hard+search" => true,
        "training" => {
            // WAR_TRAINING_ROUND2=1: round 2's fidelity settings (full-game check with
            // WAR_TRAINING_CONFIRM rollouts, default 16, of WAR_TRAINING_CONFIRM_TURNS turns;
            // kept plans); WAR_TRAINING_SKIP_AIR=1
            // leaves battles with aircraft near them to Hard.
            let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u64>().ok());
            let mut cfg = legatum_ai::training::TrainingConfig {
                cap_ms: 0,
                max_rollouts_per_battle: env("WAR_TRAINING_BUDGET").unwrap_or(1024),
                source: "scenario".into(),
                ..Default::default()
            };
            if env("WAR_TRAINING_ROUND2").unwrap_or(0) > 0 {
                cfg = cfg
                    .round2()
                    .with_confirm(env("WAR_TRAINING_CONFIRM").unwrap_or(16) as u32);
                if let Some(t) = env("WAR_TRAINING_CONFIRM_TURNS") {
                    cfg.confirm_turns = t as u32;
                }
            }
            cfg.skip_air = env("WAR_TRAINING_SKIP_AIR").unwrap_or(0) > 0;
            let _ = TRAINING_SEAT.set(cfg);
            false
        }
        other => {
            return Err(format!(
                "--brain must be hard, hard+search or training, not {other}"
            ));
        }
    };
    let seats = ParamSeats::parse(&cli.params_seats)?;
    let read = |path: &PathBuf| -> Result<AiParams, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let p: AiParams = serde_json::from_str(&text).map_err(|e| format!("params: {e}"))?;
        p.validate()?;
        Ok(p)
    };
    let params: Option<AiParams> = cli.ai_params.as_ref().map(read).transpose()?;
    let others: Option<AiParams> = cli.opponent_params.as_ref().map(read).transpose()?;
    if cli.runs == 0 {
        return Err("--runs must be at least 1".into());
    }
    let files = load(cli.dir.as_ref(), &cli.ids)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build()
        .map_err(|e| e.to_string())?;
    let began = std::time::Instant::now();
    let jobs: Vec<(usize, u32)> = (0..files.len())
        .flat_map(|f| (cli.first_run..cli.first_run + cli.runs).map(move |r| (f, r)))
        .collect();
    let mut results: Vec<(usize, Run)> = pool.install(|| {
        jobs.par_iter()
            .map(|&(f, r)| {
                play(
                    &files[f],
                    search_seat,
                    params.as_ref(),
                    seats,
                    others.as_ref(),
                    r,
                )
                .map(|x| (f, x))
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    results.sort_by_key(|(f, r)| (*f, r.run));
    let mut by_file: Vec<Vec<Run>> = (0..files.len()).map(|_| vec![]).collect();
    for (f, r) in results {
        by_file[f].push(r);
    }
    let reports: Vec<ScenarioReport> = files
        .iter()
        .zip(by_file)
        .map(|(file, runs)| report(file, runs))
        .collect();
    let suite_success =
        reports.iter().map(|r| r.success_percent).sum::<u32>() / reports.len() as u32;
    // At or past the human: met the objective in at least half the runs, no slower than the user.
    let parity = reports
        .iter()
        .filter(|r| {
            r.success_percent >= 50
                && match (&r.human, r.median_turns) {
                    (Some(h), Some(t)) => !h.met || t <= h.turns,
                    (None, _) => true,
                    _ => false,
                }
        })
        .count();
    let out = serde_json::json!({
        "format": "legatum-war-suite",
        "version": 1,
        "created_unix": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        "label": cli.label,
        "first_run": cli.first_run,
        "engine_commit": legatum_wasm::history::ENGINE_COMMIT,
        "brain": cli.brain,
        "candidate_params": params.as_ref().map(|p| &p.name),
        "opponent_params": others.as_ref().map(|p| &p.name),
        "params_seats": cli.ai_params.as_ref().map(|_| &cli.params_seats),
        "runs": cli.runs,
        "seconds": began.elapsed().as_secs(),
        "suite_success_percent": suite_success,
        "human_parity": parity,
        "scenarios": reports,
    });
    let json = serde_json::to_string_pretty(&out).expect("json");
    let mut targets: Vec<PathBuf> = cli.report.iter().cloned().collect();
    if let Some(dir) = &cli.out_dir {
        targets.push(dir.join("war-suite.json"));
    }
    for path in targets {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, &json).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!(
        "| Scenario | Goal met | Median turns | Units lost | Enemy units lost | Cities lost | Human |\n|---|---|---|---|---|---|---|"
    );
    for r in &reports {
        let human = r.human.as_ref().map_or("n/a".to_string(), |h| {
            format!(
                "{} in {} turns, {} units lost",
                if h.met { "met" } else { "not met" },
                h.turns,
                h.units_lost
            )
        });
        println!(
            "| {} | {}% | {} | {} | {} | {} | {} |",
            r.id,
            r.success_percent,
            r.median_turns.map_or("-".into(), |t| t.to_string()),
            pct(r.units_lost_x100),
            pct(r.enemy_units_lost_x100),
            pct(r.cities_lost_x100),
            human
        );
    }
    println!(
        "suite: {}% goal success, {}/{} scenarios at human parity, {} bugs, {}s ({})",
        suite_success,
        parity,
        reports.len(),
        reports.iter().map(|r| r.bugs.len()).sum::<usize>(),
        began.elapsed().as_secs(),
        cli.brain
    );
    Ok(())
}

fn main() -> std::process::ExitCode {
    use clap::Parser;
    match run(&Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            std::process::ExitCode::FAILURE
        }
    }
}

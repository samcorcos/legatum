//! `legatum-sim scenario <file>`: the AI baseline for a scenario. The scripted AI plays the
//! scenario seat from the same start state across RNG seeds (run 0 keeps the position's own
//! seed), scoring every run with the core objective scorer. Writes `<id>-baseline.json`
//! (every run, with the seat's raw orders for the first
//! decision turns) and a readable `<id>-baseline.md`.

use crate::clock;
use legatum_core::rng::Rng;
use legatum_core::scenario::{Outcome, ScoreLine, Status};
use legatum_core::{AiDifficulty, GameState, Order, PlayerId, ProductionItem};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    name = "legatum-sim scenario",
    about = "AI baseline for a scenario: the scripted AI plays the seat across seeds"
)]
pub struct Cli {
    /// Scenario file (scenarios/<id>.json).
    pub file: PathBuf,
    #[arg(long, default_value_t = 50)]
    pub runs: u32,
    /// Difficulty of the AI playing the scenario seat.
    #[arg(long, value_parser = crate::parse_difficulty, default_value = "hard")]
    pub difficulty: AiDifficulty,
    /// Worker threads (keep at most 6 while the user plays).
    #[arg(long, default_value_t = 6)]
    pub threads: usize,
    /// Turns of the window whose orders are recorded and summarized.
    #[arg(long, default_value_t = 5)]
    pub decision_turns: u32,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/sim-results/scenarios"))]
    pub out: PathBuf,
    /// Also write the summary into the scenario file's `baseline` (used by measurement reports).
    #[arg(long)]
    pub record: bool,
    /// Every AI seat, the scenario seat included, plays with these tuned parameters (a file, or
    /// the JSON itself). Use with `--out DIR`, not `--record`, to compare a gene with the baseline.
    #[arg(long)]
    pub ai_params: Option<PathBuf>,
    /// Give `--ai-params` to the scenario seat only; every other empire keeps the defaults.
    #[arg(long)]
    pub params_seat_only: bool,
    /// Measurement only: tell the seat's Hard brain the card's win condition as a pinned plan
    /// (`scenario_told.rs`), to split "does not know the goal" from "cannot execute it".
    #[arg(long)]
    pub tell_objective: bool,
    /// With `--tell-objective`, tell a hold card as `PlanKind::Hold` on the most threatened city
    /// instead of the quiet plan.
    #[arg(long)]
    pub tell_hold_city: bool,
    /// Write the seat's order trace (each unit's planning passes) for this run to
    /// `<out>/<id>-trace-run<N>.json`.
    #[arg(long)]
    pub trace_run: Option<u32>,
}

/// How the scenario seat is played, beyond its difficulty.
#[derive(Clone, Copy, Default)]
pub struct SeatOptions {
    /// Tuned parameters for every AI seat (or the scenario seat only, with `seat_only`).
    pub params: Option<&'static legatum_ai::AiParams>,
    pub seat_only: bool,
    pub tell_objective: bool,
    pub tell_hold_city: bool,
    pub trace: bool,
}

/// A city the seat lost or took.
#[derive(Debug, Serialize)]
pub struct CityChange {
    pub turn: u32,
    pub city: String,
    /// True: the seat took it; false: the seat lost it.
    pub taken: bool,
}

/// One turn of a traced run: each seat order with the passes that set it.
#[derive(Debug, Serialize)]
pub struct TraceTurn {
    pub turn: u32,
    pub orders: Vec<TraceOrder>,
    /// Plan options the planner weighed (domain "plan"), with the selected one marked.
    pub plans: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TraceOrder {
    pub actor: String,
    pub action: String,
    pub passes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct TurnPoint {
    pub turn: u32,
    pub score: i32,
    pub cities: u32,
    pub combat_units: u32,
}

#[derive(Debug, Serialize)]
pub struct TurnOrders {
    pub turn: u32,
    pub orders: Vec<Order>,
}

#[derive(Debug, Serialize)]
pub struct RunResult {
    pub run: u32,
    pub seed: String,
    pub outcome: Outcome,
    pub score: i32,
    pub reason: String,
    /// The turn being planned when the scenario ended.
    pub ended_turn: u32,
    pub score_lines: Vec<ScoreLine>,
    pub timeline: Vec<TurnPoint>,
    /// The seat's orders on the first decision turns.
    pub decisions: Vec<TurnOrders>,
    /// Production, purchase and research orders over the whole window.
    pub economy: Vec<TurnOrders>,
    /// Diplomatic orders over the whole window.
    pub diplomacy: Vec<TurnOrders>,
    /// The first turn the seat ordered a declaration of war, if any.
    pub first_war_turn: Option<u32>,
    /// Cities the seat lost or took, by turn.
    pub city_changes: Vec<CityChange>,
    /// The seat's plan switches by turn and, with `--tell-objective`, the plan told each turn.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub told: Vec<(u32, String)>,
    #[serde(skip)]
    pub trace: Vec<TraceTurn>,
}

/// The seed for one baseline run; run 0 keeps the scenario's own.
pub fn run_seed(seed: u64, run: u32) -> u64 {
    if run == 0 {
        seed
    } else {
        Rng::derive(seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    }
}

fn combat_units(state: &GameState, p: PlayerId) -> u32 {
    state
        .units_of(p)
        .filter(|u| {
            legatum_core::rules::is_military(u.kind) && u.kind != legatum_core::UnitKind::Scout
        })
        .count() as u32
}

fn is_economy(o: &Order) -> bool {
    matches!(
        o,
        Order::SetProduction { .. }
            | Order::SetProductionQueue { .. }
            | Order::BuyItem { .. }
            | Order::SetResearch { .. }
            | Order::SetResearchQueue { .. }
    )
}

/// Play one baseline run: every empire, the seat included, is the scripted AI.
pub fn play(
    file: &ScenarioFile,
    difficulty: AiDifficulty,
    run: u32,
    decision_turns: u32,
    opts: SeatOptions,
) -> Result<RunResult, String> {
    let mut start = file.prepare()?;
    start.setup.seed = run_seed(start.setup.seed, run);
    start.setup.players[file.seat.0 as usize].difficulty = difficulty;
    let active = ActiveScenario::new(file.clone(), start.clone());
    let mut session = Session::from_state(start, |_| BrainKind::Scripted);
    session.history = None;
    if let Some(params) = opts.params {
        let seats: Vec<_> = session
            .state
            .players
            .iter()
            .filter(|p| !p.is_human && (!opts.seat_only || p.id == file.seat))
            .map(|p| (p.id, p.faction))
            .collect();
        for (id, faction) in seats {
            let difficulty = session.state.setup.players[id.0 as usize].difficulty;
            let seed = legatum_wasm::session::brain_seed(session.state.setup.seed, id);
            session.replace_brain(
                id,
                Box::new(legatum_ai::ScriptedBrain::with_params(
                    faction, seed, id, difficulty, params,
                )),
            );
        }
    }
    let seat = file.seat;
    let told_log: std::sync::Arc<std::sync::Mutex<Vec<(u32, String)>>> = Default::default();
    let trace_slot: Option<crate::scenario_told::TraceSlot> = opts.trace.then(Default::default);
    // The seat plays its own scripted brain inside a wrapper that logs plan switches (and
    // tells the objective or traces orders when asked); the orders are those of the brain alone.
    let hard_v2 = session.state.setup.rule_experiments.hard_v2_ai;
    if hard_v2 && (opts.tell_objective || opts.trace) {
        return Err("--tell-objective and --trace-run need the scripted seat brain".into());
    }
    if session.brain_kind(seat) == Some(BrainKind::Scripted) && !hard_v2 {
        let faction = session.state.player(seat).faction;
        let seed = legatum_wasm::session::brain_seed(session.state.setup.seed, seat);
        let inner = match opts.params {
            Some(params) => {
                legatum_ai::ScriptedBrain::with_params(faction, seed, seat, difficulty, params)
            }
            None => legatum_ai::ScriptedBrain::with_difficulty(faction, seed, seat, difficulty),
        };
        let told = opts.tell_objective.then(|| {
            crate::scenario_told::Told::from_card(
                &file.objective.win,
                &file.objective.keep,
                active.start.turn + file.objective.turns,
                opts.tell_hold_city,
            )
        });
        session.replace_brain(
            seat,
            Box::new(crate::scenario_told::SeatBrain {
                inner,
                told,
                log: told_log.clone(),
                seen: 0,
                trace: trace_slot.clone(),
            }),
        );
    }
    let mut city_changes = Vec::new();
    let mut trace = Vec::new();
    let mut status: Status = active.status(&session.state);
    let mut timeline = vec![TurnPoint {
        turn: status.turn,
        score: status.score,
        cities: session.state.cities_of(seat).count() as u32,
        combat_units: combat_units(&session.state, seat),
    }];
    let (mut decisions, mut economy, mut diplomacy) = (vec![], vec![], vec![]);
    let mut first_war_turn = None;
    let limit = file.objective.turns + 1;
    let mut played = 0;
    while status.outcome == Outcome::Playing && played < limit {
        let turn = session.state.turn;
        let mut orders = session.ai_orders();
        if let Some(r) = &file.rusher {
            legatum_wasm::rusher::apply(&mut orders, &session.state, r);
        }
        let mine = orders.get(&seat).cloned().unwrap_or_default();
        if turn < active.start.turn + decision_turns {
            decisions.push(TurnOrders {
                turn,
                orders: mine.clone(),
            });
        }
        let eco: Vec<Order> = mine.iter().filter(|o| is_economy(o)).cloned().collect();
        if !eco.is_empty() {
            economy.push(TurnOrders { turn, orders: eco });
        }
        let dip: Vec<Order> = mine
            .iter()
            .filter(|o| legatum_core::diplomacy::is_diplomatic(o))
            .cloned()
            .collect();
        if first_war_turn.is_none() && dip.iter().any(|o| matches!(o, Order::DeclareWar { .. })) {
            first_war_turn = Some(turn);
        }
        if !dip.is_empty() {
            diplomacy.push(TurnOrders { turn, orders: dip });
        }
        if let Some(slot) = &trace_slot {
            for (t, executed, tr) in slot.lock().expect("trace").drain(..) {
                let orders = executed
                    .iter()
                    .map(|o| {
                        let (actor, action) = describe(&session.state, o);
                        let passes = legatum_core::rules::units::order_unit(o)
                            .and_then(|u| tr.passes.get(&u.0).cloned())
                            .unwrap_or_default();
                        TraceOrder {
                            actor,
                            action,
                            passes,
                        }
                    })
                    .collect();
                let plans = tr
                    .decisions
                    .iter()
                    .filter(|d| d.domain == "plan")
                    .map(|d| {
                        format!(
                            "{} {} {:?} {}",
                            d.outcome,
                            d.option,
                            d.score,
                            d.gate.as_deref().unwrap_or("")
                        )
                    })
                    .collect();
                trace.push(TraceTurn {
                    turn: t,
                    orders,
                    plans,
                });
            }
        }
        let before: Vec<legatum_core::CityId> =
            session.state.cities_of(seat).map(|c| c.id).collect();
        session.step(&orders);
        let after: Vec<legatum_core::CityId> =
            session.state.cities_of(seat).map(|c| c.id).collect();
        let name = |id: &legatum_core::CityId| {
            session
                .state
                .cities
                .get(id)
                .map_or_else(|| format!("city {}", id.0), |c| c.name.clone())
        };
        for id in before.iter().filter(|id| !after.contains(id)) {
            city_changes.push(CityChange {
                turn,
                city: name(id),
                taken: false,
            });
        }
        for id in after.iter().filter(|id| !before.contains(id)) {
            city_changes.push(CityChange {
                turn,
                city: name(id),
                taken: true,
            });
        }
        played += 1;
        status = active.status(&session.state);
        timeline.push(TurnPoint {
            turn: status.turn,
            score: status.score,
            cities: session.state.cities_of(seat).count() as u32,
            combat_units: combat_units(&session.state, seat),
        });
    }
    Ok(RunResult {
        run,
        seed: active.start.setup.seed.to_string(),
        outcome: status.outcome,
        score: status.score,
        reason: status.reason,
        ended_turn: status.turn,
        score_lines: status.score_lines,
        timeline,
        decisions,
        economy,
        diplomacy,
        first_war_turn,
        city_changes,
        told: std::mem::take(&mut *told_log.lock().expect("told log")),
        trace,
    })
}

#[derive(Debug, Serialize)]
pub struct Distribution {
    pub min: i32,
    pub p10: i32,
    pub p25: i32,
    pub median: i32,
    pub p75: i32,
    pub p90: i32,
    pub max: i32,
    /// Mean times 100, rounded down (integer math, like the rest of the sim output).
    pub mean_x100: i64,
}

pub fn distribution(scores: &[i32]) -> Distribution {
    let mut s = scores.to_vec();
    s.sort_unstable();
    let at = |pct: usize| s[((s.len() - 1) * pct + 50) / 100];
    Distribution {
        min: s[0],
        p10: at(10),
        p25: at(25),
        median: at(50),
        p75: at(75),
        p90: at(90),
        max: s[s.len() - 1],
        mean_x100: s.iter().map(|&x| i64::from(x)).sum::<i64>() * 100 / s.len() as i64,
    }
}

/// Plain words for one order, from the state it was given in.
pub fn describe(state: &GameState, order: &Order) -> (String, String) {
    let unit = |id: &legatum_core::UnitId| {
        state.units.get(id).map_or_else(
            || format!("new unit #{}", id.0),
            |u| format!("{:?} #{} at ({},{})", u.kind, id.0, u.pos.q, u.pos.r),
        )
    };
    let city = |id: &legatum_core::CityId| {
        state
            .cities
            .get(id)
            .map_or_else(|| format!("city {}", id.0), |c| c.name.clone())
    };
    let place = |h: &legatum_core::Hex| {
        if let Some(c) = state.city_at(*h) {
            format!("{} ({},{})", c.name, h.q, h.r)
        } else if let Some(u) = state
            .units_at(*h)
            .find(|u| legatum_core::rules::is_military(u.kind))
        {
            format!(
                "{} {:?} ({},{})",
                legatum_core::scenario::empire_name(state, u.owner),
                u.kind,
                h.q,
                h.r
            )
        } else {
            format!("({},{})", h.q, h.r)
        }
    };
    let item = |i: &ProductionItem| match i {
        ProductionItem::Unit { unit } => format!("{unit:?}"),
        ProductionItem::Building { building } => format!("{building:?}"),
        ProductionItem::Wonder { building, tile } => {
            format!("{building:?} at ({},{})", tile.q, tile.r)
        }
        ProductionItem::Project { project } => format!("{project:?} project"),
        ProductionItem::Improvement { improvement, tile } => {
            format!("{improvement:?} at ({},{})", tile.q, tile.r)
        }
    };
    let team = |t: &legatum_core::TeamId| {
        state.players.iter().find(|p| p.team == *t).map_or_else(
            || format!("team {}", t.0),
            |p| legatum_core::scenario::empire_name(state, p.id),
        )
    };
    match order {
        Order::MoveUnit { unit: u, to } | Order::MoveUnitPath { unit: u, to, .. } => {
            (unit(u), format!("move to {}", place(to)))
        }
        Order::RangedAttack { unit: u, target } => (unit(u), format!("shoot {}", place(target))),
        Order::SleepUnit { unit: u, .. } => (unit(u), "sleep".into()),
        Order::SkipUnit { unit: u } => (unit(u), "skip".into()),
        Order::FoundCity { unit: u } => (unit(u), "found a city".into()),
        Order::DisbandUnit { unit: u } => (unit(u), "disband".into()),
        Order::UpgradeUnit { unit: u } => (unit(u), "upgrade".into()),
        Order::UpgradeSiege { unit: u } => (unit(u), "buy Siege".into()),
        Order::SetAutomation {
            unit: u,
            automation,
            ..
        } => (unit(u), format!("automate {automation:?}")),
        Order::SetAutoFire {
            unit: u, enabled, ..
        } => (
            unit(u),
            if *enabled {
                "auto-fire"
            } else {
                "stop auto-fire"
            }
            .into(),
        ),
        Order::SetProduction { city: c, item: i } => (city(c), format!("build {}", item(i))),
        Order::SetCityAutomation { city: c, focus, .. } => (
            city(c),
            focus.map_or_else(
                || "stop automatic building".into(),
                |focus| format!("build automatically: {focus:?}"),
            ),
        ),
        Order::Raze { city: c } => (city(c), "raze".into()),
        Order::SetProductionQueue { city: c, queue } => (
            city(c),
            format!(
                "build {}",
                if queue.is_empty() {
                    "nothing".to_string()
                } else {
                    queue.iter().map(item).collect::<Vec<_>>().join(" → ")
                }
            ),
        ),
        Order::BuyItem { city: c, item: i } => (city(c), format!("buy {}", item(i))),
        Order::BuyTile { city: c, tile } => (city(c), format!("buy the tile at {}", place(tile))),
        Order::Action { action } => ("Empire".into(), format!("{action:?}")),
        Order::GiveGold { to, amount } => (
            "Empire".into(),
            format!(
                "give {amount} Gold to {}",
                legatum_core::scenario::empire_name(state, *to)
            ),
        ),
        Order::AirStrike { plane, target } => (
            format!("aircraft {}", plane.0),
            format!("strike {}", place(target)),
        ),
        Order::Rebase { plane, to } => (
            format!("aircraft {}", plane.0),
            format!("rebase to {}", city(to)),
        ),
        Order::AirRecon { plane, target } => (
            format!("aircraft {}", plane.0),
            format!("recon {}", place(target)),
        ),
        Order::UpgradeAircraft { plane, .. } => (format!("aircraft {}", plane.0), "upgrade".into()),
        Order::SetResearch { tech } => ("Research".into(), format!("{tech:?}")),
        Order::SetScienceConversion { to } => {
            ("Research".into(), format!("convert Science to {to:?}"))
        }
        Order::SetResearchGoal { tech } => (
            "Research".into(),
            tech.map_or_else(|| "no goal".into(), |t| format!("goal {t:?}")),
        ),
        Order::SetResearchQueue { techs } => (
            "Research".into(),
            techs
                .first()
                .map_or_else(|| "nothing".into(), |t| format!("{t:?}")),
        ),
        Order::HireCompany { unit, seller, .. } => (
            "Production".into(),
            format!("hire a {unit:?} company from player {}", seller.0),
        ),
        Order::DeclareWar { against, .. } => (
            "Diplomacy".into(),
            format!("declare war on {}", team(against)),
        ),
        Order::ProposeTreaty { to, kind, .. } => (
            "Diplomacy".into(),
            format!("propose {kind:?} to {}", team(to)),
        ),
        Order::RespondToProposal {
            proposal, accept, ..
        } => {
            let offer = state
                .diplomacy
                .as_ref()
                .and_then(|d| d.proposals.iter().find(|p| p.id == *proposal))
                .map_or_else(
                    || "an offer".to_string(),
                    |p| format!("{:?} from {}", p.kind, team(&p.from)),
                );
            (
                "Diplomacy".into(),
                format!("{} {offer}", if *accept { "accept" } else { "decline" }),
            )
        }
        Order::WithdrawProposal { .. } => ("Diplomacy".into(), "withdraw an offer".into()),
        Order::BreakTreaty { with, .. } => (
            "Diplomacy".into(),
            format!("leave the Alliance with {}", team(with)),
        ),
    }
}

/// A decision turn, an actor, and how many runs gave each of its orders.
type DecisionRow = (u32, String, Vec<(String, usize)>);

/// For each decision turn and actor, how often each order was given across runs.
fn decision_table(start: &GameState, runs: &[RunResult], turns: u32) -> Vec<DecisionRow> {
    let mut table: BTreeMap<(u32, String), BTreeMap<String, usize>> = BTreeMap::new();
    for r in runs {
        for t in &r.decisions {
            // Descriptions use the start state: unit ids and cities are shared by every run.
            for o in &t.orders {
                let (actor, action) = describe(start, o);
                *table
                    .entry((t.turn, actor))
                    .or_default()
                    .entry(action)
                    .or_default() += 1;
            }
        }
    }
    table
        .into_iter()
        .filter(|((turn, _), _)| *turn < start.turn + turns)
        .map(|((turn, actor), counts)| {
            let mut counts: Vec<_> = counts.into_iter().collect();
            counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            (turn, actor, counts)
        })
        .collect()
}

pub fn run(cli: &Cli) -> Result<(), String> {
    let text =
        std::fs::read_to_string(&cli.file).map_err(|e| format!("{}: {e}", cli.file.display()))?;
    let file = ScenarioFile::parse(&text)?;
    let start = file.prepare()?;
    if cli.runs == 0 {
        return Err("--runs must be at least 1".into());
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads.max(1))
        .build()
        .map_err(|e| e.to_string())?;
    let params = cli
        .ai_params
        .as_deref()
        .map(crate::load_params)
        .transpose()?;
    if cli.params_seat_only && params.is_none() {
        return Err("--params-seat-only needs --ai-params".into());
    }
    let began = std::time::Instant::now();
    let mut runs: Vec<RunResult> = pool.install(|| {
        (0..cli.runs)
            .into_par_iter()
            .map(|run| {
                let opts = SeatOptions {
                    params,
                    seat_only: cli.params_seat_only,
                    tell_objective: cli.tell_objective,
                    tell_hold_city: cli.tell_hold_city,
                    trace: cli.trace_run == Some(run),
                };
                play(&file, cli.difficulty, run, cli.decision_turns, opts)
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    runs.sort_by_key(|r| r.run);
    if let Some(n) = cli.trace_run
        && let Some(r) = runs.iter().find(|r| r.run == n)
    {
        std::fs::create_dir_all(&cli.out).map_err(|e| e.to_string())?;
        let path = cli.out.join(format!("{}-trace-run{n}.json", file.id));
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&r.trace).expect("trace json"),
        )
        .map_err(|e| e.to_string())?;
    }
    let scores: Vec<i32> = runs.iter().map(|r| r.score).collect();
    let dist = distribution(&scores);
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    for r in &runs {
        *outcomes.entry(format!("{:?}", r.outcome)).or_default() += 1;
    }
    let won = runs.iter().filter(|r| r.outcome == Outcome::Won).count();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for r in &runs {
        *reasons.entry(r.reason.clone()).or_default() += 1;
    }
    let wars: Vec<u32> = runs.iter().filter_map(|r| r.first_war_turn).collect();
    let decisions = decision_table(&start, &runs, cli.decision_turns);
    std::fs::create_dir_all(&cli.out).map_err(|e| e.to_string())?;
    let created_at = clock::timestamp(clock::unix_secs());
    let summary = serde_json::json!({
        "format": "legatum-scenario-baseline",
        "version": 1,
        "scenario": {"id": file.id, "revision": file.revision, "title": file.title,
            "seat": file.seat, "faction": file.faction, "state_hash": file.state.hash},
        "difficulty": cli.difficulty,
        "runs": cli.runs,
        "decision_turns": cli.decision_turns,
        "created_at": created_at,
        "engine_commit": legatum_wasm::history::ENGINE_COMMIT,
        "scores": dist,
        "outcomes": outcomes,
        "win_percent": won * 100 / runs.len(),
        "reasons": reasons,
        "first_war": {"runs": wars.len(), "turns": wars},
        "prediction": file.predictions,
        "decision_summary": decisions.iter().map(|(turn, actor, counts)| serde_json::json!({
            "turn": turn, "actor": actor,
            "orders": counts.iter().map(|(a, n)| serde_json::json!({"order": a, "runs": n})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "results": runs,
    });
    let json_path = cli.out.join(format!("{}-baseline.json", file.id));
    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .map_err(|e| e.to_string())?;
    let md = markdown(
        &file,
        cli,
        &runs,
        &dist,
        &outcomes,
        won,
        &reasons,
        &wars,
        &decisions,
        &created_at,
        began.elapsed().as_secs(),
    );
    let md_path = cli.out.join(format!("{}-baseline.md", file.id));
    std::fs::write(&md_path, md).map_err(|e| e.to_string())?;
    let mut turns_taken: Vec<u32> = runs
        .iter()
        .filter(|r| r.outcome == Outcome::Won)
        .map(|r| r.ended_turn.saturating_sub(start.turn))
        .collect();
    turns_taken.sort_unstable();
    let median_turns = turns_taken.get(turns_taken.len() / 2).copied();
    if cli.record {
        let mut updated = file.clone();
        updated.baseline = Some(legatum_wasm::scenario::Baseline {
            runs: cli.runs,
            difficulty: cli.difficulty,
            median: dist.median,
            p10: dist.p10,
            p90: dist.p90,
            win_percent: (!file.objective.win.is_empty()).then(|| (won * 100 / runs.len()) as u32),
            median_turns,
            measured_at: created_at.clone(),
        });
        let mut json = serde_json::to_string_pretty(&updated).expect("scenario JSON");
        json.push('\n');
        std::fs::write(&cli.file, json).map_err(|e| e.to_string())?;
    }
    println!(
        "{}: median {} (p10 {}, p90 {}), won {won}/{} -> {}",
        file.id,
        dist.median,
        dist.p10,
        dist.p90,
        runs.len(),
        md_path.display()
    );
    Ok(())
}

/// A value in hundredths as a decimal, e.g. -125 -> "-1.25". The sign is kept for values between -1 and 0.
fn hundredths(v: i64) -> String {
    let a = v.unsigned_abs();
    format!("{}{}.{:02}", if v < 0 { "-" } else { "" }, a / 100, a % 100)
}

#[allow(clippy::too_many_arguments)]
fn markdown(
    file: &ScenarioFile,
    cli: &Cli,
    runs: &[RunResult],
    dist: &Distribution,
    outcomes: &BTreeMap<String, usize>,
    won: usize,
    reasons: &BTreeMap<String, usize>,
    wars: &[u32],
    decisions: &[DecisionRow],
    created_at: &str,
    seconds: u64,
) -> String {
    let n = runs.len();
    let p = &file.predictions.ai_score;
    let mut out = format!(
        "# Scenario baseline: {} (`{}` r{})\n\n{} · seat {} ({:?}) · AI difficulty {:?} · {} runs · {} · {}s\n\n",
        file.title,
        file.id,
        file.revision,
        created_at,
        file.seat.0,
        file.faction,
        cli.difficulty,
        n,
        legatum_wasm::history::ENGINE_COMMIT,
        seconds
    );
    out += &format!(
        "**Objective:** {} ({} turns)\n\n",
        file.objective.summary, file.objective.turns
    );
    out += "## Score\n\n| | Predicted | Measured |\n|---|---|---|\n";
    out += &format!("| Median | {} | {} |\n", p.median, dist.median);
    out += &format!(
        "| Range | {} to {} | p10 {} to p90 {} (min {}, max {}) |\n",
        p.low, p.high, dist.p10, dist.p90, dist.min, dist.max
    );
    if file.objective.win.is_empty() {
        out += "| Objective met | n/a (score only) | n/a |\n";
    } else {
        out += &format!(
            "| Objective met | {}% | {}% ({won}/{n}) |\n",
            p.win_percent,
            won * 100 / n
        );
    }
    out += &format!("| Mean | | {} |\n\n", hundredths(dist.mean_x100));
    out += "Outcomes: ";
    out += &outcomes
        .iter()
        .map(|(k, v)| format!("{k} {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    out += "\n\nHow runs ended:\n\n";
    let mut reasons: Vec<_> = reasons.iter().collect();
    reasons.sort_by(|a, b| b.1.cmp(a.1));
    for (r, c) in reasons.iter().take(8) {
        out += &format!(
            "- {c} × {}\n",
            if r.is_empty() { "(still playing)" } else { r }
        );
    }
    out += "\nScore histogram:\n\n```\n";
    let lo = dist.min;
    let span = (dist.max - lo).max(1);
    let bins = 10.min(n).max(1) as i32;
    let width = (span + bins) / bins;
    let mut counts = vec![0usize; bins as usize];
    for r in runs {
        counts[(((r.score - lo) / width).min(bins - 1)) as usize] += 1;
    }
    for (i, c) in counts.iter().enumerate() {
        let from = lo + i as i32 * width;
        // An empty bin ends at the bar, without trailing whitespace (git diff --check).
        let line = format!(
            "{:>6} to {:>6} | {}",
            from,
            from + width - 1,
            "#".repeat(*c)
        );
        out += line.trim_end();
        out += "\n";
    }
    out += "```\n\n";
    out += &format!(
        "First war declared by the seat: {} of {n} runs{}\n\n",
        wars.len(),
        if wars.is_empty() {
            String::new()
        } else {
            let mut w = wars.to_vec();
            w.sort_unstable();
            format!(" (median turn {})", w[w.len() / 2])
        }
    );
    out += &format!(
        "## Key decisions, first {} turns\n\nEach line: what the AI ordered, in how many of the {n} runs.\n\n",
        cli.decision_turns
    );
    let mut last_turn = 0;
    for (turn, actor, counts) in decisions {
        if *turn != last_turn {
            out += &format!("\n### Turn {turn}\n\n");
            last_turn = *turn;
        }
        out += &format!(
            "- **{actor}**: {}\n",
            counts
                .iter()
                .take(4)
                .map(|(a, c)| format!("{a} ({c})"))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
    out += "\n## Predictions (written before any run)\n\n";
    out += &format!(
        "- Good play: {}\n- Predicted AI play: {}\n",
        file.predictions.good_play, file.predictions.ai_play
    );
    out
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    use clap::Parser;
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    run(&cli)
}

#[cfg(test)]
mod tests {
    use super::{distribution, hundredths};

    #[test]
    fn mean_keeps_its_sign_below_zero() {
        assert_eq!(hundredths(distribution(&[0, -1]).mean_x100), "-0.50");
        assert_eq!(
            hundredths(distribution(&[-1, -1, -1, -2]).mean_x100),
            "-1.25"
        );
        assert_eq!(hundredths(distribution(&[1, 2]).mean_x100), "1.50");
        assert_eq!(hundredths(0), "0.00");
    }
}

//! Does the coalition act against a runaway leader? (gene `runaway_coalition`, 2026-11-07)
//!
//!   coalition_front_probe SCENARIO.json RUNAWAY_FACTION RUNS EXTRA_TURNS PARAMS.json [PARAMS.json ...]
//!
//! Plays a Scenario Lab scenario with every seat, the scenario seat included, a Hard AI on the
//! given parameters (the `legatum-sim scenario` baseline loop), for the objective's window plus
//! EXTRA_TURNS, RUNS reseeded runs per parameter file. Per arm it prints, as means per run:
//! battles fought by other empires on the runaway's cities and units, the runaway's cities
//! taken by the seat and by third empires, Alliances signed with the seat, Peace signed with the
//! runaway (`alliances_pre_war`: Alliances among others signed before the first war on the runaway), wars declared on it, the seat's objective and starting cities, and the runaway's
//! city count at the end. One JSON line per arm.
use legatum_ai::{AiParams, ScriptedBrain};
use legatum_core::diplomacy::RelationState;
use legatum_core::events::Defender;
use legatum_core::rng::Rng;
use legatum_core::{AiDifficulty, Event, Faction, GameState, PlayerId, TeamId};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn run_seed(seed: u64, run: u32) -> u64 {
    if run == 0 {
        seed
    } else {
        Rng::derive(seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    }
}

fn load(path: &str) -> &'static AiParams {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let params: AiParams = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    params.validate().unwrap_or_else(|e| panic!("{path}: {e}"));
    Box::leak(Box::new(params))
}

fn relations(s: &GameState) -> BTreeMap<(TeamId, TeamId), RelationState> {
    s.diplomacy
        .iter()
        .flat_map(|d| &d.relations)
        .map(|r| ((r.a.min(r.b), r.a.max(r.b)), r.state))
        .collect()
}

/// Count each seat's committed operations from its AI memory: started, struck (War declared
/// and the strike phase reached), attacks ordered on the target, and how each ended.
fn track_operations(
    session: &Session,
    seen: &mut BTreeMap<PlayerId, (u32, bool)>,
    m: &mut BTreeMap<&'static str, i64>,
) {
    use legatum_ai::operation::OperationOutcome as O;
    for p in session.state.players.iter() {
        let Some(memory) = session.brain(p.id).and_then(|b| b.opponent_memory()) else {
            continue;
        };
        let Some(op) = memory.campaigns.operation else {
            continue;
        };
        let entry = seen.entry(p.id).or_insert((u32::MAX, false));
        if entry.0 != op.start_turn {
            *entry = (op.start_turn, false);
            *m.entry("ops_started").or_default() += 1;
        }
        if op.running() || entry.1 {
            continue;
        }
        entry.1 = true;
        *m.entry(if op.log.strike_turn.is_some() {
            "ops_struck"
        } else {
            "ops_never_struck"
        })
        .or_default() += 1;
        *m.entry("ops_attacks").or_default() += i64::from(op.log.attacks);
        *m.entry(match op.outcome {
            O::Captured => "ops_captured",
            O::Expired => "ops_expired",
            O::RosterLost => "ops_roster_lost",
            O::Peace => "ops_peace",
            O::TargetGone | O::TargetTaken => "ops_target_gone",
            O::DeclarationBlocked => "ops_declaration_blocked",
            _ => "ops_other_end",
        })
        .or_default() += 1;
    }
}

fn play(
    file: &ScenarioFile,
    runaway: Faction,
    run: u32,
    extra: u32,
    params: &'static AiParams,
) -> BTreeMap<&'static str, i64> {
    let mut start = file.prepare(false).expect("scenario");
    start.setup.seed = run_seed(start.setup.seed, run);
    start.setup.players[file.seat.0 as usize].difficulty = AiDifficulty::Hard;
    let active = ActiveScenario::new(file.clone(), start.clone());
    let mut session = Session::from_state(start, |_| BrainKind::Scripted);
    session.history = None;
    let seats: Vec<_> = session
        .state
        .players
        .iter()
        .filter(|p| !p.is_human)
        .map(|p| (p.id, p.faction))
        .collect();
    for (id, faction) in seats {
        let difficulty = session.state.setup.players[id.0 as usize].difficulty;
        let seed = legatum_wasm::session::brain_seed(session.state.setup.seed, id);
        session.replace_brain(
            id,
            Box::new(ScriptedBrain::with_params(
                faction, seed, id, difficulty, params,
            )),
        );
    }
    let s0 = session.state.clone();
    let leader = s0
        .players
        .iter()
        .find(|p| p.faction == runaway)
        .expect("runaway faction in the scenario");
    let (lid, lteam) = (leader.id, leader.team);
    let seat = file.seat;
    let seat_team = s0.player(seat).team;
    let leader_cities: BTreeSet<_> = s0.cities_of(lid).map(|c| c.id).collect();
    let mut m: BTreeMap<&'static str, i64> = BTreeMap::new();
    let mut attackers: BTreeSet<PlayerId> = BTreeSet::new();
    let limit = file.objective.turns + 1 + extra;
    let mut played = 0;
    let mut objective = 0;
    let mut seen = BTreeMap::new();
    // Set once a war on the runaway begins during the run (wars under way at the start do not
    // count): Alliances signed before it are blocs ahead of the war.
    let mut war_started = false;
    while played < limit && session.state.players.iter().any(|p| p.id == lid && p.alive) {
        let before = relations(&session.state);
        let orders = session.ai_orders();
        let events = session.step(&orders);
        played += 1;
        track_operations(&session, &mut seen, &mut m);
        if played <= file.objective.turns + 1
            && active.status(&session.state).outcome == legatum_core::scenario::Outcome::Won
        {
            objective = 1;
        }
        for e in &events {
            let Event::Battle {
                attacker_owner,
                defender,
                ..
            } = e
            else {
                continue;
            };
            if attacker_owner.is_barbarian() || session.state.player(*attacker_owner).team == lteam
            {
                continue;
            }
            let on_leader = match defender {
                Defender::City { owner, .. } => *owner == lid,
                Defender::Unit { owner, .. } => *owner == lid,
                #[allow(unreachable_patterns)]
                _ => false,
            };
            if !on_leader {
                continue;
            }
            let city = matches!(defender, Defender::City { .. });
            let by_seat = *attacker_owner == seat;
            *m.entry(match (city, by_seat) {
                (true, true) => "seat_city_battles",
                (true, false) => "third_city_battles",
                (false, true) => "seat_unit_battles",
                (false, false) => "third_unit_battles",
            })
            .or_default() += 1;
            if !by_seat {
                attackers.insert(*attacker_owner);
            }
        }
        let after = relations(&session.state);
        // No war on the runaway yet (before this turn's changes): an Alliance now is a bloc
        // formed ahead of the war.
        let pre_war = !war_started;
        for (k, state) in &after {
            let was = before.get(k).copied();
            if was == Some(*state) {
                continue;
            }
            let involves = |t: TeamId| k.0 == t || k.1 == t;
            match state {
                RelationState::Alliance if involves(seat_team) => {
                    *m.entry("seat_alliances").or_default() += 1;
                    if pre_war && !involves(lteam) {
                        *m.entry("alliances_pre_war").or_default() += 1;
                    }
                }
                RelationState::Alliance => {
                    *m.entry("other_alliances").or_default() += 1;
                    if pre_war && !involves(lteam) {
                        *m.entry("alliances_pre_war").or_default() += 1;
                    }
                }
                RelationState::Peace if was == Some(RelationState::War) && involves(lteam) => {
                    *m.entry("peace_with_runaway").or_default() += 1
                }
                RelationState::War if involves(lteam) => {
                    *m.entry("wars_on_runaway").or_default() += 1;
                    war_started = true;
                }
                _ => {}
            }
        }
    }
    for id in &leader_cities {
        match session.state.cities.get(id) {
            Some(c) if c.owner == seat => *m.entry("taken_by_seat").or_default() += 1,
            Some(c) if c.owner != lid && !c.owner.is_barbarian() => {
                *m.entry("taken_by_third").or_default() += 1
            }
            None => *m.entry("razed").or_default() += 1,
            _ => {}
        }
    }
    m.insert("third_attackers", attackers.len() as i64);
    m.insert("objective", objective);
    m.insert(
        "seat_start_cities_held",
        s0.cities_of(seat)
            .filter(|c| {
                session
                    .state
                    .cities
                    .get(&c.id)
                    .is_some_and(|n| n.owner == seat)
            })
            .count() as i64,
    );
    m.insert(
        "runaway_cities_end",
        session.state.cities_of(lid).count() as i64,
    );
    m.insert(
        "runaway_vp_end",
        session
            .state
            .players
            .iter()
            .find(|p| p.id == lid)
            .map_or(0, |p| {
                legatum_core::scoring::vp(&session.state, p.id)
                    .iter()
                    .map(|(_, vp)| i64::from(*vp))
                    .sum()
            }),
    );
    m.insert("turns", i64::from(played));
    m
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 6 {
        eprintln!(
            "usage: coalition_front_probe SCENARIO RUNAWAY_FACTION RUNS EXTRA_TURNS PARAMS..."
        );
        std::process::exit(2);
    }
    let file = ScenarioFile::parse(&std::fs::read_to_string(&args[1]).expect("scenario"))
        .expect("scenario");
    let runaway: Faction =
        serde_json::from_value(json!(args[2])).expect("faction name, e.g. Mongols");
    let runs: u32 = args[3].parse().expect("runs");
    let extra: u32 = args[4].parse().expect("extra turns");
    for path in &args[5..] {
        let params = load(path);
        let rows: Vec<_> = (0..runs)
            .into_par_iter()
            .map(|run| play(&file, runaway, run, extra, params))
            .collect();
        let mut sum: BTreeMap<&str, i64> = BTreeMap::new();
        let mut any: BTreeMap<&str, i64> = BTreeMap::new();
        for row in &rows {
            for (k, v) in row {
                *sum.entry(k).or_default() += v;
                if *v > 0 {
                    *any.entry(k).or_default() += 1;
                }
            }
        }
        let mean: BTreeMap<_, _> = sum
            .iter()
            .map(|(k, v)| {
                let x100 = v * 100 / i64::from(runs.max(1));
                (*k, format!("{}.{:02}", x100 / 100, x100 % 100))
            })
            .collect();
        println!(
            "{}",
            json!({"params": path, "runs": runs, "mean": mean, "runs_with": any})
        );
    }
}

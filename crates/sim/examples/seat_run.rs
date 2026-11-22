//! Seat run: Hard (the current engine, optionally with an AiParams gene) plays the designer's seat
//! from his recorded position, every other seat Hard with the real opponent memory, to the end of
//! the game (or a cap). Measures how often the seat wins from each branch turn.
//!
//!   seat_run --tape TAPE.json [--states STATES.jsonl] --out RUNS.jsonl [--seat 0] [--turns 2,40,68]
//!            [--salts 64] [--continue hard,params:FILE,...] [--end-turn 260] [--snaps 50,100,150]
//!            [--threads N] [--game LABEL] [--opponent-params FILE]
//!            [--trace DIR --trace-salts 2]
//!
//! `--states` is `audit_states` output (hash-verified original-engine states, one per turn plus the
//! final one); every state read is checked against the tape's hash. If omitted, the same states
//! are replayed from the tape. A file cut after the last
//! branch turn is fine. The seat's shadow Hard brain and every other seat's Hard brain replay the
//! recorded game on those states (orders discarded) so each has its real opponent memory at the
//! branch turn, as in `decision_audit`. Each arm then resolves with this engine: salt s > 0 salts
//! every brain seed and reseeds the dice (the map and objectives stay fixed); salt 0 keeps the
//! recorded seeds. The seat is an AI seat from the branch turn on (no human flag), and its brain is
//! BRAIN: `hard` or `params:FILE` (Hard with that AiParams); `rivals:FILE` keeps the seat Hard and
//! gives only rivals those params; `--opponent-params` sets the rivals' params for every arm
//! unless that arm names its own `rivals:FILE`. A comma list plays every BRAIN on the
//! same branch turns and salts, so a gene's arms and the control share dice and seeds.
//!
//! One JSON line per run in RUNS.jsonl: game, branch_turn, brain, salt, won (the seat's team),
//! winner (faction), winner_seats, win_turn, reason, end_turn, stalled, seat_alive, final vp of
//! every seat, and the seat's `snaps` at each absolute turn of `--snaps` (cities, military units
//! without Scouts, population, vp), taken at the start of that turn when the arm is still running.
//! Deterministic: rows are sorted by (branch_turn, brain, salt).
//!
//! `--trace DIR` also writes, for salts below `--trace-salts` (default 2), one file per run
//! (`DIR/<game>-T<branch>-<brain>-s<salt>.jsonl`): per turn the seat's cities (population,
//! production, item), Settlers and military units alive, wars, its production diagnosis (chosen
//! item, Settler gates, the Settler's score and the top three options; a diagnostic copy, so the
//! run is unchanged), and the seat's plan events, foundings, captures and losses.
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, PlayerId, player_view, resolve_turn, rules::is_military,
    scoring,
};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};

struct Args {
    tape: String,
    states: String,
    out: String,
    seat: PlayerId,
    turns: BTreeSet<u32>,
    salts: u64,
    salt_start: u64,
    brains: Vec<(String, Option<AiParams>, Option<AiParams>)>,
    end_turn: u32,
    snaps: BTreeSet<u32>,
    threads: usize,
    game: String,
    trace: Option<String>,
    trace_salts: u64,
    opponent_params: Option<AiParams>,
}

fn load_params(path: &str) -> AiParams {
    let params: AiParams =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read params"))
            .expect("params json");
    params.validate().expect("legal params");
    params
}

fn parse_args() -> Args {
    let mut args = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        seat: PlayerId(0),
        turns: BTreeSet::from([2]),
        salts: 64,
        salt_start: 0,
        brains: vec![("hard".into(), None, None)],
        end_turn: 260,
        snaps: BTreeSet::from([50, 100, 150]),
        threads: 0,
        game: String::new(),
        trace: None,
        trace_salts: 2,
        opponent_params: None,
    };
    let mut it = std::env::args().skip(1);
    let mut out_dir = None;
    while let Some(flag) = it.next() {
        let mut v = || it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => args.tape = v(),
            "--states" => args.states = v(),
            "--out" => args.out = v(),
            "--out-dir" => out_dir = Some(v()),
            "--status-file" => {
                let _ = v();
            }
            "--label" => args.game = v(),
            "--seat" => args.seat = PlayerId(v().parse().unwrap()),
            "--turns" => args.turns = v().split(',').map(|s| s.parse().unwrap()).collect(),
            "--salts" => args.salts = v().parse().unwrap(),
            "--games" => args.salts = v().parse().unwrap(),
            "--seed" => args.salt_start = v().parse().unwrap(),
            "--continue" => {
                args.brains = v()
                    .split(',')
                    .map(|name| match name.strip_prefix("params:") {
                        Some(p) if !p.is_empty() => (name.to_string(), Some(load_params(p)), None),
                        Some(_) => panic!("--continue params:FILE needs a file"),
                        None if name == "hard" => (name.to_string(), None, None),
                        None => match name.strip_prefix("rivals:") {
                            Some(p) if !p.is_empty() => {
                                (name.to_string(), None, Some(load_params(p)))
                            }
                            _ => panic!("--continue hard|params:FILE|rivals:FILE[,...]"),
                        },
                    })
                    .collect();
            }
            "--end-turn" => args.end_turn = v().parse().unwrap(),
            "--snaps" => args.snaps = v().split(',').map(|s| s.parse().unwrap()).collect(),
            "--threads" => args.threads = v().parse().unwrap(),
            "--game" => args.game = v(),
            "--trace" => args.trace = Some(v()),
            "--trace-salts" => args.trace_salts = v().parse().unwrap(),
            "--opponent-params" => args.opponent_params = Some(load_params(&v())),
            other => panic!("unknown option {other}"),
        }
    }
    if args.out.is_empty()
        && let Some(dir) = out_dir
    {
        args.out = format!("{dir}/seat-run.jsonl");
    }
    assert!(!args.tape.is_empty() && !args.out.is_empty());
    let names: BTreeSet<_> = args.brains.iter().map(|b| &b.0).collect();
    assert_eq!(names.len(), args.brains.len(), "duplicate brain");
    args
}

fn salted(seed: u64, salt: u64) -> u64 {
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

fn hard_brain(
    state: &GameState,
    p: PlayerId,
    params: Option<&AiParams>,
    salt: u64,
) -> ScriptedBrain {
    let seed = salted(legatum_wasm::session::brain_seed(state.setup.seed, p), salt);
    let faction = state.player(p).faction;
    match params {
        Some(pr) => ScriptedBrain::with_params(faction, seed, p, AiDifficulty::Hard, pr),
        None => ScriptedBrain::with_difficulty(faction, seed, p, AiDifficulty::Hard),
    }
}

struct Snapshot {
    state: GameState,
    memory: BTreeMap<PlayerId, OpponentMemory>,
}

/// Replay the tape on the recorded states with every seat's Hard brain shadowing it, and keep
/// the state and every brain's memory at each branch turn.
fn warm(tape: &Tape, args: &Args) -> BTreeMap<u32, Snapshot> {
    let last = *args.turns.iter().max().expect("--turns");
    let reader = std::io::BufReader::new(std::fs::File::open(&args.states).expect("states file"));
    let mut lines = reader.lines();
    let mut brains: Option<BTreeMap<PlayerId, ScriptedBrain>> = None;
    let mut snaps = BTreeMap::new();
    let mut migrated = 0usize;
    let mut read = |expect_hash: &str| -> GameState {
        let line = lines.next().expect("states file too short").unwrap();
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        assert_eq!(hash.to_string(), expect_hash, "recorded state hash");
        let state: GameState = serde_json::from_str(&line).expect("recorded state json");
        migrated += usize::from(state.hash() != hash);
        state
    };
    let mut state = read(&tape.initial_hash.to_string());
    for t in &tape.turns {
        assert_eq!(state.turn, t.turn, "recorded turn sequence");
        let brains = brains.get_or_insert_with(|| {
            state
                .players
                .iter()
                .map(|p| (p.id, hard_brain(&state, p.id, None, 0)))
                .collect()
        });
        if args.turns.contains(&t.turn) {
            snaps.insert(
                t.turn,
                Snapshot {
                    state: state.clone(),
                    memory: brains
                        .iter()
                        .filter_map(|(id, b)| b.opponent_memory().map(|m| (*id, m)))
                        .collect(),
                },
            );
        }
        if t.turn >= last {
            break;
        }
        let alive: Vec<PlayerId> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect();
        for id in alive {
            let _ = brains
                .get_mut(&id)
                .unwrap()
                .orders(&player_view(&state, id));
        }
        state = read(&t.hash);
    }
    eprintln!(
        "seat_run: {} states verified up to T{last}, {migrated} migrated by serde aliases, {} branch positions",
        tape.turns.iter().filter(|t| t.turn <= last).count(),
        snaps.len()
    );
    for t in &args.turns {
        assert!(snaps.contains_key(t), "branch turn {t} is not on the tape");
    }
    snaps
}

fn seat_line(state: &GameState, seat: PlayerId) -> Value {
    json!({
        "cities": state.cities_of(seat).count(),
        "mil": state.units.values().filter(|u| u.owner == seat && is_military(u.kind)
            && u.kind != legatum_core::UnitKind::Scout).count(),
        "pop": state.cities_of(seat).map(|c| c.population).sum::<i32>(),
        "vp": scoring::vp_total(state, seat),
    })
}

/// Whether an event is about the seat (foundings, captures, wars, peace, Flower Wars, losses).
fn seat_event(e: &Event, seat: PlayerId) -> bool {
    let kind = matches!(
        e,
        Event::CityFounded { .. }
            | Event::CityCaptured { .. }
            | Event::CityRazed { .. }
            | Event::SettlerCaptured { .. }
            | Event::WarDeclared { .. }
            | Event::PeaceEstablished { .. }
            | Event::FlowerWars { .. }
            | Event::PlayerEliminated { .. }
    );
    let s = json!(e).to_string();
    let id = seat.0;
    kind && (s.contains(&format!(":{id},")) || s.contains(&format!(":{id}}}")))
}

/// One trace line: the seat's empire and its production diagnosis this turn.
fn trace_turn(
    out: &mut impl Write,
    state: &GameState,
    seat: PlayerId,
    brain: &ScriptedBrain,
    plan_seen: &mut usize,
) {
    if !state.player(seat).alive {
        return;
    }
    let view = player_view(state, seat);
    let (_, diag) = brain.production_diagnosis(&view);
    let cities: Vec<Value> = state
        .cities_of(seat)
        .map(|c| {
            let y = legatum_core::economy::city_yields(state, c);
            let walls_buildable = view.cities.iter().any(|vc| {
                vc.id == c.id
                    && vc.detail.as_ref().is_some_and(|d| {
                        d.options.iter().any(|o| {
                            o.item
                                == legatum_core::ProductionItem::Building {
                                    building: legatum_core::BuildingKind::Walls,
                                }
                        })
                    })
            });
            let walled = view.cities.iter().any(|vc| vc.id == c.id && vc.walls);
            json!([
                c.id.0,
                c.population,
                y.total.production,
                c.producing,
                walled,
                walls_buildable
            ])
        })
        .collect();
    let units = |k: fn(legatum_core::UnitKind) -> bool| {
        state
            .units
            .values()
            .filter(|u| u.owner == seat && k(u.kind))
            .count()
    };
    let military: Vec<Value> = state
        .units
        .values()
        .filter(|u| {
            u.owner == seat && is_military(u.kind) && u.kind != legatum_core::UnitKind::Scout
        })
        .map(|u| json!([u.id.0, u.kind, u.pos, u.hp]))
        .collect();
    let diag: Vec<Value> = diag
        .iter()
        .map(|t| {
            let mut opts = t.options.clone();
            opts.sort_by_key(|a| std::cmp::Reverse(a.2));
            let settler = t.options.iter().find(|o| {
                o.0 == legatum_core::ProductionItem::Unit {
                    unit: legatum_core::UnitKind::Settler,
                }
            });
            json!({"city": t.city.0, "chosen": t.chosen, "gates": t.settler.as_ref().map(|g| json!({
                "threatened": g.threatened, "pop_ok": g.pop_ok, "want": g.want, "time": g.time,
                "room": g.room, "developed": g.developed, "army_ok": g.army_ok, "sites_ok": g.sites_ok,
                "pipeline_ok": g.pipeline_ok, "value": g.value})),
                "settler": settler, "top": opts.into_iter().take(3).collect::<Vec<_>>()})
        })
        .collect();
    let wars: Vec<u8> = state
        .players
        .iter()
        .filter(|p| p.alive && p.id != seat && state.at_war(seat, p.id))
        .map(|p| p.id.0)
        .collect();
    let peace: Vec<Value> = wars
        .iter()
        .map(|id| {
            let team = state.player(PlayerId(*id)).team;
            let d = brain.peace_diagnosis(&view, team);
            json!({"opponent": id, "wants": d.wants, "gate": d.gate,
            "score": d.score, "plan": d.plan, "terms": d.terms,
            "endangered": d.endangered})
        })
        .collect();
    let own: Vec<legatum_core::Hex> = state.cities_of(seat).map(|c| c.pos).collect();
    let known: Vec<Value> = view
        .cities
        .iter()
        .filter(|c| c.owner != seat)
        .map(|c| {
            let d = own.iter().map(|h| h.distance(c.pos)).min().unwrap_or(99);
            json!([c.owner.0, d, c.walls, c.population])
        })
        .collect();
    let plans = brain.plan_events();
    let new_plans = json!(plans[(*plan_seen).min(plans.len())..]);
    *plan_seen = plans.len();
    writeln!(
        out,
        "{}",
        json!({"turn": state.turn, "cities": cities,
            "settlers": units(|k| k == legatum_core::UnitKind::Settler),
            "mil": units(|k| is_military(k) && k != legatum_core::UnitKind::Scout),
            "mil_units": military,
            "gold": state.player(seat).gold, "wars": wars, "peace": peace,
            "known": known, "plans": new_plans, "prod": diag})
    )
    .unwrap();
}

fn play(
    snap: &Snapshot,
    args: &Args,
    game: &str,
    brain: &(String, Option<AiParams>, Option<AiParams>),
    salt: u64,
) -> Value {
    let mut state = snap.state.clone();
    let seat = args.seat;
    let start = state.turn;
    if salt != 0 {
        state.dice_seed = Some(salted(state.setup.seed, salt ^ 0xD1CE));
    }
    state.players[seat.0 as usize].is_human = false;
    if let Some(p) = state.setup.players.get_mut(seat.0 as usize) {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
    }
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let params = if p.id == seat {
                brain.1.as_ref()
            } else {
                brain.2.as_ref().or(args.opponent_params.as_ref())
            };
            let mut b = hard_brain(&state, p.id, params, salt);
            if let Some(m) = snap.memory.get(&p.id) {
                b.restore_opponent_memory(m.clone());
            }
            (p.id, b)
        })
        .collect();
    let mut snaps = serde_json::Map::new();
    let (mut win_turn, mut reason) = (None, None);
    let mut trace = args
        .trace
        .as_ref()
        .filter(|_| salt < args.trace_salts)
        .map(|dir| {
            std::fs::create_dir_all(dir).unwrap();
            let name = brain.0.replace(['/', ':'], "_");
            std::io::BufWriter::new(
                std::fs::File::create(format!("{dir}/{game}-T{start}-{name}-s{salt}.jsonl"))
                    .unwrap(),
            )
        });
    let mut plan_seen = 0usize;
    // Placement telemetry: the seat's foundings ([turn, q, r]) and cities it lost within 30
    // turns of their founding.
    let mut foundings: Vec<[i32; 3]> = Vec::new();
    let mut young_lost = 0u32;
    while !state.finished && state.turn <= args.end_turn {
        if args.snaps.contains(&state.turn) {
            snaps.insert(state.turn.to_string(), seat_line(&state, seat));
        }
        if let Some(out) = trace.as_mut() {
            trace_turn(out, &state, seat, &brains[&seat], &mut plan_seen);
        }
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                let view = player_view(&state, *id);
                if *id == seat && trace.is_some() {
                    let (chosen, decision) = b.order_diagnosis(&view);
                    if let Some(out) = trace.as_mut() {
                        writeln!(
                            out,
                            "{}",
                            json!({
                                "turn": state.turn, "seat": id.0,
                                "faction": format!("{:?}", state.player(*id).faction),
                                "decisions": decision.decisions,
                                "passes": decision.passes,
                                "city_passes": decision.city_passes,
                                "orders": &chosen,
                            })
                        )
                        .unwrap();
                    }
                    orders.insert(*id, chosen);
                } else {
                    orders.insert(*id, b.orders(&view));
                }
            }
        }
        let turn = state.turn;
        let (next, events) = resolve_turn(&state, &orders);
        state = next;
        for e in &events {
            match e {
                Event::GameWon { reason: r, .. } => {
                    win_turn.get_or_insert(turn);
                    reason.get_or_insert(r.clone());
                }
                Event::CityFounded { owner, at, .. } if *owner == seat => {
                    foundings.push([turn as i32, at.q, at.r]);
                }
                Event::CityCaptured { city, from, .. }
                    if *from == seat
                        && state
                            .cities
                            .get(city)
                            .is_some_and(|c| turn.saturating_sub(c.founded_turn) <= 30) =>
                {
                    young_lost += 1;
                }
                _ => {}
            }
        }
        if let Some(out) = trace.as_mut() {
            let seen: Vec<Value> = events
                .iter()
                .filter(|e| seat_event(e, seat))
                .map(|e| json!(e))
                .collect();
            if !seen.is_empty() {
                writeln!(out, "{}", json!({"turn": turn, "events": seen})).unwrap();
            }
        }
    }
    let team = state.team_of(seat);
    let won = state.finished && state.winner == Some(team);
    let winner_seats: Vec<u8> = match state.winner {
        Some(w) if state.finished => state
            .players
            .iter()
            .filter(|p| p.team == w)
            .map(|p| p.id.0)
            .collect(),
        _ => vec![],
    };
    json!({
        "game": game, "branch_turn": start, "brain": brain.0, "salt": salt,
        "won": won, "winner_seats": winner_seats,
        "winner": winner_seats.first().map(|s| format!("{:?}", state.players[*s as usize].faction)),
        "win_turn": win_turn, "reason": reason, "end_turn": state.turn,
        "stalled": !state.finished, "seat_alive": state.player(seat).alive,
        "vp": state.players.iter().map(|p| scoring::vp_total(&state, p.id)).collect::<Vec<_>>(),
        "seat_final": seat_line(&state, seat), "snaps": snaps,
        "foundings": foundings, "young_cities_lost": young_lost,
    })
}

fn main() {
    let mut args = parse_args();
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .unwrap();
    }
    let tape =
        Tape::from_json(&std::fs::read_to_string(&args.tape).expect("read tape")).expect("tape");
    if args.states.is_empty() {
        let path =
            std::env::temp_dir().join(format!("seat-run-states-{}.jsonl", std::process::id()));
        let mut state = tape.initial_state();
        let mut file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
        for turn in &tape.turns {
            assert_eq!(state.turn, turn.turn);
            writeln!(file, "{}", serde_json::to_string(&state).unwrap()).unwrap();
            state = resolve_turn(&state, &turn.orders).0;
            assert_eq!(
                state.hash().to_string(),
                turn.hash,
                "replay drift at T{}",
                turn.turn
            );
        }
        writeln!(file, "{}", serde_json::to_string(&state).unwrap()).unwrap();
        file.flush().unwrap();
        args.states = path.to_string_lossy().into_owned();
    }
    let snaps = warm(&tape, &args);
    let game = if args.game.is_empty() {
        std::path::Path::new(&args.tape)
            .file_stem()
            .expect("tape file name")
            .to_string_lossy()
            .into_owned()
    } else {
        args.game.clone()
    };
    let mut jobs: Vec<(u32, usize, u64)> = Vec::new();
    for &t in &args.turns {
        for b in 0..args.brains.len() {
            for salt in args.salt_start..args.salt_start + args.salts {
                jobs.push((t, b, salt));
            }
        }
    }
    let rows: Vec<Value> = jobs
        .par_iter()
        .map(|&(t, b, salt)| play(&snaps[&t], &args, &game, &args.brains[b], salt))
        .collect();
    let out_path = if std::path::Path::new(&args.out).is_dir() {
        std::path::Path::new(&args.out).join("seat-run.jsonl")
    } else {
        std::path::PathBuf::from(&args.out)
    };
    let mut out = std::io::BufWriter::new(std::fs::File::create(&out_path).unwrap());
    for row in &rows {
        writeln!(out, "{row}").unwrap();
    }
    let won = rows.iter().filter(|r| r["won"] == true).count();
    eprintln!(
        "seat_run: {} runs, {won} won by the seat, rows in {}",
        rows.len(),
        args.out
    );
}

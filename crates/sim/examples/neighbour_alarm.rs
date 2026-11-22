//! Neighbour alarm: what the designer's next victims and bystanders do after his first
//! capture, and how long his next conquest takes, on his own recorded game.
//!
//!   neighbour_alarm --tape TAPE.json --states STATES.jsonl --branch 18 --end 60 --out RUNS.jsonl
//!                   [--seat 0] [--salts 8] [--reseed]
//!                   [--arms control,NAME=PARAMS.json,NAME@gene=1+gene2=0,...]
//!                   [--trace DIR] [--watch 2,3,5]
//!
//! The states are `audit_states` output at the tape's release tag (hash-checked); every seat's
//! Hard brain shadows the recorded game up to `--branch` so it has its real opponent memory.
//! From the branch on the designer's seat plays his recorded orders (the tape seat: orders that
//! no longer apply are refused by the engine) and every other seat plays this engine's Hard,
//! with the arm's AiParams (`control` = shipped Hard). Salt s > 0 salts every AI brain seed;
//! `--reseed` also reseeds the dice (the tape seat drifts sooner). `--hard-seat` plays the
//! designer's seat with shipped Hard (warmed memory) instead of his orders, in every arm.
//!
//! One JSON line per run: per rival, the turn its first city fell to the seat and the seat's
//! military units destroyed within 3 hexes of that rival's cities before it; per watched rival
//! and turn (with `--trace`), its Gold, cities (walls, item), purchases, completions, military
//! units by kind, public army and the alarm diagnosis per city ([warning, local defence,
//! exposure, border threat], the first being the conqueror alarm and the second the warning that
//! positions units).
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, PlayerId, player_view, resolve_turn, rules::is_military,
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
    branch: u32,
    end: u32,
    salts: u64,
    reseed: bool,
    hard_seat: bool,
    arms: Vec<(String, Option<AiParams>)>,
    trace: Option<String>,
    watch: BTreeSet<u8>,
    diag: Option<(u8, u32, u32)>,
}

fn parse_args() -> Args {
    let mut a = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        seat: PlayerId(0),
        branch: 18,
        end: 60,
        salts: 8,
        reseed: false,
        hard_seat: false,
        arms: vec![("control".into(), None)],
        trace: None,
        watch: BTreeSet::new(),
        diag: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        if flag == "--reseed" {
            a.reseed = true;
            continue;
        }
        if flag == "--hard-seat" {
            a.hard_seat = true;
            continue;
        }
        let v = it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => a.tape = v,
            "--states" => a.states = v,
            "--out" => a.out = v,
            "--seat" => a.seat = PlayerId(v.parse().unwrap()),
            "--branch" => a.branch = v.parse().unwrap(),
            "--end" => a.end = v.parse().unwrap(),
            "--salts" => a.salts = v.parse().unwrap(),
            "--trace" => a.trace = Some(v),
            "--diag" => {
                let (p, r) = v.split_once(':').expect("--diag P:T0-T1");
                let (t0, t1) = r.split_once('-').expect("--diag P:T0-T1");
                a.diag = Some((p.parse().unwrap(), t0.parse().unwrap(), t1.parse().unwrap()));
            }
            "--watch" => a.watch = v.split(',').map(|s| s.parse().unwrap()).collect(),
            "--arms" => {
                a.arms = v
                    .split(',')
                    .map(|s| match s.split_once('=') {
                        None => (s.to_string(), None),
                        Some(_) if s.contains('@') => {
                            let (name, genes) = s.split_once('@').unwrap();
                            let mut v = serde_json::to_value(AiParams::builtin()).unwrap();
                            for g in genes.split('+') {
                                let (k, x) = g.split_once('=').expect("gene=value");
                                assert!(v["decisions"].get(k).is_some(), "unknown gene {k}");
                                v["decisions"][k] = json!(x.parse::<i64>().unwrap());
                            }
                            let p: AiParams = serde_json::from_value(v).expect("params");
                            p.validate().expect("legal params");
                            (name.to_string(), Some(p))
                        }
                        Some((name, path)) => {
                            let p: AiParams = serde_json::from_str(
                                &std::fs::read_to_string(path).expect("read params"),
                            )
                            .expect("params json");
                            p.validate().expect("legal params");
                            (name.to_string(), Some(p))
                        }
                    })
                    .collect()
            }
            other => panic!("unknown option {other}"),
        }
    }
    assert!(!a.tape.is_empty() && !a.states.is_empty() && !a.out.is_empty());
    a
}

fn salted(seed: u64, salt: u64) -> u64 {
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

fn brain(state: &GameState, p: PlayerId, params: Option<&AiParams>, salt: u64) -> ScriptedBrain {
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

fn warm(tape: &Tape, args: &Args) -> Snapshot {
    let reader = std::io::BufReader::new(std::fs::File::open(&args.states).expect("states file"));
    let mut lines = reader.lines();
    let mut read = |expect: &str| -> GameState {
        let line = lines.next().expect("states file too short").unwrap();
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        assert_eq!(hash.to_string(), expect, "recorded state hash");
        serde_json::from_str(&line).expect("recorded state json")
    };
    let mut state = read(&tape.initial_hash.to_string());
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .map(|p| (p.id, brain(&state, p.id, None, 0)))
        .collect();
    for t in &tape.turns {
        if t.turn == args.branch {
            return Snapshot {
                memory: brains
                    .iter()
                    .filter_map(|(id, b)| b.opponent_memory().map(|m| (*id, m)))
                    .collect(),
                state,
            };
        }
        for p in state.players.iter().filter(|p| p.alive) {
            let _ = brains
                .get_mut(&p.id)
                .unwrap()
                .orders(&player_view(&state, p.id));
        }
        state = read(&t.hash);
    }
    panic!("branch turn {} is not on the tape", args.branch);
}

fn watch_line(state: &GameState, p: PlayerId, b: &ScriptedBrain) -> Value {
    let view = player_view(state, p);
    let alarm: BTreeMap<String, [i32; 5]> = b
        .alarm_diagnosis(&view)
        .into_iter()
        .map(|(c, a)| (c.0.to_string(), a))
        .collect();
    let cities: Vec<Value> = state
        .cities_of(p)
        .map(|c| {
            let vc = view.cities.iter().find(|vc| vc.id == c.id);
            let walled = vc.is_some_and(|vc| vc.walls);
            let opts: Vec<Value> = vc
                .and_then(|vc| vc.detail.as_ref())
                .map(|d| {
                    d.options
                        .iter()
                        .filter(|o| match o.item {
                            legatum_core::ProductionItem::Unit { unit } => is_military(unit),
                            legatum_core::ProductionItem::Building { building } => {
                                building == legatum_core::BuildingKind::Walls
                            }
                            _ => false,
                        })
                        .map(|o| json!([o.item, o.turns]))
                        .collect()
                })
                .unwrap_or_default();
            let prod = legatum_core::economy::city_yields(state, c)
                .total
                .production;
            json!([c.id.0, c.pos, c.population, walled, c.producing, prod, opts])
        })
        .collect();
    let mut kinds: BTreeMap<String, u32> = BTreeMap::new();
    for u in state
        .units
        .values()
        .filter(|u| u.owner == p && is_military(u.kind))
    {
        *kinds.entry(format!("{:?}", u.kind)).or_default() += 1;
    }
    json!({"gold": state.player(p).gold, "army": view.army_of(p), "cities": cities,
        "mil": kinds, "alarm": alarm})
}

fn play(
    snap: &Snapshot,
    tape: &Tape,
    args: &Args,
    arm: &(String, Option<AiParams>),
    salt: u64,
) -> Value {
    let mut state = snap.state.clone();
    let seat = args.seat;
    if salt != 0 && args.reseed {
        state.dice_seed = Some(salted(state.setup.seed, salt ^ 0xD1CE));
    }
    if args.hard_seat {
        state.players[seat.0 as usize].is_human = false;
        if let Some(p) = state.setup.players.get_mut(seat.0 as usize) {
            p.is_human = false;
            p.difficulty = AiDifficulty::Hard;
        }
    }
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| p.alive && (p.id != seat || args.hard_seat))
        .map(|p| {
            // The seat (with `--hard-seat`) is shipped Hard in every arm: only rivals change.
            let params = if p.id == seat { None } else { arm.1.as_ref() };
            let mut b = brain(&state, p.id, params, salt);
            if let Some(m) = snap.memory.get(&p.id) {
                b.restore_opponent_memory(m.clone());
            }
            (p.id, b)
        })
        .collect();
    let recorded: BTreeMap<u32, _> = tape
        .turns
        .iter()
        .map(|t| (t.turn, t.orders.get(&seat).cloned().unwrap_or_default()))
        .collect();
    let mut trace = args.trace.as_ref().filter(|_| salt < 2).map(|dir| {
        std::fs::create_dir_all(dir).unwrap();
        std::io::BufWriter::new(
            std::fs::File::create(format!("{dir}/T{}-{}-s{salt}.jsonl", args.branch, arm.0))
                .unwrap(),
        )
    });
    let mut fell: BTreeMap<u8, u32> = BTreeMap::new();
    let mut seat_losses: Vec<(u32, legatum_core::Hex)> = Vec::new();
    let mut lost_near: BTreeMap<u8, u32> = BTreeMap::new();
    let mut captures: Vec<Value> = Vec::new();
    let mut at100: Option<Value> = None;
    let mut curve: Vec<(u32, usize, u32)> = Vec::new();
    while !state.finished && state.turn <= args.end {
        if state.turn.is_multiple_of(10) {
            curve.push((
                state.turn,
                state.cities_of(seat).count(),
                legatum_core::scoring::vp_total(&state, seat),
            ));
        }
        if state.turn == 100 && at100.is_none() {
            at100 = Some(json!({"seat_cities": state.cities_of(seat).count(),
                "vp": state.players.iter().map(|p| legatum_core::scoring::vp_total(&state, p.id)).collect::<Vec<_>>()}));
        }
        let turn = state.turn;
        let mut orders = legatum_wasm::session::OrderMap::new();
        if state.player(seat).alive && !args.hard_seat {
            orders.insert(seat, recorded.get(&turn).cloned().unwrap_or_default());
        }
        let mut lines = serde_json::Map::new();
        for (id, b) in brains.iter_mut() {
            if !state.player(*id).alive {
                continue;
            }
            if *id == seat {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
                continue;
            }
            if trace.is_some() && args.watch.contains(&id.0) {
                lines.insert(id.0.to_string(), watch_line(&state, *id, b));
            }
            let view = player_view(&state, *id);
            match args.diag {
                Some((p, t0, t1)) if p == id.0 && (t0..=t1).contains(&turn) && salt == 0 => {
                    let (o, tr) = b.order_diagnosis(&view);
                    eprintln!(
                        "DIAG T{turn} P{p} {}",
                        json!({"decisions": tr.decisions,
                        "production": tr.production, "defense": tr.production_defense,
                        "city_passes": tr.city_passes, "executed": tr.executed})
                    );
                    orders.insert(*id, o);
                }
                _ => {
                    orders.insert(*id, b.orders(&view));
                }
            }
        }
        let rival_cities: BTreeMap<u8, Vec<legatum_core::Hex>> = state
            .players
            .iter()
            .filter(|p| p.id != seat)
            .map(|p| (p.id.0, state.cities_of(p.id).map(|c| c.pos).collect()))
            .collect();
        let (next, events) = resolve_turn(&state, &orders);
        let mut ev: Vec<Value> = Vec::new();
        for e in &events {
            match e {
                Event::CityCaptured { city, from, to, .. } if *to == seat => {
                    fell.entry(from.0).or_insert(turn);
                    captures.push(json!([turn, from.0, city.0]));
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind,
                    at,
                    ..
                } if *owner == seat && is_military(*unit_kind) => {
                    seat_losses.push((turn, *at));
                    for (r, cs) in &rival_cities {
                        if !fell.contains_key(r) && cs.iter().any(|c| c.distance(*at) <= 3) {
                            *lost_near.entry(*r).or_default() += 1;
                        }
                    }
                }
                Event::ItemBought { owner, .. } | Event::ProductionCompleted { owner, .. }
                    if args.watch.contains(&owner.0) =>
                {
                    ev.push(json!(e));
                }
                Event::WarDeclared { .. } | Event::PeaceEstablished { .. } => ev.push(json!(e)),
                _ => {}
            }
        }
        if let Some(out) = trace.as_mut() {
            writeln!(
                out,
                "{}",
                json!({"turn": turn, "watch": lines, "events": ev})
            )
            .unwrap();
        }
        state = next;
    }
    json!({"branch": args.branch, "arm": arm.0, "salt": salt, "end": state.turn,
        "first_fall": fell, "seat_lost_near": lost_near, "seat_lost": seat_losses.len(),
        "captures": captures,
        "at100": at100, "seat_curve": curve,
        "winner": state.winner,
        "vp": state.players.iter().map(|p| legatum_core::scoring::vp_total(&state, p.id)).collect::<Vec<_>>(),
        "seat_cities": state.cities_of(seat).count(),
        "cities": state.players.iter().map(|p| state.cities_of(p.id).count()).collect::<Vec<_>>(),
        "pop": state.players.iter().map(|p| state.cities_of(p.id).map(|c| c.population).sum::<i32>()).collect::<Vec<_>>(),
        "mil": state.players.iter().map(|p| state.units.values()
            .filter(|u| u.owner == p.id && is_military(u.kind)).count()).collect::<Vec<_>>()})
}

fn main() {
    let args = parse_args();
    let tape =
        Tape::from_json(&std::fs::read_to_string(&args.tape).expect("read tape")).expect("tape");
    let snap = warm(&tape, &args);
    eprintln!("neighbour_alarm: warmed to T{}", snap.state.turn);
    let jobs: Vec<(usize, u64)> = (0..args.arms.len())
        .flat_map(|a| (0..args.salts).map(move |s| (a, s)))
        .collect();
    let rows: Vec<Value> = jobs
        .par_iter()
        .map(|&(a, s)| play(&snap, &tape, &args, &args.arms[a], s))
        .collect();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args.out).unwrap());
    for r in &rows {
        writeln!(out, "{r}").unwrap();
    }
}

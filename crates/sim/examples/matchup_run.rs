//! Matchup run: the designer's recorded attacks (his taped orders) against Hard defenders, to
//! measure what the defenders buy and build under attack and what it costs him.
//!
//!   matchup_run --tape TAPE.json --states STATES.jsonl --out RUNS.jsonl [--him 0]
//!               [--turns 30,36] [--salts 16] [--arms hard,params:FILE,...] [--horizon 15]
//!               [--him-brain tape|hard] [--threads N]
//!
//! `--states` is `audit_states` output from the tape's release tag (hash-checked against the
//! tape), so a tape recorded on older rules can branch on this engine. Every seat's Hard brain
//! shadows the recorded game up to the branch turn (so each has its real opponent memory, as in
//! `seat_run`). From the branch, every AI seat is Hard with the arm's params; his seat replays
//! his recorded orders for each turn (`tape`; orders that no longer apply are refused by the
//! engine) or is played by Hard (`hard`). Salt 0 keeps the recorded dice; salt s > 0 reseeds
//! the dice and salts every brain seed.
//!
//! One JSON line per run: his military units lost (no Scouts) and when, his captures (turn,
//! city, from), the AI's military losses, every attack of his on a city, and every military
//! purchase or production change an AI made in a city with one of his units within 4 hexes,
//! with the planning pass that bought it (`legatum_ai::buy_trace`), the city's walls and HP,
//! and the enemy kinds that AI could see within 4 hexes.
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain, buy_trace, city_siege_times};
use legatum_core::events::Defender;
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, Order, PlayerId, ProductionItem, player_view, resolve_turn,
    rules::is_military,
};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::time::Instant;

type Arm = (String, Option<AiParams>);

struct Args {
    tape: String,
    states: String,
    out: String,
    him: PlayerId,
    turns: BTreeSet<u32>,
    salts: u64,
    arms: Vec<Arm>,
    horizon: u32,
    him_brain: String,
    threads: usize,
}

fn load_params(path: &str) -> AiParams {
    let params: AiParams =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read params"))
            .expect("params json");
    params.validate().expect("legal params");
    params
}

fn parse_args() -> Args {
    let mut a = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        him: PlayerId(0),
        turns: BTreeSet::new(),
        salts: 16,
        arms: vec![("hard".into(), None)],
        horizon: 15,
        him_brain: "tape".into(),
        threads: 0,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut v = || it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => a.tape = v(),
            "--states" => a.states = v(),
            "--out" => a.out = v(),
            "--him" => a.him = PlayerId(v().parse().unwrap()),
            "--turns" => a.turns = v().split(',').map(|s| s.parse().unwrap()).collect(),
            "--salts" => a.salts = v().parse().unwrap(),
            "--horizon" => a.horizon = v().parse().unwrap(),
            "--him-brain" => a.him_brain = v(),
            "--threads" => a.threads = v().parse().unwrap(),
            "--arms" => {
                a.arms = v()
                    .split(',')
                    .map(|name| match name.strip_prefix("params:") {
                        Some(p) => (name.to_string(), Some(load_params(p))),
                        None if name == "hard" => (name.to_string(), None),
                        None => panic!("--arms hard|params:FILE[,...]"),
                    })
                    .collect()
            }
            other => panic!("unknown option {other}"),
        }
    }
    assert!(!a.tape.is_empty() && !a.states.is_empty() && !a.out.is_empty());
    assert!(!a.turns.is_empty(), "--turns");
    a
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
    let seed = salted(state.setup.seed ^ u64::from(p.0), salt);
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

/// `seat_run`'s warm-up: replay the recorded states with every seat's Hard brain shadowing.
fn warm(tape: &Tape, args: &Args) -> BTreeMap<u32, Snapshot> {
    let last = *args.turns.iter().max().unwrap();
    let reader = std::io::BufReader::new(std::fs::File::open(&args.states).expect("states file"));
    let mut lines = reader.lines();
    let mut brains: Option<BTreeMap<PlayerId, ScriptedBrain>> = None;
    let mut snaps = BTreeMap::new();
    let mut read = |expect_hash: &str| -> GameState {
        let line = lines.next().expect("states file too short").unwrap();
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        assert_eq!(hash.to_string(), expect_hash, "recorded state hash");
        serde_json::from_str(&line).expect("recorded state json")
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
    snaps
}

fn item_name(item: &ProductionItem) -> String {
    match item {
        ProductionItem::Unit { unit } => format!("{unit:?}"),
        other => format!("{other:?}"),
    }
}

fn play(
    snap: &Snapshot,
    args: &Args,
    tape_orders: &BTreeMap<u32, Vec<Order>>,
    arm: &Arm,
    salt: u64,
) -> Value {
    buy_trace::enable();
    let _ = buy_trace::take();
    let mut state = snap.state.clone();
    let him = args.him;
    let start = state.turn;
    if salt != 0 {
        state.dice_seed = Some(salted(state.setup.seed, salt ^ 0xD1CE));
    }
    let hard_him = args.him_brain == "hard";
    if hard_him {
        state.players[him.0 as usize].is_human = false;
        if let Some(p) = state.setup.players.get_mut(him.0 as usize) {
            p.is_human = false;
            p.difficulty = AiDifficulty::Hard;
        }
    }
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| p.alive && (p.id != him || hard_him))
        .map(|p| {
            let params = if p.id == him { None } else { arm.1.as_ref() };
            let mut b = hard_brain(&state, p.id, params, salt);
            if let Some(m) = snap.memory.get(&p.id) {
                b.restore_opponent_memory(m.clone());
            }
            (p.id, b)
        })
        .collect();
    let mut lost: Vec<(u32, String)> = Vec::new();
    let mut ai_lost: Vec<(u32, u8, String)> = Vec::new();
    let mut captures: Vec<(u32, u32, u8)> = Vec::new();
    let mut city_attacks: BTreeMap<u32, u32> = BTreeMap::new();
    let mut buys: Vec<Value> = Vec::new();
    let mut switches: Vec<Value> = Vec::new();
    let mut ranged_finishes: Vec<Value> = Vec::new();
    let mut diags: Vec<Value> = Vec::new();
    let mut forecasts: Vec<Value> = Vec::new();
    let mut hard_turn_us: Vec<u64> = Vec::new();
    let (mut his_attacks, mut dealt_to_him) = (0u32, 0i32);
    let mut shots: Vec<(u32, String, bool, i32)> = Vec::new();
    let end = start + args.horizon;
    while !state.finished && state.turn < end {
        let turn = state.turn;
        let threatened_builds: Vec<_> = state
            .cities
            .values()
            .filter(|c| c.owner != him && c.producing.is_some())
            .filter(|c| {
                state
                    .units
                    .values()
                    .any(|u| u.owner == him && is_military(u.kind) && u.pos.distance(c.pos) <= 4)
            })
            .map(|c| {
                (
                    c.id,
                    c.owner,
                    c.producing.unwrap(),
                    legatum_core::data::has_building(
                        &c.buildings,
                        legatum_core::BuildingKind::Walls,
                    ),
                )
            })
            .collect();
        let mut switch_passes: BTreeMap<u32, Vec<&'static str>> = BTreeMap::new();
        let mut orders = legatum_wasm::session::OrderMap::new();
        if !hard_him {
            orders.insert(him, tape_orders.get(&turn).cloned().unwrap_or_default());
        }
        for (id, b) in brains.iter_mut() {
            if !state.player(*id).alive {
                continue;
            }
            let view = player_view(&state, *id);
            if *id != him {
                let params = arm.1.clone().unwrap_or_else(AiParams::builtin);
                let horizon = if params.decisions(view.faction_of(*id)).siege_survival_clock > 0 {
                    8
                } else {
                    1
                };
                for city in view.cities.iter().filter(|c| c.owner == *id) {
                    if state.units.values().any(|u| {
                        u.owner == him && is_military(u.kind) && u.pos.distance(city.pos) <= 6
                    }) {
                        let (falls, deadline) = city_siege_times(&view, city, horizon);
                        forecasts.push(json!({"turn": turn, "ai": id.0, "city": city.id.0,
                            "falls_in": falls, "deadline_in": deadline}));
                    }
                }
            }
            let diag_on = salt == 0 && std::env::var("MATCHUP_DIAG").is_ok() && *id != him;
            if diag_on {
                let (_, diag) = b.production_diagnosis(&view);
                for t in diag {
                    let Some(c) = view.cities.iter().find(|c| c.id == t.city) else {
                        continue;
                    };
                    if !state.units.values().any(|u| {
                        u.owner == him && is_military(u.kind) && u.pos.distance(c.pos) <= 4
                    }) {
                        continue;
                    }
                    let mut opts = t.options.clone();
                    opts.sort_by_key(|o| std::cmp::Reverse(o.2));
                    let top: Vec<String> = opts
                        .iter()
                        .take(6)
                        .map(|o| format!("{}={}/{}", item_name(&o.0), o.1, o.2))
                        .collect();
                    diags.push(json!({"turn": turn, "ai": id.0, "city": c.id.0,
                        "chosen": t.chosen.as_ref().map(item_name), "top": top,
                        "gates": t.gates.iter().map(|g| format!("{}:{}", item_name(&g.0), g.1)).collect::<Vec<_>>()}));
                }
            }
            let started = Instant::now();
            let (os, trace) = b.order_diagnosis(&view);
            if *id != him {
                hard_turn_us.push(started.elapsed().as_micros() as u64);
            }
            let tags = buy_trace::take();
            if *id != him {
                for o in &os {
                    if let Order::SetProduction { city, item } = o
                        && let Some(c) = view.cities.iter().find(|c| c.id == *city)
                        && let Some(current) = c.detail.as_ref().and_then(|d| d.producing)
                        && current != *item
                    {
                        switch_passes.insert(
                            city.0,
                            trace.city_passes.get(&city.0).cloned().unwrap_or_default(),
                        );
                    }
                    let (city, item, kind) = match o {
                        Order::BuyItem { city, item } => (*city, *item, "buy"),
                        Order::SetProduction { city, item } => (*city, *item, "build"),
                        _ => continue,
                    };
                    let defensive = match item {
                        ProductionItem::Unit { unit } => is_military(unit),
                        ProductionItem::Building { building } => {
                            legatum_core::data::building_family(building)
                                == legatum_core::BuildingKind::Walls
                        }
                        _ => false,
                    };
                    if !defensive {
                        continue;
                    }
                    let Some(c) = view.cities.iter().find(|c| c.id == city) else {
                        continue;
                    };
                    let near_him = state.units.values().any(|u| {
                        u.owner == him && is_military(u.kind) && u.pos.distance(c.pos) <= 4
                    });
                    if !near_him {
                        continue;
                    }
                    let seen: Vec<String> = view
                        .units
                        .iter()
                        .filter(|u| u.owner == him && u.military && u.pos.distance(c.pos) <= 4)
                        .map(|u| format!("{:?}", u.kind))
                        .collect();
                    let own: Vec<String> = view
                        .units
                        .iter()
                        .filter(|u| u.owner == *id && u.military && u.pos.distance(c.pos) <= 1)
                        .map(|u| format!("{:?}@{}", u.kind, u.pos.distance(c.pos)))
                        .collect();
                    let pass = tags
                        .iter()
                        .find(|t| t.city == city && t.item == item)
                        .map(|t| t.pass)
                        .or_else(|| {
                            trace
                                .city_passes
                                .get(&city.0)
                                .and_then(|passes| passes.last().copied())
                        })
                        .unwrap_or("?");
                    let finish = c
                        .detail
                        .as_ref()
                        .and_then(|d| d.options.iter().find(|o| o.item == item).map(|o| o.turns));
                    let params = arm.1.clone().unwrap_or_else(AiParams::builtin);
                    let horizon = if params.decisions(view.faction_of(*id)).siege_survival_clock > 0
                    {
                        8
                    } else {
                        1
                    };
                    let (fall, deadline) = city_siege_times(&view, c, horizon);
                    buys.push(json!({
                        "turn": turn, "ai": id.0, "city": city.0, "kind": kind,
                        "unit": item_name(&item), "pass": pass, "walls": c.walls,
                        "hp": c.hp, "seen": seen, "own": own,
                        "finish_turns": if kind == "build" { finish } else { Some(0) },
                        "falls_in": fall, "deadline_in": deadline,
                    }));
                }
            }
            orders.insert(*id, os);
        }
        let (next, events) = resolve_turn(&state, &orders);
        for e in &events {
            match e {
                Event::ProductionCompleted {
                    city,
                    owner,
                    item: ProductionItem::Unit { unit },
                } if *owner != him && legatum_core::data::unit_stats(*unit).ranged > 0 => {
                    if let Some(c) = state.cities.get(city)
                        && legatum_core::data::has_building(
                            &c.buildings,
                            legatum_core::BuildingKind::Walls,
                        )
                    {
                        let distance = state
                            .units
                            .values()
                            .filter(|u| u.owner == him && is_military(u.kind))
                            .map(|u| u.pos.distance(c.pos))
                            .min();
                        ranged_finishes.push(json!({"turn": turn, "ai": owner.0,
                                "city": city.0, "unit": format!("{unit:?}"),
                                "nearest_attacker": distance}));
                    }
                }
                Event::UnitDestroyed {
                    owner, unit_kind, ..
                } if is_military(*unit_kind) && *unit_kind != legatum_core::UnitKind::Scout => {
                    if *owner == him {
                        lost.push((turn, format!("{unit_kind:?}")));
                    } else if !owner.is_barbarian() {
                        ai_lost.push((turn, owner.0, format!("{unit_kind:?}")));
                    }
                }
                Event::CityCaptured { city, from, to, .. } if *to == him => {
                    captures.push((turn, city.0, from.0));
                }
                Event::Battle {
                    attacker_owner,
                    defender,
                    damage_to_attacker,
                    damage_to_defender,
                    ..
                } => {
                    if *attacker_owner == him {
                        his_attacks += 1;
                        dealt_to_him += damage_to_attacker;
                        if let Defender::City { city, .. } = defender {
                            *city_attacks.entry(city.0).or_default() += 1;
                        }
                    } else if matches!(defender, Defender::Unit { owner, .. } if *owner == him) {
                        dealt_to_him += damage_to_defender;
                        // Shots by AI units (not city strikes): where the shooter stood.
                        if let Event::Battle {
                            attacker: legatum_core::events::Attacker::Unit(unit),
                            ranged: true,
                            attacker_at: Some(from),
                            ..
                        } = e
                        {
                            let in_city = state.cities.values().any(|c| c.pos == *from);
                            let kind = state
                                .units
                                .get(unit)
                                .map(|u| format!("{:?}", u.kind))
                                .unwrap_or_default();
                            shots.push((turn, kind, in_city, *damage_to_defender));
                        }
                    }
                }
                _ => {}
            }
        }
        for (city, owner, previous, walls) in threatened_builds {
            if events.iter().any(|e| {
                matches!(e,
                Event::ProductionCompleted { city: done, .. } if *done == city)
            }) {
                continue;
            }
            if let Some(after) = next.cities.get(&city)
                && after.owner == owner
                && let Some(item) = after.producing
                && item != previous
            {
                switches.push(json!({"turn": turn, "ai": owner.0, "city": city.0,
                    "from": item_name(&previous), "to": item_name(&item), "walls": walls,
                    "passes": switch_passes.get(&city.0)}));
            }
        }
        state = next;
    }
    json!({
        "branch_turn": start, "arm": arm.0, "salt": salt, "end_turn": state.turn,
        "lost": lost.len(), "lost_list": lost, "captures": captures,
        "ai_lost": ai_lost.len(), "ai_lost_list": ai_lost,
        "city_attacks": city_attacks, "his_attacks": his_attacks, "ai_shots": shots, "dealt_to_him": dealt_to_him, "buys": buys, "switches": switches, "ranged_finishes": ranged_finishes, "diags": diags,
        "him_cities": state.cities_of(him).count(),
        "forecasts": forecasts, "hard_turn_us": hard_turn_us,
    })
}

fn main() {
    let args = parse_args();
    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()
            .unwrap();
    }
    let tape =
        Tape::from_json(&std::fs::read_to_string(&args.tape).expect("read tape")).expect("tape");
    let tape_orders: BTreeMap<u32, Vec<Order>> = tape
        .turns
        .iter()
        .map(|t| (t.turn, t.orders.get(&args.him).cloned().unwrap_or_default()))
        .collect();
    let snaps = warm(&tape, &args);
    let mut jobs = Vec::new();
    for &t in &args.turns {
        for a in 0..args.arms.len() {
            for salt in 0..args.salts {
                jobs.push((t, a, salt));
            }
        }
    }
    let rows: Vec<Value> = jobs
        .par_iter()
        .map(|&(t, a, salt)| play(&snaps[&t], &args, &tape_orders, &args.arms[a], salt))
        .collect();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args.out).unwrap());
    for r in &rows {
        writeln!(out, "{r}").unwrap();
    }
    eprintln!("matchup_run: {} runs -> {}", rows.len(), args.out);
}

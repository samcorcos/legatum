//! Gate runner for the joint-assault fix (2026-11-03, the 8.9.25 Celts game). Diagnostic only.
//!
//! joint_assault_gate --config CONFIG.json --tape TAPE.json --games N [--salt-start S] --out FILE
//!
//! The tape is replayed with every seat's Hard brain shadowing it (seeds as the app derives them),
//! so each AI has its real opponent memory at the branch turn. Each branch then plays from that
//! turn to `cap_turn`: seat 0 is Hard (dice reseeded per salt) or replays the tape (AI seeds salted,
//! the Wild recalled whenever the goal lapses, as he did). Per-seat AiParams ("ai" for every AI).
//!
//! Counted for the watched seats (the attackers), each against seat 0's cities:
//! - `lone_exposures`: a military unit standing (at the start of a turn, when the ranged phase
//!   fires) within 2 of a walled seat-0 city with HP at war, with no other unit of its owner within
//!   2 of that city; `exposures` counts every such unit, alone or not.
//! - `strike_hits` / `lone_strike_hits`: city strikes on the seat's units (the lone ones as above).
//! - `deaths_near`: the seat's military units destroyed within 2 of a seat-0 city.
//! - `city_hits`: the seat's attacks on seat-0 cities; `captures`: seat-0 cities it took.
//! - `zero_hp`: a seat-0 city at 0 HP at the end of a turn with a unit of the seat within 2 (one
//!   per streak); `zero_hp_taken`: of those, taken by that seat within 2 turns.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::events::{Attacker, Defender};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, Order, PlayerId, player_view, resolve_turn, rules::is_military,
};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct Config {
    branches: Vec<Branch>,
}

#[derive(Deserialize)]
struct Branch {
    label: String,
    turn: u32,
    cap_turn: u32,
    /// "hard" or "tape".
    seat0: String,
    #[serde(default)]
    seat_params: BTreeMap<String, AiParams>,
    /// The attacking seats to count.
    watch: Vec<u8>,
}

fn salted(seed: u64, salt: u64) -> u64 {
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

struct Snapshot {
    state: GameState,
    memory: BTreeMap<u8, legatum_ai::OpponentMemory>,
}

fn warm(tape: &Tape, turns: &BTreeSet<u32>) -> (BTreeMap<u32, Snapshot>, u32) {
    let mut state = tape.initial_state();
    let mut brains: Vec<ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            ScriptedBrain::with_difficulty(
                p.faction,
                legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                p.id,
                AiDifficulty::Hard,
            )
        })
        .collect();
    let mut snaps = BTreeMap::new();
    let mut hash_mismatch = 0;
    let last = turns.iter().copied().max().unwrap_or(0);
    for t in &tape.turns {
        if t.turn > last {
            break;
        }
        if turns.contains(&t.turn) {
            let memory = brains
                .iter()
                .enumerate()
                .filter_map(|(i, b)| b.opponent_memory().map(|m| (i as u8, m)))
                .collect();
            snaps.insert(
                t.turn,
                Snapshot {
                    state: state.clone(),
                    memory,
                },
            );
        }
        for p in state.players.iter().filter(|p| p.alive) {
            let _ = brains[p.id.0 as usize].orders(&player_view(&state, p.id));
        }
        state = resolve_turn(&state, &t.orders).0;
        if state.hash().to_string() != t.hash {
            hash_mismatch += 1;
        }
    }
    (snaps, hash_mismatch)
}

#[derive(Default, Clone, serde::Serialize)]
struct Counts {
    exposures: u32,
    lone_exposures: u32,
    strike_hits: u32,
    lone_strike_hits: u32,
    deaths_near: u32,
    city_hits: u32,
    captures: u32,
    zero_hp: u32,
    zero_hp_taken: u32,
}

fn play(tape: &Tape, snap: &Snapshot, b: &Branch, salt: u64) -> Value {
    let mut state = snap.state.clone();
    let seat = PlayerId(0);
    let tape_seat = b.seat0 == "tape";
    if !tape_seat {
        if salt != 0 {
            state.setup.seed = salted(state.setup.seed, salt ^ 0xD1CE);
        }
        state.players[0].is_human = false;
        state.setup.players[0].is_human = false;
        state.setup.players[0].difficulty = AiDifficulty::Hard;
    }
    let builtin = AiParams::builtin();
    let params_for = |id: u8| -> &AiParams {
        b.seat_params
            .get(&id.to_string())
            .or_else(|| (id != 0).then(|| b.seat_params.get("ai")).flatten())
            .unwrap_or(&builtin)
    };
    let mut brains: Vec<Option<ScriptedBrain>> = state
        .players
        .iter()
        .map(|p| {
            if p.id == seat && tape_seat {
                return None;
            }
            let seed = salted(
                legatum_wasm::session::brain_seed(state.setup.seed, p.id),
                salt,
            );
            let mut brain = ScriptedBrain::with_params(
                p.faction,
                seed,
                p.id,
                AiDifficulty::Hard,
                params_for(p.id.0),
            );
            if let Some(m) = snap.memory.get(&p.id.0) {
                brain.restore_opponent_memory(m.clone());
            }
            Some(brain)
        })
        .collect();
    let mut counts: BTreeMap<u8, Counts> =
        b.watch.iter().map(|&w| (w, Counts::default())).collect();
    // (city, seat) -> turn the 0-HP streak began, still open.
    let mut zero_open: BTreeMap<(u32, u8), u32> = BTreeMap::new();
    let mut failed_orders = 0u32;
    let mut rings: Vec<Value> = Vec::new();
    let mut lone_detail: Vec<Value> = Vec::new();
    let mut plans: Vec<Value> = Vec::new();
    let mut last_passes: BTreeMap<u32, Vec<&'static str>> = BTreeMap::new();
    let lost_at_start = state.cities_of(seat).count();
    while !state.finished && state.turn <= b.cap_turn {
        let turn = state.turn;
        // Lone units within reach of a walled seat-0 city's strike (fired in this turn's ranged phase).
        let mut alone: BTreeSet<u32> = BTreeSet::new();
        if salt <= 1 {
            for c in state.cities_of(seat) {
                for &w in &b.watch {
                    let ring = |lo: i32, hi: i32| {
                        state
                            .units
                            .values()
                            .filter(|u| {
                                u.owner == PlayerId(w)
                                    && is_military(u.kind)
                                    && (lo..=hi).contains(&u.pos.distance(c.pos))
                            })
                            .count()
                    };
                    let (a, m, f) = (ring(0, 2), ring(3, 4), ring(5, 8));
                    if a + m + f > 0 && state.at_war(PlayerId(w), seat) {
                        rings.push(json!([turn, w, c.name, c.hp, a, m, f]));
                    }
                }
            }
        }
        for c in state.cities_of(seat) {
            if c.hp <= 0 || legatum_core::data::wall_tier(&c.buildings) == 0 {
                continue;
            }
            for &w in &b.watch {
                let me = PlayerId(w);
                if !state.at_war(me, seat) {
                    continue;
                }
                let near: Vec<_> = state
                    .units
                    .values()
                    .filter(|u| u.owner == me && is_military(u.kind) && u.pos.distance(c.pos) <= 2)
                    .collect();
                let e = counts.get_mut(&w).unwrap();
                e.exposures += near.len() as u32;
                if near.len() == 1 {
                    e.lone_exposures += 1;
                    alone.insert(near[0].id.0);
                    if salt <= 1 {
                        lone_detail.push(json!([
                            turn,
                            w,
                            c.name,
                            format!("{:?}", near[0].kind),
                            near[0].id.0,
                            last_passes.get(&near[0].id.0)
                        ]));
                    }
                }
            }
        }
        let mut orders: BTreeMap<PlayerId, Vec<Order>> = BTreeMap::new();
        for p in state
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
        {
            let mut o = match brains[p.id.0 as usize].as_mut() {
                Some(brain) if salt <= 1 && b.watch.contains(&p.id.0) => {
                    let (o, trace) = brain.order_diagnosis(&player_view(&state, p.id));
                    last_passes.extend(trace.passes);
                    o
                }
                Some(brain) => brain.orders(&player_view(&state, p.id)),
                None => tape
                    .turns
                    .iter()
                    .find(|t| t.turn == turn)
                    .and_then(|t| t.orders.get(&p.id).cloned())
                    .unwrap_or_default(),
            };
            if p.id == seat && tape_seat {
                if !legatum_core::goals::reached(&state, seat)
                    && player_view(&state, seat)
                        .actions
                        .wild
                        .as_ref()
                        .is_some_and(|w| w.blocker.is_none())
                    && !o.iter().any(|x| matches!(x, Order::Action { .. }))
                {
                    o.push(Order::Action {
                        action: legatum_core::orders::Action::CallTheWild,
                    });
                }
                failed_orders += legatum_core::validate_orders(&state, seat, &o).len() as u32;
            }
            orders.insert(p.id, o);
        }
        if salt <= 1 {
            for &w in &b.watch {
                if let Some(m) = brains[w as usize]
                    .as_ref()
                    .and_then(|x| x.opponent_memory())
                {
                    let plan = m
                        .campaigns
                        .plan
                        .active
                        .as_ref()
                        .map(|p| format!("{:?}", p.kind));
                    let target = m.campaigns.active.as_ref().map(|c| c.target.0);
                    plans.push(json!([turn, w, plan, target]));
                }
            }
        }
        let before = state.clone();
        let (next, events) = resolve_turn(&state, &orders);
        state = next;
        for e in &events {
            match e {
                Event::Battle {
                    attacker: Attacker::City { .. },
                    attacker_owner,
                    defender: Defender::Unit { unit, owner, .. },
                    ..
                } if *attacker_owner == seat => {
                    if let Some(c) = counts.get_mut(&owner.0) {
                        c.strike_hits += 1;
                        if alone.contains(&unit.0) {
                            c.lone_strike_hits += 1;
                        }
                    }
                }
                Event::Battle {
                    attacker_owner,
                    defender: Defender::City { owner, .. },
                    ..
                } if *owner == seat => {
                    if let Some(c) = counts.get_mut(&attacker_owner.0) {
                        c.city_hits += 1;
                    }
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind,
                    at,
                    ..
                } if is_military(*unit_kind) => {
                    if let Some(c) = counts.get_mut(&owner.0)
                        && before
                            .cities_of(seat)
                            .any(|city| city.pos.distance(*at) <= 2)
                    {
                        c.deaths_near += 1;
                    }
                }
                Event::CityCaptured { city, from, to, .. } if *from == seat => {
                    if let Some(c) = counts.get_mut(&to.0) {
                        c.captures += 1;
                    }
                    if let Some(start) = zero_open.remove(&(city.0, to.0))
                        && turn <= start + 2
                    {
                        counts.get_mut(&to.0).unwrap().zero_hp_taken += 1;
                    }
                }
                _ => {}
            }
        }
        // 0-HP streaks at the end of the turn.
        for c in state.cities_of(seat) {
            for &w in &b.watch {
                let key = (c.id.0, w);
                let near = state.at_war(PlayerId(w), seat)
                    && state.units.values().any(|u| {
                        u.owner == PlayerId(w) && is_military(u.kind) && u.pos.distance(c.pos) <= 2
                    });
                if c.hp <= 0 && near {
                    if let std::collections::btree_map::Entry::Vacant(e) = zero_open.entry(key) {
                        e.insert(turn);
                        counts.get_mut(&w).unwrap().zero_hp += 1;
                    }
                } else if c.hp > 0 {
                    zero_open.remove(&key);
                }
            }
        }
        zero_open.retain(|(city, _), _| {
            state
                .cities
                .get(&legatum_core::CityId(*city))
                .is_some_and(|c| c.owner == seat)
        });
    }
    json!({
        "branch": b.label, "salt": salt, "end_turn": state.turn,
        "seat0_cities_lost": lost_at_start as i64 - state.cities_of(seat).count() as i64,
        "seat0_win": state.winner == Some(state.team_of(seat)),
        "winner": state.winner.map(|t| t.0),
        "failed_orders": failed_orders,
        "counts": counts, "rings": rings, "lone_detail": lone_detail, "plans": plans,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |k: &str| {
        args.iter()
            .position(|a| a == k)
            .map(|i| args[i + 1].clone())
    };
    let config: Config =
        serde_json::from_str(&std::fs::read_to_string(arg("--config").expect("--config")).unwrap())
            .expect("config");
    let games: u64 = arg("--games").map_or(32, |s| s.parse().unwrap());
    let start: u64 = arg("--salt-start").map_or(1, |s| s.parse().unwrap());
    let turns: BTreeSet<u32> = config.branches.iter().map(|b| b.turn).collect();
    // `--scenario FILE`: branch from a Scenario Lab start (every seat Hard, fresh AI memory).
    let (tape, snaps, hash_mismatch) = if let Some(path) = arg("--scenario") {
        let file =
            legatum_wasm::scenario::ScenarioFile::parse(&std::fs::read_to_string(path).unwrap())
                .expect("scenario");
        let state = file.prepare(false).expect("scenario state");
        let tape = Tape {
            version: 0,
            setup: state.setup.clone(),
            initial_hash: 0,
            turns: Vec::new(),
        };
        let snaps = BTreeMap::from([(
            state.turn,
            Snapshot {
                state,
                memory: BTreeMap::new(),
            },
        )]);
        (tape, snaps, 0)
    } else {
        let tape =
            Tape::from_json(&std::fs::read_to_string(arg("--tape").expect("--tape")).unwrap())
                .expect("tape");
        let (snaps, hash_mismatch) = warm(&tape, &turns);
        (tape, snaps, hash_mismatch)
    };
    eprintln!("warm-up hash mismatches: {hash_mismatch}");
    let jobs: Vec<(usize, u64)> = (0..config.branches.len())
        .flat_map(|b| (start..start + games).map(move |s| (b, s)))
        .collect();
    let results: Vec<(usize, Value)> = jobs
        .into_par_iter()
        .map(|(b, salt)| {
            let branch = &config.branches[b];
            (b, play(&tape, &snaps[&branch.turn], branch, salt))
        })
        .collect();
    let mut by: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for (b, r) in results {
        by.entry(config.branches[b].label.clone())
            .or_default()
            .push(r);
    }
    let out = json!({"hash_mismatch": hash_mismatch, "branches": by});
    std::fs::write(
        arg("--out").expect("--out"),
        serde_json::to_string(&out).unwrap(),
    )
    .unwrap();
}

//! Rush ladder: does a Hard AI change help one empire survive an early conqueror?
//!
//! rush_ladder --arm LABEL[:gene=v,gene=v...] [--arm ...] [--games N] [--seed S] [--mode duel|rush]
//!             [--players 6] [--turns T] [--war-turn W] [--storm K] [--out FILE]
//!
//! Every arm plays the same seeds. Game `i` uses seed `S + i`: factions are a seeded draw from every
//! faction, the observed seat is `seed % players`, and only that seat plays the arm's genes (every
//! other seat plays the built-in Hard AI), so arms are paired game for game ("one new AI against
//! five baseline AIs, rotated seats").
//!
//! - `duel`: an ordinary Hard game; the seat's result (win, VP share, survival, cities) is the score.
//! - `rush`: the rival whose capital starts nearest the seat also plays the scripted rusher
//!   (`legatum_wasm::rusher`): it declares war on the seat from turn W and storms its nearest city
//!   once K units stand within two tiles. The score is how often the seat keeps its capital and
//!   its cities, and whether other empires join the war on the rusher.
//!
//! Prints one JSON line per game and arm (to `--out` when given) and a summary per arm on stderr.

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::rules::is_military;
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, PlayerId, UnitKind, scoring};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;

const ARMY_AT: [u32; 7] = [10, 20, 30, 40, 50, 60, 75];

struct Args {
    arms: Vec<(String, AiParams)>,
    /// Per arm: rule experiments (`x.name=value` in `--arm`), applied from the handover turn
    /// (or from turn 1 without one). Rules, not genes: every seat plays under them.
    exps: Vec<serde_json::Map<String, Value>>,
    games: u64,
    seed: u64,
    players: usize,
    turns: u32,
    rush: bool,
    war_turn: u32,
    storm: u32,
    out: Option<String>,
    /// Every seat plays the arm's genes (for genes that change how neighbours react).
    all: bool,
    /// Play on a recorded game's map and lobby: the human seat becomes Hard and is observed;
    /// `--games` salts the AI brain seeds.
    tape: Option<legatum_core::GameSetup>,
    /// With `--tape`: replay the recorded orders (the human's and the AIs') for every turn
    /// before this one, then let the brains play on. Measures how the AI handles a real
    /// runaway position. The brains still see every replayed turn, so their memory of it holds.
    handover: u32,
    recorded: Vec<legatum_wasm::tape::TapeTurn>,
}

fn arm(spec: &str) -> (String, AiParams, serde_json::Map<String, Value>) {
    let (label, genes) = spec.split_once(':').unwrap_or((spec, ""));
    let mut value = serde_json::to_value(AiParams::builtin()).expect("params json");
    let mut exps = serde_json::Map::new();
    for kv in genes.split(',').filter(|s| !s.is_empty()) {
        let (k, v) = kv
            .split_once('=')
            .unwrap_or_else(|| panic!("gene {kv}: want name=value"));
        if let Some(name) = k.strip_prefix("x.") {
            let parsed: Value = serde_json::from_str(v).unwrap_or(json!(v));
            exps.insert(
                name.into(),
                if v == "1" && name_is_flag(name) {
                    json!(true)
                } else {
                    parsed
                },
            );
            continue;
        }
        let decisions = value["decisions"]
            .as_object_mut()
            .expect("decisions object");
        assert!(decisions.contains_key(k), "unknown gene {k}");
        decisions.insert(
            k.into(),
            json!(v.parse::<i64>().expect("integer gene value")),
        );
    }
    let mut p: AiParams = serde_json::from_value(value).expect("params");
    p.name = label.into();
    p.validate().unwrap_or_else(|e| panic!("{label}: {e}"));
    experiments(&exps);
    (label.into(), p, exps)
}

/// A boolean experiment (as opposed to a number): `x.city_research=1` means true.
/// True when the experiment field takes a boolean: `x.flag=1` then means `true`, while numeric
/// fields such as `city_upkeep_free` keep the number (defaults skip every field when serialized,
/// so probe by deserializing a `true`).
fn name_is_flag(name: &str) -> bool {
    serde_json::from_value::<legatum_core::experiments::RuleExperiments>(json!({ name: true }))
        .is_ok()
}

fn experiments(
    exps: &serde_json::Map<String, Value>,
) -> legatum_core::experiments::RuleExperiments {
    serde_json::from_value(Value::Object(exps.clone()))
        .unwrap_or_else(|e| panic!("rule experiments {exps:?}: {e}"))
}

fn parse_args() -> Args {
    let mut it = std::env::args().skip(1);
    let mut a = Args {
        arms: Vec::new(),
        exps: Vec::new(),
        games: 64,
        seed: 1,
        players: 6,
        turns: 0,
        rush: false,
        war_turn: 12,
        storm: 3,
        out: None,
        all: false,
        tape: None,
        handover: 0,
        recorded: Vec::new(),
    };
    while let Some(flag) = it.next() {
        let mut v = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--arm" => {
                let (label, params, exps) = arm(&v());
                a.arms.push((label, params));
                a.exps.push(exps);
            }
            "--games" => a.games = v().parse().expect("--games"),
            "--seed" => a.seed = v().parse().expect("--seed"),
            "--players" => a.players = v().parse().expect("--players"),
            "--turns" => a.turns = v().parse().expect("--turns"),
            "--mode" => a.rush = v() == "rush",
            "--war-turn" => a.war_turn = v().parse().expect("--war-turn"),
            "--storm" => a.storm = v().parse().expect("--storm"),
            "--out" => a.out = Some(v()),
            "--all" => a.all = v() == "1",
            "--handover" => a.handover = v().parse().expect("--handover"),
            "--tape" => {
                let text = std::fs::read_to_string(v()).expect("read --tape");
                let tape = legatum_wasm::tape::Tape::from_json(&text).expect("tape");
                a.tape = Some(tape.setup);
                a.recorded = tape.turns;
            }
            other => panic!("unknown argument {other}"),
        }
    }
    if a.arms.is_empty() {
        let (label, params, exps) = arm("base");
        a.arms.push((label, params));
        a.exps.push(exps);
    }
    assert!((3..=8).contains(&a.players));
    a
}

fn setup(args: &Args, game_seed: u64) -> (GameSetup, usize) {
    if let Some(recorded) = &args.tape {
        let mut setup = recorded.clone();
        let seat = setup.players.iter().position(|p| p.is_human).unwrap_or(0);
        for p in setup.players.iter_mut() {
            p.difficulty = AiDifficulty::Hard;
            p.is_human = false;
            p.search = false;
        }
        return (setup, seat);
    }
    let mut setup = GameSetup::all_ai(game_seed);
    setup.players.truncate(args.players);
    let mut pool: Vec<Faction> = Faction::ALL.into_iter().collect();
    let mut rng = Rng::derive(game_seed, &[purpose::SETUP, 0x5A5]);
    for i in (1..pool.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        pool.swap(i, j);
    }
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = pool[i];
        p.difficulty = AiDifficulty::Hard;
        p.is_human = false;
        p.search = false;
    }
    (setup, (game_seed % args.players as u64) as usize)
}

/// The arm's experiments over the game's own: fields the arm set win.
fn merge(
    game: legatum_core::experiments::RuleExperiments,
    arm: &legatum_core::experiments::RuleExperiments,
) -> legatum_core::experiments::RuleExperiments {
    let mut g = serde_json::to_value(game).expect("experiments json");
    let a = serde_json::to_value(*arm).expect("experiments json");
    let d =
        serde_json::to_value(legatum_core::experiments::RuleExperiments::default()).expect("json");
    if let (Some(g), Some(a)) = (g.as_object_mut(), a.as_object()) {
        for (k, v) in a {
            if d.get(k) != Some(v) {
                g.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::from_value(g).expect("experiments")
}

fn army(state: &legatum_core::GameState, p: PlayerId) -> usize {
    state
        .units_of(p)
        .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
        .count()
}

fn play(
    args: &Args,
    params: &AiParams,
    exps: &legatum_core::experiments::RuleExperiments,
    index: u64,
) -> Value {
    let game_seed = args.seed + index;
    let (setup, seat_index) = setup(args, game_seed);
    let seat = PlayerId(seat_index as u8);
    let mut state = legatum_core::new_game(&setup);
    let reseed = args.tape.is_some() && index > 0;
    if reseed && args.handover == 0 {
        // Same map and lobby, fresh dice per salt (as `hill_suite` does).
        state.dice_seed = Some(setup.seed ^ index.wrapping_mul(0xD1CE_0000_9E37_79B9));
    }
    let mut mismatches = 0u32;
    if args.handover == 0 {
        state.setup.rule_experiments = merge(state.setup.rule_experiments, exps);
    }
    let master = AiParams::builtin();
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let id = PlayerId(i as u8);
            let seed = if args.tape.is_some() {
                legatum_wasm::session::brain_seed(setup.seed, id)
                    ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            } else {
                legatum_wasm::session::brain_seed(game_seed, id)
            };
            let brain: Box<dyn Brain> = Box::new(ScriptedBrain::with_params(
                p.faction,
                seed,
                id,
                AiDifficulty::Hard,
                if id == seat || args.all {
                    params
                } else {
                    &master
                },
            ));
            brain
        })
        .collect();
    // The rusher: the rival whose capital starts nearest the seat.
    let home = state
        .capital_of(seat)
        .map(|c| c.pos)
        .or_else(|| state.units_of(seat).next().map(|u| u.pos));
    let rusher = if args.rush {
        let near = (0..args.players)
            .filter(|&i| i != seat_index)
            .min_by_key(|&i| {
                let p = PlayerId(i as u8);
                let pos = state
                    .capital_of(p)
                    .map(|c| c.pos)
                    .or_else(|| state.units_of(p).next().map(|u| u.pos));
                match (home, pos) {
                    (Some(a), Some(b)) => a.distance(b),
                    _ => i32::MAX,
                }
            })
            .expect("a rival");
        Some(legatum_wasm::rusher::Rusher {
            player: PlayerId(near as u8),
            wars: vec![legatum_wasm::rusher::RusherWar {
                against: seat,
                turn: args.war_turn,
            }],
            storm: args.storm,
            target: Some(seat),
        })
    } else {
        None
    };
    let my_team = state.player(seat).team;
    let mut army_at = BTreeMap::new();
    let mut cities_at = BTreeMap::new();
    let (mut lost, mut captured) = (0u32, 0u32);
    let (mut capital_lost, mut first_lost) = (None::<u32>, None::<u32>);
    let mut first_war_on_me = None::<u32>;
    // Wars declared on the rusher by empires other than the seat, and when.
    let mut joined: BTreeMap<u8, u32> = BTreeMap::new();
    let mut rusher_captures = 0u32;
    // Every capture of an original capital from its holder: (captor seat, turn).
    let mut capital_takes: Vec<(u8, u32)> = Vec::new();
    // After a handover: how the rivals fight the seat (attacks each way, city hits, losses,
    // rival units pressing the seat's cities, army sizes every 10 turns).
    let mut front: BTreeMap<&str, i64> = BTreeMap::new();
    let mut armies: Vec<(u32, usize, usize)> = Vec::new();
    let limit = if args.turns > 0 { args.turns } else { 700 };
    let mut my_capital = None;
    let trace = std::env::var("RL_TRACE").is_ok();
    let mut builds: Vec<String> = Vec::new();
    let mut last_item: BTreeMap<legatum_core::CityId, String> = BTreeMap::new();
    while !state.finished && state.turn <= limit {
        if my_capital.is_none() {
            my_capital = state.capital_of(seat).map(|c| c.id);
        }
        if let Some(id) = my_capital
            && capital_lost.is_none()
            && state.cities.get(&id).is_none_or(|c| c.owner != seat)
        {
            capital_lost = Some(state.turn);
        }
        if ARMY_AT.contains(&state.turn) {
            army_at.insert(state.turn, army(&state, seat));
            cities_at.insert(state.turn, state.cities_of(seat).count());
        }
        if state.turn == 100 {
            cities_at.insert(100, state.cities_of(seat).count());
        }
        if trace && state.turn <= 60 {
            for c in state.cities_of(seat) {
                let item = format!("{:?}", c.producing);
                if last_item.get(&c.id) != Some(&item) {
                    builds.push(format!(
                        "T{} {}: {}",
                        state.turn,
                        c.name,
                        item.replace("Some(", "").replace(")", "")
                    ));
                    last_item.insert(c.id, item);
                }
            }
        }
        let mut orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let view = legatum_core::player_view(&state, p.id);
                (p.id, brains[p.id.0 as usize].orders(&view))
            })
            .collect();
        if let Some(r) = &rusher {
            legatum_wasm::rusher::apply(&mut orders, &state, r);
        }
        let replaying = state.turn < args.handover;
        let recorded = if replaying {
            args.recorded.iter().find(|t| t.turn == state.turn)
        } else {
            None
        };
        if let Some(t) = recorded {
            orders = t.orders.clone();
        }
        let turn = state.turn;
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        let before = state;
        state = next;
        if args.handover > 0 && turn >= args.handover {
            let seat_cities: Vec<_> = before.cities_of(seat).map(|c| c.pos).collect();
            let pressing = before
                .units
                .values()
                .filter(|u| u.owner != seat && !u.owner.is_barbarian() && is_military(u.kind))
                .filter(|u| before.teams_at_war(before.team_of(u.owner), my_team))
                .filter(|u| seat_cities.iter().any(|c| c.distance(u.pos) <= 3))
                .count();
            *front.entry("pressing_unit_turns").or_default() += pressing as i64;
            if (turn - args.handover).is_multiple_of(10) {
                let rivals: usize = before
                    .players
                    .iter()
                    .filter(|p| p.alive && p.id != seat && !p.id.is_barbarian())
                    .filter(|p| before.teams_at_war(p.team, my_team))
                    .map(|p| army(&before, p.id))
                    .sum();
                armies.push((turn, army(&before, seat), rivals));
            }
            for event in &events {
                match event {
                    Event::Battle {
                        attacker_owner, at, ..
                    } => {
                        let defender_seat = before
                            .cities
                            .values()
                            .any(|c| c.pos == *at && c.owner == seat)
                            || before
                                .units
                                .values()
                                .any(|u| u.pos == *at && u.owner == seat);
                        let city = before.cities.values().any(|c| c.pos == *at);
                        if *attacker_owner == seat {
                            *front.entry("seat_attacks").or_default() += 1;
                        } else if defender_seat && !attacker_owner.is_barbarian() {
                            *front.entry("rival_attacks").or_default() += 1;
                            if city {
                                *front.entry("rival_city_hits").or_default() += 1;
                            }
                        }
                    }
                    Event::UnitDestroyed { owner, .. } => {
                        if *owner == seat {
                            *front.entry("seat_units_lost").or_default() += 1;
                        } else if !owner.is_barbarian() {
                            *front.entry("rival_units_lost").or_default() += 1;
                        }
                    }
                    Event::PeaceEstablished { .. } => {
                        *front.entry("peaces").or_default() += 1;
                    }
                    _ => {}
                }
            }
        }
        if let Some(t) = recorded
            && state.hash().to_string() != t.hash
        {
            mismatches += 1;
        }
        if args.handover > 0 && state.turn == args.handover {
            state.setup.rule_experiments = merge(state.setup.rule_experiments, exps);
        }
        if reseed && args.handover > 0 && state.turn == args.handover {
            state.dice_seed = Some(setup.seed ^ index.wrapping_mul(0xD1CE_0000_9E37_79B9));
        }
        for event in &events {
            match event {
                Event::WarDeclared { relation } => {
                    if (relation.a == my_team || relation.b == my_team)
                        && !relation.declarers.contains(&my_team)
                    {
                        first_war_on_me.get_or_insert(turn);
                    }
                    if let Some(r) = &rusher {
                        let rt = before.team_of(r.player);
                        if relation.a == rt || relation.b == rt {
                            for d in &relation.declarers {
                                if *d != rt && *d != my_team {
                                    joined.entry(d.0).or_insert(turn);
                                }
                            }
                        }
                    }
                }
                Event::CityRazed { from, .. } if *from == seat => {
                    lost += 1;
                    first_lost.get_or_insert(turn);
                }
                Event::CityCaptured { from, to, city, .. } => {
                    if before
                        .cities
                        .get(city)
                        .is_some_and(|c| c.original_capital && c.owner == *from)
                        && before.team_of(*from) != before.team_of(*to)
                    {
                        capital_takes.push((to.0, turn));
                    }
                    if *from == seat {
                        lost += 1;
                        first_lost.get_or_insert(turn);
                    }
                    if *to == seat {
                        captured += 1;
                    }
                    if rusher.as_ref().is_some_and(|r| *to == r.player) {
                        rusher_captures += 1;
                    }
                }
                _ => {}
            }
        }
    }
    let total_vp: i64 = state
        .players
        .iter()
        .filter(|p| !p.id.is_barbarian())
        .map(|p| scoring::vp_total(&state, p.id) as i64)
        .sum();
    let my_vp = scoring::vp_total(&state, seat) as i64;
    let won = state.finished && state.winner == Some(my_team);
    json!({
        "seed": game_seed,
        "seat": seat_index,
        "faction": format!("{:?}", setup.players[seat_index].faction),
        "rusher": rusher.as_ref().map(|r| format!("{:?}", setup.players[r.player.0 as usize].faction)),
        "won": won,
        "finished": state.finished,
        "end_turn": state.turn,
        "alive": state.player(seat).alive,
        "vp": my_vp,
        "vp_share": if total_vp > 0 { my_vp as f64 / total_vp as f64 } else { 0.0 },
        "cities_end": state.cities_of(seat).count(),
        "cities_at": cities_at,
        "army_at": army_at,
        "lost": lost,
        "captured": captured,
        "first_lost": first_lost,
        "capital_lost": capital_lost,
        "first_war_on_me": first_war_on_me,
        "joined": joined.len(),
        "first_join": joined.values().min(),
        "rusher_captures": rusher_captures,
        "builds": builds,
        "mismatches": mismatches,
        "front": front,
        "armies": armies,
        "capital_takes": capital_takes,
        "winners": state.players.iter().filter(|p| state.finished && Some(p.team) == state.winner).map(|p| p.id.0).collect::<Vec<_>>(),
    })
}

fn main() {
    let args = parse_args();
    let jobs: Vec<(usize, u64)> = (0..args.arms.len())
        .flat_map(|a| (0..args.games).map(move |i| (a, i)))
        .collect();
    // Each game's line is written as soon as it finishes, so an interrupted run keeps its games;
    // with `--out`, games already in the file are skipped (resume).
    let done: std::collections::BTreeSet<(String, u64)> = args
        .out
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|text| {
            text.lines()
                .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                .filter_map(|g| Some((g["arm"].as_str()?.to_string(), g["seed"].as_u64()?)))
                .collect()
        })
        .unwrap_or_default();
    let out: std::sync::Mutex<Box<dyn Write + Send>> = std::sync::Mutex::new(match &args.out {
        Some(path) => Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("open --out"),
        ),
        None => Box::new(std::io::stdout()),
    });
    let fresh: Vec<(usize, Value)> = jobs
        .par_iter()
        .filter(|&&(a, i)| !done.contains(&(args.arms[a].0.clone(), args.seed + i)))
        .map(|&(a, i)| {
            let mut row = play(&args, &args.arms[a].1, &experiments(&args.exps[a]), i);
            row["arm"] = json!(args.arms[a].0);
            let mut out = out.lock().expect("output lock");
            writeln!(out, "{row}").expect("write");
            out.flush().expect("flush");
            (a, row)
        })
        .collect();
    let mut rows = fresh;
    if let Some(path) = &args.out {
        rows = std::fs::read_to_string(path)
            .expect("read --out")
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter_map(|g| {
                let a = args
                    .arms
                    .iter()
                    .position(|(label, _)| g["arm"] == json!(label))?;
                Some((a, g))
            })
            .collect();
    }
    for (a, (label, _)) in args.arms.iter().enumerate() {
        let games: Vec<&Value> = rows.iter().filter(|r| r.0 == a).map(|r| &r.1).collect();
        let n = games.len().max(1) as f64;
        let count = |f: &dyn Fn(&Value) -> bool| games.iter().filter(|g| f(g)).count() as f64 / n;
        let mean = |f: &dyn Fn(&Value) -> f64| games.iter().map(|g| f(g)).sum::<f64>() / n;
        eprintln!(
            "{label}: n={} win {:.1}% alive {:.1}% vp_share {:.3} capital_lost {:.1}% lost {:.2} army T10/T20/T40 {:.2}/{:.2}/{:.2} joined {:.2}",
            games.len(),
            100.0 * count(&|g| g["won"] == json!(true)),
            100.0 * count(&|g| g["alive"] == json!(true)),
            mean(&|g| g["vp_share"].as_f64().unwrap_or(0.0)),
            100.0 * count(&|g| !g["capital_lost"].is_null()),
            mean(&|g| g["lost"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["army_at"]["10"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["army_at"]["20"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["army_at"]["40"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["joined"].as_f64().unwrap_or(0.0)),
        );
    }
}

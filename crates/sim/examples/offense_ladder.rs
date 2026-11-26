//! Offense ladder: how hard does one Hard empire rush and conquer its neighbours?
//!
//! offense_ladder --arm LABEL[:gene=v,...] [--arm ...] [--games N] [--seed S] [--players 6]
//!                [--turns T] [--faction Sparta] [--tape TAPE.json] [--out FILE]
//!
//! Game `i` uses seed `S + i`; factions are a seeded draw, the observed seat is `seed % players`
//! and, with `--faction`, that seat is swapped to the named faction. Only the seat plays the
//! arm's genes; every other seat plays the built-in Hard AI, so arms are paired game for game.
//! With `--tape` the recorded map and lobby are used (the human seat is observed, every seat
//! Hard) and `--games` salts the dice and brain seeds.
//!
//! The score is the seat's offence: its first declaration, captures and capitals taken by
//! turns 30/50/75, rivals it eliminates, its army and cities, and (for full games) win and
//! VP share.

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::rules::is_military;
use legatum_core::{AiDifficulty, Event, Faction, GameSetup, PlayerId, UnitKind, scoring};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;

const AT: [u32; 8] = [20, 30, 40, 50, 75, 100, 125, 150];

struct Args {
    arms: Vec<(String, AiParams)>,
    games: u64,
    seed: u64,
    players: usize,
    turns: u32,
    faction: Option<Faction>,
    out: Option<String>,
    tape: Option<GameSetup>,
}

fn arm(spec: &str) -> (String, AiParams) {
    let (label, genes) = spec.split_once(':').unwrap_or((spec, ""));
    let mut value = serde_json::to_value(AiParams::builtin()).expect("params json");
    for kv in genes.split(',').filter(|s| !s.is_empty()) {
        let (k, v) = kv
            .split_once('=')
            .unwrap_or_else(|| panic!("gene {kv}: want name=value"));
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
    (label.into(), p)
}

fn parse_args() -> Args {
    let mut it = std::env::args().skip(1);
    let mut a = Args {
        arms: Vec::new(),
        games: 64,
        seed: 1,
        players: 6,
        turns: 0,
        faction: None,
        out: None,
        tape: None,
    };
    while let Some(flag) = it.next() {
        let mut v = || it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--arm" => a.arms.push(arm(&v())),
            "--games" => a.games = v().parse().expect("--games"),
            "--seed" => a.seed = v().parse().expect("--seed"),
            "--players" => a.players = v().parse().expect("--players"),
            "--turns" => a.turns = v().parse().expect("--turns"),
            "--faction" => {
                let name = v();
                a.faction = Some(
                    Faction::ALL
                        .into_iter()
                        .find(|f| format!("{f:?}").eq_ignore_ascii_case(&name))
                        .unwrap_or_else(|| panic!("unknown faction {name}")),
                );
            }
            "--out" => a.out = Some(v()),
            "--tape" => {
                let text = std::fs::read_to_string(v()).expect("read --tape");
                let tape = legatum_wasm::tape::Tape::from_json(&text).expect("tape");
                a.tape = Some(tape.setup);
            }
            other => panic!("unknown argument {other}"),
        }
    }
    if a.arms.is_empty() {
        a.arms.push(arm("base"));
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
    let seat = (game_seed % args.players as u64) as usize;
    if let Some(f) = args.faction
        && let Some(k) = pool.iter().position(|x| *x == f)
    {
        pool.swap(k, seat);
    }
    for (i, p) in setup.players.iter_mut().enumerate() {
        p.faction = pool[i];
        p.difficulty = AiDifficulty::Hard;
        p.is_human = false;
        p.search = false;
    }
    (setup, seat)
}

fn army(state: &legatum_core::GameState, p: PlayerId) -> usize {
    state
        .units_of(p)
        .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
        .count()
}

fn play(args: &Args, params: &AiParams, index: u64) -> Value {
    let game_seed = args.seed + index;
    let (setup, seat_index) = setup(args, game_seed);
    let seat = PlayerId(seat_index as u8);
    let mut state = legatum_core::new_game(&setup);
    if args.tape.is_some() && index > 0 {
        state.dice_seed = Some(setup.seed ^ index.wrapping_mul(0xD1CE_0000_9E37_79B9));
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
                if id == seat { params } else { &master },
            ));
            brain
        })
        .collect();
    let my_team = state.player(seat).team;
    let home = state
        .capital_of(seat)
        .map(|c| c.pos)
        .or_else(|| state.units_of(seat).next().map(|u| u.pos));
    let nearest = state
        .players
        .iter()
        .filter(|p| p.id != seat && !p.id.is_barbarian())
        .filter_map(|p| {
            let pos = state
                .capital_of(p.id)
                .map(|c| c.pos)
                .or_else(|| state.units_of(p.id).next().map(|u| u.pos))?;
            Some(home?.distance(pos))
        })
        .min();
    let mut army_at = BTreeMap::new();
    let mut cities_at = BTreeMap::new();
    let mut pop_at = BTreeMap::new();
    let mut gold_at = BTreeMap::new();
    let mut techs_at = BTreeMap::new();
    let mut first_declare = None::<u32>;
    let mut declares: Vec<u32> = Vec::new();
    let mut first_war = None::<u32>;
    let mut captures: Vec<u32> = Vec::new();
    let mut capitals: Vec<u32> = Vec::new();
    let mut eliminated: Vec<u32> = Vec::new();
    let mut lost = 0u32;
    let mut founded = 0u32;
    let mut attacks = 0u32;
    let limit = if args.turns > 0 { args.turns } else { 700 };
    while !state.finished && state.turn <= limit {
        if AT.contains(&state.turn) {
            army_at.insert(state.turn, army(&state, seat));
            cities_at.insert(state.turn, state.cities_of(seat).count());
            pop_at.insert(
                state.turn,
                state.cities_of(seat).map(|c| c.population).sum::<i32>(),
            );
            gold_at.insert(state.turn, state.player(seat).gold);
            techs_at.insert(state.turn, state.player(seat).techs.len());
        }
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let view = legatum_core::player_view(&state, p.id);
                (p.id, brains[p.id.0 as usize].orders(&view))
            })
            .collect();
        let turn = state.turn;
        if std::env::var("OL_TRACE").is_ok()
            && index == 0
            && turn
                <= std::env::var("OL_TRACE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(40)
        {
            let melee: Vec<String> = state
                .units_of(seat)
                .filter(|u| is_military(u.kind) && u.kind != UnitKind::Scout)
                .map(|u| {
                    let d = state
                        .cities
                        .values()
                        .filter(|c| c.owner != seat)
                        .map(|c| c.pos.distance(u.pos))
                        .min()
                        .unwrap_or(99);
                    format!("{:?}@{}hp{}", u.kind, d, u.hp)
                })
                .collect();
            let prod: Vec<String> = state
                .cities_of(seat)
                .map(|c| format!("{:?}", c.producing))
                .collect();
            let declares: Vec<String> = orders
                .get(&seat)
                .into_iter()
                .flatten()
                .filter(|o| matches!(o, legatum_core::Order::DeclareWar { .. }))
                .map(|o| format!("{o:?}"))
                .collect();
            let view = legatum_core::player_view(&state, seat);
            let known: Vec<String> = view
                .cities
                .iter()
                .filter(|c| c.owner != seat)
                .map(|c| {
                    format!(
                        "p{}d{}w{}",
                        c.owner.0,
                        home.map_or(0, |h| h.distance(c.pos)),
                        c.walls as u8
                    )
                })
                .collect();
            let scouts: Vec<String> = state
                .units_of(seat)
                .filter(|u| u.kind == UnitKind::Scout)
                .map(|u| {
                    format!(
                        "({},{})d{}hp{}",
                        u.pos.q,
                        u.pos.r,
                        home.map_or(0, |h| h.distance(u.pos)),
                        u.hp
                    )
                })
                .collect();
            eprintln!(
                "T{turn} gold {} {:?} {:?} {:?} known {:?} scouts {:?}",
                state.player(seat).gold,
                melee,
                prod,
                declares,
                known,
                scouts
            );
        }
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        let before = state;
        state = next;
        let mut took_from: Vec<PlayerId> = Vec::new();
        for event in &events {
            match event {
                Event::WarDeclared { relation } => {
                    if relation.a == my_team || relation.b == my_team {
                        first_war.get_or_insert(turn);
                        if relation.declarers.contains(&my_team) {
                            first_declare.get_or_insert(turn);
                            declares.push(turn);
                        }
                    }
                }
                Event::Battle { attacker_owner, .. } if *attacker_owner == seat => attacks += 1,
                Event::CityFounded { owner, .. } if *owner == seat => founded += 1,
                Event::CityCaptured { from, to, city, .. } => {
                    if *to == seat && before.team_of(*from) != my_team && !from.is_barbarian() {
                        captures.push(turn);
                        took_from.push(*from);
                        if before
                            .cities
                            .get(city)
                            .is_some_and(|c| c.original_capital && c.owner == *from)
                        {
                            capitals.push(turn);
                        }
                    }
                    if *from == seat {
                        lost += 1;
                    }
                }
                Event::CityRazed { from, .. } if *from == seat => lost += 1,
                _ => {}
            }
        }
        for event in &events {
            if let Event::PlayerEliminated { player } = event
                && took_from.contains(player)
            {
                eliminated.push(turn);
            }
        }
    }
    let by = |v: &Vec<u32>, t: u32| v.iter().filter(|&&x| x <= t).count();
    let total_vp: i64 = state
        .players
        .iter()
        .filter(|p| !p.id.is_barbarian())
        .map(|p| scoring::vp_total(&state, p.id) as i64)
        .sum();
    let my_vp = scoring::vp_total(&state, seat) as i64;
    json!({
        "seed": game_seed,
        "seat": seat_index,
        "faction": format!("{:?}", setup.players[seat_index].faction),
        "nearest": nearest,
        "won": state.finished && state.winner == Some(my_team),
        "finished": state.finished,
        "end_turn": state.turn,
        "alive": state.player(seat).alive,
        "vp_share": if total_vp > 0 { my_vp as f64 / total_vp as f64 } else { 0.0 },
        "first_declare": first_declare,
        "first_war": first_war,
        "declares": declares,
        "first_capture": captures.first(),
        "captures": captures,
        "capitals": capitals,
        "eliminated": eliminated,
        "cap30": by(&captures, 30), "cap50": by(&captures, 50), "cap75": by(&captures, 75),
        "capital50": by(&capitals, 50), "elim50": by(&eliminated, 50), "elim75": by(&eliminated, 75),
        "attacks": attacks,
        "founded": founded,
        "lost": lost,
        "army_at": army_at,
        "cities_at": cities_at,
        "pop_at": pop_at,
        "gold_at": gold_at,
        "techs_at": techs_at,
        "cities_end": state.cities_of(seat).count(),
    })
}

fn main() {
    let args = parse_args();
    // Seed-major, so an interrupted run holds paired games.
    let arms = args.arms.len();
    let jobs: Vec<(usize, u64)> = (0..args.games)
        .flat_map(|i| (0..arms).map(move |a| (a, i)))
        .collect();
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
            let mut row = play(&args, &args.arms[a].1, i);
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
        let num = |k: &'static str| move |g: &Value| g[k].as_f64().unwrap_or(0.0);
        eprintln!(
            "{label}: n={} declare<=T30 {:.0}% cap50 {:.2} capital50 {:.2} elim50 {:.2} cap75 {:.2} army T30/T50 {:.1}/{:.1} cities T50/T100 {:.1}/{:.1} win {:.1}% vp_share {:.3} alive {:.0}%",
            games.len(),
            100.0 * count(&|g| g["first_declare"].as_u64().is_some_and(|t| t <= 30)),
            mean(&num("cap50")),
            mean(&num("capital50")),
            mean(&num("elim50")),
            mean(&num("cap75")),
            mean(&|g| g["army_at"]["30"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["army_at"]["50"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["cities_at"]["50"].as_f64().unwrap_or(0.0)),
            mean(&|g| g["cities_at"]["100"].as_f64().unwrap_or(0.0)),
            100.0 * count(&|g| g["won"] == json!(true)),
            mean(&|g| g["vp_share"].as_f64().unwrap_or(0.0)),
            100.0 * count(&|g| g["alive"] == json!(true)),
        );
    }
}

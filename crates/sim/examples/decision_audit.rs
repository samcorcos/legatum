//! Decision audit: replay a recorded game, ask another decision source what it would have
//! ordered in one seat every turn, classify the differences, and value them by branching.
//!
//!   decision_audit --tape TAPE.json --out DIR [--seat 0] [--alt SOURCE] [--continue BRAIN,...]
//!                  [--memory TURN=FILE] [--every 5] [--turns 40,60] [--from T] [--to T]
//!                  [--salts 32] [--horizons 20,50] [--dice reseed|fixed] [--no-hybrids]
//!                  [--window K] [--handover keep|clear] [--diff-only] [--army-detail]
//!                  [--settle-detail] [--prod-detail] [--peace-detail] [--memory-reset]
//!                  [--top 30] [--end-turn 161] [--war-detail]
//!                  [--event-windows 5] [--rival --target-seat 0 --top 8]
//!                  [--fixture-turns 45,140]
//!
//! SOURCE (default `hard`): `hard` is the app's Hard (`ScriptedBrain::with_difficulty(.., Hard)`),
//! `params:FILE` is Hard with AiParams (a candidate gene), `orders:FILE.jsonl` reads lines
//! `{"turn": T, "orders": [Order, ...]}` from any other player (an LLM, a playbook). BRAIN (default
//! `hard`, or `params:FILE`) plays the seat after the branch turn in every arm. A comma list
//! runs every continuation on the same arms and salts. `--memory-reset` adds a cold Hard
//! continuation whose seat brain has no warmed opponent memory at the branch turn.
//!
//! Diff (every tape turn): the seat's Hard shadow brain sees the seat's own PlayerView each turn
//! (orders discarded, so its memory follows the real game); every other seat has a Hard shadow
//! brain too, which gives the branches each AI's real memory and checks that the tape's AI orders
//! reproduce. The turn is resolved twice from the same state, once with the tape's orders and
//! once with the seat's orders swapped for SOURCE's, and the two results are compared by category
//! (production, purchases, research, diplomacy, settling, army, garrison, other). `diff.jsonl`.
//!
//! Value (sampled turns): from the same state, arms that differ only in the seat's orders on that
//! one turn: `rec` (the tape), `alt` (SOURCE), `alt:<category>` (the tape with only that
//! category's orders swapped for SOURCE's) and `rec:<category>` (SOURCE with only that category
//! kept from the tape). Mixing two planners costs something by itself, so the report gives both
//! swaps and their mean (the two-player Shapley share of the category). After that turn BRAIN plays the seat, every other seat
//! is Hard with its restored memory. Salt s > 0 salts every brain seed and, with `--dice reseed`
//! (default), the dice stream. The map seed and every era's objectives stay fixed;
//! salt 0 is the recorded world. Each run records the seat's VP, cities, win chance
//! (`winchance`, 0..10000, or 10000/0 once the game is over) and the VP lead at each horizon.
//! `--window K` (a brain SOURCE only) swaps a policy instead of one turn: for K turns from the branch
//! turn the seat plays the tape's orders (`rec`), SOURCE live (`alt`), or the tape with one category
//! from SOURCE live (`alt:<category>`), and BRAIN takes over after that. Decisions SOURCE re-makes every
//! turn (production, research) barely show over one turn, since BRAIN re-decides them on the next.
//! The tape's orders misfire once the branch drifts from the recorded game (reseeded dice), so keep
//! K short (5 to 10). At the handover the seat's standing orders (destinations, paths, automation,
//! Sleep) carry over into BRAIN's play unless `--handover clear` drops them in every arm.
//! `value.jsonl`; `tools/decision-audit/report.py` turns both files into the report.
//!
//! `--army-detail` (a brain SOURCE) also writes `army.jsonl` in the diff: one row per military
//! unit of the seat each turn, with its HP, both resolutions (where it ended, its attacks and
//! kills), the nearest enemy city and unit at war before and after each, and the planning
//! passes that gave SOURCE's order for it (`ScriptedBrain::order_diagnosis`, which plans
//! exactly as an untraced call does).
//!
//! `--settle-detail` (a brain SOURCE) also writes `settle.jsonl`: for every Settler of the seat on
//! every turn, the tape's and SOURCE's orders for it and `ScriptedBrain::settler_diagnosis` (the
//! plan branch, the verdict on its own tile, the best site of each kind and, for a Settler the
//! tape later founds with, the search's verdict on that founding tile: "probe tile ..."), with the
//! distances from our nearest city to the Settler and to his founding tile.
//!
//! `--states FILE.jsonl` uses hash-verified original-engine states exported by `audit_states`
//! for the shadow replay and branch positions. Branches still resolve with this engine.
//!
//! `--prod-detail` (a brain SOURCE) also writes `prod.jsonl`: one row per own city,
//! including pre-scorer production passes, with what the tape
//! and SOURCE leave it producing after the turn, SOURCE's value and score for both items (the
//! tape's is absent when SOURCE never scored it: an earlier rule chose for the city), SOURCE's top
//! options, and the city's surroundings (nearest foreign city, nearest enemy unit at war).
//!
//! `--peace-detail` (a brain SOURCE) also writes `peace.jsonl`: one row per Peace proposal the seat
//! must answer, with the tape's and SOURCE's answers and `ScriptedBrain::peace_diagnosis` (the
//! gate that decided, the score and its terms).
//! `--war-detail` writes Hard's plan, reserve and campaign diagnostics for each turn. `--top N`
//! samples the N highest-priority resolved turn/category differences, with category and phase
//! coverage, and branches only that category from each turn. `--end-turn T` scores all branches
//! at the same absolute turn (or their earlier finish) instead of relative horizons.
use legatum_ai::winchance::WinFacts;
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::orders::Action;
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, Order, PlayerId, UnitId, player_view, resolve_turn,
    rules::is_military, scoring, validate_orders,
};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const CATS: [&str; 8] = [
    "production",
    "research",
    "diplomacy",
    "settling",
    "army",
    "garrison",
    "other",
    "purchase",
];

#[derive(Clone)]
enum Source {
    Hard(Option<Box<AiParams>>),
    Orders(BTreeMap<u32, Vec<Order>>),
}

struct Args {
    tape: String,
    initial_state: Option<String>,
    out: std::path::PathBuf,
    seat: PlayerId,
    alt: Source,
    alt_label: String,
    continuations: Vec<Continuation>,
    memory: Option<(u32, String)>,
    states: Option<String>,
    every: u32,
    turns: Option<BTreeSet<u32>>,
    from: u32,
    to: u32,
    salts: u64,
    horizons: Vec<u32>,
    reseed: bool,
    hybrids: bool,
    diff_only: bool,
    window: u32,
    clear_at_handover: bool,
    army_detail: bool,
    settle_detail: bool,
    prod_detail: bool,
    peace_detail: bool,
    war_detail: bool,
    top: Option<usize>,
    event_windows: Option<u32>,
    rival: bool,
    target_seat: PlayerId,
    fixture_turns: BTreeSet<u32>,
    end_turn: Option<u32>,
}

struct Continuation {
    name: String,
    params: Option<AiParams>,
    memory_reset: bool,
}

fn load_params(path: &str) -> AiParams {
    let params: AiParams =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read params"))
            .expect("params json");
    params.validate().expect("legal params");
    params
}

fn parse_args() -> Args {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args {
        tape: String::new(),
        initial_state: None,
        out: "decision-audit".into(),
        seat: PlayerId(0),
        alt: Source::Hard(None),
        alt_label: "hard".into(),
        continuations: vec![Continuation {
            name: "hard".into(),
            params: None,
            memory_reset: false,
        }],
        memory: None,
        states: None,
        every: 5,
        turns: None,
        from: 0,
        to: u32::MAX,
        salts: 32,
        horizons: vec![20, 50],
        reseed: true,
        hybrids: true,
        diff_only: false,
        window: 1,
        clear_at_handover: false,
        army_detail: false,
        settle_detail: false,
        prod_detail: false,
        peace_detail: false,
        war_detail: false,
        top: None,
        event_windows: None,
        rival: false,
        target_seat: PlayerId(0),
        fixture_turns: BTreeSet::new(),
        end_turn: None,
    };
    let mut it = a.into_iter();
    let mut memory_reset = false;
    while let Some(flag) = it.next() {
        let mut v = || it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => args.tape = v(),
            "--initial-state" => args.initial_state = Some(v()),
            "--states" => args.states = Some(v()),
            "--out" => args.out = v().into(),
            "--seat" => args.seat = PlayerId(v().parse().unwrap()),
            "--alt" => {
                let s = v();
                args.alt = if s == "hard" {
                    Source::Hard(None)
                } else if let Some(p) = s.strip_prefix("params:") {
                    Source::Hard(Some(Box::new(load_params(p))))
                } else if let Some(p) = s.strip_prefix("orders:") {
                    let mut m = BTreeMap::new();
                    for line in std::fs::read_to_string(p).expect("read orders").lines() {
                        if line.trim().is_empty() {
                            continue;
                        }
                        let row: Value = serde_json::from_str(line).expect("orders line");
                        let turn = row["turn"].as_u64().expect("turn") as u32;
                        let orders: Vec<Order> =
                            serde_json::from_value(row["orders"].clone()).expect("orders");
                        m.insert(turn, orders);
                    }
                    Source::Orders(m)
                } else {
                    panic!("--alt hard|params:FILE|orders:FILE")
                };
                args.alt_label = s;
            }
            "--continue" => {
                let s = v();
                args.continuations = s
                    .split(',')
                    .map(|name| Continuation {
                        name: name.to_string(),
                        params: match name.strip_prefix("params:") {
                            Some(p) if !p.is_empty() => Some(load_params(p)),
                            None if name == "hard" => None,
                            _ => panic!("--continue hard|params:FILE[,hard|params:FILE...]"),
                        },
                        memory_reset: false,
                    })
                    .collect();
            }
            "--memory-reset" => memory_reset = true,
            "--memory" => {
                let s = v();
                let (t, f) = s.split_once('=').expect("--memory TURN=FILE");
                args.memory = Some((t.parse().unwrap(), f.to_string()));
            }
            "--every" => args.every = v().parse().unwrap(),
            "--turns" => {
                args.turns = Some(v().split(',').map(|s| s.parse().unwrap()).collect());
            }
            "--from" => args.from = v().parse().unwrap(),
            "--to" => args.to = v().parse().unwrap(),
            "--salts" => args.salts = v().parse().unwrap(),
            "--horizons" => args.horizons = v().split(',').map(|s| s.parse().unwrap()).collect(),
            "--dice" => args.reseed = v() == "reseed",
            "--no-hybrids" => args.hybrids = false,
            "--diff-only" => args.diff_only = true,
            "--window" => args.window = v().parse().unwrap(),
            "--handover" => args.clear_at_handover = v() == "clear",
            "--army-detail" => args.army_detail = true,
            "--settle-detail" => args.settle_detail = true,
            "--prod-detail" => args.prod_detail = true,
            "--peace-detail" => args.peace_detail = true,
            "--war-detail" => args.war_detail = true,
            "--top" => args.top = Some(v().parse().unwrap()),
            "--event-windows" => {
                let n: u32 = v().parse().unwrap();
                assert!((2..=10).contains(&n), "--event-windows needs 2..10 turns");
                args.event_windows = Some(n);
                args.window = n;
            }
            "--rival" => args.rival = true,
            "--target-seat" => args.target_seat = PlayerId(v().parse().unwrap()),
            "--fixture-turns" => {
                args.fixture_turns = v().split(',').map(|s| s.parse().unwrap()).collect()
            }
            "--end-turn" => args.end_turn = Some(v().parse().unwrap()),
            other => panic!("unknown option {other}"),
        }
    }
    assert!(!args.tape.is_empty(), "--tape is required");
    assert!(args.top.is_none_or(|n| n > 0), "--top must be positive");
    assert!(
        args.top.is_none() || args.event_windows.is_none(),
        "choose --top or --event-windows"
    );
    assert!(
        !args.rival || args.top.is_some(),
        "--rival requires --top N"
    );
    assert!(
        !args.rival || args.seat != args.target_seat,
        "rival and target must be different seats"
    );
    if memory_reset {
        args.continuations.push(Continuation {
            name: "hard:cold".into(),
            params: None,
            memory_reset: true,
        });
    }
    assert!(!args.continuations.is_empty(), "--continue needs a BRAIN");
    let names: BTreeSet<_> = args.continuations.iter().map(|c| &c.name).collect();
    assert_eq!(
        names.len(),
        args.continuations.len(),
        "duplicate continuation"
    );
    args.horizons.sort_unstable();
    args
}

fn salted(seed: u64, salt: u64) -> u64 {
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

fn reseed_continuation_dice(state: &mut GameState, salt: u64) {
    if salt != 0 {
        state.dice_seed = Some(salted(state.setup.seed, salt ^ 0xD1CE));
    }
}

/// A Hard brain for `p` with the app's seed for the game's original seed, salted.
fn hard_brain(
    state: &GameState,
    game_seed: u64,
    p: PlayerId,
    params: Option<&AiParams>,
    salt: u64,
) -> ScriptedBrain {
    let seed = salted(legatum_wasm::session::brain_seed(game_seed, p), salt);
    let faction = state.player(p).faction;
    match params {
        Some(pr) => ScriptedBrain::with_params(faction, seed, p, AiDifficulty::Hard, pr),
        None => ScriptedBrain::with_difficulty(faction, seed, p, AiDifficulty::Hard),
    }
}

fn order_unit(o: &Order) -> Option<UnitId> {
    match o {
        Order::SetAutomation { unit, .. }
        | Order::SetAutoFire { unit, .. }
        | Order::MoveUnit { unit, .. }
        | Order::MoveUnitPath { unit, .. }
        | Order::SleepUnit { unit, .. }
        | Order::SkipUnit { unit }
        | Order::FoundCity { unit }
        | Order::DisbandUnit { unit }
        | Order::RangedAttack { unit, .. }
        | Order::UpgradeUnit { unit }
        | Order::UpgradeSiege { unit } => Some(*unit),
        Order::AirStrike { plane, .. } | Order::Rebase { plane, .. } => Some(*plane),
        _ => None,
    }
}

/// The category of a unit at the start of the turn.
fn unit_cat(state: &GameState, seat: PlayerId, unit: UnitId) -> usize {
    let Some(u) = state.units.get(&unit) else {
        return 6;
    };
    if u.kind == legatum_core::UnitKind::Settler {
        return 3;
    }
    if legatum_core::data::is_aircraft(u.kind) {
        return 4;
    }
    if !is_military(u.kind) {
        return 6;
    }
    let garrison = state.cities_of(seat).any(|c| c.pos.distance(u.pos) <= 1);
    if garrison { 5 } else { 4 }
}

fn order_cat(state: &GameState, seat: PlayerId, o: &Order) -> usize {
    match o {
        Order::SetProduction { .. } | Order::SetProductionQueue { .. } => 0,
        Order::BuyItem { .. }
        | Order::HireCompany { .. }
        | Order::UpgradeUnit { .. }
        | Order::UpgradeSiege { .. } => 7,
        Order::SetResearch { .. } | Order::SetResearchQueue { .. } => 1,
        Order::Action {
            action: Action::ChooseFreeTechs { .. },
        } => 1,
        Order::DeclareWar { .. }
        | Order::BreakTreaty { .. }
        | Order::ProposeTreaty { .. }
        | Order::RespondToProposal { .. }
        | Order::WithdrawProposal { .. } => 2,
        _ => match order_unit(o) {
            Some(u) => unit_cat(state, seat, u),
            None => 6,
        },
    }
}

fn short(o: &Order) -> String {
    let s = format!("{o:?}");
    let s = s.split(", explanation").next().unwrap_or(&s).to_string();
    let s = s.split(", path:").next().unwrap_or(&s).to_string();
    if s.len() > 140 {
        format!("{}...", &s[..137])
    } else {
        s
    }
}

fn strategic(o: &Order) -> bool {
    matches!(
        o,
        Order::DeclareWar { .. }
            | Order::ProposeTreaty { .. }
            | Order::RespondToProposal { .. }
            | Order::SetProduction { .. }
            | Order::BuyItem { .. }
    )
}

/// Orders of `rec` outside the categories in `swap`, plus `alt`'s orders inside them.
fn hybrid(
    state: &GameState,
    seat: PlayerId,
    rec: &[Order],
    alt: &[Order],
    swap: &[usize],
) -> Vec<Order> {
    let mut out: Vec<Order> = rec
        .iter()
        .filter(|o| !swap.contains(&order_cat(state, seat, o)))
        .cloned()
        .collect();
    out.extend(
        alt.iter()
            .filter(|o| swap.contains(&order_cat(state, seat, o)))
            .cloned(),
    );
    out
}

fn battles_by(events: &[Event], seat: PlayerId) -> u32 {
    events
        .iter()
        .filter(|e| matches!(e, Event::Battle { attacker_owner, .. } if *attacker_owner == seat))
        .count() as u32
}

fn bought(events: &[Event], seat: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::ItemBought {
                city,
                owner,
                item,
                gold,
            } if *owner == seat => Some(format!("c{}:{:?}:{}g", city.0, item, gold)),
            _ => None,
        })
        .collect();
    v.sort();
    v
}

/// Compare the two resolutions of one turn, per category.
#[allow(clippy::too_many_arguments)]
fn diff_turn(
    s: &GameState,
    seat: PlayerId,
    rec: &[Order],
    alt: &[Order],
    a: &GameState,
    ea: &[Event],
    b: &GameState,
    eb: &[Event],
) -> Value {
    let mut cats = serde_json::Map::new();
    let by_cat = |orders: &[Order], c: usize| -> Vec<String> {
        orders
            .iter()
            .filter(|o| order_cat(s, seat, o) == c)
            .map(short)
            .collect()
    };
    let units_moved_differently = |c: usize| -> Vec<Value> {
        s.units
            .values()
            .filter(|u| u.owner == seat && unit_cat(s, seat, u.id) == c)
            .filter_map(|u| {
                let pa = a.units.get(&u.id).map(|x| x.pos);
                let pb = b.units.get(&u.id).map(|x| x.pos);
                (pa != pb).then(|| {
                    json!({"unit": u.id.0, "kind": format!("{:?}", u.kind), "from": [u.pos.q, u.pos.r],
                        "rec": pa.map(|p| [p.q, p.r]), "alt": pb.map(|p| [p.q, p.r])})
                })
            })
            .collect()
    };
    for (c, name) in CATS.iter().enumerate() {
        let ro = by_cat(rec, c);
        let ao = by_cat(alt, c);
        let rs: BTreeSet<&String> = ro.iter().collect();
        let as_: BTreeSet<&String> = ao.iter().collect();
        let order_diff = rs.symmetric_difference(&as_).count();
        let mut entry =
            json!({"rec_orders": ro.len(), "alt_orders": ao.len(), "order_diff": order_diff});
        let mut n = match c {
            0 => {
                let prod: Vec<Value> = s
                    .cities_of(seat)
                    .filter_map(|city| {
                        let pa = a.cities.get(&city.id).map(|x| format!("{:?}", x.producing));
                        let pb = b.cities.get(&city.id).map(|x| format!("{:?}", x.producing));
                        (pa != pb).then(|| json!({"city": city.name, "rec": pa, "alt": pb}))
                    })
                    .collect();
                entry["producing_differs"] = json!(prod);
                prod.len()
            }
            1 => {
                let (ra, rb) = (a.player(seat).researching, b.player(seat).researching);
                entry["rec_research"] = json!(format!("{ra:?}"));
                entry["alt_research"] = json!(format!("{rb:?}"));
                usize::from(ra != rb)
            }
            2 => order_diff,
            3 => {
                let moved = units_moved_differently(c);
                let fa = ea
                    .iter()
                    .filter(|e| matches!(e, Event::CityFounded { owner, .. } if *owner == seat))
                    .count();
                let fb = eb
                    .iter()
                    .filter(|e| matches!(e, Event::CityFounded { owner, .. } if *owner == seat))
                    .count();
                let n = moved.len() + fa.abs_diff(fb);
                entry["units"] = json!(moved);
                entry["rec_founded"] = json!(fa);
                entry["alt_founded"] = json!(fb);
                n
            }
            7 => {
                let (ba, bb) = (bought(ea, seat), bought(eb, seat));
                entry["rec_bought"] = json!(ba);
                entry["alt_bought"] = json!(bb);
                entry["rec_gold_after"] = json!(a.player(seat).gold);
                entry["alt_gold_after"] = json!(b.player(seat).gold);
                order_diff + usize::from(ba != bb)
            }
            _ => {
                let moved = units_moved_differently(c);
                let n = moved.len();
                entry["units"] = json!(moved);
                n
            }
        };
        if c == 4 {
            let (ba, bb) = (battles_by(ea, seat), battles_by(eb, seat));
            n += ba.abs_diff(bb) as usize;
            entry["rec_attacks"] = json!(ba);
            entry["alt_attacks"] = json!(bb);
        }
        if c == 4 || c == 5 {
            let captures = |events: &[Event]| -> usize {
                events
                    .iter()
                    .filter(|e| matches!(e, Event::SettlerCaptured { to, .. } if *to == seat))
                    .count()
            };
            let (ca, cb) = (captures(ea), captures(eb));
            n += ca.abs_diff(cb);
            entry["rec_settlers_captured"] = json!(ca);
            entry["alt_settlers_captured"] = json!(cb);
        }
        entry["n"] = json!(n);
        if n > 0 || order_diff > 0 {
            entry["rec"] = json!(rs.difference(&as_).take(8).collect::<Vec<_>>());
            entry["alt"] = json!(as_.difference(&rs).take(8).collect::<Vec<_>>());
        }
        cats.insert(name.to_string(), entry);
    }
    Value::Object(cats)
}

/// `--army-detail`: one row per military unit of the seat (not aircraft) this turn.
/// `--prod-detail`: per city SOURCE's production planner scored, what each side leaves it
/// producing and how SOURCE valued both items.
fn prod_rows(
    s: &GameState,
    seat: PlayerId,
    turn: u32,
    a: &GameState,
    b: &GameState,
    trace: &legatum_ai::OrderTrace,
) -> Vec<Value> {
    let view = player_view(s, seat);
    let own_military = s
        .units
        .values()
        .filter(|u| u.owner == seat && is_military(u.kind))
        .count();
    let at_war = s
        .players
        .iter()
        .any(|o| o.alive && o.id != seat && s.at_war(seat, o.id));
    s.cities_of(seat)
        .map(|city| {
            let p = trace.production.iter().find(|p| p.city == city.id);
            let his = a.cities.get(&city.id).and_then(|c| c.producing);
            let hard = b.cities.get(&city.id).and_then(|c| c.producing);
            let scored = |item: Option<legatum_core::ProductionItem>| {
                item.and_then(|i| p.and_then(|p| p.options.iter().find(|o| o.0 == i)))
                    .map(|o| [o.1, o.2])
            };
            let mut top: Vec<_> = p.into_iter().flat_map(|p| &p.options).collect();
            top.sort_by_key(|o| -o.2);
            let foreign = view
                .cities
                .iter()
                .filter(|c| c.owner != seat && !c.owner.is_barbarian())
                .map(|c| c.pos.distance(city.pos))
                .min();
            let enemy_unit = view
                .units
                .iter()
                .filter(|u| u.owner != seat && is_military(u.kind) && view.at_war(seat, u.owner))
                .map(|u| u.pos.distance(city.pos))
                .min();
            json!({"turn": turn, "city": city.name, "city_id": city.id.0,
                "passes": trace.city_passes.get(&city.id.0),
                "defense": trace.production_defense.iter().find(|p| p.city == city.id),
                "before": city.producing.map(|i| format!("{i:?}")),
                "scored": p.is_some(), "pop": city.population,
                "capital": city.is_capital, "hp": city.hp, "max_hp": view.cities.iter().find(|c| c.id == city.id).map(|c| c.max_hp),
                "capture_alarms": view.cities.iter().filter(|c| c.owner == seat && c.id != city.id
                    && c.hp * 2 <= c.max_hp && c.pos.distance(city.pos) <= 8)
                    .map(|c| json!({"city": c.name, "resistance": c.detail.as_ref().map(|d| d.resistance),
                        "enemy_near": view.units.iter().filter(|u| u.military && !u.owner.is_barbarian()
                            && view.at_war(seat, u.owner)).map(|u| u.pos.distance(c.pos)).min()}))
                    .collect::<Vec<_>>(),
                "at_war": at_war, "military": own_military,
                "cities": s.cities_of(seat).count(), "gold": s.player(seat).gold,
                "foreign_city_d": foreign, "enemy_unit_d": enemy_unit,
                "his": his.map(|i| format!("{i:?}")), "hard": hard.map(|i| format!("{i:?}")),
                "chosen": p.and_then(|p| p.chosen).map(|i| format!("{i:?}")),
                "his_scored": scored(his), "hard_scored": scored(hard),
                "top": top.iter().take(6).map(|o| json!([format!("{:?}", o.0), o.1, o.2])).collect::<Vec<_>>(),
                "settler": p.and_then(|p| p.settler)})
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn army_rows(
    s: &GameState,
    seat: PlayerId,
    rec: &[Order],
    alt: &[Order],
    a: &GameState,
    ea: &[Event],
    b: &GameState,
    eb: &[Event],
    trace: Option<&legatum_ai::OrderTrace>,
) -> Vec<Value> {
    let view = player_view(s, seat);
    let war_cities: Vec<(legatum_core::Hex, i32, i32, bool)> = view
        .cities
        .iter()
        .filter(|c| c.owner != seat && !c.owner.is_barbarian() && view.at_war(seat, c.owner))
        .map(|c| (c.pos, c.hp, c.max_hp, c.visible))
        .collect();
    let war_units: Vec<(legatum_core::Hex, i32, legatum_core::UnitKind)> = view
        .units
        .iter()
        .filter(|u| u.owner != seat && is_military(u.kind) && view.at_war(seat, u.owner))
        .map(|u| (u.pos, u.hp, u.kind))
        .collect();
    let near_city = |p: Option<legatum_core::Hex>| {
        p.and_then(|p| war_cities.iter().map(|c| c.0.distance(p)).min())
    };
    let near_unit = |p: Option<legatum_core::Hex>| {
        p.and_then(|p| war_units.iter().map(|e| e.0.distance(p)).min())
    };
    let fights = |ev: &[Event], u: UnitId| -> (u32, u32) {
        let mut n = (0, 0);
        for e in ev {
            if let Event::Battle {
                attacker,
                defender_hp_after,
                ..
            } = e
                && attacker.unit() == Some(u)
            {
                n.0 += 1;
                n.1 += u32::from(*defender_hp_after <= 0);
            }
        }
        n
    };
    let orders_of = |os: &[Order], u: UnitId| -> Vec<String> {
        os.iter()
            .filter(|o| order_unit(o) == Some(u))
            .map(short)
            .collect()
    };
    let own_city_at = |p: legatum_core::Hex| s.cities_of(seat).any(|c| c.pos == p);
    let mut rows = Vec::new();
    for u in s.units.values().filter(|u| {
        u.owner == seat && is_military(u.kind) && !legatum_core::data::is_aircraft(u.kind)
    }) {
        let pa = a.units.get(&u.id);
        let pb = b.units.get(&u.id);
        let (ra, ka) = fights(ea, u.id);
        let (rb, kb) = fights(eb, u.id);
        let passes = trace.and_then(|t| t.passes.get(&u.id.0)).cloned();
        let dropped_at = trace.and_then(|t| {
            t.stages
                .iter()
                .find(|st| {
                    st.rejected
                        .iter()
                        .any(|r| order_unit(&r.order) == Some(u.id))
                })
                .map(|st| st.name)
        });
        let wounded_near = war_units
            .iter()
            .filter(|e| e.0.distance(u.pos) <= 2 && e.1 < 100)
            .count();
        rows.push(json!({
            "turn": s.turn, "unit": u.id.0, "kind": format!("{:?}", u.kind),
            "hp": u.hp, "from": [u.pos.q, u.pos.r],
            "cat": CATS[unit_cat(s, seat, u.id)], "in_city": own_city_at(u.pos),
            "rec_to": pa.map(|x| [x.pos.q, x.pos.r]), "alt_to": pb.map(|x| [x.pos.q, x.pos.r]),
            "rec_hp": pa.map(|x| x.hp), "alt_hp": pb.map(|x| x.hp),
            "rec_in_city": pa.is_some_and(|x| own_city_at(x.pos)),
            "alt_in_city": pb.is_some_and(|x| own_city_at(x.pos)),
            "rec_attacks": ra, "rec_kills": ka, "alt_attacks": rb, "alt_kills": kb,
            "rec_orders": orders_of(rec, u.id), "alt_orders": orders_of(alt, u.id),
            "alt_passes": passes, "alt_dropped_at": dropped_at,
            "city_d": [near_city(Some(u.pos)), near_city(pa.map(|x| x.pos)), near_city(pb.map(|x| x.pos))],
            "unit_d": [near_unit(Some(u.pos)), near_unit(pa.map(|x| x.pos)), near_unit(pb.map(|x| x.pos))],
            "wounded_near": wounded_near,
            "war": !war_cities.is_empty() || !war_units.is_empty(),
        }));
    }
    rows
}

/// One value run: snapshot index, arm label, the seat's orders on the branch turn, the swapped
/// categories, salt, continuation index.
type Job = (usize, String, Vec<Order>, Vec<usize>, u64, usize);

struct Snapshot {
    state: GameState,
    memory: BTreeMap<PlayerId, OpponentMemory>,
    rec: Vec<Order>,
    alt: Vec<Order>,
    tape_orders: legatum_wasm::session::OrderMap,
    /// Categories whose orders or outcomes differ this turn.
    differs: Vec<usize>,
    cats: Value,
}

#[derive(Clone)]
struct EventWindow {
    event_turn: u32,
    start: u32,
    kind: &'static str,
    cats: Vec<usize>,
}

fn event_windows(events: &[Event], seat: PlayerId, turn: u32, k: u32) -> Vec<EventWindow> {
    let start = turn.saturating_sub(k - 1).max(1);
    events
        .iter()
        .filter_map(|event| {
            let (kind, cats) = match event {
                Event::CityCaptured { to, from, .. } if *to == seat || *from == seat => {
                    ("city_capture", vec![4, 5])
                }
                Event::SettlerCaptured { to, from, .. } if *to == seat => {
                    ("settler_snipe", vec![3, 4, 5])
                }
                Event::SettlerCaptured { from, .. } if *from == seat => {
                    ("settler_lost", vec![3, 4, 5])
                }
                Event::Battle {
                    attacker_owner,
                    defender: legatum_core::events::Defender::City { .. },
                    ..
                } if *attacker_owner == seat => ("siege", vec![4]),
                _ => return None,
            };
            Some(EventWindow {
                event_turn: turn,
                start,
                kind,
                cats,
            })
        })
        .collect()
}

/// Candidate orders use only the rival's view and its own production scorer. Full state is
/// consulted solely to reject illegal orders; it never supplies a target or an option.
fn rival_options(
    state: &GameState,
    seat: PlayerId,
    view: &legatum_core::PlayerView,
    planned: &[Order],
    trace: &legatum_ai::OrderTrace,
) -> Vec<(&'static str, i32, usize, Vec<Order>)> {
    let mut out = Vec::new();
    let mut add = |kind, score, cat, orders: Vec<Order>| {
        if orders != planned && validate_orders(state, seat, &orders).is_empty() {
            out.push((kind, score, cat, orders));
        }
    };
    for row in &trace.production {
        let mut options = row.options.clone();
        options.sort_by_key(|(_, _, score)| std::cmp::Reverse(*score));
        for (item, _, score) in options
            .into_iter()
            .filter(|(item, _, _)| Some(item) != row.chosen.as_ref())
            .take(2)
        {
            let mut orders: Vec<Order> = planned.iter().filter(|o| !matches!(o,
                Order::SetProduction { city, .. } | Order::SetProductionQueue { city, .. } if *city == row.city
            )).cloned().collect();
            orders.push(Order::SetProduction {
                city: row.city,
                item,
            });
            add("scored_production", score, 0, orders);
        }
    }
    for city in view.cities.iter().filter(|c| c.owner == seat) {
        if let Some(detail) = city.detail.as_ref() {
            for option in detail
                .options
                .iter()
                .filter(|o| {
                    o.buy_blocker.is_none()
                        && o.buy_cost.is_some_and(|cost| cost <= view.economy.gold)
                })
                .take(3)
            {
                if planned
                    .iter()
                    .any(|o| matches!(o, Order::BuyItem { city: id, .. } if *id == city.id))
                {
                    continue;
                }
                let mut orders = planned.to_vec();
                orders.push(Order::BuyItem {
                    city: city.id,
                    item: option.item,
                });
                add(
                    "buy",
                    1000 - option.buy_cost.unwrap_or(1000) / 10,
                    7,
                    orders,
                );
            }
        }
    }
    let mut attacks = Vec::new();
    let targets: Vec<_> = view
        .units
        .iter()
        .filter(|u| u.owner != seat)
        .map(|u| u.pos)
        .chain(
            view.cities
                .iter()
                .filter(|c| c.owner != seat && c.visible)
                .map(|c| c.pos),
        )
        .collect();
    for unit in view
        .units
        .iter()
        .filter(|u| u.owner == seat && u.military && u.moves_left > 0)
    {
        for target in &targets {
            if let Some(f) = legatum_core::forecast::forecast_attack(view, unit.id, *target) {
                let score = f.expected_damage_to_defender * 10 - f.expected_damage_to_attacker * 5;
                if score <= 0 {
                    continue;
                }
                let order = if f.ranged {
                    Order::RangedAttack {
                        unit: unit.id,
                        target: *target,
                    }
                } else {
                    Order::MoveUnit {
                        unit: unit.id,
                        to: *target,
                    }
                };
                attacks.push((score, unit.id, order));
            }
        }
    }
    attacks.sort_by_key(|(score, unit, _)| (std::cmp::Reverse(*score), unit.0));
    for (score, unit, order) in attacks.into_iter().take(3) {
        let mut orders: Vec<Order> = planned
            .iter()
            .filter(|o| order_unit(o) != Some(unit))
            .cloned()
            .collect();
        orders.push(order);
        add("forecast_attack", score, 4, orders);
    }
    for settler in view
        .units
        .iter()
        .filter(|u| u.owner == seat && u.kind == legatum_core::UnitKind::Settler)
    {
        if let Some(city) = view
            .cities
            .iter()
            .filter(|c| c.owner == seat && c.pos != settler.pos)
            .min_by_key(|c| c.pos.distance(settler.pos))
            && city.pos.distance(settler.pos) <= 8
        {
            let mut orders: Vec<Order> = planned
                .iter()
                .filter(|o| order_unit(o) != Some(settler.id))
                .cloned()
                .collect();
            orders.push(Order::MoveUnit {
                unit: settler.id,
                to: city.pos,
            });
            add(
                "settler_safety",
                800 - city.pos.distance(settler.pos) * 10,
                3,
                orders,
            );
        }
        if let Some(guard) = view
            .units
            .iter()
            .filter(|u| u.owner == seat && u.military && u.pos != settler.pos)
            .min_by_key(|u| u.pos.distance(settler.pos))
            && guard.pos.distance(settler.pos) <= 3
        {
            let mut orders: Vec<Order> = planned
                .iter()
                .filter(|o| order_unit(o) != Some(guard.id))
                .cloned()
                .collect();
            orders.push(Order::MoveUnit {
                unit: guard.id,
                to: settler.pos,
            });
            add(
                "settler_escort",
                800 - guard.pos.distance(settler.pos) * 10,
                4,
                orders,
            );
        }
    }
    for (i, order) in planned.iter().enumerate() {
        if let Order::RespondToProposal {
            proposal, accept, ..
        } = order
        {
            let mut orders = planned.to_vec();
            orders[i] = Order::RespondToProposal {
                proposal: *proposal,
                accept: !accept,
                explanation: None,
            };
            add("peace_response", 500, 2, orders);
        } else if matches!(order, Order::DeclareWar { .. }) {
            let mut orders = planned.to_vec();
            orders.remove(i);
            add("avoid_war", 500, 2, orders);
        }
    }
    for action in view
        .diplomacy
        .actions
        .iter()
        .filter(|a| {
            matches!(
                &a.order,
                Order::DeclareWar { .. }
                    | Order::ProposeTreaty {
                        kind: legatum_core::diplomacy::TreatyKind::Peace,
                        ..
                    }
            )
        })
        .take(4)
    {
        if planned.contains(&action.order) {
            continue;
        }
        let mut orders = planned.to_vec();
        orders.push(action.order.clone());
        add("war_peace_choice", 400, 2, orders);
    }
    out
}

#[derive(Clone)]
struct Score {
    turn: u32,
    vp: u32,
    cities: usize,
    wc: i32,
    lead: i64,
    finished: bool,
    won: bool,
}

struct BranchTapes<'a> {
    seat: &'a BTreeMap<u32, Vec<Order>>,
    target: &'a BTreeMap<u32, Vec<Order>>,
}

struct BranchResult {
    scores: Vec<Score>,
    target_scores: Vec<Score>,
    fidelity: Option<(u32, u32)>,
    invalid_orders: usize,
    target_invalid_orders: usize,
}

fn score(state: &GameState, seat: PlayerId) -> Score {
    let team = state.team_of(seat);
    let vp = scoring::vp_total(state, seat);
    let best_other = state
        .players
        .iter()
        .filter(|p| p.alive && p.team != team)
        .map(|p| scoring::vp_total(state, p.id))
        .max()
        .unwrap_or(0);
    let won = state.finished && state.winner == Some(team);
    let wc = if state.finished {
        if won { 10_000 } else { 0 }
    } else if state.player(seat).alive {
        WinFacts::of(&player_view(state, seat)).chance()
    } else {
        0
    };
    Score {
        turn: state.turn,
        vp,
        cities: state.cities_of(seat).count(),
        wc,
        lead: i64::from(vp) - i64::from(best_other),
        finished: state.finished,
        won,
    }
}

/// Play one arm from the snapshot: `seat_orders` on the branch turn, then BRAIN.
fn branch(
    snap: &Snapshot,
    args: &Args,
    seat_orders: &[Order],
    swap: &[usize],
    tapes: &BranchTapes<'_>,
    salt: u64,
    continuation: &Continuation,
) -> BranchResult {
    let mut state = snap.state.clone();
    let seat = args.seat;
    let start = state.turn;
    let game_seed = state.setup.seed;
    if args.reseed {
        reseed_continuation_dice(&mut state, salt);
    }
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| {
            let params = if p.id == seat {
                continuation.params.as_ref()
            } else {
                None
            };
            let mut b = hard_brain(&state, game_seed, p.id, params, salt);
            if let Some(m) = snap.memory.get(&p.id)
                && !(p.id == seat && continuation.memory_reset)
            {
                b.restore_opponent_memory(m.clone());
            }
            (p.id, b)
        })
        .collect();
    let mut fidelity = None;
    let mut invalid_orders = 0;
    let mut target_invalid_orders = 0;
    let mut out = Vec::new();
    let mut target_out = Vec::new();
    let hmax = args.end_turn.map_or_else(
        || *args.horizons.last().unwrap(),
        |end| end.saturating_sub(start),
    );
    assert!(hmax > 0, "branch must precede end turn");
    while state.turn < start + hmax && !state.finished {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if !state.player(*id).alive {
                continue;
            }
            let planned = b.orders(&player_view(&state, *id));
            orders.insert(*id, planned);
        }
        if state.turn == start {
            if salt == 0 {
                let (mut ok, mut n) = (0, 0);
                for (id, os) in orders.iter().filter(|(id, _)| **id != seat) {
                    let mine: Vec<_> = os.iter().filter(|o| strategic(o)).map(short).collect();
                    let taped: Vec<_> = snap
                        .tape_orders
                        .get(id)
                        .map(|v| v.iter().filter(|o| strategic(o)).map(short).collect())
                        .unwrap_or_default();
                    n += 1;
                    ok += u32::from(mine == taped);
                }
                fidelity = Some((ok, n));
            }
            // The seat's brain planned the branch turn too (discarded), so its memory moves on
            // alike in every arm.
            orders.insert(seat, seat_orders.to_vec());
        } else if state.turn < start + args.window {
            // Window: the tape's orders, with the swapped categories from the live brain.
            let live = orders.remove(&seat).unwrap_or_default();
            let taped = tapes.seat.get(&state.turn).cloned().unwrap_or_default();
            orders.insert(seat, hybrid(&state, seat, &taped, &live, swap));
        }
        if args.rival {
            // Hold the designer's recorded policy fixed in both arms. Otherwise an early rival
            // choice is scored against Hard's weaker play in the designer seat, not the game
            // whose opponent weakness we are trying to measure.
            let recorded_orders = tapes.target.get(&state.turn).cloned().unwrap_or_default();
            target_invalid_orders +=
                validate_orders(&state, args.target_seat, &recorded_orders).len();
            orders.insert(args.target_seat, recorded_orders);
        }
        // Count legality only when the continuation actually plays. Recorded window orders
        // can become invalid after reseeding, and are not the continuation's orders.
        if state.turn >= start + args.window
            && let Some(seat_orders) = orders.get(&seat)
        {
            invalid_orders += validate_orders(&state, seat, seat_orders).len();
        }
        state = resolve_turn(&state, &orders).0;
        if state.turn == start + args.window {
            state.players[seat.0 as usize].is_human = false;
            if let Some(p) = state.setup.players.get_mut(seat.0 as usize) {
                p.is_human = false;
                p.difficulty = AiDifficulty::Hard;
            }
            if args.clear_at_handover {
                for u in state.units.values_mut().filter(|u| u.owner == seat) {
                    u.goto = None;
                    u.path.clear();
                    u.path_stop_on_block = false;
                    u.automation = None;
                    u.standing_order = None;
                }
            }
        }
        let played = state.turn - start;
        if args.end_turn.is_none() && args.horizons.contains(&played) {
            out.push(score(&state, seat));
            target_out.push(score(&state, args.target_seat));
        }
    }
    let last = score(&state, seat);
    let target_last = score(&state, args.target_seat);
    let wanted = if args.end_turn.is_some() {
        1
    } else {
        args.horizons.len()
    };
    if args.end_turn.is_some() {
        out = vec![last.clone()];
        target_out = vec![target_last.clone()];
    }
    while out.len() < wanted {
        out.push(last.clone());
        target_out.push(target_last.clone());
    }
    BranchResult {
        scores: out,
        target_scores: target_out,
        fidelity,
        invalid_orders,
        target_invalid_orders,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::objectives::era_objectives;

    #[test]
    fn reseeded_continuation_keeps_recorded_objectives_in_every_era() {
        let tape = Tape::from_json(include_str!(
            "../../wasm/tests/fixtures/playtest3-g2.tape.json"
        ))
        .expect("recorded tape");
        let recorded = tape.initial_state();
        let mut continuation = recorded.clone();
        reseed_continuation_dice(&mut continuation, 17);
        assert_eq!(continuation.setup, recorded.setup);
        assert_ne!(continuation.dice_seed(), recorded.dice_seed());
        for era in 2..=legatum_core::data::era_count() {
            assert_eq!(
                era_objectives(&continuation.setup, era),
                era_objectives(&recorded.setup, era),
                "era {era}"
            );
        }
        let mut unsalted = recorded.clone();
        reseed_continuation_dice(&mut unsalted, 0);
        assert_eq!(unsalted, recorded);
        assert_eq!(unsalted.hash(), recorded.hash());
    }
}

fn main() {
    let args = parse_args();
    std::fs::create_dir_all(&args.out).unwrap();
    let tape =
        Tape::from_json(&std::fs::read_to_string(&args.tape).expect("read tape")).expect("tape");
    let initial_state = args.initial_state.as_ref().map_or_else(
        || tape.initial_state(),
        |path| {
            let state = legatum_wasm::json::parse_state(
                &std::fs::read_to_string(path).expect("read recorded initial state"),
            )
            .expect("recorded initial state json");
            assert_eq!(state.turn, tape.turns[0].turn, "recorded initial turn");
            assert_eq!(state.hash(), tape.initial_hash, "recorded initial hash");
            state
        },
    );
    // Original-engine positions keep historical decisions meaningful after a rules change.
    // Verify every supplied state against the tape before allowing any branch computation.
    let mut migrated_states = 0usize;
    let recorded: Option<BTreeMap<u32, GameState>> = args.states.as_ref().map(|path| {
        use std::io::BufRead;
        let reader =
            std::io::BufReader::new(std::fs::File::open(path).expect("read recorded states"));
        let rows: Vec<(u64, GameState)> = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                // Check the historical canonical bytes before current serde aliases migrate
                // names (e.g. Japan -> India). Never substitute a current replay's state.
                let hash = legatum_core::state::fnv1a(line.as_bytes());
                let state: GameState = serde_json::from_str(&line).expect("recorded state json");
                migrated_states += usize::from(state.hash() != hash);
                (hash, state)
            })
            .collect();
        assert_eq!(
            rows.len(),
            tape.turns.len() + 1,
            "one recorded state per turn plus final state"
        );
        assert_eq!(rows[0].0, tape.initial_hash, "recorded initial hash");
        for (turn, (hash, state)) in tape.turns.iter().zip(rows.iter().skip(1)) {
            assert_eq!(state.turn, turn.turn + 1, "recorded turn sequence");
            assert_eq!(
                hash.to_string(),
                turn.hash,
                "recorded hash at T{}",
                turn.turn
            );
        }
        rows.into_iter()
            .map(|(_, state)| (state.turn, state))
            .collect()
    });
    let seat = args.seat;
    let mut state = recorded.as_ref().map_or_else(
        || initial_state.clone(),
        |rows| rows[&tape.turns[0].turn].clone(),
    );
    let game_seed = state.setup.seed;
    let alt_params = match &args.alt {
        Source::Hard(p) => p.as_deref().cloned(),
        Source::Orders(_) => None,
    };
    // Shadow brains: every seat Hard; the audited seat plays SOURCE when SOURCE is a brain.
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .map(|p| {
            let params = if p.id == seat {
                alt_params.as_ref()
            } else {
                None
            };
            (p.id, hard_brain(&state, game_seed, p.id, params, 0))
        })
        .collect();
    let restore: Option<(u32, BTreeMap<PlayerId, OpponentMemory>)> =
        args.memory.as_ref().map(|(t, f)| {
            (
                *t,
                serde_json::from_str(&std::fs::read_to_string(f).expect("read memory"))
                    .expect("memory json"),
            )
        });
    let sample = |t: u32| -> bool {
        t >= args.from
            && t <= args.to
            && match &args.turns {
                Some(ts) => ts.contains(&t),
                None => {
                    args.top.is_some()
                        || args.event_windows.is_some()
                        || t.is_multiple_of(args.every)
                }
            }
    };
    let mut diff_out = std::fs::File::create(args.out.join("diff.jsonl")).unwrap();
    let mut army_out = args
        .army_detail
        .then(|| std::fs::File::create(args.out.join("army.jsonl")).unwrap());
    let mut settle_out = args
        .settle_detail
        .then(|| std::fs::File::create(args.out.join("settle.jsonl")).unwrap());
    let mut prod_out = args
        .prod_detail
        .then(|| std::fs::File::create(args.out.join("prod.jsonl")).unwrap());
    let mut peace_out = args
        .peace_detail
        .then(|| std::fs::File::create(args.out.join("peace.jsonl")).unwrap());
    let mut war_out = args
        .war_detail
        .then(|| std::fs::File::create(args.out.join("war.jsonl")).unwrap());
    // `--settle-detail`: where (and when) the tape founds with each of the seat's Settlers, from
    // a plain replay, so every earlier turn can ask how the seat's Hard rates that tile.
    let founds: BTreeMap<UnitId, (u32, legatum_core::Hex)> = if args.settle_detail {
        let mut st = initial_state.clone();
        let mut m = BTreeMap::new();
        for t in &tape.turns {
            if let Some(rows) = &recorded {
                st = rows[&t.turn].clone();
            }
            for o in t.orders.get(&seat).into_iter().flatten() {
                if let Order::FoundCity { unit } = o
                    && let Some(u) = st.units.get(unit)
                {
                    m.entry(*unit).or_insert((t.turn, u.pos));
                }
            }
            st = resolve_turn(&st, &t.orders).0;
        }
        m
    } else {
        BTreeMap::new()
    };
    let mut snaps: Vec<Snapshot> = Vec::new();
    let mut rival_meta: Vec<(usize, &'static str, i32, usize)> = Vec::new();
    let mut logged_windows = Vec::<EventWindow>::new();
    let mut last_siege = BTreeMap::<legatum_core::CityId, u32>::new();
    let mut rival_settler_losses = Vec::<u32>::new();
    let (mut hash_bad, mut ai_checked, mut ai_bad) = (0u32, 0u32, 0u32);
    let mut first_ai_bad = None;
    for t in &tape.turns {
        if let Some(rows) = &recorded {
            state = rows[&t.turn].clone();
        }
        if args.fixture_turns.contains(&t.turn) {
            std::fs::write(
                args.out.join(format!("state-T{}.json", t.turn)),
                serde_json::to_string(&state).unwrap(),
            )
            .unwrap();
        }
        if let Some((rt, mems)) = &restore
            && *rt == t.turn
        {
            for (id, m) in mems {
                brains
                    .get_mut(id)
                    .unwrap()
                    .restore_opponent_memory(m.clone());
            }
            eprintln!("restored AI memory at T{rt}");
        }
        let memory: BTreeMap<PlayerId, OpponentMemory> = brains
            .iter()
            .filter_map(|(id, b)| b.opponent_memory().map(|m| (*id, m)))
            .collect();
        if args.fixture_turns.contains(&t.turn) {
            std::fs::write(
                args.out.join(format!("memory-T{}.json", t.turn)),
                serde_json::to_string(&memory).unwrap(),
            )
            .unwrap();
        }
        let alive: Vec<PlayerId> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect();
        let mut alt_orders = Vec::new();
        let mut alt_trace = None;
        if let Some(f) = war_out.as_mut().filter(|_| state.player(seat).alive) {
            let notes = brains[&seat].war_diagnosis(&player_view(&state, seat));
            writeln!(f, "{}", json!({"turn": t.turn, "notes": notes})).unwrap();
        }
        // `--settle-detail`: for every Settler of the seat, how the seat's Hard plans it this
        // turn and how its site search rates the tile the tape later founds on with it.
        let mut settle_notes = Vec::new();
        if args.settle_detail && state.player(seat).alive {
            let view = player_view(&state, seat);
            for u in state
                .units
                .values()
                .filter(|u| u.owner == seat && u.kind == legatum_core::UnitKind::Settler)
            {
                let his = founds.get(&u.id).copied();
                let notes = brains[&seat].settler_diagnosis(&view, u.id, his.map(|(_, h)| h));
                settle_notes.push((u.id, u.pos, his, notes));
            }
        }
        // `--peace-detail`: every Peace offer the seat must answer, and why its Hard would answer so.
        let mut peace_notes = Vec::new();
        if args.peace_detail && state.player(seat).alive {
            let view = player_view(&state, seat);
            for p in view.diplomacy.proposals.iter().filter(|p| {
                p.needs.contains(&view.team) && p.kind == legatum_core::diplomacy::TreatyKind::Peace
            }) {
                let other = view.diplomacy.counterparty(p, view.team);
                let diag = brains[&seat].peace_diagnosis(&view, other);
                peace_notes.push((p.id, other, format!("{diag:?}")));
            }
        }
        for id in alive {
            let planned = if id == seat && (args.army_detail || args.prod_detail || args.rival) {
                let (o, tr) = brains
                    .get_mut(&id)
                    .unwrap()
                    .order_diagnosis(&player_view(&state, id));
                alt_trace = Some(tr);
                o
            } else {
                brains
                    .get_mut(&id)
                    .unwrap()
                    .orders(&player_view(&state, id))
            };
            if id == seat {
                alt_orders = planned;
            } else if !state.player(id).is_human {
                ai_checked += 1;
                let mine: Vec<_> = planned.iter().filter(|o| strategic(o)).map(short).collect();
                let taped: Vec<_> = t
                    .orders
                    .get(&id)
                    .map(|v| v.iter().filter(|o| strategic(o)).map(short).collect())
                    .unwrap_or_default();
                if mine != taped {
                    ai_bad += 1;
                    first_ai_bad.get_or_insert((t.turn, id.0));
                }
            }
        }
        if let Source::Orders(m) = &args.alt {
            alt_orders = m.get(&t.turn).cloned().unwrap_or_default();
        }
        if let Some(f) = peace_out.as_mut() {
            let answer = |orders: &[Order], id| {
                orders.iter().find_map(|o| match o {
                    Order::RespondToProposal {
                        proposal, accept, ..
                    } if *proposal == id => Some(*accept),
                    _ => None,
                })
            };
            for (id, other, diag) in &peace_notes {
                let his = answer(t.orders.get(&seat).map_or(&[][..], |v| &v[..]), *id);
                writeln!(
                    f,
                    "{}",
                    json!({"turn": t.turn, "proposal": id.0, "with": other.0,
                        "cities": state.cities_of(seat).count(),
                        "his": his, "hard": answer(&alt_orders, *id), "diagnosis": diag})
                )
                .unwrap();
            }
        }
        if let Some(f) = settle_out.as_mut() {
            let own: Vec<legatum_core::Hex> = state
                .cities
                .values()
                .filter(|c| c.owner == seat)
                .map(|c| c.pos)
                .collect();
            let near = |h: legatum_core::Hex| own.iter().map(|c| c.distance(h)).min();
            for (unit, pos, his, notes) in &settle_notes {
                let of_unit = |orders: &[Order]| -> Vec<String> {
                    orders
                        .iter()
                        .filter(|o| order_unit(o) == Some(*unit))
                        .map(short)
                        .collect()
                };
                let his_orders = of_unit(t.orders.get(&seat).map_or(&[][..], |v| &v[..]));
                writeln!(
                    f,
                    "{}",
                    json!({"turn": t.turn, "unit": unit.0, "pos": [pos.q, pos.r],
                        "at_war": state.players.iter().any(|o| o.alive && o.id != seat && state.at_war(seat, o.id)),
                        "cities": own.len(), "pos_city_d": near(*pos),
                        "his_site": his.map(|(_, h)| [h.q, h.r]), "his_found_turn": his.map(|(t, _)| t),
                        "his_site_city_d": his.and_then(|(_, h)| near(h)),
                        "his_site_d": his.map(|(_, h)| h.distance(*pos)),
                        "founds_now": his.is_some_and(|(ft, _)| ft == t.turn),
                        "his": his_orders, "hard": of_unit(&alt_orders), "notes": notes})
                )
                .unwrap();
            }
        }
        let in_range = t.turn >= args.from && t.turn <= args.to;
        let rec_orders = t.orders.get(&seat).cloned().unwrap_or_default();
        let (next, ev_rec) = resolve_turn(&state, &t.orders);
        if args.rival
            && ev_rec
                .iter()
                .any(|e| matches!(e, Event::SettlerCaptured { from, .. } if *from == seat))
        {
            rival_settler_losses.push(t.turn);
        }
        if let Some(k) = args.event_windows {
            for event in &ev_rec {
                if let Event::Battle {
                    attacker_owner,
                    defender: legatum_core::events::Defender::City { city, .. },
                    ..
                } = event
                    && *attacker_owner == seat
                {
                    let prior = last_siege.insert(*city, t.turn);
                    if prior.is_some_and(|p| t.turn.saturating_sub(p) <= 2) {
                        continue;
                    }
                }
                logged_windows.extend(event_windows(std::slice::from_ref(event), seat, t.turn, k));
            }
        }
        if in_range && state.player(seat).alive {
            let mut swapped = t.orders.clone();
            swapped.insert(seat, alt_orders.clone());
            let (alt_state, ev_alt) = resolve_turn(&state, &swapped);
            let cats = diff_turn(
                &state,
                seat,
                &rec_orders,
                &alt_orders,
                &next,
                &ev_rec,
                &alt_state,
                &ev_alt,
            );
            if let (Some(f), Some(tr)) = (prod_out.as_mut(), alt_trace.as_ref()) {
                for row in prod_rows(&state, seat, t.turn, &next, &alt_state, tr) {
                    writeln!(f, "{row}").unwrap();
                }
            }
            if let Some(f) = army_out.as_mut() {
                for row in army_rows(
                    &state,
                    seat,
                    &rec_orders,
                    &alt_orders,
                    &next,
                    &ev_rec,
                    &alt_state,
                    &ev_alt,
                    alt_trace.as_ref(),
                ) {
                    writeln!(f, "{row}").unwrap();
                }
            }
            let differs: Vec<usize> = CATS
                .iter()
                .enumerate()
                .filter(|(_, n)| {
                    cats[**n]["n"].as_u64().unwrap_or(0) > 0
                        || cats[**n]["order_diff"].as_u64().unwrap_or(0) > 0
                })
                .map(|(c, _)| c)
                .collect();
            let p = state.player(seat);
            writeln!(
                diff_out,
                "{}",
                json!({"turn": t.turn, "gold": p.gold, "cities": state.cities_of(seat).count(),
                    "vp": scoring::vp_total(&state, seat),
                    "military": state.units.values().filter(|u| u.owner == seat && is_military(u.kind)).count(),
                    "at_war_with": state.players.iter().filter(|o| o.alive && o.id != seat && state.at_war(seat, o.id)).map(|o| o.id.0).collect::<Vec<_>>(),
                    "rec_orders": rec_orders.len(), "alt_orders": alt_orders.len(),
                    "alt_present": !matches!(&args.alt, Source::Orders(m) if !m.contains_key(&t.turn)),
                    "cats": cats})
            )
            .unwrap();
            let have_alt = !matches!(&args.alt, Source::Orders(m) if !m.contains_key(&t.turn));
            if sample(t.turn)
                && have_alt
                && !args.diff_only
                && (args.rival
                    || args.top.is_none()
                    || (!differs.is_empty() && args.end_turn.is_none_or(|end| t.turn < end)))
            {
                if args.rival {
                    let view = player_view(&state, seat);
                    for (kind, priority, cat, orders) in rival_options(
                        &state,
                        seat,
                        &view,
                        &rec_orders,
                        alt_trace.as_ref().expect("rival trace"),
                    ) {
                        rival_meta.push((snaps.len(), kind, priority, cat));
                        snaps.push(Snapshot {
                            state: state.clone(),
                            memory: memory.clone(),
                            rec: rec_orders.clone(),
                            alt: orders,
                            tape_orders: t.orders.clone(),
                            differs: vec![cat],
                            cats: cats.clone(),
                        });
                    }
                } else {
                    snaps.push(Snapshot {
                        state: state.clone(),
                        memory,
                        rec: rec_orders,
                        alt: alt_orders,
                        tape_orders: t.orders.clone(),
                        differs,
                        cats: cats.clone(),
                    });
                }
            }
        }
        state = next;
        if state.hash().to_string() != t.hash {
            hash_bad += 1;
        }
        if t.turn.is_multiple_of(20) {
            eprintln!(
                "diff T{} (hash mismatches {hash_bad}, AI order mismatches {ai_bad}/{ai_checked})",
                t.turn
            );
        }
    }
    let fidelity = json!({"tape_turns": tape.turns.len(), "hash_mismatches": hash_bad,
        "position_source": if recorded.is_some() { "verified recorded states" } else { "current engine replay" },
        "recorded_state_hash_mismatches": recorded.as_ref().map(|_| 0),
        "recorded_states_migrated": migrated_states,
        "ai_seat_turns": ai_checked, "ai_strategic_order_mismatches": ai_bad,
        "first_ai_mismatch": first_ai_bad, "alt": args.alt_label, "seat": seat.0,
        "salts": args.salts, "horizons": args.horizons, "end_turn": args.end_turn,
        "top": args.top, "event_windows": args.event_windows, "rival": args.rival,
        "target_seat": args.target_seat.0, "reseed": args.reseed, "window": args.window,
        "continuations": args.continuations.iter().map(|c| &c.name).collect::<Vec<_>>(),
        "handover": if args.clear_at_handover { "clear" } else { "keep" }});
    eprintln!("replay: {fidelity}");
    if args.diff_only || hash_bad != 0 {
        std::fs::write(args.out.join("fidelity.json"), fidelity.to_string()).unwrap();
        return;
    }
    // Value: (snapshot, arm, salt) jobs.
    assert!(
        args.window == 1 || matches!(args.alt, Source::Hard(_)),
        "--window above 1 needs a brain as the alternative"
    );
    let seat_tape: BTreeMap<u32, Vec<Order>> = tape
        .turns
        .iter()
        .map(|t| (t.turn, t.orders.get(&seat).cloned().unwrap_or_default()))
        .collect();
    let target_tape: BTreeMap<u32, Vec<Order>> = tape
        .turns
        .iter()
        .map(|t| {
            (
                t.turn,
                t.orders.get(&args.target_seat).cloned().unwrap_or_default(),
            )
        })
        .collect();
    let all: Vec<usize> = (0..CATS.len()).collect();
    let mut jobs: Vec<Job> = Vec::new();
    let mut selected: Vec<(usize, usize, i32)> = Vec::new();
    if args.rival {
        let limit = args.top.unwrap();
        rival_meta.sort_by_key(|(i, kind, score, _)| {
            let turn = snaps[*i].state.turn;
            let near_loss = (*kind == "settler_safety" || *kind == "settler_escort")
                && rival_settler_losses
                    .iter()
                    .any(|loss| turn <= *loss && loss.saturating_sub(turn) <= 5);
            let priority = (*score / 20).clamp(-100, 100)
                + if *kind == "forecast_attack" { 30 } else { 0 }
                + if near_loss {
                    200
                } else if *kind == "settler_safety" || *kind == "settler_escort" {
                    -1000
                } else {
                    0
                }
                + if turn <= 65 { 15 } else { 0 };
            (std::cmp::Reverse(priority), turn)
        });
        let mut kinds = BTreeMap::<&str, usize>::new();
        let mut turns = BTreeMap::<u32, usize>::new();
        let mut candidates = Vec::new();
        for (i, kind, priority, cat) in rival_meta {
            let s = &snaps[i];
            let turn = s.state.turn;
            if kinds.get(kind).copied().unwrap_or(0) >= 2
                || turns.get(&turn).copied().unwrap_or(0) >= 2
            {
                continue;
            }
            *kinds.entry(kind).or_default() += 1;
            *turns.entry(turn).or_default() += 1;
            let arm = format!("alt:{i}");
            candidates.push(json!({"turn": turn, "category": CATS[cat], "kind": kind,
                "priority": priority, "arm": arm,
                "alternative": s.alt.iter().map(short).collect::<Vec<_>>() }));
            for salt in 0..args.salts {
                jobs.push((i, format!("rec:{i}"), s.rec.clone(), vec![], salt, 0));
                jobs.push((i, arm.clone(), s.alt.clone(), vec![], salt, 0));
            }
            if candidates.len() == limit {
                break;
            }
        }
        std::fs::write(
            args.out.join("candidates.json"),
            serde_json::to_string_pretty(&candidates).unwrap(),
        )
        .unwrap();
    } else if let Some(limit) = args.top {
        let weights = [7, 2, 9, 7, 7, 5, 3, 10];
        let mut ranked: Vec<_> = snaps
            .iter()
            .enumerate()
            .flat_map(|(i, s)| {
                s.differs.iter().filter_map(move |&c| {
                    let turn = s.state.turn;
                    let e = &s.cats[CATS[c]];
                    if e["n"].as_u64().unwrap_or(0) == 0
                        || e["order_diff"].as_u64().unwrap_or(0) == 0
                    {
                        return None;
                    }
                    let early = if turn <= 65 {
                        12
                    } else if turn <= 120 {
                        6
                    } else {
                        0
                    };
                    let war =
                        if s.state.players.iter().any(|p| {
                            p.alive && p.id != args.seat && s.state.at_war(args.seat, p.id)
                        }) {
                            3
                        } else {
                            0
                        };
                    let mut score = weights[c] + early + war - if turn > 145 { 5 } else { 0 };
                    let count = |key: &str| e[key].as_array().map_or(0, |v| v.len().min(2) as i32);
                    let number = |key: &str| e[key].as_u64().unwrap_or(0);
                    match c {
                        0 => {
                            score += 3 * count("producing_differs");
                            if e["rec"].to_string().contains("Settler") {
                                score += 4;
                            }
                        }
                        7 => {
                            score += 13 * i32::from(e["rec_bought"] != e["alt_bought"]);
                            if e["rec_bought"].to_string().contains("Bomber") {
                                score += 10;
                            }
                        }
                        2 => score += 10,
                        3 => {
                            score += 13
                                * i32::from(number("rec_founded").max(number("alt_founded")) > 0)
                                + 2 * count("units")
                        }
                        4 => {
                            score += 10
                                * i32::from(number("rec_attacks").max(number("alt_attacks")) > 0)
                                + 2 * count("units")
                        }
                        5 => score += 2 * count("units"),
                        _ => {}
                    }
                    if c == 4 || c == 5 {
                        score += 24
                            * i32::from(
                                number("rec_settlers_captured") != number("alt_settlers_captured"),
                            );
                    }
                    Some((i, c, score))
                })
            })
            .collect();
        ranked.sort_by_key(|(i, c, score)| (std::cmp::Reverse(*score), snaps[*i].state.turn, *c));
        let mut per_cat = [0usize; 8];
        let mut per_turn = BTreeMap::<u32, usize>::new();
        let mut per_phase = [0usize; 3];
        let mut per_cat_phase = [[0usize; 3]; 8];
        let cat_cap = [6, 2, 3, 6, 6, 4, 2, 8];
        let phase_cap = [14, 10, 6];
        for (i, c, score) in ranked {
            let turn = snaps[i].state.turn;
            let phase = if turn <= 65 {
                0
            } else if turn <= 120 {
                1
            } else {
                2
            };
            if per_cat[c] >= cat_cap[c]
                || per_turn.get(&turn).copied().unwrap_or(0) >= 2
                || per_phase[phase] >= phase_cap[phase]
                || per_cat_phase[c][phase] >= 4
            {
                continue;
            }
            selected.push((i, c, score));
            per_cat[c] += 1;
            per_phase[phase] += 1;
            per_cat_phase[c][phase] += 1;
            *per_turn.entry(turn).or_default() += 1;
            if selected.len() == limit {
                break;
            }
        }
        std::fs::write(args.out.join("candidates.json"), serde_json::to_string_pretty(&selected.iter().map(|(i,c,score)| json!({"turn": snaps[*i].state.turn, "category": CATS[*c], "priority": score})).collect::<Vec<_>>()).unwrap()).unwrap();
    }
    if args.rival {
        // Rival jobs and their paired baselines were built from own-view options above.
    } else if args.event_windows.is_some() {
        let mut seen = BTreeSet::new();
        let mut windows = Vec::new();
        logged_windows.sort_by_key(|event| {
            (
                if event.kind.contains("settler") {
                    0
                } else if event.kind == "city_capture" {
                    1
                } else {
                    2
                },
                event.event_turn,
            )
        });
        for event in logged_windows {
            let Some((i, s)) = snaps
                .iter()
                .enumerate()
                .find(|(_, s)| s.state.turn == event.start)
            else {
                continue;
            };
            if !seen.insert((event.start, event.kind)) {
                continue;
            }
            let category = event
                .cats
                .iter()
                .map(|c| CATS[*c])
                .collect::<Vec<_>>()
                .join("+");
            let label = format!("alt:{}:T{}", event.kind, event.event_turn);
            let orders = hybrid(&s.state, seat, &s.rec, &s.alt, &event.cats);
            windows.push(json!({"turn": event.start, "event_turn": event.event_turn,
                "event": event.kind, "category": category, "arm": label}));
            for salt in 0..args.salts {
                jobs.push((
                    i,
                    format!("rec:{}:T{}", event.kind, event.event_turn),
                    s.rec.clone(),
                    vec![],
                    salt,
                    0,
                ));
                jobs.push((
                    i,
                    label.clone(),
                    orders.clone(),
                    event.cats.clone(),
                    salt,
                    0,
                ));
            }
            if windows.len() >= 12 {
                break;
            }
        }
        std::fs::write(
            args.out.join("candidates.json"),
            serde_json::to_string_pretty(&windows).unwrap(),
        )
        .unwrap();
    } else {
        for (i, s) in snaps.iter().enumerate() {
            if args.top.is_some() {
                for (_, c, _) in selected.iter().filter(|(j, _, _)| *j == i) {
                    let orders = hybrid(&s.state, args.seat, &s.rec, &s.alt, &[*c]);
                    for (label, arm) in [
                        ("rec".to_string(), s.rec.clone()),
                        (format!("alt:{}", CATS[*c]), orders),
                    ] {
                        for salt in 0..args.salts {
                            jobs.push((i, label.clone(), arm.clone(), vec![*c], salt, 0));
                        }
                    }
                }
                continue;
            }
            let mut arms: Vec<(String, Vec<Order>, Vec<usize>)> = vec![
                ("rec".into(), s.rec.clone(), vec![]),
                ("alt".into(), s.alt.clone(), all.clone()),
            ];
            // With a window every category gets an arm: a policy can differ on later turns.
            let cats: Vec<usize> = if args.window > 1 {
                all.clone()
            } else {
                s.differs.clone()
            };
            if args.hybrids
                && (cats.len() > 1
                    || args.window > 1
                    || (cats.len() == 1 && args.continuations.len() > 1))
            {
                for &c in &cats {
                    arms.push((
                        format!("alt:{}", CATS[c]),
                        hybrid(&s.state, seat, &s.rec, &s.alt, &[c]),
                        vec![c],
                    ));
                    // The reverse swap: SOURCE everywhere except this category, which stays his.
                    let rest: Vec<usize> = all.iter().copied().filter(|x| *x != c).collect();
                    arms.push((
                        format!("rec:{}", CATS[c]),
                        hybrid(&s.state, seat, &s.rec, &s.alt, &rest),
                        rest,
                    ));
                }
            }
            for (label, orders, swap) in arms {
                for salt in 0..args.salts {
                    for continuation in 0..args.continuations.len() {
                        jobs.push((
                            i,
                            label.clone(),
                            orders.clone(),
                            swap.clone(),
                            salt,
                            continuation,
                        ));
                    }
                }
            }
        }
    }
    eprintln!("value: {} snapshots, {} runs", snaps.len(), jobs.len());
    let done = std::sync::atomic::AtomicUsize::new(0);
    let tapes = BranchTapes {
        seat: &seat_tape,
        target: &target_tape,
    };
    let rows: Vec<Value> = jobs
        .par_iter()
        .map(|(i, label, orders, swap, salt, continuation)| {
            let snap = &snaps[*i];
            let cont = &args.continuations[*continuation];
            let BranchResult { scores, target_scores, fidelity: fid, invalid_orders, target_invalid_orders } =
                branch(snap, &args, orders, swap, &tapes, *salt, cont);
            let n = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            if n.is_multiple_of(500) {
                eprintln!("value: {n}/{} runs", jobs.len());
            }
            let horizons = args.end_turn.map_or_else(|| args.horizons.clone(), |end| vec![end]);
            json!({"turn": snap.state.turn, "arm": label, "salt": salt,
                "continuation": cont.name,
                "continuation_invalid_orders": invalid_orders,
                "target_recorded_invalid_orders": target_invalid_orders,
                "differs": snap.differs.iter().map(|c| CATS[*c]).collect::<Vec<_>>(),
                "ai_fidelity": fid,
                "target_h": horizons.iter().zip(&target_scores).map(|(h, s)| json!({"h": h, "turn": s.turn, "vp": s.vp,
                    "cities": s.cities, "wc": s.wc, "lead": s.lead, "finished": s.finished, "won": s.won})).collect::<Vec<_>>(),
                "h": horizons.iter().zip(&scores).map(|(h, s)| json!({"h": h, "turn": s.turn, "vp": s.vp,
                    "cities": s.cities, "wc": s.wc, "lead": s.lead, "finished": s.finished, "won": s.won})).collect::<Vec<_>>()})
        })
        .collect();
    let mut f = std::fs::File::create(args.out.join("value.jsonl")).unwrap();
    for r in rows {
        writeln!(f, "{r}").unwrap();
    }
    std::fs::write(args.out.join("fidelity.json"), fidelity.to_string()).unwrap();
    eprintln!("decision_audit: wrote {}", args.out.display());
}

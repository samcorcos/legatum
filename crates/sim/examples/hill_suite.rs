//! Hill-climb suite cell: one recorded position of the designer's game, played on with this
//! engine's Hard in every AI seat, many paired runs, one JSON line per run.
//!
//!   hill_suite --tape TAPE.json --states STATES.jsonl --out RUNS.jsonl --branch 51 --end 81
//!              --mode tape|hybrid|hard|storm|quiet [--seat 0] [--salts 48] [--victim 3]
//!              [--war-turn 61] [--storm 3] [--report 60] [--arms control,NAME=PARAMS.json]
//!              [--threads N] [--hybrid-defend on|off] [--accept-rules 49|50|51|52]
//!
//! `--states` is `audit_states` output (every recorded hash checked). `--accept-rules 49` reads states
//! written by a rules-49 engine through `legatum_wasm::json::upgrade_value_from_49`, and
//! `--accept-rules 50`, `51` or `52` reads rules-50 to rules-52 states through `upgrade_value_from_50`
//! (the identity: rules 51 to 53 change no field; 52's renamed identifiers load through aliases). Sim only; see them for what changes. Without the flag a state carrying a
//! rules-49-only field is refused. Every seat's Hard brain
//! shadows the recorded game up to `--branch` so it has its real opponent memory. From the branch
//! the designer's seat is played by:
//! - `tape`: his recorded orders for each turn (orders that no longer apply fail; each failure the
//!   engine reports for his seat is counted as a misfire);
//! - `hybrid`: a stand-in for him once the game drifts. His recorded orders, translated to the
//!   live game and checked with the core's validator before resolving. Unit, city and proposal
//!   ids are global counters and drift as soon as any seat plays differently, so recorded ids are
//!   mapped to live ones every turn from the recorded states: cities by tile, units by kind and
//!   nearest position (then any armed unit for an armed one), kept once paired; proposals by
//!   sender and kind. Orders the validator refused in the recording too are dropped (he lost them
//!   for real). A broken route keeps its destination, a production queue keeps the items that
//!   still apply, research puts the techs he knew by then first, a FoundCity becomes a plan to
//!   found on his tile. Other refused orders hand their unit or city to shipped Hard for the turn
//!   (warm memory, his own view; never an upgrade, disband or founding of a unit he still
//!   commands), as do live units and cities with no recorded counterpart. Paired units with no
//!   order are steered to where his recorded unit was heading. With `--hybrid-defend on` (the
//!   default) a war the recording never had is defended: threatened cities and his home units
//!   near them play Hard. His declarations and treaties stand whenever valid. Counted per run in
//!   `hybrid`: kept, dropped (missing counterpart, refused now, refused then), rerouted,
//!   salvaged, research steered, Hard orders, steered, unit-turns / city-turns handed to Hard and
//!   defence turns;
//! - `hard`: shipped Hard (warm memory);
//! - `storm`: shipped Hard with the scripted rusher on top (declares on `--victim` from
//!   `--war-turn`, every unit marches on the victim's nearest city, attacks with `--storm` units
//!   within two tiles);
//! - `quiet`: shipped Hard with every war declaration removed (the neutral control: nobody in his
//!   seat attacks).
//!
//! Every other seat is Hard with the arm's params (`control` = shipped Hard). Salt 0 keeps the
//! recorded brain seeds and dice; salt s > 0 salts every AI brain seed and reseeds the dice, the
//! same way in every arm and mode, so arms are paired run for run.
//!
//! Each line: mode, arm, salt, end turn, winner team and win turn (the last turn resolved), the
//! seat's captures ([turn, from, city, name]), losses ([turn, to, city, name], plus "razed" when
//! sacked), misfires, cities and VP at the end, every seat's cities, cities founded since the
//! branch, Science per turn and VP at `--report`, and the hybrid counters.
//!
//! Traces (stderr): `HILL_TRACE=<salt>:<arm>:<city name>` prints that city each turn (holder, HP, build, units
//! within 3 tiles, its holder's city and purchase orders) and every purchase with the planning pass that ordered
//! it; `HILL_ONLY_SALT=<s>` plays only that salt.
use legatum_ai::{AiParams, Brain, OpponentMemory, ScriptedBrain};
use legatum_core::economy::player_income;
use legatum_core::orders::Action;
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameState, Hex, Order, PlayerId, TechId, UnitId, UnitKind, player_view,
    resolve_turn, validate_orders,
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
    mode: String,
    victim: Option<PlayerId>,
    war_turn: Option<u32>,
    storm: u32,
    report: Option<u32>,
    arms: Vec<(String, Option<AiParams>)>,
    hybrid_defend: bool,
    /// Read older recorded states through the upgrade (`--accept-rules 49`, `50`, `51` or `52`).
    accept: Option<u32>,
}

fn parse_args() -> Args {
    let mut a = Args {
        tape: String::new(),
        states: String::new(),
        out: String::new(),
        seat: PlayerId(0),
        branch: 0,
        end: 0,
        salts: 48,
        mode: "tape".into(),
        victim: None,
        war_turn: None,
        storm: 3,
        report: None,
        arms: vec![("control".into(), None)],
        hybrid_defend: true,
        accept: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let v = it.next().expect("option needs a value");
        match flag.as_str() {
            "--tape" => a.tape = v,
            "--states" => a.states = v,
            "--out" => a.out = v,
            "--seat" => a.seat = PlayerId(v.parse().unwrap()),
            "--branch" => a.branch = v.parse().unwrap(),
            "--end" => a.end = v.parse().unwrap(),
            "--salts" => a.salts = v.parse().unwrap(),
            "--mode" => a.mode = v,
            "--victim" => a.victim = Some(PlayerId(v.parse().unwrap())),
            "--war-turn" => a.war_turn = Some(v.parse().unwrap()),
            "--storm" => a.storm = v.parse().unwrap(),
            "--report" => a.report = Some(v.parse().unwrap()),
            "--hybrid-defend" => a.hybrid_defend = v == "on",
            "--accept-rules" => {
                assert!(
                    v == "49" || v == "50" || v == "51" || v == "52",
                    "--accept-rules takes 49, 50, 51 or 52 (the older rules versions with an upgrade)"
                );
                a.accept = Some(v.parse().unwrap());
            }
            "--threads" => {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(v.parse().unwrap())
                    .build_global()
                    .unwrap();
            }
            "--arms" => {
                a.arms = v
                    .split(',')
                    .map(|s| match s.split_once('=') {
                        None => (s.to_string(), None),
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
    assert!(a.end > a.branch, "--end must be after --branch");
    assert!(
        ["tape", "hybrid", "hard", "storm", "quiet"].contains(&a.mode.as_str()),
        "unknown mode"
    );
    if a.mode == "storm" {
        assert!(a.victim.is_some(), "--mode storm needs --victim");
    }
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
    /// Hybrid only: his seat in the recorded game at the start of each turn, branch..=end.
    recorded: BTreeMap<u32, Recorded>,
}

struct RecUnit {
    id: u32,
    kind: UnitKind,
    pos: Hex,
    goto: Option<Hex>,
}

/// His units and cities at the start of a recorded turn.
#[derive(Default)]
struct Recorded {
    units: Vec<RecUnit>,
    cities: Vec<(u32, Hex)>,
    /// Indices of his orders this turn that the validator refused in the recorded game too:
    /// he lost them for real, so the hybrid seat drops them instead of handing them to Hard.
    refused_then: BTreeSet<usize>,
    techs: BTreeSet<TechId>,
    /// Teams his team was at war with in the recording, now or within `WAR_LOOKAHEAD` turns.
    wars: BTreeSet<legatum_core::TeamId>,
    /// Open proposals in the recorded game: id -> (from team, kind).
    proposals: BTreeMap<u32, (legatum_core::TeamId, legatum_core::diplomacy::TreatyKind)>,
}

/// The longest list built from `items` (in order, several passes so prerequisites listed later
/// still land first) that the validator accepts as `make(list)`. Salvages a research or
/// production queue whose whole is refused because one item no longer applies.
fn salvage<T: Clone + PartialEq>(
    state: &GameState,
    seat: PlayerId,
    items: &[T],
    make: impl Fn(Vec<T>) -> Order,
) -> Vec<T> {
    let mut out: Vec<T> = Vec::new();
    loop {
        let before = out.len();
        for it in items {
            if out.contains(it) {
                continue;
            }
            let mut tryl = out.clone();
            tryl.push(it.clone());
            if validate_orders(state, seat, &[make(tryl.clone())]).is_empty() {
                out = tryl;
            }
        }
        if out.len() == before {
            return out;
        }
    }
}

fn recorded_seat(state: &GameState, seat: PlayerId, his: &[Order]) -> Recorded {
    Recorded {
        refused_then: validate_orders(state, seat, his)
            .into_iter()
            .map(|e| e.index)
            .collect(),
        techs: state.player(seat).techs.clone(),
        wars: state
            .players
            .iter()
            .filter(|p| p.id != seat && state.at_war(seat, p.id))
            .map(|p| state.team_of(p.id))
            .collect(),
        proposals: state
            .diplomacy
            .iter()
            .flat_map(|d| d.proposals.iter())
            .map(|p| (p.id.0, (p.from, p.kind)))
            .collect(),
        units: state
            .units
            .values()
            .filter(|u| u.owner == seat)
            .map(|u| RecUnit {
                id: u.id.0,
                kind: u.kind,
                pos: u.pos,
                goto: u.goto,
            })
            .collect(),
        cities: state
            .cities
            .values()
            .filter(|c| c.owner == seat)
            .map(|c| (c.id.0, c.pos))
            .collect(),
    }
}

fn warm(tape: &Tape, args: &Args) -> Snapshot {
    let reader = std::io::BufReader::new(std::fs::File::open(&args.states).expect("states file"));
    let mut lines = reader.lines();
    let mut read = |expect: &str| -> GameState {
        let line = lines.next().expect("states file too short").unwrap();
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        assert_eq!(hash.to_string(), expect, "recorded state hash");
        recorded_state(&line, args.accept)
    };
    let mut state = read(&tape.initial_hash.to_string());
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .map(|p| (p.id, brain(&state, p.id, None, 0)))
        .collect();
    let mut snap: Option<Snapshot> = None;
    for t in &tape.turns {
        if let Some(s) = &mut snap {
            // Hybrid: keep reading the recorded game to the window end (his seat only).
            if t.turn > args.end {
                break;
            }
            s.recorded.insert(
                t.turn,
                recorded_seat(
                    &state,
                    args.seat,
                    t.orders.get(&args.seat).map(Vec::as_slice).unwrap_or(&[]),
                ),
            );
            state = read(&t.hash);
            continue;
        }
        if t.turn == args.branch {
            let mut s = Snapshot {
                memory: brains
                    .iter()
                    .filter_map(|(id, b)| b.opponent_memory().map(|m| (*id, m)))
                    .collect(),
                state: state.clone(),
                recorded: BTreeMap::new(),
            };
            if args.mode != "hybrid" {
                return s;
            }
            s.recorded.insert(
                t.turn,
                recorded_seat(
                    &state,
                    args.seat,
                    t.orders.get(&args.seat).map(Vec::as_slice).unwrap_or(&[]),
                ),
            );
            snap = Some(s);
            state = read(&t.hash);
            continue;
        }
        for p in state.players.iter().filter(|p| p.alive) {
            let _ = brains
                .get_mut(&p.id)
                .unwrap()
                .orders(&player_view(&state, p.id));
        }
        state = read(&t.hash);
    }
    let mut snap = snap.unwrap_or_else(|| panic!("branch turn {} is not on the tape", args.branch));
    // A war he declares within the next few turns is his plan, not a surprise to defend against.
    let turns: Vec<u32> = snap.recorded.keys().copied().collect();
    for t in turns {
        let soon: BTreeSet<_> = snap
            .recorded
            .range(t..=t + WAR_LOOKAHEAD)
            .flat_map(|(_, r)| r.wars.iter().copied())
            .collect();
        snap.recorded.get_mut(&t).unwrap().wars = soon;
    }
    snap
}

fn recorded_state(line: &str, accept: Option<u32>) -> GameState {
    if let Some(version) = accept {
        let value: Value = serde_json::from_str(line).expect("recorded state json");
        return if version == 49 {
            legatum_wasm::json::upgrade_value_from_49(value).expect("rules-49 state upgrade")
        } else {
            legatum_wasm::json::upgrade_value_from_50(value).expect("rules-50 state upgrade")
        };
    }
    assert!(
        !line.contains("\"capitals_taken\""),
        "a rules-49 recorded state (it carries capitals_taken): pass --accept-rules 49"
    );
    match serde_json::from_str(line) {
        Ok(state) => state,
        Err(error) if error.to_string().contains("unknown variant `extra`") => {
            // Check the original line's hash first, then omit only retired supply users.
            let mut value: Value = serde_json::from_str(line).expect("recorded state json");
            if let Some(players) = value.get_mut("players").and_then(Value::as_array_mut) {
                for player in players {
                    if let Some(supply) = player.get_mut("supply").and_then(Value::as_array_mut) {
                        supply.retain(|record| {
                            record.pointer("/user/kind").and_then(Value::as_str) != Some("extra")
                        });
                    }
                }
            }
            serde_json::from_value(value).expect("recorded state without retired supply")
        }
        Err(error) => panic!("recorded state json: {error}"),
    }
}

fn report_line(state: &GameState, founded: &[u32]) -> Value {
    json!({
        "turn": state.turn,
        "cities": state.players.iter().map(|p| state.cities_of(p.id).count()).collect::<Vec<_>>(),
        "founded": founded,
        "science": state.players.iter().map(|p| if p.alive { player_income(state, p.id).science.total } else { 0 }).collect::<Vec<_>>(),
        "vp": state.players.iter().map(|p| legatum_core::scoring::vp_total(state, p.id)).collect::<Vec<_>>(),
    })
}

/// A war in the recording counts as his from this many turns before he was in it.
const WAR_LOOKAHEAD: u32 = 3;

/// Recorded units pair with live units of the same kind at most this many tiles apart.
const PAIR_RADIUS: i32 = 8;

#[derive(PartialEq)]
enum Scope {
    Unit(u32),
    City(u32),
    Research,
    Respond,
    Other,
}

fn scope(o: &Order) -> Scope {
    if let Some(u) = legatum_core::rules::units::order_unit(o) {
        return Scope::Unit(u.0);
    }
    match o {
        Order::AirStrike { plane, .. }
        | Order::Rebase { plane, .. }
        | Order::AirRecon { plane, .. }
        | Order::UpgradeAircraft { plane, .. } => Scope::Unit(plane.0),
        Order::Action {
            action: Action::RetireVeteran { unit },
        } => Scope::Unit(unit.0),
        Order::Raze { city }
        | Order::SetProduction { city, .. }
        | Order::SetCityAutomation { city, .. }
        | Order::SetProductionQueue { city, .. }
        | Order::BuyItem { city, .. }
        | Order::HireCompany { city, .. }
        | Order::BuyTile { city, .. } => Scope::City(city.0),
        Order::SetResearch { .. }
        | Order::SetResearchQueue { .. }
        | Order::SetScienceConversion { .. }
        | Order::Action {
            action: Action::ChooseFreeTechs { .. },
        } => Scope::Research,
        Order::RespondToProposal { .. } => Scope::Respond,
        _ => Scope::Other,
    }
}

/// His recorded order with recorded unit and city ids replaced by their live counterparts;
/// None when one of them has no counterpart.
fn translate(
    o: &Order,
    units: &BTreeMap<u32, u32>,
    cities: &BTreeMap<u32, u32>,
    proposals: &BTreeMap<u32, u32>,
) -> Option<Order> {
    let mut v = serde_json::to_value(o).ok()?;
    let rebase = matches!(o, Order::Rebase { .. });
    let obj = v.as_object_mut()?;
    let remap =
        |obj: &mut serde_json::Map<String, Value>, key: &str, map: &BTreeMap<u32, u32>| match obj
            .get(key)
            .and_then(Value::as_u64)
        {
            None => Some(()),
            Some(n) => {
                let live = *map.get(&(n as u32))?;
                obj.insert(key.into(), json!(live));
                Some(())
            }
        };
    remap(obj, "unit", units)?;
    remap(obj, "plane", units)?;
    remap(obj, "city", cities)?;
    remap(obj, "proposal", proposals)?;
    if rebase {
        remap(obj, "to", cities)?;
    }
    if let Some(Value::Object(act)) = obj.get_mut("action") {
        remap(act, "unit", units)?;
    }
    serde_json::from_value(v).ok()
}

/// The hybrid seat: his translated orders where valid, shipped Hard where not.
#[derive(Default)]
struct Hybrid {
    /// Recorded unit id -> live unit id, kept while both exist.
    units: BTreeMap<u32, u32>,
    /// Live settler -> the tile his recorded settler founded on.
    settle: BTreeMap<u32, Hex>,
    kept: u32,
    dropped_missing: u32,
    dropped_invalid: u32,
    dropped_then: u32,
    rerouted: u32,
    salvaged: u32,
    research_steered: u32,
    defence_turns: u32,
    defence_overrides: u32,
    hard_orders: u32,
    steered: u32,
    handed_unit_turns: u32,
    handed_city_turns: u32,
    handed_units: BTreeSet<u32>,
    handed_cities: BTreeSet<u32>,
    reasons: BTreeMap<String, u32>,
    /// `HYBRID_TRACE=<salt>` prints that run's refused and untranslatable orders.
    trace: bool,
    /// `--hybrid-defend` (default on): defend against wars the recording never had.
    defend: bool,
}

impl Hybrid {
    fn pair_units(&mut self, rec: &Recorded, state: &GameState, seat: PlayerId) {
        let by_id: BTreeMap<u32, &RecUnit> = rec.units.iter().map(|u| (u.id, u)).collect();
        self.units.retain(|r, l| {
            by_id.contains_key(r)
                && state
                    .units
                    .get(&UnitId(*l))
                    .is_some_and(|u| u.owner == seat)
        });
        let mut used: BTreeSet<u32> = self.units.values().copied().collect();
        // Same kind first; then any armed unit stands in for an armed one (his Cataphract is
        // a Horseman here when Iron Working came later).
        let armed = legatum_core::rules::is_military;
        for same_kind in [true, false] {
            let mut cand: Vec<(i32, bool, u32, u32)> = Vec::new();
            for r in rec.units.iter().filter(|r| !self.units.contains_key(&r.id)) {
                for u in state.units.values() {
                    if u.owner != seat || used.contains(&u.id.0) {
                        continue;
                    }
                    let fits = if same_kind {
                        u.kind == r.kind
                    } else {
                        armed(u.kind) && armed(r.kind)
                    };
                    let d = u.pos.distance(r.pos);
                    if fits && d <= PAIR_RADIUS {
                        cand.push((d, u.id.0 != r.id, r.id, u.id.0));
                    }
                }
            }
            cand.sort();
            for (_, _, r, l) in cand {
                if self.units.contains_key(&r) || used.contains(&l) {
                    continue;
                }
                self.units.insert(r, l);
                used.insert(l);
            }
        }
    }

    fn orders(
        &mut self,
        state: &GameState,
        seat: PlayerId,
        rec: Option<&Recorded>,
        his: &[Order],
        hard: Vec<Order>,
    ) -> Vec<Order> {
        let mut cities: BTreeMap<u32, u32> = BTreeMap::new();
        // An open proposal in the recording answers to the live one from the same team and of
        // the same kind (proposal ids are global counters too).
        let mut proposals: BTreeMap<u32, u32> = BTreeMap::new();
        if let (Some(rec), Some(d)) = (rec, &state.diplomacy) {
            for (rid, (from, kind)) in &rec.proposals {
                if let Some(p) = d.proposals.iter().find(|p| {
                    p.from == *from && p.kind == *kind && !proposals.values().any(|v| *v == p.id.0)
                }) {
                    proposals.insert(*rid, p.id.0);
                }
            }
        }
        if let Some(rec) = rec {
            self.pair_units(rec, state, seat);
            for (rid, pos) in &rec.cities {
                if let Some(c) = state
                    .cities
                    .values()
                    .find(|c| c.owner == seat && c.pos == *pos)
                {
                    cities.insert(*rid, c.id.0);
                }
            }
        }
        if self.trace
            && let Some(rec) = rec
        {
            let me = state.player(seat);
            eprintln!(
                "T{} techs {}/{} researching {:?} queue {:?} cities {}/{} units {}/{} paired {} gold {}",
                state.turn,
                me.techs.len(),
                rec.techs.len(),
                me.researching,
                me.research_queue,
                state.cities_of(seat).count(),
                rec.cities.len(),
                state.units.values().filter(|u| u.owner == seat).count(),
                rec.units.len(),
                self.units.len(),
                me.gold
            );
        }
        let refused_then = rec.map(|r| &r.refused_then);
        // His city sites: a FoundCity becomes a plan to found on the tile his settler stood on,
        // carried out by its live counterpart (or the nearest free live Settler) when it gets
        // there.
        self.settle.retain(|l, _| {
            state
                .units
                .get(&UnitId(*l))
                .is_some_and(|u| u.owner == seat)
        });
        if let Some(rec) = rec {
            for o in his {
                let Order::FoundCity { unit } = o else {
                    continue;
                };
                let Some(site) = rec.units.iter().find(|u| u.id == unit.0).map(|u| u.pos) else {
                    continue;
                };
                if self.settle.values().any(|p| *p == site) {
                    continue;
                }
                let live = self.units.get(&unit.0).copied().or_else(|| {
                    state
                        .units
                        .values()
                        .filter(|u| {
                            u.owner == seat
                                && u.kind == UnitKind::Settler
                                && !self.settle.contains_key(&u.id.0)
                                && !self.units.values().any(|l| *l == u.id.0)
                        })
                        .min_by_key(|u| (u.pos.distance(site), u.id))
                        .map(|u| u.id.0)
                });
                if let Some(l) = live {
                    self.settle.insert(l, site);
                }
            }
        }
        let mut translated: Vec<Order> = his
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                if refused_then.is_some_and(|r| r.contains(&i)) {
                    self.dropped_then += 1;
                    return None;
                }
                if matches!(o, Order::FoundCity { .. }) {
                    return None;
                }
                let t = translate(o, &self.units, &cities, &proposals);
                if t.is_none() {
                    if self.trace {
                        eprintln!("T{} no counterpart {:?}", state.turn, o);
                    }
                    self.dropped_missing += 1;
                }
                t
            })
            .collect();
        // A confirmed route drawn from where his unit stood in the recording breaks once the
        // live unit stands elsewhere: keep the destination and let the core find the way.
        let mut rerouted = 0;
        for e in validate_orders(state, seat, &translated) {
            match &translated[e.index] {
                Order::MoveUnitPath { unit, to, .. } => {
                    translated[e.index] = Order::MoveUnit {
                        unit: *unit,
                        to: *to,
                    };
                    rerouted += 1;
                }
                Order::SetAutomation {
                    unit,
                    automation,
                    path,
                    stop_reason,
                } if !path.is_empty() => {
                    translated[e.index] = Order::SetAutomation {
                        unit: *unit,
                        automation: *automation,
                        path: Vec::new(),
                        stop_reason: *stop_reason,
                    };
                    rerouted += 1;
                }
                _ => {}
            }
        }
        // A production queue refused for one item keeps the items that still apply.
        for e in validate_orders(state, seat, &translated) {
            if let Order::SetProductionQueue { city, queue } = &translated[e.index] {
                let city = *city;
                let q = salvage(state, seat, queue, |queue| Order::SetProductionQueue {
                    city,
                    queue,
                });
                if !q.is_empty() {
                    translated[e.index] = Order::SetProductionQueue { city, queue: q };
                    self.salvaged += 1;
                }
            }
        }
        // Research follows his plan: techs he knew by now in the recording come first, then his
        // queue (this turn's order, else the live plan), each kept only if it applies now.
        if let Some(rec) = rec {
            let me = state.player(seat);
            let ordered: Option<Vec<TechId>> = translated.iter().rev().find_map(|o| match o {
                Order::SetResearchQueue { techs } => Some(techs.clone()),
                Order::SetResearch { tech } => Some(vec![*tech]),
                _ => None,
            });
            let ordered_ok = ordered.as_ref().is_some_and(|q| {
                validate_orders(state, seat, &[Order::SetResearchQueue { techs: q.clone() }])
                    .is_empty()
            });
            let missing: Vec<TechId> = rec
                .techs
                .iter()
                .filter(|t| !me.techs.contains(t))
                .copied()
                .collect();
            if !missing.is_empty() || (ordered.is_some() && !ordered_ok) {
                let mut want = missing;
                want.extend(ordered.unwrap_or_else(|| me.research_queue.clone()));
                let q = salvage(state, seat, &want, |techs| Order::SetResearchQueue {
                    techs,
                });
                translated.retain(|o| {
                    !matches!(
                        o,
                        Order::SetResearchQueue { .. } | Order::SetResearch { .. }
                    )
                });
                if !q.is_empty() && q != me.research_queue {
                    translated.push(Order::SetResearchQueue { techs: q });
                    self.research_steered += 1;
                }
            }
        }
        let bad: BTreeMap<usize, String> = validate_orders(state, seat, &translated)
            .into_iter()
            .map(|e| (e.index, e.message))
            .collect();
        let mut kept = Vec::new();
        let mut invalid = Vec::new();
        for (i, o) in translated.into_iter().enumerate() {
            match bad.get(&i) {
                Some(why) => {
                    if self.trace {
                        eprintln!("T{} refused {:?}: {why}", state.turn, o);
                    }
                    self.dropped_invalid += 1;
                    let key: String = why.chars().take(48).collect();
                    *self.reasons.entry(key).or_default() += 1;
                    invalid.push(scope(&o));
                }
                None => kept.push(o),
            }
        }
        // He defends against a war the recording never had (an AI declared on him here): a city
        // with such an enemy's armed unit within 3 tiles plays Hard while the threat lasts, with
        // his armed units within 5 tiles of it that were home in the recording too. (Enemies of
        // his recorded wars, and his armies passing by, are left to his plan: taking them over
        // cost Bibracte in M72.)
        let mut defend_units: BTreeSet<u32> = BTreeSet::new();
        let mut defend_cities: BTreeSet<u32> = BTreeSet::new();
        let mut defend_at: Vec<Hex> = Vec::new();
        if let Some(rec) = rec.filter(|_| self.defend) {
            let rec_of: BTreeMap<u32, u32> = self.units.iter().map(|(r, l)| (*l, *r)).collect();
            let new_enemy = |p: PlayerId| {
                p != seat
                    && !p.is_barbarian()
                    && state.at_war(seat, p)
                    && !rec.wars.contains(&state.team_of(p))
            };
            for c in state.cities.values().filter(|c| c.owner == seat) {
                let threat = state.units.values().any(|u| {
                    new_enemy(u.owner)
                        && legatum_core::rules::is_military(u.kind)
                        && u.pos.distance(c.pos) <= 3
                });
                if threat {
                    defend_cities.insert(c.id.0);
                    defend_at.push(c.pos);
                    defend_units.extend(
                        state
                            .units
                            .values()
                            .filter(|u| {
                                // Not his units bound elsewhere: one paired to a recorded unit
                                // away from this city then keeps his orders.
                                let home = rec_of.get(&u.id.0).is_none_or(|r| {
                                    rec.units
                                        .iter()
                                        .find(|x| x.id == *r)
                                        .is_some_and(|x| x.pos.distance(c.pos) <= 5)
                                });
                                u.owner == seat
                                    && legatum_core::rules::is_military(u.kind)
                                    && u.pos.distance(c.pos) <= 5
                                    && home
                            })
                            .map(|u| u.id.0),
                    );
                }
            }
        }
        if !defend_units.is_empty() || !defend_cities.is_empty() {
            let before = kept.len();
            kept.retain(|o| match scope(o) {
                Scope::Unit(u) => !defend_units.contains(&u),
                Scope::City(c) => !defend_cities.contains(&c),
                _ => true,
            });
            self.defence_overrides += (before - kept.len()) as u32;
            self.defence_turns += 1;
            for u in &defend_units {
                self.settle.remove(u);
            }
        }
        let kept_scopes: Vec<Scope> = kept.iter().map(scope).collect();
        let mapped_units: BTreeSet<u32> = self.units.values().copied().collect();
        let mapped_cities: BTreeSet<u32> = cities.values().copied().collect();
        let mut handed_units: BTreeSet<u32> = state
            .units
            .values()
            .filter(|u| u.owner == seat && !mapped_units.contains(&u.id.0))
            .map(|u| u.id.0)
            .collect();
        let mut handed_cities: BTreeSet<u32> = state
            .cities
            .values()
            .filter(|c| c.owner == seat && !mapped_cities.contains(&c.id.0))
            .map(|c| c.id.0)
            .collect();
        // Settle plans: found on the site, else walk there; a site that no longer takes a city
        // ends the plan and the settler goes to Hard.
        let mut settle_orders = Vec::new();
        let plans: Vec<(u32, Hex)> = self.settle.iter().map(|(l, p)| (*l, *p)).collect();
        for (l, site) in plans {
            let u = &state.units[&UnitId(l)];
            let o = if u.pos == site {
                Order::FoundCity { unit: UnitId(l) }
            } else {
                Order::MoveUnit {
                    unit: UnitId(l),
                    to: site,
                }
            };
            if validate_orders(state, seat, std::slice::from_ref(&o)).is_empty() {
                handed_units.remove(&l);
                settle_orders.push(o);
            } else {
                self.settle.remove(&l);
                handed_units.insert(l);
            }
        }
        handed_units.extend(defend_units.iter().copied());
        handed_cities.extend(defend_cities.iter().copied());
        for s in &invalid {
            match s {
                Scope::Unit(u) if !kept_scopes.contains(&Scope::Unit(*u)) => {
                    handed_units.insert(*u);
                }
                Scope::City(c) if !kept_scopes.contains(&Scope::City(*c)) => {
                    handed_cities.insert(*c);
                }
                _ => {}
            }
        }
        // Hard picks research when his order was refused, or when his plan ran out before the
        // recording gave the next one (research must never sit idle).
        let idle = state.player(seat).researching.is_none()
            && !kept.iter().any(|o| {
                matches!(o, Order::SetResearchQueue { techs } if !techs.is_empty())
                    || matches!(o, Order::SetResearch { .. })
            });
        let research = (invalid.contains(&Scope::Research) || idle)
            && !kept.iter().any(|o| {
                matches!(o, Order::SetResearchQueue { techs } if !techs.is_empty())
                    || matches!(o, Order::SetResearch { .. })
            });
        let respond = invalid.contains(&Scope::Respond);
        let answered: Vec<_> = kept
            .iter()
            .filter_map(|o| match o {
                Order::RespondToProposal { proposal, .. } => Some(*proposal),
                _ => None,
            })
            .collect();
        // Paired units he gave no order this turn go where his recorded unit was heading.
        let mut steer = Vec::new();
        if let Some(rec) = rec {
            let by_id: BTreeMap<u32, &RecUnit> = rec.units.iter().map(|u| (u.id, u)).collect();
            for (r, l) in &self.units {
                if kept_scopes.contains(&Scope::Unit(*l))
                    || handed_units.contains(l)
                    || self.settle.contains_key(l)
                {
                    continue;
                }
                let (ru, lu) = (by_id[r], &state.units[&UnitId(*l)]);
                if lu.attack_target.is_some() {
                    continue;
                }
                let to = match ru.goto {
                    Some(g) if lu.goto != Some(g) && lu.pos != g => Some(g),
                    None if lu.goto.is_none()
                        && lu.standing_order.is_none()
                        && lu.automation.is_none()
                        && lu.pos.distance(ru.pos) >= 2 =>
                    {
                        Some(ru.pos)
                    }
                    _ => None,
                };
                if let Some(to) = to {
                    steer.push(Order::MoveUnit {
                        unit: UnitId(*l),
                        to,
                    });
                }
            }
        }
        let extra: Vec<Order> = hard
            .into_iter()
            .filter(|o| match scope(o) {
                // A unit he still commands keeps its kind and stays in his army: Hard only
                // fights or moves it for the turn his order fails.
                // A defender stays by the threatened cities.
                Scope::Unit(u)
                    if defend_units.contains(&u)
                        && matches!(o, Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. }
                            if !defend_at.iter().any(|c| c.distance(*to) <= 5)) =>
                {
                    false
                }
                Scope::Unit(u) if mapped_units.contains(&u) => {
                    handed_units.contains(&u)
                        && !matches!(
                            o,
                            Order::UpgradeUnit { .. }
                                | Order::UpgradeSiege { .. }
                                | Order::UpgradeAircraft { .. }
                                | Order::DisbandUnit { .. }
                                | Order::FoundCity { .. }
                                | Order::Action { .. }
                        )
                }
                Scope::Unit(u) => handed_units.contains(&u),
                Scope::City(c) => handed_cities.contains(&c),
                Scope::Research => research,
                Scope::Respond => {
                    respond
                        && matches!(o, Order::RespondToProposal { proposal, .. } if !answered.contains(proposal))
                }
                Scope::Other => false,
            })
            .collect();
        self.handed_unit_turns += handed_units.len() as u32;
        self.handed_city_turns += handed_cities.len() as u32;
        self.handed_units.extend(handed_units);
        self.handed_cities.extend(handed_cities);
        let n_kept = kept.len();
        let mut merged: Vec<(u8, Order)> = kept
            .into_iter()
            .map(|o| (0, o))
            .chain(settle_orders.into_iter().chain(steer).map(|o| (1, o)))
            .chain(extra.into_iter().map(|o| (2, o)))
            .collect();
        // Added orders yield to his: drop any the validator refuses, then check again (dropping
        // frees gold and supply, so this settles).
        for _ in 0..4 {
            let list: Vec<Order> = merged.iter().map(|(_, o)| o.clone()).collect();
            let bad: BTreeSet<usize> = validate_orders(state, seat, &list)
                .into_iter()
                .map(|e| e.index)
                .filter(|i| *i >= n_kept)
                .collect();
            if bad.is_empty() {
                break;
            }
            merged = merged
                .into_iter()
                .enumerate()
                .filter(|(i, _)| !bad.contains(i))
                .map(|(_, x)| x)
                .collect();
        }
        if self.trace {
            for (k, o) in &merged {
                if *k > 0 {
                    eprintln!("T{} added({k}) {:?}", state.turn, o);
                }
            }
            if rerouted > 0 {
                eprintln!("T{} rerouted {rerouted}", state.turn);
            }
        }
        self.kept += n_kept as u32;
        self.rerouted += rerouted;
        self.steered += merged.iter().filter(|(k, _)| *k == 1).count() as u32;
        self.hard_orders += merged.iter().filter(|(k, _)| *k == 2).count() as u32;
        merged.into_iter().map(|(_, o)| o).collect()
    }

    fn report(&self) -> Value {
        let mut reasons: Vec<(&String, &u32)> = self.reasons.iter().collect();
        reasons.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        json!({"kept": self.kept, "dropped_missing": self.dropped_missing,
            "dropped_invalid": self.dropped_invalid, "dropped_then": self.dropped_then,
            "rerouted": self.rerouted, "salvaged": self.salvaged,
            "research_steered": self.research_steered,
            "defence_turns": self.defence_turns, "defence_overrides": self.defence_overrides, "hard_orders": self.hard_orders,
            "steered": self.steered, "handed_unit_turns": self.handed_unit_turns,
            "handed_city_turns": self.handed_city_turns, "handed_units": self.handed_units.len(),
            "handed_cities": self.handed_cities.len(),
            "invalid_reasons": reasons.into_iter().take(6).map(|(r, n)| json!([r, n])).collect::<Vec<_>>()})
    }
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
    if std::env::var("HILL_TRACE").is_ok() {
        legatum_ai::buy_trace::enable();
    }
    if salt != 0 {
        state.dice_seed = Some(salted(state.setup.seed, salt ^ 0xD1CE));
    }
    let hybrid = args.mode == "hybrid";
    let seat_ai = args.mode != "tape" && !hybrid;
    let mut hy = Hybrid {
        trace: std::env::var("HYBRID_TRACE").is_ok_and(|v| v == salt.to_string()),
        defend: args.hybrid_defend,
        ..Hybrid::default()
    };
    if seat_ai {
        state.players[seat.0 as usize].is_human = false;
        if let Some(p) = state.setup.players.get_mut(seat.0 as usize) {
            p.is_human = false;
            p.difficulty = AiDifficulty::Hard;
        }
    }
    let rusher = (args.mode == "storm").then(|| legatum_wasm::rusher::Rusher {
        player: seat,
        wars: vec![legatum_wasm::rusher::RusherWar {
            against: args.victim.unwrap(),
            turn: args.war_turn.unwrap_or(args.branch),
        }],
        storm: args.storm,
        target: args.victim,
    });
    let mut brains: BTreeMap<PlayerId, ScriptedBrain> = state
        .players
        .iter()
        .filter(|p| p.alive && (p.id != seat || seat_ai || hybrid))
        .map(|p| {
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
    let n = state.players.len();
    let mut founded = vec![0u32; n];
    let mut captures: Vec<Value> = Vec::new();
    let mut losses: Vec<Value> = Vec::new();
    let mut misfires = 0u32;
    let mut misfire_turns: BTreeMap<u32, u32> = BTreeMap::new();
    let mut seat_orders = 0u32;
    let mut report: Option<Value> = None;
    let mut wars: Vec<Value> = Vec::new();
    while !state.finished && state.turn <= args.end {
        let turn = state.turn;
        if args.report == Some(turn) {
            report = Some(report_line(&state, &founded));
        }
        // Screen (env HS_TECH): every AI gets his technologies each turn (tech-parity arm).
        if std::env::var("HS_TECH").is_ok() {
            let his: Vec<_> = state.player(seat).techs.iter().copied().collect();
            let ai_ids: Vec<PlayerId> = state
                .players
                .iter()
                .filter(|p| p.alive && p.id != seat)
                .map(|p| p.id)
                .collect();
            let mut ev = Vec::new();
            for id in &ai_ids {
                for t in &his {
                    if !state.player(*id).techs.contains(t) {
                        legatum_core::eras::learn(&mut state, *id, *t, &mut ev);
                    }
                }
            }
        }
        // Diagnosis (env HS_VP=salt): every seat's VP by source and cities, each turn.
        if std::env::var("HS_VP").is_ok_and(|v| v == salt.to_string()) {
            for p in state.players.iter().filter(|p| p.alive) {
                let cities: Vec<_> = state.cities_of(p.id).map(|c| c.name.clone()).collect();
                eprintln!(
                    "HSV s{salt} T{turn} P{} vp={:?} cities={:?}",
                    p.id.0,
                    legatum_core::scoring::vp(&state, p.id),
                    cities
                );
            }
        }
        let mut orders = legatum_wasm::session::OrderMap::new();
        let mut hard_seat = Vec::new();
        for (id, b) in brains.iter_mut() {
            if !state.player(*id).alive {
                continue;
            }
            let mut o = b.orders(&player_view(&state, *id));
            if *id == seat && args.mode == "quiet" {
                o.retain(|x| !matches!(x, Order::DeclareWar { .. }));
            }
            if *id == seat && hybrid {
                hard_seat = o;
                continue;
            }
            orders.insert(*id, o);
        }
        if state.player(seat).alive && !seat_ai {
            let o = recorded.get(&turn).cloned().unwrap_or_default();
            seat_orders += o.len() as u32;
            let o = if hybrid {
                hy.orders(&state, seat, snap.recorded.get(&turn), &o, hard_seat)
            } else {
                o
            };
            orders.insert(seat, o);
        }
        if let Some(r) = &rusher {
            legatum_wasm::rusher::apply(&mut orders, &state, r);
        }
        // `HILL_TRACE=<salt>:<arm>:<city name>`: that city each turn (holder, HP, build, garrison
        // nearby, the holder's orders for it and its Gold) on stderr.
        if let Ok(spec) = std::env::var("HILL_TRACE") {
            let mut it = spec.splitn(3, ':');
            let (ts, ta, tc) = (
                it.next().unwrap_or(""),
                it.next().unwrap_or(""),
                it.next().unwrap_or(""),
            );
            if ts == salt.to_string()
                && ta == arm.0
                && let Some(c) = state.cities.values().find(|c| c.name == tc)
            {
                let near: Vec<_> = state
                    .units
                    .values()
                    .filter(|u| u.pos.distance(c.pos) <= 3)
                    .map(|u| {
                        format!(
                            "p{}:{:?}@{}hp{}",
                            u.owner.0,
                            u.kind,
                            u.pos.distance(c.pos),
                            u.hp
                        )
                    })
                    .collect();
                let mine: Vec<_> = orders
                    .get(&c.owner)
                    .map(|v| {
                        v.iter()
                            .filter(|o| {
                                scope(o) == Scope::City(c.id.0)
                                    || matches!(
                                        o,
                                        Order::BuyItem { .. } | Order::UpgradeUnit { .. }
                                    )
                            })
                            .map(|o| format!("{o:?}"))
                            .collect()
                    })
                    .unwrap_or_default();
                eprintln!(
                    "TRACE T{turn} {} owner p{} hp {} pop {} producing {:?} stored {} queue {:?} gold {} walls {} | near {:?} | orders {:?}",
                    c.name,
                    c.owner.0,
                    c.hp,
                    c.population,
                    c.producing,
                    c.production_stored,
                    c.production_queue,
                    state.player(c.owner).gold,
                    c.buildings
                        .iter()
                        .any(|b| format!("{b:?}").contains("Wall")),
                    near,
                    mine
                );
            }
        }
        if std::env::var("HILL_TRACE").is_ok() {
            for t in legatum_ai::buy_trace::take() {
                eprintln!(
                    "BUY T{turn} p{} city {} {:?} pass {}",
                    t.player.0,
                    state.cities.get(&t.city).map_or("?", |c| c.name.as_str()),
                    t.item,
                    t.pass
                );
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        let name = |c: &legatum_core::CityId| next.cities.get(c).map(|c| c.name.clone());
        for e in &events {
            match e {
                Event::CityCaptured { city, from, to, .. } if *to == seat => {
                    captures.push(json!([turn, from.0, city.0, name(city)]));
                }
                Event::CityCaptured { city, from, to, .. } if *from == seat => {
                    losses.push(json!([turn, to.0, city.0, name(city)]));
                }
                // A Celtic sack (or an ordered Raze) burns the city without a capture.
                Event::CityRazed { city, from, by, .. } if *from == seat && *by != seat => {
                    let n = state.cities.get(city).map(|c| c.name.clone());
                    losses.push(json!([turn, by.0, city.0, n, "razed"]));
                }
                Event::CityFounded { owner, .. } => founded[owner.0 as usize] += 1,
                Event::OrderFailed { player, .. } if *player == seat && !seat_ai => {
                    misfires += 1;
                    *misfire_turns.entry(turn).or_default() += 1;
                }
                Event::WarDeclared { relation }
                    if relation.declarers.contains(&state.team_of(seat)) =>
                {
                    wars.push(json!([turn, relation.a.0, relation.b.0]));
                }
                _ => {}
            }
        }
        state = next;
    }
    if report.is_none() && args.report.is_some_and(|r| r >= state.turn) {
        report = Some(report_line(&state, &founded));
    }
    let seat_team = state.team_of(seat);
    json!({"mode": args.mode, "branch": args.branch, "arm": arm.0, "salt": salt,
        "end": state.turn, "finished": state.finished,
        "winner": state.winner.map(|t| t.0), "seat_won": state.winner == Some(seat_team),
        "win_turn": if state.finished { Some(state.turn - 1) } else { None },
        "captures": captures, "losses": losses, "seat_wars": wars,
        "misfires": misfires, "seat_orders": seat_orders,
        "misfire_first": misfire_turns.keys().next(),
        "seat_cities": state.cities_of(seat).count(),
        "seat_vp": legatum_core::scoring::vp_total(&state, seat),
        "cities": state.players.iter().map(|p| state.cities_of(p.id).count()).collect::<Vec<_>>(),
        "city_owner": state.cities.values().map(|c| (c.id.0, c.owner.0)).collect::<BTreeMap<_, _>>(),
        "founded": founded,
        "hybrid": hybrid.then(|| hy.report()),
        "report": report})
}

fn main() {
    let args = parse_args();
    let tape =
        Tape::from_json(&std::fs::read_to_string(&args.tape).expect("read tape")).expect("tape");
    let snap = warm(&tape, &args);
    eprintln!("hill_suite: warmed to T{} ({})", snap.state.turn, args.mode);
    let jobs: Vec<(usize, u64)> = (0..args.arms.len())
        .flat_map(|a| (0..args.salts).map(move |s| (a, s)))
        // `HILL_ONLY_SALT=<s>`: play just that salt (traces).
        .filter(|&(_, s)| std::env::var("HILL_ONLY_SALT").map_or(true, |v| v == s.to_string()))
        .collect();
    let rows: Vec<Value> = jobs
        .par_iter()
        .map(|&(a, s)| play(&snap, &tape, &args, &args.arms[a], s))
        .collect();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args.out).unwrap());
    for r in &rows {
        writeln!(out, "{r}").unwrap();
    }
    eprintln!("hill_suite: {} runs -> {}", rows.len(), args.out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_state_drops_only_retired_extra_supply() {
        let state = legatum_core::new_game(&legatum_core::GameSetup::all_ai(1));
        let mut historical = serde_json::to_value(&state).unwrap();
        historical["players"][0]["supply"] = json!([
            {"user": {"kind": "unit", "unit": 1}, "resource": "Iron", "since": 1},
            {"user": {"kind": "extra", "unit": 1}, "resource": "Oil", "since": 1}
        ]);
        let loaded = recorded_state(&historical.to_string(), None);
        let mut expected = historical;
        expected["players"][0]["supply"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert_eq!(serde_json::to_value(loaded).unwrap(), expected);
    }
}

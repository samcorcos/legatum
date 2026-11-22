//! Benchmark the Hard AI against a human playtest save (analysis only; observer metrics).
//!
//! playtest_bench replay <save.legatum>
//!     Replay the human's recorded orders (hash-checked) and report every seat's macro metrics.
//! playtest_bench play <save.legatum> <hard|normal|proxy> <salt> [rules]
//!     Same map, setup and factions; seat 0 is played by the given brain, seats 1-7 by the
//!     save's Hard AIs. Salt 0 keeps the playtest's AI brain seeds; other salts re-seed every
//!     scripted brain's tie-break jitter. `rules` is a comma list of optional rule experiments. The old pacing names parse as defaults.
//! playtest_bench salts <save.legatum> <seat brain> <hard|hardv2> <salts> [rules]
//!     `play` for salts 1..=salts in parallel, with seats 1-7 played by old Hard or Hard v2.
//!     Seat brains also include hardv2 and snowball (the Hard v2 tuned Snowball opening).
//! playtest_bench maps <first_seed> <games> <hard|proxy> [rules]
//!     legatum-sim's rotated Proxy schedule (seat i%8, faction cycle), 7 Hard AIs, barbarians
//!     on, Standard; the rotated seat is played by the given brain. One JSON line per game.
//!
//! Output is JSON lines on stdout. Nothing here reaches a brain.
use legatum_ai::{Brain, ProxyBrain, ScriptedBrain};
use legatum_core::diplomacy::RelationState;
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, GameState, PlayerId, ProductionItem, UnitKind, scoring,
};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SNAPSHOTS: [u32; 8] = [25, 50, 75, 100, 135, 150, 200, 250];

fn load_save(path: &str) -> (GameState, legatum_wasm::history::Recording) {
    let outer: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let game = outer["game"].as_str().expect("game JSON string");
    let state = legatum_wasm::json::parse_state(game).unwrap();
    let v: Value = serde_json::from_str(game).unwrap();
    let rec: legatum_wasm::history::Recording =
        serde_json::from_value(v["archive"].clone()).expect("archive");
    (state, rec)
}

fn rules(list: Option<&String>) -> legatum_core::experiments::RuleExperiments {
    let mut r = legatum_core::experiments::RuleExperiments::default();
    for name in list
        .map(|s| s.split(',').collect::<Vec<_>>())
        .unwrap_or_default()
    {
        match name {
            "" | "none" => {}
            other => r.apply(other).unwrap_or_else(|error| panic!("{error}")),
        }
    }
    r
}

fn is_military(k: UnitKind) -> bool {
    !matches!(k, UnitKind::Scout | UnitKind::Settler)
}

#[derive(Default)]
struct Seat {
    snapshots: BTreeMap<u32, Value>,
    foundings: Vec<u32>,
    captures: Vec<u32>,
    cities_lost: u32,
    buys: Vec<(u32, String, i32)>,
    settler_turns_out: u32,
    settler_turns_out_escorted: u32,
    settler_turns_out_escorted_adjacent: u32,
    settlers_built: u32,
    settlers_lost: u32,
    declared: Vec<(u32, u8)>,
    war_received: Vec<(u32, u8)>,
    peace_made: u32,
    alliances: Vec<(u32, u8, Option<u8>)>,
    alliance_proposals: u32,
    coalition_target_turns: u32,
    first_coalition_target: Option<u32>,
    /// Sum over coalition-target turns of empires at war with this seat.
    wars_while_target: u32,
    /// Sum over turns where someone else is the coalition target: at war with that target?
    war_on_target_turns: u32,
    other_target_turns: u32,
    gold_sum: i64,
    turns: u32,
    max_gold: i32,
}

struct Collector {
    seats: Vec<Seat>,
}

impl Collector {
    fn new(n: usize) -> Self {
        Self {
            seats: (0..n).map(|_| Seat::default()).collect(),
        }
    }

    fn orders(&mut self, orders: &legatum_wasm::session::OrderMap) {
        for (p, list) in orders {
            let Some(seat) = self.seats.get_mut(p.0 as usize) else {
                continue;
            };
            for o in list {
                if let legatum_core::Order::ProposeTreaty {
                    kind: legatum_core::diplomacy::TreatyKind::Alliance,
                    ..
                } = o
                {
                    seat.alliance_proposals += 1;
                }
            }
        }
    }

    fn events(&mut self, turn: u32, state: &GameState, events: &[Event]) {
        let team_seats = |t: legatum_core::TeamId| -> Vec<usize> {
            state
                .players
                .iter()
                .filter(|p| p.team == t)
                .map(|p| p.id.0 as usize)
                .collect()
        };
        for e in events {
            match e {
                Event::CityFounded { owner, .. } => {
                    if let Some(s) = self.seats.get_mut(owner.0 as usize) {
                        s.foundings.push(turn)
                    }
                }
                Event::CityCaptured { from, to, .. } => {
                    if let Some(s) = self.seats.get_mut(to.0 as usize) {
                        s.captures.push(turn)
                    }
                    if let Some(s) = self.seats.get_mut(from.0 as usize) {
                        s.cities_lost += 1
                    }
                }
                Event::ItemBought {
                    owner, item, gold, ..
                } => {
                    if let Some(s) = self.seats.get_mut(owner.0 as usize) {
                        let what = match item {
                            ProductionItem::Unit { unit } if *unit == UnitKind::Settler => {
                                "settler".to_string()
                            }
                            ProductionItem::Unit { unit } if is_military(*unit) => {
                                "military".to_string()
                            }
                            ProductionItem::Unit { .. } => "civilian".to_string(),
                            ProductionItem::Building { .. } | ProductionItem::Wonder { .. } => {
                                "building".to_string()
                            }
                            ProductionItem::Project { .. } => "project".to_string(),
                            ProductionItem::Improvement { .. } => "improvement".to_string(),
                        };
                        s.buys.push((turn, what, *gold));
                    }
                }
                Event::ProductionCompleted {
                    owner,
                    item:
                        ProductionItem::Unit {
                            unit: UnitKind::Settler,
                        },
                    ..
                } => {
                    if let Some(s) = self.seats.get_mut(owner.0 as usize) {
                        s.settlers_built += 1
                    }
                }
                Event::SettlerCaptured { from, .. } => {
                    if let Some(s) = self.seats.get_mut(from.0 as usize) {
                        s.settlers_lost += 1
                    }
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind: UnitKind::Settler,
                    ..
                } => {
                    if let Some(s) = self.seats.get_mut(owner.0 as usize) {
                        s.settlers_lost += 1
                    }
                }
                Event::WarDeclared { relation } => {
                    for (me, other) in [(relation.a, relation.b), (relation.b, relation.a)] {
                        for i in team_seats(me) {
                            if relation.declarers.contains(&me) {
                                self.seats[i].declared.push((turn, other.0));
                            } else {
                                self.seats[i].war_received.push((turn, other.0));
                            }
                        }
                    }
                }
                Event::PeaceEstablished { relation } => {
                    for t in [relation.a, relation.b] {
                        for i in team_seats(t) {
                            self.seats[i].peace_made += 1;
                        }
                    }
                }
                Event::TreatyActivated { relation, .. } => {
                    for (me, other) in [(relation.a, relation.b), (relation.b, relation.a)] {
                        for i in team_seats(me) {
                            if relation.state == RelationState::Alliance {
                                self.seats[i].alliances.push((
                                    turn,
                                    other.0,
                                    relation
                                        .security
                                        .as_ref()
                                        .and_then(|s| s.coalition_target)
                                        .map(|t| t.0),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn resolved(&mut self, turn: u32, state: &GameState) {
        let target = legatum_core::diplomacy::coalition::target_state(state);
        for p in &state.players {
            let s = &mut self.seats[p.id.0 as usize];
            if !p.alive {
                continue;
            }
            s.turns += 1;
            s.gold_sum += p.gold as i64;
            s.max_gold = s.max_gold.max(p.gold);
            for u in state.units_of(p.id).filter(|u| u.kind == UnitKind::Settler) {
                let own = state
                    .map
                    .get(u.pos)
                    .is_some_and(|t| t.owner.is_some_and(|o| state.same_team(o, p.id)));
                if own {
                    continue;
                }
                s.settler_turns_out += 1;
                let mil = state
                    .units_of(p.id)
                    .filter(|m| is_military(m.kind))
                    .map(|m| m.pos.distance(u.pos))
                    .min();
                if mil == Some(0) {
                    s.settler_turns_out_escorted += 1;
                }
                if mil.is_some_and(|d| d <= 1) {
                    s.settler_turns_out_escorted_adjacent += 1;
                }
            }
            match target {
                Some(t) if t == p.team => {
                    s.coalition_target_turns += 1;
                    s.first_coalition_target.get_or_insert(turn);
                    s.wars_while_target += state
                        .players
                        .iter()
                        .filter(|q| q.alive && q.team != p.team && state.at_war(p.id, q.id))
                        .count() as u32;
                }
                Some(t) => {
                    s.other_target_turns += 1;
                    if state
                        .players
                        .iter()
                        .any(|q| q.team == t && q.alive && state.at_war(p.id, q.id))
                    {
                        s.war_on_target_turns += 1;
                    }
                }
                None => {}
            }
            if SNAPSHOTS.contains(&turn) {
                s.snapshots.insert(
                    turn,
                    json!({
                        "cities": state.cities_of(p.id).count(),
                        "pop": state.cities_of(p.id).map(|c| c.population).sum::<i32>(),
                        "vp": scoring::vp_total(state, p.id),
                        "army": scoring::army_strength(state, p.id),
                        "techs": p.techs.len(),
                        "gold": p.gold,
                        "military_units": state.units_of(p.id).filter(|u| is_military(u.kind)).count(),
                    }),
                );
            }
        }
    }

    fn report(&self, state: &GameState, brains: &[String]) -> Value {
        let seats: Vec<Value> = state
            .players
            .iter()
            .map(|p| {
                let s = &self.seats[p.id.0 as usize];
                let buy_gold: i32 = s.buys.iter().map(|b| b.2).sum();
                let mut buy_kinds: BTreeMap<&str, u32> = BTreeMap::new();
                for b in &s.buys {
                    *buy_kinds.entry(b.1.as_str()).or_default() += 1;
                }
                json!({
                    "player": p.id.0,
                    "team": p.team.0,
                    "faction": format!("{:?}", p.faction),
                    "brain": brains[p.id.0 as usize],
                    "alive": p.alive,
                    "final_vp": scoring::vp_total(state, p.id),
                    "final_cities": state.cities_of(p.id).count(),
                    "snapshots": s.snapshots,
                    "foundings": s.foundings,
                    "captures": s.captures,
                    "cities_lost": s.cities_lost,
                    "buys": s.buys.len(),
                    "buy_gold": buy_gold,
                    "buy_kinds": buy_kinds,
                    "first_buy": s.buys.first().map(|b| b.0),
                    "buys_by_100": s.buys.iter().filter(|b| b.0 <= 100).count(),
                    "avg_gold": if s.turns > 0 { s.gold_sum / s.turns as i64 } else { 0 },
                    "max_gold": s.max_gold,
                    "settlers_built": s.settlers_built,
                    "settlers_lost": s.settlers_lost,
                    "settler_turns_out": s.settler_turns_out,
                    "settler_turns_out_escorted": s.settler_turns_out_escorted,
                    "settler_turns_out_escorted_adjacent": s.settler_turns_out_escorted_adjacent,
                    "declared": s.declared,
                    "war_received": s.war_received,
                    "peace_made": s.peace_made,
                    "alliances": s.alliances,
                    "alliance_proposals": s.alliance_proposals,
                    "coalition_target_turns": s.coalition_target_turns,
                    "first_coalition_target": s.first_coalition_target,
                    "wars_while_target": s.wars_while_target,
                    "other_target_turns": s.other_target_turns,
                    "war_on_target_turns": s.war_on_target_turns,
                })
            })
            .collect();
        json!({
            "turn": state.turn,
            "finished": state.finished,
            "winner_team": state.winner.map(|t| t.0),
            "seats": seats,
        })
    }
}

fn brain_for(kind: &str, state: &GameState, p: PlayerId, seed: u64) -> (Box<dyn Brain>, String) {
    let faction = state.player(p).faction;
    match kind {
        "hard" => (
            Box::new(ScriptedBrain::with_difficulty(
                faction,
                seed,
                p,
                AiDifficulty::Hard,
            )),
            "scripted-hard".into(),
        ),
        "normal" => (
            Box::new(ScriptedBrain::with_difficulty(
                faction,
                seed,
                p,
                AiDifficulty::Normal,
            )),
            "scripted-normal".into(),
        ),
        "hardv2" => (
            legatum_ai::hard_v2::brain(faction, seed, p),
            format!("hard-v2:{}", legatum_ai::hard_v2::draw_opening(seed).name()),
        ),
        "snowball" => {
            let mut b = legatum_ai::openings::OpeningBrain::new(
                ScriptedBrain::with_difficulty(faction, seed, p, AiDifficulty::Hard),
                legatum_ai::hard_v2::policy(legatum_ai::openings::Opening::Snowball),
            );
            b.primitives_enabled = true;
            (Box::new(b), "opening:Snowball".into())
        }
        "proxy" => (Box::new(ProxyBrain), "proxy-v4".into()),
        "proxy7" => (
            Box::new(legatum_ai::HumanPlaybookV7Brain::default()),
            "proxy-v7".into(),
        ),
        other => panic!("unknown brain {other}"),
    }
}

fn salted(seed: u64, salt: u64) -> u64 {
    if salt == 0 {
        seed
    } else {
        Rng::derive(seed, &[purpose::AI, 0x5a17, salt]).next_u64()
    }
}

/// Play to the end (or the safety cap) with `seat_kind` in `seat`; everyone else Hard.
fn play(setup: &GameSetup, seat: u8, seat_kind: &str, salt: u64) -> Value {
    play_vs(setup, seat, seat_kind, "hard", salt)
}

/// `play` with the other seats played by `others` (hard or hardv2).
fn play_vs(setup: &GameSetup, seat: u8, seat_kind: &str, others: &str, salt: u64) -> Value {
    let mut state = legatum_core::new_game(setup);
    let mut brains: Vec<(PlayerId, Box<dyn Brain>)> = Vec::new();
    let mut names = Vec::new();
    for p in &setup.players.iter().enumerate().collect::<Vec<_>>() {
        let id = PlayerId(p.0 as u8);
        let seed = salted(legatum_wasm::session::brain_seed(setup.seed, id), salt);
        let (b, n) = if id.0 == seat {
            brain_for(seat_kind, &state, id, seed)
        } else {
            brain_for(others, &state, id, seed)
        };
        brains.push((id, b));
        names.push(n);
    }
    let cap = legatum_core::data::EXPECTED_GAME_TURNS * 4;
    let mut c = Collector::new(setup.players.len());
    while !state.finished && state.turn < cap {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&legatum_core::player_view(&state, *id)));
            }
        }
        c.orders(&orders);
        let turn = state.turn;
        let (next, events) = legatum_core::resolve_turn(&state, &orders);
        state = next;
        c.events(turn, &state, &events);
        c.resolved(turn, &state);
    }
    let mut out = c.report(&state, &names);
    out["seed"] = json!(setup.seed.to_string());
    out["salt"] = json!(salt);
    out["focus_seat"] = json!(seat);
    out["focus_brain"] = json!(names[seat as usize]);
    out["rules"] = serde_json::to_value(setup.rule_experiments).unwrap();
    out
}

/// legatum-sim's setup_for_game plus proxy rotation (crates/sim/src/runner.rs, proxy_benchmark.rs).
fn sim_setup(
    seed: u64,
    index: usize,
    r: legatum_core::experiments::RuleExperiments,
) -> (GameSetup, u8) {
    let mut setup = GameSetup::all_ai(seed);
    let mut factions = Faction::ALL;
    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (player, faction) in setup.players.iter_mut().zip(factions) {
        player.faction = faction;
        player.difficulty = AiDifficulty::Hard;
    }
    let faction = Faction::ALL[(index % 8 + index / 8) % Faction::ALL.len()];
    let source = setup.players.iter().position(|p| p.faction == faction);
    let seat = index % 8;
    let old = setup.players[seat].faction;
    setup.players[seat].faction = faction;
    if let Some(source) = source {
        setup.players[source].faction = old;
    }
    setup.rule_experiments = r;
    setup.barbarians = true;
    setup.turn_limit = None;
    (setup, seat as u8)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "replay" => {
            let (_, rec) = load_save(&a[2]);
            let setup = rec.tape.setup.clone();
            let mut state = legatum_core::new_game(&setup);
            assert_eq!(state.hash(), rec.tape.initial_hash, "initial hash");
            let mut c = Collector::new(setup.players.len());
            for t in &rec.tape.turns {
                c.orders(&t.orders);
                let (next, events) = legatum_core::resolve_turn(&state, &t.orders);
                state = next;
                assert_eq!(state.hash().to_string(), t.hash, "hash at turn {}", t.turn);
                c.events(t.turn, &state, &events);
                c.resolved(t.turn, &state);
            }
            let names: Vec<String> = setup
                .players
                .iter()
                .map(|p| {
                    if p.is_human {
                        "human".into()
                    } else {
                        format!("scripted-{:?}", p.difficulty).to_lowercase()
                    }
                })
                .collect();
            let mut out = c.report(&state, &names);
            out["seed"] = json!(setup.seed.to_string());
            out["focus_seat"] = json!(0);
            out["focus_brain"] = json!("human");
            println!("{out}");
        }
        "play" => {
            let (_, rec) = load_save(&a[2]);
            let mut setup = rec.tape.setup.clone();
            setup.players[0].is_human = false;
            setup.players[0].difficulty = AiDifficulty::Hard;
            setup.rule_experiments = rules(a.get(5));
            let salt: u64 = a[4].parse().unwrap();
            // Same map as the playtest: terrain and starts depend only on the seed and size.
            let played = legatum_core::new_game(&setup);
            let human = legatum_core::new_game(&rec.tape.setup);
            assert_eq!(
                serde_json::to_string(&played.map).unwrap(),
                serde_json::to_string(&human.map).unwrap(),
                "map differs from the playtest"
            );
            println!("{}", play(&setup, 0, &a[3], salt));
        }
        "salts" => {
            let (_, rec) = load_save(&a[2]);
            let mut setup = rec.tape.setup.clone();
            setup.players[0].is_human = false;
            setup.players[0].difficulty = AiDifficulty::Hard;
            setup.rule_experiments = rules(a.get(6));
            let salts: u64 = a[5].parse().unwrap();
            let lines: Vec<String> = (1..=salts)
                .into_par_iter()
                .map(|salt| play_vs(&setup, 0, &a[3], &a[4], salt).to_string())
                .collect();
            for l in lines {
                println!("{l}");
            }
        }
        "maps" => {
            let first: u64 = a[2].parse().unwrap();
            let games: usize = a[3].parse().unwrap();
            let kind = a[4].clone();
            let r = rules(a.get(5));
            let lines: Vec<String> = (0..games)
                .into_par_iter()
                .map(|i| {
                    let (setup, seat) = sim_setup(first + i as u64, i, r);
                    let mut v = play(&setup, seat, &kind, 0);
                    v["index"] = json!(i);
                    v.to_string()
                })
                .collect();
            for l in lines {
                println!("{l}");
            }
        }
        other => panic!("unknown mode {other}"),
    }
}

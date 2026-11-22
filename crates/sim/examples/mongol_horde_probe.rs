//! Mongol playbook diagnostic: Keshig production, upgrades, fire, charges and captures.
//!
//!   mongol_keshig tape TAPE.json                 his recorded game (every Mongol seat)
//!   mongol_keshig census GAMES [first_seed]      eight-seat all-Hard AI-only games (unit_census setup)
//!   mongol_keshig hismap TAPE.json SALTS [cap]   his setup, every seat Hard, brains and dice salted
//!
//! Prints one JSON line per Mongol seat per game.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::events::{Attacker, Defender};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, BuildingKind, Event, Faction, GameSetup, GameState, PlayerId, ProductionItem,
    TeamId, TechId, UnitKind, data, player_view, resolve_turn,
};
use legatum_wasm::tape::Tape;
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const K: UnitKind = data::KESHIG_KIND;

#[derive(Default, Clone)]
struct Seat {
    m: BTreeMap<String, i64>,
    army: BTreeMap<String, Value>,
    first: BTreeMap<String, u32>,
}
impl Seat {
    fn add(&mut self, k: &str, v: i64) {
        *self.m.entry(k.into()).or_default() += v;
    }
    fn first(&mut self, k: &str, t: u32) {
        self.first.entry(k.into()).or_insert(t);
    }
}

fn is_mil(k: UnitKind) -> bool {
    data::unit_stats(k).military
}

fn nearest_enemy(s: &GameState, owner: PlayerId, at: legatum_core::Hex) -> i32 {
    let team = s.player(owner).team;
    s.units
        .values()
        .filter(|u| u.owner != owner && is_mil(u.kind))
        .filter(|u| u.owner.is_barbarian() || s.player(u.owner).team != team)
        .map(|u| u.pos.distance(at))
        .min()
        .unwrap_or(99)
}

fn step(pre: &GameState, post: &GameState, events: &[Event], seats: &mut BTreeMap<u8, Seat>) {
    let turn = pre.turn;
    // Keshig battles this turn, by unit.
    let mut fought: BTreeMap<u64, bool> = BTreeMap::new();
    for e in events {
        match e {
            Event::Battle {
                attacker: Attacker::Unit(u),
                attacker_owner,
                defender,
                attacker_at,
                damage_to_defender,
                damage_to_attacker,
                defender_hp_after,
                ranged,
                ..
            } => {
                let Some(s) = seats.get_mut(&attacker_owner.0) else {
                    continue;
                };
                let Some(unit) = pre.units.get(u) else {
                    continue;
                };
                let kind = unit.kind;
                let mode = if *ranged { "shot" } else { "charge" };
                let tgt = match defender {
                    Defender::Unit { .. } => "unit",
                    Defender::City { .. } => "city",
                };
                if kind == K {
                    fought.insert(u.0 as u64, *ranged);
                    s.add(&format!("keshig_{mode}_{tgt}"), 1);
                    if *ranged
                        && matches!(defender, Defender::City { city, .. }
                        if pre.cities.get(city).is_some_and(|c|
                            data::has_building(&c.buildings, BuildingKind::Walls)))
                    {
                        s.add("keshig_shot_wall", 1);
                    }
                    s.add(&format!("keshig_{mode}_dmg"), *damage_to_defender as i64);
                    s.add(
                        &format!("keshig_{mode}_dmg_taken"),
                        *damage_to_attacker as i64,
                    );
                    s.first(&format!("first_keshig_{mode}"), turn);
                    if let Defender::Unit { .. } = defender
                        && *defender_hp_after <= 0
                    {
                        s.add(&format!("keshig_{mode}_kills"), 1);
                    }
                    if *ranged {
                        let tpos = match defender {
                            Defender::Unit { unit, .. } => pre.units.get(unit).map(|x| x.pos),
                            Defender::City { city, .. } => pre.cities.get(city).map(|c| c.pos),
                        };
                        if let (Some(a), Some(t)) = (attacker_at.or(Some(unit.pos)), tpos) {
                            s.add(&format!("keshig_shot_range{}", a.distance(t)), 1);
                        }
                    }
                } else {
                    let class = data::unit_class(kind)
                        .map(|c| format!("{c:?}").to_lowercase())
                        .unwrap_or("other".into());
                    s.add(&format!("other_{class}_attacks"), 1);
                    if let Defender::Unit { .. } = defender
                        && *defender_hp_after <= 0
                    {
                        s.add(&format!("other_{class}_kills"), 1);
                    }
                }
                if let Defender::Unit { .. } = defender
                    && *defender_hp_after <= 0
                {
                    s.add("kills", 1);
                }
            }
            Event::ProductionCompleted {
                owner,
                item: ProductionItem::Unit { unit },
                ..
            } => {
                if let Some(s) = seats.get_mut(&owner.0)
                    && is_mil(*unit)
                {
                    s.add(&format!("built_{unit:?}"), 1);
                    s.first(&format!("first_built_{unit:?}"), turn);
                    s.add("built_military", 1);
                }
            }
            Event::ItemBought {
                owner,
                item: ProductionItem::Unit { unit },
                ..
            } => {
                if let Some(s) = seats.get_mut(&owner.0) {
                    s.add(&format!("bought_{unit:?}"), 1);
                }
            }
            Event::UnitUpgraded {
                owner, to, unit, ..
            } => {
                if let Some(s) = seats.get_mut(&owner.0) {
                    let from = pre.units.get(unit).map(|u| format!("{:?}", u.kind));
                    s.add(&format!("upg_{}_to_{to:?}", from.unwrap_or("?".into())), 1);
                    if *to == K {
                        s.first("first_upg_to_keshig", turn);
                    }
                }
            }
            Event::UnitDestroyed {
                owner, unit_kind, ..
            } => {
                if let Some(s) = seats.get_mut(&owner.0)
                    && is_mil(*unit_kind)
                {
                    s.add("lost_military", 1);
                    if *unit_kind == K {
                        s.add("keshig_lost", 1);
                    }
                }
            }
            Event::TechResearched { player, tech } if *tech == TechId::IronWorking => {
                if let Some(s) = seats.get_mut(&player.0) {
                    s.first("iron_working", turn);
                }
            }
            Event::CityCaptured { city, from, to, .. } => {
                if let Some(s) = seats.get_mut(&to.0) {
                    s.add("captures", 1);
                    s.first("first_capture", turn);
                    let pos = post.cities.get(city).map(|c| c.pos);
                    let by = post
                        .units
                        .values()
                        .find(|u| Some(u.pos) == pos && u.owner == *to && is_mil(u.kind))
                        .map(|u| u.kind);
                    s.add(
                        &format!("capture_by_{}", by.map_or("?".into(), |k| format!("{k:?}"))),
                        1,
                    );
                    if turn <= 100 {
                        s.add("captures_by_100", 1);
                    }
                }
                if let Some(s) = seats.get_mut(&from.0) {
                    s.add("cities_lost", 1);
                }
            }
            Event::WarDeclared { relation } => {
                for (&p, s) in seats.iter_mut() {
                    let team = pre.player(PlayerId(p)).team;
                    if relation.declarers.contains(&team) {
                        s.add("wars_declared", 1);
                        s.first("first_war_declared", turn);
                    }
                }
            }
            _ => {}
        }
    }
    // Per Keshig: what it did this turn.
    for (&p, s) in seats.iter_mut() {
        let pid = PlayerId(p);
        for u in pre.units.values().filter(|u| u.owner == pid && u.kind == K) {
            s.add("keshig_turns", 1);
            let after = post.units.get(&u.id);
            let wounded = u.hp < data::UNIT_HP * 60 / 100;
            let near_before = nearest_enemy(pre, pid, u.pos);
            if near_before <= 3 {
                s.add("keshig_turns_contact", 1);
            }
            match fought.get(&(u.id.0 as u64)) {
                Some(true) => s.add("keshig_act_shoot", 1),
                Some(false) => s.add("keshig_act_charge", 1),
                None => match after {
                    Some(a) if a.pos != u.pos => {
                        s.add("keshig_act_move", 1);
                        if near_before <= 3 {
                            s.add("keshig_act_move_in_contact", 1);
                        }
                        if wounded && near_before <= 3 {
                            let near_after = nearest_enemy(post, pid, a.pos);
                            s.add(
                                if near_after > near_before {
                                    "keshig_wounded_withdrew"
                                } else {
                                    "keshig_wounded_moved_closer_or_same"
                                },
                                1,
                            );
                        }
                    }
                    Some(_) => {
                        s.add("keshig_act_idle", 1);
                        if near_before <= 3 {
                            s.add("keshig_act_idle_in_contact", 1);
                        }
                    }
                    None => s.add("keshig_act_died", 1),
                },
            }
            if wounded && near_before <= 3 {
                s.add("keshig_wounded_in_contact", 1);
            }
        }
    }
    // Snapshots.
    if [37u32, 50, 75, 84, 100, 125, 150].contains(&post.turn) {
        for (&p, s) in seats.iter_mut() {
            let pid = PlayerId(p);
            let mut by: BTreeMap<String, i64> = BTreeMap::new();
            for u in post
                .units
                .values()
                .filter(|u| u.owner == pid && is_mil(u.kind))
            {
                *by.entry(format!("{:?}", u.kind)).or_default() += 1;
            }
            let cities = post.cities.values().filter(|c| c.owner == pid).count();
            s.army.insert(
                format!("T{}", post.turn),
                json!({"units": by, "cities": cities,
                       "vp": legatum_core::scoring::vp(post, pid).iter().map(|x| x.1).sum::<u32>(),
                       "gold": post.player(pid).gold}),
            );
        }
    }
}

fn mongol_seats(s: &GameState) -> BTreeMap<u8, Seat> {
    s.players
        .iter()
        .filter(|p| p.faction == Faction::Mongols)
        .map(|p| (p.id.0, Seat::default()))
        .collect()
}

fn rows(game: &str, s: &GameState, seats: BTreeMap<u8, Seat>) -> Vec<Value> {
    let winner_faction = s
        .winner
        .and_then(|team| s.players.iter().find(|p| p.team == team).map(|p| p.faction));
    let factions: Vec<_> = s.players.iter().map(|p| p.faction).collect();
    seats
        .into_iter()
        .map(|(p, seat)| {
            let pl = s.player(PlayerId(p));
            json!({"game": game, "seat": p, "end_turn": s.turn, "finished": s.finished,
                   "winner_faction": winner_faction, "factions": factions.clone(),
                   "won": s.winner.map(|w| w == pl.team).unwrap_or(false),
                   "alive": pl.alive, "m": seat.m, "first": seat.first, "snaps": seat.army})
        })
        .collect()
}

fn play_ai(
    game: String,
    mut setup: GameSetup,
    salt: u64,
    dice: Option<u64>,
    cap: u32,
    params: Option<&AiParams>,
) -> Vec<Value> {
    for p in setup.players.iter_mut() {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
    }
    setup.turn_limit = None;
    let mut state = legatum_core::new_game(&setup);
    state.dice_seed = dice;
    let n = setup.players.len() as u8;
    let mut brains: Vec<(PlayerId, ScriptedBrain)> = (0..n)
        .map(|i| {
            let id = PlayerId(i);
            let faction = state.player(id).faction;
            let seed =
                legatum_wasm::session::brain_seed(setup.seed ^ salt.wrapping_mul(0x9E37_79B9), id);
            let b = params.map_or_else(
                || ScriptedBrain::with_difficulty(faction, seed, id, AiDifficulty::Hard),
                |p| ScriptedBrain::with_params(faction, seed, id, AiDifficulty::Hard, p),
            );
            (id, b)
        })
        .collect();
    let mut seats = mongol_seats(&state);
    while !state.finished && state.turn < cap {
        let mut orders = legatum_wasm::session::OrderMap::new();
        for (id, b) in brains.iter_mut() {
            if state.player(*id).alive {
                orders.insert(*id, b.orders(&player_view(&state, *id)));
            }
        }
        let (next, events) = resolve_turn(&state, &orders);
        step(&state, &next, &events, &mut seats);
        state = next;
    }
    rows(&game, &state, seats)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    if a.iter().any(|a| a == "--games") {
        let mut games = 0_u64;
        let mut first = 0_u64;
        let mut label = String::from("mongol-horde");
        let mut out_dir = String::from(".");
        let mut params_path = None;
        let mut i = 1;
        while i + 1 < a.len() {
            let value = &a[i + 1];
            match a[i].as_str() {
                "--games" => games = value.parse()?,
                "--seed" => first = value.parse()?,
                "--label" => label = value.clone(),
                "--out-dir" => out_dir = value.clone(),
                "--ai-params" => params_path = Some(value.clone()),
                "--out" | "--status-file" => {}
                other => return Err(format!("unknown flag {other}").into()),
            }
            i += 2;
        }
        let params: Option<AiParams> = params_path
            .map(|path| -> Result<_, Box<dyn std::error::Error>> {
                let parsed: AiParams = serde_json::from_str(&std::fs::read_to_string(path)?)?;
                parsed.validate()?;
                Ok(parsed)
            })
            .transpose()?;
        let rows: Vec<Vec<Value>> = (0..games)
            .into_par_iter()
            .map(|game| {
                let seed = first + game;
                let mut setup = GameSetup::all_ai(seed);
                setup.players.truncate(8);
                (setup.width, setup.height) = (data::MAP_WIDTH, data::MAP_HEIGHT);
                let mut factions = Faction::ALL;
                let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
                for i in (1..factions.len()).rev() {
                    let j = rng.range(0, i as i64) as usize;
                    factions.swap(i, j);
                }
                for (seat, (player, faction)) in setup.players.iter_mut().zip(factions).enumerate()
                {
                    player.faction = faction;
                    player.team = TeamId(seat as u8);
                }
                if !setup.players.iter().any(|p| p.faction == Faction::Mongols) {
                    let mongols = factions
                        .iter()
                        .position(|f| *f == Faction::Mongols)
                        .unwrap();
                    setup.players[0].faction = factions[mongols];
                }
                play_ai(
                    format!("{label}-{seed}"),
                    setup,
                    0,
                    None,
                    data::EXPECTED_GAME_TURNS * 4,
                    params.as_ref(),
                )
            })
            .collect();
        std::fs::create_dir_all(&out_dir)?;
        let file = std::fs::File::create(format!("{out_dir}/mongol-horde.jsonl"))?;
        let mut writer = std::io::BufWriter::new(file);
        use std::io::Write;
        for row in rows.into_iter().flatten() {
            writeln!(writer, "{row}")?;
        }
        return Ok(());
    }
    match a[1].as_str() {
        "tape" => {
            let tape = Tape::from_json(&std::fs::read_to_string(&a[2])?)?;
            let mut state = tape.initial_state();
            let mut seats = mongol_seats(&state);
            let mut mism = 0;
            for t in &tape.turns {
                let (next, events) = resolve_turn(&state, &t.orders);
                if next.hash().to_string() != t.hash {
                    mism += 1;
                }
                step(&state, &next, &events, &mut seats);
                state = next;
            }
            eprintln!("hash mismatches {mism}");
            for r in rows("his-tape", &state, seats) {
                println!("{r}");
            }
        }
        "census" => {
            let games: u64 = a[2].parse()?;
            let first: u64 = a.get(3).map_or(9_311_003, |s| s.parse().unwrap());
            let out: Vec<Vec<Value>> = (0..games)
                .into_par_iter()
                .map(|i| {
                    let seed = first + i;
                    let mut setup = GameSetup::all_ai(seed);
                    setup.players.truncate(8);
                    (setup.width, setup.height) = (data::MAP_WIDTH, data::MAP_HEIGHT);
                    let mut factions = Faction::ALL;
                    let mut rng = Rng::derive(seed, &[purpose::SETUP, 8]);
                    for i in (1..factions.len()).rev() {
                        let j = rng.range(0, i as i64) as usize;
                        factions.swap(i, j);
                    }
                    for (seat, (p, f)) in setup.players.iter_mut().zip(factions).enumerate() {
                        p.faction = f;
                        p.team = TeamId(seat as u8);
                    }
                    if !setup.players.iter().any(|p| p.faction == Faction::Mongols) {
                        return vec![];
                    }
                    play_ai(
                        format!("c{seed}"),
                        setup,
                        0,
                        None,
                        data::EXPECTED_GAME_TURNS * 4,
                        None,
                    )
                })
                .collect();
            for v in out.into_iter().flatten() {
                println!("{v}");
            }
        }
        "hismap" => {
            let tape = Tape::from_json(&std::fs::read_to_string(&a[2])?)?;
            let salts: u64 = a[3].parse()?;
            let cap: u32 = a.get(4).map_or(400, |s| s.parse().unwrap());
            let setup = tape.setup.clone();
            let out: Vec<Vec<Value>> = (1..=salts)
                .into_par_iter()
                .map(|s| {
                    play_ai(
                        format!("h{s}"),
                        setup.clone(),
                        s,
                        Some(setup.seed.wrapping_add(s * 7919)),
                        cap,
                        None,
                    )
                })
                .collect();
            for v in out.into_iter().flatten() {
                println!("{v}");
            }
        }
        _ => panic!("mode"),
    }
    Ok(())
}

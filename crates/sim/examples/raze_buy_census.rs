//! All-AI census of captures, razes and Gold purchases (decision audit follow-up, 2026-10-31).
//! Run on the cloud box, never a long local run.
//!
//!   raze_buy_census --games 256 --seed 7300001 [--params FILE] [--threads 60] [--out FILE.json]
//!
//! Plays eight-empire Hard games (the `legatum-sim` lobby: Standard map, one seat per faction,
//! barbarians on), every seat a `ScriptedBrain::with_params` with FILE (default parameters
//! without it), and records from the engine's own events and the seats' final orders:
//! captures, ordered razes and Celtic sacks; every purchase with its price, the city's
//! Production, the turns the city would need to build it, war, and the planning pass that
//! ordered it (`legatum_ai::buy_trace`); upgrades; Gold and cities at T100/T150/T200; the
//! winner; and the seats' planning time. Prints a summary and writes the per-game rows.
use legatum_ai::{AiParams, Brain, ScriptedBrain, buy_trace};
use legatum_core::{
    AiDifficulty, Event, Faction, GameSetup, Order, PlayerId, ProductionItem, economy,
};
use legatum_wasm::session::{BrainKind, Session, brain_seed};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Instant;

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == name)
        .map_or_else(|| default.into(), |w| w[1].clone())
}

const SNAP: [u32; 3] = [100, 150, 200];
const CAP: u32 = 450;
/// Turns after entering an era that count as its unit spike.
const SPIKE_TURNS: u32 = 10;

fn class(item: ProductionItem) -> &'static str {
    match item {
        ProductionItem::Unit {
            unit: legatum_core::UnitKind::Settler,
        } => "settler",
        ProductionItem::Unit { unit } if legatum_core::data::unit_stats(unit).military => {
            "military"
        }
        ProductionItem::Unit { .. } => "civilian",
        ProductionItem::Building { .. } => "building",
        ProductionItem::Improvement { .. } => "improvement",
        ProductionItem::Wonder { .. } => "wonder",
        ProductionItem::Project { .. } => "project",
    }
}

fn lineage_index(unit: legatum_core::UnitKind) -> Option<(&'static str, usize)> {
    use legatum_core::UnitKind as U;
    const MELEE: [U; 9] = [
        U::HunterKiller,
        U::MechanisedInfantry,
        U::Infantry,
        U::TrenchInfantry,
        U::Rifleman,
        U::AssaultTrooper,
        U::Trooper,
        U::Phalangite,
        U::Soldier,
    ];
    const RANGED: [U; 9] = [
        U::Drone,
        U::RocketArtillery,
        U::Artillery,
        U::Howitzer,
        U::FieldCannon,
        U::Arquebusier,
        U::Marksman,
        U::CompositeArcher,
        U::Ranger,
    ];
    const MOUNTED: [U; 9] = [
        U::AutonomousArmour,
        U::MainBattleTank,
        U::Tank,
        U::Landship,
        U::Cavalry,
        U::Cuirassier,
        U::Knight,
        U::Cataphract,
        U::Horseman,
    ];
    for (name, line) in [("melee", MELEE), ("ranged", RANGED), ("mounted", MOUNTED)] {
        if let Some(i) = line.iter().position(|u| *u == unit) {
            return Some((name, i));
        }
    }
    None
}

#[derive(Default, Clone, Copy)]
struct Tally {
    n: u64,
    gold: i64,
    turns: i64,
    war: u64,
}

impl Tally {
    fn add(&mut self, gold: i32, turns: i32, war: bool) {
        self.n += 1;
        self.gold += i64::from(gold);
        self.turns += i64::from(turns);
        self.war += u64::from(war);
    }
    fn json(&self) -> Value {
        json!({"n": self.n, "gold": self.gold, "turns": self.turns, "war": self.war})
    }
}

fn play(seed: u64, params: &AiParams) -> Value {
    // The `legatum-sim --players 8 --difficulty hard` lobby (`runner::setup_for_game`).
    let mut setup = GameSetup::all_ai(seed);
    let mut factions = Faction::ALL;
    let mut rng = legatum_core::rng::Rng::derive(seed, &[legatum_core::rng::purpose::SETUP, 8]);
    for i in (1..factions.len()).rev() {
        let j = rng.range(0, i as i64) as usize;
        factions.swap(i, j);
    }
    for (player, faction) in setup.players.iter_mut().zip(factions) {
        player.faction = faction;
        player.difficulty = AiDifficulty::Hard;
    }
    setup.barbarians = true;
    (setup.width, setup.height) = (48, 32);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    for p in session.state.players.clone() {
        let brain: Box<dyn Brain> = Box::new(ScriptedBrain::with_params(
            p.faction,
            brain_seed(setup.seed, p.id),
            p.id,
            AiDifficulty::Hard,
            params,
        ));
        session.replace_brain(p.id, brain);
    }
    let seats = session.state.players.len();
    let mut by_pass: BTreeMap<String, Tally> = BTreeMap::new();
    let mut by_turn: BTreeMap<String, Tally> = BTreeMap::new();
    let mut seat_buys = vec![Tally::default(); seats];
    let mut captures = vec![0u32; seats];
    let mut razes = vec![0u32; seats];
    let mut sacks = vec![0u32; seats];
    let mut lost = vec![0u32; seats];
    let mut upgrades = vec![(0u32, 0i64); seats];
    let mut spike = vec![(0i64, 0i64); seats]; // (military buy + upgrade Gold in spikes, outside)
    let mut late_era_buys = (0u64, 0u64); // (two or more eras behind, all military purchases after T90)
    let mut late_generation_buys = (0u64, 0u64);
    let mut late_generation_rescues = 0u64;
    let mut late_generation_details = Vec::new();
    let mut era_turn = vec![0u32; seats];
    let mut snaps: Vec<Vec<Option<(i32, usize)>>> = vec![vec![None; SNAP.len()]; seats];
    let mut ms = 0.0f64;
    let mut seat_turns = 0u64;
    buy_trace::enable();
    while !session.state.finished && session.state.turn < CAP {
        let turn = session.state.turn;
        for (seat, p) in session.state.players.iter().enumerate() {
            if let Some(i) = SNAP.iter().position(|t| *t == turn) {
                snaps[seat][i] = p
                    .alive
                    .then(|| (p.gold, session.state.cities_of(p.id).count()));
            }
        }
        let start = Instant::now();
        let orders = session.ai_orders();
        ms += start.elapsed().as_secs_f64() * 1000.0;
        seat_turns += orders.len() as u64;
        let tags = buy_trace::take();
        let state = &session.state;
        for (id, list) in &orders {
            let seat = id.0 as usize;
            let view = legatum_core::player_view(state, *id);
            let best_trainable = view
                .cities
                .iter()
                .filter(|c| c.owner == *id)
                .filter_map(|c| c.detail.as_ref())
                .flat_map(|d| &d.options)
                .filter(|o| o.resource_blocker.is_none())
                .filter_map(|o| match o.item {
                    ProductionItem::Unit { unit }
                        if legatum_core::data::unit_stats(unit).military =>
                    {
                        Some(legatum_core::data::unit_era_tier(unit))
                    }
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            let at_war = state
                .players
                .iter()
                .any(|o| o.alive && o.team != state.player(*id).team && state.at_war(*id, o.id));
            for o in list {
                let Order::BuyItem { city, item } = o else {
                    continue;
                };
                let Some(c) = state.cities.get(city) else {
                    continue;
                };
                let Some(gold) = economy::buy_cost_for(state, c, *item) else {
                    continue;
                };
                let cost = economy::production_cost(state, *id, *item);
                let prod = economy::city_yields(state, c).total.production.max(1);
                let turns = (cost + prod - 1) / prod;
                let tag = tags
                    .iter()
                    .find(|t| t.player == *id && t.city == *city && t.item == *item);
                let pass = tag.map_or("untagged", |t| t.pass);
                let k = class(*item);
                by_pass
                    .entry(format!("{pass}/{k}"))
                    .or_default()
                    .add(gold, turns, at_war);
                let bucket = (turn / 50) * 50;
                by_turn
                    .entry(format!("{k}/T{bucket:03}"))
                    .or_default()
                    .add(gold, turns, at_war);
                seat_buys[seat].add(gold, turns, at_war);
                if k == "military" {
                    if turn > 90 {
                        late_era_buys.1 += 1;
                        if let ProductionItem::Unit { unit } = item
                            && legatum_core::data::unit_era_tier(*unit).saturating_add(2)
                                <= best_trainable
                        {
                            late_era_buys.0 += 1;
                        }
                        if let ProductionItem::Unit { unit } = item
                            && let Some((line, index)) = lineage_index(*unit)
                        {
                            let best = view
                                .cities
                                .iter()
                                .find(|home| home.id == *city)
                                .and_then(|home| home.detail.as_ref())
                                .into_iter()
                                .flat_map(|d| &d.options)
                                .filter(|option| option.resource_blocker.is_none())
                                .filter_map(|option| match option.item {
                                    ProductionItem::Unit { unit } => lineage_index(unit)
                                        .and_then(|(other, i)| (other == line).then_some(i)),
                                    _ => None,
                                })
                                .min();
                            if let Some(best) = best {
                                late_generation_buys.1 += 1;
                                if index >= best + 2 {
                                    late_generation_buys.0 += 1;
                                    late_generation_rescues +=
                                        u64::from(tag.is_some_and(|t| t.floor_rescue));
                                    late_generation_details.push(json!({"turn":turn,"player":id.0,"city":city.0,"unit":unit,"steps":index-best,"pass":pass,"floor_rescue":tag.is_some_and(|t| t.floor_rescue)}));
                                }
                            }
                        }
                    }
                    if era_turn[seat] > 0 && turn <= era_turn[seat] + SPIKE_TURNS {
                        spike[seat].0 += i64::from(gold);
                    } else {
                        spike[seat].1 += i64::from(gold);
                    }
                }
            }
        }
        let events = session.step(&orders);
        let turn_now = session.state.turn;
        for e in &events {
            match e {
                Event::CityCaptured { from, to, .. } => {
                    if !to.is_barbarian() {
                        captures[to.0 as usize] += 1;
                    }
                    if !from.is_barbarian() {
                        lost[from.0 as usize] += 1;
                    }
                }
                Event::CityRazed {
                    by, from, ordered, ..
                } => {
                    if by.is_barbarian() {
                        continue;
                    }
                    if *ordered {
                        razes[by.0 as usize] += 1;
                    } else {
                        sacks[by.0 as usize] += 1;
                        if !from.is_barbarian() {
                            lost[from.0 as usize] += 1;
                        }
                    }
                }
                Event::UnitUpgraded { owner, gold, .. } if !owner.is_barbarian() => {
                    let s = owner.0 as usize;
                    upgrades[s].0 += 1;
                    upgrades[s].1 += i64::from(*gold);
                    if era_turn[s] > 0 && turn <= era_turn[s] + SPIKE_TURNS {
                        spike[s].0 += i64::from(*gold);
                    } else {
                        spike[s].1 += i64::from(*gold);
                    }
                }
                Event::EraEntered { player, .. } if !player.is_barbarian() => {
                    era_turn[player.0 as usize] = turn_now;
                }
                _ => {}
            }
        }
    }
    let state = &session.state;
    let winners: Vec<Faction> = state
        .winner
        .map(|t| {
            state
                .players
                .iter()
                .filter(|p| p.team == t)
                .map(|p| p.faction)
                .collect()
        })
        .unwrap_or_default();
    let seat_rows: Vec<Value> = (0..seats)
        .map(|s| {
            let p = state.player(PlayerId(s as u8));
            json!({
                "faction": p.faction, "won": winners.contains(&p.faction) && state.winner == Some(p.team),
                "captures": captures[s], "razes": razes[s], "sacks": sacks[s], "lost": lost[s],
                "buys": seat_buys[s].json(), "upgrades": upgrades[s].0, "upgrade_gold": upgrades[s].1,
                "spike_gold": spike[s].0, "other_military_gold": spike[s].1,
                "snaps": snaps[s].iter().map(|x| x.map(|(g, c)| json!([g, c]))).collect::<Vec<_>>(),
                "final_cities": state.cities_of(p.id).count(),
            })
        })
        .collect();
    json!({
        "seed": seed, "turns": state.turn, "finished": state.finished,
        "ms": ms, "seat_turns": seat_turns, "seats": seat_rows,
        "late_obsolete_buys": late_era_buys.0, "late_military_buys": late_era_buys.1,
        "late_generation_buys": late_generation_buys.0, "late_lineage_buys": late_generation_buys.1,
        "late_generation_rescues": late_generation_rescues, "late_generation_details": late_generation_details,
        "by_pass": by_pass.iter().map(|(k, v)| (k.clone(), v.json())).collect::<BTreeMap<_, _>>(),
        "by_turn": by_turn.iter().map(|(k, v)| (k.clone(), v.json())).collect::<BTreeMap<_, _>>(),
    })
}

fn median(mut v: Vec<i64>) -> i64 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    v[v.len() / 2]
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let games: u64 = arg(&args, "--games", "256").parse().unwrap();
    let seed0: u64 = arg(&args, "--seed", "7300001").parse().unwrap();
    let threads: usize = arg(&args, "--threads", "0").parse().unwrap();
    let out = arg(&args, "--out", "");
    let out = if !out.is_empty() && std::path::Path::new(&out).is_dir() {
        std::path::Path::new(&out)
            .join("raze-buy-census.json")
            .to_string_lossy()
            .into_owned()
    } else {
        out
    };
    let params_path = arg(&args, "--params", "");
    let params_path = if params_path.is_empty() {
        arg(&args, "--ai-params", "")
    } else {
        params_path
    };
    let params = match params_path.as_str() {
        "" => AiParams::default(),
        f => {
            let p: AiParams =
                serde_json::from_str(&std::fs::read_to_string(f).expect("read params"))
                    .expect("params JSON");
            p.validate().expect("valid params");
            p
        }
    };
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    let rows: Vec<Value> = (0..games)
        .into_par_iter()
        .map(|i| play(seed0 + i, &params))
        .collect();
    // Summary.
    let n = rows.len() as f64;
    let obsolete: u64 = rows
        .iter()
        .map(|r| r["late_obsolete_buys"].as_u64().unwrap())
        .sum();
    let military: u64 = rows
        .iter()
        .map(|r| r["late_military_buys"].as_u64().unwrap())
        .sum();
    println!(
        "after T90, two or more eras obsolete: {obsolete}/{military} ({:.1}%)",
        100.0 * obsolete as f64 / military.max(1) as f64
    );
    let generation: u64 = rows
        .iter()
        .map(|r| r["late_generation_buys"].as_u64().unwrap())
        .sum();
    let lineage: u64 = rows
        .iter()
        .map(|r| r["late_lineage_buys"].as_u64().unwrap())
        .sum();
    let rescues: u64 = rows
        .iter()
        .map(|r| r["late_generation_rescues"].as_u64().unwrap())
        .sum();
    println!(
        "after T90, two or more generations obsolete: {generation}/{lineage} ({:.1}%)",
        100.0 * generation as f64 / lineage.max(1) as f64
    );
    println!("of those, empty-city forecast rescues: {rescues}");
    let seats: Vec<&Value> = rows
        .iter()
        .flat_map(|r| r["seats"].as_array().unwrap())
        .collect();
    let sum = |k: &str| {
        seats
            .iter()
            .map(|s| s[k].as_i64().unwrap_or(0))
            .sum::<i64>()
    };
    let captures = sum("captures");
    let razes = sum("razes");
    let sacks = sum("sacks");
    let buys_n: i64 = seats.iter().map(|s| s["buys"]["n"].as_i64().unwrap()).sum();
    let buys_gold: i64 = seats
        .iter()
        .map(|s| s["buys"]["gold"].as_i64().unwrap())
        .sum();
    println!(
        "games {} | turns mean {:.1} | captures/game {:.2} | ordered razes/game {:.2} (kept {:.1}% of non-Celtic captures) | Celtic sacks/game {:.2}",
        rows.len(),
        rows.iter()
            .map(|r| r["turns"].as_f64().unwrap())
            .sum::<f64>()
            / n,
        captures as f64 / n,
        razes as f64 / n,
        100.0 * (captures - razes) as f64 / captures.max(1) as f64,
        sacks as f64 / n
    );
    println!(
        "buys/seat {:.1} | buy Gold/seat {:.0} | upgrades/seat {:.1} | upgrade Gold/seat {:.0} | spike Gold/seat {:.0} | other military Gold/seat {:.0}",
        buys_n as f64 / seats.len() as f64,
        buys_gold as f64 / seats.len() as f64,
        sum("upgrades") as f64 / seats.len() as f64,
        sum("upgrade_gold") as f64 / seats.len() as f64,
        sum("spike_gold") as f64 / seats.len() as f64,
        sum("other_military_gold") as f64 / seats.len() as f64,
    );
    for (i, t) in SNAP.iter().enumerate() {
        let gold: Vec<i64> = seats
            .iter()
            .filter_map(|s| s["snaps"][i].as_array().map(|a| a[0].as_i64().unwrap()))
            .collect();
        let cities: Vec<i64> = seats
            .iter()
            .filter_map(|s| s["snaps"][i].as_array().map(|a| a[1].as_i64().unwrap()))
            .collect();
        println!(
            "T{t}: alive seats {} | Gold median {} mean {:.0} | cities mean {:.2} (total/game {:.1})",
            gold.len(),
            median(gold.clone()),
            gold.iter().sum::<i64>() as f64 / gold.len().max(1) as f64,
            cities.iter().sum::<i64>() as f64 / cities.len().max(1) as f64,
            cities.iter().sum::<i64>() as f64 / n
        );
    }
    let ms: f64 = rows.iter().map(|r| r["ms"].as_f64().unwrap()).sum();
    let st: f64 = rows.iter().map(|r| r["seat_turns"].as_f64().unwrap()).sum();
    println!("planning ms per seat-turn {:.3}", ms / st.max(1.0));
    let mut pass: BTreeMap<String, (i64, i64, i64, i64)> = BTreeMap::new();
    for r in &rows {
        for (k, v) in r["by_pass"].as_object().unwrap() {
            let e = pass.entry(k.clone()).or_default();
            e.0 += v["n"].as_i64().unwrap();
            e.1 += v["gold"].as_i64().unwrap();
            e.2 += v["turns"].as_i64().unwrap();
            e.3 += v["war"].as_i64().unwrap();
        }
    }
    println!("pass/class: buys per game, Gold per game, mean turns to build, % at war");
    let mut pass: Vec<_> = pass.into_iter().collect();
    pass.sort_by_key(|(_, v)| -v.1);
    for (k, (c, g, t, w)) in pass {
        println!(
            "  {k:40} {:7.2} {:8.0} {:6.1} {:5.1}%",
            c as f64 / n,
            g as f64 / n,
            t as f64 / c.max(1) as f64,
            100.0 * w as f64 / c.max(1) as f64
        );
    }
    let mut turn: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for r in &rows {
        for (k, v) in r["by_turn"].as_object().unwrap() {
            let e = turn.entry(k.clone()).or_default();
            e.0 += v["n"].as_i64().unwrap();
            e.1 += v["gold"].as_i64().unwrap();
        }
    }
    println!("class/turn bucket: buys per game, Gold per game");
    for (k, (c, g)) in turn {
        println!("  {k:28} {:7.2} {:8.0}", c as f64 / n, g as f64 / n);
    }
    let mut fac: BTreeMap<String, (u32, u32, i64, i64, i64)> = BTreeMap::new();
    for s in &seats {
        let e = fac
            .entry(s["faction"].as_str().unwrap().into())
            .or_default();
        e.0 += 1;
        e.1 += u32::from(s["won"].as_bool().unwrap());
        e.2 += s["captures"].as_i64().unwrap();
        e.3 += s["razes"].as_i64().unwrap();
        e.4 += s["buys"]["gold"].as_i64().unwrap();
    }
    println!("faction: seats wins captures/seat razes/seat buyGold/seat");
    for (f, (s, w, c, r, g)) in fac {
        println!(
            "  {f:12} {s:4} {w:4} {:6.2} {:6.2} {:7.0}",
            c as f64 / s as f64,
            r as f64 / s as f64,
            g as f64 / s as f64
        );
    }
    if !out.is_empty() {
        std::fs::write(&out, serde_json::to_vec(&rows).unwrap()).unwrap();
    }
}

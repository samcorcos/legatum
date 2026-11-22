//! Designer prediction: how often a decision source would have given the designer's own orders.
//!
//!   designer_predict --tape TAPE.json --states STATES.jsonl [--seat 0] [--source hard]
//!                    [--source designer] [--source params:FILE] [--from 1] [--to 999] [--out FILE.json]
//!
//! `--states` is `audit_states` output (hash-verified original-engine states, one per turn), as
//! `seat_run` and `decision_audit` read it. Every SOURCE gets a shadow brain for his seat that sees
//! his seat's view of the recorded position every turn (its orders discarded, so its memory follows
//! his real game), and on each turn its orders are compared with his recorded orders. A decision of
//! his is predicted when the source makes the same one on the same turn:
//!
//! - production: a city he set to build X (first item of a queue): the source leaves that city
//!   building X (its own order for the city, or the city's current item if it gives none);
//! - research: the technology he set (first of a queue), likewise;
//! - buy: a purchase of his (city and item): the source buys the same item in the same city
//!   (`buy_false` counts the source's purchases he did not make);
//! - war: a war he declared on a team: the source declares on that team the same turn (`near`:
//!   within three turns either side);
//! - found: a Settler he founded a city with: the source founds with it;
//! - move: a military unit he moved: the source moves it to the same tile (`move_near`: within
//!   two tiles); attack: a ranged attack of his: the source shoots the same tile with that unit;
//! - settler: a Settler he moved: the source moves it to the same tile (`settler_near`: two tiles).
//!
//! Prints and writes per-source hit rates by category, by game phase (turns 1-40, 41-100, 101+),
//! and their unweighted mean over the categories he used.

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rules::is_military;
use legatum_core::{AiDifficulty, GameState, Hex, Order, PlayerId, TeamId, UnitKind, player_view};
use legatum_wasm::tape::Tape;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::BufRead;

const CATS: [&str; 8] = [
    "production",
    "research",
    "buy",
    "war",
    "found",
    "move",
    "attack",
    "settler",
];

#[derive(Default, Clone)]
struct Tally {
    /// category -> (hits, his decisions, near hits)
    cats: BTreeMap<&'static str, (u32, u32, u32)>,
    buy_false: u32,
}

impl Tally {
    fn add(&mut self, cat: &'static str, hit: bool, near: bool) {
        let e = self.cats.entry(cat).or_default();
        e.0 += u32::from(hit);
        e.1 += 1;
        e.2 += u32::from(near);
    }
}

fn first_item(o: &Order) -> Option<(u32, String)> {
    match o {
        Order::SetProduction { city, item } => Some((city.0, format!("{item:?}"))),
        Order::SetProductionQueue { city, queue } => {
            queue.first().map(|i| (city.0, format!("{i:?}")))
        }
        _ => None,
    }
}

fn first_tech(o: &Order) -> Option<String> {
    match o {
        Order::SetResearch { tech } => Some(format!("{tech:?}")),
        Order::SetResearchQueue { techs } => techs.first().map(|t| format!("{t:?}")),
        _ => None,
    }
}

fn move_to(o: &Order) -> Option<(u32, Hex)> {
    match o {
        Order::MoveUnit { unit, to } | Order::MoveUnitPath { unit, to, .. } => Some((unit.0, *to)),
        _ => None,
    }
}

/// Score one turn: his orders against the source's, on the state they were both given in.
fn score_turn(state: &GameState, seat: PlayerId, his: &[Order], alt: &[Order], t: &mut Tally) {
    // Production and research: what the source leaves in place.
    for o in his {
        if let Some((city, item)) = first_item(o) {
            let predicted = alt
                .iter()
                .filter_map(first_item)
                .find(|(c, _)| *c == city)
                .map(|(_, i)| i)
                .or_else(|| {
                    state
                        .cities
                        .get(&legatum_core::CityId(city))
                        .and_then(|c| c.producing.as_ref())
                        .map(|i| format!("{i:?}"))
                });
            t.add("production", predicted.as_deref() == Some(&item), false);
        }
        if let Some(tech) = first_tech(o) {
            let predicted = alt.iter().find_map(first_tech).or_else(|| {
                state
                    .player(seat)
                    .researching
                    .as_ref()
                    .map(|x| format!("{x:?}"))
            });
            t.add("research", predicted.as_deref() == Some(&tech), false);
        }
    }
    let buys = |os: &[Order]| -> Vec<String> {
        os.iter()
            .filter_map(|o| match o {
                Order::BuyItem { city, item } => Some(format!("{}:{item:?}", city.0)),
                _ => None,
            })
            .collect()
    };
    let (hb, ab) = (buys(his), buys(alt));
    for b in &hb {
        t.add("buy", ab.contains(b), false);
    }
    t.buy_false += ab.iter().filter(|b| !hb.contains(b)).count() as u32;
    for o in his {
        match o {
            Order::DeclareWar { against, .. } => {
                let hit = alt
                    .iter()
                    .any(|a| matches!(a, Order::DeclareWar { against: x, .. } if x == against));
                t.add("war", hit, false);
            }
            Order::FoundCity { unit } => {
                let hit = alt
                    .iter()
                    .any(|a| matches!(a, Order::FoundCity { unit: u } if u == unit));
                t.add("found", hit, false);
            }
            Order::RangedAttack { unit, target } => {
                let hit = alt.iter().any(|a| {
                    matches!(a, Order::RangedAttack { unit: u, target: x } if u == unit && x == target)
                });
                t.add("attack", hit, false);
            }
            _ => {}
        }
        if let Some((unit, to)) = move_to(o) {
            let Some(u) = state.units.get(&legatum_core::UnitId(unit)) else {
                continue;
            };
            let cat = if u.kind == UnitKind::Settler {
                "settler"
            } else if is_military(u.kind) && u.kind != UnitKind::Scout {
                "move"
            } else {
                continue;
            };
            let dest = alt.iter().filter_map(move_to).find(|(x, _)| *x == unit);
            let hit = dest.is_some_and(|(_, d)| d == to);
            let near = dest.is_some_and(|(_, d)| d.distance(to) <= 2);
            t.add(cat, hit, near);
        }
    }
}

/// `hard`, `designer` (the imitation seat), `designer-genes` (its genes without its order-level
/// habits) or `params:FILE`.
fn source_brain(spec: &str, state: &GameState, seat: PlayerId) -> Result<Box<dyn Brain>, String> {
    let seed = legatum_wasm::session::brain_seed(state.setup.seed, seat);
    let faction = state.player(seat).faction;
    let with = |params: &'static AiParams| {
        Box::new(ScriptedBrain::with_params(
            faction,
            seed,
            seat,
            AiDifficulty::Hard,
            params,
        ))
    };
    Ok(match spec {
        "hard" => Box::new(ScriptedBrain::with_difficulty(
            faction,
            seed,
            seat,
            AiDifficulty::Hard,
        )),
        "designer" => Box::new(legatum_ai::designer::brain(faction, seed, seat)),
        "designer-genes" => with(legatum_ai::designer::params()),
        other => {
            let path = other.strip_prefix("params:").unwrap_or(other);
            let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            let params: AiParams =
                serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
            params.validate()?;
            with(Box::leak(Box::new(params)))
        }
    })
}

fn phase(turn: u32) -> &'static str {
    match turn {
        0..=40 => "T1-40",
        41..=100 => "T41-100",
        _ => "T101+",
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut tape, mut states, mut out) = (None, None, None);
    let (mut seat, mut from, mut to) = (0u8, 0u32, u32::MAX);
    let mut sources: Vec<String> = vec![];
    while let Some(a) = args.next() {
        let mut v = || args.next().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--tape" => tape = Some(v()?),
            "--states" => states = Some(v()?),
            "--out" => out = Some(v()?),
            "--seat" => seat = v()?.parse().map_err(|_| "--seat")?,
            "--from" => from = v()?.parse().map_err(|_| "--from")?,
            "--to" => to = v()?.parse().map_err(|_| "--to")?,
            "--source" => sources.push(v()?),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if sources.is_empty() {
        sources.push("hard".into());
    }
    let seat = PlayerId(seat);
    let tape_path = tape.ok_or("--tape is required")?;
    let tape = Tape::from_json(
        &std::fs::read_to_string(&tape_path).map_err(|e| format!("{tape_path}: {e}"))?,
    )?;
    let states_path = states.ok_or("--states is required")?;
    let reader = std::io::BufReader::new(
        std::fs::File::open(&states_path).map_err(|e| format!("{states_path}: {e}"))?,
    );
    let mut lines = reader.lines();
    // `None` past the last exported state (exports may stop before the game's end).
    let mut read = |expect: &str| -> Result<Option<GameState>, String> {
        let Some(line) = lines.next() else {
            return Ok(None);
        };
        let line = line.map_err(|e| e.to_string())?;
        let hash = legatum_core::state::fnv1a(line.as_bytes());
        if hash.to_string() != expect {
            return Err(format!("recorded state hash {hash} is not {expect}"));
        }
        serde_json::from_str(&line)
            .map(Some)
            .map_err(|e| format!("recorded state: {e}"))
    };
    let mut state = read(&tape.initial_hash.to_string())?.ok_or("empty states file")?;
    let mut scored = 0u32;
    let mut brains: Vec<Box<dyn Brain>> = sources
        .iter()
        .map(|s| source_brain(s, &state, seat))
        .collect::<Result<_, _>>()?;
    // source -> phase -> tally ("all" too)
    let mut tallies: Vec<BTreeMap<&'static str, Tally>> = vec![BTreeMap::new(); sources.len()];
    let mut his_wars: Vec<(u32, TeamId)> = vec![];
    let mut alt_wars: Vec<Vec<(u32, TeamId)>> = vec![vec![]; sources.len()];
    let declared = |os: &[Order]| -> Vec<TeamId> {
        os.iter()
            .filter_map(|o| match o {
                Order::DeclareWar { against, .. } => Some(*against),
                _ => None,
            })
            .collect()
    };
    for t in &tape.turns {
        if state.turn != t.turn {
            return Err(format!(
                "recorded turn sequence: state {} tape {}",
                state.turn, t.turn
            ));
        }
        if t.turn > to {
            break;
        }
        if state.player(seat).alive {
            let view = player_view(&state, seat);
            let his = t.orders.get(&seat).cloned().unwrap_or_default();
            his_wars.extend(declared(&his).into_iter().map(|team| (t.turn, team)));
            for (i, (brain, tally)) in brains.iter_mut().zip(tallies.iter_mut()).enumerate() {
                let alt = brain.orders(&view);
                alt_wars[i].extend(declared(&alt).into_iter().map(|team| (t.turn, team)));
                if t.turn >= from {
                    for key in ["all", phase(t.turn)] {
                        score_turn(&state, seat, &his, &alt, tally.entry(key).or_default());
                    }
                }
            }
        }
        scored += 1;
        match read(&t.hash)? {
            Some(next) => state = next,
            None => break,
        }
    }
    eprintln!(
        "designer_predict: {scored} turns compared, last T{}",
        state.turn
    );
    // A war declared within three turns of his on the same team.
    for (alt, tally) in alt_wars.iter().zip(tallies.iter_mut()) {
        for &(turn, team) in his_wars.iter().filter(|(turn, _)| *turn >= from) {
            if alt.iter().any(|&(t, x)| x == team && t.abs_diff(turn) <= 3) {
                for key in ["all", phase(turn)] {
                    if let Some(e) = tally.get_mut(key).and_then(|t| t.cats.get_mut("war")) {
                        e.2 += 1;
                    }
                }
            }
        }
    }
    let mut report = serde_json::Map::new();
    for (name, tally) in sources.iter().zip(&tallies) {
        let mut by_phase = serde_json::Map::new();
        for (key, t) in tally {
            let mut cats = serde_json::Map::new();
            let mut rates = vec![];
            for cat in CATS {
                let Some(&(hits, n, near)) = t.cats.get(cat) else {
                    continue;
                };
                rates.push(hits * 10000 / n.max(1));
                cats.insert(
                    cat.into(),
                    json!({"hits": hits, "his": n, "near": near, "pct_x100": hits * 10000 / n.max(1)}),
                );
            }
            let mean = if rates.is_empty() {
                0
            } else {
                rates.iter().sum::<u32>() / rates.len() as u32
            };
            by_phase.insert(
                (*key).into(),
                json!({"categories": cats, "mean_pct_x100": mean, "buy_false": t.buy_false}),
            );
        }
        report.insert(name.clone(), serde_json::Value::Object(by_phase));
    }
    let report = json!({"tape": tape_path, "seat": seat.0, "from": from, "turns": scored, "sources": report});
    for (name, tally) in sources.iter().zip(&tallies) {
        let Some(all) = tally.get("all") else {
            continue;
        };
        let cells: Vec<String> = all
            .cats
            .iter()
            .map(|(c, (h, n, near))| {
                format!(
                    "{c} {}/{n} ({}%, near {}%)",
                    h,
                    h * 100 / n.max(&1),
                    near * 100 / n.max(&1)
                )
            })
            .collect();
        println!(
            "{name}: {}; buys he did not make {}",
            cells.join(", "),
            all.buy_false
        );
    }
    if let Some(out) = out {
        std::fs::write(&out, serde_json::to_string_pretty(&report).expect("json"))
            .map_err(|e| format!("{out}: {e}"))?;
    }
    Ok(())
}

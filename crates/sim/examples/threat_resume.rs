//! Resume a playtest tape at a turn and let the AIs play on against a stand-in for the human.
//! threat_resume <tape.json> <resume-turn> <turns> <human: proxy6|rush|hold> <memory: watched|fresh>
//!
//! The tape's recorded orders are replayed up to <resume-turn>. With `watched`, every AI brain
//! also plans (orders discarded) on each replayed turn, so view-derived memory accumulates as
//! if it had played the game; `fresh` builds new brains at the resume turn, as a loaded save
//! without stored AI memory does. From then on AI seats play the shipped scripted brain and
//! the human seat plays the chosen stand-in: `proxy6` (the human-playbook benchmark brain),
//! `rush` (scripted Hard with the Rush bias) or `hold` (no orders).
//! Prints wars, treaties, calls, captures, eliminations, walls and each AI's top-threat marks.
use legatum_ai::{Brain, ScriptedBrain, Strategy};
use legatum_core::{
    AiDifficulty, BuildingKind, Event, Order, PlayerId, ProductionItem, data, player_view,
    resolve_turn,
};
use legatum_wasm::session::{BrainKind, brain_seed, make_brain};
use legatum_wasm::tape::Tape;
use std::collections::{BTreeMap, BTreeSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let resume: u32 = a[2].parse()?;
    let turns: u32 = a[3].parse()?;
    let human_kind = a.get(4).map_or("proxy6", |s| s.as_str());
    let watched = a.get(5).is_none_or(|s| s != "fresh");
    let mut state = tape.initial_state();
    let human = state
        .players
        .iter()
        .find(|p| p.is_human)
        .map_or(PlayerId(0), |p| p.id);
    let mut brains: Vec<Box<dyn Brain>> = state
        .players
        .iter()
        .map(|p| make_brain(BrainKind::Scripted, &state, p.id))
        .collect();
    for turn in tape.turns.iter().filter(|t| t.turn < resume) {
        if watched {
            for p in state.players.iter().filter(|p| p.alive && !p.is_human) {
                brains[p.id.0 as usize].orders(&player_view(&state, p.id));
            }
        }
        state = resolve_turn(&state, &turn.orders).0;
    }
    if !watched {
        brains = state
            .players
            .iter()
            .map(|p| make_brain(BrainKind::Scripted, &state, p.id))
            .collect();
    }
    let seed = brain_seed(state.setup.seed, human);
    let faction = state.player(human).faction;
    brains[human.0 as usize] = match human_kind {
        "rush" => {
            let mut b = ScriptedBrain::with_strategy(faction, seed, human, Strategy::Rush);
            b.difficulty = AiDifficulty::Hard;
            Box::new(b)
        }
        "hold" => Box::new(Hold),
        _ => Box::new(legatum_ai::HumanPlaybookV6Brain::default()),
    };
    println!(
        "resume at turn {} ({} memory), human seat P{} ({faction:?}) played by {human_kind}",
        state.turn,
        if watched { "watched" } else { "fresh" },
        human.0
    );
    let mut top_marked: BTreeMap<PlayerId, u32> = BTreeMap::new();
    let mut walls_ordered: BTreeSet<(PlayerId, legatum_core::CityId)> = BTreeSet::new();
    let mut summary: BTreeMap<PlayerId, Vec<String>> = BTreeMap::new();
    let start_cities = state.cities_of(human).count();
    for _ in 0..turns {
        if state.finished {
            break;
        }
        let t = state.turn;
        let mut orders = BTreeMap::new();
        for p in state.players.clone().iter().filter(|p| p.alive) {
            let view = player_view(&state, p.id);
            let o = brains[p.id.0 as usize].orders(&view);
            if std::env::var("THREAT_DEBUG").ok() == Some(p.id.0.to_string()) {
                let cities: Vec<_> = view
                    .cities
                    .iter()
                    .filter(|c| c.owner == p.id)
                    .map(|c| {
                        let d = c.detail.as_ref();
                        format!(
                            "{} walls {} producing {:?} res {:?}",
                            c.name,
                            c.walls,
                            d.and_then(|d| d.producing),
                            d.map(|d| d.resistance)
                        )
                    })
                    .collect();
                println!(
                    "    DEBUG T{t} P{}: research {:?} fort-known {} sci {} gold {} cities {:?} orders {:?}",
                    p.id.0,
                    view.research.current,
                    view.research.known.contains(&legatum_core::TechId::Masonry),
                    view.economy.science_per_turn.total,
                    view.economy.gold,
                    cities,
                    o.iter()
                        .filter(|o| !matches!(
                            o,
                            Order::MoveUnit { .. }
                                | Order::SkipUnit { .. }
                                | Order::MoveUnitPath { .. }
                        ))
                        .collect::<Vec<_>>()
                );
            }

            if p.id != human {
                if let Some(m) = brains[p.id.0 as usize].opponent_memory()
                    && let Some(top) = m.assessments.threats.top()
                    && !top_marked.contains_key(&p.id)
                {
                    top_marked.insert(p.id, t);
                    let line = format!(
                        "T{t} P{} {:?} marks P{} top threat (score {}: {})",
                        p.id.0,
                        p.faction,
                        top.rival.0,
                        top.score,
                        top.reasons().join(", ")
                    );
                    println!("  {line}");
                    summary.entry(p.id).or_default().push(line);
                }
                for order in &o {
                    let city = match order {
                        Order::SetProduction {
                            city,
                            item:
                                ProductionItem::Building {
                                    building: BuildingKind::Walls,
                                },
                        }
                        | Order::BuyItem {
                            city,
                            item:
                                ProductionItem::Building {
                                    building: BuildingKind::Walls,
                                },
                        } => Some(*city),
                        _ => None,
                    };
                    if let Some(city) = city
                        && walls_ordered.insert((p.id, city))
                    {
                        let line = format!(
                            "T{t} P{} {:?} orders Walls in {} ({})",
                            p.id.0,
                            p.faction,
                            state.cities[&city].name,
                            if matches!(order, Order::BuyItem { .. }) {
                                "buy"
                            } else {
                                "build"
                            }
                        );
                        println!("  {line}");
                        summary.entry(p.id).or_default().push(line);
                    }
                }
            }
            orders.insert(p.id, o);
        }
        let (next, events) = resolve_turn(&state, &orders);
        for e in &events {
            let line = match e {
                Event::WarDeclared { relation } => Some(format!(
                    "war: team {} vs {} (declared by {:?})",
                    relation.a.0,
                    relation.b.0,
                    relation.declarers.iter().map(|t| t.0).collect::<Vec<_>>()
                )),
                Event::TreatyActivated { relation, .. } => Some(format!(
                    "treaty: team {}-{} {:?}",
                    relation.a.0, relation.b.0, relation.state
                )),
                Event::PeaceEstablished { relation } => {
                    Some(format!("peace: team {}-{}", relation.a.0, relation.b.0))
                }
                Event::CityCaptured { city, from, to, .. } => Some(format!(
                    "capture: {} P{} -> P{}",
                    next.cities.get(city).map_or("?", |c| c.name.as_str()),
                    from.0,
                    to.0
                )),
                Event::CityRazed { from, by, .. } => {
                    Some(format!("razed: P{} city by P{}", from.0, by.0))
                }
                Event::PlayerEliminated { player } => Some(format!("eliminated: P{}", player.0)),
                Event::ProductionCompleted {
                    owner,
                    item: ProductionItem::Building { building },
                    city,
                } if data::has_building(&BTreeSet::from([*building]), BuildingKind::Walls) => {
                    Some(format!(
                        "walls done: P{} {}",
                        owner.0,
                        next.cities.get(city).map_or("?", |c| c.name.as_str())
                    ))
                }
                _ => None,
            };
            if let Some(line) = line {
                println!("T{t} {line}");
            }
        }
        state = next;
    }
    println!(
        "end turn {}: P{} cities {} (start {}), alive AIs {}",
        state.turn,
        human.0,
        state.cities_of(human).count(),
        start_cities,
        state
            .players
            .iter()
            .filter(|p| p.alive && p.id != human)
            .count()
    );
    for p in state.players.iter().filter(|p| p.id != human) {
        let rel = state
            .diplomacy
            .as_ref()
            .and_then(|d| d.relation(p.team, state.team_of(human)))
            .map(|r| format!("{:?}", r.state))
            .unwrap_or_default();
        println!(
            "  P{} {:?}: alive {} cities {} walls {} relation-to-human {rel}",
            p.id.0,
            p.faction,
            p.alive,
            state.cities_of(p.id).count(),
            state
                .cities_of(p.id)
                .filter(|c| data::has_building(&c.buildings, BuildingKind::Walls))
                .count()
        );
    }
    Ok(())
}

struct Hold;
impl Brain for Hold {
    fn plan(&mut self, _view: &legatum_core::PlayerView) -> Vec<Order> {
        Vec::new()
    }
}

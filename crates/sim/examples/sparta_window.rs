//! Four-player Standard Hard + Search Sparta strike-window cohort.
//! sparta_window <games> [first-seed=874654782136188] [threads=4]
//! One JSON row per seed; a declaration counts only while a known neighboring city is
//! currently visible and unwalled, with a healthy Sparta infantry-led field army.
use legatum_core::scoring::team_vp;
use legatum_core::{AiDifficulty, CityId, Event, Faction, GameSetup, Order, PlayerId, TeamId};
use legatum_wasm::session::{BrainKind, Session};
use rayon::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Default)]
struct WalledAttempt {
    started: u32,
    losses: u32,
    captured: bool,
    abort: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: u64 = args.get(1).map_or(32, |s| s.parse().expect("games"));
    let seed: u64 = args
        .get(2)
        .map_or(874_654_782_136_188, |s| s.parse().expect("seed"));
    let threads: usize = args.get(3).map_or(4, |s| s.parse().expect("threads"));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("thread pool");
    let results: Vec<_> =
        pool.install(|| (0..games).into_par_iter().map(|i| run(seed + i)).collect());
    for row in results {
        println!("{row}");
    }
}

fn snapshot(session: &Session, me: PlayerId) -> Value {
    let state = &session.state;
    let cities: Vec<_> = state.cities_of(me).collect();
    let yields: i32 = cities
        .iter()
        .map(|c| {
            let y = legatum_core::economy::city_yields(state, c).total;
            y.food + y.production + y.gold + y.science
        })
        .sum();
    json!({
        "cities": cities.len(),
        "population": cities.iter().map(|c| c.population).sum::<i32>(),
        "yields": yields,
        "vp": team_vp(state, state.team_of(me)),
    })
}

fn run(seed: u64) -> Value {
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(4);
    (setup.width, setup.height) = (40, 26);
    for (p, faction) in setup.players.iter_mut().zip([
        Faction::Mongols,
        Faction::India,
        Faction::Sparta,
        Faction::Athens,
    ]) {
        p.faction = faction;
        p.difficulty = AiDifficulty::Hard;
        p.search = true;
    }
    let me = PlayerId(2);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    let mut opportunity = None;
    let mut attack = None;
    let mut first_window = Value::Null;
    let mut war_declarations = Vec::new();
    let mut captures = 0;
    let mut losses = 0;
    let mut walled = BTreeMap::<CityId, WalledAttempt>::new();
    let mut at60 = Value::Null;
    let mut at100 = Value::Null;
    let mut at150 = Value::Null;
    while !session.state.finished && session.state.turn <= 150 {
        let state = &session.state;
        let turn = state.turn;
        let view = legatum_core::player_view(state, me);
        let force: Vec<_> = view
            .units
            .iter()
            .filter(|u| {
                u.owner == me && u.military && u.kind != legatum_core::UnitKind::Scout && u.hp >= 60
            })
            .collect();
        let eligible: Vec<TeamId> = if force.len() >= 3 && force.iter().any(|u| u.ranged == 0) {
            view.cities
                .iter()
                .filter(|c| {
                    c.visible
                        && !c.walls
                        && c.owner != me
                        && !c.owner.is_barbarian()
                        && view.faction_of(c.owner) != Faction::Celts
                        && state.cities_of(c.owner).all(|owned| {
                            !owned.buildings.contains(&legatum_core::BuildingKind::Walls)
                        })
                        && view
                            .cities
                            .iter()
                            .any(|home| home.owner == me && home.pos.distance(c.pos) <= 14)
                })
                .map(|c| view.team_of(c.owner))
                .collect()
        } else {
            Vec::new()
        };
        if opportunity.is_none() && !eligible.is_empty() {
            opportunity = Some(turn);
        }
        let candidate_cities: Vec<_> = if opportunity == Some(turn) {
            view.cities.iter().filter(|c| eligible.contains(&view.team_of(c.owner))).map(|c| json!({"id": c.id.0, "owner": c.owner.0, "distance": view.cities.iter().filter(|h| h.owner == me).map(|h| h.pos.distance(c.pos)).min(), "visible": c.visible, "walls": c.walls, "rival_force": state.units.values().filter(|u| u.owner == c.owner && legatum_core::rules::is_military(u.kind) && u.kind != legatum_core::UnitKind::Scout).count(), "seen_force": view.units.iter().filter(|u| u.owner == c.owner && u.military && u.kind != legatum_core::UnitKind::Scout).count()})).collect()
        } else {
            Vec::new()
        };
        // Count the first committed campaign or direct attack on each observed walled
        // city. The observer sees the true pre-turn wall state; AI decisions use only
        // their PlayerView. Losses are assigned to the nearest live attempt.
        let walls: Vec<_> = state
            .cities
            .values()
            .filter(|c| {
                c.owner != me
                    && legatum_core::data::has_building(
                        &c.buildings,
                        legatum_core::BuildingKind::Walls,
                    )
            })
            .map(|c| (c.id, c.pos))
            .collect();
        let orders = session.ai_orders();
        if let Some(mine) = orders.get(&me) {
            for (city, pos) in &walls {
                if mine.iter().any(|order| matches!(order,
                    Order::MoveUnit { to, .. } | Order::MoveUnitPath { to, .. } | Order::RangedAttack { target: to, .. }
                        if to == pos)) {
                    walled.entry(*city).or_insert_with(|| WalledAttempt { started: turn, ..Default::default() });
                }
            }
        }
        if let Some(brain) = session.brain(me) {
            for event in brain
                .campaign_events()
                .iter()
                .filter(|event| event.turn == turn)
            {
                if walls.iter().any(|(id, _)| *id == event.target) {
                    let attempt = walled.entry(event.target).or_insert_with(|| WalledAttempt {
                        started: turn,
                        ..Default::default()
                    });
                    if let Some(end) = event.end
                        && end != legatum_ai::CampaignEnd::Captured
                    {
                        attempt.abort = Some(format!("{end:?}"));
                    }
                }
            }
        }
        if opportunity == Some(turn) {
            let search = session.brain(me).and_then(|b| b.search_events().last());
            let plan = session.brain(me).and_then(|b| b.plan_events().last());
            first_window = json!({"turn":turn,"force":force.len(),"cities":candidate_cities,"search":search,"plan":plan});
        }
        if let Some(mine) = orders.get(&me) {
            for o in mine {
                if let Order::DeclareWar { against, .. } = o {
                    war_declarations.push(json!({"turn":turn,"against":against.0,"eligible":eligible.contains(against)}));
                }
            }
        }
        if attack.is_none() {
            attack = orders.get(&me).and_then(|mine| {
                mine.iter().find_map(|o| match o {
                    Order::DeclareWar { against, .. } if eligible.contains(against) => {
                        Some((turn, against.0))
                    }
                    _ => None,
                })
            });
        }
        let events = session.step(&orders);
        for event in events {
            match event {
                Event::CityCaptured { city, to, .. } if to == me => {
                    captures += 1;
                    if let Some(attempt) = walled.get_mut(&city) {
                        attempt.captured = true;
                        attempt.abort = None;
                    }
                }
                Event::UnitDestroyed {
                    owner,
                    unit_kind,
                    at,
                    ..
                } if owner == me && unit_kind != legatum_core::UnitKind::Scout => {
                    losses += 1;
                    if let Some((id, _)) = walls
                        .iter()
                        .filter(|(id, pos)| {
                            walled
                                .get(id)
                                .is_some_and(|a| !a.captured && a.abort.is_none())
                                && pos.distance(at) <= 5
                        })
                        .min_by_key(|(id, pos)| (pos.distance(at), *id))
                        && let Some(attempt) = walled.get_mut(id)
                    {
                        attempt.losses += 1;
                    }
                }
                _ => {}
            }
        }
        match session.state.turn {
            61 => at60 = snapshot(&session, me),
            101 => at100 = snapshot(&session, me),
            151 => at150 = snapshot(&session, me),
            _ => {}
        }
    }
    if at100.is_null() {
        at100 = snapshot(&session, me);
    }
    let walled_attempts = walled.len();
    let walled_captures = walled.values().filter(|a| a.captured).count();
    let walled_losses: u32 = walled
        .values()
        .filter(|a| a.captured)
        .map(|a| a.losses)
        .sum();
    let walled_aborts = walled.values().filter(|a| a.abort.is_some()).count();
    let walled_unresolved = walled_attempts - walled_captures - walled_aborts;
    json!({
        "seed": seed,
        "opportunity": opportunity,
        "attack": attack,
        "first_window": first_window,
        "war_declarations": war_declarations,
        "captures": captures,
        "losses": losses,
        "walled_attempts": walled_attempts,
        "walled_captures": walled_captures,
        "walled_losses_on_captures": walled_losses,
        "walled_aborts": walled_aborts,
        "walled_unresolved": walled_unresolved,
        "walled_detail": walled.iter().map(|(id, a)| json!({"city":id.0,"start":a.started,"captured":a.captured,"losses":a.losses,"abort":a.abort})).collect::<Vec<_>>(),
        "t60": at60,
        "t100": at100,
        "t150": at150,
    })
}

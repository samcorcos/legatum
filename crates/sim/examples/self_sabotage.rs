//! Observer-only detector for two matched tape cohorts. Usage:
//! cargo run --release -p legatum-sim --example self_sabotage -- <tapes-dir> <results.json>
//! This replays actual orders and core events. It never asks a brain or an LLM to label play.

use legatum_core::diplomacy::TreatyKind;
use legatum_core::{Event, GameState, Order, PlayerId, UnitKind, resolve_turn};
use legatum_wasm::tape::Tape;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Default, Clone, Serialize)]
struct Counts {
    abandoned_sieges: u32,
    wartime_disbands_near_city: u32,
    wartime_ordered_disbands_near_city: u32,
    wartime_bankruptcy_disbands_near_city: u32,
    favorable_peace_offers: u32,
    campaigns_started: u32,
    campaigns_captured: u32,
    campaigns_aborted: BTreeMap<String, u32>,
    campaigns_expired: u32,
    /// Cities this seat captured (core CityCaptured events).
    captures: u32,
    /// This seat's Settlers taken by anyone (rival or barbarian).
    settlers_lost: u32,
    settlers_lost_to_barbarians: u32,
    /// This seat's Settlers taken by a rival (not barbarians), by the order the Settler had
    /// that turn: founding, moving, holding or none.
    settlers_lost_founding: u32,
    settlers_lost_moving: u32,
    settlers_lost_idle: u32,
    /// Rival Settlers this seat took.
    settlers_sniped: u32,
    /// ...of which the war that allowed it was declared by this seat the same turn.
    settlers_sniped_by_surprise_war: u32,
    /// Peace offered or accepted within 5 turns of this seat declaring that war, with no
    /// city changing hands between the two teams in between: a war reversed for no visible reason.
    war_reversals: u32,
    wars_declared: u32,
    /// Strategic plan switches (branch brain log); those with no recorded reason.
    plan_switches: u32,
    plan_switches_without_reason: u32,
    blocked_expenses: u32,
    /// Seats that founded or took a second city, and the sum of those turns.
    second_city_seats: u32,
    second_city_turn_sum: u32,
    /// Seats alive at turn 60 still on one city.
    one_city_at_60: u32,
}
impl Counts {
    fn add(&mut self, other: &Self) {
        self.abandoned_sieges += other.abandoned_sieges;
        self.wartime_disbands_near_city += other.wartime_disbands_near_city;
        self.wartime_ordered_disbands_near_city += other.wartime_ordered_disbands_near_city;
        self.wartime_bankruptcy_disbands_near_city += other.wartime_bankruptcy_disbands_near_city;
        self.favorable_peace_offers += other.favorable_peace_offers;
        self.campaigns_started += other.campaigns_started;
        self.campaigns_captured += other.campaigns_captured;
        self.campaigns_expired += other.campaigns_expired;
        self.captures += other.captures;
        self.settlers_lost += other.settlers_lost;
        self.settlers_lost_to_barbarians += other.settlers_lost_to_barbarians;
        self.settlers_lost_founding += other.settlers_lost_founding;
        self.settlers_lost_moving += other.settlers_lost_moving;
        self.settlers_lost_idle += other.settlers_lost_idle;
        self.settlers_sniped += other.settlers_sniped;
        self.settlers_sniped_by_surprise_war += other.settlers_sniped_by_surprise_war;
        self.war_reversals += other.war_reversals;
        self.wars_declared += other.wars_declared;
        self.plan_switches += other.plan_switches;
        self.plan_switches_without_reason += other.plan_switches_without_reason;
        self.blocked_expenses += other.blocked_expenses;
        self.second_city_seats += other.second_city_seats;
        self.second_city_turn_sum += other.second_city_turn_sum;
        self.one_city_at_60 += other.one_city_at_60;
        for (reason, count) in &other.campaigns_aborted {
            *self.campaigns_aborted.entry(reason.clone()).or_default() += count;
        }
    }
}

#[derive(Serialize)]
struct GameCounts {
    game: usize,
    seed: u64,
    seats: BTreeMap<u8, Counts>,
}

fn favorable(state: &GameState, p: PlayerId, other: legatum_core::TeamId) -> bool {
    let view = legatum_core::player_view(state, p);
    legatum_ai::favorable_visible_capture(&view, other)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let dir = std::path::Path::new(args.get(1).ok_or("missing tapes directory")?);
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(args.get(2).ok_or("missing results json")?)?)?;
    let games = report["games"].as_array().ok_or("results has no games")?;
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut per_game = Vec::new();
    let mut per_faction: BTreeMap<String, Counts> = BTreeMap::new();
    let mut total = Counts::default();
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy();
        let index: usize = name
            .strip_prefix("game-")
            .and_then(|s| s.get(..5))
            .ok_or("unexpected tape filename")?
            .parse()?;
        let tape = Tape::from_json(&std::fs::read_to_string(&file)?)?;
        let mut state = tape.initial_state();
        let mut seats: BTreeMap<u8, Counts> = state
            .players
            .iter()
            .map(|p| (p.id.0, Counts::default()))
            .collect();
        let mut sieges: BTreeMap<(u8, legatum_core::CityId), (u8, Vec<legatum_core::UnitId>)> =
            BTreeMap::new();
        let recorded = games
            .iter()
            .find(|g| g["index"].as_u64() == Some(index as u64));
        let mut deliberate = BTreeMap::<(u8, u32), Vec<u32>>::new();
        if let Some(game) = recorded
            && let Some(players) = game["players"].as_array()
        {
            for player in players {
                let seat = player["player"]
                    .as_u64()
                    .or_else(|| player["id"].as_u64())
                    .unwrap_or_else(|| player["player_id"].as_u64().unwrap_or(255))
                    as u8;
                let Some(counts) = seats.get_mut(&seat) else {
                    continue;
                };
                let mut blocked_events = 0;
                for event in player["plan_events"].as_array().into_iter().flatten() {
                    if event["kind"].as_str() == Some("blocked_expense") {
                        blocked_events += 1;
                        continue;
                    }
                    counts.plan_switches += 1;
                    if event["reason"].is_null() || event["reason"].as_str() == Some("") {
                        counts.plan_switches_without_reason += 1;
                    }
                }
                // The brain keeps only the first blocked purchases as events; the counter
                // has them all.
                counts.blocked_expenses += player["blocked_expenses"]
                    .as_u64()
                    .map_or(blocked_events, |n| n as u32);
                for event in player["campaign_events"].as_array().into_iter().flatten() {
                    let turn = event["turn"].as_u64().unwrap_or(0) as u32;
                    let city = event["target"].as_u64().unwrap_or(0) as u32;
                    match event["end"].as_str() {
                        None => counts.campaigns_started += 1,
                        Some("captured") => counts.campaigns_captured += 1,
                        Some("deadline") => {
                            counts.campaigns_expired += 1;
                            deliberate.entry((seat, city)).or_default().push(turn);
                        }
                        Some("peace") => {}
                        Some(reason) => {
                            *counts.campaigns_aborted.entry(reason.into()).or_default() += 1;
                            deliberate.entry((seat, city)).or_default().push(turn);
                        }
                    }
                }
            }
        }
        // (seat, enemy team) -> turn this seat's declaration took effect.
        let mut declared: BTreeMap<(u8, legatum_core::TeamId), u32> = BTreeMap::new();
        let mut second_city: BTreeMap<u8, u32> = BTreeMap::new();
        for t in &tape.turns {
            if t.turn != state.turn {
                return Err(format!("{} wrong turn", file.display()).into());
            }
            for (p, orders) in &t.orders {
                let count = seats.get_mut(&p.0).unwrap();
                for order in orders {
                    match order {
                        Order::ProposeTreaty {
                            to,
                            kind: TreatyKind::Peace,
                        } if favorable(&state, *p, *to) => count.favorable_peace_offers += 1,
                        Order::RangedAttack { target, .. } | Order::MoveUnit { to: target, .. }
                            if state
                                .cities
                                .values()
                                .any(|c| c.pos == *target && state.at_war(*p, c.owner)) =>
                        {
                            if let Some(city) = state.cities.values().find(|c| c.pos == *target) {
                                let force: Vec<_> = state
                                    .units_of(*p)
                                    .filter(|u| {
                                        u.kind != UnitKind::Scout
                                            && legatum_core::rules::is_military(u.kind)
                                            && u.pos.distance(city.pos) <= 4
                                    })
                                    .collect();
                                // A lone long-range shot is harassment. A siege has a
                                // capturer and at least one supporting combat unit.
                                if force.len() >= 2
                                    && force.iter().any(|u| {
                                        legatum_core::data::unit_stats(u.kind).ranged == 0
                                            && u.pos.distance(city.pos) <= 3
                                    })
                                {
                                    sieges.insert(
                                        (p.0, city.id),
                                        (0, force.iter().map(|u| u.id).collect()),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            let (next, events) = resolve_turn(&state, &t.orders);
            if next.hash().to_string() != t.hash {
                return Err(format!("{} diverged on turn {}", file.display(), t.turn).into());
            }
            // Peace offered or accepted soon after our own declaration, with nothing gained or
            // lost: the war was reversed without a visible reason.
            for (p, orders) in &t.orders {
                for order in orders {
                    let with = match order {
                        Order::ProposeTreaty {
                            to,
                            kind: TreatyKind::Peace,
                        } => Some(*to),
                        Order::RespondToProposal {
                            proposal,
                            accept: true,
                            ..
                        } => state
                            .diplomacy
                            .iter()
                            .flat_map(|d| &d.proposals)
                            .find(|x| x.id == *proposal && x.kind == TreatyKind::Peace)
                            .map(|x| x.from),
                        _ => None,
                    };
                    if let Some(with) = with
                        && let Some(turn) = declared.remove(&(p.0, with))
                        && t.turn <= turn + 5
                    {
                        seats.get_mut(&p.0).unwrap().war_reversals += 1;
                    }
                }
            }
            for (p, orders) in &t.orders {
                for order in orders {
                    if let Order::DeclareWar { against, .. } = order
                        && !state.teams_at_war(state.team_of(*p), *against)
                        && next.teams_at_war(next.team_of(*p), *against)
                    {
                        declared.insert((p.0, *against), t.turn);
                        seats.get_mut(&p.0).unwrap().wars_declared += 1;
                    }
                }
            }
            for event in &events {
                match event {
                    Event::CityCaptured { from, to, .. } => {
                        if let Some(c) = seats.get_mut(&to.0) {
                            c.captures += 1;
                        }
                        let (a, b) = (next.team_of(*from), next.team_of(*to));
                        declared.retain(|(seat, team), _| {
                            !((next.team_of(PlayerId(*seat)) == a && *team == b)
                                || (next.team_of(PlayerId(*seat)) == b && *team == a))
                        });
                    }
                    Event::SettlerCaptured { unit, from, to, .. }
                        if state
                            .units
                            .get(unit)
                            .is_some_and(|u| u.kind == UnitKind::Settler) =>
                    {
                        if let Some(c) = seats.get_mut(&from.0) {
                            c.settlers_lost += 1;
                            if to.is_barbarian() {
                                c.settlers_lost_to_barbarians += 1;
                            } else {
                                let order = t.orders.get(from).and_then(|orders| {
                                    orders.iter().find(|o| match o {
                                        Order::FoundCity { unit: u }
                                        | Order::MoveUnit { unit: u, .. }
                                        | Order::MoveUnitPath { unit: u, .. } => u == unit,
                                        _ => false,
                                    })
                                });
                                match order {
                                    Some(Order::FoundCity { .. }) => c.settlers_lost_founding += 1,
                                    Some(_) => c.settlers_lost_moving += 1,
                                    None => c.settlers_lost_idle += 1,
                                }
                            }
                        }
                        // A Settler taken is something gained: peace after it isn't a
                        // reversal "for no visible reason".
                        let (a, b) = (next.team_of(*from), next.team_of(*to));
                        declared.retain(|(seat, team), _| {
                            !((next.team_of(PlayerId(*seat)) == a && *team == b)
                                || (next.team_of(PlayerId(*seat)) == b && *team == a))
                        });
                        if let Some(c) = seats.get_mut(&to.0) {
                            c.settlers_sniped += 1;
                            let surprise = t.orders.get(to).is_some_and(|orders| {
                                orders.iter().any(|o| {
                                    matches!(o, Order::DeclareWar { against, .. }
                                        if *against == state.team_of(*from))
                                })
                            });
                            if surprise {
                                c.settlers_sniped_by_surprise_war += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
            for p in &next.players {
                let owned = next.cities.values().filter(|c| c.owner == p.id).count();
                if owned >= 2 && !second_city.contains_key(&p.id.0) {
                    second_city.insert(p.id.0, next.turn);
                }
                if next.turn == 60 && p.alive && owned == 1 {
                    seats.get_mut(&p.id.0).unwrap().one_city_at_60 += 1;
                }
            }
            for event in &events {
                if let Event::UnitDisbanded {
                    unit,
                    owner,
                    reason,
                    ..
                } = event
                    && let Some(before) = state.units.get(unit)
                    && state.cities.values().any(|city| {
                        state.at_war(*owner, city.owner) && before.pos.distance(city.pos) <= 4
                    })
                {
                    let counts = seats.get_mut(&owner.0).unwrap();
                    counts.wartime_disbands_near_city += 1;
                    if reason.contains("Treasury ran dry") {
                        counts.wartime_bankruptcy_disbands_near_city += 1;
                    } else if reason == "Disbanded by order" {
                        counts.wartime_ordered_disbands_near_city += 1;
                    }
                }
            }
            sieges.retain(|(p, city_id), (idle, participants)| {
                let Some(city) = next.cities.get(city_id) else {
                    return false;
                };
                let player = PlayerId(*p);
                if next.team_of(city.owner) == next.team_of(player)
                    || !next.at_war(player, city.owner)
                {
                    return false;
                }
                if deliberate
                    .get(&(*p, city_id.0))
                    .is_some_and(|turns| turns.contains(&t.turn))
                {
                    return false;
                }
                // A force destroyed in battle lost its siege; it did not walk away.
                if participants.iter().all(|id| !next.units.contains_key(id)) {
                    return false;
                }
                let near = next.units_of(player).any(|u| {
                    u.kind != UnitKind::Scout
                        && legatum_core::rules::is_military(u.kind)
                        && u.pos.distance(city.pos) <= 3
                });
                if near {
                    *idle = 0;
                } else {
                    *idle += 1;
                }
                if *idle >= 2 {
                    seats.get_mut(p).unwrap().abandoned_sieges += 1;
                    false
                } else {
                    true
                }
            });
            state = next;
        }
        for (seat, turn) in &second_city {
            let c = seats.get_mut(seat).unwrap();
            c.second_city_seats += 1;
            c.second_city_turn_sum += turn;
        }
        for player in &state.players {
            let count = seats.get(&player.id.0).unwrap();
            let faction = format!("{:?}", player.faction);
            per_faction.entry(faction).or_default().add(count);
            total.add(count);
        }
        per_game.push(GameCounts {
            game: index,
            seed: tape.setup.seed,
            seats,
        });
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "definition": "Siege starts on a city attack order with a melee capturer within 3 and at least two combat units within 4; a lone ranged shot is harassment. Leaving the 3-hex ring for two turns while still at war and without capture, peace, logged abort or defeat of every participating unit is abandonment. Disbands use actual core events and pre-turn distance, including Scouts. Favorable peace runs the same public-view campaign combat and value forecast against the treaty target on both cohorts. Campaign lifecycle comes from the branch brain log; absent on master. Settlers lost/sniped use core SettlerCaptured events on Settlers; a surprise snipe is one whose captor ordered war on the victim's team that turn. A war reversal is peace offered or accepted within 5 turns of the seat's own declaration with no city or Settler changing hands between the two teams. Settlers lost to rivals are split by the Settler's own order that turn (found, move, none). Plan switches and blocked expenses come from the branch brain's plan log (absent on master). Second-city turn is the first turn a seat owns two cities.",
            "games": per_game.len(), "total": total, "per_faction": per_faction, "per_game": per_game
        }))?
    );
    Ok(())
}

//! Rule-based AI mistake audit over the checked-in human-playtest maps.
//!
//! blunder_audit --games 64 --seed 1 --map playtest-8.3-switzerland-win
//! The core resolves every turn; this example only observes states, orders and events.

use legatum_ai::{Brain, ScriptedBrain};
use legatum_core::rng::{Rng, purpose};
use legatum_core::{
    AiDifficulty, Event, GameSetup, GameState, Hex, PlayerId, UnitId, UnitKind, data,
    forecast_attack, player_view, resolve_turn,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

const MAPS: &str = include_str!("../data/hard-vs-human-maps.json");
const CLASSES: [&str; 5] = [
    "exposed_settler",
    "low_chance_attack_loss",
    "idle_army_at_war",
    "abandoned_siege",
    "unguarded_city_lost",
];

#[derive(Clone, Debug, Deserialize)]
struct MapRecord {
    id: String,
    setup: GameSetup,
    human_seat: u8,
}

#[derive(Clone, Debug, Serialize)]
struct Example {
    salt: u64,
    turn: u32,
}

#[derive(Default, Clone, Debug, Serialize)]
struct Counts {
    exposed_settler: u64,
    low_chance_attack_loss: u64,
    idle_army_at_war: u64,
    abandoned_siege: u64,
    unguarded_city_lost: u64,
    examples: BTreeMap<String, Vec<Example>>,
    turns: u64,
}

impl Counts {
    fn record(&mut self, class: &str, salt: u64, turn: u32) {
        match class {
            "exposed_settler" => self.exposed_settler += 1,
            "low_chance_attack_loss" => self.low_chance_attack_loss += 1,
            "idle_army_at_war" => self.idle_army_at_war += 1,
            "abandoned_siege" => self.abandoned_siege += 1,
            "unguarded_city_lost" => self.unguarded_city_lost += 1,
            _ => unreachable!("known class"),
        }
        let examples = self.examples.entry(class.to_owned()).or_default();
        if examples.len() < 5 {
            examples.push(Example { salt, turn });
        }
    }
}

impl AuditState {
    fn record(&mut self, owner: PlayerId, class: &str, salt: u64, turn: u32) {
        if !owner.is_barbarian() {
            self.counts
                .entry(owner)
                .or_default()
                .record(class, salt, turn);
        }
    }
}

#[derive(Default)]
struct AuditState {
    counts: BTreeMap<PlayerId, Counts>,
    idle_turns: BTreeMap<UnitId, u32>,
    idle_reported: BTreeSet<PlayerId>,
}

fn combat(state: &GameState, owner: PlayerId, kind: UnitKind) -> bool {
    data::unit_stats_for(state.player(owner).faction, kind).military
        && !matches!(kind, UnitKind::Scout | UnitKind::Settler)
}

fn at_war_with_any(state: &GameState, owner: PlayerId) -> bool {
    state
        .players
        .iter()
        .any(|p| p.alive && p.id != owner && state.at_war(owner, p.id))
}

fn can_reach(state: &GameState, unit: &legatum_core::state::Unit, target: Hex) -> bool {
    // A cheap conservative threat proxy: movement points bound hex distance; terrain and
    // occupancy may make the real route longer, so this intentionally overcounts danger.
    unit.pos.distance(target)
        <= data::unit_stats_for(state.player(unit.owner).faction, unit.kind)
            .moves
            .max(1)
}

fn exposed_settlers(state: &GameState) -> Vec<(PlayerId, UnitId)> {
    state
        .units
        .values()
        .filter_map(|settler| {
            if settler.kind != UnitKind::Settler {
                return None;
            }
            let has_escort = state.units.values().any(|ally| {
                ally.owner == settler.owner
                    && combat(state, ally.owner, ally.kind)
                    && ally.pos.distance(settler.pos) <= 1
            });
            if has_escort {
                return None;
            }
            let threatened = state.units.values().any(|enemy| {
                state.at_war(settler.owner, enemy.owner)
                    && combat(state, enemy.owner, enemy.kind)
                    && can_reach(state, enemy, settler.pos)
            });
            threatened.then_some((settler.owner, settler.id))
        })
        .collect()
}

fn idle_army_qualifies(state: &GameState, owner: PlayerId, idle: &BTreeMap<UnitId, u32>) -> bool {
    if !at_war_with_any(state, owner) {
        return false;
    }
    let idle_units: Vec<_> = state
        .units
        .values()
        .filter(|u| {
            u.owner == owner
                && combat(state, owner, u.kind)
                && idle.get(&u.id).copied().unwrap_or(0) >= 3
        })
        .collect();
    idle_units.len() >= 3
        && idle_units.iter().any(|u| {
            state.units.values().any(|enemy| {
                state.at_war(owner, enemy.owner)
                    && combat(state, enemy.owner, enemy.kind)
                    && u.pos.distance(enemy.pos) <= 6
            }) || state
                .cities
                .values()
                .any(|city| state.at_war(owner, city.owner) && u.pos.distance(city.pos) <= 6)
        })
}

fn low_chance_loss(
    before: &GameState,
    after: &GameState,
    owner: PlayerId,
    unit: UnitId,
    at: Hex,
) -> bool {
    !after.units.contains_key(&unit)
        && forecast_attack(&player_view(before, owner), unit, at)
            .is_some_and(|forecast| forecast.win_chance < 30)
}

fn abandoned_siege(state: &GameState, owner: PlayerId, from: Hex, to: Hex) -> bool {
    state.cities.values().any(|city| {
        city.owner != owner
            && state.at_war(owner, city.owner)
            && city.hp * 2 < data::CITY_HP
            && from.distance(city.pos) == 1
            && to.distance(city.pos) > 1
    })
}

fn unguarded_city_lost(state: &GameState, owner: PlayerId, pos: Hex) -> bool {
    let has_garrison = state
        .units
        .values()
        .any(|u| u.owner == owner && u.pos == pos && combat(state, u.owner, u.kind));
    !has_garrison
        && state.units.values().any(|u| {
            u.owner == owner
                && combat(state, u.owner, u.kind)
                && u.pos != pos
                && u.pos.distance(pos) <= 3
                && u.pos.distance(pos)
                    <= data::unit_stats_for(state.player(owner).faction, u.kind)
                        .moves
                        .max(1)
        })
}

fn play(map: &MapRecord, salt: u64) -> BTreeMap<PlayerId, Counts> {
    let mut setup = map.setup.clone();
    let seat = PlayerId(map.human_seat);
    setup.players[seat.0 as usize].is_human = false;
    setup.players[seat.0 as usize].difficulty = AiDifficulty::Hard;
    let mut state = legatum_core::new_game(&setup);
    let mut brains: Vec<Box<dyn Brain>> = setup
        .players
        .iter()
        .enumerate()
        .map(|(index, p)| {
            let id = PlayerId(index as u8);
            let seed = Rng::derive(
                legatum_wasm::session::brain_seed(map.setup.seed, id),
                &[purpose::AI, 0x5a17, salt],
            )
            .next_u64();
            let scripted = ScriptedBrain::with_difficulty(
                p.faction,
                seed,
                id,
                if id == seat {
                    AiDifficulty::Hard
                } else {
                    p.difficulty
                },
            );
            if p.search {
                Box::new(legatum_ai::search::SearchBrain::new(
                    scripted,
                    legatum_ai::search::SearchConfig::in_app(setup.players.len()),
                )) as Box<dyn Brain>
            } else {
                Box::new(scripted) as Box<dyn Brain>
            }
        })
        .collect();
    let mut audit = AuditState::default();
    let cap = legatum_core::data::EXPECTED_GAME_TURNS.saturating_mul(4);
    while !state.finished && state.turn < cap {
        let turn = state.turn;
        let orders: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                (
                    p.id,
                    brains[p.id.0 as usize].orders(&player_view(&state, p.id)),
                )
            })
            .collect();
        let (next, events) = resolve_turn(&state, &orders);
        let mut moved = BTreeSet::new();
        let mut attacked = BTreeSet::new();
        for event in &events {
            match event {
                Event::UnitMoved { unit, .. } => {
                    moved.insert(*unit);
                }
                Event::Battle { attacker, .. } => {
                    if let Some(unit) = attacker.unit() {
                        attacked.insert(unit);
                    }
                }
                _ => {}
            }
        }
        for player in &state.players {
            if player.alive && !player.id.is_barbarian() {
                audit.counts.entry(player.id).or_default().turns += 1;
            }
        }
        for (owner, _) in exposed_settlers(&next) {
            audit.record(owner, "exposed_settler", salt, turn);
        }
        for event in &events {
            if let Event::Battle {
                attacker,
                attacker_owner,
                at,
                ..
            } = event
                && let Some(unit) = attacker.unit()
                && low_chance_loss(&state, &next, *attacker_owner, unit, *at)
            {
                audit.record(*attacker_owner, "low_chance_attack_loss", salt, turn);
            }
            if let Event::UnitMoved {
                owner, from, to, ..
            } = event
                && abandoned_siege(&state, *owner, *from, *to)
            {
                audit.record(*owner, "abandoned_siege", salt, turn);
            }
            if let Event::CityCaptured { city, from, .. } = event
                && let Some(old_city) = state.cities.get(city)
                && unguarded_city_lost(&state, *from, old_city.pos)
            {
                audit.record(*from, "unguarded_city_lost", salt, turn);
            }
        }
        let owners: Vec<_> = state
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
            .map(|p| p.id)
            .collect();
        for owner in &owners {
            if !at_war_with_any(&state, *owner) {
                audit
                    .idle_turns
                    .retain(|unit, _| state.units.get(unit).is_none_or(|u| u.owner != *owner));
            }
        }
        for unit in state
            .units
            .values()
            .filter(|u| combat(&state, u.owner, u.kind))
        {
            if moved.contains(&unit.id) || attacked.contains(&unit.id) {
                audit.idle_turns.remove(&unit.id);
            } else {
                *audit.idle_turns.entry(unit.id).or_default() += 1;
            }
        }
        for owner in owners {
            let qualifies = idle_army_qualifies(&state, owner, &audit.idle_turns);
            if qualifies && audit.idle_reported.insert(owner) {
                audit.record(owner, "idle_army_at_war", salt, turn);
            } else if !qualifies {
                audit.idle_reported.remove(&owner);
            }
        }
        state = next;
    }
    audit.counts
}

fn cpu_seconds() -> f64 {
    // CLOCK_PROCESS_CPUTIME_ID includes all Rayon workers, unlike elapsed wall time.
    let mut ts: libc::timespec = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts) };
    assert_eq!(result, 0, "read process CPU clock");
    ts.tv_sec as f64 + ts.tv_nsec as f64 / 1e9
}

fn run() {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str, default: &str| -> String {
        args.windows(2)
            .find(|w| w[0] == flag)
            .map(|w| w[1].clone())
            .unwrap_or_else(|| default.into())
    };
    let games: u64 = value("--games", "64").parse().expect("--games integer");
    let salt_start: u64 = value("--seed", "1").parse().expect("--seed integer");
    let selected_map = value("--map", "");
    let out_dir = value("--out-dir", "");
    let mut maps: Vec<MapRecord> = serde_json::from_str(MAPS).expect("checked-in maps");
    if !selected_map.is_empty() {
        let selected: BTreeSet<_> = selected_map.split(',').collect();
        maps.retain(|m| selected.contains(m.id.as_str()));
    }
    assert!(!maps.is_empty(), "--map must name a map in maps.json");
    let jobs: Vec<_> = maps
        .iter()
        .flat_map(|map| (salt_start..salt_start + games).map(move |salt| (map, salt)))
        .collect();
    let cpu_start = cpu_seconds();
    let wall_start = Instant::now();
    let played: Vec<_> = jobs
        .par_iter()
        .map(|(map, salt)| (map.id.clone(), *salt, play(map, *salt)))
        .collect();
    let cpu_total = cpu_seconds() - cpu_start;
    let mut totals: BTreeMap<String, BTreeMap<PlayerId, Counts>> = BTreeMap::new();
    let mut game_turns = 0u64;
    for (map_id, _salt, seats) in played {
        for (seat, counts) in seats {
            game_turns += counts.turns;
            let total = totals
                .entry(map_id.clone())
                .or_default()
                .entry(seat)
                .or_default();
            total.turns += counts.turns;
            total.exposed_settler += counts.exposed_settler;
            total.low_chance_attack_loss += counts.low_chance_attack_loss;
            total.idle_army_at_war += counts.idle_army_at_war;
            total.abandoned_siege += counts.abandoned_siege;
            total.unguarded_city_lost += counts.unguarded_city_lost;
            for (class, examples) in counts.examples {
                for ex in examples {
                    let target = total.examples.entry(class.clone()).or_default();
                    if target.len() < 5 {
                        target.push(ex);
                    }
                }
            }
        }
    }
    let reports: Vec<_> = totals.into_iter().map(|(map, seats)| {
        let seat_reports: Vec<_> = seats.into_iter().map(|(seat, counts)| {
            let denominator = counts.turns.max(1) as f64;
            let classes: BTreeMap<_, _> = CLASSES.iter().map(|class| {
                let count = match *class {
                    "exposed_settler" => counts.exposed_settler,
                    "low_chance_attack_loss" => counts.low_chance_attack_loss,
                    "idle_army_at_war" => counts.idle_army_at_war,
                    "abandoned_siege" => counts.abandoned_siege,
                    _ => counts.unguarded_city_lost,
                };
                (*class, json!({"count": count, "per_100_turns": count as f64 * 100.0 / denominator,
                    "examples": counts.examples.get(*class).cloned().unwrap_or_default()}))
            }).collect();
            let human_seat = maps.iter().find(|m| m.id == map).unwrap().human_seat;
            json!({"seat": seat.0, "role": if seat.0 == human_seat {"human-seat AI"} else {"AI opponent"}, "turns": counts.turns, "classes": classes})
        }).collect();
        json!({"map": map, "seats": seat_reports})
    }).collect();
    let output = serde_json::to_string_pretty(&json!({
        "games_per_map": games, "salt_start": salt_start, "salt_end": salt_start + games - 1,
        "definitions": {
            "exposed_settler": "Per turn and settler: at turn end an at-war enemy combat unit is within its full movement points (hex distance proxy), no friendly combat unit is on or adjacent; terrain, occupancy and path blockers are ignored.",
            "low_chance_attack_loss": "A unit attacker is absent after resolution, and the core forecast_attack win_chance against the battle tile in the pre-turn player view is below 30 percent.",
            "idle_army_at_war": "One event when a seat first has at least three combat units, each with three consecutive turns neither moving nor attacking, while at war and at least one is within six hexes of an at-war enemy unit or city. Peace resets streaks.",
            "abandoned_siege": "A UnitMoved event leaves an at-war enemy city from distance one to distance greater than one while pre-turn city HP is strictly below half CITY_HP.",
            "unguarded_city_lost": "At CityCaptured, the former owner has no combat unit on the city tile in the pre-turn state, and has a combat unit within three hexes whose base movement points could cover that hex distance in one turn; terrain, occupancy and path blockers are ignored."
        },
        "maps": reports, "cpu_seconds_total": cpu_total,
        "cpu_seconds_per_game": cpu_total / (games * maps.len() as u64).max(1) as f64,
        "wall_seconds_total": wall_start.elapsed().as_secs_f64(), "turns_total": game_turns
    })).unwrap();
    if !out_dir.is_empty() {
        std::fs::write(
            std::path::Path::new(&out_dir).join("blunder-audit.json"),
            &output,
        )
        .expect("write cloud report");
    }
    println!("{output}");
}

fn main() {
    run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{
        Faction, PlayerSetup, TeamId, Terrain,
        game::{blank_game, spawn_unit},
        state::Map,
    };

    fn state() -> GameState {
        let setup = GameSetup {
            seed: 1,
            width: 12,
            height: 10,
            players: (0..2)
                .map(|i| PlayerSetup {
                    name: format!("P{i}"),
                    faction: Faction::Switzerland,
                    team: TeamId(i),
                    is_human: false,
                    difficulty: AiDifficulty::Hard,
                    search: false,
                })
                .collect(),
            turn_limit: Some(150),
            legacy_all_war: false,
            barbarians: false,
            rule_experiments: Default::default(),
        };
        let mut s = blank_game(&setup, Map::new(12, 10, Terrain::Grassland));
        for relation in &mut s.diplomacy.as_mut().unwrap().relations {
            relation.state = legatum_core::diplomacy::RelationState::War;
        }
        s
    }
    fn at(col: i32, row: i32) -> Hex {
        Hex::from_offset(col, row)
    }

    #[test]
    fn exposed_settler_flags_unsafe_move_and_ignores_escort_or_safety() {
        let mut s = state();
        spawn_unit(&mut s, PlayerId(0), UnitKind::Settler, at(5, 5));
        spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, at(7, 5));
        assert_eq!(exposed_settlers(&s).len(), 1);
        spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, at(5, 4));
        assert!(exposed_settlers(&s).is_empty());
    }

    #[test]
    fn low_chance_loss_requires_dead_attacker_and_sub_thirty_forecast() {
        let mut s = state();
        let a = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, at(4, 5));
        spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, at(5, 5));
        s.units.get_mut(&a).unwrap().hp = 1;
        let mut lost = s.clone();
        lost.units.remove(&a);
        assert!(low_chance_loss(&s, &lost, PlayerId(0), a, at(5, 5)));
        assert!(!low_chance_loss(&s, &s, PlayerId(0), a, at(5, 5)));
    }

    #[test]
    fn idle_army_needs_three_mature_idle_combat_units_near_enemy() {
        let mut s = state();
        let mut idle = BTreeMap::new();
        for col in 3..6 {
            let u = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, at(col, 5));
            idle.insert(u, 3);
        }
        spawn_unit(&mut s, PlayerId(1), UnitKind::Soldier, at(7, 5));
        assert!(idle_army_qualifies(&s, PlayerId(0), &idle));
        idle.pop_first();
        assert!(!idle_army_qualifies(&s, PlayerId(0), &idle));
    }

    #[test]
    fn abandoned_siege_requires_withdrawal_from_adjacent_low_hp_city() {
        let mut s = state();
        let id = s.alloc_city_id();
        let pos = at(7, 5);
        s.cities.insert(
            id,
            legatum_core::state::City {
                id,
                owner: PlayerId(1),
                founder: PlayerId(1),
                name: "target".into(),
                pos,
                population: 2,
                food_stored: 0,
                production_stored: 0,
                producing: None,
                bought: None,
                production_queue: vec![],
                build_automation: None,
                last_auto_pick: None,
                auto_queued: None,
                production_banked: vec![],
                production_queue_notified: Default::default(),
                wonder_lost: None,
                last_purchase_turn: None,
                buildings: Default::default(),
                hp: data::CITY_HP / 3,
                grown_tiles: vec![],
                tiles_claimed: 0,
                is_capital: false,
                original_capital: false,
                greece_captive_paid: Default::default(),
                occupied_from: None,
                founded_turn: 1,
                resistance: 0,
                last_attacked_turn: None,
            },
        );
        let from = at(6, 5);
        let to = at(5, 5);
        spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, from);
        assert!(abandoned_siege(&s, PlayerId(0), from, to));
        s.cities.get_mut(&id).unwrap().hp = data::CITY_HP;
        assert!(!abandoned_siege(&s, PlayerId(0), from, to));
    }

    #[test]
    fn unguarded_city_loss_needs_reachable_owner_combat_unit() {
        let mut s = state();
        let pos = at(7, 5);
        let unit = spawn_unit(&mut s, PlayerId(0), UnitKind::Soldier, at(6, 5));
        assert!(unguarded_city_lost(&s, PlayerId(0), pos));
        let far = at(2, 2);
        s.units.get_mut(&unit).unwrap().pos = far;
        assert!(!unguarded_city_lost(&s, PlayerId(0), pos));
    }
}

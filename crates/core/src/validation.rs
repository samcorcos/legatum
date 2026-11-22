//! Checks for untrusted setups and saved states before generation or session construction.

use crate::{GameSetup, GameState, PlayerId, TeamId};
use std::collections::BTreeSet;

/// More than a full map's possible unit population, leaving room for one turn of spawns.
pub const COUNTER_HEADROOM: u32 = 65_536;
/// Treasury headroom keeps long games and interest safely inside i32 arithmetic.
pub const MAX_GOLD: i32 = 100_000_000;
pub const MAX_STORED: i32 = 1_000_000;
pub const MAX_POPULATION: i32 = 1_000;

pub fn add_stored(current: i32, amount: i32) -> i32 {
    (i64::from(current) + i64::from(amount)).clamp(0, i64::from(MAX_STORED)) as i32
}

/// Credits stop at the treasury cap. Costs still subtract their actual amount.
pub fn credit_gold(current: i32, amount: i32) -> i32 {
    (i64::from(current) + i64::from(amount)).clamp(0, i64::from(MAX_GOLD)) as i32
}

pub fn check_setup(setup: &GameSetup) -> Result<(), String> {
    let (width, height) = (setup.width, setup.height);
    if width.checked_mul(height).is_none()
        || !(8..=128).contains(&width)
        || !(8..=128).contains(&height)
        || !(1..=8).contains(&setup.players.len())
        || setup.turn_limit.is_some_and(|n| n == 0 || n == u32::MAX)
    {
        return Err("Invalid map dimensions, player count, or turn limit".into());
    }
    if setup.players.iter().any(|p| p.team == TeamId::BARBARIAN) {
        return Err("A player cannot use the barbarian team".into());
    }
    // Starts are at least seven hexes apart. A 4x4 offset block can hold at most one;
    // only the interior can contain land. This cheap necessary bound rejects setups
    // that cannot possibly have enough starts before map generation allocates anything.
    let interior_blocks = ((width - 4 + 3) / 4) * ((height - 4 + 3) / 4);
    if setup.players.len() as i32 > interior_blocks {
        return Err("Map is too small for the requested starting positions".into());
    }
    Ok(())
}

pub fn check_state(state: &GameState) -> Result<(), String> {
    check_setup(&state.setup)?;
    let map = &state.map;
    let damaged = |what: &str| format!("The save is damaged: {what}");
    let area = map
        .width
        .checked_mul(map.height)
        .ok_or_else(|| damaged("the map size is out of range"))? as usize;
    if map.width != state.setup.width || map.height != state.setup.height {
        return Err(damaged("the map size does not match the game setup"));
    }
    if map.tiles.len() != area {
        return Err(damaged(&format!(
            "the map lists {} tiles but its size needs {area}",
            map.tiles.len()
        )));
    }
    if state.players.len() != state.setup.players.len() {
        return Err(damaged(&format!(
            "the game has {} players but its setup lists {}",
            state.players.len(),
            state.setup.players.len()
        )));
    }
    if state.turn == 0 || state.turn >= u32::MAX - COUNTER_HEADROOM {
        return Err(damaged("the turn number is out of range"));
    }
    if state.next_unit_id >= u32::MAX - COUNTER_HEADROOM
        || state.next_city_id >= u32::MAX - COUNTER_HEADROOM
    {
        return Err(damaged("a unit or city id counter is out of range"));
    }
    if state.units.len() > area * 2 || state.cities.len() > area {
        return Err(damaged("it holds more units or cities than the map can"));
    }
    let player = |id: PlayerId| (id.0 as usize) < state.players.len();
    for p in &state.players {
        if !(0..=MAX_GOLD).contains(&p.gold) {
            return Err(format!(
                "Player {} Gold must be between 0 and {MAX_GOLD}",
                p.id.0
            ));
        }
        if !(0..=MAX_STORED).contains(&p.science_stored) {
            return Err(format!("Player {} stored Science is out of range", p.id.0));
        }
        if !(0..=MAX_STORED).contains(&p.settlers_trained)
            || !(0..=100).contains(&p.decadence_percent)
        {
            return Err(format!("Player {} counters are out of range", p.id.0));
        }
        if p.science_history
            .iter()
            .any(|&n| !(0..=MAX_STORED).contains(&n))
            || p.power_history
                .iter()
                .any(|&n| !(0..=MAX_STORED).contains(&n))
        {
            return Err(format!("Player {} yield history is out of range", p.id.0));
        }
        if p.tile_memory.iter().flatten().any(|memory| {
            memory.city.as_ref().is_some_and(|city| {
                !(1..=MAX_POPULATION).contains(&city.population) || !(0..=1_000).contains(&city.hp)
            })
        }) {
            return Err(format!(
                "Player {} remembered city values are out of range",
                p.id.0
            ));
        }
    }
    for c in state.cities.values() {
        if !(1..=MAX_POPULATION).contains(&c.population) {
            return Err(format!("City {} population is out of range", c.id.0));
        }
        if !(0..=MAX_STORED).contains(&c.food_stored)
            || !(0..=MAX_STORED).contains(&c.production_stored)
            || c.production_banked
                .iter()
                .any(|bank| !(0..=MAX_STORED).contains(&bank.progress))
        {
            return Err(format!("City {} stored yields are out of range", c.id.0));
        }
        if !(0..=1_000).contains(&c.hp)
            || !(0..=MAX_STORED).contains(&c.tiles_claimed)
            || !(0..=MAX_STORED).contains(&c.resistance)
        {
            return Err(format!(
                "City {} health or counters are out of range",
                c.id.0
            ));
        }
        if c.bought.as_ref().is_some_and(|b| {
            !(0..=MAX_GOLD).contains(&b.cost)
                || !(0..=MAX_STORED).contains(&b.progress)
                || b.progress > c.production_stored
        }) {
            return Err(format!(
                "City {} purchased production is out of range",
                c.id.0
            ));
        }
    }
    for u in state.units.values() {
        if !(0..=1_000).contains(&u.hp)
            || !(0..=1_000).contains(&u.moves_left)
            || !(0..=1_000).contains(&u.fortify)
        {
            return Err(format!(
                "Unit {} health or movement is out of range",
                u.id.0
            ));
        }
    }
    if state.aircraft.iter().any(|a| !(0..=1_000).contains(&a.hp)) {
        return Err("Aircraft health is out of range".into());
    }
    let teams: BTreeSet<_> = state.players.iter().map(|p| p.team).collect();
    let mut aircraft_ids = BTreeSet::new();
    if state.players.iter().enumerate().any(|(i, p)| {
        let setup = &state.setup.players[i];
        p.id != PlayerId(i as u8)
            || p.is_human != setup.is_human
            || p.team != setup.team
            || p.explored.len() != area
            || !p.explored.has_valid_storage()
            || p.tile_memory.len() > area
            || p.science_history.len() > crate::data::SCIENCE_HISTORY_TURNS
            || p.goal_holder.is_some_and(|id| !player(id))
            || p.tile_memory
                .iter()
                .flatten()
                .any(|memory| memory.owner.is_some_and(|id| !player(id)))
            || p.observed_threats
                .iter()
                .any(|threat| !(player(threat.owner) || threat.owner.is_barbarian()))
    }) {
        return Err(damaged(
            "a player's record does not match the game setup, its explored map or its memory",
        ));
    }
    if state.units.iter().any(|(id, u)| {
        *id != u.id
            || id.0 > state.next_unit_id
            || !(player(u.owner) || u.owner.is_barbarian())
            || u.heir_of.is_some_and(|id| !player(id))
            || map.index(u.pos).is_none()
    }) {
        return Err(damaged(
            "a unit has a wrong id, an unknown owner or a position off the map",
        ));
    }
    if state.cities.iter().any(|(id, c)| {
        *id != c.id
            || id.0 > state.next_city_id
            || !player(c.owner)
            || !player(c.founder)
            || map.index(c.pos).is_none()
    }) {
        return Err(damaged(
            "a city has a wrong id, an unknown owner or a position off the map",
        ));
    }
    if map.tiles.iter().any(|t| {
        t.owner.is_some_and(|id| !player(id))
            || t.city.is_some_and(|id| !state.cities.contains_key(&id))
    }) {
        return Err(damaged("a map tile names an unknown owner or city"));
    }
    if state.aircraft.iter().any(|a| {
        a.id.0 > state.next_unit_id
            || !aircraft_ids.insert(a.id)
            || state.units.contains_key(&a.id)
            || !state.cities.contains_key(&a.city)
    }) {
        return Err(damaged(
            "an aircraft has a repeated id or sits in an unknown city",
        ));
    }
    if state
        .air_recons
        .iter()
        .any(|r| !player(r.owner) || map.index(r.target).is_none())
    {
        return Err(damaged(
            "an air reconnaissance names an unknown player or a tile off the map",
        ));
    }
    if state.era_firsts.values().any(|&id| !player(id)) {
        return Err(damaged("an era first names an unknown player"));
    }
    if state.winner.is_some_and(|team| !teams.contains(&team)) {
        return Err(damaged("the winner is not a team in this game"));
    }
    if state.vp_holds.keys().any(|team| !teams.contains(team)) {
        return Err(damaged("a victory-point hold names an unknown team"));
    }
    if map.road_hub.is_some_and(|hex| map.index(hex).is_none()) {
        return Err(damaged("the road hub is off the map"));
    }
    Ok(())
}

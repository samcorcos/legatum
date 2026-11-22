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
    check_references(state).map_err(|what| damaged(&what))
}

/// Every player and tile a saved state names must be one this game has, because rules index
/// `players` by player id (`GameState::player`, `player_mut`) and the map by tile. A player id is
/// a seat in `players`, or the barbarians where they may appear; a tile is on the map.
///
/// Team, city and unit ids are deliberately not checked here. States `resolve_turn` produces keep
/// references to things that are gone: a razed Station or Diwan city, a captured city's old records,
/// camp rosters pruned only at the next barbarian planning, memories, supply records, relations
/// and chronicle entries of a team whose empires are all eliminated. Every reader looks those up
/// (`.get`, `contains`, team-member filters) and treats a miss as "gone"; none indexes by them, so
/// a stale or unknown one cannot panic.
fn check_references(state: &GameState) -> Result<(), String> {
    use crate::types::ProductionItem;
    let map = &state.map;
    let player = |id: PlayerId| (id.0 as usize) < state.players.len();
    let player_or_barbarian = |id: PlayerId| player(id) || id.is_barbarian();
    let tile = |h: crate::hex::Hex| map.index(h).is_some();
    let item_ok = |item: &ProductionItem| match item {
        ProductionItem::Wonder { tile: h, .. } | ProductionItem::Improvement { tile: h, .. } => {
            tile(*h)
        }
        _ => true,
    };
    let fail = |what: &str| Err(what.to_string());

    if state.wonders_discovered.iter().any(|&h| !tile(h)) {
        return fail("a discovered wonder is off the map");
    }
    if map
        .tiles
        .iter()
        .any(|t| t.landmark.as_ref().is_some_and(|l| !player(l.of)))
    {
        return fail("a landmark names an unknown empire");
    }
    for p in &state.players {
        if [p.great_work_holder, p.free_techs_from]
            .into_iter()
            .flatten()
            .any(|id| !player(id))
        {
            return fail("a player's Great Work or free technology names an unknown empire");
        }
        if p.failed_city_sites.iter().any(|&h| !tile(h)) {
            return fail("a player's failed city site is off the map");
        }
        if p.observed_threats
            .iter()
            .any(|t| !player_or_barbarian(t.owner) || !tile(t.pos))
        {
            return fail("a remembered threat names an unknown owner or tile");
        }
        if p.tile_memory.iter().flatten().any(|m| {
            m.owner.is_some_and(|id| !player(id))
                || m.city.as_ref().is_some_and(|c| !player(c.owner))
        }) {
            return fail("a remembered tile names an unknown owner");
        }
    }
    for u in state.units.values() {
        if u.goto.is_some_and(|h| !tile(h)) || u.path.iter().any(|&h| !tile(h)) {
            return fail("a unit's destination or route is off the map");
        }
    }
    for c in state.cities.values() {
        if c.occupied_from.is_some_and(|id| !player(id))
            || c.greece_captive_paid.iter().any(|&id| !player(id))
            || c.wonder_lost.as_ref().is_some_and(|w| !player(w.winner))
        {
            return fail("a city's occupation, captive or wonder record names an unknown empire");
        }
        if c.grown_tiles.iter().any(|&h| !tile(h)) {
            return fail("a city's grown tile is off the map");
        }
        if !c.producing.iter().all(item_ok)
            || !c.production_queue.iter().all(item_ok)
            || !c.production_banked.iter().all(|b| item_ok(&b.item))
            || !c.production_queue_notified.iter().all(item_ok)
            || ![c.last_auto_pick, c.auto_queued]
                .iter()
                .flatten()
                .all(item_ok)
            || !c.bought.iter().all(|b| item_ok(&b.item))
        {
            return fail("a city's production names a tile off the map");
        }
    }
    if state
        .air_contacts
        .iter()
        .any(|(owner, contacts)| !player(*owner) || contacts.iter().any(|c| !player(c.owner)))
    {
        return fail("an aircraft sighting names an unknown empire");
    }
    if let Some(b) = &state.barbarians {
        if b.camps
            .iter()
            .any(|camp| !tile(camp.pos) || camp.rite.is_some_and(|id| !player(id)))
        {
            return fail("a barbarian camp is off the map or names an unknown empire");
        }
        if b.known.values().flatten().any(|&h| !tile(h)) {
            return fail("a team's barbarian camp memory is off the map");
        }
    }
    if let Some(d) = &state.diplomacy
        && (d.proposals.iter().any(|p| !player(p.proposer))
            || d.replies.iter().any(|r| !player(r.proposal.proposer))
            || d.superseded
                .iter()
                .any(|s| !player(s.player) || !player(s.winner)))
    {
        return fail("a treaty offer or superseded order names an unknown empire");
    }
    use crate::history::Chronicle;
    let chronicle_ok = |c: &Chronicle| match c {
        Chronicle::Founded { player: p, .. }
        | Chronicle::Era { player: p, .. }
        | Chronicle::GreatWork { player: p, .. }
        | Chronicle::Wonder { player: p, .. }
        | Chronicle::Objective { player: p, .. }
        | Chronicle::Goal { player: p, .. }
        | Chronicle::Eliminated { player: p, .. } => player_or_barbarian(*p),
        Chronicle::Captured { by, from, .. } | Chronicle::Razed { by, from, .. } => {
            player_or_barbarian(*by) && player_or_barbarian(*from)
        }
        _ => true,
    };
    if !state.history.chronicle.iter().all(chronicle_ok) {
        return fail("the game's chronicle names an unknown empire");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    /// A few turns into an ordinary game, so cities, units, diplomacy and barbarians exist.
    fn saved() -> Value {
        let mut state = crate::new_game(&GameSetup::all_ai(11));
        for _ in 0..3 {
            state = crate::resolve_turn(&state, &BTreeMap::new()).0;
        }
        check_state(&state).expect("a resolved game is a valid save");
        serde_json::to_value(&state).unwrap()
    }

    fn first_key(v: &Value) -> String {
        v.as_object().unwrap().keys().next().unwrap().clone()
    }

    /// Each case sets one reference in a valid save to a player, team, city, unit or tile the game
    /// does not have. `true` cases (an unknown player or a tile off the map) must be refused;
    /// `false` cases are stale team, city or unit references, which real games produce (a razed
    /// Station city, a pruned camp roster, a team with no members left) and must load.
    fn mutations(base: &Value) -> Vec<(bool, &'static str, Value, String)> {
        let c = first_key(&base["cities"]);
        let u = first_key(&base["units"]);
        let city = |field: &str| format!("cities/{c}/{field}");
        let unit = |field: &str| format!("units/{u}/{field}");
        let off = json!({"q": 9_999, "r": 9_999});
        let origin = json!({"q": 0, "r": 0});
        let wonder_off = json!({"kind": "Wonder", "building": "Walls", "tile": off});
        let farm_off = json!({"kind": "Improvement", "improvement": "Farm", "tile": off});
        let camp = |extra: Value| {
            let mut camp = json!({"id": 1, "pos": origin, "founded_turn": 1, "guard": null,
                "scout": null, "next_spawn_turn": 1});
            for (k, v) in extra.as_object().unwrap() {
                camp[k] = v.clone();
            }
            json!([camp])
        };
        let threat = |unit: Value, pos: Value| json!([{"unit": unit, "owner": 1, "kind": "Spearman", "pos": pos, "seen_turn": 1}]);
        vec![
            (
                true,
                "city occupied_from",
                json!(200),
                city("occupied_from"),
            ),
            (
                true,
                "city greece_captive_paid",
                json!([200]),
                city("greece_captive_paid"),
            ),
            (true, "city grown_tiles", json!([off]), city("grown_tiles")),
            (
                true,
                "city wonder_lost",
                json!({"turn": 1, "winner": 200, "building": "Walls", "production": 0, "gold": 0}),
                city("wonder_lost"),
            ),
            (
                true,
                "city producing",
                wonder_off.clone(),
                city("producing"),
            ),
            (
                true,
                "city production_queue",
                json!([farm_off]),
                city("production_queue"),
            ),
            (
                true,
                "city production_banked",
                json!([{"item": farm_off, "progress": 1}]),
                city("production_banked"),
            ),
            (true, "city auto_queued", wonder_off, city("auto_queued")),
            (
                true,
                "player great_work_holder",
                json!(200),
                "players/0/great_work_holder".to_string(),
            ),
            (
                true,
                "player free_techs_from",
                json!(200),
                "players/0/free_techs_from".to_string(),
            ),
            (
                false,
                "player hall_city",
                json!(9_999_999),
                "players/0/hall_city".to_string(),
            ),
            (
                false,
                "player diwan_city",
                json!(9_999_999),
                "players/0/diwan_city".to_string(),
            ),
            (
                false,
                "player free_techs_captured",
                json!(9_999_999),
                "players/0/free_techs_captured".to_string(),
            ),
            (
                false,
                "player capitals_taken",
                json!({"9999999": "Nowhere"}),
                "players/0/capitals_taken".to_string(),
            ),
            (
                true,
                "player failed_city_sites",
                json!([off]),
                "players/0/failed_city_sites".to_string(),
            ),
            (
                true,
                "player observed_threats pos",
                threat(json!(1), off.clone()),
                "players/0/observed_threats".to_string(),
            ),
            (
                false,
                "player observed_threats unit",
                threat(json!(9_999_999), origin.clone()),
                "players/0/observed_threats".to_string(),
            ),
            (
                true,
                "player tile_memory city owner",
                json!({"owner": null, "ruin": false, "city": {"id": 1, "owner": 200, "name": "X",
                    "population": 1, "hp": 100, "walls": false, "is_capital": false}}),
                "players/0/tile_memory/0".to_string(),
            ),
            (
                false,
                "player supply",
                json!([{"user": {"kind": "unit", "unit": 9_999_999}, "resource": "Iron",
                    "since": 1}]),
                "players/0/supply".to_string(),
            ),
            (true, "unit goto", off.clone(), unit("goto")),
            (true, "unit path", json!([off]), unit("path")),
            (
                false,
                "unit attack_target",
                json!(9_999_999),
                unit("attack_target"),
            ),
            (
                false,
                "unit attack_revealed_to",
                json!([77]),
                unit("attack_revealed_to"),
            ),
            (
                true,
                "air_contacts key",
                json!({"200": []}),
                "air_contacts".to_string(),
            ),
            (
                true,
                "air_contacts owner",
                json!({"0": [{"owner": 200, "turn": 1}]}),
                "air_contacts".to_string(),
            ),
            (
                false,
                "air_contacts city",
                json!({"0": [{"owner": 1, "turn": 1, "city": 9_999_999}]}),
                "air_contacts".to_string(),
            ),
            (
                true,
                "wonders_discovered",
                json!([off]),
                "wonders_discovered".to_string(),
            ),
            (
                true,
                "tile landmark",
                json!({"building": "YamRelayStation", "city": 1, "of": 200}),
                "map/tiles/0/landmark".to_string(),
            ),
            (
                true,
                "barbarian camp pos",
                camp(json!({"pos": off})),
                "barbarians/camps".to_string(),
            ),
            (
                false,
                "barbarian camp raider",
                camp(json!({"raiders": [9_999_999]})),
                "barbarians/camps".to_string(),
            ),
            (
                false,
                "barbarian camp guard",
                camp(json!({"guard": 9_999_999})),
                "barbarians/camps".to_string(),
            ),
            (
                true,
                "barbarian camp rite",
                camp(json!({"rite": 200})),
                "barbarians/camps".to_string(),
            ),
            (
                false,
                "barbarian camp raid",
                camp(
                    json!({"raid": {"city": 9_999_999, "to_spawn": 0, "next_spawn_turn": 1,
                    "until_turn": 2}}),
                ),
                "barbarians/camps".to_string(),
            ),
            (
                false,
                "barbarian known",
                json!({"77": []}),
                "barbarians/known".to_string(),
            ),
            (
                false,
                "diplomacy relation",
                json!(77),
                "diplomacy/relations/0/a".to_string(),
            ),
            (
                false,
                "diplomacy declarers",
                json!([77]),
                "diplomacy/relations/0/declarers".to_string(),
            ),
            (
                false,
                "diplomacy contacts",
                json!([[0, 77]]),
                "diplomacy/contacts".to_string(),
            ),
            (
                true,
                "diplomacy proposal",
                json!([{"id": 1, "from": 0, "to": 1, "proposer": 200, "created_turn": 1,
                    "respond_by": 2, "relation_version": 0}]),
                "diplomacy/proposals".to_string(),
            ),
            (
                false,
                "diplomacy cooldown",
                json!([{"from": 0, "to": 77, "retry_turn": 2}]),
                "diplomacy/peace_cooldowns".to_string(),
            ),
            (
                true,
                "chronicle",
                json!([{"kind": "Goal", "turn": 1, "player": 200}]),
                "history/chronicle".to_string(),
            ),
        ]
    }

    fn slot<'a>(v: &'a mut Value, path: &str) -> &'a mut Value {
        path.split('/').fold(v, |v, key| match v {
            Value::Array(items) => &mut items[key.parse::<usize>().unwrap()],
            _ => &mut v[key],
        })
    }

    #[test]
    fn unknown_players_and_off_map_tiles_are_refused_but_stale_ids_load() {
        let base = saved();
        for (refused, name, value, path) in mutations(&base) {
            let mut save = base.clone();
            *slot(&mut save, &path) = value;
            let state: GameState = serde_json::from_value(save)
                .unwrap_or_else(|e| panic!("{name}: the mutated save must still decode: {e}"));
            match check_state(&state) {
                Err(error) if refused => assert!(
                    error.starts_with("The save is damaged: "),
                    "{name}: {error}"
                ),
                Ok(()) if !refused => {}
                other => panic!("{name}: expected refused={refused}, got {other:?}"),
            }
        }
    }

    /// The app's fixtures build these states (`hall.wasm.test.ts`, `actions.wasm.test.ts`): a
    /// Station record pointing at a city that no longer exists, and a team left with no empire
    /// after a player moves to another team. Both are reachable in play and must load.
    #[test]
    fn a_razed_hall_city_and_an_emptied_team_load() {
        let mut state: GameState = serde_json::from_value(saved()).unwrap();
        state.players[0].hall_city = Some(crate::types::CityId(state.next_city_id + 99));
        state.players[0].diwan_city = Some(crate::types::CityId(state.next_city_id + 77));
        state.players[2].team = state.players[1].team;
        state.setup.players[2].team = state.players[1].team;
        check_state(&state).expect("stale city and team references load");
    }
}

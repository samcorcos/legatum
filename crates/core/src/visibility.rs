//! What each team can currently see. Shared by resolution (explored tiles) and views (fog).

use crate::bitset::BitSet;
use crate::data::CITY_SIGHT;
use crate::hex::Hex;
use crate::los;
use crate::state::{CityMemory, GameState, ThreatObservation, TileMemory, Unit};
use crate::types::{PlayerId, TeamId};
use std::cmp::Reverse;
use std::collections::BTreeMap;

/// Observation retention, not extra sight or an assertion that a unit stayed put.
pub const THREAT_MEMORY_TURNS: u32 = 12;
pub const MAX_THREAT_OBSERVATIONS: usize = 128;

/// `team` sees what `other` sees: the same team, or an active Alliance (shared vision).
pub fn shares_team_sight(state: &GameState, team: TeamId, other: TeamId) -> bool {
    team == other
        || state.diplomacy.as_ref().is_some_and(|d| {
            d.relation(team, other)
                .is_some_and(crate::diplomacy::shares_sight)
        })
}

/// Whether an ordinary enemy unit is visible because it attacked this team last turn.
pub fn attack_revealed_to(state: &GameState, team: TeamId, unit: &Unit) -> bool {
    unit.attack_revealed_turn == state.turn.checked_sub(1)
        && unit.attack_revealed_to.contains(&team)
}

/// End of a turn's resolution: an attack reveal lasts one planning turn, then it is dropped.
pub fn expire_attack_reveals(state: &mut GameState) {
    let stale_attack_reveals: Vec<_> = state
        .units
        .values()
        .filter(|u| {
            u.attack_revealed_turn
                .is_some_and(|turn| turn.saturating_add(1) < state.turn)
        })
        .map(|u| u.id)
        .collect();
    for id in stale_attack_reveals {
        let unit = state.units.get_mut(&id).expect("attack reveal");
        unit.attack_revealed_turn = None;
        unit.attack_revealed_to.clear();
    }
}

/// Refresh exclusively from visible evidence. In particular, an unseen death or move
/// cannot invalidate a sighting. Seeing the old tile empty, or the unit elsewhere, can.
pub(crate) fn threat_observations(
    state: &GameState,
    me: PlayerId,
    seen: &BitSet,
) -> Vec<ThreatObservation> {
    let player = state.player(me);
    let visible = |pos| state.map.index(pos).is_some_and(|i| seen.get(i));
    let mut observations: BTreeMap<_, _> = player
        .observed_threats
        .iter()
        .filter(|o| {
            ((o.owner.0 as usize) < state.players.len() || o.owner.is_barbarian())
                && state.map.index(o.pos).is_some()
                && crate::data::unit_stats(o.kind).military
                && o.seen_turn <= state.turn
                && state.turn - o.seen_turn < THREAT_MEMORY_TURNS
                && !visible(o.pos)
        })
        .map(|o| (o.unit, o.clone()))
        .collect();
    for unit in state.units.values().filter(|u| visible(u.pos)) {
        // Forget a unit observed in a new location even if it is now friendly/civilian.
        observations.remove(&unit.id);
        if state.team_of(unit.owner) != player.team && crate::data::unit_stats(unit.kind).military {
            observations.insert(
                unit.id,
                ThreatObservation {
                    unit: unit.id,
                    owner: unit.owner,
                    kind: unit.kind,
                    pos: unit.pos,
                    seen_turn: state.turn,
                },
            );
        }
    }
    let mut observations: Vec<_> = observations.into_values().collect();
    observations.sort_by_key(|o| (Reverse(o.seen_turn), o.unit));
    observations.truncate(MAX_THREAT_OBSERVATIONS);
    observations.sort_by_key(|o| o.unit);
    observations
}

/// Capture the decision-time view before movement can hide a just-observed threat.
pub(crate) fn remember_threats(state: &mut GameState) {
    // Observations don't change anyone's sight: native sight once per team.
    let teams = state.teams();
    let natives = native_sight_by_team(state, &teams);
    for team in teams {
        let seen = visible_from_natives(state, team, &natives);
        let observations: Vec<_> = state
            .players
            .iter()
            .filter(|p| p.team == team)
            .map(|p| (p.id, threat_observations(state, p.id, &seen)))
            .collect();
        for (id, observations) in observations {
            state.player_mut(id).observed_threats = observations;
        }
    }
}

/// Cartography explores terrain and ruins without creating contact. Once a tile is explored,
/// public ownership and city identity are available in views; units and private facts are not.
pub fn reveal_map(state: &mut GameState, owner: crate::PlayerId, at: Hex, radius: i32) -> u32 {
    let indices: Vec<_> = at
        .within(radius)
        .into_iter()
        .filter_map(|h| state.map.index(h))
        .collect();
    let visible = native_visible_tiles(state, state.team_of(owner));
    let newly_explored = indices
        .iter()
        .filter(|&&i| !state.player(owner).explored.get(i) && !visible.get(i))
        .count() as u32;
    let team = state.team_of(owner);
    for player in state.players.iter_mut().filter(|p| p.team == team) {
        player.tile_memory.resize(state.map.len(), None);
        for &i in &indices {
            player.explored.set(i);
            let memory = player.tile_memory[i].get_or_insert(TileMemory {
                owner: None,
                ruin: false,
                improved: false,
                city: None,
            });
            memory.ruin = state.map.tiles[i].ruin;
        }
    }
    newly_explored
}

/// Recursive AI gives the researcher's team a current public map snapshot. This does not add
/// unit sight or contact: unseen units remain hidden and attacks still require visible targets.
pub fn reveal_all_to_team(state: &mut GameState, owner: PlayerId) {
    let team = state.team_of(owner);
    let cities: BTreeMap<_, _> = state
        .cities
        .values()
        .filter_map(|city| {
            state
                .map
                .index(city.pos)
                .map(|i| (i, CityMemory::from(city)))
        })
        .collect();
    for player in state.players.iter_mut().filter(|p| p.team == team) {
        player.tile_memory.resize(state.map.len(), None);
        for (i, tile) in state.map.tiles.iter().enumerate() {
            player.explored.set(i);
            player.tile_memory[i] = Some(TileMemory {
                owner: tile.owner,
                ruin: tile.ruin,
                improved: tile.improved,
                city: cities.get(&i).cloned(),
            });
        }
    }
}

/// Current terrain-blocked sight of `team`, including active allies' native sight.
/// Bits use `state.map.index(hex)`; this is current sight, not remembered exploration.
pub fn visible_tiles(state: &GameState, team: TeamId) -> BitSet {
    visible_with(state, team, native_visible_tiles(state, team), |ally| {
        native_visible_tiles(state, ally)
    })
}

/// `visible_tiles(state, team)` when the team's own `native_visible_tiles` is at hand.
pub(crate) fn visible_tiles_from_native(state: &GameState, team: TeamId, native: BitSet) -> BitSet {
    visible_with(state, team, native, |ally| {
        native_visible_tiles(state, ally)
    })
}

/// `native_visible_tiles` of each team.
fn native_sight_by_team(state: &GameState, teams: &[TeamId]) -> BTreeMap<TeamId, BitSet> {
    let sights = state.map.sight.natives(
        &state.map,
        teams
            .iter()
            .map(|&t| (t, sight_sources(state, t)))
            .collect(),
    );
    teams
        .iter()
        .copied()
        .zip(sights)
        .map(|(team, mut sight)| {
            add_territory_sight(state, team, &mut sight);
            add_air_sight(state, team, &mut sight);
            add_recon_sight(state, team, &mut sight);
            (team, sight)
        })
        .collect()
}

/// `visible_tiles(state, team)` from precomputed native sight (`native_sight_by_team`).
fn visible_from_natives(
    state: &GameState,
    team: TeamId,
    natives: &BTreeMap<TeamId, BitSet>,
) -> BitSet {
    let native_of = |t: TeamId| {
        natives
            .get(&t)
            .cloned()
            .unwrap_or_else(|| native_visible_tiles(state, t))
    };
    visible_with(state, team, native_of(team), native_of)
}

/// `visible_tiles` from this team's native sight and a source of its allies' native sight.
fn visible_with(
    state: &GameState,
    team: TeamId,
    native: BitSet,
    mut native_of: impl FnMut(TeamId) -> BitSet,
) -> BitSet {
    let mut seen = native;
    if let Some(d) = &state.diplomacy {
        for r in d
            .relations
            .iter()
            .filter(|r| (r.a == team || r.b == team) && crate::diplomacy::shares_sight(r))
        {
            let ally = if r.a == team { r.b } else { r.a };
            for i in native_of(ally).iter_ones() {
                seen.set(i);
            }
        }
    }
    seen
}

/// Native sight worked out on one map, shared by its clones (`Map::sight`): each team's sight
/// for the sources it was computed from, and each source's line-of-sight footprint by
/// (tile, radius). Terrain is the only map input to line of sight, so the memo holds a copy of
/// the terrain and starts over whenever any tile's terrain differs; a team's sight is reused
/// only for exactly the same sources. Units that haven't moved cost a lookup, and a turn's
/// repeated sight queries between movement rounds cost nothing. Debug builds recompute every
/// reused result and assert equality.
#[derive(Clone, Default)]
pub struct SightMemo(std::sync::Arc<std::sync::Mutex<SightState>>);

#[derive(Default)]
struct SightState {
    width: i32,
    terrain: Vec<crate::Terrain>,
    // gate-allow: memo cache, get/insert only, never iterated
    footprints: std::collections::HashMap<(Hex, i32), Vec<u32>>,
    // gate-allow: memo cache, get/insert only, never iterated
    teams: std::collections::HashMap<TeamId, (Vec<(Hex, i32)>, BitSet)>,
}

impl std::fmt::Debug for SightMemo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SightMemo")
    }
}

impl PartialEq for SightMemo {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for SightMemo {}

impl SightMemo {
    /// The union of the footprints of `sources` (tile, radius with any Hills bonus) on `map`.
    fn native(&self, map: &crate::state::Map, team: TeamId, sources: Vec<(Hex, i32)>) -> BitSet {
        self.natives(map, vec![(team, sources)])
            .pop()
            .expect("one team")
    }

    /// `native` for several teams, checking the terrain once.
    fn natives(
        &self,
        map: &crate::state::Map,
        teams: Vec<(TeamId, Vec<(Hex, i32)>)>,
    ) -> Vec<BitSet> {
        let mut memo = self.0.lock().expect("sight memo");
        if memo.width != map.width
            || memo.terrain.len() != map.tiles.len()
            || memo
                .terrain
                .iter()
                .zip(&map.tiles)
                .any(|(a, t)| *a != t.terrain)
        {
            *memo = SightState {
                width: map.width,
                terrain: map.tiles.iter().map(|t| t.terrain).collect(),
                ..Default::default()
            };
        }
        teams
            .into_iter()
            .map(|(team, sources)| {
                if let Some((known, seen)) = memo.teams.get(&team)
                    && *known == sources
                {
                    debug_assert_eq!(*seen, Self::union(map, &sources), "reused team sight");
                    return seen.clone();
                }
                let mut seen = BitSet::new(map.len());
                for &(pos, radius) in &sources {
                    let tiles = memo
                        .footprints
                        .entry((pos, radius))
                        .or_insert_with(|| Self::footprint(map, pos, radius));
                    debug_assert_eq!(*tiles, Self::footprint(map, pos, radius), "sight footprint");
                    for &i in tiles.iter() {
                        seen.set(i as usize);
                    }
                }
                memo.teams.insert(team, (sources, seen.clone()));
                seen
            })
            .collect()
    }

    fn union(map: &crate::state::Map, sources: &[(Hex, i32)]) -> BitSet {
        let mut seen = BitSet::new(map.len());
        for &(pos, radius) in sources {
            for i in Self::footprint(map, pos, radius) {
                seen.set(i as usize);
            }
        }
        seen
    }

    /// The tiles `pos` sees within `radius` (terrain-blocked line of sight), in axial order.
    fn footprint(map: &crate::state::Map, pos: Hex, radius: i32) -> Vec<u32> {
        let mut tiles = Vec::new();
        for target in pos.within_iter(radius) {
            if let Some(i) = map.index(target)
                && los::visible(pos, target, |h| map.get(h).map(|t| t.terrain))
            {
                tiles.push(i as u32);
            }
        }
        tiles
    }
}

/// Current native team sight: terrain-blocked unit/city footprints, plus unconditional sight of
/// owned territory and its adjacent ring. Borrowed allied sight is never forwarded.
pub fn native_visible_tiles(state: &GameState, team: TeamId) -> BitSet {
    let mut visible = state
        .map
        .sight
        .native(&state.map, team, sight_sources(state, team));
    add_territory_sight(state, team, &mut visible);
    add_air_sight(state, team, &mut visible);
    add_recon_sight(state, team, &mut visible);
    visible
}

/// An aircraft in a team's hangar (Biplane, Fighter, Bomber, the jets and the Stealth pair) lifts
/// the fog `data::AIRCRAFT_SIGHT` tiles around its city, from the air: terrain does not block it
/// (the spotting job of the Machine Age, 2026-11-14; every aircraft, 2026-11-15).
fn add_air_sight(state: &GameState, team: TeamId, visible: &mut BitSet) {
    for plane in state
        .aircraft
        .iter()
        .filter(|a| crate::data::is_aircraft(a.kind))
    {
        let Some(city) = state
            .cities
            .get(&plane.city)
            .filter(|c| state.team_of(c.owner) == team)
        else {
            continue;
        };
        for hex in city.pos.within_iter(crate::data::AIRCRAFT_SIGHT) {
            if let Some(index) = state.map.index(hex) {
                visible.set(index);
            }
        }
    }
}

/// An air reconnaissance ordered last turn (`Order::AirRecon`) lifts the fog `data::RECON_RADIUS`
/// tiles around its target for this turn only, from the air: terrain does not block it.
fn add_recon_sight(state: &GameState, team: TeamId, visible: &mut BitSet) {
    for recon in state
        .air_recons
        .iter()
        .filter(|r| r.turn.checked_add(1) == Some(state.turn) && state.team_of(r.owner) == team)
    {
        for hex in recon.target.within_iter(crate::data::RECON_RADIUS) {
            if let Some(index) = state.map.index(hex) {
                visible.set(index);
            }
        }
    }
}

/// Territory grants unconditional visibility through terrain and at any distance: every tile
/// the team owns, plus one adjacent ring. Applies to every team, AI teams included.
fn add_territory_sight(state: &GameState, team: TeamId, visible: &mut BitSet) {
    for (i, tile) in state.map.tiles.iter().enumerate() {
        if tile.owner.is_some_and(|owner| state.team_of(owner) == team) {
            visible.set(i);
            for neighbor in state.map.hex_at(i).neighbors() {
                if let Some(index) = state.map.index(neighbor) {
                    visible.set(index);
                }
            }
        }
    }
}

/// A team's sight sources: (tile, sight radius including the Hills bonus), units then cities.
fn sight_sources(state: &GameState, team: TeamId) -> Vec<(Hex, i32)> {
    let sources = state
        .units
        .values()
        .filter(|u| state.team_of(u.owner) == team)
        .map(|u| {
            (
                u.pos,
                crate::data::UNIT_SIGHT
                    + i32::from(
                        !u.owner.is_barbarian()
                            && crate::data::is_medjay_rules(
                                state.setup.rule_experiments.faction_rules_off,
                                state.made_by(u),
                                u.kind,
                            ),
                    ),
            )
        })
        .chain(
            state
                .cities
                .values()
                .filter(|c| state.team_of(c.owner) == team)
                .map(|c| (c.pos, CITY_SIGHT)),
        );
    sources
        .map(|(pos, base_radius)| {
            let hills = state
                .map
                .get(pos)
                .is_some_and(|t| t.terrain == crate::Terrain::Hills);
            (pos, base_radius + i32::from(hills))
        })
        .collect()
}

/// Team pairs (lower id first) in contact right now: one team's own units or cities see a unit,
/// city or owned tile of the other. Contact is mutual, as in Civ 6: seeing them means meeting
/// them. Only native sight counts (an ally's borrowed sight introduces nobody) and barbarians are
/// never met.
pub fn contact_pairs(state: &GameState) -> std::collections::BTreeSet<(TeamId, TeamId)> {
    let mut pairs = std::collections::BTreeSet::new();
    for team in state.teams() {
        let seen = native_visible_tiles(state, team);
        team_contacts(state, team, &seen, &mut pairs);
    }
    pairs
}

/// The pairs `team` is in contact with through its own sight `seen` (see `contact_pairs`).
fn team_contacts(
    state: &GameState,
    team: TeamId,
    seen: &BitSet,
    pairs: &mut std::collections::BTreeSet<(TeamId, TeamId)>,
) {
    let visible = |pos: Hex| state.map.index(pos).is_some_and(|i| seen.get(i));
    let mut meet = |owner: crate::PlayerId| {
        if owner.is_barbarian() {
            return;
        }
        let other = state.team_of(owner);
        if other != team {
            pairs.insert((team.min(other), team.max(other)));
        }
    };
    for unit in state.units.values() {
        if visible(unit.pos) {
            meet(unit.owner);
        }
    }
    for city in state.cities.values() {
        if visible(city.pos) {
            meet(city.owner);
        }
    }
    for i in seen.iter_ones() {
        if let Some(owner) = state.map.tiles[i].owner {
            meet(owner);
        }
    }
}

/// Record only currently visible tiles in each player's exploration and memory (shared vision).
pub fn update_explored(state: &mut GameState) {
    let mut cities = vec![None; state.map.len()];
    for city in state.cities.values() {
        if let Some(i) = state.map.index(city.pos) {
            cities[i] = Some(CityMemory::from(city));
        }
    }
    let mut contacts = std::collections::BTreeSet::new();
    // Native sight depends on units, cities, terrain and techs, which this pass doesn't
    // change: compute it once per team, for the team itself and for its allies' shared sight.
    let teams = state.teams();
    let natives = native_sight_by_team(state, &teams);
    for team in teams {
        let native = natives[&team].clone();
        let seen = visible_from_natives(state, team, &natives);
        let visible_aircraft: Vec<_> = state
            .aircraft
            .iter()
            .filter_map(|a| {
                let city = state.cities.get(&a.city)?;
                (state.team_of(city.owner) != team
                    && state.map.index(city.pos).is_some_and(|i| seen.get(i)))
                .then_some((city.owner, a.kind, a.city))
            })
            .collect();
        if let Some(observer) = state.players.iter().find(|p| p.team == team).map(|p| p.id) {
            let turn = state.turn;
            for (owner, kind, city) in visible_aircraft {
                crate::rules::remember_air_contact(
                    state,
                    observer,
                    owner,
                    Some(kind),
                    turn,
                    Some(city),
                );
            }
        }
        team_contacts(state, team, &native, &mut contacts);
        let observations: BTreeMap<_, _> = state
            .players
            .iter()
            .filter(|p| p.team == team)
            .map(|p| (p.id, threat_observations(state, p.id, &seen)))
            .collect();
        for player in state.players.iter_mut().filter(|p| p.team == team) {
            player.observed_threats = observations[&player.id].clone();
            // Never fill unseen fog from live state.
            player.tile_memory.resize(state.map.len(), None);
            // Ascending map indices, just like the full scan. Public ownership and city
            // identity change on explored land; private observations change only in sight.
            for i in seen.iter_ones() {
                let tile = &state.map.tiles[i];
                if native.get(i) {
                    player.explored.set(i);
                }
                player.tile_memory[i] = Some(TileMemory {
                    owner: tile.owner,
                    ruin: tile.ruin,
                    improved: tile.improved,
                    city: cities[i].clone(),
                });
            }
            for (i, tile) in state.map.tiles.iter().enumerate() {
                if seen.get(i) || !(player.explored.get(i) || player.tile_memory[i].is_some()) {
                    continue;
                }
                let memory = player.tile_memory[i].get_or_insert(TileMemory {
                    owner: None,
                    ruin: false,
                    improved: false,
                    city: None,
                });
                memory.owner = tile.owner;
                memory.city = cities[i]
                    .as_ref()
                    .map(|current| CityMemory::with_public_identity(current, memory.city.as_ref()));
            }
        }
        if let Some(barbarians) = state.barbarians.as_mut() {
            barbarians.remember(team, &seen, &state.map);
        }
    }
    // First contact is permanent: an Alliance needs it (`diplomacy::check_order`).
    if let Some(known) = state.diplomacy.as_mut().and_then(|d| d.contacts.as_mut()) {
        known.extend(contacts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CityId, Faction, GameSetup, Order, Terrain, UnitKind,
        diplomacy::RelationState,
        game::{blank_game, spawn_unit},
        state::{City, Map},
        view::{ViewDetail, player_view_detail, public_view},
    };

    const P0: PlayerId = PlayerId(0);
    const P1: PlayerId = PlayerId(1);

    fn city(state: &mut GameState, owner: PlayerId, pos: Hex) {
        let id = CityId(state.cities.len() as u32);
        state.cities.insert(
            id,
            City {
                id,
                owner,
                founder: owner,
                name: format!("City {}", id.0),
                pos,
                population: 1,
                food_stored: 0,
                production_stored: 0,
                producing: None,
                production_banked: Vec::new(),
                production_queue: Vec::new(),
                build_automation: None,
                last_auto_pick: None,
                auto_queued: None,
                production_queue_notified: Default::default(),
                wonder_lost: None,
                last_purchase_turn: None,
                bought: None,
                buildings: Default::default(),
                hp: crate::data::CITY_HP,
                last_attacked_turn: None,
                tiles_claimed: 0,
                grown_tiles: Vec::new(),
                is_capital: true,
                original_capital: true,
                greece_captive_paid: Default::default(),
                occupied_from: None,
                founded_turn: 1,
                resistance: 0,
            },
        );
    }

    #[test]
    fn owned_land_and_its_ring_ignore_hills_and_forest() {
        let setup = GameSetup::standard(7, [Faction::India, Faction::Germany]);
        let mut state = blank_game(&setup, Map::new(40, 24, Terrain::Grassland));
        let owned = Hex::new(20, 12);
        let ring = Hex::new(21, 12);
        let far = Hex::new(22, 12);
        let blocker = Hex::new(19, 12);
        spawn_unit(&mut state, P0, UnitKind::Scout, Hex::new(18, 12));
        state.map.get_mut(owned).unwrap().owner = Some(P0);
        state.map.get_mut(owned).unwrap().terrain = Terrain::Hills;
        state.map.get_mut(blocker).unwrap().terrain = Terrain::Forest;
        state.map.get_mut(ring).unwrap().terrain = Terrain::Forest;
        assert!(
            native_visible_tiles(&state, state.team_of(P0)).get(state.map.index(owned).unwrap())
        );
        assert!(
            native_visible_tiles(&state, state.team_of(P0)).get(state.map.index(ring).unwrap())
        );
        assert!(
            !native_visible_tiles(&state, state.team_of(P0)).get(state.map.index(far).unwrap())
        );
    }

    #[test]
    fn celtic_archers_are_seen_inside_rival_territory_like_any_unit() {
        let mut setup = GameSetup::standard(7, [Faction::India, Faction::Celts]);
        setup.players[1].team = crate::TeamId(1);
        let mut state = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
        let pos = Hex::new(12, 8);
        state.map.get_mut(pos).unwrap().owner = Some(P0);
        let hunter = spawn_unit(&mut state, P1, UnitKind::Crossbowman, pos);
        let view = crate::player_view(&state, P0);
        assert!(view.tile(pos).unwrap().visible);
        assert!(view.units.iter().any(|unit| unit.id == hunter));
    }

    #[test]
    fn ai_and_human_views_share_territory_visibility() {
        let mut setup = GameSetup::standard(7, [Faction::India, Faction::Germany]);
        setup.players[1].team = crate::TeamId(1);
        let mut state = blank_game(&setup, Map::new(40, 24, Terrain::Grassland));
        let owned = Hex::new(20, 12);
        state.map.get_mut(owned).unwrap().owner = Some(P0);
        spawn_unit(&mut state, P1, UnitKind::Crossbowman, Hex::new(21, 12));
        let public = public_view(&state);
        let human = player_view_detail(&state, P0, &public, ViewDetail::Full);
        let ai = player_view_detail(&state, P0, &public, ViewDetail::Decisions);
        assert_eq!(human.tiles, ai.tiles);
        assert_eq!(human.units, ai.units);
        assert!(human.units.iter().any(|u| u.owner == P1));
    }

    #[test]
    fn ordinary_ranged_attacker_is_visible_for_the_victim_turn() {
        let mut setup = GameSetup::standard(7, [Faction::India, Faction::Germany]);
        setup.players[1].team = crate::TeamId(1);
        let mut state = blank_game(&setup, Map::new(40, 24, Terrain::Grassland));
        state.diplomacy.as_mut().unwrap().relations[0].state = RelationState::War;
        let victim = Hex::new(20, 12);
        let screen = Hex::new(21, 12);
        let shooter = Hex::new(22, 12);
        city(&mut state, P0, Hex::new(3, 3));
        city(&mut state, P1, Hex::new(36, 20));
        state.map.get_mut(screen).unwrap().terrain = Terrain::Forest;
        state.map.get_mut(shooter).unwrap().terrain = Terrain::Hills;
        spawn_unit(&mut state, P0, UnitKind::Spearman, victim);
        let attacker = spawn_unit(&mut state, P1, UnitKind::Archer, shooter);
        let (next, events) = crate::resolve_turn(
            &state,
            &[(
                P1,
                vec![Order::RangedAttack {
                    unit: attacker,
                    target: victim,
                }],
            )]
            .into(),
        );
        let view = crate::player_view(&next, P0);
        assert!(!view.tile(shooter).unwrap().visible);
        assert!(
            view.units.iter().any(|unit| unit.id == attacker),
            "events: {events:#?}"
        );
        let report = crate::event_visibility::visible_events(
            &events,
            &crate::player_view(&state, P0),
            &view,
        );
        assert!(report.iter().any(|event| matches!(event,
            crate::event_visibility::VisibleEvent::Complete(crate::Event::Battle {
                attacker_at: Some(at), ..
            }) if *at == shooter
        )));
        let (later, _) = crate::resolve_turn(&next, &Default::default());
        assert!(
            !crate::player_view(&later, P0)
                .units
                .iter()
                .any(|unit| unit.id == attacker)
        );
    }
}

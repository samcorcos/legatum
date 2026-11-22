//! Determinization: a full `GameState` built only from one seat's `PlayerView`, so rollouts
//! never read hidden state (pillar 6).
//!
//! What the seat knows is copied exactly: its own empire (treasury, research, cities with
//! their production, units with their orders), every tile it has explored, every city and
//! unit it sees or remembers, public diplomacy and great works. What it
//! doesn't know is sampled from public facts and the map-generation prior, the same way for
//! every candidate (common random numbers):
//! - unexplored terrain, ruins and resources come from a map generated with the sample seed;
//! - a rival's unseen cities (its public city count) are placed on unseen land near what we
//!   know of it, sized from observed cities;
//! - unseen rival units: remembered sightings stay where they were last seen; wartime rivals
//!   and the top threat place a weighted share of the remaining public army toward a front,
//!   and the rest is garrisoned at cities;
//! - rival treasuries scale ours by their public economy value; rival technology is
//!   approximated from our known technologies;
//! - rival cities hold no buildings beyond the Walls and Castles we've seen.

use crate::winchance::WinFacts;
use legatum_core::bitset::BitSet;
use legatum_core::data;
use legatum_core::diplomacy::DiplomacyState;
use legatum_core::rng::Rng;
use legatum_core::rules::barbarians::{BarbarianState, Camp};
use legatum_core::state::{City, CityMemory, Map, TileMemory, Unit};
use legatum_core::view::{CityView, UnitView};
use legatum_core::{
    AiDifficulty, BuildingKind, CityId, GameSetup, GameState, Hex, PlayerId, PlayerSetup,
    PlayerView, TeamId, TechId, UnitId, UnitKind, game, mapgen,
};
use std::collections::{BTreeMap, BTreeSet};

/// RNG stream for determinization draws.
const STREAM: u64 = 0x5EA4_C401;

/// A map-generation sample: terrain for unexplored tiles and plausible start positions.
pub struct Prior {
    pub map: Map,
    pub starts: Vec<Hex>,
}

/// The sampled planet for unexplored tiles: a fresh map of the same size and setup.
pub fn prior_map(view: &PlayerView, seed: u64) -> Prior {
    let (map, starts) = mapgen::generate(&setup_for(view, seed));
    Prior { map, starts }
}

fn setup_for(view: &PlayerView, seed: u64) -> GameSetup {
    GameSetup {
        rule_experiments: view.rule_experiments,
        seed,
        width: view.width,
        height: view.height,
        players: view
            .players
            .iter()
            .map(|p| PlayerSetup {
                name: p.name.clone(),
                faction: p.faction,
                team: p.team,
                is_human: p.is_human,
                difficulty: if p.is_human {
                    AiDifficulty::Normal
                } else {
                    AiDifficulty::Hard
                },
                // Rollout seats are plain scripted Hard: search never nests.
                search: false,
            })
            .collect(),
        turn_limit: view.turn_limit,
        legacy_all_war: view.diplomacy.legacy,
        // Lobby settings aren't in the view; barbarian camps are on in every current game.
        barbarians: !view.diplomacy.legacy,
    }
}

/// A full state consistent with `view`, unknowns drawn with `seed` over `prior`.
pub fn determinize(view: &PlayerView, prior: &Prior, seed: u64) -> GameState {
    determinize_with(
        view,
        prior,
        seed,
        crate::DecisionParams::default().fog_army_forward_percent,
        None,
    )
}

pub(crate) fn determinize_with(
    view: &PlayerView,
    prior: &Prior,
    seed: u64,
    forward_percent: i32,
    top_threat: Option<PlayerId>,
) -> GameState {
    let setup = setup_for(view, seed);
    let mut rng = Rng::derive(seed, &[STREAM, view.turn as u64, view.me.0 as u64]);
    let me = view.me;
    let team = view.team;
    let n = (view.width * view.height) as usize;

    // Terrain: what we've explored, else the prior.
    let mut map = prior.map.clone();
    // Unknown landmarks must not create speculative discovery payouts in rollouts.
    // Reapply only the wonders this seat has actually charted below.
    for tile in &mut map.tiles {
        tile.wonder = None;
    }
    let my_techs: BTreeSet<TechId> = view.research.known.iter().copied().collect();
    for (i, t) in view.tiles.iter().enumerate().take(n) {
        let tile = &mut map.tiles[i];
        if let Some(terrain) = t.terrain {
            tile.terrain = terrain;
            tile.ruin = t.ruin;
            tile.wonder = t.wonder;
            tile.owner = t.owner;
            if t.resource.is_some()
                || legatum_core::resources::visible_resource(tile, true, &my_techs).is_some()
            {
                tile.resource = t.resource;
            }
        } else {
            tile.owner = None;
        }
        tile.city = None;
    }

    let mut state = game::blank_game(&setup, map);
    // An explored wonder has already had its global first-sighting reward in the real game.
    for (i, tile) in view.tiles.iter().enumerate() {
        if tile.wonder.is_some() {
            state.wonders_discovered.insert(state.map.hex_at(i));
        }
    }
    state.turn = view.turn;
    if !view.air_contacts.is_empty() {
        state.air_contacts.insert(me, view.air_contacts.clone());
    }
    state.winner = view.winner;
    state.finished = view.finished;
    for p in &view.players {
        let player = state.player_mut(p.id);
        player.alive = p.alive;
        player.faction_goal_done = p.faction_goal_done;
        // Sakoku is public: a rollout must keep a closed country closed and counting.
        player.sakoku_turns = p.sakoku_turns;
        player.goal_rule = p.goal_rule;
    }
    for g in &view.great_works {
        if let Some(b) = g.completed {
            state.player_mut(g.player).great_work_completed = Some(b);
        }
    }
    let history = legatum_core::data::SCIENCE_HISTORY_TURNS;
    for p in &view.players {
        let sum = view.advancement_of(p.id);
        let each = sum / history as i32;
        let mut h = vec![each; history];
        if let Some(last) = h.last_mut() {
            *last += sum - each * history as i32;
        }
        state.player_mut(p.id).science_history = h;
    }

    // Our own empire, exactly.
    {
        let player = state.player_mut(me);
        player.gold = view.economy.gold;
        player.military_purchases = view.economy.military_purchases;
        player.science_stored = view.research.progress;
        player.researching = view.research.current;
        player.research_queue = view.research.queue.clone();
        player.techs = my_techs.clone();
        player.failed_city_sites = view.failed_city_sites.clone();
        player.observed_threats = view.observed_threats.clone();
        player.supply = own_supply(view);
    }

    // Rival treasuries (public) and technology, from public facts.
    for p in view.players.iter().filter(|p| p.team != team) {
        let player = state.player_mut(p.id);
        player.gold = p.gold.max(0);
        player.techs = my_techs.clone();
        player.settlers_trained = (p.cities as i32 - 1).max(0);
    }
    // Teammates share our research view only through what's public: treat like rivals.

    // Cities: our own with full detail, every other city we see or remember.
    let mut cities: BTreeMap<CityId, City> = BTreeMap::new();
    for c in &view.cities {
        cities.insert(c.id, city_from_view(c, view.turn, view.me));
    }
    let mut next_city = view.cities.iter().map(|c| c.id.0 + 1).max().unwrap_or(0);
    // Unseen rival cities, by public count.
    let seen: Vec<bool> = view.tiles.iter().map(|t| t.visible).collect();
    let (width, height) = (view.width, view.height);
    let visible_tile = |h: Hex| -> bool {
        let (c, r) = h.to_offset();
        c >= 0 && r >= 0 && c < width && r < height && seen[(r * width + c) as usize]
    };
    for p in view.players.iter().filter(|p| p.alive && p.team != team) {
        let known: Vec<Hex> = cities
            .values()
            .filter(|c| c.owner == p.id)
            .map(|c| c.pos)
            .collect();
        let missing = (p.cities as usize).saturating_sub(known.len());
        if missing == 0 {
            continue;
        }
        let known_pop: i32 = cities
            .values()
            .filter(|c| c.owner == p.id)
            .map(|c| c.population)
            .sum();
        let pop_each = (known_pop / known.len().max(1) as i32).max(1);
        let anchors: Vec<Hex> = known
            .iter()
            .copied()
            .chain(view.units.iter().filter(|u| u.owner == p.id).map(|u| u.pos))
            .chain(
                view.observed_threats
                    .iter()
                    .filter(|o| o.owner == p.id)
                    .map(|o| o.pos),
            )
            .chain(
                view.tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.owner == Some(p.id))
                    .map(|(i, _)| state.map.hex_at(i)),
            )
            .collect();
        for k in 0..missing {
            let Some(pos) = city_site(&state, &cities, &anchors, &visible_tile, view, &mut rng)
            else {
                break;
            };
            let id = CityId(next_city);
            next_city += 1;
            let capital = known.is_empty() && k == 0;
            cities.insert(
                id,
                City {
                    last_attacked_turn: None,
                    id,
                    owner: p.id,
                    founder: p.id,
                    name: format!("Unseen {}", id.0),
                    pos,
                    population: pop_each,
                    food_stored: 0,
                    production_stored: 0,
                    producing: None,
                    production_banked: Vec::new(),
                    production_queue: Vec::new(),
                    build_automation: None,
                    last_auto_pick: None,
                    auto_queued: None,
                    production_queue_notified: BTreeSet::new(),
                    wonder_lost: None,
                    last_purchase_turn: None,
                    bought: None,
                    buildings: BTreeSet::new(),
                    hp: data::CITY_HP,
                    tiles_claimed: 0,
                    grown_tiles: Vec::new(),
                    is_capital: capital,
                    original_capital: capital,
                    greece_captive_paid: Default::default(),
                    occupied_from: None,
                    founded_turn: 1,
                    resistance: 0,
                },
            );
        }
    }
    state.next_city_id = next_city;

    // Territory: our own cities' lists; every other known owned tile goes to its owner's
    // nearest city, and an unseen city claims its unowned surroundings by population.
    for c in view.cities.iter().filter(|c| c.owner == me) {
        if let Some(d) = &c.detail {
            for h in &d.territory {
                if let Some(t) = state.map.get_mut(*h) {
                    t.owner = Some(me);
                    t.city = Some(c.id);
                }
            }
        }
    }
    for i in 0..n {
        let h = state.map.hex_at(i);
        let Some(owner) = state.map.tiles[i].owner else {
            continue;
        };
        if state.map.tiles[i].city.is_some() {
            continue;
        }
        let nearest = cities
            .values()
            .filter(|c| c.owner == owner)
            .min_by_key(|c| (c.pos.distance(h), c.id))
            .map(|c| c.id);
        match nearest {
            Some(id) => state.map.tiles[i].city = Some(id),
            None => state.map.tiles[i].owner = None,
        }
    }
    for c in cities.values() {
        if state.map.get(c.pos).is_some_and(|t| t.city.is_none()) {
            let t = state.map.get_mut(c.pos).expect("city tile");
            t.owner = Some(c.owner);
            t.city = Some(c.id);
        }
    }
    let unseen: Vec<(CityId, PlayerId, Hex, i32)> = cities
        .values()
        .filter(|c| c.name.starts_with("Unseen "))
        .map(|c| (c.id, c.owner, c.pos, c.population))
        .collect();
    for (id, owner, pos, pop) in unseen {
        let mut claim: Vec<Hex> = pos.within(2);
        claim.sort_by_key(|h| (h.distance(pos), *h));
        let mut left = 6 + pop;
        for h in claim {
            if left == 0 {
                break;
            }
            if let Some(t) = state.map.get_mut(h)
                && t.owner.is_none()
                && !visible_tile(h)
            {
                t.owner = Some(owner);
                t.city = Some(id);
                left -= 1;
            }
        }
    }
    for c in cities.values_mut() {
        let owned = state
            .map
            .tiles
            .iter()
            .filter(|t| t.city == Some(c.id))
            .count() as i32;
        c.tiles_claimed = (owned - 7).max(0);
    }
    state.cities = cities;

    // Units: ours exactly, every visible unit as seen, then remembered sightings.
    let mut occupied: BTreeSet<Hex> = BTreeSet::new();
    for u in &view.units {
        state.units.insert(u.id, unit_from_view(u, view));
        occupied.insert(u.pos);
    }
    let mut next_unit = view.units.iter().map(|u| u.id.0 + 1).max().unwrap_or(0);
    let city_owner = |s: &GameState, h: Hex| s.city_at(h).map(|c| c.owner);
    for o in view.observed_threats.iter() {
        next_unit = next_unit.max(o.unit.0 + 1);
        if state.units.contains_key(&o.unit)
            || occupied.contains(&o.pos)
            || visible_tile(o.pos)
            || !o.owner.is_barbarian() && !state.player(o.owner).alive
            || state.map.move_cost(o.pos).is_none()
            || city_owner(&state, o.pos).is_some_and(|c| c != o.owner)
        {
            continue;
        }
        let unit = fresh_unit(&state, o.unit, o.owner, o.kind, o.pos);
        state.units.insert(o.unit, unit);
        occupied.insert(o.pos);
    }
    // A surviving rival that has lost every city must still have a Settler somewhere.
    // Model one hidden survivor near known positions or the sampled start.
    for p in view
        .players
        .iter()
        .filter(|p| p.alive && p.team != team && p.cities == 0)
    {
        if view
            .units
            .iter()
            .any(|u| u.owner == p.id && u.kind == UnitKind::Settler)
        {
            continue;
        }
        let anchors: Vec<Hex> = view
            .units
            .iter()
            .filter(|u| u.owner == p.id)
            .map(|u| u.pos)
            .collect();
        let start = prior
            .starts
            .get(p.id.0 as usize)
            .copied()
            .filter(|h| {
                anchors.is_empty()
                    && !visible_tile(*h)
                    && !occupied.contains(h)
                    && state.map.move_cost(*h).is_some()
            })
            .or_else(|| {
                city_site(
                    &state,
                    &state.cities,
                    &anchors,
                    &visible_tile,
                    view,
                    &mut rng,
                )
            });
        let Some(start) = start else {
            continue;
        };
        let mut spots = vec![start];
        let mut ring: Vec<Hex> = start.within(2);
        ring.sort_by_key(|h| (h.distance(start), *h));
        spots.extend(ring);
        let mut left = 1;
        for h in spots {
            if left == 0 {
                break;
            }
            if occupied.contains(&h) || visible_tile(h) || state.map.move_cost(h).is_none() {
                continue;
            }
            let id = UnitId(next_unit);
            next_unit += 1;
            state
                .units
                .insert(id, fresh_unit(&state, id, p.id, UnitKind::Settler, h));
            occupied.insert(h);
            left -= 1;
        }
    }
    // The rest of each rival's public army: a share of an active threat is plausibly
    // forward in fog, with the remainder at its cities (or its Settlers). Only the
    // observer's public army strength, diplomacy and sight determine this prior.
    for p in view.players.iter().filter(|p| p.alive && p.team != team) {
        let army = view.army_of(p.id);
        let placed: i32 = state
            .units
            .values()
            .filter(|u| u.owner == p.id)
            .map(|u| army_value(&state, u))
            .sum();
        let mut gap = army - placed;
        let mut kinds: Vec<UnitKind> = state
            .units
            .values()
            .filter(|u| u.owner == p.id && data::unit_stats(u.kind).military)
            .filter(|u| u.kind != UnitKind::Scout)
            .map(|u| u.kind)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if kinds.is_empty() {
            kinds.push(UnitKind::Soldier);
        }
        let mut spots: Vec<Hex> = Vec::new();
        let mut homes: Vec<&City> = state.cities.values().filter(|c| c.owner == p.id).collect();
        homes.sort_by_key(|c| (!c.is_capital, std::cmp::Reverse(c.population), c.id));
        let mut home_hexes: Vec<Hex> = homes.iter().map(|c| c.pos).collect();
        if home_hexes.is_empty() {
            home_hexes = state
                .units
                .values()
                .filter(|u| u.owner == p.id && u.kind == UnitKind::Settler)
                .map(|u| u.pos)
                .collect();
        }
        for ring in 0..=2 {
            for c in &home_hexes {
                let mut ring_hexes = if ring == 0 { vec![*c] } else { c.ring(ring) };
                ring_hexes.sort();
                spots.extend(ring_hexes);
            }
        }
        let front = if view.at_war(me, p.id) || top_threat == Some(p.id) {
            view.cities
                .iter()
                .filter(|c| c.owner == me)
                .min_by_key(|c| {
                    (
                        home_hexes
                            .iter()
                            .map(|h| h.distance(c.pos))
                            .min()
                            .unwrap_or(i32::MAX),
                        c.id,
                    )
                })
                .map(|c| c.pos)
        } else {
            let opponents: BTreeSet<_> = view
                .diplomacy
                .relations
                .iter()
                .filter(|r| r.state == legatum_core::diplomacy::RelationState::War)
                .filter_map(|r| {
                    if r.a == p.team {
                        Some(r.b)
                    } else if r.b == p.team {
                        Some(r.a)
                    } else {
                        None
                    }
                })
                .collect();
            view.cities
                .iter()
                .filter(|c| opponents.contains(&view.team_of(c.owner)))
                .min_by_key(|c| {
                    (
                        home_hexes
                            .iter()
                            .map(|h| h.distance(c.pos))
                            .min()
                            .unwrap_or(i32::MAX),
                        c.id,
                    )
                })
                .map(|c| c.pos)
        };
        let mut forward_spots: Vec<Hex> = if forward_percent > 0 {
            front.map_or_else(Vec::new, |target| {
                let mut candidates: Vec<_> = (0..state.map.tiles.len())
                    .map(|i| state.map.hex_at(i))
                    .filter(|h| {
                        !visible_tile(*h)
                            && state.map.move_cost(*h).is_some()
                            && !occupied.contains(h)
                            && city_owner(&state, *h).is_none_or(|o| o == p.id)
                            && home_hexes
                                .iter()
                                .any(|home| h.distance(*home) <= home.distance(target))
                    })
                    .collect();
                candidates.sort_by_key(|h| {
                    (
                        h.distance(target),
                        home_hexes
                            .iter()
                            .map(|home| home.distance(*h))
                            .min()
                            .unwrap_or(i32::MAX),
                        *h,
                    )
                });
                candidates
            })
        } else {
            Vec::new()
        };
        // A seed-dependent tie rotation gives the two rollout samples different legal
        // positions without biasing them away from the front.
        if forward_spots.len() > 1 {
            let first_distance = front
                .map(|target| forward_spots[0].distance(target))
                .unwrap_or(0);
            let tied = forward_spots
                .iter()
                .take_while(|h| front.is_some_and(|target| h.distance(target) == first_distance))
                .count();
            if tied > 1 {
                forward_spots[..tied].rotate_left(rng.range(0, tied as i64 - 1) as usize);
            }
        }
        let mut forward_gap = gap.max(0) * forward_percent.clamp(0, 100) / 100;
        let mut k = 0usize;
        for (h, forward) in forward_spots
            .into_iter()
            .map(|h| (h, true))
            .chain(spots.into_iter().map(|h| (h, false)))
        {
            if forward && forward_gap <= 0 {
                continue;
            }
            let kind = kinds[k % kinds.len()];
            let id = UnitId(next_unit);
            let unit = fresh_unit(&state, id, p.id, kind, h);
            // Unknown troops carry no inferred XP. Price the reconstructed unit itself,
            // rather than a newly trained Spartan (whose known starting level is one).
            let value = army_value(&state, &unit);
            if gap * 2 < value || value <= 0 {
                break;
            }
            if occupied.contains(&h)
                || visible_tile(h)
                || state.map.move_cost(h).is_none()
                || city_owner(&state, h).is_some_and(|c| c != p.id)
            {
                continue;
            }
            next_unit += 1;
            state.units.insert(id, unit);
            occupied.insert(h);
            gap -= value;
            if forward {
                forward_gap -= value;
            }
            k += 1;
        }
    }
    state.next_unit_id = next_unit;

    // Settler costs: invert our quoted Settler price for the number already trained.
    if let Some(cost) = view
        .cities
        .iter()
        .filter_map(|c| c.detail.as_ref())
        .flat_map(|d| {
            d.options
                .iter()
                .map(|o| (o.item, o.cost))
                .chain(d.unavailable.iter().map(|o| (o.item, o.cost)))
        })
        .find(|(item, _)| {
            matches!(
                item,
                legatum_core::ProductionItem::Unit {
                    unit: UnitKind::Settler
                }
            )
        })
    {
        let (item, cost) = cost;
        for trained in 0..40 {
            state.player_mut(me).settlers_trained = trained;
            if legatum_core::economy::production_cost(&state, me, item) >= cost {
                break;
            }
        }
    }

    // Diplomacy, as public.
    if !view.diplomacy.legacy {
        let d = &view.diplomacy;
        let teams: BTreeSet<TeamId> = view.players.iter().map(|p| p.team).collect();
        let mut s = DiplomacyState::new(teams.iter().copied());
        s.relations = d.relations.clone();
        // These are view-only fields: the state derives them from the Alliances.
        s.proposals = d
            .proposals
            .iter()
            .map(|p| legatum_core::diplomacy::PeaceProposal {
                needs: Vec::new(),
                required_teams: Vec::new(),
                ..p.clone()
            })
            .collect();
        s.replies = d.replies.clone();
        s.betrayals = d.betrayals.clone();
        s.superseded = d.superseded.clone();
        s.peace_cooldowns = d.peace_cooldowns.clone();
        // The rules the game runs under: this team's open calls are public to it, the others' unknown.
        s.calls = d.calls.clone();
        s.next_proposal_id = d
            .proposals
            .iter()
            .map(|p| p.id.0 + 1)
            .chain(d.calls.iter().map(|c| c.id.0 + 1))
            .max()
            .unwrap_or(0)
            .max(
                d.replies
                    .iter()
                    .map(|r| r.proposal.id.0 + 1)
                    .max()
                    .unwrap_or(0),
            )
            + 1000;
        s.next_war_id = d
            .relations
            .iter()
            .filter_map(|r| r.war_id)
            .map(|w| w + 1)
            .max()
            .unwrap_or(0)
            + 1000;
        s.next_treaty_id = 1_000_000;
        let mut met = BTreeSet::new();
        for (i, a) in teams.iter().enumerate() {
            for b in teams.iter().skip(i + 1) {
                let pair = (*a.min(b), *a.max(b));
                if (*a != team && *b != team) || d.met.contains(if *a == team { b } else { a }) {
                    met.insert(pair);
                }
            }
        }
        s.contacts = Some(met);
        state.diplomacy = Some(s);
    }

    // Barbarian camps we know of.
    if setup.barbarians {
        let mut barb = BarbarianState::default();
        for (i, camp) in view.barbarian_camps.iter().enumerate() {
            let guard = state
                .units
                .values()
                .find(|u| u.pos == camp.pos && u.owner.is_barbarian())
                .map(|u| u.id);
            barb.camps.push(Camp {
                id: i as u32,
                pos: camp.pos,
                founded_turn: 1,
                guard,
                scout: None,
                raiders: Vec::new(),
                scout_report: None,
                raid: None,
                next_raid_turn: view.turn,
                next_spawn_turn: view.turn + 10,
                rite: None,
            });
        }
        barb.next_camp_id = barb.camps.len() as u32;
        barb.last_camp_turn = view.turn;
        barb.known
            .insert(team, view.barbarian_camps.iter().map(|c| c.pos).collect());
        state.barbarians = Some(barb);
    }

    // What we've explored and remember; rivals know the land around their own cities.
    let memories: Vec<Option<CityMemory>> = (0..n)
        .map(|i| {
            let h = state.map.hex_at(i);
            view.cities
                .iter()
                .find(|c| c.pos == h)
                .map(|c| memory_of(c, &state))
        })
        .collect();
    {
        let tiles = &view.tiles;
        let map_tiles = state.map.tiles.clone();
        let player = state.player_mut(me);
        player.explored = BitSet::new(n);
        player.tile_memory = vec![None; n];
        for (i, t) in tiles.iter().enumerate().take(n) {
            if t.explored {
                player.explored.set(i);
                player.tile_memory[i] = Some(TileMemory {
                    owner: t.owner,
                    ruin: t.ruin,
                    improved: t.improvement.is_some(),
                    city: memories[i].clone(),
                });
            }
        }
        let _ = map_tiles;
    }
    let rival_ids: Vec<PlayerId> = view
        .players
        .iter()
        .filter(|p| p.team != team)
        .map(|p| p.id)
        .collect();
    for id in rival_ids {
        let homes: Vec<Hex> = state.cities_of(id).map(|c| c.pos).collect();
        let mut explored = BitSet::new(n);
        let mut memory = vec![None; n];
        for h in homes {
            for x in h.within(6) {
                if let Some(i) = state.map.index(x) {
                    explored.set(i);
                    let tile = &state.map.tiles[i];
                    memory[i] = Some(TileMemory {
                        owner: tile.owner,
                        ruin: tile.ruin,
                        improved: tile.improved,
                        city: state.city_at(x).map(CityMemory::from),
                    });
                }
            }
        }
        let player = state.player_mut(id);
        player.explored = explored;
        player.tile_memory = memory;
    }
    legatum_core::visibility::update_explored(&mut state);
    state
}

fn memory_of(c: &CityView, state: &GameState) -> CityMemory {
    let live = state.cities.get(&c.id);
    CityMemory {
        id: c.id,
        owner: c.owner,
        name: c.name.clone(),
        population: live.map_or(c.population, |l| l.population),
        hp: c.hp,
        walls: c.walls,
        citadel: c.citadel,
        star_fort: c.star_fort,
        has_workshop: c.has_workshop,
        has_air_defence: c.has_air_defence,
        is_capital: c.is_capital,
    }
}

fn city_from_view(c: &CityView, turn: u32, me: PlayerId) -> City {
    let mut buildings: BTreeSet<BuildingKind> = BTreeSet::new();
    if c.walls {
        buildings.insert(BuildingKind::Walls);
    }
    if c.citadel {
        buildings.insert(BuildingKind::Castle);
    }
    if c.star_fort {
        buildings.insert(BuildingKind::StarFort);
    }
    if c.has_workshop {
        buildings.insert(BuildingKind::Workshop);
    }
    if c.has_air_defence {
        buildings.insert(BuildingKind::AirDefence);
    }
    let mut city = City {
        last_attacked_turn: None,
        id: c.id,
        owner: c.owner,
        founder: c
            .detail
            .as_ref()
            .and_then(|d| d.former_owner)
            .unwrap_or(c.owner),
        name: c.name.clone(),
        pos: c.pos,
        population: c.population.max(1),
        food_stored: 0,
        production_stored: 0,
        producing: None,
        production_banked: Vec::new(),
        production_queue: Vec::new(),
        build_automation: None,
        last_auto_pick: None,
        auto_queued: None,
        production_queue_notified: BTreeSet::new(),
        wonder_lost: None,
        last_purchase_turn: None,
        bought: None,
        buildings,
        hp: c.hp,
        tiles_claimed: 0,
        grown_tiles: Vec::new(),
        is_capital: c.is_capital,
        original_capital: c.original_capital
            || c.detail.as_ref().is_some_and(|d| d.original_capital)
            || c.is_capital,
        greece_captive_paid: if !c.greece_captive_tech && c.owner != me {
            [me].into_iter().collect()
        } else {
            Default::default()
        },
        occupied_from: (c.owner == me).then_some(c.occupation_lock).flatten(),
        founded_turn: 1,
        resistance: 0,
    };
    if let Some(d) = &c.detail {
        city.food_stored = d.food_stored;
        city.production_stored = d.production_stored;
        city.producing = d.producing;
        city.production_banked = d.production_banked.clone();
        city.production_queue = d.production_queue.clone();
        city.buildings = d.buildings.iter().copied().collect();

        city.resistance = d.resistance;
        let mut grown: Vec<Hex> = d
            .territory
            .iter()
            .copied()
            .filter(|h| h.distance(c.pos) >= 2)
            .collect();
        grown.sort_by_key(|h| (h.distance(c.pos), *h));
        grown.truncate((c.population - 1).max(0) as usize);
        city.grown_tiles = grown;
        city.founded_turn = turn.saturating_sub(1).max(1);
    }
    city
}

fn unit_from_view(u: &UnitView, view: &PlayerView) -> Unit {
    let faction = if u.owner.is_barbarian() {
        legatum_core::Faction::India
    } else {
        view.faction_of(u.owner)
    };
    let xp = u.xp.unwrap_or_else(|| {
        legatum_core::rules::promotion::threshold_for(faction, u.level).unwrap_or(0)
    });
    Unit {
        id: u.id,
        owner: u.owner,
        kind: u.kind,
        pos: u.pos,
        hp: u.hp,
        moves_left: u.moves_left,
        goto: u.goto,
        path: u.path.clone(),
        path_stop_on_block: false,
        automation: u.automation,
        standing_order: u.standing_order,
        needs_orders: u.needs_orders,
        blocked_turns: 0,
        attack_target: u.attack_target,
        siege: u.siege,
        xp,
        fortify: u.fortify,
        acted: false,
        created_turn: u.created_turn.unwrap_or(1),
        attack_revealed_to: BTreeSet::new(),
        attack_revealed_turn: None,
        heir_of: u.heir_of,
        contract_until: u.contract_until,
        origin: u.origin,
    }
}

fn fresh_unit(state: &GameState, id: UnitId, owner: PlayerId, kind: UnitKind, pos: Hex) -> Unit {
    Unit {
        id,
        owner,
        kind,
        pos,
        hp: data::UNIT_HP,
        moves_left: state.unit_moves_for(owner, kind),
        goto: None,
        path: Vec::new(),
        path_stop_on_block: false,
        automation: None,
        standing_order: None,
        needs_orders: false,
        blocked_turns: 0,
        attack_target: None,
        siege: false,
        xp: 0,
        fortify: 0,
        acted: false,
        created_turn: 1,
        attack_revealed_to: BTreeSet::new(),
        attack_revealed_turn: None,
        heir_of: None,
        contract_until: None,
        origin: None,
    }
}

/// A unit's share of the Strongest Army title.
fn army_value(state: &GameState, u: &Unit) -> i32 {
    legatum_core::army::unit_value(state, u)
}

/// An unseen site for a rival city: land we can't see, at least four hexes from every city,
/// as close as possible to what we know of its owner (anywhere unseen if nothing).
fn city_site(
    state: &GameState,
    cities: &BTreeMap<CityId, City>,
    anchors: &[Hex],
    visible: &impl Fn(Hex) -> bool,
    view: &PlayerView,
    rng: &mut Rng,
) -> Option<Hex> {
    let mine: Vec<Hex> = view
        .cities
        .iter()
        .filter(|c| c.owner == view.me)
        .map(|c| c.pos)
        .collect();
    let mut best: Vec<(i32, Hex)> = Vec::new();
    for i in 0..state.map.len() {
        let h = state.map.hex_at(i);
        let tile = &state.map.tiles[i];
        if tile.terrain.move_cost().is_none()
            || visible(h)
            || tile
                .owner
                .is_some_and(|o| anchors.is_empty() || state.player(o).team == view.team)
            || cities.values().any(|c| c.pos.distance(h) < 4)
        {
            continue;
        }
        let score = if anchors.is_empty() {
            // Nothing known: anywhere unseen, away from our own cities.
            -mine
                .iter()
                .map(|m| m.distance(h).min(12))
                .min()
                .unwrap_or(12)
        } else {
            anchors.iter().map(|a| a.distance(h)).min().unwrap_or(0)
        };
        best.push((score, h));
    }
    best.sort();
    let top = best.first()?.0;
    let ties: Vec<Hex> = best
        .iter()
        .take_while(|(s, _)| *s <= top + 1)
        .map(|(_, h)| *h)
        .collect();
    Some(ties[rng.range(0, ties.len() as i64 - 1) as usize])
}

/// The seat's chance to win in `state`, scored from its own view (a finished game is
/// 10,000 for a win and 0 otherwise).
pub fn chance_in(state: &GameState, me: PlayerId) -> i32 {
    chance_in_with(
        state,
        me,
        &crate::DecisionParams::builtin_for(state.player(me).faction),
    )
}

/// Search leaf using the same relative-position genes as the live plan scorer.
pub fn chance_in_with(state: &GameState, me: PlayerId, decisions: &crate::DecisionParams) -> i32 {
    let team = state.team_of(me);
    if state.finished {
        return if state.winner == Some(team) {
            10_000
        } else {
            0
        };
    }
    if !state
        .team_members(team)
        .iter()
        .any(|p| state.player(*p).alive)
    {
        return 0;
    }
    WinFacts::of(&legatum_core::player_view(state, me))
        .with_relative(decisions)
        .search_chance()
}

/// Economy buildings a determinized rival city may be given, cheapest first.
const ECONOMY_BUILDINGS: [BuildingKind; 5] = [
    BuildingKind::Workshop,
    BuildingKind::Market,
    BuildingKind::MachineShop,
    BuildingKind::Clearinghouse,
    BuildingKind::Autoworks,
];

/// Give rival cities economy buildings (only those their sampled technology allows) until
/// each rival's economy value reaches its public value. Without this the
/// determinized rivals hold no buildings and run 20-40% under their real economy mid-game.
pub fn calibrate_economy(state: &mut GameState, view: &PlayerView) {
    for p in view
        .players
        .iter()
        .filter(|p| p.alive && p.team != view.team)
    {
        let target = view.economy_of(p.id);
        let techs = state.player(p.id).techs.clone();
        let allowed: Vec<BuildingKind> = ECONOMY_BUILDINGS
            .into_iter()
            .filter(|b| {
                state
                    .setup
                    .rule_experiments
                    .building_stats_for(p.faction, *b)
                    .requires
                    .is_none_or(|t| techs.contains(&t))
            })
            .collect();
        for _ in 0..64 {
            let now = legatum_core::scoring::economy_value(state, p.id);
            if now >= target {
                break;
            }
            // The city with the fewest buildings (largest first on ties) that can take one.
            let pick = state
                .cities
                .values()
                .filter(|c| c.owner == p.id)
                .filter_map(|c| {
                    allowed
                        .iter()
                        .find(|b| !c.buildings.contains(b))
                        .map(|b| (c.buildings.len(), std::cmp::Reverse(c.population), c.id, *b))
                })
                .min();
            let Some((_, _, id, building)) = pick else {
                break;
            };
            if let Some(c) = state.cities.get_mut(&id) {
                c.buildings.insert(building);
            }
        }
    }
}

/// The seat's own supply records as its view shows them: which key buildings and units are
/// idle. `since` is unknown to the view, so idle users keep their place by staying idle.
fn own_supply(view: &PlayerView) -> Vec<legatum_core::resources::SupplyRecord> {
    use legatum_core::resources::{
        SUPPLY_TIERS, SupplyRecord, SupplyUser, building_need, unit_need,
    };
    let mut out = Vec::new();
    for c in view.cities.iter().filter(|c| c.owner == view.me) {
        let Some(d) = c.detail.as_ref() else { continue };
        for tier in SUPPLY_TIERS {
            let Some(resource) = building_need(tier) else {
                continue;
            };
            if !d
                .buildings
                .iter()
                .any(|b| legatum_core::data::building_satisfies(*b, tier))
            {
                continue;
            }
            out.push(SupplyRecord {
                user: SupplyUser::Building {
                    city: c.id,
                    building: tier,
                },
                resource,
                since: if d.idle_buildings.iter().any(|i| i.building == tier) {
                    view.turn
                } else {
                    0
                },
                idle: d.idle_buildings.iter().any(|i| i.building == tier),
            });
        }
    }
    for u in view.units.iter().filter(|u| u.owner == view.me) {
        if let Some(resource) = unit_need(u.kind) {
            out.push(SupplyRecord {
                user: SupplyUser::Unit { unit: u.id },
                resource,
                since: if u.unsupplied.is_some() { view.turn } else { 0 },
                idle: u.unsupplied.is_some(),
            });
        }
    }
    out.sort_by_key(|r| r.user);
    out
}

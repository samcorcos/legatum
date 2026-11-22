use crate::bitset::BitSet;
use crate::data::{self, UNIT_HP};
use crate::hex::Hex;
use crate::mapgen;
use crate::state::*;
use crate::types::*;
use crate::visibility;
use std::collections::BTreeMap;

/// A game on a supplied map with no units or cities yet. Used by `new_game` and by tests.
pub fn blank_game(setup: &GameSetup, map: Map) -> GameState {
    let players = setup
        .players
        .iter()
        .enumerate()
        .map(|(i, ps)| Player {
            id: PlayerId(i as u8),
            name: ps.name.clone(),
            faction: ps.faction,
            team: ps.team,
            is_human: ps.is_human,
            alive: true,
            gold: 0,
            science_stored: 0,
            science_conversion: ScienceConversion::Gold,
            researching: None,
            research_queue: Vec::new(),
            techs: Default::default(),
            explored: BitSet::new(map.len()),
            tile_memory: vec![None; map.len()],
            observed_threats: Vec::new(),
            failed_city_sites: Default::default(),
            ruins_explored: 0,
            settlers_trained: 0,
            military_purchases: 0,
            science_history: Vec::new(),
            power_history: Vec::new(),
            decadence_percent: 0,
            fight: Default::default(),
            eras: Default::default(),
            capitals_sacked: Vec::new(),
            great_work_completed: None,
            great_work_holder: None,
            faction_goal_done: false,
            goal_holder: None,
            hall_city: None,
            diwan_city: None,
            faction_goal_reached_before: false,
            free_techs_pending: 0,
            free_techs_from: None,
            free_techs_captured: None,
            veterans_retired: 0,
            sakoku_turns: None,
            goal_rule: None,
            supply: Default::default(),
        })
        .collect();
    GameState {
        dice_seed: None,
        wonders_discovered: Default::default(),
        barbarians: setup
            .barbarians
            .then(crate::rules::barbarians::BarbarianState::default),
        diplomacy: (!setup.legacy_all_war).then(|| {
            let teams: Vec<_> = setup.players.iter().map(|p| p.team).collect();
            let mut d = crate::diplomacy::DiplomacyState::new(teams.iter().copied());
            if setup.rule_experiments.start_met {
                d.meet_all(&teams);
            }
            for (a, b) in setup.ai_alliance_pairs() {
                d.start_alliance(a, b);
            }
            d
        }),
        setup: setup.clone(),
        turn: 1,
        map,
        players,
        units: BTreeMap::new(),
        cities: BTreeMap::new(),
        next_unit_id: 0,
        next_city_id: 0,
        winner: None,
        finished: false,
        vp_holds: BTreeMap::new(),
        era_firsts: BTreeMap::new(),
        aircraft: Vec::new(),
        air_recons: Vec::new(),
        air_contacts: Default::default(),
        history: Default::default(),
    }
}

/// Add a unit at full health and moves. Returns its id.
pub fn spawn_unit(state: &mut GameState, owner: PlayerId, kind: UnitKind, pos: Hex) -> UnitId {
    let id = state.alloc_unit_id();
    let unit = unit_with_id(state, id, owner, kind, pos);
    state.units.insert(id, unit);
    id
}

/// A fresh full-health unit with a given id (`spawn_unit` allocates one; `rules::air` gives an
/// aircraft its own id for the turn's strikes).
pub fn unit_with_id(
    state: &GameState,
    id: UnitId,
    owner: PlayerId,
    kind: UnitKind,
    pos: Hex,
) -> Unit {
    Unit {
        id,
        owner,
        kind,
        pos,
        hp: UNIT_HP,
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
        xp: crate::rules::promotion::starting_xp(state, owner, kind),
        fortify: 0,
        acted: false,
        attack_revealed_to: Default::default(),
        attack_revealed_turn: None,
        heir_of: None,
        contract_until: None,
        origin: None,
        created_turn: state.turn,
    }
}

/// Start a new game: generate the planet, found each capital, place the starting units, reveal the start.
///
/// Each empire starts with `data::START_GOLD`, a population-1 capital on its start hex,
/// then a Scout and a Spearman on the first two passable neighbors in hex order.
pub fn new_game(setup: &GameSetup) -> GameState {
    try_new_game(setup).unwrap_or_else(|e| panic!("{e}"))
}

/// Construct a game from an untrusted setup without panicking on impossible maps.
pub fn try_new_game(setup: &GameSetup) -> Result<GameState, String> {
    let ((map, starts), _) =
        mapgen::try_generate_counted_with_options(setup, mapgen::StartOptions::default())?;
    let mut state = blank_game(setup, map);
    let start_gold = data::START_GOLD;
    let rules = setup.rule_experiments;
    for player in &mut state.players {
        player.gold = start_gold;
        // A technology that gives this faction nothing is skipped: known from the start.
        player
            .techs
            .extend(data::skipped_techs(rules, player.faction));
    }
    for (i, &start) in starts.iter().enumerate() {
        let owner = PlayerId(i as u8);
        let mut room: Vec<Hex> = start
            .neighbors()
            .into_iter()
            .filter(|&h| state.map.move_cost(h).is_some())
            .collect();
        room.sort();
        if room.len() < 2 {
            return Err("mapgen produced a start without two passable neighbors".into());
        }
        let mut events = Vec::new();
        let city = crate::rules::cities::found_city_at(
            &mut state,
            owner,
            start,
            false,
            false,
            &mut events,
        );
        state
            .history
            .chronicle
            .push(crate::history::Chronicle::Founded {
                turn: 1,
                player: owner,
                city: state.cities[&city].name.clone(),
            });
        spawn_unit(&mut state, owner, UnitKind::Scout, room[0]);
        spawn_unit(&mut state, owner, UnitKind::Spearman, room[1]);
    }
    // Place extras after all ordinary units, retaining their IDs. Breadth-first search
    // stays on connected passable land, never stacks civilians, and has a fixed hex order.
    for (i, &start) in starts.iter().enumerate() {
        let mut remaining = setup.rule_experiments.start_settlers;
        let mut seen = std::collections::BTreeSet::from([start]);
        let mut neighbors = start.neighbors();
        neighbors.sort();
        let mut queue = std::collections::VecDeque::new();
        for neighbor in neighbors {
            if state.map.move_cost(neighbor).is_some() && seen.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
        while remaining > 0 {
            let Some(h) = queue.pop_front() else { break };
            if state.map.move_cost(h).is_some()
                && state.city_at(h).is_none()
                && !state
                    .units_at(h)
                    .any(|u| u.kind == UnitKind::Settler || u.owner != PlayerId(i as u8))
            {
                spawn_unit(&mut state, PlayerId(i as u8), UnitKind::Settler, h);
                remaining -= 1;
            }
            let mut neighbors = h.neighbors();
            neighbors.sort();
            for next in neighbors {
                if state.map.move_cost(next).is_some() && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        if remaining != 0 {
            return Err("not enough connected land for starting Settlers".into());
        }
    }
    state.history.first_turn = 1;
    state.history.empires = vec![crate::history::EmpireHistory::default(); state.players.len()];
    visibility::update_explored(&mut state);
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn setup(seed: u64) -> GameSetup {
        GameSetup::standard(seed, [Faction::Switzerland, Faction::Byzantium])
    }

    #[test]
    fn every_empire_starts_with_a_capital_and_two_units_on_passable_tiles() {
        for seed in 1..=100 {
            let state = new_game(&setup(seed));
            assert_eq!(state.turn, 1);
            assert_eq!(state.cities.len(), state.players.len());
            assert_eq!(state.units.len(), 2 * state.players.len(), "seed {seed}");
            for p in &state.players {
                let mut kinds: Vec<UnitKind> = state.units_of(p.id).map(|u| u.kind).collect();
                kinds.sort();
                assert_eq!(
                    kinds,
                    vec![UnitKind::Scout, UnitKind::Spearman],
                    "seed {seed} player {:?}",
                    p.id
                );
            }
            // One military and one civilian unit per tile at most, all on passable land.
            let mut occupied = BTreeSet::new();
            for u in state.units.values() {
                assert!(state.map.move_cost(u.pos).is_some(), "seed {seed}: {u:?}");
                assert_eq!(
                    u.moves_left,
                    data::unit_stats_for(state.player(u.owner).faction, u.kind).moves
                );
                assert_eq!(u.hp, UNIT_HP);
                let class = data::unit_stats(u.kind).military;
                assert!(
                    occupied.insert((u.pos, class)),
                    "seed {seed}: stacked at {:?}",
                    u.pos
                );
            }
        }
    }

    #[test]
    fn capitals_stand_on_starts_with_escorts_adjacent() {
        for seed in 1..=100 {
            let s = setup(seed);
            let (_, starts) = mapgen::generate(&s);
            let state = new_game(&s);
            for (i, start) in starts.iter().enumerate() {
                let owner = PlayerId(i as u8);
                let capital = state.capital_of(owner).unwrap();
                assert_eq!(capital.pos, *start);
                assert_eq!(capital.population, 1);
                assert!(capital.original_capital);
                assert_eq!(capital.founded_turn, 1);
                assert!(capital.producing.is_none());
                assert!(
                    state.units_of(owner).all(|u| u.pos.distance(*start) == 1),
                    "seed {seed}"
                );
            }
        }
    }

    #[test]
    fn start_is_explored_and_teammates_share_it() {
        for seed in 1..=100 {
            let state = new_game(&setup(seed));
            for p in &state.players {
                let explored = &p.explored;
                assert_eq!(explored, &visibility::visible_tiles(&state, p.team));
                for mate in state.team_members(p.team) {
                    let capital = state.capital_of(mate).unwrap();
                    // Hills can block the outer sight radius, but the founded centre and
                    // claimed first ring must be visible before any turn resolves.
                    for h in capital.pos.within(1) {
                        if let Some(i) = state.map.index(h) {
                            assert!(
                                explored.get(i),
                                "seed {seed}: capital sight {h:?} unexplored"
                            );
                        }
                    }
                    for u in state.units_of(mate) {
                        for h in u.pos.within(1) {
                            if let Some(i) = state.map.index(h) {
                                assert!(explored.get(i), "seed {seed}: {h:?} unexplored");
                            }
                        }
                    }
                }
            }
            // Seats 0 and 1 are teammates in the standard game: identical explored sets.
            assert_eq!(state.players[0].explored, state.players[1].explored);
        }
    }

    #[test]
    fn new_game_is_deterministic() {
        for seed in [1, 42, 99] {
            let a = new_game(&setup(seed));
            let b = new_game(&setup(seed));
            assert_eq!(a, b);
            assert_eq!(a.hash(), b.hash());
        }
        assert_ne!(new_game(&setup(1)).hash(), new_game(&setup(2)).hash());
    }

    #[test]
    fn every_faction_starts_with_its_ordinary_capital_and_no_settler() {
        for faction in Faction::ALL {
            let setup = GameSetup::standard(4545, [faction, Faction::Switzerland]);
            let (_, starts) = mapgen::generate(&setup);
            let state = new_game(&setup);
            let capital = state.capital_of(PlayerId(0)).unwrap();
            assert_eq!(capital.pos, starts[0], "{faction:?}");
            assert_eq!(capital.population, 1, "{faction:?}");
            assert!(capital.original_capital, "{faction:?}");
            assert!(
                state
                    .units_of(PlayerId(0))
                    .all(|u| u.kind != UnitKind::Settler),
                "{faction:?}"
            );
            assert_eq!(state.player(PlayerId(0)).settlers_trained, 0);
            assert!(state.history.chronicle.iter().any(|entry| matches!(entry,
                crate::history::Chronicle::Founded { turn: 1, player: PlayerId(0), city }
                    if city == &capital.name)));
            assert_eq!(state, new_game(&setup));
        }
    }

    #[test]
    fn starting_capital_matches_a_settler_founding_on_the_same_hex() {
        let setup = setup(4546);
        let (map, starts) = mapgen::generate(&setup);
        let mut settled = blank_game(&setup, map);
        let settler = spawn_unit(&mut settled, PlayerId(0), UnitKind::Settler, starts[0]);
        let city =
            crate::rules::cities::found_city(&mut settled, settler, &mut Vec::new()).unwrap();
        let started = new_game(&setup);
        assert_eq!(
            settled.cities[&city],
            *started.capital_of(PlayerId(0)).unwrap()
        );
        for h in starts[0].within(1) {
            assert_eq!(settled.map.get(h), started.map.get(h));
        }
    }

    #[test]
    fn extra_start_settlers_stand_around_the_capital() {
        let mut setup = setup(4547);
        setup.rule_experiments.start_settlers = 2;
        let state = new_game(&setup);
        for player in &state.players {
            let capital = state.capital_of(player.id).unwrap();
            let settlers: Vec<_> = state
                .units_of(player.id)
                .filter(|u| u.kind == UnitKind::Settler)
                .collect();
            assert_eq!(settlers.len(), 2);
            assert!(settlers.iter().all(|u| u.pos != capital.pos));
            assert_eq!(state.player(player.id).settlers_trained, 0);
        }
    }
}

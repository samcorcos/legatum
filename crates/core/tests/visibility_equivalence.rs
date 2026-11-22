//! Bounded differential coverage for visibility union and observation updates.
//! The reference traversal shares core LOS geometry but does not use the production
//! visibility iterator or observation helper.

use legatum_core::bitset::BitSet;
use legatum_core::data::{CITY_SIGHT, UNIT_SIGHT};
use legatum_core::game::{blank_game, spawn_unit};
use legatum_core::los;
use legatum_core::rng::Rng;
use legatum_core::state::{CityMemory, Map, TileMemory};
use legatum_core::{
    BuildingKind, CityId, Faction, GameSetup, GameState, Hex, Order, PlayerId, ProductionItem,
    TeamId, TechId, Terrain, UnitId, UnitKind, new_game, player_view, resolve_turn, rules,
    validate_orders, visibility,
};
use std::collections::BTreeMap;

fn original_visible_tiles(state: &GameState, team: TeamId) -> BitSet {
    let mut seen = BitSet::new(state.map.len());
    let sources = state
        .units
        .values()
        .filter(|u| state.team_of(u.owner) == team)
        .map(|u| (u.pos, UNIT_SIGHT))
        .chain(
            state
                .cities
                .values()
                .filter(|c| state.team_of(c.owner) == team)
                .map(|c| (c.pos, CITY_SIGHT)),
        );
    for (pos, base_radius) in sources {
        let radius = base_radius
            + i32::from(
                state
                    .map
                    .get(pos)
                    .is_some_and(|tile| tile.terrain == Terrain::Hills),
            );
        for h in pos.within(radius) {
            if let Some(i) = state.map.index(h)
                && los::visible(pos, h, |at| state.map.get(at).map(|tile| tile.terrain))
            {
                seen.set(i);
            }
        }
    }
    // Owned land and the ring around it are always visible, whatever the terrain or distance.
    for (i, tile) in state.map.tiles.iter().enumerate() {
        if tile.owner.is_some_and(|owner| state.team_of(owner) == team) {
            seen.set(i);
            for neighbor in state.map.hex_at(i).neighbors() {
                if let Some(j) = state.map.index(neighbor) {
                    seen.set(j);
                }
            }
        }
    }
    seen
}

fn original_update_explored(state: &mut GameState) {
    let mut cities = vec![None; state.map.len()];
    for city in state.cities.values() {
        if let Some(i) = state.map.index(city.pos) {
            cities[i] = Some(CityMemory::from(city));
        }
    }
    for team in state.teams() {
        let seen = original_visible_tiles(state, team);
        for player in state.players.iter_mut().filter(|p| p.team == team) {
            player.tile_memory.resize(state.map.len(), None);
            for (i, tile) in state.map.tiles.iter().enumerate() {
                if seen.get(i) {
                    player.explored.set(i);
                    player.tile_memory[i] = Some(TileMemory {
                        owner: tile.owner,
                        ruin: tile.ruin,
                        improved: tile.improved,
                        city: cities[i].clone(),
                    });
                } else if player.explored.get(i) || player.tile_memory[i].is_some() {
                    let memory = player.tile_memory[i].get_or_insert(TileMemory {
                        owner: None,
                        ruin: false,
                        improved: false,
                        city: None,
                    });
                    memory.owner = tile.owner;
                    memory.city = cities[i].as_ref().map(|current| {
                        CityMemory::with_public_identity(current, memory.city.as_ref())
                    });
                }
            }
        }
    }
}

fn compare(input: &GameState) -> GameState {
    for team in input.teams().into_iter().chain([TeamId(255)]) {
        assert_eq!(
            visibility::visible_tiles(input, team),
            original_visible_tiles(input, team)
        );
    }
    let mut expected = input.clone();
    original_update_explored(&mut expected);
    let mut actual = input.clone();
    visibility::update_explored(&mut actual);
    // The historical oracle covers tile/city observations only. Recent military
    // observations are a newer field, independently exercised in threat_memory.rs.
    for (expected, actual) in expected.players.iter_mut().zip(&actual.players) {
        expected.observed_threats = actual.observed_threats.clone();
    }
    // First contact is newer too (diplomacy.rs tests); it is derived from the same sight.
    if let (Some(e), Some(a)) = (expected.diplomacy.as_mut(), actual.diplomacy.as_ref()) {
        e.contacts = a.contacts.clone();
    }
    assert_eq!(
        actual, expected,
        "full observation state, turn {}",
        input.turn
    );
    assert_eq!(actual.hash(), expected.hash());
    assert_eq!(
        serde_json::to_vec(&actual).unwrap(),
        serde_json::to_vec(&expected).unwrap()
    );
    for p in &input.players {
        let a = player_view(&actual, p.id);
        let b = player_view(&expected, p.id);
        assert_eq!(a, b, "view for {:?}", p.id);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
    }
    // Observation is idempotent when neither live state nor sources change.
    visibility::update_explored(&mut actual);
    assert_eq!(actual, expected);
    actual
}

fn generated_orders(s: &GameState) -> BTreeMap<PlayerId, Vec<Order>> {
    let mut orders = BTreeMap::new();
    for p in s.players.iter().filter(|p| p.alive) {
        let mut os = Vec::new();
        let view = player_view(s, p.id);
        for u in s.units_of(p.id) {
            let mut rng = Rng::derive(s.setup.seed, &[s.turn as u64, 0xB13, u.id.0 as u64]);
            let candidate = if u.kind == UnitKind::Settler {
                Order::FoundCity { unit: u.id }
            } else if rng.chance(50) {
                let nearby: Vec<_> = u
                    .pos
                    .within(2)
                    .into_iter()
                    .filter(|h| {
                        view.tile(*h)
                            .is_some_and(|t| t.terrain.and_then(Terrain::move_cost).is_some())
                    })
                    .collect();
                let to = nearby[rng.range(0, nearby.len() as i64 - 1) as usize];
                let path = legatum_core::path::find_path(&s.map, u.pos, to, |h| {
                    view.tile(h)?.terrain.and_then(Terrain::move_cost)
                })
                .unwrap_or_default();
                Order::MoveUnitPath {
                    unit: u.id,
                    to,
                    path,
                    stop_on_block: false,
                }
            } else {
                Order::MoveUnit {
                    unit: u.id,
                    to: Hex::from_offset(
                        rng.range(0, (s.map.width - 1) as i64) as i32,
                        rng.range(0, (s.map.height - 1) as i64) as i32,
                    ),
                }
            };
            if validate_orders(s, p.id, std::slice::from_ref(&candidate)).is_empty() {
                os.push(candidate);
            }
        }
        for c in s.cities_of(p.id) {
            if c.producing.is_none() {
                os.push(Order::SetProduction {
                    city: c.id,
                    item: ProductionItem::Unit {
                        unit: UnitKind::Scout,
                    },
                });
            }
        }
        orders.insert(p.id, os);
    }
    orders
}

#[test]
fn generated_reachable_states_match_original_observations() {
    let mut confirmed_paths = 0;
    // Six short deterministic games, 144 reachable turn inputs, both map sizes and team layouts.
    for (width, height) in [(48, 32), (72, 44)] {
        for seed in [1, 7, u64::MAX] {
            let mut setup = GameSetup::all_ai(seed);
            setup.width = width;
            setup.height = height;
            if seed != 1 {
                setup.players[1].team = setup.players[0].team;
            }
            let mut state = new_game(&setup);
            for _ in 0..24 {
                state = compare(&state);
                // Also exercise an old save with no observations and partially retained memory.
                if state.turn.is_multiple_of(6) {
                    let mut old = state.clone();
                    old.players[0].tile_memory.clear();
                    old.players[1].tile_memory.truncate(65);
                    compare(&old);
                }
                let orders = generated_orders(&state);
                confirmed_paths += orders
                    .values()
                    .flatten()
                    .filter(|o| matches!(o, Order::MoveUnitPath { .. }))
                    .count();
                state = resolve_turn(&state, &orders).0;
            }
        }
    }
    assert!(confirmed_paths > 0);
}

#[test]
fn shared_vision_updates_public_founding_capture_and_raze_without_private_fog() {
    let mut setup = GameSetup::standard(9, [Faction::Switzerland, Faction::India]);
    setup.width = 24;
    setup.height = 16;
    let mut s = blank_game(&setup, Map::new(24, 16, Terrain::Grassland));
    let (p0, p1, p2, p3) = (PlayerId(0), PlayerId(1), PlayerId(2), PlayerId(3));
    let site = Hex::from_offset(5, 5);
    let away = Hex::from_offset(21, 13);
    // Only the teammate provides sight at the city; both seats must remember it identically.
    let scout = spawn_unit(&mut s, p1, UnitKind::Scout, site);
    s.map.get_mut(site).unwrap().ruin = true;
    s = compare(&s);
    let index = s.map.index(site).unwrap();
    s.units.get_mut(&scout).unwrap().pos = away;
    let settler = spawn_unit(&mut s, p2, UnitKind::Settler, site);
    let city = rules::cities::found_city(&mut s, settler, &mut Vec::new()).unwrap();
    s = compare(&s);
    for p in [p0, p1] {
        let v = player_view(&s, p);
        assert!(!v.tiles[index].visible);
        assert_eq!(v.tiles[index].owner, Some(p2));
        assert!(
            v.cities
                .iter()
                .any(|c| c.id == city && c.owner == p2 && !c.visible),
            "a founded city on explored land must appear through fog"
        );
    }
    s.units.get_mut(&scout).unwrap().pos = site;
    s = compare(&s);
    let remembered = s.player(p0).tile_memory[index].clone();
    s.units.get_mut(&scout).unwrap().pos = away;
    rules::battle::capture_city(&mut s, city, p3, &mut Vec::new());
    let c = s.cities.get_mut(&city).unwrap();
    c.name = "Changed in fog".into();
    c.population = 7;
    c.hp = 17;
    c.buildings.insert(BuildingKind::Walls);
    s.map.get_mut(site).unwrap().ruin = false;
    s = compare(&s);
    for p in [p0, p1] {
        let memory = s.player(p).tile_memory[index].as_ref().unwrap();
        let previous = remembered.as_ref().unwrap();
        assert_eq!(memory.owner, Some(p3));
        assert_eq!(
            (memory.ruin, memory.improved),
            (previous.ruin, previous.improved)
        );
        let current_city = memory.city.as_ref().unwrap();
        let previous_city = previous.city.as_ref().unwrap();
        assert_eq!(
            (current_city.owner, current_city.name.as_str()),
            (p3, "Changed in fog")
        );
        assert_eq!(
            (current_city.population, current_city.hp, current_city.walls),
            (
                previous_city.population,
                previous_city.hp,
                previous_city.walls
            )
        );
        let v = player_view(&s, p);
        let c = v.cities.iter().find(|c| c.id == city).unwrap();
        assert_eq!(c.owner, p3);
        assert!(!c.visible && !c.walls);
        assert!(v.tiles[index].ruin);
    }
    s.units.get_mut(&scout).unwrap().pos = site;
    s = compare(&s);
    let observed = s.player(p0).tile_memory[index].clone();
    let c = observed.as_ref().unwrap().city.as_ref().unwrap();
    assert_eq!((c.owner, c.population, c.hp, c.walls), (p3, 7, 17, true));
    s.units.get_mut(&scout).unwrap().pos = away;
    // There is no optional raze order; remove the city to exercise the fog regression.
    s.cities.remove(&city);
    for tile in &mut s.map.tiles {
        if tile.city == Some(city) {
            tile.city = None;
            tile.owner = None;
        }
    }
    s = compare(&s);
    for p in [p0, p1] {
        let memory = s.player(p).tile_memory[index].as_ref().unwrap();
        assert_eq!(memory.owner, None);
        assert_eq!(
            (memory.ruin, memory.improved),
            (
                observed.as_ref().unwrap().ruin,
                observed.as_ref().unwrap().improved
            )
        );
        assert!(memory.city.is_none());
        assert!(player_view(&s, p).cities.iter().all(|c| c.id != city));
    }
    s.units.get_mut(&scout).unwrap().pos = site;
    s = compare(&s);
    for p in [p0, p1] {
        assert!(
            s.player(p).tile_memory[index]
                .as_ref()
                .unwrap()
                .city
                .is_none()
        );
        assert!(player_view(&s, p).cities.iter().all(|c| c.id != city));
    }
}

#[test]
fn empty_short_and_older_memory_gain_only_public_fog_facts() {
    let mut state = new_game(&GameSetup::all_ai(42));
    // Deserialize the actual old-save shape (missing tile_memory), not just an empty vector.
    let mut old = serde_json::to_value(&state).unwrap();
    for p in old["players"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("tile_memory");
    }
    state = serde_json::from_value(old).unwrap();
    let seen = original_visible_tiles(&state, state.players[0].team);
    let hidden = (0..state.map.len()).find(|&i| !seen.get(i)).unwrap();
    state.players[0].explored.set(hidden);
    state.map.tiles[hidden].owner = Some(PlayerId(2));
    state.map.tiles[hidden].ruin = true;
    state.players[1].tile_memory.resize(65, None);
    state.players[2]
        .tile_memory
        .resize(state.map.len() + 65, None);
    state = compare(&state);
    assert_eq!(
        state.players[0].tile_memory[hidden],
        Some(TileMemory {
            owner: Some(PlayerId(2)),
            ruin: false,
            improved: false,
            city: None
        })
    );
    let v = player_view(&state, PlayerId(0));
    assert!(v.tiles[hidden].explored && !v.tiles[hidden].visible);
    assert_eq!(v.tiles[hidden].owner, Some(PlayerId(2)));
    assert!(!v.tiles[hidden].ruin);
    // No vision sources: old observations remain, including after repeated updates.
    state.units.clear();
    state.cities.clear();
    for tile in &mut state.map.tiles {
        if tile.city.is_some() {
            tile.owner = None;
            tile.city = None;
        }
    }
    let hidden_memory = state.players[0].tile_memory[hidden].clone();
    state = compare(&state);
    assert_eq!(state.players[0].tile_memory[hidden], hidden_memory);
}

// The optional cross-build check uses serde_json from core's existing dependencies.
// This independent tape decoder is checked
// against the profiler's recorded per-turn hashes before any JSON snapshot is accepted.
fn tape_order(n: &[i32]) -> Order {
    use legatum_core::diplomacy::{Agreement, ProposalId, TreatyKind};
    let (unit, city) = (UnitId(n[1] as u32), CityId(n[1] as u32));
    match n[0] {
        0 => Order::MoveUnit {
            unit,
            to: Hex::new(n[2], n[3]),
        },
        1 => Order::SkipUnit { unit },
        2 => Order::FoundCity { unit },
        3 => Order::DisbandUnit { unit },
        4 => Order::SetProduction {
            city,
            item: if n[2] == 0 {
                ProductionItem::Unit {
                    unit: UnitKind::ALL[n[3] as usize],
                }
            } else {
                assert_eq!(n[2], 1);
                ProductionItem::Building {
                    building: BuildingKind::ALL[n[3] as usize],
                }
            },
        },
        5 => Order::BuyItem {
            city,
            // Match the profiler's explicit-item format introduced in dd1fc7e. Older
            // implicit purchases need their original producer/core; CSV has no version marker.
            item: match n[2] {
                0 => ProductionItem::Unit {
                    unit: UnitKind::ALL[n[3] as usize],
                },
                1 => ProductionItem::Building {
                    building: BuildingKind::ALL[n[3] as usize],
                },
                _ => panic!("invalid purchase kind"),
            },
        },
        6 => Order::SetResearch {
            tech: TechId::ALL[n[1] as usize],
        },
        7 => Order::RangedAttack {
            unit,
            target: Hex::new(n[2], n[3]),
        },
        8 => Order::UpgradeUnit { unit },
        9 => Order::DeclareWar {
            against: TeamId(n[1] as u8),
            breach: n[2] != 0,
        },
        10 => Order::ProposeTreaty {
            to: TeamId(n[1] as u8),
            kind: match n[2] {
                0 => TreatyKind::Peace,
                1 => TreatyKind::Alliance,
                _ => panic!("invalid treaty kind"),
            },
        },
        11 => Order::RespondToProposal {
            proposal: ProposalId(n[1] as u32),
            accept: n[2] != 0,
            explanation: (n.len() > 4).then(|| {
                let bytes: Vec<_> = n[4..]
                    .iter()
                    .map(|&byte| u8::try_from(byte).unwrap())
                    .collect();
                serde_json::from_slice(&bytes).unwrap()
            }),
        },
        12 => Order::WithdrawProposal {
            proposal: ProposalId(n[1] as u32),
        },
        14 => Order::BreakTreaty {
            with: TeamId(n[1] as u8),
            agreement: Agreement::Security,
        },
        _ => panic!("invalid tape order"),
    }
}

#[test]
fn numeric_diplomacy_rows_preserve_alliances_and_response_receipts() {
    use legatum_core::diplomacy::{DecisionExplanation, ProposalId, TreatyKind};
    for (row, order) in [
        (
            vec![10, 3, 1, 0],
            Order::ProposeTreaty {
                to: TeamId(3),
                kind: TreatyKind::Alliance,
            },
        ),
        (
            vec![11, 7, 1, 0],
            Order::RespondToProposal {
                proposal: ProposalId(7),
                accept: true,
                explanation: None,
            },
        ),
    ] {
        assert_eq!(tape_order(&row), order);
    }
    let explanation = DecisionExplanation {
        score: Some(-12),
        reasons: vec!["Trust: -12, \"ally\"\nΔ".into()],
    };
    let mut row = vec![11, 7, 0, 0];
    row.extend(
        serde_json::to_vec(&explanation)
            .unwrap()
            .into_iter()
            .map(i32::from),
    );
    assert_eq!(
        tape_order(&row),
        Order::RespondToProposal {
            proposal: ProposalId(7),
            accept: false,
            explanation: Some(explanation)
        }
    );
}

#[test]
fn numeric_market_purchase_preserves_item() {
    assert_eq!(
        tape_order(&[5, 17, 1, 3]),
        Order::BuyItem {
            city: CityId(17),
            item: ProductionItem::Building {
                building: BuildingKind::Market,
            },
        }
    );
}

#[test]
fn numeric_ranger_purchase_preserves_item() {
    assert_eq!(
        tape_order(&[5, 29, 0, 3]),
        Order::BuyItem {
            city: CityId(29),
            item: ProductionItem::Unit {
                unit: UnitKind::Archer,
            },
        }
    );
}

#[test]
fn numeric_unit_indices_preserve_production_and_purchases() {
    for (index, unit) in UnitKind::ALL.into_iter().enumerate() {
        // The profiler encodes enum discriminants and decodes indices in ALL.
        assert_eq!(unit as usize, index);
        let city = CityId(41);
        let item = ProductionItem::Unit { unit };
        assert_eq!(
            tape_order(&[4, city.0 as i32, 0, index as i32]),
            Order::SetProduction { city, item }
        );
        assert_eq!(
            tape_order(&[5, city.0 as i32, 0, index as i32]),
            Order::BuyItem { city, item }
        );
    }
}

#[test]
fn numeric_building_indices_preserve_production_and_purchases() {
    for (index, building) in BuildingKind::ALL.into_iter().enumerate() {
        assert_eq!(building as usize, index);
        let city = CityId(53);
        let item = ProductionItem::Building { building };
        assert_eq!(
            tape_order(&[4, city.0 as i32, 1, index as i32]),
            Order::SetProduction { city, item }
        );
        assert_eq!(
            tape_order(&[5, city.0 as i32, 1, index as i32]),
            Order::BuyItem { city, item }
        );
    }
}

fn assert_numeric_purchase_matches_direct(item: ProductionItem, payload: [i32; 2]) {
    let mut setup = GameSetup::all_ai(7);
    setup.width = 16;
    setup.height = 10;
    setup.players.truncate(2);
    let mut state = blank_game(&setup, Map::new(16, 10, Terrain::Grassland));
    let player = PlayerId(0);
    let settler = spawn_unit(
        &mut state,
        player,
        UnitKind::Settler,
        Hex::from_offset(3, 4),
    );
    let city = rules::cities::found_city(&mut state, settler, &mut Vec::new()).unwrap();
    // Keep a second empire alive so the fixture exercises an ordinary game turn.
    let settler = spawn_unit(
        &mut state,
        PlayerId(1),
        UnitKind::Settler,
        Hex::from_offset(12, 6),
    );
    rules::cities::found_city(&mut state, settler, &mut Vec::new()).unwrap();
    state.player_mut(player).gold = 1_000;
    state
        .player_mut(player)
        .techs
        .extend([TechId::Irrigation, TechId::Currency]);
    // Copper is a bonus tile resource; a Market needs no strategic supply.
    let copper = state.map.get_mut(Hex::from_offset(4, 4)).unwrap();
    copper.resource = Some(legatum_core::Resource::Copper);
    copper.owner = Some(player);
    visibility::update_explored(&mut state);

    let direct = Order::BuyItem { city, item };
    let decoded = tape_order(&[5, city.0 as i32, payload[0], payload[1]]);
    for order in [&direct, &decoded] {
        let errors = validate_orders(&state, player, std::slice::from_ref(order));
        assert!(errors.is_empty(), "{order:?}: {errors:?}");
    }
    let (expected, expected_events) =
        resolve_turn(&state, &BTreeMap::from([(player, vec![direct])]));
    let (actual, actual_events) = resolve_turn(&state, &BTreeMap::from([(player, vec![decoded])]));
    assert_eq!(expected.turn, state.turn + 1);
    assert!(!expected.finished);
    // Prove the direct purchase completed; two rejected orders must not count as equivalent.
    match item {
        ProductionItem::Building { building } | ProductionItem::Wonder { building, .. } => {
            assert!(expected.cities[&city].buildings.contains(&building));
        }
        ProductionItem::Unit { unit } => {
            assert_eq!(
                expected
                    .units_of(player)
                    .map(|u| u.kind)
                    .collect::<Vec<_>>(),
                vec![unit]
            );
        }
        ProductionItem::Improvement { tile, .. } => {
            assert!(expected.map.get(tile).unwrap().improved);
        }
        ProductionItem::Project { .. } => panic!("projects cannot be purchased"),
    }
    assert_eq!(actual.hash(), expected.hash(), "{item:?} purchase state");
    for p in &state.players {
        assert_eq!(
            serde_json::to_vec(&player_view(&actual, p.id)).unwrap(),
            serde_json::to_vec(&player_view(&expected, p.id)).unwrap(),
            "{item:?} purchase view for {:?}",
            p.id
        );
    }
    assert_eq!(actual_events, expected_events);
}

#[test]
fn numeric_market_purchase_replays_identical_state_and_views() {
    assert_numeric_purchase_matches_direct(
        ProductionItem::Building {
            building: BuildingKind::Market,
        },
        [1, 3],
    );
}

#[test]
fn numeric_ranger_purchase_replays_identical_state_and_views() {
    assert_numeric_purchase_matches_direct(
        ProductionItem::Unit {
            unit: UnitKind::Archer,
        },
        [0, 3],
    );
}

#[test]
#[ignore = "explicit cross-build corpus check; see scripts/perf-profile.sh and performance-pass.md"]
fn recorded_corpus_states_and_serialized_views_match() {
    use std::fs;
    use std::io::Write;
    let root =
        std::path::PathBuf::from(std::env::var_os("PERF_CORPUS_DIR").expect("PERF_CORPUS_DIR"));
    let record = match std::env::var("PERF_JSON_MODE").as_deref() {
        Ok("record") => true,
        Ok("verify") => false,
        _ => panic!("PERF_JSON_MODE=record|verify required"),
    };
    let mut count = 0;
    for (name, width, height, shared) in [
        ("sim48", 48, 32, false),
        ("team48", 48, 32, true),
        ("prd72", 72, 44, false),
    ] {
        for seed in [1, 7] {
            let dir = root.join(format!("{name}-seed{seed}"));
            let mut setup = GameSetup::all_ai(seed);
            setup.width = width;
            setup.height = height;
            setup.turn_limit = Some(150);
            if shared {
                setup.players[1].team = setup.players[0].team;
            }
            assert_eq!(
                fs::read_to_string(dir.join("setup.txt")).unwrap(),
                format!("{setup:?}\n")
            );
            let mut tape = BTreeMap::<u32, BTreeMap<PlayerId, Vec<Order>>>::new();
            for row in fs::read_to_string(dir.join("orders.csv"))
                .unwrap()
                .lines()
                .skip(1)
            {
                let n: Vec<i32> = row.split(',').map(|x| x.parse().unwrap()).collect();
                assert!(n.len() >= 6);
                let os = tape
                    .entry(n[0] as u32)
                    .or_default()
                    .entry(PlayerId(n[1] as u8))
                    .or_default();
                if n[2] != -1 {
                    os.push(tape_order(&n[2..]));
                }
            }
            let hashes: BTreeMap<u32, u64> = fs::read_to_string(dir.join("hashes.csv"))
                .unwrap()
                .lines()
                .skip(1)
                .map(|s| {
                    let (a, b) = s.split_once(',').unwrap();
                    (a.parse().unwrap(), b.parse().unwrap())
                })
                .collect();
            let mut state = new_game(&setup);
            if record {
                fs::create_dir(dir.join("json")).unwrap();
            }
            loop {
                assert_eq!(
                    state.hash(),
                    hashes[&state.turn],
                    "{name} seed {seed} turn {}",
                    state.turn
                );
                let views: Vec<_> = state
                    .players
                    .iter()
                    .map(|p| player_view(&state, p.id))
                    .collect();
                let bytes = serde_json::to_vec(&(&state, &views)).unwrap();
                let path = dir.join("json").join(format!("{}.json", state.turn));
                if record {
                    fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(path)
                        .unwrap()
                        .write_all(&bytes)
                        .unwrap();
                } else {
                    assert!(
                        fs::read(path).unwrap() == bytes,
                        "canonical JSON changed: {name} seed {seed} turn {}",
                        state.turn
                    );
                }
                count += 1;
                if state.finished {
                    break;
                }
                state = resolve_turn(&state, &tape[&state.turn]).0;
            }
            assert_eq!(tape.len() + 1, hashes.len());
        }
    }
    println!("{count} complete state + all-seat PlayerView JSON snapshots checked");
}

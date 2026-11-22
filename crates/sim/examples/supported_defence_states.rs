//! The start state of the designer's supported-defence example (2026-11-15), hand-built with the
//! engine for the two Scenario Lab cards `supported-defence-hold` and `supported-defence-take`:
//! a walled German city with a Rifleman garrison and three German Riflemen beside it on the east,
//! three Indian Riflemen beside it on the west, at War, both empires in the Industrial era.
//! India has a home city six tiles west so it is not eliminated. Both cities have production
//! queued and research chosen. The state is checked by `scenario::validate_state`.
//! cargo run --release -p legatum-sim --example supported_defence_states -- OUT.json
//! then cut it with `legatum-sim scenario-cut --state OUT.json ...` (see `scenarios/README.md`).
use legatum_core::{
    BuildingKind, Faction, GameSetup, Hex, PlayerId, TeamId, TechId, Terrain, UnitKind,
    eras::era_of_tech,
    game::{blank_game, spawn_unit},
    rules::cities::found_city,
    state::{Map, PlayerSetup},
    types::ProductionItem,
};

const INDUSTRIAL: u8 = 4;

fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("usage: supported_defence_states OUT.json");
    let player = |faction, team| PlayerSetup {
        name: format!("{faction:?}"),
        faction,
        team: TeamId(team),
        is_human: false,
        difficulty: Default::default(),
        search: false,
    };
    let setup = GameSetup {
        seed: 20261115,
        width: 14,
        height: 9,
        players: vec![player(Faction::Germany, 0), player(Faction::India, 1)],
        turn_limit: Some(300),
        legacy_all_war: false,
        barbarians: false,
        rule_experiments: Default::default(),
    };
    let mut s = blank_game(&setup, Map::new(14, 9, Terrain::Grassland));
    for relation in &mut s.diplomacy.as_mut().expect("diplomacy").relations {
        relation.state = legatum_core::diplomacy::RelationState::War;
    }
    let industrial: Vec<TechId> = TechId::ALL
        .into_iter()
        .filter(|t| era_of_tech(*t) == INDUSTRIAL)
        .collect();
    for p in [PlayerId(0), PlayerId(1)] {
        let player = s.player_mut(p);
        player.techs.extend(
            TechId::ALL
                .into_iter()
                .filter(|t| era_of_tech(*t) < INDUSTRIAL),
        );
        player.researching = industrial.first().copied();
        player.gold = 150;
    }
    let found = |s: &mut legatum_core::GameState, owner: PlayerId, at: Hex| {
        let settler = spawn_unit(s, owner, UnitKind::Settler, at);
        found_city(s, settler, &mut Vec::new()).expect("legal city site")
    };
    let at = Hex::from_offset(8, 4);
    let german = found(&mut s, PlayerId(0), at);
    let indian = found(&mut s, PlayerId(1), Hex::from_offset(2, 4));
    {
        let c = s.cities.get_mut(&german).expect("German city");
        c.population = 5;
        c.buildings.insert(BuildingKind::Walls);
    }
    s.cities.get_mut(&indian).expect("Indian city").population = 3;
    for id in [german, indian] {
        s.cities.get_mut(&id).expect("city").producing = Some(ProductionItem::Unit {
            unit: UnitKind::Rifleman,
        });
    }
    let garrison = spawn_unit(&mut s, PlayerId(0), UnitKind::Rifleman, at);
    s.units.get_mut(&garrison).expect("garrison").fortify = 2;
    // East (0, 1, 5): the supporters. West (3, 2, 4): the attackers.
    for d in [0, 1, 5] {
        spawn_unit(&mut s, PlayerId(0), UnitKind::Rifleman, at.neighbor(d));
    }
    for d in [3, 2, 4] {
        spawn_unit(&mut s, PlayerId(1), UnitKind::Rifleman, at.neighbor(d));
    }
    legatum_core::visibility::update_explored(&mut s);
    let errors = legatum_core::scenario::validate_state(&s);
    assert!(errors.is_empty(), "{errors:?}");
    std::fs::write(&out, legatum_wasm::json::state_to_json(&s)).expect("write state");
    println!("{out}: German city at {at:?}, turn {}", s.turn);
}

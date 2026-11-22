//! Reproducible playtest fixture. Uses deliberately passive orders, not the current AI policy.
//! Build the tape against the current engine to survive conservative source fingerprints.
use legatum_core::{
    game::{blank_game, spawn_unit},
    rules::cities::found_city,
    state::Map,
    *,
};
use legatum_wasm::{history::Recording, session::OrderMap};

pub fn ranger_repro() -> Recording {
    let mut setup = GameSetup::all_ai(31337);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::ALL[0];
    setup.players[1].faction = Faction::ALL[1];
    setup.players[1].is_human = true;
    setup.width = 18;
    setup.height = 12;
    // The reported failures predate cities firing back; reproduce them under those rules.
    let mut state = blank_game(&setup, Map::new(18, 12, Terrain::Grassland));
    // The reproduction starts during a war; Peace would make the recorded shots illegal.
    for relation in &mut state.diplomacy.as_mut().unwrap().relations {
        relation.state = diplomacy::RelationState::War;
    }
    for (owner, pos) in [
        (PlayerId(0), Hex::from_offset(5, 5)),
        (PlayerId(1), Hex::from_offset(15, 8)),
    ] {
        let settler = spawn_unit(&mut state, owner, UnitKind::Settler, pos);
        found_city(&mut state, settler, &mut vec![]).expect("fixture city");
        state.player_mut(owner).gold = 5000;
    }
    let guard = spawn_unit(
        &mut state,
        PlayerId(0),
        UnitKind::Soldier,
        Hex::from_offset(5, 5),
    );
    let victim = spawn_unit(
        &mut state,
        PlayerId(0),
        UnitKind::Soldier,
        Hex::from_offset(10, 3),
    );
    let scout = spawn_unit(
        &mut state,
        PlayerId(0),
        UnitKind::Scout,
        Hex::from_offset(3, 8),
    );
    let archer = spawn_unit(
        &mut state,
        PlayerId(1),
        UnitKind::Ranger,
        Hex::from_offset(7, 5),
    );
    let hunter = spawn_unit(
        &mut state,
        PlayerId(1),
        UnitKind::Ranger,
        Hex::from_offset(12, 3),
    );
    spawn_unit(
        &mut state,
        PlayerId(1),
        UnitKind::Soldier,
        Hex::from_offset(4, 8),
    );
    legatum_core::visibility::update_explored(&mut state);
    let mut recording = Recording::new(&state);
    recording.metadata.game_id = "ranger-passivity-repro".into();
    for _ in 0..4 {
        let orders = OrderMap::from([
            (
                PlayerId(0),
                vec![
                    Order::SkipUnit { unit: guard },
                    Order::SkipUnit { unit: victim },
                    Order::SkipUnit { unit: scout },
                ],
            ),
            (
                PlayerId(1),
                vec![
                    Order::RangedAttack {
                        unit: archer,
                        target: Hex::from_offset(5, 5),
                    },
                    Order::RangedAttack {
                        unit: hunter,
                        target: Hex::from_offset(10, 3),
                    },
                ],
            ),
        ]);
        let turn = state.turn;
        state = resolve_turn(&state, &orders).0;
        recording.observe(turn, &orders, &state);
    }
    recording.stop("stalled");
    recording
}

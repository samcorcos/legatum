use legatum_core::{PlayerId, PlayerView};
use legatum_wasm::{Game, json::parse_state};

#[test]
fn playtest_893_german_unit_inside_india_borders_is_visible() {
    let source = include_str!("fixtures/saves/p893-unseen-attacker-2119.current.legatum");
    let envelope: serde_json::Value = serde_json::from_str(source).unwrap();
    let game_json = envelope["game"].as_str().unwrap();
    let state = parse_state(game_json).unwrap();
    let game = Game::from_save_json(game_json).unwrap();
    let view: PlayerView = serde_json::from_str(&game.view_json(0).unwrap()).unwrap();

    let india = PlayerId(0);
    let german = PlayerId(1);
    let affected: Vec<_> = state
        .units
        .values()
        .filter(|unit| unit.owner == german)
        .filter(|unit| {
            state
                .map
                .get(unit.pos)
                .is_some_and(|tile| tile.owner == Some(india))
        })
        .collect();
    assert!(
        !affected.is_empty(),
        "save still contains a German unit in India's land"
    );
    for unit in affected {
        assert!(view.tile(unit.pos).unwrap().visible, "tile {:?}", unit.pos);
        assert!(
            view.units.iter().any(|shown| shown.id == unit.id),
            "unit {:?}",
            unit.id
        );
    }
}

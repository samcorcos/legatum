//! Browser regressions use current-rule copies of recorded states. Loading these through the
//! actual save entry point catches stale fixture shapes before Playwright waits for Continue.
use legatum_wasm::Game;
use std::io::Read;

#[test]
fn current_browser_saves_load() {
    let mut failures = Vec::new();
    for (name, save) in [
        (
            "alliance-loop",
            include_bytes!("../../../app/e2e/saves/p892-alliance-loop-t20.current.legatum.gz")
                .as_slice(),
        ),
        (
            "race-panel",
            include_bytes!("../../../app/e2e/saves/p892-race-panel-t203.current.legatum.gz")
                .as_slice(),
        ),
        (
            "sidebar",
            include_bytes!("../../../app/e2e/saves/sidebar-mid-t117.current.legatum.gz").as_slice(),
        ),
    ] {
        let mut text = String::new();
        flate2::read::GzDecoder::new(save)
            .read_to_string(&mut text)
            .unwrap();
        let wrapper: serde_json::Value = serde_json::from_str(&text).unwrap();
        if let Err(error) = Game::from_save_json(wrapper["game"].as_str().unwrap()) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, text) in [
        (
            "blue-rects",
            include_str!("fixtures/saves/p892-blue-rects-t117.current.legatum"),
        ),
        (
            "sea-rect",
            include_str!("fixtures/saves/p892-sea-rect-t141.current.legatum"),
        ),
        (
            "rects-t36",
            include_str!("fixtures/saves/p894-rects-t36.current.legatum"),
        ),
        (
            "zoom-scout",
            include_str!("fixtures/saves/p893-zoom-scout-t34.current.legatum"),
        ),
    ] {
        let wrapper: serde_json::Value = serde_json::from_str(text).unwrap();
        if let Err(error) = Game::from_save_json(wrapper["game"].as_str().unwrap()) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, state) in [
        (
            "postgame-swiss",
            include_str!("../../../app/src/test/fixtures/postgame-swiss-win.current.json"),
        ),
        (
            "postgame-rival",
            include_str!("../../../app/src/test/fixtures/postgame-rival-win.current.json"),
        ),
    ] {
        if let Err(error) = Game::from_save_json(state) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

use legatum_wasm::tape::Tape;

#[test]
fn actual_version_one_original_three_tape_is_retired() {
    let err = Tape::from_json(include_str!("fixtures/original-three-tape.json")).unwrap_err();
    assert_eq!(err, "unsupported tape version 1");
}

//! Playtest 8.9.2, turn 20 (user, 2026-10-17): Switzerland (team 6) offered Sparta's player an
//! Alliance. He declined, then declared War on Switzerland and ordered attacks; the offer came
//! back as the turn's primary decision, and answering it again cancelled the war, so his attacks
//! failed. The app kept only one diplomatic order per empire (core's rule) and dropped the
//! decline for the war, then the war for the decline. Core's side of the contract, on his save:
//! the war alone settles the offer, the attacks resolve, and the offer is gone afterwards.
use legatum_wasm::Game;
use serde_json::{Value, json};

const SWITZERLAND: u64 = 6;
const OFFER: u64 = 2;

fn game() -> Game {
    let save: Value = serde_json::from_str(include_str!(
        "fixtures/saves/p892-alliance-loop-t20.current.legatum"
    ))
    .unwrap();
    Game::from_save_json(save["game"].as_str().unwrap()).unwrap()
}

fn offers_to_me(game: &Game) -> Vec<Value> {
    let view: Value = serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
    view["diplomacy"]["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["to"] == 0)
        .cloned()
        .collect()
}

fn relation_state(game: &Game) -> Value {
    let view: Value = serde_json::from_str(&game.view_json(0).unwrap()).unwrap();
    view["diplomacy"]["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["a"] == 0 && r["b"] == SWITZERLAND)
        .unwrap()["state"]
        .clone()
}

fn events(result: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(result).unwrap();
    v["events_by_player"]["0"].as_array().unwrap().clone()
}

/// The declaration and two Spearmen sent at Neuchâtel (29,22) and its defender beside it.
fn war_and_attacks() -> Value {
    json!([
        { "type": "DeclareWar", "against": SWITZERLAND, "breach": false },
        { "type": "MoveUnit", "unit": 26, "to": { "q": 28, "r": 22 } },
        { "type": "MoveUnit", "unit": 3, "to": { "q": 29, "r": 22 } },
    ])
}

#[test]
fn the_save_has_switzerlands_alliance_offer_pending_at_peace() {
    let game = game();
    assert_eq!(game.turn(), 20);
    let offers = offers_to_me(&game);
    assert_eq!(offers.len(), 1);
    assert_eq!(offers[0]["id"], OFFER);
    assert_eq!(offers[0]["from"], SWITZERLAND);
    assert_eq!(offers[0]["kind"], "Alliance");
    assert_eq!(relation_state(&game), "Peace");
}

#[test]
fn declaring_war_on_the_sender_withdraws_its_offer_and_the_attacks_resolve() {
    let mut game = game();
    let orders = war_and_attacks();
    let errors: Value =
        serde_json::from_str(&game.validate_json(0, &orders.to_string()).unwrap()).unwrap();
    assert_eq!(
        errors,
        json!([]),
        "war plus attacks, with no reply, is a valid plan"
    );
    let result = game
        .end_turn_json(&json!({ "0": orders }).to_string())
        .unwrap();
    let events = events(&result);
    assert!(
        events.iter().any(|e| e["type"] == "WarDeclared"
            && e["relation"]["effective_turn"] == 20
            && e["relation"]["b"] == SWITZERLAND),
        "a surprise war starts this turn: {events:#?}"
    );
    // The Spearman next to Neuchâtel strikes this turn; the one two tiles out closes in (its
    // target stays out of reach this turn), and neither waits for a war that never came.
    assert!(
        events
            .iter()
            .any(|e| e["type"] == "Battle" && e["attacker"] == 3),
        "the attack goes in this turn: {events:#?}"
    );
    assert!(
        !events.iter().any(
            |e| (e["type"] == "AttackPursuit" && e["outcome"] == "AwaitingWar")
                || e["type"] == "MoveBlocked"
        ),
        "no attack waits for War: {events:#?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| e["type"] == "OrderFailed" && e["player"] == 0),
        "no order failed: {events:#?}"
    );
    let withdrawn = events
        .iter()
        .find(|e| e["type"] == "ProposalInvalidated" && e["proposal"]["id"] == OFFER)
        .unwrap_or_else(|| panic!("the war withdraws the offer: {events:#?}"));
    assert_eq!(withdrawn["reason"], "War was declared");
    assert_eq!(game.turn(), 21);
    assert_eq!(relation_state(&game), "War");
    assert!(
        offers_to_me(&game).is_empty(),
        "nothing left to answer on turn 21"
    );
}

#[test]
fn a_declined_offer_is_gone_next_turn() {
    let mut game = game();
    let decline = json!([{ "type": "RespondToProposal", "proposal": OFFER, "accept": false }]);
    let result = game
        .end_turn_json(&json!({ "0": decline }).to_string())
        .unwrap();
    assert!(
        events(&result)
            .iter()
            .any(|e| e["type"] == "ProposalRejected" && e["proposal"]["id"] == OFFER)
    );
    assert!(offers_to_me(&game).is_empty());
    assert_eq!(relation_state(&game), "Peace");
}

#[test]
fn core_takes_one_diplomatic_order_per_empire_so_the_client_must_not_send_both() {
    // The contract the app's queue follows: a reply and a war toward the same empire conflict.
    // The app now treats the war as settling the offer, so it never needs to send both.
    let game = game();
    let both = json!([
        { "type": "RespondToProposal", "proposal": OFFER, "accept": false },
        { "type": "DeclareWar", "against": SWITZERLAND, "breach": false },
    ]);
    let errors: Value =
        serde_json::from_str(&game.validate_json(0, &both.to_string()).unwrap()).unwrap();
    assert_eq!(errors.as_array().unwrap().len(), 1);
    assert_eq!(errors[0]["index"], 1);
}

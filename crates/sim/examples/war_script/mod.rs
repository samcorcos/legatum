//! Scripted seats for the war scenarios (shared by `war_suite` and `war_trace`).
//!
//! In most scenarios every empire is a Hard AI, including the seat the designer played. That
//! cannot reproduce an operation the AI does not know how to run: in the Japan air-war game the
//! designer bombed a German city empty with four Bombers and walked an Autonomous Armour in behind
//! them, and Japan's Hard AI given the same forces bombs but never captures (Germany then holds
//! every city in 64 of 64 runs). A scenario can therefore carry a script, the designer's own
//! recorded orders for his seat, replayed verbatim for the turns of his window while everything
//! else, the defender above all, is played by the AI. Orders that no longer fit the position (a
//! target that moved, a unit that died) are skipped by the engine, as in any turn.
//!
//! The scripts are `scenarios/war/scripts/<scenario id>.orders.json`, written by
//! `legatum-sim war-build` from `orders-<seat>.json`, which `tools/scenarios/extract-war-states.sh`
//! records from the save's tape with the engine it was made on.
use legatum_core::{Order, PlayerId};
use serde::Deserialize;
use std::collections::BTreeMap;

const SCRIPTS: [(&str, &str); 8] = [
    (
        "war-athens-two-pikemen-t112",
        include_str!("../../../../scenarios/war/scripts/war-athens-two-pikemen-t112.orders.json"),
    ),
    (
        "war-break-the-hold-t168",
        include_str!("../../../../scenarios/war/scripts/war-break-the-hold-t168.orders.json"),
    ),
    (
        "war-german-air-early",
        include_str!("../../../../scenarios/war/scripts/war-german-air-early.orders.json"),
    ),
    (
        "war-german-air-t219",
        include_str!("../../../../scenarios/war/scripts/war-german-air-t219.orders.json"),
    ),
    (
        "war-mongol-bombers-t150",
        include_str!("../../../../scenarios/war/scripts/war-mongol-bombers-t150.orders.json"),
    ),
    (
        "war-hollow-swiss-t76",
        include_str!("../../../../scenarios/war/scripts/war-hollow-swiss-t76.orders.json"),
    ),
    (
        "war-mongols-chur-t137",
        include_str!("../../../../scenarios/war/scripts/war-mongols-chur-t137.orders.json"),
    ),
    (
        "war-sparta-next-door-t101",
        include_str!("../../../../scenarios/war/scripts/war-sparta-next-door-t101.orders.json"),
    ),
];

#[derive(Deserialize)]
struct Raw {
    seat: u8,
    turns: BTreeMap<String, Vec<Order>>,
}

/// The recorded orders of one seat, by game turn.
pub struct Script {
    pub seat: PlayerId,
    pub turns: BTreeMap<u32, Vec<Order>>,
}

impl Script {
    /// Replace the scripted seat's orders for `turn`, if the script has them.
    pub fn apply(&self, turn: u32, orders: &mut BTreeMap<PlayerId, Vec<Order>>) {
        if let Some(os) = self.turns.get(&turn) {
            orders.insert(self.seat, os.clone());
        }
    }
}

/// The script of scenario `id`, if it has one.
pub fn load(id: &str) -> Result<Option<Script>, String> {
    let Some((_, text)) = SCRIPTS.iter().find(|(name, _)| *name == id) else {
        return Ok(None);
    };
    let raw: Raw = serde_json::from_str(text).map_err(|e| format!("script {id}: {e}"))?;
    let mut turns = BTreeMap::new();
    for (turn, orders) in raw.turns {
        let turn = turn
            .parse()
            .map_err(|_| format!("script {id}: turn {turn} is not a number"))?;
        turns.insert(turn, orders);
    }
    Ok(Some(Script {
        seat: PlayerId(raw.seat),
        turns,
    }))
}

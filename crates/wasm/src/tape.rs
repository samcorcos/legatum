//! Order tapes: a game's setup, every turn's orders and the state hash after each turn. The
//! runner writes them natively; replaying them in WASM (Node) and comparing hashes is the
//! cross-target determinism check. Hashes are decimal strings so JavaScript can read them.
//!
//! Current tapes use version 6. Versions 4 and 5 can be loaded for fixture rehashing under the
//! current rules; older diplomacy states are unsupported.
//! Format:
//! `{"version":6,"setup":{..,"seed":"<decimal>"},"initial_hash":"<decimal>",
//!   "turns":[{"turn":1,"orders":{"0":[..],..},"hash":"<decimal>"},..]}`

use crate::json;
use crate::session::OrderMap;
use legatum_core::{GameSetup, GameState, new_game, resolve_turn, try_new_game};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Games record first contact and calls to arms.
pub const TAPE_VERSION: u32 = 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TapeTurn {
    /// The turn these orders were given for (state.turn before resolution).
    pub turn: u32,
    pub orders: OrderMap,
    /// State hash after resolving this turn, decimal.
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tape {
    pub version: u32,
    pub setup: GameSetup,
    pub initial_hash: u64,
    pub turns: Vec<TapeTurn>,
}

#[derive(Serialize, Deserialize)]
struct TapeJson {
    version: u32,
    setup: Value,
    initial_hash: String,
    turns: Vec<TapeTurn>,
}

impl Tape {
    pub fn initial_state(&self) -> GameState {
        new_game(&self.setup)
    }

    fn try_initial_state(&self) -> Result<GameState, String> {
        try_new_game(&self.setup)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&TapeJson {
            version: self.version,
            setup: json::setup_to_value(&self.setup),
            initial_hash: self.initial_hash.to_string(),
            turns: self.turns.clone(),
        })
        .expect("tape serializes")
    }

    pub fn from_json(s: &str) -> Result<Tape, String> {
        // Check the envelope before decoding orders: historical v2 orders may
        // use an older schema (for example BuyItem without an explicit item).
        let mut value: Value = serde_json::from_str(s).map_err(|e| format!("invalid tape: {e}"))?;
        let version = value
            .get("version")
            .and_then(Value::as_u64)
            .ok_or("invalid tape: version must be an unsigned integer")?;
        if version == 2 {
            return Err("unsupported tape version 2: historical diplomacy hashes are not replayable; regenerate from the original scenario under version 3".into());
        }
        if version != 4 && version != 5 && version != u64::from(TAPE_VERSION) {
            return Err(format!("unsupported tape version {version}"));
        }
        // Replays from before Embassy and Archive were removed can still be rehashed. Their
        // production orders are stale under the current rules, so discard those orders.
        if let Some(turns) = value.get_mut("turns").and_then(Value::as_array_mut) {
            for turn in turns {
                let Some(orders) = turn.get_mut("orders").and_then(Value::as_object_mut) else {
                    continue;
                };
                for player_orders in orders.values_mut().filter_map(Value::as_array_mut) {
                    player_orders.retain_mut(|order| {
                        let kind = order.get("type").and_then(Value::as_str).map(str::to_owned);
                        let replacement = match kind.as_deref() {
                            Some("HealUnit" | "HoldUnit") => Some("SleepUnit"),
                            Some("WakeUnit") => Some("SkipUnit"),
                            _ => None,
                        };
                        if let Some(replacement) = replacement {
                            order["type"] = Value::String(replacement.into());
                        }
                        let known = matches!(
                            replacement.or(kind.as_deref()),
                            Some(
                                "DeclareWar"
                                    | "BreakTreaty"
                                    | "ProposeTreaty"
                                    | "RespondToProposal"
                                    | "WithdrawProposal"
                                    | "SetAutomation"
                                    | "SetAutoFire"
                                    | "MoveUnit"
                                    | "MoveUnitPath"
                                    | "SleepUnit"
                                    | "SkipUnit"
                                    | "FoundCity"
                                    | "DisbandUnit"
                                    | "Raze"
                                    | "SetProduction"
                                    | "SetProductionQueue"
                                    | "BuyItem"
                                    | "HireCompany"
                                    | "BuyTile"
                                    | "SetResearch"
                                    | "SetResearchQueue"
                                    | "RangedAttack"
                                    | "UpgradeUnit"
                                    | "UpgradeSiege"
                                    | "AirStrike"
                                    | "Rebase"
                                    | "AirRecon"
                                    | "Action"
                                    | "UpgradeAircraft"
                                    | "SetScienceConversion"
                                    | "SetCityAutomation"
                                    | "GiveGold"
                            )
                        );
                        if !known {
                            eprintln!(
                                "dropped unknown tape order kind: {}",
                                kind.as_deref().unwrap_or("<missing>")
                            );
                        }
                        // ClaimObjective was an Action payload before the World Age
                        // Veterans card replaced it. It cannot be decoded by today's
                        // Action enum and has no current-rule equivalent.
                        let removed_objective = kind.as_deref() == Some("Action")
                            && order.pointer("/action/action").and_then(Value::as_str)
                                == Some("ClaimObjective");
                        known && !removed_objective && !contains_removed_building_item(order)
                    });
                }
            }
        }
        let t: TapeJson =
            serde_json::from_value(value).map_err(|e| format!("invalid tape: {e}"))?;
        Ok(Tape {
            version: t.version,
            setup: json::setup_from_value(t.setup)?,
            initial_hash: parse_hash(&t.initial_hash)?,
            turns: t.turns,
        })
    }
}

fn contains_removed_building_item(value: &Value) -> bool {
    match value {
        Value::Object(fields) => {
            (fields.get("kind").and_then(Value::as_str) == Some("Building")
                && matches!(
                    fields.get("building").and_then(Value::as_str),
                    Some("Embassy" | "Archive")
                ))
                || fields.values().any(contains_removed_building_item)
        }
        Value::Array(values) => values.iter().any(contains_removed_building_item),
        _ => false,
    }
}

pub fn parse_hash(s: &str) -> Result<u64, String> {
    s.parse()
        .map_err(|_| format!("hash must be a decimal u64, got {s:?}"))
}

/// Result of replaying a tape through the core (no AI).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    pub initial_hash: u64,
    /// Hash after each replayed turn.
    pub hashes: Vec<u64>,
}

impl Replay {
    /// The first mismatch against the tape: `Some(0)` = initial state, `Some(n)` = after the
    /// n-th tape turn (1-based). `None` when everything matches.
    pub fn first_mismatch(&self, tape: &Tape) -> Option<usize> {
        if self.initial_hash != tape.initial_hash {
            return Some(0);
        }
        for (i, t) in tape.turns.iter().enumerate() {
            match self.hashes.get(i) {
                Some(h) if h.to_string() == t.hash => {}
                _ => return Some(i + 1),
            }
        }
        (self.hashes.len() > tape.turns.len()).then_some(tape.turns.len() + 1)
    }
}

/// Replay every turn of the tape through `resolve_turn` and record the hashes.
pub fn replay(tape: &Tape) -> Replay {
    let mut state = tape.initial_state();
    let initial_hash = state.hash();
    let mut hashes = Vec::with_capacity(tape.turns.len());
    for t in &tape.turns {
        state = resolve_turn(&state, &t.orders).0;
        hashes.push(state.hash());
    }
    Replay {
        initial_hash,
        hashes,
    }
}

/// Check the initial state and every turn against the recorded hashes, both uninterrupted
/// and with a save/load before the first turn and after every turn (including the last).
/// The restored state is used for the next turn, so this also checks continuation after load.
pub fn replay_checked(tape: &Tape) -> Result<Replay, String> {
    let mut initial = tape.try_initial_state()?;
    let initial_hash = initial.hash();
    let mut hashes = Vec::with_capacity(tape.turns.len());
    for turn in &tape.turns {
        initial = resolve_turn(&initial, &turn.orders).0;
        hashes.push(initial.hash());
    }
    let replay = Replay {
        initial_hash,
        hashes,
    };
    match replay.first_mismatch(tape) {
        Some(0) => {
            return Err(format!(
                "initial state hash differs: expected {}, got {}",
                tape.initial_hash, replay.initial_hash
            ));
        }
        Some(t) => {
            let expected = tape
                .turns
                .get(t - 1)
                .map_or("<missing>", |turn| turn.hash.as_str());
            let actual = replay
                .hashes
                .get(t - 1)
                .map_or_else(|| "<missing>".into(), u64::to_string);
            return Err(format!(
                "hash differs after turn {t}: expected {expected}, got {actual}"
            ));
        }
        None => {}
    }

    let mut state = save_load(&tape.try_initial_state()?, 0)?;
    let initial_hash = state.hash();
    if initial_hash != replay.initial_hash {
        return Err(format!(
            "initial state hash differs after save/load: expected {}, got {initial_hash}",
            tape.initial_hash
        ));
    }
    for (i, t) in tape.turns.iter().enumerate() {
        if t.turn != state.turn {
            return Err(format!(
                "tape turn {} has turn number {}, expected {}",
                i + 1,
                t.turn,
                state.turn
            ));
        }
        state = resolve_turn(&state, &t.orders).0;
        state = save_load(&state, t.turn)?;
        let hash = state.hash();
        if hash != replay.hashes[i] {
            return Err(format!(
                "hash differs after turn {} when continuing from save/load: expected {}, got {hash}",
                t.turn, t.hash
            ));
        }
    }
    Ok(replay)
}

fn save_load(state: &GameState, after_turn: u32) -> Result<GameState, String> {
    let loaded = json::parse_state(&json::state_to_json(state))
        .map_err(|e| format!("save/load failed after turn {after_turn}: {e}"))?;
    if loaded != *state {
        return Err(format!("save/load changed state after turn {after_turn}"));
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{CityId, Hex, Order, PlayerId, ProductionItem, UnitId, UnitKind};

    fn sample() -> Tape {
        let mut orders = OrderMap::new();
        orders.insert(
            PlayerId(0),
            vec![Order::MoveUnit {
                unit: UnitId(3),
                to: Hex::new(4, -2),
            }],
        );
        orders.insert(
            PlayerId(7),
            vec![Order::SetProduction {
                city: CityId(1),
                item: ProductionItem::Unit {
                    unit: UnitKind::Settler,
                },
            }],
        );
        Tape {
            version: TAPE_VERSION,
            setup: GameSetup::all_ai(u64::MAX),
            initial_hash: u64::MAX - 1,
            turns: vec![TapeTurn {
                turn: 1,
                orders,
                hash: (u64::MAX - 2).to_string(),
            }],
        }
    }

    #[test]
    fn version_two_is_rejected_explicitly_instead_of_reinterpreting_its_rules() {
        let mut tape = sample();
        tape.version = 2;
        assert!(
            Tape::from_json(&tape.to_json())
                .unwrap_err()
                .contains("unsupported tape version 2")
        );
        let mut historical: Value = serde_json::from_str(&tape.to_json()).unwrap();
        historical["turns"][0]["orders"] = serde_json::json!({
            "0": [{ "type": "BuyItem", "city": 1 }]
        });
        assert!(
            Tape::from_json(&historical.to_string())
                .unwrap_err()
                .contains("unsupported tape version 2")
        );
        tape.version = 1;
        assert!(
            Tape::from_json(&tape.to_json())
                .unwrap_err()
                .contains("unsupported tape version 1")
        );
    }

    #[test]
    fn tape_round_trips_with_string_seed_and_player_keys() {
        let tape = sample();
        let json = tape.to_json();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["setup"]["seed"], Value::from(u64::MAX.to_string()));
        assert!(v["turns"][0]["orders"]["7"].is_array());
        assert_eq!(Tape::from_json(&json).unwrap(), tape);
    }

    #[test]
    fn retired_and_unknown_orders_do_not_prevent_tape_load() {
        let mut json: Value = serde_json::from_str(&sample().to_json()).unwrap();
        json["turns"][0]["orders"]["0"] = serde_json::json!([
            { "type": "HealUnit", "unit": 3 },
            { "type": "HoldUnit", "unit": 4 },
            { "type": "FutureOrder", "unit": 5 }
        ]);
        let tape = Tape::from_json(&json.to_string()).unwrap();
        assert_eq!(
            tape.turns[0].orders[&PlayerId(0)],
            vec![
                Order::SleepUnit {
                    unit: UnitId(3),
                    until_healed: false
                },
                Order::SleepUnit {
                    unit: UnitId(4),
                    until_healed: false
                },
            ]
        );
    }

    #[test]
    fn retired_claim_objective_action_does_not_prevent_tape_load() {
        let mut json: Value = serde_json::from_str(&sample().to_json()).unwrap();
        json["turns"][0]["orders"]["0"] = serde_json::json!([
            { "type": "Action", "action": { "action": "ClaimObjective", "card": 0, "era": 6 } },
            { "type": "SleepUnit", "unit": 3 }
        ]);
        let tape = Tape::from_json(&json.to_string()).unwrap();
        assert_eq!(
            tape.turns[0].orders[&PlayerId(0)],
            vec![Order::SleepUnit {
                unit: UnitId(3),
                until_healed: false
            }]
        );
    }

    #[test]
    fn a_heal_order_survives_a_tape_round_trip_while_old_heal_names_stay_sleep() {
        let mut json: Value = serde_json::from_str(&sample().to_json()).unwrap();
        json["turns"][0]["orders"]["0"] = serde_json::json!([
            { "type": "SleepUnit", "unit": 3, "until_healed": true },
            { "type": "HealUnit", "unit": 4 },
            { "type": "SleepUnit", "unit": 5 }
        ]);
        let tape = Tape::from_json(&json.to_string()).unwrap();
        assert_eq!(
            tape.turns[0].orders[&PlayerId(0)],
            vec![
                Order::SleepUnit {
                    unit: UnitId(3),
                    until_healed: true
                },
                Order::SleepUnit {
                    unit: UnitId(4),
                    until_healed: false
                },
                Order::SleepUnit {
                    unit: UnitId(5),
                    until_healed: false
                },
            ]
        );
        assert_eq!(Tape::from_json(&tape.to_json()).unwrap(), tape);
    }

    /// One order of every `Order` variant. `variant_index` has no wildcard arm, so a new
    /// variant fails to compile until it is listed there and given a sample here.
    fn one_order_of_every_kind() -> Vec<Order> {
        use legatum_core::diplomacy::{Agreement, ProposalId, TreatyKind};
        use legatum_core::orders::Action;
        use legatum_core::{Automation, CityBuildFocus, ScienceConversion, TeamId, TechId};
        let settler = ProductionItem::Unit {
            unit: UnitKind::Settler,
        };
        vec![
            Order::DeclareWar {
                against: TeamId(1),
                breach: true,
            },
            Order::BreakTreaty {
                with: TeamId(2),
                agreement: Agreement::Security,
            },
            Order::ProposeTreaty {
                to: TeamId(3),
                kind: TreatyKind::Alliance,
            },
            Order::RespondToProposal {
                proposal: ProposalId(4),
                accept: true,
                explanation: None,
            },
            Order::WithdrawProposal {
                proposal: ProposalId(5),
            },
            Order::SetAutomation {
                unit: UnitId(6),
                automation: Some(Automation::Explore),
                path: vec![Hex::new(1, 0)],
                stop_reason: None,
            },
            Order::SetAutoFire {
                unit: UnitId(7),
                enabled: true,
                target: Some(Hex::new(2, -1)),
            },
            Order::MoveUnit {
                unit: UnitId(8),
                to: Hex::new(3, -1),
            },
            Order::MoveUnitPath {
                unit: UnitId(9),
                to: Hex::new(2, 0),
                path: vec![Hex::new(1, 0), Hex::new(2, 0)],
                stop_on_block: true,
            },
            Order::SleepUnit {
                unit: UnitId(10),
                until_healed: true,
            },
            Order::SkipUnit { unit: UnitId(11) },
            Order::FoundCity { unit: UnitId(12) },
            Order::DisbandUnit { unit: UnitId(13) },
            Order::Raze { city: CityId(14) },
            Order::SetProduction {
                city: CityId(15),
                item: settler,
            },
            Order::SetCityAutomation {
                city: CityId(16),
                focus: Some(CityBuildFocus::Military),
                picked: Some(settler),
            },
            Order::SetProductionQueue {
                city: CityId(17),
                queue: vec![settler],
            },
            Order::BuyItem {
                city: CityId(18),
                item: settler,
            },
            Order::HireCompany {
                city: CityId(19),
                unit: UnitKind::Settler,
                seller: PlayerId(2),
            },
            Order::BuyTile {
                city: CityId(28),
                tile: Hex::new(1, 1),
            },
            Order::SetResearch {
                tech: TechId::Masonry,
            },
            Order::SetResearchQueue {
                techs: vec![TechId::Writing, TechId::Currency],
            },
            Order::SetScienceConversion {
                to: ScienceConversion::Production,
            },
            Order::RangedAttack {
                unit: UnitId(20),
                target: Hex::new(0, 2),
            },
            Order::UpgradeUnit { unit: UnitId(21) },
            Order::UpgradeSiege { unit: UnitId(22) },
            Order::AirStrike {
                plane: UnitId(23),
                target: Hex::new(-1, 2),
            },
            Order::Rebase {
                plane: UnitId(24),
                to: CityId(25),
            },
            Order::UpgradeAircraft {
                plane: UnitId(26),
                to: None,
            },
            Order::AirRecon {
                plane: UnitId(27),
                target: Hex::new(3, -2),
            },
            Order::Action {
                action: Action::Broadcast,
            },
            Order::GiveGold {
                to: PlayerId(1),
                amount: 25,
            },
        ]
    }

    const ORDER_VARIANTS: usize = 32;

    fn variant_index(order: &Order) -> usize {
        match order {
            Order::DeclareWar { .. } => 0,
            Order::BreakTreaty { .. } => 1,
            Order::ProposeTreaty { .. } => 2,
            Order::RespondToProposal { .. } => 3,
            Order::WithdrawProposal { .. } => 4,
            Order::SetAutomation { .. } => 5,
            Order::SetAutoFire { .. } => 6,
            Order::MoveUnit { .. } => 7,
            Order::MoveUnitPath { .. } => 8,
            Order::SleepUnit { .. } => 9,
            Order::SkipUnit { .. } => 10,
            Order::FoundCity { .. } => 11,
            Order::DisbandUnit { .. } => 12,
            Order::Raze { .. } => 13,
            Order::SetProduction { .. } => 14,
            Order::SetCityAutomation { .. } => 15,
            Order::SetProductionQueue { .. } => 16,
            Order::BuyItem { .. } => 17,
            Order::HireCompany { .. } => 18,
            Order::SetResearch { .. } => 19,
            Order::SetResearchQueue { .. } => 20,
            Order::SetScienceConversion { .. } => 21,
            Order::RangedAttack { .. } => 22,
            Order::UpgradeUnit { .. } => 23,
            Order::UpgradeSiege { .. } => 24,
            Order::AirStrike { .. } => 25,
            Order::Rebase { .. } => 26,
            Order::UpgradeAircraft { .. } => 27,
            Order::Action { .. } => 28,
            Order::AirRecon { .. } => 29,
            Order::BuyTile { .. } => 30,
            Order::GiveGold { .. } => 31,
        }
    }

    /// 2026-11-02: `from_json` dropped `UpgradeAircraft`, `SetScienceConversion` and
    /// `SetCityAutomation` as unknown, so late-game tapes replayed as a different game.
    #[test]
    fn every_order_kind_survives_a_tape_round_trip() {
        let orders = one_order_of_every_kind();
        let mut seen = [false; ORDER_VARIANTS];
        for order in &orders {
            seen[variant_index(order)] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "one_order_of_every_kind misses a variant: {seen:?}"
        );
        let mut tape = sample();
        tape.turns[0].orders.insert(PlayerId(0), orders.clone());
        let back = Tape::from_json(&tape.to_json()).unwrap();
        assert_eq!(back.turns[0].orders[&PlayerId(0)], orders);
        assert_eq!(back, tape);
    }

    #[test]
    fn mismatch_reporting() {
        let tape = sample();
        let good = Replay {
            initial_hash: u64::MAX - 1,
            hashes: vec![u64::MAX - 2],
        };
        assert_eq!(good.first_mismatch(&tape), None);
        let bad_turn = Replay {
            hashes: vec![5],
            ..good.clone()
        };
        assert_eq!(bad_turn.first_mismatch(&tape), Some(1));
        let short = Replay {
            hashes: vec![],
            ..good.clone()
        };
        assert_eq!(short.first_mismatch(&tape), Some(1));
        let long = Replay {
            hashes: vec![u64::MAX - 2, 5],
            ..good.clone()
        };
        assert_eq!(long.first_mismatch(&tape), Some(2));
        let bad_init = Replay {
            initial_hash: 0,
            ..good
        };
        assert_eq!(bad_init.first_mismatch(&tape), Some(0));
    }
}

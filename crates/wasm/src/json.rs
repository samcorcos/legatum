//! JSON helpers for the JavaScript boundary. 64-bit integers cross into JavaScript as decimal
//! strings (JS numbers lose precision above 2^53), so the setup seed is written as a string and
//! accepted as either a string or a number. Core types stay unchanged.

use legatum_core::{GameSetup, GameState};
use serde_json::Value;

/// Maximum UTF-8 bytes in an engine save or setup before allocating a JSON tree.
/// Keep in sync with app/src/platform/saveLimits.ts for the outer save envelope.
pub const MAX_SAVE_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn check_json_size(json: &str) -> Result<(), String> {
    if json.len() > MAX_SAVE_BYTES {
        return Err(format!(
            "Save exceeds the {} MiB limit",
            MAX_SAVE_BYTES / 1024 / 1024
        ));
    }
    Ok(())
}

/// Rewrite `obj[key]` from a decimal string to a JSON number, if it is a string.
fn seed_from_string(obj: &mut Value, key: &str) -> Result<(), String> {
    if let Some(v) = obj.get_mut(key)
        && let Some(s) = v.as_str()
    {
        let n: u64 = s
            .trim()
            .parse()
            .map_err(|_| format!("{key} must be a decimal u64, got {s:?}"))?;
        *v = Value::from(n);
    }
    Ok(())
}

/// Rewrite `obj[key]` from a JSON number to a decimal string.
fn seed_to_string(obj: &mut Value, key: &str) {
    if let Some(v) = obj.get_mut(key)
        && let Some(n) = v.as_u64()
    {
        *v = Value::from(n.to_string());
    }
}

/// Parse a `GameSetup` whose `seed` is a JSON number or a decimal string.
pub fn setup_from_value(mut v: Value) -> Result<GameSetup, String> {
    seed_from_string(&mut v, "seed")?;
    let setup: GameSetup =
        serde_json::from_value(v).map_err(|e| format!("invalid GameSetup: {e}"))?;
    legatum_core::validation::check_setup(&setup)?;
    Ok(setup)
}

pub fn parse_setup(json: &str) -> Result<GameSetup, String> {
    check_json_size(json)?;
    let v: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    setup_from_value(v)
}

/// A `GameSetup` as JSON with the seed as a decimal string.
pub fn setup_to_value(setup: &GameSetup) -> Value {
    let mut v = serde_json::to_value(setup).expect("setup serializes");
    seed_to_string(&mut v, "seed");
    v
}

/// Full state JSON for saves, with the setup seed as a decimal string. Treat it as opaque in
/// JavaScript: other 64-bit words (explored-tile bitsets) are still JSON numbers.
pub fn state_to_json(state: &GameState) -> String {
    state_to_value(state).to_string()
}

/// `state_to_json` as a JSON tree, for callers that add fields to the save before writing it.
pub fn state_to_value(state: &GameState) -> Value {
    let mut v = serde_json::to_value(state).expect("state serializes");
    v["save_version"] = Value::from(legatum_core::RULES_VERSION);
    if let Some(setup) = v.get_mut("setup") {
        seed_to_string(setup, "seed");
    }
    v
}

pub fn parse_state(json: &str) -> Result<GameState, String> {
    parse_state_with_version(json, true)
}

/// Scenario re-encoding may read an older rules version to stamp it with the new one.
/// Gameplay and save loading always call `parse_state`, which keeps the version check.
pub fn parse_state_for_rehash(json: &str) -> Result<GameState, String> {
    parse_state_with_version(json, false)
}

fn parse_state_with_version(json: &str, check_version: bool) -> Result<GameState, String> {
    check_json_size(json)?;
    let v: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    state_from_value(v, check_version)
}

/// `parse_state` for a save already parsed into a JSON tree (the caller has checked its size).
/// Fields that are not part of the state, such as a session's archive, are ignored.
pub fn state_from_value(mut v: Value, check_version: bool) -> Result<GameState, String> {
    let version = v.get("save_version").and_then(Value::as_u64);
    if check_version && version != Some(u64::from(legatum_core::RULES_VERSION)) {
        return Err(format!(
            "incompatible rules version: saved {:?}, current {}",
            version,
            legatum_core::RULES_VERSION
        ));
    }
    if let Some(setup) = v.get_mut("setup") {
        seed_from_string(setup, "seed")?;
    }
    let state: GameState =
        serde_json::from_value(v).map_err(|e| format!("invalid saved game: {e}"))?;
    legatum_core::validation::check_state(&state)?;
    Ok(state)
}

/// The rules version `upgrade_from_49` reads.
pub const UPGRADABLE_RULES_VERSION: u32 = 49;

/// Simulation only (the app keeps refusing a version-49 save): read a rules-49 saved state as a
/// rules-50 one, for analysis that starts from recorded positions which cannot be regenerated
/// (hill-climb do-no-harm states). `json` is a save (`save_version` 49 or 50) or a bare state
/// line with no `save_version` (`audit_states` output). See `upgrade_value_from_49` for what changes.
pub fn upgrade_from_49(json: &str) -> Result<GameState, String> {
    check_json_size(json)?;
    let v: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    match v.get("save_version").and_then(Value::as_u64) {
        None => {}
        Some(n) if n == u64::from(UPGRADABLE_RULES_VERSION) => {}
        Some(n) if n == u64::from(legatum_core::RULES_VERSION) => return parse_state(json),
        Some(n) => {
            return Err(format!(
                "upgrade reads rules {UPGRADABLE_RULES_VERSION} or {}, got {n}",
                legatum_core::RULES_VERSION
            ));
        }
    }
    upgrade_value_from_49(v)
}

/// The rules-49 to rules-50 upgrade of a parsed state:
/// - `Player::capitals_taken` (49: every original capital the empire ever captured, kept for good)
///   is gone. Rules 50 derives a held capital's point from who holds it now, so a capital the
///   taker still holds keeps scoring and one it lost stops. A Celtic entry whose city no longer
///   exists was a sack, so it moves to `capitals_sacked` and keeps its point for good. Any other
///   entry whose city is gone (an ordered Raze) loses its point, as in rules 50.
/// - Removed fields (`objectives_deck`, retired `FightStats` counters, `Levels::techs`) are ignored.
/// - A scored Age record keeps its point, and its slot becomes 0 (one card per Age). A record's
///   counting snapshot is kept, so a 49 era-arm game's open eras count from the same turn. The
///   next turn end gives every living empire a record for each Age begun so far.
///
/// Faithful except for the slot, which only names which card scored (it never changes VP).
pub fn upgrade_value_from_49(mut v: Value) -> Result<GameState, String> {
    let mut taken: Vec<Vec<(u64, String)>> = Vec::new();
    if let Some(players) = v.get_mut("players").and_then(Value::as_array_mut) {
        for p in players {
            let list = p
                .as_object_mut()
                .and_then(|o| o.remove("capitals_taken"))
                .and_then(|m| m.as_object().cloned())
                .map(|m| {
                    m.into_iter()
                        .filter_map(|(k, name)| Some((k.parse().ok()?, name.as_str()?.to_owned())))
                        .collect()
                })
                .unwrap_or_default();
            taken.push(list);
        }
    }
    if let Some(setup) = v.get_mut("setup") {
        seed_from_string(setup, "seed")?;
    }
    let mut state: GameState =
        serde_json::from_value(v).map_err(|e| format!("invalid rules-49 state: {e}"))?;
    for (i, list) in taken.into_iter().enumerate() {
        let Some(p) = state.players.get(i) else {
            continue;
        };
        if p.faction != legatum_core::Faction::Celts {
            continue;
        }
        let gone: Vec<(legatum_core::CityId, String)> = list
            .into_iter()
            .map(|(id, name)| (legatum_core::CityId(id as u32), name))
            .filter(|(id, _)| !state.cities.contains_key(id))
            .collect();
        let sacked = &mut state.players[i].capitals_sacked;
        for entry in gone {
            if !sacked.iter().any(|(id, _)| *id == entry.0) {
                sacked.push(entry);
            }
        }
    }
    for p in &mut state.players {
        for record in p.eras.values_mut() {
            if let Some(score) = &mut record.scored {
                score.slot = 0;
            }
        }
    }
    legatum_core::validation::check_state(&state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_save_is_rejected_before_json_parsing() {
        let oversized = " ".repeat(MAX_SAVE_BYTES + 1);
        assert!(
            parse_state(&oversized)
                .unwrap_err()
                .contains("64 MiB limit")
        );
        assert!(
            crate::Game::from_save_json(&oversized)
                .err()
                .unwrap()
                .contains("64 MiB limit")
        );
    }

    #[test]
    fn numeric_extremes_name_the_bad_field() {
        let state = legatum_core::new_game(&GameSetup::all_ai(3));
        let mut save: Value = serde_json::from_str(&state_to_json(&state)).unwrap();
        for (value, field) in [(i32::MAX, "Gold"), (-1, "Gold")] {
            save["players"][0]["gold"] = value.into();
            assert!(parse_state(&save.to_string()).unwrap_err().contains(field));
        }
        save["players"][0]["gold"] = 0.into();
        save["players"][0]["science_stored"] = (-1).into();
        assert!(
            parse_state(&save.to_string())
                .unwrap_err()
                .contains("stored Science")
        );
    }

    #[test]
    fn damaged_structure_names_what_is_wrong() {
        let state = legatum_core::new_game(&GameSetup::all_ai(3));
        let save: Value = serde_json::from_str(&state_to_json(&state)).unwrap();
        type Damage = fn(&mut Value);
        let cases: [(&str, Damage, &str); 3] = [
            (
                "turn",
                |v| v["turn"] = 0.into(),
                "turn number is out of range",
            ),
            (
                "tiles",
                |v| {
                    v["map"]["tiles"].as_array_mut().unwrap().pop();
                },
                "tiles but its size needs",
            ),
            (
                "players",
                |v| {
                    v["players"].as_array_mut().unwrap().pop();
                },
                "players but its setup lists",
            ),
        ];
        for (name, damage, expected) in cases {
            let mut damaged = save.clone();
            damage(&mut damaged);
            let error = parse_state(&damaged.to_string()).unwrap_err();
            assert!(
                error.starts_with("The save is damaged: ") && error.contains(expected),
                "{name}: {error}"
            );
        }
    }

    #[test]
    fn current_save_round_trips() {
        let state = legatum_core::new_game(&GameSetup::all_ai(3));
        let save = state_to_json(&state);
        assert!(save.contains(&format!("\"save_version\":{}", legatum_core::RULES_VERSION)));
        assert_eq!(parse_state(&save).unwrap(), state);
    }

    #[test]
    fn rules_50_saves_round_trip_and_a_version_49_save_is_refused() {
        // Rules 47: held shots at fallen cities; 48: Egypt's Nile and its Grow project. The save
        // shape only gains the project value, so a city growing on the Nile round-trips at 48,
        // and a 47 save is refused rather than migrated. Rules 49: Egypt starts knowing the Bronze
        // Working it skips; the save shape is unchanged, and a 48 save is refused. Rules 50: a
        // rival capital's VP is held, so `Player::capitals_taken` is gone, and a 49 save is refused.
        assert_eq!(legatum_core::RULES_VERSION, 50);
        let mut setup = GameSetup::all_ai(3);
        setup.players[0].faction = legatum_core::Faction::Egypt;
        let mut state = legatum_core::new_game(&setup);
        let owner = legatum_core::PlayerId(0);
        let city = state
            .cities
            .values()
            .find(|c| c.owner == owner)
            .expect("an Egyptian capital")
            .id;
        let grow = legatum_core::ProductionItem::Project {
            project: legatum_core::IndustryProject::Grow,
        };
        let mut events = Vec::new();
        legatum_core::rules::cities::apply_city_orders(
            &mut state,
            owner,
            &[legatum_core::Order::SetProduction { city, item: grow }],
            &mut events,
        );
        assert_eq!(state.cities[&city].producing, Some(grow), "{events:?}");
        let save = state_to_json(&state);
        assert!(save.contains("\"save_version\":50"));
        assert!(!save.contains("capitals_taken"));
        assert!(
            state
                .player(owner)
                .techs
                .contains(&legatum_core::TechId::BronzeWorking)
        );
        assert!(save.contains("\"Grow\""));
        assert_eq!(parse_state(&save).unwrap(), state);
        let mut old: Value = serde_json::from_str(&save).unwrap();
        old["save_version"] = Value::from(49);
        assert!(
            parse_state(&old.to_string())
                .unwrap_err()
                .contains("incompatible rules version: saved Some(49), current 50")
        );
    }

    #[test]
    fn different_rules_version_is_refused_before_decoding() {
        let state = legatum_core::new_game(&GameSetup::all_ai(3));
        let mut save: Value = serde_json::from_str(&state_to_json(&state)).unwrap();
        save["save_version"] = Value::from(legatum_core::RULES_VERSION - 1);
        save["players"] = Value::Null;
        assert!(
            parse_state(&save.to_string())
                .unwrap_err()
                .contains("incompatible rules version")
        );
        save.as_object_mut().unwrap().remove("save_version");
        assert!(
            parse_state(&save.to_string())
                .unwrap_err()
                .contains("incompatible rules version")
        );
    }
}

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
    // Tooling reads of an older state (upgrades, rehashes) convert the retired Castle and Star Fort.
    if !check_version && version.is_none_or(|n| n < u64::from(legatum_core::RULES_VERSION)) {
        migrate_legacy_walls(&mut v);
    }
    if let Some(setup) = v.get_mut("setup") {
        seed_from_string(setup, "seed")?;
    }
    let state: GameState =
        serde_json::from_value(v).map_err(|e| format!("invalid saved game: {e}"))?;
    legatum_core::validation::check_state(&state)?;
    Ok(state)
}

/// Earlier rules stored Castles and Star Forts as buildings. Rules 58 stores a city's
/// completed Walls level and treats either old build order as Upgrade Walls.
fn migrate_legacy_walls(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                migrate_legacy_walls(item);
            }
        }
        Value::Object(object) => {
            let is_city = object.contains_key("population");
            let has_walls_level = object.contains_key("walls_level");
            if let Some(Value::Array(buildings)) = object.get_mut("buildings") {
                let old_level = if buildings.iter().any(|b| b == "StarFort") {
                    3
                } else if buildings.iter().any(|b| b == "Castle" || b == "Citadel") {
                    2
                } else if buildings.iter().any(|b| b == "Walls") {
                    1
                } else {
                    0
                };
                if old_level > 0 && is_city && !has_walls_level {
                    buildings.retain(|b| b != "Castle" && b != "Citadel" && b != "StarFort");
                    if !buildings.iter().any(|b| b == "Walls") {
                        buildings.push(Value::from("Walls"));
                    }
                    object.insert("walls_level".into(), Value::from(old_level));
                }
            }
            if object.contains_key("walls") && object.contains_key("citadel") {
                let level = if object.get("star_fort") == Some(&Value::Bool(true)) {
                    3
                } else if object.get("citadel") == Some(&Value::Bool(true)) {
                    2
                } else if object.get("walls") == Some(&Value::Bool(true)) {
                    1
                } else {
                    0
                };
                object.insert("walls_level".into(), Value::from(level));
            }
            if let Some(Value::String(kind)) = object.get_mut("building")
                && (kind == "Castle" || kind == "Citadel" || kind == "StarFort")
            {
                *kind = "UpgradeWalls".into();
            }
            for nested in object.values_mut() {
                migrate_legacy_walls(nested);
            }
        }
        _ => {}
    }
}

pub const UPGRADABLE_RULES_VERSION: u32 = 49;

/// Read a rules-49 to rules-67 saved state as a current (rules-68) one, for analysis that starts from
/// recorded positions which cannot be regenerated (hill-climb do-no-harm states). `json` is a save
/// (`save_version` 49 to 67) or a bare state line with no `save_version` (`audit_states` output; a
/// line carrying `capitals_taken` is read as rules 49, any other as rules 50 or later). See
/// `upgrade_value_from_49` and `upgrade_value_from_50` for what changes.
pub fn upgrade_from_49(json: &str) -> Result<GameState, String> {
    check_json_size(json)?;
    let v: Value = serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    match v.get("save_version").and_then(Value::as_u64) {
        None if carries_rules_49_fields(&v) => upgrade_value_from_49(v),
        None | Some(50) | Some(51) | Some(52) | Some(53) | Some(54) | Some(55) | Some(56)
        | Some(57) | Some(58) | Some(59) | Some(60) | Some(61) | Some(62) | Some(63) | Some(64)
        | Some(65) | Some(66) | Some(67) => upgrade_value_from_50(v),
        Some(49) => upgrade_value_from_49(v),
        Some(n) if n == u64::from(legatum_core::RULES_VERSION) => parse_state(json),
        Some(n) => Err(format!(
            "upgrade reads rules {UPGRADABLE_RULES_VERSION} to {}, got {n}",
            legatum_core::RULES_VERSION
        )),
    }
}

/// Whether a bare recorded state carries a field only a rules-49 engine wrote
/// (`Player::capitals_taken`).
pub fn carries_rules_49_fields(v: &Value) -> bool {
    v.get("players")
        .and_then(Value::as_array)
        .is_some_and(|ps| ps.iter().any(|p| p.get("capitals_taken").is_some()))
}

/// The rules-50 to rules-62 upgrade of a parsed state to rules 63: migrate old Walls buildings and orders to Walls levels (rules 58) (the rules-54
/// `Player` fields for the Colonia Victrix and the royal tomb take their defaults: no Colonia, no
/// work, no tomb; rules 56 only changes the order of city names; rules 57 shuts the Mongols' Walls
/// line for good, and Walls a Mongol city already has stay standing in an upgraded state; rules 59
/// turns Egypt's Archers into Chariot Archers and its Medjay into plain Spearmen, by rule, with no
/// field; rules 60 adds the optional research goal, rules 61 only adds the deals fields, which
/// take their defaults: deals off, no Open Borders, and rules 62 adds `City::walls_damage`,
/// absent meaning a full Walls bar; rules 63 adds city upkeep, a rule with no field; rules 65 changes India upkeep and Gold with no field, rules 66 adds `Player::city_offer`,
/// absent meaning no offer running). Rules 51
/// changes how peaceful units contest a tile, rules 52 renames identifiers (the old names still
/// parse through serde aliases) and rules 53 changes who counts for Humble the mighty
/// (`objectives::humbles`, judged each turn from the turn-start VP); rules 55 adds city
/// automation focuses and the Food project (the old `Balanced` focus parses as `Economy`). None
/// changes a field before the Walls migration, and a capture counted under the older rules stays counted.
pub fn upgrade_value_from_50(v: Value) -> Result<GameState, String> {
    state_from_value(v, false)
}

/// The rules-49 to rules-60 upgrade of a parsed state (54 only adds defaulted fields):
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
    migrate_legacy_walls(&mut v);
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
    // Other later rule changes do not alter these scored records.
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
    fn old_states_upgrade_castle_and_star_fort_to_walls_levels_for_tooling() {
        let state = legatum_core::new_game(&GameSetup::all_ai(3));
        for version in 49..=57 {
            let mut save = state_to_value(&state);
            save["save_version"] = Value::from(version);
            let cities = save["cities"].as_object_mut().unwrap();
            let ids: Vec<_> = cities.keys().take(2).cloned().collect();
            for (i, id) in ids.iter().enumerate() {
                let city = cities.get_mut(id).unwrap();
                let castle_name = if version < 52 { "Citadel" } else { "Castle" };
                city["buildings"] = serde_json::json!(if i == 0 {
                    vec!["Walls", castle_name]
                } else {
                    vec!["Walls", castle_name, "StarFort"]
                });
                city.as_object_mut().unwrap().remove("walls_level");
            }
            // The app refuses the old save; analysis tooling upgrades it.
            assert!(parse_state(&save.to_string()).is_err(), "rules {version}");
            let loaded = upgrade_from_49(&save.to_string()).unwrap();
            for (i, id) in ids.iter().enumerate() {
                let city = &loaded.cities[&legatum_core::CityId(id.parse().unwrap())];
                assert_eq!(
                    city.walls_level,
                    if i == 0 { 2 } else { 3 },
                    "rules {version}"
                );
                assert!(!city.buildings.contains(&legatum_core::BuildingKind::Castle));
                assert!(
                    !city
                        .buildings
                        .contains(&legatum_core::BuildingKind::StarFort)
                );
            }
        }
    }

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
    fn rules_68_saves_round_trip_and_a_version_67_save_is_refused() {
        // Rules 47: held shots at fallen cities; 48: Egypt's Nile and its Grow project. The save
        // shape only gains the project value, so a city growing on the Nile round-trips at 48,
        // and a 47 save is refused rather than migrated. Rules 49: Egypt starts knowing the Bronze
        // Working it skips; the save shape is unchanged, and a 48 save is refused. Rules 50: a
        // rival capital's VP is held, so `Player::capitals_taken` is gone, and a 49 save is refused.
        // Rules 51: peaceful contests, a resolution change only. Rules 52: identifiers renamed.
        // Rules 53: who counts for Humble the mighty; the save shape is unchanged. Rules 54: Rome's
        // Colonia Victrix and Egypt's royal tomb add `Player` fields. Rules 55: city automation
        // focuses and the Food project; the save shape only gains values, the app refuses a 54 save.
        // Rules 56: city names in list order; the save shape is unchanged, and the app refuses a 55 save.
        // Rules 57: the Mongols never have Walls; the save shape is unchanged, and a 56 save is
        // refused. Rules 58: Walls levels replace Castle and Star Fort. Rules 59: Egypt's Chariot
        // Archer replaces its Archer; the save shape is unchanged, and a 58 save is refused. Rules 60:
        // the research goal adds an optional `Player` field; a 59 save is refused and upgrades.
        // Rules 61: deals, behind an optional switch; the app refuses a 60 save. Rules 62: breakable
        // Walls add the optional `City::walls_damage`; a 61 save is refused and upgrades. Rules 63:
        // city upkeep; the save shape is unchanged, and a 62 save is refused. Rules 64: an upgrade
        // stops at the most advanced form its supply allows; the save shape is unchanged. Rules 65:
        // Revenue Offices waive India's city upkeep and remit full Gold; save shape unchanged.
        // Rules 66: Byzantium's Buy a city adds the optional `Player::city_offer`.
        // Rules 67: research queues may span eras; the research gate and save shape are unchanged.
        // Rules 68: pillaging (`Tile::pillaged`, `Order::Pillage`, `Event::TilePillaged`) and
        // enemy-occupied tile yield & strategic-resource denial; optional `Tile::pillaged` field.
        assert_eq!(legatum_core::RULES_VERSION, 68);
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
        assert!(save.contains("\"save_version\":68"));
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
        old["save_version"] = Value::from(67);
        assert!(
            parse_state(&old.to_string())
                .unwrap_err()
                .contains(&format!(
                    "incompatible rules version: saved Some(67), current {}",
                    legatum_core::RULES_VERSION
                ))
        );
        assert_eq!(upgrade_from_49(&old.to_string()).unwrap(), state);
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

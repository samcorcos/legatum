//! Scenario files (repo `scenarios/<id>.json`): a briefing, an objective, predictions and a
//! compressed start state. Shared by the app (through WASM), the simulator baseline and the
//! scenario builder, so all three load, prepare and score a scenario identically.
//!
//! Format, version 1:
//! `{"format":"legatum-scenario","version":1,"id":..,"revision":..,"title":..,"faction":..,
//!   "seat":<player id>,"difficulty":"Hard","minutes":12,"briefing":..,"objective":{..},
//!   "predictions":{..},"source":..,"state":{"encoding":"gzip-base64","hash":"<decimal>","data":..}}`
//! The state is save JSON (`json::state_to_json`), gzip-compressed and base64-encoded; `hash`
//! is the decoded state's hash, checked on load.

use crate::json;
use crate::rusher::Rusher;
use base64::Engine;
use legatum_core::scenario::{self, Objective, Status};
use legatum_core::{AiDifficulty, Faction, GameState, PlayerId};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub const FORMAT: &str = "legatum-scenario";
pub const VERSION: u32 = 1;
pub const RUN_FORMAT: &str = "legatum-scenario-run";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorePrediction {
    pub median: i32,
    pub low: i32,
    pub high: i32,
    /// Predicted share of AI runs that meet the objective, in percent.
    pub win_percent: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Predictions {
    /// What good play looks like, written before the human plays.
    pub good_play: String,
    /// What the Hard AI is expected to do from this position.
    pub ai_play: String,
    pub ai_score: ScorePrediction,
}

/// The measured AI baseline (`legatum-sim scenario --record`), shown on the result screen.
/// Cleared whenever the scenario's state or objective changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    pub runs: u32,
    pub difficulty: AiDifficulty,
    pub median: i32,
    pub p10: i32,
    pub p90: i32,
    /// Share of runs that met the objective, in percent; None for score-only scenarios.
    pub win_percent: Option<u32>,
    /// Median turns the runs that met the objective took, from the first turn of the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_turns: Option<u32>,
    pub measured_at: String,
}

/// The Scenario Lab card (title screen, Scenarios): what the scenario asks a human, in words
/// a player reads. A scenario file with a card is listed in the app; one without is for the
/// simulator only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabCard {
    /// The question the scenario answers, in plain words ("Can you hold two walled cities...?").
    pub question: String,
    /// Two plain lines setting the scene, shown on the card.
    pub blurb: String,
    /// What the human is asked to comment on in the feedback box when it differs from the
    /// question (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask: Option<String>,
    /// The seat's era when the scenario starts (0 is Ancient), for the card's painting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub era: Option<u8>,
    /// A change the scenario waits for ("walls halve siege damage"). While it is set the scenario is
    /// built and measured but not listed in the app; delete the field when the change lands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<String>,
    /// Who the opponent is, when it is not the normal Hard AI ("Sparta rushes on a script"). Shown
    /// on the briefing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opponent: Option<String>,
    /// Position in the list; lower first (cards without one follow by title).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
}

/// What the human player achieved from this start state in the recorded game. War scenarios
/// (`scenarios/war/`) are cut from playtest games, and the AI's runs are measured against
/// this (`legatum-sim war-suite`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanResult {
    /// The human player met the objective.
    pub met: bool,
    /// Turns from the scenario's start until the objective was met (the whole window when not).
    pub turns: u32,
    /// The human player's combat units (Scouts excluded) destroyed in that time.
    pub units_lost: u32,
    /// The human player's starting cities held by someone else at that point.
    pub cities_lost: u32,
    /// What happened, in a sentence or two.
    pub summary: String,
    /// The playtest and the engine commit it ran on.
    pub game: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedState {
    /// "gzip-base64" (save JSON, gzip, base64) or "json" (save JSON as a string).
    pub encoding: String,
    /// Decimal hash of the decoded state.
    pub hash: String,
    pub data: String,
}

/// Each AI seat's saved memory (`legatum_ai::OpponentMemory`: assessments, campaign and plan state,
/// threat log), as `{"<player id>": memory, ..}` JSON, gzip-compressed and base64-encoded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedMemory {
    /// "gzip-base64".
    pub encoding: String,
    pub data: String,
}

pub type AiMemories = std::collections::BTreeMap<PlayerId, legatum_ai::OpponentMemory>;

pub fn encode_memories(memories: &AiMemories) -> EncodedMemory {
    let json = serde_json::to_string(memories).expect("AI memory JSON");
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    gz.write_all(json.as_bytes()).expect("in-memory gzip");
    let bytes = gz.finish().expect("in-memory gzip");
    EncodedMemory {
        encoding: "gzip-base64".into(),
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
    }
}

pub fn decode_memories(encoded: &EncodedMemory) -> Result<AiMemories, String> {
    if encoded.encoding != "gzip-base64" {
        return Err(format!("unknown AI memory encoding {:?}", encoded.encoding));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.data.trim())
        .map_err(|e| format!("scenario AI memory is not base64: {e}"))?;
    let mut json = String::new();
    flate2::read::GzDecoder::new(&bytes[..])
        .read_to_string(&mut json)
        .map_err(|e| format!("scenario AI memory is not gzip JSON: {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("invalid scenario AI memory: {e}"))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioFile {
    pub format: String,
    pub version: u32,
    pub id: String,
    /// Bumped whenever the state or objective changes, so old runs are not compared with new ones.
    pub revision: u32,
    pub title: String,
    pub faction: Faction,
    /// The player the human (or the baseline AI) controls.
    pub seat: PlayerId,
    /// Difficulty of every other empire.
    pub difficulty: AiDifficulty,
    /// Expected minutes for a human to play it.
    pub minutes: u32,
    pub briefing: String,
    /// What the scenario tests, for the comparison report.
    #[serde(default)]
    pub tests: String,
    pub objective: Objective,
    pub predictions: Predictions,
    /// How the state was made (sim seed and turn, or the source game) and what was edited.
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Baseline>,
    /// The human player's own result from this position (war scenarios only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human: Option<HumanResult>,
    /// The Scenario Lab card; absent for scenarios only the simulator runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lab: Option<LabCard>,
    /// An AI empire that rushes on a script instead of the Hard AI's own play.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rusher: Option<Rusher>,
    /// What each AI seat remembered at the start (war scenarios cut from a playtest: the brains'
    /// campaign and plan state). Absent: every AI starts with fresh memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_memory: Option<EncodedMemory>,
    pub state: EncodedState,
}

pub fn encode_state(state: &GameState) -> EncodedState {
    let json = json::state_to_json(state);
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    gz.write_all(json.as_bytes()).expect("in-memory gzip");
    let bytes = gz.finish().expect("in-memory gzip");
    EncodedState {
        encoding: "gzip-base64".into(),
        hash: state.hash().to_string(),
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
    }
}

pub fn decode_state(encoded: &EncodedState) -> Result<GameState, String> {
    let state = decode_state_payload(encoded, false)?;
    let hash = state.hash().to_string();
    if hash != encoded.hash {
        return Err(format!(
            "scenario state hash {hash} does not match the recorded {}",
            encoded.hash
        ));
    }
    Ok(state)
}

/// Decode a stored state without comparing its hash, for tooling that re-encodes scenarios after
/// an intentional state-format or rules change (`rehash_scenarios`).
pub fn decode_state_unchecked(encoded: &EncodedState) -> Result<GameState, String> {
    decode_state_payload(encoded, true)
}

fn state_json(encoded: &EncodedState) -> Result<String, String> {
    match encoded.encoding.as_str() {
        "json" => Ok(encoded.data.clone()),
        "gzip-base64" => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded.data.trim())
                .map_err(|e| format!("scenario state is not base64: {e}"))?;
            let mut out = String::new();
            flate2::read::GzDecoder::new(&bytes[..])
                .read_to_string(&mut out)
                .map_err(|e| format!("scenario state is not gzip JSON: {e}"))?;
            Ok(out)
        }
        other => Err(format!("unknown scenario state encoding {other:?}")),
    }
}

fn decode_state_payload(encoded: &EncodedState, for_rehash: bool) -> Result<GameState, String> {
    let json = state_json(encoded)?;
    if for_rehash {
        json::parse_state_for_rehash(&json)
    } else {
        json::parse_state(&json)
    }
}

impl ScenarioFile {
    pub fn parse(json: &str) -> Result<Self, String> {
        let file: ScenarioFile =
            serde_json::from_str(json).map_err(|e| format!("invalid scenario: {e}"))?;
        if file.format != FORMAT {
            return Err(format!("not a scenario file (format {:?})", file.format));
        }
        if file.version != VERSION {
            return Err(format!("unsupported scenario version {}", file.version));
        }
        Ok(file)
    }

    /// The saved AI memories to restore into the seats' brains (empty when the scenario has none).
    pub fn ai_memories(&self) -> Result<AiMemories, String> {
        self.ai_memory
            .as_ref()
            .map_or_else(|| Ok(AiMemories::new()), decode_memories)
    }

    /// The start state as the scenario stores it, before seats are assigned.
    pub fn stored_state(&self) -> Result<GameState, String> {
        decode_state(&self.state)
    }

    /// The playable start state: `seat` is the only human (or nobody, for an all-AI baseline),
    /// every other empire plays at the scenario difficulty, and the position and objective are
    /// checked against the rules.
    pub fn prepare(&self, human: bool) -> Result<GameState, String> {
        let mut state = self.stored_state()?;
        if (self.seat.0 as usize) >= state.players.len() {
            return Err(format!("seat {} is not a player", self.seat.0));
        }
        if state.player(self.seat).faction != self.faction {
            return Err(format!(
                "seat {} plays {:?}, not {:?}",
                self.seat.0,
                state.player(self.seat).faction,
                self.faction
            ));
        }
        if let Some(r) = &self.rusher
            && ((r.player.0 as usize) >= state.players.len() || r.player == self.seat)
        {
            return Err(format!(
                "the rusher {} is not an AI empire of this scenario",
                r.player.0
            ));
        }
        for (i, setup) in state.setup.players.iter_mut().enumerate() {
            let me = PlayerId(i as u8) == self.seat;
            setup.is_human = me && human;
            if !me {
                setup.difficulty = self.difficulty;
            }
        }
        for p in &mut state.players {
            p.is_human = p.id == self.seat && human;
        }
        let mut errors = scenario::validate_state(&state);
        errors.extend(scenario::check_objective(
            &self.objective,
            self.seat,
            &state,
        ));
        if state.finished {
            errors.push("the game in this scenario is already over".into());
        }
        if !errors.is_empty() {
            return Err(format!(
                "scenario {} is not legal: {}",
                self.id,
                errors.join("; ")
            ));
        }
        Ok(state)
    }
}

/// A scenario file that passed `check_file`.
#[derive(Debug)]
pub struct Checked {
    pub id: String,
    pub title: String,
}

/// Everything a scenario file must satisfy before the Lab can offer it, in plain words when it
/// does not: the format, the rules version the state was made for, and that
/// the state loads (hash), is legal and has an objective that parses. The `Err` is a plain
/// reason, never a panic. Used by `legatum-sim scenario-check`.
pub fn check_file(text: &str) -> Result<Checked, String> {
    let file = ScenarioFile::parse(text)?;
    if file.lab.is_none() {
        return Err("has no Lab card (question and blurb), so the Lab cannot list it".into());
    }
    let json = state_json(&file.state)?;
    let version = serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|e| format!("scenario state is not valid JSON: {e}"))?
        .get("save_version")
        .and_then(serde_json::Value::as_u64);
    let current = u64::from(legatum_core::RULES_VERSION);
    if version != Some(current) {
        return Err(match version {
            Some(v) => format!("made for rules version {v}; this app is {current}"),
            None => format!("has no rules version; this app is {current}"),
        });
    }
    file.prepare(true)?;
    Ok(Checked {
        id: file.id.clone(),
        title: file.title.clone(),
    })
}

/// A scenario in play: its objective, its seat and the state it started from.
pub struct ActiveScenario {
    pub file: ScenarioFile,
    pub start: GameState,
}

impl ActiveScenario {
    pub fn new(file: ScenarioFile, start: GameState) -> Self {
        Self { file, start }
    }

    pub fn status(&self, now: &GameState) -> Status {
        scenario::evaluate(&self.file.objective, self.file.seat, &self.start, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::scenario::{Condition, Metric, Outcome, ScoreTerm};
    use legatum_core::{GameSetup, new_game};

    #[test]
    fn ai_memories_round_trip_and_are_optional() {
        let mut memories = AiMemories::new();
        memories.insert(PlayerId(2), legatum_ai::OpponentMemory::default());
        assert_eq!(
            decode_memories(&encode_memories(&memories)).unwrap(),
            memories
        );
        let state = new_game(&GameSetup::all_ai(5));
        let mut file = sample(&state);
        assert!(file.ai_memories().unwrap().is_empty());
        assert!(!serde_json::to_string(&file).unwrap().contains("ai_memory"));
        file.ai_memory = Some(encode_memories(&memories));
        let parsed = ScenarioFile::parse(&serde_json::to_string(&file).unwrap()).unwrap();
        assert_eq!(parsed.ai_memories().unwrap(), memories);
    }

    pub fn sample(state: &GameState) -> ScenarioFile {
        ScenarioFile {
            format: FORMAT.into(),
            version: VERSION,
            id: "sample".into(),
            revision: 1,
            title: "Sample".into(),
            faction: state.players[2].faction,
            seat: PlayerId(2),
            difficulty: AiDifficulty::Hard,
            minutes: 5,
            briefing: "Found a second city.".into(),
            tests: String::new(),
            objective: Objective {
                summary: "Found a second city by turn 6".into(),
                turns: 5,
                win: vec![Condition::MinCities(2)],
                end_on_win: true,
                keep: vec![],
                score: vec![ScoreTerm {
                    metric: Metric::Cities,
                    weight: 100,
                }],
            },
            predictions: Predictions {
                good_play: "Train a Settler and expand.".into(),
                ai_play: "Trains a Settler and expands.".into(),
                ai_score: ScorePrediction {
                    median: 100,
                    low: 100,
                    high: 100,
                    win_percent: 100,
                },
            },
            source: "new_game(all_ai(5))".into(),
            baseline: None,
            human: None,
            lab: None,
            rusher: None,
            ai_memory: None,
            state: encode_state(state),
        }
    }

    fn carded(state: &GameState) -> ScenarioFile {
        let mut file = sample(state);
        file.lab = Some(LabCard {
            question: "Can you found a city?".into(),
            blurb: "Two lines.".into(),
            ask: None,
            era: None,
            requires: None,
            opponent: None,
            order: None,
        });
        file
    }

    #[test]
    fn check_file_names_why_a_dropped_in_file_cannot_be_used() {
        let state = new_game(&GameSetup::all_ai(5));
        let good = carded(&state);
        let checked = check_file(&serde_json::to_string(&good).unwrap()).unwrap();
        assert_eq!(
            (checked.id.as_str(), checked.title.as_str()),
            ("sample", "Sample")
        );

        // Made for another rules version: the reason says which.
        let mut old = good.clone();
        let mut json: serde_json::Value =
            serde_json::from_str(&state_json(&old.state).unwrap()).unwrap();
        json["save_version"] = serde_json::Value::from(legatum_core::RULES_VERSION + 1);
        let text = serde_json::to_string(&json).unwrap();
        old.state = EncodedState {
            encoding: "json".into(),
            hash: old.state.hash.clone(),
            data: text,
        };
        let reason = check_file(&serde_json::to_string(&old).unwrap()).unwrap_err();
        assert_eq!(
            reason,
            format!(
                "made for rules version {}; this app is {}",
                legatum_core::RULES_VERSION + 1,
                legatum_core::RULES_VERSION
            )
        );

        // A tampered state, a missing card and a wrong format are refused, never panics.
        let mut tampered = good.clone();
        tampered.state.hash = "1".into();
        assert!(
            check_file(&serde_json::to_string(&tampered).unwrap())
                .unwrap_err()
                .contains("hash")
        );
        assert!(
            check_file(&serde_json::to_string(&sample(&state)).unwrap())
                .unwrap_err()
                .contains("Lab card")
        );
        let mut other: serde_json::Value = serde_json::to_value(&good).unwrap();
        other["format"] = "legatum-scenario-run".into();
        assert!(
            check_file(&other.to_string())
                .unwrap_err()
                .contains("not a scenario file")
        );
        assert!(check_file("{").unwrap_err().contains("invalid scenario"));
    }

    #[test]
    fn a_lab_card_and_a_baseline_with_turns_round_trip() {
        let state = new_game(&GameSetup::all_ai(5));
        let mut file = sample(&state);
        file.lab = Some(LabCard {
            question: "Can you hold?".into(),
            blurb: "Two lines.".into(),
            ask: None,
            era: Some(2),
            requires: None,
            opponent: Some("Scripted.".into()),
            order: Some(3),
        });
        file.baseline = Some(Baseline {
            runs: 8,
            difficulty: AiDifficulty::Hard,
            median: 100,
            p10: 90,
            p90: 110,
            win_percent: Some(75),
            median_turns: Some(4),
            measured_at: "t".into(),
        });
        let json = serde_json::to_string(&file).unwrap();
        assert!(!json.contains("\"ask\""), "an absent ask is not written");
        assert_eq!(ScenarioFile::parse(&json).unwrap(), file);
        // A file without a card or a turns median (every older scenario) still loads.
        let mut plain = serde_json::to_value(sample(&state)).unwrap();
        plain.as_object_mut().unwrap().remove("lab");
        assert!(
            ScenarioFile::parse(&plain.to_string())
                .unwrap()
                .lab
                .is_none()
        );
    }

    #[test]
    fn states_round_trip_compressed_and_hash_checked() {
        let state = new_game(&GameSetup::all_ai(5));
        let encoded = encode_state(&state);
        assert_eq!(encoded.encoding, "gzip-base64");
        assert!(encoded.data.len() < json::state_to_json(&state).len() / 3);
        assert_eq!(decode_state(&encoded).unwrap(), state);
        let mut tampered = encoded.clone();
        tampered.hash = "1".into();
        assert!(decode_state(&tampered).unwrap_err().contains("hash"));
        let plain = EncodedState {
            encoding: "json".into(),
            hash: state.hash().to_string(),
            data: json::state_to_json(&state),
        };
        assert_eq!(decode_state(&plain).unwrap(), state);
    }

    #[test]
    fn prepare_makes_the_seat_the_only_human_and_sets_difficulty() {
        let mut setup = GameSetup::all_ai(5);
        setup.players[0].is_human = true;
        let state = new_game(&setup);
        let file = sample(&state);
        let json = serde_json::to_string(&file).unwrap();
        let file = ScenarioFile::parse(&json).unwrap();
        let prepared = file.prepare(true).unwrap();
        for p in &prepared.players {
            assert_eq!(p.is_human, p.id == PlayerId(2));
            assert_eq!(prepared.setup.players[p.id.0 as usize].is_human, p.is_human);
            if p.id != PlayerId(2) {
                assert_eq!(
                    prepared.setup.players[p.id.0 as usize].difficulty,
                    AiDifficulty::Hard
                );
            }
        }
        let baseline = file.prepare(false).unwrap();
        assert!(baseline.players.iter().all(|p| !p.is_human));
        let active = ActiveScenario::new(file, prepared.clone());
        let status = active.status(&prepared);
        assert_eq!((status.outcome, status.turns_left), (Outcome::Playing, 5));
    }

    #[test]
    fn bad_files_are_rejected_with_a_reason() {
        let state = new_game(&GameSetup::all_ai(5));
        let mut file = sample(&state);
        file.faction = if file.faction == Faction::India {
            Faction::Celts
        } else {
            Faction::India
        };
        assert!(file.prepare(true).unwrap_err().contains("plays"));
        let mut file = sample(&state);
        file.objective.win = vec![Condition::OwnCity(legatum_core::CityId(40))];
        assert!(file.prepare(true).unwrap_err().contains("not legal"));
        let mut v = serde_json::to_value(sample(&state)).unwrap();
        v["version"] = 9.into();
        assert!(
            ScenarioFile::parse(&v.to_string())
                .unwrap_err()
                .contains("version")
        );
        v["version"] = 1.into();
        v["surprise"] = 1.into();
        assert!(ScenarioFile::parse(&v.to_string()).is_err());
    }
}

#[cfg(test)]
mod target_tests {
    use super::*;
    use legatum_core::scenario::{Target, TargetKind, objective_targets};

    fn targets(id: &str) -> (ScenarioFile, Vec<Target>) {
        let path = format!("{}/../../scenarios/{id}.json", env!("CARGO_MANIFEST_DIR"));
        let file = ScenarioFile::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let start = file.prepare(true).unwrap();
        let t = objective_targets(&file.objective, file.seat, &start, &start);
        (file, t)
    }

    #[test]
    fn every_shipped_scenario_marks_where_its_objective_is() {
        // Glass at the Gate: hold both India cities.
        let (_, t) = targets("rush-defense");
        assert_eq!(t.len(), 2, "{t:?}");
        assert!(
            t.iter()
                .all(|t| t.kind == TargetKind::Hold && t.bearing.is_empty())
        );
        // The Last Bridge: take Last Bridge, north of the strike force's home.
        let (_, t) = targets("rush-offense");
        let take: Vec<_> = t.iter().filter(|t| t.kind == TargetKind::Take).collect();
        assert_eq!(take.len(), 1, "{t:?}");
        assert_eq!(take[0].name, "Last Bridge");
        assert!(take[0].bearing.contains("north"), "{}", take[0].bearing);
        // The Courier King: contain Mongols (its cities are in fog for the Switzerland).
        let (file, t) = targets("runaway-conqueror");
        let contain: Vec<_> = t.iter().filter(|t| t.kind == TargetKind::Contain).collect();
        assert!(contain.len() >= 3, "{t:?}");
        assert!(contain.iter().any(|t| t.name == "Resonator"), "{t:?}");
        assert!(
            contain
                .iter()
                .all(|t| t.distance > 0 && !t.bearing.is_empty())
        );
        assert!(
            t.iter()
                .any(|t| t.kind == TargetKind::Hold && t.owner == file.seat)
        );
        // Three Fires by Fifty points at no city.
        let (_, t) = targets("opening-race");
        assert!(t.is_empty(), "{t:?}");
    }

    #[test]
    fn briefing_targets_come_from_the_start_state() {
        let path = format!(
            "{}/../../scenarios/rush-offense.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let json = std::fs::read_to_string(path).unwrap();
        let out: Vec<Target> =
            serde_json::from_str(&crate::scenario_targets_json(&json).unwrap()).unwrap();
        assert!(out.iter().any(|t| t.name == "Last Bridge"));
    }
}

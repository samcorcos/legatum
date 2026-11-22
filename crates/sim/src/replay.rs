//! Compact, omniscient dashboard replays. Frames are post-resolution snapshots, labelled
//! with the turn just resolved (not the core's next-turn counter). All maps are ordered.

use crate::stats::{FinalFacts, GameRecord, Outcome, classify};
use legatum_core::events::Defender;
use legatum_core::{
    CityId, Event, Faction, GameState, Hex, PlayerId, ProductionItem, TeamId, Terrain, UnitId,
    UnitKind, VpSource, data, resolve_turn, scoring,
};
use legatum_wasm::tape::Tape;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct Replay {
    pub version: u32,
    pub header: Header,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Serialize)]
pub struct Header {
    pub label: String,
    /// Tournament game identity; mirrored games intentionally share a seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_index: Option<usize>,
    /// Decimal string: do not round a u64 in JavaScript.
    pub seed: String,
    pub width: i32,
    pub height: i32,
    /// Row-major odd-r tiles; each value indexes terrain_kinds.
    pub terrain: Vec<u8>,
    pub terrain_kinds: Vec<Terrain>,
    pub players: Vec<Player>,
    /// Winning TEAM id, or null. Players and targets carry team ids explicitly.
    pub winner: Option<u8>,
    pub outcome: Outcome,
    pub turns: usize,
    pub targets: BTreeMap<u8, u32>,
}

#[derive(Debug, Serialize)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub faction: Faction,
    pub team: u8,
    pub color_index: usize,
}

#[derive(Debug, Serialize)]
pub struct Frame {
    pub turn: u32,
    /// [owner, run length], in row-major odd-r order. -1 means unowned.
    pub owners: Vec<(i16, u32)>,
    pub cities: Vec<City>,
    pub units: Vec<Army>,
    pub vp: Vec<Score>,
    pub events: Vec<NotableEvent>,
}

#[derive(Debug, Serialize)]
pub struct City {
    pub id: u32,
    pub owner: u8,
    pub name: String,
    /// Axial [q, r]; col = q + (r - (r & 1)) / 2.
    pub pos: [i32; 2],
    pub pop: i32,
    pub capital: bool,
}

#[derive(Debug, Serialize)]
pub struct Army {
    pub pos: [i32; 2],
    pub owner: u8,
    pub kinds: BTreeMap<UnitKind, u32>,
    /// Health-weighted military strength, identical to the Strongest Army title metric.
    pub strength: i32,
}

#[derive(Debug, Serialize)]
pub struct Score {
    pub player: u8,
    pub total: u32,
    pub sources: BTreeMap<String, u32>,
    pub alive: bool,
}

#[derive(Debug, Serialize)]
pub struct NotableEvent {
    pub turn: u32,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub file: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_index: Option<usize>,
    pub seed: String,
    pub turns: usize,
    pub winner: Option<u8>,
    pub outcome: Outcome,
    pub why_interesting: String,
}

pub fn from_tape(tape: &Tape, label: &str) -> Result<Replay, String> {
    let mut state = tape.initial_state();
    if state.hash() != tape.initial_hash {
        return Err("initial state hash differs from tape".into());
    }
    let terrain = state
        .map
        .tiles
        .iter()
        .map(|t| {
            Terrain::ALL
                .iter()
                .position(|kind| *kind == t.terrain)
                .unwrap() as u8
        })
        .collect();
    let mut frames = Vec::with_capacity(tape.turns.len());
    for t in &tape.turns {
        if state.finished || t.turn != state.turn {
            return Err(format!(
                "invalid tape turn {} (expected {}, finished: {})",
                t.turn, state.turn, state.finished
            ));
        }
        let (next, events) = resolve_turn(&state, &t.orders);
        if next.hash().to_string() != t.hash {
            return Err(format!("state hash differs after turn {}", t.turn));
        }
        frames.push(snapshot(&next, t.turn, &events));
        state = next;
    }
    let alive_teams = state
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.team)
        .collect::<BTreeSet<_>>()
        .len();
    let outcome = classify(FinalFacts {
        finished: state.finished,
        winner: state.winner.map(|t| t.0),
        alive_teams,
        winner_hold_complete: crate::stats::winner_hold_complete(&state),
        winner_holds_last_capital: crate::stats::winner_holds_last_capital(&state),
    });
    Ok(Replay {
        version: 1,
        header: Header {
            label: label.into(),
            game_index: None,
            seed: tape.setup.seed.to_string(),
            width: state.map.width,
            height: state.map.height,
            terrain,
            terrain_kinds: Terrain::ALL.to_vec(),
            players: state
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| Player {
                    id: p.id.0,
                    name: p.name.clone(),
                    faction: p.faction,
                    team: p.team.0,
                    color_index: i % 8,
                })
                .collect(),
            winner: state.winner.map(|t| t.0),
            outcome,
            turns: frames.len(),
            targets: state
                .teams()
                .into_iter()
                .map(|t| (t.0, scoring::team_target(&state, t)))
                .collect(),
        },
        frames,
    })
}

fn snapshot(state: &GameState, turn: u32, events: &[Event]) -> Frame {
    let mut owners: Vec<(i16, u32)> = Vec::new();
    for tile in &state.map.tiles {
        let owner = tile.owner.map_or(-1, |p| i16::from(p.0));
        match owners.last_mut() {
            Some((last, count)) if *last == owner => *count += 1,
            _ => owners.push((owner, 1)),
        }
    }
    let mut armies: BTreeMap<(Hex, PlayerId), Army> = BTreeMap::new();
    for u in state.units.values() {
        let army = armies.entry((u.pos, u.owner)).or_insert_with(|| Army {
            pos: [u.pos.q, u.pos.r],
            owner: u.owner.0,
            kinds: BTreeMap::new(),
            strength: 0,
        });
        *army.kinds.entry(u.kind).or_default() += 1;
        army.strength += legatum_core::army::unit_value(state, u);
    }
    Frame {
        turn,
        owners,
        cities: state
            .cities
            .values()
            .map(|c| City {
                id: c.id.0,
                owner: c.owner.0,
                name: c.name.clone(),
                pos: [c.pos.q, c.pos.r],
                pop: c.population,
                capital: c.is_capital,
            })
            .collect(),
        units: armies.into_values().collect(),
        vp: state
            .players
            .iter()
            .map(|p| {
                let sources: BTreeMap<_, _> = scoring::vp(state, p.id)
                    .into_iter()
                    .map(|(s, v)| (vp_source_label(&s), v))
                    .collect();
                Score {
                    player: p.id.0,
                    total: sources.values().sum(),
                    sources,
                    alive: p.alive,
                }
            })
            .collect(),
        events: notable_events(state, turn, events),
    }
}

fn vp_source_label(source: &VpSource) -> String {
    match source {
        VpSource::EraObjective { era } => format!("Era {era} objective"),
        VpSource::FactionGoal { .. } => "Faction goal".to_string(),
        VpSource::GreatWork { .. } => "Great Work".to_string(),
        VpSource::CapitalTaken { .. } => "Capital held".to_string(),
        VpSource::CapitalSacked { .. } => "Capital sacked".to_string(),
    }
}

#[derive(Default)]
struct BattleSummary {
    players: BTreeSet<PlayerId>,
    clashes: usize,
    dead_units: BTreeSet<UnitId>,
    city: bool,
}

fn notable_events(state: &GameState, turn: u32, events: &[Event]) -> Vec<NotableEvent> {
    let player = |p: PlayerId| state.player(p).name.as_str();
    let city = |id: CityId| {
        state
            .cities
            .get(&id)
            .map_or_else(|| format!("city {}", id.0), |c| c.name.clone())
    };
    let team_name = |t: TeamId| {
        state
            .team_members(t)
            .into_iter()
            .map(player)
            .collect::<Vec<_>>()
            .join(" & ")
    };
    let mut out = Vec::new();
    let mut battles: BTreeMap<Hex, BattleSummary> = BTreeMap::new();
    for e in events {
        let text = match e {
            Event::CityFounded { owner, name, .. } => format!("{} founded {name}", player(*owner)),
            Event::CityCaptured {
                city: id, from, to, ..
            } => format!(
                "{} captured {} from {}",
                player(*to),
                city(*id),
                player(*from)
            ),
            Event::BuildingDismantled {
                city: id,
                building,
                reason,
                ..
            } => format!(
                "The {} in {} was dismantled: {reason}",
                data::building_name(*building),
                city(*id)
            ),
            Event::PlayerEliminated { player: p } => format!("{} was eliminated", player(*p)),
            Event::FactionGoalCompleted {
                player: p,
                faction,
                vp,
                again,
            } => format!(
                "{} {} the {faction:?} faction goal (+{vp} VP)",
                player(*p),
                if *again { "regained" } else { "completed" }
            ),
            Event::FactionGoalLost {
                player: p,
                faction,
                vp,
            } => format!(
                "{} lost the {faction:?} faction goal (-{vp} VP)",
                player(*p)
            ),
            Event::MegaprojectStarted {
                city: id,
                owner,
                building,
            } => format!(
                "{} started {} in {}",
                player(*owner),
                data::building_name(*building),
                city(*id)
            ),
            Event::ProductionCompleted {
                city: id,
                owner,
                item: ProductionItem::Building { building },
            } if legatum_core::data::is_great_work(*building) => format!(
                "{} completed {} in {}",
                player(*owner),
                data::building_name(*building),
                city(*id)
            ),
            Event::VictoryHoldStarted {
                team,
                vp,
                target,
                turns_needed,
            } => format!(
                "{} reached its VP target ({vp}/{target}) and must hold it for {turns_needed} turns",
                team_name(*team)
            ),
            Event::VictoryHoldBroken {
                team,
                turns_held,
                vp,
                target,
            } => format!(
                "{} lost its VP hold after {turns_held} turns ({vp}/{target} VP)",
                team_name(*team)
            ),
            Event::GameWon { reason, .. } | Event::GameDrawn { reason } => reason.clone(),
            Event::Battle {
                at,
                attacker,
                attacker_owner,
                defender,
                attacker_hp_after,
                defender_hp_after,
                ..
            } => {
                let b = battles.entry(*at).or_default();
                b.players.insert(*attacker_owner);
                let (owner, is_city) = match defender {
                    Defender::Unit { owner, .. } => (*owner, false),
                    Defender::City { owner, .. } => (*owner, true),
                };
                b.players.insert(owner);
                b.clashes += 1;
                // HP reflects all simultaneous damage, so several events can report
                // the same unit's death, even in both attacker and defender roles.
                if *attacker_hp_after <= 0
                    && let Some(unit) = attacker.unit()
                {
                    b.dead_units.insert(unit);
                }
                if let Defender::Unit { unit, .. } = defender
                    && *defender_hp_after <= 0
                {
                    b.dead_units.insert(*unit);
                }
                b.city |= is_city;
                continue;
            }
            _ => continue,
        };
        out.push(NotableEvent { turn, text });
    }
    // A siege, a lethal engagement, or 3+ exchanges on a tile constitutes a big battle.
    for (at, b) in battles {
        if b.city || !b.dead_units.is_empty() || b.clashes >= 3 {
            let names = b
                .players
                .into_iter()
                .map(player)
                .collect::<Vec<_>>()
                .join(" vs ");
            out.push(NotableEvent {
                turn,
                text: format!(
                    "{} at ({}, {}): {names}; {} clashes, {} units lost",
                    if b.city { "Siege" } else { "Battle" },
                    at.q,
                    at.r,
                    b.clashes,
                    b.dead_units.len()
                ),
            });
        }
    }
    out
}

/// Merge by filename, preserving other runs. Write index last so readers see complete files.
pub fn write_replays(out: &Path, replays: &[(Replay, String)]) -> Result<(), String> {
    let filenames: Vec<_> = replays
        .iter()
        .map(|(r, _)| {
            let h = &r.header;
            let game = h
                .game_index
                .map_or_else(String::new, |i| format!("-game-{i:05}"));
            format!("{}{game}-seed-{}.json", crate::sanitize(&h.label), h.seed)
        })
        .collect();
    if filenames.iter().collect::<BTreeSet<_>>().len() != filenames.len() {
        return Err("duplicate replay identity (label, game index, seed)".into());
    }
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let index_path = out.join("index.json");
    let mut index: Vec<IndexEntry> = if index_path.exists() {
        serde_json::from_str(&fs::read_to_string(&index_path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("invalid existing replay index: {e}"))?
    } else {
        Vec::new()
    };
    for ((replay, why), file) in replays.iter().zip(filenames) {
        let h = &replay.header;
        crate::write_atomic(
            &out.join(&file),
            &serde_json::to_string(replay).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        index.retain(|entry| entry.file != file);
        index.push(IndexEntry {
            file,
            label: h.label.clone(),
            game_index: h.game_index,
            seed: h.seed.clone(),
            turns: h.turns,
            winner: h.winner,
            outcome: h.outcome,
            why_interesting: why.clone(),
        });
    }
    index.sort_by(|a, b| a.file.cmp(&b.file));
    crate::write_atomic(
        &index_path,
        &serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

pub fn export_path(input: &Path, out: &Path, label: &str) -> Result<(), String> {
    let mut paths = if input.is_dir() {
        fs::read_dir(input)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
            .collect()
    } else {
        vec![input.to_path_buf()]
    };
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no .json tapes in {}", input.display()));
    }
    let mut replays = Vec::new();
    let mut identities = BTreeSet::new();
    for path in paths {
        let tape = Tape::from_json(
            &fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )?;
        // Native tournament tapes carry their stable index in the filename. Keep it
        // when importing one tape or a subset: directory enumeration is not identity.
        let game_index = path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("game-"))
            .and_then(|s| s.split_once("-seed-"))
            .and_then(|(i, _)| i.parse::<usize>().ok());
        if !identities.insert((game_index, tape.setup.seed)) {
            return Err(format!(
                "duplicate replay identity (game index {game_index:?}, seed {}): use native game-index filenames or distinct labels",
                tape.setup.seed,
            ));
        }
        let mut replay = from_tape(&tape, label).map_err(|e| format!("{}: {e}", path.display()))?;
        replay.header.game_index = game_index;
        replays.push((
            replay,
            "Imported order tape; all state hashes verified".into(),
        ));
    }
    write_replays(out, &replays)?;
    eprintln!("exported {} replay(s) to {}", replays.len(), out.display());
    Ok(())
}

fn final_margin(record: &GameRecord) -> u32 {
    let mut teams: BTreeMap<u8, u32> = BTreeMap::new();
    let alive: BTreeSet<_> = record
        .players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.team)
        .collect();
    for p in &record.players {
        if alive.contains(&p.team) {
            *teams.entry(p.team).or_default() += p.vp;
        }
    }
    let mut values: Vec<_> = teams.into_values().collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    if values.len() < 2 {
        u32::MAX
    } else {
        values[0] - values[1]
    }
}

/// Deficit of the eventual winner behind a unique turn-75 leader who subsequently lost.
fn comeback(record: &GameRecord) -> Option<u32> {
    if record.turns <= 75 {
        return None;
    }
    let winner = record.winner_team?;
    let best = *record.team_vp_turn_75.values().max()?;
    if record
        .team_vp_turn_75
        .values()
        .filter(|&&v| v == best)
        .count()
        != 1
    {
        return None;
    }
    let deficit = best.checked_sub(*record.team_vp_turn_75.get(&winner)?)?;
    (deficit > 0).then_some(deficit)
}

/// Deterministic, diverse selection: reserve one conquest, then cycle close games, most
/// captures and biggest comebacks, deduplicating. Input/parallel completion order is irrelevant.
pub fn select(records: &[GameRecord], count: usize) -> Vec<(usize, String)> {
    let valid: Vec<_> = records
        .iter()
        .filter(|r| {
            !matches!(r.outcome, Outcome::Panic | Outcome::Stalled)
                && r.determinism.as_ref().is_none_or(|d| d.ok)
        })
        .collect();
    let mut close = valid.clone();
    close.sort_by_key(|r| (final_margin(r), r.index));
    let mut captures: Vec<_> = valid
        .iter()
        .copied()
        .filter(|r| r.city_captures > 0)
        .collect();
    captures.sort_by_key(|r| (std::cmp::Reverse(r.city_captures), r.index));
    let mut comebacks: Vec<_> = valid
        .iter()
        .copied()
        .filter(|r| comeback(r).is_some())
        .collect();
    comebacks.sort_by_key(|r| (std::cmp::Reverse(comeback(r)), r.index));
    let mut selected: Vec<&GameRecord> = Vec::new();
    if count > 0
        && let Some(r) = valid
            .iter()
            .copied()
            .filter(|r| r.outcome == Outcome::Conquest)
            .min_by_key(|r| r.index)
    {
        selected.push(r);
    }
    while selected.len() < count.min(valid.len()) {
        for ranking in [&close, &captures, &comebacks] {
            if selected.len() >= count {
                break;
            }
            if let Some(r) = ranking
                .iter()
                .find(|r| !selected.iter().any(|s| s.index == r.index))
            {
                selected.push(r);
            }
        }
    }
    selected
        .into_iter()
        .map(|r| {
            let mut reasons = Vec::new();
            if r.outcome == Outcome::Conquest {
                reasons.push("Conquest: last team standing".into());
            }
            if r.outcome == Outcome::Capitals {
                reasons.push("Capitals: the only team still holding an original capital".into());
            }
            if final_margin(r) != u32::MAX {
                reasons.push(format!("Final VP margin: {}", final_margin(r)));
            }
            reasons.push(format!("{} city captures", r.city_captures));
            if let Some(deficit) = comeback(r) {
                reasons.push(format!(
                    "Comeback: winner trailed the turn-75 leader by {deficit} VP"
                ));
            }
            (r.index, reasons.join(" · "))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{self, BrainMode};
    use legatum_core::{GameSetup, TeamId};

    #[test]
    fn frames_match_resolved_turns_and_valid_owners_deterministically() {
        let mut setup = GameSetup::all_ai(u64::MAX);
        setup.turn_limit = Some(8);
        let played = runner::play(&setup, BrainMode::Scripted, true, 100);
        let replay = from_tape(&played.tape, "test").unwrap();
        assert_eq!(replay.frames.len(), played.turns as usize);
        assert_eq!(replay.header.turns, replay.frames.len());
        assert_eq!(replay.header.seed, u64::MAX.to_string());
        assert_eq!(
            replay.header.terrain.len(),
            (setup.width * setup.height) as usize
        );
        let valid_owner =
            |owner: i16| owner == -1 || (0..setup.players.len() as i16).contains(&owner);
        for (i, f) in replay.frames.iter().enumerate() {
            assert_eq!(f.turn, i as u32 + 1);
            assert_eq!(
                f.owners.iter().map(|(_, n)| n).sum::<u32>(),
                (setup.width * setup.height) as u32
            );
            assert!(
                f.owners
                    .iter()
                    .all(|(owner, n)| valid_owner(*owner) && *n > 0)
            );
            assert!(f.cities.iter().all(|c| c.owner < setup.players.len() as u8));
            // Barbarian units (the Celts' rite can raise camps even this early) are owned by 255.
            assert!(f.units.iter().all(|u| u.owner < setup.players.len() as u8
                || u.owner == legatum_core::PlayerId::BARBARIAN.0));
            assert!(
                f.vp.iter()
                    .all(|v| v.total == v.sources.values().sum::<u32>())
            );
            assert!(f.events.iter().all(|e| e.turn == f.turn));
        }
        let last = replay.frames.last().unwrap();
        let decoded: Vec<_> = last
            .owners
            .iter()
            .flat_map(|(p, n)| std::iter::repeat_n(*p, *n as usize))
            .collect();
        assert_eq!(
            decoded,
            played
                .state
                .map
                .tiles
                .iter()
                .map(|t| t.owner.map_or(-1, |p| i16::from(p.0)))
                .collect::<Vec<_>>()
        );
        for p in &played.state.players {
            assert_eq!(
                last.units
                    .iter()
                    .filter(|a| a.owner == p.id.0)
                    .map(|a| a.strength)
                    .sum::<i32>(),
                scoring::army_strength(&played.state, p.id)
            );
        }
        let bytes = serde_json::to_vec(&replay).unwrap();
        assert_eq!(
            bytes,
            serde_json::to_vec(&from_tape(&played.tape, "test").unwrap()).unwrap()
        );
        assert!(
            bytes.len() < 160_000,
            "short replay unexpectedly large: {}",
            bytes.len()
        );
    }

    #[test]
    fn rejects_divergent_or_out_of_order_tapes() {
        let mut setup = GameSetup::all_ai(1);
        setup.turn_limit = Some(2);
        let tape = runner::play(&setup, BrainMode::Scripted, true, 100).tape;
        let mut bad = tape.clone();
        bad.initial_hash ^= 1;
        assert!(from_tape(&bad, "bad").unwrap_err().contains("initial"));
        bad = tape.clone();
        bad.turns[0].hash = "0".into();
        assert!(from_tape(&bad, "bad").unwrap_err().contains("hash"));
        bad = tape;
        bad.turns[0].turn = 99;
        assert!(
            from_tape(&bad, "bad")
                .unwrap_err()
                .contains("invalid tape turn")
        );
    }

    fn candidate(index: usize, margin: u32, captures: u32, deficit: u32) -> GameRecord {
        let mut r = GameRecord::panicked(index, index as u64, String::new(), 0);
        r.outcome = Outcome::Vp;
        r.panic = None;
        r.turns = 150;
        r.winner_team = Some(0);
        r.city_captures = captures;
        r.team_vp_turn_75 = BTreeMap::from([(0, 1), (1, 1 + deficit)]);
        for (id, vp) in [(0, 10), (1, 10 - margin)] {
            r.players.push(crate::stats::PlayerRecord {
                opening: Default::default(),
                strategy: None,
                archetype: None,
                id,
                faction: Faction::India,
                team: id,
                brain: "scripted".into(),
                difficulty: Default::default(),
                alive: true,
                vp,
                cities: 1,
                techs: 1,
                population: 1,
                gold: Some(0),
            });
        }
        r
    }

    #[test]
    fn selection_covers_categories_and_is_stable_without_duplicates() {
        let mut records = vec![
            candidate(0, 0, 0, 0),
            candidate(1, 5, 9, 0),
            candidate(2, 4, 0, 7),
            candidate(3, 3, 2, 0),
            candidate(4, 1, 1, 1),
        ];
        records[3].outcome = Outcome::Conquest;
        records[3].players[1].alive = false;
        let selected = select(&records, 4);
        assert_eq!(
            selected.iter().map(|r| r.0).collect::<Vec<_>>(),
            [3, 0, 1, 2]
        );
        assert!(selected[3].1.contains("7 VP"));
        records.reverse();
        assert_eq!(select(&records, 4), selected);
        assert_eq!(select(&records, 100).len(), records.len());
        assert!(select(&records, 0).is_empty());
        assert!(select(&[], 6).is_empty());
        // Tied leaders are not a unique turn-75 leader; short games are not comebacks.
        let mut tied = candidate(9, 1, 0, 5);
        tied.team_vp_turn_75.insert(2, 6);
        assert_eq!(comeback(&tied), None);
        tied.team_vp_turn_75.remove(&2);
        tied.turns = 75;
        assert_eq!(comeback(&tied), None);
    }

    #[test]
    fn targets_and_winner_use_teams_not_player_ids() {
        let mut setup = GameSetup::all_ai(2);
        setup.turn_limit = Some(2);
        setup.players[1].team = TeamId(0);
        let played = runner::play(&setup, BrainMode::Scripted, true, 100);
        let replay = from_tape(&played.tape, "teams").unwrap();
        let target = |team: u8| scoring::team_target(&played.state, TeamId(team));
        assert_eq!(replay.header.targets[&0], target(0));
        assert_eq!(replay.header.targets[&2], target(2));
        assert_eq!(
            target(0),
            2 * target(2),
            "a pair's target is two members' worth"
        );
        assert_eq!(replay.header.winner, played.state.winner.map(|t| t.0));
    }

    #[test]
    fn simultaneous_battles_count_distinct_unit_losses() {
        use legatum_core::{Breakdown, UnitId};
        let state = legatum_core::new_game(&GameSetup::all_ai(1));
        let battle =
            |(attacker, attacker_owner, attacker_hp_after),
             (defender, defender_owner, defender_hp_after)| Event::Battle {
                attacker: UnitId(attacker).into(),
                attacker_owner: PlayerId(attacker_owner),
                attacker_at: None,
                defender: Defender::Unit {
                    unit: UnitId(defender),
                    owner: PlayerId(defender_owner),
                    unit_kind: UnitKind::Soldier,
                },
                at: Hex::new(30, 9),
                attacker_strength: Breakdown::default(),
                defender_strength: Breakdown::default(),
                damage_to_attacker: 30,
                damage_to_defender: 30,
                attacker_roll: 100,
                defender_roll: 100,
                attacker_hp_after,
                defender_hp_after,
                ranged: false,
                meeting: false,
                air_strike: false,
                defender_leaving: false,
            };
        for (events, losses) in [
            // Two attacks report the same defender's death.
            (
                vec![
                    battle((10, 0, 100), (21, 1, 0)),
                    battle((11, 0, 100), (21, 1, 0)),
                ],
                1,
            ),
            // Distinct dead attackers must still count alongside the defender.
            (
                vec![
                    battle((10, 0, 0), (21, 1, 0)),
                    battle((11, 0, 0), (21, 1, 0)),
                ],
                3,
            ),
            // A dead unit can appear as both attacker and defender.
            (
                vec![
                    battle((10, 0, 0), (21, 1, 70)),
                    battle((21, 1, 70), (10, 0, 0)),
                ],
                1,
            ),
        ] {
            let notable = notable_events(&state, 73, &events);
            assert_eq!(notable.len(), 1);
            assert_eq!(notable[0].turn, 73);
            assert!(
                notable[0]
                    .text
                    .ends_with(&format!("2 clashes, {losses} units lost")),
                "{}",
                notable[0].text
            );
        }
    }

    #[test]
    fn notable_event_categories_are_readable_and_minor_battles_are_filtered() {
        use legatum_core::{Breakdown, BuildingKind, UnitId};
        let state = legatum_core::new_game(&GameSetup::all_ai(1));
        let battle = |hp| Event::Battle {
            attacker: UnitId(1).into(),
            attacker_owner: PlayerId(0),
            attacker_at: None,
            defender: Defender::Unit {
                unit: UnitId(2),
                owner: PlayerId(1),
                unit_kind: UnitKind::Soldier,
            },
            at: Hex::new(2, 3),
            attacker_strength: Breakdown::default(),
            defender_strength: Breakdown::default(),
            damage_to_attacker: 30,
            damage_to_defender: 30,
            attacker_roll: 100,
            defender_roll: 100,
            attacker_hp_after: hp,
            defender_hp_after: 70,
            ranged: false,
            meeting: false,
            air_strike: false,
            defender_leaving: false,
        };
        assert!(notable_events(&state, 10, &[battle(70)]).is_empty());
        let events = vec![
            Event::CityFounded {
                city: CityId(1),
                owner: PlayerId(0),
                name: "Dawn".into(),
                at: Hex::new(2, 3),
                veteran: false,
            },
            Event::CityCaptured {
                city: CityId(1),
                from: PlayerId(0),
                to: PlayerId(1),
                bounty: None,
            },
            Event::PlayerEliminated {
                player: PlayerId(0),
            },
            Event::FactionGoalCompleted {
                player: PlayerId(1),
                faction: Faction::India,
                vp: 2,
                again: false,
            },
            Event::MegaprojectStarted {
                city: CityId(1),
                owner: PlayerId(1),
                building: BuildingKind::RelayUplink,
            },
            Event::ProductionCompleted {
                city: CityId(1),
                owner: PlayerId(1),
                item: ProductionItem::Building {
                    building: BuildingKind::RelayUplink,
                },
            },
            Event::VictoryHoldStarted {
                team: legatum_core::TeamId(1),
                vp: 10,
                target: 10,
                turns_needed: 10,
            },
            Event::VictoryHoldBroken {
                team: legatum_core::TeamId(1),
                turns_held: 4,
                vp: 9,
                target: 10,
            },
            battle(0),
        ];
        let notable = notable_events(&state, 10, &events);
        assert_eq!(notable.len(), events.len());
        assert!(notable.iter().all(|e| e.turn == 10));
        for (event, text) in notable.iter().zip([
            "founded Dawn",
            "captured",
            "eliminated",
            "India faction goal",
            "started Great Library",
            "completed Great Library",
            "must hold it for 10 turns",
            "lost its VP hold after 4 turns",
            "1 units lost",
        ]) {
            assert!(
                event.text.contains(text),
                "{} does not contain {text}",
                event.text
            );
        }
        assert_eq!(
            notable_events(&state, 10, &[battle(70), battle(70), battle(70)]).len(),
            1
        );
    }
}

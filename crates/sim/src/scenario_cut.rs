//! `legatum-sim scenario-cut`: cut an AI measurement position from any save of a game
//! recorded on the current engine.
//!
//! The save's tape is replayed with the current engine (every turn checked against the recorded
//! hash), the state before turn `--turn` is taken, and the position is written to
//! `scenarios/lab/<id>.json` with its objective and turn limit. Measurement runs make every
//! seat an AI. `tools/scenarios/cut.sh` measures the Hard baseline. `--inspect` prints one
//! line per turn so the right turn is easy to pick.
//!
//! The save is only read.

use legatum_core::scenario::{Condition, Metric, Objective, ScoreTerm};
use legatum_core::{AiDifficulty, CityId, Faction, GameState, PlayerId, UnitKind, resolve_turn};
use legatum_wasm::rusher::{Rusher, RusherWar};
use legatum_wasm::scenario::{
    FORMAT, Predictions, ScenarioFile, ScorePrediction, VERSION, encode_state,
};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    name = "legatum-sim scenario-cut",
    about = "Cut an AI measurement position from a save at a turn"
)]
pub struct Cli {
    /// The save (a .legatum file whose game carries its recorded history).
    #[arg(long)]
    pub save: Option<PathBuf>,
    /// The turn the scenario starts on: the position before that turn's orders.
    #[arg(long)]
    pub turn: Option<u32>,
    /// A state dumped by `tools/scenarios/extract-state.sh` (a save replayed on the engine it was
    /// recorded on), used in place of `--save` for a save from an older engine.
    #[arg(long)]
    pub state: Option<PathBuf>,
    /// Print a line per turn (wars, cities, units) instead of cutting; `--from` and `--to` bound it.
    #[arg(long)]
    pub inspect: bool,
    #[arg(long, default_value_t = 1)]
    pub from: u32,
    #[arg(long, default_value_t = 10_000)]
    pub to: u32,
    /// The empire being measured: its player number, faction (Sparta) or country (Sparta).
    #[arg(long)]
    pub seat: Option<String>,
    /// File name and id: lowercase letters, digits and dashes.
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    /// Optional context for interpreting this AI measurement position.
    #[arg(long)]
    pub briefing: Option<String>,
    /// hold | take:<City>[+<City>] | eliminate:<empire>[+<empire>] | cities:<N> | score
    #[arg(long)]
    pub objective: Option<String>,
    /// Turns in the window.
    #[arg(long)]
    pub turns: Option<u32>,
    /// An AI empire that rushes on a script (see `legatum_wasm::rusher`): player number, faction or country.
    #[arg(long)]
    pub rusher: Option<String>,
    /// A war the rusher declares: `<empire>@<turn>`; repeat for several.
    #[arg(long = "war")]
    pub wars: Vec<String>,
    /// Units that must stand within two tiles of the target before the rusher attacks.
    #[arg(long, default_value_t = 3)]
    pub storm: u32,
    /// The empire the rusher marches on (default: the nearest city of any empire it is at war with).
    #[arg(long)]
    pub rush_target: Option<String>,
    /// Overwrite an existing scenario with the same id (its revision goes up).
    #[arg(long)]
    pub replace: bool,
    /// Go on when the replay's hashes differ from the save's (a save from an older engine).
    #[arg(long)]
    pub allow_mismatch: bool,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scenarios/lab"))]
    pub out: PathBuf,
}

/// The states before the turns `keep` selects (and the final state), and how many replayed
/// turns differed from the save's recorded hashes.
struct Replay {
    states: Vec<GameState>,
    mismatches: u32,
}

fn replay(cli: &Cli, keep: impl Fn(u32) -> bool) -> Result<Replay, String> {
    if let Some(path) = &cli.state {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let state = legatum_wasm::json::parse_state_for_rehash(&text)?;
        return Ok(Replay {
            states: vec![state],
            mismatches: 0,
        });
    }
    let save = cli.save.as_ref().ok_or("--save is required")?;
    let text = std::fs::read_to_string(save).map_err(|e| format!("{}: {e}", save.display()))?;
    let outer: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("not a save: {e}"))?;
    let game = outer["game"].as_str().ok_or("not a save: it has no game")?;
    let value: serde_json::Value =
        serde_json::from_str(game).map_err(|e| format!("bad game: {e}"))?;
    if value["archive"].is_null() {
        return Err("this save carries no recorded history, so it can't be replayed".into());
    }
    let history: legatum_wasm::history::Recording =
        serde_json::from_value(value["archive"].clone())
            .map_err(|e| format!("bad history: {e}"))?;
    let mut state = match &history.initial_state {
        Some(json) if cli.allow_mismatch => legatum_wasm::json::parse_state_for_rehash(json)?,
        Some(json) => legatum_wasm::json::parse_state(json)?,
        None => history.tape.initial_state(),
    };
    let mut states = vec![];
    let mut mismatches = 0;
    for t in &history.tape.turns {
        if keep(state.turn) {
            states.push(state.clone());
        }
        let (next, _) = resolve_turn(&state, &t.orders);
        state = next;
        if state.hash().to_string() != t.hash {
            mismatches += 1;
        }
    }
    if keep(state.turn) {
        states.push(state);
    }
    Ok(Replay { states, mismatches })
}

/// Country names for the factions, so `--seat Sparta` works.
fn country(f: Faction) -> &'static str {
    match f {
        Faction::Switzerland => "Switzerland",
        Faction::India => "India",
        Faction::Byzantium => "Byzantium",
        Faction::Germany => "Germany",
        Faction::Celts => "Celts",
        Faction::Mongols => "Mongols",
        Faction::Sparta => "Sparta",
        Faction::Athens => "Athens",
        Faction::Aztecs => "Aztecs",
        Faction::Egypt => "Egypt",
        Faction::Rome => "Rome",
        Faction::Rus => "Rus",
    }
}

fn player_named(state: &GameState, name: &str) -> Result<PlayerId, String> {
    let name = name.trim();
    if let Ok(n) = name.parse::<usize>() {
        return state
            .players
            .get(n)
            .map(|p| p.id)
            .ok_or_else(|| format!("no player {n}"));
    }
    state
        .players
        .iter()
        .find(|p| {
            format!("{:?}", p.faction).eq_ignore_ascii_case(name)
                || country(p.faction).eq_ignore_ascii_case(name)
        })
        .map(|p| p.id)
        .ok_or_else(|| {
            let all: Vec<String> = state
                .players
                .iter()
                .map(|p| format!("{} = {:?} / {}", p.id.0, p.faction, country(p.faction)))
                .collect();
            format!(
                "no empire called {name:?}; the players are: {}",
                all.join("; ")
            )
        })
}

fn city_named(state: &GameState, name: &str) -> Result<CityId, String> {
    state
        .cities
        .values()
        .find(|c| c.name.eq_ignore_ascii_case(name.trim()))
        .map(|c| c.id)
        .ok_or_else(|| {
            let all: Vec<&str> = state.cities.values().map(|c| c.name.as_str()).collect();
            format!(
                "no city called {name:?}; the cities are: {}",
                all.join(", ")
            )
        })
}

fn term(metric: Metric, weight: i32) -> ScoreTerm {
    ScoreTerm { metric, weight }
}

fn offence(extra: Vec<ScoreTerm>) -> Vec<ScoreTerm> {
    let mut s = vec![
        term(Metric::ObjectiveMet, 300),
        term(Metric::TurnsLeft, 20),
        term(Metric::CitiesCaptured, 50),
        term(Metric::StartingCitiesHeld, 100),
        term(Metric::CombatUnits, 10),
    ];
    s.extend(extra);
    s
}

fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The objective for a spec such as `take:Nicaea` (see `Cli::objective`), with its summary.
fn objective(
    spec: &str,
    state: &GameState,
    seat: PlayerId,
    turns: u32,
) -> Result<Objective, String> {
    let end = state.turn + turns;
    let (kind, arg) = spec.split_once(':').unwrap_or((spec, ""));
    let names = || arg.split('+').map(str::trim).filter(|s| !s.is_empty());
    let own = state.cities_of(seat).count();
    let objective = match kind {
        "hold" => Objective {
            summary: format!(
                "Hold {} through turn {end}.",
                if own == 1 {
                    "your city".to_string()
                } else {
                    format!("all {own} of your cities")
                }
            ),
            turns,
            win: vec![Condition::HoldStartingCities],
            end_on_win: false,
            keep: vec![Condition::HoldStartingCities],
            score: vec![
                term(Metric::StartingCitiesHeld, 100),
                term(Metric::CitiesFounded, 50),
                term(Metric::CitiesCaptured, 50),
                term(Metric::CombatUnits, 10),
            ],
        },
        "take" => {
            let ids = names()
                .map(|n| city_named(state, n))
                .collect::<Result<Vec<_>, _>>()?;
            if ids.is_empty() {
                return Err("take: needs a city name, e.g. take:Nicaea".into());
            }
            let shown: Vec<String> = ids.iter().map(|id| state.cities[id].name.clone()).collect();
            Objective {
                summary: format!(
                    "Take {} by turn {end}, keeping your own cities.",
                    and_list(&shown)
                ),
                turns,
                win: ids.iter().map(|id| Condition::OwnCity(*id)).collect(),
                end_on_win: true,
                keep: vec![Condition::HoldStartingCities],
                score: offence(
                    ids.iter()
                        .map(|id| term(Metric::CityDamage(*id), 1))
                        .collect(),
                ),
            }
        }
        "eliminate" => {
            let rivals = names()
                .map(|n| player_named(state, n))
                .collect::<Result<Vec<_>, _>>()?;
            if rivals.is_empty() {
                return Err("eliminate: needs an empire, e.g. eliminate:Germany".into());
            }
            let shown: Vec<String> = rivals
                .iter()
                .map(|p| country(state.player(*p).faction).to_string())
                .collect();
            Objective {
                summary: format!(
                    "Leave {} with no cities by turn {end}, keeping your own.",
                    and_list(&shown)
                ),
                turns,
                win: rivals
                    .iter()
                    .map(|p| Condition::RivalMaxCities { rival: *p, max: 0 })
                    .collect(),
                end_on_win: true,
                keep: vec![Condition::HoldStartingCities],
                score: offence(
                    rivals
                        .iter()
                        .map(|p| term(Metric::RivalCitiesLost(*p), 50))
                        .collect(),
                ),
            }
        }
        "cities" => {
            let n: u32 = arg
                .trim()
                .parse()
                .map_err(|_| "cities: needs a number, e.g. cities:7")?;
            Objective {
                summary: format!("Own {n} cities by turn {end}."),
                turns,
                win: vec![Condition::MinCities(n)],
                end_on_win: true,
                keep: vec![],
                score: vec![
                    term(Metric::ObjectiveMet, 300),
                    term(Metric::TurnsLeft, 20),
                    term(Metric::Cities, 50),
                    term(Metric::StartingCitiesHeld, 100),
                ],
            }
        }
        "score" => Objective {
            summary: format!("Gain as many VP as you can by turn {end}."),
            turns,
            win: vec![],
            end_on_win: false,
            keep: vec![],
            score: vec![
                term(Metric::VpGained, 100),
                term(Metric::Cities, 20),
                term(Metric::StartingCitiesHeld, 50),
            ],
        },
        other => {
            return Err(format!(
                "unknown objective {other:?}: use hold, take:<City>[+<City>], eliminate:<empire>[+<empire>], cities:<N> or score"
            ));
        }
    };
    Ok(objective)
}

fn inspect(cli: &Cli) -> Result<(), String> {
    let r = replay(cli, |t| t >= cli.from && t <= cli.to)?;
    println!("replay differs from the save on {} turns", r.mismatches);
    for s in &r.states {
        let mut wars = vec![];
        for a in s.players.iter().filter(|p| p.alive) {
            for b in s.players.iter().filter(|p| p.alive && p.id.0 > a.id.0) {
                if s.at_war(a.id, b.id) {
                    wars.push(format!("{}-{}", a.id.0, b.id.0));
                }
            }
        }
        let rows: Vec<String> = s
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let military = s
                    .units_of(p.id)
                    .filter(|u| {
                        legatum_core::rules::is_military(u.kind) && u.kind != UnitKind::Scout
                    })
                    .count();
                format!("{}:{}c/{}u", p.id.0, s.cities_of(p.id).count(), military)
            })
            .collect();
        let coalition = legatum_core::diplomacy::coalition::target_state(s)
            .map_or(String::new(), |t| {
                format!(" coalition-against-team {}", t.0)
            });
        println!(
            "T{} wars [{}] {}{coalition}",
            s.turn,
            wars.join(" "),
            rows.join(" ")
        );
    }
    // One turn asked for: list every city and military unit with its tile, so a briefing can say
    // who stands where.
    if let (true, [only]) = (cli.from == cli.to, r.states.as_slice()) {
        println!("details of turn {}:", only.turn);
        for p in only.players.iter().filter(|p| p.alive) {
            println!(
                "  player {} {:?} ({}): gold {}, wars with {:?}",
                p.id.0,
                p.faction,
                country(p.faction),
                p.gold,
                only.players
                    .iter()
                    .filter(|q| q.alive && q.id != p.id && only.at_war(p.id, q.id))
                    .map(|q| q.id.0)
                    .collect::<Vec<_>>()
            );
            for c in only.cities_of(p.id) {
                println!(
                    "    city {} (id {}) at {},{}: population {}, HP {}{}",
                    c.name,
                    c.id.0,
                    c.pos.q,
                    c.pos.r,
                    c.population,
                    c.hp,
                    if legatum_core::data::has_building(
                        &c.buildings,
                        legatum_core::BuildingKind::Walls
                    ) {
                        ", Walls"
                    } else {
                        ""
                    }
                );
            }
            for u in only
                .units_of(p.id)
                .filter(|u| legatum_core::rules::is_military(u.kind))
            {
                println!(
                    "    {} at {},{}: HP {}",
                    legatum_core::data::unit_name(u.kind),
                    u.pos.q,
                    u.pos.r,
                    u.hp
                );
            }
        }
    }
    if let Some(first) = r.states.first() {
        println!("players:");
        for p in &first.players {
            println!(
                "  {} = {:?} / {} (era {})",
                p.id.0,
                p.faction,
                country(p.faction),
                legatum_core::eras::era_of(first, p.id)
            );
        }
    }
    Ok(())
}

pub fn run(cli: &Cli) -> Result<(), String> {
    if cli.inspect {
        return inspect(cli);
    }
    let need = |v: &Option<String>, flag: &str| {
        v.clone()
            .filter(|s| !s.trim().is_empty())
            .ok_or(format!("--{flag} is required"))
    };
    let turn = match (&cli.state, cli.turn) {
        (Some(_), t) => t.unwrap_or(0),
        (None, Some(t)) => t,
        (None, None) => return Err("--turn is required".into()),
    };
    let id = need(&cli.id, "id")?;
    if !id
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("--id may use lowercase letters, digits and dashes only".into());
    }
    let title = need(&cli.title, "title")?;
    let briefing = cli.briefing.clone().unwrap_or_default();
    let spec = need(&cli.objective, "objective")?;
    let turns = cli.turns.filter(|t| *t > 0).ok_or("--turns is required")?;
    let seat_name = need(&cli.seat, "seat")?;

    let r = replay(cli, |t| t == turn)?;
    if r.mismatches > 0 && !cli.allow_mismatch {
        return Err(format!(
            "the replay differs from the save on {} turns: the save was recorded on another engine (dump its states with tools/scenarios/extract-state.sh and cut with --state, or pass --allow-mismatch for a different game)",
            r.mismatches
        ));
    }
    let mut state = r
        .states
        .into_iter()
        .next()
        .ok_or(format!("the save has no turn {turn}"))?;
    let turn = state.turn;
    let seat = player_named(&state, &seat_name)?;
    let mut edits = crate::war_build::modernize(&mut state, Some(seat), false);
    // --state is an extracted old-engine position. For --save, inspect the recorded
    // rules version; replay compatibility alone does not establish reveal semantics.
    let old_save = if let Some(path) = &cli.save {
        let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let outer: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        outer["game"]
            .as_str()
            .and_then(|game| serde_json::from_str::<serde_json::Value>(game).ok())
            .and_then(|game| game["save_version"].as_u64())
            .is_some_and(|version| version < 25)
    } else {
        cli.state.is_some()
    };
    if old_save {
        edits.extend(crate::scenario_reveal::migrate(&mut state));
    }
    let errors = legatum_core::scenario::validate_state(&state);
    if !errors.is_empty() {
        return Err(format!("illegal position: {}", errors.join("; ")));
    }
    let objective = objective(&spec, &state, seat, turns)?;
    let rusher = match &cli.rusher {
        None => None,
        Some(name) => {
            let player = player_named(&state, name)?;
            if player == seat {
                return Err("the rusher must not be the seat".into());
            }
            let mut wars = vec![];
            for w in &cli.wars {
                let (who, at) = w
                    .split_once('@')
                    .ok_or("--war needs <empire>@<turn>, e.g. Athens@16")?;
                wars.push(RusherWar {
                    against: player_named(&state, who)?,
                    turn: at
                        .trim()
                        .parse()
                        .map_err(|_| "--war turn must be a number")?,
                });
            }
            Some(Rusher {
                player,
                wars,
                storm: cli.storm,
                target: cli
                    .rush_target
                    .as_deref()
                    .map(|t| player_named(&state, t))
                    .transpose()?,
            })
        }
    };
    let out = cli.out.join(format!("{id}.json"));
    let revision = match std::fs::read_to_string(&out) {
        Ok(_) if !cli.replace => {
            return Err(format!(
                "{} exists; pass --replace to overwrite it",
                out.display()
            ));
        }
        Ok(old) => ScenarioFile::parse(&old).map_or(1, |f| f.revision + 1),
        Err(_) => 1,
    };
    let save_name = cli
        .save
        .as_ref()
        .or(cli.state.as_ref())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let how = if cli.state.is_some() {
        "a save replayed on the engine it was recorded on (every turn checked against the recorded hash)".to_string()
    } else {
        format!(
            "replayed with the current engine ({} hash mismatches)",
            r.mismatches
        )
    };
    let edit_note = if edits.is_empty() {
        String::new()
    } else {
        format!(" Edits: {}.", edits.join("; "))
    };
    let file = ScenarioFile {
        format: FORMAT.into(),
        version: VERSION,
        id: id.clone(),
        revision,
        title,
        faction: state.player(seat).faction,
        seat,
        difficulty: AiDifficulty::Hard,
        minutes: 0,
        briefing,
        tests: String::new(),
        objective,
        predictions: Predictions {
            good_play: String::new(),
            ai_play: String::new(),
            ai_score: ScorePrediction {
                median: 0,
                low: 0,
                high: 0,
                win_percent: 0,
            },
        },
        source: format!(
            "Cut from {save_name} at turn {turn}: the position before that turn's orders, {how}. Measurement runs use AI in every empire with fresh AI memory; the seat's standing orders are cleared. Built by `tools/scenarios/cut.sh`.{edit_note}",
        ),
        baseline: None,
        human: None,
        rusher,
        ai_memory: None,
        state: encode_state(&state),
    };
    std::fs::create_dir_all(&cli.out).map_err(|e| e.to_string())?;
    let mut text = serde_json::to_string_pretty(&file).expect("scenario JSON");
    text.push('\n');
    std::fs::write(&out, text).map_err(|e| e.to_string())?;
    println!("== {id} (revision {revision}) -> {}", out.display());
    for line in crate::war_build::describe(&state, seat) {
        println!("{line}");
    }
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    use clap::Parser;
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    run(&cli)
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{GameSetup, new_game};

    fn state() -> GameState {
        new_game(&GameSetup::all_ai(5))
    }

    #[test]
    fn seats_are_found_by_number_faction_or_country() {
        let s = state();
        let p = &s.players[2];
        assert_eq!(player_named(&s, "2").unwrap(), p.id);
        assert_eq!(
            player_named(&s, &format!("{:?}", p.faction).to_lowercase()).unwrap(),
            p.id
        );
        assert_eq!(player_named(&s, country(p.faction)).unwrap(), p.id);
        assert!(
            player_named(&s, "Atlantis")
                .unwrap_err()
                .contains("the players are")
        );
    }

    #[test]
    fn objectives_read_from_the_spec_and_name_their_end_turn() {
        let s = state();
        let seat = s.players[0].id;
        let hold = objective("hold", &s, seat, 12).unwrap();
        assert_eq!(hold.win, vec![Condition::HoldStartingCities]);
        assert!(!hold.end_on_win);
        assert!(
            hold.summary
                .ends_with(&format!("through turn {}.", s.turn + 12))
        );

        let cities = objective("cities:3", &s, seat, 20).unwrap();
        assert_eq!(cities.win, vec![Condition::MinCities(3)]);
        assert!(cities.end_on_win);
        assert_eq!(
            cities.summary,
            format!("Own 3 cities by turn {}.", s.turn + 20)
        );

        let rival = s.players[1].id;
        let out = objective(&format!("eliminate:{}", rival.0), &s, seat, 8).unwrap();
        assert_eq!(out.win, vec![Condition::RivalMaxCities { rival, max: 0 }]);
        assert!(
            out.score
                .iter()
                .any(|t| t.metric == Metric::RivalCitiesLost(rival))
        );

        assert!(objective("score", &s, seat, 5).unwrap().win.is_empty());
    }

    #[test]
    fn a_bad_spec_says_what_is_allowed() {
        let s = state();
        let seat = s.players[0].id;
        assert!(
            objective("conquer", &s, seat, 5)
                .unwrap_err()
                .contains("hold")
        );
        assert!(
            objective("take:Nowhere", &s, seat, 5)
                .unwrap_err()
                .contains("no city called")
        );
        assert!(objective("cities:many", &s, seat, 5).is_err());
        assert!(objective("take:", &s, seat, 5).is_err());
    }

    #[test]
    fn lists_read_naturally() {
        assert_eq!(and_list(&["A".into()]), "A");
        assert_eq!(and_list(&["A".into(), "B".into()]), "A and B");
        assert_eq!(
            and_list(&["A".into(), "B".into(), "C".into()]),
            "A, B, and C"
        );
    }
}

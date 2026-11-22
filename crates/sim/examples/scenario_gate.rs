//! `scenario_gate`: score AI configurations on every short scenario, paired by dice seed.
//!
//!   scenario_gate --arm base=builtin --arm cand=FILE.json [--runs 64] [--first-run 0]
//!                 [--params-seats seat|rivals] [--him hard|designer|params:FILE]
//!                 [--split FILE --set train|held_out|all] [--ids a,b] [--threads 16]
//!                 --out runs.jsonl
//!
//! Every Scenario Lab card (`scenarios/lab/*.json`) and war scenario (`scenarios/war/*.json`; a war
//! file whose start state is also a Lab card's is played once, as the card) is played `--runs`
//! times per arm. Run `r` uses the same dice seed for every arm (`legatum-sim scenario`'s
//! derivation: run 0 keeps the scenario's own seed), so two arms pair run by run. An arm is
//! `NAME=SPEC[@OFFSET]`: SPEC is `builtin` (the app's Hard) or an `AiParams` JSON file; OFFSET shifts
//! the run indices, which gives an A/A noise floor (`base=builtin --arm alt=builtin@100000`).
//!
//! `--params-seats seat` (default) gives the arm's parameters to the scenario seat only, every
//! other empire plays the built-in AI: the AI's strength in the position. `rivals` gives them to
//! every AI empire except the seat and the designer's empire (when `--him` replaces it).
//!
//! `--him` picks who plays the designer's own empire (player 0 in every tape he recorded) when it
//! is not the scenario seat: `hard` (default, as the Lab plays it), `designer` (the imitation seat,
//! `legatum_ai::designer`), or `params:FILE`. A war scenario that replays his recorded orders for
//! his seat keeps doing so.
//!
//! The scenario seat plays Hard; the other empires keep the scenario's difficulty. No seat searches
//! (the war suite gives Hard + Search opponents their search; this gate keeps every run cheap). The
//! score is the scenario's own (`ActiveScenario::status`, as the app shows it), `won` is its
//! objective met. One JSON line per run; `tools/sim/scenario_gate_report.py` pairs and pools them.

#[path = "war_script/mod.rs"]
mod war_script;

use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::rng::Rng;
use legatum_core::scenario::Outcome;
use legatum_core::{AiDifficulty, PlayerId};
use legatum_wasm::scenario::{ActiveScenario, ScenarioFile};
use legatum_wasm::session::{BrainKind, Session, brain_seed};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The designer's empire in every scenario cut from his games.
const HIM: PlayerId = PlayerId(0);

#[derive(clap::Parser, Debug)]
#[command(
    name = "scenario_gate",
    about = "Score AI configurations on every Lab card and war scenario, paired by dice seed"
)]
struct Cli {
    /// `NAME=SPEC[@OFFSET]`, SPEC `builtin` or an AiParams JSON file (repeat; at least one).
    #[arg(long = "arm", required = true)]
    arms: Vec<String>,
    #[arg(long, default_value_t = 64)]
    runs: u32,
    #[arg(long, default_value_t = 0)]
    first_run: u32,
    /// Which seats play the arm's parameters: seat or rivals.
    #[arg(long, default_value = "seat")]
    params_seats: String,
    /// Who plays the designer's empire when it is not the seat: hard, designer, params:FILE.
    #[arg(long, default_value = "hard")]
    him: String,
    /// Card directories (default: scenarios/lab and scenarios/war).
    #[arg(long = "dir")]
    dirs: Vec<PathBuf>,
    /// The fixed train/held-out split (`docs/sim-results/scenario-gate/split.json`).
    #[arg(long)]
    split: Option<PathBuf>,
    /// Which cards of the split to play: train, held_out or all.
    #[arg(long, default_value = "all")]
    set: String,
    /// Only these card ids (comma-separated).
    #[arg(long, value_delimiter = ',')]
    ids: Vec<String>,
    /// Worker threads (default: every core).
    #[arg(long, default_value_t = 0)]
    threads: usize,
    /// Print the cards (id, set, seat, his empire, turns) and exit.
    #[arg(long)]
    list: bool,
    /// JSON lines, one per run.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Clone)]
enum Config {
    Builtin,
    Params(&'static AiParams),
    Designer,
}

impl Config {
    fn parse(spec: &str) -> Result<Self, String> {
        match spec {
            "builtin" | "hard" => Ok(Self::Builtin),
            "designer" => Ok(Self::Designer),
            other => {
                let path = other.strip_prefix("params:").unwrap_or(other);
                let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
                let params: AiParams =
                    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
                params.validate().map_err(|e| format!("{path}: {e}"))?;
                Ok(Self::Params(Box::leak(Box::new(params))))
            }
        }
    }

    fn brain(
        &self,
        faction: legatum_core::Faction,
        seed: u64,
        id: PlayerId,
        difficulty: AiDifficulty,
    ) -> Box<dyn Brain> {
        match self {
            Self::Builtin => Box::new(ScriptedBrain::with_difficulty(
                faction, seed, id, difficulty,
            )),
            Self::Params(p) => {
                Box::new(ScriptedBrain::with_params(faction, seed, id, difficulty, p))
            }
            Self::Designer => Box::new(legatum_ai::designer::brain(faction, seed, id)),
        }
    }
}

struct Arm {
    name: String,
    config: Config,
    offset: u32,
}

fn parse_arm(s: &str) -> Result<Arm, String> {
    let (name, rest) = s
        .split_once('=')
        .ok_or_else(|| format!("--arm {s}: expected NAME=SPEC[@OFFSET]"))?;
    let (spec, offset) = match rest.rsplit_once('@') {
        Some((spec, off)) => (
            spec,
            off.parse()
                .map_err(|_| format!("--arm {s}: offset {off} is not a number"))?,
        ),
        None => (rest, 0),
    };
    Ok(Arm {
        name: name.to_string(),
        config: Config::parse(spec)?,
        offset,
    })
}

struct Card {
    file: ScenarioFile,
    kind: &'static str,
    set: String,
}

/// The seed for run `run`: the derivation of `legatum-sim scenario` and the war suite.
fn run_seed(seed: u64, run: u32) -> u64 {
    if run == 0 {
        seed
    } else {
        Rng::derive(seed, &[0x5CE4_A210, u64::from(run)]).next_u64()
    }
}

fn load_cards(cli: &Cli) -> Result<Vec<Card>, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dirs = if cli.dirs.is_empty() {
        vec![root.join("scenarios/lab"), root.join("scenarios/war")]
    } else {
        cli.dirs.clone()
    };
    let split: BTreeMap<String, String> = match &cli.split {
        Some(path) => {
            let text =
                std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut m = BTreeMap::new();
            for set in ["train", "held_out"] {
                for id in v[set].as_array().into_iter().flatten() {
                    let id = id.as_str().ok_or("split ids must be strings")?;
                    m.insert(id.to_string(), set.to_string());
                }
            }
            m
        }
        None => BTreeMap::new(),
    };
    let mut cards = vec![];
    let mut hashes = BTreeSet::new();
    for dir in &dirs {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let file =
                ScenarioFile::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            // A war file cut from the same position as a Lab card is the same test.
            let key = (
                file.state.hash.clone(),
                file.source.clone(),
                file.seat,
                file.objective.summary.clone(),
            );
            if !hashes.insert(key) {
                continue;
            }
            if !cli.ids.is_empty() && !cli.ids.contains(&file.id) {
                continue;
            }
            let set = split
                .get(&file.id)
                .cloned()
                .unwrap_or_else(|| "unsplit".into());
            if cli.set != "all" && set != cli.set {
                continue;
            }
            let kind = if dir.ends_with("war") { "war" } else { "lab" };
            cards.push(Card { file, kind, set });
        }
    }
    Ok(cards)
}

#[derive(serde::Serialize)]
struct Line<'a> {
    card: &'a str,
    /// Which seats played the arm's parameters (seat or rivals), and who played his empire.
    mode: &'a str,
    him: &'a str,
    kind: &'a str,
    set: &'a str,
    arm: &'a str,
    run: u32,
    seed: u64,
    score: i32,
    won: bool,
    outcome: Outcome,
    turns: u32,
    /// The designer's empire's cities at the end (when it is a rival).
    him_cities: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct Played {
    seed: u64,
    score: i32,
    outcome: Outcome,
    turns: u32,
    him_cities: Option<u32>,
}

fn play(card: &Card, arm: &Arm, run: u32, rivals: bool, him: &Config) -> Result<Played, String> {
    let file = &card.file;
    let mut start = file.prepare(false)?;
    start.setup.seed = run_seed(start.setup.seed, run + arm.offset);
    let seat = file.seat;
    start.setup.players[seat.0 as usize].difficulty = AiDifficulty::Hard;
    let script = war_script::load(&file.id)?;
    let memories = file.ai_memories()?;
    let active = ActiveScenario::new(file.clone(), start.clone());
    let mut session = Session::from_state(start, |_| BrainKind::Scripted);
    session.history = None;
    let him_rival = seat != HIM && (HIM.0 as usize) < session.state.players.len();
    let scripted_him = script.as_ref().is_some_and(|s| s.seat == HIM);
    let seats: Vec<_> = session
        .state
        .players
        .iter()
        .filter(|p| !p.is_human)
        .map(|p| (p.id, p.faction))
        .collect();
    for (id, faction) in seats {
        let difficulty = session.state.setup.players[id.0 as usize].difficulty;
        let seed = brain_seed(session.state.setup.seed, id);
        let config = if id == seat {
            if rivals {
                &Config::Builtin
            } else {
                &arm.config
            }
        } else if id == HIM && him_rival && !scripted_him && !matches!(him, Config::Builtin) {
            him
        } else if rivals {
            &arm.config
        } else {
            &Config::Builtin
        };
        let mut brain = config.brain(faction, seed, id, difficulty);
        if let Some(memory) = memories.get(&id) {
            brain.restore_opponent_memory(memory.clone());
        }
        session.replace_brain(id, brain);
    }
    let mut status = active.status(&session.state);
    let limit = file.objective.turns + 1;
    let mut played = 0;
    while status.outcome == Outcome::Playing && played < limit {
        let turn = session.state.turn;
        let mut orders = session.ai_orders();
        if let Some(r) = &file.rusher {
            legatum_wasm::rusher::apply(&mut orders, &session.state, r);
        }
        if let Some(script) = &script {
            script.apply(turn, &mut orders);
        }
        session.step(&orders);
        played += 1;
        status = active.status(&session.state);
    }
    Ok(Played {
        seed: active.start.setup.seed,
        score: status.score,
        outcome: status.outcome,
        turns: played,
        him_cities: him_rival.then(|| session.state.cities_of(HIM).count() as u32),
    })
}

fn main() -> Result<(), String> {
    use clap::Parser;
    let cli = Cli::parse();
    let rivals = match cli.params_seats.as_str() {
        "seat" => false,
        "rivals" => true,
        other => {
            return Err(format!(
                "--params-seats must be seat or rivals, not {other}"
            ));
        }
    };
    let him = Config::parse(&cli.him)?;
    let arms: Vec<Arm> = cli
        .arms
        .iter()
        .map(|s| parse_arm(s))
        .collect::<Result<_, _>>()?;
    let cards = load_cards(&cli)?;
    if cli.list {
        for c in &cards {
            let state = c.file.stored_state()?;
            println!(
                "{}\t{}\t{}\tseat {} {:?}\thim {:?}\t{} turns",
                c.file.id,
                c.kind,
                c.set,
                c.file.seat.0,
                c.file.faction,
                state.players.first().map(|p| p.faction),
                c.file.objective.turns
            );
        }
        return Ok(());
    }
    let out = cli.out.as_ref().ok_or("--out FILE is required")?;
    let jobs: Vec<(usize, usize, u32)> = (0..cards.len())
        .flat_map(|c| {
            (0..arms.len()).flat_map(move |a| {
                (cli.first_run..cli.first_run + cli.runs).map(move |r| (c, a, r))
            })
        })
        .collect();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build()
        .map_err(|e| e.to_string())?;
    let began = std::time::Instant::now();
    let writer = std::sync::Mutex::new(std::io::BufWriter::new(
        std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?,
    ));
    let errors = std::sync::atomic::AtomicUsize::new(0);
    pool.install(|| {
        jobs.par_iter().for_each(|&(c, a, run)| {
            let (card, arm) = (&cards[c], &arms[a]);
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play(card, arm, run, rivals, &him)
            }));
            let result = match caught {
                Ok(r) => r,
                Err(p) => Err(p
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic".into())),
            };
            let failed = Played {
                seed: 0,
                score: 0,
                outcome: Outcome::Lost,
                turns: 0,
                him_cities: None,
            };
            if result.is_err() {
                errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            let p = result.as_ref().unwrap_or(&failed);
            let line = Line {
                card: &card.file.id,
                mode: &cli.params_seats,
                him: &cli.him,
                kind: card.kind,
                set: &card.set,
                arm: &arm.name,
                run,
                seed: p.seed,
                score: p.score,
                won: p.outcome == Outcome::Won,
                outcome: p.outcome,
                turns: p.turns,
                him_cities: p.him_cities,
                error: result.as_ref().err().cloned(),
            };
            let text = serde_json::to_string(&line).expect("json line");
            let mut w = writer.lock().expect("writer");
            writeln!(w, "{text}").expect("write line");
        });
    });
    writer
        .into_inner()
        .map_err(|e| e.to_string())?
        .flush()
        .map_err(|e| e.to_string())?;
    eprintln!(
        "scenario_gate: {} cards x {} arms x {} runs = {} runs in {}s ({} errors) -> {}",
        cards.len(),
        arms.len(),
        cli.runs,
        jobs.len(),
        began.elapsed().as_secs(),
        errors.load(std::sync::atomic::Ordering::Relaxed),
        out.display()
    );
    Ok(())
}

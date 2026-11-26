//! Headless simulation runner: AI-only games in parallel, determinism checks, order tapes and
//! result reports in docs/sim-results/.

mod barbarian_telemetry;
mod border_telemetry;
mod card_loop;
mod clock;
mod goal_city_telemetry;
mod hold_telemetry;
mod lobby;
mod openings;
mod primitive_telemetry;
mod proxy_benchmark;
mod replay;
mod runner;
mod scenario;
mod scenario_build;
mod scenario_check;
mod scenario_cut;
mod scenario_reveal;
mod scenario_told;
mod stats;
mod telemetry;
mod vp_race;
mod war_build;

use clap::Parser;

/// mimalloc: about 15% fewer cycles per game than the system allocator (docs/sim-results/perf).
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;
use legatum_ai::Strategy;
use legatum_sim::diversity::{
    CheckStatus, DEFAULT_MIN_ENTROPY, DEFAULT_MIN_MATCHUP_GAMES, GuardrailConfig,
};
use rayon::prelude::*;
use runner::{BrainMode, RunOpts};
use serde_json::json;
use stats::{GameRecord, RunMeta, summarize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const DEFAULT_OUT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/sim-results");
const DEFAULT_REPLAY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../.progress/replays");

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum DiversityBasis {
    Observed,
    Assigned,
    Both,
}

#[derive(Parser, Debug)]
#[command(name = "legatum-sim", about = "Run AI-only Legatum games headlessly")]
struct Cli {
    /// Number of independent empires (default lobby unchanged).
    #[arg(long, default_value_t = 8, value_parser = clap::value_parser!(u8).range(2..=8),
        conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    players: u8,
    /// Map preset: small 40x26, medium 48x32, large 60x40.
    #[arg(long, value_enum, default_value_t = lobby::MapSize::Medium,
        conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    map_size: lobby::MapSize,
    /// Factions in seat order; exactly --players names, or a seeded draw when omitted.
    #[arg(long, value_delimiter = ',', value_parser = lobby::parse_faction,
        conflicts_with_all = ["proxy", "replay", "replay_export", "tape_in"])]
    factions: Vec<legatum_core::Faction>,
    /// Rotate the explicitly listed factions one seat per game.
    #[arg(long, requires = "factions", conflicts_with = "mirror")]
    rotate_factions: bool,
    /// Play each requested seed twice, reversing policies on the same map/factions.
    #[arg(long, conflicts_with_all = ["proxy", "strategy_matchups", "replay", "replay_export", "tape_in"])]
    mirror: bool,
    /// Optional design levers, saved in setups and tapes.
    #[arg(long = "rules-experiment", value_delimiter = ',',
        value_parser = parse_experiment,
        conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    rules_experiment: Vec<String>,
    /// Ablation: switch faction rules off, comma-separated snake_case names (for example
    /// `germany_ruhr_production`). An unknown name lists the valid ones.
    #[arg(long = "rule-off", value_delimiter = ',',
        value_parser = parse_rule_off,
        conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    rule_off: Vec<legatum_core::experiments::FactionRule>,
    /// Opening policy by zero-based seat; repeat or use all=Opening (left-to-right overrides).
    #[arg(long, conflicts_with_all = ["proxy", "strategies", "strategy_matchups", "replay", "replay_export", "tape_in"])]
    seat_opening: Vec<String>,
    /// Enable composable behaviour primitives for Default scripted seats as well.
    #[arg(long, conflicts_with_all = ["proxy", "strategies", "strategy_matchups", "replay", "replay_export", "tape_in"])]
    ai_primitives: bool,
    /// Hard seats that keep the scripted Hard AI under --hard-v2: `all` or seat numbers (comma
    /// list). Scripted Hard is the default; Default seats in --seat-opening runs always are.
    #[arg(long, value_delimiter = ',', conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    legacy_hard: Vec<String>,
    /// Every Hard seat plays Hard v2 (league-tuned openings; the `hard-v2-ai` setup switch).
    /// --legacy-hard seats stay scripted Hard.
    #[arg(long, conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    hard_v2: bool,
    /// Every Hard seat plays its tuned opening, then permanently switches to scripted Hard.
    /// Simulator only; --legacy-hard seats and the --challenger retain their own policies.
    #[arg(long, conflicts_with_all = ["hard_v2", "hard_v2_rotation", "seat_opening", "ai_primitives", "strategies", "strategy_matchups", "ai_params", "opponent_params", "replay", "replay_export", "tape_in"])]
    hard_v2_handoff: bool,
    /// One Hard v2 seat per game (seat = game index mod 8) against scripted Hard everywhere else.
    #[arg(long, conflicts_with_all = ["legacy_hard", "seat_opening", "proxy", "replay", "replay_export", "tape_in"])]
    hard_v2_rotation: bool,
    /// One challenger seat per game (seat = game index mod 8) plays this opening with its
    /// Hard v2 tuned genome against the lobby's AIs (Hard v2, or pre-v2 with --legacy-hard all).
    #[arg(long, conflicts_with_all = ["hard_v2_rotation", "seat_opening", "proxy", "replay", "replay_export", "tape_in"])]
    challenger: Option<String>,
    /// Flat integer genome JSON for one opening, Opening=path.json (repeatable).
    #[arg(long, requires = "seat_opening")]
    opening_genome: Vec<String>,
    /// Print opening genome bounds, built-in policies and vectors as JSON, then exit.
    #[arg(long)]
    opening_schema: bool,
    /// Number of games (seeds with --mirror, which produces twice this many games).
    #[arg(long, default_value_t = 10)]
    games: usize,
    /// First seed; game i uses seed + i.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Worker threads (default: all cores).
    #[arg(long, value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
    threads: Option<usize>,
    /// Report label: docs/sim-results/<date>-<label>.md
    #[arg(long, default_value = "run")]
    label: String,
    /// Stop runaway simulations without awarding victory (default: 4x expected length).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..u32::MAX as i64))]
    safety_cap: Option<u32>,
    /// Optional explicit turn limit; highest VP wins at this turn.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..u32::MAX as i64))]
    turn_limit: Option<u32>,
    /// Play every game twice and compare per-turn hashes.
    #[arg(long)]
    determinism: bool,
    #[arg(long, value_enum, default_value_t = BrainMode::Scripted)]
    brain: BrainMode,
    /// Seat one human-proxy benchmark opponent (0..7) against seven scripted AIs.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..8), conflicts_with_all = ["difficulty_rotation", "strategies", "strategy_matchups"])]
    proxy: Option<u8>,
    /// Rotate the Proxy's seat and faction in a balanced 64-game cycle.
    #[arg(long, requires = "proxy")]
    rotate_proxy: bool,
    /// Policy: 2 for legacy combat, 4 for alliances, 5/6/7 for versioned human playbooks (opt-in).
    /// Legacy proxy protocol version retained for a clear compatibility error.
    #[arg(long, default_value_t = legatum_ai::PROXY_VERSION, value_parser = parse_proxy_version, requires = "proxy")]
    proxy_version: u32,
    /// Run a tuned opening in the Proxy harness for matched seeds, factions and telemetry.
    #[arg(long, requires = "proxy", conflicts_with = "proxy_version")]
    proxy_opening: Option<legatum_ai::openings::Opening>,
    /// How Proxy benchmark games start: `war` (legacy all-war, as Proxy v2 was calibrated)
    /// or `peace` (current diplomacy). Recorded in every report.
    /// Defaults to Peace for v3/v4/v5/v6/v7, legacy all-war for v2.
    #[arg(long, value_enum, requires = "proxy")]
    proxy_start: Option<proxy_benchmark::ProxyStart>,
    /// Barbarian camps (true/false). Default: on, except Proxy benchmarks, which stay
    /// comparable with their barbarian-free calibration.
    #[arg(long)]
    barbarians: Option<bool>,
    /// Difficulty of the seven opponents in a Proxy benchmark, or of every seat otherwise.
    #[arg(long, default_value = "normal", value_parser = parse_difficulty)]
    difficulty: legatum_core::AiDifficulty,
    /// Hard+Search: scripted Hard seats decide plan reviews by rollouts. Value: `k=3,h=20,d=2`
    /// (any subset; `omniscient` adds the diagnostic upper bound that reads the true state).
    #[arg(long, value_parser = legatum_ai::search::SearchConfig::parse,
        conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    search: Option<legatum_ai::search::SearchConfig>,
    /// Training mode: every Hard AI seat's battles are searched (`legatum_ai::training`), and
    /// each game's log goes to DIR/game-<seed>.jsonl (the app's format, source `selfplay`).
    #[arg(long, value_name = "DIR")]
    training: Option<std::path::PathBuf>,
    /// Search every battle in a training log again (an app or self-play log) at full width,
    /// with perturbed variants, writing lines in the same schema to --retrain-out. Meant to
    /// run overnight on the Mac under `nice`: `nice -n 15 legatum-sim --retrain LOG`.
    #[arg(long, value_name = "LOG")]
    retrain: Option<std::path::PathBuf>,
    /// With --retrain: the output log (default: LOG with `.retrain.jsonl`).
    #[arg(long, value_name = "FILE", requires = "retrain")]
    retrain_out: Option<std::path::PathBuf>,
    /// With --retrain: only the logged battles, no perturbed variants.
    #[arg(long, requires = "retrain")]
    retrain_no_variants: bool,
    /// With --training or --retrain: rollouts per battle (deterministic budget; 0 = until converged).
    #[arg(long, default_value_t = 4096)]
    training_budget: u64,
    /// With --search: one search seat per game (seat = game index mod players), the rest plain.
    #[arg(long, requires = "search")]
    search_rotation: bool,
    /// 1v1 over all 64 ordered faction pairs (needs --players 2): game i plays pair
    /// (i / 2) mod 64 on seed + i / 2, side A (the --search-rotation seat) in seat i mod 2.
    #[arg(long, conflicts_with_all = ["factions", "mirror", "proxy", "strategy_matchups"])]
    duel_matrix: bool,
    /// With --duel-matrix: side B plays this opponent instead of plain Hard: a full-game
    /// opening policy on Hard (e.g. rush) or a human-playbook proxy (proxy8o, proxy8g,
    /// proxy7, proxy6, proxy5), an
    /// opponent that exploits plain Hard.
    #[arg(long, requires = "duel_matrix", conflicts_with = "seat_opening")]
    duel_opponent: Option<runner::DuelOpponent>,
    /// Tuned AI decision parameters (JSON, see docs/ai-evolution.md) for scripted seats.
    /// Knobs only: the file cannot express rules, yields, costs or vision (pillar 6).
    #[arg(long, conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    ai_params: Option<PathBuf>,
    /// Parameters for half the seats (seat s of game i when s + i is odd); the rest use
    /// --ai-params or the built-in AI. For head-to-head league games.
    #[arg(long, conflicts_with_all = ["proxy", "replay", "replay_export", "tape_in"])]
    opponent_params: Option<PathBuf>,
    /// Print the built-in AI parameters (every faction's knobs) as JSON, then exit.
    #[arg(long)]
    print_ai_params: bool,
    /// Print the bounded tactical and macro decision gene catalog.
    #[arg(long)]
    print_ai_genes: bool,
    /// Scripted strategy biases by seat, comma-separated; 1, 2, 4 or 8 values repeat to fill seats.
    #[arg(long, value_delimiter = ',', conflicts_with_all = ["strategy_matchups", "difficulty_rotation"])]
    strategies: Vec<Strategy>,
    /// Balanced round robin of all five biases; --games must be a positive multiple of 20.
    #[arg(long, conflicts_with_all = ["strategies", "difficulty_rotation"])]
    strategy_matchups: bool,
    /// Exit 1 on a diversity failure, 2 on insufficient evidence; always write the reports.
    #[arg(long, conflicts_with_all = ["replay", "replay_export", "tape_in"])]
    diversity_check: bool,
    /// Which evidence the CLI guardrail checks; reports always show both when available.
    #[arg(long, value_enum, default_value_t = DiversityBasis::Observed)]
    diversity_basis: DiversityBasis,
    #[arg(long, default_value_t = DEFAULT_MIN_ENTROPY, value_parser = parse_entropy)]
    diversity_min_entropy: f64,
    #[arg(long, default_value_t = DEFAULT_MIN_MATCHUP_GAMES, value_parser = clap::value_parser!(u64).range(1..))]
    diversity_min_matchup_games: u64,
    /// Cycle Easy/Normal/Hard across seats, starting at offset 0, 1 or 2 (scripted only).
    /// Run all three offsets with the same seeds to balance every faction and start.
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..3))]
    difficulty_rotation: Option<u8>,
    /// Write one order tape per game into this directory.
    #[arg(long)]
    tape_out: Option<PathBuf>,
    /// Write one JSONL decision file per game, with one row per Hard seat-turn.
    #[arg(long)]
    decision_trace: Option<PathBuf>,
    /// Replay every tape in this directory (no AI) and check hashes, then exit.
    #[arg(long)]
    tape_in: Option<PathBuf>,
    /// Archive every simulated game outside the repository, including capped runs.
    #[arg(long)]
    archive: bool,
    /// Diagnose AI behavior and aggregate incident/opportunity rates by faction.
    /// With --replay, write that archive's incidents and narrative to --out.
    #[arg(long, conflicts_with_all = ["tape_in", "replay_export"])]
    analyze: bool,
    /// Override LEGATUM_ARCHIVE / ~/LegatumData/archive.
    #[arg(long, requires = "archive")]
    archive_dir: Option<PathBuf>,
    /// Verify a compressed archive against the current engine, then exit.
    #[arg(long, conflicts_with_all = ["tape_in", "replay_export", "archive"])]
    replay: Option<PathBuf>,
    /// Export a tape JSON or directory of tapes as dashboard replays, then exit.
    #[arg(long, conflicts_with_all = ["tape_in", "export_replays", "tape_out"])]
    replay_export: Option<PathBuf>,
    /// Export the N most interesting games after a normal simulation run.
    #[arg(long, conflicts_with = "tape_in")]
    export_replays: Option<usize>,
    /// Replay output directory (reports still use --out-dir).
    #[arg(long, default_value = DEFAULT_REPLAY_DIR)]
    out: PathBuf,
    /// Rewrite a small progress JSON here every ~2 seconds.
    #[arg(long)]
    status_file: Option<PathBuf>,
    /// Where reports go.
    #[arg(long, default_value = DEFAULT_OUT_DIR)]
    out_dir: PathBuf,
}

fn parse_rule_off(name: &str) -> Result<legatum_core::experiments::FactionRule, String> {
    legatum_core::experiments::FactionRule::parse(name)
}

fn parse_experiment(flag: &str) -> Result<String, String> {
    legatum_core::experiments::RuleExperiments::default().apply(flag)?;
    Ok(flag.to_string())
}

fn parse_proxy_version(value: &str) -> Result<u32, String> {
    match value.parse::<u32>() {
        Ok(v @ (2 | 4 | 5 | 6 | 7)) => Ok(v),
        Ok(3) => Err("proxy protocol version 3 is no longer supported".into()),
        _ => Err(format!("expected 2, 4, 5, 6 or 7, got {value}")),
    }
}

fn parse_difficulty(value: &str) -> Result<legatum_core::AiDifficulty, String> {
    match value {
        "easy" => Ok(legatum_core::AiDifficulty::Easy),
        "normal" => Ok(legatum_core::AiDifficulty::Normal),
        "hard" => Ok(legatum_core::AiDifficulty::Hard),
        _ => Err("difficulty must be easy, normal, or hard".into()),
    }
}

fn parse_entropy(value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
        .ok_or_else(|| "entropy threshold must be finite and between 0 and 1".into())
}

type Subcommand = fn(Vec<String>) -> Result<(), String>;

fn main() -> ExitCode {
    // Scenario subcommands take their own arguments; everything else is the game runner.
    let args: Vec<String> = std::env::args().collect();
    let sub: Option<Subcommand> = match args.get(1).map(String::as_str) {
        Some("scenario") => Some(scenario::main),
        Some("card-loop") => Some(card_loop::main),
        Some("scenario-build") => Some(scenario_build::main),
        Some("scenario-check") => Some(scenario_check::main),
        Some("scenario-cut") => Some(scenario_cut::main),
        Some("scenario-reveal-migrate") => Some(scenario_reveal::main),
        Some("war-build") => Some(war_build::main),
        _ => None,
    };
    if let Some(sub) = sub {
        return match sub(args[1..].to_vec()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }
    let cli = Cli::parse();
    if let Some(log) = &cli.retrain {
        return retrain(
            log,
            cli.retrain_out.as_deref(),
            cli.training_budget,
            !cli.retrain_no_variants,
        );
    }
    if let Some(dir) = &cli.training {
        let _ = runner::TRAINING.set(runner::TrainingSim {
            dir: dir.clone(),
            cfg: legatum_ai::training::TrainingConfig {
                cap_ms: 0,
                max_rollouts_per_battle: cli.training_budget,
                source: "selfplay".into(),
                ..Default::default()
            },
        });
    }
    if cli.opening_schema {
        println!(
            "{}",
            serde_json::to_string_pretty(&legatum_ai::openings::OpeningPolicy::schema())
                .expect("schema JSON")
        );
        return ExitCode::SUCCESS;
    }
    if cli.print_ai_genes {
        println!(
            "{}",
            serde_json::to_string_pretty(&legatum_ai::DecisionParams::genes()).expect("gene JSON")
        );
        return ExitCode::SUCCESS;
    }
    if cli.print_ai_params {
        println!(
            "{}",
            serde_json::to_string_pretty(&legatum_ai::AiParams::builtin()).expect("params JSON")
        );
        return ExitCode::SUCCESS;
    }
    if let Some(path) = &cli.replay {
        if cli.analyze {
            return match legatum_archive::read(path)
                .and_then(|r| legatum_analysis::analyze_recording(&r, Default::default()))
                .and_then(|r| legatum_analysis::write_report(&r, &cli.out))
            {
                Ok(manifest) => {
                    println!("{manifest}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("Analysis failed: {error}");
                    ExitCode::FAILURE
                }
            };
        }
        return match legatum_archive::read(path)
            .and_then(|r| r.replay().map(|_| r.metadata.turn_count))
        {
            Ok(turns) => {
                println!("Compatible: {turns} turns replayed; all state hashes match");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Replay failed: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let threads = cli.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });
    let pool = match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("thread pool: {e}");
            return ExitCode::FAILURE;
        }
    };
    let result = pool.install(|| match (&cli.replay_export, &cli.tape_in) {
        (Some(input), _) => {
            replay::export_path(input, &cli.out, &sanitize(&cli.label)).map(|()| ExitCode::SUCCESS)
        }
        (_, Some(dir)) => replay_tapes(dir),
        _ => run(&cli, threads),
    });
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("legatum-sim: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Read and validate a tuned-parameter file. Leaked once: the process uses it until exit.
/// A parameter file, or the JSON itself when the argument starts with `{` (cloud batches carry
/// no input files).
fn load_params(path: &Path) -> Result<&'static legatum_ai::AiParams, String> {
    let inline = path.to_str().filter(|s| s.trim_start().starts_with('{'));
    let text = match inline {
        Some(json) => json.to_string(),
        None => fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?,
    };
    let params: legatum_ai::AiParams =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    params
        .validate()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Box::leak(Box::new(params)))
}

/// Keep labels filename-safe.
fn sanitize(label: &str) -> String {
    let s: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if s.is_empty() { "run".into() } else { s }
}

/// Write via a temp file + rename so readers never see a half-written file.
fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)
}

fn status_json(label: &str, total: usize, records: &[GameRecord], started_at: &str) -> String {
    let s = summarize(records);
    let rates: serde_json::Map<String, serde_json::Value> = s
        .by_faction
        .iter()
        .map(|(f, w)| (format!("{f:?}"), json!(w.rate)))
        .collect();
    json!({
        "label": label,
        "games_total": total,
        "games_done": records.len(),
        "running_win_rate_by_faction": rates,
        "proxy": s.proxy.as_ref().map(|p| json!({
            "wins": p.wins, "games": p.games, "win_rate": p.win_rate,
            "median_winning_turn": p.winning_turn.median,
        })),
        "avg_turns": s.avg_turns,
        "panics": s.panics,
        "stalled": s.outcomes.get(&stats::Outcome::Stalled).copied().unwrap_or(0),
        "started_at": started_at,
        "updated_at": clock::timestamp(clock::unix_secs()),
    })
    .to_string()
}

fn lobby_options(cli: &Cli) -> Result<lobby::Config, String> {
    let players = usize::from(cli.players);
    if cli.duel_matrix && players != 2 {
        return Err("--duel-matrix needs --players 2".into());
    }
    if !cli.factions.is_empty() && cli.factions.len() != players {
        return Err("--factions must name exactly --players factions in seat order".into());
    }
    // The Proxy benchmark and old eight-seat strategy schedule retain their calibration.
    if players != 8 && (cli.proxy.is_some() || cli.strategy_matchups) {
        return Err("--proxy and --strategy-matchups require --players 8".into());
    }
    if cli.mirror && cli.brain != BrainMode::Scripted {
        return Err("--mirror requires --brain scripted".into());
    }
    for value in &cli.seat_opening {
        let (seat, _) = value
            .split_once('=')
            .ok_or("expected <seat|all>=<Opening>")?;
        if seat != "all" && seat.parse::<usize>().map_or(true, |s| s >= players) {
            return Err(format!(
                "opening seat {seat} outside this {players}-player lobby"
            ));
        }
    }
    for seat in &cli.legacy_hard {
        if seat != "all" && seat.parse::<usize>().map_or(true, |s| s >= players) {
            return Err(format!(
                "legacy Hard seat {seat} outside this {players}-player lobby"
            ));
        }
    }
    let factions = (!cli.factions.is_empty()).then(|| {
        let mut factions = std::array::from_fn::<_, 8, _>(|i| legatum_core::Faction::ALL[i]);
        factions[..players].copy_from_slice(&cli.factions);
        factions
    });
    Ok(lobby::Config {
        players,
        map_size: cli.map_size,
        factions,
        rotate_factions: cli.rotate_factions,
        mirror: cli.mirror,
    })
}

fn run(cli: &Cli, threads: usize) -> Result<ExitCode, String> {
    if cli.hard_v2_handoff && cli.brain != BrainMode::Scripted {
        return Err("--hard-v2-handoff requires --brain scripted".into());
    }
    let lobby = lobby_options(cli)?;
    let games = cli
        .games
        .checked_mul(if cli.mirror { 2 } else { 1 })
        .ok_or("game count overflow")?;
    if cli.difficulty_rotation.is_some() && cli.brain != BrainMode::Scripted {
        return Err("--difficulty-rotation requires --brain scripted".into());
    }
    if cli.proxy.is_some() && cli.brain != BrainMode::Scripted {
        return Err("--proxy requires --brain scripted".into());
    }
    if cli.difficulty_rotation.is_some() && cli.difficulty != legatum_core::AiDifficulty::Normal {
        return Err("--difficulty conflicts with --difficulty-rotation".into());
    }
    let params = cli.ai_params.as_deref().map(load_params).transpose()?;
    let opponent = cli
        .opponent_params
        .as_deref()
        .map(load_params)
        .transpose()?;
    if (!cli.strategies.is_empty() || cli.strategy_matchups) && cli.brain != BrainMode::Scripted {
        return Err("strategy biases require --brain scripted".into());
    }
    if !cli.strategies.is_empty() && ![1, 2, 4, 8].contains(&cli.strategies.len()) {
        return Err(
            "--strategies needs 1, 2, 4 or 8 values; use --strategy-matchups for all five".into(),
        );
    }
    if cli.strategy_matchups && (cli.games == 0 || !cli.games.is_multiple_of(20)) {
        return Err("--strategy-matchups needs --games to be a positive multiple of 20".into());
    }
    if cli.diversity_check
        && !matches!(cli.diversity_basis, DiversityBasis::Observed)
        && cli.strategies.is_empty()
        && !cli.strategy_matchups
    {
        return Err("assigned diversity checks require --strategies or --strategy-matchups".into());
    }
    let openings = openings::Config {
        ai_primitives: cli.ai_primitives,
        seats: openings::seats(&cli.seat_opening)?,
        policies: Some(openings::genomes(&cli.opening_genome)?),
        legacy_hard: openings::legacy_seats(&cli.legacy_hard)?,
        hard_v2: [false; 8],
        hard_v2_rotation: cli.hard_v2_rotation,
        hard_v2_handoff: cli.hard_v2_handoff,
        challenger: cli
            .challenger
            .as_deref()
            .map(str::parse::<legatum_ai::openings::Opening>)
            .transpose()?,
    };
    let mut rule_experiments = legatum_core::experiments::RuleExperiments::default();
    for flag in &cli.rules_experiment {
        rule_experiments.apply(flag)?;
    }
    for rule in &cli.rule_off {
        rule_experiments.faction_rules_off.insert(*rule);
    }
    rule_experiments.hard_v2_ai |= cli.hard_v2;
    let label = sanitize(&cli.label);
    let started_secs = clock::unix_secs();
    let started_at = clock::timestamp(started_secs);
    let source = stats::SourceRevision::current();
    let archive = cli
        .archive
        .then(|| {
            legatum_archive::archive_root(cli.archive_dir.clone())
                .and_then(legatum_archive::Archive::open)
        })
        .transpose()?;
    let opts = RunOpts {
        lobby,
        barbarians: cli.barbarians.unwrap_or(cli.proxy.is_none()),
        safety_cap: cli.safety_cap,
        turn_limit: cli.turn_limit,
        brain: cli.brain,
        proxy: cli.proxy.map(|seat| proxy_benchmark::Config {
            seat,
            rotate: cli.rotate_proxy,
            difficulty: cli.difficulty,
            start: cli.proxy_start.unwrap_or(if cli.proxy_version == 2 {
                proxy_benchmark::ProxyStart::War
            } else {
                proxy_benchmark::ProxyStart::Peace
            }),
            version: cli.proxy_version,
            opening: cli.proxy_opening,
        }),
        difficulty_rotation: cli.difficulty_rotation,
        strategies: std::array::from_fn(|seat| {
            if cli.strategies.is_empty() {
                None
            } else {
                Some(cli.strategies[seat % cli.strategies.len()])
            }
        }),
        strategy_matchups: cli.strategy_matchups,
        rule_experiments,
        tuning: runner::Tuning {
            openings,
            params,
            opponent,
            difficulty: (cli.proxy.is_none()
                && cli.difficulty_rotation.is_none()
                && cli.difficulty != legatum_core::AiDifficulty::Normal)
                .then_some(cli.difficulty),
            search: cli.search,
            search_rotation: cli.search_rotation,
            duel_matrix: cli.duel_matrix,
            duel_opponent: cli.duel_opponent,
        },
        record: cli.archive
            || cli.proxy.is_some()
            || cli.analyze
            || cli.determinism
            || cli.tape_out.is_some()
            || cli.export_replays.is_some_and(|n| n > 0),
        determinism: cli.determinism,
    };
    if let Some(dir) = &cli.tape_out {
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    if let Some(dir) = &cli.decision_trace {
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    eprintln!(
        "legatum-sim: {} games from seed {}, {} threads, brain {}{}",
        games,
        cli.seed,
        threads,
        if cli.proxy.is_some() {
            "proxy-vs-ai"
        } else {
            cli.brain.name()
        },
        if cli.determinism { ", determinism" } else { "" }
    );

    let done: Mutex<Vec<GameRecord>> = Mutex::new(Vec::with_capacity(games));
    let finished = AtomicBool::new(false);
    let start = Instant::now();
    let tape_errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let replay_tapes = Mutex::new(std::collections::BTreeMap::new());
    let analysis = Mutex::new(legatum_analysis::Aggregate::default());

    std::thread::scope(|scope| {
        if let Some(path) = &cli.status_file {
            let (done, finished, label, started_at) = (&done, &finished, &label, &started_at);
            scope.spawn(move || {
                let mut last = Instant::now() - Duration::from_secs(10);
                loop {
                    let stop = finished.load(Ordering::Acquire);
                    if stop || last.elapsed() >= Duration::from_secs(2) {
                        let body = {
                            let recs = done.lock().unwrap_or_else(|p| p.into_inner());
                            status_json(label, games, &recs, started_at)
                        };
                        if let Err(e) = write_atomic(path, &body) {
                            eprintln!("status file {}: {e}", path.display());
                        }
                        last = Instant::now();
                    }
                    if stop {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });
        }

        (0..games).into_par_iter().for_each(|i| {
            let (mut rec, tape, archive_error) = if let Some(dir) = cli.decision_trace.as_deref() {
                runner::run_game_archived_with_trace(
                    i,
                    cli.seed,
                    &opts,
                    archive.as_ref(),
                    Some(dir),
                )
            } else {
                runner::run_game_archived(i, cli.seed, &opts, archive.as_ref())
            };
            if let Some(error) = archive_error {
                tape_errors
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push(format!("archive game {i}: {error}"));
            }
            if let (Some(dir), Some(tape)) = (&cli.tape_out, &tape) {
                let path = dir.join(format!("game-{i:05}-seed-{}.json", rec.seed));
                if let Err(e) = fs::write(&path, tape.to_json()) {
                    tape_errors
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .push(format!("{}: {e}", path.display()));
                }
            }
            if cli.analyze || cli.proxy.is_some() {
                let result = tape
                    .as_ref()
                    .ok_or_else(|| "missing recorded tape".to_owned())
                    .and_then(|t| {
                        legatum_analysis::analyze_tape(
                            t,
                            &format!("sim-{i}-{}", rec.seed),
                            Default::default(),
                        )
                    });
                match result {
                    Ok(mut report) => {
                        if let Some(proxy) = rec.telemetry.as_mut().and_then(|t| t.proxy.as_mut()) {
                            proxy.observe_incidents(&report);
                            // The existing aggregate's AI denominator must exclude the Proxy.
                            // Factions are unique in these lobbies; no replay state is changed.
                            report.rates.retain(|r| r.faction != proxy.faction);
                            report.seats.retain(|s| s.player != proxy.seat);
                        }
                        analysis
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .add(&report);
                    }
                    Err(error) => tape_errors
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .push(format!("analyze game {i}: {error}")),
                }
            }
            if cli.export_replays.is_some_and(|n| n > 0)
                && let Some(tape) = tape
            {
                replay_tapes
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .insert(i, tape);
            }
            let mut recs = done.lock().unwrap_or_else(|p| p.into_inner());
            recs.push(rec);
            let n = recs.len();
            if n.is_multiple_of(10) || n == games {
                eprintln!("  {n}/{games} games");
            }
        });
        finished.store(true, Ordering::Release);
    });

    let wall_ms = start.elapsed().as_millis() as u64;
    let mut records = done.into_inner().unwrap_or_else(|p| p.into_inner());
    records.sort_by_key(|r| r.index);
    if let Some(count) = cli.export_replays.filter(|n| *n > 0) {
        let mut tapes = replay_tapes.into_inner().unwrap_or_else(|p| p.into_inner());
        let mut exports = Vec::new();
        for (index, why) in replay::select(&records, count) {
            let tape = tapes
                .remove(&index)
                .ok_or_else(|| format!("missing tape for game {index}"))?;
            let mut replay = replay::from_tape(&tape, &label)?;
            replay.header.game_index = Some(index);
            exports.push((replay, why));
        }
        replay::write_replays(&cli.out, &exports)?;
        eprintln!(
            "exported {} replay(s) to {}",
            exports.len(),
            cli.out.display()
        );
    }
    let mut summary = summarize(&records);
    summary.diversity.check(GuardrailConfig {
        min_entropy: cli.diversity_min_entropy,
        min_matchup_games: cli.diversity_min_matchup_games,
    });
    let meta = RunMeta {
        label: label.clone(),
        date: clock::date(started_secs),
        started_at,
        games,
        first_seed: cli.seed.to_string(),
        threads,
        brain: if cli.proxy.is_some() {
            "proxy-vs-ai"
        } else {
            cli.brain.name()
        }
        .into(),
        proxy: opts.proxy,
        proxy_version: opts
            .proxy
            .filter(|p| p.opening.is_none())
            .map(|p| p.version),
        difficulty_rotation: cli.difficulty_rotation,
        strategies: cli.strategies.clone(),
        strategy_matchups: cli.strategy_matchups,
        safety_cap: opts.safety_cap(),
        turn_limit: cli.turn_limit,
        determinism: cli.determinism,
        wall_ms,
        source,
        command: std::env::args().collect(),
        first_setup: runner::setup_for(cli.seed, &opts),
    };

    fs::create_dir_all(&cli.out_dir)
        .map_err(|e| format!("create {}: {e}", cli.out_dir.display()))?;
    let base = cli.out_dir.join(format!("{}-{}", meta.date, label));
    let md_path = base.with_extension("md");
    let json_path = base.with_extension("json");
    let analysis = (cli.analyze || cli.proxy.is_some())
        .then(|| analysis.into_inner().unwrap_or_else(|p| p.into_inner()));
    let mut markdown = stats::render_markdown(&meta, &summary, &records);
    if let Some(analysis) = &analysis {
        markdown.push_str(&analysis.markdown());
    }
    fs::write(&md_path, markdown).map_err(|e| format!("write {}: {e}", md_path.display()))?;
    let raw = json!({
        "meta": meta, "summary": summary, "games": records,
        "metric_definitions": telemetry::DEFINITIONS,
        "proxy_metric_definitions": proxy_benchmark::DEFINITIONS,
        "ai_analysis": analysis,
        "diversity_classifier_version": legatum_sim::diversity::CLASSIFIER_VERSION,
        "ai_params": { "params": params, "opponent": opponent },
        "opening_policies": openings.policies,
        "ai_primitives": cli.ai_primitives,
    });
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&raw).expect("records serialize"),
    )
    .map_err(|e| format!("write {}: {e}", json_path.display()))?;

    eprintln!(
        "done in {:.1}s: {} games, {} panics, avg {:.1} turns{}",
        wall_ms as f64 / 1000.0,
        summary.games,
        summary.panics,
        summary.avg_turns,
        if cli.determinism {
            format!(
                ", determinism {}/{} ok",
                summary.determinism_checked - summary.determinism_failures,
                summary.determinism_checked
            )
        } else {
            String::new()
        }
    );
    let holds = &summary.victory_holds;
    if holds.observed_games > 0 {
        eprintln!(
            "VP holds: {} started, {} broken ({:.1}%), {} completed, {} unfinished; {} hold wins, \
             {} comebacks, {} stalled",
            holds.started,
            holds.broken,
            100.0 * holds.break_rate,
            holds.completed,
            holds.unfinished,
            holds.hold_wins,
            holds.comebacks,
            summary
                .outcomes
                .get(&stats::Outcome::Stalled)
                .copied()
                .unwrap_or(0),
        );
    }
    eprintln!("wrote {}", md_path.display());
    eprintln!("wrote {}", json_path.display());
    let tape_errors = tape_errors.into_inner().unwrap_or_else(|p| p.into_inner());
    for e in &tape_errors {
        eprintln!("tape write failed: {e}");
    }
    if !summary.telemetry.scores_reconcile() {
        eprintln!("terminal VP sources failed to reconcile; see report");
    }
    let ok = summary.panics == 0
        && summary.determinism_failures == 0
        && summary.telemetry.scores_reconcile()
        && tape_errors.is_empty();
    if ok && cli.diversity_check {
        let checks: Vec<_> = match cli.diversity_basis {
            DiversityBasis::Observed => summary.diversity.observed_check.iter().collect(),
            DiversityBasis::Assigned => summary.diversity.assigned_check.iter().collect(),
            DiversityBasis::Both => summary
                .diversity
                .observed_check
                .iter()
                .chain(summary.diversity.assigned_check.iter())
                .collect(),
        };
        for c in &checks {
            eprintln!("diversity {:?}: {}", c.status, c.reasons.join("; "));
        }
        if checks.iter().any(|c| c.status == CheckStatus::Fail) {
            return Ok(ExitCode::FAILURE);
        }
        if checks.is_empty()
            || checks
                .iter()
                .any(|c| c.status == CheckStatus::InsufficientData)
        {
            return Ok(ExitCode::from(2));
        }
    }
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn replay_tapes(dir: &Path) -> Result<ExitCode, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no .json tapes in {}", dir.display()));
    }
    let results: Vec<(PathBuf, Result<usize, String>)> = paths
        .into_par_iter()
        .map(|p| {
            let r = fs::read_to_string(&p)
                .map_err(|e| e.to_string())
                .and_then(|s| runner::check_tape(&s));
            (p, r)
        })
        .collect();
    let mut failed = 0;
    for (p, r) in &results {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        match r {
            Ok(turns) => println!("ok    {name} ({turns} turns)"),
            Err(e) => {
                failed += 1;
                println!("FAIL  {name}: {e}");
            }
        }
    }
    if failed == 0 {
        println!(
            "PASS: {} tapes replayed natively, all hashes match (initial state, every turn, and save/load)",
            results.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        println!("FAIL: {failed} of {} tapes diverged", results.len());
        Ok(ExitCode::FAILURE)
    }
}

/// `--retrain LOG`: search every logged battle again, at full width, with variants.
fn retrain(
    log: &std::path::Path,
    out: Option<&std::path::Path>,
    budget: u64,
    variants: bool,
) -> ExitCode {
    use std::io::{BufRead, Write};
    let out = out
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| log.with_extension("retrain.jsonl"));
    let cfg = legatum_ai::training::TrainingConfig {
        cap_ms: 0,
        max_rollouts_per_battle: budget,
        source: "retrain".into(),
        ..Default::default()
    };
    let input = match std::fs::File::open(log) {
        Ok(f) => std::io::BufReader::new(f),
        Err(e) => {
            eprintln!("cannot read {}: {e}", log.display());
            return ExitCode::FAILURE;
        }
    };
    let mut output =
        std::io::BufWriter::new(std::fs::File::create(&out).expect("create retrain output"));
    let (mut battles, mut written, mut failed) = (0, 0, 0);
    for line in input
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
    {
        match legatum_ai::training::retrain_line(&line, &cfg, variants) {
            Ok(records) => {
                battles += usize::from(!records.is_empty());
                for r in records {
                    let line = legatum_ai::training::LogLine::Decision(Box::new(r)).json();
                    writeln!(output, "{line}").expect("write retrain output");
                    written += 1;
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("skipped a line: {e}");
            }
        }
    }
    output.flush().expect("flush retrain output");
    eprintln!(
        "retrained {battles} battles into {written} lines ({failed} skipped): {}",
        out.display()
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    #[test]
    fn threads_must_be_at_least_one() {
        use super::*;
        assert!(Cli::try_parse_from(["sim", "--threads", "0"]).is_err());
        assert_eq!(
            Cli::try_parse_from(["sim", "--threads", "2"])
                .unwrap()
                .threads,
            Some(2)
        );
    }

    #[test]
    fn duel_cli_validates_lobby_and_faction_assignments() {
        use super::*;
        let default = Cli::try_parse_from(["sim"]).unwrap();
        assert_eq!(lobby_options(&default).unwrap().players, 8);
        assert_eq!(default.map_size, lobby::MapSize::Medium);
        assert!(!default.mirror);
        for players in 2..=8 {
            for size in ["small", "medium", "large"] {
                let cli = Cli::try_parse_from([
                    "sim",
                    "--players",
                    &players.to_string(),
                    "--map-size",
                    size,
                ])
                .unwrap();
                assert_eq!(lobby_options(&cli).unwrap().players, players as usize);
            }
        }
        let cli = Cli::try_parse_from([
            "sim",
            "--players",
            "2",
            "--mirror",
            "--factions",
            "India,Celts",
        ])
        .unwrap();
        assert_eq!(
            &lobby_options(&cli).unwrap().factions.unwrap()[..2],
            &[legatum_core::Faction::India, legatum_core::Faction::Celts]
        );
        for args in [
            vec!["sim", "--players", "1"],
            vec!["sim", "--players", "9"],
            vec!["sim", "--map-size", "huge"],
            vec!["sim", "--factions", "unknown,India"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
        for args in [
            vec!["sim", "--players", "2", "--factions", "India"],
            vec!["sim", "--players", "2", "--seat-opening", "2=Rush"],
            vec!["sim", "--players", "2", "--legacy-hard", "3"],
            vec!["sim", "--players", "2", "--proxy", "0"],
            vec!["sim", "--mirror", "--brain", "mixed"],
        ] {
            assert!(lobby_options(&Cli::try_parse_from(args).unwrap()).is_err());
        }
    }

    #[test]
    fn rule_off_takes_snake_case_names_and_an_unknown_one_lists_the_valid_ones() {
        let cli =
            super::Cli::try_parse_from(["legatum-sim", "--rule-off", "germany_ruhr_production"])
                .unwrap();
        assert_eq!(
            cli.rule_off,
            [legatum_core::experiments::FactionRule::GermanyRuhrProduction]
        );
        let err = super::Cli::try_parse_from(["legatum-sim", "--rule-off", "germany_nope"])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("germany_nope") && err.contains("germany_goal"),
            "{err}"
        );
    }

    #[test]
    fn opening_cli_exports_schema_and_validates_genomes_before_play() {
        let cli = super::Cli::try_parse_from([
            "legatum-sim",
            "--seat-opening",
            "all=Rush",
            "--seat-opening",
            "1=Economic",
            "--rules-experiment",
            "objective-scale=125,start-settlers=1",
        ])
        .unwrap();
        assert_eq!(
            super::openings::seats(&cli.seat_opening).unwrap()[1],
            legatum_ai::openings::Opening::Economic
        );
        assert_eq!(cli.rules_experiment.len(), 2);
        assert!(
            super::Cli::try_parse_from(["legatum-sim", "--opening-schema"])
                .unwrap()
                .opening_schema
        );
        for flag in ["tech-cost-percent=101", "start-settlers=9", "unknown"] {
            assert!(
                super::Cli::try_parse_from(["legatum-sim", "--rules-experiment", flag]).is_err()
            );
        }
        let dir =
            std::env::temp_dir().join(format!("legatum-opening-genome-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rush.json");
        let genome =
            legatum_ai::openings::OpeningPolicy::builtin(legatum_ai::openings::Opening::Rush)
                .genome();
        std::fs::write(&path, serde_json::to_string(&genome).unwrap()).unwrap();
        let argument = format!("Rush={}", path.display());
        let policies = super::openings::genomes(std::slice::from_ref(&argument)).unwrap();
        assert_eq!(
            policies[&legatum_ai::openings::Opening::Rush].genome(),
            genome
        );
        assert!(super::openings::genomes(&[argument.clone(), argument.clone()]).is_err());
        std::fs::write(&path, "[0.5]").unwrap();
        assert!(super::openings::genomes(&[argument]).is_err());
    }

    use super::*;

    #[test]
    fn cli_parses_all_flags() {
        let cli = Cli::try_parse_from([
            "legatum-sim",
            "--games",
            "50",
            "--seed",
            "18446744073709551615",
            "--threads",
            "4",
            "--label",
            "balance v1",
            "--turn-limit",
            "80",
            "--determinism",
            "--brain",
            "mixed",
            "--tape-out",
            "tapes",
            "--status-file",
            "s.json",
        ])
        .unwrap();
        assert_eq!(cli.games, 50);
        assert_eq!(cli.seed, u64::MAX);
        assert_eq!(cli.threads, Some(4));
        assert_eq!(cli.turn_limit, Some(80));
        assert!(cli.determinism);
        assert_eq!(cli.brain, BrainMode::Mixed);
        assert_eq!(cli.tape_out, Some(PathBuf::from("tapes")));
        assert_eq!(cli.status_file, Some(PathBuf::from("s.json")));
        assert_eq!(sanitize(&cli.label), "balance-v1");
    }

    #[test]
    fn cli_defaults() {
        let cli = Cli::try_parse_from(["legatum-sim"]).unwrap();
        assert_eq!((cli.games, cli.seed), (10, 1));
        assert_eq!(cli.brain, BrainMode::Scripted);
        assert!(!cli.hard_v2_handoff);
        assert_eq!(cli.difficulty_rotation, None);
        assert_eq!((cli.turn_limit, cli.safety_cap), (None, None));
        assert!(!cli.determinism && cli.tape_in.is_none() && cli.status_file.is_none());
        assert!(Cli::try_parse_from(["legatum-sim", "--brain", "genius"]).is_err());
    }

    #[test]
    fn cli_difficulty_rotation_is_a_three_way_offset() {
        for offset in 0..3 {
            let cli =
                Cli::try_parse_from(["legatum-sim", "--difficulty-rotation", &offset.to_string()])
                    .unwrap();
            assert_eq!(cli.difficulty_rotation, Some(offset));
        }
        for bad in ["3", "255", "-1", "hard"] {
            assert!(Cli::try_parse_from(["legatum-sim", "--difficulty-rotation", bad]).is_err());
        }
        let cli = Cli::try_parse_from([
            "legatum-sim",
            "--difficulty-rotation",
            "0",
            "--brain",
            "random",
        ])
        .unwrap();
        assert!(
            run(&cli, 1)
                .unwrap_err()
                .contains("requires --brain scripted")
        );
    }

    #[test]
    fn safety_cap_is_separate_from_the_optional_limit() {
        let cli = Cli::try_parse_from(["legatum-sim", "--safety-cap", "77"]).unwrap();
        assert_eq!((cli.turn_limit, cli.safety_cap), (None, Some(77)));
        for args in [
            ["--turn-limit", "0"],
            ["--safety-cap", "0"],
            ["--turn-limit", "4294967295"],
        ] {
            assert!(Cli::try_parse_from(["legatum-sim", args[0], args[1]]).is_err());
        }
    }

    #[test]
    fn cli_replay_modes() {
        let cli = Cli::try_parse_from([
            "legatum-sim",
            "--replay-export",
            "tape.json",
            "--out",
            "replays",
        ])
        .unwrap();
        assert_eq!(cli.replay_export, Some(PathBuf::from("tape.json")));
        assert_eq!(cli.out, PathBuf::from("replays"));
        let cli = Cli::try_parse_from(["legatum-sim", "--export-replays", "6"]).unwrap();
        assert_eq!(cli.export_replays, Some(6));
        assert!(
            Cli::try_parse_from([
                "legatum-sim",
                "--export-replays",
                "6",
                "--replay-export",
                "tapes"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "legatum-sim",
                "--tape-in",
                "tapes",
                "--replay-export",
                "tapes"
            ])
            .is_err()
        );
    }

    #[test]
    fn status_json_shape() {
        let v: serde_json::Value =
            serde_json::from_str(&status_json("x", 7, &[], "2026-09-26T00:00:00Z")).unwrap();
        for k in [
            "label",
            "games_total",
            "games_done",
            "running_win_rate_by_faction",
            "avg_turns",
            "panics",
            "started_at",
            "updated_at",
        ] {
            assert!(v.get(k).is_some(), "missing {k}");
        }
        assert_eq!(v["games_total"], 7);
        assert_eq!(v["games_done"], 0);
    }

    #[test]
    fn cli_ai_params_are_validated_before_any_game() {
        let dir = std::env::temp_dir().join(format!("legatum-params-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let good = dir.join("good.json");
        fs::write(&good, r#"{"schema":1,"name":"cand-1"}"#).unwrap();
        let bad = dir.join("bad.json");
        fs::write(&bad, r#"{"schema":1,"name":"x","yields":{}}"#).unwrap();
        assert_eq!(load_params(&good).unwrap().name, "cand-1");
        assert!(load_params(&bad).unwrap_err().contains("yields"));
        let cli = Cli::try_parse_from([
            "legatum-sim",
            "--ai-params",
            good.to_str().unwrap(),
            "--opponent-params",
            good.to_str().unwrap(),
            "--difficulty",
            "hard",
        ])
        .unwrap();
        assert!(cli.ai_params.is_some() && cli.opponent_params.is_some());
        assert!(
            Cli::try_parse_from(["legatum-sim", "--proxy", "0", "--opponent-params", "x.json"])
                .is_err()
        );
        let cli =
            Cli::try_parse_from(["legatum-sim", "--ai-params", bad.to_str().unwrap()]).unwrap();
        assert!(run(&cli, 1).unwrap_err().contains("yields"));
    }

    #[test]
    fn cli_proxy_seat_rotation_and_difficulty() {
        let cli = Cli::try_parse_from([
            "legatum-sim",
            "--proxy",
            "7",
            "--rotate-proxy",
            "--difficulty",
            "hard",
        ])
        .unwrap();
        assert_eq!(cli.proxy, Some(7));
        assert!(cli.rotate_proxy);
        assert_eq!(cli.difficulty, legatum_core::AiDifficulty::Hard);
        assert_eq!(cli.proxy_version, 4);
        assert_eq!(cli.proxy_start, None);
        let peace =
            Cli::try_parse_from(["legatum-sim", "--proxy", "0", "--proxy-start", "peace"]).unwrap();
        assert_eq!(peace.proxy_start, Some(proxy_benchmark::ProxyStart::Peace));
        for args in [
            vec!["legatum-sim", "--proxy-version", "2"],
            vec!["legatum-sim", "--proxy", "0", "--proxy-version", "1"],
            vec!["legatum-sim", "--proxy", "0", "--proxy-version", "8"],
            vec!["legatum-sim", "--proxy-start", "war"],
            vec!["legatum-sim", "--proxy", "8"],
            vec!["legatum-sim", "--rotate-proxy"],
            vec!["legatum-sim", "--proxy", "0", "--difficulty-rotation", "1"],
            vec!["legatum-sim", "--proxy", "0", "--difficulty", "impossible"],
            vec!["legatum-sim", "--proxy", "0", "--strategies", "wide"],
            vec!["legatum-sim", "--proxy", "0", "--strategy-matchups"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }
}

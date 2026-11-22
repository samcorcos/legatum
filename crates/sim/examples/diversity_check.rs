//! Recheck saved measurements without replaying games. Demonstrates the public library API.
use clap::Parser;
use legatum_sim::diversity::{
    CheckStatus, DEFAULT_MIN_ENTROPY, DEFAULT_MIN_MATCHUP_GAMES, DiversitySummary, GuardrailConfig,
    check_diversity, render_markdown,
};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
struct Args {
    /// Plain sim report JSON (decompress .json.gz first).
    report: PathBuf,
    #[arg(long, default_value_t = DEFAULT_MIN_ENTROPY)]
    min_entropy: f64,
    #[arg(long, default_value_t = DEFAULT_MIN_MATCHUP_GAMES)]
    min_matchup_games: u64,
    /// Optional standalone Markdown with the newly evaluated diversity sections.
    #[arg(long)]
    markdown: Option<PathBuf>,
}

fn run(args: Args) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(args.report)?)?;
    let config = GuardrailConfig {
        min_entropy: args.min_entropy,
        min_matchup_games: args.min_matchup_games,
    };
    let mut checks = serde_json::Map::new();
    let mut markdown = String::new();
    let mut failed = false;
    let mut insufficient = false;
    for (basis, label) in [
        ("observed", "observed behavior (classifier v2)"),
        ("assigned", "assigned scripted biases"),
    ] {
        let value = &report["summary"]["diversity"][basis];
        if value.is_null() {
            continue;
        }
        let summary: DiversitySummary = serde_json::from_value(value.clone())?;
        let check = check_diversity(&summary, config);
        failed |= check.status == CheckStatus::Fail;
        insufficient |= check.status == CheckStatus::InsufficientData;
        markdown += &render_markdown(&summary, &check, label);
        checks.insert(format!("{basis}_check"), serde_json::to_value(check)?);
    }
    if checks.is_empty() {
        return Err("report contains no diversity measurements".into());
    }
    if let Some(path) = args.markdown {
        std::fs::write(path, markdown)?;
    }
    println!("{}", serde_json::to_string_pretty(&checks)?);
    Ok(if failed {
        ExitCode::FAILURE
    } else if insufficient {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    })
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("diversity check: {e}");
            ExitCode::FAILURE
        }
    }
}

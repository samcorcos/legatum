use clap::Parser;
use legatum_analysis::{Config, analyze_recording, scenario, write_report};
use std::{fs, path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "legatum-analyze",
    about = "Replay a verified archive and diagnose AI behavior"
)]
struct Cli {
    /// .tape.json.zst, .tape.json.gz, or archive envelope JSON.
    #[arg(
        required_unless_present_any = ["write_repro", "engine_info"],
        conflicts_with_all = ["write_repro", "engine_info"]
    )]
    archive: Option<PathBuf>,
    #[arg(long, default_value = ".reviews/ai-analysis")]
    out: PathBuf,
    #[arg(long, default_value_t = 12, value_parser = clap::value_parser!(u32).range(1..))]
    settler_turns: u32,
    /// Create the deterministic Ranger/scout/garrison reproduction with the compiled rules.
    #[arg(long)]
    write_repro: Option<PathBuf>,
    /// Print the compiled engine identity without reading or replaying an archive.
    #[arg(long, conflicts_with = "write_repro")]
    engine_info: bool,
}
fn main() -> ExitCode {
    let cli = Cli::parse();
    if cli.engine_info {
        println!(
            "{}",
            serde_json::json!({
                "rules_fingerprint": legatum_wasm::history::RULES_FINGERPRINT,
                "engine_commit": legatum_wasm::history::ENGINE_COMMIT,
            })
        );
        return ExitCode::SUCCESS;
    }
    let result = (|| {
        let recording = if let Some(path) = cli.write_repro {
            let recording = scenario::ranger_repro();
            let json = serde_json::to_string_pretty(&recording).map_err(|e| e.to_string())?;
            fs::write(path, json).map_err(|e| e.to_string())?;
            recording
        } else {
            legatum_archive::read(cli.archive.as_deref().expect("clap requires archive"))?
        };
        let report = analyze_recording(
            &recording,
            Config {
                settler_turns: cli.settler_turns,
            },
        )?;
        write_report(&report, &cli.out)
    })();
    match result {
        Ok(manifest) => {
            println!("{manifest}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("legatum-analyze: {error}");
            ExitCode::FAILURE
        }
    }
}

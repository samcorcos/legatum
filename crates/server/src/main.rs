use clap::Parser;
use legatum_server::{RelayConfig, RoomAnnounce, Server, lan_urls};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[command(about = "Host a Legatum game for browsers on your LAN")]
struct Args {
    #[arg(long, default_value_t = 7878)]
    port: u16,
    #[arg(long = "static", default_value = "app/dist")]
    static_dir: PathBuf,
    /// Saves folder (default: the platform data folder's `saves`, for example
    /// ~/.local/share/legatum/saves on Linux).
    #[arg(long)]
    saves: Option<PathBuf>,
    /// Override LEGATUM_ARCHIVE / ~/LegatumData/archive (not inside the Legatum source checkout).
    #[arg(long)]
    archive_dir: Option<PathBuf>,
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u32).range(1..))]
    archive_checkpoint_turns: u32,
    /// Also open an online room on this relay (e.g. https://legatum-relay.example.workers.dev).
    #[arg(long, requires_all = ["name", "build"])]
    relay: Option<String>,
    /// The host name shown in the online lobby (with --relay).
    #[arg(long, requires = "relay")]
    name: Option<String>,
    /// The build label shown in the online lobby; guests need the same one (with --relay).
    #[arg(long, requires = "relay")]
    build: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let saves_dir = match args.saves {
        Some(path) => path,
        None => legatum_archive::data_dir()
            .ok_or("no home folder found; pass --saves <dir>")?
            .join("saves"),
    };
    if !args.static_dir.join("index.html").is_file() {
        return Err(format!(
            "Missing {}/index.html. Run: cd app && npm run build",
            args.static_dir.display()
        )
        .into());
    }
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", args.port)).await?;
    let port = listener.local_addr()?.port();
    let urls = lan_urls(port)?;
    // The archive is a record of finished games; without it the game still runs.
    let archive_dir = legatum_archive::archive_root(args.archive_dir)
        .inspect_err(|e| eprintln!("Game archive off: {e}"))
        .ok();
    let server = Server::open_with_archive(
        saves_dir.clone(),
        port,
        urls.clone(),
        archive_dir.clone(),
        args.archive_checkpoint_turns,
    )?;
    if let Some(dir) = &archive_dir {
        println!("Archive: {}", dir.display());
    }
    println!("Host: http://localhost:{port}");
    for url in urls {
        println!("Share on your LAN: {url}");
    }
    println!("Saves: {}", saves_dir.display());
    let relay = match (args.relay, args.name, args.build) {
        (Some(url), Some(name), Some(build)) => {
            let relay = server
                .run_relay(RelayConfig {
                    url,
                    announce: RoomAnnounce::new(name, build),
                })
                .await?;
            println!("Online room: {}", relay.room());
            Some(relay)
        }
        _ => None,
    };
    axum::serve(
        listener,
        server
            .router(args.static_dir)
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    if let Some(relay) = relay {
        relay.stop().await;
    }
    Ok(())
}

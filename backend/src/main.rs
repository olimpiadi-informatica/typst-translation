use std::path::PathBuf;

use backend::config::AppConfig;
use backend::{AppState, init_logging};
use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // ignore missing .env files.
    let _ = dotenvy::dotenv();

    init_logging();

    let cli = Args::parse();

    let database_url = std::env::var("DATABASE_URL").unwrap_or("./db.sqlite".to_string());

    let config = AppConfig::from_file(&cli.config_path)?;

    let state = AppState::new(config.clone(), &database_url).await?;
    state
        .clone()
        .serve(state.config().listen_address.parse()?)
        .await?;

    Ok(())
}

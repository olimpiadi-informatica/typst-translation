use std::path::PathBuf;

use backend::AppState;
use backend::config::AppConfig;
use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let db = backend::init().await?;

    let config = AppConfig::from_file(&cli.config_path)?;

    let state = AppState::new(config.clone(), db).await?;
    state
        .clone()
        .serve(state.config().listen_address.parse()?)
        .await?;

    Ok(())
}

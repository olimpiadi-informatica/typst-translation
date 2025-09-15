use std::net::SocketAddr;

use backend::{AppState, init_logging};
use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:3000")]
    listen_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // ignore missing .env files.
    let _ = dotenvy::dotenv();

    init_logging();

    let cli = Args::parse();

    let database_url = std::env::var("DATABASE_URL").unwrap_or("./db.sqlite".to_string());

    let state = AppState::new(&database_url).await?;
    state.clone().serve(cli.listen_address).await?;

    Ok(())
}

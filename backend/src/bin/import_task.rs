use std::path::PathBuf;

use backend::db_ops::task_db;
use clap::Parser;
use color_eyre::eyre::Result;
use sqlx::query;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    statement_zip: PathBuf,
    #[clap(long)]
    contest: String,
    #[clap(long, default_value_t = false)]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();
    let db = backend::init().await?;

    let contest_id = query!("SELECT id FROM contests WHERE name = ?", cli.contest)
        .fetch_one(&db)
        .await?
        .id;

    let zip_file = std::fs::read(&cli.statement_zip)?;
    task_db::import_task(&db, contest_id, cli.update, zip_file).await?;

    Ok(())
}

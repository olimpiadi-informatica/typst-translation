use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, PathBuf};

use backend::db_ops::language_db;
use backend::file_storage::save_file;
use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::{OptionExt, Result, bail};
use sqlx::query;
use sqlx::types::Json;

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

    // TODO(veluca): make this into actual web endpoints for admins to use.

    let file = File::open(cli.statement_zip)?;
    let mut zip = zip::ZipArchive::new(file)?;

    let mut task_name = None;

    for f in 0..zip.len() {
        let entry = zip.by_index(f)?;
        let Some(name) = entry.enclosed_name() else {
            continue;
        };

        if name.components().count() <= 1 {
            continue;
        }

        let Some(Component::Normal(dir)) = name.components().next() else {
            bail!("non-relative path in zipfile: {name:?}");
        };

        let dir = dir.to_string_lossy();

        if task_name.as_ref().is_some_and(|x: &String| x != &dir) {
            bail!("multiple directories in zipfile: {task_name:?} and {name:?}");
        }

        task_name = Some(dir.to_string());
    }

    let mut tx = db.begin().await?;

    let contest_id = query!("SELECT id FROM contests WHERE name = ?", cli.contest)
        .fetch_one(&mut *tx)
        .await?
        .id;

    let task_name = task_name.ok_or_eyre("No task folder found")?;

    let task = query!(
        "SELECT * FROM tasks WHERE name = ? AND contest_id = ?",
        task_name,
        contest_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let task_id = if let Some(task) = task {
        if !cli.update {
            bail!("task {task_name} already present and --update not given");
        } else {
            task.id
        }
    } else {
        let id = query!(
            "INSERT INTO tasks(contest_id, name) VALUES (?, ?) RETURNING id;",
            contest_id,
            task_name
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        for language in language_db::get_all(&mut *tx).await? {
            query!(
                "INSERT INTO translations(task_id, language_id) VALUES (?, ?)",
                id,
                language.id
            )
            .execute(&mut *tx)
            .await?;
        }

        id
    };

    let mut files = HashMap::new();

    for f in 0..zip.len() {
        let mut entry = zip.by_index(f)?;
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let mut data = vec![];
        entry.read_to_end(&mut data)?;
        let hash = save_file("files", &data).await?;
        files.insert(name.to_string_lossy().to_string(), hash);
    }

    let content_manifest = Json(files);

    let date = Utc::now().naive_utc();

    query!(
        r#"
        INSERT INTO statement_versions(task_id, content_manifest, is_live, created_at)
        VALUES (?, ?, ?, ?)
        "#,
        task_id,
        content_manifest,
        true,
        date
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

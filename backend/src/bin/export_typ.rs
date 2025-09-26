use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;
use sqlx::query;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    contest: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = backend::init().await?;

    let mut tx = db.begin().await?;

    let translations = query!(
        r#"
        SELECT languages.code, content_hash AS "content_hash!", tasks.name
        FROM translations
          JOIN languages ON translations.language_id = languages.id
          JOIN tasks ON translations.task_id = tasks.id
        WHERE content_hash IS NOT NULL
        "#
    )
    .fetch_all(&mut *tx)
    .await?;

    for translation in translations {
        let in_path = backend::file_storage::path_of_file(&translation.content_hash)?;
        let out_path = PathBuf::from(format!("out/{}/{}.typ", translation.name, translation.code));
        std::fs::create_dir_all(out_path.parent().unwrap())?;
        std::fs::copy(&in_path, &out_path)?;
    }

    Ok(())
}

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

    let languages = query!(
        r#"
        SELECT id, code
        FROM languages
        "#
    )
    .fetch_all(&mut *tx)
    .await?;

    for lang in languages {
        let contestants = query!(
            r#"
            SELECT code
            FROM contestants
            WHERE language_id = ?
            "#,
            lang.id
        )
        .fetch_all(&mut *tx)
        .await?;

        if contestants.is_empty() {
            continue;
        }

        println!("Language: {}    Count: {}", lang.code, contestants.len());
        for contestant in contestants {
            println!("  {}", contestant.code);
        }
    }

    Ok(())
}

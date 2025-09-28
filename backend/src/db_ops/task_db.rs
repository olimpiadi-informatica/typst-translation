use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Component;

use chrono::Utc;
use common::error::Error;
use common::task::TaskDb;
use sqlx::types::Json;
use sqlx::{Executor, Sqlite, query};
use zip::ZipArchive;

use crate::db_ops::language_db;
use crate::file_storage::save_file;

pub async fn get_task_by_id<'e, E>(executor: E, id: i64) -> Result<TaskDb, Error>
where
    E: Executor<'e, Database = Sqlite> + Clone,
{
    sqlx::query_as!(TaskDb, "SELECT * FROM tasks WHERE id = ?", id)
        .fetch_optional(executor.clone())
        .await?
        .ok_or(Error::NotFound)
}

pub(crate) async fn import_task(
    db: &sqlx::Pool<Sqlite>,
    contest_id: i64,
    update: bool,
    zip_file: Vec<u8>,
) -> Result<(), Error> {
    let mut zip = ZipArchive::new(Cursor::new(zip_file))?;

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
            return Err(Error::InvalidInput(format!(
                "non-relative path in zipfile: {name:?}"
            )));
        };

        let dir = dir.to_string_lossy();

        if task_name.as_ref().is_some_and(|x: &String| x != &dir) {
            return Err(Error::InvalidInput(format!(
                "multiple directories in zipfile: {task_name:?} and {name:?}"
            )));
        }

        task_name = Some(dir.to_string());
    }

    let mut tx = db.begin().await?;

    let Some(task_name) = task_name else {
        return Err(Error::InvalidInput("No task folder found".to_string()));
    };

    let task = query!(
        "SELECT * FROM tasks WHERE name = ? AND contest_id = ?",
        task_name,
        contest_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let task_id = if let Some(task) = task {
        if !update {
            return Err(Error::InvalidInput(format!(
                "task {task_name} already present and update not given"
            )));
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
        let hash = save_file(&data).await?;
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

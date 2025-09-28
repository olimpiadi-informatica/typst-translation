use std::collections::HashMap;

use common::error::Error;
use common::statement_version::StatementVersion;
use sqlx::types::Json;
use sqlx::{Executor, Sqlite};

pub async fn get_latest_statement_version_by_task_id<'e, E>(
    executor: E,
    task_id: i64,
) -> Result<StatementVersion, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        StatementVersion,
        r#"
        SELECT
            id,
            task_id,
            content_manifest as "content_manifest: Json<HashMap<String, String>>",
            is_live,
            created_at
        FROM statement_versions
        WHERE task_id = ?
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        task_id
    )
    .fetch_optional(executor)
    .await?
    .ok_or(Error::NotFound)
}

pub async fn get_all_statement_versions_by_task_id<'e, E>(
    executor: E,
    task_id: i64,
) -> Result<Vec<StatementVersion>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    Ok(sqlx::query_as!(
        StatementVersion,
        r#"
        SELECT
            id,
            task_id,
            content_manifest as "content_manifest: Json<HashMap<String, String>>",
            is_live,
            created_at
        FROM statement_versions
        WHERE task_id = ?
        ORDER BY created_at DESC
        "#,
        task_id
    )
    .fetch_all(executor)
    .await?)
}

pub async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<StatementVersion, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        StatementVersion,
        r#"
        SELECT
            id,
            task_id,
            content_manifest as "content_manifest: Json<HashMap<String, String>>",
            is_live,
            created_at
        FROM statement_versions
        WHERE id = ?
        LIMIT 1
        "#,
        id
    )
    .fetch_optional(executor)
    .await?
    .ok_or(Error::NotFound)
}

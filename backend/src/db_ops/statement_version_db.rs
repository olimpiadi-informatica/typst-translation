use std::collections::HashMap;

use common::error::Error;
use common::statement_version::StatementVersion;
use sqlx::types::Json;
use sqlx::{Executor, Sqlite};

pub async fn get_latest_statement_version_by_task_id<'e, E>(
    executor: E,
    task_id: i64,
) -> Result<Option<StatementVersion>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let statement_version = sqlx::query_as!(
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
    .await?;

    Ok(statement_version)
}

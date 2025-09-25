use common::error::Error;
use common::task::TaskDb;
use sqlx::{Executor, Sqlite};

pub async fn get_task_by_id<'e, E>(executor: E, id: i64) -> Result<TaskDb, Error>
where
    E: Executor<'e, Database = Sqlite> + Clone,
{
    sqlx::query_as!(TaskDb, "SELECT * FROM tasks WHERE id = ?", id)
        .fetch_optional(executor.clone())
        .await?
        .ok_or(Error::NotFound)
}
